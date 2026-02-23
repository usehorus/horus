# Architecture

This document is the map; the RFCs in [`spec/`](../spec/) are the territory.

## Crate graph

```
horus-core ──────────────┐  shared types (Pubkey, Scope, Error)
   ▲                      │
   ├── horus-crypto       │  BLAKE3 commitments, Ed25519 verify
   │      ▲               │
   ├── horus-access ──────┤  capabilities (RFC-0004)
   ├── horus-settlement ──┤  escrow + claims (RFC-0007)
   ├── horus-registry ────┤  listings + predicate commitments (RFC-0009)
   └── horus-zk ──────────┘  EXPERIMENTAL predicate proofs (RFC-0011, flagged)
          ▲
   horus-gateway          ties the above together at runtime
          ▲
   horus-cli / sdk/*      operator + client surfaces
```

Dependencies point downward only. `horus-core` depends on nothing in the
workspace; the gateway depends on everything except the CLI.

## Trust boundaries

| Boundary | Trusted for | NOT trusted for |
|----------|-------------|-----------------|
| Gateway  | holding the decryption key, answering queries | honest usage accounting (proven on-chain) |
| Buyer    | presenting a valid capability | scope it wasn't granted |
| Owner    | setting the price schedule | reporting usage (escrow reads price at claim) |
| Chain    | escrow custody, monotonic claim counter | dataset contents (never on-chain) |

## Request lifecycle (happy path)

1. Buyer presents `(capability, query)` to the gateway.
2. Gateway runs `horus-access::Capability::verify` — six rules, all offline.
3. Gateway checks the query class against the granted `Scope`.
4. Gateway decrypts, answers, and **only then** bumps the usage meter.
5. Periodically the gateway posts a settlement claim; `horus-settlement`
   recomputes `owed` from the listing price and releases funds, clamping at the
   remaining budget.

The security-critical arithmetic lives in `horus-access` and
`horus-settlement`, both of which carry `#[cfg(test)]` vectors mirroring their
RFCs. The gateway crate is intentionally thin glue.

## Why ZK is behind a flag

`horus-zk` compiles to `FeatureDisabled` stubs unless built with
`--features zk-experimental`. The circuits are unaudited and unsound; gating
them keeps an experimental dependency out of the default build and makes the
"not for production" boundary a compile-time fact rather than a footnote. See
RFC-0011.
