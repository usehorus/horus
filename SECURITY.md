# Security Policy

## Status

HORUS is **pre-audit, experimental software**. The on-chain programs have not
been audited and the ZK circuits (`crates/horus-zk`, behind
`ENABLE_ZK_EXPERIMENTAL`) carry no soundness guarantees. Do not use HORUS to
custody funds or data of value.

## Supported versions

We patch security issues only on the latest `0.x` minor. There is no LTS while
the protocol is pre-1.0.

| Version | Supported |
|---------|-----------|
| 0.4.x   | ✅ |
| < 0.4   | ❌ |

## Reporting a vulnerability

Please **do not** open a public issue for security-sensitive bugs.

- Email: security@usehorus.xyz
- Or use GitHub's private vulnerability reporting (Security → Report a vulnerability).

Include a description, affected component (`access`, `settlement`, `registry`,
`crypto`, `zk`, `gateway`), and a reproduction if possible. We aim to
acknowledge within 72 hours.

## Disclosure

We follow coordinated disclosure. Once a fix is available and deployed, we
credit reporters in the release notes unless anonymity is requested.

## Threat model

The protocol's security argument is documented in
[`docs/security-model.md`](docs/security-model.md). In short:

- The gateway is **trusted with the decryption key** but **not** with honest
  accounting — settlement is proven on-chain (RFC-0007).
- Capabilities are bearer tokens; revocation is best-effort and time-bounded
  (RFC-0004 §Revocation).
- Predicate commitments bind listing claims; the ZK opening is **not** yet
  sound and must not be relied on (RFC-0011).
