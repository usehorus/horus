//! Escrow and per-query settlement (RFC-0007).
//!
//! Funds sit in an on-chain escrow and are released to the owner per *proven*
//! answered query, minus a protocol fee. The price is read from the listing at
//! claim time, never trusted from the claim payload.

#![forbid(unsafe_code)]

use horus_core::{Error, Lamports, Pubkey, Result, Scope};

/// Per-query price schedule (RFC-0007 §Pricing).
#[derive(Clone, Debug)]
pub struct PriceSchedule {
    pub p_count: Lamports,
    pub p_agg: Lamports,
    pub p_row_base: Lamports,
    pub p_row_unit: Lamports,
}

impl PriceSchedule {
    /// `price(Row{n}) = p_row_base + n * p_row_unit`.
    pub fn price(&self, scope: &Scope) -> Lamports {
        match scope {
            Scope::Count => self.p_count,
            Scope::Aggregate => self.p_agg,
            Scope::Row { max_rows } => self
                .p_row_base
                .saturating_add((*max_rows as u64).saturating_mul(self.p_row_unit)),
        }
    }
}

/// On-chain escrow account (RFC-0007 §Accounts).
#[derive(Clone, Debug)]
pub struct Escrow {
    pub buyer: Pubkey,
    pub listing: Pubkey,
    pub owner: Pubkey,
    pub budget: Lamports,
    pub spent: Lamports,
    pub fee_bps: u16,
    pub not_after: i64,
    pub closed: bool,
}

/// Where lamports go when a claim settles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settlement {
    pub to_owner: Lamports,
    pub to_treasury: Lamports,
    pub exhausted: bool,
}

/// Protocol fee on `owed`, with a 1-lamport floor on non-zero amounts to close
/// the sub-unit griefing vector (RFC-0007 §Rounding). Mirrors the minimum-slash
/// guard pattern.
fn fee(owed: Lamports, fee_bps: u16) -> Lamports {
    if owed == 0 {
        return 0;
    }
    let raw = owed.saturating_mul(fee_bps as u64) / 10_000;
    raw.max(1)
}

impl Escrow {
    /// Process a claim for `newly_answered` queries, each priced by `prices`.
    ///
    /// 1. monotonicity (`new_count > recorded`) is enforced by the caller via
    ///    `recorded_count`/`new_count`;
    /// 2. `owed = Σ price(scope_i)`;
    /// 3. if `spent + owed > budget`, clamp to remaining and mark exhausted;
    /// 4. split off the fee;
    /// 5. update `spent`.
    pub fn claim(
        &mut self,
        recorded_count: u32,
        new_count: u32,
        scopes: &[Scope],
        prices: &PriceSchedule,
        now: i64,
    ) -> Result<Settlement> {
        if self.closed {
            return Err(Error::SettlementRejected("escrow closed"));
        }
        // Monotonicity: prevents replay / double-claim.
        if new_count <= recorded_count {
            return Err(Error::SettlementRejected("non-monotonic answered count"));
        }
        let delta = (new_count - recorded_count) as usize;
        if scopes.len() != delta {
            return Err(Error::SettlementRejected("scope count mismatch"));
        }
        let _ = now; // claims are valid any time before close; not_after gates refunds.

        let owed: Lamports = scopes
            .iter()
            .fold(0u64, |acc, s| acc.saturating_add(prices.price(s)));

        let remaining = self.budget.saturating_sub(self.spent);
        let (settled, exhausted) = if owed > remaining {
            (remaining, true)
        } else {
            (owed, false)
        };

        let f = fee(settled, self.fee_bps);
        let to_owner = settled - f;
        self.spent = self.spent.saturating_add(settled);

        Ok(Settlement {
            to_owner,
            to_treasury: f,
            exhausted,
        })
    }

    /// Buyer reclaims `budget - spent` after `not_after`. Idempotent.
    pub fn refund(&mut self, now: i64) -> Result<Lamports> {
        if now < self.not_after {
            return Err(Error::SettlementRejected("refund before not_after"));
        }
        if self.closed {
            return Ok(0);
        }
        let refundable = self.budget.saturating_sub(self.spent);
        self.closed = true;
        Ok(refundable)
    }
}

#[cfg(test)]
mod tests {
    //! Vectors mirror RFC-0007 §Failure modes & tests.
    use super::*;

    fn prices() -> PriceSchedule {
        PriceSchedule {
            p_count: 1_000,
            p_agg: 5_000,
            p_row_base: 2_000,
            p_row_unit: 10,
        }
    }

    fn escrow(budget: Lamports) -> Escrow {
        Escrow {
            buyer: Pubkey([1; 32]),
            listing: Pubkey([2; 32]),
            owner: Pubkey([3; 32]),
            budget,
            spent: 0,
            fee_bps: 250, // 2.5%
            not_after: 10_000,
            closed: false,
        }
    }

    #[test]
    fn double_claim_same_answered_is_rejected() {
        let mut e = escrow(1_000_000);
        let err = e.claim(5, 5, &[], &prices(), 0).unwrap_err();
        assert_eq!(err, Error::SettlementRejected("non-monotonic answered count"));
    }

    #[test]
    fn happy_claim_splits_fee() {
        let mut e = escrow(1_000_000);
        // two count queries: owed = 2_000, fee = 2.5% = 50.
        let s = e.claim(0, 2, &[Scope::Count, Scope::Count], &prices(), 100).unwrap();
        assert_eq!(s.to_treasury, 50);
        assert_eq!(s.to_owner, 1_950);
        assert!(!s.exhausted);
        assert_eq!(e.spent, 2_000);
    }

    #[test]
    fn claim_exceeding_budget_is_clamped() {
        let mut e = escrow(1_500);
        // one aggregate query owes 5_000 but only 1_500 remains.
        let s = e.claim(0, 1, &[Scope::Aggregate], &prices(), 100).unwrap();
        assert!(s.exhausted);
        assert_eq!(s.to_owner + s.to_treasury, 1_500);
        assert_eq!(e.spent, 1_500);
    }

    #[test]
    fn fee_floors_at_one_on_subunit_prices() {
        // owed * 250 / 10000 rounds to 0 for owed < 40; floor must lift it to 1.
        assert_eq!(fee(39, 250), 1);
        assert_eq!(fee(40, 250), 1);
        assert_eq!(fee(0, 250), 0);
    }

    #[test]
    fn refund_before_not_after_is_rejected() {
        let mut e = escrow(1_000);
        assert_eq!(
            e.refund(9_999).unwrap_err(),
            Error::SettlementRejected("refund before not_after")
        );
    }

    #[test]
    fn refund_after_not_after_returns_remainder() {
        let mut e = escrow(10_000);
        e.claim(0, 1, &[Scope::Count], &prices(), 100).unwrap(); // spends 1_000
        let back = e.refund(10_000).unwrap();
        assert_eq!(back, 9_000);
        assert!(e.closed);
        // idempotent
        assert_eq!(e.refund(10_001).unwrap(), 0);
    }

    #[test]
    fn claim_on_closed_escrow_is_rejected() {
        let mut e = escrow(10_000);
        e.refund(10_000).unwrap();
        assert_eq!(
            e.claim(0, 1, &[Scope::Count], &prices(), 100).unwrap_err(),
            Error::SettlementRejected("escrow closed")
        );
    }

    #[test]
    fn row_price_scales_with_rows() {
        let p = prices();
        assert_eq!(p.price(&Scope::Row { max_rows: 100 }), 2_000 + 100 * 10);
    }
}

//! crypto: note constant-time requirement on compare

//! access: clarify capability scope doc
