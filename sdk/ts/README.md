# @usehorus/sdk (beta)

TypeScript client for [HORUS](https://github.com/usehorus/horus).

```bash
npm install @usehorus/sdk
```

```ts
import { commit, estimateBudget, type Facts } from "@usehorus/sdk";

const facts: Facts = {
  nRows: 12_000n,
  schemaHash: new Uint8Array(32),
  freshness: 1_730_000_000n,
  fieldPresence: 0n,
};

const commitment = commit(facts, new Uint8Array(32).fill(7));
const budget = estimateBudget(1_000n, 50); // 50_000n lamports
```

The commitment hashing must match the Rust reference (`horus-crypto::commit`)
byte-for-byte. See RFC-0009.
