# RFC-0007 — Escrow & per-query settlement

- **Status:** Accepted
- **Authors:** e.morales, k.tanaka
- **Created:** 2025-12-05
- **Implements:** `crates/horus-settlement`

## Motivation

Neither party should have to trust the other. The buyer should not pre-pay for
queries that are never answered; the owner should not answer queries that are
never paid for. HORUS resolves this with an on-chain **escrow** that releases
funds per *proven* answered query.

## Accounts

```
Escrow {
  buyer:       Pubkey,
  listing:     Pubkey,
  owner:       Pubkey,
  budget:      u64,     // lamports deposited
  spent:       u64,     // lamports released so far
  fee_bps:     u16,     // protocol fee, basis points (e.g. 250 = 2.5%)
  not_after:   i64,     // refund unlocks for buyer after this
  closed:      bool,
}
```

## Pricing

A listing carries a `price_schedule` mapping a `Scope` class to a per-query price
in lamports. The settlement program reads the price from the listing at claim
time; it is **not** trusted from the claim payload.

```
price(Count)      =  p_count
price(Aggregate)  =  p_agg
price(Row{n})     =  p_row_base + n * p_row_unit
```

## Claim

The owner (or gateway delegate) submits a claim referencing the escrow and a
**usage proof**: the capability nonce, the new `answered` counter, and a signature
chain binding the answered queries. The program:

1. verifies the claim's `answered` is strictly greater than the escrow's recorded
   count for that nonce (monotonicity — prevents double-claiming);
2. computes `owed = Σ price(scope_i)` for the newly-answered queries;
3. checks `spent + owed <= budget`;
4. transfers `owed * (1 - fee_bps/10000)` to `owner` and the remainder to the
   protocol treasury;
5. updates `spent`.

If `spent + owed > budget` the claim is **clamped** to the remaining budget and the
escrow is marked exhausted. The gateway MUST stop answering once budget is reached.

## Rounding

Integer division on `fee_bps` truncates toward zero. To avoid a zero-fee griefing
vector on tiny prices, the fee is `max(1, owed * fee_bps / 10000)` whenever
`owed > 0`. This minimum-fee rule mirrors the minimum-slash guard pattern and is
covered by a unit test (`fee_floor_on_small_amounts`).

## Refunds

After `not_after`, the buyer MAY close the escrow and reclaim `budget - spent`.
Closing is idempotent and sets `closed = true`.

## Failure modes & tests

- double claim (same `answered`) → reject (monotonicity)
- claim exceeding budget → clamped, escrow exhausted
- fee floor on sub-unit prices → fee == 1
- refund before `not_after` → reject
- refund after `not_after` → transfers `budget - spent`
- claim on closed escrow → reject
