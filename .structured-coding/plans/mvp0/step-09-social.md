# Step 09 / PR 10 — Social Café system set (S8): the town, social life, and routines

**Role:** step document for S8, proposing a split into three PRs (§2.7). It also holds the full PR
design for the first of them, **PR 10a**. PRs 10b and 10c are specified here at commit level, and each is
re-audited and detailed only after the PR before it merges (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 S8, §7 (current position and the world-data
follow-up)
**Lifecycle:** `DESIGN FROZEN` (2026-10-07, primary session; answers in §10.1). 10a frozen to the
commit; 10b and 10c frozen at the step level and detailed after the previous PR merges.
**Base:** `main @ f4301c1`. S7 merged as `4f4cb1d`, and `f4301c1` is the docs-only post-merge update.
**Branch / worktree:** `mvp0/pr-10-social` in `/Users/yuema137/mineworld-worktrees/s8-social`. Only
this session holds it. `vis-character` and `vis-environment` belong to other agents. This session
reads `vis-environment` and never writes to it.
**Depends on:**
- S4: `Process`, `wake`, `defer`.
- S5: `PersistentWorld` and `ARC-25`.
- S6: `MovementSystem`, `ARC-26`, and the disclosure of passages.
- S7: `mineworld run`, `PacedRuleController`, `ARC-27`, `inspect`.
- 05a: `ConversationSystem`.
- 05c: the World Pack loader.

Binding:
- [`CLAUDE.md`](../../../CLAUDE.md) §4 rules 1, 2, 5, 7, 8, 9, 10, 11, 14, 15
- [`docs/MVP.md`](../../../docs/MVP.md) §§2–5, §9 (`AC-2`, `AC-5`, `AC-6`, `AC-9`, `AC-11`, `AC-12`, `AC-13`, `AC-15`), §9.2
- [`docs/HUMAN_REVIEW_QUEUE.md`](../../../docs/HUMAN_REVIEW_QUEUE.md), Milestone B
- [`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) §§2, 4.4, 5, 9, 10, 11, 13
- [`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `ARC-23`, `ARC-25`, `ARC-26`, `ARC-27`
- [`step-04-clock-scheduler-process.md`](step-04-clock-scheduler-process.md)
- [`step-06-persistence.md`](step-06-persistence.md)
- [`step-07-movement.md`](step-07-movement.md) §10.1
- [`step-08-headless.md`](step-08-headless.md) §§1.3, 10.1

## Why this is PR 10, and why the file is `step-09`

The overall plan calls this step S8. PR 09 was S7, so S8 ships as **PR 10**, split here into 10a, 10b
and 10c. The step-document number follows the file sequence (`step-08-headless.md` was S7), so this
file is `step-09-social.md`.

---

# 1. Goal

S8 changes `worlds/social-cafe` in three ways:

1. It becomes the small town of `MVP.md` §3, with a café a player can actually stand in. Its interior
   matches the 3D slice, and someone at the counter can talk to Alice.
2. The people in it get to know each other, and relationships change because of what happens between
   them.
3. They live by routines and do things together.

Everything is shown headless, seeded and reproducible. Everything survives a real process restart.

The overall S8 checkpoint, and Milestone B, are the floor of this design:

```text
CP-1  a headless seeded run produces conversations, relationship-value changes and group activities
      (overall §3 S8), with the real systems, located before counted (ARC-23)
CP-2  a Person's objective biography, derived from the event log, matches the events that produced it
      (overall §3 S8; CORE_CONCEPTS §5.2; INV-4, INV-11)
CP-3  Milestone B: Alice and Bob persist, know each other, share an activity, and survive a restart
      with their history — through S5's real persistence and a real killed-and-restarted process,
      never in memory
CP-4  routines run on S4's Process: a person's day is a schedule-owned process whose wakes are the
      causes of the facts that begin each part of the day (AC-9 with Process causes that occur)
CP-5  the world-data follow-up (overall §7): a client standing at the café counter can `talk` to Alice
CP-6  change amplification (CLAUDE.md §4 rule 5): the new capabilities arrive as new modules, their
      contracts, registration and tests — no edit to kernel/, contracts/, Person, or an unrelated
      System Pack (presence, movement, conversation)
```

## 1.1 Scope, by PR

```text
PR 10a  the town                     (fully designed in §4.1)
  worlds/social-cafe/             café re-authored to the slice's layout (§2.5); the MVP town: apartments,
                                  cafe, store, park, workplace, joined by a street; 12 Persons
  cognition/rule-controller/      PacedRuleController chooses among a place's doors (seeded), instead of
                                  always the first (F-6)
  tests (existing)                social-cafe literals updated, each claim unchanged (I-5, §2.6)
  clients/protocol/demo/demo.gd   the 2D demo's room drawing re-sized to the new café (drawing only)
  clients/protocol/evidence/      AC-13 / AC-15 transcripts re-recorded from a real run, never edited
  docs                            worlds/social-cafe/README, MVP_STATUS

PR 10b  social life → Milestone B    (commit-level in §4.2)
  systems/group-activity/         new System Pack: invite, accept-invitation, decline-invitation, join,
                                  leave; a group activity is a Process it owns
  systems/relationships/          new System Pack: the `knows` relation and its values, changed only by
                                  reducing the social facts it subscribes to
  worldpack/src/catalog.rs        registration of both (Capability variants), and each pack's
                                  biographical event types
  tools/cli/src/biography.rs      `mineworld biography`: a Person's objective biography from a save
  cognition/rule-controller/      PacedRuleController invites, answers invitations, joins and leaves
  worlds/social-cafe/world.yaml   systems: + group-activity, relationships
  docs                            DECISIONS (relationship state; biography), MODULE_SPEC §8.1, systems
                                  READMEs, MVP_STATUS, HUMAN_REVIEW_QUEUE (Milestone B)

PR 10c  routines                     (outline in §4.3)
  systems/schedule/               new System Pack: Routine and Agenda; one routine Process per person
  worldpack (format, read, catalog)  a person's authored `routine:` → schedule's genesis fact (Q9)
  cognition/rule-controller/      PacedRuleController walks toward its agenda's place
  worlds/social-cafe/             routines for the population; systems: + schedule
  docs                            DECISIONS (schedule is an agenda, never a mover), PACKAGE_FORMAT,
                                  MODULE_SPEC §4.1
```

## 1.2 Non-goals

```text
any change to kernel/, contracts/, persistence/ or server/ code      none is needed (§2.4); I-1
any edit to presence, movement or conversation                       none is needed; I-1
Items, Inventory, Organizations, Jobs, Economy, Employment           S9 (market-town)
a relationship value that decays with time, moods, needs, sleep      later System Packs
romance or any typed relationship beyond `knows` and its values      MVP §4 non-goals
travel as a Process between non-adjoining places                     routes are walked (ARC-26)
biography compression (L1–L3), memory, LM context                    S10 / AC-10
System Pack configuration (tunable constants per world)              first pack that needs a different
                                                                     value (ARC-26 note, S7 Q5)
a generic pack-content seeding seam                                  S9's items/jobs (Q9; F-4)
RuleController (--agent) taking any new initiative                   unchanged (AC-15; I-9)
the 3D client's geometry or binding                                  vis-environment's (§2.5)
save migration                                                       still refused by name (ARC-25)
```

## 1.3 Frozen invariants (proposed; frozen only by the primary session)

- **I-1 No change below the composition root, and no edit to an unrelated System Pack.** No edit to
  `kernel/`, `contracts/`, `persistence/` or `server/` code. No edit to `systems/presence`,
  `systems/movement` or `systems/conversation`. New capabilities are new crates. The registration
  points are named in §2.4 and are the only shared files a new pack touches.
- **I-2 Single ownership.**
  - Relationship state, which is the `knows` edges and their values, is written only by
    `RelationshipsSystem`'s reducers.
  - Group-activity state, which is invitations, participation and the activity process, is written
    only by `GroupActivitySystem`.
  - Routine and agenda state is written only by `ScheduleSystem`.
  - None of the three states presence's `Arrived`. A change to another system's state travels only as
    a fact that the owner reduces.
- **I-3 Controllers stay stateless (`ARC-27`).** `PacedRuleController::decide(&self, &Observation)`
  stays a pure function of the seed, the pace and the observation. Every new decision reads only
  world state disclosed in the observation: pending invitations, the observer's activity, and the
  observer's agenda.
- **I-4 Activity before determinism, extended (S7 `I-9`).** Before any comparison of runs:
  - every seat has an accepted `move` and an accepted `talk` in every 30-day bucket, as today;
  - from 10b on, at least one `became-acquainted`, `relationship-changed`, `group-activity-started`
    and `group-activity-ended` fact occurs in every 30-day bucket;
  - from 10c on, at least one `agenda-changed` fact occurs per person per simulated day.
  A comparison made without these counts is not evidence.
- **I-5 Existing tests keep their claims.** A test may be edited only to update a literal that
  `social-cafe` used to supply: an id, a position, a count, or a seat list. The claim it makes and the
  meaning of every assertion stay the same. §9 lists every such edit. All 350 existing tests stay green.
- **I-6 A biography is derived and never stored.** It is computed from the fact log alone (`INV-4`,
  `INV-11`). Every entry cites the `EventId` it came from. It is regenerated identically from a
  restarted world's save.
- **I-7 No floating point** in any new or changed Rust. Relationship values are bounded integers.
- **I-8 Milestone B is shown through real persistence.** The test runs `run --save` with SIGKILL and
  then re-runs the same command. A separate process then reads the save. No in-memory world stands
  in for a restart.
- **I-9 `RuleController` and `--agent` behave exactly as today** (`AC-15`).

---

# 2. Re-audit: what is actually left of S8

## 2.1 What exists, from source (`main @ f4301c1`)

| S8 item | State | Where |
| --- | --- | --- |
| `conversation` System Pack | **done** (05a) | `systems/conversation`. Owns `ConversationHistory`, a projection bounded at 32. Provides `talk`. Emits `conversation-started` and `spoke`. Depends on presence. |
| seeded headless driver, `PacedRuleController` | **done** (S7) | `tools/cli/src/run.rs`, `cognition/rule-controller/src/paced.rs`. Submits only `talk` and `move` (paced.rs:110–231). |
| first-class `Relation` machinery | **done** (S1, S3), **unused by any real system** | `contracts/src/relation.rs`, `kernel/src/relations.rs`. `WorldView::relate`/`unrelate`, `Declarations::relation`. |
| per-relation state | **specified, never built** | `contracts/src/relation.rs` doc and step-01 `DD-6` say it is "a component keyed by the triple, owned by the declaring system". Components are keyed by `EntityId` only (`kernel/src/components.rs`). See F-2. |
| `Process` with `wake`, `defer` | **done** (S4), **unused by any real system** | `kernel/src/process.rs`, `WorldView::start_process` / `reschedule_process` / `end_process`. No real system starts one (S7 F-1). |
| perception seam for new packs | **done** (05a) | `PerceptionProvider::offers` / `discloses` (`systems/presence/src/interaction.rs`). A new pack contributes affordances and component records without editing presence. |
| `relationships` | **missing** | — |
| `schedule` | **missing** | — |
| `group_activity` | **missing** | — |
| MVP population (§3: 10–15 Persons, 5 Places) | **missing** | `worlds/social-cafe`: 4 Persons (3 of them seats), 2 places (café and street). |
| objective biography from the log | **missing** | `mineworld inspect` lists facts and causes. It knows no Person's history. |
| café layout matching the 3D slice | **wrong** | Coordinator's evidence (2026-10-06): with the door aligned, Alice and Bob are 0.3–0.5 m outside the café's west wall, and `talk` to Alice is refused `too_far_away` from inside the slice's café. See §2.5. |

## 2.2 What S8 must still add

```text
relationships      knowing someone (a `knows` edge), and values that change only because a social fact
                   happened (conversation's `spoke`, group-activity's acceptance/decline/ending)
schedule           a routine per person, run by a schedule-owned Process whose wakes change the agenda
group_activity     a shared activity several people join: invite → accept/decline → join/leave → ends
MVP population     12 Persons and the five MVP places joined by a street, café re-authored (§2.5)
biography          a Person's objective biography projected from the fact log, citing event ids
initiative         PacedRuleController must use the new actions, or a headless run exercises none of it
                   (ARC-23: an instrument that cannot see what it measures)
```

The last line repeats S7's key finding. Every new system acts only when somebody acts, and the only
thing that acts headless is `PacedRuleController`. Unless it learns to invite, answer, join, leave and
follow an agenda, CP-1 would count zero group activities and report it cleanly. The controller change
is therefore part of the step, not a convenience.

## 2.3 How the three systems divide the state

```text
                         reads                         owns (single writer)           states (facts)
presence        —                                Presence, present-in              arrived, person-entered-place
movement        presence                         Passages                          (presence's arrived)
conversation    presence                         ConversationHistory               conversation-started, spoke
group-activity  presence                         Invitations, Participation,       invited, invitation-accepted,
                                                 the group-activity Process        invitation-declined,
                                                                                   group-activity-started,
                                                                                   joined-group-activity,
                                                                                   left-group-activity,
                                                                                   group-activity-ended
relationships   — (reduces facts it subscribes   `knows` edges, Acquaintances      became-acquainted,
                  to: spoke, invitation-accepted, (values per counterpart)         relationship-changed
                  invitation-declined,
                  group-activity-ended)
schedule        presence (for the agenda's       Routine, Agenda,                  agenda-changed
                  place to be a known place)     the routine Process
```

Cargo dependency graph, one way, with no cycle:

```text
presence ◄── movement
presence ◄── conversation ◄──────┐
presence ◄── group-activity ◄────┤  relationships   (types only, to decode what it subscribes to)
presence ◄── schedule            │
                                 └── no system-level dependency (Q6)
```

`relationships` declares **no system dependency** (Q6). It subscribes to the facts of `conversation`
and `group-activity` and imports their crates only to decode the payloads. A world without
conversation is still a world with relationships. Nothing in it would change a value, and nothing
would be broken. This follows `CORE_CONCEPTS.md` §13.1's canonical shape, where the owner of the state
(`EconomySystem`) reacts to the requester's own fact (`WageDue`).

## 2.4 Change amplification (`CLAUDE.md` §4 rule 5), audited before designing

What adding each capability touches. **Bold** marks a shared file:

| Capability | New module | Registration (shared) | Other edits | Verdict |
| --- | --- | --- | --- | --- |
| relationships | `systems/relationships` | **root `Cargo.toml`**, **`worldpack/src/catalog.rs`**, `worldpack/Cargo.toml`, `world.yaml` | none | module + registration ✓ |
| group-activity | `systems/group-activity` | same | **`cognition/rule-controller`** (initiative) | ✓ for S8; see F-3 for S9 |
| schedule | `systems/schedule` | same | **`worldpack/src/{format,read,catalog}.rs`**, `PACKAGE_FORMAT.md`, `MODULE_SPEC.md` §4.1 (the `routine:` field); controller | the pack-format edit is raised as Q9 |
| biography | `tools/cli/src/biography.rs` | each new pack exports its biographical event types; catalog aggregates them | none | ✓ |

No row touches `kernel/`, `contracts/`, `persistence/`, `server/`, any existing System Pack, or any
"Person code". Person is not a type; a Person's meaning is the components systems attach to it
(`CORE_CONCEPTS.md` §3). **No material change-amplification failure was found for S8.** Two findings
are forwarded instead (F-1 and F-3), because they decide `AC-1` in S9 and not anything here.

## 2.5 The world-data follow-up: decided, re-author in 10a

**Decision: re-author `social-cafe`'s café to the slice's layout, in PR 10a, first.**

The reasons, in order of weight:

1. **It is a user-visible failure, not a cosmetic mismatch.** The coordinator ran the 3D slice against
   the real server. Walking café → street → café passes, 50/50 moves accepted. But `talk` to Alice is
   refused `too_far_away`, because the pack puts Alice and Bob 0.3–0.5 m west of the café's west wall
   in the slice's frame. A player in Demo B cannot talk to the barista. Milestone B is about people
   meeting, so leaving that broken would ship the milestone with its 3D path broken.
2. **No binding can fix it.** The world frame is fixed (+x east, +y north, `CORE_CONCEPTS.md` §6.1), so
   a client may only translate. The pack's door is **east** of everyone in the café. The slice's door
   is at the café's **west** end, as reference `03` draws it. A mirror would fit both, and the frame
   forbids a mirror. (Source: `vis-environment` `pr-01a-slice.md` §7b; `slice_link.gd:55–59`.)
3. **S8 rewrites the pack anyway.** The population grows from 4 to 12 Persons and places are added
   (§2.6). Positions authored now against a layout known to be wrong would be authored twice. Every
   test pinned to pack literals also churns once, not twice.
4. **The 3D client is first-class and permanent** (`CLAUDE.md` §1.1). The pack follows the reference
   the operator accepted. The client must not bend to an arbitrary early pack.

**The café, re-authored.** These are derived from the slice's own constants
(`vis-environment @ 27820df`, `clients/3d-spike/scripts/slice/{cafe.gd, cafe_interior.gd,
slice_world.gd}`). The frame's origin is the inner face of the west wall at the inner face of the front
wall. +x runs east along the frontage. +y runs north into the room, so the slice's −Z is +y, as
`MineWorldSpace.to_3d` maps it.

```text
room interior          x 0 … 8 320 mm, y 0 … 10 320 mm        (W 9.00 − 2×0.34; DEPTH 11.00 − 2×0.34)
doorway (here)         (1 610, 200)                          (slice DOOR_X −2.55 → 1.61 m from the inner
                                                             west face; 0.2 m into the room, which is the
                                                             point slice_link.gd binds the door to)
doorway (there)        unchanged, (0, 3 000) in the street's frame
counter                x 3 860 … 8 320, customer face y ≈ 6 570, staff face y ≈ 7 350
alice                  (6 000, 8 000), facing south (yaw 180 000)   behind the counter
"at the counter"       (6 000, 6 200) — 1.80 m from Alice, inside INTERACTION_RANGE (3 m)
bob                    on a counter stool, (4 500, 6 100) — 2.42 m from Alice: Alice and Bob can still
                       speak without anybody moving, the property bob.yaml states today
visitor                inside the door, (1 610, 600) — out of reach of Alice and Bob, as today
wanderer               by the second table, (7 110, 4 000) — 4.15 m from Alice, out of reach, as today
```

The exact millimetres are re-derived from the constants during C1 and recorded in §9. The acceptance
check does not move: **a client standing at the counter can `talk` to Alice.**

- **Server side, 10a.** It is a test over the real pack. Move a person to "at the counter" in legal
  strides from the door, then `talk` Alice and expect it accepted. Move the person back to the door,
  then `talk` again and expect it refused `too_far_away`.
- **Far side.** `./mineworld-slice --world --link`, run with a talk to Alice from the counter, belongs
  to the `vis-environment` session. That worktree and its launcher are outside this session's write
  and run permissions. 10a asks for it explicitly (Q2) and does not claim it.

**What the slice does not draw.** Apartments, store, park and workplace have no geometry in the slice,
and the slice's florist has no place in the pack. Each new place's doorway on the street is authored
along the street's two façade lines in the street's frame. 10a claims nothing about where any of them
fall in the slice. Binding them is the vis track's choice, made from what movement discloses.

## 2.6 The MVP population, and what it costs the existing tests

**Town.**
- Places are `apartments`, `cafe`, `park`, `store`, `street` and `workplace`. That is `MVP.md` §3's five
  places, plus the street that joins them.
- The street is the hub. Each of the five has one passage, onto the street. Every route between two
  places is therefore at most two doors long.
- `store` and `workplace` are plain places in S8. Their Organizations, Items and Jobs are S9's (Q3).

**People.**
- 12 Persons. `alice` (the only `barista`), `bob`, `visitor` and `wanderer` keep their keys and roles.
  Eight are added.
- **Seats.** Every Person is a seat except one added non-seat. That keeps `server_command.rs:152`'s
  claim (a non-seat cannot be occupied) with a subject. Bob becomes a seat, because Milestone B needs
  Bob to answer an invitation.
- `MVP.md` §6 asks for 8–12 NPCs and up to 4 players. A seat is either.

**Cost to existing tests (audited; full table in §8.2 F-5).** `worldpack/tests/social_cafe.rs`,
`tools/cli/tests/{commands, server_command, ac15_one_alice, restart, inspect, run, run_restart}.rs`,
and `persistence/tests/kill_and_resume.rs` assert pack literals:
- entity ids, which are assigned places-then-people in key order;
- the genesis fact count, which is 5 today;
- positions and stride counts;
- the seat list;
- the composition list.

The Godot `clients/protocol/evidence/*` transcripts record "6 entities, 3 systems". `demo.gd` draws a
5 × 4.4 m café at 60 px/m. Under I-5 each test keeps its claim and only the literal moves. The
transcripts are re-recorded from a real `run.sh evidence` run (Q11), never edited.

**Cost to `run`.** `mineworld run` drives every seat. Consults grow linearly: 11 seats is about 3.7×
today's 129 600 consults for 300 days. S7 measured 19.1 s in memory and 37.9 s saved, so expect about
70 s and 140 s in the debug profile. `tools/cli/tests/run.rs` runs three such runs in parallel. Under
S7's Q10 rule, the pace rises rather than the days falling, if one in-memory 300-day run exceeds about
60 s. The rule is applied from measurement in C3 and recorded (Q4).

**A controller trap the town would spring (F-6).** `PacedRuleController`'s *leave* always heads for
the place's **first** passage, in `PlaceId` order (`paced.rs:216–231`, `movement/src/component.rs:73`).
With five doors on the street, everyone on the street walks into the lowest-id place and never comes
back. 10a makes the door a seeded choice among the disclosed passages.

## 2.7 Why three PRs, and why this split

S8 has three independently checkable outcomes, and the dependencies run one way:

```text
10a  the town          pack data, the café fix, one controller fix, test literals, Godot evidence
                       checkpoint: a client at the counter talks to Alice; the 12-person town runs
                       300 days, every seat active (I-4); AC-13/AC-15 re-recorded and passing
10b  social life       group-activity + relationships + biography + controller initiative
                       checkpoint: CP-1, CP-2, CP-3 — Milestone B
10c  routines          schedule + the pack's routine field + controller agenda
                       checkpoint: CP-4
```

- **10a first.** It fixes a user-visible defect now. It also absorbs all of the fixture churn (ids,
  counts, positions, seats) in one reviewable PR that contains **no new semantics**. 10b's and 10c's
  diffs are then about systems, not literals.
- **relationships and group-activity together in 10b.** Relationships reduces group-activity's facts.
  Shipping relationships alone and adding group-activity later would edit relationships when
  group-activity lands, which is exactly the amplification §2.4 guards against. Both packs are born
  with their final subscriptions, and Milestone B closes in one PR.
- **schedule last.** No Milestone B clause needs it, and it carries the one open format question (Q9).
  It is still S8's (overall §3), and it is not deferred out of S8.
- **Not one PR.** One PR would be roughly two systems' worth of S6, plus S7's controller work, plus a
  fixture rewrite, plus a format change. Reviewing that as one diff hides the fixture churn inside the
  semantics.
- **Not four.** Splitting relationships from group-activity is rejected above.

---

# 3. Design decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-1** | Three PRs, in the order and with the scopes of §2.7. | §2.7. |
| **SD-2** | The café is re-authored to the slice's layout in 10a, with the values in §2.5. The acceptance is a client at the counter talking to Alice. | §2.5. |
| **SD-3** | The town is a star. The `street` is the hub, and each of `apartments`, `cafe`, `park`, `store` and `workplace` has one passage onto it. | `MVP.md` §3. Routes are at most two doors, so a stateless controller can reach any place: through the target's door if it is adjacent, otherwise to the street first. Travel is walked (`ARC-26`), never teleported. |
| **SD-4** | `PacedRuleController`'s *leave* picks a door by `mix(seed, observer, instant)` among the disclosed passages. From 10c, it prefers the door toward its agenda's place. | F-6. Still a pure function of the observation (I-3). |
| **SD-5** | **Relationship state.** `relationships` declares a directed relation type `knows` (Person → Person). It owns a component `Acquaintances` on the *from* Person, mapping each counterpart `PersonId` to `RelationshipValues { familiarity: 0..=1000, regard: −1000..=1000, exchanges: u32, activities_shared: u32, first_met: (WorldTime, EventId), last_contact: WorldTime }`. The edge and the entry are written together, in the same reduction, by the one owner. A test checks that they always agree. | `CORE_CONCEPTS.md` §§4.4, 9: relationships are typed edges with system-owned values. Components are keyed by `EntityId` (F-2), so "a component keyed by the triple" (`DD-6`) is realized as a component on `from`, keyed by `to`, under the one relation type this system declares. It is the same identity, and no kernel change is needed. Values are integers (I-7) and asymmetric (Alice's regard for Bob is not Bob's for Alice). |
| **SD-6** | **What changes a relationship**, all in `relationships`' reducers. (1) `spoke`: in both directions familiarity +10 and `exchanges` +1. (2) `invitation-accepted`: regard +50 in both directions. (3) `invitation-declined`: the inviter's regard for the decliner −30. (4) `group-activity-ended`: for every ordered pair of participants, familiarity +50, regard +20 and `activities_shared` +1. All are saturating. The constants are published and documented, with no world configuration (ARC-26 note). | Every change is a reduction of a fact another system stated, so a replayed log produces the same values (`ARC-25`). The values move both up and down, which makes "changes because of events" observable in both directions. |
| **SD-7** | **Relationship facts.** `became-acquainted { person, counterpart }` is emitted when an edge is first formed, once per direction. `relationship-changed { person, counterpart, from: Level, to: Level }` is emitted when a derived **level** crosses a boundary. The levels are `Acquaintance`, `Friendly` (familiarity ≥ 300 and regard ≥ 100), `Friend` (≥ 600 and ≥ 300) and `Close` (≥ 900 and ≥ 600), and a level can go back down when regard falls. Fine-grained value changes are reductions that state no fact. Each fact is caused by the fact that crossed the boundary (`Causation::Event`). | One fact per `spoke` would roughly double a run's log, since S7's 300-day run has 43 141 `spoke` facts. Levels are what a biography and a client can name. The values remain derivable from the log, because they are reductions of logged facts (Q5). |
| **SD-8** | `relationships` declares **no system dependency**. It subscribes to `spoke`, `invitation-accepted`, `invitation-declined` and `group-activity-ended`, and Cargo-depends on `mineworld-conversation` and `mineworld-group-activity` for their types. | §2.3. It installs in any world. Disabling conversation leaves relationships enabled and inert toward speech (`AC-2` direction). Registry rule: only emitting another system's vocabulary requires depending on it (`kernel/src/registry.rs:186–205`), and subscribing does not. Q6. |
| **SD-9** | **Relationship disclosure.** `RelationshipsSystem::discloses` returns a Person's `Acquaintances` only to that Person (observer == subject). | `INV-13`: how Alice regards Bob is Alice's to know. It is in her own observation for her controller (AC-5, S10), and it is not shown to Bob or a third party. |
| **SD-10** | **Group activity.** Actions:<br>• `invite { target, kind }` — same place, within 3 000 mm.<br>• `accept-invitation { from }` — same place, and the invitation is pending and not expired.<br>• `decline-invitation { from }`.<br>• `join { activity }` — an activity in the actor's place.<br>• `leave`.<br>An invitation is pending for 600 s; expiry is checked at the answer, so no deferral is needed. Accepting joins the inviter's current activity, or starts one with both people. A group activity is a Process of kind `group-activity` owned by `GroupActivitySystem`, at the place, with expected end = start + 3 600 s. Its state holds the `kind` and the member list. `Participation` (on each member) names the process.<br>The activity ends at its expected end (`wake`), or as soon as fewer than two members remain. It always emits `group-activity-ended { activity, kind, place, members }`, where `members` is everyone who took part. A member who enters another place leaves the activity: the system reacts to presence's `person-entered-place`. Rejections are kernel vocabulary (`TooFarAway`, `TargetUnavailable`, `PreconditionFailed`, `Busy`), decided server-side. | `MVP.md` §5: invite, accept, reject, join, leave. `CORE_CONCEPTS.md` §10: something happening over time is a Process, and the owner decides how it ends (`INV-3`). `kind` is a validated slug the inviter proposes and the system does not interpret, so "coffee" and "walk" are content, not code (`INV-12`). |
| **SD-11** | Group-activity disclosure. To the observer: its own pending invitations and its own `Participation`. To anyone perceiving a member: which activity that member is in (process id and kind). | What a controller needs to answer an invitation or join, and what a client needs to show "they are together". It discloses nothing about people not perceived. |
| **SD-12** | **Biography** is a generic projection over fact envelopes, not a pack-specific narrator. An entry is `{ at, event_id, event_type, place, counterparts }`, read from the envelope's kernel fields: type, subjects, participants, location, id. A fact is biographical for Person P if P is among its subjects or participants **and** its event type is in the composition's biographical set. Each pack exports `pub const BIOGRAPHICAL: &[EventTypeId]`, its own judgement over its own vocabulary, and `Capability::biographical()` aggregates them in the catalog. In S8 the set is `became-acquainted`, `relationship-changed`, `group-activity-started`, `joined-group-activity`, `left-group-activity` and `group-activity-ended`, plus from 10c `agenda-changed` when the agenda's label changes. Shown by `mineworld biography <world> --save DIR --person KEY [--json]`: one line per entry, counterparts by authoring key, and the event id on every line. | `CORE_CONCEPTS.md` §5.2: biography is derived and always regenerable. §5.4: every summary keeps its event ids. §4.4: prose is generated for display and never read back. Generic over envelopes, so adding a pack adds a constant and no biography code. No contracts change. Compression is AC-10's (S10). Q10. |
| **SD-13** | **Schedule is an agenda, never a mover.** `ScheduleSystem` owns `Routine` (authored segments `{ from: seconds-of-day, place, label }` covering the day) and `Agenda` (the current segment, since when, until when). It owns one Process of kind `routine` per person, whose expected end is the next boundary. `wake` emits `agenda-changed { person, place, label }` (`Causation::Process`) and reschedules the process. It never states `Arrived`. A controller reads its agenda in its own observation and walks there through `move`. A human may ignore it. | `INV-1`, `INV-6`: controllers propose, and a schedule that teleported people would override a human's control and bypass movement's rules. `ARC-26`: being somewhere is presence's, and getting there is movement's. It is also what gives the run its first real `Process` causes (CP-4, S7 L-4). The time of day is world seconds since the epoch mod 86 400, a schedule-pack convention that the kernel does not know (`INV-12`). Q8. |
| **SD-14** | A person's `routine:` is authored in the person's pack file (`people/*.yaml`). The loader turns it into schedule's genesis fact `routine-assigned`, built by schedule's constructor, exactly as `location:` becomes presence's `arrived` (`worldpack/src/catalog.rs` `located`). | The same shape as `location` and `passages`: one optional field of an existing content kind mapped to one owner's genesis constructor. `catalog.rs`'s own doc names the trigger for a generic seeding seam as "a pack seeding a content kind of its own — items, jobs" (S9). Q9 raises the alternative. |
| **SD-15** | `PacedRuleController` gains initiative over the new actions. It answers a pending invitation (accept with 70/100, otherwise decline). It invites someone available who is not in its activity, joins a perceived activity, and leaves an activity. From 10c it walks toward its agenda's place before anything else. All of this is new roll bands in the existing priority scheme, and every choice is a `mix(seed, observer, instant)`. `RuleController` is unchanged. | §2.2: without initiative, CP-1 measures nothing. Stateless: the invitation, the activity and the agenda are world state disclosed to the observer, so a controller rebuilt after a restart decides identically (`ARC-27`). |
| **SD-16** | **Milestone B evidence** is a real-binary test in 10b. `run worlds/social-cafe --headless --seed S --days D --save A` is killed with SIGKILL partway and the same command re-run. A separate `mineworld biography` process then reads `A`. The test **locates** before counting (`ARC-23`): it finds the first `became-acquainted` between alice and bob and the first `group-activity-ended` whose members include both, and fails by name if either is missing. It also checks that alice's biography from the restarted save equals the uninterrupted control's byte for byte, and that both biographies match the log (CP-2's check). | `I-8`, `ARC-25`. The restart is a real process death, and the history is read by a process that never held the world. |

---

# 4. Commit plan

Each commit tracks implementation, validation and review separately. Evidence is recorded in §9 as
each item completes. Line counts are estimates, not targets.

## 4.1 PR 10a — the town (full design)

### C0 — Design (this document) — docs only

- [x] Implementation: §§1–11 written from the audit in §8.
- [x] Validation: `python3 scripts/check_decision_ids.py`, `python3 scripts/check_doc_headings.py` — §9 E-0.
- [x] Review: every claim in §2 cites a file and line, a command, or the coordinator's evidence. The
  one material forward finding (F-1, F-3) is raised, not routed around.

### C1 — The café, re-authored to the slice's layout (SD-2)

**Goal.** A Person at the café counter can talk to Alice, and the pack matches the layout the 3D client
draws.

**Scope.**
- `worlds/social-cafe/places/cafe.yaml`: the passage `here`, and the comments.
- `worlds/social-cafe/people/{alice,bob,visitor,wanderer}.yaml`: positions, facing, comments.
- The literals of tests pinned to those positions, under I-5:
  - `worldpack/tests/social_cafe.rs`
  - `tools/cli/tests/{ac15_one_alice, restart, server_command}.rs`
  - `persistence/tests/kill_and_resume.rs`

No new place or person yet, and no system change. **Depends on:** freeze.

- [ ] Implementation:
  - [ ] Derive the frame and values of §2.5 from `vis-environment @ 27820df`'s constants. Record the
    derivation, constant by constant, in §9 E-1.
  - [ ] Rewrite the four people's positions and the café doorway's `here`. Comments state why each
    person stands where they do, as today's do, and cite reference `03` and the slice for the layout.
  - [ ] Update every test literal that names a position or a stride count, keeping each claim. In
    particular:
    - "the visitor starts out of reach of Alice" stays true;
    - "Bob and Alice can talk without moving" stays true;
    - the walk through the door still produces `[arrived, arrived, person-entered-place]`.
- [ ] Validation:
  - [ ] New test `worldpack/tests/social_cafe.rs::a_person_at_the_counter_can_talk_to_alice`. From the
    door, through legal strides only, the person reaches "at the counter"; `talk` to Alice is
    Accepted. Back at the door, it is refused `TooFarAway`. The bound is `INTERACTION_RANGE`, the
    published constant, never one derived from the positions under test (`ARC-23` rule 2).
  - [ ] Every test touched still passes, and the diff of each shows only literal changes (I-5).
  - [ ] Mutation: put Alice back at (1200, 2400) and the new test fails. Revert.
- [ ] Review:
  - The doorway, counter and people fit inside the slice's room interior. Locate this: print each
    person's position against the room bounds, rather than asserting "fits".
  - No position sits on the door-to-counter lane the slice keeps clear (`cafe_interior.gd`
    `_loose_seating` comment).
  - No test's claim was weakened.

**Acceptance.** As validation. **Failure cases.** A literal that cannot be updated without changing a
claim is a material finding: stop and report. **Commit boundary.** Pack positions and test literals
only.

### C2 — The town: places, people, seats (SD-3)

**Goal.** `MVP.md` §3's population and places.

**Scope.**
- `worlds/social-cafe/places/{apartments,park,store,workplace}.yaml` (new), each with one passage onto
  the street.
- `worlds/social-cafe/people/*.yaml`: eight new people, each with tags, a note and a location.
- `world.yaml`: places, population, seats.
- The test literals that name ids, counts, the seat list or the population, under I-5:
  - `worldpack/tests/social_cafe.rs`
  - `tools/cli/tests/{commands, server_command, ac15_one_alice, inspect, run}.rs`
  - `server_command.rs:152`'s non-seat subject

**Depends on:** C1.

- [ ] Implementation:
  - [ ] Four place files. Doorways on the street are spaced along the street's two façade lines.
    Doorways inside each place are near that place's south edge, by convention.
  - [ ] Eight people distributed across the places, each with a one-line note. Only Alice keeps the
    `barista` tag, because `ac15_one_alice` and `demo.gd` find her by it.
  - [ ] Seats: every Person but one added non-seat. Re-point `server_command.rs`'s non-seat test from
    `bob` to that person, with the claim unchanged.
  - [ ] Update id, count and list literals, keeping claims.
- [ ] Validation:
  - [ ] `mineworld validate worlds/social-cafe` lists six places, 12 people and 11 seats, with genesis
    = passages + arrivals. The count is printed, and the test checks it against the pack's own lists
    rather than against a literal derived from the loader.
  - [ ] Every touched test passes, each diff literal-only (I-5).
  - [ ] `cargo test -p mineworld-worldpack`, `-p mineworld-cli` (all but `run`, which is C3's).
- [ ] Review:
  - Every place is reachable from every other in at most two doors. Locate this: print the adjacency
    from the loaded `Passages`.
  - No person is placed outside their place's authored extent (comments only; places have no extent
    in the format).

### C3 — Doors are a seeded choice; the 300-day run over the town (SD-4, F-6)

**Goal.** People on the street can go anywhere and come back. `AC-11`/`AC-12` hold for the 12-person
town.

**Scope.**
- `cognition/rule-controller/src/{paced.rs, paced_tests.rs}`
- `tools/cli/tests/{run.rs, run_restart.rs, headless/mod.rs}` (the SEATS list becomes the pack's seats;
  thresholds re-located)
- `run.rs`'s pace, only if Q4's rule triggers

**Depends on:** C2.

- [ ] Implementation:
  - [ ] *leave* picks among disclosed passages with `mix(seed, observer, instant)`. Nothing else in the
    priority scheme changes.
  - [ ] The headless tests' SEATS become the pack's seat list, read from the pack and not hard-coded,
    so I-4 covers every seat.
- [ ] Validation:
  - [ ] Unit, in `paced_tests.rs`: from the street with five doors, the doors chosen over 64 seeds ×
    16 consults cover all five. From each non-street place, the only door is the street's.
  - [ ] `run.rs`. AC-11 (300 days, seed 7): I-4 holds for every seat in every bucket. AC-11 also
    requires `person-entered-place` into **each** of the six places, located per place. AC-12 compares
    bytes as in S7.
  - [ ] Measure one 300-day in-memory debug run's wall time and record it. If it exceeds about 60 s,
    raise the pace per Q4 and record the measurement that triggered it.
  - [ ] `run_restart.rs`: kill points and the stop-and-continue test re-located against the new
    history. A straddling line is still found from the log, never assumed.
  - [ ] Mutation: *leave* back to "first passage", and the per-place entry check fails (most places
    never entered). Revert.
- [ ] Review: decide stays `&self`; no `HashMap`; no float.

### C4 — The 2D demo and the Godot evidence, from a real run

**Goal.** `AC-13` and `AC-15` are evidenced against the new pack, never by editing old recordings.

**Scope.**
- `clients/protocol/demo/demo.gd`: the room drawing's centre and scale for an 8.3 × 10.3 m café.
  Drawing only, no rule.
- `clients/protocol/evidence/*`, re-recorded by `clients/protocol/run.sh evidence`.

**Depends on:** C2 (C3 does not affect a hosted world).

- [ ] Implementation: `demo.gd` constants; nothing else in the client.
- [ ] Validation:
  - [ ] `clients/protocol/run.sh evidence` (Godot 4.7.2 headless). The transcripts show the new entity
    count, Alice's recall line, and "the scripted run is over".
  - [ ] `cargo test -p mineworld-cli --test ac13_semantic_parity --test ac15_one_alice` over them.
  - Needs `run.sh` permission (Q11). If it is not granted, this item is reported `NOT RUN` and the
    primary session runs it.
- [ ] Review: no validity rule in `demo.gd`. The recorded request files differ only where the pack's
  positions moved (`actor_location`), which is exactly AC-13's allowed difference.

### C5 — Documentation and ledger close for 10a

- [ ] `worlds/social-cafe/README.md` (the town, the seats, the café's provenance), `docs/MVP_STATUS.md`.
- [ ] Full gates once on the final executable head (§6).
- [ ] Review: the README links to MODULE_SPEC §4 and does not restate rules.

## 4.2 PR 10b — social life → Milestone B (commit level; detailed after 10a merges)

```text
C0  design refresh against 10a's merged state; pr-10b section frozen by the primary session
C1  specs before code: DECISIONS ARC-28 (relationship state: knows edge + values on the from-Person,
    level facts, no system dependency — SD-5…SD-9), ARC-29 (biography: generic over envelopes,
    pack-declared selection — SD-12); MODULE_SPEC §8.1 `biography`; CORE_CONCEPTS untouched
C2  systems/group-activity: actions, Invitations, Participation, the group-activity Process (start,
    wake → end, end below two members, member leaves the place → left), facts, offers and disclosure;
    tests over a hand-built world as movement's are: invite/accept/decline/expire, join/leave, wake
    ends with members, leaving the place, Busy/TooFarAway/PreconditionFailed from the system;
    a persisted world restarted mid-activity resumes it (wake after restart) — real persistence
C3  systems/relationships: `knows`, Acquaintances, SD-6 reductions, SD-7 level facts, disclosure to
    self only; tests: edge ⇔ entry agree after every reduction; values move both ways; level crossings
    caused by the crossing fact; a world with relationships and no conversation installs and is inert;
    disabling conversation leaves relationships enabled (AC-2 direction)
C4  registration: worldpack catalog (Capability::GroupActivity, ::Relationships; biographical());
    world.yaml systems; literal updates under I-5 (composition lists, "systems" lines, the AC-15 talk
    fact count which grows by relationships' facts)
C5  PacedRuleController initiative (SD-15); unit tests: answers a pending invitation once per window,
    invites only an available person, never decides on anything not in the observation; restart
    equivalence (fresh instance decides identically)
C6  `mineworld biography` (tools/cli/src/biography.rs); tests: on a real 30-day save, every entry's
    event exists, names the person, has the entry's type/at/place; every biographical fact naming the
    person appears exactly once (completeness, both directions); mutation — drop one event type from
    the selection, or skip the participants field — fails
C7  Milestone B (SD-16): run --save, SIGKILL, same command; `biography --person alice` from a fresh
    process equals the control's byte for byte; located: alice↔bob became-acquainted, a shared
    group-activity-ended, a relationship-changed for the pair; I-4 extended counts per 30-day bucket;
    `inspect` still resolves every cause
C8  docs: systems/{relationships,group-activity}/README, systems/README, MVP_STATUS,
    HUMAN_REVIEW_QUEUE Milestone B (with the exact launch command and what to look at), gates
```

## 4.3 PR 10c — routines (outline; detailed after 10b merges)

```text
C1  specs: DECISIONS ARC-30 (schedule is an agenda, never a mover — SD-13); PACKAGE_FORMAT and
    MODULE_SPEC §4.1 `routine:` (or the seam, per Q9)
C2  systems/schedule: Routine, Agenda, routine Process (genesis start, wake → agenda-changed,
    reschedule), disclosure to self; tests incl. a restart across a boundary
C3  worldpack: `routine:` → routine-assigned genesis via schedule's constructor; refusals by name
    (unknown place, overlapping or empty segments)
C4  PacedRuleController walks toward its agenda's place (SD-4/SD-15); social-cafe routines
C5  the run: agenda-changed per person per day located; `inspect` now meets Process causes —
    extend its AC-9 check to resolve them against the routine and group-activity process facts, or
    keep counting them, decided from the audit then; gates; docs
```

---

# 5. Integration checkpoints

```text
10a  a client at the café counter → talk alice → Accepted; from the door → TooFarAway        CP-5
     mineworld validate worlds/social-cafe → 6 places, 12 people, 11 seats
     mineworld run worlds/social-cafe --headless --seed 7 --days 300 → every seat moved and talked
         in every 30-day bucket; every place entered; no fault                                AC-11
     same seed again → identical facts/journal/snapshots; seed 8 → different                  AC-12
     clients/protocol/run.sh evidence → AC-13 / AC-15 transcripts from the new pack           AC-13/15

10b  mineworld run … --seed S --days D --save A → conversations, relationship changes and group
         activities in every 30-day bucket, located before compared                          CP-1
     mineworld biography worlds/social-cafe --save A --person alice → entries == the log's     CP-2
     killed and re-run → biography and save byte-identical to the control; alice and bob
         acquainted, a shared activity, read by a process that never held the world          CP-3

10c  every person's agenda changes on schedule, caused by a Process; people follow it         CP-4
```

**Adversarial criteria (`ARC-23`).**
- Two idle histories agreeing is excluded by I-4's located counts.
- A biography that is empty matches any log, so this is excluded by requiring located entries of each
  biographical type for alice and by the completeness direction.
- A relationship test that only ever increases is excluded by asserting a regard decrease from a
  `decline`.
- A restart that re-read nothing is excluded by S7's reported tail.
- The counter test's bound is the published `INTERACTION_RANGE`, never one derived from the positions.
- An edited test that quietly changed its claim is excluded by I-5's per-file listing in §9.

# 6. Test ownership and verification

```text
STATIC      fmt, check, clippy -D warnings (owns: no HashMap/HashSet, unused code). No lint enforces
            I-7 (no float); it is owned by review.
UNIT        rule-controller: door choice, invitation answers, agenda walking, restart equivalence;
            each new pack's local semantics (value arithmetic, level boundaries, invitation expiry)
INTEGRATION each new pack over a hand-built world (as systems/movement/tests do): actions → facts →
            reductions → persistence restart; worldpack over the real pack
REAL-LIFECYCLE  the real binary over the real pack: run (AC-11/12), run_restart (AC-6), Milestone B
            (SIGKILL + re-run + biography from a fresh process), inspect (AC-9); Godot evidence (10a C4)
REAL-LLM    NOT REQUIRED — no model anywhere (CLAUDE.md §5)
CI          no workflow exists (S13); the local gates below are terminal evidence, once on each PR's
            final executable head
```

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
cargo test -p mineworld-persistence --test kill_and_resume
python3 scripts/check_decision_ids.py
python3 scripts/check_doc_headings.py
```

The budget follows S7's Q10: long runs and kill tests total a few minutes of wall time. The pace rises
before days fall.

# 7. Self-review against the frozen specifications

```text
CHECKED  INV-12 / I-1: no kernel, contracts, persistence or server change; days, levels, routines and
         activities live in packs; "seconds of day" is schedule's convention
CHECKED  INV-7 / CLAUDE.md §4 rule 1: each new state has one writer (§2.3); relationships changes only
         by reducing facts others stated; schedule never writes presence (SD-13)
CHECKED  INV-3: a group activity and a routine are Processes, ended by their owner; their facts are
         events
CHECKED  INV-4 / INV-11: biography is derived from the log, never stored, cites event ids (I-6)
CHECKED  INV-13: relationships disclosed to self only; group-activity discloses only about perceived
         people; agenda to self only
CHECKED  INV-1 / INV-6: schedule proposes nothing and moves no one; controllers walk
CHECKED  ARC-26: no new pack states another's vocabulary; relationships subscribes (no dependency
         needed); group-activity reacts to presence's person-entered-place as a subscriber
CHECKED  ARC-27: controller stays stateless; new decisions read disclosed world state only
CHECKED  CLAUDE.md §4 rule 5: §2.4 — modules, registration, tests; no edit to an unrelated pack
CHECKED  CLAUDE.md §4 rule 11: no abstraction for one use — biography's selection is a constant per
         pack, not a trait; the routine field follows location's shape (SD-14), the seam waits for S9
CHECKED  CLAUDE.md §4 rule 16: no dependency added
CHECKED  the two gate questions (overall §2 item 8): invite/join declare their spatial requirement in
         the pack and are decided server-side; both clients receive them as affordances, unchanged
FLAGGED  F-1, F-3: S9's AC-1 test — forwarded, not S8's to fix
FLAGGED  Q9: schedule's routine field edits the pack format
```

---

# 8. Source audit (`main @ f4301c1`, 2026-10-06)

## 8.1 What was inspected

```text
worlds/social-cafe/** (every file)
worldpack/src/catalog.rs (Capability, located, opened); worldpack test coupling (delegated audit, below)
systems/conversation/src/{lib,event,component}.rs; systems/presence/src/{interaction,observe}.rs
kernel/src/{system.rs (System trait: validate/resolve/react/wake/interrupt), view.rs (WorldView:
  relate/unrelate/defer/start_process/reschedule_process/end_process/request_interrupt),
  registry.rs (dependency and borrowed-vocabulary rules), process.rs, relations.rs}
contracts/src/relation.rs (module doc, DD-6); contracts/src/time.rs (epoch)
cognition/rule-controller/src/paced.rs (decision bands, via the delegated audit)
tools/cli/tests/*, persistence/tests/kill_and_resume.rs, clients/protocol/{demo,evidence,run.sh}
  — delegated read-only audit of every file naming social-cafe, results in F-5
vis-environment @ 27820df (read only): mineworld-slice; clients/3d-spike/scripts/slice/{cafe.gd,
  cafe_interior.gd, slice_world.gd, street.gd, slice_link.gd}; clients/protocol/mineworld/space.gd;
  .structured-coding/plans/vis-3d-godot-2/pr-01a-slice.md §7b
docs: MVP §§2–6, 9, 9.2; HUMAN_REVIEW_QUEUE; CORE_CONCEPTS §§1–5, 6.1, 8–13; DECISIONS ARC-23, ARC-25,
  ARC-26, ARC-27; overall §§1–3, 7; step-06 §10.1; step-07 §10.1; step-08 (whole)
coordinator evidence, 2026-10-06: the slice against the real server, talk to Alice refused
  too_far_away from inside the café
```

## 8.2 Findings

```text
F-1  (forwarded to S9) Every new System Pack edits worldpack/src/catalog.rs (a closed enum, by design
     until ARC-8's WASM registry), root Cargo.toml and Cargo.lock. CLAUDE.md §4 rule 5 permits
     "registration", and MVP.md's AC-1 forbids edits to kernel, Person, renderer and controllers only;
     but overall §1 glosses AC-1 as "a diff that touches only systems/ and worlds/", which no linked
     Rust pack can satisfy. S9's mechanical test must name the registration points it allows, or the
     overall's gloss is reworded. Not S8's to resolve; S8 does not make it worse.
F-2  Per-relation state is specified as "a component keyed by the triple" (contracts/src/relation.rs,
     DD-6), and the kernel keys components by EntityId only. Realized without a kernel change as a
     component on the `from` Person keyed by `to` (SD-5) and recorded in ARC-28 so the spec's sentence
     has a stated implementation. Not a code/spec contradiction — no code claimed to implement it.
F-3  (forwarded to S9) PacedRuleController is action-specific: it submits only actions it knows
     (talk, move; +invite/accept/decline/join/leave in 10b). AC-1 forbids controller changes between
     social-cafe and market-town, so market-town's headless people will not buy or work unless S9
     finds an affordance-generic policy or scopes AC-1's "controller implementations" explicitly.
     S8 changing the controller is within S8; the consequence for S9 is recorded here.
F-4  A pack field is a closed schema mapped by worldpack (location → presence, passages → movement).
     Schedule's routine is the third mapping of the same shape (SD-14); catalog.rs names items/jobs —
     content kinds of their own — as the trigger for a generic seam. Q9.
F-5  Existing tests pin social-cafe literals (delegated audit): ids are assigned places-then-people
     in key order, so any new place or person renumbers; genesis count 5; positions and stride counts
     in ac15_one_alice, restart, server_command, social_cafe, kill_and_resume; seat list in run,
     run_restart, server_command; composition lists in social_cafe, commands, server_command, inspect;
     the AC-15 first talk yields exactly 2 facts (grows in 10b); demo.gd's room is sized for 5 × 4.4 m;
     Godot transcripts record "6 entities, 3 systems". Absorbed in 10a/10b under I-5.
F-6  PacedRuleController's leave always takes the first passage in PlaceId order: with a street of
     five doors everyone converges on the lowest-id place and never returns. Fixed in 10a C3.
F-7  run cost is linear in seats; 11 seats ≈ 3.7× S7's consults. Q4.
F-8  No real system yet uses Relation or Process. S8 is the first real use of both; the kernel's
     relation and process paths are exercised by real packs and by persistence restart for the first
     time (10b C2's restart-mid-activity test, 10c C2's restart-across-boundary test).
```

## 8.3 Material findings

None that changes a frozen invariant of an earlier step, a public contract, or an ownership boundary
within S8. F-1 and F-3 are material **for S9's `AC-1` test** and are forwarded with evidence rather
than resolved here.

---

# 9. Ledger and evidence

```text
E-0  C0 design, 2026-10-06, on main @ f4301c1 + this file. check_decision_ids: 37 ids, all distinct;
     check_doc_headings: 142 numbered sections across 22 documents, none duplicated. No cargo gate run
     (docs only). Test count 350 taken from overall §7 / step-08 E-final, not re-counted in Phase 1.
```

## 9.1 Limitations (expected)

```text
L-1  Relationship values do not decay; a pair who never meet again stay at their level.
L-2  Activity kinds are uninterpreted slugs; an activity is "being together, at a place, for a while".
L-3  Routes are found only in a star town (SD-3); a general route-finder is not built.
L-4  Biography is L0 only (CORE_CONCEPTS §5.4): long runs give long biographies; compression is S10.
L-5  The 3D far-side counter check is the vis-environment session's to run (Q2).
L-6  RuleController (--agent) takes no new initiative: a hosted Alice answers but never invites (I-9).
```

---

# 10. Questions for the primary session / operator

```text
Q1   Split S8 into PR 10a (the town), 10b (social life, Milestone B), 10c (routines), in that order
     (§2.7). Alternatives: one PR (hides fixture churn in semantics); four (relationships before
     group-activity — would edit relationships when group-activity lands). Recommended: three.
Q2   Re-author the café to the slice's layout in 10a (§2.5), acceptance "a client at the counter can
     talk to Alice", server-side test here; the 3D far-side run (`./mineworld-slice --world --link`
     talking from the counter) is requested from the vis-environment session after 10a merges.
     Recommended.
Q3   Social-cafe becomes the MVP §3 town: apartments, cafe, park, store, workplace + street, all plain
     places; organizations, items and jobs wait for S9. Alternative: only the places S8's systems use
     (apartments, cafe, park), adding store/workplace in S9 — renumbers every id a second time.
     Recommended: all five now.
Q4   12 Persons, 11 seats (Bob becomes a seat; one added non-seat keeps the non-seat test's subject).
     If one in-memory 300-day debug run exceeds ~60 s, the pace rises (S7 Q10), recorded with the
     measurement. Recommended.
Q5   Relationship facts at level crossings only (SD-7); fine values reduced silently from logged facts.
     Alternative: relationship-changed on every value change (~+43 000 facts per 300 days).
     Recommended: levels.
Q6   relationships declares no system dependency; subscribes and decodes (SD-8). Alternative: depend on
     conversation and group-activity — forbids disabling either while relationships is enabled.
     Recommended: no dependency.
Q7   Group activity semantics as SD-10 (invite/accept/decline/join/leave; 600 s invitation; 3 600 s
     process; ends below two members; leaving the place leaves the activity). Recommended.
Q8   Schedule is an agenda, never a mover (SD-13). Alternative: schedule states presence's Arrived at
     segment boundaries — bypasses movement and overrides any controller, human included. Recommended:
     agenda.
Q9   `routine:` as a per-person pack field mapped like `location:` (SD-14; PACKAGE_FORMAT + MODULE_SPEC
     §4.1 + worldpack format edits in 10c). Alternative: introduce now a generic seam where a System
     Pack owns a section of a person/place file and seeds its own genesis facts — the seam S9's items
     and jobs will need, and would keep 10c's diff out of worldpack's format. Recommended: the field
     now, the seam in S9 when a second content kind differs (F-4); the operator may prefer building the
     seam in 10c to spare S9 a format change.
Q10  Biography: generic over envelopes, pack-declared biographical types, `mineworld biography <world>
     --save DIR --person KEY` (SD-12). Alternative: per-pack narrator functions returning typed entries
     — more expressive, and an abstraction with one shape so far. Recommended: generic.
Q11  10a C4 needs `clients/protocol/run.sh evidence` (Godot) to re-record AC-13/AC-15 transcripts. It
     was allow-listed for S6 (step-07 Q8) but is not in this session's brief. Grant it for C4, or the
     primary session runs that step. Recommended: grant.
Q12  I-5: existing tests may have pack literals updated, claims unchanged, each edit listed in §9.
     Alternative: keep the four-person café as a frozen second fixture for the old tests — two
     fixtures for one world, and the old one would stop being "the" social-cafe. Recommended: update.
```

## 10.1 Answers — primary session review, 2026-10-07

Decided under the operator's autonomous authorization (overall §7) and their standing rule that
objective architectural correctness belongs to the agent. **10a is frozen to the commit.** 10b and
10c are frozen at the step level: their scope, invariants and acceptance are fixed here, and each is
detailed to the commit and reviewed after the previous PR merges. Eleven answers follow the
recommendation; **Q9 is answered the other way**; Q6 carries a condition.

**Q1 — ACCEPTED.** Three PRs, in order. Milestone B closes at 10b.

**Q2 — ACCEPTED.** The server-side acceptance — a client at the counter can `talk` to Alice and is
refused from the door — is 10a's. The 3D far-side run is routed to the environment session after 10a
merges.

**Q3 — ACCEPTED.** All five MVP places now, to avoid renumbering every id twice. The 3D slice
depicts only the café, the street and the florist; the other places are semantic, which is permitted —
presentation may lag the world, never the reverse.

**Q4 — ACCEPTED.**

**Q5 — ACCEPTED.**

**Q6 — ACCEPTED, with a condition.** Subscribing is not emitting. `ARC-26` requires a dependency only
to *state* another system's vocabulary, and reacting to facts is the intended inter-system channel
(`CLAUDE.md` §4 rule 1). No registry dependency is right, and it is better for `AC-2`: relationships
stays enabled with conversation disabled and simply hears nothing. **Condition:** decode through the
owner crate's published event types — a Cargo dependency on the vocabulary, not a system dependency
— never by re-implementing their payload shape locally. A local copy of another pack's schema is
exactly the drift risk R-3 names, and it would bypass the `…TooNew`/`…Outdated` refusals.

**Q7 — ACCEPTED.**

**Q8 — ACCEPTED.** An agenda that controllers follow, never a mover. Anything else would bypass
movement and override a human player.

**Q9 — ANSWERED THE OTHER WAY: build the generic seam in 10c, not a `routine:` field.** The design's
own trigger is "when a second content kind differs", and `routine` *is* the second kind: `location`
was the first, and it is owned by presence. `CLAUDE.md` §4 rule 11 asks for exactly this — observe the
repeated concept, then define the abstraction. A per-system field in the World Pack format is also
the same change-amplification pattern as F-1: adding a system edits a central format. The seam — a
System Pack owns a named section of a person or place file, validates it, and seeds its own genesis
facts — keeps 10c's diff out of worldpack's format and spares S9 a second format change. Record it in
`docs/PACKAGE_FORMAT.md` and `MODULE_SPEC` §4.1 when 10c lands, and migrate `location` onto it only if
that stays a no-op for every existing test; otherwise leave `location` as it is and say why.

**Q10 — ACCEPTED.**

**Q11 — RESOLVED.** `clients/protocol/run.sh` is already allow-listed for every session, bare,
with arguments, or through `bash`. Run it yourself.

**Q12 — ACCEPTED.** Every changed literal is listed in §9 with its unchanged claim.

### F-1 and F-3 are carried to S9 as material findings against `AC-1`

They are recorded correctly here and are not S8's to fix. But they bear on the project's frozen
top-level criterion — *"materially different games … composing the same core entities with
different independently installable interaction systems, without modifying the kernel"* — so they are
named here and in overall §7, not left in a limitations list:

- **F-1:** installing a pack today means editing `worldpack/src/catalog.rs`, the root `Cargo.toml`
  and `Cargo.lock`. The kernel stays unmodified, so the letter of `AC-1` holds. But "independently
  installable" does not: a pack cannot be added without recompiling a central list.
- **F-3:** `PacedRuleController` submits only the actions it knows by name, so a new pack's actions
  go unused until the controller is edited. The likely direction is a controller that acts on the
  **affordances an observation offers**, which the protocol already carries, rather than on a
  hard-coded action list.

**S9's design must resolve both, or record why `AC-1` is still met without resolving them.**

---

# 11. Execution contract (proposed for PR 10a; confirmed only at freeze)

```text
PROJECT / PR        MVP-0 · Step 09 / PR 10a — the town (S8, first of three)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-09-social.md (this file), §4.1
RELATED / BINDING   overall.md §§2, 3 (S8), 7; MVP §§3, 6, 9 (AC-11, AC-12, AC-13, AC-15);
                    CORE_CONCEPTS §6.1; DECISIONS ARC-23, ARC-26, ARC-27; step-08 §§1.3, 10.1
IMPLEMENTATION BASE main @ f4301c1; branch mvp0/pr-10-social; worktree
                    /Users/yuema137/mineworld-worktrees/s8-social (held by this session only)
APPROVED SCOPE      §1.1 PR 10a, as answered in §10
FROZEN INVARIANTS   §1.3 I-1 … I-9
SEQUENCE            C0 → C1 → C2 → C3 → C4 → C5, each committed and pushed when coherent
VALIDATION BUDGET   unit/integration/static: unrestricted; real-model: NOT REQUIRED; long runs and kill
                    tests a few minutes of wall time in total (S7 Q10)
LIVE DOCUMENTATION  this file (§4 checkboxes, §9 ledger)
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for PR 10a at C1
ENDPOINT AUTHORITY
  implementation + local validation   after DESIGN FROZEN only — source: the brief ("Phase 2 — only
                                      after you are told the design is frozen")
  semantic commits, branch push       authorized — source: the brief ("Commit and push after every
                                      small step"); D-12
  PR creation / update                authorized — source: the brief ("Open a PR with gh pr create")
  clients/protocol/run.sh             UNRESOLVED — Q11
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only; the brief: "do not merge it"
POST-MERGE SYNC     the planning session owns step/overall updates; this session owns this document
NORMAL STOP         PR 10a READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any change to §1.3, to a public contract beyond §1.1, to ownership, or to scope;
                    any existing test whose claim cannot be kept under I-5 — stop and report
```
