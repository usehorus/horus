# Draft — on-chain (Solana) proof verifier program

> Working notes for #3 / the on-chain verifier line item in #2. Not normative.
> Depends on a transparent proof system replacing the current Groth16 setup.

## Why on-chain

Today settlement (RFC-0007) trusts that a claim's attached proof was checked
off-chain before escrow releases. That is a trust hole: a colluding relayer can
release escrow for an unverified claim. Moving verification into a Solana
program closes it — the escrow PDA releases only on a transaction that carries a
proof the program itself verified in the same instruction.

## Constraints

- **Compute budget.** A single Solana instruction is capped at 1.4M CU. Groth16
  BN254 pairing checks are feasible via the `alt_bn128` syscalls, but a naive
  verifier blows the budget. Each public input costs an `alt_bn128_multiplication`;
  keep the public-input vector small (commitment + packed predicate params).
- **Trusted setup.** A per-listing Groth16 setup is operationally hostile (#3).
  The on-chain verifier should target whatever transparent system that issue
  lands on; the program's verifying-key account layout must not assume Groth16.
- **Account model.** Escrow PDA holds funds; a separate verifying-key account is
  set once per circuit version. The claim instruction references both.

## Sketch

```
instruction: settle_claim
  accounts: [escrow_pda, vk_account, owner, buyer, system_program]
  data:     { proof, public_inputs, owed }
  logic:
    1. load vk from vk_account
    2. verify(proof, public_inputs, vk)         // alt_bn128 syscalls
    3. require public_inputs.commitment == escrow_pda.commitment
    4. require owed <= escrow_pda.remaining_budget
    5. transfer owed - fee(owed) -> owner; fee -> protocol; debit escrow
```

## Open

- [ ] Measure CU for a single pairing check on devnet; decide if batching claims
      (see #5) is a prerequisite for economic viability.
- [ ] Verifying-key account upgrade path (circuit version bump without
      orphaning live escrows).
- [ ] Reorg / replay protection on the claim nonce (mirror the access-layer
      nonce discipline in RFC-0004).

Blocked on #3 (transparent setup) — do not ship a mainnet verifier pinned to a
placeholder trusted setup.
