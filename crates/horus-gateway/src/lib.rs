//! Gateway runtime.
//!
//! Ties the other crates together: it holds the decryption key, checks the
//! capability (RFC-0004), answers the query, and accrues usage that is later
//! proven on-chain at claim time (RFC-0007). This crate is intentionally thin —
//! the security-critical logic lives in `horus-access` and `horus-settlement`.

#![forbid(unsafe_code)]

use horus_access::{Capability, RevocationSet, VerifyContext};
use horus_core::{Error, Pubkey, Result, Scope};

/// Per-capability usage the gateway tracks off-chain (RFC-0004 §Usage accounting).
#[derive(Default, Debug)]
pub struct UsageMeter {
    answered: u32,
}

impl UsageMeter {
    pub fn answered(&self) -> u32 {
        self.answered
    }

    /// Record a *successful* answer. A failed answer MUST NOT call this, so
    /// malformed queries never consume quota (RFC-0004 §Usage accounting).
    fn record_success(&mut self) {
        self.answered += 1;
    }
}

/// A query the buyer submits.
pub struct Query<'a> {
    pub signer: Pubkey,
    pub class: Scope,
    pub body: &'a [u8],
}

/// The gateway's answer plus the usage counter to carry into settlement.
pub struct Answered {
    pub bytes: Vec<u8>,
    pub answered_count: u32,
}

/// Process one query end to end.
///
/// Order matters: verify the capability, then check scope, then answer. The
/// meter is bumped only after `answer_fn` succeeds.
pub fn handle_query<F>(
    cap: &Capability,
    ctx: &VerifyContext<'_>,
    meter: &mut UsageMeter,
    query: &Query<'_>,
    answer_fn: F,
) -> Result<Answered>
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>>,
{
    cap.verify(ctx, meter.answered())?;
    if !cap.permits_scope(&query.class) {
        return Err(Error::CapabilityRejected("scope escalation"));
    }
    let bytes = answer_fn(query.body)?;
    meter.record_success();
    Ok(Answered {
        bytes,
        answered_count: meter.answered(),
    })
}

/// Re-export so a gateway binary can build a revocation set without depending on
/// `horus-access` directly.
pub fn empty_revocation_set() -> RevocationSet {
    RevocationSet::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use horus_access::CAPABILITY_VERSION;

    const HOLDER: Pubkey = Pubkey([0x11; 32]);

    fn signed_cap() -> Capability {
        let sk = SigningKey::from_bytes(&[0x42; 32]);
        let mut cap = Capability {
            version: CAPABILITY_VERSION,
            listing: Pubkey([0x22; 32]),
            holder: HOLDER,
            scope: Scope::Aggregate,
            max_queries: 2,
            not_before: 0,
            not_after: 10_000,
            nonce: [0xAB; 16],
            issuer_sig: [0; 64],
        };
        cap.issuer_sig = sk.sign(&cap.signing_bytes()).to_bytes();
        cap
    }

    fn ctx<'a>(rev: &'a RevocationSet) -> VerifyContext<'a> {
        VerifyContext {
            issuer_key: SigningKey::from_bytes(&[0x42; 32]).verifying_key().to_bytes(),
            query_signer: HOLDER,
            now: 500,
            revoked: rev,
        }
    }

    #[test]
    fn successful_query_consumes_one_quota() {
        let cap = signed_cap();
        let rev = empty_revocation_set();
        let mut meter = UsageMeter::default();
        let q = Query { signer: HOLDER, class: Scope::Count, body: b"count *" };
        let out = handle_query(&cap, &ctx(&rev), &mut meter, &q, |_| Ok(b"42".to_vec())).unwrap();
        assert_eq!(out.answered_count, 1);
        assert_eq!(meter.answered(), 1);
    }

    #[test]
    fn failed_answer_does_not_consume_quota() {
        let cap = signed_cap();
        let rev = empty_revocation_set();
        let mut meter = UsageMeter::default();
        let q = Query { signer: HOLDER, class: Scope::Count, body: b"bad" };
        let res = handle_query(&cap, &ctx(&rev), &mut meter, &q, |_| {
            Err(Error::Malformed("query"))
        });
        assert!(res.is_err());
        assert_eq!(meter.answered(), 0, "failed query must not consume quota");
    }

    #[test]
    fn scope_escalation_blocked_before_answering() {
        let cap = signed_cap(); // Aggregate
        let rev = empty_revocation_set();
        let mut meter = UsageMeter::default();
        let q = Query { signer: HOLDER, class: Scope::Row { max_rows: 5 }, body: b"" };
        let res = handle_query(&cap, &ctx(&rev), &mut meter, &q, |_| Ok(vec![]));
        assert_eq!(res.err().unwrap(), Error::CapabilityRejected("scope escalation"));
        assert_eq!(meter.answered(), 0);
    }
}

//! registry: extract commitment hashing helper
