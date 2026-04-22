"""HORUS Python SDK (beta).

Mirrors the reference Rust SDK. The commitment must match
``horus-crypto::commit`` byte-for-byte (BLAKE3, little-endian integers) or the
on-chain verifier will reject it. See RFC-0009.
"""
from __future__ import annotations

from dataclasses import dataclass

import blake3

__version__ = "0.4.0b1"

__all__ = ["Facts", "commit", "estimate_budget", "__version__"]


@dataclass(frozen=True)
class Facts:
    n_rows: int
    schema_hash: bytes  # 32 bytes
    freshness: int  # unix seconds
    field_presence: int


def _le_u64(v: int) -> bytes:
    return (v & ((1 << 64) - 1)).to_bytes(8, "little")


def _le_i64(v: int) -> bytes:
    return (v & ((1 << 64) - 1)).to_bytes(8, "little", signed=False)


def commit(facts: Facts, salt: bytes) -> bytes:
    """PredicateCommitment = H(n_rows || schema_hash || freshness || field_presence || salt)."""
    if len(facts.schema_hash) != 32:
        raise ValueError("schema_hash must be 32 bytes")
    if len(salt) != 32:
        raise ValueError("salt must be 32 bytes")
    h = blake3.blake3()
    h.update(_le_u64(facts.n_rows))
    h.update(facts.schema_hash)
    h.update(_le_i64(facts.freshness))
    h.update(_le_u64(facts.field_presence))
    h.update(salt)
    return h.digest()


def estimate_budget(per_query: int, n: int) -> int:
    """Estimate escrow budget for ``n`` queries at ``per_query`` lamports."""
    return per_query * n
