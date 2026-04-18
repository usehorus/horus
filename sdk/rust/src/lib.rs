//! HORUS Rust client SDK.
//!
//! High-level helpers a buyer or owner uses to talk to the protocol. This is
//! the reference SDK; the TS and Python clients (`sdk/ts`, `sdk/py`) track it
//! and are currently beta.

#![forbid(unsafe_code)]

pub use horus_access::{Capability, RevocationSet};
pub use horus_core::{Lamports, Pubkey, Scope};
pub use horus_crypto::{commit, Facts};

/// Build the predicate commitment an owner publishes with a listing.
///
/// ```
/// use horus_sdk::{Facts, commit};
/// let facts = Facts { n_rows: 12_000, schema_hash: [0; 32], freshness: 1_730_000_000, field_presence: 0 };
/// let salt = [7u8; 32];
/// let c = commit(&facts, &salt);
/// assert_eq!(c, commit(&facts, &salt)); // deterministic given the salt
/// ```
pub fn listing_commitment(facts: &Facts, salt: &[u8; 32]) -> [u8; 32] {
    commit(facts, salt)
}

/// Estimate the cost of `n` queries of a given class before opening an escrow.
pub fn estimate_budget(per_query: Lamports, n: u32) -> Lamports {
    per_query.saturating_mul(n as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_estimate() {
        assert_eq!(estimate_budget(1_000, 50), 50_000);
    }
}
