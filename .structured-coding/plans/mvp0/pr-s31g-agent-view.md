# PR S31-g — The agent (MCP) view of trade and notices: prices, wallet, asks, notices, and the `act` kinds that change them

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-19), FROZEN 2026-10-20
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31g, branch docs/pr-design-s31g,
                        planning base origin/main @ 44f050e3 (#171 12d merged; #261 ARC-153 merged; #239 S27-d
                        merged; S31-a and S31-b DESIGN FROZEN, neither merged)
Approved by / evidence: the operator, 2026-10-20, primary session (§14.1). The step's rulings (Q-S31-1 … Q-S31-8, step-31 §18), S31-a's Q-S31a-1 … 4,
                        S31-b's Q-S31b-1 … 4 and ARC-153 bind and are not re-asked. §14 holds this PR's questions
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S31-a, S31-b,
                        S31-c, S31-h, S27-f and S28-d
Start condition:        step-31 §16.5: "Nothing starts before Milestone F's checklist has been handed to the
                        operator (the S29 timing rule). Each PR starts only once its own PR design is frozen."
                        S31-g is second wave: "S31-g once S27-d has merged". Its PRs, at 2026-10-19:
                          S27-d (#239)  MERGED 2026-10-18 (42bc3ceb)
                          ARC-153 (#261) MERGED 2026-10-19 (94183574) — one `act` tool
                          S31-a          FROZEN, not merged (impl-s31a worktree at the base, no commit)
                          S31-b          FROZEN, not merged (impl-s31b: C0, C1 committed; pack code in progress)
                        Then this design frozen, then the primary session's dispatch
Execution contract:     §12 (filled; ends at MERGE-READY — the coordinator merges; charter "SPEED FIRST")
Implementer:            Haiku-class (step-31 §16.2, the S31-g row); every commit is written as numbered steps
                        with exact paths and commands
Lifecycle:              FROZEN; implementing (impl-s31g, branch mvp0/pr-s31g)
```

**Effort:** `mvp0`, release-blocking (Q-S31-1 re-ruled) · **Step:**
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15) §§0.2 ("Agents are first-class
players", "Information-rich observations"), 1 (outcome 10), 4.1, 5.5, 5.6, 7.1, 7.3, 8.5, 8.6, 9.3, **11 in full**,
13, 14 (I-S31-9, I-S31-11, I-S31-12), 15 (**AC-S31-8**, AC-S31-6's agent half; **X-9 through the agent**), 16.2 (the
S31-g row), 16.4 (S27, S28), 16.5 · **Parent:** [`overall.md`](overall.md) · **Consumes:**
[`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (FROZEN; R-S31g-a-1),
[`pr-s31b-notices.md`](pr-s31b-notices.md) (FROZEN; R-S31g-b-1), [`pr-s27d-mcp-stdio.md`](pr-s27d-mcp-stdio.md)
(MERGED), [`pr-s27d1-single-act.md`](pr-s27d1-single-act.md) (MERGED, `ARC-153`) · **Coordinates with:**
`pr-s31h-haggling.md` (DRAFT, `design-s31h` @ 4d5a81f6: its §5.14 and Q-S31h-3), `pr-s31c-price-survey.md`
(DRAFT, `design-s31c` @ 9c4b5296: R-S31g-c-1), [`pr-s27f-agent-guide-reference.md`](pr-s27f-agent-guide-reference.md)
(FROZEN, not merged: the generated agent reference), step-28 S28-d (not designed: `Rendered.anchors`) · **Binding
parents:** `CLAUDE.md` §§1.1, 2, 3, 4 (items 3, 7, 8, 9, 10, 12, 13, 15, 16); `docs/CORE_CONCEPTS.md` §§12, 15;
step-27 INV-S27-1 … 13; decision records `ARC-34`, `ARC-96`, `ARC-111` … `ARC-115` (as S31-a and S31-b write them),
`ARC-153`.

**Decision ids.** None new. The step assigned no id to S31-g, and `ARC-188` (the sub-range offered) is **not
used**: the one rule this PR adds — "an action an agent must send with its own payload is one typed kind of the
`act` tool" — is a later note on `ARC-153`, written as `## ARC-153 note — …` inside `docs/decisions/ARC-153.md`
(C1). S35 N-e (frozen) adds a `sleep` kind by the same rule; whichever of the two lands first writes the note, and
the second cites it (C0 checks).

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-31 §16.2, the S31-g row, verbatim).** "§11 surfaces and tools; the AC-S31-8 scenario over MCP".
  Depends on "S31-a, S31-b; **S27-d merged**". Checkpoint "AC-S31-8 over MCP". Adversarial "X-9 (through the
  agent)". Implementer **Haiku**.
- **Step §11 (the surfaces).** `WorldLine.currency`; `MeLine` `wallet`, `wealth { amount, unvalued }`, `asks`,
  `subscriptions`; `here.people[].asks`; `offers[].price` also on person-to-person buys, `unavailable` with the
  named code; `View.notices` (current notices, each with `valid_until`); the inbox reading `notice-published`,
  `sold`, `price-set` by name; the tools `list_for_sale`, `set_price`, `withdraw`, `publish_notice`, `subscribe`,
  `unsubscribe`; "every amount is shown as minor units and as `display` (formatted from the disclosed
  currency)". "What an agent never sees: another person's wallet, wealth, holdings counts, subscriptions or price
  book."
- **AC-S31-8 (verbatim).** "An agent seat in Market Town sees the café's listed prices with currency display, sees
  Grace's asks when beside her, and buys from both. `inspect` on another person discloses no wallet, wealth,
  holdings counts, subscriptions or price book."
- **X-9 (verbatim).** "economy discloses `Wallet` to any observer" must fail AC-S31-8.
- **R-S31g-a-1 (S31-a §10).** "`OfferLine.price` for person-seller buys comes from the payload's `price`; the economy
  S28 renderer, if S28-d has merged." SD-S31a-9: "Economy's S28 renderer deferred to S31-g."
- **R-S31g-b-1 (S31-b §10).** "`View.notices` reads the `notice-board` record; `MeLine.subscriptions` the
  `subscriptions` record; the inbox reads `notice-published` by name; the typed tools submit §5.6's payloads."
- **ARC-153 (merged 2026-10-19).** "The tool list is nine tools … `act`'s `inputSchema` is `ActArgs` (`handle`, and
  `act`: … by `kind`) … Not reversed: QS27-9 … refused a *generic* `act(action_type, target, payload)` tool."
- **Order with haggling (step-31 §16.5).** "If they [S31-e, S31-g] merge first, S31-h adds its own sale-panel rows
  and agent tools. If S31-h merges first, S31-e and S31-g include them." The lane lead's task: define the economy
  act kinds once, the same way for both PRs, and state the interface (§5.12).
- **Operator principles (2026-10-18).** One currency per world; any seller sets its own price; a price clerk
  publishes suggestions; notices reach subscribers' inboxes; **outside agents (MCP) play by the same server path
  and rules**. The adapter invents no state.
- **Charter "SPEED FIRST" (operator, 2026-10-18).** The contract ends at MERGE-READY and the coordinator merges.
  Per-PR validation is the build, the tests each commit writes, and `fast`; the step's acceptance criteria and
  planted mutations run **at the milestone**. No world-specific hard-coding, and **every acceptance criterion
  names at least one second world or a generated variant** besides Market Town (§8).
- **R-LOAD-1.** Locally: only the commit's own test files, one at a time, plus ruff, ruff format and pyright on
  the member. No local whole-member suite. The real binary is built through `cast-slots/with-slot.sh`.

---

## 1. Goal

An agent playing a seat over MCP sees the market and the notice board as a person in that seat does, and can do
what that person can do there.

- It sees the world's **currency** once per view, and every **amount** both as minor units (what the world stores
  and what a request carries) and as a decimal in major units formatted with the disclosed exponent.
- It sees its **wallet**, its **wealth**, its own **asks**, the **asks of the sellers it perceives**, the **price of
  every buy offer** (a shop's or a person's), and the **name** of each item kind involved.
- It sees the **current notices** of its topics, with their expiry, and its **subscriptions**.
- Its inbox reads **sales, ask changes, notices and subscription changes** by name, as it reads `spoke`.
- It can **list, re-price and withdraw** an ask (its own or, as a keeper, its shop's), **publish** a notice,
  **subscribe** and **unsubscribe**: six new typed **kinds of the one `act` tool** (`ARC-153`). The tool list
  stays nine tools.

Every one of these is a rendering of the seat's own observation or a request through the server's ordinary path
(INV-S27-1, INV-S27-2). The adapter computes no price, filters no notice and checks no rule. Example: Grace asks
4.50 for flowers. The agent standing 2 m from her sees `{"item":"E40","name":{"text":"Flowers","kind":"name"},
"price":{"minor":450,"display":"4.50"},"available":true}` on her person line, and a buy offer with the same price.
It buys with `act {kind: attempt}`; the world settles the sale; the agent's wallet drops by exactly 450, and its
`accepted` outcome shows the `sold` fact read as a sale at 4.50.

---

## 2. Scope

### 2.1 In scope

1. **`clients/agent/src/mineworld_agent/`** (the only package whose code changes for the agent):
   - `models.py`: the new lines and readings (§5.3, §5.5), the six act bodies, their `*Args` and `*Act` models
     (§5.6), `Amount`, `TopicName`; `MeLine.wallet` and `OfferLine.price` become `Amount` (§5.4).
   - `lexicon.py` (new): the per-handle `Lexicon` — the currency, item names and reference values the seat was
     disclosed — and `display_of(minor, exponent)` (§5.2).
   - `disclosed.py`: readers for `currency`, `wealth`, `asks`, `subscriptions`, `notice-board`, `item-catalogue`;
     `buy_price` (§5.3).
   - `view.py`: `render` fills the new lines; `offer_line` reads a buy's price from its payload; paging of
     `notices` and `me.asks`; deltas (§5.4).
   - `inbox.py`: by-name readings (§5.5); `Inbox` takes the handle's `Lexicon`.
   - `handle.py`: owns the `Lexicon` and passes it to `render` and `Inbox`.
   - `player.py`: six act operations and their payload builders (§5.6).
   - `mcp_server.py`: the six kinds in `ACTS`; `ACT_INTRO` without a count (§5.7).
   - `__init__.py`: exports.
2. **`clients/agent/tests/`**: new `test_market_view.py`, `test_notices_view.py`, `test_market_acts.py`,
   `test_market_inbox.py` (unit, planted records), `test_play_market_mcp.py` (real binary, §8); updated
   `test_info.py`, `test_play_market_town.py`, `test_play_mcp.py` (the `Amount` shape), `test_mcp_tools.py`,
   `mcp_double.py`, `test_mcp_secrets.py`, `test_world_text.py`, `test_descriptions.py` (the new kinds and models),
   `golden/tools.json` (regenerated, reviewed).
3. **`cognition/lm-controller`**: `src/mineworld_cognition/memory/render/economy.py` (new) and its registry lines —
   the economy S28 renderer S31-a deferred here (SD-S31a-9; §5.9) — with cases in `tests/test_memory_render.py`.
4. **Documentation:** the `ARC-153` note (§0 "Decision ids"); `clients/agent/README.md` (one line: the market and
   notice lines); `docs/reference/agent-tools.{json,md}` regenerated **only if S27-f has merged at C0**.

### 2.2 Conditional (whichever merges second; C0 decides)

| Rows | Here when | Otherwise |
| --- | --- | --- |
| Haggling: `MeLine.proposals`, four `act` kinds, four facts read by name (§5.12) | S31-h merged before this PR's C0 | S31-h implements §5.12 exactly (R-S31h-g-1) |
| The price book: `me.survey` and the paged list `notes` (§5.13) | S31-c merged before C0 **and** Q-S31g-4 answered yes | S31-c implements §5.13 exactly (R-S31c-g-1), or nobody if Q-S31g-4 is no |
| Topic catalogue: `View.topics` (§5.14) | Q-S31g-3 answered yes **and** its record merged before C0 | the PR that adds the record adds the line (R-S31b-g-2) |

### 2.3 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The 2D and 3D sale panel, price tags, stall, notice board, refusal words | S31-e |
| An LM seat attempting an incomplete offer by name (the brief's material finding) | S31-d. **S31-g does not need it:** the MCP adapter already attempts an incomplete offer by name through `mineworld_sdk.offers.request` (`say`, `walk_to`, `move` do, `player.py:534-543`) |
| Market Town content (keepers, person asks, topics, the reporter) | S31-f |
| `Rendered.anchors` on economy's renderer | S28-d or the first PR after it (R-S28d-g-1) |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/`, `worlds/`, `sdk/` | never in this PR (I-g-1). X-9's plant in `systems/economy` is a test-only edit, never committed |

---

## 3. Audit (`origin/main @ 44f050e3`, 2026-10-19)

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | The MCP surface is nine tools; `ACTS` binds each `act` kind to one `Player` operation and its `*Args` model; the `act` handler builds the `*Args` from `handle` and the kind's fields, generically | `clients/agent/src/mineworld_agent/mcp_server.py:124-160, 381-388` | A new kind is one `ACTS` entry plus models and a `Player` operation; the handler does not change |
| A-2 | `ACT_INTRO` says "one call, five kinds"; `ActArgs` and its field docstrings name the five | `mcp_server.py:155`; `models.py:756-766` | Rewritten without a count (§5.7) |
| A-3 | `Player._request(handle, action, target, payload)` sends through `offers.request`, which refuses locally (`not_offered`) unless the newest observation offers that action on that target as available | `player.py:534-543`; `sdk/python/src/mineworld_sdk/offers.py:55-82` | The six kinds reach incomplete offers by name with no new mechanism (INV-S27-3) |
| A-4 | S31-a's seller offers and S31-b's offers are incomplete, one per action type, target `None` | `pr-s31a-currency-asks.md` §5.6; `impl-s31b/systems/notices/src/system.rs:274-300` (`Offer::new`) | `offers.request(…, target=None, …)` matches them |
| A-5 | `MeLine { …, wallet: int \| None, holdings, job, agenda, acquaintances, invitations }`; `OfferLine { …, about, price: int \| None }`; `ItemCount.name` "is `None` until a pack discloses item names" | `models.py:224-231, 289-309, 358-372` | `wallet` and `price` become `Amount`; `name` gets filled (Q-S31g-2) |
| A-6 | `offer_line` takes `price` from the place's `shop` listing for any complete offer that names exactly one listed item | `view.py:217-234`; `disclosed.py:213-224` | After S31-a a person-seller buy has no listing entry, and a `give` shows a shop price: both wrong (F-S31g-4) |
| A-7 | `render` builds `MeLine` from per-pack readers in `disclosed.py`; lists of `me` are paged by keeping them apart in `Rendered` (`acquaintances`, `invitations`); `delta_of` answers a whole view when a paged list of `me` changed | `view.py:96-114, 255-337, 343-500`; `models.py:158` | `me.asks` and `notices` page the same way |
| A-8 | The inbox reads `spoke` and `walk-ended` by name; every other fact shows type and participants; `detail` adds the raw payload | `inbox.py:1-12, 128-144` | §5.5 adds the market and notice readings beside `words` |
| A-9 | `WorldLine` is built from public `GET /status` (composition only) by `seats` | `player.py:214-230`; `models.py:465-474` | The step's `WorldLine.currency` cannot come from there (F-S31g-2) |
| A-10 | Every string outside `WorldText` must carry a pattern, enum or const, except a closed `FREE_TEXT` list; exactly 23 operation models | `tests/test_world_text.py:42-95` | The currency's name and symbol, item names and notice text are `WorldText`; the count becomes 29 |
| A-11 | Every model and field has a schema description except `UNDOCUMENTED`, which may only shrink | `tests/test_descriptions.py` | Every new field is documented |
| A-12 | The item pack (12d, merged in #171) discloses `item-catalogue { kinds: [{ item, category, name }] }` on a place to whoever perceives it; names ≤ 64 bytes | `systems/item/src/component.rs:70-111`; `system.rs:89-110`; `name.rs:7` | Item names are on the wire now; F-S27c-9 can close (Q-S31g-2) |
| A-13 | Typed ids serialize as `{ entity, entity_type }`; `EntityId` as an opaque string; `WorldTime` as integer seconds | `contracts/src/ids.rs:705-735, 250-290`; `contracts/src/time.rs:28-29` | Readers accept a typed reference **or** a bare id string, as `inbox._entity` already does (`inbox.py:55-61`) |
| A-14 | S31-b in progress: `Subscriptions` disclosed as `subscriptions { topics }`; `notice-board { notices: [Notice { id, topic, publisher, body, issued_at, valid_until }] }`; `NoticeBody` externally tagged `{"prices":[…]}` / `{"text":"…"}`; `PriceLine.item` is an `EntityId` in the code, an `ItemId` in the design | `impl-s31b/systems/notices/src/{component.rs:14-75, board.rs:40-104, body.rs:22-80, action.rs:15-101, event.rs:166-273}`; `pr-s31b-notices.md` §5.5 | C0 pins the merged shape; the payload builder sends what the merged pack decodes (F-S31g-6) |
| A-15 | `TopicId` is dotted segments of `[a-z0-9-]`, ≤ 64 bytes | `impl-s31b/systems/notices/src/topic.rs:27-70` | `TopicName` uses the same pattern |
| A-16 | No record discloses the world's topics: `notices-configured` is `SystemInternal`; the board lists only current notices | `pr-s31b-notices.md` §5.8; `event.rs:82-104` | An unsubscribed agent cannot learn a topic's name (F-S31g-7, Q-S31g-3) |
| A-17 | The agent fixtures predate S31-a and 12d (no `item-catalogue`, `buy { item }` without price); S31-a C6 re-records them | `tests/fixtures/SOURCE.md`; `pr-s31a-currency-asks.md` C6 step 4 | C0 checks they were re-recorded; new records are planted in the shapes the merged packs serialize (the `test_info.py` precedent, lines 1-10) |
| A-18 | The real-binary harness runs a scratch world by absolute path, and a plain SDK client can drive another seat | `tests/realserver.py:47-50`; `tests/test_mcp_same_answers.py:45-60` | §8's worlds are scratch copies built in the test |
| A-19 | `mineworld server` drives no seat unless `--town` or `--agent` is given | `docs/reference/cli.md:21-22` | Grace stands where her copied file puts her: no timing hazard |
| A-20 | The S28 renderer registry maps event type → pure renderer; no economy renderer exists; no `anchors` field | `cognition/lm-controller/src/mineworld_cognition/memory/render/__init__.py:112-145`; F-S31a-18 | §5.9 adds one module and four registry lines |
| A-21 | S35 N-e (frozen) adds an `act` kind `sleep` "if #261 is merged" | `pr-s35-ne-presentation.md` §4.6 | Precedent: kinds are added per action; the `ARC-153` note records the rule once |

### 3.2 Findings

- **F-S31g-1 — The step's "typed tools" are typed kinds of `act` (against step §11, the "tools" row).** `ARC-153`
  made the surface one `act` tool, and its own record says the kinds are closed. Six new tools would reverse that
  ruling. Six new **kinds** keep it: each is typed, so QS27-9's refusal of a free-form act stands. S35 N-e already
  adds a kind the same way. → **Q-S31g-1** (one answer also settles S31-h's Q-S31h-3).
- **F-S31g-2 — The currency cannot live on `WorldLine` (against step §11, the `WorldLine` row).** `WorldLine` is
  the public `GET /status` (A-9): composition only, the same for every visitor, served by the server, which knows
  nothing of money (kernel ignorance). The currency is disclosed in each seat's observation (S31-a §5.3). So it is
  `View.currency`. Bounded: the information is the same, only its place differs (SD-S31g-1).
- **F-S31g-3 — A `display` string with the symbol in it would carry world text outside `WorldText`.** The symbol
  is written by a world author (1 … 8 bytes). INV-S27-12 allows world text only in `WorldText`. So `display` is
  the decimal alone (`"4.50"`), constrained by a pattern, and the symbol and its placement sit once in
  `View.currency` as `WorldText`. The exponent still drives every `display` (SD-S31g-2).
- **F-S31g-4 — S27-c's offer price is wrong after S31-a.** It is looked up in the place's shop listing (A-6). A
  buy from Grace, standing in the café, would show the café's price for the kind; a `give` of coffee in the café
  shows 300 today. After S31-a every `buy` payload carries its own `price`. So `OfferLine.price` is the price of
  a `buy` payload, read by name, and nothing else carries one (SD-S31g-3; R-S31g-a-1).
- **F-S31g-5 — Item names are on the wire now (A-12).** F-S27c-9 ("item names not disclosed by any pack") was
  written before 12d merged. An agent that sees `about: ["E31"]` and a price cannot tell coffee from bread, so
  it cannot shop. → **Q-S31g-2**.
- **F-S31g-6 — S31-b's price-line shape differs between its design and its code (A-14).** The design says
  `item: ItemId`, the code in progress an `EntityId`. The adapter must send exactly what the merged pack decodes,
  or every prices notice is refused `PreconditionFailed`. C0 pins it from merged code; the real-binary arm
  (AC-g-5) proves it. Recorded for the lane lead.
- **F-S31g-7 — Nobody can discover a topic (A-16).** An agent (or a 2D player) that subscribes to nothing never
  sees a topic's name; a publisher with no current notice does not see its own topics either. `subscribe` is
  offered, but no record says to what. This is S31-b's disclosure, not the adapter's. → **Q-S31g-3**.
- **F-S31g-8 — The economy renderer's condition is stated two ways.** SD-S31a-9 defers it to S31-g outright;
  R-S31g-a-1 says "if S28-d has merged"; the step says "filed with S31-g if S28-d is later". Reading: S31-g writes
  the renderer now (gists), as S31-b did for notices, and anchors join when S28-d exists (SD-S31g-9).
- **F-S31g-9 — A keeper cannot tell from its offers whether it may act for the shop.** S31-a offers one
  `set-price` (target `None`) whether the ask is the person's or its shop's, and no record names the keepers. An
  agent therefore sends `for_shop: true` and learns `PermissionDenied` if it is not a keeper. That is honest and
  the same for a human client; recorded, no change here.
- **F-S31g-10 — A buy at a moved price is caught locally first.** `attempt` re-locates an offer by its payload
  (`view.py:506-529`), and a buy's payload holds the price. When the newest observation already shows the new
  price, the adapter answers `stale_offer` with the fresh view; only when it does not yet is the request sent and
  refused `price-changed` by the server. Both are honest, and neither pays the new price (AC-g-3).

### 3.3 What does not exist

No agent line for a currency, an amount, wealth, an ask, a notice, a subscription or an item name; no reading of
any economy or notices fact; no `act` kind beyond the five of S27-d; no economy memory renderer; no test world with
a person seller or a notice topic.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| Money formatting | `babel.numbers`, `py-moneyed` | **Not adopted.** The adapter needs one integer-to-decimal function over a disclosed exponent (0 … 4), with no locale and no symbol (F-S31g-3). A library would bring locale tables a fantasy currency cannot use (`DEP-63`'s reasoning). Below the `DEP` threshold |
| MCP tool shape | one tool per action; a generic `request` kind | Declined: `ARC-153` and QS27-9 (§5.7, Q-S31g-1) |
| Reference tables (`P1`, `E1`) | the existing `RefTable` | **Reused** for entities; proposals get the same pattern only if the haggling rows land here (§5.12) |

---

## 5. Design

### 5.1 Terms, from plain to exact

| Term | Plain | Exact |
| --- | --- | --- |
| **minor units** | cents, for a dollar | the integer economy stores and a request carries |
| **display** | "4.50" | `minor` written in major units with the disclosed exponent, no symbol: `"4.50"` for 450 at exponent 2, `"450"` at exponent 0, `"0.450"` at exponent 3 |
| **lexicon** | what this seat has learned about money and item names | per handle: the newest disclosed currency, and every item name and reference value its observations disclosed, kept by item id |
| **reading** | the agent's typed copy of a fact's payload | `EventLine.reading`, for the fact types this library reads by name |
| **kind** | one thing the `act` tool can do | one variant of the `Act` union, bound to one `Player` operation (`ARC-153`) |

### 5.2 The lexicon and `display`

```text
Lexicon (per handle; module lexicon.py)
  currency   CurrencyLine | None        the newest `currency` record on me
  exponent   int | None                 currency.exponent
  names      dict[EntityId, str]        from `item-catalogue` on my place; a name, once disclosed, is kept
  references dict[EntityId, int]        from currency.references; kept the same way
  learn(observation) -> None            called by `render` before any line is built
display_of(minor: int, exponent: int | None) -> str | None
  None when exponent is None; str(minor) when exponent = 0;
  f"{minor // 10**e}.{minor % 10**e:0{e}d}" otherwise
amount(minor) -> Amount                 Amount(minor=minor, display=display_of(minor, exponent))
```

- **Why kept.** A name or a reference value never changes in a run (configuration), and the catalogue is disclosed
  only on a place. A fact read while the seat is between places still gets names. This is the seat's own past
  perception, like `RefTable` (INV-S27-2 holds).
- **Nothing is invented.** No disclosed currency → every `display` is `None`; no disclosed name → `name` is `None`.
- **No locale, no separators.** The decimal is for a program to read. A client's formatter is S31-e's.

### 5.3 What is read, from where (`disclosed.py`)

| Record (component type) | Pack, audience | Reader → line |
| --- | --- | --- |
| `currency { name, symbol, exponent, symbol_at, references }` | economy; about me, every living person | `currency(me) -> CurrencyLine \| None`; feeds the lexicon |
| `wallet { balance }` | economy; holder | `wallet(me, lexicon) -> Amount \| None` |
| `wealth { amount, unvalued }` | economy; holder | `wealth(me, lexicon) -> WealthLine \| None` |
| `asks { asks: [{ item, price, available }] }` (S31-a's `AsksView`; C0 pins its component type) | economy; whoever perceives the seller | `asks(refs, entity, lexicon) -> list[AskLine]`, for me and for each person |
| `shop { operator, listed }` | economy; the place | no longer read for prices (F-S31g-4) |
| `subscriptions { topics }` | notices; holder | `subscriptions(me) -> list[TopicName]` |
| `notice-board { notices }` | notices; the observer | `notices(refs, me, lexicon) -> list[NoticeLine]` |
| `item-catalogue { kinds }` | item; the place | `item_names(place) -> dict[EntityId, str]`; feeds the lexicon |

Each reader follows the module's rule (`disclosed.py:1-7`): an unknown shape, a missing record or an absent pack
gives `None` or nothing. An id is read with one helper, `any_entity(value)`, that accepts `{ entity, entity_type }`
or a bare id string (A-13).

### 5.4 The view (`models.py`, `view.py`)

```text
TopicName  = str, pattern ^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)*$, ≤ 64   (A-15)
Amount       { minor: int ≥ 0, display: str (pattern ^[0-9]+(\.[0-9]+)?$) | None }
CurrencyLine { name: WorldText(kind=name), symbol: WorldText(kind=label), exponent: int, symbol_at: before|after }
WealthLine   { amount: Amount, unvalued: int }
AskLine      { item: Ref, name: WorldText | None, price: Amount, available: bool, reference: Amount | None }
NoticePriceLine { item: Ref, name: WorldText | None, price: Amount, basis: int }
NoticeLine   { topic: TopicName, publisher: Ref, issued_at: int, valid_until: int,
               prices: [NoticePriceLine] = [], text: WorldText(kind=speech, by=publisher) | None = None }

ItemCount   name now filled from the lexicon (Q-S31g-2); + reference: Amount | None = None (holdings only)
MeLine      wallet: Amount | None                     (was int | None)
            + wealth: WealthLine | None = None
            + asks: [AskLine] = []                    paged as list `asks` (like acquaintances)
            + subscriptions: [TopicName] = []
PersonLine  + asks: [AskLine] = []                    at most ASKS_PER_PERSON = 8, in item order
            + asks_total: int = 0                     how many the seller asks; the rest are its buy offers
OfferLine   price: Amount | None                      (was int | None) — a `buy` payload's `price` only
            + item_name: WorldText | None = None      the one item a `buy` names
            + reference: Amount | None = None         that item's reference value
View        + currency: CurrencyLine | None = None
            + notices: [NoticeLine] = []              paged as list `notices`
ViewDelta   + currency: CurrencyLine | None = None    None when unchanged
            + notices: [NoticeLine] | None = None     the whole list when it changed, else None
ListName    + "asks", "notices"
Caps        + asks = 24, notices = 8
```

- **Rendering rules.**
  - `View.notices` is the board record, unchanged in content and order. The adapter compares no `valid_until`
    with any time: the record is already current (S31-b §5.3). M-g-3 plants that filter.
  - A notice's `text` is world text written by its publisher: `WorldText(kind="speech", by=<publisher ref>)`.
  - `OfferLine.price`, `item_name`, `reference` are set only for `action_type == "buy"` with an integer `price`
    and one item in its payload (economy's `buy`, read by name: `disclosed.buy_price`). Every other offer has
    `None`. `about` is unchanged.
  - `PersonLine.asks` shows at most 8 asks, `asks_total` says how many exist. Every ask of a perceived seller is
    also a buy offer (S31-a §5.6), and offers are paged, so nothing is out of reach (SD-S31g-5).
  - A delta answers a whole view when `me.asks` changed (as for `acquaintances`); `currency` and `notices` are
    carried like `calendar` and `offers`.
- **Token cost.** An `Amount` is about 30 bytes. The fitting loop (`view.py:428-439`) shortens the largest shown
  list one entry at a time, `asks` and `notices` included. One 32-line prices notice with 64-byte names is about
  5 KB and still fits a page (R-S31g-2).

### 5.5 The inbox: readings by name (`inbox.py`)

`EventLine` gains one field, `reading: Reading | None = None`. One field rather than five keeps every other
line's JSON small (a `None` field costs its key in `structuredContent`).

```text
Reading = SaleReading | AskReading | NoticeReading | TopicReading          discriminated by `kind`
  SaleReading    { kind: "sale",  seller: Ref, buyer: Ref, item: Ref, name, count: int, price: Amount, place: Ref | None }
                   ← sold { seller, buyer, item, count, price, place }
  AskReading     { kind: "ask",   seller: Ref, item: Ref, name, price: Amount | None, was: Amount | None }
                   ← listed { seller, item, price }        price = price, was = None
                   ← price-set { seller, item, from, to }  price = to,    was = from
                   ← withdrawn { seller, item }            price = None,  was = None
  NoticeReading  NoticeLine + { kind: "notice" }           ← notice-published { notice }
  TopicReading   { kind: "topic", topic: TopicName }       ← subscribed / unsubscribed { person, topic }
```

- `event_type` still says which fact it was (`listed`, `price-set`, `withdrawn`; `subscribed`, `unsubscribed`).
- A payload that does not decode gives `reading: None`; the fact is still listed (`inbox.py`'s rule).
- `words` is unchanged (`spoke`). `money-transferred` and `items-transferred` are not read: the sale's reading
  holds the price, and their participants are shown as today.

### 5.6 The six act kinds (`models.py`, `player.py`)

| Kind | Body (fields after `kind`) | `Player` operation | Request sent through `offers.request` |
| --- | --- | --- | --- |
| `list_for_sale` | `item: Ref`, `price: int 0 … 2⁶⁴−1`, `for_shop: bool = false` | `list_for_sale(ListForSaleArgs)` | `list-for-sale`, target `None`, `{ item: <ItemId>, price, for?: "shop" }` |
| `set_price` | `item: Ref`, `price`, `for_shop` | `set_price(SetPriceArgs)` | `set-price`, target `None`, the same shape |
| `withdraw` | `item: Ref`, `for_shop` | `withdraw(WithdrawArgs)` | `withdraw`, target `None`, `{ item, for?: "shop" }` |
| `publish_notice` | `topic: TopicName`, `valid_for_s: int 0 … 2³²−1`, `body: TextBody \| PricesBody` | `publish_notice(PublishNoticeArgs)` | `publish-notice`, target `None`, `{ topic, body: {"text": …} \| {"prices": [{ item, price, basis }]}, valid_for }` |
| `subscribe` | `topic: TopicName` | `subscribe(SubscribeArgs)` | `subscribe`, target `None`, `{ topic }` |
| `unsubscribe` | `topic: TopicName` | `unsubscribe(UnsubscribeArgs)` | `unsubscribe`, target `None`, `{ topic }` |

```text
TextBody   { kind: "text",   text: str (1 … 280 UTF-8 bytes) }               notices' TEXT_MAX_BYTES
PricesBody { kind: "prices", lines: [PriceArg] (1 … 32) }                    notices' PRICE_LINES_MAX
PriceArg   { item: Ref, price: int 0 … 2⁶⁴−1, basis: int 0 … 65 535 }
```

- **Each body is defined once** (`ListForSale`, …), then `ListForSaleArgs(ListForSale, Seated)` for the library
  and `ListForSaleAct(ListForSale)` with `kind` for the tool — `ARC-153`'s pattern (`models.py:648-766`).
- **What the adapter does, step by step** (one helper `_market(handle, action, payload_from_refs)`):
  1. resolve every `Ref` (`item`, each line's `item`) through `handle.refs`; an unknown one answers the outcome
     `refused { unknown_ref }`, as `say` does;
  2. build the payload in the pack's own shape (item as a typed `ItemId`, `for` only when `for_shop`; the
     price-line item in the shape C0 pinned, F-S31g-6);
  3. **sort price lines by item id.** The agent cannot see ids, so it cannot sort; sorting is encoding, not a
     rule. Duplicates stay and the world refuses them (`body-invalid`) (SD-S31g-7);
  4. `self._request(handle, ACTION, None, payload)` (A-3).
- **Bounds.** Numbers are bounded by their wire type only, so the world answers its own rules by name
  (`price-out-of-range`, `validity-out-of-range`) to an agent as to anyone. Text length and line count copy the
  pack's two size bounds, as `say` copies the conversation pack's (`models.py:49-50, 656-661`), so an agent learns
  them from the schema (SD-S31g-6).
- **Descriptions** (they become the `act` description's paragraphs): each operation's docstring says what it
  does, that amounts are minor units (with the exponent example), and the refusal codes the world may answer:
  `list_for_sale` — `already-listed` (use `set_price`), `not-held`, `price-out-of-range`, `PermissionDenied`
  (not a keeper); `set_price` — `not-for-sale`, `price-unchanged`; `withdraw` — `not-for-sale`;
  `publish_notice` — `unknown-topic`, `PermissionDenied` (not a publisher), `body-mismatch`, `body-too-large`,
  `body-invalid`, `validity-out-of-range`, `over-daily-limit`; `subscribe` — `unknown-topic`,
  `already-subscribed`; `unsubscribe` — `not-subscribed`.

### 5.7 The MCP surface (`mcp_server.py`)

- `ACTS` gains the six rows; the tool list stays the nine tools of `ARC-153` (I-d-4 unchanged).
- `ACT_INTRO` becomes: "Act in the world: one call; `act.kind` chooses what to do. Each kind is:". `ActArgs`'s
  docstrings list no kinds by count; the union's schema lists them.
- **No generic act.** Action payload schemas are not in the generated reference (`docs/reference/schema/` has
  sections and configurations only), so QS27-9's condition is still unmet. Each action is its own typed kind.
- `golden/tools.json` is regenerated (`main(["schema", "--out", …])`) and read whole. The before and after
  `tools/list` byte sizes are recorded (`catalogue_json(compact=True)`, ARC-153's measure).

### 5.8 Spatial requirements, rules and refusals

The adapter evaluates no requirement. Every requirement and refusal is the owning pack's, shown unevaluated on
offers and answered on requests:

- `buy` from a shop: `at_place(here)`; from a person: `same_place().within(3 000 mm)` (S31-a §5.5). Shown in
  `OfferLine.requirement`; `TooFarAway` comes from the server.
- The six kinds: `NONE` for oneself and for notices; `at_place(here)` for `for: shop` (S31-a SD-S31a-3).
- An act's refusal is data in `ActResult.outcome` (`rejected { reason, system_code }`) — the named code
  (`not-for-sale`, `price-changed`, …) arrives in `system_code` with no adapter mapping.

### 5.9 Economy's memory renderer (`cognition/lm-controller`)

`memory/render/economy.py`, four pure renderers in S31-b's pattern (`pr-s31b-notices.md` §5.9):

```text
sold, I am the buyer:   "I bought <item> from <seller> for <price>"
sold, I am the seller:  "I sold <item> to <buyer> for <price>"
sold, otherwise:        "<buyer> bought <item> from <seller> for <price>"
listed:                 "<seller> put <item> on sale at <price>"
price-set:              "<seller> changed <item> from <from> to <to>"
withdrawn:              "<seller> took <item> off sale"
item:  names.name(item) or "item <id>";  amounts in minor units (memory reads no currency, as SD-S31b-11)
```

- `salience` `notable`; `mine` = I am buyer or seller; importance `SALE = 400` with me, `300` without (between the
  generic 300 and 500; a structural constant beside `pr-fg-b` §5.4's table, never fitted, `ARC-23`).
- A payload that does not decode → `fallback`. No `anchors` until S28-d (R-S28d-g-1).
- `tests/golden/s28a_renders.json` must not move (its synthetic stream has no economy fact); C6 checks it.

### 5.10 Ownership, persistence, determinism

- The adapter owns no world state. The lexicon, references and pages are presentation caches of one handle,
  rebuilt from observations; nothing is persisted but S27-c's resume secret and cursor.
- Every list keeps the record's order; `display` is integer arithmetic; no float, no clock read for a rule.

### 5.11 Both clients

No client changes here. S31-e reads the same records (R-S31e-a-1, R-S31e-b-1). The gate questions (`CLAUDE.md`
§4): a Minecraft-like 3D client needs nothing new from this PR, and the 2D and 3D clients duplicate no logic of it
(the adapter adds none). Yes and yes.

### 5.12 The haggling kinds — the one interface for S31-g and S31-h (conditional, §2.2)

Whichever PR merges second implements exactly this; the other PR cites it.

```text
ProposalRef = str, pattern ^P[1-9][0-9]{0,8}$      per handle, first-seen order, like E-refs and F-refs
ProposalLine { proposal: ProposalRef, seller: Ref, buyer: Ref, item: Ref, name: WorldText | None,
               price: Amount, ask: Amount | None, valid_until: int, status: open | declined, for_shop: bool }
MeLine + proposals: { incoming: [ProposalLine], outgoing: [ProposalLine] } | None     ← S31-h's `proposals` record
kinds:
  propose_price     { to: Ref | None = None, item: Ref, price: int }  → propose-price  target to (None = my shop)
  accept_proposal   { proposal: ProposalRef }                         → accept-proposal  target = the proposal's buyer
  decline_proposal  { proposal: ProposalRef }                         → decline-proposal target None
  withdraw_proposal { proposal: ProposalRef }                         → withdraw-proposal target None
Reading + ProposalReading { kind: "proposal", proposal, seller, buyer, item, name, price: Amount, valid_until: int | None }
          ← price-proposed, proposal-accepted, proposal-declined, proposal-withdrawn
```

- **References, not ids.** A proposal id is an `ActionId`: a global counter (S31-h SD-S31h-1). The step-27 rule
  shows no raw id (§5.1: they cost tokens and invite guessing). `refs.py` gains `ProposalTable` (id ↔ `P<n>`,
  remembering each proposal's buyer when first shown). An unknown `P` ref → `refused { unknown_ref }`.
- **The buyer for `accept_proposal`** comes from that table, so the agent cannot send a wrong target; the server
  still decides (`no-such-proposal` if it is gone).
- **Difference from S31-h's draft §5.14:** it writes `proposal: int ≥ 1`. The lane lead reconciles to `P` refs
  (R-S31h-g-1).

### 5.13 The price book (conditional; Q-S31g-4)

`MeLine.survey: SurveyLine | None` = the `survey-book` record's header (`categories`, `depth`, `window_s`,
`resurvey_after_s`, `surveyed: [{ place: Ref, at }]`), and its notes as a paged list `notes` of
`NoteLine { item: Ref, name, at, place: Ref, seller: Ref, price: Amount, source: ask | sale, in_stock: int | None }`
in the record's order (kinds by item, notes oldest first). Nothing is summarised (R-S31g-c-1: "infers nothing").
An agent on the reporter's seat then has exactly the deterministic model's input, through `observe`; `survey` is a
complete offer and needs no kind.

### 5.14 The topic catalogue (conditional; Q-S31g-3)

If `notices` discloses a topic catalogue, `View.topics: [TopicLine { topic, body: prices | text, valid_min_s,
valid_max_s, per_day, may_publish: bool, subscribed: bool }]`, read whole (≤ 32 topics).

---

## 6. Invariants this PR holds

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-g-1 | No path changes outside `clients/agent/`, `cognition/lm-controller/{src/mineworld_cognition/memory/render/, tests/}`, `docs/decisions/ARC-153.md`, `docs/decisions/INDEX.md`, this plan and its handoff (and `docs/reference/agent-tools.*` if S27-f merged) | C8 diff scan |
| I-g-2 (INV-S27-1, -3) | Every new kind reaches the world only through `Player._request` → `offers.request` → `submit`; the adapter evaluates no requirement, compares no price, filters no notice | `test_market_acts.py`; source scan in `test_mcp_confined.py` (no `offers` import in `mcp_server.py`, kept) |
| I-g-2b | The tool list is still the nine tools of `ARC-153` | `test_the_tool_list_is_closed` (unchanged) |
| I-g-3 (INV-S27-2, I-S31-9) | Every line is read from this seat's own observation or perceived facts; nothing about another person is shown that the server did not disclose | AC-g-2; X-9 through the agent |
| I-g-4 (INV-S27-12) | Currency name and symbol, item names and notice text are `WorldText`; `display` and `TopicName` are pattern-constrained | `test_world_text.py` (both halves) |
| I-g-5 | Every amount is `Amount`, and every `display` comes from the disclosed exponent, never a constant | AC-g-4; M-g-1 |
| I-g-6 | Each kind reaches its own `Player` operation | `test_each_kind_reaches_its_own_operation` (extended); M-1 of pr-s27d1 |
| I-g-7 | Every model and new field has a description; the published schemas are the models | `test_descriptions.py`; `test_the_published_schemas_are_the_models` |
| I-g-8 (I-S31-11) | No currency name or symbol of any reference world appears in `clients/agent/src` | S31-a's `currency_names.rs` scan covers `clients/` |

---

## 7. Test ownership

| Layer | Owns | Where (`clients/agent/tests/` unless named) |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict on `clients/agent` and `cognition/lm-controller` | CI `fast` |
| Unit (planted records) | readers, `display_of` at exponents 0, 2, 3, 4, lines, paging and fitting, deltas, readings, payload building and sorting, unknown refs | `test_market_view.py`, `test_notices_view.py`, `test_market_inbox.py`, `test_market_acts.py` |
| In-memory MCP | kinds ↔ operations, schemas, descriptions, closed list, no echo, secrets | `test_mcp_tools.py`, `test_mcp_secrets.py`, `test_descriptions.py`, `test_world_text.py` |
| Python (renderer) | the four gists, mine/not mine, fallback | `cognition/lm-controller/tests/test_memory_render.py` |
| Real binary (`real_server`) | AC-g-1 … AC-g-6 over stdio on three scratch worlds | `test_play_market_mcp.py` |
| Gate 1 | NOT REQUIRED: no model; the agent's model is the user's | — |

Oracles are literal values written in each test, the raw planted payloads, the server's answers and `mineworld
perceived`; never a value the adapter reports about itself (test rules §25). Per the charter, a commit runs its own
new test files; the AC-g table and the mutations run **at the milestone**.

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`; run at the milestone)

**Worlds.** No criterion uses Market Town alone. Every world is built by the test from a repository world, as
`test_mcp_same_answers.py` builds its copy (A-18):

- **W1 `market-g`** — a scratch copy of `worlds/market-town`. Grace's file: `location: { place: cafe, position:
  { x: 2610, y: 600 } }` (1 m from the visitor), `holdings` gains `flowers: 2`, `economy` gains `asks: { flowers:
  450 }`. Felix is a keeper of the store (`keepers: [felix]`). `notices` enabled with `configure/notices.yaml`:
  `news.daily` (`text`, publishers `[visitor]`, 1 h … 7 d, `per_day: 4`) and `prices.ingredients` (`prices`,
  publishers `[felix]`, 1 h … 48 h); Grace subscribes to both. The server runs without `--town` (A-19).
- **W2 `market-g-crown`** — W1 with `configure/economy.yaml` = `{ name: crown, symbol: cr, exponent: 0,
  symbol_at: after }` (step AC-S31-6).
- **W3 `cafe-notices`** — a scratch copy of `worlds/social-cafe` with only `notices` added and the same two topics
  (publishers a café seat): notices without economy.
- **Unit worlds** — the recorded observations with planted records at exponents 0, 2, 3 and 4.

| Id | Criterion | Worlds | Step ref |
| --- | --- | --- | --- |
| AC-g-1 | **Prices and display.** Joined on `visitor`: `View.currency` = US dollar / `$` / 2 / before (both `WorldText`); every café `buy` offer has `price.minor` equal to its payload's price and `display` its decimal, and `item_name` the catalogue's name; Grace's person line lists `flowers` 450 / "4.50", `available: true`. On W2 the same offers read `display` "300", "450" and the currency crown / cr / 0 / after; no Rust, Python or protocol file differs between the runs | W1, W2 | AC-S31-8; AC-S31-6 (agent half) |
| AC-g-2 | **Buys from both, and nothing private leaks.** `act attempt` the café's coffee and then Grace's flowers: both `accepted`; each `outcome.events` holds a `sold` whose `SaleReading.price` equals the offer's; `me.wallet.minor` falls by exactly the two prices; `holdings` +1 each. `inspect` Grace: no `wallet`, `wealth`, `holdings`, `subscriptions`, `survey-book` or `proposals` component; her `asks` shows `available` and no count | W1, W2 | AC-S31-8; I-S31-9 |
| AC-g-3 | **A moved price is never paid.** A plain client on Grace's seat sets flowers to 500; after the agent's view changes, attempting the old offer answers `refused { stale_offer }` with a fresh view showing 500, and no wallet moves (F-S31g-10). The step's server-side `price-changed` stays S31-a's AC-S31-2 | W1 | AC-S31-2 (agent side) |
| AC-g-4 | **Seller kinds.** `list_for_sale { apple, 120 }` accepted → `me.asks` lists it and an `AskReading` (`listed`) is in the outcome; `set_price { apple, 150 }` accepted (`was` 120); `set_price` 150 again → `rejected { system, price-unchanged }`; `list_for_sale` apple again → `already-listed`; `list_for_sale { apple, 0 }` → `price-out-of-range`; `withdraw { apple }` accepted; `set_price { bread, 300, for_shop: true }` in the store → `rejected { permission_denied }` for the visitor; a plain client as Felix in the store sends the same and is accepted, and the agent beside him reads the `AskReading` | W1, W2 | AC-S31-3 (agent side), §5.2 |
| AC-g-5 | **Notices.** `publish_notice { news.daily, text "Market opens at 8", 7 200 }` accepted; the agent's `View.notices` lists it (publisher = me, `valid_until` = issue + 7 200); `subscribe { prices.ingredients }` → `me.subscriptions` holds it and a `TopicReading`; Felix (plain client) publishes a 2-line prices notice → the agent's `wait(any_event)` returns `notice-published` with a `NoticeReading` whose lines carry names and `display`, and `View.notices` lists it; `unsubscribe` removes it; `subscribe { no.such }` → `unknown-topic`. The same on W3, where `display` is `None` for no currency is disclosed | W1, W3 | R-S31g-b-1; AC-S31-13 (b)'s agent half |
| AC-g-6 | **The surface.** The tool list is nine tools; `act` has eleven kinds (fifteen with §5.12); the before and after `tools/list` sizes are recorded; every result of AC-g-1 … 5 is ≤ 8 KiB JSON and text | all | I-d-4, I-d-9 |
| AC-g-7 | **Memory.** For `sold` (buyer, seller, witness), `listed`, `price-set` and `withdrawn`, the renderer's gist equals the literal of §5.9 for each; a malformed payload falls back; `s28a_renders.json` unchanged | synthetic facts of W1 and W2 | step §7.3 |

### 8.1 Planted mutations (run at the milestone; each turns its test red by name, then is restored from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-9 (through the agent) | economy's `discloses` returns `Wallet` for any observer (test-only edit in `systems/economy`, binary rebuilt, never committed) | AC-g-2's `inspect` arm |
| M-g-1 | `display_of` divides by 100 always | AC-g-1's W2 arm ("3.00" instead of "300"); unit exponents 0, 3, 4 |
| M-g-2 | `offer_line` takes the price from the shop listing again | AC-g-1 (Grace's buy has no price); `test_market_view.py` person-buy case |
| M-g-3 | the adapter drops notices with `valid_until ≤ at` itself | `test_notices_view.py::the_board_is_shown_as_disclosed` (a planted record holding an expired notice is shown whole) |
| M-g-4 | `set_price` bound to the `list_for_sale` operation in `ACTS` | `test_each_kind_reaches_its_own_operation` |
| M-g-5 | price lines sent in the agent's order | `test_market_acts.py::price_lines_are_sent_in_item_order`; AC-g-5 (`body-invalid`) |
| M-g-6 | the currency symbol copied into `display` | `test_world_text.py` behavioural half |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and its evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit (Haiku implementer: follow it literally)

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31g`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full.
2. Make the numbered steps in order. Do not do a later commit's work.
3. Run only the test files this commit writes or names, one at a time: `uv run --locked pytest
   clients/agent/tests/<file>`. Record the count, the wall time, PASS / FAIL / INCONCLUSIVE.
4. Static: `uv run --locked ruff check clients/agent`; `uv run --locked ruff format --check clients/agent`;
   `uv run --locked pyright clients/agent` (and `cognition/lm-controller` in C6).
5. Review: answer the commit's review item in §16 in one or two sentences.
6. `git status --short`: every changed path is in the commit's **Files**, or revert it or stop and record why.
7. Update §16 and `handoff-s31g.md` in the same commit.
8. Stage by explicit path; `git commit -m "<message>"` with the session's attribution line. Never `git add -A`,
   heredoc writes, `sed -i` or `python3 -c`.
9. A material stop (§12) is reported, not improvised around.

#### C0 — Base, prerequisites, shapes (Markdown only)

- **Files:** this file (§16), `.structured-coding/plans/mvp0/handoff-s31g.md` (new).
- [ ] 1. Record the base commit; quote the primary's dispatch. Confirm S31-a and S31-b are merged
  (`git log --oneline origin/main | grep -i "s31-a\|s31-b"`); either missing → material stop.
- [ ] 2. Record whether S31-c, S31-h, S27-f, S28-d and the topic catalogue (Q-S31g-3) are merged, and choose each
  branch of §2.2 and §2.1 item 4.
- [ ] 3. **Pin the shapes from merged code**, quoting file and line: the component type of S31-a's asks view; the
  JSON of `currency`, `wealth`, `asks`, `sold`, `listed`, `price-set`, `withdrawn`; the `buy`, `list-for-sale`,
  `set-price`, `withdraw` payloads (the `for` field); S31-b's `subscriptions`, `notice-board`,
  `notice-published`, `subscribed`, `publish-notice` (the `PriceLine.item` shape, F-S31g-6). A shape that differs
  from §5.3 or §5.6 is a bounded correction recorded here; a missing record is a material stop.
- [ ] 4. Confirm S31-a re-recorded `tests/fixtures/market-town-*.json` (`buy` payloads carry `price`; an
  `item-catalogue` is on the café). Not re-recorded → record it; C2 re-records them with `record.py` (A-17).
- [ ] 5. Check whether S35 N-e already wrote the `ARC-153` note; record which.
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  [ ] Review: §2.2's choices hold on the base.
- **Commit message:** `docs(plan): S31-g — open the ledger at the base`

#### C1 — The decision note

- **Files:** `docs/decisions/ARC-153.md`, `docs/decisions/INDEX.md`.
- [ ] 1. Unless C0 step 5 found it: append `## ARC-153 note — 2026-10-19: an action sent with its own payload is a
  typed kind of `act`` with: the rule (one kind per pack action an agent sends by name; one `Player` operation per
  kind; the tool list stays nine; QS27-9 still refuses a free-form act because payload schemas are not published);
  S31-g's six kinds, S35 N-e's `sleep`, S31-h's four as the cases; the Q-S31g-1 ruling.
- [ ] 2. `python3 scripts/check_decision_ids.py --write-index`.
- [ ] Validation: the two doc scripts. [ ] Review: no new identifier; the note heading follows `DECISIONS.md`'s rule.
- **Commit message:** `docs(decisions): ARC-153 note — typed act kinds for actions sent by name (S31-g)`

#### C2 — Amounts, the lexicon and the market lines

- **Files:** `clients/agent/src/mineworld_agent/{models.py, lexicon.py (new), disclosed.py, view.py, handle.py,
  __init__.py}`; `clients/agent/tests/{test_market_view.py (new), test_info.py, test_play_market_town.py,
  test_play_mcp.py, test_view.py, test_world_text.py, test_descriptions.py}`; the fixtures and `SOURCE.md` only
  if C0 step 4 said so.
- [ ] 1. `models.py`: `Amount`, `CurrencyLine`, `WealthLine`, `AskLine`; `MeLine.wallet: Amount | None`,
  `wealth`, `asks`; `PersonLine.asks`, `asks_total`; `OfferLine.price: Amount | None`, `item_name`, `reference`;
  `ItemCount.reference`; `View.currency`; `ViewDelta.currency`; `ListName` + `"asks"`. Every new field has a
  docstring.
- [ ] 2. `lexicon.py`: `Lexicon`, `display_of`, `amount` (§5.2).
- [ ] 3. `disclosed.py`: `any_entity`, `currency`, `wealth`, `asks`, `item_names`, `buy_price`; `wallet` returns
  `Amount`; `holdings` fills `name` and `reference` from the lexicon; the module table gains the rows.
- [ ] 4. `view.py`: `render(observation, refs, lexicon, number, acted_through)`; `offer_line` per §5.4; `Caps.asks`;
  `me.asks` paged; `PersonLine.asks` capped at 8 with `asks_total`; deltas. `handle.py` creates and passes the
  lexicon.
- [ ] 5. Tests: `test_market_view.py` with records planted in C0's pinned shapes (exponents 0, 2, 3, 4; a person
  buy; a `give` without a price; 30 asks on one person → 8 shown, `asks_total` 30; 40 own asks → paged); update
  the `wallet` / `price` reads in `test_info.py` (`.minor`), `test_play_market_town.py:215-216`,
  `test_play_mcp.py:369-384`; `test_world_text.py` (the count of operation models is unchanged here; the new
  `WorldText` places planted in the behavioural half); `test_descriptions.py` (`ItemCount.name` leaves
  `UNDOCUMENTED`).
- [ ] Validation: `test_market_view.py`, `test_info.py`, `test_view.py`, `test_world_text.py`,
  `test_descriptions.py`, one at a time. [ ] Review: no price compared, no rule evaluated; the symbol never
  enters `display`.
- **Commit message:** `feat(agent): the currency, amounts with display, wallet, wealth, asks and item names in the view (S31-g)`

#### C3 — The notice board and subscriptions in the view

- **Files:** `models.py`, `disclosed.py`, `view.py`, `clients/agent/tests/test_notices_view.py (new)`.
- [ ] 1. `TopicName`, `NoticePriceLine`, `NoticeLine`; `MeLine.subscriptions`; `View.notices` and
  `ViewDelta.notices`; `ListName` + `"notices"`; `Caps.notices = 8`.
- [ ] 2. Readers `subscriptions`, `notices` (text as `WorldText` speech by the publisher).
- [ ] 3. Tests: a planted board with a text and a 32-line prices notice; an expired notice planted in the record is
  shown (the adapter does not filter); paging of 12 notices; a record without economy (W3's shape: `display`
  `None`).
- [ ] Validation: `test_notices_view.py`. [ ] Review: no time comparison in the adapter (`grep -n valid_until
  clients/agent/src` shows only reading).
- **Commit message:** `feat(agent): current notices and my subscriptions in the view (S31-g)`

#### C4 — Readings in the inbox

- **Files:** `models.py`, `inbox.py`, `handle.py`, `clients/agent/tests/test_market_inbox.py (new)`.
- [ ] 1. `SaleReading`, `AskReading`, `NoticeReading`, `TopicReading`, `Reading`; `EventLine.reading`.
- [ ] 2. `Inbox(refs, me, bound, lexicon)`; `line()` reads `sold`, `listed`, `price-set`, `withdrawn`,
  `notice-published`, `subscribed`, `unsubscribed` by name (§5.5); a bad payload → `None`.
- [ ] Validation: `test_market_inbox.py`; `test_inbox.py`. [ ] Review: `words` unchanged; the readings' values are
  the payload's.
- **Commit message:** `feat(agent): the inbox reads sales, asks, notices and subscriptions by name (S31-g)`

#### C5 — The six act kinds

- **Files:** `models.py`, `player.py`, `mcp_server.py`, `__init__.py`; `clients/agent/tests/{test_market_acts.py
  (new), test_mcp_tools.py, mcp_double.py, test_mcp_secrets.py, test_world_text.py, test_descriptions.py,
  golden/tools.json}`.
- [ ] 1. Bodies, `*Args`, `*Act` models and the `Act` union (§5.6); `FREE_TEXT` gains the notice text in its three
  places; the operation-model count becomes 29.
- [ ] 2. `Player.list_for_sale`, `set_price`, `withdraw`, `publish_notice`, `subscribe`, `unsubscribe`, with the
  docstrings of §5.6, through one `_market` helper.
- [ ] 3. `ACTS` six rows; `ACT_INTRO` (§5.7); `mcp_double.OPERATIONS` gains the six names.
- [ ] 4. Tests: payloads built for each kind (literal JSON in the test), lines sorted by id, unknown ref →
  `refused { unknown_ref }`, not offered → `refused { not_offered }`; `ACT_CALLS` extended; secrets through every
  kind.
- [ ] 5. Regenerate `golden/tools.json` (`uv run --locked mineworld-agent schema --out
  clients/agent/tests/golden/tools.json`), read it whole, record the before/after sizes in §16.
- [ ] Validation: `test_market_acts.py`, `test_mcp_tools.py`, `test_mcp_secrets.py`, `test_world_text.py`,
  `test_descriptions.py`, `test_cli.py`. [ ] Review: no kind bypasses `offers.request`; the tool list is nine.
- **Commit message:** `feat(agent): list, re-price, withdraw, publish, subscribe and unsubscribe as kinds of act (S31-g)`

#### C6 — Economy's memory renderer

- **Files:** `cognition/lm-controller/src/mineworld_cognition/memory/render/{economy.py (new), __init__.py}`,
  `cognition/lm-controller/tests/test_memory_render.py`.
- [ ] 1. §5.9's four renderers; four registry lines. [ ] 2. Cases for every gist and the fallback.
- [ ] Validation: `uv run --locked pytest cognition/lm-controller/tests/test_memory_render.py`; ruff; pyright;
  `git diff --stat` shows `tests/golden/s28a_renders.json` untouched. [ ] Review: pure, no I/O, no clock.
- **Commit message:** `feat(cognition): people remember the sales and prices they saw (S31-g)`

#### C7 — The real-binary checkpoint and the conditional rows

- **Files:** `clients/agent/tests/test_play_market_mcp.py (new)`; for §2.2's rows that C0 selected, the files of
  §5.12 / §5.13 / §5.14 and their tests.
- [ ] 1. The three world builders (W1, W2, W3) as literal text in the test; AC-g-1 … AC-g-6 as one test per world.
- [ ] 2. The conditional rows, each in its own commit if selected (`feat(agent): haggling rows (S31-g, S31-h §5.12)`
  and so on), following §5.12 … §5.14.
- [ ] Validation: build the binary once through `cast-slots/with-slot.sh cargo build -p mineworld-cli`; run the new
  file once locally (wall times INCONCLUSIVE by R-LOAD-1). [ ] Review: every oracle is the server's.
- **Commit message:** `test(agent): an MCP agent shops, sells and reads notices in three generated worlds (S31-g)`

#### C8 — Close-out to MERGE-READY

- **Files:** this file (§16), `handoff-s31g.md`, `clients/agent/README.md`; `docs/reference/agent-tools.*` if S27-f
  merged.
- [ ] I-g-1's diff scan; push `mvp0/pr-s31g`; open the PR (body ends with the Claude Code line); `fast` green on the
  exact head; report **MERGE-READY** with the PR number and head SHA.
- **Commit message:** `docs(plan): S31-g close-out`

---

## 10. Requirements this PR places on other PRs

- **R-S31h-g-1 (on S31-h).** If S31-h merges second, it implements §5.12 exactly (`P` refs, the four kinds, the
  `ProposalLine`, the readings). S31-h's draft §5.14 writes `proposal: int`; the lane lead reconciles it to §5.12.
- **R-S31c-g-1 (on S31-c).** If S31-c merges second and Q-S31g-4 is yes, it implements §5.13 exactly.
- **R-S31b-g-2 (on S31-b or its follow-up).** If Q-S31g-3 is yes: a `topics` record from `notices` to every living
  person (topic, body kind, validity bounds, `per_day`, `may_publish`, `subscribed`), and the agent line of §5.14.
- **R-S31e-g-1 (on S31-e).** The 2D and 3D formatters show the same decimal as `display` for the same exponent;
  the symbol placement is theirs.
- **R-S28d-g-1 (on S28-d or the first PR after it).** Economy's renderer gains `anchors` (the item and the place).
- **R-S27f-g-1 (on S27-f, if it merges after this PR).** The guide's transcript and `agent-tools.*` come from the
  regenerated golden.

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31g-1 | The currency is `View.currency`, not `WorldLine` | F-S31g-2 |
| SD-S31g-2 | `display` is the decimal without the symbol | F-S31g-3, INV-S27-12 |
| SD-S31g-3 | `OfferLine.price` comes only from a `buy` payload | F-S31g-4, R-S31g-a-1 |
| SD-S31g-4 | `MeLine.wallet` and `OfferLine.price` become `Amount` (a clean break, no adapter) | `CLAUDE.md` §4 rule 12; six test lines change |
| SD-S31g-5 | `PersonLine.asks` capped at 8 with `asks_total`; `me.asks` and `notices` paged | the 8 KiB cap; every ask is also a paged buy offer |
| SD-S31g-6 | Numbers bounded by wire type only; text and line count copy the pack's size bounds | the world's codes stay reachable; the `say` precedent for sizes |
| SD-S31g-7 | The adapter sorts price lines by item id; it never removes a duplicate | the agent cannot see ids; the world judges duplicates |
| SD-S31g-8 | `TopicName` is a pattern-constrained authoring key, like `SeatName` | it must round-trip into `subscribe`; the pattern leaves no room for prose |
| SD-S31g-9 | Economy's renderer lands now, anchors later | F-S31g-8; S31-b's notices precedent |
| SD-S31g-10 | One `EventLine.reading` union, not five fields | a `None` field costs its key in every line |
| SD-S31g-11 | A notice text is `WorldText` kind `speech`, by its publisher | words someone wrote, the nearest existing kind |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-g — the agent (MCP) view of trade and notices
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31g-agent-view.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§0, 11, 13–16); pr-s31a-currency-asks.md (§§5, 10);
                         pr-s31b-notices.md (§§5, 10); pr-s27d-mcp-stdio.md; pr-s27d1-single-act.md;
                         docs/decisions/ARC-153.md; step-27-agent-players.md (§§3, 5); CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; LANE_LEAD_CHARTER.md ("SPEED FIRST")
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31g, its own, held by one session
BRANCH:                  mvp0/pr-s31g, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-31 §16.5) AND S31-a merged AND
                         S31-b merged (S27-d #239 and ARC-153 #261: merged) AND this design frozen AND the
                         primary session's dispatch                                         (step; operator)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–4; §2.2's rows as C0 selects them; as §14's rulings shape them
                                                                                             (operator, pending)
FROZEN INVARIANTS:       INV-S27-1 … 13; ARC-153 (nine tools); §6 I-g-1 … I-g-8; §8 as written
SEQUENCE:                C0 … C8 (§9)
ALLOWED COMMANDS:        uv run --locked pytest <one file> | ruff | pyright | mineworld-agent schema; cast-slots/
                         with-slot.sh cargo build -p mineworld-cli; target/*/mineworld *; git; gh (pr create/view/
                         checks; never merge); python3 scripts/check_doc_headings.py, check_decision_ids.py
                         (--write-index); mkdir -p; cp; cmp; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; a whole-member or workspace test run
                         (R-LOAD-1); reading any key or .env; calling any model; committing an edit outside I-g-1
                         (X-9's plant is reverted, never staged); merging
MATERIAL STOPS:          S31-a or S31-b not merged; a record or payload of §5.3/§5.6 missing on the base; a needed
                         change outside I-g-1's paths; a 10th tool; a result over 8 KiB that paging cannot fit; any
                         change to a frozen invariant or acceptance
PLATFORMS:               Linux, macOS, Windows (CI `fast`; the milestone runs the real-binary file on all three)
VALIDATION BUDGET:       per commit: the build and the test files it writes or names, one at a time, plus `fast`
                         on push. AC-g-1 … AC-g-7 and §8.1's mutations: at the milestone (charter). Gate 1: NOT
                         REQUIRED. No paid call
LIVE DOCUMENTATION:      this document (§16)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31g.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to `fast` green: authorized           (working rules §21)
                         endpoint: MERGE-READY — pushed, `fast` green on the exact head, reported with PR
                                   number and head SHA                    (charter "SPEED FIRST", 2026-10-18)
                         merge: NOT by this session — the coordinator merges (charter, 2026-10-18)
POST-MERGE SYNC OWNER:   primary: step-31, overall.md, the progress page; implementation session: this ledger
STOP CONDITION:          S31-g MERGE-READY — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31g-1 | S31-a's or S31-b's merged shapes differ from their designs (F-S31g-6 already shows one) | C0 pins every shape from merged code; readers accept typed or bare ids |
| R-S31g-2 | A view with a 32-line notice and long names does not fit 8 KiB | paging and the fitting loop; a unit case with 32 lines of 64-byte names; a failure is a material stop |
| R-S31g-3 | The tool list grows past what an agent client shows its model (the reason for ARC-153) | sizes recorded before and after (AC-g-6); descriptions short; Q-S31g-1 states the cost |
| R-S31g-4 | Haggling rows land twice or not at all | §2.2's rule, checked at C0 by both PRs; §5.12 is the one interface |
| R-S31g-5 | The real-binary file is slow on Windows CI | no `--town`, no walking; three short worlds; a timing failure is INCONCLUSIVE |
| R-S31g-6 | An agent cannot find topics (F-S31g-7) | Q-S31g-3 |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S31g-1 [operator — step §11 vs ARC-153]** | The step asks for six new MCP tools; ARC-153 made one `act` tool. Make them six typed **kinds** of `act` (`list_for_sale`, `set_price`, `withdraw`, `publish_notice`, `subscribe`, `unsubscribe`), and haggling's four the same way (S31-h's Q-S31h-3)? | **Yes.** The tool list stays nine, each kind is typed, and no free-form act is opened (QS27-9 holds). One note under ARC-153 records the rule |
| **Q-S31g-2 [operator — scope]** | 12d now discloses item names. Should S31-g show them (holdings, asks, buy offers, notice lines, sales), closing F-S27c-9? | **Yes.** An agent that cannot tell coffee from bread cannot shop; the names are already on the wire, one reader |
| **Q-S31g-3 [operator — S31-b gap]** | No record discloses the world's topics, so an agent (or a 2D player) can subscribe only to names it already knows. Add a public `topics` record to `notices`? | **Yes, as a small follow-up to S31-b** (before S31-e and S31-g merge); S31-g then shows `View.topics`. Without it `subscribe` is guesswork |
| **Q-S31g-4 [operator — R-S31g-c-1]** | Should the agent view show the reporter's price book (paged), so an MCP agent on the reporter's seat has the deterministic model's exact input? | **Yes,** in whichever of S31-c and S31-g merges second. Agents are first-class players; `inspect` cuts a 30 KB book at 4 KiB |

### 14.1 Rulings

Ruled 2026-10-20 by the operator in the primary session, relayed in the S31-g dispatch: every question is
answered as recommended.

| Id | Ruling | Consequence here |
| --- | --- | --- |
| Q-S31g-1 | **Yes: typed act kinds.** The six market and notice actions are six typed kinds of `act`; haggling's four the same way | C5 adds six kinds; the tool list stays nine; C1 writes the `ARC-153` note |
| Q-S31g-2 | **Yes: show item names** | already on main through #402 (`disclosed.item_kinds`); C2 extends the names to asks, buy offers, notice lines and sales, kept per handle in the lexicon |
| Q-S31g-3 | **Yes: a public `topics` record in `notices`, as a follow-up PR (S31-b2)** | S31-b2 is not merged at C0, so §2.2's third row takes its "otherwise" branch: S31-b2 adds `View.topics` (R-S31b-g-2) |
| Q-S31g-4 | **Yes: show the reporter's price book, in whichever of S31-c and S31-g merges second** | S31-c is not merged at C0, so §2.2's second row takes its "otherwise" branch: S31-c implements §5.13 (R-S31c-g-1) |

---

## 15. Proposed edits elsewhere (made by the PR named)

- **`step-31-wealth-prices.md` §11 (primary):** the `WorldLine` row → `View.currency` (F-S31g-2); the tools row →
  "typed kinds of the `act` tool (`ARC-153`)" (F-S31g-1); the amounts row → "`display` is the decimal in major units;
  the symbol is in `View.currency`" (F-S31g-3); §5.9's "the tools `propose_price` …" → kinds (§5.12).
- **`step-27-agent-players.md` §5.5 (primary):** "`attempt` is the generic act … free-form actions are reached only
  through named tools" → "… through named kinds of `act`".
- **`docs/decisions/ARC-153.md` (this PR, C1):** the note of §0.
- **`pr-s31h-haggling.md` §5.14 (S31-h's owner):** `proposal: ProposalRef` (§5.12).
- **`overall.md` (primary):** S31-g's state; F-S27c-9 closed if Q-S31g-2 is yes.

---

## 16. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] base, merges, shapes pinned (below) | [x] doc scripts | [x] §2.2's choices hold on the base |
| C1 | (this commit) | [x] `## ARC-153 note` appended; `INDEX.md` unchanged (`--write-index`: "index current", a note adds no id) | [x] `check_decision_ids.py` 214 distinct; `check_doc_headings.py` clean | [x] no new identifier; heading follows the `ARC-6 note` precedent |
| C2 | (this commit) | [x] `Amount`, `CurrencyLine`, `WealthLine`, `AskLine`; `lexicon.py` (`Lexicon`, `display_of`); `disclosed.{any_entity, currency, wallet, wealth, asks, buy_price, item_names}`, `shop_prices` and `item_name` removed; `render(…, lexicon=)`, `offer_line` per §5.4, `Caps.asks`, `ASKS_PER_PERSON`, `me.asks` paged, deltas; `Handle.lexicon` | [x] PASS, one file at a time: `test_market_view.py` 20, `test_info.py` 20, `test_view.py` 22, `test_world_text.py` 5, `test_descriptions.py` 3, `test_item_names.py` 7, `test_caps.py` 5, `test_needs_view.py` 12, `test_reference_pages.py` + `test_cli.py` 12; ruff, ruff format, pyright clean on `clients/agent` | [x] no price compared and no rule evaluated (a price is read from a `buy` payload only); the symbol never enters `display` (`test_the_symbol_never_enters_display`) |
| C3 | (this commit) | [x] `TopicName` (`TOPIC_PATTERN`), `NoticePriceLine`, `NoticeLine`; `MeLine.subscriptions`; `View.notices`, `ViewDelta.notices`; `ListName` + `notices`; `Caps.notices = 8`; `disclosed.{topic, subscriptions, notice_line, notices}`; the fitting loop's two last resorts (D-g-8); test helpers moved to `tests/planted.py` | [x] PASS one at a time: `test_notices_view.py` 8, `test_market_view.py` 20, `test_caps.py` 5, `test_view.py` 22, `test_info.py` 20, `test_world_text.py` 5, `test_descriptions.py` 3, `test_reference_pages.py` 3; ruff, format, pyright clean | [x] no time comparison in the adapter: `grep -n valid_until clients/agent/src` shows only the reader (`disclosed.py:240`) and the model field |
| C4 | (this commit) | [x] `SaleReading`, `AskReading`, `NoticeReading` (a `NoticeLine` with `kind`), `TopicReading`, `Reading`; `EventLine.reading` (left out when `None`); `inbox.reading(event, refs, lexicon)`; `Inbox(refs, me, bound, lexicon)`; `Player._handle` makes one `Lexicon` and gives it to the handle and its inbox (the plan's `handle.py` edit lands in `player.py`, where the inbox is built) | [x] PASS: `test_market_inbox.py` 7, `test_inbox.py` 7, `test_inbox_status.py` 4, `test_descriptions.py` 3, `test_reference_pages.py` 3; ruff, format, pyright clean | [x] `words` unchanged (still `spoke` only); every reading's values are the payload's, a bad payload gives `None` and the fact is still listed |
| C5 | (this commit) | [x] bodies `ListForSale`, `SetPrice`, `Withdraw`, `PublishNotice` (`TextBody`, `PricesBody`, `PriceArg`), `Subscribe`, `Unsubscribe`; their `*Args` and `*Act`; `Act` is twelve kinds; `Player.{list_for_sale, set_price, withdraw, publish_notice, subscribe, unsubscribe}` (one `_market` helper for the seller three; price lines sorted by integer id); `ACTS` six rows; `ACT_INTRO` without a count; exports; `mcp_double.OPERATIONS`; `FREE_TEXT` + `TextBody.text`; operation models 24 → 30 | [x] PASS: `test_market_acts.py` 8, `test_mcp_tools.py` 15, `test_mcp_secrets.py` 2, `test_world_text.py` 5, `test_descriptions.py` 3, `test_cli.py` 9, `test_reference_pages.py` 3, `test_mcp_confined.py` 4, `test_mcp_resources.py` 10, `test_guide_chapter.py` 7, `test_markdown_reference.py` 4; ruff, format, pyright clean. `test_http_endpoint.py`: INCONCLUSIVE (needs the real binary; milestone) | [x] every kind reaches the world through `Player._request` → `offers.request` (an unoffered kind answers `not_offered`, nothing sent); the tool list is still nine (`test_the_tool_list_is_closed`) |
| C6 … C8 | — | [ ] | [ ] | [ ] |

### C0 — the base (2026-10-20)

**Dispatch (primary session, relayed by the lane lead):** "The operator froze it on 2026-10-20 with every §14
question answered as recommended … S31-a, S31-b, S27-d and ARC-153 are merged. S31-c and S31-h are being
implemented in parallel and are NOT merged, and the topics record is not merged … PUSH RULE: implement and commit
LOCALLY only. Do NOT push and do NOT open a PR … When the PR is complete, hand back with the local head SHA."

- **Base:** `origin/main @ fec906f2` (2026-10-20), worktree `impl-s31g`, branch `mvp0/pr-s31g`.
- **Merged:** S31-a (#380, `16f19d3f`), S31-b (#349, `0574b977`), S27-d (#239), ARC-153 (#261), S35 N-e (the
  `sleep` kind is on `ACTS`), S27-f (`docs/reference/agent-tools.{json,md}` exist and are the golden: `golden/tools.json`
  is gone, `test_cli.py:14-15`), S28-d (#c64b3f60 …), the item-names fix #402 (`7fc588c7`).
- **Not merged:** S31-c, S31-h, S31-b2 (the topics record). §2.2: all three rows take their "otherwise" branch;
  R-S31h-g-1, R-S31c-g-1 and R-S31b-g-2 bind those PRs. §2.1 item 4: `docs/reference/agent-tools.*` are
  regenerated here (S27-f merged).
- **The `ARC-153` note:** not written by S35 N-e (`docs/decisions/ARC-153.md` has no `## ARC-153 note`); C1 writes it
  and names `sleep` as an existing case.

**Shapes pinned from merged code** (every one matches §5.3 / §5.6 unless marked):

| Record / payload | Merged shape | Source |
| --- | --- | --- |
| `currency` (about me, every living person) | `{ name, symbol, exponent: u8, symbol_at: "before"\|"after", references: [{ item: ItemId, value }] }` | `systems/economy/src/currency.rs:65-80, 137-143`; `configuration.rs:313-322` |
| `wallet` | `{ balance: u64 }` (holder) | `component.rs:18-21`; `system.rs:636-643` |
| `wealth` | `{ amount: u64, unvalued: u64 }` (holder, when it has a wallet) | `component.rs:395-399`; `system.rs:652-657` |
| asks view, component type **`asks`** | `{ asks: [{ item: ItemId, price, available }] }`, to anyone perceiving a living person seller with an ask | `component.rs:337-376`; `offer.rs:47-60`; `system.rs:658-663` |
| `shop` (the place) | `{ operator, listed: [{ item, price, in_stock }] }` | `component.rs:262-300` |
| `listed` | `{ seller: EntityId (bare string), item: ItemId, price }` | `event.rs:122-129` |
| `price-set` | `{ seller: EntityId, item: ItemId, from, to }` | `event.rs:403-411` |
| `withdrawn` | `{ seller: EntityId, item: ItemId }` | `event.rs:450-456` |
| `sold` | `{ seller: EntityId, buyer: EntityId, item: ItemId, count: u32, price, place: PlaceId }` | `event.rs:480-491` |
| `buy` | `{ item: ItemId, price }` (complete, one per ask) | `action.rs:16-30` |
| `list-for-sale`, `set-price` | `{ item: ItemId, price, for?: "shop" }`, `deny_unknown_fields`; incomplete offers, target `None` | `action.rs:51-104`; `listing.rs:149-190` |
| `withdraw` | `{ item: ItemId, for?: "shop" }` | `action.rs:106-124` |
| `subscriptions` (holder) | `{ topics: [TopicId] }` | `systems/notices/src/component.rs:14-58`; `system.rs:313-323` |
| `notice-board` (holder; current notices of subscribed or published topics, (topic, publisher) order; absent when empty) | `{ notices: [{ id: u64, topic, publisher: PersonId (typed), body, issued_at, valid_until }] }` | `component.rs:62-75`; `board.rs:30-48`; `system.rs:324-342` |
| notice `body` | externally tagged `{"prices": [{ item, price, basis }]}` or `{"text": "…"}` | `body.rs:70-80` |
| **`PriceLine.item`** | **`EntityId`, a bare id string** (F-S31g-6 settled: the design's `ItemId` is not what merged) | `body.rs:24-30` |
| `notice-published` | `{ notice: Notice }` | `event.rs:243-248` |
| `subscribed`, `unsubscribed` | `{ person: PersonId, topic }` | `event.rs:166-210` |
| `publish-notice` | `{ topic, body, valid_for: u32 }`, `deny_unknown_fields` | `action.rs:15-25` |
| `subscribe`, `unsubscribe` | `{ topic }`; offered incomplete, target `None` (`unsubscribe` only while subscribed to one) | `action.rs:55-90`; `system.rs:287-301` |
| bounds | `TEXT_MAX_BYTES = 280`, `PRICE_LINES_MAX = 32`, `TOPIC_MAX_BYTES = 64` | `body.rs:13-16`; `topic.rs:13` |

**Bounded corrections from the base (recorded, not operator questions):**

- **D-g-1 — the golden is `docs/reference/agent-tools.json`.** §2.1 item 2 and C5 step 5 name
  `clients/agent/tests/golden/tools.json`; S27-f made the published reference the one golden (QS27f-4). C5
  regenerates `docs/reference/agent-tools.json` and `agent-tools.md` with the two commands of
  `test_reference_pages.py:30-40`.
- **D-g-2 — seven kinds today, not five.** S35 N-e added `sleep`; `act` goes from 6 to 12 kinds, and the operation
  models from 24 to 30 (`test_world_text.py`), not 23 to 29. AC-g-6's "eleven kinds" reads twelve.
- **D-g-3 — item names are already on the view (#402).** `disclosed.item_kinds` reads my place's `item-catalogue`;
  C2 keeps the lexicon (names kept per handle across places) and uses it for asks, buy offers, notice lines and
  readings. `ItemCount.name` is already documented.
- **D-g-4 — anchors need no renderer change.** S28-d's `render` scans every fact's payload for anchors generically
  (`memory/render/__init__.py:150-157`), so economy's renderer gets anchors with no code (R-S28d-g-1 satisfied).
- **D-g-5 — the agent fixtures were not re-recorded.** `clients/agent/tests/fixtures/SOURCE.md` is still the
  2026-10-14 recording (no `item-catalogue`, `buy` without `price`). Re-recording needs the real binary and moves
  every hash; under the charter it is milestone work. The new unit tests plant records in the pinned shapes, and
  take the real post-S31-a shapes from `tests/fixtures/observations/items/market-town.json` (recorded after S31-a:
  `currency`, `wealth`, priced `buy` payloads). Recorded for the milestone.
- **D-g-7 (C2) — the reference is regenerated by each commit that changes a schema**, not only C5, so every local
  head keeps `test_reference_pages.py` and `test_cli.py` green. `render` takes the lexicon as a keyword
  (`lexicon=`), not positionally as §9 C2 step 4 writes it, so its many existing callers stay unchanged. The
  old-fixture price test of `test_info.py` now asserts that a pre-S31-a `buy` (no payload price) shows no price,
  although the café's shop lists every kind (SD-S31g-3); the priced case is `test_market_view.py` on the real
  post-S31-a recording. `test_play_station_mcp.py:255` reads `price.minor` (the `Amount` shape; real-binary file,
  milestone run).
- **D-g-8 (C3) — R-S31g-2 measured: the largest notice does not fit one page with its names.**

  ```text
  Previous assumption:  "One 32-line prices notice with 64-byte names is about 5 KB and still fits a page" (§5.4).
  Audit evidence:       measured on the recorded café view under the Player's cap (8 192 − 512 = 7 680 bytes):
                        a notices-only page is 7 412 bytes at realistic prices (450, basis 6; the notice alone
                        5 564) and 8 180 bytes at the pack's largest (10^12 minor units, basis 65 535; alone
                        6 332). Before this commit the fitting loop also raised OverflowError whenever every
                        shown list held one entry, so such a board would have made `observe` fail outright.
  Corrected:            the fitting loop has two last resorts before refusing a view: when every shown list holds
                        one entry, the first moves to its own page (listed in `more`); when one entry is still too
                        large, a notice's price lines are shown without item names (each item still answers
                        `inspect`). Nothing else changes, and a view that fit before is unchanged.
  Validation:           test_notices_view.py: names kept at realistic prices; names dropped, all 32 lines and
                        prices kept, at the largest prices; both under the Player's cap.
  ```

  Bounded: no contract, scope or acceptance change (§12 names "a result over 8 KiB that paging cannot fit" as the
  stop; paging now fits it). Recorded for the lane lead.
- **D-g-6 — push rule.** The dispatch says commit locally and hand back the head; C8's push and PR are the lane lead's
  call, so C8 ends at a local head.

### Evidence

**`tools/list` size** (`catalogue_json(compact=True)`, `ARC-153`'s measure, UTF-8 bytes):

| Head | Bytes | What grew |
| --- | --- | --- |
| base `fec906f2` | 100 988 | — |
| after C4 (the view lines and readings) | 137 851 | `View` (in `join`, `observe` and `act`'s `ActResult`), `EventLine.reading` (in `events`, `wait`, `ActResult`) |
| after C5 (six kinds) | 144 417 | the six `*Act` variants and their descriptions |

Growth +43 %, about 86 % of it from the information-rich view (R-S31g-3). Recorded for the lane lead: MCP has no
shared definitions between tools, so each output schema carries `View`; reducing it would change what a result
holds, which `ARC-153` did not do and this PR does not do either (F-g-12).

### Mutations

(none yet; run at the milestone: X-9, M-g-1 … M-g-6 as §8.1 lists them)

### Deviations and findings

F-S31g-1 … F-S31g-10 (§3.2) and SD-S31g-1 … 11 (§11), recorded at design time.
