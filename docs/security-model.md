# Security model

> **Pre-audit.** This document describes the *intended* security argument, not a
> verified one. Nothing here has been reviewed by a third party.

## Assets

- The **dataset** (owner's encrypted blob) — confidentiality.
- The **escrow budget** (buyer's lamports) — integrity of release.
- **Listing claims** (predicate commitment) — soundness of what's advertised.

## Adversaries

| Adversary | Capability | Mitigation |
|-----------|-----------|------------|
| Malicious buyer | replays / over-uses a capability | monotonic on-chain claim counter; offline rules 1–6 (RFC-0004) |
| Malicious gateway | over-reports usage to drain escrow | escrow recomputes `owed` from the listing price, ignores claim payload price (RFC-0007 §Pricing) |
| Malicious gateway | answers from a cheaper fake dataset | query-correctness proof binds the answer to the committed dataset (RFC-0011 — **not yet implemented**) |
| Dishonest owner | advertises false dataset facts | predicate commitment + ZK opening (RFC-0011 — **soundness not established**) |
| Network observer | learns buyer's queries | **out of scope** for v0 (RFC-0001 non-goals) |

## What holds today

- **Capability verification** is fully implemented and tested: signature,
  validity window, holder binding, quota (with the documented off-by-one
  boundary), scope, and revocation.
- **Settlement** is implemented and tested: monotonicity, budget clamp, fee
  floor, idempotent refund.

## What does NOT hold yet

- **ZK soundness.** The predicate-proof backend is a placeholder transcript when
  `zk-experimental` is enabled, and `FeatureDisabled` otherwise. It proves
  nothing cryptographically. Do not rely on predicate proofs.
- **Query correctness.** A gateway can still answer from the wrong dataset; the
  binding proof is design-stage only.
- **Trusted setup.** The Groth16 path uses a placeholder setup. A real
  deployment needs a transparent system or a real ceremony (RFC-0011 §Future).

## Key handling

The gateway holds the dataset decryption key in process memory. Key custody,
rotation, and HSM integration are operator concerns and out of scope for this
repo. Capabilities and claims are signed with Ed25519; `horus-crypto::verify_sig`
is the single verification entry point.

## Residual risks (accepted for v0)

These are known and intentionally unaddressed at this stage. They are listed so
they are not mistaken for oversights.

- **Query privacy.** A gateway sees every query in plaintext. Private
  information retrieval is out of scope for v0 (RFC-0001 non-goals).
- **Gateway availability.** A capability grants the *right* to query, not a
  liveness guarantee. A gateway that goes dark strands paid-for queries; the
  buyer's recourse is the post-`not_after` refund path, not forced answering.
- **Metadata leakage.** Listing size, update cadence, and claim volume are
  on-chain and public. Traffic analysis is possible even though dataset
  contents are not revealed.
- **Side channels.** `digest_eq` is constant-time, but the broader gateway
  answer path is not audited for timing leaks against the decrypted dataset.
