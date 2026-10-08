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
- **PR 11d** `MERGED` as `70e532f` (GitHub #43, 2026-10-07), with a merge commit — the first half of
  the measured AC-1 transformation (ARC-35 point 1). Designed in §4.4 (`DESIGN FROZEN` per §4.4.0);
  evidence §9.4. The primary session's review on the actual merge diff `70e532f^1..70e532f`: 0 paths
  outside `systems/`, `worlds/`, `Cargo.lock` and Markdown; gates re-run 479/0; its own 300-day seed-7
  market-town run 0 faults, 15.2 s, every seat moved and talked in every bucket; its own 30-day save
  holds `items-transferred` facts each caused by an action, 37 090 facts in all (= the implementer's);
  its own mutation `PERSON_CAPACITY` 6→7 failed two capacity tests (overall §7).
- **F-37's doc fix** (authoring's `Seeding` doc) is a docs-only PR after 11d's merge, outside the AC-1
  range (§4.4.0).
- **PR 11e** `MERGED` as `2dddda8` (GitHub #46, 2026-10-07), with a merge commit — the second and
  last half of the measured AC-1 transformation (ARC-35 point 1, six market packs by the operator's
  QS-35). Designed in §4.5 (`DESIGN FROZEN` per §4.5.0); evidence §9.5; deviations §4.5.7. The
  primary session's review on the actual merge diff `2dddda8^1..2dddda8`: 0 paths outside
  `systems/`, `worlds/`, `Cargo.lock` and Markdown; gates re-run 510/0; its own 300-day seed-7
  market-town run with `--save`: 0 faults, 372 755 facts (= the implementer's), 33.7 s, and
  `inspect` shows 2 717 `items-consumed`, 3 300 `money-transferred` and no `wage-unpaid`; its own
  mutation (the payer not debited, breaking money conservation) failed two economy tests, the buy
  test among them (overall §7). L-13 (the bounded-horizon economy) is recorded; F-47 and F-48 are
  carried to the S9 closeout.
- **The measured transformation is complete.** Its two merges are 11d `70e532f` and 11e `2dddda8`,
  each a merge commit read against its own first parent (ARC-35 point 1).
- **PR 11f** — the proof — is next, outside the AC-1 range: the mechanical AC-1 test, market-town's
  activity check as a committed test, AC-2 at world level for every market pack, and Milestone C
  through the real server. Detailed in §4.6 by the planning session on `mvp0/s9-11f-plan` from
  `main @ 2dddda8`.

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


## 4.4 PR 11d — the transformation, part 1: owning and giving things (full design; DESIGN FROZEN 2026-10-07)

### 4.4.0 Freeze record (primary session, 2026-10-07)

The 11d design is frozen and its execution contract (§15) is confirmed. These rulings bind and
override any other text in §4.4 and §15.

- **QS-27 accepted.** A person may hold at most six items and organizations are unbounded. The cap
  is a pack rule in `inventory`, not a controller change (I-9). The spike's evidence for it is
  F-39 (Otto as a sink).
- **QS-28 to QS-34, QS-36 and QS-37 accepted as recommended.** That includes `items-produced`
  deferred to 11e, ARC-37, and per-seat give evidence from a scratch reader until 11f.
- **QS-35 was decided by the operator on 2026-10-07: a consumption pack.** The operator was offered
  shops buying back, a consumption pack, or relaxing CP-4, and chose the consumption pack. It does
  not change 11d. It **widens 11e's scope** by one System Pack under `systems/`, so it sits inside
  the AC-1 change set:
  - The pack consumes items; for example, eating or drinking at the café uses a food item.
  - It closes QS-10's MVP gap for `eat`.
  - `inventory` stays the sole writer of holdings. The consumption pack states the removal through
    inventory's checked constructor (ARC-26).
  - CP-4 keeps its "purchase in every bucket" criterion unchanged.
  - 11e's medium scope in §4.5 is amended to include the pack. Its detailing settles the pack's
    name, what it consumes, and whether it needs items to be produced.
- **F-37 (the `Seeding` doc misstates when genesis facts become visible)** is fixed after 11d merges,
  in a docs-only PR outside the AC-1 range. 11d's in-pack workaround stays.
- **F-41 (clients cannot name items)** is recorded for S12.
- **The merge rules are 11a's:** a merge commit, never squash. The AC-1 change-set check (D-1) runs on
  the actual merge diff before the operator-review handoff, and again by the primary session at merge.

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

- [x] Implementation: as scoped. ARC-37 appended after ARC-36; MODULE_SPEC §4.1 "two sections" →
  four (`item`, `holdings`); PACKAGE_FORMAT §8 names the four; systems/README lists the three packs and
  their test commands.
- [x] Validation: both doc checks; ARC-37 absent from every `origin/*` branch (§9.4 E-D1).
- [x] Review: no defined term redefined (`Item` stays `ARC-36`'s kind; "holdings" is inventory's
  component, not a core term); the section table matches SD-16/SD-17. One bounded refinement,
  recorded as D-D1: a category also may not begin or end with `-` (SD-16 said only "1–32 bytes of
  `a–z`, `0–9`, `-`"), the usual slug rule, so `-drink` and `drink-` cannot name two categories that
  read alike. ARC-37 and MODULE_SPEC state it.

### D-C2 — `systems/item`

**Scope.** `systems/item/{Cargo.toml, README.md, src/{lib,system,section,event,component,category,
codec}.rs, tests/item.rs}`, laid out as `naming`. Dependencies: authoring, contracts, kernel, presence,
sdk, serde, serde_json, thiserror (`workspace = true`); dev: serde-saphyr (decode a section as the
loader does), as naming's tests.

- [x] Implementation: `ItemSystem` (`impl SystemPack { owns_section!(); }`, empty
  `PerceptionProvider`), `Category`, `ItemKind` (`owned_component!`, `item-kind`), `ItemKindDeclared`,
  `AuthoredSection` (`item`, carried by items), `is_declared`. Files: `systems/item/{Cargo.toml,
  README.md, src/{lib,system,section,event,component,category,codec}.rs, tests/item.rs}`. The
  reduction also refuses a declaration about an entity that is not a living Item
  (`FactRefusedByOwner`), because an `ItemId`'s type tag is only a label in the payload.
- [x] Validation: `cargo test -p mineworld-item` 4 passed; clippy `-D warnings` and fmt clean (§9.4
  E-D2). Tests: the section seeds one public genesis fact reduced into ItemKind; bad categories
  refused with Category's message, an unknown key by name; `is_declared` false for an Item entity
  without the section; a seed for a place or person refused; a hand-built declaration about a place
  refused at reduction, writing nothing. M-D2 is run in D-C3 (it needs inventory).
- [x] Review: no dependency on any market pack (manifest: authoring, contracts, kernel, presence for
  the trait, sdk, serde, serde_json, thiserror); no disclosure (empty provider); owns exactly one
  component (`item-kind`); the only write is in `react`.

### D-C3 — `systems/inventory`

**Scope.** `systems/inventory/{Cargo.toml, README.md, src/{lib,system,section,event,component,admit,
codec}.rs, tests/{inventory.rs, persisted.rs, support/mod.rs}}`. `mineworld-item = { path = "../item" }`;
dev: persistence, serde-saphyr.

- [x] Implementation: SD-17 … SD-19 — `Holdings`/`Held`, `PERSON_CAPACITY`, `can_take`,
  `admit_transfer`, `transfer`, `Stocked`, `ItemsTransferred` (accessors), `holdings:` section
  (`references` = Items), reductions, disclosure to the holder. Files: `systems/inventory/{Cargo.toml,
  README.md, src/{lib,system,section,event,component,admit,codec}.rs, tests/{inventory.rs,
  persisted.rs, support/mod.rs}}`. `stocked` is checked at reduction by a crate-private `admit_stock`
  (living holder, count ≥ 1, declared kind, `can_take`) — the stock half of SD-18's "checked for a
  declared kind at reduction". The authored count is `NonZeroU32`, so a zero is refused as it is
  decoded, at its line. `can_take` answers false for anything that is not a living holder.
  The tests' stater `hands` (support/mod.rs) exists only in the tests: inventory provides no action,
  so `pass` decides transfers either through the checked constructor or **forged** past it.
- [x] Validation: `cargo test -p mineworld-inventory` 9 passed (inventory 8, persisted 1); clippy
  `--all-targets -D warnings`, fmt clean. M-D2, M-D3, M-D4 (constructor half) each fail and are reverted;
  F-38 reproduced (§9.4 E-D3).
- [x] Review: one write path per fact (`react`: `stocked` → `adding`; `items-transferred` →
  `removing` + `adding`); `admit_transfer` is the refusal logic for transfers and `admit_stock` for
  stock, both calling `can_take`; no `f32`/`f64`; the authored map is a `BTreeMap`, `Holdings` a sorted
  `Vec` (binary search by `ItemId`). The seed's capacity check sums the authored counts itself
  (`AuthoredHoldings::total`), because no state exists at seeding (F-37); the reduction's `can_take`
  is the second check.

### D-C4 — `systems/item-transfer`

**Scope.** `systems/item-transfer/{Cargo.toml, README.md, src/{lib,system,action,offer,codec}.rs,
tests/{give.rs, removable.rs, paced.rs, support/mod.rs}}`. `mineworld-inventory = { path =
"../inventory" }`; dev: `mineworld-item` by path, movement, rule-controller (`workspace = true`, for
D-6's controller test — a dev-dependency from a market pack to the controller, the direction ARC-35
check 2 allows).

- [x] Implementation: SD-20. Files: `systems/item-transfer/{Cargo.toml, README.md,
  src/{lib,system,action,offer,codec}.rs, tests/{give.rs, removable.rs, paced.rs, support/mod.rs}}`.
  `GIVE_RANGE = 3 000 mm`, `give_requirement()`; a missing target is `NoSupportedInteraction` (SD-20's
  "a different living Person target (NoSupportedInteraction otherwise)"). Dev-dependencies: authoring,
  item (path), serde-saphyr (seed through the section contract), movement, rule-controller.
- [x] Validation: `cargo test -p mineworld-item-transfer` 10 passed (give 4, removable 3, paced 3);
  clippy `--all-targets -D warnings`, fmt clean. D-5 (offer/dispatch half), D-6, D-7; M-D4 (offer
  half), M-D5, M-D6 each fail by name and are reverted (§9.4 E-D4).
- [x] Review: owns no component (declaration: depends on inventory and presence, provides `give`,
  emits `items-transferred` only); `resolve` emits only `mineworld_inventory::transfer`'s emission;
  offers exclude the observer, non-persons and observers with no holdings; complete offers only for
  kinds held, in item order (Holdings is sorted).

### D-C5 — Install: three lines each in `systems/installed`

- [x] Implementation: `systems/installed/Cargo.toml` three `path` lines; `src/lib.rs` `Item`,
  `Inventory`, `ItemTransfer` after `Schedule`; `Cargo.lock` regenerated by the build (+3 lines in
  mineworld-installed-systems' dependency list; the three path packages arrived with D-C2…D-C4).
- [x] Validation: `cargo test -p mineworld-installed-systems -p mineworld-worldpack` all passed
  (installed 3; worldpack unit 4, content_kinds 1, refusals 38, social_cafe 15, structure 2, doc 1);
  D-2's sha = E-0 and validate byte-identical to E-D0 (§9.4 E-D5).
- [x] Review: the two files gain exactly those lines (`git diff --stat`: 3 + 3, Cargo.lock 3); no root
  manifest edit was needed.

### D-C6 — `worlds/market-town`

- [x] Implementation: SD-21 — copied with `git read-tree --prefix=worlds/market-town/ -u
  HEAD:worlds/social-cafe`, then: world.yaml header (a Market Town paragraph above Social Café's
  unchanged header), id/name, `item, inventory, item-transfer` appended to `systems` with a comment,
  an `items:` list of 20 kinds; `items/<key>.yaml` × 20 (tags + `item: { category }`: drink 4, food 6,
  goods 10); one commented `holdings:` block appended to each of the 12 person files (28 entries, 30
  items, 2–3 per person, Otto 2); README.md rewritten for Market Town (human orientation).
- [x] Validation: D-9 a–d, D-10, the scratch reader on local branch `scratch/11d-reader` (deleted;
  `git ls-remote --heads origin | grep -c scratch` → 0); M-D7 on that branch; the seven-item scratch
  copy refused by `validate` (§9.4 E-D6).
- [x] Review: no social-cafe field or section changed in the copy (D-10's diff has removals only in
  README.md and the id/name lines); Otto present with holdings; every authored person within capacity
  (max 3).

### D-C7 — Close: status, change set, full gates, ledger

- [x] Documentation: `docs/MVP_STATUS.md` — a capability row ("Owning and giving things", after
  "Independently installable System Packs") and an evidence row (after CP-3's); the `Updated:` line,
  the S9 row and the `worlds/market-town` artefact row (still ❌) are left to the planning session;
  §4.4 checkboxes; §9.4 `E-D*`; the handoff.
- [x] Validation, once, on the final executable head 341f2f2 (§9.4 E-D-final): all PASS.
- [x] Review: D-1 … D-11 each with evidence (E-D-final's table); deviations D-D1 … D-D4 (§4.4.6); the
  PR is to be merged **with a merge commit** (ARC-35 point 1 reads `M^1..M`), said in the PR body.

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

### 4.4.6 Deviations and discoveries during implementation (11d session)

```text
D-D1  bounded  A category may not begin or end with '-' (SD-16 listed only the byte set). Reason: the
               usual slug rule; two spellings of one category cannot differ by a stray dash. Stated in
               ARC-37 and MODULE_SPEC §4.1. Validation: item's category test.
D-D2  bounded  M-D3's named test case (over-transfer) is not the one that fails: Holdings::removing's
               checked_sub is a second guard, so with admit_transfer skipped that case is still refused
               (FactRefusedByOwner, PreconditionFailed). The "to oneself" and capacity cases fail
               instead (E-D3). The guard is kept: a reduction that cannot underflow is not a weaker
               owner. No test weakened; the mutation is caught.
D-D3  bounded  inventory's tests need a stater, because inventory provides no action. A test-only
               System `hands` (tests/support/mod.rs) provides `pass`, honest (through transfer) or
               forged (bytes built by hand). It depends on inventory and declares the emission, as
               ARC-26 requires of any stater. Lives only in the tests.
D-D4  bounded  M-D4 fails two capacity tests, not "the three": the authored-holdings test sums the
               authored counts at seeding (no state exists then, F-37) and does not call can_take, so it
               survives by construction; the constructor/reduction test and the offer/dispatch test
               both fail (E-D3, E-D4).
D-D5  process  One read-only `xargs cat` slipped into a wait command (reading a task log), against the
               kickoff's tool discipline. No file was changed by it. Recorded, not repeated.
```

## 4.5 PR 11e — the transformation, part 2: work, money, shops and consumption (full design; DESIGN FROZEN 2026-10-07)

### 4.5.0 Freeze record (primary session, 2026-10-07)

The 11e design is frozen, and its execution contract (§16) is confirmed. These rulings bind, and they
override any other text in §4.5 and §16.

- **QS-39 — accepted (primary session).** `consumption` provides `drink` beside `eat`. The operator's
  QS-35 decision chose a consumption pack, and a café that consumes drinks is that decision applied.
  It does not widen it.
- **QS-45 — accepted.** Economy discloses a shop's stock count to whoever is in the shop. It reads
  inventory's state for this, which is allowed because economy declares inventory as a dependency;
  it never writes that state.
- **QS-47 — accepted, with the horizon recorded.** "No wallet drained" means three things together:
  zero `wage-unpaid`, no wallet ever below the cheapest price, and money conserved.
  - People without a job live on endowments sized for 300 days.
  - That is a **bounded-horizon economy, not a closed one**. Recorded as living-world gap L-13: past
    the measured horizon, those people run out. Closing it (more jobs, or income without one) is a
    later step's work, not a tuning of this one.
- **QS-40 to QS-44, QS-46, QS-48 to QS-53 — accepted as recommended.** This includes:
  - `items-produced` by employment, prorated by attendance (QS-41);
  - 11e editing the merged `inventory` pack, which is under `systems/` and inside the AC-1 range
    (QS-42);
  - shops authored in the operator organization's `economy:` section (QS-43);
  - ARC-38 plus an ARC-35 note (QS-52).
- **F-47 and F-48** are framework limits that 11e works around inside its packs. Neither blocks AC-1.
  They are carried to the S9 closeout as findings, not fixed here.
- **E-9 b's conditions were stated before the first measured run, and they bind.** If the final
  content fails any of them, fix the content or the packs, never the controller (I-9). Record every
  sizing run, including the failures.
- **Merge rules:** a merge commit, never a squash. Check the AC-1 change set on the actual merge diff
  before the handoff, and the primary session checks it again at merge.

The medium scope below is kept as it was written before 11d merged, so a reviewer can see what the
detailing changed. Where it and §§4.5.1 … 4.5.6 disagree, §§4.5.1 … 4.5.6 govern, and each change is
named there with its finding (§8.6) and question (§10, QS-39 …).

**Scope amended 2026-10-07 (QS-35, operator).** 11e also adds a **consumption System Pack** under
`systems/`, so that items are used up and the market keeps turning. Its removal of items goes through
inventory's checked constructor. Details are in §4.4.0.

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

### 4.5.1 Identity, base, approved scope

```text
PR            11e — work, money, shops and consumption (S9, fifth of six; the second and last half of
              the measured AC-1 transformation, ARC-35 point 1, widened by the operator's QS-35)
base          main @ 70e532f (11d merged) plus the docs-only merges named at freeze (#44, and this
              planning branch once merged). Re-audit §8.6 if anything under systems/, worlds/,
              authoring/, sdk/, worldpack/src/load.rs or cognition/rule-controller/src/offered.rs moved
branch        mvp0/pr-11e-work-money-shops, in its own worktree, held by the implementing session only
audit         §8.6 (70e532f), including the R-S9-1 spike (§9 E-6)
scope         §1.1 PR 11e as amended by QS-35; SD-13 as refined by SD-22 … SD-28; QS-39 … QS-53 as
              answered
depends on    11a (installed set), 11b (organizations as content), 11c (complete affordances), 11d
              (item, inventory, item-transfer, market-town): all merged
```

**Goal.** People in Market Town work, earn, spend, and use things up. Three new System Packs —
`economy` (wallets, shops and `buy`; the only mover of money), `employment` (jobs, shifts as a
Process, `wage-due`, production) and `consumption` (`eat` and `drink`) — are installed by three lines
each in `systems/installed`. `inventory` (11d's) gains the two facts that create and remove items,
`items-produced` and `items-consumed`, each with its checked constructor. `worlds/market-town` gains two
organizations, wallets, two shops, two jobs and the three packs. Every headless person buys, eats and
drinks through the unchanged paced controller's offer band, because `buy`, `eat` and `drink` are offered
as complete affordances (`ARC-34`); work is attendance, so the two job holders work by following the
routines they already have (`ARC-32`, QS-8).

**The change set is inside AC-1 by construction (I-1).** Every path this PR may touch:

```text
systems/economy/**                      new pack
systems/employment/**                   new pack
systems/consumption/**                  new pack
systems/inventory/{src/**, tests/**, README.md}   items-produced, items-consumed, produce, consume
                                        (a merged pack, under systems/ — inside the range, QS-42)
systems/installed/Cargo.toml            three dependency lines, by path
systems/installed/src/lib.rs            three installed! lines
systems/README.md                       the pack list (Markdown)
worlds/market-town/**                   organizations/, sections appended to person files, world.yaml
Cargo.lock                              three new path packages under systems/, and
                                        mineworld-installed-systems' dependency list; nothing else
docs/*.md                               DECISIONS (ARC-38, ARC-35 note), MODULE_SPEC §4.1, PACKAGE_FORMAT §8,
                                        MVP_STATUS
.structured-coding/plans/mvp0/*.md      this ledger, handoff
```

Why nothing else can be needed, from the spike (§9 E-6): the same three packs, the inventory edit,
the six install lines and the content touched exactly `Cargo.lock`, `systems/{economy,employment,
consumption,inventory,installed}/**` and `worlds/market-town/**` (29 paths), compiled on the first build,
and the whole workspace suite passed unchanged (480 = 479 + the scratch reader). **A needed edit to any
other path is a material stop**, reported with evidence and answered by a precursor justified without the
market (I-2), never by a quiet edit inside the range.

**Non-goals.** No sleep, hunger or needs (QS-10 stays open for `sleep`; consumption closes `eat`, not
hunger); no shop buying back (QS-35 (a), declined by the operator); no hiring, firing, promotion or
business ownership (§1.2); no item instances (`ARC-36`); no names for items or organizations (F-41); no
client change (QS-13); no world-level test in `tools/cli/tests` or `tests/acceptance` — the AC-1 test,
CP-4's test, AC-2 at world level and Milestone C are 11f's, outside the range; no change to any pack but
`inventory`, nor to `kernel/`, `contracts/`, `authoring/`, `sdk/`, `worldpack/`, `cognition/`, `tools/`,
`server/` or `persistence/`; no edit to any place file of market-town.

### 4.5.2 Design (SD-22 … SD-28)

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-22** | **`inventory` gains the two facts that create and remove items.** `items-produced { holder, item, count }` and `items-consumed { holder, item, count }`, both its vocabulary, both reduced by it alone, both visible to the holder (`Participants`, the holder). Checked constructors `produce(world, holder, item, count)` and `consume(world, holder, item, count) -> Result<Emission, Rejection>`, each asking a public `admit_production` (living holder, count ≥ 1, declared kind, `can_take` — a person still carries at most six) or `admit_consumption` (living holder, count ≥ 1, declared kind, holds at least `count`); the reduction asks the same function again and on refusal writes nothing and fails `FactRefusedByOwner`. | `ARC-26`, `ARC-37` item 3's pattern, applied to the two holdings changes 11d did not have. QS-28 is answered here: production comes in now because consumption destroys items, so without it the shops' stock and CP-4's purchases end (QS-41). The fact names say what happened to holdings, not why: the stater (a shift, a meal) is the cause (QS-42). |
| **SD-23** | **`employment` owns jobs, the `employed-by` edge and the `shift` Process; work is attendance.** Section `job:` on person files: `{ employer: <organization key>, workplace: <place key>, from: "HH:MM", until: "HH:MM", wage: <minor units per hour>, produces: { <item key>: <count per full shift ≥ 1> } }`, `from < until` (no shift across midnight, QS-50), times in schedule's `TimeOfDay` (a Cargo dependency on its type only). Genesis fact `hired { employee, job }` → `Employment` on the person, the `employed-by` edge (Person → Organization), and one `shift` Process woken at each next `from` and `until`. At `from` the wake states `shift-started { employee, present }` (present = presence's `Presence` at the workplace); between wakes employment reacts to presence's `person-entered-place` for an employee on shift — entering the workplace starts a present span, leaving it (`from` = workplace) adds the span to the worked seconds (`ARC-28`: its own state, written by reacting). At `until` the wake states `shift-ended { employee, worked }`, then — when worked > 0 — `wage-due { employee, employer, amount = wage × worked ÷ 3 600 }` and, per produced kind, inventory's `items-produced` for the employer, `count = per_shift × worked ÷ shift length` (floor, skipped at 0), through `produce`. Depends on `inventory` (states its fact) and `presence` (reads positions). Discloses `Employment` to the employee only. `hired` is biographical (QS-49). Never reads or writes a `Wallet`. | QS-8 as approved, made exact by the spike: `PersonEnteredPlace` carries `from` (F-49), so a departure is seen without polling. Production is prorated by attendance, so an absent worker produces nothing and is paid nothing. Integer arithmetic throughout (I-6). |
| **SD-24** | **`economy` owns `Wallet` and `Shop`, and is the only mover of money.** One section, **`economy:`**, on person and organization files: `{ wallet: <minor units> }`, and on an organization optionally `shop: { at: <place key>, prices: { <item key>: <price ≥ 1> } }` — shops are authored on their operator, because a pack owns exactly one section (F-47, QS-43). Genesis facts `funded { holder, balance }` (the holder) and `shop-opened { place, operator, prices }` (public) → `Wallet { balance: u64 }` on the holder, `Shop { operator, prices }` on the place. `money-transferred { from, to, amount }` is the one fact that moves money; its reduction refuses an amount larger than the payer holds, a zero amount or a payer that is the payee (`FactRefusedByOwner`, writing nothing). **`buy { item }`**: no target; requirement `at_place(shop)` and target available; offered, to a living person standing in a shop's place, as **one complete affordance per priced kind**, available iff the operator holds one (`admit_transfer`), the buyer can pay, and the buyer can carry it — otherwise unavailable `TargetUnavailable`, the one reason an offer can carry (F-48, QS-44). `validate` asks the same through `SpatialRequirement::evaluate`; `resolve` states `money-transferred` (buyer → operator, `Place(shop)`, QS-45) and inventory's `items-transferred` (operator → buyer) through `transfer`. **Wages:** economy subscribes to employment's `wage-due` (Cargo dependency on employment's type, **no system dependency**, `ARC-28`) and states `money-transferred` (employer → employee, caused by the `wage-due`) or, when the employer cannot pay, `wage-unpaid { employee, employer, amount }`. Depends on `inventory` and `presence`. Discloses a `Wallet` to its holder, and to everyone perceiving a shop's place the shop's listing: operator, and per priced kind its price and how many the operator holds (QS-45). Nothing biographical. | `CORE_CONCEPTS.md` §13.1's canonical example implemented literally; CP-5. `u64` makes a negative balance unrepresentable and the refusal path is the one a test shows taken. The listing is how a second client perceives a purchase (CP-7, 11f) without anybody's holdings being disclosed. |
| **SD-25** | **`consumption` provides `eat { item }` and `drink { item }`, and owns nothing.** A held kind whose `ItemKind` category is `food` is eaten, `drink` drunk (published constants `EATEN`, `DRUNK`); goods are never consumed. Requirement: none — a person eats what they carry, wherever they are (QS-40). Offered, to a living person, as one complete affordance per edible or drinkable kind held, target none, in item order. `validate`: payload, a living Person actor, no target, the action matching the kind's category (`NoSupportedInteraction` otherwise), then `admit_consumption(actor, item, 1)`. `resolve`: inventory's `items-consumed` through `consume`, nothing else. Depends on `inventory` (system and Cargo); Cargo dependency on `item` for `ItemKind`. Nothing biographical, no disclosure. | QS-35's pack (QS-39). It closes MVP §5's `eat` as an interaction (and adds `drink`, without which drinks would fill hands, F-50); hunger, appetite and `sleep` remain a needs pack's (QS-10). The category is item's public attribute, introduced as "the smallest real attribute" (QS-32); what is edible is this pack's rule about it. |
| **SD-26** | **Shops sell consumables only, and stock is produced by the shift.** The café sells coffee, tea, croissant, cake, sandwich, soup; the store apple, bread, milk, juice. Goods (mug, pen, …) keep circulating by `give` as in 11d but are not sold, because nothing consumes them and a bought good would occupy one of six places in a person's hands for good (F-50, QS-48). | The item loop closes: produce → buy → give → eat/drink. |
| **SD-27** | **`market-town` gains configuration only.** `world.yaml`: `economy, employment, consumption` appended to `systems` (after `item-transfer`; each depends only on packs above it), an `organizations:` list (`cafe-company`, `corner-store` — keys distinct from the places', F-16), header paragraph. `organizations/<key>.yaml`: tags, note, `holdings:` (opening stock) and `economy:` (wallet, shop). Every person file: one appended `economy: { wallet }` block; `alice` (café, 05:30–14:00 — her routine's `work`) and `felix` (store, 08:00–13:00 — inside his routine's store part, and he starts there) one appended `job:` block (F-17, QS-46). No place file, no existing field or section, changes (`ARC-35` check 3). Sizes (endowments, prices, wages, production) are content, fixed by E-9's 300-day measurement against a criterion stated now (I-9; §9 E-6 gives the starting values). | CP-1's world delta stays configuration only. Fitting jobs to existing routines avoids changing Social Café's configuration. |
| **SD-28** | **The decisions are recorded before code.** `ARC-38` (SD-22 … SD-27, the money loop and how it is sized, the accepted limitations); an `ARC-35` dated note: the transformation's second merge carries six market packs in all (QS-35), so check 2's "market pack" and check 3's "the five market packs" read "the six"; MODULE_SPEC §4.1's section table gains `economy` and `job`; PACKAGE_FORMAT §8 names them; systems/README lists the three packs. | `CLAUDE.md` §2.2. |

### 4.5.3 Acceptance (decided before measuring, `ARC-23`)

Each guard names the mutation shown to break it. A mutation is applied to the working tree, observed to
fail by name, and reverted; `git status` and `git grep MUTATION -- systems` are recorded afterwards.

```text
E-1  The change set is inside AC-1 (I-1). `git diff --name-only <base>...HEAD` lists only the paths of
     §4.5.1's table; Cargo.lock gains exactly three [[package]] entries (mineworld-economy, -employment,
     -consumption), none with a `source`, all under systems/, and changes otherwise only the dependency
     lists of those packages and of mineworld-installed-systems. Guard: the recorded commands of E-C8.
     Mutation M-E1: a comment added to kernel/src/lib.rs in the working tree → the same command lists it.
E-2  Nothing outside the market moves (I-4). 300-day seed-7 social-cafe sha (all but `wall`) = E-0
     (ad49c723…c64b); `validate worlds/social-cafe` identical to the base's; every existing test passes;
     no existing test outside systems/inventory/tests is edited, and inventory's existing tests keep their
     claims (an edit there is listed with its unchanged claim).
E-3  Inventory still decides (I-3, ARC-26, CP-5). `items-produced` and `items-consumed` stated past the
     constructors — an undeclared kind, a count of 0, a non-holder, production past a person's six,
     consumption of more than is held — are refused FactRefusedByOwner and write nothing; through the
     constructors they change exactly one holder by exactly the count. Guards: systems/inventory/tests.
     Mutation M-E2: the items-consumed reduction skips admit_consumption → the undeclared-kind and
     non-holder cases fail (over-consumption alone may survive by Holdings::removing's checked_sub, as
     D-D2 found for transfers; recorded if so).
E-4  Only economy moves money; a wallet never goes negative (I-3, I-6, CP-5). A `money-transferred`
     stated past `buy` for more than the payer holds is refused FactRefusedByOwner, writing nothing; a
     `wage-due` against an employer whose wallet is short becomes `wage-unpaid`, both balances unchanged
     and no `money-transferred`; one the employer can pay becomes one `money-transferred` caused by the
     `wage-due`. Guards: systems/economy/tests. Mutation M-E3: economy's wage reaction pays without the
     balance check → the wage-unpaid test fails (the payment is refused by the reduction instead).
     Static, recorded not tested: employment cannot write a Wallet (INV-7 write tokens; it has none).
E-5  Buying works through the unchanged controller (CP-3 for buy). Hand-built world (presence,
     movement, item, inventory, economy; a shop place with an operator holding two kinds, one priced
     above a buyer's wallet): an observer in the shop is offered exactly one complete `buy` per priced
     kind, available iff stocked, affordable and carriable, otherwise TargetUnavailable; an observer
     elsewhere is offered none; an accepted buy moves one item operator → buyer and the price buyer →
     operator, both facts caused by the request (AC-9); a buy refused at dispatch for the same three
     reasons; driven by PacedRuleController on `mineworld run`'s schedule for 10 days people buy; two runs
     of one seed are byte-identical. Guards: systems/economy/tests. Mutation M-E4: offers built with
     Offer::new (incomplete) → the controller-buys test fails. Mutation M-E5: `purchasable` ignores the
     balance → the over-budget test fails.
E-6  Work is attendance, exactly (QS-8). Hand-built world (presence, movement, item, inventory,
     employment): a person present for the whole shift is paid wage × shift ÷ 3 600 and the employer
     receives each `produces` count in full; one who leaves at a known instant is paid for the time
     present only, and production is prorated (floor); one absent all shift gets `shift-started {present:
     false}`, `shift-ended {worked: 0}`, no wage-due and no production; a world stopped mid-shift, saved
     and resumed ends the shift with the same facts as the uninterrupted run (the schedule/persisted.rs
     pattern). Guards: systems/employment/tests. Mutation M-E6: the reaction ignores a departure from the
     workplace → the leaves-early test fails (paid for the whole shift).
E-7  Removability, each direction (AC-2, pack level; CP-6's pattern). economy installed without
     employment loads and sells; employment installed without economy states `wage-due` that nobody
     reduces, and the world runs with no fault; a world without consumption offers no `eat`/`drink`,
     answers them Unavailable, and holdings change only by gives, purchases and production; a world
     without economy offers no `buy` and answers it Unavailable. The registry refuses employment or
     consumption without inventory, naming inventory. Guards: the three packs' tests. Mutation M-E7
     (negative control): the "without consumption" world enables it → the no-offer assertion fails.
E-8  Consumption eats food and drinks drinks only. Offers: one per edible or drinkable kind held, none
     for goods, none to another person; `eat` of a drink and `drink` of a food are NoSupportedInteraction
     at dispatch; an accepted eat removes one through `items-consumed` caused by the request. Guards:
     systems/consumption/tests. Mutation M-E8: the category check removed from validate → the
     cross-category test fails.
E-9  market-town lives for 300 days with the money loop closed (CP-4's precondition, I-7, I-9, R-S9-2,
     R-S9-3) — ledger evidence, never a test inside the range (world-level tests are 11f's). In order:
       a. `mineworld validate worlds/market-town` → valid; ids 1–38 exactly 11d's; the organizations
          after them (39, 40); genesis = 11d's 101 + the organizations' stocked + one funded per wallet +
          one shop-opened per shop + one hired per job.
       b. Activity first (I-7). `run --seed 7 --days 300 --save`, then a scratch reader on the save
          (never committed, F-45's pattern), every condition stated here before the first run:
          - faults 0; every seat moved and talked in every 30-day bucket;
          - in every bucket: ≥ 1 purchase; each job holder paid ≥ 1 wage; ≥ 1 items-produced; ≥ 1 eat
            or drink; ≥ 1 give;
          - no employer drained: zero `wage-unpaid` in 300 days;
          - no wallet drained: no wallet — person or organization — is ever below the cheapest price in
            the town (QS-47's reading);
          - money conserved: the wallets' total at the end equals the genesis total;
          - nobody ever holds more than six.
       c. Only then determinism: two 30-day seed-7 runs print identical lines but `wall`; a save run to
          day 15 then resumed to 30 equals the uninterrupted 30-day save's history and fingerprint.
       d. Cost: the 300-day run's wall ≤ 60 s with --save (R-S9-2).
     If b fails, the remedy is content or pack design, re-measured against the same conditions — never
     the controller (I-9); a failure no content can fix is a material stop (R-S9-3).
     Mutation M-E9 (scratch branch, never pushed): `consumption` removed from market-town's systems →
     b fails: purchases stop once hands are full (QS-35's failure, seen by the instrument).
E-10 The world delta is configuration only (ARC-35 check 3, by hand until 11f automates it):
     `git diff --no-index worlds/social-cafe worlds/market-town` shows, beyond 11d's delta, only
     world.yaml's appended systems, organizations list and header; added organizations/ files; and, per
     person file, the appended `economy:` block (and `job:` for alice and felix). No place file differs.
E-11 The documents say it first (CLAUDE.md §2.2): ARC-38, the ARC-35 note, MODULE_SPEC §4.1,
     PACKAGE_FORMAT §8 and systems/README exist before the code; both doc checks pass.
E-12 11d's market-town evidence is superseded, and said so: MVP_STATUS's "Owning and giving things"
     capability row and its 11d evidence row stay true of 11d's merge; a new capability row ("Work, money,
     shops and eating") and evidence row carry E-9; the `worlds/market-town` artefact row is updated.
```

### E-C0 — Design (this section) — docs only

- [x] Implementation: §4.5.1 … 4.5.6, §8.6, §9 E-6/E-7, QS-39 … QS-53, §16, by the planning session on
  `mvp0/s9-11e-plan`.
- [x] Validation: both doc checks (§9 E-7).
- [x] Review: every file and symbol named here was read on `70e532f` (§8.6); the riskiest cross-pack
  flows were run end to end on a scratch branch before this design was finalized (E-6); the change set
  is shown inside AC-1 by that run; operator-material points are marked (§10). Self-review by the
  planning session only; the primary session's review is pending.

### E-C1 — Specs before code: ARC-38, the ARC-35 note, MODULE_SPEC §4.1, PACKAGE_FORMAT §8, systems/README

**Goal.** Ownership of money, work, production and consumption, how the loop is sized, and what the
AC-1 measurement now counts are reviewable before code. All Markdown: inside the range.

**Scope.** `docs/DECISIONS.md` — **ARC-38** *Work, money, shops and consumption*, appended after ARC-37
(`git fetch`, confirm ARC-38 free first): SD-22 … SD-27; options for closing the item loop (the
operator's QS-35 record; buy-back declined), for attendance (polling vs `person-entered-place`), for
authoring shops (place section vs operator's section, F-47); limitations (non-employees have no income;
no hunger or sleep; one rejection reason for a buy; shifts within a day). An **ARC-35 note** (six market
packs). `docs/MODULE_SPEC.md` §4.1: "four sections" → six (`economy`, `job`). `docs/PACKAGE_FORMAT.md`
§8: names the six. `systems/README.md`: the three packs.

- [x] Implementation: as scoped. ARC-38 appended after ARC-37 (SD-22 … SD-27 as items 1–6, the five
  option tables, the dependency diagram, limitations incl. L-13); the ARC-35 note "six market packs"
  appended after its 11b note; MODULE_SPEC §4.1 "four sections" → six (`economy`, `job` rows);
  PACKAGE_FORMAT §8 names the six; systems/README lists the three packs and their test commands.
- [x] Validation: both doc checks (49 ids distinct; 143 sections / 22 documents); ARC-38 absent from
  every `origin/*` branch after `git fetch` (§9.5 E-E1).
- [x] Review: no defined term redefined — `Organization`, `Item`, `Process` used as CORE_CONCEPTS
  defines them; "wallet", "shop", "job", "listing" are packs' components or views, not core terms;
  the section table matches SD-23/SD-24 (`economy` on people and organizations, `job` on people). One
  bounded refinement recorded as DE-1 (§4.5.7): `money-transferred`'s reduction also refuses a party
  that is not a living Person or Organization (SD-24 listed amount, zero and self), the same "living
  holder" rule inventory applies; ARC-38 states it.

### E-C2 — `systems/inventory`: produced and consumed

**Scope.** `systems/inventory/src/{event.rs, admit.rs, system.rs, lib.rs}`, `tests/inventory.rs` (new
cases; existing ones unchanged), README. No dependency change.

- [x] Implementation: SD-22 — `ItemsProduced`, `ItemsConsumed` (accessors, `pub(crate)` constructors),
  `admit_production`, `admit_consumption`, `produce`, `consume`; declaration emits and subscribes to both;
  two reduction arms, each asking its admit function before writing. Files: `systems/inventory/src/
  {admit,event,lib,system}.rs`, `README.md`, `tests/{inventory.rs, support/mod.rs}`. Both facts are
  `Visibility::Participants` with the holder as the only participant (a private `held_fact` helper).
  The tests' stater for the new facts is a second test-only system, `workshop` (action `change`), so
  `hands` and every existing test's world are unchanged (DE-2).
- [x] Validation: `cargo test -p mineworld-inventory` → inventory.rs 10 passed (8 existing + 2 new),
  persisted.rs 1 passed, 0 failed; clippy `--all-targets -D warnings` and fmt clean. E-3 and M-E2 in
  §9.5 E-E2 (M-E2 caught by the count-zero case, not the undeclared-kind and non-holder cases: DE-3).
- [x] Review: one write path per fact (`react`: produced → `adding`, consumed → `removing`, each after
  its admit function); `admit_production` calls `can_take`, so a person's six holds for production
  (the forged "past six" case and the positive control "up to six exactly" show it); `transfer`,
  `admit_transfer`, `admit_stock` and `stocked` byte-for-byte unchanged; no existing test edited —
  the diff of `tests/inventory.rs` adds imports and two functions only.

### E-C3 — `systems/employment`

**Scope.** `systems/employment/{Cargo.toml, README.md, src/{lib,system,section,event,component,process,
codec}.rs, tests/{employment.rs, persisted.rs, removable.rs, support/mod.rs}}`, laid out as `schedule`.
Dependencies: authoring, contracts, kernel, presence, sdk, schedule (TimeOfDay only), serde, serde_json
(`workspace = true`); `mineworld-inventory = { path = "../inventory" }`; dev: item (path), movement,
persistence, serde-saphyr.

- [x] Implementation: SD-23 — `Job`, `Produces`, `Employment` + `OnShift` (`owned_component!`
  `employment`; the shift in progress is `Option<OnShift>`, so "on shift" and its spans cannot
  disagree), `employed_by()` / `employed_by_declaration()` (Person → Organization), `ShiftProcess` /
  `ShiftState`, `Hired`, `ShiftStarted`, `ShiftEnded`, `WageDue` (public accessors — economy decodes
  it), the `job:` section (`references`: Organization, Place, Items; `#[serde(try_from)]` refuses
  `until ≤ from` as it is decoded, `EmploymentError::ShiftOutOfOrder`), react (hired, shift-started,
  shift-ended, person-entered-place), wake, disclosure to the employee, `BIOGRAPHICAL = [hired]`.
  Files: `systems/employment/{Cargo.toml, README.md, src/{lib,system,section,event,component,error,
  process,codec}.rs, tests/{employment.rs, persisted.rs, removable.rs, support/mod.rs}}` — `error.rs`
  added beside §4.5's list, as schedule has one (DE-4). Wage `wage × worked ÷ 3 600` and production
  `per_shift × worked ÷ shift` computed in `u128`, floored, no float.
- [x] Validation: `cargo test -p mineworld-employment` → employment 5, persisted 1, removable 3 passed, 0
  failed; clippy `--all-targets -D warnings`, fmt clean. E-6, the employment half of E-7, M-E6 in §9.5
  E-E3.
- [x] Review: names no Wallet and no economy in code or manifest (grep: comments only, saying it does
  not); wage arithmetic integer; a wake reschedules exactly once per branch (start → `until`, end →
  `from`), shown by the second day's shifts at the same times; `items-produced` only through
  `mineworld_inventory::produce`; production refused by inventory (an undeclared kind) is skipped,
  the wage still due — the only refusal possible for an organization (DE-5).

### E-C4 — `systems/economy`

**Scope.** `systems/economy/{Cargo.toml, README.md, src/{lib,system,section,event,component,action,offer,
codec}.rs, tests/{buy.rs, wages.rs, removable.rs, paced.rs, support/mod.rs}}`. `mineworld-inventory` and
`mineworld-employment` by path (employment for `WageDue`'s type only, `ARC-28`); dev: item (path),
movement, rule-controller (`workspace = true`, D-6's direction), serde-saphyr.

- [x] Implementation: SD-24 — `Wallet`, `Shop`, `Price`, `Listing`/`Listed` (the typed view a shop's
  place discloses), `Funded`, `ShopOpened`, `MoneyTransferred`, `WageUnpaid`, `Buy`, `buy_requirement`,
  `purchasable`, `admit_payment`/`balance` (`money.rs`, DE-6), the `economy:` section (shop only on an
  organization; references Place and Items), validate/resolve, react (funded, shop-opened,
  money-transferred, wage-due), offers, disclosure (wallet to holder; listing to the place's
  perceivers). Files: `systems/economy/{Cargo.toml, README.md, src/{lib,system,section,event,component,
  action,offer,money,codec}.rs, tests/{buy.rs, wages.rs, removable.rs, paced.rs, support/mod.rs}}`.
  The reduction refuses a second `funded` for one holder and a second shop in one place (DE-7).
- [x] Validation: `cargo test -p mineworld-economy` → buy 6, paced 3, removable 3, wages 1 passed, 0
  failed; clippy `--all-targets -D warnings`, fmt clean. E-4, E-5, the economy half of E-7; M-E3,
  M-E4, M-E5 in §9.5 E-E4.
- [x] Review: `depending_on([inventory, presence])`, never employment (asserted by removable.rs on the
  declaration); `subscribing_to::<WageDue>`; the only `insert`s of Wallet and Shop are in `react`; no
  `f32`/`f64` (u64 throughout, `checked_add` for the payee); offers only when the observer stands in
  a place with a Shop, none with a target. The wage answer and `purchasable` each ask
  `admit_payment` themselves, so the reduction's check is the second guard (M-E3 shows it).

### E-C5 — `systems/consumption`

**Scope.** `systems/consumption/{Cargo.toml, README.md, src/{lib,system,action,offer,codec}.rs,
tests/{eat.rs, removable.rs, paced.rs, support/mod.rs}}`. `mineworld-inventory`, `mineworld-item` by path.

- [x] Implementation: SD-25 — `Eat`, `Drink`, `EATEN = "food"`, `DRUNK = "drink"`, `read_meal` (the
  action type picks the payload and the category it needs), `meals` offers (complete, no target,
  `SpatialRequirement::NONE`, in item order), validate (payload → living Person, no target →
  category matches → `admit_consumption`), resolve (`consume` only). Files: `systems/consumption/
  {Cargo.toml, README.md, src/{lib,system,action,offer,codec}.rs, tests/{eat.rs, removable.rs, paced.rs,
  support/mod.rs}}`.
- [x] Validation: `cargo test -p mineworld-consumption` → eat 3, paced 2, removable 2 passed, 0 failed;
  clippy `--all-targets -D warnings`, fmt clean. E-8, the consumption half of E-7, the 10-day paced
  run; M-E7, M-E8 in §9.5 E-E5.
- [x] Review: declares no component (`owning` absent); `resolve` returns exactly
  `mineworld_inventory::consume`'s emission; goods fall through `meals`' match and are refused at
  validate; the category is read from item's `ItemKind`, never stored here.

### E-C6 — Install: three lines each in `systems/installed`

- [x] Implementation: `Cargo.toml` three `path` lines; `src/lib.rs` `Economy`, `Employment`,
  `Consumption` after `ItemTransfer`; `Cargo.lock` regenerated by the build (+3 names in
  mineworld-installed-systems' dependency list; the three path packages arrived with E-C3 … E-C5).
- [x] Validation: `cargo test -p mineworld-installed-systems -p mineworld-worldpack` all passed; E-2's
  sha = E-0 and validate byte-identical (§9.5 E-E6).
- [x] Review: `git diff --stat` 3 + 3 lines and Cargo.lock +3; no root manifest edit was needed.

### E-C7 — `worlds/market-town`

- [x] Implementation: SD-26, SD-27 at those starting sizes (they are §9 E-6 run 3's, which the text
  above calls run 2's starting values plus run 3's two changes): people 200 000, alice and felix
  20 000; café 20 000, store 300 000; alice 120/h at the café 05:30–14:00, felix 200/h at the store
  08:00–13:00; per shift café coffee 4, tea 2, croissant 3, cake 2, sandwich 2, soup 2, store apple 4,
  bread 2, milk 2, juice 2; prices café 250–600, store 100–300; opening stock café 58, store 50. Files:
  `worlds/market-town/{world.yaml, organizations/{cafe-company,corner-store}.yaml, people/*.yaml}`,
  every block commented. Content commit 1611c1a.
- [x] Validation: E-9 a, then b (one 300-day sizing run; it passed, so nothing was re-sized), then c,
  then d; E-10; M-E9 — all in §9.5 E-E7. 300-day market-town runs used: 2 of 4 (E-9, M-E9).
- [x] Review: no place file or existing line changed (E-10's removals are 11d's README and id/name
  only); every authored person within capacity (max 3); every key resolves (validate). The world.yaml
  header paragraph above Social Café's header is market-town's own (11d's), rewritten for 11e.

### E-C8 — Close: status, change set, full gates, ledger

- [x] Documentation: `docs/MVP_STATUS.md` rows of E-12 (DE-8); §4.5 checkboxes; §9.5 `E-E0` … `E-E7`,
  `E-E-final`; §4.5.7 DE-1 … DE-10; the handoff; worlds/market-town/README.md.
- [x] Validation, once, on the final executable head 15c4651: all PASS (§9.5 E-E-final).
- [x] Review: E-1 … E-12 each with evidence (E-E-final's table); deviations DE-1 … DE-10 (§4.5.7); the
  PR is to be merged **with a merge commit** (ARC-35 reads `M^1..M`), said in the PR body.

### 4.5.4 Test ownership for 11e

```text
STATIC      fmt, clippy -D warnings; installed! bounds; INV-7 write tokens (employment and consumption
            cannot write Wallet or Holdings — they have no token)
UNIT        inventory's admit_production/admit_consumption refusals; economy's purchasable and money
            reduction; employment's attendance arithmetic and the job section's refusals (E-3, E-4, E-6)
INTEGRATION hand-built worlds through the kernel's real dispatch, presence's real observe() and real
            processes: owner refusals of forged facts (E-3, E-4); buy offered, validated, resolved,
            reduced, and attempted by the unchanged PacedRuleController (E-5); shifts with arrivals and
            departures, and a persisted mid-shift restart (E-6); AC-2 in each direction (E-7); eat and
            drink (E-8); every existing test (E-2)
REAL RUN    validate; the 300-day seed-7 market-town run with its scratch reader; two 30-day runs; a
            15+15 resume; the 300-day social-cafe comparison (E-2, E-9)
GATE 1      NOT REQUIRED — no model
GATE 2      the real runs above
CI          none configured (S13); the full local gate once on the final head
```

### 4.5.5 Is any of this material?

- **No framework change is needed.** The spike (E-6) ran every flow the operator named — the shift
  Process reading presence and stating `wage-due`, economy reducing it into a wallet move with no system
  dependency, a purchase at a shop stated through inventory's checked constructor and paid through
  economy, consumption through inventory's checked constructor, all attempted by the unchanged paced
  controller through complete affordances — with no edit outside `systems/**`, `worlds/**` and
  `Cargo.lock`. No precursor is proposed.
- **Two framework limits are worked within, not changed** (F-47 one section per pack; F-48 one
  pack-supplied unavailability reason). Both are bounded inside the packs; each is raised so the primary
  session sees the shape it forces (QS-43, QS-44).
- **Operator-material:** QS-45 (economy discloses a shop's stock count, read from inventory's state, to
  everyone in the shop — an `INV-13` reading), QS-47 (how "no wallet drained" is read, and that people
  without a job live on endowments for the 300 days), QS-39 (the pack adds `drink` beside MVP §5's `eat`).
  QS-43 changes the authoring format §4.5's medium scope proposed (shops on the operator, not the place).
- **New ownership, approved in QS-7 and QS-35:** Wallet, Shop (economy); Employment, `employed-by`, the
  shift Process (employment); consumption owns nothing. Inventory's ownership grows by two facts of its
  own vocabulary (QS-42); no existing ownership moves.
- Nothing changes a public contract, `kernel/`, `contracts/`, or a frozen invariant.

### 4.5.6 What 11e supersedes of 11d's evidence

11d's market-town measurements (§9.4 E-D6: 17 639 gives, 30 items conserved, Otto ending with six,
37 090 facts in 30 days, 32.5 s) describe 11d's merge and stay true of it; they are not claims about
market-town after 11e, where items are produced, bought and consumed and are no longer conserved. 11e's
E-9 replaces them as the current market-town evidence. 11d's per-seat give criterion (D-9 b) is not
carried forward as a criterion: CP-4 is world-level, and gives continue (E-6: ≥ 1 196 per bucket) beside
purchases and meals; per-seat gives are reported, not required. The social-cafe evidence (E-0) is not
superseded: E-2 requires it unchanged.

### 4.5.7 Deviations and discoveries during implementation (11e session)

Deviation ids are `DE-<n>` (11d's were `D-D<n>`; `E-D<n>` are 11d's evidence ids).

```text
DE-1  bounded  money-transferred's reduction also refuses a party that is not a living Person or
               Organization (SD-24 listed only "more than the payer holds, zero, payer = payee"). Reason:
               a Wallet lives on holders only, as inventory's Holdings do (is_holder); without it a
               forged fact could create a wallet on a place. Stated in ARC-38 item 3. Validation:
               economy's forged-money test.
DE-2  bounded  inventory's tests state the new facts through a second test-only system, `workshop`
               (action `change { holder, item, count, way: Make | UseUp, forged }`), not by extending
               `hands`. Reason: `hands`' declaration is part of every existing test's world; a new
               system leaves those worlds byte-identical (E-2: existing tests keep their claims).
DE-3  bounded  M-E2 (items-consumed's reduction skips admit_consumption) is caught by the count-zero
               case, not by the undeclared-kind and non-holder cases §4.5.3 E-3 named: those are
               still refused because a holder can only hold declared kinds and a non-holder holds
               nothing, so Holdings::removing returns None (D-D2's second guard, again). A zero count
               is the case only the owner's rule refuses — removing 0 is a no-op write. The mutation is
               caught; no test weakened.
DE-4  bounded  employment has an `error.rs` (EmploymentError::ShiftOutOfOrder) beside §4.5 E-C3's file
               list, so `until ≤ from` is refused with the pack's own message as the section is
               decoded (schedule's pattern, ScheduleError). No behaviour beyond SD-23.
DE-5  bounded  At a shift's end, a production line inventory's `produce` refuses is skipped, not a
               failed wake. An organization is unbounded, so the only refusal is an undeclared kind (an
               item file without `item:`); failing the wake would stop the world for content no pack
               trades. The wage is still due. Validation: by construction (no test world authors an
               undeclared produced kind; market-town's `validate` names every kind).
DE-6  bounded  economy has a `money.rs` (admit_payment, balance, the private `pay` constructor) beside
               §4.5 E-C4's file list: the one payment rule asked by `purchasable`, by the wage answer
               and by the reduction (inventory's admit.rs pattern). `Listing`/`Listed` are typed structs
               in component.rs, so the disclosed listing is not an untyped JSON blob (CLAUDE.md §4
               rule 7).
DE-7  bounded  economy's reduction also refuses a second `funded` for a holder that already has a
               Wallet, and a `shop-opened` for a place that already has a Shop or whose operator is not a
               living Person or Organization. SD-24 did not say. Reason: both facts are genesis facts
               from a pack's own section, so a repeat is a defect; two shops in one place would make
               "the shop here" ambiguous. Validation: by construction; no test world repeats either.
DE-8  bounded  MVP_STATUS's `worlds/market-town` artefact row is updated here, because E-12 requires it;
               §16's POST-MERGE SYNC line gives the artefact line to the planning session. The
               `Updated:` line and the S9 row are left to the planning session. The 11d capability row's
               "No money, work or shops yet" became a pointer to the new row; its 11d evidence row is
               kept and marked superseded as current evidence (QS-51, §4.5.6).
DE-9  process  The handoff edit made on the scratch branch was swept into a local scratch commit by
               `git commit -a` and deleted with the branch; the handoff was rewritten at E-C8. Nothing
               scratch reached origin (`git ls-remote --heads origin | grep -c scratch` → 0). No
               `python3 -c`, `sed -i`, `awk`, `xargs` or `curl` was used in this session.
DE-10 observ.  E-9 b passed on the first sizing run, with the spike's run-3 sizes unchanged; the numbers
               equal §9 E-6 run 3 exactly (372 755 facts, buy 2 710, eat 1 710, drink 1 007, give
               13 032). The store's shelf is empty at day 300 (it sells each morning's production the
               same day); purchases there continue every bucket. Recorded, not a criterion.
```

QS-51 (what 11e supersedes): §4.5.6 stands as written. 11d's market-town measurements (§9.4 E-D6) and its
MVP_STATUS evidence row describe 11d's merge and stay true of it; §9.5 E-E7 replaces them as the current
market-town evidence. Social Café's E-0 is not superseded (E-2 re-measured it unchanged).

QS-47 (L-13, the bounded-horizon economy): recorded in ARC-38's accepted limitations and in market-town's
world.yaml header: ten of twelve people have no income; their lowest wallets over 300 days are 95 150 …
200 000 of 200 000 (E-E7), so the measured horizon holds with margin; past it they run out.

## 4.6 PR 11f — the proof (full design; DESIGN FROZEN 2026-10-07)

### 4.6.0 Freeze record (primary session, 2026-10-07)

The 11f design is frozen and its execution contract (§17) is confirmed. These rulings bind and
override any other text in §4.6 and §17.

- **QS-54: accepted by the operator (2026-10-07).** ARC-35 check 2's "no dependency path" rule
  counts normal and build edges only. Dev edges stay bound by check 2's other two rules: every
  dependent of a market pack lives under `systems/`, and no code outside `systems/`, `worlds/` and
  `tests/acceptance/` names a market crate. Record this as part of 11f's ARC-35 note (P-C1), quoting
  the operator's decision.
- **QS-65: accepted, worded precisely.** At 11f's merge:
  - AC-1 is recorded as **demonstrated as ARC-35 measures it, within ARC-33's static-linking
    boundary**;
  - Market Town composition becomes ✅;
  - Milestone C is "demonstrated, awaiting the operator's review".

  The operator's own acceptance of AC-1 and Milestone C is asked for at the S9 closeout, with a
  short runnable checklist. It is never assumed.
- **QS-55 to QS-64 and QS-66: accepted as recommended.** That includes:
  - the full 300-day activity test, default-on (QS-59);
  - market facts read by type name, never through a market crate (QS-58);
  - Milestone C at the café after a 2-day run (QS-62).
- **11f is not part of the measured transformation.** It may touch the paths §4.6 lists. It must
  change no pack, no world (except market-town's README) and no framework behaviour. A proof that
  needs a behaviour change is a material stop.
- **The merge is a merge commit.** After the merge, the AC-1 test must pass on `main` itself, in
  the primary session's own run.

### 4.6.0 Freeze record

Not frozen. Detailed by the planning session on `mvp0/s9-11f-plan` from `main @ 2dddda8` (11e merged).
The operator-material questions are QS-54 and QS-65 (§10); the others are the primary session's. A
freeze record written here by the primary session binds and overrides any other text in §4.6 and §17.

The medium scope written before 11e merged is kept at the end of this section (§4.6.7), so a reviewer
can see what the detailing changed. Where it and §§4.6.1 … 4.6.6 disagree, §§4.6.1 … 4.6.6 govern, and
each change is named there with its finding (§8.7) and question (§10, QS-54 …).

### 4.6.1 Identity, base, approved scope

```text
PR            11f — the proof (S9, sixth and last; outside the AC-1 range by design, ARC-35 point 1)
base          main @ 2dddda8 (11e merged) plus the docs-only merges named at freeze (#47, and this
              planning branch once merged). Re-audit §8.7 if anything under tests/acceptance/,
              tools/cli/tests/, tools/cli/src/run.rs, persistence/src/, server/src/protocol.rs,
              worlds/ or systems/ moved
branch        mvp0/pr-11f-proof, in its own worktree, held by the implementing session only
audit         §8.7 (2dddda8), including the planning measurement §9 E-8
scope         §1.1 PR 11f as refined by SD-29 … SD-37; CP-1, CP-4, CP-6 (for every market pack),
              CP-7; QS-11 (CP-4's precondition before determinism), QS-12 (Milestone C as a real-server
              test plus the operator's runnable list) and QS-37 (per-seat evidence leaves the scratch
              readers for a committed test), all approved earlier and carried out here; QS-54 … QS-66
              as answered
depends on    11a … 11e, all merged; the two transformation merges 11d 70e532f (#43) and 11e 2dddda8 (#46)
```

**Goal.** Prove what S9 built, mechanically, through the command a person types and the server a client
joins, and change nothing that is proven:

1. **AC-1, the primary criterion**, by `ARC-35`'s three independent checks as a committed test in
   `tests/acceptance`: the change set of the two transformation merges, the dependency structure from
   `cargo metadata`, and the configuration-only world delta. Each check fails closed when it cannot
   run, and each is shown to bite.
2. **CP-4, the market lives**, as a committed test that reads a real 300-day save: 11e's E-9 b
   conditions and 11d's give conditions, **activity before determinism** (I-7), then AC-11, AC-12 and
   AC-6 for Market Town.
3. **AC-2 at world level for every market pack** (CP-6, widened from item-transfer to the six by
   QS-61): each pack removed from Market Town leaves a world that runs, or is refused, exactly as the
   declared dependencies say.
4. **Milestone C** (CP-7, `HUMAN_REVIEW_QUEUE.md`): *work → earn → buy → holdings change → another
   client perceives it → it survives a server restart*, through the real server and real sockets, in
   the `milestone_b.rs` pattern.

**11f is outside the measured range, and changes no behaviour.** It adds tests and documentation. It
must not change any System Pack, any world's configuration, or any framework crate's behaviour: if the
proof turns out to need a behaviour change, that is a **material stop** (I-1's spirit, I-8, I-9),
reported with evidence, never a quiet edit. Every path this PR may touch:

```text
tests/acceptance/Cargo.toml             two dev-dependencies: mineworld-worldpack, serde-saphyr (both
                                        `workspace = true`, already workspace entries; SD-32)
tests/acceptance/src/lib.rs             the doc table gains ac1_composability (a comment)
tests/acceptance/tests/ac1_composability.rs   new: the AC-1 test (SD-29 … SD-32)
tools/cli/tests/market/mod.rs           new: the shared Market Town reader (SD-33)
tools/cli/tests/market_town.rs          new: CP-4, then AC-11 / AC-12 / AC-6 (SD-34)
tools/cli/tests/market_composition.rs   new: AC-2 at world level for the six packs (SD-35)
tools/cli/tests/milestone_c.rs          new: Milestone C (SD-36)
tools/cli/tests/fixture/mod.rs          one new function, `assert_quiet_in` (SD-36); the existing
                                        `assert_quiet` and its callers unchanged
Cargo.lock                              mineworld-acceptance's dependency list gains two names; nothing
                                        else (F-69)
docs/DECISIONS.md                       ARC-35 note (11f) (SD-37)
docs/MVP_STATUS.md, docs/HUMAN_REVIEW_QUEUE.md        status and Milestone C (SD-37)
worlds/market-town/README.md            human orientation: the proof and how to run it (Markdown only)
.structured-coding/plans/mvp0/{step-10-market,handoff}.md   this ledger, the handoff
```

No path under `systems/`, under `worlds/` other than that README, nor `kernel/`, `contracts/`,
`persistence/`, `server/`, `cognition/`, `sdk/`, `authoring/`, `worldpack/`, `tools/cli/src/`, `clients/`
or the root `Cargo.toml` (F-59: `tests/acceptance` is already a member). The existing tests under
`tools/cli/tests/` other than `fixture/mod.rs`'s addition are not edited.

**Non-goals.** No fix to F-47, F-48, L-12, L-13 or relationship saturation (closeout items, §4.6.6); no
needs pack, no `sleep` (QS-10); no client change and no client test (QS-13; the 2D client's use of
complete affordances is S12's); no CI workflow (S13 — but the AC-1 test's need for full history is
recorded for it, §4.6.6); no `mineworld` command or flag (a world-level test uses the commands that
exist); no change to how any market pack behaves or how Market Town is configured (a failing proof is
a material stop, not a retune — I-9).

### 4.6.2 Design (SD-29 … SD-37)

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-29** | **The AC-1 test is one file, `tests/acceptance/tests/ac1_composability.rs`, with one `#[test]` per check**, so a failing check never hides another: `check_1_the_change_set`, `check_2_the_dependency_structure`, `check_3_the_world_delta`. It holds two tables, both literals: `TRANSFORMATION` — `{ pr: "11d", number: 43, branch: "mvp0/pr-11d-owning-things", merge: "70e532f383ff81464d00066ff85e1b7fdc2296b0" }` and `{ pr: "11e", number: 46, branch: "mvp0/pr-11e-work-money-shops", merge: "2dddda86a4347d54de7bd2303d5268a421870b5f" }` — and `MARKET_PACKS`, the six pack ids (`item`, `inventory`, `item-transfer`, `economy`, `employment`, `consumption`; ARC-35's 11e note), from which the crate names `mineworld-<id>` and `mineworld_<id with _>` are derived. A guard test asserts that each listed pack is a directory `systems/<id>/` whose `Cargo.toml` names that crate, so the list cannot go stale silently. Each check's core is a function over plain inputs (paths, lock text, metadata, YAML values), unit-tested on hand-written inputs, and called by its `#[test]` with what the repository holds. | `ARC-35` items 1–5 and its two notes; `ARC-23` (three independent checks); SD-15. The precursor scan's file is the template (`precursor_vocabulary.rs`: `git` through one helper that names its failure; fail closed; unit tests on text). |
| **SD-30** | **Check 1, the change set, from history, fail closed.** (a) The repository must be a work tree and **not shallow** (`git rev-parse --is-shallow-repository` = `false`); a shallow clone fails naming the missing history and the remedy (`fetch-depth: 0`). (b) Each row's merge is found on `git log --first-parent --merges HEAD` by its GitHub subject, `Merge pull request #<number> from <owner>/<branch>`: exactly one match, or the test fails naming zero or both; the match's id must equal the recorded `merge`, and it must have exactly two parents (a squash would have one, and is refused, as `ARC-35` item 7 says of the scan). (c) `git diff --name-only --no-renames M^1 M` ⊆ allowed: `systems/**`, `worlds/**`, `Cargo.lock`, and `**/*.md` under `docs/`, `systems/`, `worlds/`, `.structured-coding/plans/`; every other path is named. (d) **`Cargo.lock`:** both versions are read with `git show <rev>:Cargo.lock` and parsed by a line reader of the `[[package]]` blocks (name, version, source, dependencies; no TOML dependency is added, F-69). Every package added, removed, or changed in any field between `M^1` and `M` must have no `source` and be a crate under `systems/` at `M` — found by `git grep` at `M` for `name = "<name>"` in `systems/*/Cargo.toml` — or the test fails naming the package and what changed. | `ARC-35` items 1, 2, 5. The subject **and** the id (QS-55): the subject proves the id is that PR's merge on `main`'s first-parent chain; the id pins it, so a later merge with a recycled branch name cannot take its place. "Changed in any field" is `R-S9-6`'s wording ("fails on any changed non-path package"), tighter than item 2's "added" and "dependency list changed" (QS-56). |
| **SD-31** | **Check 2, the structure, from `cargo metadata --no-deps --format-version 1 --offline` at the working tree**, run through the `cargo` that built the test (`env!("CARGO")`); a `cargo` that cannot run, or output that does not parse, fails naming it. `--no-deps` is enough and is offline: only a workspace member can depend on a path crate, so every path to a market pack runs through workspace members, whose declared dependencies (`name`, `kind`, `path`) the output lists (F-60: full metadata wants Windows-only crates that are never downloaded here). Three bullets, each naming what it finds: (1) every workspace crate that declares a market pack as a dependency, **of any kind** (normal, build or dev), lives under `systems/`; (2) no path over **normal and build** dependency edges leads to a market pack from `mineworld-kernel`, `-contracts`, `-persistence`, `-server`, `-authoring`, `-sdk` or `-rule-controller` (the path is printed); (3) no `*.rs` or `Cargo.toml` outside `systems/`, `worlds/` and `tests/acceptance/` — tracked files and untracked files Git does not ignore, read from the working tree so an uncommitted edit is seen — contains a market pack's crate name as a word (`mineworld-economy`, `mineworld_economy`, …; `mineworld-item` does not match inside `mineworld-item-transfer` by accident, both being market names). | `ARC-35` item 3. Bullet 2's edge reading is **QS-54, operator-material**: `persistence`'s *test* `kill_and_resume` dev-depends on `worldpack`, which links the installed set and so the market packs (F-58). Counting dev edges would fail AC-1 for a test that loads a World Pack — the composition `ARC-33` built — while nothing in `persistence` knows the market; bullets 1 and 3 still see any dev dependency on, or naming of, a market pack. |
| **SD-32** | **Check 3, the world delta, structurally.** Both packs are first read with `WorldPack::read` (a pack that does not read fails the check, naming the refusal). Then each YAML file of both packs is parsed with `serde-saphyr` into `serde_json::Value` (supported by the parser for exactly this, F-62) and compared: `world.yaml` — `world.id` and `world.name` may differ; `systems` is Social Café's list, in order, followed by exactly the six market packs (as a set; their order among themselves is the registry's to judge, and `validate` already does); `places`, `population` and `seats` are equal; `items` and `organizations` are absent in Social Café and present in Market Town; no other top-level key differs. Every file under `places/` and `people/`: the same set of files; every top-level key of Social Café's file is present in Market Town's with an equal value; every key Market Town adds is a section whose owner, by the build's own `Capability::owning_section`, is one of the six market packs. `items/` and `organizations/` exist in Market Town only. `README.md` is not configuration and is not compared. Every difference is named by file and key. | `ARC-35` item 4 with its 11e note (six packs). `WorldPack::read` hands sections over decoded as opaque `AuthoredContent` (F-62), so the comparison is of the authored values, which is what "configuration" means; the section's owner comes from the build, not from a list in the test. Two dev-dependencies are added to `tests/acceptance` (F-69). |
| **SD-33** | **World-level tests read market facts and components by their type slugs, through test-local mirrors — never through a market crate.** `tools/cli/tests/market/mod.rs` holds `MARKET_TOWN` (the pack path), `run_market(seed, days, save)`, mirrors of the ten payloads the tests read (`funded`, `shop-opened`, `money-transferred`, `wage-due`, `wage-unpaid`, `hired`, `stocked`, `items-transferred`, `items-produced`, `items-consumed`) and of the `wallet`, `holdings` and `shop` (listing) components — each a `#[derive(Deserialize)] #[serde(deny_unknown_fields)]` struct over `mineworld-contracts`' id types, decoded from `EventRecord::payload()` only after `event_type()` matches the literal slug and `schema_version()` is 1 — and a `Ledger` that replays wallets and holdings from those facts. | `ARC-35` check 2 bullet 3 forbids naming a market crate outside `systems/`, `worlds/`, `tests/acceptance/`, and a test that runs the `mineworld` binary must live in `tools/cli` (`CARGO_BIN_EXE_mineworld` exists only there) (F-61, QS-58). Literal slugs and shapes are also the independent oracle rules §25 asks for; `deny_unknown_fields` and the version check make a changed shape a loud failure, not a test that keeps passing. |
| **SD-34** | **CP-4 is `tools/cli/tests/market_town.rs`, at the full 300-day horizon, in the default suite (no `#[ignore]`)** (QS-59). One test, in the order I-7 binds: the 300-day seed-7 run with `--save`, the two 30-day runs and the killed-and-rerun 30-day run are started together (the `run.rs` pattern); then **activity is checked first**, on the 300-day save — the conditions of P-5 — and on the 30-day control — the same conditions over its one bucket; **only then** are runs compared: the two 30-day saves byte-identical (facts, journal, snapshots), and the 30-day run SIGKILLed after day 15 and run again equal to the uninterrupted one byte for byte (the `run_restart.rs` pattern). Money conservation and the capacity are checked against the **state** in the save's newest snapshot, not against the facts alone (F-63). | CP-4 is a 300-day claim and `L-13`'s bounded horizon is exactly the measured 300 days, so a shorter committed horizon would let content that drains after its end pass. `run.rs` already runs three 300-day Social Café runs in the default suite; the market run costs ~34 s of a ~3-minute gate, in parallel with the 30-day runs. An ignored test is a test not run (§3.1 of `CLAUDE.md`). Replaying money from `money-transferred` alone would conserve it by construction; comparing with the wallets economy actually wrote is what can see a payer that was never debited (the primary session's 11e mutation). |
| **SD-35** | **AC-2 at world level is `tools/cli/tests/market_composition.rs`, one copy of Market Town per market pack** (the `social_composition.rs` pattern): the pack's id removed from `world.yaml`'s `systems`, and its section — `item:`, `holdings:`, `economy:`, `job:`; `item-transfer` and `consumption` own none — removed from every file that carries it, so the only difference is the pack. Expected, from the declared dependencies (§2.6, `ARC-37`, `ARC-38`): **refused** — without `item`, `validate` fails naming `inventory`'s missing dependency `item`; without `inventory`, it fails naming `inventory` as a dependency of a pack still enabled. **Runs** — without `item-transfer`, `economy`, `employment` or `consumption`, `validate` succeeds and a 30-day seed-7 run has 0 faults and every seat moving and talking in its bucket, with the pack's interactions absent and the others present (P-7's table). | MVP §9's AC-2 names `ItemTransferSystem`; QS-61 applies the same question to every pack the transformation installed, because "independently installable" is only half shown if removal is not. A refusal by name is the honest answer for a pack others depend on (`INV-10`, the registry's dependency rule), not a failure of AC-2. |
| **SD-36** | **Milestone C is `tools/cli/tests/milestone_c.rs`**, in `milestone_b.rs`'s shape. `run worlds/market-town --headless --seed 7 --days 2 --save K`; in K, **work and earn are located**: alice's `hired`, a `shift-started { present: true }` and `shift-ended { worked > 0 }`, a `wage-due` for her and the `money-transferred` to her caused by it (`Causation::Event`). `fixture::assert_quiet_in(market-town's people, head, 3 600, …)` — a new function reading every `from: "HH:MM"` **and** `until: "HH:MM"`, so job shifts count as well as routines (F-64) — then `server worlds/market-town --save K`. Clients **alice** and **bob** join; each walks from where the save left them, through the doorways the pack declares (read with `WorldPack::read`), along the street and into the **café** — the shop alice works for (F-65) — with `move` strides the server accepts one by one. Alice's own disclosed `wallet` equals the `Ledger`'s balance for her at the head (earned wages included). If she carries six, she first submits one offered, available `eat` or `drink` unchanged (F-66). Bob reads the café's `shop` listing. Alice submits one **available complete `buy` affordance unchanged** (`PROTOCOL.md` §6) and it is accepted with exactly two events. Then: alice's wallet is lower by the price, her holdings higher by one of that kind; **bob's next observation shows that kind's `in_stock` one lower**, and discloses neither alice's `wallet` nor her `holdings` (INV-13). The server is SIGKILLed and started again on K: the same `instance`, the same `revision` for both, alice's wallet and holdings and bob's listing unchanged. Finally `inspect K` resolves every cause. | CP-7 and `HUMAN_REVIEW_QUEUE.md`'s Milestone C, literally. The work and the wage happen in the saved run because a hosted clock runs one world second per wall second and a shift is hours long; the buy, the perception and the restart happen through the protocol, which is the claim. The café, because the store sells out each day (E-E7: empty at day 300) and alice buying from her own employer is the loop *run a shop* closes. |
| **SD-37** | **Documents.** `docs/DECISIONS.md`: an **ARC-35 note (11f)** — the two merges by subject and id, the Cargo.lock reading, check 2's edge reading as answered (QS-54), check 3's structural comparison, that world-level tests read the market by slugs, the 300-day horizon, and the evidence. `MVP_STATUS.md`: the Market Town composition axis, the S9 row, an evidence row per proof, the `worlds/market-town` artefact. `HUMAN_REVIEW_QUEUE.md`: Milestone C demonstrated, awaiting the operator, with a "how to see it for yourself" section and its launch commands. `worlds/market-town/README.md`: the proof and how to run it. Overall §7 and the step header are the planning session's (POST-MERGE SYNC). | `CLAUDE.md` §2.2; ARC-35's accepted limitation ("the proof records them"). |

### 4.6.3 Acceptance (decided before measuring, `ARC-23`)

Each guard names the mutation shown to break it. A working-tree mutation is applied, observed to fail by
name, and reverted; `git status` and `git grep MUTATION` are recorded afterwards. A history mutation is
made on a local scratch branch, never pushed, deleted after (`git ls-remote --heads origin | grep -c
scratch` → 0).

```text
P-1  AC-1 check 1 holds and bites. On the PR head: both merges found by subject, equal to the recorded
     ids, two parents each; 0 paths outside the allowed set; Cargo.lock: 11d +3 path packages under
     systems/ (item, inventory, item-transfer) and mineworld-installed-systems' list, 11e +3 (consumption,
     economy, employment) and that list — no package with a source added, removed or changed.
     Mutations:
     M-P1  scratch branch: a merge commit "Merge pull request #999 from scratch/ac1-mutation" whose second
           parent adds a comment to kernel/src/lib.rs and a file under systems/, and a scratch row for it
           → check 1 fails naming kernel/src/lib.rs.
     M-P2  the same scratch merge also adds to Cargo.lock a [[package]] with source = "registry+…" under
           a market pack's dependencies → check 1 fails naming that package.
     M-P3  the 11d row's recorded id replaced by its first parent e3a1106 → fails: the merge of #43 on
           the first-parent chain is 70e532f, not the recorded id.
     M-P4  the 11e row pointed at 11c's merge (#40, mvp0/pr-11c-affordances, c5dc51c) → fails naming the
           contracts/ and cognition/ paths 11c changed (the instrument sees a framework merge on real
           history).
P-2  Check 1 fails closed — committed tests, not mutations: run against a temporary repository with one
     commit, it fails naming the missing merge of #43; against a shallow clone of a two-commit temporary
     repository, it fails naming the shallow history and fetch-depth 0. It never skips.
P-3  AC-1 check 2 holds and bites. On the PR head: the market packs' dependents are systems/* only; no
     normal/build path from the seven framework crates; no code file outside the three directories names
     a market crate. Mutations (working tree):
     M-P5  `mineworld-economy = { path = "../../systems/economy" }` added to
           cognition/rule-controller/Cargo.toml's [dependencies] → bullet 1 names mineworld-rule-controller
           (and bullet 2 the path).
     M-P6  `mineworld-installed-systems = { workspace = true }` added to server/Cargo.toml's
           [dependencies] → bullet 2 names the path server → installed-systems → a market pack.
     M-P7  `// mineworld_economy` appended to tools/cli/src/run.rs → bullet 3 names the file and line.
     A unit test feeds the core function a hand-written metadata value with a dev edge from persistence
     through worldpack (F-58's shape) and asserts the reading QS-54 decides.
P-4  AC-1 check 3 holds and bites. On the PR head: Market Town = Social Café + the six packs appended to
     systems + items/ and organizations/ + sections owned by market packs only (holdings, economy, job
     on people; item on items; holdings, economy on organizations); README.md not compared. Mutations
     (working tree):
     M-P13 one tag of worlds/market-town/people/bob.yaml changed → fails naming the file and `tags`.
     M-P14 one `routine:` entry of worlds/market-town/people/carol.yaml changed → fails naming the file
           and `routine` (a section, but not a market pack's).
P-5  CP-4: Market Town lives for 300 days, read from a real save, activity first (I-7). From the 300-day
     seed-7 save, per 30-day bucket (the last bucket includes the final instant, social::per_bucket's
     rule):
       - faults 0; every seat accepted a move and a talk (the printed activity table);
       - ≥ 1 purchase (a money-transferred caused by an action);
       - each job holder — every person with a `hired`, and there must be exactly two — paid ≥ 1 wage
         (a money-transferred to them caused by a wage-due naming them);
       - ≥ 1 items-produced; ≥ 1 items-consumed;
       - every seat gave ≥ 1 (an items-transferred from that seat caused by an action — 11d's D-9 b,
         reinstated as a criterion by QS-60);
     over the run:
       - zero wage-unpaid;
       - no wallet, person or organization, ever below the cheapest price of any shop-opened (replayed
         from funded and money-transferred in fact order);
       - nobody ever holds more than six (replayed from stocked, items-transferred, items-produced,
         items-consumed);
       - against the state in the save's newest snapshot: every wallet and every holding equals the
         replay through that snapshot's revision, and the wallets' total equals the genesis funded total
         (money conserved).
     Only then (P-6). Mutations:
     M-P8  `consumption` removed from worlds/market-town/world.yaml's systems (working tree; M-E9's shape)
           → fails at bucket 1: no purchase (people are full), before any comparison runs.
     M-P9  economy's money-transferred reduction credits the payee without debiting the payer (working
           tree; the primary session's 11e mutation) → fails on the snapshot: wallets' total above the
           genesis total, naming a holder whose state differs from the replay.
     M-P10 economy's buy offers built with Offer::new, incomplete (working tree; §4.6.7's planned
           mutation) → fails: no purchase in bucket 0, before any comparison runs.
P-6  AC-11 / AC-12 / AC-6 for Market Town, only after P-5. The 30-day control first passes P-5's
     conditions over its one bucket; then two 30-day seed-7 saves are byte-identical (facts, journal,
     snapshots, headless::Tables::assert_same_history); a 30-day run SIGKILLed after printing day 15 and
     run again reports resuming at the head on disk and is byte-identical to the control. Guard of
     the comparison's sight: a seed-8 30-day save differs from the control at a located row (the run.rs
     pattern), so equality is not vacuous.
P-7  AC-2 at world level for every market pack (CP-6). From copies of Market Town (SD-35):
       without item           validate fails, naming inventory and item
       without inventory      validate fails, naming inventory as a missing dependency
       without item-transfer  runs 30 days, faults 0, every seat active; no give affordance offered to
                              anybody and no `give` request accepted; buy, eat or drink, a wage paid
                              and items produced all occur
       without economy        runs; no buy and no money fact; wage-due stated and nobody pays it (no
                              money-transferred, no wage-unpaid); items produced; gives and meals occur
       without employment     runs; no hired, shift, wage-due or items-produced; purchases, gives and
                              meals occur
       without consumption    runs; no eat or drink, no items-consumed; purchases, wages, production
                              and gives occur
     Mutation M-P11 (negative controls): the "without item-transfer" copy keeps item-transfer → the "no
     give" assertion fails; the "without item" copy keeps item → the expected refusal does not occur and
     the test fails saying so.
P-8  Milestone C through the real server (CP-7), every claim located (the milestone_b.rs table):
       work, earn            in the 2-day save: alice hired, present at a shift start, worked > 0, a
                             wage-due for her and a money-transferred to her caused by it
       hosted                server on the save; alice and bob walk into the café, every stride
                             accepted; alice's disclosed wallet = the Ledger's balance at the head
       buy                   one available complete buy, submitted unchanged, accepted with exactly two
                             events, caused by its action id
       holdings change       alice's wallet − price, holdings + 1 of the kind
       another client sees   bob's listing: that kind's in_stock − 1; bob is disclosed neither alice's
                             wallet nor her holdings
       persists              SIGKILL (signal 9) and restart on the save: same instance, same revision;
                             alice's wallet and holdings and bob's listing unchanged; inspect: every cause
                             resolves
     Mutation M-P12 (negative controls, test code): the restarted server started without --save → fails
     naming the instance (a fresh world); bob left on the street instead of walking in → fails: no
     listing perceived (the perception claim is located, not assumed).
P-9  Nothing moves (I-4, I-8, I-9, and §4.6.1's "no behaviour change"). `git diff --name-only <base>...HEAD`
     lists only §4.6.1's paths; Cargo.lock changes only mineworld-acceptance's dependency list; 300-day
     seed-7 social-cafe sha-256 of all but `wall` = E-0 (ad49c723…c64b); `validate worlds/social-cafe` and
     `validate worlds/market-town` byte-identical to the base's; the 300-day market-town summary lines but
     `wall` identical to the base's (the 11e evidence stays reproducible: 372 755 facts); every existing
     test passes and none is edited (fixture/mod.rs gains a function only); the I-2 scan passes unchanged.
P-10 The documents say it first (CLAUDE.md §2.2): the ARC-35 note (11f) is committed before the test code;
     both doc checks pass.
P-11 Status is true: MVP_STATUS's Market Town composition axis ✅ with the AC-1 evidence; the S9 row; one
     evidence row each for AC-1, CP-4, AC-2 (market packs) and Milestone C; HUMAN_REVIEW_QUEUE Milestone C
     "demonstrated, awaiting the operator's review" with its launch commands.
```

### P-C0 — Design (this section) — docs only

- [x] Implementation: §4.6.0 … 4.6.6, §8.7, §9 E-8/E-9, QS-54 … QS-66, §17, by the planning session on
  `mvp0/s9-11f-plan`.
- [x] Validation: both doc checks (§9 E-9).
- [x] Review: every file and symbol named here was read on `2dddda8` (§8.7); the riskiest unknowns were
  measured, not assumed — the dependency graph (`cargo tree`, F-58), offline `cargo metadata` (F-60), the
  two merges' diffs and lock changes (§9 E-8), a 2-day market-town save's head and activity (F-64, E-8);
  operator-material points are marked (§10). Self-review by the planning session only; the primary
  session's review is pending.

### P-C1 — Specs before code: the ARC-35 note (11f)

**Goal.** How the proof measures — and the readings QS-54 … QS-56 settle — are reviewable before code.

**Scope.** `docs/DECISIONS.md`: an ARC-35 dated note after its 11e note (`git fetch` first; no new ARC id):
the merges by subject and id (SD-30, QS-55); the Cargo.lock rule as R-S9-6 words it (QS-56); check 2's
edge reading as answered (QS-54) and why (F-58); check 3's structural comparison and the section owner
from the build (SD-32); world-level tests reading the market by slugs (SD-33, QS-58); CP-4's 300-day
committed horizon (QS-59); fail closed without full history (CI needs `fetch-depth: 0`, S13).

- [x] Implementation: the note, as scoped — `docs/DECISIONS.md`, ARC-35 "Note, 2026-10-07 (S9, PR
  11f)", eight numbered points (merges by subject and id; the lock rule; QS-54 quoting the operator;
  how check 2 reads; check 3 structural; slugs; 300 days; fail closed) and "what the proof does not
  claim".
- [x] Validation: `check_decision_ids` 49 distinct; `check_doc_headings` 143 / 22, none duplicated
  (§9.6 E-P1).
- [x] Review: the only relaxation is item 3's path rule, and it carries the operator's QS-54 words
  verbatim; the lock rule (QS-56) and the subject+id match (QS-55) tighten; Market Town, Social Café,
  System Pack, World Pack used as CORE_CONCEPTS/MODULE_SPEC define them; the note closes with the
  ARC-33/ARC-8 boundary.

### P-C2 — `tests/acceptance`: check 1 (the change set), fail closed

**Scope.** `tests/acceptance/tests/ac1_composability.rs` (new): `TRANSFORMATION`, `MARKET_PACKS`, the
`git` helper (the scan's), `transformation_merge(repo, row)`, `changed_paths`, `outside_allowed`,
`LockPackage` + `read_lock(text)` + `lock_changes(before, after)`, `check_1_the_change_set`; unit tests on
text; the two fail-closed tests on temporary repositories under `CARGO_TARGET_TMPDIR`; the
market-pack-list guard. `tests/acceptance/src/lib.rs`'s table (a comment).

- [x] Implementation: SD-29, SD-30 — `c11418c`. `TRANSFORMATION`, `MARKET_PACKS`, `git(repo, …)`,
  `is_merge_of`, `transformation_merge`, `allowed`, `changed_paths`, `LockPackage`/`read_lock`/
  `lock_changes`/`refused_lock_changes`, `systems_crates` (`git grep` at `M`), `file_at`,
  `change_set_failures`; tests `check_1_the_change_set`, `check_1_fails_closed_without_the_merges`,
  `check_1_fails_closed_on_a_shallow_clone`, `every_market_pack_is_a_crate_under_systems`,
  `the_subject_match_is_exact`, `the_allowed_set_is_arc_35_item_2`,
  `the_lock_rule_refuses_any_change_that_is_not_a_path_crate_under_systems`. `src/lib.rs` table row.
- [x] Validation: 7 passed; M-P1 … M-P4 each fail by name (§9.6 E-P2); clippy `-p
  mineworld-acceptance --all-targets -D warnings` clean; fmt clean.
- [x] Review: every `git` failure becomes a named failure (the work-tree and shallow checks return
  early; a row whose merge is not found is a failure, not a skip); the subject is matched by its exact
  prefix `#<n> from ` and the whole branch, unit-tested against `#430`, `#4`, a longer branch and a
  missing owner; the lock reader skips the `version = 4` header (outside any block) and reads a
  package without `dependencies` (unit test); each row reads `M^1..M` (`changed_paths`, `file_at
  M^1`), never a base range. The positive run's print is skipped for a row whose merge is not found,
  so the failure is always the check's own report (seen first under M-P3, then restructured).

### P-C3 — `tests/acceptance`: check 2 (the dependency structure)

**Scope.** The same file: `metadata()` (`env!("CARGO") metadata --no-deps --format-version 1 --offline`),
`Workspace` (crate → directory, declared dependencies with kind), `direct_dependents`, `paths_to`
(normal + build edges per QS-54's answer), `names_a_market_crate` over the working tree's code files,
`check_2_the_dependency_structure`; a unit test on a hand-written metadata value (F-58's shape).

- [x] Implementation: SD-31 — `FRAMEWORK`, `MAY_NAME_THE_MARKET`, `Kind`, `Member`, `metadata(repo)`
  (`env!("CARGO") metadata --no-deps --format-version 1 --offline`), `workspace`,
  `dependents_outside_systems` (bullet 1), `linked_paths` (bullet 2, normal + build, BFS with the path
  printed), `names_a_market_crate` + `code_naming_the_market` (bullet 3, `git ls-files --cached
  --others --exclude-standard`, read from the working tree), `market_crate_spellings`; tests
  `check_2_the_dependency_structure`, `a_dev_edge_is_no_linked_path_but_is_still_a_dependent` (F-58's
  shape, QS-54), `a_market_crate_is_named_only_as_a_word`.
- [x] Validation: 10 passed; M-P5 … M-P7 and an untracked-file probe fail by name (§9.6 E-P3);
  clippy and fmt clean.
- [x] Review: a name matches only with no `[A-Za-z0-9_-]` on either side (unit test: `mineworld-item`
  not in `mineworld-item-transfer` nor `mineworld_item_transfer`); untracked files are listed by
  `--others --exclude-standard` and shown seen by the probe; a framework crate missing from the
  workspace is a failure, unit-tested with `mineworld-renamed`; a dev edge is no linked path but a dev
  dependency on a market pack from outside `systems/` is a bullet-1 failure (unit test). Mutations ran
  against the already-built test binary so that no build rewrote `Cargo.lock` (`git status` showed
  only the uncommitted test file after each revert).

### P-C4 — `tests/acceptance`: check 3 (the world delta)

**Scope.** The same file: `read_yaml(path) -> serde_json::Value`, `compare_manifests`, `compare_files`,
`market_section(key)` (through `mineworld_worldpack::Capability::owning_section` and `MARKET_PACKS`),
`check_3_the_world_delta`; `tests/acceptance/Cargo.toml` gains `mineworld-worldpack` and `serde-saphyr`
dev-dependencies (`workspace = true`); `Cargo.lock` regenerated (two names in mineworld-acceptance's list).

- [x] Implementation: SD-32 — `SOCIAL_CAFE`, `MARKET_TOWN`, `ENTITY_FIELDS`, `read_yaml`
  (serde-saphyr → `serde_json::Value`), `market_section` (`Capability::owning_section` ∩
  `MARKET_PACKS`), `compare_systems`, `compare_manifests`, `compare_content`, `market_only_content`,
  `entries`, `world_delta_failures`; tests `check_3_the_world_delta`,
  `a_section_is_the_market_s_by_the_build_s_own_catalog`,
  `the_world_delta_names_every_difference_by_file_and_key`. `tests/acceptance/Cargo.toml` gains
  `mineworld-worldpack` and `serde-saphyr` (`workspace = true`); `Cargo.lock`: exactly those two
  names added to mineworld-acceptance's list (F-69).
- [x] Validation: 13 passed; M-P13, M-P14 fail by name (§9.6 E-P4); clippy and fmt clean; the I-2
  scan (precursor_vocabulary) still 4 passed, unchanged.
- [x] Review: `market_section("no-such-section")` is false (unit test), so a key no installed pack
  owns fails; `serde_json::Value` compares maps by key (unit test: reordered keys equal) and lists in
  order (unit test: `["b","a"]` ≠ `["a","b"]`); a pack that does not read fails the check before any
  comparison (`world_delta_failures` returns `Err`, the test panics with the refusal). Bounded
  addition: an item or organization file's keys other than `tags`/`note` must also be market
  sections (§4.6.8 DP-2). File size: 1 343 lines, past the ~800 warning (§4.6.8 DP-1).

### P-C5 — `tools/cli/tests`: the Market Town reader and CP-4

**Scope.** `tools/cli/tests/market/mod.rs` (new, SD-33) and `tools/cli/tests/market_town.rs` (new,
SD-34). Reuses `headless::{Tables, mineworld, fresh, every_seat_active_in_every_bucket, deterministic}`
unchanged.

- [x] Implementation: the mirrors, the `Ledger`, the per-bucket conditions, the snapshot cross-check;
  the one test in I-7's order — `42c22ea`. `market/mod.rs`: `run_pack`, `run_market`, `Town`,
  `MarketFact` + `market_fact` (slug match, schema version 1, exact fields), `wallet_balance`,
  `holdings`, `listing`, `Ledger`, `market_lives`, `genesis_money`, `state_matches_replay`.
  `market_town.rs`: one test.
- [x] Validation: PASS 44.3 s (runs 35.6 s in parallel); M-P8, M-P9, M-P10 each fail by name before any
  comparison (§9.6 E-P5).
- [x] Review: every required count is printed per bucket with zeros shown, and every failure is
  collected and reported together (restructured after M-P8's first run reported only the first
  inline wallet violation — DP-3); job holders read from `hired`, asserted exactly two; the
  snapshot check takes the newest snapshot and replays facts with ids below the first fact of any
  later revision; the files name slugs only (check 2 bullet 3 passes over them). serde is not a
  dependency of mineworld-cli, so the mirrors are exact-field checks over `serde_json::Value`
  decoding each field into the contracts' id types, equivalent to `deny_unknown_fields` (DP-4).

### P-C6 — `tools/cli/tests`: AC-2 at world level

**Scope.** `tools/cli/tests/market_composition.rs` (new, SD-35): `without(pack, section)` copies of
Market Town under `CARGO_TARGET_TMPDIR`; the six cases run in parallel threads.

- [x] Implementation: SD-35 — `PACKS` (pack, section, number of carrying files), `strip_section`,
  `copy_dir`, `files`, `without` (copy, strip, then assert the copy equals Market Town but for the
  pack's line and its section), `refused`, `Ran` (requests and facts of a 30-day run), one test with
  six cases in scoped threads.
- [x] Validation: PASS, 4.5 s; M-P11 both controls fail by name (§9.6 E-P6); clippy, fmt clean.
- [x] Review: `without` asserts per file that the copy is the original minus the pack's line or its
  section, and the number of carriers (item 20, holdings 14, economy 14, job 2); a refusal is read
  from `validate`'s non-zero exit and its stderr, by name; "requested" sums every outcome line of a
  request type (first draft read only "accepted", corrected before commit). "No give offered" is
  read as "no give requested by a controller that attempts what it is offered" (module doc).

### P-C7 — `tools/cli/tests`: Milestone C

**Scope.** `tools/cli/tests/milestone_c.rs` (new, SD-36); `tools/cli/tests/fixture/mod.rs` gains
`assert_quiet_in(people_dir, from, length, test)` reading `from:` and `until:` (the existing
`assert_quiet` keeps its body and its callers).

- [x] Implementation: SD-36 — `740b0ad` and its follow-up. `locate_work_and_pay`, `social_entry`,
  `own`, `listing_in`, `discloses_private`, `walk_into_the_cafe`, `seated`, `in_the_cafe`,
  `unchanged`; one test. `fixture::assert_quiet_in` added; `assert_quiet` and its callers untouched.
- [x] Validation: PASS, 1.0 s; M-P12 both controls fail by name (§9.6 E-P7); clippy, fmt clean.
- [x] Review: doorways from `WorldPack::read(MARKET_TOWN).places()[..].passages`, the start from the
  observer's own `self_location`; every stride through `walk_accepted`/`submit_accepted`; the buy and
  any meal are the offered affordance submitted unchanged (`unchanged`: its action type, target and
  payload); the kind bought and its price are read from what changed (holdings, wallet, listing),
  never from the payload; `assert_quiet_in` guards the revision claim. "Present at a shift start"
  does not hold in the save and is located differently (§4.6.8 DP-5).

### P-C8 — Close: documents, status, full gate, ledger

- [ ] Documentation: SD-37's rows; `worlds/market-town/README.md`; §4.6 checkboxes; §9.6 `E-P<n>`,
  `E-P-final`; deviations in a new §4.6.8; the handoff.
- [ ] Validation, once, on the final executable head: `cargo fmt --all --check`; `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`; `cargo test --workspace --no-fail-fast`; the two doc
  checks; P-9's comparisons (social-cafe sha, both `validate`s, the market-town 300-day summary vs the
  base's).
- [ ] Review: P-1 … P-11 each with evidence; the PR is merged **with a merge commit**, said in the PR body
  (the I-2 scan and check 1 read merges).

### 4.6.4 Test ownership for 11f

```text
STATIC      fmt, clippy -D warnings
UNIT        the lock reader and lock rule, the path filter, the subject match (check 1); the metadata
            reading and the edge rule (check 2); the YAML comparison (check 3) — each on hand-written
            inputs; the fail-closed tests on temporary repositories (P-2)
INTEGRATION the three checks against the repository, its history and its build (P-1, P-3, P-4)
REAL RUN    the 300-day market-town save and its reader, two 30-day saves, a SIGKILLed run (P-5, P-6);
            six composition copies through validate and run (P-7); the real server on a real save, two
            real WebSocket clients, a real SIGKILL (P-8); the social-cafe comparison (P-9)
GATE 1      NOT REQUIRED — no model
GATE 2      the real runs above and the scratch mutations M-P1 … M-P14
CI          none configured (S13); the full local gate once on the final head
```

### 4.6.5 Is any of this material?

- **No behaviour changes.** 11f adds tests and documents; the audit (§8.7) found every seam the proof
  needs already public: `git` history, `cargo metadata --no-deps`, `WorldPack::read`,
  `Capability::owning_section`, the save's tables through `SqliteBackend`, `EventRecord::payload()`, the
  server's protocol. A needed edit to any pack, world or framework crate is a material stop.
- **Operator-material:** QS-54 (how ARC-35 check 2 reads "a dependency path": normal and build edges,
  because of F-58) and QS-65 (declaring AC-1 demonstrated, and Milestone C's review).
- **Bounded readings the primary session settles:** QS-55 (merges by subject and id), QS-56 (the
  Cargo.lock rule as R-S9-6 words it), QS-57 (check 3 structural), QS-58 (slugs, not crates, in
  tools/cli), QS-59 (300 days, default-on), QS-60 (11d's per-seat gives reinstated as a criterion), QS-61
  (AC-2 for all six), QS-62 (Milestone C at the café, after a 2-day run), QS-63 (the quiet window reads
  shifts).
- No public contract, ownership boundary or frozen invariant changes. Two dev-dependencies are added to
  a test crate, both already workspace entries; no external dependency arrives.

### 4.6.6 The S9 closeout (recorded by the planning session after 11f merges)

S9 closes when 11f merges. The post-merge update then records, in the step header, overall §7 and
`MVP_STATUS.md`:

```text
AC-1      demonstrated, as ARC-35 measures it and within ARC-33's static-linking boundary (installing =
          a directory, two lines in systems/installed, a rebuild; without a rebuild is ARC-8, outside
          MVP-0) — once ac1_composability passes on main with full history and P-1 … P-4's mutations are
          recorded biting. The operator's acceptance of that claim is QS-65.
AC-2      confirmed in S9 for the six market packs at world level (P-7), beside S6's movement and S8's
          social packs.
CP-4      the market lives 300 days, as a committed test (P-5).
Milest. C demonstrated through the real server, awaiting the operator's review (HUMAN_REVIEW_QUEUE).
F-47      a System Pack owns one section, so a shop is authored on its operator (ARC-38) — a framework
          limit worked within; lifting it is framework work for a later step, justified without the market.
F-48      an offer carries one pack-supplied unavailability reason, so out of stock, cannot pay and cannot
          carry all read TargetUnavailable — a contract limit for a later step.
F-41      item kinds and organizations have no names; a client shown a listing cannot name what it sells
          (S12, where a client first shows one).
L-12      people walk ~8 m per in-world hour at the headless pace; routines are authored in ≥ 4 h parts.
L-13      a bounded-horizon economy: ten of twelve people have no income and live on endowments sized for
          300 days; past that they run out (ARC-38). Closing it is a later step's work.
Rel.sat.  relationships never decay; the social graph saturates after ~60 days (step-09, 10b).
QS-10     `sleep` is still an open MVP §5 interaction, with no step; `eat` (and `drink`) closed by
          consumption (QS-35). Hunger and needs remain a later needs pack.
F-58      persistence's kill_and_resume test links the market packs through worldpack and the installed
          set; check 2 reads normal and build edges (QS-54's answer).
S13       the AC-1 test needs full history: CI must check out with fetch-depth 0, or the test fails (by
          design, never a skip).
```

### 4.6.7 Medium scope, as written before 11e merged (superseded by §§4.6.1 … 4.6.6)

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

### 4.6.8 Deviations and discoveries during implementation (11f session)

```text
DP-1  ac1_composability.rs is 1 343 lines, past ENGINEERING_STANDARDS' ~800-line warning.
      Reason: SD-29 fixes one file with one #[test] per check; each check's core and its unit tests
      sit in its own section (~370 lines for check 1, ~360 for check 2, ~350 for check 3). A module
      split (tests/acceptance/tests/ac1/…) is a path §4.6.1 does not list. Impact: none on behaviour.
      Recorded as a review finding, not changed.
DP-2  Check 3 also reads items/ and organizations/ files: every key other than the format's own
      `tags` and `note` must be a section a market pack owns. SD-32 names only "items/ and
      organizations/ exist in Market Town only"; this tightens it within ARC-35 item 4's intent
      ("plus sections owned by market packs only") and loosens nothing. Holds on the head.
DP-3  market_lives first asserted the run conditions inline, fact by fact. M-P8's first run therefore
      failed at day 43 naming cafe-company's drained wallet, not the expected "bucket 1: no
      purchase". Both are before any comparison, but the report hid the rest. Now every failure is
      collected and reported together, per-bucket absences first. Re-run of M-P8: as expected.
DP-4  SD-33 names `#[derive(Deserialize)] #[serde(deny_unknown_fields)]` mirrors. mineworld-cli has no
      serde dependency and its Cargo.toml is not a §4.6.1 path, so the mirrors are written as an exact
      field-set check over serde_json::Value, each field decoded into the contracts' id type. Same
      strength: an added or missing field fails by name.
DP-5  Previous assumption (SD-36, P-8): the 2-day save holds `shift-started { present: true }` for
      alice. Audit evidence: all four shift-started in the save have `present: false` (both alice's
      and felix's); alice's routine and her shift both begin at 05:30, so she is walking to the café
      when the shift starts, and employment's reaction to her person-entered-place opens the worked
      span (ARC-38 item 2). Corrected understanding: presence at work is her arrival during the
      shift, not presence at its start. Implementation consequence: P-8's "present at a shift start"
      is located as a shift-started for alice, her person-entered-place into the café between it and
      the shift-ended with worked > 0 (#443 t19 800, #580 t26 100, #928 t50 400). No world or pack is
      changed (I-9); Milestone C's claim, work → earn, is unchanged and located. Raised in the report
      for the primary session; not treated as a material stop because the parent claim holds and the
      test's sub-assertion was a planning assumption measured false.
DP-6  M-P12's second control was "bob left on the street"; it was run as "bob never walks" (he stays
      in the apartments, as the save left him) — the same claim: bob outside the café perceives no
      listing.
DP-8  P-11 lists "the S9 row" of MVP_STATUS, but §17's POST-MERGE SYNC gives MVP_STATUS's Updated and
      S9 lines to the planning session. Followed §17: this PR changes the Market Town axis (✅ with
      QS-65's wording), the market-town artefact row and adds four evidence rows (AC-1, CP-4, AC-2,
      Milestone C); the S9 row and the Updated line are left for the S9 closeout.
DP-7  Tool slip: one `sed -i` was used on milestone_c.rs's doc header (forbidden by the session's
      tool rules); the line was then rewritten with the Edit tool. No other effect.
```

---

# 5. Integration checkpoints

| PR | Integration checkpoint | Adversarial criterion (decided before measuring, `ARC-23`) |
| --- | --- | --- |
| 11a | social-cafe's 300-day run byte-identical to E-0; a canary pack installed by two lines in `systems/installed/` (A-1 … A-4) | removing a list line, or re-adding a pack import to `worldpack`, fails a named guard |
| 11b | a pack with items and organizations loads; social-cafe's ids, genesis count and run unchanged | items' sections seeded after people's fails the genesis-order test |
| 11c | a synthetic pack's action, unknown to the controller, is attempted and accepted headless; social-cafe byte-identical | the band taking an unavailable affordance fails a named test; the band reusing the walking roll's draw fails the greeting-coexistence test (F-28 — the social-cafe comparison cannot see it) |
| 11d | market-town (owning, giving) validates; over 300 days every seat gives in every bucket and nobody holds more than six; the unchanged paced controller gives in a pack test; per-pack tests incl. restart; AC-2 for item-transfer at pack level (§4.4.3, D-1 … D-11) | a transfer beyond what the giver holds, stated directly, is refused by inventory itself (M-D3); without the capacity the gives collapse into the unseated person (M-D7) |
| 11e | market-town (work, money, shops, eating) lives 300 days: purchases, wages, production, consumption and gives in every bucket, zero wage-unpaid, no wallet below the cheapest price, money conserved; the unchanged controller buys in a pack test; restart mid-shift; each of economy, employment and consumption removable in its direction (§4.5.3, E-1 … E-12) | wage-due against a short wallet yields wage-unpaid, never a negative (M-E3); without consumption, purchases stop once hands are full (M-E9) |
| 11f | the AC-1 test (three checks, fail closed without history), CP-4 over 300 days from a real save then AC-11/12/6, AC-2 at world level for all six market packs, Milestone C through the real server (§4.6.3, P-1 … P-11) | a framework path, an external package or a misidentified merge fails check 1 by name (M-P1 … M-P4); a controller or server linking a market pack fails check 2 (M-P5 … M-P7); a changed tag or routine fails check 3 (M-P13, M-P14); without consumption, or with a payer never debited, CP-4 fails before any comparison (M-P8 … M-P10) |

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
FLAGGED  QS-35 — operator-material: without consumption, 11e's purchases stop once people are full;
         11e must close the item loop (QS-10 can no longer be deferred past 11e's design). QS-27 (the
         capacity) is part of how 11d answers QS-10, and is operator-visible
FLAGGED  11e (§4.5.5): QS-39, QS-45, QS-47 operator-material; QS-43 amends the medium scope's authoring
         format; no framework gap (E-6)
FLAGGED  11f (§4.6.5): QS-54 (check 2's edge reading, F-58) and QS-65 (declaring AC-1 demonstrated)
         operator-material; no behaviour change and no framework gap (§8.7, E-8)
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

## 8.5 Re-audit for 11d (`main @ c5dc51c`, 2026-10-07)

Made by the planning session after 11b and 11c merged, before detailing §4.4. Where §4.4's medium scope
and the source disagree, the source wins and the finding says so.

**Inspected.**

```text
sdk/rust/src/{pack.rs, section.rs} (whole), installed.rs (expansion surface)
systems/installed/{Cargo.toml, src/lib.rs, tests/installed.rs (manifest parser)}
systems/naming/src/{lib, section, system, event, component, codec, name}.rs (whole: the section-owner
  template); systems/schedule/{Cargo.toml, src/section.rs}; systems/movement/src/system.rs (validate,
  resolve stating presence's Arrived, react refusing, offers); systems/presence/src/{system.rs (whole),
  event.rs (arrival, admit), interaction.rs (whole: Offer::complete), observe.rs (:60–289)};
  systems/conversation/src/{action.rs (talk_requirement), system.rs (validate), codec.rs}, Cargo.toml
authoring/src/{section.rs (:110–201: Reference, Seeding, AuthoredSection), content.rs (whole)}
worldpack/src/load.rs (:40–100 seeded, :220–300 assemble/load/initial_facts); worldpack/tests/
  refusals.rs (:169–188, :774–795: assertions over the installed set and the known sections)
kernel/src/{view.rs (WorldRead/WorldView API), dispatch.rs (:355–400 genesis), system.rs (trait
  outline)}; contracts/src/{ids.rs (:392–396 EntityKey, :850–905 ItemId), event.rs (:296–310
  Visibility)}; persistence/src/{lib.rs, format.rs (JSON throughout)}
cognition/rule-controller/src/offered.rs (whole)
tools/cli/src/biography.rs (:40–110, reading a save); the run summary format (E-3's output)
worlds/social-cafe (world.yaml, people/otto.yaml, people/alice.yaml, places/cafe.yaml)
docs: DECISIONS ARC-26, ARC-28, ARC-31 … ARC-36; MODULE_SPEC §§3, 3.1, 4, 4.1; PACKAGE_FORMAT §§6, 8;
  CORE_CONCEPTS §§1–3, 6.3, 7, 8, 11–15; MVP §3 (~20 item types)
git: every origin/* branch for ARC-37 (free)
runs: E-3 (main), E-4 (the R-S9-1 spike)
```

**Findings.**

```text
F-36  The riskiest cross-pack flow needs no framework change (E-4). A complete `give` offered by a new
      pack, chosen by the unchanged paced controller, validated by item-transfer, stated through
      inventory's checked constructor (ARC-26) and reduced by inventory alone, seeded from an item
      file's and a person file's sections, saved and resumed — all ran with edits only under
      systems/**, worlds/** and Cargo.lock, first build.
F-37  Sections are seeded against a world with no state. WorldPack::assemble computes every genesis
      fact from the assembled world's read view before World::genesis reduces any (load.rs
      initial_facts, then genesis), although authoring's Seeding doc says "no state yet beyond the
      genesis facts stated before this one". So inventory's seed cannot ask whether item has declared
      a kind. It checks the reference's entity type at seeding (Seeding::resolve) and the declared kind
      at reduction, which follows item's in genesis order (ARC-36 point 7). Bounded, inside the packs;
      the Seeding doc sentence is imprecise and is reported, not changed (authoring/ is outside 11d's
      range; a later precursor or doc fix may correct it). Fixed after 11d's merge by a docs-only PR
      (`docs/s9-11d-merged`): the doc now says every section is seeded before genesis reduces any
      fact, and that a check needing another fact's state belongs in the owner's reduction.
F-38  An ItemId serializes as a struct ({entity, type}), so it cannot be a JSON object key; payloads,
      observations and snapshots are all JSON (DEP-5). Holdings is therefore a sorted list of
      {item, count}, not a BTreeMap<ItemId, u32> as §4.4's medium scope wrote.
F-39  An unseated person is an absorbing sink. Otto (no seat) is given to and never gives. Spike run 1,
      no bound: by day 30 Otto held 31 of the 33 items, and gives fell from 739 in days 1–15 to 312 in
      days 16–30. Run 2, a person carries at most 6: 18 349 gives over 300 days, 1 737–1 887 in every
      30-day bucket, every seat ≥ 118 per bucket, Otto ends holding exactly 6. Without consumption
      (QS-10) this is how 11d's flow stays alive and bounded (SD-19, QS-27).
F-40  Perception asks a provider about every target present, the observer included (observe.rs:239
      `present` holds the observer), so a pack must exclude giving to oneself in its offers.
F-41  Items are never perceived: an observation lists the observer's place and the people in it. So
      ItemKind is disclosed to no one, and a client shown `give { item: {entity: 21, …} }` cannot name
      the item. Recorded as a limitation for S12 (QS-13's client work), not solved in 11d.
F-42  Cost (R-S9-2): the 300-day market-town run with --save took 32.7 s (chimes, the worst case, 43 s;
      social-cafe without save 12.2 s). Capacity bounds give offers at 6 per person nearby.
F-43  Installing the three packs breaks nothing outside the range: on the spike branch the whole
      workspace suite passed (457 = main's 456 + the scratch reader), social-cafe's 300-day sha = E-0,
      and refusals.rs asserts only `contains` over the installed set and the known sections.
F-44  The I-2 scan does not see 11d's lines: all three precursor rows are merged on the first-parent
      chain, so their ranges are base..M^2 (ARC-35 point 7). 11d needs no allow-list entry and adds
      no row.
F-45  `mineworld run`'s per-seat activity line counts move and talk only; per-seat give counts need a
      save reader. World-level tests live in tools/cli/tests or tests/acceptance, outside the range, so
      11d's per-seat evidence uses a scratch reader on a scratch branch (as 11c's E-C6), and 11f
      makes it a test.
F-46  ItemsTransferred's audience is inventory's choice (the vocabulary owner builds the emission). The
      spike used Participants (giver and taker). A stating pack cannot widen it; whether a purchase in
      11e should be place-visible is 11e's question (QS-30).
```

## 8.6 Re-audit for 11e (`main @ 70e532f`, 2026-10-07)

Made by the planning session after 11d merged, before detailing §4.5. Where §4.5's medium scope and the
source disagree, the source wins and the finding says so.

**Inspected.**

```text
systems/inventory/src/{admit, system, event, component, section, lib}.rs (whole), Cargo.toml
systems/item/src/{system, section, event, component, lib}.rs (whole); category.rs (API)
systems/item-transfer/src/{system, offer, action, codec}.rs (whole), Cargo.toml
systems/schedule/src/{system, process, time}.rs (whole), lib.rs (exports)
systems/presence/src/{interaction.rs (whole: Offer::complete), observe.rs (whole: present, perceived,
  disclosed, affordances, verdict)}, event.rs (PersonEnteredPlace: person, place, from), lib.rs, Cargo.toml
systems/relationships/src/system.rs (relation declaration, install)
systems/installed/{Cargo.toml, src/lib.rs}
sdk/rust/src/pack.rs (whole: SystemPack, owns_section!)
cognition/rule-controller/src/offered.rs (whole)
authoring/src/section.rs (Seeding, AuthoredSection); worldpack/src/load.rs (:190–335 assemble, load,
  initial_facts)
contracts/src/{spatial.rs (:356–515 SpatialRequirement, at_place, evaluate), action.rs (Rejection),
  event.rs (Visibility, Causation, EventRecord), relation.rs (EntityTypeSet, directed), ids.rs (id types)}
kernel/src/{process.rs, view.rs, system.rs} (public API)
persistence/src/{lib.rs, backend.rs (last_facts), format.rs (decode)}; tools/cli/src/biography.rs
  (:40–110, reading a save)
worlds/market-town (world.yaml, every person file's routine/location/holdings, places/cafe.yaml,
  places/store.yaml, items' categories)
docs: MVP §§2–5; CORE_CONCEPTS §§7, 8, 9, 10, 13.1; MODULE_SPEC §4.1 (sections); PACKAGE_FORMAT §8;
  DECISIONS ARC-26, ARC-28, ARC-33 … ARC-37
runs: E-6 (the R-S9-1 spike)
```

**Findings.**

```text
F-47  A System Pack owns at most one section (sdk/rust/src/pack.rs: `SECTION: Option<SectionOwner>`,
      one AuthoredSection impl through owns_section!). §4.5's medium scope gave economy two (`wallet:`
      on people and organizations, `shop:` on places). Bounded inside the pack: one `economy:` section
      on people and organizations, a shop authored on its operator as `shop: { at, prices }`, Shop still
      written on the place by economy's reduction. A side effect: no place file changes (QS-43). No
      precursor: the limit is a reasonable shape, and lifting it would be framework work justified only
      by the market (I-2).
F-48  An Offer carries only `target_available` as a pack-supplied verdict, and a target-less offer's
      requirement can say only "at that place" (contracts spatial.rs evaluate: SamePlaceAsActor needs a
      target; at_place(place) does not). So `buy` is target-less with `at_place(shop)` and
      `requiring_target_available`, and "out of stock", "cannot pay" and "cannot carry" all read
      TargetUnavailable, in the affordance and at dispatch alike (QS-44).
F-49  PersonEnteredPlace carries `from` (presence event.rs), so leaving the workplace is visible as an
      entry elsewhere whose `from` is the workplace. Attendance is exact without polling.
F-50  Only food and drink can be consumed; goods sold by a shop would fill a person's six places for
      good and stop buying and receiving (the 11d sink in another form). Shops sell consumables only
      (QS-48), and `drink` exists beside `eat`, or drinks would clog the same way (QS-39).
F-51  The money loop, measured (§9 E-6). Run 1 (the first sizing, wages 1 500 per hour): 28 of 58
      wage-dues unpaid in 30 days — wages far above what the shops take. Run 2 at the first realistic
      sizing: every CP-4 condition held but felix's wallet went to 3 by day 90 and the store's fell
      every month — a store employee spends more than a 100/h wage and the store sells less than it
      pays. Run 3 (felix 200/h, the store's capital 300 000): every condition of E-9 b held. So the loop
      is sized in content, as I-9 requires, and the instrument sees a drained wallet when there is one.
F-52  Without a job a person has no income: ten of twelve people live on their endowment and spend
      ~2 000–10 000 per 30 days. 300 days is within reach of an endowment; years are not (R-S9-3, §1.2:
      a market that balances itself over years is a non-goal). Recorded in ARC-38 (QS-47).
F-53  dev never stands in a shop (park, workplace, apartments) and buys nothing, and Otto is driven by
      nobody: a per-seat purchase criterion is impossible by content, so CP-4's purchase criterion is
      world-level, as written.
F-54  The café's place record reaches every perceiver (observe.rs perceived(): the place first, its
      components from disclosed()), so a listing disclosed on the shop's place is seen by everyone there
      and by nobody elsewhere — CP-7's perception path, with no client change.
F-55  economy's react can answer `wage-due` with its own `money-transferred`, caused by the `wage-due`
      (Causation::Event), and a subscription to an event whose owner is not installed is legal (as
      relationships hears `spoke` without conversation). The ARC-28 direction holds with a Cargo
      dependency economy → employment and no system dependency.
F-56  Cost: 300 days of the spike's market-town with --save took 34.5 s (11d: 32.5 s). R-S9-2 holds.
F-57  Process: during this audit one `xargs` (listing file sizes) and one `awk` (summing test counts)
      slipped into read-only commands, against the kickoff's tool discipline. Neither changed a file.
      Recorded, not repeated.
```

## 8.7 Re-audit for 11f (`main @ 2dddda8`, 2026-10-07)

What was inspected, on `2dddda8` (11e merged), for the proof:

```text
tests/acceptance/{Cargo.toml, src/lib.rs, tests/precursor_vocabulary.rs (whole), tests/complete_affordances.rs
  (header)}; root Cargo.toml (members, [workspace.dependencies], [profile.dev])
tools/cli/{Cargo.toml, src/main.rs (commands, validate, serve), src/run.rs (the pace loop, the end,
  the printed report), src/inspect.rs (what it reads and prints)}
tools/cli/tests/{milestone_b.rs (whole), headless/mod.rs (whole), social/mod.rs (whole), support/mod.rs
  (whole), fixture/mod.rs (whole), run.rs (header, the 300-day test), run_restart.rs (header),
  social_composition.rs (header, without / without_owning)}
persistence/src/{lib.rs (exports), backend.rs (the trait), world.rs (snapshots)}; kernel/src/snapshot.rs
  (WorldSnapshot); contracts/src/event.rs (EventRecord: event_type, schema_version, payload, payload_for)
worldpack/src/{lib.rs (exports), read.rs (WorldPack accessors), format.rs (WorldManifest, Authored*,
  FoundSection, SectionState)}; authoring/src/content.rs (AuthoredContent); sdk/rust/src/installed.rs
  (Capability: resolve, id, section, owning_section)
server/PROTOCOL.md (whole)
worlds/market-town/{world.yaml, people/felix.yaml, people/alice.yaml (routine, job comments),
  places/{store,apartments}.yaml (passages)}; git ls-files of both worlds
systems/{economy/src/{event.rs, component.rs, system.rs (discloses)}, inventory/src/{event.rs,
  component.rs}, employment/src/event.rs}: payload fields, slugs, schema versions, component types
docs/{DECISIONS.md ARC-35 (with both notes), ARC-36, ARC-37, ARC-38; HUMAN_REVIEW_QUEUE.md (whole);
  ACCEPTANCE.md (whole); MVP.md §§1–2, 9; MVP_STATUS.md}
history: git log --first-parent --merges HEAD (both transformation merges); git diff --name-only and
  -- Cargo.lock of 70e532f^1..70e532f and 2dddda8^1..2dddda8
commands run: cargo tree -i per market pack and -p per framework crate (offline); cargo metadata (full,
  then --no-deps); mineworld run worlds/market-town --days 2 --save, then inspect (§9 E-8)
```

Findings (F-58 onward; F-1 … F-57 are earlier sections'):

```text
F-58  persistence's dev-dependency on worldpack (for the kill_and_resume checkpoint) reaches the market
      packs through the installed set: `cargo tree -p mineworld-persistence -e normal,dev,build` shows
      them, `-e normal,build` does not. kernel, contracts, server, authoring, sdk and rule-controller reach
      none by any edge. ARC-35 check 2's "no dependency path" is ambiguous about dev edges, and read with
      them it fails AC-1 for a test that loads a World Pack (QS-54, operator-material).
F-59  tests/acceptance is already a workspace member (11a); 11f needs no root Cargo.toml edit. §4.6.7's
      "root Cargo.toml: one member line" is stale.
F-60  `cargo metadata --offline` (full) fails: it wants anstyle-wincon, a Windows-only crate never
      downloaded here. `cargo metadata --no-deps --offline` runs in ~0.03 s and lists every workspace
      package's declared dependencies with `kind` and `path` — enough, because only a workspace member can
      depend on a path crate, so every path to a market pack runs through members.
F-61  World-level tests that run the binary must live in tools/cli (CARGO_BIN_EXE_mineworld exists only
      for the binary's own package), and ARC-35 check 2 bullet 3 forbids naming a market crate there. The
      existing social tests decode facts with the owner packs' types (social/mod.rs); the market tests
      cannot. EventRecord::payload() gives the bytes unchecked beside event_type() and schema_version(),
      so test-local mirrors keyed by slug are possible without any framework change (SD-33, QS-58).
F-62  WorldPack::read keeps a section only as Arc<dyn AuthoredContent> (Debug; owner, section,
      references, seed), so check 3 cannot compare sections through it. serde-saphyr documents
      deserializing YAML into serde_json::Value for this case; the section's owner is answered by the
      build's Capability::owning_section (SD-32).
F-63  `mineworld run` writes no snapshot at its end (no checkpoint call in run.rs); a save holds a snapshot
      every 64 revisions (DEFAULT_SNAPSHOT_INTERVAL). A state cross-check is therefore at the newest
      snapshot, against the facts at or before its revision (SD-34).
F-64  A save's head is the last journaled input, not midnight: a 2-day market-town save's head is
      revision 1 946 at t171 910 (day 2, 23:45:10), the last consult. Every routine and job boundary of
      market-town lies in 05:00–23:00, so an hour hosted from the head is quiet. fixture::assert_quiet
      reads social-cafe's people and only `from:`, so it cannot see a job's `until:` (SD-36).
F-65  The store sells each morning's production the same day (E-E7: 0 items at day 300), so a buy at
      the store at the head is likely unavailable; the café keeps stock (170 items at day 300). Milestone
      C buys at the café, alice's employer (SD-36).
F-66  A buyer carrying six is offered `buy` unavailable (TargetUnavailable, ARC-38 item 3), and E-E7
      saw every person at six at some point. Milestone C frees one place first, through an offered
      `eat` or `drink`, when the buyer is full (SD-36).
F-67  The I-2 scan reads each precursor's base..M^2, all three merged; 11f's lines are in no precursor's
      range, so 11f may name market words freely and adds no row.
F-68  The CP-4-style helpers are social-cafe-bound (headless::PACK, fixture's PEOPLE, seats()). Market
      Town needs its own pack path; a new shared module avoids editing those helpers' existing callers.
F-69  11f's Cargo.lock change is mineworld-acceptance's dependency list gaining mineworld-worldpack and
      serde-saphyr — outside systems/, which is allowed for 11f (not in the AC-1 range), and never read
      by check 1, which reads only the two transformation merges.
```

Material findings: F-58 (QS-54). Everything else is bounded, and no framework gap was found: every seam
the proof needs is already public.

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
E-4  The R-S9-1 spike for 11d. Local scratch branch `scratch/11d-spike` from c5dc51c, scratch commit
     74bf597, never pushed, deleted after (`git ls-remote --heads origin | grep -c scratch` → 0).
     Logs in /tmp/s9-11d-plan/. Flow under test: a person gives an item offered as a complete
     affordance, attempted by the unchanged paced controller.
     Built: systems/item (ItemKind, `item:` section, item-kind-declared), systems/inventory (Holdings
     as a sorted list, `holdings:` section, stocked, items-transferred, admit_transfer + transfer),
     systems/item-transfer (give, complete offers per held kind, validate/resolve through
     inventory::transfer), three lines each in systems/installed, worlds/market-town (social-cafe
     copied by `git read-tree`, 5 kinds, holdings on all 12 people, 33 items).
     Changed paths vs c5dc51c: Cargo.lock, systems/installed/{Cargo.toml, src/lib.rs},
     systems/{item,inventory,item-transfer}/**, worlds/market-town/** (25 files). Nothing else.
     Cargo.lock: +3 [[package]] (mineworld-item, -inventory, -item-transfer), no `source`; +3 lines in
     mineworld-installed-systems' dependencies. Compiled on the first build.
     validate: valid; ids 1–18 = social-cafe's, kinds 19–23; 74 genesis facts (53 + 5 + 16).
     Run 1 (no capacity), 30 days, seed 7, --save: exit 0, faults 0, give accepted 1 051, no rejected
       request line, 36 899 facts, wall 2.8 s. Resumed 15 → 30 on one save: fingerprint
       0fc3aba2ae64ce4b = the uninterrupted run's. Reader: Otto (16, no seat) holds 31 of 33 items at
       day 30; gives 739 in days 1–15, 312 in days 16–30 → F-39, the sink.
     Run 2 (PERSON_CAPACITY = 6 in inventory; give's target available iff it can take one), 300 days,
       seed 7, --save: exit 0, faults 0, 364 799 facts, wall 32.7 s; give accepted 18 349, move
       173 049, talk 60 073; activity minimum talk 416 per seat-bucket. Reader: gives per bucket
       1 861, 1 887, 1 737, 1 850, 1 836, 1 807, 1 822, 1 878, 1 843, 1 828; all 110 (seat, bucket)
       cells present, minimum 118; final holdings max 6 (Otto, 6).
     Two 30-day runs: identical but `wall`. social-cafe 300-day on the spike build: sha-256 =
       ad49c723…c64b = E-0. `cargo test --workspace --no-fail-fast`: exit 0, 457 passed (456 + the
       reader), 0 failed.
     Verdict: PASS — no framework gap; F-36 … F-46. No precursor proposed.
E-5  End of this planning branch: check_doc_headings → 143 numbered sections across 22 documents, none
     duplicated; check_decision_ids → 47 ids, all distinct (ARC-37 a proposal in this file only).
     Docs-only branch; no cargo gate run beyond E-3 and the spike.

--- after 11d merged; planning 11e (branch mvp0/s9-11e-plan, base main @ 70e532f) ---

E-6  The R-S9-1 spike for 11e, 2026-10-07. Local scratch branch `scratch/11e-spike` from 70e532f,
     scratch commits 22a6b58 (packs, content) and d856f1d (sizing), never pushed, deleted after
     (`git ls-remote --heads origin | grep -c scratch` → 0). Logs in /tmp/s9-11e-plan/.
     Flows under test (the operator's list): employment's shift Process reading presence and stating
     wage-due, reduced by economy into a wallet move (ARC-28); a purchase at a shop place stated through
     inventory's checked constructor and paid through economy; consumption through inventory's checked
     constructor; all attempted by the unchanged paced controller through complete affordances.
     Built: inventory + items-produced/items-consumed with produce/consume and admit_*; systems/
     employment (job: section, hired, Employment, employed-by, shift Process, attendance from Presence and
     person-entered-place, shift-started/-ended, wage-due, items-produced via produce); systems/economy
     (economy: section with wallet and an organization's shop, Wallet, Shop, funded, shop-opened,
     money-transferred, wage-unpaid, buy offered complete per priced kind at a shop, wages on wage-due with
     no system dependency, wallet and listing disclosure); systems/consumption (eat/drink by category,
     complete offers per held kind, validate + consume); three lines each in systems/installed;
     market-town + two organizations, an economy: block per person, jobs for alice and felix.
     Changed paths vs 70e532f: Cargo.lock, systems/{economy,employment,consumption}/**,
     systems/inventory/src/{admit,event,lib,system}.rs, systems/installed/{Cargo.toml,src/lib.rs},
     worlds/market-town/** — 29 files, nothing else. Cargo.lock: +3 [[package]] (mineworld-consumption,
     -economy, -employment), no `source`; +3 names in mineworld-installed-systems' dependencies.
     Compiled on the first build.
     validate: valid; ids 1–38 = 11d's, cafe-company 39, corner-store 40; 129 genesis facts = 101 + 10
       stocked (organizations) + 14 funded + 2 shop-opened + 2 hired.
     Run 1 (first sizing: wallets 50 000, wages 1 500/h, production per hour), 30 days, --save: exit 0,
       faults 0, 37 999 facts, 3.5 s; buy accepted 372, eat 249, drink 127, give 1 329, no rejected
       request line; shift-started/ended 60/60, wage-due 58, money-transferred 402 (372 purchases + 30
       wages), wage-unpaid 28 → the employers were drained by wages ~10× their takings (F-51).
     Criterion for the 300-day runs, stated before the first of them (= E-9 b): faults 0; every seat
       moved and talked in every bucket; per bucket ≥ 1 purchase, both job holders paid, ≥ 1 produced,
       ≥ 1 eaten or drunk, ≥ 1 given; zero wage-unpaid; no wallet ever below 100 (the cheapest price);
       money conserved; cost ≤ 60 s.
     Run 2 (people 200 000, job holders 20 000, organizations 20 000, alice 120/h, felix 100/h,
       production per full shift), 300 days, --save: exit 0, faults 0, 372 621 facts, wall 34.2 s; buy
       2 627, eat 1 690, drink 943, give 12 797; every bucket ≥ 235 purchases, 57–60 wages paid,
       ≥ 191 produced, ≥ 238 consumed, ≥ 1 196 given; zero wage-unpaid; money conserved (2 080 000).
       FAILED the wallet condition: felix (12) fell to 3 (bucket 2) and lived on each wage; the store
       (40) fell from 27 354 to 6 388 over the run (F-51). The instrument sees a drain.
     Run 3 (felix 200/h, store 300 000; nothing else changed), 300 days, --save: exit 0, faults 0,
       372 755 facts, wall 34.5 s; buy 2 710, eat 1 710, drink 1 007, give 13 032; no `move 0`/`talk 0`
       in any bucket; per bucket: purchases 349, 256, 276, 265, 250, 271, 247, 264, 258, 274; wages paid
       59, 59, 60, 60, 58, 59, 58, 58, 59, 60 (wage-due equal, wage-unpaid 0); produced 191–203; consumed
       251–358; given 1 253–1 382. Lowest wallet over the run: felix 2 313, store 246 190, café 21 440,
       alice 16 710, every other person ≥ 95 150. Money conserved: 2 360 000 at genesis and at the end.
       Final holdings: no person above 6 (Otto 6, grace 6). → PASS of every E-9 b condition.
     Determinism (run 3's content): two 30-day runs identical but `wall`; 15 days saved then resumed to
       30 = the uninterrupted 30-day save (38 004 facts, fingerprint aaface7abc5a3097, both).
     social-cafe 300-day on the spike build: sha-256 of all but `wall` = ad49c723…c64b = E-0.
     `cargo test --workspace --no-fail-fast`: exit 0, 480 passed (479 + the scratch reader), 0 failed.
     Verdict: PASS — no framework gap; F-47 … F-56. No precursor proposed.
E-7  End of this planning branch: check_doc_headings → 143 numbered sections across 22 documents, none
     duplicated; check_decision_ids → 48 ids, all distinct (ARC-38 a proposal in this file only; absent
     from every origin/* branch after `git fetch`). Docs-only branch; no cargo gate beyond the spike.

--- after 11e merged; planning 11f (branch mvp0/s9-11f-plan, base main @ 2dddda8) ---

E-8  The 11f audit's measurements, 2026-10-07, on 2dddda8 (debug, opt-level 1); logs in /tmp/s9-11f-plan/.
     No code was written; nothing was committed but this ledger.
     History: `git log --first-parent --merges HEAD` holds exactly one `Merge pull request #43 from
       yuema137/mvp0/pr-11d-owning-things` = 70e532f383ff… (parents e3a1106, ff3509c) and one `#46 from
       yuema137/mvp0/pr-11e-work-money-shops` = 2dddda86a434… (parents 4f2a4cd, d5efd02).
       `git diff --name-only M^1 M` minus systems/, worlds/, Cargo.lock: only
       .structured-coding/plans/mvp0/{handoff,step-10-market}.md and docs/{DECISIONS,MODULE_SPEC,
       MVP_STATUS,PACKAGE_FORMAT}.md, for both merges. Cargo.lock of 70e532f: +3 [[package]]
       (mineworld-inventory, -item, -item-transfer), none with a source, and three names in
       mineworld-installed-systems' list — check 1's positive case, by hand, before the test exists.
     Structure: `cargo tree --offline -i mineworld-<pack> -e normal,dev,build --depth 1` for the six packs:
       every direct dependent is under systems/ (the packs themselves and mineworld-installed-systems).
       `cargo tree -p <crate> -e normal,dev,build`: kernel, contracts, server, authoring, sdk,
       rule-controller reach no market pack; persistence, worldpack and cli do (F-58); persistence by
       normal and build edges alone reaches none. `git grep` of the six crate names (both spellings) in
       *.rs and *Cargo.toml outside systems/, worlds/, tests/acceptance/: no match.
       `cargo metadata --offline --locked --format-version 1`: FAILED (anstyle-wincon not downloaded);
       `--no-deps`: 0.03 s, dependencies listed with kind and path (F-60).
     Market Town, 2 days: `mineworld run worlds/market-town --headless --seed 7 --days 2 --save
       /tmp/s9-11f-plan/mt2` → buy 23, eat 20, drink 9, give 85 accepted; 2 hired, 4 shift-started, 4
       shift-ended, 4 wage-due, 27 money-transferred (= 23 purchases + 4 wages), 14 items-produced, 29
       items-consumed. `inspect`: head revision 1 946 at t171 910 (day 2, 23:45:10) (F-64); every cause
       resolves (2 763 facts). The work → earn half of Milestone C exists in a 2-day save.
     Verdict: no framework gap; F-58 … F-69. No precursor and no behaviour change proposed.
E-9  End of this planning branch: check_doc_headings → 143 numbered sections across 22 documents, none
     duplicated; check_decision_ids → 49 ids, all distinct (11f proposes no new id: an ARC-35 note,
     QS-64). Docs-only branch; no cargo gate run beyond E-8's commands. The post-merge update for 11e is
     the separate docs PR #47 (branch docs/s9-11e-merged), which touches the step header, overall §7 and
     MVP_STATUS — none of the lines this branch changes.
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

## 9.4 Evidence — PR 11d

Written by the 11d implementation session only (`E-D<n>`).

```text
--- PR 11d (branch mvp0/pr-11d-owning-things, base main @ e3a1106) ---

E-D0 Base captures on e3a1106 before any edit, 2026-10-07 (debug, opt-level 1):
     300-day seed-7 social-cafe: 339 lines, sha-256 of all but `wall` =
     ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-0; wall 13.2 s.
     `mineworld validate worlds/social-cafe` sha-256 ebcd60a0…f56a8 = E-C0's.
     The R-S9-1 spike commit 74bf597 is still in the object store (unreachable, never pushed); it is
     used as a reference only, and nothing is cherry-picked from it.
E-D1 D-C1 specs: check_decision_ids 48 ids, all distinct (ARC-37 new); check_doc_headings 143 sections
     across 22 documents, none duplicated. ARC-37 absent from every origin/* branch after `git fetch`.
E-D2 D-C2 item: `cargo test -p mineworld-item` → tests/item.rs 4 passed, 0 failed; clippy -p
     mineworld-item --all-targets -D warnings clean; fmt clean. Cargo.lock: +1 path package
     (mineworld-item, no `source`) — a workspace member is locked whether or not it is installed.
     First run of the reduction-refusal test failed on its own fixture (a hand-written `{entity, type}`
     JSON that did not decode, EventTypeMismatch); fixed by re-pointing a real ItemId's encoding.
E-D3 D-C3 inventory: `cargo test -p mineworld-inventory` → inventory.rs 8 passed, persisted.rs 1
     passed, 0 failed; item 4 passed; clippy -p inventory -p item --all-targets -D warnings clean; fmt
     clean. Cargo.lock: +1 path package (mineworld-inventory).
     M-D2 (item's react writes nothing): 5 of 8 inventory tests FAILED, each at genesis with
       `FactRefusedByOwner { system: "inventory", event_type: "stocked", reason: PreconditionFailed }`
       — the declared-kind check at reduction is real. Reverted.
     M-D3 (the transfer reduction skips admit_transfer): 2 FAILED —
       transfers_stated_past_the_constructor… (the "to oneself" case was Accepted, writing both sides
       from one read) and a_person_carries_at_most_six… (a forged transfer past capacity Accepted).
       Observed deviation from §4.4.3's expectation: the *over-transfer* case alone survives M-D3,
       because `Holdings::removing` refuses a count larger than held (checked_sub) and the reduction
       turns its None into FactRefusedByOwner — a second guard, not the owner's rule. Recorded D-D2;
       the mutation is still caught by two cases. Reverted.
     M-D4 constructor half (can_take always true): a_person_carries_at_most_six… FAILED ("bob is
       full"). The seeding capacity test survives by design (the seed sums authored counts; no state
       exists at seeding). The offer and dispatch halves run in D-C4. Reverted;
       `git grep MUTATION -- systems` empty.
     F-38 reproduced: serde_json::to_vec(&BTreeMap<ItemId, u32>) → Err("key must be a string")
       (throwaway test file, deleted, never committed).
     D-8: persisted.rs creates a SQLite save, makes 2 of 4 passes, drops, resumes into a freshly
       composed world: state bytes equal the saved ones; alice's holdings located (apple 2, coffee 1,
       tea 1); the remaining 2 passes end in the uninterrupted world's state, and the last facts are
       byte-identical; verify() passes.
E-D4 D-C4 item-transfer: `cargo test -p mineworld-item-transfer` → give 4, removable 3, paced 3 passed,
     0 failed; clippy --all-targets -D warnings clean (after two lints in test code: an elidable
     lifetime, a complex tuple type named `Step`); fmt clean. Cargo.lock: +1 path package.
     D-6 paced run (10 days, seed 7, pace 900 s, 3 seats in one café): 1 630 requests, 351 gives by all
       3 people, every give accepted, transfers per day 37, 36, 30, 28, 28, 36, 45, 39, 32, 40; every
       items-transferred caused by the give that asked for it; holdings conserved, nobody past six.
       First run FAILED on the test's own constant (expected 4 items; HOLDINGS authors 3) — fixed to
       3, the claim unchanged.
     D-7: a world without item-transfer offers no give to anybody, answers a give Unavailable, holds
       still; the scripted run's facts (genesis + 2 walks) equal the enabled run's minus its 3
       items-transferred, compared by (instant, type, payload, cause); installing item-transfer
       without inventory → SystemDependencyMissing { item-transfer, inventory }.
     M-D4 offer half (can_take always true): give.rs a_give_to_a_full_person… FAILED ("Bob carries six":
       both offers available) and inventory's a_person_carries_at_most_six… FAILED; two capacity tests,
       not three: the authored-holdings test sums its own counts at seeding (no state exists then) and
       survives by design. Reverted.
     M-D5 (offers built with Offer::new, incomplete): give.rs 2 FAILED ("a complete affordance carries
       its request"); paced.rs the_unchanged_paced_controller_gives… FAILED ("at least two people give:
       {} (0 gives)") and two_runs… FAILED ("the comparison is of runs that gave"). Reverted.
     M-D6 (WITHOUT = true: the "without" world enables item-transfer): removable.rs 2 FAILED ("alice is
       offered a give in a world without item-transfer"; the fact comparison). Reverted;
       `git grep MUTATION -- systems` empty; all three packs green again.
E-D5 D-C5 install: `cargo test --no-fail-fast -p mineworld-installed-systems -p mineworld-worldpack` →
     installed 3, worldpack unit 4, content_kinds 1, refusals 38, social_cafe 15, structure 2, doc 1;
     0 failed. Diff: systems/installed/Cargo.toml +3, src/lib.rs +3, Cargo.lock +3 (the three names
     in mineworld-installed-systems' dependencies). I-4 on this working tree (debug, opt-level 1):
     300-day seed-7 social-cafe sha-256 of all but `wall` = ad49c723…c64b = E-0; faults 0; 365 330
     facts; fingerprint 59339a9c281829c9; wall 12.2 s. `validate worlds/social-cafe` byte-identical
     to E-D0's (cmp; sha ebcd60a0…f56a8).
E-D6 D-C6 market-town, on 069e9e4's build (debug, opt-level 1), 2026-10-07. In the order I-7 binds:
     a. `mineworld validate worlds/market-town` → valid; ids 1–18 identical to social-cafe's (diff of
        the id lines empty), kinds 19–38; 101 genesis facts = 53 + 20 kinds + 28 authored holdings.
     b. ACTIVITY FIRST. `run worlds/market-town --headless --seed 7 --days 300 --save …` → exit 0,
        faults 0, no rejected/unavailable request line, 365 126 facts; requests: give accepted 17 639,
        move 173 161, talk 60 520. Every seat moved and talked in every 30-day bucket (minima: move
        1 370, talk 412 per seat-bucket).
        Scratch reader (systems/item-transfer/tests/scratch_reader.rs on scratch/11d-reader, decoding
        with inventory's own Stocked/ItemsTransferred; never pushed): gives per bucket 1 727, 1 749,
        1 720, 1 803, 1 719, 1 810, 1 725, 1 841, 1 772, 1 773 (total 17 639 = the accepted gives);
        all 110 (seat, bucket) cells present — seats 7–15, 17, 18 × buckets 0–9 — minimum 114 per
        cell; the most any person ever held was 6 (all 12, Otto included); final holdings total 30
        (conserved), Otto 6. → PASS.
     c. ONLY THEN DETERMINISM. Two 30-day seed-7 runs: identical but `wall` (diff empty; sha-256 of all
        but wall 6e27e8b0…7d71a19). A save run to day 15 (give accepted 884) then resumed to day 30
        (843 more) vs the uninterrupted 30-day save: both 37 090 facts, fingerprint 46f09300e34ad1ad
        = the unsaved run's; every fact dumped from each save (scratch `dump`) byte-identical (cmp;
        sha-256 6523edd5…3f96). → PASS.
     d. COST. The 300-day run with --save: wall 32.5 s (≤ 60 s). → PASS.
     M-D7 (PERSON_CAPACITY = u32::MAX, scratch commit, never pushed): 300 days with --save, exit 0,
       faults 0, wall 32.8 s; reader FAILED "a seat that did not give in a bucket": gives 993 in
       bucket 0, 17 in bucket 1, none after; 92 of 110 cells empty; Otto (16) held 30 of 30 items at the
       end, the most any seat ever held 10. The instrument sees the sink. Scratch branch deleted.
     D-10: `git diff --no-index --stat worlds/social-cafe worlds/market-town` → 34 files, 167+ 49−; the
       removals are README.md (rewritten) and world.yaml's id/name only; additions: world.yaml header
       paragraph, appended systems with comment, items list; 20 items/ files; one holdings: block with
       its comment per person file. No place file differs. → PASS.
     D-5 through the real CLI: a /tmp copy with bob authored at 7 items → `validate` exit 1:
       "[mineworld] …/people/bob.yaml: the 'inventory' system refused 'bob''s 'holdings' section:
       TargetUnavailable".
E-D-final on 341f2f2 (clean tree; final executable head — later commits are Markdown only), base
     e3a1106, 2026-10-07; logs /tmp/s9-11d/final/:
     cargo fmt --all --check                                           PASS
     cargo clippy --workspace --all-targets --all-features -D warnings PASS
     cargo test --workspace --no-fail-fast    479 passed, 0 failed, 0 ignored across 104 test binaries
                                              (main's 456 + 23 new: item 4, inventory 9, item-transfer
                                              10); 173 s wall
     kill_and_resume                          cafe PASS (0.3 s), clock PASS (0.1 s)
     check_decision_ids                       48 ids, all distinct
     check_doc_headings                       143 sections across 22 documents, none duplicated
     I-2 scan                                 the_precursors_add_no_market_concept PASS inside the run
                                              (rows 11a/11b/11c read as merged; no row or entry added)
     D-2 / I-4                                300-day seed-7 social-cafe sha-256 of all but wall =
                                              ad49c723…c64b = E-0; faults 0; 365 330 facts; wall 12.0 s;
                                              `validate worlds/social-cafe` byte-identical to E-D0's
     D-1                                      `git diff --name-only e3a1106...HEAD`: 83 paths — docs/
                                              {DECISIONS,MODULE_SPEC,MVP_STATUS,PACKAGE_FORMAT}.md,
                                              .structured-coding/plans/mvp0/{handoff,step-10-market}.md,
                                              Cargo.lock, systems/README.md, systems/installed/
                                              {Cargo.toml,src/lib.rs}, systems/item/** (10),
                                              systems/inventory/** (12), systems/item-transfer/** (11),
                                              worlds/market-town/** (40: README, world.yaml, 20 items,
                                              12 people, 6 places); paths outside the allowed set: 0.
                                              `git diff e3a1106...HEAD -- Cargo.lock`: +3 [[package]]
                                              (mineworld-item, -inventory, -item-transfer), none with a
                                              `source`; +3 names in mineworld-installed-systems'
                                              dependency list; nothing else.
     M-D1                                     `// MUTATION M-D1` added to kernel/src/lib.rs in the working
                                              tree → the same filter (`git diff --name-only e3a1106`)
                                              listed `kernel/src/lib.rs`; reverted (git checkout), count
                                              of outside paths back to 0, tree clean.
     CI: none configured (S13).

     D-1 … D-11 at a glance:
     D-1  PASS (above)            D-2  PASS (above; no existing test edited — D-1's list has none)
     D-3  PASS (E-D2; M-D2 E-D3)  D-4  PASS (E-D3; M-D3, D-D2)
     D-5  PASS (E-D3, E-D4, E-D6 CLI; M-D4, D-D4)
     D-6  PASS (E-D4; M-D5)       D-7  PASS (E-D4; M-D6)
     D-8  PASS (E-D3 persisted; F-38 reproduced)
     D-9  PASS a, then b, then c, then d (E-D6; M-D7)
     D-10 PASS (E-D6)             D-11 PASS (D-C1 committed first, 27dfbca; doc checks above)
```

## 9.5 Evidence — PR 11e

Written by the 11e implementation session only (`E-E<n>`).

```text
--- PR 11e (branch mvp0/pr-11e-work-money-shops, base main @ 4f2a4cd) ---

E-E0 Base captures on 4f2a4cd before any edit, 2026-10-07 (debug, opt-level 1):
     300-day seed-7 social-cafe: exit 0, 339 lines, sha-256 of all but `wall` =
     ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-0; wall 12.0 s.
     `mineworld validate worlds/social-cafe` sha-256 ebcd60a0…f56a8 = E-D0's.
     `git diff 70e532f 4f2a4cd -- systems worlds Cargo.lock` empty: §8.6's audit stands (no re-audit).
     The R-S9-1 spike commits 22a6b58 and d856f1d are still in the object store (unreachable, never
     pushed); used as a reference only — the packs are written to §4.5's layout, not cherry-picked.
E-E1 E-C1 specs: check_decision_ids 49 ids, all distinct (ARC-38 new); check_doc_headings 143 sections
     across 22 documents, none duplicated. ARC-38 absent from every origin/* branch after `git fetch`.
E-E2 E-C2 inventory: `cargo test -p mineworld-inventory` → inventory.rs 10 passed (8 + 2 new),
     persisted.rs 1 passed, 0 failed; clippy -p mineworld-inventory --all-targets -D warnings clean;
     fmt clean. No dependency change (Cargo.lock untouched).
     E-3: through produce/consume, kiosk coffee 20 → 23 and alice's last tea removed (no zero entry),
     nobody else changed; each fact Participants = [holder], caused by the request. Past the
     constructors (forged, workshop): produced undeclared / 0 / into a place / past bob's six, consumed
     undeclared / 0 / from a place / more than held → each FactRefusedByOwner { inventory,
     items-produced | items-consumed } with PreconditionFailed (TargetUnavailable for past six), state
     bytes unchanged, and the constructor refuses each with the same reason; positive controls (forged
     valid production up to exactly six, forged valid consumption) reduced. → PASS.
     M-E2 (`// MUTATION M-E2`: the items-consumed arm skips admit_consumption): the refusal test
     FAILED — "inventory must refuse as the owner, but got Ok(… Accepted … items-consumed …
     ActionId(6))", the count-zero case (DE-3). Reverted; `git grep MUTATION -- systems` empty;
     10 + 1 passed again.
E-E3 E-C3 employment: `cargo test -p mineworld-employment` → employment.rs 5, persisted.rs 1,
     removable.rs 3 passed, 0 failed; clippy -p mineworld-employment --all-targets -D warnings clean;
     fmt clean. Cargo.lock: +1 path package (mineworld-employment, no `source`).
     E-6 (job 08:00–12:00, 120/h, coffee 4 + croissant 2 per full shift; numbers worked by hand in the
     test's doc): shift-started present alice true, bob true, carol false, dave false; shift-ended
     worked 14 400 / 7 200 / 3 600 / 0; wage-due 480 / 240 / 120 and none for dave; items-produced for
     the employer coffee 4, croissant 2 (alice), coffee 2, croissant 1 (bob), coffee 1 (carol: the
     croissant's 0.5 floored and skipped); employer's stock 1 → coffee 8, croissant 3; every shift
     fact caused by its Process; wage-due Participants = [employee, employer]; no money-transferred.
     Day 1: one start per employee at 08:00 and one end at 12:00, nothing between shifts. hired:
     genesis, visible to the employee, Employment written, employed-by edge to the organization.
     Disclosure: to the employee only. Section: until ≤ from, an empty shift, an unknown key, a zero
     production and a negative wage refused at decode. → PASS.
     Persisted (E-6 restart): saved at 10:30 (alice's open span since 08:00, bob's closed 7 200 s —
     located in the saved state), resumed into a freshly composed world: both Employments equal; the
     shift ends worked [14 400, 7 200, 3 600, 0]; the facts after the stop byte-identical to the
     uninterrupted world's; verify() passes. → PASS.
     E-7 (employment half): without economy, three days state 7 wage-dues and 12 shift-ends with no
     money fact and no fault; a world without employment runs the same walks and its facts equal the
     world-with's minus hired/shift-*/wage-due/items-produced (causes compared by the causing fact's
     instant, type and payload, since the hired facts shift event ids); employment without inventory →
     SystemDependencyMissing { employment, inventory }. → PASS.
     M-E6 (`// MUTATION M-E6`: the reaction ignores a departure, `&& false`): employment.rs
     a_shift_is_paid… FAILED "seconds at the workplace during the shift: left [… ("bob", 14400) …]
     right [… ("bob", 7200) …]" (paid for the whole shift), and persisted.rs FAILED too. Reverted;
     `git grep MUTATION -- systems` empty; 5 + 1 + 3 passed again.
E-E4 E-C4 economy: `cargo test -p mineworld-economy` → buy.rs 6, paced.rs 3, removable.rs 3, wages.rs 1
     passed, 0 failed; clippy -p mineworld-economy --all-targets -D warnings clean; fmt clean.
     Cargo.lock: +1 path package (mineworld-economy, no `source`).
     E-5 (store: apple 100, bread 300, juice 200 priced, juice not stocked, pen unpriced): alice (1 000)
     offered apple ✓ bread ✓ juice ✗; bob (150) apple ✓ bread ✗ juice ✗; erin (carrying six) all ✗;
     every ✗ TargetUnavailable; carol on the street none; no buy carries a target. alice buys bread →
     money-transferred alice → corner-store 300, Visibility::Place(store), and items-transferred
     corner-store → alice bread 1, Participants, both Causation::Action; wallets 700 / 1 300. At
     dispatch: cannot pay, out of stock, cannot carry → TargetUnavailable; on the street, unpriced →
     NoSupportedInteraction; each writes nothing. Listing seen by bob in the store: (100, 3), (300, 2),
     (200, 0), operator corner-store; not seen by carol on the street; after alice buys an apple bob
     sees (100, 2) — CP-7's perception path. Wallet disclosed to its holder only. Paced, 10 days, seed
     7, pace 900 s, seats alice/bob/carol, store stocked apple 40 + bread 40: 1 507 requests, 7 buys by
     2 buyers, every buy accepted, each caused exactly [money-transferred, items-transferred] paid by
     the asker; nobody past six; two runs byte-identical; the controller's manifest names none of the
     six market packs. → PASS.
     E-4: forged money-transferred (ledger stater) — more than the payer holds, zero, to oneself, to a
     place → FactRefusedByOwner { economy, money-transferred, PreconditionFailed }, state bytes
     unchanged; positive control (bob → alice 150, to exactly zero) reduced. Wages (employment
     installed): alice's 480 from corner-store (1 000) → one money-transferred caused by
     Causation::Event(her wage-due), Participants; bob's 480 from poor-co (100) → wage-unpaid caused by
     his wage-due, no money-transferred; wallets alice 1 480, corner-store 520, bob 150, poor-co 100. →
     PASS. Static, recorded not tested: employment cannot write a Wallet (no write token; its manifest
     does not name economy).
     E-7 (economy half): economy's declaration does not depend on employment, and a world without
     employment sells; a world without economy offers no buy, answers Unavailable, holdings and wallets
     unchanged; economy without inventory → SystemDependencyMissing { economy, inventory }. → PASS.
     M-E3 (`// MUTATION M-E3`: the wage answer pays without the balance check): wages.rs FAILED —
     "advances: FactRefusedByOwner { economy, money-transferred, PreconditionFailed }": the payment
     was refused by the reduction instead, as §4.5.3 predicted. Reverted.
     M-E4 (offers built with Offer::new, incomplete): buy.rs a_person_in_a_shop… FAILED ("a complete
     affordance carries its request"); paced.rs the_unchanged… FAILED ("at least two people buy: {}
     (0 buys)") and two_runs… FAILED ("the comparison is of runs that bought"). Reverted.
     M-E5 (`purchasable` ignores the balance, `|| true`): buy.rs a_person_in_a_shop… FAILED ("bob's 150
     pays for an apple (100), not for bread (300)": bread offered available) and a_buy_is_refused…
     FAILED (bob's bread passed validate and was refused by the reduction); both paced tests FAILED the
     same way. Reverted; `git grep MUTATION -- systems` and grep of the untracked packs empty; 6 + 3 +
     3 + 1 passed again.
E-E5 E-C5 consumption: `cargo test -p mineworld-consumption` → eat.rs 3, paced.rs 2, removable.rs 2
     passed, 0 failed; clippy -p mineworld-consumption --all-targets -D warnings clean; fmt clean.
     Cargo.lock: +1 path package (mineworld-consumption, no `source`).
     E-8: alice (coffee, croissant, mug) offered [drink coffee, eat croissant], no target, available;
     bob [drink tea]; carol nothing; the mug never. alice eats the croissant and drinks the coffee →
     one items-consumed each (and nothing else), Participants = [alice], Causation::Action; she keeps
     the mug. Refused at dispatch, writing nothing: eat a drink, drink a food, eat goods, a meal with a
     target → NoSupportedInteraction; eat what one does not carry → PreconditionFailed. Paced, 10
     days, seed 7, pace 900 s: 11 meals by 2 people, every one accepted, each exactly one
     items-consumed caused by it and eaten by the asker; at the end alice holds only the mug and bob
     nothing; two runs byte-identical. → PASS.
     E-7 (consumption half): without consumption, no eat/drink offered to anybody, both answered
     Unavailable, holdings unchanged; consumption without inventory → SystemDependencyMissing {
     consumption, inventory }. → PASS.
     M-E7 (`WITHOUT = true`: the "without" world enables consumption) and M-E8 (`// MUTATION M-E8`:
     validate's category check removed) applied together, each failing in a binary the other cannot
     touch: removable.rs FAILED "alice is offered a meal in a world without consumption" (offers do not
     pass through validate); eat.rs the_wrong_meal… FAILED "a drink is not eaten: left Accepted, right
     Rejected(NoSupportedInteraction)" (eat.rs's world does not read WITHOUT). Reverted; grep MUTATION
     over systems/ empty; 3 + 2 + 2 passed again.
E-E6 E-C6 install: `cargo test --no-fail-fast -p mineworld-installed-systems -p mineworld-worldpack` →
     installed 3, worldpack unit 4, content_kinds 1, refusals 38, social_cafe 15, structure 2, doc 1;
     0 failed. Diff: systems/installed/Cargo.toml +3, src/lib.rs +3, Cargo.lock +3 (the three names in
     mineworld-installed-systems' dependencies). I-4 on this working tree (debug, opt-level 1):
     300-day seed-7 social-cafe sha-256 of all but `wall` = ad49c723…c64b = E-0; wall 12.3 s.
     `validate worlds/social-cafe` byte-identical to E-E0's (cmp).
E-E7 E-C7 market-town, content commit 1611c1a (the spike's run-3 sizing, unchanged), on c2d9b8f's
     build (debug, opt-level 1), 2026-10-07. The scratch reader lived on local branch
     scratch/11e-reader (systems/employment/tests/scratch_reader.rs, decoding JSON payloads; commits
     fe882e1, 5feffc1, 8b9633d), never pushed, deleted after (`git branch -D`; `git ls-remote --heads
     origin | grep -c scratch` → 0). Logs in /tmp/s9-11e/. In the order I-7 binds:
     a. `mineworld validate worlds/market-town` → valid; ids 1–38 exactly 11d's (places 1–6, people
        7–18, kinds 19–38), cafe-company 39, corner-store 40; 129 genesis facts = 101 + 10 stocked
        (organizations) + 14 funded + 2 shop-opened + 2 hired. → PASS.
     b. ACTIVITY FIRST. Sizing run 1 — the only sizing run; it passed, so no re-sizing — `run
        worlds/market-town --headless --seed 7 --days 300 --save /tmp/s9-11e/mt300-run1` → exit 0,
        faults 0, 372 755 facts; requests: buy accepted 2 710, eat 1 710, drink 1 007, give 13 032, move
        173 113, talk 60 524, no rejected or unavailable line. Every seat moved and talked in every
        30-day bucket (minima: move 1 395, talk 388 per seat-bucket). Reader, every condition stated in
        §4.5.3 E-9 b before measuring:
          purchases per bucket   349, 256, 276, 265, 250, 271, 247, 264, 258, 274        (≥ 1 ✓)
          wages paid alice/felix [30,29] [30,29] [30,30] [30,30] [30,28] [30,29] [30,28] [30,28]
                                 [30,29] [30,30]                                       (each ≥ 1 ✓)
          items-produced facts   203, 197, 194, 195, 194, 195, 198, 196, 200, 197       (≥ 1 ✓)
          items-consumed facts   358, 256, 275, 265, 251, 270, 251, 262, 258, 271       (≥ 1 ✓)
          gives                  1 382, 1 317, 1 293, 1 316, 1 253, 1 274, 1 262, 1 349, 1 290, 1 296
                                 (≥ 1 ✓; per seat reported, not required: every seat gave in every
                                 bucket, fewest 77)
          wage-due 590, wage-unpaid 0                                                   (✓)
          lowest wallet ever     felix 2 313, alice 16 710, café 20 000 (its opening balance), store
                                 246 190, every other person ≥ 95 150 — all ≥ 100, the cheapest price
                                 (✓; Otto and dev never spend: 200 000)
          money                  2 360 000 at genesis and at the end                     (✓)
          most anybody held      6, all twelve people                                   (✓)
        At the end the café holds 170 items and 670 100; the store holds 0 items (it sells each
        morning's production the same day) and 246 290. → PASS. These numbers equal the spike's run 3
        (§9 E-6) exactly: the same content on the same packs' semantics.
     c. ONLY THEN DETERMINISM. Two 30-day seed-7 runs: identical but `wall` (diff empty; sha-256 of all
        but wall cf859e57…24ed; 38 004 facts, faults 0). A save run to day 15 (19 175 facts) then
        resumed to day 30 ("resumed … at revision 14500 (snapshot 14464 + 36 re-executed)") vs the
        uninterrupted 30-day save: both 38 004 facts, fingerprint f4055c0cff59c9fe; every fact dumped
        from each save (scratch `dump`) byte-identical (cmp; sha-256 a5a50bf6…16e4). → PASS.
     d. COST. The 300-day run with --save: wall 33.5 s (≤ 60 s). → PASS.
     M-E9 (scratch commit: `consumption` removed from world.yaml's systems): 300 days with --save, exit
       0, faults 0, wall 35.2 s, buy accepted 42, give accepted 253; reader FAILED: purchases 42 in
       bucket 0 and none after; gives 253 then none (everybody holds six: the 11d sink, everywhere);
       consumed 0; alice paid 30 then 13 then never — wage-unpaid 257 (the café's wallet fell to 30);
       "holder 39 fell to 30". QS-35's failure, seen by the instrument. Scratch branch deleted.
     E-10: `git diff --no-index worlds/social-cafe worlds/market-town` → 36 files, 293+ 49−; the
       removals are README.md (rewritten in 11d) and world.yaml's id/name only. Beyond 11d's delta:
       world.yaml's header paragraph, the three appended systems with their comment, the
       `organizations:` list; organizations/cafe-company.yaml and corner-store.yaml; per person file an
       appended commented `economy:` block, and `job:` for alice and felix. No place file differs. →
       PASS.
E-E-final on 15c4651 (clean tree; final executable head — the last non-Markdown commit is 1611c1a, and
     every later commit is Markdown only), base 4f2a4cd, 2026-10-07; logs /tmp/s9-11e/final/:
     cargo fmt --all --check                                           PASS
     cargo clippy --workspace --all-targets --all-features -D warnings PASS
     cargo test --workspace --no-fail-fast    510 passed, 0 failed, 0 ignored across 121 test binaries
                                              and doc-test runs (11d's 479 + 31 new: inventory 2,
                                              employment 9, economy 13, consumption 7); 176 s wall
     kill_and_resume                          cafe PASS (0.2 s), clock PASS (0.1 s)
     check_decision_ids                       49 ids, all distinct
     check_doc_headings                       143 sections across 22 documents, none duplicated
     I-2 scan                                 the_precursors_add_no_market_concept PASS inside the run
                                              (no row or entry added)
     E-2 / I-4                                300-day seed-7 social-cafe sha-256 of all but wall =
                                              ad49c723…c64b = E-0; faults 0; 365 330 facts; wall 12.1 s;
                                              `validate worlds/social-cafe` byte-identical (E-E6)
     E-1                                      `git diff --name-only 4f2a4cd...HEAD`: 74 paths —
                                              .structured-coding/plans/mvp0/{handoff,step-10-market}.md,
                                              Cargo.lock, docs/{DECISIONS,MODULE_SPEC,MVP_STATUS,
                                              PACKAGE_FORMAT}.md, systems/README.md,
                                              systems/consumption/** (11), systems/economy/** (16),
                                              systems/employment/** (14), systems/installed/{Cargo.toml,
                                              src/lib.rs}, systems/inventory/{README.md, src/{admit,event,
                                              lib,system}.rs, tests/{inventory.rs,support/mod.rs}},
                                              worlds/market-town/{README.md, world.yaml,
                                              organizations/{cafe-company,corner-store}.yaml, people/*.yaml
                                              (12)}; non-Markdown paths outside systems/, worlds/ and
                                              Cargo.lock: 0. `git diff 4f2a4cd...HEAD -- Cargo.lock`: +3
                                              [[package]] (mineworld-consumption, -economy, -employment),
                                              none with a `source`; +3 names in
                                              mineworld-installed-systems' dependency list; nothing else.
     M-E1                                     `// MUTATION M-E1` added to kernel/src/lib.rs in the working
                                              tree → the same filter (`git diff --name-only 4f2a4cd`)
                                              listed `kernel/src/lib.rs`; reverted (git checkout), count
                                              of outside paths back to 0, tree clean.
     CI: none configured (S13).

     E-1 … E-12 at a glance:
     E-1  PASS (above; M-E1)               E-2  PASS (above; no existing test outside
                                                inventory's edited; inventory's 8 existing cases
                                                unchanged, two added)
     E-3  PASS (E-E2; M-E2, DE-3)          E-4  PASS (E-E4; M-E3)
     E-5  PASS (E-E4; M-E4, M-E5)          E-6  PASS (E-E3; M-E6)
     E-7  PASS (E-E3, E-E4, E-E5; M-E7)    E-8  PASS (E-E5; M-E8)
     E-9  PASS a, then b, then c, then d (E-E7; M-E9)
     E-10 PASS (E-E7)                      E-11 PASS (E-C1 committed first, 75d3dc1; doc checks above)
     E-12 PASS (MVP_STATUS: the 11d rows kept and marked superseded as current market-town evidence;
          a new capability row and evidence row; the artefact row updated — DE-8)
```

## 9.6 Evidence — PR 11f

Branch `mvp0/pr-11f-proof`, worktree `/Users/yuema137/mineworld-worktrees/s9-11f`, base `main @ e97a408`
(2dddda8 + the docs-only merges #47 and #48; nothing under systems/, worlds/, tools/cli/src/,
persistence/src/, server/src/ or tests/acceptance/ moved since 2dddda8). Debug profile, opt-level 1.
Logs under /tmp/s9-11f/.

```text
E-P1 P-C1 — the ARC-35 note (11f), committed before any test code (P-10).
     python3 scripts/check_decision_ids.py → 49 decision ids, all distinct (no new id: a note).
     python3 scripts/check_doc_headings.py → 143 numbered sections across 22 documents, none duplicated.
E-P0 Baselines on the base build (e97a408's code; 690d584 changes Markdown only), before any test code:
     `mineworld run worlds/social-cafe --headless --seed 7 --days 300` → faults 0, 365 330 facts,
       sha-256 of all but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-0;
       wall 13.8 s.
     `mineworld run worlds/market-town --headless --seed 7 --days 300` → faults 0, 372 755 facts
       (= E-E7), sha-256 of all but `wall` = 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d;
       wall 16.0 s. Both `validate` outputs kept in /tmp/s9-11f/base/ for P-9.
E-P2 P-C2 (c11418c): `cargo test -p mineworld-acceptance --test ac1_composability` → 7 passed, 0.27 s.
     Positive (P-1): 11d 70e532f…: 83 paths, Cargo.lock = mineworld-inventory, -item, -item-transfer
       added (no source) + mineworld-installed-systems changed (dependencies); 11e 2dddda8…: 74 paths,
       mineworld-consumption, -economy, -employment added + installed-systems changed (dependencies);
       0 failures. P-2: the one-commit repository fails "11d: 0 merges of #43 …"; the shallow clone
       (`git clone --depth 1 file://…` of a two-commit repository) fails with exactly one failure naming
       "a shallow clone" and "fetch-depth: 0".
     M-P3 (11d's id → its first parent e3a1106c…): FAIL "11d: the merge of #43 on the first-parent chain
       is 70e532f383ff…, not the recorded e3a1106c…". First seen as a panic of the test's print loop;
       the loop now skips an unfound row, and the check itself reports it (re-run, same message).
     M-P4 (11e's row → #40, mvp0/pr-11c-affordances, c5dc51c0…): FAIL naming 16 paths — clients/
       protocol/ADOPTION.md, cognition/rule-controller/src/{lib,offered,offered_tests,paced,tests}.rs,
       contracts/src/{action,observation}.rs, contracts/tests/…, server/PROTOCOL.md,
       spike/server/src/world.rs, tests/acceptance/… — and "Cargo.lock: mineworld-acceptance 0.0.0
       changed (dependencies) — not a crate under systems/".
     M-P1 + M-P2, local scratch branches scratch/ac1-main and scratch/ac1-mutation from c11418c: a side
       commit adding a comment to kernel/src/lib.rs, a file systems/economy/MUTATION-M-P1.txt, and to
       Cargo.lock a package `rust-decimal-scratch 1.0.0` with source registry+… listed in
       mineworld-economy's dependencies; merged --no-ff as "Merge pull request #999 from
       scratch/ac1-mutation" = 4674c419…; a working-tree scratch row for it. FAIL with exactly two
       failures: "a path outside the allowed set: kernel/src/lib.rs" and "Cargo.lock:
       rust-decimal-scratch 1.0.0 added — source = registry+…" (the systems/ file and economy's changed
       list are admitted, as they should be). Working tree restored, both branches deleted (`git branch
       -D`), never pushed: `git ls-remote --heads origin | grep -c scratch` → 0; `git grep MUTATION --
       tests tools kernel systems Cargo.lock` empty; `git status` clean.
E-P3 P-C3: `cargo test -p mineworld-acceptance --test ac1_composability` → 10 passed, 0.30 s (`cargo
     metadata --no-deps --offline` runs inside `cargo test`). Positive (P-3): 24 workspace members; the
     six packs' direct dependents are mineworld-installed-systems and sibling market packs only (item:
     consumption, economy, employment, installed-systems, inventory, item-transfer; employment:
     economy, installed-systems; …); no normal/build path from the seven framework crates; no code file
     outside systems/, worlds/, tests/acceptance/ names a market crate.
     Mutations, each in the working tree, run against the built test binary, then `git restore`d:
     M-P5 (`mineworld-economy = { path = "../../systems/economy" }` in cognition/rule-controller's
       [dependencies]) → FAIL, three named: "mineworld-rule-controller (cognition/rule-controller/)
       depends on mineworld-economy (Normal): only systems/ may"; "a linked path to a market pack:
       mineworld-rule-controller → mineworld-economy"; "cognition/rule-controller/Cargo.toml:25 names
       mineworld-economy".
     M-P6 (`mineworld-installed-systems = { workspace = true }` in server's [dependencies]) → FAIL,
       six paths "mineworld-server → mineworld-installed-systems → mineworld-<pack>", one per pack.
     M-P7 (`// mineworld_economy` appended to tools/cli/src/run.rs) → FAIL "tools/cli/src/run.rs:517
       names mineworld_economy".
     Probe: an untracked tools/cli/tests/probe_untracked.rs naming mineworld-item → FAIL
       "tools/cli/tests/probe_untracked.rs:1 names mineworld-item"; removed.
     After: `git status` shows only the uncommitted test file; `git grep -n 'MUTATION\|mineworld_economy'
     -- tools/cli/src server cognition` empty.
E-P4 P-C4: `cargo test -p mineworld-acceptance --test ac1_composability` → 13 passed, 0.31 s; the
     whole crate: precursor_vocabulary 4, complete_affordances 4, ac1_composability 13, 0 failed.
     Positive (P-4): both packs read by WorldPack::read; world.yaml: id/name differ, systems = Social
     Café's seven then the six market packs, items and organizations in Market Town only, every other
     key equal; places/ and people/: the same files, every Social Café key equal; the keys Market Town
     adds are {people: economy, holdings, job} (places: none), each owned by a market pack by the
     build's catalog; items/ (item, tags) and organizations/ (holdings, economy, tags, note) only in
     Market Town. `git diff Cargo.lock`: +"mineworld-worldpack", +"serde-saphyr" in
     mineworld-acceptance's dependencies, nothing else.
     M-P13 (bob.yaml `- regular` → `- newcomer`) → FAIL "people/bob.yaml: `tags` differs from Social
       Café's". M-P14 (carol.yaml routine "07:00" → "07:30") → FAIL "people/carol.yaml: `routine`
       differs from Social Café's". Each run against the built test binary and `git restore`d; `git grep
       -n MUTATION -- worlds tests tools` empty.
E-P5 P-C5 (42c22ea): `cargo test -p mineworld-cli --test market_town` → PASS, 44.3 s (five runs in
     parallel, 35.6 s). 300-day seed 7, per bucket: purchases 349, 256, 276, 265, 250, 271, 247, 264,
     258, 274; wages paid alice 30 every bucket, felix 28–30; items-produced 194–203; items-consumed
     251–358; every seat gave 77–166 per bucket; zero wage-unpaid; no wallet below 100; nobody above
     six. Snapshot at revision 289 024 (head 289 045): 14 wallets equal to the replay, total 2 360 000
     = genesis; holdings equal. Purchase and wage counts equal E-6 run 3's, independently. The four
     30-day saves pass the same conditions over one bucket (control 349 purchases; seed 8 315); then
     control = twin byte for byte and in print; the run killed after day 15 resumed at its on-disk
     head and equals the control; seed 8 first differs at fact #144.
     M-P8 (consumption removed from market-town's systems) → FAIL, 128 conditions, first "bucket 0: no
       items-consumed", "bucket 1 (days 31-60): no purchase", …, wage-unpaid, "cafe-company's wallet
       fell to 30 … (day 43)". (First run, before DP-3: failed on the wallet alone.)
     M-P9 (economy's money-transferred reduction does not debit the payer) → bucket conditions pass,
       then FAIL on the snapshot: "a wallet economy wrote differs from the replay … #7: stored 255500,
       replayed 31050; …"; stored total 3 587 610 vs genesis 2 360 000.
     M-P10 (buy offered with Offer::new, incomplete) → FAIL, 30 conditions, first "bucket 0 (days
       1-30): no purchase".
     Each reverted (`git restore`); `git grep MUTATION -- systems worlds tools tests` empty. market_town
     runs used: 5 of 6 (positive, M-P8 twice, M-P9, M-P10); the sixth is the final gate.
E-P6 P-C6: `cargo test -p mineworld-cli --test market_composition` → PASS, 4.5–5.3 s. 30 days, seed 7:
       without item           refused: "system 'inventory' depends on 'item', which is not installed"
       without inventory      refused: "system 'item-transfer' depends on 'inventory', which is not
                              installed"
       without item-transfer  faults 0, every seat active; give requested 0, gives 0; purchases 382,
                              wages paid 58, items-produced 197, eat+drink 397
       without economy        buy requested 0; funded, shop-opened, money-transferred, wage-unpaid 0;
                              wage-due 60 (nobody pays); items-produced 203; gives 1 120; meals 14
       without employment     hired, shift-*, wage-due, items-produced 0; purchases 108; gives 1 256;
                              meals 121
       without consumption    eat+drink 0, items-consumed 0; purchases 42; wages paid 57; produced
                              201; gives 253
     M-P11 (test code, reverted): the item-transfer copy and the item copy given back Market Town's
       world.yaml → FAIL "without item-transfer: a give requested occurred 1382 time(s)". The item
       copy first kept item but not the `item:` sections: still refused, by inventory's stocked
       reduction (PreconditionFailed), so the name check failed instead — "the refusal does not name
       item". With items/ restored too: FAIL "without item: validate was expected to refuse the world,
       and it accepted it". `git grep MUTATION -- tools` empty.
E-P7 P-C7: `cargo test -p mineworld-cli --test milestone_c` → PASS, 1.0 s (after the 2-day run).
     Located in the 2-day seed-7 save: #57 hired (t0); #443 shift-started (t19 800, day 1, present
       false — DP-5); #580 person-entered-place alice → café (t26 100); #928 shift-ended worked > 0
       (t50 400); #929 wage-due; #936 money-transferred to alice caused by #929.
     assert_quiet_in(worlds/market-town/people, head, 3 600) passed (head t171 910, 23:45:10).
     Hosted: alice and bob each walked from the apartments into the café in 9 accepted strides
       (apartments door → street → café door → café); alice's disclosed wallet 19 400 = the replayed
       balance; she carried fewer than six, so no meal first; one available complete buy submitted
       unchanged → accepted with 2 events (action 1899); her wallet 19 400 → 19 000, +1 of item #22
       (cake, listed 400); bob's listing: #22 in_stock 6 → 5; bob perceives alice and is disclosed
       neither her wallet nor her holdings; revision 1 965.
     SIGKILL (signal 9) and `server … --save` again: the same instance, both seats at revision 1 965,
       alice's wallet 19 000 and holdings and bob's listing unchanged. `inspect`: "AC-9 every cause
       resolves".
     M-P12a (restart without --save) → FAIL "the same world" (instance …ad282ed83c83… vs
       …91e721d83c77…). M-P12b (bob not walked in) → FAIL "no observation with bob perceiving the
       café's listing arrived within 20s". Reverted; `git grep MUTATION -- tools systems worlds` empty.
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

**Raised while detailing 11d (2026-10-07, on `c5dc51c`).**

```text
QS-27  [primary session; operator-visible — part of QS-10's answer] A person carries at most six items,
       all kinds together; organizations are unbounded (SD-19). The spike found that a person no seat
       names absorbs every item given to him (F-39), and without consumption nothing else drains it.
       Alternatives: (a) consent — the taker must accept, as a pending offer (a process, a second action;
       an unseated person never accepts, so it also works, at about twice the code); (b) give only to
       people a controller drives — impossible, a world does not know who is driven (INV-1); (c) content
       only — nothing in content stops a person receiving; (d) retune the controller — forbidden (I-9).
       Recommended: capacity 6 (measured, E-4 run 2), as inventory's rule.

QS-28  `items-produced` is not added in 11d (amends §4.4's medium scope, "born final"). It has no stater
       until employment exists, and a fact tested only by hand-built statements would be designed
       ahead of its consumer (CLAUDE.md §4 rule 11). 11e adds it to inventory; that edit is under
       systems/**, inside ARC-35's allowed set. Recommended: defer to 11e.

QS-29  None of the three packs is biographical. 18 000 gives in 300 days would bury a biography (the
       reasoning of ARC-28 point 4 for relationships), and owning a coffee is not an event in a life.
       Alternative: items-transferred biographical. Recommended: none.

QS-30  items-transferred is visible to its participants (giver and taker) only (F-46). A bystander
       learns of a give only by seeing it in a later observation of holdings, which strangers are not
       shown (INV-13). 11e decides separately whether a purchase is place-visible. Recommended: accept.

QS-31  Holdings is a sorted list of {item, count}, not a map keyed by ItemId (F-38). A representation
       detail forced by JSON; raised because the component's shape reaches clients. Recommended: accept.

QS-32  The `item:` section is `{ category: <slug> }`. Alternatives: an empty marker (`item: {}`), or a
       kind declared by the file's presence alone (impossible: a pack seeds only from its section).
       Category is the smallest real attribute (drink, food, goods) and costs nothing to ignore.
       Recommended: category.

QS-33  market-town's 11d content: social-cafe copied byte for byte; ~20 item kinds (MVP §3), each with
       tags and a category; 1–4 items on every person, Otto included (~30 in all); no organizations
       (11e). Recommended: accept.

QS-34  D-6's controller test lives in systems/item-transfer/tests with dev-dependencies on movement and
       rule-controller. tests/acceptance would be the usual home but is outside the range; the
       dependency points from the market pack to the controller, which ARC-35 check 2 allows.
       Recommended: accept.

QS-35  [OPERATOR-MATERIAL — S9 scope or CP-4, forward to 11e] Nothing consumes items in S9 (QS-10). In
       11d, gives conserve items, so the flow stays alive. In 11e, purchases move items from shops to
       people, and with a bound of six per person, buying stops once everyone is full; without the
       bound, people hoard. Either way, CP-4's "a purchase in every 30-day bucket" would fail by month
       N. 11e's design must close the loop, and the choice is the operator's:
       (a) economy also lets a shop buy items back (`sell`), so items and money both circulate — no
           new pack, the smallest change;
       (b) a small consumption pack (eat, drink) joins S9 — MVP §5's verbs, a sixth market pack, a
           scope change;
       (c) CP-4 relaxed for purchases — weakens the milestone.
       Recommended: (a), decided at 11e's detail, with (b) kept for the needs pack QS-10 already
       anticipates.

QS-36  ARC-37 records 11d's decisions (SD-16 … SD-20, the capacity and why), in DECISIONS.md, inside
       the range as Markdown under docs/. Recommended: accept.

QS-37  11d's per-seat give evidence is ledger evidence from a scratch reader on a scratch branch, never
       committed (F-45), exactly as 11c's E-C6; 11f turns it into market-town's world-level test.
       Recommended: accept.

QS-38  The execution contract for 11d (§15): fresh session, its own worktree on
       mvp0/pr-11d-owning-things, commits and push and PR authorized as for 11a–11c, a scratch branch
       for D-C6's measurements, merge the operator's with a merge commit. Recommended: confirm at
       freeze.
```

**Raised while detailing 11e (2026-10-07, on `70e532f`).**

```text
QS-39  [OPERATOR-MATERIAL — the interaction set, within QS-35's scope] The consumption pack is
       `consumption`, providing `eat { item }` for kinds of category `food` and `drink { item }` for
       `drink`; goods are never consumed (SD-25). It closes MVP §5's `eat` as an interaction and adds
       `drink`, which MVP §5 does not list: without it drinks fill hands (F-50). Hunger and `sleep` stay
       open (QS-10). Alternatives: (a) `eat` only, drinks not sold — the café without coffee; (b) one
       `consume` action — not MVP's word; (c) a pack-owned `consumable:` section on item files instead of
       reading item's category — more content and a genesis fact per kind for a rule the category
       already carries. Recommended: eat + drink by category.
QS-40  A person eats or drinks what they carry, anywhere (requirement none). Alternative: only at the
       café or a place with a shop — a rule tied to places no MVP criterion needs, and fewer meals for
       people whose day passes no shop. Recommended: anywhere.
QS-41  Production (QS-28 answered): `items-produced` comes in now, in inventory's vocabulary, stated by
       employment when a shift ends, for the employer, prorated by the worked share of the shift
       (`produces: { item: per full shift }`). Consumption destroys items, so without production the
       shops empty and CP-4's purchases end. Alternative: endow shops for 300 days and drop production —
       CP-4 asks for an item produced in every bucket. Recommended: as designed.
QS-42  11e edits a merged pack: inventory gains `items-produced`, `items-consumed`, `produce`,
       `consume` and their admit functions. Under systems/, so inside ARC-35's range; inventory stays the
       only writer of holdings, and the facts name what happened to holdings, not why. Alternative:
       facts in the stating packs' vocabularies reduced by inventory via ARC-28 — inventory would then
       reduce other packs' vocabulary, against ARC-26 point 1. Recommended: accept.
QS-43  [operator-visible — authoring format of a new pack; amends §4.5's medium scope] A pack owns one
       section (F-47), so economy's is `economy:` on person and organization files — `{ wallet }`, and on
       an organization `shop: { at: <place>, prices }`. The medium scope's `shop:` section on place files
       is not possible without a framework change. Side effect: no place file of market-town changes.
       Alternative: a precursor letting a pack own several sections (framework work justified only by the
       market — I-2 forbids). Recommended: accept.
QS-44  `buy` is target-less, `at_place(shop)` + target available; out of stock, cannot pay and cannot carry
       all read TargetUnavailable, in the affordance and at dispatch (F-48). Alternative: offer only the
       buys that would succeed — a client could not show what is for sale but unaffordable. Recommended:
       accept; finer reasons wait for a contract that lets an offer carry one.
QS-45  [OPERATOR-MATERIAL — an INV-13 reading] Economy discloses, to everyone perceiving a shop's
       place, the listing: operator, prices, and how many of each kind the operator holds — a count read
       from inventory's Holdings, which inventory itself discloses to the holder only. It is economy's
       judgement that a shop's shelf is visible to whoever is in the shop, and it is how a second client
       perceives a purchase (CP-7, F-54). A purchase's `money-transferred` is `Place(shop)`-visible
       (answers QS-30); `items-transferred` stays participants-only (inventory's choice). Alternatives:
       prices only (CP-7 then sees nothing change until a kind runs out); a stock component economy keeps
       itself — a second truth about holdings. Recommended: the listing with counts.
QS-46  Two jobs, read literally from MVP §3 (alice at the café 05:30–14:00, felix at the store 08:00–13:00,
       each inside a routine they already have, F-17). Alternative: "2 Jobs" as two roles held by
       several people, so most seats earn — more circulation, more content, and a reading of MVP §3 the
       operator has not made. Recommended: two job holders.
QS-47  [OPERATOR-MATERIAL — how CP-4's "no wallet or employer drained" is read] Read as: zero `wage-unpaid`
       in 300 days, and no wallet — person or organization — ever below the cheapest price in the town;
       money conserved. With two jobs, ten people have no income and live on an endowment sized for 300
       days (F-52); over years they would drain, which §1.2 already places outside S9 (R-S9-3). The spike
       shows the instrument sees a drain (F-51 run 2) and that content alone closes it (run 3), the
       controller untouched (I-9). Alternatives: more job holders (QS-46); a basic income — a rule no MVP
       criterion needs. Recommended: as read, recorded as an ARC-38 limitation.
QS-48  Shops sell consumables only (food and drink); goods keep circulating by give (F-50). Recommended:
       accept.
QS-49  Biographical: employment's `hired` only — a life event, once per job. Shifts, wages, purchases and
       meals are thousands of facts that would bury a biography (QS-29's reasoning). Recommended: accept.
QS-50  Employment reuses schedule's `TimeOfDay` (a Cargo dependency on its type, no system dependency) so
       the town has one time-of-day convention (ARC-32); a shift lies within one day (`from < until`).
       Recommended: accept.
QS-51  11d's market-town evidence is superseded as current evidence, not withdrawn (§4.5.6); per-seat
       gives are reported, not required. Recommended: accept.
QS-52  ARC-38 records 11e; an ARC-35 dated note says the transformation carries six market packs (the
       sixth by QS-35), so its checks read "the six market packs". The AC-1 test that reads the list is
       11f's. Recommended: accept.
QS-53  The execution contract for 11e (§16): fresh session, its own worktree on
       mvp0/pr-11e-work-money-shops, commits, push and PR authorized as for 11a–11d, a scratch branch for
       E-C7's measurements and M-E9, merge the operator's with a merge commit. Recommended: confirm at
       freeze.
```

**Raised while detailing 11f (2026-10-07, on `2dddda8`).**

```text
QS-54  [OPERATOR-MATERIAL — how ARC-35 check 2 is read] "No dependency path leads to a market pack from
       kernel, contracts, persistence, server, authoring, sdk or rule-controller." persistence's
       kill_and_resume test dev-depends on worldpack, which links the installed set and so every market
       pack (F-58). Counted over every edge kind, check 2 fails AC-1 for a test that loads a World Pack,
       though nothing in persistence knows the market.
       Proposal: bullet 2 follows normal and build edges — what a library or a binary links. Dev edges are
       still seen by bullet 1 (any declared dependency on a market pack, of any kind, must come from
       systems/) and bullet 3 (no code file outside the three directories names a market crate).
       Alternatives: (b) every edge, but a path through mineworld-installed-systems is admitted as
       ARC-33's composition — then a kernel that linked worldpack in production would pass, which (a)
       refuses; (c) every edge, and persistence's checkpoint stops loading the repository's World Pack —
       an edit to an existing test outside the proof's scope, to satisfy a measurement.
       Recommended: (a), recorded in the ARC-35 note (11f).
QS-55  The two transformation merges are found on `git log --first-parent --merges HEAD` by their GitHub
       subject (`Merge pull request #43 from <owner>/mvp0/pr-11d-owning-things`, #46 for 11e), exactly
       one match each, with two parents, and each must equal the id recorded in the test (70e532f…,
       2dddda8…). ARC-35's limitation says the proof records the ids; the subject proves the recorded id is
       that PR's merge on main's first-parent chain, so a mistyped id cannot point the check at an
       innocuous merge (M-P3, M-P4). Recommended: accept (bounded).
QS-56  The Cargo.lock rule as R-S9-6 words it: every package added, removed, or changed in any field
       between M^1 and M has no source and is a crate under systems/ at M. ARC-35 item 2 names "added"
       and "dependency list changed"; this also refuses a version or checksum change of an external
       package (a `cargo update` inside the range). Tightens, loosens nothing; both merges pass it
       (E-8). Recommended: accept.
QS-57  Check 3 compares the authored YAML structurally (serde-saphyr into serde_json::Value) after both
       packs pass WorldPack::read; a section Market Town adds must be owned, by the build's
       Capability::owning_section, by one of the six market packs; world.id/name may differ; the six
       packs follow Social Café's list as a set; README.md is not configuration. tests/acceptance gains
       two dev-dependencies, worldpack and serde-saphyr, both existing workspace entries (F-62, F-69).
       Alternative: byte-prefix of each file (market-town's file begins with social-cafe's) — stricter
       about layout and comments, blind to what the appended text is. Recommended: structural.
QS-58  The world-level tests in tools/cli read market facts and components by their type slugs into
       test-local mirrors (deny_unknown_fields, schema version 1 asserted), never through a market crate:
       ARC-35 check 2 bullet 3 forbids naming one outside systems/, worlds/ and tests/acceptance/, and a
       test that runs the binary must live in tools/cli (F-61). Literal slugs are also the independent
       oracle rules §25 asks for. Alternatives: amend check 2 to admit tools/cli/tests (weakens AC-1's
       instrument for convenience); a test-only command that prints market state (a CLI change in the
       proof). Recommended: slugs and mirrors.
QS-59  CP-4's committed horizon is the full 300 days, in the default suite, not #[ignore]d: CP-4 and L-13
       are 300-day claims, run.rs already runs three 300-day runs by default, and the market run costs
       ~34 s of a ~3-minute gate, in parallel with the 30-day runs. An ignored test is a test not run.
       Alternatives: (b) 90 days committed (sees M-E9 at bucket 1 and F-51's drain at bucket 2) with the
       300-day run as a named manual gate — cheaper, but content that drains after day 90 passes the
       committed suite; (c) 300 days #[ignore]d with a named gate — runs only when someone remembers.
       Recommended: (a), 300 days default-on.
QS-60  The committed CP-4 conditions are E-9 b's plus 11d's give conditions: every seat gave in every
       bucket, and nobody ever holds more than six. QS-51 made per-seat gives "reported, not required"
       for 11e's ledger evidence; the operator's 11f brief asks for 11d's give conditions in the test,
       and E-E7 shows them holding with margin (fewest 77 per seat-bucket). Money conservation and the
       capacity are checked against the save's snapshot state as well as the replayed facts, because
       facts alone conserve money by construction. Recommended: accept (primary session).
QS-61  AC-2 at world level for all six market packs, not item-transfer alone (CP-6 widened): a pack
       removed with its section either runs (item-transfer, economy, employment, consumption — each with
       its interactions absent and the others present) or is refused by validate naming the dependency
       (item: inventory needs it; inventory: four packs need it), as the declared dependencies say.
       Recommended: accept.
QS-62  Milestone C's shape: a 2-day seed-7 market-town run with --save (alice works at the café and is
       paid — located in the save); the server hosts the save; alice and bob walk into the café through
       the street; alice submits an available complete `buy` unchanged (after an offered eat or drink
       if she carries six, F-66); bob perceives the café's in_stock fall by one and sees neither alice's
       wallet nor her holdings; SIGKILL and restart: same instance and revision, alice's wallet and
       holdings and bob's listing unchanged; inspect resolves every cause.
       Alternatives: (a) the store, with felix — it sells out every day (F-65), so at the head a buy is
       likely unavailable; (b) hosting a fresh world and working through the protocol — a shift is hours
       of wall time at one world second per wall second. Recommended: the café after a 2-day run.
       Operator-visible: it is the milestone the operator reviews (HUMAN_REVIEW_QUEUE).
QS-63  The quiet-window check for a hosted Market Town reads `from:` and `until:` of every person file in
       worlds/market-town, so a job's boundary counts as a routine's does (F-64); a new function beside
       fixture::assert_quiet, whose body and callers are unchanged. Recommended: accept.
QS-64  11f's documents: an ARC-35 note (11f) rather than a new ARC — the proof applies ARC-35, it decides
       nothing new beyond the readings QS-54 … QS-57 settle; MVP_STATUS, HUMAN_REVIEW_QUEUE (Milestone C
       with launch commands), worlds/market-town/README. Overall §7 and the step header stay the planning
       session's. Recommended: accept.
QS-65  [OPERATOR-MATERIAL — declaring the primary criterion met] When 11f merges with the AC-1 test green
       on main and its mutations recorded, MVP_STATUS marks the Market Town composition ✅ and the S9
       closeout records AC-1 as demonstrated — as ARC-35 measures it and within ARC-33's static-linking
       boundary (installing = a directory, two lines in systems/installed, a rebuild). ✅ there means
       "actually run and inspected", which the agent can establish; whether that is the criterion the
       project set itself is the operator's to accept. Milestone C is marked "demonstrated, awaiting the
       operator's review", like B. Recommended: so, with the operator's review of Milestone C and of the
       AC-1 claim at 11f's merge.
QS-66  The execution contract for 11f (§17): fresh session, its own worktree on mvp0/pr-11f-proof, commits,
       push and PR authorized as for 11a–11e, a local scratch branch for M-P1 and M-P2 and working-tree
       mutations for the rest, merge the operator's with a merge commit. Recommended: confirm at freeze.
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

# 15. Execution contract for PR 11d (proposed; confirmed at 11d's freeze)

```text
PROJECT / PR        MVP-0 · Step 10 / PR 11d — owning and giving things (S9, fourth of six; the first
                    half of the measured AC-1 transformation)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-10-market.md §4.4 (4.4.1–4.4.5); evidence in a
                    new §9.4 (E-D<n>)
RELATED / BINDING   overall.md §§1, 7; this file §§1.3 (I-1, I-3, I-4, I-6, I-7, I-9), 2.5, 2.6, 3
                    (SD-13), 8.5, 9 E-3/E-4, 10 (QS-7, QS-10, QS-27 … QS-38, as answered);
                    DECISIONS ARC-26, ARC-28, ARC-31, ARC-33, ARC-34, ARC-35, ARC-36; MODULE_SPEC §§3.1,
                    4.1; CORE_CONCEPTS §§6.3, 7, 8, 13.1, 15.2
IMPLEMENTATION BASE the main named at freeze (main @ c5dc51c + the docs-only planning merges); branch
                    mvp0/pr-11d-owning-things; worktree /Users/yuema137/mineworld-worktrees/s9-11d
                    (proposed), held by the implementing session only
APPROVED SCOPE      §4.4: D-C1 … D-C7; only the paths of §4.4.1's table
FROZEN INVARIANTS   I-1 (any path outside §4.4.1's table is a material stop), I-3, I-4 (sha
                    ad49c723…c64b), I-6, I-7 (D-9 b before c), I-8, I-9 (no controller change; a quiet
                    market is fixed in packs or content); I-2's scan unchanged (no row, no entry)
SEQUENCE            D-C1 → D-C2 → D-C3 → D-C4 → D-C5 → D-C6 → D-C7, each committed and pushed when
                    coherent
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: 300-day social-cafe (~15 s) twice,
                    300-day market-town with save (~35 s) at most three times (D-9, M-D7), 30-day runs;
                    one full workspace gate on the final head (background); about one hour in total;
                    real-model: NOT REQUIRED
LIVE DOCUMENTATION  §4.4 checkboxes; §9.4 E-D ledger
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 11d at D-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the primary session's freeze message
  semantic commits, branch push       recommended authorized, as for 11a–11c
  PR creation / update                recommended authorized, as for 11a–11c
  scratch branch (D-C6, M-D7)         recommended authorized, local only, deleted after evidence
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns the step header, §§1–3, §5, §10, overall and MVP_STATUS's
                    Updated/S9 lines; the implementing session owns §4.4 and §9.4
NORMAL STOP         PR 11d READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed edit outside §4.4.1's paths (a framework gap: report it and propose a
                    precursor justified without the market, I-2); a changed social-cafe run or an
                    edited existing test; a Cargo.lock change other than three path packages and the
                    installed set's list; a need to change the controller or its constants (I-9); D-9 b
                    failing with the capacity (the market cannot be kept alive by packs or content)
```

# 16. Execution contract for PR 11e (proposed; confirmed at 11e's freeze)

```text
PROJECT / PR        MVP-0 · Step 10 / PR 11e — work, money, shops and consumption (S9, fifth of six; the
                    second half of the measured AC-1 transformation)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-10-market.md §4.5 (4.5.1–4.5.6); evidence in a
                    new §9.5 (E-E<n>); deviations in a new §4.5.7
RELATED / BINDING   overall.md §§1, 7; this file §§1.3 (I-1, I-3, I-4, I-6, I-7, I-8, I-9), 2.5, 2.6, 3
                    (SD-13), 4.4.0 (QS-35), 8.6, 9 E-6, 10 (QS-7 … QS-11, QS-39 … QS-53, as answered);
                    DECISIONS ARC-23, ARC-26, ARC-28, ARC-31 … ARC-37; MODULE_SPEC §§3.1, 4.1;
                    CORE_CONCEPTS §§7, 8, 10, 13.1, 15.2; MVP §§3, 5
IMPLEMENTATION BASE the main named at freeze (main @ 70e532f + the docs-only merges #44 and this planning
                    branch); branch mvp0/pr-11e-work-money-shops; worktree
                    /Users/yuema137/mineworld-worktrees/s9-11e (proposed), held by the implementing
                    session only
APPROVED SCOPE      §4.5: E-C1 … E-C8; only the paths of §4.5.1's table
FROZEN INVARIANTS   I-1 (any path outside §4.5.1's table is a material stop), I-3 (only economy moves
                    money; employment never touches a Wallet; only inventory writes Holdings; consumption
                    removes only through `consume`), I-4 (sha ad49c723…c64b), I-6 (integer minor units),
                    I-7 (E-9 b before c before d), I-8, I-9 (no controller change; a quiet or draining
                    market is fixed in packs or content); I-2's scan unchanged (no row, no entry)
SEQUENCE            E-C1 → E-C2 → E-C3 → E-C4 → E-C5 → E-C6 → E-C7 → E-C8, each committed and pushed when
                    coherent (employment before economy: economy's crate names employment's WageDue)
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: 300-day social-cafe (~15 s) twice;
                    300-day market-town with save (~35 s) at most four times (E-9, re-sizing, M-E9); 30-day
                    runs; one full workspace gate on the final head (background); about one hour in total;
                    real-model: NOT REQUIRED
LIVE DOCUMENTATION  §4.5 checkboxes; §9.5 E-E ledger; §4.5.7 deviations
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 11e at E-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the primary session's freeze message
  semantic commits, branch push       recommended authorized, as for 11a–11d
  PR creation / update                recommended authorized, as for 11a–11d
  scratch branch (E-C7, M-E9)         recommended authorized, local only, deleted after evidence
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns the step header, §§1–3, §5, §10, overall and MVP_STATUS's
                    Updated/S9/artefact lines; the implementing session owns §4.5 and §9.5
NORMAL STOP         PR 11e READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed edit outside §4.5.1's paths (a framework gap: report it and propose a precursor
                    justified without the market, I-2); a changed social-cafe run or an edited existing
                    test outside inventory's (or one there whose claim changes); a Cargo.lock change other
                    than three path packages and their lists; a need to change the controller or its
                    constants (I-9); E-9 b failing after content re-sizing (the loop cannot be kept alive by
                    packs or content); an answer to QS-39, QS-43, QS-45 or QS-47 other than the design's
```

# 17. Execution contract for PR 11f (proposed; confirmed at 11f's freeze)

```text
PROJECT / PR        MVP-0 · Step 10 / PR 11f — the proof (S9, sixth and last; outside the AC-1 range)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-10-market.md §4.6 (4.6.1–4.6.6); evidence in a
                    new §9.6 (E-P<n>); deviations in a new §4.6.8
RELATED / BINDING   overall.md §§1, 4, 7; this file §§1 (CP-1, CP-4, CP-6, CP-7), 1.3 (I-1, I-3, I-4, I-7,
                    I-8, I-9), 2.5, 2.6, 3 (SD-6, SD-15), 4.4.3 D-9, 4.5.3 E-9, 4.5.7, 8.7, 9 E-8, 10
                    (QS-11, QS-12, QS-37, QS-51, QS-54 … QS-66, as answered); DECISIONS ARC-23, ARC-25,
                    ARC-33, ARC-35 (with its notes), ARC-37, ARC-38; MVP §§2, 9 (AC-1, AC-2, AC-6, AC-11,
                    AC-12); HUMAN_REVIEW_QUEUE (Milestones B, C); server/PROTOCOL.md §§5, 6
IMPLEMENTATION BASE the main named at freeze (main @ 2dddda8 + the docs-only merges #47 and this planning
                    branch); branch mvp0/pr-11f-proof; worktree /Users/yuema137/mineworld-worktrees/s9-11f
                    (proposed), held by the implementing session only
APPROVED SCOPE      §4.6: P-C1 … P-C8; only the paths of §4.6.1's table
FROZEN INVARIANTS   no behaviour change: no edit under systems/, worlds/ (but market-town's README.md),
                    kernel/, contracts/, persistence/, server/, cognition/, sdk/, authoring/, worldpack/,
                    tools/cli/src/, clients/, nor the root Cargo.toml; I-4 (social-cafe sha ad49c723…c64b;
                    the 300-day market-town summary but `wall` = the base's, 372 755 facts); I-7 (activity
                    before any comparison, in every test that compares runs); I-9 (a failing proof is never
                    answered by a controller, pack or content change); no existing test edited (fixture/
                    mod.rs gains one function); no market crate named outside systems/, worlds/,
                    tests/acceptance/ (ARC-35 check 2, which this PR's own test enforces); the I-2 scan
                    unchanged (no row, no entry)
SEQUENCE            P-C1 → P-C2 → P-C3 → P-C4 → P-C5 → P-C6 → P-C7 → P-C8, each committed and pushed when
                    coherent; P-C5 … P-C7 may run in any order after P-C1
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: the market_town test (~40 s) at most
                    six times (P-5/P-6, M-P8 … M-P10, one re-run); market_composition and milestone_c freely
                    (each under a minute); the 300-day social-cafe comparison (~13 s) twice; one full
                    workspace gate on the final head (background, ~4 min); about one hour in total;
                    real-model: NOT REQUIRED
LIVE DOCUMENTATION  §4.6 checkboxes; §9.6 E-P ledger; §4.6.8 deviations
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 11f at P-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the primary session's freeze message
  semantic commits, branch push       recommended authorized, as for 11a–11e
  PR creation / update                recommended authorized, as for 11a–11e
  scratch branch (M-P1, M-P2)         recommended authorized, local only, never pushed, deleted after
                                      evidence; every other mutation in the working tree, reverted
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns the step header, §§1–3, §5, §10, overall and MVP_STATUS's
                    Updated/S9 lines and the S9 closeout (§4.6.6); the implementing session owns §4.6,
                    §9.6 and P-11's status rows (MVP_STATUS evidence and axis rows, HUMAN_REVIEW_QUEUE)
NORMAL STOP         PR 11f READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed edit to any pack, world (beyond its README) or framework crate — the proof
                    needs a behaviour change; a check that fails on the merged history and is not a defect
                    of the test (AC-1 does not hold as ARC-35 measures it); CP-4, AC-2 or Milestone C
                    failing for a reason no test defect explains; an answer to QS-54 or QS-65 other than
                    the design's; a need to name a market crate outside the three directories
```
