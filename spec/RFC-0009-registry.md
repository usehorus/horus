# RFC-0009 — Dataset registry & predicate commitments

- **Status:** Draft
- **Authors:** r.singh, k.tanaka
- **Created:** 2026-01-16
- **Implements:** `crates/horus-registry`

> **Draft.** The commitment scheme below is likely to change once RFC-0011 settles
> on a proof system. Treat field layouts as unstable.

## Motivation

A buyer chooses what to buy based on the listing's claims. If those claims are
unverifiable, the market is a market for lemons. HORUS binds each listing to a
**predicate commitment**: a cryptographic commitment to a set of facts about the
dataset that the owner can later prove (RFC-0011) without revealing the data.

## Listing record

```
Listing {
  id:          Pubkey,
  owner:       Pubkey,
  blob_ptr:    Uri,            // where the encrypted blob lives (IPFS/Arweave/HTTPS)
  schema_hash: [u8; 32],       // hash of the column schema descriptor
  commitment:  PredicateCommitment,
  price:       PriceSchedule,
  created_at:  i64,
  revoked:     Vec<[u8; 16]>,  // revoked capability nonces (RFC-0004 §Revocation)
}
```

## Predicate commitment

The committed facts (v0 set):

| Fact | Type | Meaning |
|------|------|---------|
| `n_rows` | u64 | exact row count |
| `schema_hash` | [u8;32] | binds the column set + types |
| `freshness` | i64 | max timestamp present in the data |
| `field_presence` | bitmap | which optional fields are populated |

```
PredicateCommitment = H(n_rows || schema_hash || freshness || field_presence || salt)
```

`salt` is a per-listing random value retained by the owner so the commitment is
hiding. RFC-0011 specifies the circuit that opens this commitment to a *predicate*
(e.g. `n_rows > 10_000`) without revealing `n_rows` itself.

## Listing lifecycle

```
list  ──►  active  ──►  delisted
              │
              └─ updated (new commitment, supersedes previous)
```

An `update` MUST carry a fresh commitment; gateways serving a stale commitment
fail verification at claim time.

## Open questions

- Whether `n_rows` should be range-committed rather than exact (privacy vs.
  usefulness trade-off).
- Schema descriptor canonicalization (column ordering) — see discussion in #41.
