//! Core types shared across the HORUS workspace.
//!
//! Nothing in here touches the network or the chain; it is the vocabulary the
//! other crates speak. See `spec/RFC-0001-overview.md` for terminology.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// A 32-byte Ed25519 public key (Solana-style).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pubkey(pub [u8; 32]);

impl core::fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Short form: first 4 bytes, enough to disambiguate in logs.
        write!(f, "Pubkey({}…)", hex::encode(&self.0[..4]))
    }
}

/// Lamports — the indivisible unit of value used in settlement (RFC-0007).
pub type Lamports = u64;

/// The class of query a capability authorizes (RFC-0004 §Capability).
///
/// A gateway MUST reject a query whose class exceeds the granted scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    /// Count-only queries (cardinality of a filtered set).
    Count,
    /// Aggregate queries (sum/avg/min/max over a column).
    Aggregate,
    /// Row-returning queries, capped at `max_rows`.
    Row { max_rows: u32 },
}

impl Scope {
    /// True if `self` permits a query of class `requested`.
    ///
    /// `Row` is the widest scope and subsumes `Aggregate` and `Count`;
    /// `Aggregate` subsumes `Count`. A narrower grant never permits a wider
    /// query.
    pub fn permits(&self, requested: &Scope) -> bool {
        use Scope::*;
        match (self, requested) {
            (Count, Count) => true,
            (Aggregate, Count | Aggregate) => true,
            (Row { max_rows }, Count | Aggregate) => {
                let _ = max_rows;
                true
            }
            (Row { max_rows: g }, Row { max_rows: r }) => r <= g,
            _ => false,
        }
    }
}

/// Errors that can cross a crate boundary in HORUS.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("capability rejected: {0}")]
    CapabilityRejected(&'static str),
    #[error("settlement rejected: {0}")]
    SettlementRejected(&'static str),
    #[error("registry error: {0}")]
    Registry(&'static str),
    #[error("feature disabled: {0}")]
    FeatureDisabled(&'static str),
    #[error("malformed input: {0}")]
    Malformed(&'static str),
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_lattice_count_is_narrowest() {
        assert!(Scope::Count.permits(&Scope::Count));
        assert!(!Scope::Count.permits(&Scope::Aggregate));
        assert!(!Scope::Count.permits(&Scope::Row { max_rows: 1 }));
    }

    #[test]
    fn scope_row_subsumes_lower_classes() {
        let grant = Scope::Row { max_rows: 100 };
        assert!(grant.permits(&Scope::Count));
        assert!(grant.permits(&Scope::Aggregate));
        assert!(grant.permits(&Scope::Row { max_rows: 100 }));
        assert!(grant.permits(&Scope::Row { max_rows: 50 }));
        // …but not a wider row request.
        assert!(!grant.permits(&Scope::Row { max_rows: 101 }));
    }

    #[test]
    fn pubkey_debug_is_short() {
        let k = Pubkey([0xab; 32]);
        assert_eq!(format!("{k:?}"), "Pubkey(abababab…)");
    }
}
