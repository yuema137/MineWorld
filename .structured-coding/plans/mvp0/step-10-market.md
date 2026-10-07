# Step 10 / PR 11 — Market Town and the AC-1 composability proof (S9)

**Role:** step document for S9, proposing a split into six PRs (§2.8). It also holds the full PR
design for the first of them, **PR 11a**, and a proposed execution contract for it (§11). PRs 11b–11f
are specified at medium scope and are each re-audited and detailed to the commit only after the PR
before them merges (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 S9, §4 (`AC-1`, `AC-2`), §7 (F-1, F-3)
**Lifecycle:** step design `DESIGN FROZEN (2026-10-07)` for §§1–3 and the six-PR split.
- **PR 11a** `MERGED` as `c472636` (GitHub #36, 2026-10-07; post-merge docs `7ed1648`).
- **PR 11b** `MERGED` as `ae1a315` (GitHub #39, 2026-10-07), merged first.
- **PR 11c** `MERGED` as `c5dc51c` (GitHub #40, 2026-10-07), rebased onto `ae1a315` before merging
  (§12.0, E-C-rebase).
- **The precursors are complete.** 11a, 11b and 11c are on `main`, each by a merge commit, so the I-2
  scan reads all three as merged on `main`'s first-parent chain (§9 E-3). Every framework capability
  the transformation relies on now exists.
- **PR 11d** is next. The planning session details it to the commit in §4.4, on `mvp0/s9-11d-plan`
  from `main @ c5dc51c`.
- PRs 11e and 11f stay at medium scope until each is detailed and frozen in turn.

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
| **SD-12** | **Clients read the payload and decide nothing new.** `server/PROTOCOL.md` documents the field; ~~`clients/protocol/mineworld/observation.gd` gains `payload(action_type, target)`~~ (amended by QS-20, §12.0: no GDScript change in S9 — that lookup is ambiguous when several complete affordances share a type and target; client use is S12's); `ADOPTION.md` says a client may submit it unchanged. `demo.gd` is not changed in S9. | `ENGINEERING_RULES.md` §§8–9: a payload is the server's answer, carried; no rule moves into a client. Using it in the 2D client is S12's. |
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
- [x] Full gates once on the final executable head `70b0857` (§9 E-A-final): all PASS.
- [x] Review: A-1 (E-A4, E-A-final: sha = E-0; 428 tests pass; the only existing-test edit is F-10's),
  A-2 (`worldpack/tests/structure.rs`, mutation M2), A-3 (`systems/installed/tests/installed.rs`,
  mutations M1 and M3), A-4 (E-A5) each hold with recorded evidence; the I-2 scan (C4b, E-A4b, E-A5b)
  is green on the PR and shown to bite; the scratch branches were never pushed. Deviations D-A1 … D-A3
  are bounded and recorded; nothing material arose.

**PR 11a lifecycle: MERGED** — GitHub #36, merge commit `c472636` (2026-10-07), a merge commit as the
I-2 scan's merged range requires. Final executable head `70b0857`. Parent synchronization: overall §1's
AC-1 gloss citing ARC-35 and overall §7 were recorded in `a151c41` (merged as `7ed1648`); this step's
header and this line were recorded by the planning session that detailed 11b and 11c (§4.2, §4.3).

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

## 4.2 PR 11b — items and organizations as World Pack content kinds (full design; MERGED as `ae1a315`)

### 4.2.1 Identity, base, approved scope

```text
PR            11b — items and organizations as World Pack content kinds (S9, second of six; a precursor)
base          main @ 7ed1648 plus this planning branch once merged, or the main the primary session
              names at freeze (re-audit §8.4 if anything under authoring/, worldpack/, tools/cli/src/
              or tests/acceptance/ moved)
branch        mvp0/pr-11b-content-kinds, in its own worktree, held by the implementing session only;
              runs in parallel with 11c under §12
audit         §8 (b9e5937) and §8.4 (7ed1648)
scope         §1.1 PR 11b; SD-7, SD-8; F-13, F-16, F-20 … F-23, F-31, F-32; QS-5, QS-6 (approved)
depends on    11a (merged): the generated Capability, SectionOwner, the I-2 scan
```

**Goal.** A World Pack can declare `Item` and `Organization` entities — `items:` and `organizations:` in
`world.yaml`, one file each under `items/` and `organizations/` — exactly as `MODULE_SPEC.md` §4's frozen
layout already lists them, and those files can carry sections, so a System Pack can own state on them
the way `naming` and `schedule` own state on people (`ARC-31`). The capability is the format's, not the
market's: §4.1's implemented subset simply stops omitting two of the four kinds of entity
`CORE_CONCEPTS.md` defines.

**Justified and proven without the market (I-2).** Every fixture and test of 11b uses neutral content,
chosen so that nothing in it is or implies trade:

```text
items           lantern, pebble          things that exist in a world and can be named by key
organizations   chess-club               CORE_CONCEPTS §8 names "club" among its examples
people, places  the existing fixtures', or social-cafe copied at runtime (never committed)
```

Nothing 11b adds is a section owner; no installed pack is taught to carry a section on an item or an
organization file (F-22). The genesis order and the reference typing of sections on those files are
proven with a probe section owner that exists only in worldpack's own unit tests.

**Non-goals.** No System Pack, no installed-set line, no section owned by an existing pack on the new
kinds (`naming` stays people-only: §1.2); no `Seeding` change (F-20); no item instances (SD-8, QS-6); no
`create` template change; no client, server, persistence, kernel or contracts change (I-8).

**Acceptance (all observable, decided before measuring, `ARC-23`).** Each criterion that guards
something names the mutation shown to break it; every mutation is applied to the working tree, observed
to fail by name, and reverted, with `git status` recorded afterwards.

```text
B-1  Nothing existing moves (I-4). The 300-day seed-7 social-cafe run prints, apart from `wall`, the
     lines of E-0 (sha-256 ad49c723…c64b); `mineworld validate worlds/social-cafe` is byte-identical to
     the base's output; every existing test passes, and no existing test is edited — a literal update
     that turns out to be needed is listed with its unchanged claim.
B-2  A pack declaring items and organizations reads and loads, and they are allocated after people:
     places, then people, then items, then organizations, each in key order. Their entity types are
     Item and Organization; their tags reach the registry; their provenance names items/<key>.yaml and
     organizations/<key>.yaml; tags-only files add no genesis fact.
     Guards: worldpack/tests/content_kinds.rs. Mutation M-B1: create items before people in
     load.rs → the id assertions fail, naming the first moved key.
B-3  A bad item or organization file is refused by name and file (the refusals.rs standard): a declared
     key with no file; a file no list declares; a key declared in two lists across kinds (both lists
     named); a field the kind does not have (`location:` on an item), with line and column; a section
     its owner does not let the kind carry (`name:` on an item, `NotCarriedHere`, naming `person`).
     Guards: refusals.rs. Mutation M-B2: drop items from check_keys_are_declared_once → the
     cross-kind duplicate test fails.
B-4  Sections on the new kinds are seeded first and referenced by type. A pack whose items,
     organizations, places and people each carry a probe section seeds them in the order items',
     organizations', places', people' (each in key order); a section naming an item key as an Item is
     accepted, and naming it as a Place is refused `SectionNamesUnknownEntity`.
     Guards: worldpack's unit tests with a probe owner (F-22). Mutation M-B3: move items after people
     in the one genesis-order function → the order test fails (§4.2's adversarial). Mutation M-B4:
     restore the `_ => false` reference arm (F-21) → the Item-reference test fails.
B-5  Inert content changes no simulation, through the real CLI. Social-cafe copied at runtime with two
     items and one organization added: `mineworld validate` lists them in `items` and `organizations`
     lines and allocates them ids 19–21 after social-cafe's unchanged 1–18, with 53 genesis facts; a
     10-day seed-7 run's fact table is byte-identical to social-cafe's own 10-day run; the same world
     run to day 5 and continued to day 10 on one save equals its uninterrupted 10-day run in facts,
     journal and snapshots (`AC-6`, `AC-12` for a world holding Item and Organization entities).
     Guards: tools/cli/tests/content_kinds.rs. Mutation M-B5: allocate items before places → the id
     assertion fails.
B-6  I-2 holds and the scan sees past an admitted word. The scan is green with 11b's row; its allow-list
     admits only the words `item` and `items`, per file, each with a reason (§4.2.2); every other market
     word on an admitted line is refused. Planted, in the working tree: `// PLANTED: an item_price` in
     worldpack/src/format.rs → refused naming `price` and not `item`; an untracked file
     worldpack/tests/wages.rs → refused by its path. Mutation M-B6: admit lines instead of words
     (11a's rule) → the scan's unit test `an_admitted_word_admits_no_other` fails.
B-7  The documents say it first (`CLAUDE.md` §2.2): ARC-36, MODULE_SPEC §4.1, PACKAGE_FORMAT §8 and a
     CORE_CONCEPTS §7 note exist before the code that relies on them; both doc checks pass.
```

**Commit plan.** Six commits after this design (C0). Evidence goes into §9.2 as `E-B<n>`; a planned
commit may become several coherent commits, and the mapping is recorded.

### B-C0 — Design (this section) — docs only

- [x] Implementation: §4.2, §8.4, §12, §13 and QS-15 … QS-19, by the planning session (`mvp0/s9-11bc-plan`).
- [x] Validation: both doc checks (§9 E-2).
- [x] Review: every file and symbol named here was read on `7ed1648` (§8.4); I-2's neutral content and
  allow-list are stated before any code exists; operator-material points are marked (§10). Self-review
  by the planning session only; the primary session's review is pending.

### B-C1 — Specs before code: ARC-36, ARC-35 note, MODULE_SPEC §4.1, PACKAGE_FORMAT §8, CORE_CONCEPTS §7

**Goal.** The format extension and the reading of `Item` exist as reviewable specification before code
relies on them (`CLAUDE.md` §2.2, B-7).

**Scope.**
- `docs/DECISIONS.md` — **ARC-36** *An authored Item is a kind; items and organizations are content
  kinds of a World Pack*, appended at the end of the file (§12): SD-7 and SD-8; stacked items only, a
  held quantity being a count of a kind (`CORE_CONCEPTS.md` §7's "stacked items"); instances need a pack
  that creates entities, which no system may do (F-12); ids after people so no existing id moves; keys
  one namespace across the four kinds (F-16); sections may be carried by item and organization files,
  extending `ARC-31`'s person-or-place wording; their genesis order and why (what people's and places'
  sections refer to is seeded first). Accepted limitations: no instances; no names for items or
  organizations (`naming` is people-only); no installed pack carries a section on them until 11d.
- `docs/DECISIONS.md` — a dated **note on ARC-35** point 7 (the I-2 scan): an allow-list entry admits
  named words, never a whole line (F-31, QS-16); and, only if QS-15 is approved, the merged-range
  detection reads every merge reachable from `HEAD`, requiring exactly one match (F-30).
- `docs/MODULE_SPEC.md` §4.1 — `world.yaml` gains `items:` and `organizations:`; the two file kinds
  with their fields (`tags`, `note`); sections may be carried by them; rule 1 says a key is stated once
  across all four lists; rule 2's "entity identities are allocated in key order" becomes "places, then
  people, then items, then organizations, each in key order"; the genesis paragraph's "places' before
  people's" becomes "items', organizations', places', people's". The §4 model above it is unchanged.
- `docs/PACKAGE_FORMAT.md` §8 — the World Pack fields row names `items`, `organizations` and their files.
- `docs/CORE_CONCEPTS.md` §7 — one sentence: in MVP-0 an authored Item is an item kind and holdings are
  counts of kinds (`ARC-36`). A pointer, not a new rule; the ontology text is unchanged.

**Depends on:** freeze. **Non-goals:** no code.

- [x] Implementation: as scoped. `git fetch` on 2026-10-07: no `origin/*` branch holds `ARC-36`
  (`git grep -l ARC-36 <branch> -- docs/DECISIONS.md` empty for every branch). ARC-36 appended after
  ARC-35; the ARC-35 note sits directly under ARC-35 (11c inserts ARC-34 *before* `## ARC-35`, so the
  hunks stay distinct). QS-15 was declined (§12.0), so the note records that the first-parent detection
  is unchanged and names §12.0's one-at-a-time rule instead of an amendment. MODULE_SPEC §4.1: example
  `items:`/`organizations:` lists, the item/organization file block, the sections sentence and the
  "which content files" bullet; rules 1 and 2 and the genesis paragraph as scoped. PACKAGE_FORMAT §8's
  row names `items`, `organizations`, the four content directories and four carrying kinds.
  CORE_CONCEPTS §7: one pointer sentence. Handoff `handoff-11b.md` initialized.
- [x] Validation: E-B1 — `check_decision_ids` 46 ids distinct; `check_doc_headings` 143 sections, none
  duplicated; `MODULE_SPEC.md:265 ## 4.1`, `CORE_CONCEPTS.md:389 # 7. Item`, `PACKAGE_FORMAT.md:286
  # 8.` resolve.
- [x] Review: ARC-36 reads `Item` as one of the two forms CORE_CONCEPTS §7 permits (stacked) and says
  so, no new term; MODULE_SPEC §4.1 still has six numbered rules (1 and 2 extended in place); the
  ARC-35 note only tightens point 7 and states QS-15 as declined with §12.0 as the source. Prose in the
  docs is not scanned (Markdown), so market words there are not an I-2 concern.

**Acceptance.** As validation. **Commit boundary.** Documentation only.

### B-C2 — The I-2 scan admits words, not lines; 11b's row

**Goal.** Close F-31 before 11b relies on the allow-list for the word `item`, and register 11b with the
scan (freeze condition 1, B-6).

**Scope** (`tests/acceptance/tests/precursor_vocabulary.rs` only):
- `Allowed` gains `words: Words`, with `enum Words { Any, Only(&'static [&'static str]) }`. `Only` names
  the exact lowercase words an entry admits; `Any` is legal only for the scan's own file, and the test
  fails naming any other entry that uses it.
- Every market word on an added line is found (not only the first); each is refused unless an entry
  for that PR and path whose `contains` the line holds admits that exact word. A refusal names the
  word. The stale-entry rule is unchanged: an entry that admits nothing fails.
- 11a's entry gains `words: Words::Any`; its meaning is unchanged (the scan's own file).
- `PRECURSORS` gains `{ pr: "11b", base: "<the full sha the branch was cut from>", branch:
  "mvp0/pr-11b-content-kinds" }`. When the branch later integrates main, the base moves to the
  integrated main commit in the same commit (§12).
- `ALLOWED` gains 11b's self-entry `{ pr: "11b", path: <this file>, contains: "", words: Words::Any,
  reason: "this scan: its allow-list names the words it admits" }`. Every other 11b entry is added in the
  commit that adds the lines it admits, because an entry that matches nothing fails.
- **Only if QS-15 is approved:** `merged_head` reads `git log --merges --format=%H %s HEAD` (every merge
  reachable from `HEAD`, not only the first-parent chain) and requires exactly one subject of the form
  `Merge pull request #N from <owner>/<branch>`; two are a failure naming both. The parsing becomes a
  pure function over the log text so it can be unit-tested.
- New unit test `an_admitted_word_admits_no_other`: under an entry `Only(&["item", "items"])`, the lines
  `let item_price = 1;`, `struct ItemPrice;` and `items: [wage]` are each refused, naming `price`,
  `price` and `wage`; `let itemprice = 1;` is refused as `itemprice` (not the admitted word); `let items
  = 1;` passes. If QS-15 is approved, a second unit test over hand-written log text: one match → the
  merge's `^2`; none → unmerged; two → the failure.

**Depends on:** B-C1. **Non-goals:** no change to the vocabulary, to what is scanned (every non-Markdown
added line and added path) or to fail-closed behaviour.

- [x] Implementation: as scoped. `Words { Any, Only }` with `admit`; `THIS_SCAN` const; `market_words`
  (every match) replaces `market_word` (first match); the pure `refused_words(pr, path, text, allowed,
  used)` decides each word and marks used entries; a guard fails naming any `Any` entry outside
  `THIS_SCAN`. Row 11b base `da316134e8bf8a82d1f65bbeaab62f3368222a3d`; 11b self-entry `Any`.
  QS-15 conditional item not implemented (declined, §12.0). **Literal update D-B1:** the existing
  `the_matcher_sees_every_form_of_a_market_word` called `market_word(text).is_some()` /
  `.is_none()`; it now calls `!market_words(text).is_empty()` / `.is_empty()`. Claim unchanged (the
  same six lines match, the same four do not).
- [x] Validation (E-B2):
  - [x] `cargo test -p mineworld-acceptance`: 4 passed (3 existing + `an_admitted_word_admits_no_other`),
    including the 11b row over the working tree.
  - [x] Mutation M-B6 (`&& (true || entry.words.admit(&word))`, the word check bypassed) →
    `an_admitted_word_admits_no_other` FAILED: `in: let item_price = 1;` left `[]` right `["price"]`;
    the other three passed. Reverted (grep finds no `true ||`; `git status` shows only the intended
    edit). This is QS-16's required mutation: an `item_price` line refused for `price`.
  - [x] `cargo clippy -p mineworld-acceptance --all-targets -- -D warnings` clean; `cargo fmt --all
    --check` clean.
- [x] Review: only tightens — a word is admitted only by an entry that, before, would have admitted its
  whole line, plus the word condition; `Any` is guarded to `THIS_SCAN`; both `Any` entries have reasons;
  stale-entry, fail-closed and vocabulary unchanged; matches ARC-35's note (B-C1) item by item.

**Failure cases.** A precursor line that needs a market word other than `item`/`items` is a material stop
(ARC-35 point 7), never an entry added to pass.

### B-C3 — Items and organizations are read: kinds, format, files, refusals

**Goal.** B-3, and the read half of B-2: `WorldPack::read` accepts the two keys and their files and
refuses a bad one by name. Kinds and reading land together because adding the `ContentKind` variants
makes worldpack's `fields(kind)` match non-exhaustive — one does not compile without the other.

**Scope.**
- `authoring/src/section.rs` — `ContentKind::{Item, Organization}` after `Place` (the derived order of
  the existing two is unchanged); `directory` → `items`, `organizations`; `describes` → `item`,
  `organization`; `entity_type` → `EntityType::Item`, `EntityType::Organization`; `pub const ALL:
  [ContentKind; 4]` in that order, for guards that must cover every kind. Doc comments say "a person,
  place, item or organization file".
- `worldpack/src/format.rs` — `WorldManifest.items`, `WorldManifest.organizations`: `Vec<EntityKey>`,
  `#[serde(default)]`, each "names a file in `items/`" / "`organizations/`"; `AuthoredItem` and
  `AuthoredOrganization`, each `{ tags: Tags, note: Option<String>, sections: Vec<FoundSection> }`,
  `Default`, documented like `AuthoredPlace`. Two structs, not one shared one: the kinds are distinct
  concepts that will diverge, and a shared struct now would be the premature abstraction `CLAUDE.md`
  §4 rule 11 forbids.
- `worldpack/src/content.rs` — `ITEM_FIELDS` and `ORGANIZATION_FIELDS` = `["tags", "note"]`; `fields`
  gains both arms; `ContentFile::item` and `ContentFile::organization` beside `person` and `place`. A
  `location:` or `passages:` key on the new kinds is therefore an unknown key, refused at its line by the
  existing visitor — no new refusal path.
- `worldpack/src/error.rs` — `Declared::{Items, Organizations}`, displayed `items`, `organizations`.
- `worldpack/src/read.rs`:
  - `WorldPack` gains `items` and `organizations` (`BTreeMap<EntityKey, Authored…>`) and accessors
    `items()`, `organizations()`, documented "in key order".
  - `check_keys_are_declared_once` covers all four lists in the order places, population, items,
    organizations, so a cross-kind duplicate names both lists.
  - `read_content` for `items/` and `organizations/`; `check_nothing_undeclared` for both directories
    (a directory that does not exist is still fine).
  - One function states the order every per-file pass uses — items, organizations, places, people, each
    in key order — and both `check_sections` (here) and genesis (B-C4) iterate it, so the order the
    loader refuses in and the order it seeds in are one statement.
  - `check_sections` takes a `BTreeMap<EntityKey, EntityType>` of every declared key, built once, and
    accepts a reference iff the key is declared with the required type; the `_ => false` arm goes (F-21).
  - The module's numbered check list is updated (step 5 covers four lists; step 6 four directories).
- `worldpack/src/lib.rs` — the layout diagram gains `items/` and `organizations/`; re-export
  `AuthoredItem`, `AuthoredOrganization`.
- `worldpack/src/catalog.rs` — the section-namespace guard checks every kind's fields
  (`ContentKind::ALL`), not only people's and places'.
- `worldpack/tests/refusals.rs` — new tests, each writing its own `items/` or `organizations/`
  directory (F-23):
  - `a_declared_item_with_no_file_is_refused_by_name` (`ContentFileMissing`, kind Item);
  - `an_organization_file_no_list_declares_is_refused_by_name` (`ContentFileNotDeclared`, list
    Organizations);
  - `a_key_declared_as_a_place_and_an_item_is_refused_naming_both_lists` (`KeyDeclaredTwice`, Places,
    Items) and the same for population and organizations;
  - `a_field_an_item_does_not_have_is_refused_at_its_line` (`location:` in `items/lantern.yaml`,
    `Malformed` with line and column, listing `tags`, `note`);
  - `a_section_an_item_may_not_carry_is_refused_naming_the_kinds_that_may` (`name:` on an item with
    `naming` enabled → `SectionNotCarriedHere`, kind Item, carried by `person`);
  - `every_refusal_names_the_file_it_is_about` gains an item case and an organization case.

**Depends on:** B-C2 (the scan must see these lines with word-level admission). **Non-goals:** loading
(ids, genesis) is B-C4; no CLI change.

- [x] Implementation: as scoped, plus `ALLOWED` entries for the seven files that now hold `item`/`items`
  (section.rs, format.rs, content.rs, error.rs, read.rs, lib.rs, refusals.rs), each
  `Words::Only(&["item", "items"])` through one const `ITEM`, each with its reason. Before the entries
  existed the scan refused exactly 109 hits, all `item`/`items`, all in those seven files. No other
  market word anywhere.
  - The order function is `WorldPack::sectioned_files` (pub(crate)). `check_sections` now takes the
    assembled `&WorldPack`, so `read` builds `Self` before the section check, which is still the last
    check. The reference rule is `declared_entities().get(key) == Some(&type)`, built once from all
    four maps (F-21's `_ => false` is gone).
  - `check_keys_are_declared_once` reads one array of the four lists in the order places, population,
    items, organizations. `check_nothing_undeclared` runs over the four kinds in one loop.
  - **Bounded deviation D-B2:** two `PackError` messages wrote "a {kind}", which would read "a item".
    error.rs gains a private `one(kind)` ("a person", "a place", "an item", "an organization") used in
    `ContentFileMissing` and `ContentFileNotDeclared`. The person and place text is byte-identical, and
    no variant, field or meaning changes.
  - **Bounded deviation D-B3:** `authoring/Cargo.toml`'s and `worldpack/Cargo.toml`'s
    descriptions/comments still say "person or place file". They are left unedited because they are
    manifest prose, outside the design's file list, with no reader that depends on them. Recorded as
    a doc follow-up.
- [x] Validation (E-B3):
  - [x] `cargo test --no-fail-fast -p mineworld-worldpack -p mineworld-authoring -p mineworld-acceptance`:
    all pass. refusals 38 (32 existing + 6 new + `every_refusal…` extended with item and organization
    cases), social_cafe 15, structure 2, worldpack unit 2 (the section-namespace guard now over
    `ContentKind::ALL`), acceptance 4. No existing test edited except the design-sanctioned extension of
    `every_refusal_names_the_file_it_is_about`.
  - [x] Mutation M-B2 (the `items` row dropped from `check_keys_are_declared_once`) →
    `a_key_declared_as_a_place_and_an_item_is_refused_naming_both_lists` FAILED (37 passed, 1 failed);
    reverted, 38 passed.
  - [x] `cargo clippy -p …worldpack -p …authoring -p …acceptance --all-targets --all-features -D
    warnings` clean; `cargo fmt --all --check` clean (after `cargo fmt`).
- [x] Review: every new refusal reuses an existing variant (`ContentFileMissing`,
  `ContentFileNotDeclared`, `KeyDeclaredTwice`, `Malformed`, `SectionNotCarriedHere`) with the new kind
  or list as a value; `fields(kind)` is the only statement of legal fields, so `location:` on an item is
  refused by the existing unknown-key path; `sectioned_files` is the only statement of per-file order;
  no pack crate named (structure tests pass); social-cafe's check order is unchanged for packs without
  the new kinds (places then people).

**Failure cases.** A needed new `PackError` variant is a bounded addition, recorded; a needed change to
an existing variant's meaning is a stop (it changes what authors are told).

### B-C4 — Items and organizations are loaded: identity, provenance, genesis order

**Goal.** B-2 and B-4: entities are created after people, and sections on the new kinds are seeded first.

**Scope.**
- `worldpack/src/load.rs`:
  - `assemble` creates places, then people (unchanged), then items, then organizations, each in key
    order, with provenance `items/<key>.yaml` / `organizations/<key>.yaml` and the file's `note`.
  - `initial_facts` seeds sections over B-C3's order function: items', organizations', places',
    people', each in key order, within a file in composition order. Passages and locations stay first.
  - The module documentation's creation-order block and genesis paragraph state the new order and why
    (what other sections refer to is seeded before them; existing ids and event ids do not move).
- Unit tests, in worldpack's own source (F-22): a probe section owner `Probe` (`AuthoredSection`,
  carried by all four kinds, seeding one `probed { subject }` fact of its own vocabulary, with
  `references` naming whatever key the authored value lists and the type it lists it as), decoded with
  `Decode::<Probe>` and attributed to an installed `Capability` for ranking only — as
  `a_section_that_seeds_another_packs_fact_is_refused_by_name` does with `Trespasser`. A
  `#[cfg(test)]` constructor builds a `WorldPack` holding probe sections on two items, one organization,
  one place and one person. Two tests:
  - `sections_on_items_and_organizations_are_seeded_before_places_and_people`: `assemble()`'s facts, in
    order, are the probe facts for the items (key order), the organization, the place, the person.
  - `a_section_may_name_an_item_as_an_item_and_not_as_a_place`: through `check_sections`, a reference
    `(lantern, Item)` is accepted and `(lantern, Place)` is refused `SectionNamesUnknownEntity`.
- `worldpack/tests/content_kinds.rs` (new) — a fixture pack written at runtime: places `cafe`, `park`;
  people `alice`, `bob`; items `pebble`, `lantern` (listed out of key order on purpose); organization
  `chess-club`; `systems: [presence]`; tags-only item and organization files. One test,
  `items_and_organizations_are_allocated_after_people_in_key_order`: ids cafe 1, park 2, alice 3, bob
  4, lantern 5, pebble 6, chess-club 7; entity types; tags; provenance paths; genesis facts identical
  in ids, types and payload bytes to the same pack with the two lists and directories removed.

**Depends on:** B-C3.

- [x] Implementation: as scoped, plus `ALLOWED` entries for `worldpack/src/load.rs` and
  `worldpack/tests/content_kinds.rs`. Before those two entries existed, the scan refused only
  `item`/`items` in those two files.
  - `assemble` creates item kinds and then organizations after people.
  - `initial_facts` seeds sections over `WorldPack::sectioned_files()`, the function B-C3 added. Its
    hand-built places-then-people chain is gone.
  - The module docs state the new creation and genesis order.
  - Test-only seams:
    - `#[cfg(test)] WorldPack::in_memory(systems, places, people, items, organizations)` in read.rs;
    - `check_sections` became `pub(crate)` so the reference test can reach it.
  - The probe owner `Probe` (section `probe`, carried by `ContentKind::ALL`) is test-only. It seeds
    one `probed { subject }` fact of its own vocabulary, and its `references` list whatever
    `{key, entity_type}` pairs its value holds.
  - **Bounded deviation D-B4:** the content_kinds fixture places alice and bob with `location:`, so
    genesis is non-empty and the byte comparison compares something. The test asserts that. The design
    named only tags-only item and organization files, and those are unchanged.
- [x] Validation (E-B4):
  - [x] `cargo test --no-fail-fast -p mineworld-worldpack -p mineworld-acceptance`: all pass. Worldpack
    unit 4 (2 new probe tests), content_kinds 1 (new), refusals 38, social_cafe 15 (unedited: ids,
    genesis 53, sections-do-not-move), structure 2, doc 1, acceptance 4.
  - [x] Mutation M-B1: an items loop was placed before places and the real one disabled.
    `content_kinds` FAILED with left `[("lantern", 1), ("pebble", 2), ("cafe", 3), ("park", 4),
    ("alice", 5), …]` and right `[("cafe", 1), …, ("alice", 3), …]`. The probe order test failed too.
    Reverted, and `git grep MUTATION` is empty.
  - [x] Mutation M-B3 (`organizations.chain(places).chain(people).chain(items)` in `sectioned_files`)
    → `sections_on_items_and_organizations_are_seeded_before_places_and_people` FAILED. Reverted.
  - [x] Mutation M-B4: the reference rule was restricted to Place and Person, with `_ => false` for
    every other type. `a_section_may_name_an_item_as_an_item_and_not_as_a_place` FAILED on the Item
    acceptance. Reverted.
  - [x] clippy `-D warnings` (worldpack, acceptance; forced recheck) clean; fmt clean.
- [x] Review:
  - Existing ids cannot move: items and organizations are created after the last person.
  - Event ids cannot move:
    - their sections are seeded after every passage and location;
    - in a pack with neither kind, `sectioned_files` yields exactly places then people, as before;
    - social_cafe's genesis-53 and sections-do-not-move guards pass unedited.
  - `Probe` and `in_memory` exist only under `#[cfg(test)]`, and `Probe` is never installed.
  - `seeded()`'s `SectionStatedAnotherPacksFact` guard runs for every file `sectioned_files` yields,
    and that includes the new kinds.

### B-C5 — The real CLI: `validate` names them; inert content changes no run

**Goal.** B-5, through the real binary (`CLAUDE.md` §4 rule 9: create world → load → act → persist →
restart → verify), and F-32's summary gap.

**Scope.**
- `tools/cli/src/main.rs` `validate` — prints `  items      …` and `  organizations …` **only when the
  pack declares some**, so social-cafe's report is byte-identical (B-1). The id list needs no change
  (F-32).
- `tools/cli/tests/content_kinds.rs` (new; uses the existing `headless` module by `mod headless;`, editing
  nothing in it):
  - a helper copies `worlds/social-cafe` into `CARGO_TARGET_TMPDIR/with-things/`, sets `id:
    with-things`, appends `items: [lantern, pebble]` and `organizations: [chess-club]` to `world.yaml`,
    and writes the three tags-only files. Nothing is committed under `worlds/` (I-2: copied social-cafe
    content would be added lines with market words, E-A5).
  - `validate_lists_items_and_organizations_after_every_existing_id`: success; the two summary lines;
    `19  lantern`, `20  pebble`, `21  chess-club`; every id line 1–18 equal to social-cafe's own report's;
    `53 genesis fact(s)`.
  - `inert_items_and_organizations_change_no_fact_of_a_run`: `run --seed 7 --days 10 --save` of both
    worlds; the two fact tables are equal row for row and byte for byte. If a fact row turns out to
    carry the pack id, the comparison is of the decoded envelopes without it, recorded as a bounded
    deviation with the reason.
  - `a_world_with_items_and_organizations_resumes_byte_for_byte`: `--days 5 --save C`, then `--days 10
    --save C`, against an uninterrupted `--days 10 --save A`: `Tables::assert_same_history`.

**Depends on:** B-C4.

- [x] Implementation: as scoped, plus `ALLOWED` entries for `tools/cli/src/main.rs` and
  `tools/cli/tests/content_kinds.rs`.
  - `validate` prints `  items      …` and `  organizations …` only when non-empty (QS-18).
  - The test copies social-cafe recursively into `CARGO_TARGET_TMPDIR`, rewrites the id to
    `with-things`, and appends the two lists and three tags-only files.
  - The fact tables compare row for row, byte for byte. No pack id is in a fact row, so the
    conditional deviation the design allowed for was not needed.
  - First-run corrections, both to the test's own expectations, not the CLI:
    - `listed()` prints names unquoted;
    - `EntityId`'s `Display` ignores `{:>4}`, so the id lines read `  19  lantern`.
- [x] Validation (E-B5):
  - [x] `cargo test --no-fail-fast -p mineworld-acceptance -p mineworld-cli --test precursor_vocabulary
    --test content_kinds --test commands`: scan 4, commands 4 (unedited), content_kinds 3 — all pass.
    The inert-run test asserts the social-cafe 10-day fact table has > 1 000 rows before comparing.
  - [x] `mineworld validate worlds/social-cafe` byte-identical to the **base's**. The base binary was
    built from `da31613` in a scratch detached worktree (`/Users/yuema137/mineworld-worktrees/s9-11b-base`,
    removed after the final gate). `diff /tmp/s9-11b-base-validate.txt /tmp/s9-11b-validate-after.txt`
    is empty.
  - [x] Mutation M-B5 (items allocated before places) → `validate_lists_items_and_organizations_after_
    every_existing_id` FAILED with "every existing id stays where it was" (left `1 lantern, 2 pebble,
    3 apartments, 4 cafe …`, right `1 apartments, 2 cafe …`). Reverted; `git diff worldpack/src/load.rs`
    is empty.
  - [x] clippy `-D warnings` (cli, acceptance) clean; fmt clean.
- [x] Review: the CLI change only presents what the pack declared; `run`, `inspect`, `biography` and
  `server` are untouched. The resume test compares facts, journal and snapshots of a save holding Item
  and Organization entities, so they survive a snapshot and a resume.

### B-C6 — Close: status, planted violations, full gates, ledger

- [x] Documentation (8350ba4):
  - `docs/MVP_STATUS.md` gains its two rows at the stated anchors. The `Updated:` line and the S9 row
    are untouched.
  - `worldpack/README.md` names the four kinds and their id order.
  - §4.2 checkboxes, §9.2 `E-B*` and `handoff-11b.md` are kept current.
- [x] Validation, once, on the final executable head **8350ba4** (clean tree), E-B6:
  - [x] fmt PASS (0 s). clippy `--workspace --all-targets --all-features -D warnings` PASS (26 crates
    checked, 2 s). `cargo test --workspace --no-fail-fast` 441 passed, 0 failed, 0 ignored, 185 s wall.
    That is the base's 428 (E-A-final) plus 13 new tests:
    - scan +1;
    - refusals +6;
    - worldpack unit +2;
    - worldpack content_kinds +1;
    - cli content_kinds +3.
  - [x] B-1: the 300-day seed-7 run exits 0 with 339 lines, faults 0, history 365 330 facts and
    fingerprint 59339a9c281829c9. sha-256 of all lines but `wall` =
    `ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b` = E-0 (wall 12.9 s).
    `validate worlds/social-cafe` is identical to the base's (E-B5).
  - [x] B-6: `// PLANTED: an item_price` in worldpack/src/format.rs →
    `11b: worldpack/src/format.rs:235: \`price\` in: // PLANTED: an item_price`, and `item` is not
    named. The untracked `worldpack/tests/wages.rs` → `11b: the added file worldpack/tests/wages.rs is
    named \`wages\``. Both were reverted with `git checkout` and `rm`; `git status` is then clean, and
    the scan passes 4/4 again.
  - [x] kill_and_resume: cafe PASS, clock PASS (6 s). check_decision_ids: 46 ids, distinct.
    check_doc_headings: 143 sections, none duplicated.
- [x] Review:
  - B-1 … B-7 hold (§9.2).
  - Deviations D-B1 … D-B4 are listed in B-C2 … B-C4, plus process deviation D-B5, below.
  - `git diff --stat da31613..8350ba4` touches only authoring/, worldpack/, tools/cli/,
    tests/acceptance/, docs/ and the plan files: no contracts/, kernel/, persistence/, server/,
    clients/, cognition/ or systems/.
  - **D-B5 (process):** an `awk` one-liner was used once to sum test counts. The kickoff's tool
    discipline forbids `awk`. It changed no file and the sum is reproducible from
    `/tmp/s9-11b-final/test.txt`, but it is recorded, not hidden.

**PR 11b lifecycle: MERGED** — GitHub #39, merge commit `ae1a315` (2026-10-07), merged first, by a
merge commit. Operator's review: gates re-run 441/0; the operator's own plant (`// an item shop` in
`worldpack/src/read.rs`) was refused naming `shop`; the diff touched no `contracts`, `kernel`,
`persistence`, `server`, `clients` or `systems`. Recorded by the planning session (§12.0, "After both
merge"). The record below is the implementing session's, as it stood at review:
`DESIGN FROZEN (2026-10-07)`, execution contract §13 confirmed (freeze record
§12.0). **READY FOR OPERATOR REVIEW**: final executable head `8350ba4` with all gates in E-B6. Later
commits are ledger and handoff only. The PR is to be merged with a merge commit (§12). The
implementation context is CLOSED / AWAITING OPERATOR ACTION. If 11c merges first, this branch
rebases per §12.0 and its evidence is rerun.

### 4.2.2 The 11b allow-list

`ARC-35` point 7 admits a match that is not a market concept. 11b needs exactly one such word: the
defined term **Item** (`CORE_CONCEPTS.md` §7), which is already `EntityType::Item` and `ItemId` in
`contracts/src/ids.rs` and `items/` in `MODULE_SPEC.md` §4's frozen layout. Its plural is the
directory's and the key's name. So every 11b entry is `Words::Only(&["item", "items"])`, and any other
market word on the same line is still refused (B-C2).

```text
path                                            words          why the word is there (the entry's reason)
authoring/src/section.rs                        item, items    ContentKind::Item, its directory `items`
                                                               and its description `item`
worldpack/src/format.rs                         item, items    world.yaml's `items:` list and AuthoredItem
worldpack/src/content.rs                        item, items    ITEM_FIELDS and ContentFile::item
worldpack/src/error.rs                          item, items    Declared::Items, displayed `items`
worldpack/src/read.rs                           item, items    the items map, its accessor, its reading
                                                               and the per-file order
worldpack/src/load.rs                           item, items    creating Item entities; the probe tests
worldpack/src/lib.rs                            item, items    the layout diagram and the AuthoredItem
                                                               re-export
worldpack/tests/refusals.rs                     item, items    refusal fixtures for items/<key>.yaml
worldpack/tests/content_kinds.rs                item, items    the loading fixture's items
tools/cli/src/main.rs                           items          validate's `items` summary line
tools/cli/tests/content_kinds.rs                item, items    the CLI fixture's items
tests/acceptance/tests/precursor_vocabulary.rs  Any            this scan: its allow-list names the words
                                                               it admits
```

Rules the implementing session follows: an entry is added in the commit that adds the lines it admits;
a listed file that ends up not needing the word loses its entry (an unused entry fails); a file not
listed that needs `item`/`items` for the same reason gets an entry of the same form, recorded as a
bounded deviation; prose in comments uses another word where `item` would mean "entry of a list"
(11a's E-A4b precedent); **any other market word is a material stop**. No file 11b adds has a market
word in its path: fixtures are written at runtime.

### 4.2.3 Test ownership for 11b

```text
STATIC      fmt, clippy -D warnings; exhaustive matches over ContentKind (a kind with no fields, no
            directory or no entity type does not compile)
UNIT        the scan's word-level admission (B-C2); genesis order of sections on the new kinds and
            typed references, with a probe owner (B-C4); the section-namespace guard over every kind
INTEGRATION refusals over real pack directories (B-C3); loading a fixture pack (B-C4); every existing
            worldpack, CLI, persistence and server test unchanged (B-1)
REAL RUN    validate and 10-day runs of a social-cafe copy with inert items and organizations, saved and
            resumed (B-C5); the 300-day social-cafe comparison (B-1)
GATE 1      NOT REQUIRED — no model
GATE 2      the real runs above
CI          none configured (S13); the full local gate once on the final head
```

### 4.2.4 Is any of this material?

- The World Pack format extension (QS-5) and the reading of `Item` as a kind (QS-6) were approved at
  S9's freeze; ARC-36 records them. No public contract in `contracts/` changes, and `kernel/` is untouched
  (I-8).
- **QS-15 is operator-material**: it would amend how ARC-35 point 7 finds a merged precursor. If it is
  declined, nothing in 11b changes except B-C2's conditional item, and §12's fallback applies.
- QS-16 (word-level admission) tightens an approved check and is recorded as an ARC-35 note; it is not
  material, and is raised so the primary session sees it.
- Nothing in 11b changes ownership: no state, no System Pack, no fact.

## 4.3 PR 11c — complete affordances (full design; MERGED as `c5dc51c`)

### 4.3.1 Identity, base, approved scope

```text
PR            11c — complete affordances: a controller acts on what it is offered (S9, third of six;
              a precursor; resolves F-3)
base          main @ 7ed1648 plus this planning branch once merged, or the main the primary session
              names at freeze (re-audit §8.4 if contracts/, systems/presence/, cognition/ or
              tests/acceptance/ moved)
branch        mvp0/pr-11c-affordances, in its own worktree, held by the implementing session only;
              runs in parallel with 11b under §12
audit         §8 (b9e5937) and §8.4 (7ed1648)
scope         §1.1 PR 11c; SD-9 … SD-12 as refined by QS-20 … QS-22; F-3, F-24 … F-29, F-33, F-35;
              QS-4 (approved)
depends on    11a (merged). Independent of 11b.
```

**Goal.** An affordance may carry the complete request payload the offering system would accept (a
**complete affordance**), and any requester — a controller, a client — may submit it unchanged without
knowing the action. The paced controller gains exactly one band that attempts an available complete
affordance by a seeded draw at a fixed rate. Every existing observation, transcript and run is
unchanged, because no existing pack offers a complete affordance.

**Justified and proven without the market (I-2, I-9).** The proof is a synthetic test-only System Pack
`chimes`, defined inside one acceptance test file and compiled into no library, so the controller crate
has never been compiled against it:

```text
chimes   owns nothing it does not need; provides `ring { bell }`; states its own `rang { ringer, bell }`
         offers one complete `ring` per bell its belfry hangs (target none), with the requirement
         "at the belfry's place" — so the same offers are available in the belfry and unavailable,
         with the reason, anywhere else
         offers one incomplete action as well, so "only complete affordances are attempted" is observed
```

The band's constants are fixed in this PR against `chimes` (I-9), measured under worst-case exposure on
a scratch install that is never merged (C-C6). 11c adds **no** allow-list entry to the I-2 scan: nothing
it needs is a market word (F-35 lists the existing lines it must not rewrite).

**Non-goals.** `RuleController` / `--agent` unchanged (I-5, SD-11); no change to the rule-controller's
manifest; free-form actions (`talk`, `move`, `invite`) stay known by name (§2.3); no GDScript code
change (QS-20); no existing pack offers a complete affordance; no kernel change (I-8).

**Acceptance (decided before measuring, `ARC-23`).**

```text
C-1  Nothing existing moves (I-4, I-5). The 300-day seed-7 social-cafe run's sha (all but `wall`) =
     E-0; `validate worlds/social-cafe` byte-identical to the base's; every existing test passes,
     including AC-13 and AC-15 against their recorded transcripts. Existing-test edits are limited to
     type annotations forced by Affordance<P> (F-24) and, if the compiler's wording changes, the
     trybuild .stderr of the private-fields test — each listed with its unchanged claim.
C-2  The contract (ARC-34). An affordance without a payload serializes to the frozen literal captured
     on the base before any edit; a frame without the field decodes; a payload round-trips;
     Affordance::request builds exactly the request the affordance names (type, target, encoded
     payload); the availability/reason agreement is still refused.
     Mutation M-C1: drop `skip_serializing_if` → the frozen-literal test fails.
C-3  Presence carries it. Offer::complete(&action, requirement) reaches the observation with the
     action's own type and its payload, for available and unavailable verdicts; Offer::new carries
     none; a disabled pack's complete offers are absent (INV-10, AC-2).
     Mutation M-C2: verdict drops the payload → the presence test fails.
C-4  The band. Only available complete affordances are attempted; the submitted request equals the
     chosen affordance's; greetings still occur when complete affordances are offered; RuleController
     never attempts one.
     M-C3: ignore is_available → the availability test fails. M-C4: OFFER_DRAW = 0 (reuse the walking
     roll, F-28) → the coexistence test fails. M-C5: call the band from RuleController → its test fails.
C-5  CP-3 end to end. In a hand-built world (presence + chimes) driven by the paced schedule, people in
     the belfry ring every simulated day; people elsewhere never ring; every `rang` is caused by the
     request that asked for it (AC-9); two runs of one seed are byte-identical; with chimes disabled,
     no `ring` request is ever made.
     M-C6: remove the band's call from decide → the ring assertion fails.
C-6  I-9. ATTEMPTS_OFFERED, OFFER_DRAW = 14, OFFERED_CHOICE_DRAW = 15 and the band's position are
     fixed from C-C6's measurement and recorded; no later S9 PR changes them.
C-7  I-2 and I-8. The scan is green with 11c's row and no 11c allow-list entry; kernel/ diff empty;
     contracts/ diff is observation.rs (the field and its methods) plus one pub(crate) labelling
     constructor in action.rs (F-25), and its tests.
```

### C-C0 — Design (this section) — docs only

- [x] Implementation: §4.3, §8.4, §12, §14, QS-20 … QS-25 by the planning session.
- [x] Validation: both doc checks (§9 E-2).
- [x] Review: draw indices 14 and 15 confirmed free from source (F-27); the drafted adversarial (2)
  shown not to bite and replaced (F-28); operator-material points marked (§10).

### C-C1 — Specs before code: ARC-34, CORE_CONCEPTS §15.2, MODULE_SPEC §5, PROTOCOL, ADOPTION

**Scope.**
- `docs/DECISIONS.md` — **ARC-34** *Complete affordances: an offer may carry the request it would
  accept*, inserted **immediately before `## ARC-35`** (its reserved slot; §12): §2.3's options and
  choice; `Affordance<P>.payload`, serialized only when present; `Affordance::request(actor, encode)`;
  `Offer::complete`; the paced controller's band, its constants and position, fixed against a synthetic
  pack (I-9); RuleController unchanged; free-form actions stay known by name. Accepted limitations:
  the offerer must be able to enumerate the choices; payload size grows with offers (R-S9-2).
- `docs/CORE_CONCEPTS.md` §15.2 — the Affordance block gains `payload  the complete request, when the
  offering system can state one`, and one paragraph: a requester may submit it unchanged; the server
  still revalidates; a controller still cannot create an interaction (INV-10).
- `docs/MODULE_SPEC.md` §5 — hard constraint 1 gains a sentence: a Controller Pack may attempt a
  complete affordance it was offered; that is not creating an interaction.
- `server/PROTOCOL.md` §5 (`observation`) — the field and its meaning; §6 — submitting it: `request.
  payload = { "action_type": <affordance's>, "payload": <affordance's payload> }`, target the
  affordance's.
- `clients/protocol/ADOPTION.md` — `affordances()` carries `payload` on complete affordances; submit it
  with `submit(action_type, target, payload)`; `affordance(type, target)` returns only the first of
  several complete affordances sharing a type and target (F-29).

- [x] Implementation: as scoped; `git fetch` first and confirm ARC-34 is free everywhere. ARC-34
  inserted before `## ARC-35`; CORE_CONCEPTS §15.2 (field + one paragraph); MODULE_SPEC §5 constraint
  1; PROTOCOL §5 (field, example, "absent not null", entries sharing type and target) and §6
  (submitting it); ADOPTION.md §2 (`submit(action_type, target, payload)`, `affordance()` returns the
  first). No `.gd` file touched (QS-20). E-C1.
- [x] Validation: both doc checks (E-C1); cited sections (`CORE_CONCEPTS` §15.2, `MODULE_SPEC` §5,
  `PROTOCOL` §§5–6, `ARC-34`) exist; `submit`'s real signature read from `world_client.gd:155`.
- [x] Review: no defined term redefined ("complete affordance" is a qualified Affordance, defined in
  §15.2); PROTOCOL stays revision 1 — the field is optional and absent by default.

### C-C2 — `contracts/`: `Affordance<P>` gains its payload

**Scope.**
- `contracts/src/observation.rs`: `Affordance<P = Vec<u8>>` with a last field `payload: Option<P>`,
  `#[serde(default, skip_serializing_if = "Option::is_none")]`, serde `try_from = "AffordanceFields<P>"`
  with a deserialize bound; `available`/`unavailable` unchanged in signature (payload `None`);
  `#[must_use] fn with_payload(self, P) -> Self`; `fn payload(&self) -> Option<&P>`; `fn request<Q>(&self,
  actor: EntityId, encode: impl FnOnce(&P) -> Q) -> Option<ActionRequest<Q>>` — `None` without a payload,
  otherwise labelled with the affordance's own action type and targeted at its target; availability is
  the caller's judgement. `Observation<P>` holds, offers and returns `Affordance<P>`. Docs: what a
  complete affordance is (ARC-34).
- `contracts/src/action.rs`: `pub(crate) fn ActionRecord::labelled(action_type, payload)` (F-25).
- `contracts/tests/observation.rs`: new tests — `an_affordance_without_a_payload_is_the_shape_it_always
  _was` (the literal captured on the base **before** the edit and recorded in §9.3), `an_old_frame_
  decodes_and_a_payload_round_trips`, `a_complete_affordance_requests_exactly_what_it_offers`.
- Forced type annotations (F-24): presence `observe.rs` (`Vec<Affordance<Value>>`), conversation and
  group-activity tests, contracts' own `:203` if inference requires it, `spike/server/src/world.rs:350`
  (QS-23). Each listed with its unchanged claim.

- [x] Implementation: as scoped. `observation.rs`: `Affordance<P = Vec<u8>>`, last field `payload:
  Option<P>` with `skip_serializing_if`; the `#[serde(default)]` sits on `AffordanceFields<P>`'s field
  (the deserialize side, because of `try_from`) — a bounded placement detail; `with_payload`,
  `payload`, `request(actor, encode)`; `Observation<P>` holds/offers/returns `Affordance<P>`. `action.rs`:
  `pub(crate) const fn ActionRecord::labelled`. Tests: the three named tests plus test-only `Ring` /
  `Knock` actions. Forced annotations (F-24), each claim unchanged: presence `observe.rs`
  `affordances`/`verdict` return `Affordance<Value>` (lines 207/220 untouched); conversation test
  `:234` and group-activity test `:306` (`Affordance<serde_json::Value>`); contracts' own `:203`
  (`Affordance::<String>::available` — inference needed it); `spike/server/src/world.rs:350`
  (`Vec<Affordance<Value>>`, QS-23's one line). Rule-controller tests needed no edit (inferred inside
  `.offering`). E-C2.
- [x] Validation: E-C2 — the five suites pass; M-C1 fails three tests including the frozen-literal
  test, reverted; trybuild `.stderr` regenerated (wording changed: "cannot construct `Affordance<_>`
  with struct literal syntax due to private fields", plus a note naming `payload`) — claim unchanged,
  private fields refused; spike `cargo check --offline` PASS on base and after.
- [x] Review: additive only — no existing frame changes (the base literals are asserted); `request`
  labels with the affordance's own type and `labelled` is crate-private; the encoding is the caller's
  closure, none decided in contracts. Finding: my first `an_old_frame…` test built its expected text
  with `trim_end_matches('}')`, which strips both closing braces — a test defect, fixed to
  `strip_suffix`; it was one of the three M-C1 failures, so M-C1 is re-stated against the frozen-literal
  test alone, which failed under M-C1 for the intended reason.

### C-C3 — presence: `Offer::complete`

**Scope.** `systems/presence/src/interaction.rs`: `Offer` gains `payload: Option<Value>`;
`pub fn complete<A: Action + Serialize>(action: &A, requirement) -> Result<Self, serde_json::Error>` (the
type is read off the value, so a payload of another action cannot be attached — QS-22); `payload()`.
`observe.rs` `verdict`: the payload travels into the affordance whatever the verdict. Tests in
`systems/presence/tests/presence.rs` (a test pack as its `Mover` pattern): complete offer → affordance
with its type and payload, available in place and unavailable elsewhere; `Offer::new` → none; pack
disabled → absent.

- [x] Implementation (do not rewrite observe.rs:207 or :220 — F-35). `Offer` gains `payload:
  Option<Value>`; `Offer::complete<A: Action>(&A, requirement) -> Result<Offer, serde_json::Error>`
  (struct update over `Offer::new::<A>`, so the type is read off `A`); `Offer::payload()`. `verdict`
  computes the affordance as before and then attaches the offer's payload whatever the verdict. Tests:
  a `Bellringer` test pack (complete `test-toll { bell }` low/high at its belfry, plus one `Offer::new`),
  `a_complete_offer_reaches_the_observation_with_its_own_type_and_payload`,
  `a_disabled_packs_complete_offers_are_absent`. `Action` already implies `Serialize`, so the bound is
  `A: Action` (the design's `Action + Serialize` is redundant). E-C3.
- [x] Validation: E-C3 — presence 15 passed (13 + 2); M-C2 fails the payload test, reverted;
  conversation, group-activity, movement suites green; clippy `-D warnings` on presence and contracts
  clean; presence's own vocabulary scan still green.
- [x] Review: perception attaches the payload and decides nothing about it; the `INV-10` filter (`world
  .systems().provider(...)`) is the unchanged `continue` before `verdict`, so a disabled pack's complete
  offer never reaches `verdict` (shown by the disabled test).

### C-C4 — the paced controller's offer band

**Scope.** `cognition/rule-controller/src/offered.rs`: `ATTEMPTS_OFFERED = 20` (out of 100),
`OFFER_DRAW = 14`, `OFFERED_CHOICE_DRAW = 15`; `attempt(observation, draw)`: the available affordances
with a payload, in observation order, collected into a `Vec`; none → `None`; draw 14 ≥ rate → `None`;
else the one chosen by draw 15, through `Affordance::request` with `serde_json::to_vec`. `paced.rs`
`decide`: one call after the social initiative, before the walking roll; the module doc gains a table of
every draw index (0–6, 8–13, 14, 15; 7 free). `src/offered_tests.rs`:
`only_available_complete_affordances_are_attempted`, `the_request_is_exactly_the_offer`,
`greetings_still_happen_where_offers_are_made` (over SEEDS × instants, both greet and offer occur),
`incomplete_affordances_of_unknown_actions_are_never_attempted`, `no_complete_affordance_no_new_draw
_decides_anything`; `src/tests.rs`: `the_reactive_controller_never_attempts_an_offer`.
`Cargo.toml` of the crate is **not** edited.

- [x] Implementation (no line naming `shifted`, no `Iterator<Item = …>` — F-35). `offered.rs`:
  `ATTEMPTS_OFFERED = 20`, `OFFER_DRAW = 14`, `OFFERED_CHOICE_DRAW = 15`, `attempt(observation, &Draw)`
  as scoped (the rate check and the empty check share one `if`; draw 14 is read only when something
  complete is available). `paced.rs`: one call after `if social.is_some()`, before the door closure and
  the walking roll; module doc gains the draw table; `decide`'s doc names the band. `lib.rs`: `mod
  offered;` and `mod offered_tests;` only — `RuleController` untouched. The five named tests in
  `offered_tests.rs` (their observation is a hand-built hall offering `ring`, a name no dependency
  defines) and `tests.rs::the_reactive_controller_never_attempts_an_offer`. `Cargo.toml` not edited.
  E-C4.
- [x] Validation: E-C4 — crate 34 passed (28 + 6); M-C3, M-C4, M-C5 each fail by name, reverted; the
  300-day social-cafe sha at this state (see E-C4).
- [x] Review: `decide(&self, …)` still pure — the band reads only the observation and the `Draw`
  (ARC-27); `offered.rs` imports only contracts and serde_json, no pack type; draw indices 0–6, 8–13,
  14, 15 distinct (the table); the band is placed after the agenda and social bands, so it never
  pre-empts an answer, an invitation reply, the day's walk or a leave/invite/join.

### C-C5 — CP-3: `tests/acceptance/tests/complete_affordances.rs`

**Scope.** `tests/acceptance/Cargo.toml` gains `[dev-dependencies]` (all existing workspace entries:
contracts, kernel, presence, rule-controller, serde, serde_json) and its comment and `src/lib.rs` doc
say the crate now also holds S9's cross-crate checkpoints (QS-24). The test file defines `chimes`
(system, `Ring`, `Rang`, a `Belfry` with two bells at one place, its offers as above), builds a world
with `belfry` and `hall`, two people in each, presence + chimes, and drives it with
`PacedRuleController` on the `mineworld run` schedule (pace 900 s, seat k at genesis + k + m·P) for 10
days. Assertions C-5. A further assertion: `cognition/rule-controller/Cargo.toml` names no `chimes`.

- [x] Implementation. `tests/acceptance/Cargo.toml` `[dev-dependencies]` (the six workspace entries)
  and its comment; `src/lib.rs` doc names both tests. `tests/complete_affordances.rs`: `chimes`
  (depends on presence, provides `ring { bell }`, emits its own `rang { ringer, bell }`; `validate`
  applies the same `at_place(belfry)` requirement its offers declare, against presence's state, and
  refuses an unknown bell), two complete offers (`low`, `high`) plus one `Offer::new::<Ring>`, a world
  with `belfry` and `hall` and two people in each placed by genesis through presence's `arrival`, the
  `mineworld run` schedule (P = 900, seat k at genesis + k + m·P) for 10 days, seed 7. Four tests:
  `a_headless_person_rings_a_bell_the_controller_never_heard_of` (every belfry person rings every day;
  hall people request nothing; every `rang` has `Causation::Action` of an accepted `ring` request by
  the ringer for that bell — AC-9; every request is a ring and every ring is accepted),
  `two_runs_of_one_seed_are_byte_identical`, `with_chimes_disabled_no_ring_is_ever_requested`,
  `the_controller_was_never_compiled_against_chimes`. The I-2 scan gains 11c's row (base
  `da316134…`, no allow-list entry). Bounded deviation (recorded): the belfry is the pack value's
  configuration rather than an owned `Belfry` component — `chimes` "owns nothing it does not need",
  and two bells at one place need no state; the observable claims are unchanged. E-C5.
- [x] Validation: E-C5 — acceptance 4 + 3 passed; M-C6 fails, reverted; the scan's first run caught
  `item` in a C-C4 comment ("`ARC-35` item 6"), reworded to "point 6" (the scan working as built);
  planted violations refused by name, removed.
- [x] Review: the driver calls only what `run` calls — `advance_to`, `observe` with the providers,
  `decide`, `ActionIntent::allocate`, `dispatch` — in `run`'s order and schedule; `chimes` lives in a
  test file of a `publish = false` crate with no library code, so no library is compiled against it;
  `Cargo.lock` gains only acceptance's dependency list (six names, no new package).

### C-C6 — I-9 measured on a scratch install (evidence, never merged); close

- [x] Scratch branch `scratch/11c-chimes` (local, deleted after) — done, PASS at 20, E-C6: `systems/chimes/` as a real pack
  offering `ring { low | high }` complete to everyone, requirement none (worst-case exposure); two lines
  in `systems/installed/`; `worlds/chimes-cafe/` = social-cafe + `chimes`. Criterion, stated before the
  run: over 300 days seed 7, the step-08 I-9 activity precondition holds in every bucket (every seat moved
  and talked) **and** every seat's `ring` is accepted in every bucket (counted from the save by a
  scratch-only reader). If either fails at 20, lower `ATTEMPTS_OFFERED` on the branch, re-measure, record
  both runs; the value that passes is frozen (C-6). Also: social-cafe's sha on the scratch build = E-0.
  `git ls-remote --heads origin | grep -c scratch` → 0.
- [x] Documentation: `docs/MVP_STATUS.md` — one capability row directly after "Conversation", one
  evidence row appended after the table's last row (done; plus ARC-34's measurement note); `Updated:` line and S9 row not edited (§12); §4.3
  checkboxes, §9.3 `E-C*`, `handoff-11c.md`.
- [x] Full gates once on the final head (fmt, clippy -D warnings, workspace tests in background), C-1's
  sha and validate diff, both doc checks — E-C-final, all PASS.
- [x] Review: C-1 … C-7 each with evidence; deviations listed.
  - C-1 PASS — sha = E-0 and validate identical (E-C0, E-C4, E-C-final); every existing test passes,
    AC-13 and AC-15 included (`tools/cli/tests/ac15_one_alice.rs`, `server` parity tests, in the 443);
    existing-test edits are only the F-24 annotations and the trybuild `.stderr` (E-C2).
  - C-2 PASS — frozen literals captured on the base (E-C0) and asserted; old frame decodes; payload
    round-trips; `request` exact; disagreement refused; M-C1 fails (E-C2).
  - C-3 PASS — E-C3; M-C2 fails.
  - C-4 PASS — E-C4; M-C3, M-C4, M-C5 fail by name.
  - C-5 PASS — E-C5; M-C6 fails.
  - C-6 PASS — 20 / 14 / 15 and the position, fixed by E-C6's measurement (passed at 20) and recorded
    in ARC-34's note.
  - C-7 PASS — scan green with 11c's row and no allow-list entry; planted violations refused; kernel/
    empty; contracts/ as scoped (E-C5, E-C-final).
  - Deviations, all bounded: `#[serde(default)]` placed on `AffordanceFields<P>` (the `try_from`
    side) rather than on the struct field; `Offer::complete` bound is `A: Action` (it already implies
    `Serialize`); `chimes` in C-C5 configures its belfry in the pack value instead of owning a `Belfry`
    component; the C-C2 test bug (`trim_end_matches`) found and fixed; the scan caught `item` in a
    C-C4 comment, reworded. No material deviation; no stop condition reached.

**PR 11c lifecycle: MERGED** — GitHub #40, merge commit `c5dc51c` (2026-10-07), second, after a
rebase onto `ae1a315`, by a merge commit. Operator's review: gates re-run on the rebased head 456/0;
the operator's own mutation (`ATTEMPTS_OFFERED = 0`) failed the CP-3 ring test and the byte-identity
test; the contract change is additive; the worst-case `chimes` measurement passed at 20. Recorded by
the planning session. The record below is the implementing session's, as it stood at review:
`DESIGN FROZEN (2026-10-07)`, execution contract §14 confirmed. See the
freeze record for 11b and 11c in §12.0. **READY FOR OPERATOR REVIEW** — rebased onto main @
`ae1a315` after 11b merged (E-C-rebase); final executable head `6003e07`, gated there; the PR head is
the Markdown-only commit after it (the pre-rebase heads `20cf29b` / `c7b33df` are superseded).
Merge with a **merge commit** only (§12). Implementation context CLOSED / AWAITING OPERATOR ACTION.
Post-merge: this session owns §4.3 and §9.3; the planning session owns the header, §§1–3, overall
and MVP_STATUS's `Updated:` / S9 lines.

### 4.3.2 Test ownership for 11c

```text
STATIC      fmt, clippy; the type parameter (a payload of another encoding does not compile into an
            observation); Offer::complete makes a mislabelled payload unrepresentable
UNIT        contract serialization and request() (C-C2); presence's verdict (C-C3); the band (C-C4)
INTEGRATION CP-3 through real observe() and kernel dispatch with a synthetic pack (C-C5); every
            existing test, AC-13/AC-15 transcripts included (C-1)
REAL RUN    300-day social-cafe comparison; the scratch chimes install over 300 days (C-C6)
GATE 1      NOT REQUIRED
CI          none configured; the full local gate once on the final head
```

### 4.3.3 Is any of this material?

- The public contract change (`Affordance<P>.payload`) and the controller band were approved as QS-4.
  The refinements QS-21 (`request` takes the encoder) and QS-22 (`Offer::complete` instead of
  `with_payload::<A>`) keep that shape and are raised for the primary session, not as operator-material.
- QS-20 amends frozen SD-12 (no GDScript change) — a primary-session decision.
- No ownership boundary changes; `kernel/` untouched; the controller's dependencies unchanged.


## 4.4 PR 11d — the transformation, part 1: owning and giving things (full design; PROPOSED, NOT FROZEN)

### 4.4.1 Identity, base, approved scope

```text
PR            11d — owning and giving things (S9, fourth of six; the first half of the measured AC-1
              transformation, ARC-35 point 1)
base          main @ c5dc51c plus the docs-only planning merges (#41, and this branch once merged), or
              the main the primary session names at freeze. Re-audit §8.5 if anything under
              systems/, worlds/, authoring/, sdk/, worldpack/src/load.rs or
              cognition/rule-controller/src/offered.rs moved
branch        mvp0/pr-11d-owning-things, in its own worktree, held by the implementing session only
audit         §8.5 (c5dc51c), including the R-S9-1 spike (§9 E-4)
scope         §1.1 PR 11d; SD-13 as refined by SD-16 … SD-21; QS-7 (approved); QS-27 … QS-38 as
              answered
depends on    11a (installed set), 11b (items as content), 11c (complete affordances): all merged
```

**Goal.** People in a town own things and give them to each other. Three new System Packs —
`item` (what kinds of things exist), `inventory` (who holds how many of each, the only writer of
holdings) and `item-transfer` (`give`, which owns nothing and states inventory's fact) — are installed by
three lines each in `systems/installed`, and `worlds/market-town` is created as Social Café plus those
three packs and their content. Every headless person gives, through the unchanged paced controller's
offer band, because `give` is offered as complete affordances (`ARC-34`).

**The change set is inside AC-1 by construction (I-1).** Every path this PR may touch:

```text
systems/item/**                         new pack
systems/inventory/**                    new pack
systems/item-transfer/**                new pack
systems/installed/Cargo.toml            three dependency lines, by path
systems/installed/src/lib.rs            three installed! lines
systems/README.md                       the pack list (Markdown)
worlds/market-town/**                   new world
Cargo.lock                              three new path packages under systems/, and
                                        mineworld-installed-systems' dependency list; nothing else
docs/*.md                               DECISIONS (ARC-37), MODULE_SPEC §4.1, PACKAGE_FORMAT §8, MVP_STATUS
.structured-coding/plans/mvp0/*.md      this ledger, handoff
```

Why nothing else can be needed, from source and from the spike (§8.5, §9 E-4): the root manifest lists
`"systems/*"` (no member line); new packs name their new siblings by `path` and existing crates by
`workspace = true` entries that already exist (no root dependency line); `worldpack` reads the generated
`Capability` (no catalog arm); the controller attempts complete affordances it was never compiled
against (no controller edit); every test outside `systems/` that names installed systems asserts
`contains`, never an exact list (F-10, `refusals.rs:184`, `:790`). The spike made the same change on a
scratch branch and touched exactly `Cargo.lock`, `systems/{item,inventory,item-transfer,installed}/**`
and `worlds/market-town/**`, and the whole workspace suite passed unchanged. **A needed edit to any other
path is a material stop**, reported with evidence, and answered by a precursor PR justified without the
market (I-2), never by a quiet edit inside the range.

**Non-goals.** No `items-produced` (QS-28: 11e adds it with its first stater); no organization content
in market-town (11e's); no `buy`, money, jobs or shops; no consumption — nothing eats or drinks (QS-10,
QS-35); no item instances (`ARC-36`); no names or disclosure for item kinds (F-41); no client change
(QS-13); no world-level test in `tools/cli/tests` or `tests/acceptance` (11f's, outside the range); no
change to any existing pack, to `kernel/`, `contracts/`, `authoring/`, `sdk/`, `worldpack/`,
`cognition/`, `tools/`, `server/` or `persistence/`.

### 4.4.2 Design (SD-16 … SD-21)

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-16** | **`item` owns `ItemKind { category }` on Item entities**, from the `item:` section of an item file, `{ category: <slug> }` (1–32 bytes of `a–z`, `0–9`, `-`, its own `Category` type, refused at its line and column). Genesis fact `item-kind-declared { item, category }`, public, reduced by `item` alone. No action, no process, no dependency, no disclosure, nothing biographical. `pub fn is_declared(world, ItemId) -> bool` is the question other packs ask. | QS-7 keeps `item` as the vocabulary inventory, economy and employment share. A kind exists for the market only once `item` has declared it, so a file under `items/` with no `item:` section is an inert entity no pack trades (11b's B-5 property, kept). Items are never perceived (presence perceives places and people), so `ItemKind` is disclosed to nobody (F-41). |
| **SD-17** | **`inventory` owns `Holdings` on Persons and Organizations, and only it writes them.** `Holdings` is a list of `{ item: ItemId, count: u32 }` sorted by item, no zero entries (F-38: an `ItemId` is not a JSON map key, and snapshots and payloads are JSON). Section `holdings:` on people and organization files, `{ <item key>: <count ≥ 1> }`, references Items. Facts: `stocked { holder, item, count }` (genesis, visible to the holder) and `items-transferred { from, to, item, count }` (visible to its participants). Depends on `item` (system and Cargo). Discloses a holder's `Holdings` to that holder only (`INV-13`). Nothing biographical (QS-29). | `CLAUDE.md` §4 rule 1 and I-3. One writer; both facts are its vocabulary and its reductions. |
| **SD-18** | **The owner decides, three times, through one function (`ARC-26`).** `pub fn admit_transfer(world, from, to, item, count) -> Result<(), Rejection>` is the whole of what `Holdings` refuses about a transfer: count ≥ 1; `from ≠ to`; both living Persons or Organizations; `item` declared (`is_declared`); `from` holds at least `count`; `to` can take `count` (SD-19). It is asked by a deciding pack's `validate`, by the checked constructor `pub fn transfer(world, from, to, item, count) -> Result<Emission, Rejection>`, and again by inventory's own reduction, which on refusal writes nothing and fails with `KernelError::FactRefusedByOwner`. Seeding is the exception the source forces (F-37): a section is seeded against a world with no state yet, so `holdings:` is checked for kind (an Item) and count (≥ 1, within capacity) at seeding, and for a declared kind at reduction, which follows `item-kind-declared` in genesis order (`ARC-36` point 7). | Presence's `admit`/`arrival`/`react` pattern, unchanged. Holdings change only in inventory's reductions, whoever decided. |
| **SD-19** | **A person carries at most `PERSON_CAPACITY = 6` items, all kinds together; an organization is not bounded.** Inventory's rule, in `admit_transfer` and in seeding: a transfer that would take a person past it is refused `TargetUnavailable`; a person's authored holdings past it are refused at genesis naming the file. `pub fn can_take(world, holder, count) -> bool` answers it for an offer. | The spike found an **absorbing sink** (F-39): Otto, the one person no seat names, receives and never gives, and held 31 of 33 items by day 30, so gives fell from 739 in days 1–15 to 312 in days 16–30. A bound on what one person carries makes the sink finite: with 6, over 300 days every seat gave in every 30-day bucket (min 118) and Otto ends holding exactly 6. It is the remedy I-9 requires — in a pack, never in the controller — and it also bounds R-S9-2 (at most 6 kinds held, so at most 6 `give` offers per person nearby). Primary-session decision, operator-visible (QS-27). |
| **SD-20** | **`item-transfer` provides `give { item: ItemId, count: u32 }`, targeting a Person, and owns nothing.** Requirement: same place, within 3 000 mm, target available. It offers, to an observer who holds something and for each other living Person present, **one complete affordance per kind held, count 1**, in item order; target availability is `inventory::can_take(target, 1)`. Never offered to oneself (perception lists the observer among the present, F-40). `validate`: payload, a living Person actor and a different living Person target (`NoSupportedInteraction` otherwise), the requirement through `SpatialRequirement::evaluate` against presence's positions, then `admit_transfer`. `resolve`: `inventory::transfer`, nothing else. Depends on `inventory` and `presence` (system and Cargo). | `ARC-34`: the offerer can enumerate the choices, so the paced controller gives without knowing `give`. `CORE_CONCEPTS.md` §6.3 already names "give item" as an action with a spatial requirement. AC-2 for this pack: disabled, `give` is answered `Unavailable` and offered nowhere, and holdings never change (INV-10). |
| **SD-21** | **`market-town` is Social Café plus the three packs and their content.** Every file of `worlds/social-cafe` copied byte for byte, then: `world.yaml`'s `id`/`name` and header comment, `item, inventory, item-transfer` appended to `systems`, an `items:` list; `items/<key>.yaml` for ~20 kinds (MVP §3), each with `tags` and `item: { category }`; a `holdings:` block appended to each person file (Otto included), 1–4 items each, ~30 in all; `README.md` (human orientation). No place file and no existing field or section changes (`ARC-35` check 3). | CP-1's world delta is configuration only. Otto is kept because the sink is a property of real worlds — a person nobody drives — and the pack must hold under it, not the content around it. |

### 4.4.3 Acceptance (decided before measuring, `ARC-23`)

Each guard names the mutation shown to break it. A mutation is applied to the working tree, observed to
fail by name, and reverted; `git status` is recorded afterwards.

```text
D-1  The change set is inside AC-1 (I-1). `git diff --name-only <base>...HEAD` lists only the paths of
     §4.4.1's table; Cargo.lock gains exactly three [[package]] entries, none with a `source`, all under
     systems/, and changes otherwise only mineworld-installed-systems' dependency list; no root
     Cargo.toml, worldpack, cognition, tools, kernel or contracts path appears.
     Guard: the recorded commands of D-C7 (the instrument that enforces this permanently is 11f's AC-1
     test). Mutation M-D1: a comment added to kernel/src/lib.rs in the working tree → the recorded
     command lists `kernel/src/lib.rs` (the instrument sees an outside path), reverted.
D-2  Nothing existing moves (I-4). 300-day seed-7 social-cafe sha (all but `wall`) = E-0
     (ad49c723…c64b); `validate worlds/social-cafe` identical to the base's; every existing test passes
     and no existing test is edited.
D-3  Item kinds are declared by their owner. An item file's `item:` section becomes one public
     `item-kind-declared` that `item` alone reduces into ItemKind; a bad category is refused at its line
     and column with item's message; `item:` is carried by item files only (CARRIED_BY; the loader's
     NotCarriedHere refusal itself is 11b's, already guarded in worldpack — a pack test cannot reach
     the loader, which depends on the installed set and so on the pack). Guards: systems/item/tests.
     Mutation M-D2 (run in D-C3, it needs inventory): item's reducer writes nothing → inventory's
     genesis `stocked` is refused by its owner (genesis fails with FactRefusedByOwner naming
     inventory) — the declared-kind check is real.
D-4  Only inventory writes Holdings, and it still decides (I-3, ARC-26, CP-5). A `stocked` or
     `items-transferred` stated past the checked constructor — more than the giver holds, an undeclared
     kind, a count of 0, to oneself — is refused with FactRefusedByOwner and writes nothing.
     Guards: systems/inventory/tests. Mutation M-D3: the reduction skips `admit_transfer` → the
     over-transfer test fails (holdings go wrong instead of the refusal).
D-5  A person carries at most six (SD-19). The constructor refuses a transfer past it
     TargetUnavailable; `give` to a full person is offered unavailable with TargetUnavailable and is
     refused the same at dispatch; authored holdings past six are refused by inventory's seed (which
     the loader reports as SectionRefusedByOwner naming the file — shown once through the real CLI on
     a scratch copy in D-C6); an organization holding more is accepted.
     Guards: inventory and item-transfer tests. Mutation M-D4: `can_take` always true → the three
     capacity tests fail.
D-6  Giving works through the unchanged controller (CP-3 for a real pack). In a hand-built world
     (presence, movement, item, inventory, item-transfer; two people within reach, one out of reach),
     an observer holding two kinds is offered exactly two complete `give`s per other person present —
     available within 3 m, TooFarAway beyond — and none to itself; an accepted give moves one item and
     the fact is caused by the request (AC-9); driven by PacedRuleController on `mineworld run`'s
     schedule for 10 days, people give; two runs of one seed are byte-identical; the rule-controller's
     manifest names no market pack. Guards: systems/item-transfer/tests. Mutation M-D5: offers built
     with Offer::new (incomplete) → the controller-gives test fails.
D-7  AC-2 for item-transfer, at pack level (CP-6's pack half). The same world without item-transfer
     loads and runs: no `give` affordance in any observation, a `give` request is answered Unavailable,
     holdings never change, and every other pack's facts are those of the run with it minus the gives'.
     A world enabling item-transfer without inventory is refused by the registry naming inventory.
     Guards: systems/item-transfer/tests. Mutation M-D6 (negative control): the test's world enables
     item-transfer → the "no give affordance" assertion fails.
D-8  Holdings survive a restart. A saved world with holdings and transfers, resumed, has the same
     Holdings and continues identically (the schedule/persisted.rs pattern). Guard:
     systems/inventory/tests/persisted.rs. Mutation: N/A — the property is persistence's (ARC-25) and
     already guarded there; this test is a regression that inventory's state is JSON-snapshottable
     (F-38), which the compiler cannot show. Shown red instead by the F-38 counter-example recorded in
     D-C3 (a BTreeMap<ItemId, u32> component fails to snapshot), if it reproduces.
D-9  market-town is valid and lives, bounded, without consumption (QS-10, I-7, I-9, R-S9-2) — ledger
     evidence, never a test inside the range (world-level tests are 11f's). In order:
       a. `mineworld validate worlds/market-town` → valid; ids 1–18 exactly social-cafe's; the item
          kinds after them; genesis = social-cafe's 53 + one per kind + one per authored holding.
       b. Activity first (I-7): `run --seed 7 --days 300 --save` → faults 0; every seat moved and
          talked in every 30-day bucket; and, from the save by a scratch reader (never committed),
          every seat gave at least once in every bucket and no person ever holds more than six.
       c. Only then determinism: two 30-day seed-7 runs print identical lines but `wall`; the run
          stopped at day 15 and resumed to 30 equals the uninterrupted 30-day run's history and
          fingerprint.
       d. Cost: the 300-day run's wall ≤ 60 s with --save (R-S9-2).
     Mutation M-D7 (on a scratch branch, never pushed): PERSON_CAPACITY = u32::MAX → b fails: gives
     collapse into the unseated person (the instrument sees the sink, as the spike's first run did).
D-10 The world delta is configuration only (ARC-35 check 3, by hand until 11f automates it):
     `git diff --no-index worlds/social-cafe worlds/market-town` shows only world.yaml's id, name,
     header, appended systems and items list; added items/ files and README; and one appended
     `holdings:` block (with its comment) per person file.
D-11 The documents say it first (CLAUDE.md §2.2): ARC-37, MODULE_SPEC §4.1's section table,
     PACKAGE_FORMAT §8 and systems/README exist before the code; both doc checks pass.
```

### D-C0 — Design (this section) — docs only

- [x] Implementation: §4.4, §8.5, §9 E-3/E-4, QS-27 … QS-38, §15, by the planning session on
  `mvp0/s9-11d-plan`.
- [x] Validation: both doc checks (§9 E-5).
- [x] Review: every file and symbol named here was read on `c5dc51c` (§8.5); the riskiest cross-pack
  flow was run end to end on a scratch branch before this design was finalized (E-4); the change set
  is shown inside AC-1 by that run; operator-material points are marked (§10). Self-review by the
  planning session only; the primary session's review is pending.

### D-C1 — Specs before code: ARC-37, MODULE_SPEC §4.1, PACKAGE_FORMAT §8, systems/README

**Goal.** The three packs' ownership, the capacity rule and how the market stays alive without
consumption are reviewable before code (`CLAUDE.md` §2.2). All Markdown: inside the range.

**Scope.**
- `docs/DECISIONS.md` — **ARC-37** *Owning and giving: kinds, holdings, give, and a person's
  capacity*, appended at the end (after ARC-36; `git fetch` and confirm ARC-37 is free first). SD-16 …
  SD-20; options for the sink (capacity; consent to receive; giving only to the controlled — impossible,
  `INV-1`; content only — cannot stop receiving); limitations: no consumption, no item names, no
  instances, organizations unbounded; the forward implication for 11e (QS-35).
- `docs/MODULE_SPEC.md` §4.1 — "MVP-0 has two sections" becomes four: `item` (item, items),
  `holdings` (inventory, people and organizations).
- `docs/PACKAGE_FORMAT.md` §8 — the sections parenthesis names the four.
- `systems/README.md` — the three packs in its list.

- [ ] Implementation: as scoped.
- [ ] Validation: both doc checks; ARC-37 absent from every `origin/*` branch.
- [ ] Review: no defined term redefined (`Item` stays `ARC-36`'s kind; "holdings" is inventory's
  component, not a core term); the section table matches SD-16/SD-17 exactly.

### D-C2 — `systems/item`

**Scope.** `systems/item/{Cargo.toml, README.md, src/{lib,system,section,event,component,category,
codec}.rs, tests/item.rs}`, laid out as `naming`. Dependencies: authoring, contracts, kernel, presence,
sdk, serde, serde_json, thiserror (`workspace = true`); dev: serde-saphyr (decode a section as the
loader does), as naming's tests.

- [ ] Implementation: `ItemSystem` (`impl SystemPack { owns_section!(); }`, empty
  `PerceptionProvider`), `Category`, `ItemKind` (`owned_component!`, `item-kind`), `ItemKindDeclared`,
  `AuthoredSection` (`item`, carried by items), `is_declared`.
- [ ] Validation: `cargo test -p mineworld-item`; clippy `-D warnings`. Tests: the section seeds one
  fact reduced into ItemKind; bad categories refused with the message; `is_declared` false for an Item
  entity without the section and for a non-item. M-D2 is run in D-C3 (it needs inventory).
- [ ] Review: no dependency on any market pack; no disclosure; owns exactly one component.

### D-C3 — `systems/inventory`

**Scope.** `systems/inventory/{Cargo.toml, README.md, src/{lib,system,section,event,component,admit,
codec}.rs, tests/{inventory.rs, persisted.rs, support/mod.rs}}`. `mineworld-item = { path = "../item" }`;
dev: persistence, serde-saphyr.

- [ ] Implementation: SD-17 … SD-19 — `Holdings`/`Held`, `PERSON_CAPACITY`, `can_take`,
  `admit_transfer`, `transfer`, `Stocked`, `ItemsTransferred` (accessors), `holdings:` section
  (`references` = Items), reductions, disclosure to the holder.
- [ ] Validation: `cargo test -p mineworld-inventory`; clippy. Tests: genesis stocking (people and an
  organization); D-4's four direct-statement refusals; D-5's capacity refusals at genesis and in the
  constructor; disclosure only to the holder; persisted restart (D-8). Mutations M-D2, M-D3, M-D4
  (constructor half). F-38's counter-example recorded if it reproduces.
- [ ] Review: the one write path per fact; `admit_transfer` is the only refusal logic; no `f32`/`f64`;
  every map a `BTreeMap` or a sorted `Vec`.

### D-C4 — `systems/item-transfer`

**Scope.** `systems/item-transfer/{Cargo.toml, README.md, src/{lib,system,action,offer,codec}.rs,
tests/{give.rs, removable.rs, paced.rs, support/mod.rs}}`. `mineworld-inventory = { path =
"../inventory" }`; dev: `mineworld-item` by path, movement, rule-controller (`workspace = true`, for
D-6's controller test — a dev-dependency from a market pack to the controller, the direction ARC-35
check 2 allows).

- [ ] Implementation: SD-20.
- [ ] Validation: `cargo test -p mineworld-item-transfer`; clippy. D-5 (offer/dispatch half), D-6,
  D-7; mutations M-D4 (offer half), M-D5, M-D6.
- [ ] Review: owns no component; emits only inventory's fact, declared and depended on (the kernel
  refuses it otherwise); offers never include the observer; complete offers only for kinds held.

### D-C5 — Install: three lines each in `systems/installed`

- [ ] Implementation: `systems/installed/Cargo.toml` three `path` lines; `src/lib.rs` `Item`,
  `Inventory`, `ItemTransfer` after `Schedule`; `Cargo.lock` regenerated by the build.
- [ ] Validation: `cargo test -p mineworld-installed-systems -p mineworld-worldpack` (the consistency
  guard and the section-namespace guard now cover `item` and `holdings`); D-2's sha and validate;
  `git diff -- Cargo.lock` matches D-1's rule.
- [ ] Review: the two files gain exactly those lines; no root manifest edit was needed.

### D-C6 — `worlds/market-town`

- [ ] Implementation: SD-21 — copy with `git read-tree --prefix=worlds/market-town/ -u
  HEAD:worlds/social-cafe` then edit; items; holdings; README.
- [ ] Validation: D-9 a–d, D-10, with the scratch reader on a local scratch branch (deleted after;
  `git ls-remote --heads origin | grep -c scratch` → 0); M-D7 on that branch; a scratch copy with one
  person authored at seven items → `validate` refuses it naming the file and inventory (D-5).
- [ ] Review: no social-cafe field or section changed in the copy; Otto present with holdings; every
  authored person within capacity.

### D-C7 — Close: status, change set, full gates, ledger

- [ ] Documentation: `docs/MVP_STATUS.md` — a capability row ("Owning and giving things") and an
  evidence row (`Updated:` line and S9 row stay the planning session's); §4.4 checkboxes; §9.4 `E-D*`;
  the handoff.
- [ ] Validation, once, on the final executable head: fmt, clippy `--workspace --all-targets
  --all-features -D warnings`, `cargo test --workspace --no-fail-fast` (background), kill_and_resume,
  both doc checks, D-2's sha and validate diff; D-1: `git diff --name-only <base>...HEAD`, `git diff
  <base>...HEAD -- Cargo.lock`, M-D1.
- [ ] Review: D-1 … D-11 each with evidence; deviations listed; the PR is to be merged **with a merge
  commit** (ARC-35 point 1 reads `M^1..M`).

### 4.4.4 Test ownership for 11d

```text
STATIC      fmt, clippy -D warnings; the installed! bounds (a listed pack that is not SystemPack,
            Default and PerceptionProvider does not compile); INV-7 write tokens (item-transfer cannot
            write Holdings — it has no token)
UNIT        item: category and section; inventory: admit_transfer's refusals, capacity, the sorted
            Holdings (D-3 … D-5)
INTEGRATION hand-built worlds through the kernel's real dispatch and presence's real observe(): the
            owner's refusals of facts stated directly (D-4); give offered, validated, resolved,
            reduced (D-6); the unchanged PacedRuleController giving (D-6); AC-2 (D-7); a persisted
            restart (D-8); the loader over the real market-town (D-9a); every existing test (D-2)
REAL RUN    validate, the 300-day seed-7 market-town run with its scratch reader, two 30-day runs, a
            15+15 resume, the 300-day social-cafe comparison (D-2, D-9)
GATE 1      NOT REQUIRED — no model
GATE 2      the real runs above
CI          none configured (S13); the full local gate once on the final head
```

### 4.4.5 Is any of this material?

- **No framework change is needed.** The spike (E-4) ran the riskiest flow — a person giving an item
  offered as a complete affordance, attempted by the unchanged paced controller, validated by
  item-transfer, stated through inventory's checked constructor and reduced by inventory alone, seeded
  from an item file's section and a person file's section, saved and resumed — with no edit outside
  `systems/**`, `worlds/**` and `Cargo.lock`. No precursor PR is proposed.
- **New ownership, already approved.** `ItemKind` (item) and `Holdings` (inventory) are the boundaries
  QS-7 approved; no existing ownership changes.
- **A new rule, the capacity (SD-19, QS-27)** — a pack decision under I-9, raised because it is part of
  how QS-10 is answered.
- **Two refinements of the medium scope** — `items-produced` deferred to 11e (QS-28), holdings as a
  sorted list (F-38) — are bounded, and raised so the primary session sees them.
- **Forward, operator-material (QS-35):** without consumption, 11e's purchases will fill people to
  capacity and stop. 11e's design must answer it; deciding how is the operator's.
- Nothing changes a public contract, `kernel/`, `contracts/`, or a frozen invariant.

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
| 11c | a synthetic pack's action, unknown to the controller, is attempted and accepted headless; social-cafe byte-identical | the band taking an unavailable affordance fails a named test; the band reusing the walking roll's draw fails the greeting-coexistence test (F-28 — the social-cafe comparison cannot see it) |
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
FLAGGED  QS-15 — operator-material (amends ARC-35 point 7's merged-range detection); QS-20 amends
         frozen SD-12 (primary session)
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

## 8.4 Re-audit for 11b and 11c (`main @ 7ed1648`, 2026-10-07)

Made by the planning session after 11a merged, before detailing §4.2 and §4.3. Nothing below was
inferred from §1.1's medium scope; where that scope and the source disagree, the source wins and the
finding says so.

**Inspected.**

```text
authoring/src/{section.rs, content.rs, lib.rs} (whole)
worldpack/src/{format.rs, content.rs, read.rs, load.rs, lib.rs} (whole); catalog.rs (:60–129, the
  section-namespace guard); error.rs (:1–60, Declared, ContentKind re-export)
worldpack/tests/refusals.rs (:1–100 Fixture, :450–520 every_refusal_names_the_file_it_is_about);
  social_cafe.rs (:193–226 genesis count 53, :337–392 sections do not move earlier facts)
sdk/rust (Cargo.toml, src/{lib,pack,section,installed}.rs); systems/installed (lib.rs, Cargo.toml)
tests/acceptance/tests/precursor_vocabulary.rs (whole)
contracts/src/observation.rs (whole); contracts/src/action.rs (:330–530, ActionRecord, ActionRequest);
  contracts/tests/observation.rs (:69–96 scene, :202–277 shape tests); contracts/tests/compile_fail/
  affordance_cannot_claim_both_availability_and_a_reason.{rs,stderr}
systems/presence/src/{interaction.rs (whole), observe.rs (:1–80, :180–281)}; systems/presence/tests/
  presence.rs (:690–770, a test system that provides an action)
cognition/rule-controller/{Cargo.toml, src/paced.rs (whole), src/social.rs (:60–156), src/agenda.rs
  (draw constant), src/lib.rs (module layout), src/paced_tests.rs (:1–60, test list)}
tools/cli/src/{main.rs (:240–275 validate), run.rs (:1–200 the run driver)};
  tools/cli/tests/{commands.rs (:20–60), headless/mod.rs (:1–60, :160–260 Tables), run_restart.rs
  (:1–60)}
server/src/protocol.rs (:55–110 WireObservation, WirePayload); server/PROTOCOL.md (§§5–6)
clients/protocol/mineworld/{observation.gd (:140–202), world_client.gd (:155–172)}; ADOPTION.md (grep)
spike/server/{Cargo.toml, src/world.rs (:340–401)}
kernel/src/world.rs (:571–590, create_entity / create_authored_entity)
docs: DECISIONS ARC-26, ARC-27, ARC-28, ARC-31, ARC-33, DEP-12, ARC-35; MODULE_SPEC §§3, 3.1, 4, 4.1, 5;
  PACKAGE_FORMAT §8; CORE_CONCEPTS §§7, 8, 15; MVP_STATUS (table anchors)
git: every origin/* branch for ARC-34+, ARC-36+ and DEP-13+ (none); merge-commit history of origin/main
  (main-into-branch merges are this project's practice: 11e47f5 into mvp0/pr-05b-server)
run: the 300-day seed-7 social-cafe baseline on 7ed1648 (§9 E-1)
```

**Findings.**

```text
F-20  Seeding needs no typed lookups. §1.1's "Seeding gains item(key) and organization(key)" is stale:
      Seeding::resolve(key, entity_type) is already generic over EntityType (authoring/src/section.rs).
      11b adds nothing to Seeding.
F-21  read.rs check_sections refuses every reference to an Item or an Organization: its type match is
      `Place => places, Person => people, _ => false`. 11b replaces it with one key → entity-type map
      built once from all four declared lists, which is also what makes keys one namespace (F-16).
F-22  No installed pack can carry a section on an item or organization file in 11b, and none should
      (I-2): worldpack decodes sections only through the generated Capability. So the genesis order of
      item and organization sections and the typing of references to them cannot be observed through a
      real pack before 11d. They are observable through worldpack's own unit tests with a probe owner
      that is not installed, decoded with authoring's Decode and attributed to an installed Capability
      for ranking only — the pattern load.rs's `a_section_that_seeds_another_packs_fact_is_refused_by_name`
      already uses (its `Trespasser`).
F-23  refusals.rs's Fixture creates only people/ and places/, and Fixture::write does not create a
      parent directory. 11b's fixtures create items/ and organizations/ themselves; no existing helper's
      behaviour changes for an existing test.
F-24  Affordance is not generic: Observation<P> holds Vec<Affordance>. Making it Affordance<P> (SD-9)
      changes the type in four places outside contracts that name it in type position against an
      Observation<Value>: systems/presence/src/observe.rs (affordances, verdict — production),
      systems/conversation/tests/conversation_and_presence.rs:234, systems/group-activity/tests/
      group_activity.rs:306 (tests — a type annotation, claim unchanged), and spike/server/src/world.rs
      :350 (a separate workspace that consumes contracts by path). Call sites inside `.offering(vec![…])`
      infer the parameter and need no edit. The trybuild expectation affordance_cannot_claim_both_
      availability_and_a_reason.stderr may change wording once the struct has a sixth private field.
F-25  An ActionRecord can be labelled only from a type `A: Action` (action.rs:357). Affordance::request
      must label a record with the affordance's own ActionTypeId, so contracts needs a pub(crate)
      labelling constructor in action.rs. No trust is lost: ActionRecord already derives Deserialize, so
      any label is constructible from JSON today, and dispatch decodes with the owner's type.
F-26  Observations carry serde_json::Value payloads (presence observe, WireObservation); the paced
      controller returns ActionRequest<Vec<u8>> of JSON bytes (paced.rs `record`). So a complete
      affordance's payload must be re-encoded on its way into a request: request() takes the encoder.
F-27  Draw indices in use by PacedRuleController: 0 (the walking roll), 1 (answer), 2, 3 (greet), 4
      (approach), 5, 6 (wander) in paced.rs; 8 ANSWER_DRAW, 9 INITIATIVE_DRAW, 10 INVITEE_DRAW, 11
      KIND_DRAW, 12 JOINED_DRAW in social.rs; 13 AGENDA_DRAW in agenda.rs. The door choice is a separate
      mix (seed ^ "door", observer, instant ÷ 21 600), not a Draw index. Free: 7, 14, 15 — so 14 and 15
      are free, as SD-10 assumed.
F-28  §4.3's adversarial (2) as drafted would not bite. In social-cafe no pack offers a complete
      affordance, so the band never returns a decision whatever index it draws, and the social-cafe byte
      comparison stays green under "the band draws index 0". Index reuse shows only where a complete
      affordance is offered: with index 0, the band fires exactly when the walking roll is below 20,
      which is the greeting band, so greetings vanish. §4.3 replaces the check with a coexistence test.
      (The converse — an existing band's index changed — is caught by the social-cafe comparison.)
F-29  The Godot module's affordance(action_type, target) returns the first match (observation.gd:154).
      Complete affordances routinely share an action type and a target (one per offered choice, target
      none), so SD-12's payload(action_type, target) would be ambiguous by construction. affordances()
      already returns every affordance as the raw dictionary, payload included once it is serialized.
F-30  The I-2 scan finds a precursor's merge with `git log --first-parent --merges HEAD`. When two
      precursors run in parallel and the second integrates main by a merge — this project's practice
      (11e47f5) — the first one's `Merge pull request` commit is reachable from the second's branch only
      through a second parent, so on that branch the first precursor reads as unmerged: its range becomes
      base → working tree, which then holds the second PR's lines. Harmless only if the second PR adds no
      market word at all; fatal if the second is 11b (its `item` lines would be scanned under 11c's row,
      whose allow-list does not admit them). Once both are merged to main the first-parent chain holds
      both merges and the problem disappears. §12 and QS-15.
F-31  An allow-list entry admits a whole line: market_word returns the first match only, and an entry
      whose substring the line contains admits the line. A line `item_price` admitted for `item` would
      hide `price`. 11b needs the word `item`, so this must be closed before 11b relies on the allow-list
      (§4.2 C2, QS-16).
F-32  `mineworld validate` prints the allocated ids from loaded.ids() (main.rs:262–267), so item and
      organization keys appear in the id list without a CLI change; the summary lines (`places`,
      `people`, `seats`) are explicit and would not list them.
F-33  The rule-controller's tests are in-crate (src/*_tests.rs) and the crate has no dev-dependencies.
      An end-to-end check through presence's real observe() and the kernel's real dispatch needs the
      kernel and presence; tests/acceptance already exists as the acceptance home (11a C4b).
F-34  ARC-34 and ARC-36 (reserved by §§2.3–2.4) are free on every origin/* branch after `git fetch`.
F-35  Existing lines in files 11c edits already contain market-prefixed words, which the scan would
      refuse if 11c re-added them: presence observe.rs:207 ("employs") and :220 ("priced");
      rule-controller paced.rs `shifted` (:333, :413–414, :426) and `Iterator<Item = …>` (:346). 11c
      leaves those lines untouched and writes its own code without such words, so it needs no
      allow-list entry.
```

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
E-A-final on 70b0857 (clean tree), 2026-10-07:
     cargo fmt --all --check                                         PASS
     cargo clippy --workspace --all-targets --all-features -D warnings PASS
     cargo test --workspace --no-fail-fast    428 passed, 0 failed, 0 ignored (425 at C4 + the scan's
                                              3); 1 926 s wall — the machine was shared with another
                                              session's Godot renders and three concurrent 300-day
                                              runs; C4's identical suite took 174 s
     kill_and_resume                          cafe PASS, clock PASS
     check_decision_ids                       45 ids, all distinct
     check_doc_headings                       143 sections, none duplicated
     A-1                                      300-day seed-7 run, sha-256 (all but wall) =
                                              ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
                                              = E-0; wall 12.2 s
     CI: none configured (S13).

--- planning 11b and 11c (branch mvp0/s9-11bc-plan, base main @ 7ed1648) ---

E-1  I-4 baseline after 11a, on 7ed1648 (debug, opt-level 1), 2026-10-07:
     `mineworld run worlds/social-cafe --headless --seed 7 --days 300` → exit 0, 339 lines, faults 0,
     365 330 facts, sha-256 of all but `wall` =
     ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-0; wall 12.6 s.
     §8.4's audit; no other code run.
E-2  End of this planning branch: check_doc_headings → 143 numbered sections across 22 documents, none
     duplicated; check_decision_ids → 45 ids, all distinct (ARC-34, ARC-36 still proposals in this file
     only). Docs-only branch; no cargo gate run.

--- after 11b and 11c merged; planning 11d (branch mvp0/s9-11d-plan, base main @ c5dc51c) ---

E-3  §12.0 "After both merge", confirmed on c5dc51c (debug, opt-level 1), 2026-10-07:
     `cargo test -p mineworld-acceptance` → precursor_vocabulary 4 passed, complete_affordances 4
     passed, 0 failed: the I-2 scan is green on main with rows 11a, 11b, 11c, all three read as merged
     on the first-parent chain (c472636, ae1a315, c5dc51c).
     `mineworld run worlds/social-cafe --headless --seed 7 --days 300` → exit 0, 339 lines, faults 0,
     365 330 facts, fingerprint 59339a9c281829c9, sha-256 of all but `wall` =
     ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-0; wall 12.2 s.
     This is 11d's I-4 baseline.
```

Each implementation PR records its evidence in its own section — §9.2 for 11b, §9.3 for 11c — so that
two sessions writing at once never append to the same block (§12).

## 9.1 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-S9-1** | A transformation PR finds it needs a framework change (a kernel API, a presence hook, a contract field). Inside 11d/11e that fails AC-1 by construction. | Each transformation design begins with a throwaway spike of its riskiest cross-pack flow on the post-precursor `main`. A gap found there becomes a precursor PR, justified without the market (I-2), or a material stop. Never a quiet edit inside the range. |
| **R-S9-2** | Complete affordances multiply offers per consult (one per kind held, per person nearby, per priced kind); run time grows, and the pace cannot be raised (F-19). | Measured in 11d and 11e on 30-day runs before 11f. If one 300-day market-town run exceeds ~60 s, the packs offer less (e.g. give only kinds held, to people within reach — already scoped), never a CLI change. |
| **R-S9-3** | The closed money loop drains: customers spend endowments, employers cannot pay wages, CP-4's buckets go empty by month N. | Content in 11e (endowments, prices, wages, production rates) is sized from a 300-day measurement before 11f; I-9 forbids retuning the controller. If no content makes the market live, that is a finding about the packs' design, reported, not hidden. |
| **R-S9-4** | A declarative macro that generates the catalog is harder to read and to debug than the hand-written enum. | The macro is one file, documented method by method; its expansion is exercised by every existing test (A-1). |
| **R-S9-5** | New content kinds or genesis order perturb social-cafe. | I-4: byte comparison against E-0 in every PR. |
| **R-S9-6** | `Cargo.lock` churn unrelated to the market (a `cargo update`) lands inside a transformation PR. | AC-1's `Cargo.lock` rule fails on any changed non-path package; a transformation PR never runs `cargo update`. |
| **R-S9-7** | 11b and 11c run in parallel and collide: in shared files, or in the I-2 scan's view of what each added (F-30). | §12: an owner and an anchor for every shared file; the second PR to merge integrates main and moves its own scan row's base in the same commit; merge commits only. |
| **R-S9-8** | Making `Affordance` generic (11c) ripples through every crate that names it (F-24). | The four places are known before implementation; any further one the compiler finds is a type annotation, recorded with its unchanged claim. A change of behaviour anywhere is a stop. |

## 9.2 Evidence — PR 11b

Written by the 11b implementation session only (`E-B<n>`).

```text
--- PR 11b (branch mvp0/pr-11b-content-kinds, base main @ da31613) ---

E-B1 B-C1 (docs): check_decision_ids 46 ids distinct (ARC-36 new); check_doc_headings 143 sections /
     22 documents, none duplicated. ARC-36 free on every origin/* branch after git fetch.
E-B2 B-C2: cargo test -p mineworld-acceptance 4 passed; clippy -D warnings and fmt clean. M-B6 (word
     check bypassed) → an_admitted_word_admits_no_other fails on `let item_price = 1;` (expected
     ["price"], got []); reverted. Literal update D-B1 (market_word → market_words), claim unchanged.
E-B3 B-C3: worldpack + authoring + acceptance all pass (refusals 38, social_cafe 15, structure 2,
     worldpack unit 2, acceptance 4). The scan, without entries, refused 109 hits, only item/items, only
     in the seven listed files; with them it is green. M-B2 → the place-and-item duplicate test fails;
     reverted. clippy -D warnings, fmt clean. D-B2 (message article), D-B3 (manifest prose left).
E-B4 B-C4: worldpack + acceptance all pass (unit 4, content_kinds 1, refusals 38, social_cafe 15
     unedited, structure 2, doc 1, acceptance 4). M-B1 → content_kinds fails (lantern 1 … alice 5 vs
     cafe 1 … alice 3); M-B3 → probe order test fails; M-B4 → item-reference test fails; each reverted,
     `git grep MUTATION` empty. clippy (forced) and fmt clean. D-B4 (fixture locations).
E-B5 B-C5: scan 4, cli commands 4 (unedited), cli content_kinds 3 — pass. validate worlds/social-cafe
     identical to da31613's (base binary built in a scratch worktree; diff empty). M-B5 → the validate
     test fails ("every existing id stays where it was"); reverted. clippy, fmt clean.
E-B6 final gate on 8350ba4 (clean tree), 2026-10-07; logs /tmp/s9-11b-final/:
     cargo fmt --all --check                                           PASS (0 s)
     cargo clippy --workspace --all-targets --all-features -D warnings PASS (26 crates, 2 s)
     cargo test --workspace --no-fail-fast    441 passed, 0 failed, 0 ignored (428 base + 13 new); 185 s
     kill_and_resume                          cafe PASS, clock PASS (6 s)
     check_decision_ids                       46 ids, all distinct
     check_doc_headings                       143 sections, none duplicated
     B-1 / I-4  300-day seed-7 social-cafe: 339 lines, faults 0, 365 330 facts, fingerprint
                59339a9c281829c9; sha-256 (all but wall) =
                ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-0; wall 12.9 s
     B-6 planted: `// PLANTED: an item_price` refused naming `price` (not `item`); untracked
                worldpack/tests/wages.rs refused by its path; both reverted, tree clean, scan 4/4.
     diff da31613..8350ba4: 21 files; no contracts/, kernel/, persistence/, server/, clients/,
     cognition/, systems/ or Cargo.lock path.
     CI: none configured (S13).
```

## 9.3 Evidence — PR 11c

Written by the 11c implementation session only (`E-C<n>`).

```text
--- PR 11c (branch mvp0/pr-11c-affordances, base main @ da31613) ---

E-C0 Base captures on da31613 before any code edit, 2026-10-07 (debug, opt-level 1):
     300-day seed-7 social-cafe: 339 lines, faults 0, 365 330 facts, fingerprint 59339a9c281829c9,
     sha-256 of all but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
     = E-0; wall 12.2 s.
     `mineworld validate worlds/social-cafe`: exit 0, sha-256 of output
     ebcd60a0252f0b343b799dc5d7f78575f5eeaec1c7f4c4050dbac9a72ecf56a8.
     C-2's frozen no-payload literals, printed by a throwaway (uncommitted, deleted) test against the
     base contracts crate:
       available  {"action_type":"talk","target":"42","available":true,"unavailable_reason":null,
                   "requirement":{"place":"any","within_range":null,"requires_line_of_access":false,
                   "requires_target_available":false}}
       unavailable {"action_type":"talk","target":"43","available":false,"unavailable_reason":
                   "too_far_away","requirement":{"place":"same_place_as_actor","within_range":3000,
                   "requires_line_of_access":false,"requires_target_available":false}}
       no target  {"action_type":"ring","target":null,"available":true,"unavailable_reason":null,
                   "requirement":{"place":"any","within_range":null,"requires_line_of_access":false,
                   "requires_target_available":false}}
     (each on one line in the test). contracts observation suite on base: 6 passed.
E-C1 C-C1 specs: check_decision_ids 46 ids, all distinct (ARC-34 added); check_doc_headings 143
     sections across 22 documents, none duplicated. ARC-34 absent from every origin/* branch after
     `git fetch`.
E-C2 C-C2 contracts. `cargo test --no-fail-fast -p mineworld-contracts -p mineworld-presence
     -p mineworld-conversation -p mineworld-group-activity -p mineworld-server` → exit 0, every binary
     ok, 0 failed (contracts observation 9 = 6 existing + 3 new; conversation_and_presence 14;
     group_activity 10; presence 13; server two_clients 9, headless 4).
     M-C1 (drop `skip_serializing_if`): an_affordance_without_a_payload_is_the_shape_it_always_was
     FAILED (and an_affordance_cannot_disagree…, whose base literal also gains "payload":null) →
     reverted (`grep -c skip_serializing_if` = 1).
     spike/server: `cargo check --offline --manifest-path spike/server/Cargo.toml` exit 0 on da31613;
     after the change E0308 at world.rs:401 until the one-line annotation at :350, then exit 0.
E-C3 C-C3 presence. `cargo test -p mineworld-presence` → presence.rs 15 passed (13 + 2 new), 0 failed.
     M-C2 (verdict keeps the affordance, drops the payload): a_complete_offer_reaches_the_observation
     _with_its_own_type_and_payload FAILED, 14 passed → reverted. conversation 14, group_activity 10,
     movement 1 + 3 + 7 + 1 passed. clippy -p presence -p contracts --all-targets -D warnings clean.
E-C4 C-C4 the offer band. `cargo test -p mineworld-rule-controller` → 34 passed (28 + 6), 0 failed;
     clippy -p rule-controller --all-targets -D warnings clean.
     M-C3 (drop `is_available()` from the filter): only_available_complete_affordances_are_attempted
       and no_complete_affordance_no_new_draw_decides_anything FAILED → reverted.
     M-C4 (OFFER_DRAW = 0): greetings_still_happen_where_offers_are_made FAILED → reverted.
     M-C5 (RuleController::decide calls offered::attempt first): the_reactive_controller_never
       _attempts_an_offer FAILED → reverted.
     I-4: 300-day seed-7 social-cafe on this working tree (base 7922cb2+C-C3 d2f6c8c + C-C4 diff):
       exit 0, 339 lines, faults 0, 365 330 facts, sha-256 of all but `wall` = ad49c723…c64b = E-0;
       wall 23.0 s (machine shared with the parallel 11b session).
E-C5 C-C5 CP-3. `cargo test -p mineworld-acceptance` → complete_affordances 4 passed (0.03 s),
     precursor_vocabulary 3 passed, 0 failed.
     M-C6 (remove the band's call from decide): a_headless_person_rings_a_bell_the_controller_never
       _heard_of and two_runs_of_one_seed_are_byte_identical FAILED → reverted (`git diff cognition/`
       empty).
     I-2 scan with 11c's row: first run FAILED naming `cognition/rule-controller/src/offered.rs:18:
       item` ("`ARC-35` item 6" in a comment) → reworded; then PASS with no 11c allow-list entry.
     Planted violations (untracked tests/acceptance/tests/planted_11c.rs `SHOP_PRICE`; tracked edit
       `// a wage is due` in offered.rs): refused by name — `shop` at planted_11c.rs:1, `wage` at
       offered.rs:59 → both removed.
     Cargo.lock: +8 lines, mineworld-acceptance's dependency list only.
E-C6 C-C6 I-9 measurement, scratch branch `scratch/11c-chimes` from 6486d5b, local only, 2026-10-07.
     Scratch commit 998a7ad (deleted after; `git ls-remote --heads origin | grep -c scratch` → 0).
     Changed paths: systems/chimes/{Cargo.toml, src/lib.rs, tests/count.rs}, systems/installed/
     {Cargo.toml, src/lib.rs} (one line each), worlds/chimes-cafe/** (social-cafe copied, id/name
     changed, `chimes` appended to `systems`), Cargo.lock. `chimes` offers complete `ring { low |
     high }` to every observer, target none, requirement NONE (worst-case exposure); emits `rang`.
     Criterion (QS-25, stated before measuring): over 300 days seed 7, every seat moves and talks in
     every 30-day bucket, AND every seat's `ring` is accepted in every bucket.
     Run 1, ATTEMPTS_OFFERED = 20: `mineworld run worlds/chimes-cafe --headless --seed 7 --days 300
       --save …` exit 0, faults 0, 367 125 facts, wall 43.0 s (with --save).
       requests: ring accepted 29 907; move accepted 167 348; talk accepted 57 741 (social-cafe:
       180 665 / 67 752); no rejected or unavailable request lines.
       activity: no `move 0` or `talk 0` in any of the 10 buckets × 11 seats; minima per bucket move
       1 346, talk 393.
       rings from the save (scratch reader systems/chimes/tests/count.rs, `rang` facts by ringer and
       ⌊at / 30 d⌋): 110 (ringer, bucket) cells = ids 7–15, 17, 18 (the 11 seats; 16 is otto, no seat)
       × buckets 0–9; minimum 197 per cell; total 29 907 = the accepted ring requests.
       → PASS at 20. No second run needed. ATTEMPTS_OFFERED = 20, OFFER_DRAW = 14,
       OFFERED_CHOICE_DRAW = 15, position after the social initiative and before the walking roll:
       FROZEN for S9 (C-6, I-9). Recorded in ARC-34's note.
     social-cafe on the scratch build (chimes installed, not enabled): 300-day seed-7 sha-256 of all
       but `wall` = ad49c723…c64b = E-0 — installing a pack that offers complete affordances changes
       no world that does not enable it.
E-C-final on 20cf29b (clean tree; final executable head — later commits are Markdown only), main
     unchanged at da31613 (11b not merged), 2026-10-07:
     cargo fmt --all --check                                         PASS
     cargo clippy --workspace --all-targets --all-features -D warnings PASS (re-run after touching
                                              contracts/src/lib.rs: all 18 crates re-linted, clean)
     cargo test --workspace --no-fail-fast    443 passed, 0 failed, 0 ignored across 90 test binaries
                                              (428 at 11a's final + 15 new: contracts 3, presence 2,
                                              rule-controller 6, acceptance 4); 159 s wall
     kill_and_resume                          cafe PASS (0.2 s), clock PASS (0.1 s)
     check_decision_ids                       46 ids, all distinct
     check_doc_headings                       143 sections across 22 documents, none duplicated
     C-1 / I-4                                300-day seed-7 social-cafe: exit 0, faults 0, 365 330 facts,
                                              sha-256 of all but wall = ad49c723…c64b = E-0; wall 12.2 s;
                                              `validate worlds/social-cafe` byte-identical to E-C0's
     I-2 scan                                 green inside the workspace run, 11c row, no allow-list entry
     I-8                                      `git diff da31613 HEAD -- kernel/` empty; contracts/ diff =
                                              observation.rs, action.rs (labelled), their tests and the
                                              trybuild .stderr
     CI: none configured (S13).
E-C-rebase 11b merged first (GitHub #39, merge commit ae1a315). Per §12.0, 2026-10-07:
     `git rebase origin/main` (main @ ae1a3157e85c05279daecf21f82bfea539597dd6); never a merge of main.
     C-C1 … C-C4 applied cleanly; C-C5 conflicted only in precursor_vocabulary.rs — resolved to 11b's
     structure (THIS_SCAN, `Words`, 11b's allow-list kept as merged) with rows 11a, 11b, 11c in order;
     11c's row base moved to ae1a3157… in the same (rebased C-C5) commit. DECISIONS.md (ARC-34 before
     ARC-35, ARC-36 at the end), MVP_STATUS and this file merged without conflict; this file's diff
     against main touches only §4.3 and §9.3. Rebased commits: 2f15998, 37f37aa, 5ab5f40, 0985554,
     d0ed2ca, 0c054b4, 6003e07.
     Gate on 6003e07 (clean tree; earlier evidence does not carry over):
     cargo fmt --all --check                                         PASS
     cargo clippy --workspace --all-targets --all-features -D warnings PASS (18 crates re-linted)
     cargo test --workspace --no-fail-fast    456 passed, 0 failed, 0 ignored across 92 binaries
                                              (main's 441 + 11c's 15); 145 s wall
     kill_and_resume                          cafe PASS (0.2 s), clock PASS (0.1 s)
     I-2 scan                                 PASS: 11b read as merged (base..M^2), 11c unmerged from
                                              ae1a315; no 11c allow-list entry
     planted violations                       untracked planted_11c.rs `SHOP_PRICE` and tracked
                                              `// a wage is due` in offered.rs → refused by name (`wage`
                                              offered.rs:59; `shop` and `price` planted_11c.rs:1 — word
                                              level, 11b's matcher) → removed; scan PASS again
     I-4                                      300-day seed-7 social-cafe sha-256 of all but wall =
                                              ad49c723…c64b = E-0; faults 0; 365 330 facts; wall 12.3 s;
                                              `validate worlds/social-cafe` identical to E-C0's
     check_decision_ids 47 ids distinct; check_doc_headings 143 sections, none duplicated.
     Branch force-pushed (permitted by §12.0 for this rebase only).
```

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

**Raised while detailing 11b and 11c (2026-10-07, on `7ed1648`).**

```text
QS-15  [OPERATOR-MATERIAL — amends ARC-35 point 7, how I-2 is measured] The scan finds a merged
       precursor on HEAD's first-parent chain only (F-30). With 11b and 11c in parallel, the second to
       merge integrates main by a merge (project practice), so on its branch the first one's PR merge is
       reachable only through a second parent and reads as unmerged. If 11b is second, its own `item`
       lines are then scanned under 11c's row and refused.
       Proposal: in 11b's B-C2, search every merge reachable from HEAD and require exactly one
       `Merge pull request #N from <owner>/<branch>` per row (two fail closed, naming both); unit-tested
       on log text; an ARC-35 dated note. Once both are on main nothing changes (first-parent holds).
       Alternative (no amendment): the second PR integrates main by rebase and force-pushes its own
       unmerged branch; its evidence is then re-run on the rebased head anyway.
       Recommended: the amendment.

QS-16  The scan admits whole lines (F-31): an admitted `item` would hide `price` on the same line. 11b
       makes an entry admit named words (`Words::Only`), `Any` only for the scan's own file; every other
       market word on an admitted line is refused. Tightens an approved check; an ARC-35 note.
       Not operator-material. Recommended: accept.

QS-17  11b's allow-list (§4.2.2): only the words `item`, `items`, per listed file, each with its reason;
       a further file needing them for the same reason is a recorded bounded addition; any other market
       word is a material stop. Recommended: accept.

QS-18  `mineworld validate` prints `items` and `organizations` summary lines only when a pack declares
       some, so social-cafe's report stays byte-identical (F-32). Alternative: always print them (social-
       cafe's report gains two `none` lines). Recommended: only when declared.

QS-19  11b proves the order and reference typing of sections on items and organizations with a probe
       owner in worldpack's unit tests (F-22), and teaches no installed pack to carry them (`naming`
       stays people-only, §1.2). The first real owners arrive in 11d. Recommended: accept.

QS-20  [primary session — amends frozen SD-12] No GDScript change in 11c: `affordance(type, target)`
       returns the first match, and SD-12's `payload(action_type, target)` would be ambiguous whenever
       several complete affordances share a type and target, which is the normal case (F-29).
       `affordances()` already returns the payload in each dictionary; ADOPTION.md documents it; the
       accessor arrives in S12 with its first consumer. Recommended: amend SD-12 so.

QS-21  `Affordance::request(actor, encode)` takes the encoder, because observations carry JSON values and
       dispatch takes bytes (F-26); it needs a pub(crate) labelling constructor on ActionRecord (F-25).
       A refinement of SD-9's `request(actor)` within QS-4. Recommended: accept.

QS-22  `Offer::complete(&action, requirement) -> Result<Offer, serde_json::Error>` replaces SD-9's
       `Offer::with_payload::<A>(&A)` with a debug assertion: the type is read off the value, so a
       payload of another action cannot be attached at all. Recommended: accept.

QS-23  `spike/server` (its own workspace, consuming contracts by path) stops compiling once Affordance is
       generic (F-24). Options: one type annotation in 11c, checked with `cargo check --manifest-path
       spike/server/Cargo.toml` (N/A if its base does not build offline); or leave the frozen spike stale
       and say so in spike/README. Recommended: the one-line update.

QS-24  CP-3's end-to-end test lives in tests/acceptance (gaining dev-dependencies on contracts, kernel,
       presence, rule-controller, serde, serde_json), so the rule-controller's manifest is not edited and
       the synthetic pack is compiled into no library. Alternatives: rule-controller dev-dependencies
       (its manifest says it depends on no kernel); tools/cli/tests (wrong owner). Recommended:
       tests/acceptance.

QS-25  The offer band: ATTEMPTS_OFFERED 20, OFFER_DRAW 14, OFFERED_CHOICE_DRAW 15, after the social
       initiative and before the walking roll. Criterion, fixed now: on a scratch install of chimes
       offering to everyone everywhere (worst case), 300 days of social-cafe keep step-08 I-9's activity
       precondition and every seat rings in every bucket; if 20 fails, 11c lowers it, and the passing
       value is frozen for S9 (I-9). §4.3's drafted adversarial (2) is replaced (F-28).
       Recommended: accept.

QS-26  Parallel coordination (§12): one handoff per PR (handoff-11b.md, handoff-11c.md), each PR's own
       ledger sections, fixed anchors in MVP_STATUS, ARC-34 before ARC-35 and ARC-36 at the end of
       DECISIONS, scan rows kept side by side, the second PR moving its scan base on integrating main,
       merge commits only. Recommended: accept.
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

---

# 12. Running 11b and 11c in parallel

## 12.0 Freeze record for 11b and 11c (primary session, 2026-10-07)

Both designs are frozen. The rulings below bind, and they override any other text in §§4.2, 4.3,
12, 13 and 14.

- **QS-15 — declined; the fallback is adopted instead.**
  - ARC-35 point 7 keeps its approved first-parent merge detection. The operator approved that
    measurement, and changing it is not needed to run two PRs in parallel.
  - 11b and 11c are implemented in parallel but **merged one at a time**.
  - The PR that merges second must **rebase onto the new main** before its final gates. It must
    never merge main into its branch.
  - That rebase moves its scan row's base to the main it rebased onto, and it re-runs its evidence
    (the sha, the planted violations and the full gate) on the rebased head.
  - Force-pushing that PR's own branch after the rebase is permitted, and only for that.
  - §12's merge-main steps are superseded by this ruling.
- **QS-16 — accepted.** The scan admits words, not lines. The change is recorded as an ARC-35 note
  because it tightens the check without loosening it. The mutation that shows the tightening (an
  `item_price` line refused for `price`) is part of B-C2's validation.
- **QS-17, QS-18, QS-19 — accepted.** These cover the allow-list as tabled, the conditional
  `validate` lines, and the probe section owner.
- **QS-20 — accepted, amending frozen SD-12.** 11c makes no GDScript change. Client-side use of
  complete affordances is S12's. SD-12's text is amended to say so, with this record cited.
- **QS-21, QS-22, QS-23, QS-24, QS-25, QS-26 — accepted as recommended.** QS-25's
  pass criterion was stated before measuring, and it binds: if 20 fails, the value that passes is
  frozen, and the scratch measurement is recorded.
- **The order of the two merges** is decided by whichever PR is ready first. Neither waits for the
  other.

Two sessions, two worktrees, two branches, both from the same `main`. Neither touches the other's
worktree (`CLAUDE.md` §3.1). Every file both could touch has an owner or an anchor here, so a conflict
is either avoided or resolved mechanically by whoever merges second.

```text
file                                    11b                          11c                          how
tests/acceptance/tests/                 row 11b; word-level          row 11c; no allow-list       second to merge keeps both rows
  precursor_vocabulary.rs               allow-list (B-C2); entries   entry                        in order 11a, 11b, 11c and takes
                                        (§4.2.2); merged detection                                11b's structure; textual only
                                        (QS-15)
the PR's own scan row base              moved on integrating main    moved on integrating main    in the same commit as the
                                                                                                  integration (below)
tests/acceptance/{Cargo.toml,           —                            dev-dependencies, doc        11c only
  src/lib.rs}
Cargo.lock                              no change expected           acceptance's dependency      11c only; if 11b needs a
                                                                     list                         dependency after all, the second
                                                                                                  regenerates with `cargo metadata`
docs/DECISIONS.md                       ARC-36 appended at the end;  ARC-34 inserted directly     different hunks
                                        ARC-35 note under ARC-35     before `## ARC-35`
                                                                     (ARC-35's note is 11b's)
docs/MODULE_SPEC.md                     §4.1                         §5                           different sections
docs/CORE_CONCEPTS.md                   §7 note                      §15.2                        different sections
docs/PACKAGE_FORMAT.md                  §8 row                       —                            11b only
server/PROTOCOL.md,                     —                            yes                          11c only
  clients/protocol/ADOPTION.md
docs/MVP_STATUS.md                      capability row after         capability row after         distinct anchors; `Updated:` line
                                        "Authored content owned by   "Conversation"; evidence     and S9 stage row edited by
                                        packs"; evidence row after   row after the table's last   neither (planning session)
                                        "People are named, …"        row
step-10-market.md                       §4.2, §9.2, §13 live parts   §4.3, §9.3, §14 live parts   header, §§1–3, §5, §10, §12 and
                                                                                                  every other PR's sections are the
                                                                                                  planning session's
handoff                                 handoff-11b.md               handoff-11c.md               handoff.md stays 11a's closed
                                                                                                  record until the planning session
                                                                                                  folds both back after merge
overall.md                              —                            —                            planning session only
authoring/, worldpack/, tools/cli/      11b only                     —
contracts/, systems/presence/,          —                            11c only
  cognition/, systems/{conversation,
  group-activity}/tests, spike/server
```

**Integrating main.** Whichever PR is still open when the other merges integrates `main` before its
final gates (by merge, project practice). In that same commit it sets its own `PRECURSORS` row's `base`
to the `main` commit it integrated, so its unmerged range — `base` to the working tree — again holds only
its own lines. If QS-15 is declined, it integrates by rebase instead and force-pushes its own branch.
After integrating, it re-runs its targeted tests, the I-2 scan, the 300-day sha (I-4) and its final full
gate on the integrated head; earlier evidence does not carry over.

**Merging.** Each PR is merged with a **merge commit** — never squash, never rebase-merge — because the
scan's merged range is `base..M^2` of `Merge pull request #N from yuema137/<branch>` (ARC-35 point 7). A
squash would fall back to `base..HEAD` and fail loudly once later PRs add the market.

**After both merge.** The planning session records both merges in §4.2 and §4.3, the header, overall §7
and MVP_STATUS's `Updated:` line and S9 row; folds the two handoffs into `handoff.md`; confirms on `main`
that the I-2 scan passes with three rows and that the 300-day sha is E-0.

# 13. Execution contract for PR 11b (proposed; confirmed at 11b's freeze)

```text
PROJECT / PR        MVP-0 · Step 10 / PR 11b — items and organizations as World Pack content kinds
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-10-market.md §4.2 (4.2.1–4.2.4), §9.2
RELATED / BINDING   overall.md §§1, 7; this file §§1.3, 2.4, 2.5, 3 (SD-7, SD-8), 8.4, 10 (QS-5, QS-6,
                    QS-15 … QS-19, as answered), 12; MODULE_SPEC §§4, 4.1; PACKAGE_FORMAT §8;
                    CORE_CONCEPTS §§7, 8; DECISIONS ARC-15, ARC-31, ARC-33, ARC-35, DEP-10
IMPLEMENTATION BASE the main named at freeze (main @ 7ed1648 + this planning branch); branch
                    mvp0/pr-11b-content-kinds; worktree /Users/yuema137/mineworld-worktrees/s9-11b (proposed),
                    held by the implementing session only
APPROVED SCOPE      §4.2: B-C1 … B-C6; nothing in contracts/, kernel/, persistence/, server/, clients/,
                    cognition/, systems/
FROZEN INVARIANTS   I-2 (§4.2.2's allow-list; any other market word is a material stop), I-4 (B-1),
                    I-8 (no contracts/ or kernel/ diff); the parallel rules of §12
SEQUENCE            B-C1 → B-C2 → B-C3 → B-C4 → B-C5 → B-C6, each committed and pushed when coherent
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: 300-day social-cafe (~15 s) and the
                    10-day CLI runs; one full workspace gate on the final head (background); about one
                    hour in total; real-model: NOT REQUIRED
LIVE DOCUMENTATION  §4.2 checkboxes; §9.2 E-B ledger
HANDOFF             .structured-coding/plans/mvp0/handoff-11b.md, initialized at B-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the primary session's freeze message
  semantic commits, branch push       recommended authorized, as for 11a
  PR creation / update                recommended authorized, as for 11a
  integrating main (§12)              recommended authorized, with the base move
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns step header/§§1–3, overall and MVP_STATUS's Updated/S9 lines;
                    the implementing session owns §4.2 and §9.2
NORMAL STOP         PR 11b READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a market word other than item/items needed; a change to a public contract, the kernel
                    or ownership; an existing id, event id or the social-cafe run changing; a needed new
                    meaning for an existing PackError variant; QS-15 unanswered when the parallel
                    integration arises (use §12's fallback only if the operator declined it)
```

# 14. Execution contract for PR 11c (proposed; confirmed at 11c's freeze)

```text
PROJECT / PR        MVP-0 · Step 10 / PR 11c — complete affordances (F-3)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-10-market.md §4.3 (4.3.1–4.3.3), §9.3
RELATED / BINDING   overall.md §§1, 7; this file §§1.3 (I-5, I-8, I-9), 2.3, 3 (SD-9 … SD-12), 8.4,
                    10 (QS-4, QS-20 … QS-25, as answered), 12; CORE_CONCEPTS §15; MODULE_SPEC §5;
                    server/PROTOCOL.md §§5–6; DECISIONS ARC-23, ARC-26, ARC-27, ARC-35
IMPLEMENTATION BASE the main named at freeze (main @ 7ed1648 + this planning branch); branch
                    mvp0/pr-11c-affordances; worktree /Users/yuema137/mineworld-worktrees/s9-11c (proposed),
                    held by the implementing session only
APPROVED SCOPE      §4.3: C-C1 … C-C6; contracts/ only observation.rs plus the pub(crate) labeller in
                    action.rs and their tests; no kernel/, worldpack/, authoring/, tools/cli/src change
FROZEN INVARIANTS   I-2 (no 11c allow-list entry), I-4 (C-1), I-5 (decide stays pure; RuleController and
                    the rule-controller manifest unchanged), I-8, I-9 (constants fixed by C-C6's
                    criterion, then frozen); the parallel rules of §12
SEQUENCE            C-C1 → C-C2 → C-C3 → C-C4 → C-C5 → C-C6, each committed and pushed when coherent
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: 300-day social-cafe (~15 s) at C-C4 and
                    the final head; the scratch chimes install's 300-day run (≤ 2 runs if the rate must be
                    lowered); one full workspace gate on the final head; about one hour in total;
                    real-model: NOT REQUIRED
LIVE DOCUMENTATION  §4.3 checkboxes; §9.3 E-C ledger
HANDOFF             .structured-coding/plans/mvp0/handoff-11c.md, initialized at C-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the primary session's freeze message
  semantic commits, branch push       recommended authorized, as for 11a
  PR creation / update                recommended authorized, as for 11a
  scratch branch for chimes (C-C6)    recommended authorized, local only, deleted after evidence
  integrating main (§12)              recommended authorized, with the base move
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     as §13
NORMAL STOP         PR 11c READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a market word needed; a contracts/ change beyond the payload field and its methods; a
                    change to RuleController, decide's purity or the rule-controller manifest; a change of
                    behaviour (not a type annotation) in any existing pack or test; the social-cafe run
                    or an AC-13/AC-15 transcript changing; C-C6's criterion failing at every rate ≥ 5
```
