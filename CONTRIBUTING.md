# Contributing to HORUS

Thanks for your interest. HORUS is a protocol-first project: most non-trivial
changes start as an RFC in [`spec/`](spec/), not as code.

## Workflow

1. **Discuss first.** Open an issue describing the problem before writing code.
   For protocol-level changes, open an RFC PR against `spec/` (copy the format
   of an existing RFC).
2. **Branch from `main`.** Use `feature/<short-name>` or `fix/<short-name>`.
3. **Keep PRs scoped.** One logical change per PR. A PR that touches the
   settlement math and reformats the registry is two PRs.
4. **Reference the RFC** your change implements in the PR body.

## Local checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --features zk-experimental   # slow; circuits
```

CI runs the first three on every PR. The ZK feature is excluded from required
checks because the prover is slow and the circuits are experimental.

## Commit style

- Imperative subject under ~72 chars: `access: reject capability past not_after`.
- Reference issues/RFCs in the body where relevant.
- Sign-off is not required, but commits must be your own work.

## Code conventions

- `thiserror` for typed errors at crate boundaries; no `unwrap()` outside tests.
- Public items get doc comments. Module-level `//!` docs explain the *why*.
- New on-chain accounting paths need a `#[cfg(test)]` vector mirroring the RFC.

## Areas that need help

- TS/Py SDK parity with the Rust client (`sdk/`).
- Property tests for the settlement clamp/fee-floor logic.
- Replacing the Groth16 trusted-setup placeholder (see RFC-0011 §Future).
