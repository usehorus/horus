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

## [0.4.0] - 2026-05-22

### Added
- `horus-registry`: predicate-commitment listing records (RFC-0009).
- Gateway `--features zk-experimental` flag gate returning typed
  `FeatureDisabled` instead of degrading silently.

### Changed
- Capability verification consolidated into six explicit rules (RFC-0004).

### Fixed
- Off-by-one in usage accounting: a failed query no longer consumes quota.

## [0.3.0] - 2026-03-18

### Added
- `horus-settlement`: per-query escrow with monotonic claim counter (RFC-0007).
- Refund path unlocking after `not_after`.

## [0.2.0] - 2026-01-29

### Added
- `horus-access`: bearer capability issuance + offline verification (RFC-0004).
- Scope classes: `Count`, `Aggregate`, `Row { max_rows }`.

## [0.1.0] - 2025-12-04

### Added
- Initial workspace scaffold, `horus-core` types, RFC-0001 overview.
