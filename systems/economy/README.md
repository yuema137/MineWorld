# systems/economy/

**`mineworld-economy`**: wallets, sellers' asks, shops, buying and wages — and the only pack that moves
money.

```text
configures  configure/economy.yaml   required: the world's one currency (ARC-111); optional
                                     price_modifiers that answer happenings (ARC-88)
section     economy              a person's `economy: { wallet, asks? }`; an organization's
                                 `economy: { wallet, shop?: { at, prices, keepers? } }`; an item
                                 kind's `economy: { reference }`
emits       currency-declared    at genesis, SystemInternal
            funded, shop-opened  at genesis
            listed               an ask: at genesis, or from list-for-sale
            price-set, withdrawn from set-price and withdraw
            sold                 a sale, stated beside its money-transferred and items-transferred
            money-transferred    the one fact that moves money: a sale, or a wage
            wage-unpaid          a wage the employer could not pay
            deduction-unpaid     a wage deduction the employee could not pay (ARC-287)
            items-transferred    inventory's fact, for a sale, through `transfer`
owns        Wallet, Asks, Shop,  a wallet on a person or organization; asks on a seller; a shop on
            ReferenceValue       its place; a reference value on an item kind
provides    buy { item, price }  one complete affordance per ask: in a shop, or of a person seller
            list-for-sale, set-price, withdraw   incomplete; `for: shop` by one of the shop's keepers
hears       wage-due,            employment's — no system dependency on employment
            wage-deduction-due   (moves the amount from the employee to the employer)
discloses   Wallet, wealth       to its holder only
            currency             to every living person: name, symbol, exponent, reference values
            asks                 a person seller's prices and whether each is available
            the shop's listing   to whoever is there: operator, prices, stock
depends on  inventory, presence
```

Money is integer minor units (`u64`), so a balance never goes negative. A world says what one unit is
in its `configure/economy.yaml`: no currency is written into code. A buyer submits the price it saw;
if the seller changed it, the buy is refused `price-changed` and nothing moves. A sale is settled in
one dispatch: `sold`, the payment and the hand-over, or a refusal by name and nothing at all.

```sh
cargo test -p mineworld-economy
```

The decisions are [`ARC-38`](../../docs/decisions/ARC-38.md), [`ARC-111`](../../docs/decisions/ARC-111.md)
… [`ARC-114`](../../docs/decisions/ARC-114.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Designs and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.5 and
[`pr-s31a-currency-asks.md`](../../.structured-coding/plans/mvp0/pr-s31a-currency-asks.md).
