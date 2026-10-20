# PR S31-e — Trade in the 2D and 3D clients: one money formatter, one trade row model, price tags, the sale panel, the stall and the notice board

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31e, branch docs/pr-design-s31e,
                        planning base origin/main @ 44f050e3 (#171, 12d merged; step-31 APPROVED, S31-a and
                        S31-b FROZEN on main)
Approved by / evidence: none yet. The step's rulings (Q-S31-1 … Q-S31-8, step-31 §18), S31-a's Q-S31a-1 … 4
                        and S31-b's Q-S31b-1 … 4 bind and are not re-asked. §14 lists this PR's questions
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S31-a, S31-b,
                        16c (#245), 16d (#250), S31-h and 12e (#254)
Start condition:        step-31 §16.5: nothing in S31 starts until Milestone F's checklist has been handed to
                        the operator (the S29 timing rule); then S31-a merged (NOT met: frozen, not
                        implemented, 2026-10-18); then 16d (#250, open; it is stacked on 16c, #245, open)
                        merged, because this PR's 3D half builds on 16d's menus and readers; then this design
                        frozen; then the primary session's dispatch. S31-b merged is needed by C6 only
                        (SD-S31e-9)
Execution contract:     §12 (filled; ends at MERGE-READY, the coordinator merges; charter 2026-10-18)
Implementer:            Haiku-class (step-31 §16.2, the S31-e row: "Haiku (2D); Opus review of 3D frames"),
                        from this commit plan; an Opus-class review of the 3D frames at the milestone
Lifecycle:              FROZEN 2026-10-20 (rulings §14.1); IMPLEMENTING on mvp0/pr-s31e from origin/main
                        @ fec906f2
```

**Effort:** `mvp0`, release-blocking (Q-S31-1 re-ruled) · **Step:**
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15) §§1 (outcome 10), 4.1 (display
text), 5.5, 5.6, 7.1, 8.6, 12, 14 (I-S31-9, I-S31-11, I-S31-12), 15 (AC-S31-6 client half, AC-S31-15; X-8, X-10),
16.2 (the S31-e row), 16.5 · **Parent:** [`overall.md`](overall.md) · **Depends on:**
[`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (FROZEN 2026-10-18; its R-S31e-a-1),
[`pr-s31b-notices.md`](pr-s31b-notices.md) (FROZEN 2026-10-18; its R-S31e-b-1), step-15 §22 (16d, DESIGN
FROZEN 2026-10-09, PR #250 open) and §20 (16c, PR #245 open) · **Coordinates with:** S31-h
(`pr-s31h-haggling.md`, DRAFT at `design-s31h` commit 4d5a81f6; its R-S31e-h-1 and §5.13), S31-g (the agent
view; nothing shared but wording ideas) · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 3, 6, 7, 8, 9, 13,
15, 16); `docs/ENGINEERING_RULES.md` §§2–3, 7–12; `clients/protocol/ADOPTION.md` §3.3; `clients/shared/SETTINGS.md`
§6; `docs/decisions/` `ARC-47`, `ARC-70`, `ARC-111`, `ARC-112`, `ARC-114`, `ARC-115`.

**Decision ids.** This PR writes **`ARC-184`** (from the range the lane lead gave this design: `ARC-184`,
`ARC-185`, `DEP-101`): "both reference clients read trade records and format money through one shared module,
`clients/shared/trade`; no client assumes a currency scale or compares money". `ARC-185` and `DEP-101` stay
unused: no dependency is adopted or declined (§4). Confirmed free on `origin/main @ 44f050e3`
(`ls docs/decisions | grep "ARC-18[45]\|DEP-101"`, no match).

No hosted model is called by this design or by anything it specifies for CI. No LM seat is needed: every
action in this PR is sent by a human player through a client (the S31-h finding on incomplete offers does not
apply).

---

## 0. Binding inputs, exactly restated

- **Scope (step-31 §16.2, S31-e row).** "shared formatter; HUD; tags; sale panel; stall; notice board; i18n;
  client-rules patterns; drives and shots". Depends on "S31-a (S31-b for the board)". Checkpoint "AC-S31-15".
  Adversarial "X-8, X-10". Implementer "**Haiku** (2D); **Opus** review of 3D frames".
- **Step §12 (the clients table and rule).** "The rules are evaluated on the server only. Clients render
  disclosed records and send intents." The client-rules scan "gains two patterns: a client comparing a wallet
  with a price, and a client reading `reference` to compute a price. Either fails CI (X-8)." "Both clients use
  one shared formatter and one panel model, so no game logic is duplicated." "3D work is limited to the HUD and
  tags, with no new geometry."
- **AC-S31-15.** "2D and 3D drives: walk to the store, read the price tag, buy, see the wallet drop by the price.
  Walk to Grace, buy flowers. Refusals show their words. Recorded shots on three OSes."
- **AC-S31-6 (client half).** Renaming the currency to `{ crown, cr, 0, after }` changes "the client display
  (2D shot)" and nothing outside the world's directory. **X-10:** "the 2D formatter keeps `/100`" must fail
  because "AC-S31-6's `exponent: 0` shot shows '0.03 cr'". **X-8:** "2D client greys out *Buy* by comparing
  wallet with price" must fail the client-rules scan.
- **R-S31e-a-1 (S31-a §10).** "Read the `currency` record (§5.3) and the `asks` view; incomplete
  `list-for-sale`/`set-price`/`withdraw` already appear in the 2D menu as disabled raw entries until then."
- **R-S31e-b-1 (S31-b §10).** "The board panel renders the `notice-board` record; the codes of §5.7 get words in
  `en.po` / `zh_Hans.po`; the client evaluates no rule (no expiry computed client-side: the record is already
  current)."
- **R-S31e-h-1 (S31-h DRAFT §10).** "If S31-h merges first, S31-e includes §5.13's rows, words and AC-h-12. If
  S31-e merges first, its panel model has a row type for an incoming offer with two buttons and a price-entry
  row, so C8 adds rows and no new component." Step §16.5: "whichever merges second carries" the rows.
- **Charter update (operator, 2026-10-18; lane brief "SPEED FIRST and no hard-coding").** The contract ends at
  MERGE-READY (pushed, `fast` green on the exact head, reported with PR number and head SHA); the coordinator
  merges. Per-PR validation is the build plus the tests each commit writes; acceptance criteria, drives, shots
  and planted mutations run at the milestone. Mechanisms are generic and data-driven, and **every acceptance
  criterion names at least one second world or a generated variant** besides Market Town.

---

## 1. Goal

Make trading visible and playable in both reference clients, for any world's currency, items and topics.

The world already says what money is called and how many decimal places it has (S31-a's `currency` record).
Today both clients ignore that and divide by 100 (F-S31e-1). After this PR, one shared GDScript module formats
every amount from the disclosed currency, and one shared reader turns the disclosed trade records and offers
into rows. Each client draws those rows in its own look:

- a money line on the HUD (balance, and wealth beside it);
- price tags: a board at a shop's door, and a tag over a person who sells;
- a sale panel beside the interaction menu, with one row per priced offer and a *Buy* button that sends the
  offer exactly as the server made it;
- a stall: list, re-price and withdraw your own goods (and a shop's, if you keep it), with a price field in
  major units;
- a notice board panel with the current notices and a badge for new ones;
- every refusal in words, in English and Simplified Chinese.

No client decides anything. A row that the server marked unavailable is still shown and still pressable, and
the server answers. The client never compares a wallet with a price, never reads a reference value, and never
computes whether a notice has expired. A world without `economy` shows none of this.

---

## 2. Scope

### 2.1 In scope

1. **A new shared client module, `clients/shared/trade/`** (taken by both clients as `res://mineworld_trade`,
   the `mineworld_environment` precedent of RL-e0, commit `d8c7482b`):
   - `money.gd` — `MineWorldMoney`: `format(amount, currency)` and `parse(text, currency)` (§5.2);
   - `trade.gd` — `MineWorldTrade`: readers of the trade records, and the row model both clients draw (§5.3,
     §5.4);
   - `clients/shared/TRADE.md` — the module's specification (written in C1, before code);
   - `clients/shared/checks/money_check.gd`, `trade_check.gd` — headless checks over hand-built frames;
   - the symlinks `clients/2d/mineworld_trade` and `clients/3d-spike/mineworld_trade`; the module loops of
     `mineworld-2d` and `mineworld-slice`; the symlink checks of `.github/workflows/nightly.yml`;
     `clients/shared/README.md`.
2. **Wording, in the shared catalog layer** (`clients/shared/settings/locale/{en.po,zh_Hans.po,messages.pot}`;
   16d D-16d-6's rule: keys both clients use live once there): the money formats, the trade panel, stall and
   board words, and `reason.*` for every code of S31-a §5.5 and S31-b §5.7 (§5.10).
3. **The 2D client** (`clients/2d/scripts/`): `hud/words.gd` (the `/100` removed), `hud/readers.gd` (the
   currency, wealth and asks records), `hud/status.gd` (the money line), a new `hud/trade_panel.gd` (sale
   panel, stall), a new `hud/board_panel.gd` (notice board), `intents.gd` (three composed types),
   `scene/people.gd` (seller tag), `scene/places.gd` (shop board), `app.gd`, `controls.gd`; the harness
   (`harness/interact.gd`: `TRADE` and `BOARD` reports and the trade steps); `clients/2d/PRESENTATION.md` §7;
   `presentation/mineworld-default/2D/i18n/{en,zh_Hans}.po` (only to drop keys that move to the shared layer).
4. **The 3D client** (`clients/3d-spike/scripts/slice/`, on 16d's files): `things.gd` (the `/100` removed;
   panels from `MineWorldTrade`), a new `trade_panel.gd` (`SliceTradePanel`, sale panel and stall, with a price
   field), a new `board_panel.gd` (`SliceBoardPanel`), `intents.gd` (the same three composed types),
   `figures.gd` (seller tag), a new `price_boards.gd` (`SlicePriceBoards`, shop boards at bound doors),
   `slice_main.gd` (HUD money line, key N, wiring), `slice_probe_market.gd` (the reports and steps).
5. **The client-rules scan**, `tests/acceptance/tests/client_rules.rs`: check **d** (a wallet compared with a
   price; a reference value read), and the new action-literal admissions for the two `intents.gd` files (§5.11).
6. **The real-client test**, `tools/cli/tests/client_trade.rs` (every test `#[ignore]`, needs Godot), with the
   generated world variants of §5.12; shots under each client's `shots/` (ignored output).
7. **Documentation:** `docs/decisions/ARC-184.md` and the regenerated `docs/decisions/INDEX.md`; `TRADE.md`;
   `clients/2d/README.md` and `clients/3d-spike/README.md` key tables; this ledger.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Any server rule: prices, refusals, offers, what is disclosed to whom | S31-a, S31-b (frozen); never a client |
| Haggling rows (*Offer a price…*, incoming offers with *Accept* / *Decline*, *Withdraw* an offer) | S31-h if S31-e merges first; this PR if S31-h merged before C0 (§5.13; step §16.5) |
| Subscribing and unsubscribing from a client | not in this PR: the topic list is not disclosed to a non-subscriber (F-S31e-6, Q-S31e-2). Agents subscribe through S31-g's `act` kinds |
| Publishing a notice from a client | not in this PR (the step's §12 table has no publish row); publishers are NPC seats (S31-d) and agents (S31-g) |
| The agent view (MCP `WorldLine.currency`, `MeLine`, `act` kinds) | S31-g (after S27-d; under `ARC-153` one `act` tool with typed kinds) |
| Market Town content (keepers, person asks, references, topics) | S31-f |
| New 3D geometry, a 3D Presentation Pack | never here (step §12: "no new geometry"); 16f |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/`, `cognition/`, `worlds/`, `clients/protocol/`, `clients/agent/` | never in this PR (I-e-1) |

### 2.3 Dependencies

- **Required before C0:** Milestone F's checklist handed over (operator); S31-a merged (the `currency`,
  `wealth` and `asks` records, the priced `buy` offers, the seller actions); 16d (#250) merged, and with it 16c
  (#245), because the 3D sale panel sits beside 16d's `SliceOfferMenu`, reads through 16d's `SliceThings`, and
  places shop boards through 16c's `SliceLayout`.
- **Required before C6 only:** S31-b merged (the `notice-board` record and its codes). If it has not merged when
  C5 is done, C6 is skipped, the PR goes MERGE-READY without it, and C6 becomes PR S31-e2 under this same design
  (SD-S31e-9).
- **Either order, rule "whichever merges second" (step §16.5):** S31-h. If S31-h merged before C0, C7 adds its
  rows (§5.13); otherwise C7 is N/A and S31-h's C8 adds them through §5.13's interface.
- **Either order, rebase rule:** 12e (#254) edits `slice/intents.gd`, `controls_hud.gd` and the probe; the
  later of 12e and this PR rebases (16d's R-16d-5 rule). S23 R-a (#146) exports the clients; if it merges first,
  its export presets gain `mineworld_trade` in C1.

---

## 3. Audit (`origin/main @ 44f050e3`, 2026-10-18; branches as named)

### 3.1 Source anchors

| Id | Fact | Source |
| --- | --- | --- |
| A-1 | The 2D formatter divides by 100 with two decimals, then applies `format.money` | `clients/2d/scripts/hud/words.gd:68-73` |
| A-2 | `format.money` is `"{amount}"`, documented as "already divided by 100 with two decimals" | `presentation/mineworld-default/2D/i18n/en.po:158`; `clients/2d/PRESENTATION.md:208` |
| A-3 | 2D panels: every own component in arrival order, then the place's `shop`; a component with no reader is shown raw under "other" (`I-3`); readers for `wallet`, `holdings`, `shop` | `clients/2d/scripts/hud/readers.gd:76-150` |
| A-4 | 2D groups: I = things, H = conversations | `clients/2d/scripts/hud/panels.gd:1-9, 58-61` |
| A-5 | 2D menu: one entry per affordance in server order; kinds `complete`, `composed`, `point`, `unsupported`, `walk`; verdicts read where entries are built, never where one is chosen | `clients/2d/scripts/menu.gd:1-21, 111-180` |
| A-6 | 2D `intents.gd` is the only submitter; `COMPOSED` lists the composed types; `INPUTS` says what the player supplies; `compose` builds payloads by type | `clients/2d/scripts/intents.gd:16-26, 69-83` |
| A-7 | The 2D typed line: `talk_line.ask(prompt, suggestions, on_done)` | `clients/2d/scripts/hud/talk_line.gd:70` |
| A-8 | 2D status line: who, when, hint | `clients/2d/scripts/hud/status.gd:47-62` |
| A-9 | 2D figure label and participation marker per person | `clients/2d/scripts/scene/people.gd:69-75, 204-214` |
| A-10 | 2D façade anchor per place | `clients/2d/scripts/scene/places.gd:155` |
| A-11 | 2D keys: Esc, E, Q, I, H in one table | `clients/2d/scripts/controls.gd:18-24` |
| A-12 | 2D results become toasts worded from `action.<type>.done` and `reason.<code>` | `clients/2d/scripts/app.gd:340-371` |
| A-13 | `check_client_rules.py` scans `clients/2d/**/*.gd` only (R1–R6); R3' covers `intents.gd`, `menu.gd` and `hud/**`; R6 forbids sentences in UI sinks outside `hud/words.gd` and the harness | `scripts/check_client_rules.py:1-33, 49-67, 166-233` |
| A-14 | `client_rules.rs` reads every `*.gd` under `clients/` (shared modules included, symlinks not followed); check a: an action type as a literal only where `ACTION_LITERALS` admits it; check b: rule-named constants (`REACH`, `RANGE`, `CLEARANCE`, `NUDGE`, `CAPACITY`, `MAX_STRIDE`); a GDScript lexer separates code, strings and comments | `tests/acceptance/tests/client_rules.rs:1-30, 40-113` |
| A-15 | Shared modules are symlinked per client; the launchers refuse a missing one; the nightly Windows jobs check that they are directories | `clients/2d/mineworld_environment` → `../shared/environment`; `mineworld-2d:86`; `mineworld-slice:79`; `.github/workflows/nightly.yml:458, 501` |
| A-16 | Precedent for a shared formatter: `MineWorldClockFormat` in the settings module, worded by the catalogs | `clients/shared/settings/clock_format.gd:1-8`; `clients/shared/SETTINGS.md` §6.6 |
| A-17 | Observation API: `component_value(id, type)`, `own_component(type)`, `affordances(type, target)` (server order; `""` = target-less), `is_complete`, `events_of(type)`; the client's `perceived` signal | `clients/protocol/mineworld/observation.gd:42, 137, 152, 188, 222`; `world_client.gd:56` |
| A-18 | 3D on main composes `talk` only; no money on screen | `clients/3d-spike/scripts/slice/intents.gd:14-65` |
| A-19 | **(16d, `origin/mvp0/pr-16d` @ 6644674b)** `SliceThings.money` divides by 100 (13b's formatting), "S31 replaces it with the disclosed currency"; `panels` reads only `wallet`, `holdings`, `shop` | `clients/3d-spike/scripts/slice/things.gd:16-19, 99-157` (branch) |
| A-20 | **(16d)** `SliceOfferMenu`: B opens your menu with "what you have" beside it, Tab the menu of the person under the ray; 13b's entry model; the menu holds gameplay input while open | `clients/3d-spike/scripts/slice/offer_menu.gd:1-35, 92-123, 160-221` (branch) |
| A-21 | **(16d)** `SliceIntents.submit_offered` sends a complete affordance unchanged; `composes` is `talk` only | `clients/3d-spike/scripts/slice/intents.gd` (branch) |
| A-22 | **(16d, D-16d-6)** the concept keys both clients use move to the shared layer, among them `panel.wallet`, `panel.holdings`, `panel.shop`, `ui.row.shop`, `format.money`; the two item readers (2D `readers.gd`, 3D `things.gd`) are a "known duplication … raised for a later coordinated module PR" (D-16d-5, Q-16d-5) | step-15 §22.3 (branch) |
| A-23 | **(16c, `origin/mvp0/pr-16c` @ 4e39ea69)** `SliceLayout` binds each disclosed place to a drawn door by doorway coincidence; `SliceFigures` builds one `Label3D` name per person | `slice_layout.gd:1-60`; `figures.gd:182-191` (branch) |
| A-24 | **(S31-a, frozen)** records: `currency { name, symbol, exponent, symbol_at, references }` about the observer; `wealth { amount, unvalued }` to the holder; a person's `asks { asks: [{ item, price, available }] }` to whoever perceives them; the `shop` listing keeps its shape `{ operator, listed: [{ item, price, in_stock }] }` (SD-S31a-12) | `pr-s31a-currency-asks.md` §§5.3, 5.8, 5.9 |
| A-25 | **(S31-a)** offers: one complete `buy { item, price }` per ask (target-less in a shop, targeted at a person seller), refused with its named code when it cannot happen; `list-for-sale`, `set-price`, `withdraw` offered **incomplete**, at most once per type, target-less | `pr-s31a-currency-asks.md` §§5.6, 5.7 |
| A-26 | **(S31-b, implementation branch `mvp0/pr-s31b` @ f6ef0ee2)** records `subscriptions { topics }` (holder only) and `notice-board { notices: [Notice] }`, `Notice { id, topic, publisher, body, issued_at, valid_until }`, body `{"prices": [{item, price, basis}]}` or `{"text": "…"}` | `/Users/yuema137/mineworld-worktrees/impl-s31b/systems/notices/src/component.rs:13-75`, `board.rs:42-50`, `body.rs:20-90` |
| A-27 | **(S31-b)** the board lists the current notices of the topics the observer subscribes to or publishes; topics are configured `SystemInternal` and never disclosed as a list | `pr-s31b-notices.md` §§5.2, 5.8 |
| A-28 | Only Market Town enables `economy`; Grace holds `{ candle, soap, umbrella }` and a wallet, no asks; `items/flowers.yaml` exists; `configure/` holds `calendar.yaml`, `weather.yaml` | `worlds/market-town/people/grace.yaml`; `worlds/market-town/items/`; `worlds/market-town/configure/` |
| A-29 | Real-client test helpers: `hosted`, `ids`, `facts`, `play`, `menus`, `steps_file`, `copy_dir`; a stub server that serves hand-built records (`record`, `offering`) | `tools/cli/tests/godot2d/worlds.rs:19-141`; `godot2d/mod.rs:389-535` |
| A-30 | 2D harness steps (`open`, `choose`, `await`, `shoot`, …) resolve selectors against rendered entries and press them through the real path; reports `MENU`, `PANELS`, `TOAST`, `STEP` | `clients/2d/scripts/harness/interact.gd:1-60` |

### 3.2 Findings

- **F-S31e-1 — Two money formatters, both assuming two decimals** (A-1, A-19). 2D `words.gd` and 3D (16d)
  `things.gd` each divide by 100. A world with `exponent: 0` shows "0.03 cr" for 3 crowns (X-10). The fix is one
  formatter that reads the disclosed exponent.
- **F-S31e-2 — There is no shared home for reading game records.** The settings module holds wording and the
  clock formatter (A-16); the protocol module is reserved to the protocol PRs (16d's ruling 4); 16d recorded the
  duplicated readers as a later coordinated PR (A-22). Step §12 requires "one shared formatter and one panel
  model". A new shared module, `clients/shared/trade`, is the cheapest honest home (A-15) → **Q-S31e-1**.
- **F-S31e-3 — The step names `check_client_rules.py`, which scans only `clients/2d`** (A-13). The 3D client and
  the shared modules are scanned by `client_rules.rs` (A-14). Putting the two new patterns there covers both
  clients and the new module with one rule → **Q-S31e-3**.
- **F-S31e-4 — Before this PR, S31-a's records show raw.** After S31-a merges, 2D shows `currency`, `wealth` and
  `asks` as raw JSON under "other" (A-3's `I-3`), and both clients list `list-for-sale`, `set-price`,
  `withdraw` as disabled "not supported" entries (R-S31a-3). That is correct and temporary; this PR gives them
  readers and words.
- **F-S31e-5 — The shop listing does not name its keepers** (A-24, SD-S31a-12). A client cannot tell a keeper
  from a visitor. It does not need to: a `for: shop` action from a non-keeper is refused `PermissionDenied` by
  the server. The stall shows the shop's section to anyone offered the seller actions in a shop, collapsed
  behind one button (SD-S31e-5).
- **F-S31e-6 — The topic list is not disclosed** (A-27). A client cannot offer "subscribe to …" without a list,
  and typing a topic id is not a player experience → **Q-S31e-2**.
- **F-S31e-7 — Market Town has no person seller before S31-f** (A-28). AC-S31-15's "walk to Grace, buy flowers"
  cannot run on Market Town as it stands after S31-a. The drive uses a generated variant in which Grace holds
  and asks for flowers, and S31-f re-runs the same drive on its content → **Q-S31e-4**.
- **F-S31e-8 — No second economy world exists** (A-28). Every acceptance criterion therefore names a generated
  variant: a scratch copy of Market Town with another currency, other sellers and other topics, written by the
  test (16d's D-16d-10 and S31-a's AC-a-6 precedent; `copy_dir`, A-29). No file under `worlds/` changes.
- **F-S31e-9 — 3D has no typed input** (16d Q-16d-7: `talk` sends a default utterance). The price field is new
  in 3D: a `LineEdit` in the trade panel, held by 16d's input gate while open.
- **F-S31e-10 — An incomplete seller offer carries no item** (A-25). The player supplies the item and the price.
  For one's own stall, the rows are one's held kinds (the `holdings` record) and one's asks; for a shop, the
  rows are the shop's listing, and "list another kind" picks from the place's `item-catalogue`. The server still
  answers `not-held`, `already-listed`, `not-for-sale`, `price-out-of-range`.
- **F-S31e-11 — A lexical "wallet vs price" check must not misread `->`.** GDScript's return arrow contains `>`.
  The lexer already separates strings and comments; check d tokenizes comparison operators explicitly (§5.11).
- **F-S31e-12 — New composed types need literals and admissions.** Composing `list-for-sale`, `set-price`,
  `withdraw` names them in each client's `intents.gd`; R2 reads `COMPOSED` (2D) and check a needs six new
  `ACTION_LITERALS` entries (three per client).

### 3.3 What does not exist

No client reads `currency`, `wealth`, `asks`, `subscriptions` or `notice-board`; no client composes a seller
action; no client draws a price tag; no client test drives a person-to-person sale; no world variant with a
second currency exists. `git grep -n "mineworld_trade\|MineWorldMoney\|MineWorldTrade"` on the base: no match.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| Money formatting | Godot's `String.num`, `%d` formats; ICU number formatting (not in Godot's GDScript API); CLDR currency patterns | **Godot built-ins only.** Integer division and zero padding are the whole job; the separators and the symbol's place are wording keys. CLDR patterns would duplicate the world's own `symbol_at` and cannot know a fantasy coin. No dependency, so no `DEP` record |
| Parsing a typed amount | `String.is_valid_int`, `String.to_int` | **Built-ins.** The decimal part is split and padded by hand to the exponent; no float ever touches an amount |
| A UI toolkit for panels | Godot `Control` nodes, as both clients already use | **Reuse.** No new look: 2D's warm panels, 3D's dark HUD panels (16d D-16d-8) |
| A second shared reader module vs the protocol module | `clients/protocol/mineworld` (the protocol module) | **Not there.** The protocol module reads frames for any game; trade records are one pack family's shapes. A sibling module keeps the protocol module game-neutral (Q-S31e-1) |

---

## 5. Design

### 5.1 The module (`clients/shared/trade/`, `res://mineworld_trade`)

```text
clients/shared/trade/
  money.gd     MineWorldMoney   format and parse amounts from a disclosed currency           (≈ 120 lines)
  trade.gd     MineWorldTrade   readers of the trade records; the row model both clients draw (≈ 350 lines)
clients/shared/TRADE.md         the module's specification (agents' document, §2.1 of CLAUDE.md)
clients/shared/checks/money_check.gd, trade_check.gd                                       (headless)
clients/2d/mineworld_trade        -> ../shared/trade      (symlink)
clients/3d-spike/mineworld_trade  -> ../shared/trade      (symlink)
```

- **What it may know:** the JSON *shapes* of the records S31-a and S31-b disclose, the shape of an affordance,
  and wording keys. It never knows an action type by name, an item, a topic, a currency or a coordinate.
- **What it may not do:** decide whether something is allowed, compare two amounts, read `references`, compute
  expiry, submit anything. Submitting stays in each client's `intents.gd` (ARC-47 R1).
- **Dependencies:** `MineWorldText` (settings module) for wording; `MineWorldObservation` (protocol module) for
  reading frames. Both clients already take both modules.
- **Platforms:** no `OS.get_name()`, no feature branch; the launchers and the nightly Windows symlink check
  list the new module beside the others (A-15).

### 5.2 Money: one formatter, one parser (`MineWorldMoney`)

**The currency value.** `MineWorldMoney.currency_of(observation) -> Dictionary` reads the observer's own
`currency` record (`own_component("currency")`) and keeps only `{ name, symbol, exponent, symbol_at }`. It
never returns `references` (I-e-4). With no record, it returns `{}`.

**Formatting.** `format(amount: int, currency: Dictionary) -> String`:

1. With `currency` empty (no economy, or a frame before the record arrives): the amount is shown as a bare
   integer through `format.money.plain` = `"{amount}"`. Nothing is divided, because no scale was disclosed.
2. Otherwise, with `e = exponent` (0 … 4): `major = amount / 10^e`, `minor = amount % 10^e`. `major` is grouped
   in threes with `format.money.group_mark`. If `e > 0`, `minor` is zero-padded to `e` digits and joined with
   `format.money.decimal_mark`. A negative input (never sent by the server; amounts are `u64`) keeps a leading
   `-`.
3. The symbol is placed by the world's `symbol_at`: `format.money.before` = `"{symbol}{amount}"`,
   `format.money.after` = `"{amount} {symbol}"`. The wording decides spacing; the world decides the side.

| Currency (as a world declares it) | Amount (minor units) | Shown (en) |
| --- | --- | --- |
| `{ US dollar, "$", 2, before }` (Market Town after S31-a) | 200 000 | `$2,000.00` |
| the same | 300 | `$3.00` |
| `{ crown, "cr", 0, after }` (AC-S31-6's rename) | 3 | `3 cr` (X-10: never `0.03 cr`) |
| `{ dinar, "KD", 3, before }` | 1 500 | `KD1.500` |
| `{ unidad, "UF", 4, after }` | 12 345 | `1.2345 UF` |
| any | 1 000 000 000 000 (`PRICE_MAX`), e = 2 | `$10,000,000,000.00` |

**Parsing a typed price.** `parse(text: String, currency: Dictionary) -> Variant` returns an `int` of minor units,
or `null` when the text is not an amount in this currency:

- leading and trailing spaces are dropped; the symbol, if typed, is dropped; group marks are dropped;
- at most one decimal mark (`.` or the wording's `decimal_mark`), with at most `e` digits after it, padded
  with zeros to `e`; with `e = 0`, any decimal mark is `null`;
- digits only otherwise; no sign; at most 18 digits in total, so the value fits a GDScript `int`;
- `currency` empty → `null` (no scale, so no conversion).

Example: in dollars, `"2.6"` → 260, `"2.60"` → 260, `"2.605"` → `null`; in crowns, `"12"` → 12, `"1.5"` →
`null`; `"0"` → 0. **The parser checks representation, not rules.** `0` and an amount above `PRICE_MAX` parse and
are sent, and the server refuses them `price-out-of-range` (AC-e-5). A `null` is not sent; the field says
`trade.price.not-an-amount` and stays open.

### 5.3 The records read (`MineWorldTrade` readers)

Every reader takes a `MineWorldObservation` and returns plain data. Each shape is pinned from a live frame at
C1 (16d's rule, A16d-7), never from the Rust struct; the table states the expected shapes.

| Reader | Record (owner) | Where in the frame | Returns |
| --- | --- | --- | --- |
| `currency_of(obs)` | `currency` (economy) | the observer's own components | `{name, symbol, exponent, symbol_at}` or `{}` |
| `money_of(obs)` | `wallet` (economy) | own | `{balance}` or `{}` |
| `wealth_of(obs)` | `wealth` (economy) | own | `{amount, unvalued}` or `{}` |
| `held_of(obs)` | `holdings` (inventory) | own | `[{item, count}]` |
| `own_asks(obs)` | `asks` (economy) | the observer's own entity, if disclosed to itself | `[{item, price, available}]` |
| `asks_of(obs, person)` | `asks` (economy) | `component_value(person, "asks")` | `[{item, price, available}]` |
| `listing_of(obs)` | `shop` (economy) | `component_value(obs.place(), "shop")` | `{operator, listed: [{item, price, in_stock}]}` or `{}` |
| `catalogue_of(obs)` | `item-catalogue` (item, 12d) | the observer's place | `[{item, category, name}]` |
| `board_of(obs)` | `notice-board` (notices) | own | `[Notice]` in the record's order |
| `topics_of(obs)` | `subscriptions` (notices) | own | `[topic]` |

If the observer's own `asks` is not disclosed to the observer itself (C1 checks the live frame), `own_asks`
reads `asks_of(obs, obs.observer())`; if neither exists, own asks are not shown and C1 records a finding for
S31-a (a holder must see its own asks; that is a server fix, never a client inference).

### 5.4 The row model (the interface both clients draw, and S31-h extends)

A **row** is one line of a trade view. A **button** on a row is one thing the player can do. Rows are plain
dictionaries, so both clients draw the same data and the harness prints it as is.

```text
Row    = { kind: "sale" | "ask" | "held" | "shelf" | "answer" | "note",
           subject: String,            the seller (a person id), "" for the shop here, the observer for own rows
           item: String,               an item id, "" when none
           text: String,               already worded, names from the world, amounts through MineWorldMoney
           amount: int | null,         the price shown, in minor units (data for reports; never compared)
           available: bool | null,     the server's verdict on the row's offer, shown only
           reason: Variant,            the server's reason, shown only
           buttons: [Button] }
Button = { label: String,              already worded
           affordance: Dictionary,     the offer this button acts on, exactly as the frame carried it
           fill: Dictionary,           payload fields the row already knows (item, for, later: proposal)
           ask: PackedStringArray }    what the player still supplies: "amount", "item" (a picker), or nothing
```

**Pressing a button** (each client, one path for click, Enter and the harness):

1. If `affordance` is complete and `ask` is empty → `intents.submit_offered(affordance)`, unchanged (ARC-34).
2. Else the client collects `ask`: an item from `catalogue_of(obs)` (a picker), an amount through the price
   field and `MineWorldMoney.parse`. Then `intents.compose(affordance, fill ∪ { item, price })`. `intents.gd`
   alone maps those values to the action's payload (§5.6).
3. The client never reads `available` or `reason` in this path. Both are shown when the row is built (R3').

**Builders** (`MineWorldTrade`), each taking `inputs: Dictionary` = the client's own table of the action types it
composes and what each asks for (2D `intents.INPUTS`, 3D `SliceIntents.INPUTS`), so the module names no action:

| Builder | Rows | Built from |
| --- | --- | --- |
| `sale_rows(obs, subject, inputs)` | one `sale` row per **priced offer** of the subject, in the server's order; button *Buy* = that offer | priced offer = a complete affordance whose payload carries an integer `price` and a typed item reference; `subject ""` = target-less offers (the shop where I stand), else offers targeted at that person |
| `stall_rows(obs, inputs)` | `ask` rows (my asks: *Change price…*, *Withdraw*), `held` rows (held kinds I do not ask for: *List for sale…*) | `own_asks`, `held_of`, and the observer's incomplete offers whose type `inputs` marks as a seller action |
| `shelf_rows(obs, inputs)` | `shelf` rows (the shop's listing: *Change price…*, *Withdraw*, each with `fill.for = "shop"`), one `note` row with *List another kind for the shop…* (`ask: ["item", "amount"]`) | `listing_of` and the same incomplete offers; shown only while a listing is disclosed here and those offers exist (SD-S31e-5) |
| `answer_rows(obs, inputs)` | none in S31-e: the function exists and returns `[]` | S31-h fills it from its `proposals` record (§5.13) |

`inputs` entries for S31-e (in each client's intents file, never in the module):

```text
"list-for-sale": { "ask": ["amount"], "row": "held" | "note" }      the row fills item (and for)
"set-price":     { "ask": ["amount"], "row": "ask" | "shelf" }
"withdraw":      { "ask": [],         "row": "ask" | "shelf" }
```

The module matches an offer to a row by the `row` tag the client's table gives that offer's type. It never
spells a type. Example: the 2D table says `set-price` serves `ask` and `shelf` rows, so each such row gets a
*Change price…* button whose `affordance` is the one incomplete `set-price` offer and whose `fill` is
`{item}` (or `{item, for: "shop"}` on a shelf row).

### 5.5 The sale panel

- **When it shows.** Beside the interaction menu, whenever the menu's subject has at least one priced offer:
  in 2D when the menu opens on a person (click, E) or on oneself (Q) while standing in a shop; in 3D when 16d's
  B (your menu, in a shop) or Tab (the person under the ray) opens a menu. It closes with the menu. This is the
  step's "the existing interaction prompt opens it" in both clients.
- **Rows.** `sale_rows(obs, subject, inputs)`: "Flowers · 4.50 cr · available", or greyed with the reason's words
  ("Bread · $3.00 · not enough money"). The amount is the offer's own `price` (the asking price the server put in
  the payload), formatted. The row's *Buy* sends that offer unchanged; a greyed row's *Buy* is still pressable and
  the server answers (16d O-3's rule).
- **After an answer.** The toast is worded from the result (A-12). The panel is rebuilt from the next frame:
  the wallet line, the holdings and the row's `available` all come from the server.
- **Example.** Bob perceives Grace, who asks 450 cr for flowers, holds two and stands 2 m away. Bob clicks Grace's
  figure in 2D (in 3D: looks at her and presses Tab) → her menu plus the sale panel: "Flowers · 450 cr · *Buy*". He presses *Buy*;
  the request is Grace's `buy { item: flowers, price: 450 }` affordance, byte for byte. If Grace re-priced to 500
  a moment earlier, the server answers `price-changed`, the toast says "the price has changed", and the next frame
  shows the row at 500.

### 5.6 The stall

- **Where.** A section of the trade panel shown with one's own menu (2D Q, 3D B), when `stall_rows` or
  `shelf_rows` returns rows.
- **Rows and buttons** (§5.4): my asks with *Change price…* and *Withdraw*; my held kinds without an ask with
  *List for sale…*; in a shop, a collapsed *This shop's prices…* button that reveals the shelf rows (SD-S31e-5).
- **The price field.** 2D uses the typed line (`talk_line.ask`, A-7) with the prompt
  `trade.price.prompt` ("Price for {item}, in {currency}") and the current price as the suggestion. 3D uses a
  `LineEdit` in `SliceTradePanel`, focused when shown, Enter to send, Esc to cancel, gameplay input held by 16d's
  gate. The text goes through `MineWorldMoney.parse`; a `null` keeps the field open with
  `trade.price.not-an-amount`.
- **Payloads, built only in `intents.gd`** (both clients), from `fill ∪ { item, price }`:

  | Type | Payload |
  | --- | --- |
  | `list-for-sale` | `{ item: <typed item reference>, price: <int> }`, plus `for: "shop"` when filled |
  | `set-price` | the same |
  | `withdraw` | `{ item }`, plus `for: "shop"` when filled |

  The item reference is written in the shape the server's `buy` offers use for `item` (copied from a live
  offer at C1, never from a Rust struct). `target` is `null` (A-25). 2D sends `actor_location: null`; 3D sends
  its `actor_location()` (16d D-16d-7).
- **Example (crown variant).** The visitor holds a scarf. Stall: "Scarf × 1 · *List for sale…*". She types `12`
  → `list-for-sale { item: scarf, price: 12 }`; accepted; the next frame shows "Scarf · 12 cr · *Change price…*
  · *Withdraw*", and Bob's client, beside her, shows a tag "Scarf 12 cr" over her figure.

### 5.7 Money on the HUD

- **2D:** `status.gd` gains a fourth line `hud.money` = "{money}" or, with a `wealth` record, `hud.money.wealth`
  = "{money} · worth {wealth}" (+ `hud.money.unvalued` "({count} unvalued)" when `unvalued > 0`). The I panel's
  `panel.wallet` row uses the same formatter. The step's "on hover" becomes a visible second clause, because a
  hover cannot be driven or shot reliably in either client (SD-S31e-6).
- **3D:** `slice_main.gd` adds the same line with `ControlsHud.add_line`, rendered each frame from its key like
  16a's lines. `things.gd`'s wallet row calls `MineWorldMoney.format`.
- With no `wallet` record the line is absent. Social Café shows nothing new (AC-e-11).

### 5.8 Price tags

- **On a person who sells.** 2D: `people.gd` gives each figure a tag label under the participation marker,
  filled from `asks_of(obs, id)`: "Flowers 450 cr" for one ask, "Flowers 450 cr · +2" for more (`trade.tag.more`).
  An ask with `available: false` is shown with `trade.tag.sold-out`. 3D: `figures.gd` adds a second `Label3D`
  under the name, same text, faded by camera distance between `TAG_FADE_FROM_M = 6.0` and `TAG_FADE_TO_M = 12.0`
  (presentation constants; no rule word in their names, check b).
- **At a shop.** 2D: `places.gd` draws a board at the shop place's façade anchor (A-10): the listing's first
  four rows "Bread $3.00", then `trade.tag.more`. 3D: `SlicePriceBoards` puts the same text on a `Label3D` above
  the door that 16c's `SliceLayout` bound to the shop's place; a shop whose place is bound to no drawn door gets
  no board (16c R-16c-1: nothing is guessed). The listing is read from the latest frame that disclosed it; a
  board for a place not in view keeps its last rows, dimmed (presentation memory, like 13b's name cache), and is
  dropped on an instance change.
- No tag is computed: each shows a disclosed price, formatted. No tag compares anything.

### 5.9 The notice board (needs S31-b; C6)

- **Panel.** 2D: a third panel group toggled by **N** (`Controls2D`, `ui.settings.key.notices`). 3D: **N** opens
  `SliceBoardPanel` (16d's look, input held while open). Rows from `board_of(obs)`, in the record's order:
  - title: "{topic} · {publisher} · until {time}" (`board.row.title`), the publisher by display name, the time
    through `MineWorldClockFormat` (day and time);
  - a `text` body as one line;
  - a `prices` body as one line per price line, "Bread $2.80 (6 seen)" (`board.row.price`), amounts through
    `MineWorldMoney`, item names through the catalogue or the last name seen, never an id.
- **The record decides what is current** (R-S31e-b-1, I-S31-8). The client shows every notice the record holds
  and nothing else. It never compares `valid_until` with the clock (M-e-3).
- **Badge.** The status line (2D) and a HUD line (3D) show `board.badge` "{count} new" when the record holds
  notice ids not in the set shown when the board was last opened. The set lives in memory only and is dropped
  on an instance change.
- **No subscribe or publish buttons** (Q-S31e-2; §2.2). The `subscribe`, `unsubscribe` and `publish-notice`
  offers stay listed in the menus as disabled "not supported by this client" entries.

### 5.10 Words

All new keys go into the shared layer (`clients/shared/settings/locale/`), `en` and `zh_Hans`, and
`messages.pot`; `client_text.rs` checks that each used key exists in both (16d G-4).

| Family | Keys |
| --- | --- |
| money | `format.money.plain`, `format.money.before`, `format.money.after`, `format.money.group_mark`, `format.money.decimal_mark` (`format.money` is removed from both layers; `PRESENTATION.md` §7 says so) |
| HUD | `hud.money`, `hud.money.wealth`, `hud.money.unvalued`, `board.badge` |
| trade panel | `panel.trade`, `panel.stall`, `trade.row.sale`, `trade.row.ask`, `trade.row.held`, `trade.row.shelf`, `trade.buy`, `trade.change-price`, `trade.withdraw`, `trade.list`, `trade.list-for-shop`, `trade.shop-prices`, `trade.price.prompt`, `trade.price.not-an-amount`, `trade.item.pick`, `trade.tag.more`, `trade.tag.sold-out` |
| board | `panel.board`, `board.row.title`, `board.row.price`, `board.empty` |
| actions | `action.list-for-sale`, `action.set-price`, `action.withdraw` and their `.done` forms (with `{item}`) |
| refusals | `reason.not-for-sale`, `reason.price-changed`, `reason.insufficient-funds`, `reason.out-of-stock`, `reason.cannot-carry`, `reason.already-listed`, `reason.price-out-of-range`, `reason.not-held`, `reason.price-unchanged` (S31-a §5.5, SD-S31a-14); `reason.unknown-topic`, `reason.body-mismatch`, `reason.body-too-large`, `reason.body-invalid`, `reason.validity-out-of-range`, `reason.over-daily-limit`, `reason.already-subscribed`, `reason.not-subscribed` (S31-b §5.7). Kernel reasons (`too_far_away`, `permission_denied`, …) are already worded |
| keys | `ui.settings.key.notices` (2D and 3D key maps) |

`REASON_KEY` in `check_client_rules.py` (A-13) already accepts hyphenated codes; `--check-pack` stays green.

### 5.11 The client-rules scan: check d (X-8)

`client_rules.rs` gains check **d**, over every `*.gd` it already reads (2D, 3D, the shared modules), comments
removed by the existing lexer:

- **d1 — money compared.** In one top-level function, a **comparison token** (`<`, `>`, `<=`, `>=`, as lexer
  tokens; `->`, `<<`, `>>` and characters inside strings are not comparisons) together with a **wallet word**
  (`balance`, `wallet`, `wealth`, as an identifier or a string literal) and a **price word** (`price`, `amount`,
  `ask`, `cost`, as an identifier or a string literal). Finding: "compares money: a client decides nothing about
  prices (step-31 §12, X-8)".
  - Example caught (X-8's plant): `if int(row["amount"]) > int(MineWorldTrade.money_of(obs).get("balance", 0)):`
    in the 2D sale panel's builder.
  - Example not caught: `MineWorldMoney.format`, whose loop compares a digit counter with 3 but mentions no
    wallet word.
- **d2 — reference read.** Any string literal `"references"` or `"reference"`. Finding: "reads a reference value:
  never charged, never computed with in a client (step-31 §6.1, I-S31-6)".
- **Admissions.** `MONEY_ADMITTED: [(&str, &str, &str); 0]` (file, function, why); an admission that admits
  nothing fails, as checks a and b do. None is needed by this design.
- **Check a admissions** (F-S31e-12): `list-for-sale`, `set-price`, `withdraw` in `clients/2d/scripts/intents.gd`
  and in `clients/3d-spike/scripts/slice/intents.gd`, each with "composed by the client's only submitter from the
  player's item and price (S31-e §5.6)".
- `check_client_rules.py` R2 needs no change: it reads `COMPOSED` from `intents.gd`.
- A self-test beside the existing `the_lexer_reads_strings_and_not_comments` asserts d1 on the X-8 line and not on
  a `func f() -> int:` line.

### 5.12 The test: worlds, harness, reports

**Generated variants** (written by `client_trade.rs` into a temporary directory with `copy_dir` and whole-file
writes; nothing under `worlds/` changes; F-S31e-8):

| Variant | Made from Market Town by | Purpose |
| --- | --- | --- |
| `V-usd` | nothing (Market Town as S31-a leaves it: US dollar, exponent 2) | the default world |
| `V-crown` | `configure/economy.yaml` → `{ crown, cr, 0, after }`; Grace `holdings` + `flowers: 2`, `economy: { wallet: 200000, asks: { flowers: 450 } }`; the corner store's shop gains `keepers: [felix]`; with S31-b: `configure/notices.yaml` with topics `news.street` (text, publisher `hana`) and `prices.street` (prices, publisher `carol`), and `notices: { subscribe: [news.street, prices.street] }` on the visitor | another currency (exponent 0, after), a person seller, a keeper, other topic names |
| `V-dinar` | `configure/economy.yaml` → `{ dinar, KD, 3, before }`; Ivan `asks: { umbrella: 12500 }` with an umbrella held | exponent 3, another seller and kind |
| `V-none` | `worlds/social-cafe` unchanged | a world without economy or notices |

**Harness** (both clients, 13b's step format, A-30; 16d shares it with the 3D probe):

```text
steps:   {"trade": "open", "of": "<person id>" | "here" | "self"}     open the menu on the subject; the trade
                                                                       panel must appear (or must not, "absent": true)
         {"trade": "press", "row": {kind, item}, "button": "<index>",  press a button through the real path
          "amount": "<typed text>", "item": "<id>"}                    (the typed text goes into the price field)
         {"board": "open"}                                             press N
reports: TRADE {subject, rows: [Row without affordance bodies; buttons as {label, action_type, complete, ask}], t_ms}
         BOARD {notices: [{id, topic, publisher, lines}], badge, t_ms}
         HUD {money, tags: [{subject, text}], t_ms}                    the money line and the drawn tags
```

The steps name ids, item ids and typed text that the Rust test resolves from genesis (`ids`, A-29); no `.gd`
file names a world's item, person or topic. Selectors never match by label text (16d M16d-13).

### 5.13 Haggling rows: the interface S31-h uses (R-S31e-h-1)

If S31-h has merged before C0, C7 adds its rows here; otherwise S31-h's C8 adds them, with no change to the
row model, the views or the module's public functions:

1. **Offer a price…** on each `sale` row: S31-h adds `"propose-price": { "ask": ["amount"], "row": "sale" }` to
   each client's `INPUTS` and `COMPOSED`. `sale_rows` already attaches, to each `sale` row, one button per
   incomplete offer **against the same subject** whose `row` tag is `sale`, with `fill = { item }`. Nothing else
   changes. Words: `sale.offer_price` (S31-h's key).
2. **Incoming and outgoing offers:** S31-h implements `answer_rows(obs, inputs)` in `trade.gd`, reading its
   `proposals` record, and returns `answer` rows: text "Bob offers 2.55 for bread (ask 3.00)", buttons *Accept*
   and *Decline* (incomplete offers targeted at the buyer, `fill = { proposal: <id> }`, `ask: []`), or *Withdraw*
   for an outgoing line. Both views already draw `answer` rows (any number of buttons) above the sale rows; C3 and
   C5 test that with a hand-built `answer` row in `trade_check.gd` and a stub frame.
3. **Enabled exactly as the server offers** (S31-h §5.13): an *Accept* button carries the `accept-proposal` offer
   for that buyer; its `available` and `reason` are shown on the row; pressing sends regardless.
4. `intents.compose` maps `proposal` from `fill` into the payload, as it maps `item` and `for` here.

### 5.14 The agent view and MCP agents

Nothing here. An MCP agent sees the same records through S31-g (`WorldLine.currency`, `MeLine`, typed kinds of
the one `act` tool under `ARC-153`) and acts on the same server path. The two clients and an agent therefore
see the same prices and refusals, because all three read the server's records.

---

## 6. Invariants this PR holds

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-e-1 (I-S31-1) | No path under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/`, `cognition/`, `worlds/`, `clients/protocol/`, `clients/agent/` changes | the close-out diff scan |
| I-e-2 (I-S31-12) | No client evaluates a trade rule: no comparison of money, no reference read, no expiry computed, no row hidden for being unavailable | check d (§5.11); AC-e-4 and AC-e-8 arms; R3' |
| I-e-3 | One formatter and one row model: neither client divides an amount or reads a trade record except through `mineworld_trade` | a grep at close-out for `/ 100`, `% 100`, `"balance"`, `"listed"`, `"asks"`, `"notice-board"` outside `clients/shared/trade/` and the harness |
| I-e-4 (I-S31-6) | No client script reads `references` | check d2 |
| I-e-5 (I-S31-11) | No currency name, symbol, item, person, topic or coordinate of any world appears in client code | the existing `currency_names.rs` (S31-a) extended to `clients/shared/trade`; review |
| I-e-6 | No entity id on screen: items, people and publishers are named from the disclosure or the unnamed word | AC-e-9 |
| I-e-7 | A complete offer is sent exactly as the server made it; a composed one only through `intents.gd` | check a, R1, R2; AC-e-12's request comparison |
| I-e-8 | A world without `economy` and `notices` draws and sends exactly what it did before | AC-e-11 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | GDScript parse (Godot import), check a/b/d, R1–R6, wording files well formed and complete | `client_rules.rs`, `check_client_rules.py`, `client_text.rs`, `--check-pack` |
| Unit (headless Godot, no server) | `format` and `parse` over the currency table of §5.2, both placements, every exponent, group marks, `null` cases; row builders over hand-built frames (priced offers, stall, shelf, board, a stub `answer` row) | `clients/shared/checks/money_check.gd`, `trade_check.gd` |
| Real client against a stub server | refusal words for every code; a stale board record still shown (M-e-3); no economy → no panels | `client_trade.rs` (stub frames, A-29) |
| Real client against a real server (Gate 2 class) | AC-e-2 … AC-e-12 on the variants | `client_trade.rs` (`#[ignore]`, milestone) |
| Frames | AC-e-13 | windowed runs at the milestone; 3D frames reviewed by Opus |
| Gate 1 | NOT REQUIRED: no LM surface | — |

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`; run at the milestone, charter 2026-10-18)

Every criterion names its worlds (F-S31e-8). "Both clients" means the 2D client and the 3D slice, each against
its own fresh server.

| Id | Criterion | Worlds | Step ref |
| --- | --- | --- | --- |
| AC-e-1 | **One formatter, any currency.** `money_check.gd`: every row of §5.2's table formats exactly as written; `parse(format(x))` = x for 200 sampled amounts per currency (SplitMix64 seed 31, 0 … `PRICE_MAX`); each `null` case of §5.2 is `null`; with `{}` the amount is a bare integer | the table's four currencies | AC-S31-6 |
| AC-e-2 | **The currency renames with zero client code.** Both clients, seat `visitor`: the HUD money line reads `$2,000.00` on V-usd and `200,000 cr` on V-crown; the café's coffee row reads `$3.00` and `300 cr`; no client file differs between the runs | V-usd, V-crown, V-dinar (`KD200.000`) | AC-S31-6 (client half); X-10 |
| AC-e-3 | **Buy at a shop.** Both clients: walk to the store, the shop board shows bread's price; open the menu, the sale panel's bread row equals the listing's price; *Buy*; within 2 s the HUD money line is lower by exactly the save's `money-transferred.amount`, which equals the row's `amount` | V-usd, V-dinar | AC-S31-15 |
| AC-e-4 | **Buy from a person, and greyed rows stay pressable.** Both clients: walk to Grace, her tag reads "Flowers 450 cr"; her sale panel row; *Buy* → `sold { price: 450 }`, the money line −450 cr. Then on a copy of V-crown whose visitor has `wallet: 100`, the row is shown with `insufficient-funds`'s words, *Buy* still sends, the server refuses, and no wallet changes | V-crown (Grace), V-dinar (Ivan, umbrella) | AC-S31-15 |
| AC-e-5 | **The stall.** Both clients, visitor: list the scarf at `12` → `list-for-sale { scarf, 12 }` accepted; change to `15` → `set-price` accepted; type `1.5` → not sent, the field says "not an amount"; type `0` → sent, refused `price-out-of-range` with its words; withdraw → accepted. On V-usd, `2.6` is sent as 260 | V-crown, V-usd | step §12 "My stall" |
| AC-e-6 | **A keeper's shelf, seen by another client.** 2D on Felix's seat changes the store's bread to `4` through *This shop's prices…*; the 3D client on Bob's seat, outside the store, shows the store board "Bread 4 cr" within 2 s of the next frame that discloses the store. The visitor pressing the same button in the store is refused `PermissionDenied` | V-crown | step §5.1 keepers |
| AC-e-7 | **Refusals in words.** For each code of §5.10's refusal row: a stub frame offering a refused priced offer (S31-a codes) and a stub result (S31-b codes) → both clients show the `en` words, and with `--settings` zh_Hans the `zh_Hans` words; no code is shown raw | stub frames over V-crown's ids and V-usd's | AC-S31-15 |
| AC-e-8 | **The board.** With S31-b: Hana publishes a text notice and Carol a prices notice through the protocol test client (S31-b AC-b-8's); both clients show both on the board with the badge "2 new"; the prices line reads in crowns; after opening, the badge is gone. A stub frame whose board record holds a notice with `valid_until` before `at()` still shows it (the record is the authority, M-e-3) | V-crown (with notices); stub | R-S31e-b-1; AC-S31-13 (b) client view |
| AC-e-9 | **No id on screen.** At each step of AC-e-3 … AC-e-8, a walk of every visible `Label`, `Button` and `Label3D` (16d N-2's rule) finds no whole-word token equal to an id the session has seen | all of the above | I-e-6 |
| AC-e-10 | **The scan holds.** `client_rules.rs` is green on the head with no `MONEY_ADMITTED` entry; X-8's plant fails it naming d1; a planted `.get("references")` fails it naming d2 | the code | X-8; I-S31-12 |
| AC-e-11 | **No economy, nothing new.** On V-none both clients' `MENU`, `PANELS` and `HUD` reports equal the base build's for 13b's and 16d's existing steps; no `TRADE` or `BOARD` panel opens; no money line, no tag | V-none | AC-S31-9 (client half) |
| AC-e-12 | **Both clients, one model.** For the subjects of AC-e-3 … AC-e-5, the 2D and 3D `TRADE` reports are equal after dropping labels (kind, subject, item, amount, available, reason, the buttons' action types and asks), and each pair of requests sent by a press has `differing_fields ⊆ {ActorLocation}` (16d P-1's comparison) | V-crown, V-usd | step §12 "one panel model" |
| AC-e-13 | **Shots on three OSes.** Windowed, 1600×900, per client: the store board, a person's tag, the sale panel, the stall with the price field, the HUD money line, the board, the same in zh_Hans; macOS, Linux, Windows. The 3D frames are reviewed by an Opus-class reviewer and then by the operator; 16d's accepted frames do not move except where these elements appear (16d V-1, V-3) | V-crown, V-usd | AC-S31-15 |

### 8.1 Planted mutations (milestone; each must turn its test red by name, then be reverted from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-8 | the 2D sale panel greys out and disables *Buy* when the row's amount exceeds the wallet balance | AC-e-10 (check d1) and AC-e-4's poor-visitor arm (no request sent) |
| X-10 | `MineWorldMoney.format` divides by 100 regardless of `exponent` | AC-e-1 and AC-e-2 (`0.03 cr`-style text on V-crown) |
| M-e-1 | `sale_rows` drops rows with `available: false` | AC-e-4's poor-visitor arm |
| M-e-2 | `parse` multiplies by 100 instead of `10^exponent` | AC-e-5 (V-crown `12` sent as 1200) and AC-e-1 |
| M-e-3 | the board view drops notices whose `valid_until` is before `at()` | AC-e-8's stub arm |
| M-e-4 | 3D `things.gd` keeps its own `/100` | AC-e-2's 3D arm |
| M-e-5 | 2D `intents.compose` takes the item from the row's label text | AC-e-12 (Payload differs) and AC-e-5 |
| M-e-6 | the module reads `currency.references` and shows a "typical" price | AC-e-10 (check d2) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and its evidence; an inapplicable item is `N/A` with the audited reason. Per the charter, each
commit's validation is the build (Godot parse of the touched projects) plus the tests that commit writes, run
headless; the drives, shots, mutations and `client_trade.rs` run at the milestone.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31e`. Never `cd` elsewhere.

1. Re-read this commit's section, the design sections it cites, and every file you will edit, in full.
2. Make the numbered steps in order; do no later commit's work.
3. Godot runs only through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`. Parse:
   `godot --headless --path clients/<project> --import` for each touched project (`shared`, `2d`, `3d-spike`);
   PASS iff no "Parse Error", "SCRIPT ERROR" or "Cannot get class" line.
4. Static: `python3 scripts/check_client_rules.py`; `python3 scripts/check_client_rules.py --check-pack
   presentation/mineworld-default/2D`; `cargo test -p mineworld-acceptance --test client_rules` and
   `--test client_text` when the commit touches them.
5. Review: answer the commit's review item in one or two sentences in §15.
6. `git status --short`, `git diff --stat`: every changed path is in the commit's **Files**.
7. Commit with the message given, ending with
   `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` (or the implementer's model line).

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this file (§15), `handoff-s31e.md`.
- **Steps:** record the base commit; record the merge state of S31-a, S31-b, 16c, 16d, S31-h, 12e, S23 R-a; stop
  if S31-a or 16d is not merged (material stop, §12).
- [ ] Implementation. [ ] Validation: `git log origin/main` shows the merges. [ ] Review: base and decision
  for C6/C7 recorded.
- **Commit message:** `docs(plan): S31-e — open the ledger at the base`

#### C1 — The specification, the decision, the words

- **Goal:** design before code (`CLAUDE.md` §2.2); the wording both clients will read.
- **Files:** `clients/shared/TRADE.md` (new: §§5.1–5.4, 5.12's report shapes, as the module's specification),
  `clients/shared/README.md`, `docs/decisions/ARC-184.md` (new), `docs/decisions/INDEX.md` (regenerated),
  `clients/shared/settings/locale/{en.po,zh_Hans.po,messages.pot}`, `presentation/mineworld-default/2D/i18n/
  {en.po,zh_Hans.po}` (drop `format.money`), `clients/2d/PRESENTATION.md` §7.
- **Steps:** 1. write `TRADE.md`. 2. write `ARC-184` (§16) and run `python3 scripts/check_decision_ids.py
  --write-index`. 3. add §5.10's keys in `en` and `zh_Hans`; remove `format.money` from both layers.
  4. pin the live shapes: run the 2D client once with `--drive --frame` on a V-usd server and paste the
  `currency`, `wealth`, `asks`, `shop`, `item-catalogue` JSON and one `buy` offer into `TRADE.md` §"Pinned shapes".
- [ ] Implementation. [ ] Validation: `check_decision_ids.py` PASS; `check_doc_headings.py` PASS;
  `--check-pack` PASS; `client_text` PASS (words exist in both languages). [ ] Review: every reason code of
  S31-a §5.5 and S31-b §5.7 has words; no requirement lives only in `TRADE.md` that this design does not state.
- **Commit message:** `docs(clients): the shared trade module's specification, ARC-184, and its words (S31-e)`

#### C2 — The shared module: money and readers

- **Files:** `clients/shared/trade/{money.gd,trade.gd}` (readers and `sale_rows`, `stall_rows`, `shelf_rows`,
  `answer_rows`), `clients/shared/checks/{money_check.gd,trade_check.gd}`, the two symlinks, `mineworld-2d`,
  `mineworld-slice` (module loops), `.github/workflows/nightly.yml` (the two symlink checks), `.uid` files.
- **Steps:** 1. `MineWorldMoney` per §5.2. 2. the readers of §5.3. 3. the builders of §5.4 with `inputs` passed
  in. 4. the two checks over hand-built frames (priced offers available and refused, an incomplete
  `set-price`-shaped offer tagged by a test `inputs` table with neutral type names, a listing, a stub `answer`
  row). 5. the symlinks and launcher loops.
- [ ] Implementation. [ ] Validation: both checks print `PASS` headless; parse of `shared`, `2d`, `3d-spike`.
  [ ] Review: no action type, item, currency or comparison of two amounts in the module.
- **Commit message:** `feat(clients): one money formatter and one trade row model, shared by both clients (S31-e)`

#### C3 — 2D: money, tags, the sale panel and the stall

- **Files:** `clients/2d/scripts/hud/{words.gd,readers.gd,status.gd,trade_panel.gd (new)}`, `intents.gd`,
  `menu.gd` (open/close the trade panel with the menu; no entry change), `scene/{people.gd,places.gd}`, `app.gd`,
  `clients/2d/README.md`.
- **Steps:** 1. `words.money` delegates to `MineWorldMoney` with the latest currency. 2. readers: `currency` is
  `SHOWN_ELSEWHERE`; `wealth` and `asks` get readers. 3. the money line. 4. `trade_panel.gd`: builds rows (reads
  verdicts there only), draws `sale`, `ask`, `held`, `shelf`, `note` and `answer` rows, presses through one
  `press(row, button)` that calls only `intents` (R3'). 5. `intents.gd`: `COMPOSED` and `INPUTS` gain the three
  types; `compose` builds §5.6's payloads. 6. tags and the shop board.
- [ ] Implementation. [ ] Validation: parse; `check_client_rules.py` PASS (R1–R6, R3' on `trade_panel.gd`);
  `client_rules` PASS with the three 2D admissions. [ ] Review: no rule in `trade_panel.gd`; greyed rows are
  pressable.
- **Commit message:** `feat(2d): money from the world's currency, price tags, the sale panel and the stall (S31-e)`

#### C4 — 3D: the same, on 16d's menus

- **Files:** `clients/3d-spike/scripts/slice/{things.gd,intents.gd,figures.gd,slice_main.gd,trade_panel.gd
  (new),price_boards.gd (new)}`, `clients/3d-spike/README.md`.
- **Steps:** 1. `SliceThings.money` delegates to `MineWorldMoney`; `panels` adds wealth. 2. the HUD money line.
  3. `SliceTradePanel` beside `SliceOfferMenu` (opened and closed with it), with the price `LineEdit` and the
  item picker, under 16d's input gate. 4. `SliceIntents`: `INPUTS`, `composes`, `compose` for the three types.
  5. the seller `Label3D` in `figures.gd`; `SlicePriceBoards` on `SliceLayout`'s bound doors.
- [ ] Implementation. [ ] Validation: parse; `client_rules` PASS with the three 3D admissions; `client_text`
  PASS. [ ] Review: no edit to the files 16d's V-2 protects (`human.gd`, `player.gd`, scene builders); no rule.
- **Commit message:** `feat(3d): money from the world's currency, price tags, the sale panel and the stall (S31-e)`

#### C5 — The scan: check d

- **Files:** `tests/acceptance/tests/client_rules.rs` (and its `support/gdscript.rs` only if a token kind is
  missing).
- **Steps:** §5.11: d1, d2, `MONEY_ADMITTED`, the self-test.
- [ ] Implementation. [ ] Validation: `cargo test -p mineworld-acceptance --test client_rules` PASS on the head.
  [ ] Review: no false positive on `->`; a planted X-8 line is caught (recorded once, reverted).
- **Commit message:** `test(clients): no client compares money or reads a reference value (S31-e, X-8)`

#### C6 — The notice board (only if S31-b has merged; else S31-e2, SD-S31e-9)

- **Files:** `clients/shared/trade/trade.gd` (`board_rows`), `trade_check.gd`, `clients/2d/scripts/{hud/
  board_panel.gd (new),controls.gd,app.gd,hud/status.gd}`, `clients/3d-spike/scripts/slice/{board_panel.gd
  (new),slice_main.gd}`, both READMEs.
- **Steps:** §5.9: rows, panel, key N, badge.
- [ ] Implementation. [ ] Validation: parse; `trade_check.gd` board cases PASS (a stale record shown);
  `check_client_rules.py`, `client_rules`, `client_text` PASS. [ ] Review: no time comparison in either client.
- **Commit message:** `feat(clients): the notice board and its badge in 2D and 3D (S31-e)`

#### C7 — Haggling rows (only if S31-h merged before C0; else N/A, §5.13)

- **Files and steps:** S31-h's rows through this design's §5.13 interface: `INPUTS` entries, `answer_rows`, words.
- [ ] Implementation. [ ] Validation: parse; `trade_check.gd` answer cases. [ ] Review: as C3.
- **Commit message:** `feat(clients): offer a price, and answer offers, in the sale panel (S31-e with S31-h)`

#### C8 — The harness and the real-client test

- **Files:** `clients/2d/scripts/harness/interact.gd` (steps and `TRADE`, `BOARD`, `HUD` reports),
  `clients/3d-spike/scripts/slice/slice_probe_market.gd` (the same), `tools/cli/tests/client_trade.rs` (new;
  the variants of §5.12; AC-e-2 … AC-e-12, each `#[ignore]`), `tools/cli/tests/godot3d/mod.rs` if a helper is
  missing.
- [ ] Implementation. [ ] Validation: `cargo build -p mineworld-cli --tests`; one smoke run of the AC-e-3 2D arm
  through `with-slot.sh` (the test this commit writes); the rest at the milestone. [ ] Review: no selector by
  label; every world fact comes from `ids`.
- **Commit message:** `test(clients): drive trading in both clients on generated world variants (S31-e)`

#### C9 — Close-out, MERGE-READY

- **Files:** this file (§15), `handoff-s31e.md`.
- **Steps:** I-e-1's diff scan; I-e-3's grep; open `mvp0/pr-s31e`; push; `fast` CI green on the exact head;
  report PR number and head SHA.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S31-e close-out`
- **Then:** MERGE-READY — the coordinator merges.

---

## 10. Requirements this PR places on other PRs

- **R-S31a-e-1 (on S31-a's implementation).** The observer must see its own asks (an `asks` record about
  itself, or its own entity's `asks` view in its frame); otherwise §5.3's fallback leaves the stall without asks
  and C1 records the gap as a server fix.
- **R-S31h-e-1 (on S31-h).** Use §5.13's interface: an `INPUTS` entry with `row: "sale"` for `propose-price`, and
  `answer_rows` returning `answer` rows whose buttons carry the incomplete offers with `fill: { proposal }`. Its
  words keep its own keys (`sale.*`), in the shared layer. Its S31-h §5.13 table is consistent with this.
- **R-S31f-e-1 (on S31-f).** Re-run AC-e-3 and AC-e-4 on Market Town's own content (Grace selling, keepers) as
  part of AC-S31-15, with Market Town in place of V-crown (Q-S31e-4).
- **R-S31g-e-1 (on S31-g).** Name the same reason codes the same way in the agent's `display` lines; nothing
  shared in code.
- **R-S31e2 (conditional).** If C6 moves to S31-e2, it starts after S31-b merges, under this design, C6 alone
  plus AC-e-8.

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31e-1 | Amounts are shown grouped in threes with worded marks; no locale number library | §4; separators are wording, the side is the world's |
| SD-S31e-2 | With no `currency` record, amounts show as bare integers | nothing assumed (F-S31e-1) |
| SD-S31e-3 | A priced offer is recognised by shape (complete, integer `price`, typed item), not by action type | the module names no action; a future priced action shows with no client change |
| SD-S31e-4 | The trade panel opens with the interaction menu in both clients; no new key for it | step §12 "the existing interaction prompt opens it"; the menu stays the full list |
| SD-S31e-5 | The shop's shelf is behind one collapsed button, shown when a listing is here and seller offers exist; the server refuses non-keepers | F-S31e-5; the client never decides who keeps a shop |
| SD-S31e-6 | Wealth is a visible second clause of the money line, not a hover | a hover cannot be driven or shot reliably |
| SD-S31e-7 | 3D tags fade between 6 and 12 m of camera distance | presentation only; constants named `TAG_FADE_*`, no rule word |
| SD-S31e-8 | The parser refuses more than `exponent` fraction digits and more than 18 digits, and sends 0 | representation, not a rule; bounds stay the server's |
| SD-S31e-9 | The board (C6) lands here if S31-b merged, else as S31-e2 under this design | the step lets the board wait for S31-b; speed first |
| SD-S31e-10 | Shops' boards come from the last frame that disclosed the listing, dimmed when the place is out of view | presentation memory, like 13b's name cache; never a guess |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-e — trade in the 2D and 3D clients
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31e-clients.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§1, 4.1, 5.5, 5.6, 7.1, 8.6, 12, 14, 15, 16.2, 16.5);
                         pr-s31a-currency-asks.md (§§5.3–5.9, 10); pr-s31b-notices.md (§§5.7, 5.8, 10);
                         step-15-demo-3d.md §§20, 22 (16c, 16d, as merged); pr-s31h-haggling.md §5.13 (as
                         frozen); clients/protocol/ADOPTION.md §3.3; clients/shared/SETTINGS.md §6;
                         clients/2d/PRESENTATION.md §7; docs/decisions ARC-47, ARC-70, ARC-111, ARC-112,
                         ARC-114, ARC-115; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md;
                         /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md (SPEED FIRST)
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31e, its own, held by one session
BRANCH:                  mvp0/pr-s31e, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-31 §16.5) AND S31-a merged AND
                         16d (#250) merged (with 16c, #245) AND this design frozen by the operator AND the
                         primary session's dispatch. None is met at 2026-10-18 except the planning base
                                                                                          (step; primary)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–7, as §14's rulings shape them                     (operator, pending)
FROZEN INVARIANTS:       step-31 I-S31-1, I-S31-6, I-S31-9, I-S31-11, I-S31-12 (their client parts);
                         §6 I-e-1 … I-e-8; §8 as written; 16d's V-1/V-2 (accepted frames, protected files)
SEQUENCE:                C0 … C9 (§9); C6 conditional on S31-b, C7 on S31-h
ALLOWED COMMANDS:        godot (through cast-slots/with-slot.sh only); cargo build / test -p
                         mineworld-acceptance --test client_rules|client_text; cargo build -p mineworld-cli
                         --tests; one targeted client_trade test through with-slot.sh; target/*/mineworld
                         server|validate; python3 scripts/check_client_rules.py, check_decision_ids.py
                         (--write-index), check_doc_headings.py; git; gh (pr create/view/checks, run view;
                         never merge); mkdir -p; ln -s (the two symlinks); cp; cmp; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or all
                         of tools/cli locally (R-LOAD-1); two Godot windows at once; reading any key or .env;
                         calling any model; editing a path I-e-1 forbids or a file 16d's V-2 protects;
                         merging
MATERIAL STOPS:          S31-a or 16d not merged at C0; a record shape at C1 that §5.3 cannot read without a
                         server change (report it; do not infer); a needed change under I-e-1's paths; a
                         needed new dependency (DEP); check d unable to pass without an admission; any change
                         to a frozen step invariant or acceptance
PLATFORMS:               macOS, Linux, Windows: no OS branch in client code; the symlink checks and launcher
                         loops cover the new module; shots on all three at the milestone
VALIDATION BUDGET:       per commit: Godot parse of touched projects, the module checks, the static scans,
                         and the tests the commit writes (charter 2026-10-18). Milestone: client_trade.rs
                         (≤ 20 min on Linux CI), AC-e-13 shots, §8.1 mutations. Gate 1: NOT REQUIRED.
                         Gate 2: the real-client runs above, at the milestone. No paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31e.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to `fast` green: authorized           (working rules §21; charter)
                         endpoint: MERGE-READY — pushed, `fast` green on the exact head, reported with PR
                                   number and head SHA       (source: operator charter, 2026-10-18)
                         merge: NOT by the implementation session; the coordinator merges
                                   (source: operator charter, 2026-10-18)
POST-MERGE SYNC OWNER:   primary: step-31, overall.md, the progress page; implementation session: this
                         ledger, the merge identity, evidence, deviations
STOP CONDITION:          S31-e MERGE-READY — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31e-1 | 16c, 16d and 12e land late or change `SliceOfferMenu`, `SliceThings`, `SliceLayout` | the start condition waits for 16d; C0 re-reads their merged shape; the later of 12e and this PR rebases |
| R-S31e-2 | A record's live JSON differs from S31-a's or S31-b's design text | C1 pins shapes from a live frame; a mismatch the readers cannot take is a material stop, reported to the server's PR |
| R-S31e-3 | Check d1 flags an innocent function (both words and a comparison in one function) | split the function; `MONEY_ADMITTED` exists for a reasoned exception; the self-test pins the known shapes |
| R-S31e-4 | Visitors see the shop's shelf button and get refused | collapsed behind one button; the refusal is worded; Q-S31e-1's module can show a keeper list if S31-a later discloses it |
| R-S31e-5 | The 3D price field fights 16d's input gate or Esc order | one gate (16d D-16d-9); Esc order: price field, then the menu, then settings |
| R-S31e-6 | A new symlinked module breaks a Windows checkout or an export | the launchers and nightly symlink checks list it (A-15); S23 R-a's presets gain it if merged first |
| R-S31e-7 | Haiku implements the 3D half with a rule in it | check d and R3'-style review; Opus review of the 3D frames and diff at the milestone |
| R-S31e-8 | Long panels clip in zh_Hans or at small windows | the shots of AC-e-13 include zh_Hans; panels keep 16d's keep-on-screen rule |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S31e-1 [module boundary]** | Both clients must share one money formatter and one trade panel model (step §12). Where does that code live? (a) a new shared module `clients/shared/trade`, symlinked into both clients like the far-field module; (b) inside the settings module, beside the clock formatter; (c) inside the protocol module | **(a).** It keeps settings about settings and the protocol module game-neutral, and it costs two symlinks and two launcher lines (A-15). `ARC-184` records it |
| **Q-S31e-2 [scope]** | Players cannot subscribe or unsubscribe from a client, because the server does not disclose the list of topics to a non-subscriber (F-S31e-6). Leave subscribing out of S31-e? | **Yes, leave it out.** The board shows what you subscribe to; agents subscribe through S31-g. A later PR can add "subscribe" once `notices` discloses a topic catalogue |
| **Q-S31e-3 [step wording]** | The step puts the two new client-rules patterns in `check_client_rules.py`, which scans only the 2D client (F-S31e-3). Put them in `client_rules.rs` instead, which scans 2D, 3D and the shared modules? | **Yes.** One rule covers both clients and the new module. The step's §12 sentence gets a dated note |
| **Q-S31e-4 [acceptance on content]** | AC-S31-15 says "walk to Grace, buy flowers", but Grace sells nothing in Market Town until S31-f (F-S31e-7). Run S31-e's drive on a generated copy where Grace sells flowers, and have S31-f re-run it on the real town? | **Yes.** S31-e stays independent of S31-f, and the charter wants a second world for every criterion anyway |

### 14.1 Rulings (operator, primary session, 2026-10-20; relayed by the lane lead)

| Id | Ruling |
| --- | --- |
| Q-S31e-1 | **(a)** a new shared module `clients/shared/trade`, symlinked into both clients; `ARC-184` records it |
| Q-S31e-2 | Subscribing and unsubscribing are left out of S31-e |
| Q-S31e-3 | The two new patterns go in `tests/acceptance/tests/client_rules.rs` (check d) |
| Q-S31e-4 | The drive runs on a generated copy of Market Town in which Grace sells flowers; S31-f re-runs it on the real town |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] base and merge state recorded | [x] `git log origin/main` shows #380 (S31-a), #349 (S31-b), #245 (16c), #250 (16d), #189 (WE-a), #254 (12e) | [x] C6 in scope; C7 N/A |
| C1 | (next) | [x] `TRADE.md`, `ARC-184` + index, §5.10 words in en/zh_Hans + pot, `format.money` retired, `PRESENTATION.md` §7, `SETTINGS.md` §2, shared README, `client_text.rs` families/retired/module roots | [x] `check_decision_ids` PASS (215 ids); `check_doc_headings` PASS; `--check-pack` PASS; `client_text`: 5/6 PASS, the catalog test lists exactly the keys C3/C4 use and the two `format.money` callers C3/C4 replace (expected; re-run at C4) | [x] every S31-a §5.5 and S31-b §5.7 code has en and zh_Hans words (the test's reason families found none missing); `TRADE.md` states nothing §5 does not |
| C1 | `f1768fcd` | (above) | (above) | (above) |
| C2 | (next) | [x] `money.gd`, `trade.gd` (readers, `sale_rows`, `stall_rows`, `shelf_rows`, `answer_rows`, tags, boards, `board_rows`, `report`), `money_check.gd`, `trade_check.gd`, two symlinks, launcher loops, nightly symlink checks | [x] `godot --import` of `shared`, `2d`, `3d-spike`: no Parse Error / SCRIPT ERROR / Cannot get class; `money_check: PASS`; `trade_check: PASS` | [x] the module spells no action type, item, currency or place (neutral `act-*` types in the check prove matching by tag); no `<`/`>` beside a wallet word; no `references` literal; refused rows kept |
| C3 … C9 | — | [ ] | [ ] | [ ] |

### Evidence

**C0 — base (2026-10-20).** Implementation base `origin/main @ fec906f2` (#412). Merged: S31-a (#380,
`16f19d3f`), S31-b (#349, `0574b977`), 16c (#245, `32cdff7c`), 16d (#250, `9c232c8d`), 12e (#254, `effd23ae`),
WE-a (#189, `9a9e50ff`). S31-a's action keys `action.{list-for-sale,set-price,withdraw}` and their `.done` forms
already exist in the shared layer (`a7d145f5`); C1 does not re-add them. **Not merged:** S31-h, so **C7 is N/A**
and R-S31e-h-1 binds S31-h (its C8 adds the rows through §5.13). S23 R-a: no client export presets on main
(`clients/*/export_presets.cfg` absent), so nothing to add. A notices `topics` record is coming in S31-b2; this
PR does not depend on it (Q-S31e-2 stands).

**Validation budget in force (lane lead, SPEED FIRST).** Per commit: build/parse, fmt, clippy, and only the
tests this PR writes or changes. Local commits only; push and PR wait for the lane lead (lane CI cap).

### Mutations

(none yet; X-8, X-10, M-e-1 … M-e-6 at the milestone, as §8.1 lists them)

### Deviations and findings

F-S31e-1 … F-S31e-12 (§3.2) and SD-S31e-1 … 10 (§11), recorded at design time.

Bounded, recorded during implementation:

- **D-e-1 (C1) — the money scan is check *e*, not *d*.** `client_rules.rs` already has a "check (d)" (S33-c's
  place keys, `place_key_findings`). The new check keeps §5.11's content under the next letter, so no name means
  two things.
- **D-e-2 (C1) — shapes pinned from the serialisers, live pin at the milestone.** C1 step 4 (paste live JSON) needs
  a running server and a client drive; under SPEED FIRST the shapes were pinned from the System Packs' serde
  types at `fec906f2` (`TRADE.md` §4.1) and the live pin moves to the milestone drive. The asks view is disclosed
  about every person to every perceiver, the observer included (`EconomySystem::discloses`), so R-S31a-e-1 holds
  and `own_asks` reads the observer's own entity.
- **D-e-3 (C1) — `client_text.rs` changes.** Not in C1's file list but required by it: the key families gain
  `trade` and `board`; `format.money` moves from `MOVED_BY_16D` to a new `RETIRED` list (each retired key must be
  13b's and in no layer); `clients/shared/trade` joins the scanned module roots.
- **D-e-4 (C1) — extra wording keys.** `trade.tag` ("{item} {price}"), `trade.tag.rest` ("+n more", the shop
  board's overflow line) and `trade.cancel` (the 3D price field and picker) join §5.10's list; `trade.tag.more`
  takes `{tag}` and `{count}`, `hud.money.unvalued` takes `{line}` and `{count}`.
- **D-e-5 (C2) — the `inputs` table carries a `label` key and `row` is a list;** the shop side is filled as
  `shop: true` and mapped to `for: "shop"` by `intents.gd` only, so the module spells no payload value.

---

## 16. Proposed edits elsewhere (made by the PR named)

- **`docs/decisions/ARC-184.md` (this PR, C1).** First line `## ARC-184 — Clients format money and read trade
  records through one shared module`. Body: the module `clients/shared/trade` (`MineWorldMoney`,
  `MineWorldTrade`), taken by both clients as `res://mineworld_trade`; amounts are formatted only from the
  disclosed `currency` (no scale assumed; bare integers without one); a priced offer is recognised by shape;
  clients compare no money, read no reference value and compute no expiry (check d of `client_rules.rs`);
  rejected alternatives (settings module; protocol module; two client-local readers, 16d Q-16d-5); consequences
  (16d's duplicated readers for wallet and shop are replaced; R-16d-7 closed for trade shapes).
- **`step-31-wealth-prices.md` (the primary session, on freeze).** Dated notes: §12 — the patterns live in
  `client_rules.rs` (Q-S31e-3), the trade panel opens with the interaction menu, wealth is a visible clause, no
  subscribe UI (Q-S31e-2); §15 AC-S31-15 — S31-e runs it on a generated variant and S31-f on Market Town
  (Q-S31e-4).
- **`clients/2d/PRESENTATION.md` §7 (this PR, C1).** `format.money` replaced by the `format.money.*` keys.
- **`clients/shared/SETTINGS.md` (this PR, C1).** One line in §2 naming the sibling `trade` module and its
  dependency on `MineWorldText`.
- **`pr-s31h-haggling.md` (its owner).** §5.13's rows map onto this design's §5.13 (R-S31h-e-1).
