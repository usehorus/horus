/**
 * HORUS TypeScript SDK (beta).
 *
 * Tracks the reference Rust SDK. The commitment construction here MUST match
 * `horus-crypto::commit` byte-for-byte (BLAKE3, little-endian integers) or
 * proofs generated against it will not verify on-chain. See RFC-0009.
 */
import { blake3 } from "@noble/hashes/blake3";

export type Scope =
  | { kind: "count" }
  | { kind: "aggregate" }
  | { kind: "row"; maxRows: number };

export interface Facts {
  nRows: bigint;
  schemaHash: Uint8Array; // 32 bytes
  freshness: bigint; // unix seconds
  fieldPresence: bigint;
}

function leU64(v: bigint): Uint8Array {
  const out = new Uint8Array(8);
  let x = BigInt.asUintN(64, v);
  for (let i = 0; i < 8; i++) {
    out[i] = Number(x & 0xffn);
    x >>= 8n;
  }
  return out;
}

function leI64(v: bigint): Uint8Array {
  // two's-complement little-endian, matching Rust i64::to_le_bytes
  return leU64(BigInt.asUintN(64, v));
}

/**
 * PredicateCommitment = H(n_rows || schema_hash || freshness || field_presence || salt)
 */
export function commit(facts: Facts, salt: Uint8Array): Uint8Array {
  if (facts.schemaHash.length !== 32) throw new Error("schemaHash must be 32 bytes");
  if (salt.length !== 32) throw new Error("salt must be 32 bytes");
  const h = blake3.create();
  h.update(leU64(facts.nRows));
  h.update(facts.schemaHash);
  h.update(leI64(facts.freshness));
  h.update(leU64(facts.fieldPresence));
  h.update(salt);
  return h.digest();
}

/** Estimate escrow budget for `n` queries at `perQuery` lamports. */
export function estimateBudget(perQuery: bigint, n: number): bigint {
  return perQuery * BigInt(n);
}

export const VERSION = "0.4.0-beta.1";

export type Cluster = "devnet" | "mainnet-beta" | "localnet";

export interface ConnectOpts {
  cluster: Cluster;
  rpcUrl?: string;
}

export interface BuyOpts {
  queries: number;
  ttlSecs: number;
}

export interface QueryResult {
  rows: unknown[];
  /** Query-correctness proof. `null` until RFC-0011 lands. */
  proof: Uint8Array | null;
}

/**
 * High-level client. Beta: the on-chain calls are stubbed until the Solana
 * programs are deployed (RFC-0001 §End-to-end flow). The shape is stable enough
 * to write integration code against.
 */
export class Horus {
  private constructor(readonly opts: ConnectOpts) {}

  static async connect(opts: ConnectOpts): Promise<Horus> {
    return new Horus(opts);
  }

  /** Open an escrow and request a capability for `listingId`. */
  async buyAccess(_listingId: string, _opts: BuyOpts): Promise<Uint8Array> {
    throw new Error("buyAccess: on-chain settlement not yet wired (beta)");
  }

  /** Submit a query against a capability. */
  async query(_cap: Uint8Array, _q: string): Promise<QueryResult> {
    throw new Error("query: gateway transport not yet wired (beta)");
  }

  /** Verify a query-correctness proof. Always false until RFC-0011 is sound. */
  async verify(_proof: Uint8Array | null): Promise<boolean> {
    return false;
  }
}
