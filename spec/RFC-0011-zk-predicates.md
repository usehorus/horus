# RFC-0011 — ZK predicate & query-correctness proofs

- **Status:** Experimental
- **Authors:** r.singh
- **Created:** 2026-02-27
- **Implements:** `crates/horus-zk` (behind `ENABLE_ZK_EXPERIMENTAL`)

> **Experimental.** This RFC describes work in progress. The circuits are not
> audited, the proving backend is slow and unoptimized, and the soundness of the
> construction has **not** been established. Do not rely on these proofs for
> anything of value. Tracked in #42.

## Goal

Two proof obligations:

1. **Predicate proof** — the owner proves the committed dataset (RFC-0009)
   satisfies a public predicate (e.g. `n_rows > 10_000`, `freshness > T`) without
   revealing the committed values.
2. **Query-correctness proof** — the gateway proves an answer was computed
   correctly against the *same* committed dataset the buyer paid for, so the
   gateway cannot answer from a different (cheaper, fake) dataset.

## Approach (current)

- Commitment opening via a Groth16 circuit over BN254.
- The circuit takes the private witness `(n_rows, freshness, field_presence, salt)`
  and public inputs `(commitment, predicate_params)` and outputs `1` iff the
  predicate holds and `H(witness) == commitment`.
- Query-correctness is **not implemented**; the current design sketch hashes the
  decrypted dataset into a Merkle root and proves the answer is a function of
  authenticated leaves. This is expensive and likely to be redesigned.

## Status table

| Component | Status | Notes |
|-----------|--------|-------|
| Commitment-opening circuit | 🟡 WIP | Compiles; trusted setup is a placeholder. |
| Predicate proof (range) | 🟡 WIP | `n_rows > k` only; no compound predicates. |
| Query-correctness proof | 🔴 Design | Merkle-leaf sketch only, not implemented. |
| Recursive aggregation | 🔴 Planned | Batch many claims into one proof. |
| On-chain verifier | 🔴 Planned | Solana verifier program not started. |

## Why it's behind a flag

The default build excludes `crates/horus-zk` entirely. Enabling it:

```bash
cargo build --features zk-experimental
ENABLE_ZK_EXPERIMENTAL=true horus gateway ...
```

attempting to use a ZK path without the flag returns a typed
`FeatureDisabled("zk-experimental")` error rather than silently degrading.

## Future

- Replace Groth16 + per-listing trusted setup with a transparent system (PLONK/STARK).
- Query privacy from the gateway operator (oblivious query execution) — out of scope
  for v0, noted in RFC-0001 non-goals.
