# Step 18 — The World's Interaction List: interactions between any entities, as content and pluggable code

**Role:** the step design for one concept that replaces two: **S17 — Physics list** and **S18 —
Configurable rules** (`overall.md`, "Physics list and configurable rules (operator, 2026-10-08)"). It
records the operator's broadened requirement, an audit of every pack's hard-coded rules, a reuse
comparison, the unified design, a PR split replacing PL-a … PL-d and S18, invariants, risks and
questions (QIL-1 …). It holds no frozen PR design.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md); siblings: [`step-11-bodies.md`](step-11-bodies.md)
(S15), [`step-16-packages.md`](step-16-packages.md) (S16), [`step-17-cognition.md`](step-17-cognition.md)
(S10), [`step-12-server.md`](step-12-server.md) (S11-C perception).
**Lifecycle:** `DRAFT — awaiting the primary session's review`. Nothing here authorizes implementation.
Edits to `overall.md`, `docs/DECISIONS.md` and the specifications are proposed in §12 and not applied.
**Authority.** This document is the one authority for the merged step.
[`step-18-physics-list.md`](step-18-physics-list.md) (S17, frozen at step level) is **superseded** by it.
Its header says so. The following parts are incorporated by reference **as the design of the `bodies`
section**, unchanged except where §4.9 here says otherwise:
- §2.1 (audit of bodies' constants and rules);
- §3 (the physics reuse comparison);
- §4.3–§4.5 (the physics document, parameters and engine constants, resolution);
- §4.7 (interaction kinds and consequence packs);
- §4.9 items 3–7 (determinism).

Every other part of that file is replaced here.
**Branch:** `plan/interaction-list` from `main @ 21f96ff`, worktree
`/Users/yuema137/mineworld-worktrees/plan-physics-list`, one writer.
**Source audited:** `main @ 21f96ff` (12c merged: `systems/bodies` VERSION 2).
**Decision ids:** the numbers already assigned to S17 and S18 are reused (§12): ARC-61 (configuration
seam), ARC-62 (extension catalogs), ARC-63 (the Interaction List), ARC-64 (entity classes), ARC-65
(consequence routing), DEP-28 (build our own, on Cedar's semantics). The primary session confirms them.

## Why a new file

The concept changed: the old file's title, vocabulary ("body class") and scope (bodies only) would be
wrong in place. A successor file plus a one-line "superseded" header in the old one keeps the frozen
record readable and leaves exactly one authority.

---

# 1. The requirement

## 1.1 Verbatim (operator, 2026-10-08, second statement)

> 我们这里说的physics list，并非只是物理规律，而是不同的object之间的相互作用，比如，人物是否可以对话，对话内容是否进入npc的biography和history，物品是否可以交易（改变人物归属）等等，是广义的虚拟object之间的相互作用

```text
R-IL-1  The "physics list" is not only physical law: it is the interactions between virtual objects in
        general — people, items, places, organizations.
R-IL-2  Examples: whether persons can talk; whether what is said enters an NPC's biography and history;
        whether items can be traded (ownership change).
```

The first statement (R-PL-1 … R-PL-4 in the superseded file: modular, a framework, a high-level
abstraction layer, user-added lists) still binds.

## 1.2 Operator decisions that bind this design

From `overall.md` (2026-10-08), kept unchanged:
- everything here is in MVP-0, including the ice/fragile demonstration (QPL-13);
- a list is data, never scripts; new behaviour comes from installed packs (QPL-1);
- the generic `configure:` seam, typed by its owning pack (QPL-2, ARC-61);
- resume is refused on configuration drift (QPL-12);
- no runtime entity creation by a System (QPL-11).

Primary-session rulings kept: the timing of QPL-4 (the seam beside 12d after S16 E-a; bodies after
12d); QPL-10 (presence's `resolution:` line migrates to the generic extension form in the seam PR).

## 1.3 The answer in brief

```text
Classes        Every entity may have a class: a name the World's Interaction List gives to a tag
               selector (`noble` = a person tagged `noble`). Tags already exist on every entity, are
               authored in content, are disclosed, and are "a taxonomy, not state" (contracts/src/entity.rs).
               So classes add no fact and no component. Unclassed worlds are byte-identical.

One list       The World's Interaction List is one document in sections. `classes` is shared. Each other
               section is owned and typed by one System Pack. The SDK provides the schema: rules (which
               actor class may do which action to which target and object class), parameters (ranges,
               capacities, amounts, typed by the pack, bounded by the pack), and consequences (each fact's
               audience, narrowing only, and whether it is biographical). The pack provides the meaning and
               enforces it. Bodies' physics is one section; every pack gets one.

Routing        A list may narrow the Visibility an owner gives a fact, within bounds the owner declares,
               never widen it. It may turn a fact's biographical-ness on or off where the owner allows. The
               fact log always records (INV-11). Who perceives a fact — and so what an NPC can remember —
               follows the envelope's Visibility through S11-C's unchanged audience function.

Enforcement    In the owning pack, through the SDK's pure lookups, on top of the pack's own validation: a
               list can restrict and parameterize, never grant what the pack does not implement. Denials
               are `PermissionDenied`, through the existing affordance mechanism. The kernel is unchanged.

Composition    pack bounds < pack reference list < a reference list the world extends < the world's section
               < a region (place) override. Within one level, the more specific selector wins, and on a
               tie deny wins (Cedar's forbid-overrides-permit). The resolved list is stored in the save.
               Drift is refused at resume.

Proof          A demonstration world shows the operator's three examples by configuration alone, with no
               code edit: two classes of people cannot talk; one class's lines enter no biography; one
               class of item cannot be traded. The ice/fragile physics demonstration stays.
```

---

# 2. Audit — what every pack hard-codes today (`main @ 21f96ff`)

## 2.1 What already exists that the design builds on

- **A-1 Tags.** `contracts/src/entity.rs:18–26`: "Tags are a taxonomy, not state: a system reads them to
  decide whether an entity is of interest to it." `Tags` is a `BTreeSet<Tag>` (fixed order) on every
  entity record, authored in every content file (`tags:`), read through `WorldRead::entity(..).tags()`,
  and disclosed to clients in observations (`systems/presence/src/observe.rs:130, :139`). Item kinds
  carry them (`worlds/market-town/items/apple.yaml`: `tags: [food, fruit]`). They are immutable after
  genesis.
- **A-2 Item kinds and categories (ARC-36).** An item file is a kind; holdings are counts of kinds; a
  kind has a `category` in item's own section, which consumption reads (`EATEN = "food"`,
  `DRUNK = "drink"`, `systems/consumption/src/action.rs:10, :13`). A loose object (bodies) is one Item
  entity whose file carries `body:` (12c).
- **A-3 Visibility is chosen in code per emission** (`contracts/src/event.rs:296`: `Public`,
  `Place(p)`, `Participants`, `Entities(S)`, `SystemInternal`). Counted on `main`: bodies 5, presence 3,
  conversation 2, economy 4, employment 4, group-activity 3, inventory 3, item 1, movement 1, naming 1,
  relationships 1, schedule 1.
- **A-4 Biographical-ness is a compile-time constant per pack** (ARC-29): `pub const BIOGRAPHICAL`,
  aggregated by `Capability::biographical`. Declared on `main`: group-activity (4 facts), relationships
  (`became-acquainted`, `relationship-changed`), employment (`hired`), schedule (`agenda-changed`).
  conversation, presence, movement, naming, economy, inventory, item, consumption, item-transfer and
  bodies declare none. So **`spoke` is not biographical today.**
- **A-5 Perception of facts is presence's `audience` function** (S10 §3.3, owned by S11-C, coordination
  ruling 2). It reads only the envelope's Visibility and presence's own whereabouts fold. Cognition's
  memory ingests only perceived events (S10 §3.8.1). So memory follows Visibility, by construction.
- **A-6 A semantic denial already exists.** `Rejection::PermissionDenied` (`contracts/src/action.rs:221`)
  is one of the five rejections `CLAUDE.md` §4 rule 15 names. Offers carry availability and an
  unavailable reason (`Offer`, ARC-34), so a client already learns "not allowed" from the affordance.
- **A-7 The configuration seam and extension catalogs** are frozen as S17's ARC-61 and ARC-62 (not yet
  implemented).
- **A-8 Resume does not re-read content** (`persistence/src/world.rs:116–121`); ARC-61's drift check is
  the decided remedy for configuration.

## 2.2 Every pack, mapped to the schema

Columns: **Rules** are who may do what to whom. **Parameters** are the numbers. **Consequences** are each
fact's audience and biography. "—" means the pack has none of that kind. Files are under `systems/<pack>/src/`.

| Pack | Actions | Rules a list would govern | Parameters (today's constant → default) | Consequences (fact: today's Visibility; biographical today) |
| --- | --- | --- | --- | --- |
| **presence** | — | — (owner of where people are and of the audience function; not configurable in this step) | — | `arrived`, `person-entered-place`, `stopped-short`: `Place`; no. **Not opened** (QIL-9) |
| **movement** | `move` | `move` by actor class into a **place class** (e.g. `staff-only`), checked when a stride crosses a passage | `MAX_STRIDE` 2 000 mm (`action.rs:23`) | `passage-opened`: `Public`, genesis; no |
| **conversation** | `talk` | `talk`: actor class × target class | `INTERACTION_RANGE` 3 000 mm (`action.rs:16`); `CONVERSATION_GAP` 300 s (`system.rs:29`); `REMEMBERED_AT_MOST` 32 (`component.rs:18`); engine: `UTTERANCE_MAX_BYTES` 480 | `spoke`: `Place` (overhearing), narrowable to `Participants`; biographical: no today, **configurable**. `conversation-started`: `Participants`; configurable. Pack-specific: whether the listener's `Remembered` log keeps the line (`remember`) |
| **relationships** | — (reacts) | `acquaint`: whether a pair of classes forms a relationship at all | `SPOKE_FAMILIARITY` 10, `ACCEPTED_REGARD` 50, `DECLINED_REGARD` −30, `ACTIVITY_FAMILIARITY` 50, `ACTIVITY_REGARD` 20 (`system.rs:22–30`), per class pair; engine: ranges (`component.rs:11–12`) | `became-acquainted`, `relationship-changed`: `Participants`; yes, configurable |
| **group-activity** | `invite`, `accept-invitation`, `decline-invitation`, `join-group-activity`, `leave-group-activity` | `invite`, `join`: actor class × target class | `INVITE_RANGE` 3 000 mm (`action.rs:21`), `INVITATION_LIFETIME` 1 800 s (`component.rs:23`), `ACTIVITY_LENGTH` 3 600 s (`process.rs:11`); engine: `KIND_MAX_BYTES` | its four facts: `Participants` or `Place`; yes, configurable |
| **item** | — | — | — | `item-kind-declared`: `Public`, genesis; no |
| **inventory** | — (constructors) | `hold`: holder class × **item class**: may this holder hold this kind at all (refusing a give, a buy, a production) | `PERSON_CAPACITY` 6 (`admit.rs:18`), per holder class | `stocked`, `items-transferred`, `items-produced`, `items-consumed`: `Entities` or `Participants`; no; narrowable only |
| **item-transfer** | `give` | `give`: actor class × target class × **item class** — the operator's "can this be traded" | `GIVE_RANGE` 3 000 mm (`action.rs:12`) | states inventory's `items-transferred` (inventory's section governs it) |
| **economy** | `buy` | `buy`: actor class × shop **place class** × item class | — (prices and wallets are content) | `funded`, `money-transferred`, `shop-opened`, `wage-unpaid`: `Public`, `Place`, `Participants`, `Entities`; no; narrowable only |
| **employment** | — (process) | — (jobs are content) | engine: `HOUR` | `hired` (yes), `shift-started`, `shift-ended`, `wage-due`: `Entities` or `Participants`; configurable biography |
| **consumption** | `eat`, `drink` | `eat`, `drink`: actor class × item class | `EATEN` "food", `DRUNK` "drink" → `eats: [categories]`, `drinks: [categories]` per actor class | states inventory's `items-consumed` |
| **naming** | — | — | — | `named`: `Public`; no. Gating names on acquaintance stays ARC-31's later decision |
| **schedule** | — | — | engine: `MAX_SEGMENTS` 24 | `agenda-changed`: `Participants`; yes, configurable |
| **bodies** | `kick`, `throw`, `shove` | the physics pair table (`step-18-physics-list.md` §4.3), with classes from tags (§4.9) | the 26 parameters of `step-18-physics-list.md` §4.4 | `object-moved`, `person-shoved`: `Place`; genesis facts `Public`/`Place`; no |

**Findings.**
- **F-IL-1** Three of the operator's examples are each one row: `talk` rules (conversation), `spoke`'s
  biography and audience (conversation), and `give`/`buy`/`hold` rules by item class (item-transfer,
  economy, inventory).
- **F-IL-2** No pack today asks "is this actor *allowed*": every refusal is spatial, a precondition, or
  a capacity. `PermissionDenied` is produced by no pack (grep on `main`). Rules are new behaviour in
  every pack, so each pack's conversion PR adds one lookup in `validate` and its offers.
- **F-IL-3** Facts stated through another pack's constructor (inventory's facts stated by item-transfer,
  economy and consumption) get their audience from the **owner**. So the owner's section governs them,
  and the stating pack cannot.
- **F-IL-4** "Enters an NPC's history" has three distinct meanings in this codebase, and the design must
  keep them apart (INV-4):
  1. the **fact log**, which always records (INV-11);
  2. the **biography**, the objective projection (ARC-29), which is configurable;
  3. **memory**, which is subjective and cognition's. It derives only from perceived facts (S10 §3.8),
     so it is governed through audience, never written by the world. Conversation's `Remembered` log
     (32 lines) is world state, conversation's own, and so a pack-specific consequence.
