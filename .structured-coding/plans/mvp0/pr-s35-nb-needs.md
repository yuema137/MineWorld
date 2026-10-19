# PR N-b — The `needs` System Pack: hunger and tiredness, stored lazily, restored by meals and sleep

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes)

```text
Design revision:        revision 1 (2026-10-18), design session
Approved by / evidence: the operator, 2026-10-18, in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes
Implementation base:    origin/main at C0 (N-a merged), or stacked on mvp0/pr-s35-na once its C2 is pushed;
                        C0 records which
Start condition:        this design FROZEN by the operator AND DOC-S35 (#278) merged AND N-a's event types
                        available (merged, or stacked)
Execution contract:     §11
Lifecycle:              DESIGN FROZEN 2026-10-18
```

Follows the frozen step-35 in every rule and number. It does **not** follow DOC-S35's section *shape* on item files
in one respect, because the audit found that a pack owns exactly one section (finding F-Nb-1): Q-Nb-1 asks the
operator to accept `needs: { nourishment: { hunger: 600 } }` on an item file instead of a top-level
`nourishment: { hunger: 600 }`. Two small seam additions in the loader (Q-Nb-2) make the step's load-time refusals
reachable. Both questions carry recommendations; nothing else is new.

**Effort:** `mvp0`, release scope. **Parents:** [`step-35-needs-and-sleep.md`](step-35-needs-and-sleep.md) §§3, 5, 6.1,
7, 9 (AC-S35-3, AC-S35-5, AC-S35-6 needs half, AC-S35-13 "decay twice"), 10.1 row N-b, 11; records
[`ARC-284`](../../../docs/decisions/ARC-284.md) (needs), [`ARC-285`](../../../docs/decisions/ARC-285.md)
(nourishment), [`ARC-283`](../../../docs/decisions/ARC-283.md) (sleep, whose facts this pack hears),
[`ARC-286`](../../../docs/decisions/ARC-286.md) (controllers, the consumers of the disclosure), written by DOC-S35
(PR #278); [`pr-s35-na-sleep.md`](pr-s35-na-sleep.md).
**Consumed by:** N-c (rule and motivated read `needs`), N-d (Market Town content), N-e (HUD rows, agent and LM lines).

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-s35-na-nb`, branch
`docs/pr-design-s35-na-nb`, audit base `origin/main @ a8bba658` (2026-10-18).

**Binding:** [`CLAUDE.md`](../../../CLAUDE.md) §2.2, §4 rules 1–3, 5, 7, 8, 10, 11, 13; the lane charter's SPEED FIRST
and "no world-specific hard-coding" blocks; `INV-13`; `ARC-15` (authored content becomes state only as a genesis
fact), `ARC-23`, `ARC-25`, `ARC-28`, `ARC-31` (sections), `ARC-36` (sections on item files), `ARC-61`
(configurations, `requires`), `ARC-63`/`ARC-64` (the Interaction List), `ARC-86` (lazy decay), `ARC-90`.

---

## 1. Goal, in one paragraph

Nobody is ever hungry or tired (F-S35-1). This PR adds a System Pack, `needs`, that gives each person whose file says
so two integer **deficits**, hunger and tiredness, from 0 (sated, rested) to 1 000 (famished, exhausted). They rise
with world time and fall when the person eats (inventory's `items-consumed`) or sleeps (sleep's `fell-asleep` …
`woke`). Nothing is written while they rise: the pack stores a level, the instant it was true and whether the person
was awake, and computes the current level wherever it is read, as `relationships` forgets (`ARC-86`). So over thirty
days the pack states exactly one fact per person, at genesis. The person alone sees their needs, in the pack's words
(`hungry`, `sleepy`), with the rates in force and how much each food they carry would nourish them, so every
controller, prompt and HUD reads the same numbers and words. Every number is data: the rates and thresholds in
`configure/needs.yaml`, the starting levels in person files, the nourishment in item files.

---

## 2. Audit (`origin/main @ a8bba658`, 2026-10-18)

### 2.1 What the pack is built from

| Thing | Where | What this PR takes from it |
| --- | --- | --- |
| Lazy decay | `systems/relationships/src/decay.rs` (pure curves, integer only, "applied lazily where a value is read"); `systems/relationships/src/component.rs` `RelationshipValues::reading`; `kernel/src/view.rs:57-62` (`WorldRead::at` "so a system that keeps lazily derived state … reads it as of that instant without consulting any clock") | the `level.rs` module; reads at `WorldRead::at` |
| A section with two forms | `systems/bodies/src/section.rs` (`body:` carried by place and item files, one type with a place form and an object form; the wrong form on a file refused when seeded with `bodies-section-kind`) | SD-Nb-2, the answer to F-Nb-1 |
| One section per pack | `sdk/rust/src/pack.rs:50-52` (`const SECTION: Option<SectionOwner>`), `:347-349` ("a pack owns one section"); `authoring/src/section.rs:173-212` (`AuthoredSection` implemented on the system type, one `SECTION`) | finding F-Nb-1 |
| Sections on item files | `systems/item/src/section.rs` (`item:` on `ContentKind::Item`); `systems/consumption/tests/support/mod.rs` header ("items' sections first, then people's", `ARC-36` item 7) | the item form is seeded before people |
| A person section with defaults and a holder | `systems/inventory/src/section.rs` (`holdings:`; subject must be a holder) | the person form |
| Configuration `requires` | `authoring/src/configuration.rs:119,166,222`; `worldpack/src/configure.rs:315-329` `check_requires` (only for a written file); `worldpack/src/error.rs:619-630` `ConfigurationRequiresSystem`; FG-a's use, `systems/relationships/src/interactions.rs:151-163` | SD-Nb-5; finding F-Nb-2 |
| An unconfigured pack | `worldpack/src/configure.rs:160-181` (`read` handles only the keys in `configure:`); `sdk/rust/src/interactions/lookup.rs:1-6` ("With no component … every lookup returns the compiled default") | finding F-Nb-2 |
| Section refusals | `worldpack/src/error.rs:380-470` (`ContentNeedsASystem`, `SectionNotCarriedHere`, `SectionRefusedByOwner`) | the refusal wording of AC-Nb-1 |
| Parameters with bounds, field roles | `sdk/rust/src/interactions/section.rs` `parameters!`, `ParameterBound` (a range or a named choice with `schema`); `FIELD_ROLES` (`mod.rs:86-94`); `parameters_unplaced` (`lookup.rs:199-214`) | `NeedsParameters`, the `restore` and `thresholds` bounds |
| Facts it hears | `systems/inventory/src/event.rs:120-160` `ItemsConsumed { holder, item, count }` (`Visibility::Participants`, stated by consumption through `consume`); N-a's `FellAsleep`, `Woke` | the reactions |
| Hearing without a dependency | `systems/economy/src/system.rs:70-73` (subscribes to employment's `wage-due` with no registry dependency, `ARC-28`) | `needs` depends on nothing in the registry |
| Holdings, item kinds | `systems/inventory/src/component.rs` `Holdings::held()`; `systems/item/src/component.rs` `ItemKind::category` | the disclosed `nourishment` list |
| Holder-only disclosure | `systems/employment/src/system.rs` `discloses` | the `needs` record |
| Test worlds, configured sections | `systems/relationships/tests/support/mod.rs:59-70` (`configured(section, classes)`); `tools/cli/tests/forgetting_sections.rs` (scratch copies of a reference world, the real loader) | the two in-crate worlds and the loader fixtures |
| Pack-level removal | `systems/employment/tests/removable.rs` (the "shared fact" comparison) | AC-Nb-6 |
| Kill and resume | `systems/employment/tests/persisted.rs` (`mineworld-persistence`, `mineworld-test-support`) | AC-Nb-4 |

### 2.2 Findings

- **F-Nb-1 — a pack owns one section.** `ARC-284`/`ARC-285` and DOC-S35's `MODULE_SPEC.md` §4.1 give `needs` two
  sections, `needs:` on people and `nourishment:` on items. The seam does not allow it: `SystemPack::SECTION` is one
  `Option<SectionOwner>`, `AuthoredSection` is implemented once on the system type, and the SDK says "a pack owns one
  section". Two ways out: (a) one `needs:` section with a person form and an item form, the `bodies` precedent, no
  seam change; (b) teach the seam several sections per pack (`SECTION` becomes a slice; `owns_section!`, the catalog's
  namespace test, the loader's section lookup and the reference generator change for every pack). **This design
  takes (a)** and asks Q-Nb-1. Every rule of `ARC-285` stays: the amount lives with the item kind, only `needs` reads
  it, bounds, refusals, disclosed to nobody.
- **F-Nb-2 — two of the step's load refusals have no seam.** (i) "`nourishment` without `consumption` is refused by
  name": a section cannot say it requires a system (`AuthoredSection` has `references`, not `requires`). (ii) "the
  default `tiredness: [sleeping]` restorer is refused by name when `sleep` is not enabled": `requires` runs only for a
  written `configure/<pack>.yaml`; a world that enables `needs` and writes no file is never checked, so it would keep
  a tiredness that silently never falls — the exact failure step-35 §7 forbids. SD-Nb-5 adds both checks, generic, in
  `authoring` and `worldpack`.
- **F-Nb-3 — integer folds round.** `level + rate × Δt ÷ 3 600` truncated toward zero at each fold loses up to one
  unit per fold. Folds happen only at facts (a meal, falling asleep, waking: a few a day), so the loss is a few units a
  day, bounded and deterministic. Decided: truncate toward zero, documented, and the hand-computed tests use the same
  rule.
- **F-Nb-4 — controllers need more than the levels.** The lane lead's requirements (2026-10-18), recorded for N-c:
  the eat band picks the held kind with the highest nourishment, and the bedtime length needs `rest_per_hour` in
  force. Both are owner-known values, so the pack discloses them to the holder (SD-Nb-6). Without that, `rule` would
  read item files or hard-code 125.

---

## 3. Scope

### 3.1 In scope

1. Seam additions (SD-Nb-5): `AuthoredSection::requires` with a loader check; a loader check of an enabled but
   unconfigured Interaction List pack's default `requires`.
2. New crate `systems/needs` (`mineworld-needs`): the levels, the section in two forms, the Interaction List section,
   reactions, disclosure (§4).
3. The installed set: one line each in `systems/installed`; the workspace member.
4. Tests on two in-crate test worlds; loader tests on scratch copies of two reference worlds; pack-level removal;
   the zero-facts and kill/resume acceptance tests (written here; their 30-day runs at the milestone).
5. Docs: `systems/needs/README.md`; dated notes on `ARC-284`, `ARC-285`, `ARC-31`, `ARC-61`; `MODULE_SPEC.md` §4.1
   (the item form) and §4.2 (the disclosure fields); the generated reference.

### 3.2 Not in scope

- Controllers (N-c), content and Market Town's supply (N-d), clients and agent/LM lines (N-e).
- Thirst, hygiene, comfort; any forced consequence (Q-S35-2, Q-S35-3). S30 attributes (Q-S35-11).
- A restorer catalog (Q-S35-4): the two restorers are a closed enum.
- Changing `consumption`, `inventory`, `item` or `sleep`.

### 3.3 Invariants

- **I-Nb-1** The kernel and contracts are unchanged; `consumption`, `inventory`, `item`, `sleep` are unchanged.
- **I-Nb-2** `needs` writes only `Needs` (on people), `Nourishment` (on item kinds) and its IL component; it states
  only `needs-assigned`, `nourishment-assigned` and its configured fact, all at genesis.
- **I-Nb-3** Decay is lazy: no Process of any kind, no fact after genesis (AC-S35-5).
- **I-Nb-4** Integer arithmetic only; no randomness; time only from `WorldRead::at` and fact instants.
- **I-Nb-5** A world that does not enable `needs` is byte-identical (I-S35-5).
- **I-Nb-6** Disclosed to the holder only (`INV-13`); the nourishment of an item kind is disclosed only as part of
  its holder's own record.
- **I-Nb-7** Nothing in the crate names a world, a person, an item, a place or an hour.
- **I-Nb-8** Defaults are step-35 §5.5's, fixed before any run (`ARC-23`): hunger +120/h awake, +30/h asleep;
  tiredness +60/h awake, −125/h asleep; thresholds 400 / 600 / 900; genesis levels hunger 300, tiredness 700;
  restore `{ hunger: [eating], tiredness: [sleeping] }`.

---

## 4. Design

### 4.1 SD-Nb-1 — Contract

```text
crate        mineworld-needs (systems/needs)              System id `needs`, version 1
section      needs:                                       person form and item form (SD-Nb-2)
configure    configure/needs.yaml                         its Interaction List section (SD-Nb-4)
emits        needs-assigned { person, levels }            genesis, from a person's form; audience: the person
             nourishment-assigned { item, hunger }        genesis, from an item's form; SystemInternal
             needs-interactions-configured                genesis, its IL fact
owns         Needs { hunger: Level, tiredness: Level }    on a person; Level = { level: u16, at, mode }
             Nourishment { hunger: u16 }                  on an item kind
             needs-interactions                           the IL component (SDK)
process      none
depends on   nothing (registry)
hears        items-consumed (inventory), fell-asleep, woke (sleep)   through their owners' types (ARC-28)
discloses    needs   to its holder only (SD-Nb-6)
biography    nothing
```

```rust
pub enum Mode { Awake, Asleep }                                   // serde kebab-case
pub struct Level { level: u16, at: WorldTime, mode: Mode }
pub struct Needs { hunger: Level, tiredness: Level }              // Component "needs" v1
pub struct Nourishment { hunger: u16 }                            // Component "nourishment" v1
pub struct Levels { hunger: u16, tiredness: u16 }
pub struct NeedsAssigned { person: PersonId, levels: Levels }     // Event "needs-assigned" v1
pub struct NourishmentAssigned { item: ItemId, hunger: u16 }      // Event "nourishment-assigned" v1
pub enum Restorer { Eating, Sleeping }                            // closed set, serde kebab-case
pub enum HungerWord { Ok, Peckish, Hungry, Famished }
pub enum TirednessWord { Ok, Sleepy, Tired, Exhausted }
```

Why `nourishment-assigned` exists although step-35 §5.1 lists only `needs-assigned`: authored content becomes state
only as a genesis fact (`ARC-15`), and the nourishment of an item kind is state `needs` reads when someone eats. It
is a genesis fact, like the configured fact, so AC-S35-5's intent — nothing after genesis, nothing on a timer — is
kept exactly; AC-Nb-3 states the count precisely.

### 4.2 SD-Nb-2 — The `needs:` section, two forms (F-Nb-1)

```yaml
# people/ana.yaml — the person form
needs: {}                         # hunger 300, tiredness 700 (a person at midnight, step-35 §5.6)
needs: { hunger: 200 }            # tiredness takes its default

# items/soup.yaml — the item form
item: { category: food }
needs: { nourishment: { hunger: 600 } }
```

- One type, `AuthoredNeeds`, `#[serde(try_from = "Written")]`, the `bodies` pattern: `Written` has optional
  `hunger`, `tiredness` and `nourishment`, `deny_unknown_fields`. Mixing `nourishment` with `hunger`/`tiredness`, or
  an unknown key, is refused by the decoder at the section's line, naming the keys. Each number is `0 ..= 1 000`,
  refused at its line and column otherwise (`nourishment: { hunger: 1001 }`).
- `CARRIED_BY: [Person, Item]`. Seeding: the person form on a person → `needs-assigned`; the item form on an item →
  `nourishment-assigned`; a form on the other kind of file → `Rejection::System { code: "needs-section-kind" }`, the
  `bodies-section-kind` precedent.
- `requires` (SD-Nb-5): the item form requires `consumption` (nothing else could ever eat the item).
- Only a person with the section has needs (Q-S35-5). An item kind without the item form nourishes 0.
- A `needs:` section in a world that does not enable `needs` is refused by name by the existing seam (`ARC-31` rule 2).

### 4.3 SD-Nb-3 — Levels, lazily (`systems/needs/src/level.rs`, pure)

```text
current(l, now, r)  = clamp(l.level + rate(r, need, l.mode) × (now − l.at) ÷ 3 600, 0, 1 000)
                      integer, i64 intermediate, ÷ truncates toward zero (F-Nb-3)
rate(hunger,    awake)  = +hunger_per_hour          rate(hunger,    asleep) = +hunger_asleep_per_hour
rate(tiredness, awake)  = +tiredness_per_hour       rate(tiredness, asleep) = −rest_per_hour
                                                    … or +tiredness_per_hour when `sleeping` is not a restorer
fold(l, at, r)       = Level { level: current(l, at, r), at, mode: l.mode }
word(level, [a,b,c]) = ok < a ≤ peckish/sleepy < b ≤ hungry/tired < c ≤ famished/exhausted
```

`r` are the parameters in force for the person (`parameters_unplaced` with `actor` = the person; rates may be scoped
by an actor class). A person's classes are fixed after genesis (tags are fixed, `ARC-64`), so the rates in force for
a person never change during a run, and the stored instant plus the current parameters give the same value on replay
(`ARC-25`). `now` is `WorldRead::at` at disclosure and the fact's instant in a reaction; no host clock.

### 4.4 SD-Nb-4 — The Interaction List section (`configure/needs.yaml`)

```rust
mineworld_sdk::parameters! {
    pub struct NeedsParameters, partial NeedsPartial {
        hunger_per_hour:        u16 = 120, 10 ..= 500;
        hunger_asleep_per_hour: u16 = 30,  0 ..= 500;
        tiredness_per_hour:     u16 = 60,  10 ..= 500;
        rest_per_hour:          u16 = 125, 10 ..= 1_000;
        thresholds:  Thresholds = [400, 600, 900], StrictlyIncreasing (each in 1 ..= 1 000);
        restore:     Restore    = { hunger: [eating], tiredness: [sleeping] }, ClosedRestorers;
    }
}
ACTIONS          none (a need is a condition, not an action); a `rules:` entry is refused as undeclared
FACTS            needs-assigned: roles actor = participant 0; default Participants, narrowest Participants,
                 biography not configurable
PARAMETER_ROLES  [actor]
FIELD_ROLES      thresholds: [], restore: []      base only; refused under a role or in a region
requires(s)      eating in s's resolved base restore → consumption; sleeping → sleep
```

`Thresholds` and `Restore` are named-choice bounds (`ParameterBound` with `describe` and `schema`), so a bad value is
refused at its line and column with the pack's words: `thresholds` "three integers in 1 … 1 000, strictly
increasing"; `restore` "a list from [eating] for hunger and from [sleeping] for tiredness, each at most once".
`restore: { tiredness: [] }` states explicitly that tiredness never falls.

### 4.5 SD-Nb-5 — Two generic load checks (F-Nb-2)

1. **A section may require systems.** `mineworld_authoring::AuthoredSection` gains
   `fn requires(authored: &Self::Authored) -> Vec<SystemId> { Vec::new() }`. The loader, after it has checked that a
   section's owner is enabled, refuses a section whose `requires` names a system the world does not enable:
   `PackError::SectionRequiresSystem { subject, section, system, requires, path }` — *"items/soup.yaml gives 'soup' a
   'needs' section that needs 'consumption', which world.yaml does not enable"*. Every existing section returns
   nothing, so no other world or refusal changes.
2. **An unconfigured pack's defaults may require systems.** For every enabled pack with an Interaction List section
   that the world does not list in `configure:`, the loader asks `S::requires(&Section::default())` (through a new
   `SectionDecl::requires_default: fn() -> Vec<SystemId>` filled by `interactions!()`), and refuses a missing system:
   `PackError::DefaultConfigurationRequiresSystem { by, requires, path: world.yaml }` — *"world.yaml enables 'needs',
   whose default configuration needs 'sleep', which this world does not enable: enable 'sleep', or write
   configure/needs.yaml saying what restores tiredness"*. Every existing pack's default requires nothing (FG-a's default
   curve is `none`), so no other world changes.

Both are generic; neither names `needs`. Recorded as dated notes on `ARC-31` and `ARC-61` (no new record: they
extend the existing `requires` idea from configurations to sections and to defaults, so ARC-288 is not needed).

### 4.6 SD-Nb-6 — Reactions and disclosure

Reactions (each folds first, at the fact's instant, then applies, then stores `(new level, instant, mode)`):

| Fact (owner) | Condition | Effect |
| --- | --- | --- |
| `needs-assigned` (own) | — | insert `Needs` with both levels at the genesis instant, `mode: awake` |
| `nourishment-assigned` (own) | — | insert `Nourishment` on the item kind |
| `items-consumed` (inventory) | holder has `Needs`; `eating` ∈ restore.hunger | hunger := max(0, current − `Nourishment.hunger` × count); an item without `Nourishment` changes nothing and stores nothing |
| `fell-asleep` (sleep) | person has `Needs` | both needs folded; mode := asleep |
| `woke` (sleep) | person has `Needs` | both needs folded; mode := awake |

A fact that changes nothing writes nothing (no component write for a person without `Needs`, or for a food that
nourishes 0), so a world where nobody eats nourishing food keeps the stored bytes untouched.

Disclosure, `observer == subject` only, at `WorldRead::at`:

```text
needs {
  hunger: u16, tiredness: u16,                                  current levels
  words: { hunger: HungerWord, tiredness: TirednessWord },
  asleep: bool,                                                 the mode needs holds (from sleep's facts)
  rates: { hunger_per_hour, hunger_asleep_per_hour,             in force for the holder (F-Nb-4)
           tiredness_per_hour, rest_per_hour },
  restore: { hunger: [..], tiredness: [..] },
  nourishment: [{ item: ItemId, hunger: u16 }]                  every kind the holder carries that has
}                                                               `Nourishment`, in ItemId order (F-Nb-4)
```

The record keeps the component type `needs`; the stored `Needs` is never disclosed raw (the LE `agenda` precedent: a
disclosed view of owned state). `thresholds` are not disclosed: the words are (step-35 §5.4).

### 4.7 Edge cases, decided

| Case | Behaviour |
| --- | --- |
| Eating while asleep (a controller asks `eat` for a sleeper) | hunger falls as for anyone; the mode stays asleep; `sleep` does not wake them (N-a §4.7) |
| `items-consumed` for an organization or an item holder | ignored (no `Needs`) |
| A need at 1 000 keeps rising | clamped at 1 000; nothing is stated |
| A sleep of 14 h | tiredness falls 1 750 → clamped at 0 |
| `restore: { tiredness: [] }` in a world with `sleep` | asleep, tiredness rises at `tiredness_per_hour`; hunger uses the asleep rate |
| Save and resume | `Needs` is persisted state; current values equal (AC-Nb-4) |
| Two `items-consumed` at one instant | two folds at the same instant; the second adds nothing for time (Δt = 0) |
| A `needs:` person form on an organization file | refused by the seam (`CARRIED_BY` is person and item) |

---

## 5. Acceptance criteria (fixed now, before any run; `ARC-23`)

Two test-only worlds, in-crate (`systems/needs/tests/support`): **`flat-and-cafe`** — `flat` (tag `residential`) and
`cafe`; `ana` (`needs: {}`), `ben` (`needs: { hunger: 900 }`), `cy` (no section); kinds `soup` (food, `needs: {
nourishment: { hunger: 600 } }`), `apple` (food, 200), `coffee` (drink, no form); real `presence`, `movement`,
`item`, `inventory`, `consumption`, `sleep`, `needs`; and **`night-shift`** — other places and people, classes
`night` (person tag `night`), `configure/needs.yaml` with `{ actor: night, hunger_per_hour: 90, rest_per_hour: 200 }`
and `thresholds: [300, 500, 800]`. The loader criteria use scratch copies of `worlds/social-cafe` and
`worlds/lakeside` (test-only copies) with `needs` (and, per case, `sleep`, `consumption`) appended to `systems:`.

| Id | Criterion | Worlds | Written in | Runs |
| --- | --- | --- | --- | --- |
| **AC-Nb-1** (AC-S35-3) | Refused at load, by name and position, each on both scratch copies: a `needs:` section without `needs` enabled (by name); `needs: { nourishment: … }` without `consumption` (by name, `SectionRequiresSystem`); `needs` enabled without `sleep` and without `configure/needs.yaml` (`DefaultConfigurationRequiresSystem`, naming `sleep`); `restore: { tiredness: [sleeping] }` written without `sleep` (`ConfigurationRequiresSystem`); `thresholds: [600, 400, 900]` (line and column); `rest_per_hour: 5` (line and column); `nourishment: { hunger: 1001 }` (line and column); `thresholds` under `actor:` (line and column); `needs: { hunger: 1, nourishment: { hunger: 1 } }` (line). And accepted: the same copy with `restore: { tiredness: [] }` and no `sleep` | social-cafe copy, lakeside copy | C1, C6 | C6 |
| **AC-Nb-2** | Hand-computed values: in flat-and-cafe, ana at 00:00 (300, 700); at 05:00 awake (900, 1 000); eats one soup at 05:00 → hunger 300; falls asleep at 06:00 → reading at 14:00 gives hunger 300 + 120 + 30 × 8 = 660 and tiredness 1 000 − 125 × 8 = 0; words `hungry` / `ok`. In night-shift, the same script for a `night` person with its rates and thresholds gives the literal values of the test's table | both | C2, C5 | C2 (pure), C5 (in a world) |
| **AC-Nb-3** (AC-S35-5) | Over the run, `needs` states exactly one `needs-assigned` per person with the section (2 in flat-and-cafe), one `nourishment-assigned` per item kind with the item form (2), one configured fact when configured (night-shift), and **no fact after genesis**; `WorldRead::processes()` yields no process owned by `needs` at any sampled instant. Default test: 2 days with the seeded script below; `#[ignore]`d milestone test: 30 days | both | C7 | 2-day in C7; 30-day at the milestone |
| **AC-Nb-4** | Kill and resume: saved at 05:30 and resumed, every disclosed level at 06:00, 12:00 and 23:00 equals the uninterrupted world's | flat-and-cafe | C7 | C7 |
| **AC-Nb-5** (AC-S35-6 needs half) | Over one day with all three people in `cafe`: ben's observations never contain ana's `needs`; ana's contain hers only; cy's contain no `needs` record at all; `nourishment` lists only kinds ana holds | both | C5 | C5 |
| **AC-Nb-6** (pack-level `AC-2`) | flat-and-cafe without `needs` (and without the sections) runs the same scripted meals and sleeps; every other pack's facts equal the world with it, minus `needs`' own | flat-and-cafe | C6 | C6 |
| **AC-Nb-7** (I-Nb-5) | Reference worlds unchanged: no `scripts/baselines.txt` digest moves | all four | — | milestone |
| **AC-Nb-8** (AC-S35-13, mutations at the milestone) | **M-Nb-1** "decay twice": `fold` adds the elapsed rise twice → AC-Nb-2 and AC-Nb-4 fail; **M-Nb-2** `needs` disclosed to every observer → AC-Nb-5 fails; **M-Nb-3** a reaction stores a level for a food that nourishes 0 → AC-Nb-3's byte check of `Needs` fails | — | — | milestone |

The seeded script for AC-Nb-3: test code, seeds 7 and 42; every 15 world minutes each person with a held food may
`eat`, may `sleep` at `flat` for 6–9 h, may walk; it is driven through the real dispatch, so `consumption` and
`sleep` decide.

---

## 6. Test ownership

| Failure class | Owner |
| --- | --- |
| `current`, `fold`, `word` arithmetic and rounding | `systems/needs/src/level.rs` unit tests against a literal table (C2) |
| section forms, bounds, kinds | `systems/needs/tests/section.rs` (C3) |
| IL parameters, bounds, field roles, `requires` | `systems/needs/tests/configuration.rs` (C4) |
| seam checks (section `requires`, default `requires`) | `worldpack/tests/requirements.rs` (exists; C1 adds cases with a stub pack); `tools/cli/tests/needs_sections.rs` (C6, real reference-world copies) |
| reactions, disclosure audience | `systems/needs/tests/{reactions,perception}.rs` (C5) |
| removal, zero facts, kill/resume | `systems/needs/tests/{removable,lazy}.rs` (C6, C7) |
| real-LLM Gate | **N/A**: no model |
| platforms, baselines | CI `fast` per push; the full matrix and the long parity record at the milestone |

**Speed rule.** Per commit: build, the commit's own tests, clippy for the touched crates, through `with-slot.sh`. Per
push: `fast`. At the milestone, batched with N-a, N-c, N-d: the 30-day runs, the baselines record, three platforms,
the mutations.

---

## 7. Baselines

Nothing moves: no reference world enables `needs` here (I-Nb-5). The generated reference changes in C8. The seam
checks of SD-Nb-5 add no refusal to any existing world (C1 runs `mineworld validate` on all four reference worlds).

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Q-Nb-1 answered (b) | C3 becomes a seam PR of its own first (~300 lines across `sdk`, `authoring`, `worldpack`, `tools/cli` reference); everything else in this design is unchanged |
| Rounding drift visible in long runs | F-Nb-3: bounded by a few units a day; AC-S35-8's bounds are wide; a milestone finding, never a retune |
| `nourishment` shown to the holder lets an LM read numbers | intended (F-Nb-4); the words remain the authority for "hungry" |
| A world forgets `restore` without `sleep` | refused at load (SD-Nb-5.2), never silent |

---

## 9. Commit plan

Each commit tracks **Implementation**, **Validation** and **Review** separately.

### C0 — Freeze record, base audit, source re-verification (Markdown only)

1. **Goal.** Record the freeze, the base (or the stack on N-a), and re-verify the one secondary source.
2. **Scope.** This file's header and §13.
3. **Implementation plan.**
   - [ ] Record the base, and whether N-a is merged or stacked.
   - [ ] Re-verify Watson N. F. et al., "Recommended Amount of Sleep for a Healthy Adult: A Joint Consensus Statement
         of the American Academy of Sleep Medicine and Sleep Research Society", *Sleep* 38(6):843–844, 2015
         (doi:10.5665/sleep.4716): read the statement (publisher page or PubMed abstract) and record the exact
         sentence ("7 or more hours per night on a regular basis") and the URL in §13. If it differs, record the
         difference and stop before `ARC-284`'s note cites it.
   - [ ] Re-confirm F-Nb-1 and F-Nb-2 anchors on the base.
4. **Validation plan.** Reading only.
5. **Acceptance.** The source recorded with its URL and the quoted sentence; anchors hold.
6. **Failure and edge cases.** Source unreachable: record `INCONCLUSIVE`, keep the citation marked secondary, continue
   (the default 125/h derives from 8 h, not from this source).
7. **Commands.** a web search for the title; `grep -n` on the anchors.
8. **Commit boundary.** Markdown only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C1 — Seam: a section's `requires`, an unconfigured pack's default `requires`

1. **Goal.** SD-Nb-5, generic, before any pack uses it.
2. **Scope.** `authoring/src/section.rs` (`AuthoredSection::requires`, default empty); `sdk/rust/src/pack.rs` and
   `owns_section!` (carry it into the capability); `sdk/rust/src/interactions/{decl,mod}.rs`
   (`SectionDecl::requires_default`); `worldpack/src/{content,configure,error}.rs` (the two checks and the two
   `PackError` variants); `worldpack/tests/requirements.rs` (stub-pack cases).
3. **Implementation plan.**
   - [ ] The trait method and its plumbing; - [ ] the section check after the owner-enabled check;
   - [ ] the unconfigured-default check after `check_requires`; - [ ] the two error variants with the words of §4.5.
4. **Validation plan.** Stub-pack tests: a section requiring a disabled system refused by name; an unconfigured stub
   whose default requires a disabled system refused; both accepted when the system is enabled; `mineworld validate`
   on the four reference worlds still green.
5. **Acceptance.** Tests pass; no existing refusal text changes.
6. **Failure and edge cases.** A pack configured *and* with default requirements: only the written file's `requires`
   applies (the default is not consulted).
7. **Commands.** `with-slot.sh cargo test -p mineworld-authoring -p mineworld-worldpack --test requirements`;
   `with-slot.sh target/debug/mineworld validate worlds/<each>`.
8. **Commit boundary.** Seam only; no pack uses it yet.

- [ ] Implementation · [ ] Validation · [ ] Review (every existing section and pack returns nothing)

### C2 — `systems/needs`: the crate and the pure level arithmetic

1. **Goal.** SD-Nb-3, independently testable.
2. **Scope.** New `systems/needs/{Cargo.toml,src/lib.rs,level.rs,component.rs,error.rs,codec.rs}`; workspace member.
3. **Implementation plan.** - [ ] `Level`, `Mode`, `Needs`, `Nourishment`; - [ ] `current`, `fold`, `word`.
4. **Validation plan.** `level.rs` unit tests: AC-Nb-2's flat-and-cafe numbers; clamp at 0 and 1 000; truncation
   toward zero for a rising and a falling rate (rate 125, Δt 1 s → 0; Δt 29 s → −1 … documented literal);
   `restore` without `sleeping` makes the asleep tiredness rate positive.
5. **Acceptance.** Tests pass.
6. **Failure and edge cases.** Δt negative (a fact before `at`): impossible by kernel ordering; `debug_assert`.
7. **Commands.** `with-slot.sh cargo test -p mineworld-needs --lib`.
8. **Commit boundary.** Pure code and types only.

- [ ] Implementation · [ ] Validation · [ ] Review (no float, no clock)

### C3 — The `needs:` section, two forms; genesis facts

1. **Goal.** SD-Nb-2.
2. **Scope.** `systems/needs/src/{section.rs,event.rs,system.rs}` (`AuthoredSection` with `requires`; `System` with
   `declaration`, `install`, `react` for the two genesis facts); `tests/support/mod.rs` (both worlds);
   `tests/section.rs`.
3. **Implementation plan.** - [ ] `AuthoredNeeds` with `Written`; - [ ] seeding per kind; - [ ] the reducers.
4. **Validation plan.** Forms decode and refuse as §4.2; `needs-section-kind`; genesis gives the literal facts.
5. **Acceptance.** Tests pass in both worlds.
6. **Failure and edge cases.** `needs: {}` on an item: refused as the item form needs `nourishment`.
7. **Commands.** `with-slot.sh cargo test -p mineworld-needs --test section`.
8. **Commit boundary.** Section and genesis only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C4 — `configure/needs.yaml`

1. **Goal.** SD-Nb-4.
2. **Scope.** `systems/needs/src/interactions.rs` (`InteractionSection`, `NeedsParameters`, `Thresholds`, `Restore`
   and their bounds, `requires`); `tests/configuration.rs`.
3. **Implementation plan.** - [ ] the parameter block; - [ ] the named-choice bounds; - [ ] `requires` from restore.
4. **Validation plan.** Decoding: each bound refused at line and column; `thresholds` under `actor:` refused;
   `requires` returns `[sleep]` / `[consumption]` / both / none as restore says; night-shift's actor-scoped rates
   resolve for a `night` person and not for others.
5. **Acceptance.** Tests pass.
6. **Failure and edge cases.** `restore: { hunger: [eating, eating] }` refused ("each at most once").
7. **Commands.** `with-slot.sh cargo test -p mineworld-needs --test configuration`.
8. **Commit boundary.** The section only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C5 — Reactions and disclosure

1. **Goal.** SD-Nb-6; AC-Nb-2 in a world; AC-Nb-5.
2. **Scope.** `systems/needs/src/{system.rs,disclose.rs}`; `tests/{reactions,perception}.rs`.
3. **Implementation plan.** - [ ] the three reactions with fold-first; - [ ] the disclosed view; - [ ] holder only.
4. **Validation plan.** AC-Nb-2 driven through real `eat` and `sleep` dispatch in both worlds; AC-Nb-5.
5. **Acceptance.** Literal values equal; audiences as stated.
6. **Failure and edge cases.** §4.7 rows: eating asleep, coffee (no form) writes nothing, cy (no section) unaffected.
7. **Commands.** `with-slot.sh cargo test -p mineworld-needs --test reactions --test perception`.
8. **Commit boundary.** Reactions and perception only.

- [ ] Implementation · [ ] Validation · [ ] Review (a fact that changes nothing writes nothing)

### C6 — Installed, loaded, removable

1. **Goal.** The build offers `needs`; AC-Nb-1 through the real loader; AC-Nb-6.
2. **Scope.** `systems/installed/{Cargo.toml,src/lib.rs}` (`Needs => mineworld_needs::NeedsSystem`, after `Sleep`);
   `Cargo.lock`; new `tools/cli/tests/needs_sections.rs` (scratch copies, the `forgetting_sections.rs` pattern);
   `systems/needs/tests/removable.rs`; literal lists of available systems found by the audit.
3. **Implementation plan.** - [ ] install lines; - [ ] the AC-Nb-1 table as one test per row; - [ ] removal.
4. **Validation plan.** AC-Nb-1, AC-Nb-6.
5. **Acceptance.** Every row refuses with the stated name and position, and the accepted case loads and runs 1 day.
6. **Failure and edge cases.** A refusal at a different position than stated: fix the code, not the expectation.
7. **Commands.** `with-slot.sh cargo test -p mineworld-cli --test needs_sections`; `with-slot.sh cargo test -p
   mineworld-needs --test removable -p mineworld-installed-systems`.
8. **Commit boundary.** Installation and loader tests only.

- [ ] Implementation · [ ] Validation · [ ] Review

### C7 — Lazy acceptance: zero facts, no process, kill and resume

1. **Goal.** AC-Nb-3 and AC-Nb-4 written; their short forms run.
2. **Scope.** `systems/needs/tests/lazy.rs` (the seeded script, the counts, the `#[ignore]` 30-day variant, the
   resume case with `mineworld-persistence` and `mineworld-test-support`).
3. **Implementation plan.** - [ ] script; - [ ] literal counts; - [ ] resume equality.
4. **Validation plan.** 2-day runs on both worlds and both seeds locally; the 30-day variant listed for the milestone.
5. **Acceptance.** Short runs pass; at least one meal and one sleep per person with needs per run (non-vacuous).
6. **Failure and edge cases.** None beyond the script's minimum counts.
7. **Commands.** `with-slot.sh cargo test -p mineworld-needs --test lazy`.
8. **Commit boundary.** Tests only.

- [ ] Implementation · [ ] Validation · [ ] Review (expected counts are literals)

### C8 — Docs, records, reference

1. **Goal.** Documentation law.
2. **Scope.** `systems/needs/README.md`; `docs/MODULE_SPEC.md` §4.1 (the item form under `needs:`, per Q-Nb-1) and
   §4.2 (the disclosed fields; `restore` default check); dated notes on `docs/decisions/ARC-284.md` (Watson
   re-verified, the disclosure fields), `ARC-285.md` (the section shape), `ARC-31.md` (a section may require systems;
   two forms), `ARC-61.md` (an unconfigured pack's defaults are checked); `docs/reference/` regenerated; §13.
3. **Implementation plan.** - [ ] README; - [ ] notes; - [ ] reference.
4. **Validation plan.** `python3 scripts/check_decision_ids.py`; `with-slot.sh cargo test -p mineworld-cli --test
   reference`.
5. **Acceptance.** Checks green; no new record id.
6. **Failure and edge cases.** None.
7. **Commands.** as above.
8. **Commit boundary.** Docs and reference only.

- [ ] Implementation · [ ] Validation · [ ] Review

**Size.** About 1.5 – 1.8 kLOC: seam ~180, the pack ~650, tests ~700, docs ~150. **M–L**. Implementer: Opus for C1,
C3–C5; Haiku acceptable for C2's tests, C6–C8 from this plan.

---

## 10. Requirements on other work

- N-c reads `needs.{hunger,tiredness,words,asleep,rates,nourishment}` and N-a's `sleep-terms`; it never reads item
  files or hard-codes a rate. Its eat band's "highest nourishment" uses `needs.nourishment`.
- N-d writes `needs: {}` on Market Town's people and `needs: { nourishment: … }` on its food items (step-35 §5.5
  values), enables `consumption`, `sleep` and `needs`, and writes no `restore` (both restorers are enabled).
- N-e renders `words` and a 5-step bar from `hunger`/`tiredness` (presentation only; the word is the pack's).

---

## 11. Execution contract (proposed; filled at freeze)

```text
PROJECT / PR:            MineWorld mvp0, PR N-b — the needs System Pack
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s35-nb-needs.md
RELATED / BINDING DOCS:  step-35-needs-and-sleep.md; pr-s35-na-sleep.md; docs/decisions/ARC-284.md, ARC-285.md,
                         ARC-283.md, ARC-286.md, ARC-31.md, ARC-36.md, ARC-61.md, ARC-63.md, ARC-86.md;
                         docs/MODULE_SPEC.md §§4.1–4.2; CLAUDE.md; the lane charter
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s35-nb, its own, held by one session
BRANCH:                  mvp0/pr-s35-nb, from origin/main (N-a merged) or from mvp0/pr-s35-na (stacked; rebased
                         onto main when N-a merges)
START CONDITION:         this design FROZEN (Q-Nb-1, Q-Nb-2 ruled) AND DOC-S35 merged AND N-a's types available
IMPLEMENTATION BASE:     recorded at C0
APPROVED SCOPE:          §3.1, as frozen
FROZEN INVARIANTS:       I-Nb-1 … I-Nb-8; SD-Nb-1 … SD-Nb-6; §5 thresholds
SEQUENCE:                C0 … C8
ALLOWED COMMANDS:        cargo * (heavy ones through cast-slots/with-slot.sh); git; gh (never merge);
                         python3 scripts/*; target/*/mineworld *; mkdir -p; sed -n; a web search in C0
NEVER:                   python3 -c; sed -i; perl -pi; curl/wget; editing kernel/, contracts/, persistence/, server/,
                         systems/{consumption,inventory,item,sleep}, any file under worlds/; a local full-workspace
                         test run (R-LOAD-1)
MATERIAL STOPS:          a change to SD-Nb-1's contract; a §4 default; a reference-world digest moving; C0's source
                         contradicting the step; any §5 FAIL at the milestone (fixed in the batch, never by retuning)
PLATFORMS:               Linux, macOS, Windows (CI at the milestone)
VALIDATION BUDGET:       per commit: build, the commit's tests, clippy; per push: `fast`
LIVE DOCUMENTATION:      this document (§13)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s35-nb.md
ENDPOINT AUTHORITY:      implementation, semantic commits, branch push, PR creation/update, CI repair: authorized
                         (source: the lane charter, operator 2026-10-18, SPEED FIRST); merge: the lane lead after
                         `fast` is green, per the charter
POST-MERGE SYNC OWNER:   lane lead: step-35 progress line; coordinator: overall.md, progress page; this session: §13
STOP CONDITION:          READY FOR LANE-LEAD MERGE (`fast` green on the exact head)
```

---

## 12. Questions

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-Nb-1** [OPERATOR] | A pack owns one section (F-Nb-1). Write nourishment on an item file as (a) `needs: { nourishment: { hunger: 600 } }` — one `needs:` section with a person form and an item form, the `bodies` precedent, no seam change — or (b) keep DOC-S35's top-level `nourishment: { hunger: 600 }` by teaching the seam several sections per pack (~300 more lines touching every pack's macro, the loader and the reference generator)? | **(a).** Same rules as `ARC-285` (with the kind, read by `needs` only, bounded, refused by name, disclosed only to the holder); smaller and already proven by `bodies`; C8 adds dated notes to `ARC-285` and `MODULE_SPEC.md` §4.1. Choose (b) only if one-section-per-pack is to change project-wide anyway |
| **Q-Nb-2** (coordinator) | Add the two generic load checks of SD-Nb-5 (a section may require systems; an unconfigured pack's defaults are checked), so the step's refusals "nourishment without consumption" and "tiredness restorer without sleep" exist? | **Yes.** Without them a world can carry food nobody can eat, or a tiredness that silently never falls — the failure step-35 §7 forbids. Both are additive and change no existing world; recorded as notes on `ARC-31` and `ARC-61`, so no ARC-288 |

---

## 13. Ledger (live during implementation)

### Evidence

- 2026-10-18 (design session): audit on `origin/main @ a8bba658`; DOC-S35 read from `origin/docs/s35-specs` (#278).
- Requirements from the lane lead (2026-10-18), applied: disclose each held kind's nourishment and the
  `rest_per_hour` in force (SD-Nb-6); `sleep-terms` is N-a's (N-a SD-Na-6).
- 2026-10-19 (implementation session, C0): **implementation base: stacked on `mvp0/pr-s35-na @ 8cb40e77`** (PR
  #297, open, not merged); the branch is rebased onto main by the lane lead after N-a merges. DOC-S35 (#278) is
  merged on that base (`docs/decisions/ARC-284.md`, `ARC-285.md` present).
- C0 source re-verification: Watson N. F. et al., *Sleep* 38(6):843–844, 2015, doi:10.5665/sleep.4716,
  <https://pubmed.ncbi.nlm.nih.gov/25979105/> (also <https://www.aasm.org/resources/pdf/adultsleepdurationconsensus.pdf>):
  "Adults should sleep 7 or more hours per night on a regular basis to promote optimal health." `PASS`: matches
  step-35 §5.5 ("≥ 7 h").
- C0 anchors re-checked on the base: `SystemPack::SECTION: Option<SectionOwner>` (`sdk/rust/src/pack.rs:50-52`),
  `owns_section_in_context!` "a pack owns one section" (F-Nb-1 holds); `check_requires` runs only over `found`
  configurations (`worldpack/src/configure.rs:316`), so an unconfigured pack is never checked (F-Nb-2 holds).
  `worldpack/tests/requirements.rs` exists but tests the package `requires:` (ARC-54), and the installed catalog is
  closed, so C1's stub-pack cases live in-crate (`worldpack/src/configure/tests.rs`, the existing `Probe`).

### Deviations and findings

- F-Nb-1 … F-Nb-4 (§2.2), from the design audit.
