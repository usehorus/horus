//! Dataset registry and predicate-commitment listings (RFC-0009).
//!
//! A listing binds an encrypted blob pointer to a predicate commitment and a
//! price schedule. An `update` MUST carry a fresh commitment; gateways serving
//! a stale commitment fail verification at claim time.

#![forbid(unsafe_code)]

use horus_core::{Error, Pubkey, Result};
use horus_crypto::Digest;

/// Lifecycle state of a listing (RFC-0009 §Listing lifecycle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Active,
    Delisted,
}

/// On-chain listing record.
#[derive(Clone, Debug)]
pub struct Listing {
    pub id: Pubkey,
    pub owner: Pubkey,
    pub blob_ptr: String,
    pub schema_hash: Digest,
    pub commitment: Digest,
    pub created_at: i64,
    pub state: State,
    pub revoked: Vec<[u8; 16]>,
}

impl Listing {
    pub fn new(
        id: Pubkey,
        owner: Pubkey,
        blob_ptr: impl Into<String>,
        schema_hash: Digest,
        commitment: Digest,
        created_at: i64,
    ) -> Self {
        Self {
            id,
            owner,
            blob_ptr: blob_ptr.into(),
            schema_hash,
            commitment,
            created_at,
            state: State::Active,
            revoked: Vec::new(),
        }
    }

    /// Replace the commitment (and optionally the blob pointer). Only the owner
    /// may update, and the new commitment must differ from the current one so a
    /// no-op update can't be used to mask a stale dataset.
    pub fn update(
        &mut self,
        caller: &Pubkey,
        new_commitment: Digest,
        new_blob_ptr: Option<String>,
    ) -> Result<()> {
        if *caller != self.owner {
            return Err(Error::Registry("only owner may update"));
        }
        if self.state != State::Active {
            return Err(Error::Registry("listing not active"));
        }
        if new_commitment == self.commitment {
            return Err(Error::Registry("update must carry a fresh commitment"));
        }
        self.commitment = new_commitment;
        if let Some(ptr) = new_blob_ptr {
            self.blob_ptr = ptr;
        }
        Ok(())
    }

    pub fn delist(&mut self, caller: &Pubkey) -> Result<()> {
        if *caller != self.owner {
            return Err(Error::Registry("only owner may delist"));
        }
        self.state = State::Delisted;
        Ok(())
    }

    /// A gateway must reject a claim whose presented commitment does not match
    /// the registry's current value.
    pub fn commitment_matches(&self, presented: &Digest) -> bool {
        horus_crypto::digest_eq(&self.commitment, presented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: Pubkey = Pubkey([0xAA; 32]);
    const STRANGER: Pubkey = Pubkey([0xBB; 32]);

    fn listing() -> Listing {
        Listing::new(Pubkey([1; 32]), OWNER, "ipfs://blob", [7; 32], [1; 32], 1000)
    }

    #[test]
    fn owner_can_update_with_fresh_commitment() {
        let mut l = listing();
        assert!(l.update(&OWNER, [2; 32], None).is_ok());
        assert_eq!(l.commitment, [2; 32]);
    }

    #[test]
    fn stranger_cannot_update() {
        let mut l = listing();
        assert_eq!(
            l.update(&STRANGER, [2; 32], None).unwrap_err(),
            Error::Registry("only owner may update")
        );
    }

    #[test]
    fn no_op_update_is_rejected() {
        let mut l = listing();
        assert_eq!(
            l.update(&OWNER, [1; 32], None).unwrap_err(),
            Error::Registry("update must carry a fresh commitment")
        );
    }

    #[test]
    fn stale_commitment_fails_match() {
        let l = listing();
        assert!(l.commitment_matches(&[1; 32]));
        assert!(!l.commitment_matches(&[9; 32]));
    }

    #[test]
    fn delisted_cannot_be_updated() {
        let mut l = listing();
        l.delist(&OWNER).unwrap();
        assert!(l.update(&OWNER, [2; 32], None).is_err());
    }
}
