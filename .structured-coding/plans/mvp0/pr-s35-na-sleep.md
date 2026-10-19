# PR N-a — The `sleep` System Pack: sleep, wake, and four ways a sleep ends

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes)

```text
Design revision:        revision 1 (2026-10-18), design session
Approved by / evidence: the operator, 2026-10-18, in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes
Implementation base:    origin/main at C0; C0 records the commit
Start condition:        this design FROZEN by the operator AND DOC-S35 (#278) merged
Execution contract:     §11
Lifecycle:              DESIGN FROZEN 2026-10-18
```

Follows the frozen step-35 (no new operator question). One bounded refinement of *how* step-35 §4.2 rule 4 is
implemented (SD-Na-3, finding F-Na-1) keeps every observable rule of the step and of `ARC-283`; Q-Na-1 asks the
coordinator to confirm it, because it adds one lookup to the SDK.

**Effort:** `mvp0`, release scope. **Parents:** [`step-35-needs-and-sleep.md`](step-35-needs-and-sleep.md) §§3, 4, 6.1,
7, 8, 9 (AC-S35-4, AC-S35-6 asleep half, AC-S35-7), 10.1 row N-a, 11; records
[`ARC-283`](../../../docs/decisions/ARC-283.md) (sleep), [`ARC-284`](../../../docs/decisions/ARC-284.md) (needs, the
consumer of this PR's facts), [`ARC-286`](../../../docs/decisions/ARC-286.md) (controllers; consumer of the offers),
all written by DOC-S35 (PR #278).
**Consumed by:** PR N-b (`needs` hears `fell-asleep` and `woke`), PR N-c (controllers request `sleep` and `wake` and
read `Asleep` and `sleep-terms`), PR N-e (the clients render `Asleep`).

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-s35-na-nb`, branch
`docs/pr-design-s35-na-nb`, audit base `origin/main @ a8bba658` (2026-10-18).

**Binding:** [`CLAUDE.md`](../../../CLAUDE.md) §2.2, §4 rules 1–3, 5, 7, 8, 10, 11, 13, 15; the lane charter's SPEED
FIRST and "no world-specific hard-coding" blocks (operator, 2026-10-18); `ARC-25` (replay), `ARC-28` (decode another
pack's fact through its published type, no system dependency), `ARC-31` (sections), `ARC-33` (installed set),
`ARC-34` (offers), `ARC-63`/`ARC-64`/`ARC-65` (the Interaction List, classes, consequences), `ARC-90` (generated
reference); `docs/MODULE_SPEC.md` §4.2 (the `sleep` section, as written by DOC-S35).

---

## 1. Goal, in one paragraph

Nobody in a MineWorld world ever sleeps (F-S35-2). This PR adds a System Pack, `sleep`, that lets any person ask to
sleep for a number of minutes at a place where sleeping is allowed, and to get up again. While a person sleeps the
pack holds one component, `Asleep { since, alarm }`, and one Process that ends at the alarm. A sleep ends in exactly
four ways, each the pack's own reaction: the alarm, the sleeper's `wake`, somebody talking to the sleeper, and the
sleeper acting (walking or speaking). No other pack learns that sleep exists. Where one may sleep is data: the
world's Interaction List decides, and when it says nothing the pack's rule is "at a place tagged `residential`". The
pack discloses that a person is asleep to everyone who sees them, the alarm to the sleeper only, and the bounds of
`for` to the person who may ask. Nothing in the pack names a world, a place or a person.

---

## 2. Audit (`origin/main @ a8bba658`, 2026-10-18)

### 2.1 What the pack is built from

| Thing | Where | What this PR takes from it |
| --- | --- | --- |
| A pack with a Process and reactions | `systems/employment/src/system.rs` (`react` starts a `ShiftProcess` from its own `hired`; `wake` at the process end; `react` to presence's `person-entered-place` decoded through presence's type); `systems/employment/src/process.rs` (`ProcessKind` with `Owner`) | the shape of `SleepProcess`, of reducing one's own facts, and of hearing another pack's fact |
| A process that ends | `systems/group-activity/src/system.rs:342-349` (`wake` → `end_process`), `:541`; `systems/group-activity/src/component.rs:144` (a `ProcessId` stored in a component) | `Asleep` stores its `ProcessId`; the alarm wake ends the process; an early wake ends it from the reducer |
| Kernel process API | `kernel/src/view.rs:350` `start_process` (an expected end must be strictly later), `:364` `end_process`, `:386` `reschedule_process` ("a wake for the old end is ignored"); `kernel/src/process.rs` `ProcessStart::{with_participants, at_place, ending_at, uninterruptible}` | the alarm is `ending_at(now + for)`; `for ≥ 15 min` keeps it strictly later |
| An action pack with validate / resolve / offers | `systems/consumption/src/{system,offer,codec}.rs` (`validate` in numbered order; `resolve` states facts; `Offer::complete`; `living_person`) | the validation order and the "living Person" check |
| Parametric offer | `systems/movement/src/system.rs:425-426` (`Offer::new::<WalkTo>`: no payload) | `sleep` is offered parametric (`for` is the requester's); `wake` is offered complete |
| Offer refusal | `systems/presence/src/interaction.rs` `Offer::refused` (`ARC-34` note: an IL `PermissionDenied` is shown on the offer) | the `sleep` offer carries the same refusal `validate` gives |
| The Interaction List | `sdk/rust/src/interactions/{mod,decl,section,resolve,lookup}.rs`; `systems/conversation/src/interactions.rs` (a section with one action, parameters by role, two facts with audiences) | `SleepSystem` implements `InteractionSection`; `parameters!` with bounds |
| Lookup with nothing configured | `sdk/rust/src/interactions/lookup.rs:157-177` `permits`: "With nothing configured, `Ok` without reading anything else" | finding F-Na-1 |
| Classes | `authoring/src/classes.rs` (`{ class, of, tag }` in `configure/classes.yaml`; `matches`, `type_of`); `sdk/rust/src/interactions/resolve.rs:226-246` (a class a world section names must be defined, `ClassUndefined`), `:296` (`classes.restricted_to(referenced)` copies only classes the world's own entries name) | finding F-Na-1 |
| Reference lists | `InteractionSection::reference_lists` (default: `default`, empty); no pack overrides it (`grep -rn "fn reference_lists" systems cognition` → nothing) | finding F-Na-1 |
| Hearing another pack's fact without a dependency | `systems/economy/src/system.rs:70-73,193-195` (hears `wage-due`, answers with a fact) | `sleep` hears `spoke` with no registry dependency on conversation |
| Facts it hears | `systems/presence/src/event.rs:31-58` `Arrived { person, location }` (`arrived`); `systems/conversation/src/event.rs` `Spoke { speaker, listener, utterance }` (`spoke`) | the `acted` and `addressed` triggers |
| Computed disclosure with a declared table | `systems/calendar/src/system.rs:173,184,298` (`CalendarLight` declared, installed, disclosed as a computed value) | `SleepTerms` is declared and installed, never written |
| Holder-only disclosure | `systems/employment/src/system.rs` `discloses` (`subject != observer` → nothing) | `alarm` and `sleep-terms` |
| Disclosure to whoever perceives | `systems/presence/src/interaction.rs:97-119` (`discloses` is called once per perceived entity) | `Asleep`'s audience is perception's, place-bounded today (Q-S35-14) |
| Installed set | `systems/installed/src/lib.rs`, `Cargo.toml`; the appearance install commit `318f6ffc` (one line each, plus `worldpack/tests/refusals.rs`) | C5 |
| Test worlds | `systems/relationships/tests/support/mod.rs:59-70` (`configured(section, classes)`: a configured IL fact built as the loader builds it); `systems/consumption/tests/support/mod.rs` (a café from the real packs); `tools/cli/tests/forgetting_sections.rs` (scratch copies of a reference world, loaded by the real loader) | the two in-crate test worlds and the loader fixtures |
| `residential` tag | `worlds/{social-cafe,market-town}/places/apartments.yaml`, `worlds/lakeside/places/{cottages,boathouse}.yaml` | content already uses the tag the pack's default reads |

### 2.2 Findings

- **F-Na-1 — the step's mechanism for "where one may sleep" cannot work as written.** Step-35 §4.2 rule 4 and
  `ARC-283` say: the pack's reference list `default` forbids `sleep` except at places of the class `residential`.
  The audit shows three reasons this does nothing useful today:
  1. With no `configure/sleep.yaml`, `interactions::permits` returns `Ok` without reading any list
     (`lookup.rs:165-177`), so the reference list is never consulted and sleep would be permitted everywhere.
  2. A class is world content (`configure/classes.yaml`); a reference list compiled into the pack cannot define
     one, and `ClassUndefined` is checked only for the world's own entries (`resolve.rs:233-246`).
  3. The classes copied into the configured fact are only those the world's own entries name
     (`restricted_to(referenced)`, `resolve.rs:296`), so a class named only by a reference list would not match
     even in a world that defines it.
  The result the step wants — default: sleep at home; an author permits a bench or forbids a host's home without
  code — is kept by SD-Na-3 below.
- **F-Na-2 — the `sleep` offer cannot carry bounds.** `Offer` has a payload only when complete
  (`interaction.rs:183-192`); a parametric offer carries no field description. A controller would have to hard-code
  15 … 840. The lane lead's requirement (2026-10-18): disclose the bounds in force. SD-Na-6 adds a computed,
  holder-only `sleep-terms` record.
- **F-Na-3 — `wake` needs no place rule.** DOC-S35's `MODULE_SPEC.md` §4.2 text lists `wake` as an action of the
  section. If it is declared, a world writing `default: forbid` would forbid getting up. SD-Na-3 declares only
  `sleep` in `ACTIONS`; a rule naming `wake` is refused at load as an undeclared action (the SDK's existing refusal).
  C7 corrects the `MODULE_SPEC.md` line.
- **F-Na-4 — `arrived` also follows a shove.** `bodies` can move a person; presence states `arrived` for them. A
  shoved sleeper is woken with `acted`. That is the honest answer (being shoved wakes you); recorded, not special-cased.

---

## 3. Scope

### 3.1 In scope

1. SDK: one lookup, `interactions::ruling`, that says whether the world's list has a rule for an action here
   (SD-Na-3). Additive; no stored or encoded form changes.
2. New crate `systems/sleep` (`mineworld-sleep`): the actions, facts, component, process, Interaction List section,
   reactions, disclosure and offers of §4.
3. The installed set: one line in `systems/installed/Cargo.toml` and `src/lib.rs`; the workspace `Cargo.toml` member.
4. Tests: the pack's own integration tests on two in-crate test worlds; loader tests on scratch copies of two
   reference worlds; a pack-level `AC-2` removal test; the AC-S35-7 property test (written here, the 30-day run at the
   milestone).
5. Docs: `systems/sleep/README.md`; the `MODULE_SPEC.md` §4.2 `sleep` entry corrected (F-Na-1, F-Na-3); dated notes on
   `ARC-283` and `ARC-63`; the generated reference regenerated.

### 3.2 Not in scope

- `needs` (N-b). Controllers requesting sleep, politeness to sleepers, alarms from a commute (N-c). Any world enabling
  `sleep` (N-d; reference worlds stay without it in this PR). Client rendering, the agent client's `AsleepLine`, the LM
  prompt (N-e).
- A bed, a lying posture, sleep quality (Q-S35-10). A forced sleep when exhausted (Q-S35-3). Time skip (Q-S35-9).
- Making `movement` or `conversation` refuse a sleeper (`ARC-283` option (b), declined).

### 3.3 Invariants

- **I-Na-1** The kernel, `contracts/`, `movement`, `conversation` and `presence` are unchanged (I-S35-1).
- **I-Na-2** `sleep` writes only `Asleep` and its own `sleep-interactions` component; it states only `fell-asleep`,
  `alarm-set`, `woke` and its configured fact (I-S35-2). It never writes or names another pack's state.
- **I-Na-3** One `sleep` Process per sleeper, started on `fell-asleep`, ended on `woke`; no periodic Process
  (I-S35-3).
- **I-Na-4** No randomness, no clock but world time (I-S35-4).
- **I-Na-5** A world that does not enable `sleep` is byte-identical: same facts, same journal, same baselines
  (I-S35-5). Installing the pack moves nothing but the generated reference and the build's list of available systems.
- **I-Na-6** Per person, `fell-asleep` and `woke` strictly alternate, starting with `fell-asleep`; `Asleep` exists
  exactly between the two (AC-S35-7).
- **I-Na-7** Nothing in the crate names a world, a place key, a person or an hour. The one vocabulary word is the tag
  `residential` (SD-Na-3), like `consumption`'s category `food`.
- **I-Na-8** Defaults are fixed here, before any run (`ARC-23`): `shortest_minutes` 15, `longest_minutes` 840,
  `woken_by_talk` true, the tag `residential`.

---

## 4. Design

### 4.1 SD-Na-1 — Contract

```text
crate        mineworld-sleep (systems/sleep)              System id `sleep`, version 1
provides     sleep { for: u16 }   minutes; the alarm is now + for
             wake  { }
emits        fell-asleep { person, place }                audience: the place (IL-narrowable to participants);
                                                          no alarm (step-35 §4.1: the alarm is not in the fact)
             alarm-set   { person, alarm }                audience: the sleeper only (Visibility::Entities);
                                                          stated with fell-asleep, so replay rebuilds Asleep
             woke        { person, slept, reason }        audience: the place the sleeper is in; slept = seconds
             sleep-interactions-configured                its IL fact, genesis only, SystemInternal
owns         Asleep { since, alarm, process }             on a sleeping person
             SleepTerms                                   declared, never stored: the computed `sleep-terms` record
             sleep-interactions                           the IL component on every Place (SDK)
process      SleepProcess "sleep"                         one per sleeper: participants [sleeper], at the place,
                                                          ending at the alarm, uninterruptible
depends on   presence                                     (registry) where the sleeper is; Arrived's type
hears        arrived (presence), spoke (conversation)     decoded through their published types (ARC-28)
discloses    asleep        { asleep: true, since }        to every observer who perceives the person
                           + alarm                        only when observer = subject
             sleep-terms   { shortest_minutes,            to the observer about themself, a living Person, at a
                             longest_minutes }            place where the world has `sleep` installed
section      configure/sleep.yaml (ARC-63)                no person or item section
biography    none by default; `consequences` may switch it on for either fact (Q-S35-13)
```

Rust types (public, for decoding by N-b, N-c, N-e and tests):

```rust
pub struct Sleep { r#for: u16 }                 // serde name "for"; Action, OWNER = sleep
pub struct Wake {}                              // Action
pub struct FellAsleep { person: PersonId, place: PlaceId }                     // Event "fell-asleep" v1
pub struct AlarmSet { person: PersonId, alarm: WorldTime }                     // Event "alarm-set" v1
pub struct Woke { person: PersonId, slept: u64, reason: WokeReason }           // Event "woke" v1
pub enum WokeReason { Alarm, Asked, Addressed, Acted }                         // serde kebab-case
pub struct Asleep { since: WorldTime, alarm: WorldTime, process: ProcessId }    // Component "asleep" v1
pub struct SleepTerms { shortest_minutes: u16, longest_minutes: u16 }          // Component "sleep-terms" v1
pub struct SleepState { sleeper: PersonId }                                     // the process state
pub const SLEEPING_TAG: &str = "residential";
```

The disclosed `asleep` record is built from `Asleep` at disclosure: `{ asleep: true, since }` for any observer, plus
`alarm` when the observer is the sleeper. `process` is never disclosed. The alarm reaches nobody else: `fell-asleep`,
which the place perceives, does not carry it (step-35 §4.1, the lane lead's ruling of 2026-10-18), and `alarm-set`,
which does, is perceived by the sleeper alone. `alarm-set` is not in the Interaction List's `FACTS` (its audience is
not a world's choice) and is never biographical.

### 4.2 SD-Na-2 — Validation, in order (step-35 §4.2)

`sleep`:

1. The payload decodes as `Sleep`; otherwise `PreconditionFailed`.
2. The actor is a living Person and there is no target; otherwise `NoSupportedInteraction`
   (`consumption::offer::living_person`'s rule, written again here: a three-line read).
3. The actor is not asleep (`Asleep` absent); otherwise `Busy`.
4. The actor is located (`Presence`); otherwise `PreconditionFailed`. Its place is `here`.
5. `for` lies within `shortest_minutes … longest_minutes`, looked up with `actor` = the actor and `place` = `here`;
   otherwise `Rejection::System { code: "sleep-for-out-of-bound", detail: "for must be <s> … <l> minutes, not <n>" }`.
6. Where one may sleep (SD-Na-3) permits `here` for this actor; otherwise `PermissionDenied`.

`wake`:

1. Decodes; living Person; no target (as above).
2. `Asleep` present; otherwise `NoSupportedInteraction`.

No spatial requirement on either (`SpatialRequirement::NONE`): one sleeps where one is (step-35 §4.2 rule 5). Every
refusal leaves the world unchanged (the kernel records nothing for a refused request; AC-Na-1 compares snapshots).

Resolution: `sleep` states, in this order, `fell-asleep { person, place: here }` with the consequence the list routes
(default `Visibility::Place(here)`) and `alarm-set { person, alarm: now + for × 60 }` to the sleeper only. `wake` states `woke { reason: asked }` (SD-Na-4).

### 4.3 SD-Na-3 — Where one may sleep: the list's rule, else the pack's tag rule

A new SDK lookup, beside `permits` (`sdk/rust/src/interactions/lookup.rs`):

```rust
/// What the world's list says about `action` here, before any pack default:
pub enum Ruling {
    Unconfigured,          // the world has no section for this pack
    Rule(bool),            // a rule matched; true = permit (most specific wins, forbid among equals)
    Default(bool),         // no rule matched; the section's resolved `default` (permit when unsaid)
}
pub fn ruling<S: InteractionSection>(read, at: PlaceId, action: &ActionTypeId, roles: &Roles) -> Ruling;
```

It reads the same `Interactions<S>` component and the same precedence as `Resolution::permits`
(`resolve.rs:60-81`), split so a pack can tell "a rule said so" from "nothing said so". `permits` is unchanged and
keeps its callers; `Resolution` and every encoded form are unchanged (no baseline moves).

`sleep` decides `allowed(actor, here)`:

```text
Rule(e)          → e                                     the author wrote a rule that matches
Default(false)   → false                                 the author wrote `default: forbid`
Default(true)    → here carries the tag `residential`    nothing matched and nothing forbids by default
Unconfigured     → here carries the tag `residential`    the world wrote no configure/sleep.yaml
```

Examples, each a test in C2 (`systems/sleep/tests/where.rs`), on a world whose places are `flat` (tag
`residential`), `park` (tag `park`), `guest-room` (tags `residential`, `guest`):

| `configure/sleep.yaml` (classes in `configure/classes.yaml`) | flat | park | guest-room |
| --- | --- | --- | --- |
| none | permit | `PermissionDenied` | permit |
| `rules: [{ action: sleep, place: bench-park, effect: permit }]`, class `bench-park` = place tag `park` | permit | permit | permit |
| `rules: [{ action: sleep, place: guest-place, effect: forbid }]`, class `guest-place` = place tag `guest` | permit | denied | denied |
| `default: forbid`, `rules: [{ action: sleep, place: bench-park, effect: permit }]` | denied | permit | denied |
| `rules: [{ action: sleep, effect: permit }]` (every place) | permit | permit | permit |
| `rules: [{ action: sleep, actor: visitor, effect: forbid }]`, actor class `visitor` | permit for others, denied for a visitor | denied | permit for others |

Why this and not the alternatives (recorded for Q-Na-1):

| Option | Verdict |
| --- | --- |
| (a) The step's literal reference list naming a class `residential` | does nothing in an unconfigured world (F-Na-1.1) and cannot name a world class (F-Na-1.2, 1.3) |
| (b) Require every world that enables `sleep` to write `configure/sleep.yaml` and `configure/classes.yaml` | every world repeats the same two files; a world that forgets them fails silently (nobody can sleep) or needs a new "required configuration" loader rule |
| (c) Teach the SDK "implicit tag classes" for reference lists | changes class semantics for every pack (`ARC-64`), and still leaves F-Na-1.1 |
| (d) **The list decides when it says something; otherwise the pack's tag rule** | chosen: every author need of the step is data; one additive SDK lookup; nothing world-specific in code |

The tag `residential` is the pack's vocabulary — what a home is called — in the way `consumption`'s `food` is the
category that is eaten. A world whose homes carry another tag either tags them `residential` or writes one rule.

### 4.4 SD-Na-4 — Reactions and the Process

```text
fell-asleep (own)      nothing to write (the place it names is read again from Presence when needed)
alarm-set (own)        insert Asleep { since: at, alarm, process } after starting SleepProcess:
                       participants [person], at_place(the sleeper's place), ending_at(alarm), uninterruptible
woke (own)             remove Asleep; end SleepProcess when it is still running (an early wake)
wake (process, alarm)  state woke { reason: alarm, slept: alarm − since }; end the process
wake request           resolve states woke { reason: asked }
spoke (conversation)   listener asleep ∧ woken_by_talk(listener, listener's place) → woke { reason: addressed }
                       speaker asleep                                         → woke { reason: acted }
                       (both asleep: each gets its own woke, speaker first)
arrived (presence)     person asleep, and the cause is not world genesis       → woke { reason: acted }
```

- **One truth.** `Asleep` is written only while reducing the pack's own `alarm-set` and `woke` (the
  conversation-history discipline: state is a projection of the pack's own log). A reaction that wakes someone
  emits `woke` and does not touch `Asleep`; the reducer does.
- **No double wake.** A reaction emits `woke` only while `Asleep` is present. Two triggers that reach the pack in
  one cascade before the first `woke` is reduced (in practice: none today; a `walk-step` states one `arrived`, a
  `talk` one `spoke`) are guarded by a per-person check against the facts already emitted by the same reaction; C3
  tests the one reachable pair, an alarm and a `wake` request at the same instant (the ended process's wake is
  ignored by the kernel, `view.rs:386-405`).
- **`slept`** is `woke.at − since` in whole seconds. Facts at the same instant as `fell-asleep` cannot occur (an
  alarm is ≥ 15 min later; a reaction at the same instant gives `slept: 0`, allowed).
- **Audience of `woke`.** `Visibility::Place(the sleeper's place)` by default, IL-narrowable to participants
  (`FactDecl`). Its place is the sleeper's `Presence` at the instant, not the `fell-asleep` place (a walker has moved).
- **Parameters** are looked up with `actor` = the sleeper and `place` = the sleeper's place at the instant.

### 4.5 SD-Na-5 — The Interaction List section (`configure/sleep.yaml`)

```rust
mineworld_sdk::parameters! {
    pub struct SleepParameters, partial SleepPartial {
        woken_by_talk:    bool = true, false ..= true;
        longest_minutes:  u16  = 840,  60 ..= 840;
        shortest_minutes: u16  = 15,   15 ..= 120;
    }
}
ACTIONS          [ sleep: roles [actor, place], regional: true ]          (wake is not declared, F-Na-3)
FACTS            fell-asleep  roles actor = participant 0, place = envelope place; default Place, narrowest
                              Participants, biography configurable
                 woke         the same
PARAMETER_ROLES  [actor, place]
FIELD_ROLES      none (every field may be scoped by actor or place)
CONFIGURED       "sleep-interactions-configured";  COMPONENT "sleep-interactions"
reference lists  only `default`, empty (SD-Na-3 makes a non-empty one unnecessary)
```

`shortest_minutes > longest_minutes` after resolution (possible only when both are scoped differently) makes every
`for` refused with the bound named; the pack does not reorder them. A load-time check of that is not added (both
values are in range; the refusal names the problem at request time).

### 4.6 SD-Na-6 — Disclosure and offers

- `asleep`: `discloses(observer, subject)` returns the record when `subject` has `Asleep`, for any observer
  perception hands it (perception has already decided the observer perceives the subject); `alarm` is included only
  when `observer == subject`.
- `sleep-terms`: when `observer == subject`, a living Person, and located: `{ shortest_minutes, longest_minutes }`
  in force for them at their place. Declared and installed as a component table (the `CalendarLight` precedent),
  never written. A world without `sleep` has no such table and no such record (I-Na-5).
- Offers, with `target: None`, for a living Person observer:
  - awake: `Offer::new::<Sleep>(SpatialRequirement::NONE)`, `.refused(PermissionDenied)` when SD-Na-3 denies
    `here`, `.refused(PreconditionFailed)` when not located; parametric, like `walk-to`;
  - asleep: `Offer::complete(&Wake {}, NONE)`, and `sleep` offered `.refused(Busy)`.
  With a target: nothing.

### 4.7 Edge cases, decided

| Case | Behaviour |
| --- | --- |
| A sleeper is spoken to with `woken_by_talk: false` | stays asleep; the line is still heard (conversation's history is untouched) |
| A sleeper's walk is requested and accepted by `movement` | the first `arrived` wakes them (`acted`), before or as they leave; movement is unchanged |
| A sleeper eats, buys or gives | stays asleep; those packs do not know about sleep, and those acts are not "walking or speaking" (step-35 §4.3). Recorded; N-c's controllers ask nothing but `wake` while asleep |
| The sleeper's entity is destroyed while asleep | the process wake finds no living person: end the process, state nothing (the `ProcessNotRunning`-style guard of employment) |
| Save and resume mid-sleep | `Asleep` and the process are persisted state; the alarm fires at the same instant (C6 tests it) |
| `for` exactly at a bound (15, 840) | accepted |
| A world sets `longest_minutes: 60` and a controller asks 840 | refused, naming "15 … 60" |
| Two sleepers talk to each other | the speaker wakes `acted`, the listener `addressed` (if `woken_by_talk`) |

---

## 5. Acceptance criteria (fixed now, before any run; `ARC-23`)

Each is exercised on **two worlds**, both test-only: **`flat-and-park`** (in-crate, `systems/sleep/tests/support`:
places `flat` tagged `residential`, `street`, `park`; people `ana`, `ben`, `cy`; real `presence`, `movement`,
`conversation`, `sleep`) and **`hostel`** (in-crate variant: places `dorm` tagged `dormitory` only, `garden`,
`hall`; classes `dorm-place` = place tag `dormitory`, `traveller` = person tag `traveller`; `configure/sleep.yaml`
permitting `dorm-place`, forbidding `traveller` in `garden`, `woken_by_talk: false`, `longest_minutes: 600`). The
loader criteria use scratch copies of `worlds/social-cafe` and `worlds/lakeside` with `sleep` appended to `systems:`
(test-only copies; the reference worlds are not edited).

| Id | Criterion | Worlds | Written in | Runs |
| --- | --- | --- | --- | --- |
| **AC-Na-1** (AC-S35-4) | `sleep` at a forbidden place → `PermissionDenied`; `sleep` while asleep → `Busy`; `wake` while awake → `NoSupportedInteraction`; `for: 5` and `for: 841` → `sleep-for-out-of-bound` naming "15 … 840" (hostel: `for: 601` naming "15 … 600"); each records no event and leaves `World::snapshot()` equal to the snapshot before the request (the step's
"revision equal"; the consumption `eat.rs` "writes nothing" pattern) | both | C2 | C2 (its own test) |
| **AC-Na-2** | SD-Na-3's table, every cell | flat-and-park (+ the classes) | C2 | C2 |
| **AC-Na-3** | The four endings: alarm at exactly `since + for × 60` with `slept = for × 60`; `wake` → `asked`; `talk` to a sleeper → `addressed` (flat-and-park) and **no** `woke` (hostel, `woken_by_talk: false`); a sleeper's `walk-to` → `woke acted` at the instant of their first `arrived`; a sleeper's `talk` → `acted`; after each, `Asleep` absent and no `sleep` process running | both | C2, C3 | C2, C3 |
| **AC-Na-4** (AC-S35-6 asleep half) | Over one simulated day with ana asleep in `flat`: every observation of a person in `flat` contains ana's `asleep` record; no observation of a person in `street` or `park` does; `alarm` appears only in ana's own; `sleep-terms` only in each person's own; the `fell-asleep` payload has exactly the keys `person` and `place`, and `alarm-set` is in ana's perceived facts and in nobody else's | both | C4 | C4 |
| **AC-Na-5** (AC-S35-7, property) | A seeded script (test code, seed 7 and 42) drives every person in each world: every 15 world minutes each picks one of sleep(for drawn from the bounds) / wake / talk to someone present / walk to another place / nothing. Over the run, per person: `fell-asleep` and `woke` strictly alternate starting with `fell-asleep`; every `woke.slept` equals `woke.at − fell-asleep.at`; every `woke acted` is at the instant of an `arrived` or `spoke` of that person; `Asleep` is present exactly between the pair at each sampled instant; the number of running `sleep` processes equals the number of sleepers. Default test: 3 days per world and seed; `#[ignore]`d milestone test: 30 days | both | C6 | 3-day in C6; 30-day at the milestone |
| **AC-Na-6** | Kill and resume: a world saved mid-sleep and resumed gives the same `woke` (instant, payload) as the uninterrupted world | flat-and-park | C6 | C6 |
| **AC-Na-7** (pack-level `AC-2`) | flat-and-park without `sleep` runs the same scripted walks and talks; every other pack's facts equal those of the world with it, minus `sleep`'s own (the employment `removable.rs` comparison) | flat-and-park | C5 | C5 |
| **AC-Na-8** | Loader: `configure/sleep.yaml` in a world that does not enable `sleep` refused by name; `longest_minutes: 900` refused at its line and column; a rule naming `wake` refused as an undeclared action; a class the world does not define refused (`ClassUndefined`); the scratch Social Café and Lakeside with `sleep` enabled load and run 1 day | social-cafe copy, lakeside copy | C5 | C5 |
| **AC-Na-9** (I-Na-5) | Reference worlds unchanged: `scripts/baselines.txt` digests of all four reference worlds do not move | all four | — | milestone (the long parity record) |
| **AC-Na-10** (mutations, at the milestone) | **M-Na-1** `asleep` disclosed with `alarm` to every observer → AC-Na-4 fails; **M-Na-2** the `arrived` reaction removed → AC-Na-3 and AC-Na-5 fail; **M-Na-3** SD-Na-3's `Default(false)` arm treated as the tag rule → AC-Na-2 fails | — | — | milestone |

---

## 6. Test ownership

| Failure class | Owner |
| --- | --- |
| `ruling` precedence (rule / default / unconfigured) | `sdk/rust/src/interactions/tests.rs` unit tests (C1) |
| validation order and refusals; where-one-may-sleep | `systems/sleep/tests/{refusals,where}.rs` (C2) |
| endings, process lifecycle | `systems/sleep/tests/endings.rs` (C2, C3) |
| disclosure audiences, offers | `systems/sleep/tests/perception.rs` (C4) |
| loader refusals, real reference-world copies | `worldpack/tests/refusals.rs`, `tools/cli/tests/sleep_sections.rs` (C5) |
| removal at pack level | `systems/sleep/tests/removable.rs` (C5) |
| integrity property, kill/resume | `systems/sleep/tests/integrity.rs` (C6) |
| real-LLM Gate | **N/A**: no model; controllers are N-c |
| platforms, baselines | CI `fast` per push; the full matrix and the long parity record at the milestone (charter SPEED FIRST, R-LOAD-1) |

**Speed rule for this PR.** Per commit: `cargo build -p <crate>`, the commit's own new tests, `cargo clippy -p <crate>
--all-targets -- -D warnings`, through `cast-slots/with-slot.sh`. Per push: `fast`. Not per PR: the 30-day property
run, the baselines record, the three-platform run and the mutations — they run once at the milestone, batched with
N-b … N-d.

---

## 7. Baselines

Nothing moves: no reference world enables `sleep` in this PR (I-Na-5). The generated reference
(`docs/reference/{sections,interaction-list,configure}.md` and `docs/reference/schema/`) changes in C7, and a test
that prints the build's available systems (a refusal listing them, `worldpack/tests/refusals.rs`) gains `sleep`;
C5 audits each such literal and updates it.

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Q-Na-1 not accepted (the SDK lookup) | fall back to option (b) of SD-Na-3: N-d writes `configure/sleep.yaml` and `classes.yaml` for each enabling world; the pack's code changes in one function |
| A cascade wakes one sleeper twice | the reaction guard (SD-Na-4) and AC-Na-5's strict alternation |
| The alarm leaks to the room | `fell-asleep` carries none; `alarm-set` is sleeper-only; AC-Na-4 checks both |
| Rule chatter wakes every sleeper at night | not this PR: N-c's politeness band; `woken_by_talk` per world |

---

## 9. Commit plan

Each commit tracks **Implementation**, **Validation** and **Review** separately. `[ ]` until done with evidence; `N/A`
with an audited reason.

### C0 — Freeze record and base audit (Markdown only)

1. **Goal.** Record the freeze and the base; re-check §2's anchors on the base.
2. **Scope.** This file's header and §13. No code.
3. **Implementation plan.**
   - [ ] Record the base commit and the freeze reference.
   - [ ] Re-confirm F-Na-1 (the three line ranges in `lookup.rs`, `resolve.rs`) and F-Na-2 on the base.
   - [ ] Confirm DOC-S35 (#278) is merged and `ARC-283` reads as cited.
4. **Validation plan.** Reading only.
5. **Acceptance.** Every §2 anchor holds or its change is recorded.
6. **Failure and edge cases.** An SDK that already exposes a per-rule lookup: use it, drop C1, record.
7. **Commands.** `git log -1 origin/main`; `grep -n` on the cited files.
8. **Commit boundary.** Markdown only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C1 — SDK: `interactions::ruling`

1. **Goal.** Let a pack tell "a rule said so" from "nothing said so" (SD-Na-3).
2. **Scope.** `sdk/rust/src/interactions/resolve.rs` (`Resolution::rule_effect` returning the matching rule's
   effect, if any, beside `permits`, which is rewritten on top of it), `lookup.rs` (`pub enum Ruling`, `pub fn
   ruling`), `mod.rs` re-exports, `tests.rs`. Non-goals: no change to `permits`'s result, to `Resolution`'s fields or
   serialization, to any pack.
3. **Implementation plan.**
   - [ ] `Resolution::rule_effect(classes, action, subjects) -> Option<Effect>`; `permits` = `rule_effect.unwrap_or(default)`.
   - [ ] `Ruling { Unconfigured, Rule(bool), Default(bool) }` and `ruling::<S>`.
4. **Validation plan.** Unit: unconfigured → `Unconfigured`; a matching rule → `Rule`; two equal-specificity rules,
   one forbid → `Rule(false)`; no match with `default: forbid` → `Default(false)`; no match, unsaid → `Default(true)`;
   the existing `permits` tests unchanged and green.
5. **Acceptance.** `cargo test -p mineworld-sdk` green; no other crate changes.
6. **Failure and edge cases.** A region: `ruling` uses the region's resolution like `permits`.
7. **Commands.** `with-slot.sh cargo test -p mineworld-sdk --lib interactions`; clippy for the crate.
8. **Commit boundary.** SDK only.

- [ ] Implementation · [ ] Validation · [ ] Review (`permits` callers see identical answers)

### C2 — `systems/sleep`: the pack, `sleep` and `wake`, the alarm

1. **Goal.** The contract of SD-Na-1, validation SD-Na-2, where SD-Na-3, the process and the alarm ending of SD-Na-4,
   the section SD-Na-5.
2. **Scope.** New `systems/sleep/{Cargo.toml,src/lib.rs,action.rs,event.rs,component.rs,process.rs,interactions.rs,
   place.rs,system.rs,codec.rs,error.rs}`; workspace `Cargo.toml` member; `tests/support/mod.rs` (both test worlds,
   built with the relationships `configured` helper for hostel's section); `tests/{refusals,where,endings}.rs`.
   Dependencies: contracts, kernel, sdk, authoring (classes in tests), presence (registry and `Arrived`),
   conversation (types only, `Spoke`), serde, serde_json, schemars.
3. **Implementation plan.**
   - [ ] Types of SD-Na-1; `SleepSystem` with `SystemPack` (`interactions!()`), `System` (`declaration` depending on
         presence, owning `Asleep`, `SleepTerms` and the IL component, emitting and subscribing to its facts),
         `install`.
   - [ ] `validate` and `resolve` (SD-Na-2); `place::allowed` (SD-Na-3, the one place `SLEEPING_TAG` is read).
   - [ ] `react` for `alarm-set` and `woke`; `wake` for the alarm.
4. **Validation plan.** AC-Na-1, AC-Na-2, AC-Na-3 (alarm and `asked` rows) on both worlds.
5. **Acceptance.** Those tests pass; the crate builds with `-D warnings`.
6. **Failure and edge cases.** §4.7 rows "for at a bound", "longest 60"; a destroyed sleeper at the alarm.
7. **Commands.** `with-slot.sh cargo test -p mineworld-sleep --test refusals --test where --test endings`; clippy.
8. **Commit boundary.** The new crate only (plus its workspace line).

- [ ] Implementation · [ ] Validation · [ ] Review (no world-specific word but the tag; refusal order equals §4.2)

### C3 — The other endings: being addressed and acting

1. **Goal.** SD-Na-4's `spoke` and `arrived` reactions; `woken_by_talk`.
2. **Scope.** `systems/sleep/src/system.rs` (`react` arms, the guard), `tests/endings.rs` (new cases).
3. **Implementation plan.** - [ ] `spoke` arm (listener / speaker); - [ ] `arrived` arm (not genesis);
   - [ ] the per-cascade guard; - [ ] the alarm-and-`wake`-at-one-instant test.
4. **Validation plan.** AC-Na-3 remaining rows on both worlds; F-Na-4 (a shove) recorded, not tested (bodies is not in
   the test worlds).
5. **Acceptance.** Every ending row passes; exactly one `woke` per sleep in each test.
6. **Failure and edge cases.** Both talkers asleep; a sleeper who talks to themself is refused by conversation, no fact.
7. **Commands.** `with-slot.sh cargo test -p mineworld-sleep --test endings`.
8. **Commit boundary.** Reactions only.

- [ ] Implementation · [ ] Validation · [ ] Review (`movement` and `conversation` untouched — I-Na-1)

### C4 — Disclosure and offers

1. **Goal.** SD-Na-6; the lane lead's `for` bounds requirement (F-Na-2).
2. **Scope.** `systems/sleep/src/{system.rs,offer.rs}` (`PerceptionProvider`), `tests/perception.rs`.
3. **Implementation plan.** - [ ] `asleep` record with holder-only `alarm`; - [ ] `sleep-terms`; - [ ] offers with
   refusals.
4. **Validation plan.** AC-Na-4 on both worlds; the offer at `park` refused `PermissionDenied` while `validate` gives
   the same; asleep → `wake` complete and `sleep` refused `Busy`.
5. **Acceptance.** Tests pass; no record names anyone but its subject.
6. **Failure and edge cases.** An observer who is not a Person (a test with no observer person): nothing.
7. **Commands.** `with-slot.sh cargo test -p mineworld-sleep --test perception`.
8. **Commit boundary.** Perception only.

- [ ] Implementation · [ ] Validation · [ ] Review (offer and dispatch answer the same, in the same order)

### C5 — Installed, loaded, removable

1. **Goal.** The build offers `sleep`; the real loader decodes its section; removal at pack level.
2. **Scope.** `systems/installed/{Cargo.toml,src/lib.rs}` (`Sleep => mineworld_sleep::SleepSystem`, after
   `Appearance`); `Cargo.lock`; `worldpack/tests/refusals.rs` (AC-Na-8 rows that need no world copy); new
   `tools/cli/tests/sleep_sections.rs` (scratch copies of `worlds/social-cafe` and `worlds/lakeside`, the
   `forgetting_sections.rs` pattern); `systems/sleep/tests/removable.rs`; any literal list of available systems the
   audit finds.
3. **Implementation plan.** - [ ] the two install lines; - [ ] the loader tests; - [ ] the removal test.
4. **Validation plan.** AC-Na-7, AC-Na-8; `cargo test -p mineworld-installed-systems`.
5. **Acceptance.** Tests pass; `mineworld validate` accepts both scratch copies.
6. **Failure and edge cases.** A copy that moves a reference world file: forbidden; copies live in a temp dir only.
7. **Commands.** `with-slot.sh cargo test -p mineworld-installed-systems -p mineworld-sleep --test removable`;
   `with-slot.sh cargo test -p mineworld-worldpack --test refusals`; `with-slot.sh cargo test -p mineworld-cli --test
   sleep_sections`.
8. **Commit boundary.** Installation and loader tests only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C6 — Integrity property and kill/resume

1. **Goal.** AC-Na-5 and AC-Na-6.
2. **Scope.** `systems/sleep/tests/integrity.rs` (the seeded script, the checks, the `#[ignore]` 30-day variant);
   `mineworld-persistence` and `mineworld-test-support` as dev-dependencies (the employment `persisted.rs` pattern).
3. **Implementation plan.** - [ ] the scripted driver (test-local seeded draws); - [ ] the five checks; - [ ] save /
   resume mid-sleep.
4. **Validation plan.** The 3-day default for both worlds and both seeds, run locally; the 30-day `#[ignore]` run is
   listed for the milestone batch.
5. **Acceptance.** 3-day runs pass; the test asserts literal minimum counts (at least one sleep, one `addressed`,
   one `acted` per world and seed) so a vacuous pass fails.
6. **Failure and edge cases.** A script that never sleeps at a permitted place: the minimum-count assertion catches it.
7. **Commands.** `with-slot.sh cargo test -p mineworld-sleep --test integrity`.
8. **Commit boundary.** Tests only.

- [ ] Implementation · [ ] Validation · [ ] Review (the test's expectations are literals, never read from the run)

### C7 — Docs, records, reference

1. **Goal.** Documentation law (`CLAUDE.md` §2).
2. **Scope.** `systems/sleep/README.md`; `docs/MODULE_SPEC.md` §4.2 `sleep` entry (replace the reference-list
   sentence with SD-Na-3's rule; `wake` not a section action; `sleep-terms`); a dated note on
   `docs/decisions/ARC-283.md` (how `residential` resolves, F-Na-1 → SD-Na-3) and on `docs/decisions/ARC-63.md`
   (item 8 gains `ruling`); `docs/reference/` regenerated; this file's §13.
3. **Implementation plan.** - [ ] the README (short, human); - [ ] the two notes; - [ ] `mineworld reference` and
   `reference --check`.
4. **Validation plan.** `python3 scripts/check_decision_ids.py`; `with-slot.sh cargo test -p mineworld-cli --test
   reference`.
5. **Acceptance.** Checks green; no record id added.
6. **Failure and edge cases.** None.
7. **Commands.** as above.
8. **Commit boundary.** Docs and generated reference only.

- [ ] Implementation · [ ] Validation · [ ] Review

**Size.** About 1.1 – 1.3 kLOC: SDK ~70, the pack ~550, tests ~550, loader tests ~120, docs ~120. **M**.
Implementer: Opus for C1–C3 (the SDK seam and the reactions), Haiku acceptable for C4–C7 from this plan.

---

## 10. Requirements on other work

- N-b decodes `FellAsleep`, `Woke` and `WokeReason` (public types; mode switches on both).
- N-c reads `asleep`, `sleep-terms` and the `sleep`/`wake` offers; it never hard-codes 15 … 840.
- N-d: Market Town's homes already carry `residential`; enabling `sleep` needs no `configure/sleep.yaml` there.
- N-e: renders `asleep` (Zz, pose), `alarm` for oneself, the `Wake up` button from the complete `wake` offer.

---

## 11. Execution contract (proposed; filled at freeze)

```text
PROJECT / PR:            MineWorld mvp0, PR N-a — the sleep System Pack
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s35-na-sleep.md
RELATED / BINDING DOCS:  step-35-needs-and-sleep.md; docs/decisions/ARC-283.md, ARC-284.md, ARC-286.md, ARC-63.md,
                         ARC-64.md, ARC-31.md, ARC-28.md; docs/MODULE_SPEC.md §4.2; CLAUDE.md; the lane charter
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s35-na, its own, held by one session
BRANCH:                  mvp0/pr-s35-na, from origin/main
START CONDITION:         this design FROZEN AND DOC-S35 (#278) merged
IMPLEMENTATION BASE:     origin/main at C0; C0 records the commit
APPROVED SCOPE:          §3.1, as frozen
FROZEN INVARIANTS:       I-Na-1 … I-Na-8; SD-Na-1 … SD-Na-6; §5 thresholds
SEQUENCE:                C0 … C7
ALLOWED COMMANDS:        cargo * (heavy ones through cast-slots/with-slot.sh); git; gh (never merge);
                         python3 scripts/*; target/*/mineworld *; mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; perl -pi; curl/wget; editing kernel/, contracts/, persistence/, server/,
                         systems/{movement,conversation,presence}, any file under worlds/; a local full-workspace
                         test run (R-LOAD-1)
MATERIAL STOPS:          a change to SD-Na-1's contract or SD-Na-3's rule; Q-Na-1 answered "no"; a reference-world
                         digest moving; any §5 FAIL at the milestone (fixed in the milestone batch, never by retuning
                         a default)
PLATFORMS:               Linux, macOS, Windows (CI at the milestone)
VALIDATION BUDGET:       per commit: build, the commit's tests, clippy for the touched crates; per push: `fast`
LIVE DOCUMENTATION:      this document (§13)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s35-na.md
ENDPOINT AUTHORITY:      implementation, semantic commits, branch push, PR creation/update, CI repair: authorized
                         (source: the lane charter, operator 2026-10-18, SPEED FIRST); merge: the lane lead after
                         `fast` is green, per the charter's SPEED FIRST block
POST-MERGE SYNC OWNER:   lane lead: step-35 progress line; coordinator: overall.md, progress page; this session: §13
STOP CONDITION:          READY FOR LANE-LEAD MERGE (`fast` green on the exact head)
```

---

## 12. Questions (none operator-material)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-Na-1** (coordinator) | Accept SD-Na-3 — the world's list decides when a rule matches or it says `default: forbid`; otherwise the pack's tag rule (`residential`) — with one additive SDK lookup `interactions::ruling`, instead of the step's literal reference list (F-Na-1)? | **Yes.** Same observable rules as step-35 §4.2 and `ARC-283`; works in an unconfigured world; nothing world-specific; no encoded form changes; recorded by notes on `ARC-283` and `ARC-63` (no new record, so no ARC-288) |

---

## 13. Ledger (live during implementation)

### Evidence

- 2026-10-18 (design session): audit on `origin/main @ a8bba658`; DOC-S35's records read from
  `origin/docs/s35-specs` (PR #278).

### Deviations and findings

- F-Na-1 … F-Na-4 (§2.2), all from the design audit.
- 2026-10-18, lane lead's ruling on the former Q-Na-2: follow step-35 §4.1, `fell-asleep { person, place }` without
  the alarm. Because `Asleep` is a projection of the pack's own facts and replay must rebuild it, the alarm travels in
  a second fact, `alarm-set`, perceived by the sleeper only (SD-Na-1, SD-Na-4).
