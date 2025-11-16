# RFC-0001 — Protocol overview & terminology

- **Status:** Accepted
- **Authors:** k.tanaka, e.morales
- **Created:** 2025-11-07

## Summary

HORUS lets a **data owner** sell **time-boxed, query-scoped access** to an
encrypted dataset, with payment settled atomically on Solana. The raw dataset is
never transferred. This RFC fixes the terminology and the end-to-end flow that the
remaining RFCs refine.

## Terminology

- **Dataset** — an opaque, owner-encrypted blob plus a schema descriptor.
- **Listing** — an on-chain record pointing at a dataset, carrying a *predicate
  commitment* (RFC-0009) and a price schedule.
- **Predicate commitment** — a binding commitment to facts about the dataset
  (row count, schema hash, freshness) that can be proven later in ZK (RFC-0011).
- **Capability** — a signed grant authorizing a buyer to issue up to `N` queries
  within a time window `[not_before, not_after]` against a listing (RFC-0004).
- **Gateway** — the process (run by the owner or a delegate) that holds the
  decryption key, answers queries, and posts settlement (RFC-0007).
- **Settlement** — the on-chain escrow that holds buyer funds and releases them to
  the owner per answered query, minus the protocol fee.

## End-to-end flow

```
1.  owner  : encrypt(dataset) -> blob; commit(facts) -> C
2.  owner  : registry.list(blob_ptr, C, price_schedule)          [on-chain]
3.  buyer  : settlement.open_escrow(listing, budget)             [on-chain]
4.  buyer  : access.request_capability(listing, queries, ttl)
5.  owner  : access.issue_capability(...)  -> cap (signed)
6.  buyer  : gateway.query(cap, q)
7.  gateway: check(cap) -> answer(q) + proof π                   [off-chain]
8.  gateway: settlement.claim(escrow, usage, π)                  [on-chain]
9.  escrow : transfer (price - fee) -> owner, fee -> treasury
```

Steps 6–8 repeat until the capability is exhausted or expires. Unused escrow is
refundable to the buyer after `not_after`.

## Non-goals

- HORUS does **not** provide anonymity for the *buyer's queries* against a
  malicious gateway operator that logs them. Query privacy from the operator is
  future work (see RFC-0011 §Future).
- HORUS does **not** prevent a buyer from re-selling results they legitimately
  obtained. It controls *access*, not downstream use.

## Open questions

- Multi-owner / shared-custody datasets (deferred to a later RFC).
- Cross-listing capability bundles.
