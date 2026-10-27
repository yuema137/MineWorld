# The shared trade module — specification

This is the specification of `clients/shared/trade`, the module both reference clients use to show money,
prices, the sale panel, the stall and the notice board. It is a document for coding agents (`CLAUDE.md`
§2.1). The design it implements is `.structured-coding/plans/mvp0/pr-s31e-clients.md` (S31-e, frozen
2026-10-20); the decision is [`docs/decisions/ARC-184.md`](../../docs/decisions/ARC-184.md).

## 1. Purpose and boundary

The world says what its money is called and how many decimal places it has (`economy`'s `currency` record,
`ARC-111`). It says who sells what at which price (`asks`, `shop`, and the priced `buy` offers, `ARC-112`),
and which notices are current (`notices`' `notice-board`, `ARC-115`). Before this module each client divided
every amount by 100 and read the wallet and the shop listing with its own copy of the same reader. Now one
module does the reading and the formatting, and each client draws the result in its own look.

The module may know:

- the JSON **shapes** of the records `economy`, `inventory`, `item` and `notices` disclose (§4);
- the shape of an affordance (`clients/protocol/PROTOCOL.md` §5): `action_type`, `target`, `payload`,
  `available`, `unavailable_reason`;
- wording keys (`format.money.*`, `trade.*`, `board.*`, `hud.money*`, `panel.trade`, `panel.stall`,
  `panel.board`, `ui.unavailable`).

The module may not:

- name an action type, an item, a person, a place, a topic, a currency or a coordinate of any world;
- decide whether anything is allowed: it never compares two amounts, never compares a wallet with a price,
  never hides a row because the server marked it unavailable;
- read a reference value (the `currency` record's `references` field is dropped unread by `currency_of`);
- compute whether a notice has expired: every notice of the `notice-board` record is current;
- submit anything: each client's `intents.gd` is its only submitter (`ARC-47` R1).

The client-rules scan holds the first three structurally (`tests/acceptance/tests/client_rules.rs`, check
**e**, §6).

## 2. Layout and adoption

```text
clients/shared/trade/
  money.gd     MineWorldMoney   format and parse amounts in a disclosed currency (§3)
  trade.gd     MineWorldTrade   readers of the trade records (§4); the row model both clients draw (§5)
clients/shared/checks/
  money_check.gd, trade_check.gd   headless checks of the module over hand-built frames
clients/2d/mineworld_trade        -> ../shared/trade   (symlink, res://mineworld_trade)
clients/3d-spike/mineworld_trade  -> ../shared/trade   (symlink, res://mineworld_trade)
```

The module depends on `MineWorldText` (the settings module) for wording. It reads observations through the
`MineWorldObservation` interface (`own_component`, `component_value`, `affordances`, `observer`, `place`,
`ids`), duck-typed, so it runs inside `clients/shared` without the protocol module. The launchers
(`mineworld-2d`, `mineworld-slice`) refuse a missing `mineworld_trade` as they refuse the other modules, and
the nightly Windows jobs check that the symlinks arrived as directories. There is no platform branch.

Run the checks:

```sh
godot --headless --path clients/shared --import
godot --headless --path clients/shared --script res://checks/money_check.gd
godot --headless --path clients/shared --script res://checks/trade_check.gd
```

## 3. Money (`MineWorldMoney`)

### 3.1 The currency value

`MineWorldMoney.currency_of(observation) -> Dictionary` reads the observer's own `currency` record and keeps
exactly `{ name, symbol, exponent, symbol_at }`; `exponent` is clamped to 0 … 4. With no record, or a record
without a numeric `exponent` and a string `symbol`, it returns `{}`.

### 3.2 Formatting

`format(amount, currency) -> String`, `amount` an integer of minor units:

1. With `currency` empty, the amount is the bare integer, through `format.money.plain` (`"{amount}"`).
   Nothing is divided, because no scale was disclosed.
2. Otherwise, with `e = exponent`: the major part is `amount / 10^e`, grouped in threes with
   `format.money.group_mark`; if `e > 0` the minor part is `amount % 10^e`, zero-padded to `e` digits, after
   `format.money.decimal_mark`. A negative amount keeps a leading `-` (the server sends none).
3. The symbol goes where the world says: `format.money.before` (`"{symbol}{amount}"`) when `symbol_at` is
   `before`, `format.money.after` (`"{amount} {symbol}"`) when it is `after`.

| Currency | Amount | Text (en) |
| --- | --- | --- |
| `{ US dollar, "$", 2, before }` | 200 000 | `$2,000.00` |
| the same | 300 | `$3.00` |
| `{ crown, "cr", 0, after }` | 3 | `3 cr` (never `0.03 cr`) |
| `{ dinar, "KD", 3, before }` | 1 500 | `KD1.500` |
| `{ unidad, "UF", 4, after }` | 12 345 | `1.2345 UF` |
| `{ US dollar, "$", 2, before }` | 1 000 000 000 000 | `$10,000,000,000.00` |

### 3.3 Parsing a typed price

`parse(text, currency) -> int | null`:

- `currency` empty → `null`;
- leading and trailing spaces, the symbol, spaces and group marks are dropped;
- at most one decimal mark (`.` or the wording's `format.money.decimal_mark`), with 1 … `e` digits after it,
  padded with zeros to `e`; with `e = 0`, any decimal mark is `null`;
- otherwise digits only; no sign; at most 18 digits in total.

Examples, in dollars: `"2.6"` → 260, `"2.60"` → 260, `"$2.60"` → 260, `"2.605"` → `null`, `"-1"` → `null`.
In crowns: `"12"` → 12, `"1.5"` → `null`. `"0"` → 0 in both.

The parser checks representation, not rules. `0` and an amount above the world's bound are sent, and the
server refuses them `price-out-of-range`. A `null` is never sent: the price field says
`trade.price.not-an-amount` and stays open.

## 4. The records read (`MineWorldTrade`, static)

Each reader takes an observation and returns plain data. An identity arrives either as a string or as the
contract's typed reference `{ entity, entity_type }`; readers return the id string as `item` and keep the
reference exactly as disclosed as `ref`, because a request must carry the reference in the shape the server
wrote it.

| Reader | Record (owner) | Where | Returns |
| --- | --- | --- | --- |
| `MineWorldMoney.currency_of` | `currency` (economy) | own | `{name, symbol, exponent, symbol_at}` or `{}` |
| `money_of` | `wallet` (economy) | own | `{balance}` or `{}` |
| `wealth_of` | `wealth` (economy) | own | `{amount, unvalued}` or `{}` |
| `held_of` | `holdings` (inventory) | own | `[{item, ref, count}]` |
| `asks_of(obs, person)` | `asks` (economy) | about `person` | `[{item, ref, price, available}]` |
| `own_asks` | `asks` (economy) | about the observer | the same |
| `listing_of(obs, place = here)` | `shop` (economy) | about the place | `{operator, listed: [{item, ref, price, in_stock}]}` or `{}` |
| `shops_in_view` | `shop` | every perceived entity | the ids of places that disclose a listing |
| `catalogue_of` | `item-catalogue` (item) | about the observer's place | `[{item, ref, category, name}]` |
| `board_of` | `notice-board` (notices) | own | `[{id, topic, publisher, body, issued_at, valid_until}]`, in the record's order |
| `has_board` | `notice-board` | own | whether a board is disclosed at all |
| `topics_of` | `subscriptions` (notices) | own | `[topic]` |

### 4.1 Shapes, as the server writes them

Pinned from the System Packs' serialisation (`systems/economy/src/{currency.rs,component.rs,action.rs}`,
`systems/notices/src/{component.rs,board.rs,body.rs}`, `contracts/src/ids.rs`, at `origin/main @ fec906f2`).
A live-frame pin is part of the milestone drive (§7).

```text
currency      { "name": "US dollar", "symbol": "$", "exponent": 2, "symbol_at": "before",
                "references": [ … ] }                         references: never read by a client
wallet        { "balance": 200000 }
wealth        { "amount": 201500, "unvalued": 2 }
asks          { "asks": [ { "item": {"entity": "41", "entity_type": "item"}, "price": 450,
                            "available": true } ] }          about a person seller, to every perceiver
shop          { "operator": {"entity": …}, "listed": [ { "item": {…}, "price": 300, "in_stock": 6 } ] }
buy offer     { "action_type": "buy", "target": null | "<person>", "available": …,
                "unavailable_reason": …, "payload": { "item": {…}, "price": 300 } }
seller offers list-for-sale, set-price, withdraw: incomplete (no "payload"), target-less, once each
notice-board  { "notices": [ { "id": 7, "topic": "prices.street", "publisher": "<person>",
                               "body": {"prices": [ {"item": "41", "price": 280, "basis": 6} ]}
                                     | {"text": "The market opens late tomorrow."},
                               "issued_at": 3600, "valid_until": 25200 } ] }
subscriptions { "topics": [ "news.street", "prices.street" ] }
```

## 5. The row model

### 5.1 Rows and buttons

A **row** is one line of a trade view; a **button** is one thing the player can do on it. Rows are plain
dictionaries, so both clients draw the same data and the harnesses print it as it is.

```text
Row    = { kind: "sale" | "ask" | "held" | "shelf" | "note" | "answer",
           subject: String,        the seller (a person id), "" for the shop here, the observer for own rows
           item: String,           an item id, "" when none
           text: String,           already worded: names from the world, amounts through MineWorldMoney
           amount: int | null,     the price shown, in minor units (data for reports; never compared)
           available: bool | null, the server's verdict on the row's offer, shown only
           reason: Variant,        the server's reason, shown only
           buttons: [Button] }
Button = { label: String,          already worded
           affordance: Dictionary, the offer this button acts on, exactly as the frame carried it
           fill: Dictionary,       request values the row already knows: item (a reference), shop (true)
           ask: [String] }         what the player still supplies: "item" (a picker), "amount" (a price)
```

### 5.2 The client's table

A `MineWorldTrade` instance is made with `MineWorldTrade.new(inputs, names, reasons)`:

- `inputs` is the client's own table of the incomplete actions it composes for trade, in its `intents.gd`:
  `action type → { ask: [...], row: [row kinds it serves], label: wording key }`. This table is the only
  place an action type is spelled. The module matches an incomplete offer to a row by the row kind its type's
  entry lists, and never by the type's name.
- `names: Callable(id) -> String` names an id from the client's name memory and never returns the id.
- `reasons: Callable(reason) -> String` words a rejection.

S31-e's entries, the same in both clients:

```text
"list-for-sale": { "ask": ["amount"], "row": ["held", "note"], "label": "trade.list" }
"set-price":     { "ask": ["amount"], "row": ["ask", "shelf"], "label": "trade.change-price" }
"withdraw":      { "ask": [],         "row": ["ask", "shelf"], "label": "trade.withdraw" }
```

### 5.3 Builders

| Builder | Rows |
| --- | --- |
| `sale_rows(obs, subject, currency)` | one `sale` row per **priced offer** about `subject`, in the server's order. A priced offer is a complete affordance whose payload carries an integer `price` and a typed `item` reference (SD-S31e-3). `subject ""` reads the target-less offers (the shop where the observer stands); a person id reads the offers targeted at that person. Button *Buy* = that offer. Each incomplete offer about the same subject whose entry lists `sale` adds a button with `fill = { item }` (S31-h's *Offer a price…*) |
| `stall_rows(obs, currency)` | one `ask` row per own ask (buttons from the entries listing `ask`, `fill = { item }`), then one `held` row per held kind without an ask (entries listing `held`) |
| `shelf_rows(obs, currency)` | while the observer's place discloses a listing and some entry lists `shelf` or `note`: one `shelf` row per listed kind (`fill = { item, shop: true }`) and one `note` row, *List another kind for the shop…*, whose buttons add `item` to `ask` |
| `answer_rows(obs, currency)` | none in S31-e; S31-h implements it from its record (§5.6) |
| `tag_of(obs, person, currency)` | a person's price tag: the first ask, `trade.tag` or `trade.tag.sold-out`, then `trade.tag.more` for more; `""` for none |
| `shop_board(listing, currency)` | a shop's board: up to four `trade.tag` lines, then `trade.tag.rest` |
| `board_rows(obs, currency, clock)` | one row per notice of the board record: `{id, topic, publisher, title, lines}`; a `text` body is one line, a `prices` body one `board.row.price` line per price line |
| `report(rows)` | the rows as the `TRADE` report prints them: buttons as `{label, action_type, complete, ask}` |

A row whose offer the server marked unavailable is worded with `ui.unavailable` and is **kept**: its
buttons stay pressable and the server answers.

### 5.4 Pressing a button

Each client has one path for a click, Enter and the harness:

1. If the button's `affordance` is complete and `ask` is empty → `intents.submit_offered(affordance)`,
   unchanged (`ARC-34`).
2. Otherwise the client collects what `ask` lists: an item from `catalogue_of` (a picker), an amount through
   the price field and `MineWorldMoney.parse`. Then `intents` composes the request from
   `fill ∪ { item, price }`. Only `intents.gd` maps those values to a payload:

   | Type | Payload |
   | --- | --- |
   | `list-for-sale`, `set-price` | `{ item: <the reference>, price: <int> }`, plus `for: "shop"` when `fill.shop` |
   | `withdraw` | `{ item }`, plus `for: "shop"` when `fill.shop` |

3. The press path never reads `available` or `reason`; they are read where rows are built.

### 5.5 What each client draws

| View | 2D | 3D |
| --- | --- | --- |
| money line | `hud/status.gd`, a fourth line | `slice_main.gd`, a `ControlsHud` line |
| sale panel, stall, shelf | `hud/trade_panel.gd`, beside the interaction menu | `SliceTradePanel`, beside `SliceOfferMenu` |
| price field | the typed line (`talk_line.ask`) | a `LineEdit` in the trade panel |
| item picker | buttons in the trade panel | buttons in the trade panel |
| person tag | `scene/people.gd`, a label under the participation marker | `figures.gd`, a second `Label3D` |
| shop board | `scene/places.gd`, at the façade anchor | `SlicePriceBoards`, above the bound door |
| notice board | `hud/board_panel.gd`, key N | `SliceBoardPanel`, key N |

The trade panel opens and closes with the interaction menu (SD-S31e-4): on a person, it shows that person's
sale rows; on oneself, the sale rows of the shop here, the stall, and a collapsed *This shop's prices…*
button for the shelf (SD-S31e-5). It is rebuilt from every observation while open. With no rows it is not
shown.

The notice board shows every notice of the record and nothing else. Its badge, `board.badge`, counts notice
ids not in the set shown when the board was last opened; the set lives in memory and is dropped on an
instance change.

### 5.6 The interface S31-h extends

S31-h adds rows without changing this model or these functions:

1. *Offer a price…*: an `inputs` entry `"propose-price": { ask: ["amount"], row: ["sale"], label: … }`;
   `sale_rows` already attaches a button for each incomplete offer about the subject whose entry lists `sale`.
2. Offers to answer: S31-h implements `answer_rows`, returning `answer` rows whose buttons carry the incomplete
   offers with `fill = { proposal: <id> }`; both clients already draw `answer` rows first, with any number of
   buttons.
3. `intents` maps `proposal` from `fill` into the payload, as it maps `item` and `shop`.

## 6. The scan that holds the boundary

`tests/acceptance/tests/client_rules.rs` check **e**, over every client script it reads (2D, 3D, the shared
modules), comments removed:

- **e1, money compared:** a top-level function that holds a comparison token (`<`, `>`, `<=`, `>=`; not
  `->`, `<<`, `>>`, nor characters inside strings) together with a wallet word (`balance`, `wallet`,
  `wealth`) and a price word (`price`, `amount`, `ask`, `cost`), as identifiers or string literals.
- **e2, reference read:** any string literal `"references"` (the `currency` record's field) or
  `"reference-value"` (economy's component, `ARC-113`). A bare `"reference"` is not matched: the 3D client's
  cast names a reference body with it.
- `MONEY_ADMITTED` admits a (file, function) with a reason; it is empty, and an admission that admits nothing
  fails.

## 7. Reports

The harnesses of both clients print, for the real-client test `tools/cli/tests/client_trade.rs`:

```text
TRADE {subject, rows: MineWorldTrade.report(rows), t_ms}
BOARD {notices: [{id, topic, publisher, lines}], badge, t_ms}
HUD   {money, tags: [{subject, text}], t_ms}
```

Steps name ids and typed text the test resolves from genesis; no script names a world's item, person or
topic, and no selector matches a label's text.
