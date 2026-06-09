# Draft — recursive proof aggregation for settlement claims

> Working notes for #5. Not normative. Expect this to change once the
> commitment-opening circuit (RFC-0011) stabilizes.

## Problem

Settlement (RFC-0007) releases escrow per answered query. A gateway serving a
busy listing produces one claim — and today, one proof — per answered query.
On-chain verification cost scales linearly with query volume, which makes
high-throughput listings uneconomical: the protocol fee on a sub-cent query is
dwarfed by the verification cost of its proof.

## Sketch

Batch a window of per-query correctness proofs into a single recursive proof so
the on-chain verifier checks **one** proof per settlement window instead of one
per query.

```
   π_1  π_2  ...  π_n          (per-query correctness proofs)
     \   |        /
      \  |       /
       fold (recursion)
            │
            ▼
          Π_window               (one proof; verifies all π_i)
```

- Each `π_i` attests "answer_i was computed correctly against commitment C".
- The folding step proves "I verified n proofs π_1..π_n, all against the same C,
  for claims summing to `owed_total`".
- The escrow verifier checks `Π_window` once and releases `owed_total` minus fee.

## Open

- [ ] Pick a recursion-friendly backend (Nova/SuperNova folding vs. a
      Halo2-style accumulator). Groth16 (current RFC-0011 backend) does not
      recurse cheaply.
- [ ] Bind every folded claim to the *same* listing commitment so a gateway
      cannot smuggle a claim against a different dataset into the batch.
- [ ] Decide the settlement-window boundary (block height vs. claim count).
- [ ] Cost model: prove the batched path is cheaper than `n` direct claims for
      realistic `n`.

Blocked on RFC-0011 circuit stabilization. See the status table there.
