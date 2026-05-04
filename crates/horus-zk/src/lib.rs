//! EXPERIMENTAL zero-knowledge predicate proofs (RFC-0011).
//!
//! **Do not use this for anything of value.** The circuits are not audited, the
//! prover is slow, and soundness has not been established. The entire surface
//! is gated behind the `zk-experimental` feature; without it, every entry
//! point returns [`horus_core::Error::FeatureDisabled`] rather than silently
//! degrading.

#![forbid(unsafe_code)]

use horus_core::{Error, Result};
use horus_crypto::{Digest, Facts};

/// A predicate the prover claims the committed dataset satisfies.
#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    /// `n_rows > k`. The only predicate the WIP circuit supports.
    RowsGreaterThan(u64),
    /// `freshness > t`.
    FresherThan(i64),
}

/// An opaque proof blob. Layout is unstable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof(pub Vec<u8>);

const FLAG: &str = "zk-experimental";

#[cfg(not(feature = "zk-experimental"))]
mod backend {
    use super::*;

    pub fn prove(_facts: &Facts, _salt: &Digest, _p: &Predicate) -> Result<Proof> {
        Err(Error::FeatureDisabled(super::FLAG))
    }

    pub fn verify(_commitment: &Digest, _p: &Predicate, _proof: &Proof) -> Result<bool> {
        Err(Error::FeatureDisabled(super::FLAG))
    }
}

#[cfg(feature = "zk-experimental")]
mod backend {
    //! WIP Groth16-over-BN254 backend. The "proof" here is a placeholder hash
    //! transcript, NOT a sound argument — it exists so the gateway plumbing and
    //! the SDK surface can be exercised end to end while the real circuit is
    //! built out. See RFC-0011 §Status table.
    use super::*;

    fn holds(facts: &Facts, p: &Predicate) -> bool {
        match *p {
            Predicate::RowsGreaterThan(k) => facts.n_rows > k,
            Predicate::FresherThan(t) => facts.freshness > t,
        }
    }

    pub fn prove(facts: &Facts, salt: &Digest, p: &Predicate) -> Result<Proof> {
        if !holds(facts, p) {
            return Err(Error::Malformed("predicate does not hold for witness"));
        }
        // Placeholder transcript binding the commitment to the predicate.
        let commitment = horus_crypto::commit(facts, salt);
        let mut blob = Vec::with_capacity(40);
        blob.extend_from_slice(&commitment);
        match *p {
            Predicate::RowsGreaterThan(k) => {
                blob.push(0);
                blob.extend_from_slice(&k.to_le_bytes());
            }
            Predicate::FresherThan(t) => {
                blob.push(1);
                blob.extend_from_slice(&t.to_le_bytes());
            }
        }
        Ok(Proof(blob))
    }

    pub fn verify(commitment: &Digest, p: &Predicate, proof: &Proof) -> Result<bool> {
        if proof.0.len() < 33 || &proof.0[..32] != commitment {
            return Ok(false);
        }
        let tag = proof.0[32];
        let expected = matches!(
            (tag, p),
            (0, Predicate::RowsGreaterThan(_)) | (1, Predicate::FresherThan(_))
        );
        Ok(expected)
    }
}

/// Prove the committed dataset satisfies `predicate` without revealing facts.
pub fn prove(facts: &Facts, salt: &Digest, predicate: &Predicate) -> Result<Proof> {
    backend::prove(facts, salt, predicate)
}

/// Verify a predicate proof against a public commitment.
pub fn verify(commitment: &Digest, predicate: &Predicate, proof: &Proof) -> Result<bool> {
    backend::verify(commitment, predicate, proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            n_rows: 50_000,
            schema_hash: [1; 32],
            freshness: 1_730_000_000,
            field_presence: 0,
        }
    }

    #[cfg(not(feature = "zk-experimental"))]
    #[test]
    fn disabled_without_flag() {
        let err = prove(&facts(), &[0; 32], &Predicate::RowsGreaterThan(10_000)).unwrap_err();
        assert_eq!(err, Error::FeatureDisabled("zk-experimental"));
    }

    #[cfg(feature = "zk-experimental")]
    #[test]
    fn roundtrip_when_predicate_holds() {
        let salt = [5; 32];
        let p = Predicate::RowsGreaterThan(10_000);
        let proof = prove(&facts(), &salt, &p).unwrap();
        let commitment = horus_crypto::commit(&facts(), &salt);
        assert!(verify(&commitment, &p, &proof).unwrap());
    }

    #[cfg(feature = "zk-experimental")]
    #[test]
    fn refuses_to_prove_false_predicate() {
        let p = Predicate::RowsGreaterThan(1_000_000);
        assert!(prove(&facts(), &[5; 32], &p).is_err());
    }
}

//! gateway: document usage-accrual invariant
