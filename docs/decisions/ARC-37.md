## ARC-37 — Owning and giving: kinds, holdings, give, and what a person can carry

**Date** 2026-10-07 · **Approved by** the primary session at 11d's design freeze (step-10 §4.4.0;
QS-27 operator-visible, QS-28 … QS-34, QS-36, QS-37 accepted) · **Implements**
[`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§6.3, 7, 13.1, [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 ·
**Relates to** `INV-7`, `INV-10`, `INV-13`, `ARC-23`, `ARC-26`, `ARC-28`, `ARC-31`, `ARC-34`, `ARC-35`,
`ARC-36`, [`MVP.md`](MVP.md) §9 `AC-1`, `AC-2` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §2.6, SD-13, SD-16 … SD-21, §4.4 (S9, PR 11d)

**Problem.** Market Town begins with people who own things and give them to each other. Three
questions decide whether that can be done as installed System Packs without breaking single ownership
(`CLAUDE.md` §4 rule 1):
1. Who owns what an item kind *is*, and who owns how many of each kind somebody holds.
2. How a pack that decides a give — and later a purchase or a shift's production — changes holdings
   it does not own.
3. How a world whose people only give, and never use anything up, stays alive. Nothing in MVP-0 eats,
   drinks or sleeps (step-10 QS-10).

The R-S9-1 spike (step-10 §9 E-4) answered the third with a measurement. Social Café has one person
whom no seat names, Otto, and no controller consults him. He is given to and never gives, so he is an
**absorbing sink**. With no bound, he held 31 of the town's 33 items by day 30, and gives fell from
739 in days 1–15 to 312 in days 16–30.

**Options considered, for the sink.**

```text
(a) a person carries at most N items, all kinds    a pack rule in inventory; the sink becomes finite.
    together                                       Chosen
(b) consent: the taker must accept a pending give  a process and a second action; an undriven person
                                                   never accepts, so it also works, at about twice the
                                                   code and with offers nobody headless can answer
(c) give only to people a controller drives        impossible: a world does not know who is driven
                                                   (INV-1)
(d) content only                                   nothing in content stops a person receiving
(e) retune the paced controller                    forbidden: its offer band was frozen before the
                                                   market existed (ARC-34, ARC-35 item 6, I-9)
```

**Choice.**

1. **`item` owns what a kind is.** It owns `ItemKind { category }` on Item entities, from the `item:`
   section of an item file, `{ category: <slug> }`. A category is 1–32 bytes of `a–z`, `0–9` and `-`,
   neither beginning nor ending with `-`, its own type, refused at its line and column. It states the
   public genesis fact `item-kind-declared { item, category }` and alone reduces it. It provides no
   action, runs no process, depends on nothing, discloses nothing and has nothing biographical.
   `is_declared(world, item)` is the question other packs ask. An item file with no `item:` section is
   an inert entity that no pack trades (`ARC-36`).
2. **`inventory` owns holdings, and only it writes them.** It owns `Holdings` on Persons and
   Organizations: a list of `{ item, count }` sorted by item, with no zero entries. It is a list and not
   a map keyed by item because an `ItemId` serializes as `{ entity, type }`, which cannot be a JSON
   object key, and payloads, observations and snapshots are JSON (`DEP-5`). The `holdings:` section of
   a person or organization file, `{ <item key>: <count ≥ 1> }`, names Items. Its facts are `stocked {
   holder, item, count }` (genesis, visible to the holder) and `items-transferred { from, to, item,
   count }` (visible to its two participants). It depends on `item`. It discloses a holder's
   `Holdings` to that holder only (`INV-13`). Nothing it states is biographical: eighteen thousand
   gives in 300 days would bury a biography, and owning a coffee is not an event in a life.
3. **The owner decides, three times, through one function (`ARC-26`).** `admit_transfer(world, from,
   to, item, count)` is the whole of what `Holdings` refuses about a transfer:
   - the count is at least one;
   - `from` and `to` differ;
   - both are living Persons or Organizations;
   - `item` is a declared kind;
   - `from` holds at least `count`;
   - `to` can take `count` (item 5).

   It is asked by a deciding pack's `validate`, by the checked constructor `transfer(world, from, to,
   item, count)`, and again by inventory's own reduction. On refusal the reduction writes nothing and
   fails with `KernelError::FactRefusedByOwner`. A `stocked` fact is checked at reduction the same way
   (a living holder, a declared kind, a count of at least one, within capacity).
4. **Seeding is checked in two halves, because the source forces it** (step-10 F-37). Every genesis
   fact of a World Pack is computed before any is reduced, so a section is seeded against a world with
   no state yet, and inventory's seed cannot ask whether `item` has declared a kind. The seed checks
   what it can — each key names an Item, each count is at least one, a person's total is within
   capacity — and the reduction checks the declared kind. That reduction follows `item-kind-declared`
   in genesis order, because items' sections are seeded before people's and organizations' (`ARC-36`
   item 7).
5. **A person carries at most `PERSON_CAPACITY = 6` items, all kinds together; an organization is not
   bounded.** This is inventory's rule. A transfer that would take a person past it is refused
   `TargetUnavailable`, by the constructor and the reduction alike. A person's authored holdings past
   it are refused at genesis, naming the file. `can_take(world, holder, count)` answers the question
   for an offer. In the spike, with this bound, every seat gave in every 30-day bucket over 300 days
   (at least 118 times each), and Otto ended holding exactly six. The bound also limits observation
   size (step-10 R-S9-2): a person holds at most six kinds, so at most six `give` offers per person
   nearby.
6. **`item-transfer` provides `give { item, count }`, targeting a Person, and owns nothing.** The
   requirement is the same place, within 3 000 mm, and an available target. To an observer who holds
   something, and for each *other* living Person present, it offers **one complete affordance per kind
   held, count 1**, in item order (`ARC-34`). The target is available when it can take one more. It
   never offers a give to the observer itself: perception asks a provider about every person present,
   the observer included (step-10 F-40). `validate` reads the payload, requires a living Person actor
   and a different living Person target (`NoSupportedInteraction` otherwise), evaluates the
   requirement against presence's positions, then asks `admit_transfer`. `resolve` states inventory's
   `items-transferred` through `transfer` and nothing else. It depends on `inventory` and `presence`.
   Disabled, `give` is answered `Unavailable`, is offered nowhere, and holdings never change (`INV-10`,
   `AC-2`).

The resulting dependencies, one way:

```text
item ◄── inventory ◄── item-transfer ──► presence
```

**Accepted limitations.**
- **Nothing is consumed.** Gives conserve items, so 11d's flow stays alive under the bound. Purchases
  in step-10 PR 11e move items from shops to people, and with a bound of six per person buying would
  stop once everyone is full. The operator decided on 2026-10-07 (step-10 QS-35) that 11e adds a
  consumption System Pack, which removes items through inventory's checked constructor. Inventory
  stays the only writer of holdings.
- **Item kinds have no names, and items are never perceived.** An observation lists the observer's
  place and the people in it, so `ItemKind` is disclosed to no one. A client shown `give { item:
  { entity: 21, type: item } }` cannot name the item (step-10 F-41, recorded for S12).
- **No item instances** (`ARC-36` item 3).
- **Organizations are unbounded.** A shop's stock is content; a limit would be a later pack's rule.
- **`items-produced` does not exist yet.** It arrives in 11e with its first stater, `employment`
  (step-10 QS-28). Adding a fact before anything states it would design it ahead of its consumer
  (`CLAUDE.md` §4 rule 11).
- **The capacity is a published constant**, not world configuration, under the `ARC-26` note's rule.

**Note, 2026-10-08 (S15, PR 12d; step-11 SD-D10, QD-4, QD-12, QD-15, QD-16; step-13 R-PK-2, F-41) —
item kinds have names, disclosed as a catalogue.** The accepted limitation "Item kinds have no names"
is lifted; "items are never perceived" stands. Nothing else in this decision changes.

1. **A kind has a display name.** The `item:` section is `{ category, name }`. `name` is required: 1–64
   bytes, no control characters, no surrounding whitespace — the rule `naming` applies to a person's
   name, restated in `item`, which depends on no pack. Two kinds may share a name; a name is display,
   never identity. `item-kind-declared` and the `ItemKind` component carry it (schema 2 each); `item`
   is version 2.
2. **The catalogue is disclosed on a place.** To whoever perceives a place, `item` discloses one
   `item-catalogue` record on it: `[{ item, category, name }]` for every declared kind of the world, in
   `ItemId` order, built from `ItemKind` at disclosure and never stored (economy's listing is the
   precedent, `ARC-38`). A world without `item` discloses none.
3. **Items are still never perceived as entities.** An observation's entities are the observer's place
   and the people in it, as before; a kind appears only inside the catalogue. A client shown `give {
   item: { entity: 21, type: item } }` now finds entity 21's name there (F-41).
4. Loose objects (`ARC-36` note) are not declared kinds, so the catalogue never lists them.
   Organizations still have no names; `ARC-38`'s accepted limitation is lifted for kinds only.
