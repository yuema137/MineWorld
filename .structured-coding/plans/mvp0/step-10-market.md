# Step 10 / PR 11 — Market Town and the AC-1 composability proof (S9)

**Role:** step document for S9, proposing a split into six PRs (§2.8). It also holds the full PR
design for the first of them, **PR 11a**, and a proposed execution contract for it (§11). PRs 11b–11f
are specified at medium scope and are each re-audited and detailed to the commit only after the PR
before them merges (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 S9, §4 (`AC-1`, `AC-2`), §7 (F-1, F-3)
**Lifecycle:** `DRAFT — awaiting the primary session's review`. Nothing here is frozen. No
implementation is authorized by this document.

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
  docs                         DECISIONS (DEP-12, ARC-33), MODULE_SPEC §3.1, systems/README, sdk README

PR 11b  items and organizations in the World Pack format (MODULE_SPEC §4's frozen model, F-4)
  authoring/                   ContentKind gains Item and Organization
  worldpack/                   world.yaml `items:` and `organizations:`; `items/<key>.yaml` and
                               `organizations/<key>.yaml` carrying tags, note and sections; entity ids
                               allocated after people, so every existing id stays put
  docs                         MODULE_SPEC §4.1, PACKAGE_FORMAT §8

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
