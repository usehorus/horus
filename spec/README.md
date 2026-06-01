# HORUS Protocol Specification

This directory holds the **normative** definitions of the HORUS protocol. When the
implementation in [`../crates`](../crates) and a spec document disagree, the spec is
correct and the code is a bug.

## RFC lifecycle

```
Draft  ──►  Accepted  ──►  Implemented  ──►  (Superseded)
  │            │
  └─ Rejected  └─ Experimental (merged behind a feature flag)
```

- **Draft** — under discussion, may change substantially.
- **Accepted** — rough consensus reached; safe to implement.
- **Experimental** — implemented behind a feature flag; soundness/stability not guaranteed.
- **Superseded** — replaced by a later RFC (linked at the top of the doc).

## Index

| RFC | Title | Status |
|-----|-------|--------|
| [RFC-0001](RFC-0001-overview.md) | Protocol overview & terminology | Accepted |
| [RFC-0004](RFC-0004-access.md) | Capability-based access grants | Accepted |
| [RFC-0007](RFC-0007-settlement.md) | Escrow & per-query settlement | Accepted |
| [RFC-0009](RFC-0009-registry.md) | Dataset registry & predicate commitments | Draft |
| [RFC-0011](RFC-0011-zk-predicates.md) | ZK predicate & query-correctness proofs | Experimental |

## Conventions

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are to be interpreted as
described in RFC 2119. All byte strings are little-endian unless noted. All on-chain
amounts are denominated in lamports.
