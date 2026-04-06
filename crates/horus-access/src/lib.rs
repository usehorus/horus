//! Capability-based access grants (RFC-0004).
//!
//! A capability is a bearer token the buyer presents to the gateway. The
//! gateway verifies the issuer signature and a handful of local invariants
//! offline — no chain round-trip per query. Usage is tracked off-chain and
//! proven on-chain at claim time (RFC-0007).

#![forbid(unsafe_code)]

use horus_core::{Error, Pubkey, Result, Scope};

pub const CAPABILITY_VERSION: u8 = 1;

/// A signed access grant. The signature covers every field except `issuer_sig`
/// itself, serialized in declaration order (see [`Capability::signing_bytes`]).
#[derive(Clone, Debug)]
pub struct Capability {
    pub version: u8,
    pub listing: Pubkey,
    pub holder: Pubkey,
    pub scope: Scope,
    pub max_queries: u32,
    pub not_before: i64,
    pub not_after: i64,
    pub nonce: [u8; 16],
    pub issuer_sig: [u8; 64],
}

/// The revocation set a gateway syncs lazily (RFC-0004 §Revocation).
///
/// Entries are `(listing, nonce)` pairs; because all capabilities expire, the
/// set is pruned of entries past their `not_after`.
#[derive(Default, Debug)]
pub struct RevocationSet {
    revoked: Vec<(Pubkey, [u8; 16])>,
}

impl RevocationSet {
    pub fn revoke(&mut self, listing: Pubkey, nonce: [u8; 16]) {
        if !self.contains(&listing, &nonce) {
            self.revoked.push((listing, nonce));
        }
    }

    pub fn contains(&self, listing: &Pubkey, nonce: &[u8; 16]) -> bool {
        self.revoked
            .iter()
            .any(|(l, n)| l == listing && n == nonce)
    }
}

impl Capability {
    /// Canonical bytes the issuer signs over: every field except the signature,
    /// in declaration order, integers little-endian.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(128);
        out.push(self.version);
        out.extend_from_slice(&self.listing.0);
        out.extend_from_slice(&self.holder.0);
        match self.scope {
            Scope::Count => out.push(0),
            Scope::Aggregate => out.push(1),
            Scope::Row { max_rows } => {
                out.push(2);
                out.extend_from_slice(&max_rows.to_le_bytes());
            }
        }
        out.extend_from_slice(&self.max_queries.to_le_bytes());
        out.extend_from_slice(&self.not_before.to_le_bytes());
        out.extend_from_slice(&self.not_after.to_le_bytes());
        out.extend_from_slice(&self.nonce);
        out
    }

    /// Verify the capability against the gateway's local view.
    ///
    /// Rules mirror RFC-0004 §Verification rules, evaluated in order. The first
    /// failing rule short-circuits. `answered` is the gateway's recorded usage
    /// counter for this capability's nonce.
    pub fn verify(
        &self,
        ctx: &VerifyContext<'_>,
        answered: u32,
    ) -> Result<()> {
        // Rule 1: version.
        if self.version != CAPABILITY_VERSION {
            return Err(Error::CapabilityRejected("version"));
        }
        // Rule 2: issuer signature over the canonical bytes.
        horus_crypto::verify_sig(&ctx.issuer_key, &self.signing_bytes(), &self.issuer_sig)?;
        // Rule 3: time window. not_before inclusive, not_after exclusive.
        if ctx.now < self.not_before || ctx.now >= self.not_after {
            return Err(Error::CapabilityRejected("outside validity window"));
        }
        // Rule 4: holder binds the query signer.
        if self.holder != ctx.query_signer {
            return Err(Error::CapabilityRejected("holder mismatch"));
        }
        // Rule 5: quota. Checked BEFORE answering; equality means exhausted.
        if answered >= self.max_queries {
            return Err(Error::CapabilityRejected("quota exhausted"));
        }
        // Rule 6: revocation.
        if ctx.revoked.contains(&self.listing, &self.nonce) {
            return Err(Error::CapabilityRejected("revoked"));
        }
        Ok(())
    }

    /// Whether this capability permits a query of the given class.
    pub fn permits_scope(&self, requested: &Scope) -> bool {
        self.scope.permits(requested)
    }
}

/// Everything the gateway needs that is *not* on the capability itself.
pub struct VerifyContext<'a> {
    pub issuer_key: [u8; 32],
    pub query_signer: Pubkey,
    pub now: i64,
    pub revoked: &'a RevocationSet,
}

#[cfg(test)]
mod tests {
    //! Test vectors mirror RFC-0004 §Test vectors. Signatures are produced with
    //! a deterministic test keypair so the happy path actually verifies rather
    //! than being stubbed.
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const HOLDER: Pubkey = Pubkey([0x11; 32]);
    const LISTING: Pubkey = Pubkey([0x22; 32]);

    fn issuer() -> SigningKey {
        SigningKey::from_bytes(&[0x42; 32])
    }

    fn signed_cap(scope: Scope, not_before: i64, not_after: i64, max_queries: u32) -> Capability {
        let sk = issuer();
        let mut cap = Capability {
            version: CAPABILITY_VERSION,
            listing: LISTING,
            holder: HOLDER,
            scope,
            max_queries,
            not_before,
            not_after,
            nonce: [0xAB; 16],
            issuer_sig: [0u8; 64],
        };
        let sig = sk.sign(&cap.signing_bytes());
        cap.issuer_sig = sig.to_bytes();
        cap
    }

    fn ctx<'a>(now: i64, revoked: &'a RevocationSet) -> VerifyContext<'a> {
        VerifyContext {
            issuer_key: issuer().verifying_key().to_bytes(),
            query_signer: HOLDER,
            now,
            revoked,
        }
    }

    #[test]
    fn issue_then_verify_happy_path() {
        let cap = signed_cap(Scope::Count, 1000, 2000, 5);
        let rev = RevocationSet::default();
        assert!(cap.verify(&ctx(1500, &rev), 0).is_ok());
    }

    #[test]
    fn expired_is_rejected() {
        let cap = signed_cap(Scope::Count, 1000, 2000, 5);
        let rev = RevocationSet::default();
        // now >= not_after
        assert_eq!(
            cap.verify(&ctx(2000, &rev), 0),
            Err(Error::CapabilityRejected("outside validity window"))
        );
    }

    #[test]
    fn not_yet_valid_is_rejected() {
        let cap = signed_cap(Scope::Count, 1000, 2000, 5);
        let rev = RevocationSet::default();
        assert!(cap.verify(&ctx(999, &rev), 0).is_err());
    }

    #[test]
    fn quota_exhaustion_is_rejected() {
        let cap = signed_cap(Scope::Count, 1000, 2000, 5);
        let rev = RevocationSet::default();
        // answered == max_queries → exhausted (the classic off-by-one boundary).
        assert_eq!(
            cap.verify(&ctx(1500, &rev), 5),
            Err(Error::CapabilityRejected("quota exhausted"))
        );
        // one below the cap still passes.
        assert!(cap.verify(&ctx(1500, &rev), 4).is_ok());
    }

    #[test]
    fn scope_escalation_is_rejected() {
        let cap = signed_cap(Scope::Count, 1000, 2000, 5);
        assert!(!cap.permits_scope(&Scope::Row { max_rows: 10 }));
        assert!(cap.permits_scope(&Scope::Count));
    }

    #[test]
    fn revoked_nonce_is_rejected() {
        let cap = signed_cap(Scope::Count, 1000, 2000, 5);
        let mut rev = RevocationSet::default();
        rev.revoke(LISTING, [0xAB; 16]);
        assert_eq!(
            cap.verify(&ctx(1500, &rev), 0),
            Err(Error::CapabilityRejected("revoked"))
        );
    }

    #[test]
    fn forged_signature_is_rejected() {
        let mut cap = signed_cap(Scope::Count, 1000, 2000, 5);
        cap.max_queries = 9999; // tamper after signing
        let rev = RevocationSet::default();
        assert_eq!(
            cap.verify(&ctx(1500, &rev), 0),
            Err(Error::CapabilityRejected("signature"))
        );
    }

    #[test]
    fn wrong_holder_is_rejected() {
        let cap = signed_cap(Scope::Count, 1000, 2000, 5);
        let rev = RevocationSet::default();
        let mut c = ctx(1500, &rev);
        c.query_signer = Pubkey([0x99; 32]);
        assert_eq!(
            cap.verify(&c, 0),
            Err(Error::CapabilityRejected("holder mismatch"))
        );
    }
}

//! gateway: document usage-accrual invariant
