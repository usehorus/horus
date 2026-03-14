# RFC-0004 — Capability-based access grants

- **Status:** Accepted
- **Authors:** k.tanaka
- **Created:** 2025-11-21
- **Implements:** `crates/horus-access`

## Motivation

Access to a listing must be (a) bounded in *count* and *time*, (b) verifiable
offline by the gateway, and (c) revocable. ACL lookups per query are too chatty
for an on-chain system, so HORUS uses **bearer capabilities**: a signed token the
buyer presents to the gateway. The gateway verifies the signature and a small set
of local invariants without a round-trip.

## Capability structure

```
Capability {
  version:     u8,             // = 1
  listing:     Pubkey,         // listing this grant is scoped to
  holder:      Pubkey,         // buyer key authorized to use it
  scope:       Scope,          // query class allowed (see below)
  max_queries: u32,            // hard cap on answered queries
  not_before:  i64,            // unix seconds
  not_after:   i64,            // unix seconds, MUST be > not_before
  nonce:       [u8; 16],       // anti-replay, unique per issuance
  issuer_sig:  Signature,      // owner/issuer signature over the above
}
```

`Scope` is one of `Count`, `Aggregate`, or `Row { max_rows: u32 }`. A gateway MUST
reject a query whose class exceeds the granted scope.

## Verification rules

A gateway MUST reject a capability if **any** of:

1. `version != 1`.
2. `issuer_sig` does not verify against the listing's `issuer` key.
3. `now < not_before` or `now >= not_after`.
4. `holder` does not match the query signer.
5. the capability's recorded usage `>= max_queries`.
6. the capability's `nonce` appears in the revocation set (see below).

Rules 1–5 are checkable offline. Rule 6 requires the revocation set, which the
gateway syncs lazily (see §Revocation).

## Usage accounting

Usage is tracked **off-chain by the gateway** and **proven on-chain at claim time**
(RFC-0007). The gateway maintains a monotonically increasing `answered` counter per
capability nonce. The counter is included in the settlement claim so the buyer
cannot be over-charged and the owner cannot under-report.

Off-by-one is the classic bug here: the check is `answered < max_queries` evaluated
*before* answering, and `answered` is incremented only after a successful answer is
produced. A failed answer (e.g., malformed query) MUST NOT consume quota.

## Revocation

Capabilities are bearer tokens, so revocation is best-effort and time-bounded by
`not_after`. An issuer MAY publish a revocation entry `(listing, nonce)` to the
registry; gateways sync the set on an interval. Because all capabilities expire,
the revocation set is pruned of entries past `not_after`.

## Test vectors

See `crates/horus-access` unit tests:

- issue → verify happy path
- expired (`now >= not_after`) → reject
- not-yet-valid (`now < not_before`) → reject
- quota exhaustion (`answered == max_queries`) → reject
- scope escalation (`Count` cap, `Row` query) → reject
- revoked nonce → reject
- failed query does not consume quota
