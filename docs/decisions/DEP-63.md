## DEP-63 — Money, currency and matching libraries declined: amounts are bare minor units, formatting is presentation, prices are posted

**Date** 2026-10-18 · **Approved by** the operator: step-31 (approved 2026-10-15), frozen with the S31-a PR design
2026-10-18 · **Relates to** `ARC-111`, `ARC-112`, [`REUSE_POLICY.md`](../REUSE_POLICY.md) · **Design**
`.structured-coding/plans/mvp0/step-31-wealth-prices.md` §19.4; `.structured-coding/plans/mvp0/pr-s31a-currency-asks.md` §4

**Problem.** A currency, prices and sales invite three kinds of library: money types with ISO tables, formatting,
and order books or matching engines.

**Options considered.**

```text
currency codes and money types   iso_currency, rusty-money, currency_rs
order book / matching engine     orderbook-rs and exchange crates
escrow / two-phase commit        none suitable; the design needs none (ARC-112 item 6)
property testing                 proptest, quickcheck
```

**Choice.** All declined.

1. **Money and currency crates.** Amounts stay bare `u64` minor units, as `ARC-38` made them; a world may declare
   a fantasy coin no ISO table holds; and formatting an amount is presentation, done by each client from the
   disclosed `exponent` and `symbol` (`ARC-111`). Only ISO 4217's minor-unit column is reused — as data, and as the
   citation for `exponent ∈ 0 … 4`. A dependency would add ISO tables a fantasy currency cannot use.
2. **Order books and matching engines.** MineWorld has posted prices settled by one owner at one instant, not a
   continuous double auction.
3. **Property testing.** The tree's in-test SplitMix64 idiom covers the seeded states the settlement proof needs
   with no new dependency.

**Why not them.** Each would bring a model MineWorld does not have — currency identity in every amount, an
exchange's order semantics — into economy's contract, against `REUSE_POLICY.md`'s "never force MineWorld inside a
framework's model". **Interface.** None: nothing is adopted. **Limitation.** If several currencies or an exchange
are ever wanted, this is revisited as a new record.
