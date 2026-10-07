# Step 10 / PR 11 — Market Town and the AC-1 composability proof (S9)

**Role:** step document for S9, proposing a split into six PRs (§2.8). It also holds the full PR
design for the first of them, **PR 11a**, and a proposed execution contract for it (§11). PRs 11b–11f
are specified at medium scope and are each re-audited and detailed to the commit only after the PR
before them merges (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 S9, §4 (`AC-1`, `AC-2`), §7 (F-1, F-3)
**Lifecycle:** step design `DESIGN FROZEN (2026-10-07)` for §§1–3 and the six-PR split; **PR 11a**
`DESIGN FROZEN (2026-10-07)` with its execution contract (§11) confirmed. PRs 11b–11f stay at medium
scope until each is detailed and frozen in turn.

**Freeze record (2026-10-07).**

The operator approved how AC-1 is measured (QS-2 → `ARC-35`), in reply to the primary session's
question. The approval covers these points:

- Three precursor PRs, 11a–11c, add framework capability and name no market concept.
- The transformation is the merges of 11d and 11e only. Those merges may touch only `systems/**`,
  `worlds/**`, `Cargo.lock` and Markdown documentation, and three independent checks enforce this.
- Statically linked installation still needs two lines in `systems/installed` and a rebuild. This
  is recorded as stated.

That approval also covers QS-3 (the F-1 mechanism, `ARC-33`, `systems/installed`) and QS-4 (the
additive `payload` on `Affordance`, and the paced controller's offer band). The primary session
accepts QS-1 and QS-5 to QS-9, QS-11, QS-12 and QS-14, as recommended. Kinds and counts
(QS-6, `ARC-36`) are accepted for MVP-0. QS-13 stays out of S9.

QS-10 is recorded as an MVP-0 gap, to be placed with the operator later: nothing eats or sleeps,
so held items are never used up.

**Two conditions, binding:**

1. **I-2 is checked mechanically, not just stated.**
   - Each of 11a, 11b and 11c adds a test, or extends one shared test, that scans the PR's added
     lines in identifiers, tests and fixtures for the market vocabulary (`item`, `inventory`,
     `money`, `price`, `wage`, `job`, `shift`, `shop`, `economy`, `employ`).
   - The scan has an explicit allow-list for pre-existing uses, e.g. `items/` in MODULE_SPEC §4 and
     11b's content-kind names, and every allow-list entry carries its reason.
   - It must be shown to fail on a planted violation.
   - A precursor that turns out to need a market word is a material stop.
2. **ARC-33's record carries the static-linking boundary.** It states that installation means
   "add a directory, two lines in `systems/installed`, rebuild", and that installation without a
   rebuild belongs to the WASM plugin model (`ARC-8`), outside MVP-0. A reader must not take
   "independently installable" as more than that.

Implementation of 11a starts in a fresh session on `mvp0/pr-11a-installable`, in its own worktree.

**Base:** `main @ b9e5937` (S8 complete: 10c merged as `9ab5e62`; `b9e5937` is the docs-only
post-merge update).
**Branch / worktree:** `mvp0/s9-plan` in `/Users/yuema137/mineworld-worktrees/s9-market`, held by this
planning session only. Implementation of any PR starts in a fresh session on its own branch.

**Depends on (merged):**
- S3/S4: `System`, the registry and its dependency and borrowed-vocabulary rules, `Process`, `wake`.
- S5: `PersistentWorld`, `ARC-25`.
- S6: `ARC-26` (an event type's owner is its vocabulary and its reducer).
- S7: `mineworld run`, `PacedRuleController`, `ARC-27`.
- S8: `ARC-28` (a subscriber needs no system dependency; decode through the owner's types),
  `ARC-29` (biography), `ARC-31` (sections), `ARC-32` (agendas that controllers follow).

Binding:
- [`CLAUDE.md`](../../../CLAUDE.md) §1 (the frozen top-level criterion), §2.2, §4 rules 1, 2, 3, 4, 5,
  7, 8, 9, 10, 11, 13, 15, 16
- [`docs/MVP.md`](../../../docs/MVP.md) §§1–5, §9 (`AC-1`, `AC-2`, `AC-5`, `AC-6`, `AC-9`, `AC-11`,
  `AC-12`, `AC-13`), §10
- [`docs/HUMAN_REVIEW_QUEUE.md`](../../../docs/HUMAN_REVIEW_QUEUE.md), Milestones C and E
- [`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) §§1, 2, 3, 7, 8, 9, 10, 11, 13, 15
- [`docs/MODULE_SPEC.md`](../../../docs/MODULE_SPEC.md) §§3, 4, 4.1, 5, 9, 10
- [`docs/PACKAGE_FORMAT.md`](../../../docs/PACKAGE_FORMAT.md) §§6, 8
- [`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) §§2, 9, 12, 14
- [`docs/ENGINEERING_RULES.md`](../../../docs/ENGINEERING_RULES.md) §§3, 7–9, 13, 20, 22
- [`docs/ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md) §§5–9, 12, 16, 28–30
- [`docs/REUSE_POLICY.md`](../../../docs/REUSE_POLICY.md) §§2, 3, 6, 11, 12, 17
- [`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `ARC-8`, `ARC-15`, `ARC-23`, `ARC-25` … `ARC-32`,
  `DEP-10`
- [`step-09-social.md`](step-09-social.md) §8.2 F-1, F-3, F-4; §10.1 ("S9's design must resolve both,
  or record why `AC-1` is still met without resolving them")

## Why this is PR 11, and why the file is `step-10`

The overall plan calls this step S9. S8 shipped as PR 10 (10a, 10b, 10c), so S9 ships as **PR 11**,
split here into 11a … 11f. The step-document number follows the file sequence
(`step-09-social.md` was S8), so this file is `step-10-market.md`.

---

# 1. Goal

S9 settles the project's frozen top-level criterion:

> **The framework must demonstrate that materially different games can be constructed by composing
> the same core entities with different independently installable interaction systems, without
> modifying the kernel.**

Concretely (`overall.md` §1, §3 S9; `MVP.md` §2):

```text
worlds/social-cafe   (presence, movement, conversation, group-activity, relationships, naming,
                      schedule)
        + install item, inventory, item-transfer, economy, employment
        + change world configuration
        = worlds/market-town          own items · exchange items · work · earn money · spend money
                                      · run a shop

with a change set that touches only systems/ and worlds/,
and no change to kernel/, contracts/, a controller, or a renderer
```

S8 forwarded two findings that decide whether that sentence can be true as written (step-09 §10.1):

- **F-1.** Installing a System Pack today edits `worldpack/src/catalog.rs` (a closed enum with seven
  `match` statements), `worldpack/Cargo.toml`, the root `Cargo.toml` (`members` and
  `[workspace.dependencies]`) and `Cargo.lock`. So a pack is not independently installable, and no
  linked Rust pack can satisfy "touches only `systems/` and `worlds/`".
- **F-3.** `PacedRuleController` submits only actions it knows by name. A new pack's actions go
  unused by every headless person until the controller is edited, and `AC-1` forbids that edit. A
  market town whose people never buy or work would pass a path check and prove nothing (`ARC-23`).

The S9 checkpoint, and Milestone C, are the floor of this design:

```text
CP-1  AC-1, mechanically: the change set that turns social-cafe into market-town touches only the
      allowed paths (§2.5), adds no external dependency, and leaves kernel/, contracts/, every
      controller and every renderer byte-for-byte unchanged — checked by a test, not by reading
CP-2  F-1 resolved: installing a System Pack edits no file outside systems/ except the generated
      Cargo.lock, demonstrated by a real install before any market pack exists (§2.2)
CP-3  F-3 resolved: a headless person attempts an action of a pack the controller has never been
      compiled against, decided by the offering system's own complete affordance (§2.3)
CP-4  the market lives: a seeded 300-day headless market-town run has, in every 30-day bucket, at least
      one purchase, one wage paid, one item produced and one item given — located before counted
      (ARC-23) — and replays byte for byte (AC-11, AC-12)
CP-5  single ownership (CLAUDE.md §4 rule 1): only economy writes money; employment states wage-due
      and never touches a wallet; only inventory writes holdings; every cross-pack effect is a fact
      its owner reduces (ARC-26, ARC-28)
CP-6  AC-2 for item-transfer, as MVP.md §9 words it: disabling it removes item giving, changes no other
      pack, and the world still runs
CP-7  Milestone C: work → earn → buy → holdings change → another client perceives it → it survives a
      server restart (HUMAN_REVIEW_QUEUE)
CP-8  social-cafe is unaffected: its 300-day seed-7 run is byte-identical before and after S9
```

## 1.1 Scope, by PR

```text
PR 11a  installable System Packs (F-1)                   (fully designed in §4.1)
  sdk/rust/                    new crate mineworld-sdk: the SystemPack trait a pack implements, the
                               SectionOwner record, and the `installed!` macro
  systems/installed/           new crate mineworld-installed-systems: the build's installed set —
                               one line per pack, the only list of packs in the build
  systems/*/                   each of the seven packs implements SystemPack (its id, biographical
                               types and section, said once, by itself)
  worldpack/                   catalog.rs loses every per-pack arm; Cargo.toml loses every pack it
                               does not read for a format field
  Cargo.toml (root)            members: "systems/*" and "sdk/rust"; two workspace dependencies
  docs                         DECISIONS (DEP-12, ARC-33, ARC-35 — how AC-1 is measured),
                               MODULE_SPEC §3.1, systems/README, sdk README
  tests (existing)             worldpack/tests/refusals.rs: the unknown-system fixture stops being
                               called `economy` (F-10), claim unchanged

PR 11b  items and organizations in the World Pack format (MODULE_SPEC §4's frozen model, F-4)
  authoring/                   ContentKind gains Item and Organization
  worldpack/                   world.yaml `items:` and `organizations:`; `items/<key>.yaml` and
                               `organizations/<key>.yaml` carrying tags, note and sections; entity ids
                               allocated after people, so every existing id stays put
  docs                         DECISIONS (ARC-36 — an authored Item is a kind), MODULE_SPEC §4.1,
                               PACKAGE_FORMAT §8

PR 11c  complete affordances: a controller acts on what it is offered (F-3)
  contracts/                   Affordance gains an optional payload: the complete request the
                               offering system would accept, typed by the action it was offered as
  systems/presence/            Offer::with_payload::<A>; observe() carries it through
  cognition/rule-controller/   PacedRuleController gains one band: attempt an available complete
                               affordance, seeded; nothing else changes
  server/PROTOCOL.md, clients/protocol/mineworld/   the field documented and readable (no client
                               decides anything new)
  docs                         DECISIONS (ARC-34), CORE_CONCEPTS §15.2, MODULE_SPEC §5

PR 11d  the transformation, part 1 — owning and giving things      (AC-1 range)
  systems/item/, systems/inventory/, systems/item-transfer/; systems/installed/ (three lines)
  worlds/market-town/          social-cafe's content verbatim, plus item kinds, holdings, and the
                               three packs enabled

PR 11e  the transformation, part 2 — work, money and shops         (AC-1 range)
  systems/economy/, systems/employment/; systems/installed/ (two lines)
  worlds/market-town/          organizations, wallets, shops and prices, jobs

PR 11f  the proof
  tests/acceptance/            new crate: the AC-1 composability test (change set, dependency
                               structure, configuration-only world delta)
  tools/cli/tests/             market-town's AC-11/AC-12 run with the market precondition; AC-2 for
                               item-transfer at world level; Milestone C through the real server
  docs                         MVP_STATUS, HUMAN_REVIEW_QUEUE (Milestone C), README of market-town,
                               DECISIONS note, overall §7
```

## 1.2 Non-goals

```text
eat, sleep, hunger, energy (MVP §5's eat and sleep)        a needs System Pack, not in S9's list (QS-10)
unique item instances, item condition, temperature         CORE_CONCEPTS §7 allows stacked kinds; S9
                                                           ships stacks only (QS-6)
property, business ownership, hiring and firing, promotion MVP §4 non-goals or later packs
dynamic loading of packs at runtime; WASM Tier 1           ARC-8; out of MVP-0 (overall §1 non-goals)
`mineworld install` / `add-system` commands                MODULE_SPEC §8 says not in MVP-0
names for items, places or organizations                   naming carries people only (ARC-31 limits)
a 2D or 3D client that renders a shop or an inventory      S12 / S14; clients stay generic
the reactive RuleController (--agent) acting on offers     unchanged (AC-15; I-5)
System Pack configuration (world-tunable constants)        first pack that needs a world-specific value
                                                           (ARC-26 note); prices are content, not config
a market that balances itself over years                   CP-4 asks that the market lives in every
                                                           bucket of 300 days, nothing more (R-S9-3)
```

## 1.3 Frozen invariants (proposed; frozen only by the primary session)

- **I-1 The transformation touches only what AC-1 allows.** The merge diffs of PR 11d and PR 11e, each
  against its own first parent, contain only paths in the allowed set of §2.5. Any other path in
  either diff is a material stop, never a bounded deviation.
- **I-2 The precursors know no market.** PRs 11a, 11b and 11c name no item, inventory, money, price,
  wage, job, shift or shop in any identifier, test or fixture they add. Each is justified and proven
  with packs that already exist or with a synthetic test-only pack. A precursor shaped by what
  market-town needs, rather than by what any pack needs, would be the transformation hidden inside
  the framework.
- **I-3 Single ownership.**
  - `ItemKind` is written only by `item`'s reductions.
  - `Holdings` (what a Person or an Organization holds) only by `inventory`'s.
  - `Wallet` and `Shop` only by `economy`'s.
  - `Employment`, the `employed-by` edge and the shift Process only by `employment`.
  - A change to another pack's state travels only as a fact the owner reduces: stated in the owner's
    vocabulary under `ARC-26`, or reacted to under `ARC-28`.
- **I-4 Existing worlds are unchanged.** The 300-day seed-7 `social-cafe` run prints the same lines
  (all but `wall`) on `main @ b9e5937` and after every S9 PR (§9 E-0 records the baseline). The
  precursors change no existing test's claim; a literal update is listed with its unchanged claim, as
  step-09 I-5 required.
- **I-5 Controllers stay stateless, and the reactive controller does not change.**
  `PacedRuleController::decide(&self, &Observation)` stays a pure function of seed, pace and
  observation (`ARC-27`). `RuleController` and `--agent` behave exactly as today (`AC-15`).
- **I-6 No floating point.** Money is integer minor units; holdings are integer counts; wages and
  prices are integers.
- **I-7 Activity before determinism, for market-town (ARC-23).** CP-4's precondition is checked before
  any comparison of market-town runs. A comparison made without it is not evidence.
- **I-8 The kernel crate is untouched by S9.** `kernel/` has no diff in any S9 PR. `contracts/` changes
  only in PR 11c, and only by the additive field of §2.3.
- **I-9 The paced controller's offer band is decided before the market exists.** Its constants are
  fixed in 11c against a synthetic pack. If market-town later behaves badly, the remedy is in
  `systems/` or `worlds/` — prices, wages, stock, what a pack offers — never in the controller. A
  controller retuned for the market would fail AC-1 in substance while passing it in form.

---

# 2. Re-audit: what AC-1 needs that does not exist yet

## 2.1 What exists, from source (`main @ b9e5937`)

| S9 need | State | Where |
| --- | --- | --- |
| Item and Organization as entity types | **exist in contracts, unused** | `contracts/src/ids.rs:698–709` (`EntityType::Item`, `::Organization`), `ItemId` (:850), `OrganizationId` (:911). `World::create_entity` takes any `EntityType` (`kernel/src/world.rs:571`). No World Pack can declare one: the loader reads `places` and `population` only. |
| a content kind for items and organizations | **specified, not implemented** | `MODULE_SPEC.md` §4 lists `items/` and `organizations/` in the frozen World Pack layout; §4.1's implemented subset omits them; `authoring::ContentKind` has `Person` and `Place` only (`authoring/src/section.rs`). |
| per-pack content seeding | **done** (10c, `ARC-31`) | `AuthoredSection`; worldpack decodes a section with its owner's type and seeds it. Sections are carried by people and places only. |
| a System Pack registration point | **closed enum** | `worldpack/src/catalog.rs`: `Capability` (7 variants), `AVAILABLE`, and seven `match`es — `id`, `section`, `decode_section`, `biographical`, `install`, `provider`, plus `Display`. F-1, §2.2. |
| cross-pack facts | **done** | `ARC-26` (state another pack's vocabulary: declare it, depend on the owner); `ARC-28` (react to another's fact: subscribe, decode through its crate, no system dependency). |
| Process owned by a pack, woken over time | **done, in real use** | group-activity (`systems/group-activity/src/process.rs`), schedule's routine process (`systems/schedule/src/process.rs`). |
| presence facts a pack can react to | **done** | group-activity reacts to `person-entered-place` (step-09 SD-10). |
| a controller that acts on what it is offered | **missing** | `PacedRuleController` reads affordances only to *check* actions it already knows: `walk` (`paced.rs` `walk`), `may_talk_to` (`lib.rs`), `available::<A>` (`social.rs`). F-3, §2.3. |
| an affordance a controller can submit without knowing the action | **missing** | `Affordance` = action type, target, availability, reason, requirement (`contracts/src/observation.rs:172`). No payload: a client or controller must already know how to build the request. |
| a client that shows unknown actions | **done, generically** | `clients/protocol/demo/demo.gd:483–505` lists every affordance by `action_type` with a `VERB` fallback to the raw type. It can submit only `talk` (`:278`). |
| an acceptance-test home | **empty** | `tests/` is an empty directory; `ARCHITECTURE.md` §14 names it "conformance and acceptance tests". `sdk/` is empty; §14 names `sdk/rust`, and overall §3 schedules "sdk/rust with S9 (Rust pack authoring)". |

## 2.2 F-1: why a pack is not installable today, and what installation can honestly mean in MVP-0

### Every place a System Pack is named outside its own directory (audited)

```text
Cargo.toml (root)          members: 7 lines "systems/<name>"; [workspace.dependencies]: 7 lines
Cargo.lock                 one [[package]] per pack, and every dependent's dependency list
worldpack/Cargo.toml       7 dependencies, one per pack
worldpack/src/catalog.rs   Capability: 7 variants; AVAILABLE: [Capability; 7]; id, section,
                           decode_section, biographical, install, provider: one arm per pack each
worldpack/tests/social_cafe.rs   the composition list (Capability::Presence … ::Schedule) — a test of
                           social-cafe, which legitimately names its own composition
cognition/rule-controller/Cargo.toml   conversation, movement, group-activity, naming, schedule — the
                           vocabulary the paced controller is written against (F-3, §2.3)
tools/cli/Cargo.toml       conversation, naming, presence (+ dev: group-activity, relationships,
                           schedule) — naming for biography names, presence for observe()
```

So adding one pack today costs about eleven edits in five files, three of them outside `systems/`, and
eight of them in one Rust file that has to be understood to be edited. That is the
change-amplification pattern (`ENGINEERING_STANDARDS.md` §8) in its plainest form, and `ARC-31`'s
accepted limitations already say so: "The catalog is still a closed list compiled into the build
(F-1). The seam removes the format edit, not the registration."

### What cannot be removed, and why

MVP-0's System Packs are **trusted, statically linked Rust** (`ARCHITECTURE.md` §12 "v0 trusted Rust
systems"; `PACKAGE_FORMAT.md` §8 "MVP-0 ships trusted in-process Rust systems"). In a statically linked
Rust program, a crate is in the binary only if some crate in the build declares it as a Cargo
dependency. No mechanism — not `build.rs`, not linker-section registration, not a macro — can make
Cargo link a crate nobody declares. Every option below therefore keeps **one declarative line**
naming the pack in some manifest; the question is only where that line lives and what else must
change with it.

Dynamic installation without a rebuild is `ARC-8`'s WASM component model (Tier 1), explicitly outside
MVP-0 (overall §1 non-goals: "WASM plugin sandbox"). Loading native libraries at runtime would need
`unsafe`, an unstable Rust ABI and a trust model this project has rejected for downloaded code
(`PACKAGE_FORMAT.md` §6.1). So in MVP-0, **installing a System Pack means adding it to the build's
installed set and rebuilding**, and F-1 is resolved when that act is one declarative line in a place
whose only purpose is the installed set — with no code anywhere else that has to learn the pack.

### Options considered (`REUSE_POLICY.md` §§11–12, §17 — both directions)

```text
(a) status quo: a closed enum in worldpack, one arm per pack     ~11 edits in 5 files; F-1 stands
(b) linker-section registration — the `inventory` crate          still needs a dependency line AND a
    (dtolnay; MIT/Apache-2.0) or `linkme` (distributed_slice)    `use pack as _;` line, or the linker
                                                                 drops the unreferenced crate;
                                                                 registers values, so the generic
                                                                 section decoder needs type erasure
                                                                 (option d); life-before-main (`inventory`)
                                                                 or per-platform linker support (`linkme`)
(c) dynamic loading — `libloading`, abi_stable                   unsafe, ABI-fragile, a native-code
                                                                 trust model; ARC-8 already chose WASM
(d) a value registry with type-erased section decoding —         one new dependency; whether
    `erased-serde`                                               serde-saphyr's line and column survive the
                                                                 erasure round trip is unverified, and
                                                                 they are DEP-10's reason for the parser
(e) worldpack generic over a Catalog type supplied by the binary  every caller of WorldPack::read (CLI,
                                                                 server tests, persistence tests,
                                                                 worldpack's own tests) changes; a
                                                                 dev-dependency cycle for worldpack's tests
(f) build.rs scanning systems/*/Cargo.toml to generate the list  Cargo still needs the dependency line;
                                                                 parsing TOML in a build script is a
                                                                 dependency (`toml`) for a two-line saving
(g) a SystemPack trait each pack implements, and an installed-    chosen
    set crate whose only content is the list, expanded by a
    declarative macro into today's closed enum
```

**Choice: (g)** (SD-1 … SD-4, §3). Every pack declares itself once, in its own crate, by implementing
`SystemPack`. The build's installed set is a crate in `systems/installed/` whose manifest names each
pack once and whose `lib.rs` is one macro invocation listing them. The macro expands into exactly the
closed enum and the seven `match`es `catalog.rs` holds today, so every code path — in particular the
monomorphic section decode that keeps line and column (`DEP-10`) — is unchanged. Root `members`
becomes the glob `"systems/*"` (verified on `b9e5937`: `cargo metadata` resolves all fifteen members
with the glob and `systems/README.md` present; reverted, §8.2 F-9). A new pack depends on its sibling
packs by `path`, so the root `[workspace.dependencies]` need not learn it.

Installing a pack after 11a:

```text
systems/<new>/                         the pack                                   (new directory)
systems/installed/Cargo.toml           mineworld-<new> = { path = "../<new>" }     (one line)
systems/installed/src/lib.rs           <Variant> => mineworld_<new>::<NewSystem>,  (one line)
Cargo.lock                             regenerated by Cargo                        (generated)
```

Why not `inventory`/`linkme` (b), stated because rejecting a mature crate needs a record as much as
adopting one (`CLAUDE.md` §4 rule 16): they solve *distributed registration of values*, and MineWorld's
remaining cost after (g) is not registration code — it is the one Cargo line no crate can remove. They
would add a dependency and life-before-main (or per-platform linker support) to save one list line,
and they would force the section decoder from a generic function into a type-erased value, which is
(d)'s unverified risk to `DEP-10`. This is a case of "forcing an existing wheel where it does not fit"
(`REUSE_POLICY.md` §17), not of reinventing one: (g) is a `macro_rules!` over the code that already
exists. Recorded as **DEP-12** (proposed), with the trigger for revisiting it — the first build that
installs packs it does not compile from this repository, which is `ARC-8`'s Tier 1.

### Why AC-1 then holds, precisely

`MVP.md` §2 forbids modifying "kernel, Person, renderer, controller implementations", and §9's `AC-1`
row says the same. After 11a, installing `economy` touches none of those: it adds `systems/economy/`,
two lines in `systems/installed/`, and regenerates `Cargo.lock`. The overall's stricter gloss —
"touches only `systems/` and `worlds/`" — also holds, with `Cargo.lock` admitted as a generated
artefact under a check that it adds only path packages under `systems/` (§2.5). That admission and the
placement of the installed set under `systems/` are **operator-material** (QS-3), because they decide
how the frozen criterion is measured.

What 11a does **not** give is Milestone E's "a real world assembled from independently installable
packs" in the publishing sense (`.mwpack`, a registry, packs from outside this repository). That needs
`ARC-8`'s Tier 1 and stays a later step; 11a is the MVP-0 form of it, and says so.

## 2.3 F-3: why a controller cannot use a new pack, and how it can without knowing it

### What the controller can and cannot do today (audited)

`PacedRuleController::decide` (`cognition/rule-controller/src/paced.rs`) is a fixed priority list over
named actions: answer an invitation (`social.rs`, group-activity's types), reply (`lib.rs`, `Talk`),
follow the agenda (`agenda.rs`, schedule's `Agenda`), leave/invite/join (`social.rs`), then greet,
approach, wander or use a door (`Talk`, `Move`). Every request it builds is typed by a pack crate it is
compiled against (`cognition/rule-controller/Cargo.toml`: conversation, movement, group-activity,
naming, schedule). It reads affordances, but only to ask "is *this* action, which I know, available?"
(`may_talk_to`, `walk`, `available::<A>`).

So a market pack's `buy` would appear in every observation as an affordance, and nothing headless
would ever submit it. The reason is not that the controller is badly written. It is that an
`Affordance` carries no payload (`contracts/src/observation.rs:172`): to submit an offered action, a
requester must already know the action's payload shape. The same is true of a client — `demo.gd`
lists every affordance generically and can submit only `talk`.

`CORE_CONCEPTS.md` §15.2 already states the intent this gap blocks: "a language-model controller is
told what is possible instead of guessing". An affordance tells it *that* something is possible, not
*what exactly to send*.

### Options considered

```text
(a) the controller learns market actions                 the edit AC-1 forbids; every future pack
                                                         repeats it
(b) a Controller Pack policy as world data (YAML: "at a   payloads authored in a World Pack are rules
    shop, attempt buy {item: coffee}")                   in content (MODULE_SPEC §4 constraint 3),
                                                         and the policy still has to know payloads
(c) a generic `interact` action the server resolves      the server would choose the action for the
    (ENGINEERING_RULES §8's InteractIntent)              person (INV-1, INV-6); one resolver must
                                                         know every pack's precedence — a God object
(d) payload schemas in affordances (JSON-Schema-like)     a second schema language for an enumerable
                                                         problem; heavy before any LM controller exists
(e) scope AC-1's "controllers" so headless people        the market would be untestable headless: an
    need not use new packs                               instrument that cannot see (ARC-23)
(f) complete affordances: the offering system may         chosen
    attach the exact payload it would accept; any
    requester may submit it unchanged
```

**Choice: (f)** (SD-9 … SD-12, §3). An affordance gains an optional `payload`: the complete request
payload, typed by the action the offer was made as (the type is read off `A` exactly as `Offer::new`
reads it today, so a payload cannot be mislabelled). An affordance with a payload is a **complete
affordance**. The owning system decides which of its offers are complete — `buy { item: coffee }` once
per item a shop has, `give { item, count: 1 }` once per item the giver holds — and the server still
revalidates whatever is submitted (`ARCHITECTURE.md` §9). The paced controller gains exactly one
band: when its observation holds an available complete affordance, it attempts one, by a seeded draw,
at a fixed rate. It needs no pack's types for that.

What (f) does not cover, recorded rather than hidden: actions whose payload is free-form — `talk`'s
utterance, `move`'s position, `invite`'s kind — cannot be enumerated by the offerer, and the paced
controller keeps knowing those by name. They are the foundation vocabulary of a walking, talking world,
and a new pack whose actions are free-form is usable headless only through an LM controller (S10) or
by offering complete affordances for a bounded choice.

Why this is **not** an AC-1 violation although it edits contracts, presence and the controller: it is
a precursor PR (11c) that lands before the transformation, names no market concept (I-2), is proven
with a synthetic pack the controller has never been compiled against, and leaves social-cafe
byte-identical, because no existing pack offers a complete affordance (I-4). Its constants are frozen
before the market exists (I-9). The transformation PRs then change no controller, and the AC-1 test
checks that the controller crate has no dependency path to any market pack (§2.5). Whether the
precursor approach is an acceptable reading of the frozen criterion is **operator-material** (QS-2,
QS-4), and the public contract change in `contracts/` is operator-material on its own.

## 2.4 Items and organizations: what the World Pack must be able to say (F-4)

`MVP.md` §3 asks for ~20 item types and two Organizations (café, store) with two jobs. Today no World
Pack can declare an Item or an Organization entity (§2.1). Three ways:

```text
(a) a pack creates Item/Organization entities at genesis   systems cannot create entities: WorldView has
                                                           no create (kernel/src/view.rs); adding one is a
                                                           kernel change
(b) no Item entities: item kinds as slugs inside an         an item kind would have no identity, no tags,
    inventory, organizations as tags on places             and nothing for item, economy and employment to
                                                           share; an Organization is a CORE_CONCEPTS §8
                                                           primitive, not a tag
(c) complete MODULE_SPEC §4's frozen layout: `items/` and   chosen; ids allocated after people, so no
    `organizations/` content files, each an entity with     existing id moves
    tags, a note and sections
```

**Choice: (c)**, as PR 11b, a precursor: it implements part of a *frozen* format model (`MODULE_SPEC.md`
§4 already lists `items/` and `organizations/`), names no market concept (I-2), and is proven by
refusal and loading tests with tags-only files. Its sections are seeded by the same code path as
people's and places' (`ARC-31`), first exercised by a real owner in 11d.

**What an authored Item is (QS-6, ontology).** `CORE_CONCEPTS.md` §7 separates `ItemType` from
`ItemInstance` and allows "unique items, stacked items, or abstract resources". S9 ships **stacked
items only**: an Item entity declared in `items/coffee.yaml` *is the kind* `coffee`, and what a person
holds is a count of it. Unique instances (`coffee_18517`, with a temperature) are a later pack's, and
would be Item entities created by that pack's process — which needs the kernel change in (a), recorded
as the reason they are out of S9. Because this reads a defined term (`Item`) in one of two ways the
ontology permits, it is raised as operator-material rather than assumed.

## 2.5 How AC-1 is measured (CP-1)

The AC-1 test is three independent checks, so that no single blind spot passes it (`ARC-23`):

```text
1  the change set       for each transformation merge commit M (11d, 11e):
                        git diff --name-only M^1 M  ⊆  allowed
                        allowed  = systems/**  ∪  worlds/**  ∪  Cargo.lock
                                   ∪  documentation: **/*.md under docs/, systems/, worlds/ and
                                      .structured-coding/plans/
                        Cargo.lock: every [[package]] added between M^1 and M has no `source`
                        (a path package) and lives under systems/; every [[package]] whose
                        dependency list changed lives under systems/ — no external dependency
                        arrives with the market
2  the structure        from `cargo metadata` at HEAD, independent of history:
                        - the direct dependents of each market pack are systems/* crates only
                        - no path leads to a market pack from kernel, contracts, persistence, server,
                          authoring, sdk or rule-controller
                        - no code file outside systems/, worlds/ and tests/acceptance/ (*.rs,
                          Cargo.toml) names a market pack's crate (`mineworld-economy`,
                          `mineworld_economy`, …). Crate names, not action or event type slugs:
                          contract tests already use stub ids such as `inventory-stub` and
                          `item-transferred` (contracts/tests/event.rs), which name no pack
3  the world delta      market-town is social-cafe plus configuration:
                        - systems: social-cafe's list, in order, then the five market packs
                        - places, population and seats: identical keys
                        - every person and place file: social-cafe's fields and sections unchanged,
                          plus sections owned by market packs only
                        - items/ and organizations/: present only in market-town
```

Check 1 needs git history (`fetch-depth: 0` in a future CI); without it the test **fails** naming the
missing history — it never skips (fail closed). Check 2 catches what a path diff cannot: a kernel or
controller that was taught the market *before* the transformation range. Check 3 is "produced from
social-cafe by changing configuration", made mechanical.

Two consequences are **operator-material** (QS-2): the transformation is defined as two named PRs
rather than "everything S9 changed", and documentation (Markdown only) is admitted in those PRs,
because a PR here always updates its ledger.

## 2.6 The five packs: who owns what

```text
            owns (single writer)                 states (facts)                provides
item        ItemKind on Item entities            item-kind-declared (genesis)  —
            ← `item:` section, items/*.yaml
inventory   Holdings on Persons and              stocked (genesis),            —
            Organizations ← `holdings:` section  items-transferred,
                                                 items-produced
item-       —                                    (inventory's items-transferred, give { item, count }
transfer                                          under ARC-26)                 target: a Person here,
                                                                                within 3 m; complete
                                                                                affordance per item held
economy     Wallet on Persons and Organizations  funded (genesis),             buy { item }
            ← `wallet:`; Shop on Places          money-transferred,            target: none; at a shop
            ← `shop:` { operator, prices }       wage-unpaid;                  place; complete
                                                 (inventory's items-transferred affordance per priced,
                                                  under ARC-26)                 stocked item
employment  Employment on Persons + the          hired (genesis), shift-started,
            employed-by edge (Person →           shift-ended, wage-due;        —
            Organization) ← `job:` section;      (inventory's items-produced
            the `shift` Process, one per job     under ARC-26)
```

Dependencies, one way, no cycle (`ARC-26` for stating another's vocabulary; `ARC-28` for reacting):

```text
item ◄── inventory ◄── item-transfer           (system dependency: states items-transferred)
              ▲  ▲
              │  └──── economy                 (system dependency: states items-transferred;
              │            ┆                     reads Holdings to price a shop's stock)
              │            ┆ subscribes to wage-due — Cargo dependency on employment's types only,
              │            ┆ no system dependency (ARC-28, step-09 Q6's condition)
              └──── employment ── presence     (system dependencies: states items-produced; reads
                                                Presence and reacts to person-entered-place)
```

**Money moves only in economy (CLAUDE.md §4 rule 1).** `employment` decides that a shift was worked
and states its own `wage-due { employee, employer, amount }`. `economy` reacts: it moves the amount from
the employer's Wallet to the employee's and records `money-transferred` caused by the `wage-due`, or
records `wage-unpaid` when the employer cannot pay. `employment` never reads or writes a Wallet. This
is `CORE_CONCEPTS.md` §13.1's canonical example, implemented literally.

**Holdings change only in inventory.** `item-transfer` decides a give is legal; `economy` decides a
purchase is legal; `employment` decides a shift produced something. Each states inventory's own fact
through inventory's checked constructor, and inventory alone reduces it and may refuse it (`ARC-26`).

**Work is attendance, not an action (QS-8).** A job names its workplace, its shift (time of day, as
schedule's convention) and its wage. Employment owns one `shift` Process per job, woken at the shift's
start and end. A person present at the workplace during the shift is working — read from presence's
state at a wake, and from presence's `person-entered-place` between wakes — and the worked seconds
become `wage-due` at the shift's end. No controller has to know that work exists: the people in
social-cafe already walk to their agenda's place (`ARC-32`), and market-town gives the two job holders
routines that take them there. A human player works by being there, which is also what a job is.

**Buying is at a shop, not from a person (QS-9).** A Place's `shop:` section names the operating
Organization and its prices. `buy { item }` is available to a person in that place when the operator
holds the item and the buyer can pay; otherwise it is offered unavailable with the reason. Economy
discloses the shop's listing (prices and what is in stock) to everyone who perceives the place, which
is how a second client perceives a purchase (CP-7) without anyone's Holdings being disclosed to a
stranger (`INV-13`).

**Run a shop** is then the composition, not a rule of its own: a job at the café produces coffee into
the café Organization's Holdings; customers buy it, paying the Organization; the Organization pays
wages from what it took in.

## 2.7 Change amplification (`CLAUDE.md` §4 rule 5), audited before designing

| Capability | New module | Registration (shared) | Other edits | Verdict |
| --- | --- | --- | --- | --- |
| installable packs | `sdk/rust`, `systems/installed` | root `Cargo.toml` (once, 11a) | each existing pack gains a ~10-line `impl SystemPack`; `worldpack/src/catalog.rs` shrinks | framework change, once; justified in §2.2 |
| items, organizations | — | — | `authoring` (`ContentKind`), `worldpack` (format, read, load) | completes a frozen spec; once (11b) |
| complete affordances | — | — | `contracts` (one field), `presence` (Offer, observe), `rule-controller` (one band), protocol docs | framework change, once (11c) |
| item, inventory, item-transfer | 3 new crates | `systems/installed` (3 lines) | none | **module + registration ✓** |
| economy, employment | 2 new crates | `systems/installed` (2 lines) | none | **module + registration ✓** |
| market-town | `worlds/market-town` | — | none | **configuration ✓** |

After 11a–11c, the market rows touch nothing but new modules and the installed set. That is the
change-amplification test passing, and it is what AC-1's check 1 measures.

**A trap found in the audit, and closed in 11a (F-10).** `worldpack/tests/refusals.rs:169–184`
(`an_unknown_system_is_refused_by_name_and_lists_the_ones_that_exist`) uses **`economy`** as the
example of a system the build does not provide. The day economy is installed, that test fails, and the
only fix is an edit to `worldpack/tests/` — outside the AC-1 range. So 11a changes the fixture's
unknown name to one no pack will ever take (a slug that is not a word, e.g. `no-such-system`), with
its claim unchanged. Every other test that names a market word outside `systems/` was audited and is a
contract-layer stub that names no pack (§8.2 F-10).

## 2.8 Why six PRs, and why this split

```text
11a  installable packs (F-1)       checkpoint: social-cafe byte-identical; a canary pack installed on a
                                   scratch branch touches only systems/ and Cargo.lock
11b  items and organizations       checkpoint: a pack with items/ and organizations/ loads; every
                                   existing id and genesis fact unchanged
11c  complete affordances (F-3)    checkpoint: a synthetic pack's action, unknown to the controller, is
                                   attempted and accepted headless; social-cafe byte-identical
11d  owning and giving             checkpoint: market-town (items) runs; people give; AC-2 for
                                   item-transfer; restart
11e  work, money, shops            checkpoint: shifts → wage-due → wages; purchases; production;
                                   restart mid-shift
11f  the proof                     checkpoint: AC-1 test green and shown to bite; CP-4; CP-7
```

- **Precursors first, each alone.** Each changes framework code for a reason that holds without the
  market, and each must be reviewable as such. Folding any of them into a transformation PR would put
  `contracts/` or `worldpack/` into the AC-1 range and fail the criterion by construction.
- **11b and 11c are independent of each other.** Both depend on 11a (11b edits `worldpack`, which 11a
  reshapes); they may run in parallel worktrees after 11a merges.
- **The transformation in two PRs.** Five packs and a world in one PR would be roughly 10b and 10c
  combined. Item, inventory and item-transfer form a complete capability (own and give) that 11d can
  check end to end; economy and employment depend on inventory and are checked together because a
  wage is meaningless without something to spend it on. Check 1 of §2.5 reads each merge against its
  own first parent, so unrelated PRs merging in between do not enter the range.
- **The proof last, separately.** The acceptance test reads the two transformation merges by their
  commit ids, which exist only after they merge. Market-town's world-level runs live in
  `tools/cli/tests/` and so cannot be inside the range; they land here.
- **Not fewer.** One S9 PR would mix framework contracts with market content and fail AC-1's own
  check. **Not more.** Splitting item from inventory would create a PR with nothing to check end to end.

---

# 3. Design decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-1** | **A System Pack declares itself.** A new trait `SystemPack: System + Default`, in a new crate `mineworld-sdk` (`sdk/rust/`), carries what the build needs to know about a pack beyond `System`: `BIOGRAPHICAL` (default `&[]`), `SECTION: Option<SectionOwner>` (default `None`) and `decode_section` (default: refuses, "owns no section"). A section owner writes `mineworld_sdk::owns_section!();` inside its `impl`, which defines `SECTION` and `decode_section` together from its `AuthoredSection` impl, so the two cannot disagree. `SectionOwner` moves from `worldpack::catalog` to the SDK unchanged. | §2.2 (g). What `catalog.rs` says about each pack today, said once by the pack. `sdk/rust` is where `ARCHITECTURE.md` §14 and overall §3 put Rust pack authoring. The SDK depends on `authoring`, `contracts`, `kernel` and `serde` — never on a pack, so `presence` can implement it without a cycle. |
| **SD-2** | **The installed set is a crate whose only content is the list.** `systems/installed/` (crate `mineworld-installed-systems`) depends on the SDK, on `presence` (for `PerceptionProvider`) and on every installed pack, and its `lib.rs` is one invocation: `mineworld_sdk::installed! { Presence => mineworld_presence::PresenceSystem, … }`. The macro expands to today's `Capability` enum, `AVAILABLE`, and the methods `resolve`, `id`, `section`, `owning_section`, `decode_section`, `biographical`, `install`, `provider` and `Display` — the same closed enum and matches, generated. | §2.2. Keeps every code path monomorphic, so the section decoder still reads straight from the YAML stream and keeps line and column (`DEP-10`). `provider` coerces the pack to `Box<dyn PerceptionProvider>` at the install site, so a pack that is not a perception provider does not compile into the set. |
| **SD-3** | **Root manifest: glob members, sibling packs by path.** `members` gains `"systems/*"` (replacing seven lines) and `"sdk/rust"`. `[workspace.dependencies]` gains `mineworld-sdk` and `mineworld-installed-systems`. Existing packs keep their `workspace = true` sibling dependencies; **a new pack depends on a sibling pack by `path = "../<name>"`**, so the root manifest never learns it. Verified: the glob resolves on `b9e5937` with `systems/README.md` present (§8.2 F-9). | The root manifest stops being a registration point. Path dependencies between sibling packs add no external dependency, so the root's "a crate that needs a dependency not listed here is adding a dependency" policy is untouched. |
| **SD-4** | **`worldpack` names only the packs its format fields belong to.** Its `[dependencies]` keep `presence` and `movement` (a person's `location` and a place's `passages` are format fields bound to them, `ARC-31` item 5) and gain `mineworld-sdk` and `mineworld-installed-systems`; the other five pack dependencies go. `catalog.rs` keeps `LOCATION_OWNER`, `PASSAGE_OWNER`, `opened`, `located` and the section-namespace guard, and re-exports `Capability`, `AVAILABLE` and `SectionOwner`. A structural test holds the allow-list. | §2.2. The public API of `mineworld_worldpack` does not change, so no caller changes (I-4). |
| **SD-5** | **Installing is two lines in `systems/installed/`, and nothing else is edited.** Shown before any market pack exists by a canary install on a scratch branch (§4.1 C5), and checked for good by AC-1's check 1 on 11d and 11e. | CP-2. Located before counted (`ARC-23`): the mechanism is demonstrated on a pack nobody needs before it is relied on. |
| **SD-6** | **How AC-1 is measured** is §2.5's three checks, the transformation being the merges of 11d and 11e. Recorded as a decision before any market code exists. | CP-1. A measurement chosen after seeing the result is not a measurement (`ARC-23`). Operator-material (QS-2). |
| **SD-7** | **Items and organizations are content kinds** (11b): `items:` and `organizations:` in `world.yaml`; `items/<key>.yaml` and `organizations/<key>.yaml` carry `tags`, `note` and sections. Entity ids: places, then people (unchanged), then items, then organizations, each in key order. Genesis: passages, locations, then sections in the order items, organizations, places, people. Keys remain one namespace across every kind. | §2.4 (c). Items and organizations are allocated after people, so no existing id moves; their sections seed first because they are what people's and places' sections refer to. |
| **SD-8** | **An authored Item is an item kind; holdings are counts** (stacked items, `CORE_CONCEPTS.md` §7). Unique instances are a later pack's. | §2.4. Operator-material (QS-6). |
| **SD-9** | **Complete affordances** (11c). `Affordance<P>` gains `payload: Option<P>`, serialized only when present. It is set only through `Offer::with_payload::<A>(&A)` in presence, where `A` is the action the offer was made for; a payload of another action type cannot be attached. `Affordance::request(actor)` turns a complete affordance into an `ActionRequest` with the affordance's own action type and target. | §2.3 (f). Additive and absent by default, so every existing observation, transcript and test is unchanged (I-4). Operator-material: a public contract (QS-4). |
| **SD-10** | **The paced controller attempts what it is offered.** One new band in `decide`, after the social initiative and before the walking scheme: if the observation holds at least one *available* complete affordance, then with draw index 14 below `ATTEMPTS_OFFERED` (proposed 20 of 100) it submits one, chosen by draw index 15 among them in observation order. No pack type is imported for it. | §2.3. Stateless (`ARC-27`). No existing pack offers a complete affordance, so social-cafe draws nothing new and decides byte-identically (I-4). Constants fixed in 11c (I-9). |
| **SD-11** | `RuleController` (`--agent`) does not attempt complete affordances. | I-5, `AC-15`. It answers; it takes no initiative. |
| **SD-12** | **Clients read the payload and decide nothing new.** `server/PROTOCOL.md` documents the field; `clients/protocol/mineworld/observation.gd` gains `payload(action_type, target)`; `ADOPTION.md` says a client may submit it unchanged. `demo.gd` is not changed in S9. | `ENGINEERING_RULES.md` §§8–9: a payload is the server's answer, carried; no rule moves into a client. Using it in the 2D client is S12's. |
| **SD-13** | **The market packs and their ownership** are §2.6. Work is attendance during a shift (QS-8); buying happens at a shop place (QS-9); economy reacts to employment's `wage-due` without a system dependency (`ARC-28`); item-transfer, economy and employment state inventory's facts under `ARC-26`. | CP-5. `CORE_CONCEPTS.md` §13.1. |
| **SD-14** | **market-town is social-cafe plus configuration**, checked by §2.5 check 3: the same places, people, seats, routines and names; the five packs appended to `systems`; items, organizations, holdings, wallets, shops and jobs added. Jobs are fitted to the routines people already have, because changing a routine would change social-cafe's configuration rather than add to it. | CP-1, CP-8. |
| **SD-15** | **The proof is a crate of its own** at `tests/acceptance/` (`mineworld-acceptance`), the home `ARCHITECTURE.md` §14 gives acceptance tests. It reads the two transformation merge commits by id, which are recorded in it in 11f. | §2.5, §2.8. |

---

# 4. Commit plan

Each commit tracks implementation, validation and review separately. Evidence goes into §9 as
`E-A<n>` for 11a. A planned commit may become several coherent commits; the mapping is recorded.
Line counts are estimates, never targets.

## 4.1 PR 11a — installable System Packs (full design)

### 4.1.1 Identity, base, approved scope

```text
PR            11a — installable System Packs (S9, first of six)
base          main @ b9e5937, or the main the primary session names at freeze (re-audit if it moved)
branch        mvp0/pr-11a-installable (proposed), in a worktree held by the implementing session only
audit         §8 (files and symbols read on b9e5937)
scope         §1.1 PR 11a; SD-1 … SD-6; F-10's fixture
```

**Acceptance (all observable, decided before measuring):**

```text
A-1  no behaviour changes: the 300-day seed-7 social-cafe run prints, apart from `wall`, exactly the
     lines of E-0 (sha-256 ad49c723…c64b over those lines); every existing test passes; the only
     edit to an existing test is F-10's fixture name, its claim unchanged
A-2  worldpack's [dependencies] and src/ name no System Pack crate other than presence and movement —
     a structural test over worldpack's own manifest and sources, shown to fail when a pack import
     is added back
A-3  the installed set is consistent: every pack crate systems/installed depends on is listed in its
     `installed!` invocation and vice versa, and no two listed packs share a SystemId — a test, shown
     to fail when a list line is removed
A-4  a canary pack installed on a scratch branch changes only systems/** and Cargo.lock (plus the
     scratch world that enables it under worlds/**); Cargo.lock gains exactly one path package under
     systems/ and changes only systems/installed's dependency list; `mineworld validate` of a world
     enabling it lists it. Evidence recorded; the branch is never merged
```

### C0 — Design (this document) — docs only

- [x] Implementation: §§1–11 of this file, from the audit in §8 (draft; not frozen).
- [x] Validation: `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py` (§9 E-0).
- [x] Review: every claim in §2 cites a file and line, a command, or a measurement; F-1 and F-3 are
  resolved or bounded, not routed around; the operator-material questions are marked (§10). Self-review
  by the drafting session only; the primary session's review is pending.

### C1 — Specs before code: DEP-12, ARC-33, ARC-35, MODULE_SPEC §3.1

**Goal.** The installation mechanism and the AC-1 measurement exist as reviewable specifications
before any code relies on them (`CLAUDE.md` §2.2).

**Scope.**
- `docs/DECISIONS.md`:
  - **DEP-12** — *System Pack registration: a declared installed set, not linker-section registration
    or dynamic loading.* §2.2's options (b)–(g), the reason each was declined, the isolating interface
    (`mineworld-sdk`'s `SystemPack` and `installed!`), the trigger for revisiting (`ARC-8` Tier 1).
  - **ARC-33** — *A System Pack is installed by declaring it in the build's installed set.* SD-1 …
    SD-5; what "independently installable" means in MVP-0 and what it does not (Milestone E's
    publishing sense).
  - **ARC-35** — *How AC-1 is measured.* SD-6 / §2.5, the transformation being named PRs, the allowed
    set, the `Cargo.lock` rule, documentation admitted, fail-closed without history.
  - Ids are proposals: before writing, `git grep "^## \(ARC\|DEP\)-"` over every `origin/*` branch
    carrying `DECISIONS.md`, and take the next free numbers if these were taken (record the mapping).
- `docs/MODULE_SPEC.md`: new §3.1 *Installing a System Pack in MVP-0* (the two lines, the
  `SystemPack` trait, path dependencies between sibling packs, what is refused and how). §8's
  sentence "`install` and `add-system` are not implemented in MVP-0" stays true and gains a pointer.
- `systems/README.md`: "Adding a pack" — the trait, the two lines, the sibling-path convention.

**Depends on:** freeze. **Non-goals:** no code.

- [x] Implementation: the three records, §3.1, the README section. `docs/DECISIONS.md` gains ARC-33,
  DEP-12 and ARC-35 (ids as proposed: no `origin/*` branch holds ARC-33+ or DEP-12+, re-checked
  2026-10-07 after `git fetch`). ARC-33 carries freeze condition 2 (the static-linking boundary) in
  its own paragraph; ARC-35 item 7 carries freeze condition 1 (how the I-2 scan works, including how
  "the PR's added lines" are determined — §9 E-A1). `docs/MODULE_SPEC.md` §3.1 and a pointer in §8;
  `systems/README.md` "Adding a pack".
- [x] Validation: `check_decision_ids` → 45 ids, all distinct; `check_doc_headings` → 143 numbered
  sections across 22 documents, none duplicated. Cross-references checked by grep: ARCHITECTURE §12
  ("v0 trusted Rust systems", :431) and §14 (`sdk/`, `tests/`), overall §1 non-goals ("WASM plugin
  sandbox", :72), ARC-5 (merge commits), ARC-8, ARC-26, ARC-28, ARC-29, ARC-31, DEP-10,
  `interaction.rs` (`PerceptionProvider`'s defaults return nothing, :85–93, :117–126), kernel
  `SystemDependencyMissing` (error.rs:210).
- [x] Review: DEP-12 answers §11's six questions (problem; options; choice; why ours — a
  `macro_rules!` over existing code; isolating interface; limitations and revisit trigger) and gives
  a §12 reason per declined option. ARC-35 states its relation to MVP §2 and overall §1's gloss
  ("What this does not claim"). "Installing" (into the build) and "enabling" (a world's `systems:`)
  are kept distinct, matching CORE_CONCEPTS' use of "enabled system" (INV-10); no defined term is
  redefined. Overall §1's gloss citing ARC-35 is a parent-document update left to the planning session
  (handoff).

**Acceptance.** As validation. **Commit boundary.** Documentation only.

### C2 — `sdk/rust`: the `SystemPack` trait, `SectionOwner`, `owns_section!` and `installed!`

**Goal.** The vocabulary a pack uses to declare itself, and the macro an installed set is written in.

**Scope.**
- `sdk/rust/Cargo.toml` — crate `mineworld-sdk`; dependencies `mineworld-authoring`,
  `mineworld-contracts`, `mineworld-kernel`, `serde` (all `workspace = true`). No pack.
- `sdk/rust/src/lib.rs` — crate documentation (what a pack declares and why, with the two-line
  install), `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, re-exports used by the macros
  (`$crate::__private::{…}` for `World`, `KernelError`, `SystemIdentity`, `SystemId`, `EventTypeId`,
  `AuthoredContent`, `Decode`, `MapAccess`, `Arc`), so an installed set needs no dependency but the
  SDK, its perception trait's crate and its packs.
- `sdk/rust/src/pack.rs` — `pub trait SystemPack: System + Default` with `const BIOGRAPHICAL`,
  `const SECTION: Option<SectionOwner>`, `fn decode_section<'de, A: MapAccess<'de>>(map: &mut A) ->
  Result<Arc<dyn AuthoredContent>, A::Error>` (default: `A::Error::custom("the '<id>' system owns no
  section")`, the message `catalog.rs` gives today); `macro_rules! owns_section` defining both from
  `AuthoredSection` (`Some(SectionOwner::of::<Self>())`, `map.next_value_seed(Decode::<Self>::new())`).
- `sdk/rust/src/section.rs` — `SectionOwner { name, carried_by }`, `SectionOwner::of::<S>()`,
  `carried_by(kind)`, moved verbatim from `worldpack/src/catalog.rs:75–98`.
- `sdk/rust/src/installed.rs` — `macro_rules! installed`, input
  `perception: <path to the PerceptionProvider trait>; $( $Variant:ident => $System:path ),+`,
  expanding to: `pub enum Capability { $Variant, … }` with the derives today's has; `pub const
  AVAILABLE: [Capability; N]` in the listed order; `impl Capability` with `resolve`, `id`, `section`,
  `owning_section`, `decode_section` (now `pub`), `biographical`, `install`, `provider`, and
  `type_name` (from `core::any::type_name`, used only by the installed set's own guard); `impl
  Display`. Each method's doc comment is carried from `catalog.rs`.
- Root `Cargo.toml`: member `"sdk/rust"`; `[workspace.dependencies]` `mineworld-sdk = { path =
  "sdk/rust" }`.
- `sdk/rust/README.md` — the human orientation (`CLAUDE.md` §2.1): one paragraph and the two lines.

**Depends on:** C1. **Non-goals:** no pack implements the trait yet; `worldpack` unchanged.

- [x] Implementation:
  - [x] the crate and its four modules, as scoped: `sdk/rust/{Cargo.toml, README.md,
    src/lib.rs, src/pack.rs, src/section.rs, src/installed.rs}`; root `Cargo.toml` member `sdk/rust`
    and `mineworld-sdk` workspace dependency; `sdk/.gitkeep` removed (the directory has content).
  - [x] `installed!`'s expansion reproduces `catalog.rs`'s public surface method by method — read side
    by side: `resolve`, `id`, `section` (still `const fn`), `owning_section`, `decode_section` (now
    `pub`), `biographical`, `install`, `provider`, `Display`; plus `type_name` for the guard. Each
    method's doc comment carried. The C4 build and the existing suites are the check that matters.
  - Bounded detail: the list's system type is a `ty` fragment (`$System:ty`), not `path`, because it
    is used as `<$System as Trait>::…`; the perception trait stays a `path` (`dyn $perception`). Variant
    docs are generated ("The System Pack `…`.") so an install line needs no doc line beside it. `N` in
    `[Capability; N]` is the length of the stringified variant list, so no counting macro exists.
- [x] Validation:
  - [x] `cargo check -p mineworld-sdk`, `cargo clippy -p mineworld-sdk --all-targets -- -D warnings`:
    clean (E-A2).
  - [x] Unit (`pack.rs`): `a_pack_that_owns_no_section_refuses_to_decode_one_and_says_which_pack_it_is`
    — a stub pack refuses with exactly "the 'silent' system owns no section", the message the
    catalog's last arm gave. Driven through serde's own `MapDeserializer`, so no dependency was added.
    1 passed.
- [x] Review: the SDK's manifest names authoring, contracts, kernel and serde only — no pack, and no
  perception trait (the installed set passes it in by path). `#![forbid(unsafe_code)]`. Both
  `#[macro_export]` macros refer only to `$crate::…` and `::core`/`::std` paths. Defaults: no
  biography, no section, and `decode_section` refuses — the safe direction; a pack that sets `SECTION`
  by hand without `owns_section!` still refuses to decode, never decodes wrongly.

**Acceptance.** The SDK builds and lints clean with no pack depending on it; its one test passes.
**Failure cases.** A macro that cannot express `decode_section` generically over `MapAccess` would
invalidate SD-2: stop and report (material — it reopens §2.2's choice).

### C3 — Every existing pack declares itself

**Goal.** What `catalog.rs` says about each pack is said by the pack.

**Scope (each pack: its `Cargo.toml` gains `mineworld-sdk = { workspace = true }`; its system struct
gains `#[derive(Default)]` (and keeps whatever it derives); one `impl SystemPack`):**

```text
presence        impl SystemPack for PresenceSystem {}
movement        impl SystemPack for MovementSystem {}
conversation    impl SystemPack for ConversationSystem {}
group-activity  impl SystemPack for GroupActivitySystem { const BIOGRAPHICAL = BIOGRAPHICAL; }
relationships   impl SystemPack for RelationshipsSystem { const BIOGRAPHICAL = BIOGRAPHICAL; }
naming          impl SystemPack for NamingSystem { const BIOGRAPHICAL = BIOGRAPHICAL; owns_section!(); }
schedule        impl SystemPack for ScheduleSystem { const BIOGRAPHICAL = BIOGRAPHICAL; owns_section!(); }
```

Each impl sits beside the pack's `impl System` (`systems/<pack>/src/system.rs`), or in `section.rs`
beside `impl AuthoredSection` for the two section owners — whichever the pack's own layout makes
obvious; the choice is recorded.

**Depends on:** C2. **Non-goals:** no behaviour change in any pack; `worldpack` still uses its own
enum, so nothing reads these impls yet.

- [x] Implementation: seven impls, seven manifests. Every impl sits in `systems/<pack>/src/system.rs`,
  directly after `impl SystemIdentity` (recorded choice: one place per pack, beside the identity it
  extends and the `BIOGRAPHICAL` constant it names; the two section owners' `owns_section!()` reads its
  key from the `AuthoredSection` impl in `section.rs`, so nothing is stated twice). Each struct gains
  `#[derive(Default)]` (none derived anything before). Each manifest gains `mineworld-sdk`.
- [x] Validation: clippy `-D warnings` over the seven packs, all targets: clean. `cargo test -p` each:
  presence 13, movement 11, conversation 14, group-activity 11, relationships 6, naming 5, schedule 8
  (68 passed, 0 failed). Counts unchanged by construction: the C3 diff adds and removes no `#[test]`
  (`git diff -U0 -- systems | grep -c '#\[test\]'` → 0). `owns_section!` compiles in naming and
  schedule, the macro's first real use.
- [x] Review: side by side with `catalog.rs:120–131` (`section`) and `:169–177` (`biographical`):
  presence, movement, conversation → `&[]` and no section (trait defaults); group-activity,
  relationships → their `BIOGRAPHICAL`, no section; naming, schedule → their `BIOGRAPHICAL` and
  `SectionOwner::of::<Self>()`. Identical to the arms (§9 E-A3). No behaviour changed in any pack:
  the impls are read by nothing until C4.

### C4 — The installed set; `worldpack` names no pack it does not read; the root manifest stops registering

**Goal.** A–1, A–2, A–3. After this commit, installing a pack edits only `systems/`.

**Scope.**
- `systems/installed/Cargo.toml` — crate `mineworld-installed-systems`; dependencies: `mineworld-sdk`,
  and the seven packs (`workspace = true`, as they are existing workspace dependencies).
- `systems/installed/src/lib.rs` — crate documentation (this is the build's installed set; how to
  install; why it lives in `systems/`, ARC-33), and:
  `mineworld_sdk::installed! { perception: mineworld_presence::PerceptionProvider; Presence =>
  mineworld_presence::PresenceSystem, Movement => …, Conversation => …, GroupActivity => …,
  Relationships => …, Naming => …, Schedule => … }` — the order of today's `AVAILABLE`, which is the
  order refusals list systems in.
- `systems/installed/tests/installed.rs` — the consistency guard (A-3): parses its own `Cargo.toml`
  (`env!("CARGO_MANIFEST_DIR")`, `[dependencies]` table, `mineworld-*` keys minus `mineworld-sdk`) and
  compares with the crate prefixes of `Capability::type_name` over `AVAILABLE`; asserts no two
  capabilities share an `id()`. Hand-parsed line by line (a `[dependencies]` table of `name = { … }`
  lines), so no TOML dependency is added; a line it cannot parse fails the test, never skips.
- `worldpack/Cargo.toml` — remove `mineworld-conversation`, `-group-activity`, `-naming`,
  `-relationships`, `-schedule`; add `mineworld-sdk`, `mineworld-installed-systems`; keep `presence`
  and `movement` with a comment naming the format fields they own.
- `worldpack/src/catalog.rs` — delete `Capability`, `AVAILABLE`, `SectionOwner` and the `impl`s
  (`:46–204`); add `pub use mineworld_installed_systems::{AVAILABLE, Capability};` and `pub use
  mineworld_sdk::SectionOwner;`; keep `LOCATION_OWNER`, `PASSAGE_OWNER`, `opened`, `located` and the
  section-namespace guard test; rewrite the module documentation (what moved where, ARC-33).
- `worldpack/src/{content,format,read,load}.rs` — imports only, if any path changes (`decode_section`
  becomes `pub`).
- `worldpack/src/lib.rs` — the layout diagram's `catalog` line.
- `worldpack/tests/structure.rs` (new) — the allow-list guard (A-2): every `mineworld-*` key in
  worldpack's `[dependencies]` and every `mineworld_*` crate named in `src/**/*.rs` is one of
  `{contracts, kernel, authoring, sdk, installed_systems, presence, movement}`. The allow-list names
  infrastructure, never a pack beyond the two format owners, so adding a pack never edits it.
- `worldpack/tests/refusals.rs:171,178` — F-10: `economy` → `no-such-system`; the claim (an unknown
  system is refused by name and the available ones are listed) unchanged.
- Root `Cargo.toml` — `members`: the seven `"systems/<name>"` lines become `"systems/*"`;
  `[workspace.dependencies]` gains `mineworld-installed-systems = { path = "systems/installed" }`.
  The seven existing pack entries stay (existing packs and tests use them).

**Depends on:** C3.

- [x] Implementation: as scoped. `systems/installed/{Cargo.toml, README.md, src/lib.rs,
  tests/installed.rs}`; `worldpack/{Cargo.toml, src/catalog.rs, src/lib.rs, tests/structure.rs,
  tests/refusals.rs}`; root `Cargo.toml` (`"systems/*"`, `mineworld-installed-systems`).
  `content.rs`, `format.rs`, `read.rs`, `load.rs` needed no edit at all (the generated enum has the
  same name, variants and methods).
  - **Deviation D-A1 (bounded).** *Design:* `worldpack` loses five pack dependencies. *Source
    evidence:* `worldpack/tests/social_cafe.rs:23,25,263` reads conversation's, naming's and schedule's
    types to check the loaded world. *Resolution:* those three become `[dev-dependencies]` with a
    comment; `[dependencies]` is exactly as designed. *Impact:* none on A-2, which by design checks
    `[dependencies]` and `src/` only — a test may name the packs whose state it checks. *Validation:*
    `structure.rs` passes; mutation 2 below.
  - **Deviation D-A2 (bounded).** *Stale comment outside worldpack:* `tools/cli/src/biography.rs:14–15`
    said "a new pack adds a constant there [the catalog]", false after this commit. Comment-only edit
    naming `SystemPack::BIOGRAPHICAL` (`CLAUDE.md` §2.1(4)); no code or API outside `worldpack`
    changed.
  - **Detail.** The A-3 negative control is permanent rather than a one-off mutation:
    `the_id_check_sees_two_packs_that_share_an_id` lists two stub packs under one id with the real
    `installed!` macro and asserts the check reports it. Its dev-dependencies are contracts and kernel.
  - `systems/installed/README.md` added (a human orientation, `CLAUDE.md` §2.1).
- [x] Validation (E-A4):
  - [x] `cargo fmt --all`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
    clean. `cargo test --workspace --no-fail-fast`: **425 passed, 0 failed, 0 ignored**, 174 s wall —
    the base's 419 (step-09 E-C-final) plus the six new tests (sdk 1, installed 3, structure 2); none
    removed.
  - [x] A-1: 300-day seed-7 run, 339 lines, sha-256 of all but `wall` =
    `ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b` = E-0. wall 12.2 s.
  - [x] `mineworld validate worlds/social-cafe` byte-identical to the base's output (diff empty).
  - [x] Mutations, each reverted (`git status` afterwards shows only the intended edits; targeted
    suites re-run: installed + worldpack 55 passed, validate identical again):
    - M1 remove `Schedule => …` from `installed!` → `every_pack_depended_on_is_listed_…` FAILS:
      "linked into the build but missing from the installed! list, so never installable:
      [\"mineworld_schedule\"]". `social_cafe.rs` does not compile (it names `Capability::Schedule`)
      rather than failing with `UnknownSystem` as the design predicted — a stronger failure, recorded
      as observed. The loader's refusal shown through the real CLI instead: `mineworld validate
      worlds/social-cafe` → "world.yaml enables the system 'schedule', which this build does not
      provide (it has: 'presence', … 'naming')", exit 1.
    - M2 `use mineworld_naming as _;` in `catalog.rs` + the dependency back → both `structure.rs` tests
      FAIL, naming `mineworld_naming` (the manifest one, and the source one with its file).
    - M3 blind `listed_twice` (`&& false`) → `the_id_check_sees_two_packs_that_share_an_id` FAILS
      (left `{}`, right `{SystemId("twin")}`). The real-list test would have stayed green under M3,
      which is why the negative control exists.
- [x] Review:
  - `worldpack`'s public API is unchanged: `git diff` of `worldpack/src/lib.rs` touches only the two
    diagrams (no `pub use` line changed); `catalog` still exports `Capability`, `AVAILABLE`,
    `SectionOwner`, `LOCATION_OWNER`, `PASSAGE_OWNER`, `opened`, `located`. Callers outside worldpack
    (`tools/cli/src/biography.rs`, tests) unchanged except D-A2's comment.
  - The section decoder still reads from the stream: `refusals.rs`'s line-and-column assertions pass
    unchanged (worldpack 32 + 15 + … all green).
  - The only existing-test edit is F-10's (`refusals.rs`: `economy` → `no-such-system`, with a comment);
    claim unchanged — an unknown system is refused by name and the available ones are listed.
  - `systems/installed/` holds the list, its manifest and its guard; no logic.

**Failure cases.** An existing test that fails for a reason other than F-10 means the expansion is not
equivalent to the old catalog: fix the macro, never the test. A changed run fingerprint is a stop:
something reordered installation or reduction (`MODULE_SPEC.md` §4.1 rule 2).

### C4b — The I-2 scan (freeze condition 1; added at freeze, after this plan was drafted)

**Goal.** I-2 is checked mechanically: a test reads every line 11a (and later 11b, 11c) adds and
refuses a market word, with a reasoned allow-list, shown to fail on a planted violation, fail-closed.
Specified in `ARC-35` point 7 (C1).

**Placement decision.** `tests/acceptance/` (crate `mineworld-acceptance`, test
`tests/precursor_vocabulary.rs`), created now rather than in 11f: `ARCHITECTURE.md` §14 gives `tests/`
to acceptance tests, SD-15 already names this crate as the AC-1 proof's home, and the I-2 scan is part
of how AC-1 is measured (`ARC-35`). A precursor-owned crate (sdk, worldpack) would be the wrong owner,
and `systems/installed` is edited by 11d/11e. 11f adds the AC-1 test beside it; 11b and 11c add their
rows and allow-list entries to this file. Bounded deviation **D-A3**: root `members` gains
`"tests/acceptance"` in 11a; `tests/.gitkeep` removed.

**How "this PR's added lines" are determined (recorded, deterministic).** One row per precursor: PR,
base commit, branch. If `HEAD`'s first-parent history holds that branch's GitHub merge commit `M`
(subject `Merge pull request #N from <owner>/<branch>`), the range is `base..M^2` — the PR's own head,
so later PRs (11d, 11e) never enter it. Otherwise the range is `base` → working tree: `git diff
--unified=0 --no-renames <base>` plus every untracked, unignored file (whole content and path). All
non-Markdown files are scanned, comments included. Fail closed: no git, not a work tree, base missing,
or `HEAD` not descending from base → failure naming the cause.

- [x] Implementation: `tests/acceptance/{Cargo.toml, src/lib.rs, tests/precursor_vocabulary.rs}`; root
  `Cargo.toml` member. No dependency (std only; `git` is run as a process). Allow-list for 11a: one
  entry — the scan's own file ("it names the vocabulary it looks for"). Two permanent unit tests own
  the tokenizer (`items`, `ShopFront`, `employer`, `JOB_BOARD`, a fixture path match; `iterate`,
  `workshop`, `priority`, `SystemId` do not) and the diff reader (line numbers, a `+++`-prefixed added
  line, a new file's path).
- [x] Validation (E-A4b):
  - **First real run FAILED, correctly**, on five comment lines this PR had added: "`ARC-31` item 5" (×4)
    and "ARC-35 item 7" — the word *item* in the sense "list entry". Resolution: reworded to "point 5"
    / "point 7" rather than allow-listed (an allow-list entry is for what cannot be reworded). The scan
    was thereby shown to see this PR's own lines before any plant.
  - Then green: 3 passed.
  - **Planted violations** (working tree, never committed): a comment
    `// PLANTED: the ShopKeeper reads a price list.` in `sdk/rust/src/section.rs`, and an untracked
    `worlds/plant-check/jobs.yaml` holding `wage: 3` → FAILED with exactly three refusals:
    `sdk/rust/src/section.rs:5: \`shop\``, "the added file worlds/plant-check/jobs.yaml is named
    \`jobs\`", `worlds/plant-check/jobs.yaml:2: \`wage\``. Reverted (file restored from a copy, the
    fixture deleted; `git status` clean of both).
  - **Fail closed:** base set to a sha not in history → FAILED: "11a: its base 0123… is not in this
    repository's history (a shallow clone?): `git cat-file -e …` failed". Reverted.
  - The merged-range path is exercised on a local scratch merge (E-A5b, C5), never pushed.
  - clippy `-D warnings` and fmt clean.
- [x] Review: the scan never skips; Markdown is the only exclusion and is stated; the allow-list has one
  entry with its reason, and an unused entry fails; nothing in 11a needed a market word, so no material
  stop arose.

### C5 — The canary install (evidence, never merged); documentation and ledger close

**Goal.** A-4: show, before any market pack exists, that installing a pack touches only what ARC-33
says it does (`ARC-23`: an instrument shown to see before it is trusted).

- [x] Implementation (local scratch branch `scratch/11a-canary` from `1f05232` — C4b's head, the
  executable head after C4 plus the scan; deleted afterwards, never pushed):
  - [x] `systems/canary/`: `Wave` (no target), `Waved`, an offer through `PerceptionProvider` with
    `SpatialRequirement::NONE`, `impl SystemPack for CanarySystem {}`; sibling dependency by path
    (`mineworld-presence = { path = "../presence" }`), infrastructure by `workspace = true` (already in
    the root, so no root edit). Compiled on the first build.
  - [x] Two lines in `systems/installed/`.
  - [x] `worlds/canary-cafe/`: a copy of `worlds/social-cafe`, `id: canary-cafe`, `canary` appended.
- [x] Validation (§9 E-A5; the branch then deleted):
  - [x] `git diff --name-only 1f05232 HEAD` → `Cargo.lock`, `systems/canary/{Cargo.toml,src/lib.rs}`,
    `systems/installed/{Cargo.toml,src/lib.rs}`, `worlds/canary-cafe/**` (25 files, 20 of them the
    world copy). Nothing else.
  - [x] `Cargo.lock`: one new `[[package]] mineworld-canary` with no `source`; one line added to
    `mineworld-installed-systems`'s dependencies; nothing else.
  - [x] `mineworld validate worlds/canary-cafe` → "systems presence, …, schedule, canary", valid;
    `mineworld run worlds/canary-cafe --headless --seed 7 --days 1` → exit 0, faults 0, 1 368 facts. No
    `wave` was attempted: the paced controller knows no action it was not compiled against (F-3, 11c's
    to resolve) — expected, recorded, not a defect of 11a.
  - [x] installed + worldpack suites on the scratch branch: 55 passed, 0 failed — the same as without
    the canary (installing a pack breaks no loader test; F-10's property). social-cafe's 300-day sha on
    the scratch build = E-0 (installing a pack a world does not enable changes nothing).
  - [x] The I-2 scan on the scratch branch FAILED, naming 12 lines of `worlds/canary-cafe/` (`shop`,
    `shopping`, `items`, `job`, `employment` in social-cafe's own comments and tags, copied). Correct:
    on that branch they are added lines. Recorded because 11b/11c must not copy social-cafe content
    without allow-list entries for "pre-existing use carried".
  - [x] **E-A5b, the scan's merged-range path:** scratch `scratch/11a-merged` from `b53e19d`, `git merge
    --no-ff 1f05232 -m "Merge pull request #999 from yuema137/mvp0/pr-11a-installable"`, then a commit
    adding `worlds/later/shop.yaml` (`price: 3`) → scan PASSES (range `base..M^2` excludes the later
    commit). Control `scratch/11a-merged-control`: same, merge subject "Merge branch 'something-else'"
    → FAILS naming `worlds/later/shop.yaml` (path and `price`). Both branches deleted.
  - [x] `git ls-remote --heads origin | grep -c scratch` → 0.
- [x] Documentation: `worldpack/README.md` (third bullet: names no pack but two),
  `docs/MVP_STATUS.md` (a capability row, an evidence row, S9's stage line), the handoff.
- [ ] Full gates once on the final executable head (§6), recorded with counts and wall time (§9
  E-A-final).
- [ ] Review: A-1 … A-4 each hold with recorded evidence (§9); the scratch branches were never pushed.

**Commit boundary.** Only documentation and the ledger are committed in C5; the canary lives only in
the ledger's evidence.

### 4.1.2 Test ownership for 11a

```text
STATIC      cargo fmt, cargo clippy -D warnings: formatting, unused imports, the trait bounds (a listed
            pack that is not a PerceptionProvider or not Default does not compile)
UNIT        sdk: the fail-closed default of decode_section (C2)
            installed: manifest ↔ list consistency; distinct ids (C4)
            worldpack: the dependency allow-list (C4); the section-namespace guard (moved unchanged)
INTEGRATION every existing test, unchanged (A-1) — the loader, the CLI, persistence and the server all
            run through the generated catalog
REAL RUN    the 300-day social-cafe run, byte-compared with E-0 (A-1); the canary install (A-4)
GATE 1      NOT REQUIRED — no model anywhere in S9
GATE 2      the real run and the canary install above are this PR's real-lifecycle evidence
CI          none configured (S13); the local full gate runs once on the final head
```

### 4.1.3 Is any of this material?

Yes, and it is raised rather than assumed:
- **ARC-33 / SD-2** change how a System Pack becomes part of a build — a public contract for pack
  authors (`SystemPack`) and the meaning of "independently installable" for `AC-1` (QS-3).
- **ARC-35 / SD-6** decide how the frozen top-level criterion is measured (QS-2).

Nothing in 11a changes a frozen invariant of an earlier step, `kernel/`, `contracts/`, or the behaviour
of any world.

## 4.2 PR 11b — items and organizations in the World Pack format (medium scope; detailed after 11a merges)

**Goal.** A World Pack can declare Item and Organization entities and give them sections, as
`MODULE_SPEC.md` §4's frozen layout already says it can (SD-7, F-4, F-13).

```text
authoring/src/section.rs   ContentKind::{Item, Organization}: directory (`items`, `organizations`),
                           describes, entity_type; Seeding gains typed lookups item(key) and
                           organization(key) beside place(key) and person(key)
worldpack/src/format.rs    WorldManifest: `items: Vec<EntityKey>`, `organizations: Vec<EntityKey>`
                           (both default empty); AuthoredItem, AuthoredOrganization: tags, note, sections
worldpack/src/content.rs   ITEM_FIELDS, ORGANIZATION_FIELDS = ["tags", "note"]; ContentFile::{item,
                           organization}
worldpack/src/read.rs      read items/<key>.yaml and organizations/<key>.yaml; keys stay one namespace
                           across all four kinds (check_keys_are_declared_once gains two Declared
                           variants); a section's references may name items and organizations
worldpack/src/load.rs      ids: places, people (unchanged), items, organizations, each in key order;
                           genesis: passages, locations, then sections — items', organizations',
                           places', people's
worldpack/tests/           refusals (an item file with an unknown key; a `name` section on an item file
                           refused NotCarriedHere; an item key that is also a place key; a missing
                           items/<key>.yaml), loading (a tags-only pack with items and organizations:
                           ids after people, entity types right), social-cafe unchanged
docs                       MODULE_SPEC §4.1 (the two keys and files, the order rule), PACKAGE_FORMAT §8
                           row, DECISIONS ARC-36 (an authored Item is a kind — SD-8, if QS-6 is accepted)
```

**Checkpoint.** A pack with items and organizations validates and loads; every refusal names the file;
`social-cafe`'s ids, genesis count (53) and 300-day run are unchanged (I-4).
**Adversarial.** A pack whose people's sections were seeded before an item's would let a holding name an
undeclared kind: the genesis-order test fails if items' sections are moved after people's (mutation).
**Non-goals.** No section is carried by items or organizations yet — the first owners arrive in 11d and
11e; no change to `naming` (names for items and organizations are a later choice).
**Material?** A public format extension, implementing a frozen model (QS-5). No contract or kernel change.

## 4.3 PR 11c — complete affordances (medium scope; detailed after 11a merges)

**Goal.** A requester can submit an action it does not know, exactly as the offering system offered it
(SD-9 … SD-12, ARC-34).

```text
contracts/src/observation.rs   Affordance<P = Vec<u8>> gains `payload: Option<P>` (serde: default,
                               skip_serializing_if none — so every existing observation serializes
                               byte-identically); Affordance::with_payload(P); payload(); request(actor)
                               → Option<ActionRequest<P>> built from the affordance's own action type and
                               target. Observation<P>'s affordances become Vec<Affordance<P>>.
                               AffordanceFields gains the field; the availability-agreement check stays
contracts/tests/               round trip with and without a payload; an old frame (no field) still
                               decodes; request() carries the affordance's type, target and payload
systems/presence/src/          Offer gains `payload: Option<Value>`; Offer::with_payload::<A: Action +
  interaction.rs, observe.rs   Serialize>(self, &A) — refuses (debug-asserts) an A other than the offer's
                               own type; verdict() carries it into the Affordance
cognition/rule-controller/     src/offered.rs: attempt(observation, draw) — the available complete
                               affordances, in observation order; draw 14 < ATTEMPTS_OFFERED (proposed 20)
                               → pick by draw 15 → Affordance::request; decide() calls it after the social
                               initiative, before the walking roll. Payload Value → bytes with serde_json,
                               as the controller already encodes
cognition/rule-controller/     a synthetic test pack (in the test module, never a real pack): one action
  tests                        `ring-bell { bell }`, offered complete once per bell; the paced controller,
                               which has never been compiled against it, rings bells in a hand-built
                               world through presence's real observe() and the kernel's real dispatch
server/PROTOCOL.md             the field, its meaning, and that a client may submit it unchanged
clients/protocol/mineworld/    observation.gd: payload(action_type, target); ADOPTION.md
docs                           DECISIONS ARC-34; CORE_CONCEPTS §15.2 (the Affordance row and one
                               paragraph); MODULE_SPEC §5 (a controller may attempt complete affordances;
                               it still cannot create an interaction, INV-10)
```

**Checkpoint.** In a hand-built world with presence and the synthetic pack, a seeded paced controller
attempts the synthetic action and it is Accepted, its fact caused by that request (`AC-9`); with the
synthetic pack disabled, no such request is ever made (`INV-10`); the 300-day social-cafe run is
byte-identical to E-0 (I-4); the recorded AC-13/AC-15 transcripts are unchanged (no payload anywhere).
**Adversarial (decided now).** (1) Mutation: the band ignores `is_available()` → a test that offers an
unavailable complete affordance fails. (2) Mutation: draw index 14 reused by another band → the
social-cafe byte comparison fails. (3) A payload attached for the wrong action type cannot be
constructed (typed `with_payload::<A>`); shown by a test that the affordance's type is the offer's.
**I-9.** `ATTEMPTS_OFFERED` and the band's position are fixed here, against the synthetic pack, and are
not changed by any later S9 PR.
**Material?** Yes: `contracts/` public contract and the controller contract (QS-4).

## 4.4 PR 11d — the transformation, part 1: owning and giving things (medium scope; AC-1 range)

**Allowed paths only** (I-1, §2.5). Detailed after 11b and 11c merge, beginning with a throwaway spike
of inventory's checked constructor being used by another pack on the post-11c `main` (R-S9-1).

```text
systems/item/            ItemKind { category: slug } on Item entities, from the `item:` section of
                         items/*.yaml (Authored: { category }); genesis fact item-kind-declared; no
                         action; nothing disclosed in S9 (items are not perceived)
systems/inventory/       depends on item. Holdings { counts: BTreeMap<ItemId, u32> } on Persons and
                         Organizations, from the `holdings:` section of people and organization files
                         ({ <item key>: <count> }, references = Items); vocabulary, born final so 11e
                         need not edit it: stocked (genesis), items-transferred { from, to, item,
                         count }, items-produced { holder, item, count } — each with a checked public
                         constructor that refuses what Holdings may not take (a transfer the giver
                         cannot cover, a kind item has not declared). Discloses Holdings to its holder
                         only (INV-13). BIOGRAPHICAL: none or items-transferred (decided at detail)
systems/item-transfer/   depends on inventory. `give { item, count }` targeting a Person in the same
                         place within 3 000 mm; one complete affordance per kind the giver holds
                         (count 1); validate → resolve states inventory's items-transferred (ARC-26).
                         Owns no state. AC-2: disabled → give is Unavailable, the world runs, nothing
                         else changes (systems/item-transfer/tests)
systems/installed/       three lines (Cargo.toml + installed!)
worlds/market-town/      social-cafe's files verbatim (world.yaml: id, name, and `item, inventory,
                         item-transfer` appended to systems); items/*.yaml (the MVP's ~20 kinds:
                         coffee, tea, croissant, …), `holdings:` on people; README.md
```

**Checkpoint (inside the range).** Each pack's own tests over hand-built worlds (the movement/
group-activity pattern), including a persisted restart for inventory; `mineworld validate
worlds/market-town`; `mineworld run worlds/market-town --headless --seed 7 --days 30` with `give`
accepted at least once per seat-bucket — recorded as ledger evidence (the world-level test lands in 11f,
outside the range). **Adversarial.** A transfer of more than the giver holds is refused by inventory's
constructor even if item-transfer's validate were skipped (owner still decides, ARC-26): a test states
the fact directly and expects `FactRefusedByOwner`.

## 4.5 PR 11e — the transformation, part 2: work, money and shops (medium scope; AC-1 range)

**Allowed paths only.** Detailed after 11d merges, beginning with a throwaway spike of employment's
shift Process reading presence and stating inventory's `items-produced` (R-S9-1).

```text
systems/economy/         depends on inventory; Cargo-depends on employment for `wage-due`'s type only
                         (ARC-28, no system dependency). Wallet { balance: u64 minor units } on Persons
                         and Organizations ← `wallet:` (people, organizations); Shop { operator:
                         OrganizationId, prices: BTreeMap<ItemId, u64> } on Places ← `shop:` (places).
                         `buy { item }`: no target; SamePlaceAsActor at a shop place; one complete
                         affordance per priced kind, available when the operator holds one and the buyer
                         can pay, otherwise unavailable with the reason; resolve states its own
                         money-transferred and inventory's items-transferred. Reacts to wage-due:
                         money-transferred (caused by it) or wage-unpaid. Discloses a Wallet to its
                         holder; a shop's listing (prices, in stock) to everyone perceiving the place
                         (CP-7). Facts: funded (genesis), money-transferred, wage-unpaid
systems/employment/      depends on inventory and presence. `job:` section on people: { employer: <org
                         key>, role: slug, workplace: <place key>, shift: "HH:MM"–"HH:MM", wage per
                         hour, produces: optional { item, per hour } }. Owns Employment + the
                         employed-by edge (Person → Organization) + one `shift` Process per job, woken at
                         the shift's start and end (the time-of-day convention of ARC-32). Attendance:
                         Presence at a wake, and person-entered-place between wakes. Facts: hired
                         (genesis), shift-started, shift-ended { worked seconds }, wage-due { employee,
                         employer, amount }; states inventory's items-produced for the employer.
                         Never reads or writes a Wallet (CP-5)
systems/installed/       two lines
worlds/market-town/      organizations/ (the café's and the store's operators — keys distinct from the
                         places', F-16), wallets, shops on `cafe` and `store`, two jobs fitted to
                         existing routines (alice at the café 05:30–14:00; a store clerk inside a routine
                         that already spends that time at the store, F-17), organization holdings
```

**Checkpoint.** Per-pack tests including a persisted restart mid-shift (the schedule pattern);
`economy` with employment disabled installs and still sells (no system dependency); employment with
economy disabled states `wage-due` that nobody reduces, and nothing breaks (AC-2 direction); a 30-day
market-town run shows purchases, wages and production — ledger evidence.
**Adversarial.** A test that states `wage-due` for an employer with an empty Wallet gets `wage-unpaid`,
never a negative balance (`u64` makes the negative unrepresentable; the test shows the refusal path is
the one taken). A mutation that lets employment write a Wallet does not compile (`INV-7` by the kernel's
write tokens) — recorded as a static guarantee, not a test.

## 4.6 PR 11f — the proof (medium scope; detailed after 11e merges)

```text
tests/acceptance/        new workspace member `mineworld-acceptance` (root Cargo.toml: one member line —
                         outside the range, by design). tests/ac1_composability.rs:
                           TRANSFORMATION = [<11d merge sha>, <11e merge sha>]
                           check 1  the change set (git diff --name-only M^1 M per merge; Cargo.lock rule)
                           check 2  the structure (cargo metadata; crate-name scan of code files)
                           check 3  the world delta (both packs read with WorldPack::read, compared
                                    field by field and section by section, sections compared by their
                                    authored YAML values)
                         each check fails closed: missing git history, a missing cargo, an unreadable
                         pack are failures that say so, never skips
tools/cli/tests/         market_town.rs: AC-11 300 days with CP-4's precondition per 30-day bucket
                         (located before counted: purchases, wages paid, items produced, items given),
                         then AC-12 by bytes; a SIGKILL-and-resume run (the run_restart pattern); AC-2 at
                         world level: market-town written to a temporary directory without item-transfer
                         runs 30 days, no give is accepted, purchases and wages still occur;
                         milestone_c.rs: the server hosts market-town with --save; one client in the
                         store submits a buy from its complete affordance; a second client in the store
                         perceives the listing change; the server is killed and restarted, and both the
                         buyer's Holdings and the listing are as they were
docs                     MVP_STATUS (AC-1, AC-2 rows), HUMAN_REVIEW_QUEUE (Milestone C, with the launch
                         commands), worlds/market-town/README, DECISIONS ARC-35 note (the merge ids and
                         the evidence), overall §7
```

**Checkpoint.** CP-1, CP-4, CP-6, CP-7 green on the final head. **Adversarial (decided now; each shown
to bite, then reverted, on a scratch branch):**
- check 1: a scratch "transformation" commit that also touches `kernel/src/lib.rs` (a comment) → fails
  naming the path; one that adds an external crate to a market pack → fails on the `Cargo.lock` rule;
- check 2: `mineworld-economy` added as a dependency of `cognition/rule-controller` → fails naming the
  path from the controller to the pack;
- check 3: one tag of one person changed in market-town → fails naming the file and the field;
- CP-4: market-town with `buy` never offered complete (economy edited on the scratch branch) → the
  precondition fails before any determinism comparison runs (the instrument sees the absence).

---

# 5. Integration checkpoints

| PR | Integration checkpoint | Adversarial criterion (decided before measuring, `ARC-23`) |
| --- | --- | --- |
| 11a | social-cafe's 300-day run byte-identical to E-0; a canary pack installed by two lines in `systems/installed/` (A-1 … A-4) | removing a list line, or re-adding a pack import to `worldpack`, fails a named guard |
| 11b | a pack with items and organizations loads; social-cafe's ids, genesis count and run unchanged | items' sections seeded after people's fails the genesis-order test |
| 11c | a synthetic pack's action, unknown to the controller, is attempted and accepted headless; social-cafe byte-identical | the band taking an unavailable affordance, or sharing a draw index, fails a named test or the byte comparison |
| 11d | market-town (owning, giving) validates and runs 30 days with gives accepted; per-pack tests incl. restart; AC-2 for item-transfer at pack level | a transfer beyond what the giver holds is refused by inventory itself |
| 11e | wages, purchases and production in a 30-day run; restart mid-shift; economy and employment each install without the other | wage-due against an empty wallet yields wage-unpaid, never a negative |
| 11f | the AC-1 test (three checks), CP-4 over 300 days, AC-2 at world level, Milestone C through the real server | each of the four scratch mutations in §4.6 fails its check by name |

# 6. Test ownership and verification

```text
STATIC        cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings: formatting,
              lints, trait bounds of the installed set, INV-7 write tokens (a pack writing another's
              component does not compile)
UNIT          fail-closed registry and guard logic (11a); format refusals (11b); the offer band's
              choice and its availability filter (11c); each market pack's validate/resolve/reduce over
              hand-built worlds (11d, 11e)
INTEGRATION   every existing test, unchanged except F-10 (11a–11c); loader over real packs (11b);
              presence's real observe() + kernel dispatch + paced controller (11c); per-pack persisted
              restarts (11d, 11e)
REAL RUNS     mineworld run over social-cafe (every PR, I-4) and market-town (11d, 11e evidence; 11f
              tests); the real server and real sockets for Milestone C (11f)
GATE 1        NOT REQUIRED — no language model in S9
GATE 2        the real runs above; the canary install (11a); the scratch mutations (11f)
CI            none configured (S13). Each PR runs the full local gate once on its final head:
              cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings &&
              cargo test --workspace; plus python3 scripts/check_doc_headings.py and
              python3 scripts/check_decision_ids.py
```

The 300-day run takes ~13 s at `opt-level = 1` (E-0); the full gate on `b9e5937` is ~4–6 minutes
(step-09 E-final). No PR in S9 needs a run longer than a few minutes.

# 7. Self-review against the frozen specifications

```text
CHECKED  frozen top-level criterion / AC-1: measured by three independent checks over a named change
         set (§2.5); F-1 and F-3 resolved by precursors bounded by I-2 and I-9 rather than by
         reinterpreting the criterion; the reinterpretation that remains (measurement, Cargo.lock,
         docs, systems/installed) is raised as operator-material, not assumed
CHECKED  INV-12: no S9 PR teaches the kernel anything; kernel/ untouched (I-8); money, jobs, shops,
         items live in packs; time of day stays schedule's and employment's convention
CHECKED  INV-7 / CLAUDE.md §4 rule 1: one writer per state (§2.6); money moves only in economy;
         employment emits wage-due; holdings change only in inventory's reductions
CHECKED  ARC-26: item-transfer, economy and employment state inventory's facts only through inventory's
         checked constructors and only with a system dependency on it
CHECKED  ARC-28: economy reacts to wage-due by Cargo dependency on employment's types, no system
         dependency; neither depends on the other at the registry
CHECKED  INV-1 / INV-6 / ARC-32: no pack decides for a person; work is being where your job is; buying is
         a request a controller or a human makes
CHECKED  INV-10: a complete affordance is offered only for an action an enabled system provides, and
         dispatch revalidates (ARCHITECTURE §9); a controller still cannot create an interaction
CHECKED  INV-13: holdings and wallets disclosed to their holder only; a shop's listing to those who
         perceive the shop
CHECKED  ARC-27 / I-5: the paced controller stays a pure function; RuleController unchanged
CHECKED  CLAUDE.md §4 rule 5: after 11a–11c, adding the market is modules + registration + tests (§2.7)
CHECKED  CLAUDE.md §4 rule 11: SystemPack is defined after seven packs repeat the same five facts in the
         catalog; complete affordances after a concrete second consumer (controller and clients) exists
CHECKED  CLAUDE.md §4 rule 16 / REUSE_POLICY §§11–12, 17: inventory/linkme/libloading/erased-serde/a
         TOML build script considered and declined with reasons (DEP-12); no dependency added in S9
CHECKED  ENGINEERING_RULES §§9, 11–12 (the two gate questions): complete affordances are a server
         answer both clients can carry and neither evaluates; a shop is a place a 3D player walks into
         and a 2D player clicks into; nothing engine-specific enters a contract
CHECKED  determinism (AC-12): every new map is a BTreeMap; every choice is a seeded draw; no float
FLAGGED  QS-2, QS-3, QS-4, QS-5, QS-6 — operator-material (§10)
FLAGGED  MVP §5's eat and sleep are not in S9 (QS-10)
```

---

# 8. Source audit (`main @ b9e5937`, 2026-10-07)

## 8.1 What was inspected

```text
Cargo.toml (root: members, [workspace.dependencies], profile); every crate manifest naming a pack
worldpack/src/{catalog.rs (whole), content.rs (whole), format.rs (:1–140), load.rs (:230–330,
  :440–488), read.rs (:205–240, LOCATION_OWNER/PASSAGE_OWNER uses), lib.rs}; worldpack/Cargo.toml;
  worldpack/tests/refusals.rs (:1–40, :165–200, :779), social_cafe.rs (composition list)
authoring/src/{lib.rs, section.rs}
cognition/rule-controller/{Cargo.toml, src/lib.rs, paced.rs, social.rs, agenda.rs} (whole)
contracts/src/observation.rs (whole: Affordance, Observation); contracts/src/action.rs (ActionRecord,
  ActionRequest, Rejection); contracts/src/ids.rs (EntityType::Item/Organization, ItemId,
  OrganizationId); contracts/Cargo.toml
kernel/src/view.rs (public API: no entity creation by a system); kernel/src/world.rs:571
  (create_entity); kernel/src/system.rs (public API outline)
systems/presence/src/{interaction.rs (:1–260), observe.rs (:150–290)}; systems/*/src/system.rs (the
  seven system types; BIOGRAPHICAL constants); systems/naming/src/section.rs; systems/README.md
tools/cli/{Cargo.toml, src/run.rs (:1–80), src/biography.rs (:160–185), tests/commands.rs (:20–40)}
server/Cargo.toml, persistence/Cargo.toml
clients/protocol/demo/demo.gd (:255–300, :480–520); clients/protocol/mineworld/{observation.gd,
  world_client.gd} (affordance and submit surfaces)
docs: MVP (whole), ACCEPTANCE (whole), MODULE_SPEC (whole), PACKAGE_FORMAT (whole), CORE_CONCEPTS
  (whole), REUSE_POLICY (whole), ENGINEERING_RULES (whole), ARCHITECTURE §§1–2, 9, 12–14,
  ENGINEERING_STANDARDS §§5–9, 16, 28–30, DECISIONS ARC-26 … ARC-32, HUMAN_REVIEW_QUEUE
  (milestones); overall.md (whole); step-09-social.md §§1–2.7, 4.3.1–4.3.2, 7, 8, 10, 10.1, 11
git grep for market words in code outside systems/ and worlds/ (F-10, F-14); every origin/* branch for
  ARC-33+ / DEP-12+ (none)
experiments (reverted): root members as "systems/*" with cargo metadata (F-9); the 300-day seed-7
  social-cafe baseline run (E-0)
```

## 8.2 Findings

```text
F-1   (carried, step-09) Installing a pack edits ~11 lines in 5 files, 3 outside systems/ (§2.2 list).
      Resolved by 11a's design (SD-1 … SD-5); what remains (one Cargo line, a rebuild) is inherent to
      statically linked Rust and recorded in ARC-33.
F-3   (carried) The paced controller acts only on actions it is compiled against, because an
      Affordance carries no payload (contracts/src/observation.rs:172). Resolved by 11c's design.
F-4   (carried) A per-pack World Pack field was the old pattern; ARC-31's sections replaced it for
      people and places. Items and organizations still have no content kind (11b).
F-9   A glob member "systems/*" resolves all workspace members on b9e5937 with systems/README.md
      present (`cargo metadata --no-deps`, 15 packages); the root manifest was restored byte for byte.
F-10  worldpack/tests/refusals.rs:171,178 uses `economy` as the system the build does not provide.
      Installing economy would break it, and its fix lies outside AC-1's allowed set. 11a renames it.
F-11  demo.gd lists every affordance generically (:483–505) but can submit only `talk` (:278); the Godot
      module's submit takes a payload the caller must build (world_client.gd:155–172).
F-12  WorldView has no way for a system to create an entity (kernel/src/view.rs public API); entities
      come from the loader (World::create_entity, kernel/src/world.rs:571). So item *instances* cannot
      be created by a pack without a kernel change: S9 ships stacked kinds (SD-8, QS-6).
F-13  MODULE_SPEC §4's frozen layout lists items/ and organizations/; §4.1's implemented subset omits
      them, and world.yaml refuses the keys as unknown. 11b closes the gap rather than inventing format.
F-14  Market words appear outside systems/ only in contract-layer stubs (`inventory-stub`,
      `item-transferred` in contracts/tests/event.rs; `Employment` in contracts/tests/identity.rs) and in
      the 3D spike's GDScript (`shop` geometry). None names a pack, so AC-1's check 2 matches crate
      names, not slugs.
F-15  tests/ and sdk/ exist only as .gitkeep. ARCHITECTURE §14 gives them to acceptance tests and the
      SDK; S9 uses both for their stated purpose.
F-16  Entity keys are one namespace across kinds (read.rs check_keys_are_declared_once; the kernel
      refuses a second entity claiming a key). An Organization operating the café cannot be keyed
      `cafe`.
F-17  §2.5 check 3 keeps social-cafe's sections verbatim, routines included. Jobs must therefore fit
      routines people already have; this is a content constraint on 11e, chosen over relaxing check 3.
F-18  catalog.rs's own documentation anticipates the change: "ARC-8 makes system extension a WASM
      component later, at which point this becomes a registry populated at startup … The shape of the
      question does not change." SD-2 keeps that shape.
F-19  `mineworld run`'s pace is a CLI constant (tools/cli/src/run.rs:42). If market-town's run were too
      slow, raising it would edit tools/ (outside the range) and change social-cafe's runs (I-4). Run
      cost must be held by what the packs offer (R-S9-2).
```

## 8.3 Material findings

F-1 and F-3, carried from S8, are resolved here only by framework changes that are themselves
material: the installation mechanism and the measurement of `AC-1` (QS-2, QS-3), and a public contract
change in `contracts/` (QS-4). The ontology reading of `Item` (QS-6) and the format extension (QS-5) are
material in their own right. Nothing found contradicts a frozen invariant of an earlier step.

---

# 9. Ledger and evidence

```text
E-0  C0 design, 2026-10-07, on main @ b9e5937 + this file.
     Baseline for I-4 / A-1: `mineworld run worlds/social-cafe --headless --seed 7 --days 300` (debug,
     opt-level 1): 339 lines; history 365 330 facts, fingerprint 59339a9c281829c9; faults 0;
     wall 13.1 s; sha-256 of every line but `wall` =
     ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b.
     F-9 experiment: cargo metadata --no-deps with members "systems/*" → 15 workspace packages; root
     Cargo.toml restored (cmp identical), Cargo.lock untouched (git status clean).
     check_doc_headings: 142 numbered sections across 22 documents, none duplicated.
     check_decision_ids: 42 decision ids, all distinct (ARC-33 … ARC-36 and DEP-12 are proposals in
     this file only; DECISIONS.md is untouched; no origin/* branch holds ARC-33+ or DEP-12+).
     No cargo gate run (docs only). The baseline command was run once to fix A-1's reference, not as
     a gate.

--- PR 11a (branch mvp0/pr-11a-installable, base main @ b53e19d) ---

E-A0 E-0 reproduced on b53e19d before any edit (debug, opt-level 1): 339 lines, sha-256 of all but
     `wall` = ad49c723…c64b, faults 0, fingerprint 59339a9c281829c9, wall 12.5 s.
E-A1 C1 (83850df): check_decision_ids 45 ids distinct; check_doc_headings 143 sections / 22 documents.
     ARC-33 (static-linking boundary), DEP-12, ARC-35 (point 7: the I-2 scan and how "the PR's added
     lines" are determined). Ids free on every origin/* branch after `git fetch`.
E-A2 C2 (816b80c): cargo check/clippy -p mineworld-sdk clean; sdk unit test 1 passed.
E-A3 C3 (8c82c1e): the seven impls vs catalog.rs arms — biographical: presence/movement/conversation
     &[], group-activity/relationships/naming/schedule their BIOGRAPHICAL; section: naming and
     schedule only, via owns_section!. Identical. Pack suites 68 passed (13/11/14/11/6/5/8).
E-A4 C4 (d9c9394): fmt, clippy --workspace --all-targets --all-features -D warnings clean;
     cargo test --workspace --no-fail-fast 425 passed 0 failed (419 base + 6 new), 174 s.
     A-1 sha = E-0 (wall 12.2 s); validate output identical to base. Mutations M1–M3 each fail by
     name and are reverted (§4.1 C4).
E-A4b C4b (1f05232): the I-2 scan — first run caught 'item' in five comments this PR added
     (reworded); planted violations (tracked edit + untracked fixture) fail with three named
     refusals; a missing base fails closed. Reverted.
E-A5 C5 canary (scratch, local, deleted): changed paths ⊆ {systems/canary/**,
     systems/installed/{Cargo.toml,src/lib.rs}, worlds/canary-cafe/**, Cargo.lock}; Cargo.lock +1 path
     package, +1 dependency line; validate lists canary; 1-day run exit 0, faults 0; installed +
     worldpack 55 passed. E-A5b: the scan's merged-range path passes with a later market commit and
     fails without a recognised merge.
```

## 9.1 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-S9-1** | A transformation PR finds it needs a framework change (a kernel API, a presence hook, a contract field). Inside 11d/11e that fails AC-1 by construction. | Each transformation design begins with a throwaway spike of its riskiest cross-pack flow on the post-precursor `main`. A gap found there becomes a precursor PR, justified without the market (I-2), or a material stop. Never a quiet edit inside the range. |
| **R-S9-2** | Complete affordances multiply offers per consult (one per kind held, per person nearby, per priced kind); run time grows, and the pace cannot be raised (F-19). | Measured in 11d and 11e on 30-day runs before 11f. If one 300-day market-town run exceeds ~60 s, the packs offer less (e.g. give only kinds held, to people within reach — already scoped), never a CLI change. |
| **R-S9-3** | The closed money loop drains: customers spend endowments, employers cannot pay wages, CP-4's buckets go empty by month N. | Content in 11e (endowments, prices, wages, production rates) is sized from a 300-day measurement before 11f; I-9 forbids retuning the controller. If no content makes the market live, that is a finding about the packs' design, reported, not hidden. |
| **R-S9-4** | A declarative macro that generates the catalog is harder to read and to debug than the hand-written enum. | The macro is one file, documented method by method; its expansion is exercised by every existing test (A-1). |
| **R-S9-5** | New content kinds or genesis order perturb social-cafe. | I-4: byte comparison against E-0 in every PR. |
| **R-S9-6** | `Cargo.lock` churn unrelated to the market (a `cargo update`) lands inside a transformation PR. | AC-1's `Cargo.lock` rule fails on any changed non-path package; a transformation PR never runs `cargo update`. |

---

# 10. Questions for the primary session / operator

`[OPERATOR-MATERIAL]` marks a question whose answer changes a public contract, an ownership boundary, a
frozen invariant, or how the frozen top-level criterion is read. Those are not the primary session's to
settle alone under the autonomous authorization (overall §7: "frozen invariants still require an
explicit revision with evidence").

```text
QS-1   Split S9 into six PRs: 11a installable packs, 11b items and organizations, 11c complete
       affordances, 11d + 11e the transformation, 11f the proof (§2.8). 11b and 11c may run in parallel
       after 11a. Alternatives: fold precursors into the transformation (fails AC-1's check by
       construction); one transformation PR (five packs and a world, ~10b + 10c in size).
       Recommended: six.

QS-2   [OPERATOR-MATERIAL] How AC-1 is measured (§2.5, SD-6, ARC-35). The transformation is the merge
       diffs of 11d and 11e against their first parents; the allowed set is systems/**, worlds/**,
       Cargo.lock (only new path packages under systems/; only systems/ dependency lists change) and
       Markdown documentation; plus a dependency-structure check and a configuration-only world-delta
       check; fail closed without git history. The precursors (11a–11c) are outside the range and
       bounded instead by I-2 (they name no market concept and are proven without it) and I-9 (the
       controller's offer band is fixed before the market exists).
       Alternatives: (a) measure all of S9 — impossible for any linked pack, and would forbid the
       framework fixes F-1/F-3 require; (b) read AC-1 only as MVP.md words it (no edit to kernel, Person,
       renderer, controllers) and drop overall §1's path gloss — weaker and less mechanical.
       Recommended: accept, recorded as ARC-35 in 11a C1, and overall §1's gloss amended to cite it.

QS-3   [OPERATOR-MATERIAL] F-1's resolution (§2.2, SD-1 … SD-5, ARC-33, DEP-12). A pack implements
       SystemPack (sdk/rust); the build's installed set is systems/installed, one macro invocation;
       root members glob "systems/*"; new packs depend on siblings by path. Installing = a pack
       directory + two lines in systems/installed + a regenerated Cargo.lock, then a rebuild. What it
       does not deliver: installing without a rebuild, or packs from outside the repository — ARC-8's
       Tier 1, and Milestone E's publishing sense.
       Alternatives: linker-section registration (`inventory`/`linkme`), dynamic loading, a type-erased
       registry (`erased-serde`), worldpack generic over a catalog (§2.2, all declined with reasons);
       or the same design with the installed set outside systems/ (e.g. `distribution/`) and overall's
       gloss amended to admit that one directory.
       Recommended: as designed, installed set in systems/.

QS-4   [OPERATOR-MATERIAL — public contract, controller contract] F-3's resolution (§2.3, SD-9 … SD-12,
       ARC-34). `Affordance<P>` gains an optional payload, attachable only through the offer's own
       action type; `PacedRuleController` gains one band that attempts an available complete affordance
       at a fixed rate; `RuleController` unchanged; existing packs offer no complete affordance, so every
       existing world and transcript is unchanged. Free-form actions (talk, move, invite) stay
       known-by-name.
       Alternatives: controller learns market actions (fails AC-1); policies as world data (rules in a
       World Pack); a server-resolved `interact` (the server choosing for the person, and a resolver
       that knows every pack); payload schemas (heavier than the problem); scoping headless use out of
       AC-1 (an instrument that cannot see, ARC-23).
       Recommended: complete affordances.

QS-5   [OPERATOR-MATERIAL — public World Pack format] Items and organizations as content kinds (11b,
       SD-7): world.yaml `items:`/`organizations:`; items/ and organizations/ files with tags, note and
       sections; ids after people; their sections seeded before places' and people's. It implements
       the frozen MODULE_SPEC §4 layout (F-13) rather than inventing format. Alternative: items as slugs
       inside an inventory and organizations as place tags — no identity to share between packs, and an
       Organization is a CORE_CONCEPTS §8 primitive. Recommended: accept.

QS-6   [OPERATOR-MATERIAL — ontology] An authored Item entity is an item *kind*; what a Person or an
       Organization holds is a count of kinds (CORE_CONCEPTS §7's "stacked items"). Unique instances
       need a pack that creates entities, which the kernel does not allow a system to do (F-12), so
       they are out of S9. Alternative: add entity creation to the kernel's WorldView now — a kernel
       change in S9 (I-8) for a capability no MVP criterion needs. Recommended: kinds and counts,
       recorded as ARC-36.

QS-7   The five packs and their boundaries (§2.6, SD-13): item (kinds), inventory (holdings; the only
       writer), item-transfer (give; no state), economy (wallets, shops, buy; the only mover of money;
       reacts to wage-due with no system dependency), employment (jobs, shifts, wage-due, production;
       never touches a wallet). These establish new ownership; none changes an existing one.
       Sub-question: keep `item` as its own pack, or fold item kinds into inventory? Recommended: keep —
       it is the vocabulary inventory, economy and employment share, and MVP §2 names Item as an
       installable system.

QS-8   Work is attendance during a shift (a Process employment owns; presence read at wakes and
       person-entered-place between), not an action a controller must know. Alternative: start-work /
       stop-work actions offered as complete affordances. Recommended: attendance — a human works by
       being there, the paced controller already walks to its agenda (ARC-32), and no controller has
       to learn that jobs exist.

QS-9   Buying happens at a shop place (target none, complete affordance per priced, stocked kind; the
       operator is an Organization). Alternative: buy from a staff Person on duty — more embodied for
       Demo B, but economy would depend on employment's state. Recommended: place-based in S9; staffed
       selling a later refinement.

QS-10  MVP §5 lists `eat` and `sleep`; overall §3 S9 does not list a needs pack, and neither is required
       by any AC. S9 delivers neither, so held items have no sink. Recommended: record them in overall
       §4 as an MVP-0 interaction gap with no step, for the operator to place (a needs pack after S9, or
       MVP-1).

QS-11  CP-4's precondition (per 30-day bucket of a 300-day run: ≥1 purchase, wage paid, item produced,
       item given), checked before any determinism comparison, with I-9: if the market does not live,
       the remedy is pack design or content, never the controller. Recommended: accept.

QS-12  Milestone C's evidence (CP-7) is a real-server test with two clients and a server restart (§4.6),
       plus the operator's runnable list at the milestone. Recommended: accept.

QS-13  Should the 2D demo learn to submit complete affordances (so a human can buy in Demo A) in S9?
       Recommended: no — it is S12's (Demo A's "basic items"); S9's CP-7 is shown at the protocol level,
       and keeping clients/ out of S9 keeps AC-1's "no renderer change" trivially true.

QS-14  The execution contract for 11a (§11): fresh session, its own worktree and branch, push and PR
       authorized as for 10a–10c, merge the operator's. Recommended: confirm at freeze.
```

---

# 11. Execution contract for PR 11a (proposed; confirmed at 11a's freeze)

```text
PROJECT / PR        MVP-0 · Step 10 / PR 11a — installable System Packs (S9, first of six)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-10-market.md (this file), §4.1 (4.1.1–4.1.3)
RELATED / BINDING   overall.md §§1, 2, 3 (S9), 7; MVP §§1–2, 9 (AC-1, AC-2, AC-12); MODULE_SPEC §§3, 4.1,
                    8; PACKAGE_FORMAT §§6, 8; ARCHITECTURE §§12, 14; REUSE_POLICY §§11–12, 17;
                    DECISIONS ARC-8, ARC-26, ARC-28, ARC-31, DEP-10; step-09-social.md §10.1 (F-1, F-3);
                    this file §§1.3, 2.2, 2.5, 3, 10 (as answered)
IMPLEMENTATION BASE main @ b9e5937 (or the main named at freeze, re-audited); branch
                    mvp0/pr-11a-installable; a worktree under /Users/yuema137/mineworld-worktrees/,
                    held by the implementing session only — unresolved until the primary session names it
APPROVED SCOPE      §1.1 PR 11a; SD-1 … SD-6; F-10's fixture — as QS-1 … QS-3 are answered
FROZEN INVARIANTS   §1.3 I-2, I-4, I-5, I-8 (I-1, I-3, I-6, I-7, I-9 bind later PRs)
SEQUENCE            C1 → C2 → C3 → C4 → C5, each committed and pushed when coherent
VALIDATION BUDGET   unit/integration/static: unrestricted; real-model: NOT REQUIRED; real runs: the 300-day
                    social-cafe comparison (~15 s) and the canary install; about one hour in total
LIVE DOCUMENTATION  this file (§4.1 checkboxes, §9 E-A ledger)
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for PR 11a at C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved — this document is a planning deliverable; the source
                                      will be the primary session's freeze message
  semantic commits, branch push       unresolved until freeze; recommended authorized, as for 10a–10c
                                      ("Commit and push after every small step")
  PR creation / update                unresolved until freeze; recommended authorized (D-12)
  scratch branch for the canary (C5)  recommended authorized, local only, deleted after evidence
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only; never inherited, never widened
POST-MERGE SYNC     the planning session owns step/overall updates; the implementing session owns §4.1
                    and §9 E-A
NORMAL STOP         PR 11a READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any change to §1.3, to a public contract beyond SD-1 … SD-4, to ownership, or to
                    scope; an existing test that fails for any reason but F-10's; a changed social-cafe
                    run (A-1); a needed edit to kernel/, contracts/, persistence/, server/, clients/ or
                    cognition/; C2's failure case (the macro cannot express the generic decoder)
```
