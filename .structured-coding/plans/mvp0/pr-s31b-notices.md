# PR S31-b — The `notices` System Pack: time-limited public notices, delivered to subscribers' inboxes

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every §14 question as recommended)

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S31-b PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31b-s29a, branch
                        docs/pr-design-s31b-s29a-rev, planning base origin/main @ b85f2804 (#260 merged;
                        step-31 APPROVED and S31-a FROZEN on main)
Approved by / evidence: the operator, 2026-10-18, in the primary session: Q-S31b-1 … Q-S31b-4 as recommended. The step's
                        rulings (Q-S31-1 … Q-S31-8, step-31 §18) and S31-a's Q-S31a-3 bind and are not
                        re-asked
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S30-c, S28-d,
                        S31-a and #258 (the decision-log split)
Start condition:        step-31 §16.5: nothing in S31 starts until Milestone F's checklist has been handed
                        to the operator (the S29 timing rule, overall.md "Milestone F"; step-31 §16.5,
                        "Start"); then this design frozen; then the primary session's dispatch. S31-b needs
                        no other PR merged first (step-31 §16.2: "none (parallel with S31-a)")
Execution contract:     §12 (filled, endpoint authority included; ends at READY FOR OPERATOR REVIEW)
Implementer:            Opus-class (step-31 §16.2, the S31-b row: "new pack; audience and time semantics");
                        reviewer the primary session plus an Opus logic review of C4
Lifecycle:              DESIGN FROZEN 2026-10-18 (implementation still waits for the start condition)
```

**Effort:** `mvp0`, release-blocking (Q-S31-1 re-ruled) · **Step:**
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15) §§0.1, 1 (outcomes 6, 9), 3, 8
(all), 13, 14 (I-S31-1, I-S31-2, I-S31-7, I-S31-8, I-S31-9, I-S31-10), 15 (AC-S31-9; X-4, X-5), 16.2 (the S31-b
row), 16.4 (S27, S28, S30), 16.5, 19.1, 19.4 (`ARC-115`) · **Parent:** [`overall.md`](overall.md) ·
**Parallel with:** [`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (FROZEN 2026-10-18; no shared file
but `systems/installed`, `docs/MODULE_SPEC.md` §4.1 and the decision log) · **Followed by:** S31-d (the reporter
publishes through this pack), S31-e (the notice-board panel), S31-f (Market Town's topics and subscriptions),
S31-g (the agent's `View.notices` and typed tools) · **Coordinates with:**
[`pr-s30c-attributes-pack.md`](pr-s30c-attributes-pack.md) (FROZEN 2026-10-18, not merged: `AttributeOwner`,
R-S31-c-1, Q-S30c-2), step-28 S28-d (not designed: `Rendered.anchors`) · **Binding parents:** `CLAUDE.md` §§2,
3, 4 (items 1, 2, 3, 5, 7, 8, 9, 10, 11, 13, 15, 16); `docs/CORE_CONCEPTS.md` §§12, 15.2;
`docs/MODULE_SPEC.md` §4.1; `docs/DECISIONS.md` `ARC-26`, `ARC-31`, `ARC-33`, `ARC-34`, `ARC-43`, `ARC-61`,
`ARC-90`, `ARC-124`.

**Decision ids.** This PR writes **`ARC-115`** (assigned by the step, step-31 §19.4: "`notices`: topics,
subscriptions, audience = subscribers, lazy expiry"; confirmed free on `origin/main @ b85f2804` by
`grep -n "^## ARC-115" docs/DECISIONS.md`, no match) and one dated note under **`ARC-124`** (S31-b codes against
the owner interface and adopts the crate when S30-c merges). No other id is used. Where the record is written
depends on #258 (open at 2026-10-18): `docs/DECISIONS.md` while the log is one file, `docs/decisions/ARC-115.md`
plus the regenerated index once #258 has merged (C1, SD-S31b-12).

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Operator requirement, 2026-10-15 (step-31 §0.1, verbatim):** "这种有时效性的公共信息可以发送到别的npc的
  inbox里，每个npc可以有订阅公共信息的属性" — *"this kind of time-sensitive public information can be sent to
  other NPCs' inboxes, and each NPC can have an attribute for subscribing to public information."* The same
  requirement makes the reporter's backend modular ("可以是llm agent，也可以是确定性的算法"): S31-b is the channel
  that backend publishes on, so whatever publishes is validated alike (§5.5).
- **Operator principles relayed with this task (2026-10-18).** (1) The backend can be an LLM agent or a
  deterministic algorithm, modular. (2) Each NPC has a subscription attribute. (3) One currency per world.
  (4) Any seller sets its own price. (5) A price clerk publishes suggested prices. Items (3) and (4) are S31-a's
  (frozen) and are consumed here only as "a price line is an item and an amount, advisory"; items (1), (2) and
  (5) shape this PR (§1).
- **The coordinator's restatement (step-31 §0.1 items 4 and 5).** "Public notices are a general
  publish/subscribe mechanism. A notice has a topic, a publisher, a body, an issue time and an expiry. It is
  owned by one System Pack. Agents over MCP receive notices through S27-d's inbox. Received notices become S28
  memories. An expired notice stops being current but is still remembered." "Subscriptions are a person
  attribute that S30 owns. S31 specifies the semantics and the shape." (Q-S31-7 ruled: `notices` owns the
  component; S30 shows it.)
- **Scope (step-31 §16.2, S31-b row).** "`systems/notices`: configuration, section, three actions,
  `notice-published`, `subscribed`/`unsubscribed`, board record, S28 renderer (Python, with `Rendered.anchors`
  once S28-d lands), `ARC-115`." Depends on "none (parallel with S31-a)". Checkpoint "fixture world:
  publisher, two subscribers, one non-subscriber; expiry at the boundary; kill-and-resume". Adversarial
  "X-4, X-5". Implementer **Opus**.
- **Q-S31a-3 (ruled 2026-10-15, binding here).** "`subscribe` and `unsubscribe` are offered **incomplete**, one
  each, so a generic controller never changes a subscription at random. S31-b's design applies this." (step-31
  §8.2 dated note; S31-a R-S31b-a-1.)
- **Start (step-31 §16.5).** "Nothing starts before Milestone F's checklist has been handed to the operator
  (the S29 timing rule). Each PR starts only once its own PR design is frozen." First wave: S31-a and S31-b in
  parallel.
- **Order with S30 (step-31 §8.5, ARC-124).** "If S31-b freezes before S30-c merges, S31-b ships its own
  section and tag defaults, and adopts the crate in the PR that follows S30-c. There is still one writer
  throughout." S30-c ruled (Q-S30c-2, 2026-10-18): "an owner's attributes exist only where `attributes` is
  enabled"; R-S31-c-1: owners "ship their declarations with `owner:` themselves, receive initial values through
  `attributes-handed`, disclose with `disclose_owned`".
- **R-LOAD-1 (coordinator rule, in force).** Locally: only the targeted test files of the commit, plus `cargo fmt
  --check` and `cargo clippy -p <crate> --all-targets -- -D warnings`. Never a local `cargo test --workspace`,
  never the whole `tools/cli` suite. CI on the exact head is the gate (§12).

---

## 1. Goal

Give every MineWorld world an optional, general publish/subscribe channel for **public, time-limited
information**, owned by one System Pack, `notices`. A world declares its **topics** in `configure/notices.yaml`:
for each, who may publish, what kind of body a notice carries (typed: `prices` or `text`), and how long a notice
may stay valid. Each Person holds a **subscription** set, its "attribute for subscribing to public information",
seeded from its file or from tag defaults and changed only by its own `subscribe` / `unsubscribe` actions. A
named publisher's `publish-notice` — whether its seat is a deterministic model, an LM, a human or an MCP agent —
is validated identically and stated as one fact whose audience is exactly the topic's subscribers at that
instant plus the publisher. The kernel's existing perception path (presence's audience rule, `ARC-43`) then puts
it into exactly those perceived streams: an NPC's observation, an agent's inbox (S27-d), and every subscriber's
S28 memory through a new renderer. A **board** record discloses to each person the notices of its topics that
are current now, computed from world time at disclosure, so an expired notice leaves the board exactly at its
`valid_until` with no wake and no fact, while the inbox and memory keep it. Nothing changes in `kernel/` or
`contracts/` (I-S31-1). No reference world enables the pack in this PR; Market Town's topics and subscriptions
are S31-f content.

---

## 2. Scope

### 2.1 In scope

1. **`systems/notices`** (new crate `mineworld-notices`, `SystemId` `notices`):
   - `Cargo.toml` (dependencies exactly as `systems/naming`'s plus `schemars`: `mineworld-authoring`,
     `-contracts`, `-kernel`, `-presence` (for `PerceptionProvider` only; **no system dependency**), `-sdk`,
     `schemars`, `serde`, `serde_json`, `thiserror`; dev: `serde-saphyr`).
   - `src/topic.rs`: `TopicId` (dotted slug), `BodyKind`, `ValidFor { min, max }`, `Topic`, `NoticeDuration`.
   - `src/configuration.rs`: `NoticesConfiguration { topics, defaults }` and its bounds (§5.2).
   - `src/board.rs`: `NoticesProcess` (never woken) and `BoardState` (§5.3).
   - `src/body.rs`: `NoticeBody`, `PriceLine`, `NoticeText`, and `judge_body` (§5.5).
   - `src/component.rs`: `Subscriptions` (component), the disclosed records `subscriptions` and `notice-board`.
   - `src/section.rs`: the person section `notices: { subscribe: [topic] }` (§5.4).
   - `src/action.rs`: `PublishNotice`, `Subscribe`, `Unsubscribe`, their requirements (`SpatialRequirement::NONE`).
   - `src/judge.rs`: `judge_publish`, `judge_subscription` — the one decision `offers` and `validate` both call.
   - `src/event.rs`: `NoticesConfigured`, `SubscriptionsSeeded`, `Subscribed`, `Unsubscribed`,
     `NoticePublished` (§5.8).
   - `src/system.rs`, `src/codec.rs`, `src/lib.rs`: declaration, `validate`, `resolve`, `react` (none),
     `offers`, `discloses`.
   - `tests/support/mod.rs` and `tests/{configuration.rs, subscriptions.rs, publish.rs, board.rs}`.
   - `README.md`.
2. **`systems/installed`**: one line in `Cargo.toml`, one in `src/lib.rs` (`Notices =>
   mineworld_notices::NoticesSystem`, appended last), as `ARC-33` prescribes.
3. **`cognition/lm-controller`**: `src/mineworld_cognition/memory/render/notices.py` (the `notice-published`
   renderer, §5.9) and its registry line in `render/__init__.py`; `tests/test_memory_render.py` cases and
   goldens, captured by `tests/capture_golden_renders.py`'s documented method.
4. **`tools/cli/tests/notices.rs`** (new): the real-binary checkpoint on a scratch fixture world (§8 AC-b-8 …
   AC-b-10).
5. **Documentation:** `ARC-115` (and the `ARC-124` note); `docs/MODULE_SPEC.md` §4.1 (the `notices` section
   row and the `configure/notices.yaml` entry); the generated `docs/reference/` files regenerated by
   `mineworld reference` (S25a, `ARC-90`: every section and configuration type derives `JsonSchema`).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The price reporter (`price-reporter` model), the price book, pricing policies reading the board | S31-c, S31-d |
| Market Town's `configure/notices.yaml`, its reporter as publisher, subscriptions by role, the 30-day convergence checkpoint (AC-S31-13) | S31-f |
| The agent's `View.notices`, `MeLine.subscriptions`, the inbox's by-name reading of `notice-published`, the typed tools `publish_notice`, `subscribe`, `unsubscribe` | S31-g |
| The 2D / 3D notice-board panel and inbox badge, i18n words for the codes | S31-e |
| `notices.subscriptions` as an S30 attribute (`AttributeOwner`, `attributes-handed`, `disclose_owned`, role defaults through S30's `roles`) | the PR that follows S30-c's merge (step-31 §8.5 "Order"; R-S30c-b-1 here, §10) |
| `Rendered.anchors` on the notice renderer (item-anchored price memory) | S28-d, or the first PR after it (step-31 §16.4) |
| System-stated notices (`weather-warnings`), the `departures` body kind | later (step-31 §8.7) |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `systems/` other than `notices` and `installed`, `clients/` | never in this PR (I-b-1) |

### 2.3 Dependencies

- **Required before C0:** Milestone F's checklist handed over (operator); this design frozen; the primary's
  dispatch. No PR needs to merge first.
- **Either order, no conflict of content:** S31-a (#230 frozen). Both append one line to `systems/installed`
  and one row to `MODULE_SPEC.md` §4.1, and each writes its own decision records. Whichever lands second
  rebases those three lines (R-S31b-1).
- **Either order, convention rule:** #258 (decision-log split). C1 writes `ARC-115` wherever the log lives on
  the base it starts from, and C0 records which (SD-S31b-12).
- **Either order, adoption rule:** S30-c. If S30-c has merged at C0, the design's §5.4 still ships as written
  (this PR's frozen scope), and the adoption PR follows (R-S30c-b-1). Nothing here pre-empts S30-c's shapes.
- **Either order, renderer rule:** S28-d. If it has merged at C5, the renderer also states `anchors` (the
  items of a `prices` body); otherwise it does not, and S28-d or S31-g adds them (step-31 §16.4).

---

## 3. Audit (`origin/main @ b85f2804`, 2026-10-18)

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `Visibility::Entities(BTreeSet<EntityId>)`: "Exactly these entities", a `BTreeSet` so the audience serializes in one order | `contracts/src/event.rs:296-310` | The audience is expressible with no contract change (I-S31-1) |
| A-2 | **Presence**, not the kernel, owns who learns of a fact: `admits` answers `Entities(audience) => audience.contains(&observer)`, and the server's frames, the `perceived` stream and `mineworld perceived` all call it | `systems/presence/src/audience.rs:1-31, 104-117` (`ARC-43`) | Step-31 §8.4's "the kernel's existing audience rule delivers it" is presence's rule; the consequence is the same (F-S31b-1) |
| A-3 | `PerceptionProvider` is presence's trait; a pack that only discloses depends on the `mineworld-presence` **crate** and declares no system dependency (`naming`: "For PerceptionProvider … No system dependency") | `systems/naming/Cargo.toml` `[dependencies]` comment; `systems/naming/src/system.rs:72-90` | "`notices` depends on no other System" (step §8.1) holds at the system level; the crate dependency is the standard one |
| A-4 | `PackConfiguration { Configuration: DeserializeOwned + JsonSchema, FACTS, references, requires, attachments, check, seed }`; configuration facts "should be stated `Visibility::SystemInternal` with no subjects" (`ARC-61` item 8, guidance) | `authoring/src/configuration.rs:88-160` | The topics are a configuration; `notices-configured` is `SystemInternal`. Publishers named by key are checked by the loader through `references()` |
| A-5 | `Seeding` hands an owner "the assembled world … and no state at all: the loader seeds every section before genesis reduces any fact"; configuration seeds before sections (`ARC-61` item 6) | `authoring/src/section.rs:130-160`; `docs/DECISIONS.md` `ARC-61` | A section cannot see the topics; an unknown topic in a person's section is refused at **reduction** (as S31-a's SD-S31a-1), which `mineworld validate` runs (A-6) |
| A-6 | `mineworld validate` is `WorldPack::read` plus `load(EPOCH)`: genesis is reduced, so a reduction refusal fails validate by name | `tools/cli/src/main.rs:419-421`, `587-589` | Every genesis refusal of §5.7 is a `validate` refusal |
| A-7 | Entities carry `Tags` fixed after genesis; `WorldRead::entities()` iterates them; `EntityType::{Person, Item, …}`; `ItemId`, `PersonId` | `contracts/src/entity.rs:78-110`; `kernel/src/view.rs:112-126`; `contracts/src/ids.rs:691, 734, 856`; `authoring/src/classes.rs:1-8` ("Tags are fixed after genesis") | Tag defaults are computable at configuration seeding from the assembled world; a price line's item is checked as a living `Item` entity with no `item`-pack dependency |
| A-8 | Calendar keeps world configuration in a never-woken-between-events Process and discloses declared, never-carried records; WE-a and S31-a keep economy's configuration in a never-woken Process | `systems/calendar/src/process.rs:1-68`, `system.rs:274-305`; `pr-s31a-currency-asks.md` §5.2 (`CurrencyProcess`) | Topics, the notice counter and the current notices live in `NoticesProcess` (§5.3) |
| A-9 | `WorldView::set_process_state`, `start_process`; `WorldRead::at()` is the world time at a read | `kernel/src/view.rs:94, 350-360, 410-430` | Lazy expiry is computed at disclosure from `at()`; reductions write the process state |
| A-10 | `WorldTime` / `SimDuration` are seconds; "nothing in this crate knows what a date, a weekday … is" | `contracts/src/time.rs:1-17, 29-70` | A "per day" limit cannot mean a calendar day without a dependency on `calendar` (F-S31b-2) |
| A-11 | `Offer::new` is an incomplete offer (no payload); `Offer::complete` carries a payload; `refused(reason)` is reported before the requirement | `systems/presence/src/interaction.rs:154-216` | Every offer of this pack is `Offer::new` (incomplete), refused by code where it would fail (Q-S31a-3) |
| A-12 | Generic controllers attempt available **complete** affordances at random (rule: 20 % of consults; motivated: every available complete affordance is a choice) | `cognition/rule-controller/src/offered.rs:26-52`; `cognition/motivated/src/options.rs:285-289` | With incomplete offers only, no NPC publishes or subscribes unless a model chooses to (AC-b-6) |
| A-13 | Section facts of genesis are today `Entities({subject})` (economy's `funded`, employment's job facts) | `systems/economy/src/section.rs:95-115`; `systems/employment/src/section.rs:170` | Precedent for a genesis subscription fact visible to its person; Q-S31a-2 set the opposite precedent for configuration facts (Q-S31b-2) |
| A-14 | Installing a pack is one line in `systems/installed/Cargo.toml` and one in `src/lib.rs` (`installed!` list, lines 38-62, `Appearance` last); S26-a's install commit moved no golden (`318f6ffc`: installed, `Cargo.lock`, `worldpack/tests/refusals.rs` only) | `systems/installed/src/lib.rs:38-62`; `git show --stat 318f6ffc` | C2 appends `Notices`; no baseline moves (AC-b-11) |
| A-15 | S25a (#200, merged): a section's and a configuration's type derive `JsonSchema`; `mineworld reference` regenerates `docs/reference/`, and CI fails on drift | `authoring/src/configuration.rs:97-107`; `docs/reference/README.md`, `sections.md`, `configure.md` | C2 regenerates the reference in the same commit |
| A-16 | The S28 renderer registry: event type → pure function `(fact, me, names) → Rendered`; unknown types render generically; `Rendered { gist, counterparts, salience, mine, importance, level, names, fallback }` — no expiry and no anchors field | `cognition/lm-controller/src/mineworld_cognition/memory/render/__init__.py:28-145` | The notice renderer is one module plus one registry line; "(expired)" at recall needs a field memory does not have (F-S31b-4, Q-S31b-3) |
| A-17 | Recall prints a record as `Day <n> <HH:MM>: <gist>` from the fact's `at`, with `day_number(at) = at // 86 400 + 1` and `clock(at)` | `memory/retrieve.py:248-262`; `compress/summarize.py:22-30` | The renderer writes "valid until Day <n> <HH:MM>" with the same two helpers, so recall shows both times in one convention |
| A-18 | The agent inbox drains the seat's `perceived` stream (bound 512); payloads other than `spoke` and `walk-ended` show type and participants, `detail` adds the payload (≤ 1 KiB) | `clients/agent/src/mineworld_agent/inbox.py:1-12` | A `notice-published` reaches an MCP agent's inbox with no change here; S31-g reads it by name |
| A-19 | No world, test or fixture names `notices`, `notice-published`, `subscribe` (as an action) or `configure/notices.yaml` | `git grep -n "notice-published\|notices.yaml"` (plan files only) | The pack is new; nothing to migrate |
| A-20 | `ARC-124` names `notices` as an owner (`notices.subscriptions`, `set` over `notices.topics`, ≤ 32, `private`, `evolving`) and says "If S31 freezes before S30-c merges, it codes against this text and adopts the crate at S30-c's merge; a conflicting change is a material stop for both" | `docs/DECISIONS.md:9003-9022`; step-30 §9.6 N-1 … N-6 (`step-30-person-attributes.md:1306-1325`) | §5.4's shapes are chosen so the adoption changes no fact (R-S30c-b-1) |
| A-21 | S30-c (frozen, not merged): `AttributeOwner`; Q-S30c-2 ruled "an owner's attributes exist only where `attributes` is enabled"; R-S31-c-1 | `pr-s30c-attributes-pack.md:421-430, 744-746, 826` | Without `attributes`, subscriptions are still `notices`' own state and actions; they are just not shown as an attribute |
| A-22 | The real-binary restart pattern: `mineworld server <world> --save DIR`, a test client acts through the protocol, SIGKILL, the same command again, then `mineworld replay` | `tools/cli/tests/restart.rs:1-30` (IC-2) | C6's kill-and-resume follows it; publishing needs a client, since no NPC controller attempts an incomplete offer (A-12) |
| A-23 | `mineworld perceived` exports a person's perceived facts from a save through presence's `perceived_by` | `tools/cli/src/main.rs:524`; `systems/presence/src/audience.rs:118-140` (`perceived_by`) | AC-b-8's audience oracle is the export, not the pack's own state |
| A-24 | #258 (open) splits `docs/DECISIONS.md` into `docs/decisions/<ID>.md` with a generated `INDEX.md` and `check_decision_ids.py` | `gh pr view 258` | SD-S31b-12 |

### 3.2 Findings

- **F-S31b-1 — The audience rule is presence's.** Step §8.4 says "the kernel's existing audience rule delivers
  it". The rule is presence's `admits` (A-2, `ARC-43`); the kernel only stores the envelope. Nothing changes in
  the design: the fact states `Entities(…)` and presence honours it. A world with `notices` and without
  `presence` has no perception at all, which is true of every pack today. Recorded; `ARC-115` says "presence's
  audience rule".
- **F-S31b-2 — "Per day" has no day to count in.** `per_day` (step §8.1) needs a day boundary. The contracts
  know only seconds (A-10), and `notices` depends on no system, so it cannot read `calendar`'s day. →
  **Q-S31b-4** (recommended: a rolling 24-hour window of world time; the key keeps its name and its reference
  text says so).
- **F-S31b-3 — Tag defaults and a person's own section meet at genesis in two different seeds.**
  Configuration seeds before sections and a section cannot see the configuration (A-5), so "the person's
  `notices.subscribe` if present, otherwise the union of `defaults.by_tag`" (step §8.5) cannot be decided in
  one seed. Design: the configuration's seed states one `subscriptions-seeded { origin: default }` per person
  whose tags match a default; the section's seed states one `subscriptions-seeded { origin: authored }`; the
  reduction of an `authored` one **replaces** the set. Genesis order makes the outcome exactly the step's rule,
  and the log shows both (SD-S31b-3).
- **F-S31b-4 — Memory cannot mark a notice "(expired)" at recall.** Step §8.6: "Recall adds `(expired)` when
  the consult's time is past `valid_until`." A record holds its gist and its fact's `at`; `Rendered` has no
  expiry (A-16), and recall formats lines without reading payloads (A-17). Marking it needs an S28 schema field
  (`valid_until` on the access row), which is S28's to own. → **Q-S31b-3** (recommended: the gist states
  "valid until Day n HH:MM" and recall already prints the record's own time; the marker waits for an S28 PR).
- **F-S31b-5 — The step's refusal list has no code for a malformed body line.** `body-mismatch` (wrong kind)
  and `body-too-large` (> 32 lines, > 280 bytes) exist; a line naming a non-Item, a zero or over-bound price, a
  zero `basis`, an unsorted or duplicated item, or a text with a control character has none. Added
  `body-invalid` with the reason in `detail` (SD-S31b-6), bounded: one code, same refusal class.
- **F-S31b-6 — `notices` cannot read economy's `PRICE_MAX`.** It depends on no system (step §8.1). A price
  line's bound is `notices`' own `NOTICE_PRICE_MAX = 10^12`, stated equal to economy's `PRICE_MAX` by design
  (`ARC-112`); if S31-a has merged at C4, a dev-dependency test asserts the equality (SD-S31b-7).
- **F-S31b-7 — The subscriptions bound is the topics bound.** A person subscribes only to declared topics, so
  capping the world's topics at 32 (step §8.5's `max: 32`) makes `Subscriptions` ≤ 32 by construction, with no
  subscription-limit code (SD-S31b-8).
- **F-S31b-8 — The publisher sees its own notices on the board.** Step §8.6's board lists "the current notices
  of the topics the observer subscribes to". A publisher that does not subscribe to its own topic would see its
  notice only in its inbox (it is in the audience, step §8.4), and a deterministic publisher model could not
  tell its last bulletin is still current. The board also lists the topics the observer publishes
  (SD-S31b-9).
- **F-S31b-9 — Genesis subscription facts would land in every inbox and memory.** If `subscriptions-seeded` is
  `Entities({person})` (A-13's precedent), every subscribed person's inbox and S28 memory starts with a fact no
  one witnessed. S31-a's Q-S31a-2 ruled the same question the other way for configuration facts. →
  **Q-S31b-2**.
- **F-S31b-10 — Until S30-c, the "attribute" is `notices`' own section.** The operator's "each NPC can have an
  attribute for subscribing" is satisfied in this PR by the `subscriptions` component, the person section and
  the holder-only `subscriptions` record; it becomes an S30 attribute (`notices.subscriptions`) when the
  adoption PR lands, without a new fact (R-S30c-b-1). → **Q-S31b-1** confirms keeping the tag defaults until
  then.

### 3.3 What does not exist

No `notices` crate, topic, notice, subscription or board anywhere (A-19). No body-kind catalog. No renderer for
any S31 fact. No world enables a publish/subscribe channel.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| Publish/subscribe delivery | a message broker (NATS, MQTT, Redis pub/sub), an in-process bus crate | **Not applicable**: delivery is perception, already owned by presence (A-2) and persisted with the world. A broker would put world state outside the event log and the server-authoritative path (`CLAUDE.md` §4 rules 3, 16: never force MineWorld inside a framework's execution model). No `DEP` record: nothing is adopted and no infrastructure is rebuilt; the pack adds a domain rule (who is subscribed) over existing infrastructure |
| Duration strings (`"6h"`, `"7d"`) | `humantime`, `duration-str` | **Declined, recorded in `ARC-115`'s consequences**: three suffixes (`m`, `h`, `d`) over a positive integer, bounded 1 min … 30 d, are a ten-line `TryFrom<String>` with a message naming the key (calendar's `Epoch` precedent, `systems/calendar/src/configuration.rs:69-100`); `humantime` accepts forms (`1h 30m`, `2weeks`, fractional units) the reference would then have to document and bound. Below the `DEP` threshold (no substantial infrastructure) |
| Text validation | `unicode-segmentation` | **Not needed**: the bound is bytes (1 … 280) and the rule is "no control character" (`char::is_control`), both std |

---

## 5. Design

### 5.1 Module layout (`systems/notices/src/`)

```text
lib.rs            exports; the pack table in the module docs
topic.rs          TopicId, BodyKind, ValidFor, NoticeDuration, Topic
configuration.rs  NoticesConfiguration, Defaults; impl PackConfiguration (FACTS, references, seed)
board.rs          NoticesProcess (never woken), BoardState, NoticeId; current(), issued_within()
body.rs           NoticeBody, PriceLine, NoticeText, NOTICE_PRICE_MAX, judge_body
component.rs      Subscriptions (component); SubscriptionsRecord, NoticeBoardRecord (disclosed)
section.rs        AuthoredNotices { subscribe }; impl AuthoredSection (Person only)
action.rs         PublishNotice, Subscribe, Unsubscribe; requirement NONE
judge.rs          judge_publish, judge_subscription (one decision for offers and validate)
event.rs          the five facts of §5.8
system.rs         declaration, validate, resolve, offers, discloses
codec.rs          payload readers per action
```

Each file stays under ~300 lines; `system.rs` dispatches only.

### 5.2 Configuration (`configure/notices.yaml`)

```yaml
# configure/notices.yaml
topics:                              # 1 … 32 topics; keys are dotted slugs (step-30 N-1's form)
  prices.ingredients:
    body: prices                     # prices | text   (the body-kind catalog, §5.5)
    publishers: [carol]              # 1 … 16 person keys; checked by the loader (references)
    valid_for: { min: "1h", max: "48h" }   # each "<n>m" | "<n>h" | "<n>d", within 1 min … 30 d, min ≤ max
    per_day: 4                       # 1 … 96 notices per publisher on this topic in any 24 h of world time (Q-S31b-4)
  news.daily:
    body: text
    publishers: [hana]
    valid_for: { min: "6h", max: "7d" }
    per_day: 1
defaults:                            # optional: default subscriptions by person tag
  by_tag: { barista: [prices.ingredients], staff: [prices.ingredients] }
```

- **Decoding is validating** (`ARC-61` item 1). `NoticesConfiguration { topics: BTreeMap<TopicId, Topic>,
  defaults: Defaults }`, `deny_unknown_fields` throughout; each bound a `try_from` whose message names the key:
  - `TopicId`: 1 … 64 bytes, segments `[a-z0-9]([a-z0-9-]*[a-z0-9])?` joined by `.`;
  - topics 1 … 32 (F-S31b-7), publishers 1 … 16 and no duplicate key;
  - `NoticeDuration`: `"<n>m" | "<n>h" | "<n>d"`, `n ≥ 1`, total 60 s … 2 592 000 s (30 d); `min ≤ max`;
  - `per_day` 1 … 96;
  - `defaults.by_tag`: keys are `Tag`s; each value a list of declared topics with no duplicate (an undeclared
    topic → "defaults.by_tag.<tag>: unknown topic `<t>`").
- **`references()`** lists every publisher key as a `Person`, so the loader refuses an unknown or non-person
  publisher by name and file (A-4).
- **`FACTS`** = `[notices-configured, subscriptions-seeded]`.
- **`seed`** states, in order:
  1. one `notices-configured { topics }` — the whole decoded topic table with durations in seconds and
     publishers as `PersonId`s — `SystemInternal`, no subjects (`ARC-61` item 8);
  2. for every living Person of the assembled world in `EntityId` order whose tags meet `defaults.by_tag`, one
     `subscriptions-seeded { person, topics: <sorted union>, origin: default }`, visibility per Q-S31b-2
     (recommended `SystemInternal`).

### 5.3 Where the board lives: `NoticesProcess`

`notices-configured`'s reduction starts one `NoticesProcess` (`ProcessTypeId` `notices-board`, owner
`NoticesSystem`, never woken: no due time). A second `notices-configured` is refused `FactRefusedByOwner`.

```text
BoardState {
  topics:  BTreeMap<TopicId, TopicState { body: BodyKind, publishers: BTreeSet<PersonId>,
                                          valid_for: (u32, u32) seconds, per_day: u8 }>,
  next:    NoticeId,                                   // u64, from 1; allocated by resolve, never reused
  latest:  BTreeMap<(TopicId, PersonId), Notice>,      // the newest notice of each (topic, publisher)
  issued:  BTreeMap<(TopicId, PersonId), VecDeque<WorldTime>>,   // issue times within the last 24 h, ≤ per_day
}
```

- **One writer:** only reductions of `notices-configured` and `notice-published` write it (I-S31-2).
- **Bounded:** `latest` ≤ 32 × 16 entries; `issued` ≤ 96 per key and pruned at each reduction to the last
  86 400 s. Older notices live in the log, the inbox and memory only, never in state.
- **"Current"** (step §3): `n.issued_at ≤ now < n.valid_until`, and no later notice from the same publisher on
  the same topic supersedes it. Because only the newest per (topic, publisher) is kept, `current(now)` is
  `latest.values().filter(|n| n.issued_at <= now && now < n.valid_until)`, ordered by `(topic, publisher)`.
  An expired newest notice means that publisher has nothing current, which is the step's rule (an older one is
  superseded anyway).

### 5.4 Subscriptions: section, component, genesis (the attribute until S30-c)

```yaml
# people/alice.yaml — notices' section; optional; when present it replaces the tag defaults
notices: { subscribe: [prices.ingredients, news.daily] }
```

- `AuthoredNotices { subscribe: Vec<TopicId> }`, `deny_unknown_fields`, carried by `Person` only; 0 … 32
  entries, no duplicate (decode refusals at line and column). `subscribe: []` is a valid authored "nothing",
  and it overrides the defaults.
- `seed` states one `subscriptions-seeded { person, topics: sorted, origin: authored }`, visibility per Q-S31b-2.
- **Component** `Subscriptions { topics: Vec<TopicId> }` on a Person, sorted, ≤ 32, written only by `notices`'
  reductions of `subscriptions-seeded`, `subscribed` and `unsubscribed`.
- **Reduction of `subscriptions-seeded`** (F-S31b-3): `origin: default` inserts the set when the person has
  none; `origin: authored` replaces whatever is there; each topic must be declared in the board's `topics`,
  else `FactRefusedByOwner` carrying `Rejection::System { code: unknown-topic }` — a `validate` refusal naming
  the code (A-5, A-6). With no `NoticesProcess` (a world that enables `notices` and authors sections but no
  configuration) every `subscriptions-seeded` is refused `notices-unconfigured` (S31-a SD-S31a-1's pattern).
- **Disclosure:** the `subscriptions` record (`{ topics }`) to the holder only (observer = subject), never to
  another observer (I-S31-9). Absent component → no record.
- **S30 shape compatibility (A-20).** The set is exactly step-30 N-2's `set` over `notices.topics` (≤ 32); the
  topic vocabulary is the configuration's `topics` keys (ARC-124 item 1: "from its world configuration"). The
  adoption PR replaces the section and `defaults.by_tag` with S30's resolution and states the same
  `subscriptions-seeded` per person from `attributes-handed` (R-S30c-b-1).

### 5.5 Bodies (the catalog) and `judge_body`

```text
NoticeBody = Prices(Vec<PriceLine { item: ItemId, price: u64, basis: u16 }>)    1 … 32 lines, strictly sorted by item
           | Text(NoticeText)                                                     1 … 280 bytes UTF-8, no control char
NOTICE_PRICE_MAX = 10^12        // = economy's PRICE_MAX by design (ARC-112), F-S31b-6
```

JSON (externally tagged, `rename_all = "snake_case"`): `{"prices": [{"item": 41, "price": 300, "basis": 6}]}` or
`{"text": "The market opens late tomorrow."}`.

`judge_body(world, topic, body) -> Result<(), Rejection>`, in order:

1. the body's kind equals the topic's `body`, else `body-mismatch`;
2. `Prices`: more than 32 lines → `body-too-large`; `Text`: more than 280 bytes → `body-too-large`;
3. `Prices`: zero lines; an `item` that is not a living entity of type `Item` (kernel identity, A-7); `price`
   0 or above `NOTICE_PRICE_MAX`; `basis` 0; items not strictly ascending (unsorted or duplicate) → each
   `body-invalid`, `detail` naming the line index and the reason (F-S31b-5);
   `Text`: empty, or a `char::is_control` character (newline included: one line, like an utterance) →
   `body-invalid`.

`notices` does not know what a price means (step §8.3): a line is an item and an amount the publisher claims,
with `basis` the publisher's own count of observations. Adding a kind (`departures`) is one variant here plus
its renderer, reviewed as a vocabulary change.

### 5.6 Actions, payloads, requirements, offers

| Action | Payload (JSON) | Actor | Requirement | Offer |
| --- | --- | --- | --- | --- |
| `publish-notice` | `{ topic, body, valid_for }`, `valid_for` in seconds (`u32`) | a living Person | `SpatialRequirement::NONE` (a bulletin is sent, like `send message`) | one **incomplete** offer, to a person who publishes at least one topic; `refused(PermissionDenied)` never offered (a non-publisher is not offered it) |
| `subscribe` | `{ topic }` | a living Person | `NONE` | one **incomplete** offer while some declared topic is not in its set (Q-S31a-3, ruled) |
| `unsubscribe` | `{ topic }` | a living Person | `NONE` | one **incomplete** offer while its set is non-empty (Q-S31a-3, ruled) |

- One incomplete offer per action type (S31-a SD-S31a-6: an incomplete offer carries no payload that could
  name a topic). A world with no configuration offers nothing.
- **No compatibility layer** is needed: everything is new.

### 5.7 Refusals, by name (`judge_publish`, `judge_subscription`; offers and validate call the same function)

| Name | Code / reason | When |
| --- | --- | --- |
| NotAPerson | kernel `NoSupportedInteraction` | the actor is not a living Person, or the world has no `NoticesProcess` |
| UnknownTopic | `unknown-topic` | the topic is not declared |
| NotAPublisher | kernel `PermissionDenied` (step §8.2: "`not-a-publisher` (`PermissionDenied`)") | `publish-notice` by a person the topic does not list |
| BodyMismatch | `body-mismatch` | §5.5 step 1 |
| BodyTooLarge | `body-too-large` | §5.5 step 2 |
| BodyInvalid | `body-invalid` (F-S31b-5) | §5.5 step 3 |
| ValidityOutOfRange | `validity-out-of-range` | `valid_for` outside the topic's `[min, max]` seconds |
| OverDailyLimit | `over-daily-limit` | this publisher already issued `per_day` notices on this topic in the last 86 400 s (Q-S31b-4) |
| AlreadySubscribed | `already-subscribed` | `subscribe` to a topic in the set |
| NotSubscribed | `not-subscribed` | `unsubscribe` of a topic not in the set |
| Malformed | kernel `PreconditionFailed` | a payload that does not decode (an unknown body kind, a negative number) |

`publish-notice` checks in the order NotAPerson, UnknownTopic, NotAPublisher, BodyMismatch, BodyTooLarge,
BodyInvalid, ValidityOutOfRange, OverDailyLimit; the order is fixed and tested, so two reasons at once always
report the same name. The codes are `Rejection::System { code }` built with `RejectionCode::from_static`
(compile-time checked).

### 5.8 Facts, components, records, version

| Item | Payload | Audience | Reduces to |
| --- | --- | --- | --- |
| `notices-configured` | `{ topics: [{ topic, body, publishers, valid_min, valid_max, per_day }] }` | `SystemInternal`, no subjects | starts `NoticesProcess` |
| `subscriptions-seeded` | `{ person, topics, origin: default \| authored }` | per Q-S31b-2 (recommended `SystemInternal`); `about: [person]` | `Subscriptions` (§5.4) |
| `subscribed` | `{ person, topic }` | `Entities({person})`, `about: [person]` | inserts the topic |
| `unsubscribed` | `{ person, topic }` | `Entities({person})`, `about: [person]` | removes the topic |
| `notice-published` | `{ notice: { id, topic, publisher, body, issued_at, valid_until } }` | `Entities(subscribers of the topic at that instant ∪ { publisher })`, `about: [publisher]`, participants `[publisher]` | `latest`, `issued`, `next` in `BoardState` |
| `Subscriptions { topics }` | component, owned by `notices` | — | — |
| `subscriptions` record | `{ topics }` | the holder only | — |
| `notice-board` record | `{ notices: [Notice] }`: `current(at())` (§5.3) of the observer's subscribed topics ∪ the topics it publishes (F-S31b-8), ordered by `(topic, publisher)` | the observer itself (observer = subject), a living Person; absent when the list is empty | — |
| `SystemVersion` | 1 (new pack) | — | — |

- `issued_at` is the dispatch's world time; `valid_until = issued_at + valid_for`. `resolve` reads the
  subscribers from `Subscriptions` components at that instant, so a person who subscribes later does not receive
  the old notice but sees it on the board while it is current (step §8.4).
- **Biographical:** none (`BIOGRAPHICAL = []`): reading the paper is not an event in a life.
- **Lazy expiry (I-S31-8):** no process wake, no expiry fact; the board is computed at disclosure from
  `WorldRead::at()` (A-9). Determinism: integer seconds, `BTreeMap` order, no floats, no draws.

### 5.9 The S28 renderer (`memory/render/notices.py`)

```text
notice-published, I am the publisher:
  "I posted <topic> (valid until Day <n> <HH:MM>): <summary>"
notice-published, otherwise:
  "<publisher name> posted <topic> (valid until Day <n> <HH:MM>): <summary>"
summary, prices: up to 6 lines "<item name> <price>" joined by ", ", then "… and <k> more"
                 (item name: names.name(item) or "item <id>"; amounts in minor units — the currency is
                 economy's record and is not read here, step-31 §11: S31-g adds `display`)
summary, text:   quote(text)   (QUOTE_LIMIT, «…»)
```

- Pure function `(fact, me, names) -> Rendered`; `salience` `notable`; `mine` = (publisher == me);
  `counterparts` = the fact's participants other than me (the publisher); importance `NOTICE = 400` (a notable
  fact I did not take part in, between the generic 300 and 500; recorded beside `pr-fg-b` §5.4's table as this
  PR's structural constant, never fitted, `ARC-23`).
- A payload that does not decode returns `fallback(...)` (counted), never an exception (the registry's rule).
- `valid until` uses `day_number` and `clock` (A-17), so the time reads in recall's own convention.
- If S28-d has merged at C5: `anchors` = the `prices` lines' items, in order (step §7.3). Otherwise none.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-b-1 (I-S31-1) | No path under `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `cognition/` except `cognition/lm-controller/{src/mineworld_cognition/memory/render/, tests/}`, and no `systems/*` except `notices` and `installed`, changes | C7 diff scan |
| I-b-2 (I-S31-2) | Only `notices` writes `Subscriptions` and `NoticesProcess`'s state, and only in reductions of its own five facts | `declaration()` review; `subscriptions.rs`, `publish.rs` |
| I-b-3 (I-S31-7) | A `notice-published` audience equals the topic's subscribers at that instant plus the publisher | `publish.rs` (pack) and AC-b-8 through `mineworld perceived` (binary); X-4 |
| I-b-4 (I-S31-8) | The board never shows a notice with `valid_until ≤ now` or a superseded one | `board.rs`; X-5, M-b-3 |
| I-b-5 (I-S31-9, the notices part) | `Subscriptions`, the `subscriptions` record and the board go to their holder only | `subscriptions.rs`, `board.rs`; M-b-4 |
| I-b-6 (I-S31-10, the notices part) | A world without `notices` loads and runs unchanged; every reference world's baselines stay byte-identical | AC-b-11 |
| I-b-7 | `offers` and `validate` call the same `judge_*`; an offer's refusal equals the dispatch's for the same state | `publish.rs::offer_and_dispatch_agree` |
| I-b-8 (Q-S31a-3) | No action of this pack is offered complete | `subscriptions.rs::every_offer_is_incomplete`; AC-b-6 |
| I-b-9 | Swapping the publisher's controller changes no contract: the notice is validated by the same `judge_publish` whoever submits it | review; AC-b-3 submits through the dispatcher directly, AC-b-8 through a protocol client |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt, clippy `-D warnings`, the `RejectionCode::from_static` literals; `ruff` and `pyright` over the renderer | CI `fast`, `core`, `python` |
| Unit (notices) | decode bounds at line and column (`TopicId`, `NoticeDuration`, counts, `defaults`); `judge_body` per reason; the rolling window arithmetic at its edges | `systems/notices/tests/configuration.rs`, `publish.rs` |
| Integration (notices, real kernel dispatch over a hand-built store with presence installed) | AC-b-1 … AC-b-7; X-4, X-5, M-b-1 … M-b-4; offers ≡ dispatch | `tests/{subscriptions,publish,board}.rs` |
| Python (renderer) | gists for both bodies, mine / not mine, truncation, fallback | `cognition/lm-controller/tests/test_memory_render.py` |
| Real binary | AC-b-8 (audience through `mineworld perceived`), AC-b-9 (kill-and-resume, replay), AC-b-10 (genesis refusals through `mineworld validate`), AC-b-6's run | `tools/cli/tests/notices.rs` |
| Long | AC-b-11: no reference world's baseline moves (the PR's CI `parity` group; no nightly dispatch needed, nothing is re-baselined) | CI |
| Gate 1 | NOT REQUIRED: no LM-facing surface changes in S31-b (the renderer is deterministic and tested by goldens; LM reading of notices is S31-d/S31-g) | — |
| Gate 2 | NOT REQUIRED: no client change | — |

Oracles are literal topics, subscribers and times written in each test, presence's export (`mineworld
perceived`), or the stored facts; never a value `notices` reports about itself (test rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

Fixture (pack tests and the scratch world alike): topic `news.daily` (`text`, publisher Pia, `valid_for`
1 h … 7 d, `per_day` 2) and topic `prices.ingredients` (`prices`, publisher Pia, 1 h … 48 h, `per_day` 4);
Sam and Sue subscribe to both; Nat subscribes to nothing.

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-b-1 | **Audience.** Pia publishes on `news.daily` at t = 10 000 s with `valid_for` 3 600: one `notice-published` whose audience is exactly {Pia, Sam, Sue}; Nat's perceived stream holds no fact of `notices` beyond genesis | I-S31-7; X-4 |
| AC-b-2 | **Expiry at the boundary.** Sam's board at t = 13 599 lists the notice; at t = 13 600 (= `valid_until`) it does not; no fact was stated between them; Sam's perceived stream still holds the `notice-published` | I-S31-8; X-5 |
| AC-b-3 | **Supersession.** Pia publishes again on `news.daily` at t = 11 000 (valid 7 200): the board at 12 000 lists only the second; at 18 200 neither | step §3 "current"; M-b-3 |
| AC-b-4 | **Each refusal by name, nothing moves:** `unknown-topic`, `PermissionDenied` (Sam publishes), `body-mismatch` (text on `prices.ingredients`), `body-too-large` (33 lines; 281 bytes), `body-invalid` (a person as `item`; price 0; price `10^12 + 1`; basis 0; unsorted; duplicate; empty text; a newline), `validity-out-of-range` (3 599; 7 d + 1 s), `over-daily-limit` (a third `news.daily` within 86 400 s of the first; accepted again at first + 86 400), `already-subscribed`, `not-subscribed`, `NoSupportedInteraction` (a world with the pack enabled and no configuration) — each by dispatch and, where an offer exists, by the offer's refusal; state bytes unchanged | step §8.2 |
| AC-b-5 | **Subscriptions route.** Nat subscribes to `news.daily` at t = 10 500: Nat does not receive the t = 10 000 notice, sees it on the board until 13 600, and receives the next one; Sue unsubscribes at 10 600 and does not receive it; each change is one `subscribed` / `unsubscribed` visible to that person only | step §8.4, §8.5 |
| AC-b-6 | **No random subscription or publication.** Every offer of the pack is incomplete (I-b-8). A scratch copy of `social-cafe` with `notices` enabled, `news.daily` published by `alice` and two seats subscribed, run headless 30 days, seed 7, with every seat on its bundled controller: the log holds no `subscribed`, `unsubscribed` or `notice-published` | Q-S31a-3; M-b-1 |
| AC-b-7 | **Genesis.** With `defaults.by_tag { barista: [prices.ingredients] }`: a barista without a section is subscribed by default; a barista with `notices: { subscribe: [] }` is subscribed to nothing; a person with a section naming an undeclared topic fails `mineworld validate` naming `unknown-topic`; each configuration bound fails at its line naming the key; a non-person publisher fails naming the key | step §8.1, §8.5; F-S31b-3 |
| AC-b-8 | **Real binary audience.** On the scratch fixture world under `mineworld server --save`, a protocol client on Pia's seat publishes; `mineworld perceived <save> --person <key>` lists the notice for Pia, Sam and Sue, and not for Nat | I-S31-7 (binary path) |
| AC-b-9 | **Kill and resume.** After AC-b-8's publish the server is SIGKILLed and started again with the same command: the restarted server's frame for Sam discloses the same `notice-board` record; a second publish gets `NoticeId` = first + 1; `mineworld replay` reproduces the save | step row "kill-and-resume"; `ARC-25` |
| AC-b-10 | **Memory.** For a subscriber's perceived `notice-published`, the renderer's gist equals the golden for each body kind (prices with 8 lines truncated to 6; text quoted), `mine` is true only for the publisher, and a malformed payload renders `fallback` | step §8.6, §16.4 |
| AC-b-11 | **Optional and removable.** Every reference world's keys in the branch's CI `parity` record equal `scripts/baselines.txt` on `main` (no world enables the pack); the scratch world with `notices` removed from `systems:` (and its sections and configuration) runs | AC-S31-9; I-S31-10 |
| AC-b-12 | **No kernel, no contract.** I-b-1's diff scan is empty | I-S31-1 |

### 8.1 Planted mutations (each must turn its test red by name, then be reverted from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-4 | `notice-published` audience `Public` | AC-b-1 (Nat's stream holds it) and AC-b-8 |
| X-5 | the board omits the `valid_until` filter | AC-b-2 |
| M-b-1 | `subscribe` offered complete, one per topic | `every_offer_is_incomplete`; AC-b-6's run states a `subscribed` |
| M-b-2 | the rolling window never forgets (`issued` not pruned) | AC-b-4's "accepted again at first + 86 400" arm |
| M-b-3 | the board keeps every notice instead of the newest per (topic, publisher) | AC-b-3 |
| M-b-4 | `discloses` returns `subscriptions` to any observer | `subscriptions.rs::only_the_holder_sees_its_subscriptions` |
| M-b-5 | the reduction of an `authored` `subscriptions-seeded` merges instead of replacing | AC-b-7's `subscribe: []` arm |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31b`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full.
2. Make the numbered steps, in order. Do not do a later commit's work.
3. Run the commit's validation. Record per command: test count, wall time, PASS / FAIL / INCONCLUSIVE.
4. For each listed mutation: copy the file aside, plant, run the named test, record the red (test name, code),
   restore from the copy (`cmp` clean), run green.
5. Static: `cargo fmt --all --check`; `cargo clippy -p mineworld-notices --all-targets -- -D warnings` (and
   `-p mineworld-installed-systems` in C2). Python: `uv run --locked ruff check` and `uv run --locked pyright` on
   the touched files. Markdown: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
   **R-LOAD-1:** never `cargo test --workspace`, never all of `tools/cli`; one `--test <file>` at a time.
6. Review: answer the commit's review questions in §15, one or two sentences each.
7. `git status --short` and `git diff --stat`: every changed path is in the commit's **Files**, or revert it or
   stop and record why.
8. Update §15 (ledger row, evidence, findings) and `handoff-s31b.md` in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the session's attribution line.
   No heredoc writes, no `sed -i`, no `python3 -c`.
10. Stop and report a material stop (§12) instead of improvising.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s31b.md` (new).
- **Steps:**
  1. Record the base commit. Quote the primary's dispatch confirming Milestone F's checklist was handed over;
     missing → material stop.
  2. Record the merge state of S31-a, S30-c, S28-d and #258, and choose the branch of §2.3 for each.
  3. Re-read A-1 … A-24 on the base; record moved lines (bounded) or changed statements (stop if one changes a
     decision here). In particular: the test client `restart.rs` uses, and `mineworld perceived`'s arguments.
  4. Confirm the freeze header and §14.1's rulings; apply nothing that is not ruled.
- [ ] Implementation. [ ] Validation: the two doc scripts. [ ] Review: §2.3 holds on the base.
- **Commit message:** `docs(plan): S31-b — open the ledger at the base`

#### C1 — Specifications first

- **Goal:** the decisions exist before the code they govern (`CLAUDE.md` §2.2).
- **Files:** `docs/DECISIONS.md` or `docs/decisions/{ARC-115.md, ARC-124.md, INDEX.md}` (SD-S31b-12),
  `docs/MODULE_SPEC.md`.
- **Steps:**
  1. `ARC-115` in the log's record format: problem (step §0.1's requirement), options (a broker — declined, §4;
     happenings' `Public` announcements — declined, they reach everybody; **one pack owning topics,
     subscriptions and notices, audience = subscribers, lazy expiry** — chosen), the choice (§5.2 … §5.8 as ruled
     by §14), consequences (presence's audience rule, F-S31b-1; the body-kind catalog as code; the rolling
     window; the S30 adoption; the duration parser decision of §4).
  2. A dated note under `ARC-124`: S31-b ships `notices`' own section and `defaults.by_tag` and adopts
     `AttributeOwner` in the PR after S30-c's merge, stating the same facts (R-S30c-b-1).
  3. `MODULE_SPEC.md` §4.1: a `notices` row (people: `{ subscribe: [topic] }`) and a `configure/notices.yaml`
     entry pointing to the generated reference.
- **Validation:** `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`
  (`--write-index` first if #258 merged).
- [ ] Implementation. [ ] Validation. [ ] Review: no new ontology term (`CLAUDE.md` §2.1 rule 3); every code of
  §5.7 is in `ARC-115`; nothing contradicts a SD of §11.
- **Commit message:** `docs: public notices — ARC-115, the notices pack (S31-b)`

#### C2 — The pack, its configuration and subscriptions at genesis

- **Files:** `systems/notices/{Cargo.toml, README.md (stub), src/{lib.rs, topic.rs, configuration.rs, board.rs,
  component.rs, section.rs, event.rs, codec.rs, system.rs}, tests/{support/mod.rs, configuration.rs,
  subscriptions.rs}}`; `systems/installed/{Cargo.toml, src/lib.rs}`; `Cargo.lock`; `docs/reference/*` as
  `mineworld reference` regenerates them; `worldpack/tests/refusals.rs` (one case: the section refused on an
  Item file by name and at its line, S26-a's install precedent, A-14).
- **Steps:**
  1. The crate (§2.1 item 1's dependencies), `NoticesSystem`, `SystemPack` with `configures!()` and
     `owns_section!()`, `BIOGRAPHICAL = []`.
  2. `TopicId`, `BodyKind`, `NoticeDuration`, `ValidFor`, `Topic`, `NoticesConfiguration` with their bounds and
     `JsonSchema` (§5.2).
  3. `NoticesConfigured`, `SubscriptionsSeeded`; `PackConfiguration::seed` (§5.2 items 1–2); `NoticesProcess`
     and `BoardState` (§5.3), started by the reduction.
  4. `AuthoredNotices` and its seed (§5.4); `Subscriptions`; the reduction rules of §5.4 with
     `unknown-topic` and `notices-unconfigured`.
  5. `discloses`: the `subscriptions` record to the holder only.
  6. Install (`Notices` last in `installed!`); regenerate `docs/reference/` with `target/debug/mineworld
     reference --out docs/reference`.
- **Validation:** `cargo test -p mineworld-notices --test configuration` (every bound at line and column;
  a non-person publisher; `defaults` with an unknown topic); `--test subscriptions` (AC-b-7 by the hand-built
  store; `only_the_holder_sees_its_subscriptions`, M-b-4; M-b-5); `cargo test -p mineworld-installed-systems
  --test installed`; `cargo test -p mineworld-worldpack --test refusals`; `target/debug/mineworld reference
  --check`.
- [ ] Implementation: steps 1–6. [ ] Validation, M-b-4, M-b-5. [ ] Review: no system dependency declared; the
  configuration fact is `SystemInternal`; genesis visibility is the ruled one (Q-S31b-2).
- **Commit message:** `feat(notices): topics as world configuration, subscriptions seeded from files and tags (S31-b)`

#### C3 — Subscribe and unsubscribe

- **Files:** `systems/notices/src/{action.rs, judge.rs, event.rs, codec.rs, system.rs, lib.rs}`,
  `systems/notices/tests/subscriptions.rs`.
- **Steps:**
  1. `Subscribe`, `Unsubscribe`, `Subscribed`, `Unsubscribed`; codecs.
  2. `judge_subscription` (§5.7) shared by `offers` and `validate`; `resolve` states the fact (§5.8); reductions.
  3. Offers: one incomplete each (§5.6, Q-S31a-3).
- **Validation:** `--test subscriptions` (`already-subscribed`, `not-subscribed`, `unknown-topic`; audiences
  `Entities({person})`; `every_offer_is_incomplete`, M-b-1's pack half).
- [ ] Implementation: steps 1–3. [ ] Validation, M-b-1. [ ] Review: no complete offer anywhere in the crate
  (`git grep -n "Offer::complete" systems/notices` empty).
- **Commit message:** `feat(notices): people subscribe and unsubscribe by their own choice (S31-b)`

#### C4 — Publishing, the audience, the rolling limit, the board

- **Files:** `systems/notices/src/{body.rs, action.rs, judge.rs, event.rs, board.rs, codec.rs, component.rs,
  system.rs, lib.rs}`, `systems/notices/tests/{publish.rs, board.rs}`; `systems/notices/Cargo.toml` (only the
  dev-dependency on `mineworld-economy` if S31-a has merged, SD-S31b-7).
- **Steps:**
  1. `NoticeBody`, `PriceLine`, `NoticeText`, `NOTICE_PRICE_MAX`, `judge_body` (§5.5).
  2. `PublishNotice`, `NoticePublished`; `judge_publish` (§5.7, the fixed order); the rolling window over
     `issued` (Q-S31b-4).
  3. `resolve`: allocate `NoticeId` from `next`; read subscribers at that instant; state the fact with the
     audience of §5.8.
  4. The reduction into `latest`, `issued`, `next` (§5.3).
  5. The `notice-board` record (§5.8; F-S31b-8), computed from `at()`.
  6. The incomplete `publish-notice` offer (§5.6).
- **Validation:** `--test publish` (AC-b-1, AC-b-4, AC-b-5, `offer_and_dispatch_agree`; X-4; M-b-2);
  `--test board` (AC-b-2, AC-b-3; X-5, M-b-3; a later subscriber sees the current notice).
- [ ] Implementation: steps 1–6. [ ] Validation, X-4, X-5, M-b-2, M-b-3. [ ] Review: the audience is read from
  components at resolve time, never from the reduction; the board reads `at()` and no wall clock; no float, no
  `HashMap`.
- **Commit message:** `feat(notices): named publishers post typed, time-limited notices to their subscribers (S31-b)`

#### C5 — The memory renderer

- **Files:** `cognition/lm-controller/src/mineworld_cognition/memory/render/{notices.py (new), __init__.py}`,
  `cognition/lm-controller/tests/test_memory_render.py`, its golden file as `capture_golden_renders.py`
  documents.
- **Steps:** the renderer of §5.9; its registry line (`"notice-published": notices.notice_published`); cases for
  AC-b-10; the goldens captured by the script's documented command and reviewed by hand against §5.9's rule.
- **Validation:** `uv run --locked pytest cognition/lm-controller/tests/test_memory_render.py`; ruff; pyright.
- [ ] Implementation. [ ] Validation. [ ] Review: pure (no store, no clock, no I/O); the golden's text was read
  against the rule, not accepted from the capture alone (test rules §25).
- **Commit message:** `feat(cognition): subscribers remember the notices they received (S31-b)`

#### C6 — The real-binary checkpoint

- **Files:** `tools/cli/tests/notices.rs` (new).
- **Steps:**
  1. A scratch fixture built by copying `worlds/social-cafe` (the `market_composition.rs` copy pattern), adding
     `notices` to `systems:` and `configure:`, writing `configure/notices.yaml` (§8's fixture: Pia, Sam, Sue,
     Nat mapped to four Social Café seats, named in the test) and the persons' `notices:` sections as literal
     text.
  2. `audience_through_perceived` (AC-b-8): the restart pattern's server and client (A-22); the client
     submits `publish-notice`; read `mineworld perceived` per person.
  3. `kill_and_resume` (AC-b-9).
  4. `genesis_refusals_by_validate` (AC-b-7's binary arms).
  5. `no_random_subscription` (AC-b-6): `mineworld run <copy> --headless --seed 7 --days 30 --save`; scan the
     save. CI only (R-LOAD-1).
  6. `removable` (AC-b-11's scratch arm).
- **Validation:** `cargo test -p mineworld-cli --test notices` locally for steps 2–4 and 6 (each a short run);
  step 5 in CI only; record wall times as INCONCLUSIVE.
- [ ] Implementation: steps 1–6. [ ] Validation. [ ] Review: the oracle is presence's export and the stored
  facts; the kill is proven (exit status, short head).
- **Commit message:** `test: notices reach exactly their subscribers and survive a kill (S31-b)`

#### C7 — Close-out

- **Files:** this file (§15), `handoff-s31b.md`, `systems/notices/README.md`.
- **Steps:** the README (what the pack owns, its file shapes, its codes, linking `ARC-115`); AC-b-12's diff scan;
  fill §8's evidence; push `mvp0/pr-s31b`; open the PR; CI on the exact head (Linux, macOS, Windows) green,
  repair to green; confirm AC-b-11 from the PR's `parity` record.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded; §11's parent
  edits listed for the primary.
- **Commit message:** `docs(plan): S31-b close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S31d-b-1 (on S31-d).** The reporter publishes `{ topic, body: { prices: [...] }, valid_for }` through
  `publish-notice` by name (ARC-34); a pricing policy reads only the `notice-board` record (current notices),
  never remembered ones (X-6); the deterministic and the LM backend submit the same request shape, validated by
  the same `judge_publish` (AC-S31-11).
- **R-S31e-b-1 (on S31-e).** The board panel renders the `notice-board` record; the codes of §5.7 get words in
  `en.po` / `zh_Hans.po`; the client evaluates no rule (no expiry computed client-side: the record is already
  current).
- **R-S31f-b-1 (on S31-f).** Market Town's topic id is `prices.ingredients` (step-31 §8.5); its `per_day` reads
  as "in any 24 h" (Q-S31b-4); its subscriptions use `defaults.by_tag` or sections until the S30 adoption PR.
- **R-S31g-b-1 (on S31-g).** `View.notices` reads the `notice-board` record; `MeLine.subscriptions` the
  `subscriptions` record; the inbox reads `notice-published` by name; the typed tools submit §5.6's payloads.
- **R-S30c-b-1 (on the PR after S30-c's merge).** `notices` implements `AttributeOwner` for
  `notices.subscriptions` (step-30 N-2 … N-5): it ships the declaration with `owner: notices`, receives the
  resolved set through `attributes-handed`, states the same `subscriptions-seeded` it states today (one per
  person, `origin` from S30's resolution), and drops its own `defaults.by_tag` in the same PR; disclosure moves
  to `disclose_owned`. In a world without `attributes`, the section path remains (Q-S30c-2: the attribute is
  not shown there, the state still exists).
- **R-S28d-b-1 (on S28-d, or the first PR after it).** The notice renderer gains `anchors` (a prices body's
  items), giving item-anchored price memory (step-31 §7.3).

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31b-1 | Topics, the counter and current notices live in a never-woken `NoticesProcess` | A-8: no entity carries world configuration; WE-a / S31-a precedent |
| SD-S31b-2 | Only the newest notice per (topic, publisher) is state; older ones live in the log, inboxes and memory | §5.3: "current" needs nothing more, and state stays bounded |
| SD-S31b-3 | Tag defaults are seeded by the configuration; an authored section's fact replaces them at reduction | F-S31b-3 |
| SD-S31b-4 | `unknown-topic` in a section is refused at reduction (a `validate` refusal), not at decode | A-5, A-6; S31-a SD-S31a-1 |
| SD-S31b-5 | One incomplete offer per action type | S31-a SD-S31a-6; Q-S31a-3 |
| SD-S31b-6 | `body-invalid` added for malformed lines and text | F-S31b-5 |
| SD-S31b-7 | `NOTICE_PRICE_MAX = 10^12`, asserted equal to economy's `PRICE_MAX` by a dev-dependency test once S31-a has merged | F-S31b-6 |
| SD-S31b-8 | At most 32 topics per world, hence ≤ 32 subscriptions, with no subscription-limit code | F-S31b-7 |
| SD-S31b-9 | The board also lists the topics the observer publishes | F-S31b-8 |
| SD-S31b-10 | `valid_for` in the payload is integer seconds; the configuration's bounds accept `"<n>m|h|d"` | typed boundary (`CLAUDE.md` §4 rule 7); §4's parser decision |
| SD-S31b-11 | The renderer prints amounts in minor units with no currency | `notices` does not read economy; S31-g adds `display` (step §11) |
| SD-S31b-12 | `ARC-115` goes where the decision log lives on C1's base (one file before #258, `docs/decisions/ARC-115.md` after) | A-24; no content difference |
| SD-S31b-13 | Text bodies are one line (a newline is a control character) | an utterance's convention; a bulletin board line, not a document |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-b — the notices System Pack (topics, subscriptions, notices,
                         the board, the memory renderer)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31b-notices.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§0, 1, 3, 8, 13, 14, 15, 16.2, 16.4, 16.5, 18, 19);
                         pr-s31a-currency-asks.md §14.1 (Q-S31a-3); step-30-person-attributes.md §9.6;
                         pr-s30c-attributes-pack.md (R-S31-c-1, Q-S30c-2); step-28-shared-memory.md §5;
                         docs/CORE_CONCEPTS.md §§12, 15.2; docs/MODULE_SPEC.md §4.1; docs/DECISIONS.md
                         ARC-26, ARC-31, ARC-33, ARC-34, ARC-43, ARC-61, ARC-90, ARC-124;
                         .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31b, its own, held by one session
BRANCH:                  mvp0/pr-s31b, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-31 §16.5, "Nothing starts
                         before Milestone F's checklist has been handed to the operator") AND this design
                         frozen by the operator AND the primary session's dispatch. **Implementation waits
                         for the Milestone F checklist; it is not met at 2026-10-18.** No PR must merge first
                         (step-31 §16.2: parallel with S31-a)                                (step; primary)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–5, as §14's rulings shape them                     (operator, pending)
FROZEN INVARIANTS:       step-31 I-S31-1, I-S31-2, I-S31-7 … I-S31-10 (their S31-b parts); §6 I-b-1 … I-b-9;
                         §8 as written; Q-S31a-3 (incomplete offers)
SEQUENCE:                C0 … C7 (§9)
ALLOWED COMMANDS:        cargo build / test -p <crate> --test <file> / clippy -p / fmt; target/*/mineworld *;
                         uv run --locked pytest|ruff|pyright on named files; git; gh (pr create/view/checks,
                         run view/download; never merge); python3 scripts/check_doc_headings.py,
                         check_decision_ids.py (--write-index), ci_layer.py --list; mkdir -p; cp; cmp;
                         sha256sum/shasum; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or all
                         of tools/cli locally (R-LOAD-1); reading any key or .env; calling any model; editing
                         a path outside I-b-1's allowance; a nightly dispatch (nothing is re-baselined);
                         merging
MATERIAL STOPS:          Milestone F not handed over; a needed change under I-b-1's forbidden paths (in
                         particular presence's audience rule or the kernel's process API); a needed new
                         external dependency (DEP); any reference world's baseline moving (AC-b-11); S30-c or
                         S28-d merged with a shape that §2.3's rules do not cover; AC-b-6 showing an NPC
                         publishing or subscribing; any change to a frozen step invariant or acceptance
PLATFORMS:               Linux, macOS, Windows (CI core and python jobs on all three)
VALIDATION BUDGET:       notices test files unrestricted locally, one at a time; tools/cli notices.rs steps 2–4
                         and 6 locally (single file, short hosted runs), step 5's 30-day run in CI only
                         (≤ 120 s on Linux CI); the renderer's pytest file locally. Gate 1: NOT REQUIRED.
                         Gate 2: NOT REQUIRED. No paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to review readiness: authorized       (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — never by the implementation session; only with the
                         operator's explicit authorization after the primary's and an Opus logic review
                         (working rules §22)
POST-MERGE SYNC OWNER:   primary: step-31, overall.md, the progress page; implementation session: this
                         ledger, the merge identity, evidence, deviations
STOP CONDITION:          S31-b READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31b-1 | S31-a and S31-b both edit `systems/installed`, `MODULE_SPEC.md` §4.1 and the decision log | one line each; the second to land rebases (§2.3); no shared semantic content |
| R-S31b-2 | The S30 adoption changes a fact's shape and forces a re-baseline of S31-f's Market Town | §5.4 states the same `subscriptions-seeded` either way (R-S30c-b-1); a shape change is a material stop for both (`ARC-124`) |
| R-S31b-3 | A large audience (every person subscribed) makes `notice-published` envelopes big | ≤ the world's population; the envelope's `BTreeSet` is what presence already reads; S28-g's 100-person scale gate covers it (S31-f subscribes 12) |
| R-S31b-4 | An LM publisher floods a topic | `per_day` (rolling), `valid_for` bounds, 32 lines / 280 bytes; refusals are named so a controller learns |
| R-S31b-5 | The hosted kill-and-resume test is flaky under CI load | the restart pattern's proven-kill check (IC-2); a timing failure is INCONCLUSIVE, never re-tuned silently |
| R-S31b-6 | Memory keeps expired price notices that an LM then quotes as current | the gist states "valid until"; the board (current) is what policies read (R-S31d-b-1); Q-S31b-3 records the open `(expired)` marker |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

Ruled by the operator on 2026-10-18 in the primary session: every question below is answered with its
recommendation.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S31b-1 [operator — step §8.5 "Order", F-S31b-10]** | S30-c (the attribute owner crate) is frozen but not merged. Should S31-b ship its own subscription section **and** `defaults.by_tag` (subscription by role tag), replaced when the S30 adoption PR lands — or ship only the per-person section now and leave role defaults to S30's `roles`? | **Ship both now** (the step's "Order" rule). A cook subscribing to ingredient prices by role is the operator's own example, S31-f needs it, and the adoption PR replaces it with S30's `roles` while stating the same facts (R-S30c-b-1). Shipping only sections would make S31-f write a section into every person file and then delete them again |
| **Q-S31b-2 [operator — visibility of genesis subscriptions, F-S31b-9]** | Step §8.5 makes subscription facts "visible to the person only". At genesis that puts one unwitnessed "subscribed" fact into every subscriber's inbox and S28 memory. Q-S31a-2 ruled the same question the other way for economy's configuration facts. Are genesis subscriptions `SystemInternal`? | **Yes: `subscriptions-seeded` is `SystemInternal`; runtime `subscribed` / `unsubscribed` stay `Entities({person})`.** The holder still sees its subscriptions at once through the `subscriptions` record; memory and the inbox keep only what happened in the world. The alternative (`Entities({person})`, like economy's `funded` today) is consistent with older section facts but clutters 12 inboxes in Market Town with a line no one perceived |
| **Q-S31b-3 [operator / primary — step §8.6 "Recall adds (expired)", F-S31b-4]** | Memory records keep a fixed gist and the fact's time; marking a notice "(expired)" at recall needs a new S28 field (`valid_until` on the record) that S28 owns. Amend the step's sentence for S31-b? | **Yes: the gist states "valid until Day n HH:MM", and recall already prints the record's own time, so a reader can tell; the `(expired)` marker moves to S28 as a bounded extension (beside S28-d's `Rendered.anchors`), not S31-b.** The board (current notices only) is what pricing policies read, so no behaviour depends on the marker. The alternative — S31-b edits S28's record schema — puts a memory schema change in a System Pack PR |
| **Q-S31b-4 [operator — `per_day`, F-S31b-2]** | The kernel has no "day" and `notices` depends on no system, so it cannot read `calendar`'s date. What does `per_day: 4` mean? | **"At most 4 notices on this topic by this publisher in any 24 hours of world time"**, a rolling window of 86 400 s; the key keeps its name and its generated reference says "in any 24 h". It needs no dependency, works in a world without `calendar`, and is what a rate limit means. The alternative — a calendar-day limit — makes `notices` depend on `calendar` (or on the world's epoch being midnight), against step §8.1 |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] ledger and handoff opened | [x] doc scripts (below) | [x] §2.3 holds on the base |
| C1 | (this commit) | [x] `docs/decisions/ARC-115.md`, the `ARC-124 note` heading in `docs/decisions/ARC-124.md`, the regenerated `INDEX.md`, MODULE_SPEC §4.1 row and configure paragraph | [x] `check_decision_ids.py --write-index`: 176 ids, all distinct, index current; `check_doc_headings.py` clean | [x] no new ontology term; every §5.7 code is in ARC-115; no SD contradicted |
| C2 … C7 | — | [ ] | [ ] | [ ] |

### Evidence

- **C0 (2026-10-18).** Base `origin/main @ a1e05874` (#273 merged). Dispatch: the coordinator's task of
  2026-10-18 to the S31+S29 lane lead — "Prepare both now: implement locally on branches from main, and push
  when the coordinator says F's checklist is out, or once 12d and 16c have merged." Milestone F's checklist is
  **not yet handed over**, so C0 step 1's material stop is read as applying to the **push** (§9 C7), not to
  local commits: the coordinator authorized local implementation explicitly. Nothing is pushed before that
  condition holds. Merge states at the base: S31-a (#230 design frozen; implementation not merged), S30-c (design
  frozen #253; no `systems/attributes` crate on main), S28-d (not designed or merged; `Rendered` has no
  `anchors`), #258 (open: the decision log is still one file). §2.3's branches chosen: ARC-115 in
  `docs/DECISIONS.md`; no economy dev-dependency (SD-S31b-7 waits for S31-a); no `anchors` in the renderer.

### Mutations

(none yet; X-4, X-5, M-b-1 … M-b-5 as §8.1 lists them)

### Deviations and findings

F-S31b-1 … F-S31b-10 (§3.2) and SD-S31b-1 … 13 (§11), recorded at design time.

- **D-S31b-3 (2026-10-19, rebase).** The branch was rebased from `a1e05874` onto `origin/main @ 44f050e3`,
  where #258 (the decision-log split), #261 (ARC-153, one MCP `act` tool) and #171 (12d) had merged.
  SD-S31b-12's second branch now applies: C1 writes `docs/decisions/ARC-115.md`, adds a `## ARC-124 note —`
  heading to `docs/decisions/ARC-124.md` and regenerates `docs/decisions/INDEX.md`; `docs/DECISIONS.md` is
  main's rules-only file again. The record text is unchanged from the first C1 (moved by `cp` from a copy
  of that commit's text).
- **D-S31b-1 (C1, process).** ARC-115's text was appended to `docs/DECISIONS.md` with a shell heredoc
  (`cat >>`), which §12's NEVER list forbids. The content was reviewed in full after the write and the doc
  checks ran on it; later commits use the editor tools only. No effect on content.
- **D-S31b-2 (C1, bounded).** `MODULE_SPEC.md` §4.1 said "nine sections"; with `notices` it says "ten". The
  separate stale sentence "An eighth section, `attributes`" predates this PR and is left to S30-c.
