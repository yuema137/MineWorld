# systems/economy/

**`mineworld-economy`**: wallets, shops, buying and wages — and the only pack that moves money.

```text
section     economy              a person's or organization's `economy: { wallet }`; an organization
                                 may add `shop: { at: <place>, prices: { <item>: <price> } }`
emits       funded, shop-opened  at genesis
            money-transferred    the one fact that moves money: a purchase, or a wage
            wage-unpaid          a wage the employer could not pay
            items-transferred    inventory's fact, for a purchase, through `transfer`
owns        Wallet, Shop         a wallet on a person or organization; a shop on its place
provides    buy { item }         in a shop: one complete affordance per priced kind
hears       wage-due             employment's — no system dependency on employment
discloses   Wallet               to its holder; a shop's listing (prices, stock) to whoever is there
depends on  inventory, presence
```

Money is integer minor units (`u64`), so a balance never goes negative: a payment larger than the
balance is refused. When employment says a wage is due, economy pays it from the employer's wallet,
or records `wage-unpaid`. A buy that cannot happen — out of stock, cannot pay, cannot carry — is still
offered, unavailable, so a client can show what is for sale.

```sh
cargo test -p mineworld-economy
```

The decision is [`ARC-38`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.5 (E-C4).
