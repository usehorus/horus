# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project uses
`0.x` semver (minor = breaking until 1.0).

## [Unreleased]

### Added
- `horus-zk`: range-predicate circuit skeleton (`n_rows > k`) behind
  `zk-experimental` (RFC-0011). Not sound; do not rely on it.

### Changed
- Settlement fee now floors at 1 lamport on non-zero `owed` to close a
  sub-unit griefing vector (RFC-0007 §Rounding).

## [0.4.0] - 2026-05-20

### Added
- `horus-registry`: predicate-commitment listing records (RFC-0009).
- Gateway `zk-experimental` feature gate returning typed `FeatureDisabled`
  instead of degrading silently.

### Changed
- Capability verification consolidated into six explicit rules (RFC-0004).

### Fixed
- Off-by-one in usage accounting: a failed query no longer consumes quota.

## [0.3.1] - 2026-05-17

### Changed
- Pin Rust edition and MSRV to `1.78` across every workspace crate; the
  experimental ZK paths now fail closed behind the `zk-experimental` feature
  rather than compiling into a default build.

## [0.3.0] - 2026-05-04

### Added
- `horus-settlement`: per-query escrow with monotonic claim counter (RFC-0007).
- Refund path unlocking after `not_after`.
- Issue and pull-request templates; RFC template for spec changes.

## [0.2.1] - 2026-04-24

### Fixed
- SDK parity: the TypeScript and Python commitment codecs now match
  `horus-crypto::commit` byte-for-byte (little-endian integers, BLAKE3 field
  order per RFC-0009). Proofs built with an SDK now verify against the Rust
  reference.

## [0.2.0] - 2026-04-11

### Added
- `horus-access`: bearer capability issuance + offline verification (RFC-0004).
- Scope classes: `Count`, `Aggregate`, `Row { max_rows }`.
- Gateway: verify capability, answer query, accrue usage.

## [0.1.0] - 2026-03-11

### Added
- Initial workspace scaffold, `horus-core` types, RFC-0001 overview.
- `horus-crypto`: BLAKE3 predicate commitment + ed25519 verification.
