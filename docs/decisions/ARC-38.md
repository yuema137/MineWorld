## ARC-38 — Work, money, shops and consumption: who owns each, and how the loop is kept alive

**Date** 2026-10-07 · **Approved by** the primary session at 11e's design freeze (step-10 §4.5.0;
QS-39, QS-45 and QS-47 operator-material, accepted as designed; QS-40 … QS-44, QS-46, QS-48 … QS-53
accepted as recommended); the consumption pack itself by the operator (step-10 QS-35, 2026-10-07) ·
**Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§8, 10, 13.1,
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1, [`MVP.md`](MVP.md) §§3, 5 · **Relates to** `INV-7`, `INV-10`,
`INV-13`, `ARC-23`, `ARC-26`, `ARC-28`, `ARC-29`, `ARC-31`, `ARC-32`, `ARC-34`, `ARC-35`, `ARC-36`,
`ARC-37`, [`MVP.md`](MVP.md) §9 `AC-1`, `AC-2` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §2.6, SD-13, SD-22 … SD-28, §4.5 (S9, PR 11e)

**Problem.** Market Town's people own and give things (`ARC-37`). The second half of the measured
transformation (`ARC-35` item 1) adds work, money, shops and the using-up of things, as installed
System Packs only. Five questions decide whether that keeps single ownership (`CLAUDE.md` §4 rule 1)
and a living market:
1. Who moves money, when a shift that one pack decides was worked must pay a person from an
   organization's funds.
2. How items come into existence and leave it, when only `inventory` writes holdings.
3. How work is noticed, when no controller knows that work exists.
4. How a shop is authored, when a System Pack owns at most one section.
5. Whether the money and item loops stay alive for the measured 300 days, and what is changed if
   they do not.

**Options considered.**

```text
closing the item loop  (a) shops buy goods back            declined by the operator (QS-35)
                       (b) a consumption pack              chosen by the operator (QS-35)
                       (c) relax CP-4's purchase criterion declined (QS-35)
production and use     (a) facts in the stating packs'     inventory would reduce another pack's
                           vocabularies, reduced by           vocabulary (against ARC-26 item 1)
                           inventory under ARC-28
                       (b) inventory's own facts,          chosen (QS-42)
                           stated through checked
                           constructors (ARC-26)
attendance             (a) polling presence at a fixed     wakes the whole town for one fact; worked
                           interval                         time only as fine as the interval
                       (b) presence at the shift's start,  chosen: exact, no polling; presence's
                           then presence's                 person-entered-place carries `from`
                           person-entered-place between      (step-10 F-49)
                           wakes (ARC-28: reacting into
                           employment's own state)
authoring a shop       (a) a `shop:` section on the place  a second section for economy; a pack owns
                                                           one (step-10 F-47) — a framework change
                                                           justified only by the market (I-2)
                       (b) on its operator organization,   chosen (QS-43); no place file changes
                           inside economy's one section
a drained market       (a) retune the paced controller     forbidden (ARC-34, ARC-35 item 6, I-9)
                       (b) size content against a          chosen
                           criterion stated before
                           measuring
```

**Choice.**

1. **`inventory` gains the two facts that create and remove items** (QS-41, QS-42). `items-produced {
   holder, item, count }` and `items-consumed { holder, item, count }` are inventory's vocabulary,
   reduced by inventory alone, visible to the holder. Their checked constructors `produce` and
   `consume` ask `admit_production` (a living holder, a count of at least one, a declared kind, and
   `can_take` — production into a person respects the capacity of six) and `admit_consumption` (a
   living holder, a count of at least one, a declared kind, at least `count` held). The reduction asks
   the same function again and, on refusal, writes nothing and fails `FactRefusedByOwner` (`ARC-26`,
   `ARC-37` item 3). The fact names say what happened to holdings, not why; the stater is the cause.
2. **`employment` owns jobs, the `employed-by` edge and the `shift` Process; work is attendance**
   (QS-8). Its section `job:` on a person file is `{ employer: <organization key>, workplace: <place
   key>, from: "HH:MM", until: "HH:MM", wage: <minor units per hour>, produces: { <item key>: <count
   per full shift ≥ 1> } }`. `from < until`: a shift lies within one day (QS-50). Times are schedule's
   `TimeOfDay`, a Cargo dependency on its type only, so the town has one time-of-day convention
   (`ARC-32`).
   - The genesis fact `hired { employee, job }` gives the person `Employment`, the `employed-by` edge
     (Person → Organization) and one `shift` Process, woken at each next `from` and `until`.
   - At `from` the wake states `shift-started { employee, present }`, where `present` is presence's
     answer: is the employee at the workplace.
   - Between wakes employment reacts to presence's `person-entered-place` for an employee on shift.
     Entering the workplace starts a present span; entering anywhere else *from* the workplace closes
     it into the worked seconds. That is employment writing its own state by reacting (`ARC-28`).
   - At `until` the wake states `shift-ended { employee, worked }`, then, when `worked > 0`,
     `wage-due { employee, employer, amount = wage × worked ÷ 3 600 }` and, per produced kind,
     inventory's `items-produced` for the employer with `count = per_shift × worked ÷ shift length`
     (integer floor, skipped at 0), through `produce`.
   - It depends on `inventory` (it states inventory's fact) and `presence` (it reads positions and hears
     arrivals). It discloses `Employment` to the employee only. `hired` is biographical (QS-49):
     shifts, wages, purchases and meals are thousands of facts that would bury a biography.
   - **It never reads or writes a `Wallet`.** It has no write token for one (`INV-7`), and its
     manifest does not name economy.
3. **`economy` owns `Wallet` and `Shop`, and is the only mover of money.**
   - Its one section, `economy:`, on person and organization files: `{ wallet: <minor units> }`, and on
     an organization optionally `shop: { at: <place key>, prices: { <item key>: <price ≥ 1> } }`
     (QS-43). The genesis facts `funded { holder, balance }` (visible to the holder) and `shop-opened {
     place, operator, prices }` (public) become `Wallet { balance: u64 }` on the holder and `Shop {
     operator, prices }` on the place. `u64` minor units: no floating point and no negative balance
     can be represented (I-6).
   - `money-transferred { from, to, amount }` is the one fact that moves money. Its reduction refuses
     an amount larger than the payer holds, a zero amount, a payer who is the payee, or a party that is
     not a living Person or Organization — `FactRefusedByOwner`, writing nothing.
   - **`buy { item }`** has no target. Its requirement is `at_place(shop)` with an available target,
     because a target-less offer can only say "at that place" (step-10 F-48). It is offered to a
     living person standing in a shop's place as **one complete affordance per priced kind**
     (`ARC-34`): available when the operator holds one (`admit_transfer`), the buyer can pay and the
     buyer can carry it; otherwise unavailable with `TargetUnavailable`, the one reason an offer can
     carry (QS-44). `validate` asks the same through `SpatialRequirement::evaluate`; a buyer who is not
     in a shop, or a kind the shop does not price, is `NoSupportedInteraction`. `resolve` states
     `money-transferred` (buyer → operator, visible in the shop's place, QS-45) and inventory's
     `items-transferred` (operator → buyer) through `transfer`.
   - **Wages.** Economy subscribes to employment's `wage-due`, decoding it through employment's
     published type — a Cargo dependency, **no system dependency** (`ARC-28`). It answers with
     `money-transferred` (employer → employee) caused by the `wage-due`, or, when the employer cannot
     pay, `wage-unpaid { employee, employer, amount }`. This is `CORE_CONCEPTS.md` §13.1's example,
     implemented literally.
   - It depends on `inventory` (it states `items-transferred`, and reads the operator's stock) and
     `presence`. It discloses a `Wallet` to its holder, and to everyone perceiving a shop's place the
     shop's **listing**: the operator, and per priced kind its price and how many the operator holds
     (QS-45). That count is read from inventory's state, which economy may read because it depends on
     inventory; it never writes it. Inventory itself still discloses holdings to the holder only. The
     listing is how a second client perceives a purchase (step-10 CP-7) without anybody's holdings being
     disclosed. Nothing economy states is biographical.
4. **`consumption` provides `eat { item }` and `drink { item }`, and owns nothing** (QS-35, QS-39).
   A held kind whose `ItemKind` category is `food` is eaten and one of category `drink` is drunk
   (published constants `EATEN` and `DRUNK`); goods are never consumed. There is no spatial
   requirement: a person eats what they carry, wherever they are (QS-40). It offers a living person one
   complete affordance per edible or drinkable kind held, without a target, in item order. `validate`:
   the payload, a living Person actor, no target, the action matching the kind's category
   (`NoSupportedInteraction` otherwise), then `admit_consumption(actor, item, 1)`. `resolve` states
   inventory's `items-consumed` through `consume` and nothing else. It depends on `inventory` (system
   and Cargo) and reads `item`'s `ItemKind` (Cargo). It closes `MVP.md` §5's `eat` as an interaction
   and adds `drink`, without which drinks would fill hands (step-10 F-50).
5. **Shops sell consumables only, and their stock is produced by the shift** (QS-48). Goods keep
   circulating by `give` (`ARC-37`); a bought good would occupy one of a person's six places for
   good. The item loop is then produce → buy → give → eat or drink.
6. **The loop is sized in content, against a criterion stated before measuring** (`ARC-23`, I-7, I-9).
   Endowments, prices, wages, production and opening stock are World Pack content. They are fixed by a
   300-day seed-7 run of Market Town read from its save, against conditions written down before the
   first run (step-10 §4.5.3 E-9 b): in every 30-day bucket a purchase, a wage paid to each job holder,
   an item produced, an item eaten or drunk, and a give; zero `wage-unpaid`; no wallet ever below the
   cheapest price in the town; money conserved; nobody holding more than six. A run that fails is
   recorded and the content or the packs are changed, never the controller.

The resulting dependencies, one way:

```text
item ◄── inventory ◄── item-transfer ──► presence
            ▲  ▲  ▲
            │  │  └──── consumption ┄┄► item           (┄┄ Cargo only: ItemKind)
            │  └─────── economy ──► presence
            │              ┆
            │              ┆ subscribes to wage-due: Cargo dependency on employment's type, no system
            │              ┆ dependency (ARC-28)
            └────────── employment ──► presence
```

**Accepted limitations.**
- **A bounded-horizon economy, not a closed one** (QS-47; living-world gap **L-13**). Two people hold
  jobs (MVP §3, QS-46). The other people have no income and live on an endowment sized for the
  measured 300 days; past that horizon they run out. Closing the loop — more jobs, or income without
  one — is a later step's work, not a retuning of this one. A market that balances itself over years
  is outside S9 (step-10 §1.2, R-S9-3).
- **No hunger, appetite or sleep.** Consumption is an interaction, not a need: people eat because they
  are offered food, at the paced controller's rate. A needs pack is later work (step-10 QS-10).
- **One reason for an unavailable buy.** Out of stock, cannot pay and cannot carry all read
  `TargetUnavailable`, in the offer and at dispatch, because an offer carries one pack-supplied verdict
  (step-10 F-48). Finer reasons wait for a contract that lets an offer carry one.
- **Shifts lie within one day**: `from < until`, no shift across midnight (QS-50).
- **No hiring, firing, promotion, business ownership or buying back** (step-10 §1.2, QS-35).
- **Item kinds and organizations still have no names** (step-10 F-41): a listing names kinds by id.
- **Prices, wages and endowments are content, not System Pack configuration** — they are values a
  world states, as `ARC-26`'s note says of constants that are not yet configuration.
- **A pack owns one section** (F-47): economy's shop is authored on its operator. The limit is worked
  within, not changed; lifting it would be framework work justified only by the market.

---

## ARC-38 note — sellers set prices; item 5 lifted; QS-44 superseded (2026-10-18, S31-a)

Amended by [`ARC-112`](ARC-112.md) (step-31, S31-a; operator-approved 2026-10-15, frozen 2026-10-18).

- **Item 3, the shop and `buy`.** `Shop` on the place keeps its operator and gains `keepers`; its prices move to
  the operator's `asks`, the same record a keeper changes. `buy { item }` becomes `buy { item, price }`, the price
  the buyer observed, from a shop (no target) or from a Person seller (targeted). A purchase states `sold` before
  the payment and the transfer, in the same dispatch. Everything else in item 3 — wallets, `money-transferred` as
  the one mover of money, wages — is unchanged.
- **Item 3's single reason, `TargetUnavailable` (step-10 QS-44), is superseded.** A buy is refused with a named
  code — `not-for-sale`, `price-changed`, `insufficient-funds`, `out-of-stock`, `cannot-carry` — in the offer and
  at dispatch alike, because `Offer::refused` carries it (`ARC-34` note). The limitation "one reason for an
  unavailable buy" no longer holds.
- **Item 5 is lifted.** Shops and Persons may sell any kind; what a seller lists is the seller's choice. The
  reason behind item 5 — a bought good would fill one of a person's six places — is kept by inventory's capacity
  (`cannot-carry`), not by a ban.
- Amounts gain a declared scale: the world's currency is economy's configuration (`ARC-111`).
