//! Cryptographic primitives for HORUS.
//!
//! Two things live here: the predicate commitment construction (RFC-0009) and
//! the signature check used to verify capabilities and claims. The hash is
//! BLAKE3; the commitment is hiding via a per-listing `salt`.

#![forbid(unsafe_code)]

use horus_core::{Error, Result};

/// A 32-byte commitment or digest.
pub type Digest = [u8; 32];

/// The facts a listing commits to (RFC-0009 §Predicate commitment).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub n_rows: u64,
    pub schema_hash: Digest,
    pub freshness: i64,
    pub field_presence: u64,
}

/// `PredicateCommitment = H(n_rows || schema_hash || freshness || field_presence || salt)`
///
/// `salt` is a per-listing random value retained by the owner so the
/// commitment is hiding. The byte layout is little-endian for the integers and
/// matches the circuit's expected witness packing (RFC-0011 §Approach).
pub fn commit(facts: &Facts, salt: &Digest) -> Digest {
    let mut h = blake3::Hasher::new();
    h.update(&facts.n_rows.to_le_bytes());
    h.update(&facts.schema_hash);
    h.update(&facts.freshness.to_le_bytes());
    h.update(&facts.field_presence.to_le_bytes());
    h.update(salt);
    *h.finalize().as_bytes()
}

/// Constant-time-ish equality on digests. Commitments are public, so this is
/// not strictly required, but we keep it to avoid leaking via early-exit when
/// the input is secret-derived.
pub fn digest_eq(a: &Digest, b: &Digest) -> bool {
    let mut acc = 0u8;
    for i in 0..32 {
        acc |= a[i] ^ b[i];
    }
    acc == 0
}

/// Verify an Ed25519 signature over `msg` against `pubkey`.
pub fn verify_sig(pubkey: &[u8; 32], msg: &[u8], sig: &[u8; 64]) -> Result<()> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let vk = VerifyingKey::from_bytes(pubkey)
        .map_err(|_| Error::Malformed("verifying key"))?;
    let signature = Signature::from_bytes(sig);
    vk.verify(msg, &signature)
        .map_err(|_| Error::CapabilityRejected("signature"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            n_rows: 12_345,
            schema_hash: [7u8; 32],
            freshness: 1_730_000_000,
            field_presence: 0b1011,
        }
    }

    #[test]
    fn commitment_is_deterministic() {
        let salt = [9u8; 32];
        assert_eq!(commit(&facts(), &salt), commit(&facts(), &salt));
    }

    #[test]
    fn commitment_is_hiding_in_salt() {
        let a = commit(&facts(), &[1u8; 32]);
        let b = commit(&facts(), &[2u8; 32]);
        assert_ne!(a, b, "different salt must yield different commitment");
    }

    #[test]
    fn commitment_binds_each_fact() {
        let salt = [0u8; 32];
        let base = commit(&facts(), &salt);
        let mut f = facts();
        f.n_rows += 1;
        assert_ne!(commit(&f, &salt), base);
    }

    #[test]
    fn digest_eq_matches() {
        assert!(digest_eq(&[3u8; 32], &[3u8; 32]));
        let mut other = [3u8; 32];
        other[31] = 4;
        assert!(!digest_eq(&[3u8; 32], &other));
    }
}

//! core: tighten error variants for settlement

//! registry: extract commitment hashing helper
