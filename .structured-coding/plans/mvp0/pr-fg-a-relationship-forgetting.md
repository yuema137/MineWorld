# PR FG-a — System-owned forgetting in `relationships`: lazy decay and an acquaintance cap

## DESIGN FROZEN 2026-10-12 (primary session; operator QFGA-1)

```text
Design revision:        revision 2 (2026-10-12): revision 1 (1b80a6d on docs/fg-a-design) with §13.1's
                        rulings recorded and §14's contract filled, as committed with this header
Approved by / evidence: operator ruled QFGA-1 YES (read-only WorldRead::at() in the kernel); QFGA-2 …
                        QFGA-12 accepted as recommended; relayed by the primary session on 2026-10-12
                        (§13.1)
Implementation base:    main after DOC-24 (#175) merges, with DOC-24 corrected per F-FGA-1 and F-FGA-8;
                        exact commit recorded in FGA-C1's evidence
Execution contract:     §14 (filled at the freeze)
Lifecycle:              FROZEN — implementation starts only on the operator's go, and not before DOC-24
                        (#175) has merged
```

Scope (§1), section shapes (§4), design and decisions (§5), acceptance FA-1 … FA-12 and the commit plan are
frozen. Progress, evidence, findings and bounded corrections stay writable (§10, §15, §16). No FG-a question
remains open. Implementation runs in a fresh session (§14); this planning session does not implement.

*Superseded header:* `DRAFT — PR design, awaiting operator review (NOT frozen)`, revision 1.

**Effort:** `mvp0` (step-24 ruling 2026-10-12: forgetting is MVP-0; QS24-14: MVP-0 PRs are filed under
`mvp0/`). **Step:** S24, [`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md)
(approved 2026-10-12): §1.2 rulings 1–2, §2.4 rows G11 … G13, §3.2, §8.2, §10.1, §10.2 AC-FG-6 and AC-FG-7,
§11.2 row FG-a, §14.1. **Parent:** [`overall.md`](overall.md).
**Decision records this PR implements:** `ARC-86` (System-owned forgetting is lazy, pure and in the owning
System; no model touches it), drafted in DOC-24 (#175, open at this writing). It also adds dated notes to
`ARC-62` (a third extension catalog line), `ARC-63` (per-field roles, named-choice parameters) and `ARC-28`
(the "values never decay" gap is closed by configuration). No new decision number is requested (§13
QFGA-11).
**Planning base:** `origin/main @ cda4c13` (#174). Planning worktree
`/Users/yuema137/mineworld-worktrees/design-fg-a`, branch `docs/fg-a-design`, one writer.
**Siblings designed in parallel:** DL-a/DL-b (`design-dl`), FG-b/FG-c (`design-fg-bc`), FG-d (`design-sr-b`),
WE-a (`design-we-a`). The files shared with them are listed in §9.

---

## 1. Identity, goal, scope

```text
PR            FG-a — forgetting of System-owned person state in relationships (lazy decay of familiarity
              and regard; a per-person acquaintance cap) and conversation (forget_after_days), configured
              through the World's Interaction List; default: no decay, cap 150
base          main after DOC-24 (#175). Re-audit §3 if any of these moved: systems/{relationships,
              conversation,installed}, sdk/rust/src/interactions, kernel/src/{view,world,dispatch}.rs,
              worldpack/src/{read,configure}.rs, systems/presence/src/observe.rs, scripts/baselines.txt
branch        mvp0/pr-fg-a from main; worktree /Users/yuema137/mineworld-worktrees/impl-fg-a
merge         a merge commit, never a squash (ARC-5)
```

**Goal.** A world can say, by configuration alone, that people forget each other: familiarity and regard fade
toward a floor with time since the last contact, by a named, pure, integer curve; a person keeps at most a
bounded number of acquaintances; and a listener's remembered lines age out. Users can supply their own curve
as code through an `ARC-62` extension catalog. Forgetting writes no fact of its own, starts no Process, adds
no journal row and no save growth, and is replayed exactly. A world that configures nothing runs as before:
every fact and every run summary byte-identical (§6 FA-1 states precisely what moves and why).

What a world writes (illustrative; exact field table in §4):

```yaml
# configure/relationships.yaml
parameters:
  - { familiarity_decay: half-life, familiarity_decay_days: 90, familiarity_floor: 50,
      regard_decay: linear, regard_decay_days: 365, known_at_most: 150 }
  - { target: kin, familiarity_decay_days: 365 }        # kin fade slower (Roberts & Dunbar 2011, shape only)
  - { actor: hermit, known_at_most: 15 }
# configure/conversation.yaml
parameters:
  - { target: resident, forget_after_days: 7 }
```

**Approved scope (to be frozen).** Step-24 §11.2's FG-a row, made concrete in §4–§5:

1. relationships: decay of `familiarity` and `regard` computed lazily from the stored `(value, last_contact)`
   and world time, at every read (contact, disclosure, level reading, eviction ranking); a `DecayCurve`
   catalog with four bundled curves (`none`, `linear`, `half-life`, `power`) and an `ARC-62` extension line
   for user curves; per-value time constant and floor; the cap `known_at_most` with deterministic eviction,
   the new fact `acquaintance-forgotten`, and the `knows` edge removed in the same reduction.
2. conversation: `forget_after_days` (0 = never, the default), applied at the next write to the listener's
   history and on disclosure.
3. The enabling seams, each additive and generic (§5.6): the kernel's `WorldRead::at()`; four SDK additions
   to the Interaction List schema; the loader registering extension catalogs before it decodes
   configuration.
4. The proofs: AC-FG-6 and AC-FG-7 as refined in §6, through scripted worlds and through the binary.

**Non-goals.**

- Controller-owned memory (FG-b, FG-c), save compaction (FG-d), archival (FG-e), the decision list (DL-a),
  any model. **No language model touches this layer** (step-24 §8.2 "Why no LLM here", `ARC-86`).
- A decay Process, a timer, or any per-tick or per-day fact. A biographical `relationship-faded` fact on a
  timer is out (step-24 §8.2 "Levels"); the level a decay crossed is stated by the next reduction that
  touches the pair (§5.4).
- Data-defined curves (a table of points in `configure/`): recorded as a possible follow-up (§13 QFGA-6);
  FG-a's user extension point is code, through the catalog.
- Curve-specific parameter types (step-24 §8.2's sketch had `type Parameters` per curve): dropped, because an
  `ARC-62` catalog stores `Box<dyn Trait>`, which needs an object-safe trait (D-FGA-3).
- Changing any compiled default that a reference world exercises, any file under `worlds/`, or the
  increments of IL-e.
- `group-activity` and `schedule`: they hold nothing that grows (step-24 §2.4 G14; §3 F-FGA-11).
- A renderer for `acquaintance-forgotten` in P4's memory (`render/__init__.py` renders unknown types
  generically, F-FGA-12); a dedicated one is FG-b's or P6's choice.
- `mineworld inspect` showing decayed values: it shows stored state; disclosure shows the current reading
  (§5.3).

---

## 2. Binding inputs

- `CLAUDE.md` §4 rules 1 (single ownership), 2 (kernel ignorance), 5 (change amplification), 7 (typed
  boundaries), 8–10 (integration tests, real execution, deterministic and headless), 11 (no premature
  abstraction), 12 (no compatibility debt), 16 (reuse).
- Operator requirements for this PR (2026-10-12, relayed): pluggable — users can write their own rule, and
  the design says how; deterministic pure functions; single ownership — the rule lives in the owning
  System's configure section, never in a controller; no LLM in this layer; byte-identical towns when
  unconfigured; macOS, Linux and Windows.
- Step-24 §1.2 ruling 2 (forgetting is modular; (6): System-owned rules live in the owning System's configure
  section), §3.2 rules 1–5, §8.2, §10.1 (the 160 KiB per-person bound), §14.1 (QS24-7: decay off by default,
  cap 150 on).
- `ARC-25` (replay re-executes the journal and compares bytes; a SystemVersion change refuses old saves),
  `ARC-28` (relationships), `ARC-61` (configuration decoded with line and column; drift refused at resume),
  `ARC-62` (extension catalogs: write-once, per process, implementations pure; types on a line are installed
  packs), `ARC-63`/`ARC-64`/`ARC-65` (the Interaction List, classes, consequences), `ARC-86` (DOC-24 draft).
- IL-e as merged (#159, `pr-il-e-social.md`): relationships' section, its five parameters, `acquaint`,
  SD-IE-9 (version bumps when a section's payload shape grows), QIL-18.
- `INV-7` (only the owner writes), `INV-11` (the log is the truth), `INV-12` (kernel ignorance), `INV-13`
  (holder-only disclosure), `INV-15` (traceability: every fact has a cause).
- Test rules (`.claude/skills/structured-coding/prompts/test-ci-gate-rules.md`): one owner per failure class;
  no self-referential expected values; literal oracles; mutation proof where load-bearing.

---

## 3. Source audit (`origin/main @ cda4c13`, 2026-10-12)

Every finding was read in source or measured; none is inferred from a plan.

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-FGA-1** | `RelationshipValues` **already stores `last_contact`**, set by every contact in `adjust`. The `acquaintances` component's encoding needs no new field. Step-24 §8.2 ("Storing `last_contact` changes the `acquaintances` component's encoding") and DOC-24's draft `ARC-86` Consequences repeat a premise that is false. | `systems/relationships/src/component.rs:24` (field), `:82–90` (`adjust` sets it), `:185` (`schema_version = 1`) | The component schema stays at 1; snapshots of unconfigured worlds are unchanged. The version bump that does happen has another cause (F-FGA-8). Routed to DOC-24 (§9); otherwise FGA-C1 records a dated correction on `ARC-86`. |
| **F-FGA-2** | Contact is the only write: `apply` reads the entry, `entry()` inserts it sorted by counterpart, `adjust` saturates and clamps, `relate` and `insert` write edge and entry together, then `became-acquainted` (new) and `relationship-changed` (level crossed) are emitted. `before`/`after` levels are read from stored values. | `system.rs:203–254`; `component.rs:146–167` | Decay enters in exactly one place on the write path: before `adjust`, the stored values are replaced by their current reading (§5.2). |
| **F-FGA-3** | Disclosure clones the stored component to JSON for the holder only. | `system.rs:283–309` | Disclosure becomes "the stored component with each value read as of now" — same record shape (§5.3). |
| **F-FGA-4** | `PerceptionProvider::discloses(world: &WorldRead, observer, subject)` has **no instant**, and `WorldRead` has no clock: it holds entities, components, relations and processes only. `WorldView` has `at()`; the `Dispatcher` has `at`; `World::now()` exists; `observe(world, observer, at, providers)` receives an instant but cannot pass it to a provider. | `kernel/src/view.rs:59–80`, `:241–243`; `kernel/src/dispatch.rs:428–437`, `:463–468`; `kernel/src/world.rs:310`, `:557–564`; `systems/presence/src/interaction.rs:97–128`; `systems/presence/src/observe.rs:71–92`, `:179–189` | Lazy decay on read needs the instant at the read. Options in §5.6 K1; recommended: `WorldRead::at()`. |
| **F-FGA-5** | In `mineworld run`, the world is advanced to `t` and then every seat observes at `t`, so `World::now()` equals the observation instant. The hosted server observes at `clock.now_at(wall)`, which may be later than the world's last instant. | `tools/cli/src/run.rs:149–160`, `:206–209`; `server/src/runtime.rs:127`, `server/src/perception.rs:49–118` | With K1, a disclosure reads as of the world's instant. Hosted: at most the time since the last advance or request — seconds against day-scale decay; deterministic and stated (§5.3). |
| **F-FGA-6** | Interaction-List parameters are looked up **only at a place**: `parameters::<S>(read, at: PlaceId, roles)`; with no section the compiled defaults return without reading state. A selector naming a class matches only a filled role; an unfilled role matches `*` only. Relationships' `PARAMETER_ROLES` is `[actor, target, place]`, place = the causing fact's place. | `sdk/rust/src/interactions/lookup.rs:144–196`; `selector.rs:112–121`; `relationships/src/interactions.rs:106`, `:153–167` | Decay and the cap are properties of the pair and the holder, not of a place: a disclosure has no causing place. A place-free lookup is needed, and a `place:` or region entry setting those fields must be **refused**, not silently ignored (§5.6 S2, S3). |
| **F-FGA-7** | `parameters!` accepts only `$ty = $default:literal, $low:literal ..= $high:literal`; the bound is checked with `RangeInclusive::contains` at decode, refusals carry line and column. `bool` fields already use `false ..= true`. A named choice (a curve id) cannot be declared. The resolved base parameters are serialized **in full** in `<pack>-interactions-configured`. | `sdk/rust/src/interactions/section.rs:94–188` (`:140` the check); `resolve.rs:20–33`; `conversation/src/interactions.rs:44–63` | A curve name needs a generalised bound (§5.6 S1). New fields grow the configured fact's payload, so the pack's version is bumped (SD-IE-9 precedent). |
| **F-FGA-8** | A save's genesis journal row records the assembled world, including the composition (each installed system's declaration and version). After IL-e (#159) — version bumps and declaration changes, no fact or summary change — `journal-300s` moved in **every** world, "first differing chunk 0", while summaries and facts did not; `scripts/baselines.txt` was re-recorded with a reason line (13c, F-13c-impl-3). | `persistence/src/input.rs:42–57` (`WorldInput::Genesis`); `persistence/src/format.rs:34–45`; `pr-13c-nightly.md:1375–1385`; `scripts/baselines.txt` reason lines | FG-a declares a new fact and bumps two versions, so `journal-300s` moves in all four worlds by construction. AC-FG-6 ("baselines unchanged") cannot hold for that column; §6 FA-1 states the exact claim (QFGA-4). |
| **F-FGA-9** | `ARC-62` catalogs: a register function per trait, write-once per process, a different list panics naming both; types on a line must be installed packs and implement the trait and `Default`; an empty list is allowed. `WorldPack::compose` registers the catalogs; **configuration is decoded earlier**, in `WorldPack::read_with` → `configure::read`. | `docs/DECISIONS.md` `ARC-62`; `systems/presence/src/resolve.rs:183–209`; `sdk/rust/tests/extensions.rs:1–60`; `worldpack/src/load.rs:196–206`; `worldpack/src/read.rs:108–141`; `systems/installed/src/lib.rs:36–40` | To validate a user curve's name at decode (line and column), the loader registers catalogs before decoding configuration (§5.6 W1). |
| **F-FGA-10** | `ConversationHistory` keeps `Heard { speaker, at, utterance }`, oldest first, FIFO-bounded by `remembered` (1 … 64, default 32) through `remember_within`; written only in `react` on `spoke`; disclosed to the holder only; `in_conversation` reads the stored history against `gap` (≤ 86 400 s). Conversation's `PARAMETER_ROLES`: actor = speaker, target = listener, place. VERSION 3. | `systems/conversation/src/component.rs:21`, `:91–103`; `system.rs:68`, `:220–258`, `:341–359`, `:452–471`; `interactions.rs:44–63` | `forget_after_days` is a property of the listener: role `target` only, not placeable (S2). Applied at write and on disclosure; continuity is unaffected because a line older than one day is past every legal `gap` (§5.7). |
| **F-FGA-11** | `group-activity` (`Invitations`, `ActivityState`) is pruned on expiry or end; `schedule` holds today's agenda only. | step-24 §2.4 G14; `systems/group-activity/src/*` | Nothing to add (non-goal). |
| **F-FGA-12** | Readers of the acquaintances record: the 2D client lists counterparts only (`clients/2d/scripts/hud/readers.gd:127–129`); P4's memory renders `became-acquainted` and `relationship-changed` and any other type generically (`cognition/lm-controller/src/mineworld_cognition/memory/render/__init__.py:83–120`); no CLI source reads familiarity (`git grep`). No client computes a level. | as cited | A same-shape disclosure changes no reader. An evicted counterpart disappears from the 2D list, which is the intended observation. |
| **F-FGA-13** | The pinned facts to move on purpose: relationships' pinned test asserts VERSION 2 (`interactions.rs:194`); conversation's asserts VERSION 3 (`conversation/src/interactions.rs:139`); `tools/cli/tests/inspect.rs:55`, `social_composition.rs:379–380, :419–420` pin "conversation v3 … relationships v2"; `restart.rs:91` says "acquainted (relationships, once per direction)" — still true below the cap. `BecameAcquainted`'s doc says "Once per direction, ever" (`event.rs:9`), which the cap falsifies (a forgotten acquaintance can be made again). | as cited | Pins move with the bumps (D-IB-12 precedent); the doc sentence is corrected in FGA-C6. |
| **F-FGA-14** | Only three reference worlds install relationships (`social-cafe`, `lakeside`, `market-town`); `bodies-yard` installs conversation. None configures either section (`worlds/market-town/configure/` holds `calendar.yaml` and `weather.yaml` only). | `worlds/*/world.yaml`; `find worlds -path '*configure*'` | Unconfigured everywhere; the cap's default (150) is inert at ≤ 12 people. |

### 3.1 Growth, measured on the towns (G11, G12, G13)

**Instrument (E-FGA-d0).** Binary `/Users/yuema137/MineWorld/target/debug/mineworld`, built 2026-10-12 21:50
from main; no non-Markdown file changed between `0de1de4` and `cda4c13`, so it is `cda4c13`'s code. Commands,
from the planning worktree, scratch outside every repository
(`/Users/yuema137/mineworld-worktrees/scratch-fg-a`):

```text
mineworld run worlds/social-cafe  --headless --seed 7 --days 30 --save <scratch>/cafe30          wall 5.6 s
mineworld run worlds/market-town  --headless --seed 7 --days 30 --save <scratch>/market-town30   wall 9.6 s
mineworld run worlds/lakeside     --headless --seed 7 --days 30 --packs entities
          --packs presentation/mineworld-default --save <scratch>/lakeside30                     wall 4.2 s
sqlite3 <save> "SELECT count(*), max(json_extract(CAST(fact AS TEXT),'$.at')) FROM facts
                WHERE instr(CAST(fact AS TEXT),'\"became-acquainted\"')>0"
sqlite3 <save> "SELECT writefile('<s>.snap.zst', snapshot) FROM snapshots ORDER BY revision DESC LIMIT 1"
zstd -d <s>.snap.zst ; jq over .components[] (component_type, payload length, decoded payload)
```

| World (30 days, seed 7) | People | Directed entries | Of N(N−1) | Last `became-acquainted` | Entries at familiarity 1 000 | Median regard | Acquaintances payload (all people) | Same, as stored in the snapshot JSON | `knows` edges |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| social-cafe | 12 | 132 | 100 % | 343 803 s (day 4.0) | 110 of 132 | 1 000 | 26 281 B (≤ 2 200 per person) | 93 072 B | 132 |
| market-town | 12 | 132 | 100 % | 765 907 s (day 8.9) | 110 of 132 | 1 000 | 26 319 B (≤ 2 205) | 93 186 B | 132 |
| lakeside | 10 | 86 | 96 % | 1 092 606 s (day 12.6) | 54 of 86 | 970 | 17 119 B (≤ 1 797) | 60 815 B | 86 |

Conversation history at day 30 (32 lines cap): payload ≤ 8 149 B per person (social-cafe), ≤ 28 967 B per
person as stored in the snapshot JSON. `relationship-changed` facts by day 30: 363 / 360 / 183.

**What it shows.**

1. **Population² by construction, and reached quickly.** Every directed pair that ever meets keeps an entry
   forever: the graph is complete by day 4 (café) and day 9 (Market Town). Entries ≈ N(N−1).
2. **Saturation (F-S24-3, `ARC-28`'s known gap), measured.** 83 % of the café's entries sit at familiarity
   1 000 and the median regard is 1 000 after 30 days: the social graph carries no information any more.
3. **Bytes per entry.** ≈ 199 B of canonical component payload (26 281 ÷ 132), ≈ 705 B as the snapshot stores
   it (93 072 ÷ 132; the snapshot encodes each payload as a JSON array of byte values, ≈ 3.5×). This
   reproduces `pr-s6-save-retention.md` §4.2's 93 102 B at day 30.

**Projection (computed from the measured 199 B / 705 B per entry; not evidence for acceptance):**

| People N | Entries uncapped N(N−1) | Payload uncapped | Entries capped N·150 | Payload capped | Snapshot-JSON capped |
| --- | --- | --- | --- | --- | --- |
| 12 | 132 | 26 KB | 132 (cap inert) | 26 KB | 93 KB |
| 300 | 89 700 | 17.9 MB | 45 000 | 9.0 MB | 31.7 MB |
| 1 000 | 999 000 | 199 MB | 150 000 | 29.9 MB | 106 MB |

Per person, capped: ≤ 150 × 199 B ≈ 29.9 KB payload (≈ 105.8 KB snapshot-JSON). With `remembered` at its
maximum 64 and long utterances (≈ 255 B per line measured), conversation adds ≈ 16.3 KB payload (≈ 58 KB
snapshot-JSON). Against step-24 §10.1's "≤ 160 KiB per person in a snapshot": **46 KB on the payload
measure, ≈ 164 KB on the snapshot-JSON measure** — the latter exceeds 160 KiB (163 840 B) in the worst
configuration. Which measure the bound means is QFGA-7; the ×3.5 inflation is a persistence encoding matter
routed to FG-d (§9, F-FGA-15).

- **F-FGA-15 (persistence, routed).** The snapshot stores component payloads as JSON arrays of byte values,
  ≈ 3.5× the payload. Not FG-a's to change; zstd and SR-b reduce it on disk. Routed to FG-d's design.

---

## 4. Section shapes (typed parameters and bounds)

### 4.1 `configure/relationships.yaml` (owner: relationships; VERSION 2 → 3)

`RelationshipParameters` grows by seven fields. The five IL-e fields are unchanged.

| Field | Type | Default | Bound | Roles it may be scoped by | Meaning |
| --- | --- | --- | --- | --- | --- |
| `familiarity_decay` | `CurveName` | `none` | a bundled curve or one registered in the catalog | actor, target | the curve familiarity fades by |
| `familiarity_decay_days` | `u16` | 365 | 1 ..= 3 650 | actor, target | the curve's time constant (§5.1) |
| `familiarity_floor` | `u16` | 0 | 0 ..= 1 000 | actor, target | familiarity never fades below this |
| `regard_decay` | `CurveName` | `none` | as above | actor, target | the curve regard fades toward 0 by, from either side |
| `regard_decay_days` | `u16` | 365 | 1 ..= 3 650 | actor, target | its time constant |
| `regard_floor` | `u16` | 0 | 0 ..= 1 000 | actor, target | `|regard|` never fades below this |
| `known_at_most` | `u16` | 150 | 5 ..= 1 500 | actor | acquaintances a person keeps |

- **Roles.** actor = the holder (the person whose values fade, as for the increments), target = the
  counterpart. The decay fields are about the pair and the cap about the holder; neither is about a place.
  An entry with a `place:` selector, or a region entry, that sets any of these fields is refused at load
  with line and column; an entry with `target:` that sets `known_at_most` likewise (§5.6 S2). Lookups are
  place-free (S3).
- **Bounds.** 3 650 days = ten years, the longest time constant that still decays inside a long world;
  one day the shortest. `known_at_most` 5 … 1 500 and default 150 are step-24's (Dunbar's layers ≈ 5, 15, 50,
  150; QS24-7). The `*_decay_days` default (365) is **not calibrated** — no decay rate was verified (step-24
  §6.6, §15) — and is inert while the curve is `none`; a world that opts in states its own (QFGA-6).
- **`CurveName`** is a typed name (lowercase `a-z`, `0-9`, `-`, 1 … 32 bytes), checked syntactically by its
  `Deserialize` and against the catalog by the field's bound (S1). Serialized as a string in the configured
  fact.
- **Classes.** Scoping by class works as for every IL parameter (`ARC-64`): `{ target: kin,
  familiarity_decay_days: 365 }` lets a world make kin fade slower than friends (Roberts & Dunbar 2011's
  observation; shape only, no number taken from it).

**Facts** (one new):

| Fact | Roles → position | Default audience | Narrowest | Bio. configurable | Compiled biographical |
| --- | --- | --- | --- | --- | --- |
| `became-acquainted` | unchanged | participants | participants | yes | yes |
| `relationship-changed` | unchanged | participants | participants | yes | yes |
| **`acquaintance-forgotten`** `{ person, counterpart }` | `actor` → `Subject(0)` (holder), `target` → `Participant(1)`, `place` | participants | participants | yes | **yes** (QFGA-5) |

### 4.2 `configure/conversation.yaml` (owner: conversation; VERSION 3 → 4)

| Field | Type | Default | Bound | Roles | Meaning |
| --- | --- | --- | --- | --- | --- |
| `forget_after_days` | `u16` | 0 | 0 ..= 3 650 | target | a listener's lines older than this many days are dropped; 0 = never |

Roles: target = the listener (the history is the listener's). An `actor:` or `place:` scoped entry, or a
region entry, that sets it is refused (S2).

### 4.3 What these sections cannot do

- Raise a value: forgetting only lowers a value toward its floor (the host's clamp, §5.1).
- Decay by place, or differently in a region (refused).
- Remove the `knows` edge or the entry other than through the cap (no rule forgets a named person).
- Run a model, a timer, or emit a decay fact.
- Exceed `known_at_most` (bounded by construction, §5.5) or `remembered` (IL-e).

---

## 5. Design

### 5.1 Curves: the arithmetic, the catalog, and how a user writes one

**Inputs common to every curve.** `excess` = the part of the value above its floor (familiarity: `f − floor`
when `f > floor`, else nothing fades; regard: `|r| − regard_floor` when `|r| > regard_floor`, sign kept);
`elapsed` = `at − last_contact` in seconds (`WorldTime::duration_since`; `None` → 0); `T` = the time
constant in seconds (`*_decay_days × 86 400`). All arithmetic is integer (`i64`/`i128`, no float anywhere),
so every platform computes the same value.

| Curve | `remaining(excess, elapsed, T)` | Note |
| --- | --- | --- |
| `none` | `excess` | the default; nothing is computed |
| `linear` | `excess − min(excess, 1 000 × elapsed ÷ T)` | loses the whole range (1 000) in `T` |
| `half-life` | `k = elapsed ÷ T`; `i = (elapsed mod T) × 64 ÷ T`; `((excess × HALF[i]) >> 16) >> k`, and 0 when `k ≥ 16` | `HALF[i] = round(65 536 × 2^(−i/64))`, 65 literals, `HALF[0] = 65 536`, `HALF[32] = 46 341`, `HALF[64] = 32 768`; exact halving at every whole half-life; ≤ 1.1 % below the continuous curve between steps (Ebbinghaus-style exponential, approximated by table) |
| `power` | `excess × T ÷ (T + elapsed)` | the power-law family `(1 + t/T)^(−β)` with β = 1 (Murre & Dros 2015's shape; β fixed because the arithmetic is integer — another β is a catalog curve) |

**The host is the owning System** (`ARC-86`): whatever a curve returns, relationships clamps it to
`[0, excess]` before adding the floor back. Forgetting therefore never raises a value, never crosses its
floor, and never changes regard's sign. Monotonicity in `elapsed` is a curve's contract, checked for every
bundled curve by a property test and offered to curve authors as a template (§6 FA-8); the host cannot check
it without state (R-FGA-5).

**The catalog (`ARC-62`, third line).** relationships owns the trait and its register function:

```rust
/// A pure retention curve (`ARC-86`): how much of a value's excess over its floor is left after `elapsed`
/// with no contact, given the world's time constant. Implemented by a pack and listed on relationships'
/// extension line in `systems/installed`; selected by name in `configure/relationships.yaml`.
pub trait DecayCurve: Send + Sync {
    /// The name a world selects it by; never a bundled curve's name.
    fn name(&self) -> CurveName;
    /// The excess left, `0 ..= excess`. Pure: reads only its arguments, keeps nothing.
    fn remaining(&self, excess: u32, elapsed: SimDuration, time_constant: SimDuration) -> u32;
    /// The pack that implements it: the world must enable this pack to select the curve.
    fn curve_of(&self) -> SystemId;
}

pub fn register_curves(curves: Vec<Box<dyn DecayCurve>>);   // write-once per process; ARC-39 item 5's rules
```

- Bundled curves are compiled into relationships and need no registration; `register_curves` panics, naming
  both, on a name that collides with a bundled curve or with another listed curve, and on a second, different
  list (`ARC-62` item 4).
- `systems/installed/src/lib.rs` gains one line with an empty list:
  `extension mineworld_relationships::DecayCurve => mineworld_relationships::register_curves: [];`
  (empty lists are accepted, `sdk/rust/tests/extensions.rs` `Echo`).

**How a user writes their own curve** (the operator's "state how"):

```text
1  write a pack crate (a System Pack, like bodies for presence's and movement's catalogs) whose type
   implements mineworld_relationships::DecayCurve and Default; it may own nothing else
2  install it: its line in systems/installed/Cargo.toml and installed!, and its type on relationships'
   extension line — the ARC-33 two lines plus one catalog entry; rebuild
3  enable it in the world's systems: list (so the save's composition records its identity and version)
4  name it: configure/relationships.yaml  { familiarity_decay: my-curve, familiarity_decay_days: 120 }
5  the loader refuses an unknown name at its line and column, and a curve whose pack the world does not
   enable, by name (S4); the host clamps whatever it returns; replay and resume work as for every reduction
```

No existing pack, contract, kernel type or host is edited to add a curve (`CLAUDE.md` §4 rule 5). The
curve's pack being enabled (step 3) is QFGA-3: it makes a curve change a composition change, so a save made
with one curve is refused by name under another instead of diverging at replay.

### 5.2 Lazy decay at contact (the only write)

`apply` (F-FGA-2) becomes, for one directed change at the causing fact's instant `at`:

```text
1  acquaint forbidden?             → nothing, as today (IL-e)
2  entry = known.entry(counterpart)  (new entries: met(at, by), as today)
3  stated = level(stored values)                       the level last stated for this pair
4  rule   = parameters_unplaced(holder, counterpart)   decay fields only (S3)
5  values.rebase(rule, at)          familiarity, regard ← their reading at `at`   (no-op when both `none`)
6  values.adjust(increments, at)    as today: saturate, clamp, last_contact ← at
7  after  = level(values)
8  write edge and entry; emit became-acquainted (new), relationship-changed(stated → after) when they differ
9  new and len > known_at_most(holder)  → evict (§5.5)
```

- Step 5 re-bases both values at every contact, because `last_contact` is shared: "the last time anything
  passed between them".
- With both curves `none` (every reference world), step 5 changes nothing and steps 3/7 read exactly
  today's `before`/`after`: the facts are byte-identical (FA-1).
- **Levels (§5.4)** come out of steps 3 and 7.
- Re-basing at many small contacts rounds down at each step, so a pair seen hourly fades marginally faster
  than the closed form says. Deterministic and replayed identically; documented, not corrected (R-FGA-6).

### 5.3 Lazy decay on read: disclosure, perception, saves, replay

**Disclosure.** `discloses` returns, to the holder only (unchanged, `INV-13`), the same `acquaintances` record
— same component type, schema version 1, same JSON shape — with each entry's `familiarity` and `regard`
replaced by their reading at `read.at()` (K1). `last_contact`, `first_met`, counters are the stored ones. With
no section configured the stored component is encoded as today, without computing anything (byte-identical
observations). A public read helper, `mineworld_relationships::current(read, holder, counterpart) ->
Option<RelationshipValues>`, gives Rust readers (tests, a future in-process decision model) the same reading.

- *Why the same shape* (QFGA-10): an observation is a disclosure, not the state (`INV-13`); readers
  (2D client, P4, the paced controller) need no change (F-FGA-12); a reader that wants the stored base value
  has no business with it.
- *Instant.* `read.at()` is the world's instant (K1). In `run` it equals the observation instant (F-FGA-5);
  on the hosted server it may trail the observation by the time since the last request or advance, which is
  stated in relationships' README.
- *Unknown curve at read time* (only reachable if a save is opened in a build without the curve, which the
  composition check refuses first under QFGA-3): **the record is not disclosed** — it fails closed, as
  presence's filters do, rather than showing undecayed values — and the reduction path faults (§5.2 step 4
  returns a `KernelError` naming the curve, journaled as a fault).

**Perception and memory.** No fact is added by decay, so what any seat perceives (`mineworld perceived`, P4's
store) is unchanged except for `acquaintance-forgotten` and the level facts whose timing decay moves. P4's
impressions record the disclosed value with its instant (step-24 R-S24-3).

**Saves.** Stored state changes only inside reductions that already happen: no Process, no deferred event, no
journal row, no fact for decay (the cap adds `acquaintance-forgotten` inside the reduction that caused it).
Snapshots hold stored values; the snapshot of an unconfigured world is byte-identical to today's (F-FGA-1).

**Replay (`ARC-25`).** A reduction reads `(stored values, at)` and the configured fact, all reproduced by
re-executing the journal, so replay produces the same bytes. A resume from a snapshot reads the same stored
values. Kill-and-resume equality (`AC-6`) holds because nothing outside the world is consulted. A changed
curve or parameter is a configuration change, refused at resume by the drift check (`ARC-61`); a curve pack's
code change is a version change of an enabled pack (QFGA-3).

### 5.4 Levels and facts

- **One crossing fact, net, at the next contact.** `relationship-changed { from: stated, to: after }` where
  `stated` is the level of the stored values (the last level a fact stated, because every crossing at a
  reduction is stated) and `after` is the level after decay and increment. A pair that faded from Friend
  and is restored to Friend by the contact states nothing; one that faded and is not restored states
  Friend → Friendly at that contact, caused by it (`INV-15`). The chain of `relationship-changed` facts per
  pair stays consistent. No `relationship-faded` fact, no timer (step-24 §8.2; QFGA-9).
- **Disclosure shows the current level's inputs**, so a controller sees a faded friendship before any fact
  states it. That is the lazy model's intended lag (step-24 "Levels").

### 5.5 The cap: eviction order (deterministic)

When step 2 of §5.2 created an entry and `known.len() > known_at_most(holder)` (looked up with actor = holder,
place-free):

```text
candidates  every entry except the one just created (the contact happening now is the most recent;
            evicting it would form and forget in one reduction)
rank        1. lowest current familiarity at `at` (decay applied, §5.1)
            2. then oldest last_contact
            3. then lowest counterpart PersonId (EntityId order)
evict       remove the entry; unrelate knows(holder → counterpart); emit acquaintance-forgotten
            { person: holder, counterpart } about the holder, participants [holder, counterpart], at the
            causing fact's place, caused by it; audience and biography through the section (§4.1)
repeat      while len > known_at_most (once in practice: the cap is fixed at genesis)
```

- Bounded by construction: after every reduction, `known.len() ≤ known_at_most(holder)`; edges equal entries
  (the existing invariant, `tests/relationships.rs` "the edge and the entry agree").
- The ranking is a total order on distinct counterparts, so the victim is unique on every platform.
- Meeting a forgotten counterpart again forms a new entry (`met`, familiarity 0) and states
  `became-acquainted` again: "once per formation", not "once per direction, ever" (F-FGA-13).
- The counterpart's own entry about the holder is untouched (directed, like `knows`).
- Cost: ranking only when an entry is created beyond the cap — never in the reference worlds.

### 5.6 The enabling seams (generic, additive)

**K1 — kernel: `WorldRead::at() -> WorldTime`** (recommended; QFGA-1). `WorldRead` gains the instant the read
is taken at: `WorldView::read()` passes the view's instant, the dispatcher's validate read its `at`,
`World::read()` the world's `now()`. Four construction sites, all crate-private (`view.rs:66–80`,
`dispatch.rs:463`, `world.rs:557`, `system/tests.rs:227`). The kernel already owns time (`CLAUDE.md` §4 rule
2); this exposes it read-only, with no domain meaning. Every pack that keeps lazily derived state needs it:
FG-a now, conversation's `forget_after_days`, DL-c's `needs` (step-24 §6.5) later.
*Alternatives:* (b) change `PerceptionProvider::discloses` to take the instant — 13 implementers edited for one
user (change amplification); (c) a provided method `discloses_at` on presence's trait defaulting to
`discloses` — two methods for one concept, an adapter before a stable contract (`CLAUDE.md` §4 rule 12).

**S1 — SDK: named-choice parameters.** `parameters!`'s field grammar becomes `$field: $ty = $default:expr,
$bound:expr;` with a trait `ParameterBound<T> { fn admits(&self, &T) -> bool; fn describe(&self) -> String }`
implemented for `RangeInclusive<T>` with today's message (`'{field}' is {low} … {high}, not {value}`, byte for
byte) so every existing invocation compiles and refuses unchanged. relationships supplies `CurveCatalog`
(admits bundled names and registered ones; describes "one of none, linear, half-life, power[, …]").

**S2 — SDK: per-field roles.** `InteractionSection` gains `const FIELD_ROLES: &'static [(&'static str,
&'static [Role])] = &[];` (a field not listed may be scoped by every `PARAMETER_ROLES` role, as today). The
section decoder refuses, at the entry's line and column, an entry that sets a listed field with a selector on
a role the field does not allow, and a region entry that sets a field whose roles exclude `place`.
`Fields` gains `fn set_names(&self) -> Vec<&'static str>` (macro-generated). Precedent: `ActionDecl`'s roles
and `regional: false` (IL-e's `acquaint`).

**S3 — SDK: place-free lookup.** `interactions::parameters_unplaced::<S>(read, roles) -> S::Parameters`: the
base resolution with the place role unfilled; the compiled defaults, reading nothing, when the pack is
unconfigured (as `parameters`). Correct for exactly the fields S2 makes unplaceable, which is how relationships
and conversation use it.

**S4 — SDK: a section's required systems.** `InteractionSection` gains `fn requires(section: &Section<Self>)
-> Vec<SystemId> { Vec::new() }`; `HeldSection::requires` forwards to it, so the loader's existing
`check_requires` refuses by name a configuration requiring a system the world does not enable. relationships
returns the implementing packs of every catalog curve its section names.

**W1 — loader: register catalogs before decoding configuration.** `WorldPack::read_with` calls
`Capability::register_extensions()` before `configure::read` (`read.rs:141`), as `compose` already does
(idempotent for the same list). Without it a user curve's name is unknown at decode.

**I1 — installed set.** relationships' empty extension line (§5.1).

### 5.7 conversation: `forget_after_days`

- **At write** (`react` on `spoke`, the only writer): after `remember_within`, drop lines with
  `at − heard.at > forget_after_days × 86 400` (strictly older), looked up place-free with target = listener.
- **On disclosure:** the same filter at `read.at()`, so an observation never shows a line older than the
  world allows even when no line was written since.
- **Continuity** (`in_conversation`, `gap` ≤ 86 400 s): reads the stored history; a dropped line is older than
  one day ≥ `gap`, so it could not have continued a conversation. Unchanged decisions.
- **Default 0** reads nothing more than today and changes nothing.
- `ConversationHistory::forget_older_than(cutoff: WorldTime)` is the one new mutator, called only from
  `react`.

### 5.8 Decisions (D-FGA-*)

| ID | Decision | Rationale |
| --- | --- | --- |
| **D-FGA-1** | Decay is computed at reads from stored `(value, last_contact)`; stored state changes only at contact (§5.2–5.3). | `ARC-86`; zero facts, rows and growth; replay exact. |
| **D-FGA-2** | Disclosure keeps the record shape and carries current values (§5.3). | No reader changes (F-FGA-12); `INV-13`. QFGA-10. |
| **D-FGA-3** | `DecayCurve` is object-safe: `name`, `remaining`, `curve_of`; no per-curve `Parameters`; one time constant and floor per value, in the section. | `ARC-62` stores `Box<dyn Trait>`; step-24 §8.2's sketch (`type Parameters`, `const ID`) is not object-safe. A shape variant is another named curve. Bounded deviation from the step text, recorded in §16. |
| **D-FGA-4** | Bundled curves: `none`, `linear`, `half-life` (64-step table), `power` (β = 1). | Step-24's four names; integer-exact; β ≠ 1 needs fractional powers. QFGA-6. |
| **D-FGA-5** | Eviction: lowest current familiarity, then oldest contact, then lowest id; never the entry just formed (§5.5). | Step-24 §8.2's order plus the newcomer rule; total order. |
| **D-FGA-6** | `acquaintance-forgotten`: participants, biography configurable, compiled biographical. | Mirror of `became-acquainted`; QFGA-5. |
| **D-FGA-7** | Level facts are net at contact (§5.4). | Consistent per-pair chain; no timer. QFGA-9. |
| **D-FGA-8** | K1, S1–S4, W1, I1 as in §5.6. | F-FGA-4, -6, -7, -9. QFGA-1, QFGA-2. |
| **D-FGA-9** | Versions: relationships 2 → 3, conversation 3 → 4; component schemas unchanged; old saves refused by name. | SD-IE-9: the configured fact's payload grows; F-FGA-1. |
| **D-FGA-10** | The 300-person proof (AC-FG-7) is a scripted world in relationships' tests, not a World Pack run through the binary. | A 300-person `run` scales super-linearly in perception (each consult observes everyone present) and is not a CI-sized test; the scripted world exercises the same reductions. A binary-level 300-person run is an optional measurement (§6 FA-3 note). Bounded deviation from step-24's "test-only pack" wording. |

---

## 6. Acceptance (decided before measuring, `ARC-23`)

Rules as IL-e's: every guarded criterion names its mutation; a mutation is applied in the working tree,
observed to fail by name, then reverted; `git status` and `git grep MUTATION -- '*.rs'` are recorded after.
Expected values are literals from hand arithmetic, the test's own layout, or E-FGA-0's captured values —
never from the implementation under test.

```text
FA-1  Byte identity, unconfigured (AC-FG-6 as refined, QFGA-4). On the final head, with the four reference
      worlds unconfigured:
        (a) `python3 scripts/ci_parity.py record --binary target/release/mineworld --profile long`, then
            `baseline check`: summary-300, summary-1000, summary-300s and facts-300s equal to
            scripts/baselines.txt for every world;
        (b) journal-300s differs from the committed value in the three worlds installing relationships and
            in bodies-yard (conversation) — expected (F-FGA-8); `ci_parity.py diff` against E-FGA-0's base
            record locates the first difference in chunk 0 and no later chunk differs;
        (c) row-level proof: 30-day `--save` runs of all four worlds with the base binary (E-FGA-0) and the
            head binary; `sqlite3 <save> "SELECT revision, at, action_id, hex(entry) FROM journal WHERE
            revision >= 2 ORDER BY revision"` (schema: revision, at, action_id, entry) piped to
            `shasum -a 256` is equal base vs head for every world, and the revision-1 row differs;
        (d) `mineworld validate` output for the four worlds: identical, except lines that name a system's
            version, each listed in the evidence;
        (e) baselines.txt: journal-300s re-recorded with `reason FG-a: genesis row carries the composition
            (relationships v3, conversation v4, acquaintance-forgotten declared); no fact or summary moved`.
      M-FA1a: compiled known_at_most 150 → 10: social-cafe 30-day summary differs (acquaintance-forgotten
      appears) — the instrument sees the cap.
      M-FA1b: compiled familiarity_decay none → half-life with 1 day: the 30-day summary differs.
FA-2  Explicit default: a scratch social-cafe with configure: [relationships, conversation], each file
      `{ extends: default, parameters: [ { familiarity_decay: none, regard_decay: none, known_at_most: 150 } ] }`
      (conversation: `forget_after_days: 0`), 30 days seed 7 --save, against the unconfigured copy: facts
      equal except exactly two genesis configuration facts; every later event id offset by 2; request
      outcomes equal.
FA-3  The cap at scale (AC-FG-7, scripted, relationships' tests). A scripted world of 300 people in one place
      (presence, conversation, relationships), contacts over 100 simulated days: person i talks to 200
      distinct others in an order fixed by the test's layout; default cap 150:
        (a) after every reduction, every person's entries ≤ 150, and knows edges equal entries;
        (b) acquaintance-forgotten count equals the literal the layout implies; each names a counterpart
            that is then absent from the holder's entries and edges, and is caused by the spoke that formed
            the 151st entry;
        (c) per person, acquaintances payload ≤ 150 × the largest entry and (acquaintances +
            conversation-history) payload ≤ 160 KiB (QFGA-7); the snapshot-JSON size is recorded beside it;
        (d) a 6-person case with known_at_most 5 and hand-set values: the victims, in order, are the literal
            counterparts §5.5's ranking names (a tie on familiarity broken by last_contact; a tie on both by
            id), and the entry just formed is never evicted.
      M-FA3a: rank by highest familiarity → (d) fails. M-FA3b: no unrelate → (a) fails.
      M-FA3c: the new entry eligible → (d)'s newcomer case fails.
      Note: a 300-person World Pack through `mineworld run` is optional, measured only if cheap (D-FGA-10).
FA-4  Curve arithmetic (unit, literal oracles): half-life (E 1 000, T): t = T → 500; 2T → 250; T/2 → 707
      (1 000 × 46 341 >> 16); 16T → 0; linear (E 1 000, T 10 d): 5 d → 500, 10 d → 0, 20 d → 0;
      power (E 1 000): t = T → 500, 3T → 250; none → E. Floors: familiarity 600 with floor 200, half-life,
      t = T → 400. Regard −800, floor 100, half-life, t = T → −450 (sign kept, magnitude 100 + 350).
      HALF[0], HALF[32], HALF[64] literals; the table strictly decreasing. Every bundled curve non-increasing
      over a fixed grid of (E, t, T). A curve returning more than E, or 0 for t = 0, is clamped by the host
      to [floor, value].
      M-FA4: HALF[32] 46 341 → 46 340 → the T/2 literal fails.
FA-5  Lazy, and no decay fact (AC-FG-7, scripted). Half-life 1 day configured for everyone. Alice ↔ Bob
      talk to familiarity F (literal from the increments) at t0; then the world advances one day with
      unrelated activity elsewhere:
        (a) Alice's observation at t0 + 1 d discloses familiarity F/2 (floor division), the same record shape
            and component type as unconfigured;
        (b) the stored component still holds F with last_contact t0;
        (c) no relationships fact between t0 and t0 + 1 d;
        (d) a talk at t0 + 1 d stores F/2 + 10 with last_contact t0 + 1 d, and states relationship-changed
            from the stored level to the new one exactly when they differ, caused by that spoke (literal
            levels chosen so they do);
        (e) the same script unconfigured discloses F at t0 + 1 d (no decay).
      M-FA5a: disclosure returns stored values → (a) fails. M-FA5b: contact adds to stored, not current →
      (d) fails. M-FA5c: before-level read after rebase → (d)'s fact is missing.
FA-6  Through the binary, saves and replay (tools/cli tests). A scratch social-cafe with
      { familiarity_decay: half-life, familiarity_decay_days: 2, regard_decay: linear,
        regard_decay_days: 30, known_at_most: 5 }, 30 days seed 7 --save:
        (a) in the final snapshot, every acquaintances entry count ≤ 5;
        (b) ≥ 1 acquaintance-forgotten (else INCONCLUSIVE, not PASS);
        (c) `mineworld replay <save>` PASS;
        (d) kill-and-resume: --days 15 then --days 30 on one save gives facts byte-equal to the uninterrupted
            30-day save;
        (e) the set of fact types equals the unconfigured twin's plus acquaintance-forgotten (no decay fact);
        (f) relationship-changed counts differ from the twin (decay observable; else INCONCLUSIVE).
      M-FA6: eviction skipped → (a) fails.
FA-7  Refusals at load, each at its line and column (worldpack loader): unknown curve name; a `place:` entry
      setting familiarity_decay_days; a region setting known_at_most; a `target:` entry setting
      known_at_most; an `actor:` entry setting forget_after_days; known_at_most 4 and 1 501;
      familiarity_decay_days 0. A catalog curve whose pack the world does not enable: refused by name (S4).
      Existing IL-e refusal messages byte-identical (S1).
      M-FA7: S2's check skipped → the place case loads and the test fails.
FA-8  A user curve through the catalog (own test binary, so its process registers its own list, ARC-62):
      a test pack's `step-down` curve (excess → excess / 2 once elapsed ≥ T) is registered; a section naming
      it decodes; a scripted world discloses the literal its arithmetic gives; a hostile test curve returning
      excess + 5 000 is clamped to the stored value; registering a curve named `half-life` panics naming
      both; registering a different list a second time panics naming both. The README's five steps (§5.1)
      are the steps the test performs, minus the rebuild.
      M-FA8: host clamp removed → the hostile case fails.
FA-9  conversation forget_after_days (scripted): `{ target: resident, forget_after_days: 1 }`; a line to a
      resident at t0: disclosed at t0 + 1 d, not disclosed at t0 + 1 d + 1 s, still stored until the next
      line to that resident, which drops it; a commuter's line kept; default 0 keeps everything; continuity
      (conversation-started) decisions equal to the unconfigured twin's for lines within `gap`.
      M-FA9: the disclosure filter removed → the t0 + 1 d + 1 s case fails.
FA-10 Cost: social-cafe 30 days seed 7, dev profile, sequential on one idle machine, median of 3 walls: head
      unconfigured ≤ base × 1.05; FA-6's configuration ≤ head unconfigured × 1.10. Base walls spreading > 5 %
      → INCONCLUSIVE: re-run once, then report rather than retry.
FA-11 Guards: no diff outside §11's change set; root Cargo.toml and Cargo.lock unchanged; the acceptance
      vocabulary tests (ac1_composability, precursor_vocabulary, seam_vocabulary, configuration_vocabulary,
      client_text) unedited and passing.
FA-12 Full gate on the final head: cargo fmt --check, check, clippy -D warnings, test (workspace); both doc
      scripts; check_scratch.py scan; CI green on the exact PR head, including test-windows and test-macos
      (FA-3, FA-4, FA-5 and FA-9 assert literals, so the three platforms are compared by construction).
```

**Determinism across platforms.** Integer arithmetic only; sorted vectors only (no hash iteration); literals
asserted in tests that CI runs on Linux, macOS and Windows; FA-1(a) on one platform, with the nightly's
parity-long comparing platforms afterwards (`ARC-83`).

---

## 7. Test data fixed now

- Scratch copies only, made at test time by IL-b's helper pattern; nothing is written under `worlds/`.
- Classes for FA-7/FA-9: `{ class: resident, of: person, tag: resident }`, `{ class: commuter, of: person,
  tag: commuter }` (IL-e's §7 set).
- Runs: 30 days, seed 7, `--save` where a save is read. FA-3: 300 people, 200 contacts each, 100 simulated
  days. FA-5: half-life 1 day.

---

## 8. Test ownership

| Failure class | Owner | Layer |
| --- | --- | --- |
| curve arithmetic, table, clamp, monotonicity | relationships unit (`src/decay.rs` tests) | Unit — FA-4 |
| lazy reading at contact, disclosure, levels | relationships integration (`tests/forgetting.rs`, scripted world) | Integration — FA-5 |
| cap, eviction order, edges = entries, scale | relationships integration (`tests/forgetting.rs`) | Integration — FA-3 |
| user curve through the catalog | relationships integration, own binary (`tests/curve_catalog.rs`) | Integration — FA-8 |
| conversation aging | conversation integration (`tests/conversation_and_presence.rs` gains a case, or a new `tests/forgetting.rs`) | Integration — FA-9 |
| decode-time refusals, S1 message parity, S2, S4 | SDK tests (`sdk/rust/src/interactions/tests.rs`, `sdk/rust/tests/interactions.rs`) + loader (`tools/cli/tests/social_sections.rs` pattern) | Unit + integration — FA-7 |
| `WorldRead::at()` at each construction site | kernel test (one per site: view, validate, `World::read`) | Unit |
| byte identity, saves, replay, resume, cost | binary (`tools/cli/tests/forgetting_runs.rs`), `ci_parity.py`, evidence runs | Real lifecycle — FA-1, FA-2, FA-6, FA-10 |
| platform equality | CI test jobs on three OS; nightly parity | CI |

Real-LLM Gate: **N/A** — no language model is involved, by architecture (`ARC-86`). Static: fmt, clippy own
formatting, unused code and typing; no test duplicates them. No test asserts a getter, a constructor or an enum
listing.

---

## 9. Shared files with parallel lanes

| File | FG-a | Other lane | Rule |
| --- | --- | --- | --- |
| `docs/DECISIONS.md` `ARC-86` | the F-FGA-1/F-FGA-8 correction | DOC-24 (#175) owns the record | Preferred: DOC-24 corrects its draft before merging ("the component encoding is unchanged; the version bump comes from the section's payload and the new fact; `journal-300s` moves at its genesis row"). Otherwise FGA-C1 adds a dated note. |
| `docs/DECISIONS.md` `ARC-62`, `ARC-63`, `ARC-28` | dated notes | WE-a may note `ARC-62`/`ARC-63` | appended blocks; keep all |
| `sdk/rust/src/interactions/*` | S1–S4 | WE-a (a kinds catalog) and DL-a (typed model parameters) may want S1 | FG-a lands S1 first if both are in flight; the later PR reuses it; neither forks the macro |
| `kernel/src/view.rs` etc. | K1 | none known | — |
| `worldpack/src/read.rs` | W1, one line | DL-a reads `decisions:` in `worldpack` | different functions; merge by hand |
| `systems/installed/src/lib.rs`, `Cargo.toml` | one extension line | WE-a adds `happenings` | merge by hand; both kept |
| `scripts/baselines.txt` | journal-300s, four worlds, one reason line | PR 12d (#171) re-baselines the towns; FG-d may move journal bytes | whichever merges second re-records on its merge head; reason lines kept byte for byte (`baseline write`) |
| `tools/cli/tests/{inspect,social_composition}.rs` version pins | conversation v4, relationships v3 | any lane bumping another pack on the same lines | merge the line by hand; all versions kept |
| `docs/MVP_STATUS.md` | its S24 cell | other S24 lanes | per-lane wording |

---

## 10. Commit plan

Each commit lists implementation, validation and review separately; each `[x]` needs evidence in §15.

### FGA-C0 — Design (this document), docs only

**Goal.** A reviewable design before code (`CLAUDE.md` §2.2). **Scope.** This file. **Boundary.** Markdown
only; no other file.

- [x] Implementation: written from §3's audit and §3.1's measurements (planning session, 2026-10-12).
- [x] Validation: `python3 scripts/check_doc_headings.py` → "220 numbered sections across 31 documents, none
  duplicated", exit 0; `python3 scripts/check_decision_ids.py` → "114 decision ids, all distinct", exit 0
  (planning worktree, before the first commit, and again at the freeze revision); CI `fast` PASS on #179 at
  1b80a6d (run 38352358089); the freeze head's `fast` is recorded in the PR.
- [x] Review (self, planning session): every finding cites a file and line or a measurement; every guarded
  criterion names a mutation; no decision number is taken; terms are `CORE_CONCEPTS.md`'s.

### FGA-C1 — Specs before code

**Goal.** The documentation the code must match. **Scope.** `systems/relationships/README.md` (decay, floors,
the cap, `acquaintance-forgotten`, "Writing your own curve" with §5.1's five steps; the "Known gap" paragraph
replaced); `systems/conversation/README.md` (`forget_after_days`); `docs/MODULE_SPEC.md` §4.2 (named-choice
parameters, per-field roles, place-free lookups, a section's required systems); `docs/DECISIONS.md` dated
notes on `ARC-62` (third line), `ARC-63` (S1–S4), `ARC-28` (decay by configuration), and on `ARC-86` only if
DOC-24 did not correct it (§9); `docs/MVP_STATUS.md` S24 row; `handoff-fg-a.md` created.
**Non-goals.** No code. No new decision number (QFGA-11). **Dependencies.** DOC-24 merged.

- [x] Implementation: relationships README (decay table, curves, cap, levels, "Writing your own curve" with
  §5.1's five steps; the "Known gap" paragraph replaced), conversation README (`forget_after_days` row and
  roles), `MODULE_SPEC.md` §4.2 (bounds as values, per-field roles, place-free lookups, required systems;
  refusal list), `DECISIONS.md` dated notes on `ARC-28`, `ARC-62`, `ARC-63`, and on `ARC-86` (E-FGA-1);
  `handoff-fg-a.md` created. The `MVP_STATUS.md` row moved to FGA-C9 (DV-7).
- [x] Validation: `check_doc_headings.py` → "223 numbered sections across 31 documents, none duplicated";
  `check_decision_ids.py` → "122 decision ids, all distinct" (E-FGA-1).
- [x] Review: README tables equal §4.1/§4.2 field by field (type, default, bound, roles); every README rule
  is in `MODULE_SPEC.md` §4.2, `ARC-86` or this design; terms are `CORE_CONCEPTS.md`'s and §4.2's ("section",
  "parameter", "role", "region", "extension catalog").
**Acceptance.** A reader can configure decay and write a curve from the README alone, and every rule there
is in a spec. **Failure cases.** A term conflict with `CORE_CONCEPTS.md` is a stop (terminology law).
**Commands.** the two doc scripts. **Boundary.** Markdown only.

### FGA-C2 — Kernel: `WorldRead::at()`

**Goal.** K1: a read knows its instant. Separate commit because it is the only kernel change and must be
reviewable alone.
**Scope.** `kernel/src/view.rs` (`WorldRead` field `at`, `pub const fn at(&self) -> WorldTime`, `new` takes
it; `WorldView::read` passes `parts.at`); `kernel/src/dispatch.rs:463` (passes `self.at`);
`kernel/src/world.rs:557` (`World::read` passes `self.now()`); `kernel/src/system/tests.rs:227`.
**Non-goals.** No other kernel change; no caller changes behaviour.

- [x] Implementation: `WorldRead` gains `at: WorldTime` and `pub const fn at()`; `new` takes it;
  `WorldView::read` passes `parts.at`, the dispatcher's validate read `self.at`, `World::read` `self.now()`,
  the unit bench `WorldTime::EPOCH` (its request instant).
- [x] Validation: NEW `kernel/tests/read_instant.rs`, three tests: validate's read reports 4 321 for a request
  dispatched at 4 321 (through a rejection detail); resolve's and a subscriber's react read report 86 400 for
  a dispatch at 86 400 (through payloads); `World::read().at()` is the epoch, then 7 200 after
  `advance_to(7 200)`. `cargo test -p mineworld-kernel`: every suite ok; `cargo clippy -p mineworld-kernel
  --all-targets -- -D warnings` clean (E-FGA-2).
- [x] Review: no site reads a wall clock (each passes the instant it already holds); `WorldRead` adds one
  `i64`-sized field; the doc says "world time, never a wall clock (`INV-12`)"; `new` stays crate-private,
  so no caller outside the kernel changes.
**Acceptance.** The three sites return the instants named. **Failure cases.** None at run time (a field read).
**Boundary.** kernel only; workspace builds.

### FGA-C3 — SDK: named-choice bounds, per-field roles, place-free lookup, section requirements

**Goal.** S1–S4 with no change to any existing section's behaviour or message.
**Scope.** `sdk/rust/src/interactions/{section,lookup,mod}.rs` (and `decl.rs` if `FIELD_ROLES` lives there);
SDK tests. **Dependencies.** none (independent of C2).

- [x] Implementation: `section.rs` — `ParameterBound<T>` (`admits`, `describe`) with the `RangeInclusive`
  impl (`"{start} … {end}"`); `parameters!` grammar `$default:expr, $bound:expr`, refusal
  `'{field}' is {describe}, not {value}`; `Fields::set_names` (macro and `()`); `param()` collects named
  roles and calls `field_roles::<S>(set, named, in_region)` after the entry; region parameter lists decode
  with `in_region = true`. `mod.rs` — `FIELD_ROLES` (default `&[]`), `requires(section)` (default none),
  `HeldSection::requires` forwards; exports `ParameterBound`, `parameters_unplaced`. `lookup.rs` —
  `parameters_unplaced` (base resolution, place unfilled; compiled defaults when unconfigured).
- [x] Validation: two SDK unit tests — the range bound admits its ends and describes itself `1 … 100`,
  `-1000 … 1000`, `false … true`; `field_roles` refuses `reach` under `target` and in a region with the
  literal messages, accepts it under `actor`, and leaves an unlisted field free; `set_names` literal.
  Message parity, unedited and passing: `tests/acceptance` `interaction_schema` (4) and
  `configuration_seam` (8) (both pin `'step' is 1 … 100, not …`), `tools/cli --test interactions` (4,
  pins `'gap' is 1 … 86400, not 0` at line and column), `mineworld-worldpack` all suites; `cargo test -p
  mineworld-sdk` all ok; workspace `cargo check --all-targets` clean (every `parameters!` expands);
  `cargo clippy -p mineworld-sdk --all-targets -D warnings` clean. The decode-time refusals through YAML
  (named choice, restricted field in an entry and a region at line and column, `requires` reaching
  `check_requires`) and `parameters_unplaced` on a configured world are proven with the real packs in
  FGA-C4 … C7 (DV-8) (E-FGA-3).
- [x] Review: no existing `parameters!` invocation edited (`git diff` touches none); the new refusal is raised
  inside the entry's `visit_map`, so it keeps `DEP-10`'s position as the existing post-loop refusals do;
  nothing in the SDK names a pack; `FIELD_ROLES` empty by default, so every existing section decodes as
  before.
**Acceptance.** Existing sections decode, resolve and refuse byte-identically; the new refusals fire.
**Failure cases.** A bound's `describe` differing from today's for ranges → caught by the unedited message
tests. **Boundary.** SDK only.

### FGA-C4 — relationships: decay (curves, catalog, lazy reads)

**Goal.** §5.1–5.4: decay at contact and on disclosure, the catalog, the loader seam.
**Scope.** NEW `systems/relationships/src/decay.rs` (`CurveName`, `CurveCatalog`, `DecayCurve`,
`register_curves`, bundled curves, `HALF`); `component.rs` (`RelationshipValues::reading(&rule, at)`,
`rebase`); `interactions.rs` (the six decay fields — `known_at_most` lands in C5 — `FIELD_ROLES`,
`requires`, VERSION pin); `system.rs` (`apply` steps 3–7; `discloses` as-of; `current`); `lib.rs` exports and the "Values
never decay" section rewritten; `systems/installed/src/lib.rs` (I1); `worldpack/src/read.rs` (W1);
relationships' pinned test (VERSION 3); `tools/cli/tests/{inspect,social_composition}.rs` pins.
**Dependencies.** C2, C3.

- [x] Implementation: NEW `decay.rs` (`CurveName` — `Cow<'static, str>`, form-checked by `new`/`Deserialize`,
  `const` bundled names; `DecayCurve` object-safe; `register_curves`/`registered_curves` with presence's
  write-once rules plus a bundled-name collision panic; `Curve` (bundled or `&'static dyn` catalog entry);
  `HALF` 65 literals computed to 60 digits (scratch `half_table.py`, no ties); `Decay`/`DecayRule` — the host
  clamp `remaining(..).min(excess)`; `CurveCatalog` bound "one of none, linear, half-life, power[, …]");
  `component.rs` (`RelationshipValues::reading`/`rebase`, `Acquaintances::reading`); `interactions.rs` (six
  decay fields, `FIELD_ROLES` actor+target, `requires` → catalog curves' packs, `decay_rule` via
  `parameters_unplaced`, pinned test VERSION 3 + decay defaults); `system.rs` (VERSION 3; `apply`: rule
  lookup after `acquaint`, stated level before `rebase`, `rebase` before `adjust`; unknown curve →
  `FactRefusedByOwner { relationships, <cause>, System { unknown-decay-curve, "<curve> …" } }`; `discloses`
  as-of `world.at()`, withheld on an unknown curve; public `current`); `lib.rs` (docs, exports);
  `systems/installed` I1 line and docs; `worldpack/src/read.rs` W1; CLI pins relationships v3;
  `systems/installed/tests/resolution.rs` line count 2 → 3 (DV-10).
- [x] Validation: FA-4 — `src/decay/tests.rs` 9 tests, every literal of §6 FA-4 plus `65 536 → 46 341` at
  T/2 (see M-FA4 below), the monotone grid (4 curves × 5 excesses × 3 T × 141 steps), the host clamp with a
  greedy (+5 000) and an eager (0 at t = 0) curve, and name form. FA-5 — NEW `tests/forgetting.rs`
  (scripted café, half-life 1 day): (a) 150 disclosed, (b) 300 stored / last contact t0, (c) no relationships
  fact in the day, (d) 160 stored at t0 + 1 d + 1 s and `relationship-changed` Friendly → Acquaintance both
  ways caused by the spoke, (e) unconfigured twin discloses 300, same record shape. Existing relationships
  tests unedited and passing (6 + 11 lib); `tools/cli --test inspect --test social_composition` 3 + 4 ok;
  `mineworld-installed-systems` ok; workspace clippy `-D warnings` clean. Mutations (E-FGA-4): M-FA4 red by
  name, M-FA5a, M-FA5b, M-FA5c red by name; all reverted, `git grep MUTATION -- '*.rs'` empty.
- [x] Review: no float anywhere (`i128`/`u64` integer arithmetic); unconfigured, `parameters_unplaced` returns
  the compiled defaults (curves `none`) and `reading`/`rebase` return before computing, so stored values are
  disclosed and reduced unchanged; disclosure re-encodes the same `Acquaintances` type (shape asserted in
  FA-5(e)); contact, disclosure and (C5) ranking all go through `RelationshipValues::reading`. Finding
  F-FGA-17: M-FA4 as designed (T/2 at excess 1 000) survives arithmetically — 1 000 × 46 340 >> 16 is still
  707 — and was caught only by the table literal; the added excess-65 536 case makes the arithmetic itself
  red.
**Acceptance.** FA-4 and FA-5 pass; unconfigured, every relationships test's literals hold unchanged.
**Failure cases.** Unknown curve at reduction → `KernelError` naming it (journaled fault); at disclosure →
record withheld (§5.3). `elapsed` negative → 0. **Boundary.** No cap yet; no conversation.

### FGA-C5 — relationships: the cap

**Goal.** §5.5: `known_at_most`, eviction, `acquaintance-forgotten`.
**Scope.** `event.rs` (`AcquaintanceForgotten`; `BecameAcquainted` doc "once per formation");
`interactions.rs` (`FactDecl`, the `known_at_most` field with roles `[actor]`); `system.rs` (declaration `.emitting`, `apply` step 9,
`BIOGRAPHICAL`); `component.rs` (`Acquaintances::evict_beyond(at_most, &rule, at, keep) -> Vec<PersonId>`);
`lib.rs`.
**Dependencies.** C4.

- [x] Implementation: `event.rs` (`AcquaintanceForgotten { person, counterpart }`; `BecameAcquainted` "once
  per formation"); `interactions.rs` (`known_at_most: u16 = 150, 5 ..= 1_500`, `FIELD_ROLES` actor only, the
  third `FactDecl`, `known_at_most()` lookup with the holder alone, place-free; module doc; pinned test
  150); `component.rs` (`Acquaintances::evict_beyond(at_most, rule_of, at, keep)` — rank (current
  familiarity, last contact, counterpart id), the newcomer excluded); `system.rs` (declaration
  `.emitting::<AcquaintanceForgotten>()`, `BIOGRAPHICAL` all three, `apply` step 9 through
  `beyond_the_cap` and `unknow` — the edge removed or `FactRefusedByOwner { acquaintance-without-edge }`;
  forgotten facts after the other two, through `fact()`); `lib.rs` (table, export).
- [x] Validation: FA-3 in `tests/forgetting.rs` with NEW `tests/support/crowd.rs`: (a)–(c) a 300-person crowd
  (100 speakers × 200 listeners, DV-11), 20 000 talks over 100 simulated days: after every talk both
  holders ≤ 150 and edges = entries; exactly 5 000 `acquaintance-forgotten`, each `(speaker, listener
  r − 150)`, caused by the spoke, the victim gone from entries and edges; at the end each speaker knows
  listeners 150 … 299; widest person 29 561 B of component data (bound 163 840 B), p000's acquaintances
  ≤ 118 247 B as the snapshot encodes them. (d) cap 5, eight people: victims p1 (tie on familiarity and
  instant → lower id), p3 (tie on familiarity → older contact; newcomer p7 at 10 kept), p7 (no longer the
  newcomer); p1 met again states `became-acquainted` again. "The edge and the entry agree" unedited and
  passing; `cargo test -p mineworld-relationships` all ok (3.5 s for FA-3); workspace clippy clean.
  Mutations M-FA3a, M-FA3b, M-FA3c red by name (E-FGA-5).
- [x] Review: the victim is ranked by `RelationshipValues::reading` at the causing fact's instant; the fact
  goes through `fact()` — about the holder, participants holder and victim, at the cause's place, audience
  via `consequence` — and the kernel records it as caused by the cause (asserted); ranking runs only when a
  new entry takes the list past the cap. Finding F-FGA-18: the script's first oracle for (d) step E was
  wrong (it kept protecting p7 after the reduction that formed it); the run followed §5.5, the oracle was
  corrected and the correction is written in the test's doc.
**Acceptance.** FA-3 passes. **Failure cases.** `known_at_most` below the current size cannot happen (fixed at
genesis); `unrelate` reporting "absent" is a broken invariant → `KernelError`. **Boundary.** relationships
only.

### FGA-C6 — conversation: `forget_after_days`

**Goal.** §5.7. **Scope.** `systems/conversation/src/{interactions,component,system}.rs`; pinned test
(VERSION 4); the CLI pins. **Dependencies.** C2, C3.

- [x] Implementation: `interactions.rs` (`forget_after_days: u16 = 0, 0 ..= 3_650`, `FIELD_ROLES` target only,
  `kept_for(read, listener)` — place-free, `None` at 0; module doc; pinned test VERSION 4 and default 0);
  `component.rs` (`ConversationHistory::forget_older_than(cutoff)`, `pub(crate)`); `system.rs` (VERSION 4;
  `react` drops lines older than `at − kept` after `remember_within`; `discloses` shows a filtered copy at
  `world.at()` and nothing when every line aged out); CLI pins conversation v4.
- [x] Validation: FA-9 — NEW `systems/conversation/tests/forgetting.rs` (sam, resident rita, commuter carl;
  `{ target: resident, forget_after_days: 1 }`): shown at t0 + 1 d, not shown at t0 + 1 d + 1 s, still stored,
  dropped by the next line; carl keeps both; the unconfigured twin keeps everything; conversation-started per
  line equal to the twin's, `[true, true, false, true, true]`. Existing conversation tests unedited and
  passing (14 + 2 lib + 1); CLI `inspect` 3, `social_composition` 4; workspace clippy clean. M-FA9 red by
  name (E-FGA-6).
- [x] Review: `continues_a_conversation` / `within_the_gap` untouched (a dropped line is older than a day ≥
  every legal `gap`); default 0: one `parameters_unplaced` per reduction and per disclosure, which returns
  the compiled default without reading state when unconfigured; the only new mutator is called only from
  `react` (disclosure filters a clone).
**Acceptance.** FA-9 passes. **Boundary.** conversation only.

### FGA-C7 — The catalog and the loader, scripted

**Goal.** FA-7 and FA-8: refusals through the real loader and a user curve through the catalog.
**Scope.** NEW `systems/relationships/tests/curve_catalog.rs` (own binary); loader refusals in a
`tools/cli/tests` or `worldpack/tests` file following `social_sections.rs`'s scratch pattern.
**Dependencies.** C4–C6.

- [x] Implementation: NEW `tools/cli/tests/forgetting_sections.rs` (FA-7: scratch social-cafe copies through
  `WorldPack::read`/`load`, nine refusals + two accepted sections); NEW
  `systems/relationships/tests/curve_catalog.rs` (FA-8: one `#[test]`, its own process).
- [x] Validation: FA-7 — each refusal by its literal words at `error: line 2 column …` (line 3 for the
  region): unknown curve ("'familiarity_decay' is one of none, linear, half-life, power, not fast"),
  malformed name, `place: place` setting `familiarity_decay_days`, a region setting `known_at_most`,
  `target:` setting `known_at_most`, conversation's `actor:` setting `forget_after_days`, `known_at_most` 4
  and 1 501, `familiarity_decay_days` 0; the same fields scoped as allowed (base, `target:`, `actor:`, a
  region setting an increment) read and load. FA-8 — impostor `half-life` panics naming it before anything
  registers; `[step-down, greedy]` registers (same list again a no-op; `registered_curves` sorted); a
  different list panics naming both lists; the section naming `step-down` requires `test-step-down`
  (through `decode_configuration(..).requires()`, what `check_requires` asks), a bundled curve requires
  nothing; an unknown name is refused listing all six curves; the scripted café discloses 300 a second
  before the time constant and 150 at it; greedy is clamped to 300. S4's refusal by name is the loader's
  existing `check_requires` (worldpack `configure/tests.rs`), reached through `requires()` as shown — the
  real installed set has no user curve, so a real-loader run with an unenabled curve pack is not
  constructible without a test-only installed set (DV-12). M-FA7, M-FA8 red by name (E-FGA-7).
- [x] Review: the test follows the README's five steps (pack + `Default`, registration standing in for the
  installed line, the requirement that enables the pack, the name, refusals) minus the rebuild; the catalog
  is process-wide, and only `curve_catalog.rs` registers a non-empty list — every other test binary that
  registers uses the real installed set's empty line.
**Acceptance.** FA-7, FA-8 pass.

### FGA-C8 — Through the binary

**Goal.** FA-2 and FA-6. **Scope.** NEW `tools/cli/tests/forgetting_runs.rs`. **Dependencies.** C4–C6.

- [ ] Implementation: scratch copies; one unconfigured twin shared by the cases.
- [ ] Validation: FA-2, FA-6 with INCONCLUSIVE rules as written; M-FA6 observed.
- [ ] Review: no assertion depends on a wall clock or a platform path; scratch through `scratch!`.
**Acceptance.** FA-2, FA-6 pass.

### FGA-C9 — Close: byte identity, cost, guards, gate, ledger

**Goal.** FA-1, FA-10, FA-11, FA-12; `baselines.txt`; ledger and handoff.
**Scope.** `scripts/baselines.txt` (journal-300s, reason line); `docs/MVP_STATUS.md`; §15; handoff.

- [ ] Implementation: E-FGA-0 (captured before C2 on the base: a long record and four 30-day saves with the
  base release binary); the head's long record; `baseline check`; the row-level proof; `baseline write`.
- [ ] Validation: FA-1 (M-FA1a, M-FA1b), FA-10 medians, FA-11 guard diffs, FA-12 full gate; CI on the exact
  head.
- [ ] Review: every changed path is in §11 or recorded in §16; every `[x]` has evidence.
**Acceptance.** FA-1 … FA-12 PASS on the exact head.

---

## 11. Change set and paths with no diff

```text
docs/DECISIONS.md                    dated notes: ARC-62, ARC-63, ARC-28 (and ARC-86 only per §9)
docs/MODULE_SPEC.md                  §4.2
docs/MVP_STATUS.md                   S24 row
kernel/src/{view,dispatch,world}.rs, kernel/src/system/tests.rs           K1
sdk/rust/src/interactions/{section,lookup,mod,decl}.rs, sdk tests         S1–S4
worldpack/src/read.rs                W1 (one call)
systems/installed/src/lib.rs         I1 (one line)
systems/relationships/src/{decay.rs NEW,component,event,interactions,system,lib}.rs, README.md,
  tests/{forgetting.rs NEW, curve_catalog.rs NEW}, tests/support (if a 300-person builder is needed)
systems/conversation/src/{interactions,component,system}.rs, README.md, tests/
tools/cli/tests/{inspect,social_composition}.rs   version pins
tools/cli/tests/forgetting_runs.rs NEW; loader refusal tests (FA-7)
scripts/baselines.txt                journal-300s, four worlds, one reason line
.structured-coding/plans/mvp0/{pr-fg-a-relationship-forgetting,handoff-fg-a}.md
```

No diff: `contracts/`, `persistence/`, `server/`, `clients/`, `presentation/`, `cognition/`, `authoring/`,
`worlds/**`, `systems/presence/` and every other pack, `tools/cli/src/`, `tests/acceptance/**`, root
`Cargo.toml`, `Cargo.lock`. A needed edit there is a material stop (§14).

---

## 12. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-FGA-1 | S1's grammar change breaks a `parameters!` invocation or changes a refusal message. | Range bounds keep today's message byte for byte; every existing refusal test runs unedited (C3); `cargo check --all-targets` over the workspace. |
| R-FGA-2 | `WorldRead::at()` is read as a wall clock, or a site passes the wrong instant. | One test per construction site (C2); the doc names world time (`INV-12`). |
| R-FGA-3 | Disclosure cost with decay configured at large populations (≤ 150 readings per holder per observation). | Unconfigured short-circuit; FA-10 bounds the configured cost at 1.10×; a per-pair-class cache only if FA-10 fails (bounded fix, recorded). |
| R-FGA-4 | `baselines.txt` collides with 12d or FG-d. | §9's rule: the second to merge re-records with reason lines kept. |
| R-FGA-5 | A user curve is non-monotone: a value appears to recover between contacts. | The host clamps to `[floor, value]` (never above the stored value); monotonicity is the trait's documented contract and FA-8 ships the property-test template; QFGA-3 makes curve code part of the composition. |
| R-FGA-6 | Re-basing at frequent contacts rounds down at each step. | Deterministic and replayed identically; stated in the README; the size of the effect is ≤ 1 per contact. |
| R-FGA-7 | The 160 KiB per-person bound is ambiguous (payload vs snapshot encoding; §3.1). | QFGA-7; FA-3(c) reports both. |
| R-FGA-8 | Step-24 and the draft `ARC-86` state a false premise (F-FGA-1) and an unqualified byte-identity claim (F-FGA-8). | Routed to DOC-24 (§9); FA-1 states the exact claim. |
| R-FGA-9 | The newcomer rule hides a defect where a person with a full list never keeps new people. | Intended (the newcomer always stays; the weakest old tie goes); FA-3(d) names it. |
| R-FGA-10 | The scope (kernel, SDK, two packs) is too wide for one PR. | Commits are separable; QFGA-12 offers a split (FG-a1 seams + decay, FG-a2 cap + conversation). |

---

## 13. Operator questions (each with a recommendation)

| ID | Question | Recommendation |
| --- | --- | --- |
| **QFGA-1** | Lazy decay on read needs the read's instant (F-FGA-4). (a) Kernel: `WorldRead::at()`; (b) change `PerceptionProvider::discloses` to take the instant (13 packs edited); (c) a second, defaulted `discloses_at` method on presence's trait. | **(a).** The kernel already owns time; one accessor, four crate-private sites, no domain meaning; DL-c's `needs` reuses it. (b) is change amplification, (c) an adapter before a stable contract. |
| **QFGA-2** | Approve the four additive SDK changes S1–S4 (named-choice bounds, per-field roles, place-free lookup, a section's required systems). Without S2/S3 a `place:`-scoped decay entry would be silently ignored; without S1 a curve cannot be named. Alternative to S1: relationships hand-writes its `Parameters`/`Fields` impls (~90 lines duplicating the macro). | **Approve S1–S4.** Each is generic, keeps every existing message, and has a second user in sight (WE-a's kinds, DL's model parameters). |
| **QFGA-3** | A user curve is a pack implementing `DecayCurve` on relationships' `ARC-62` line. Must the world enable that pack in `systems:` to select its curve (enforced through S4)? | **Yes.** It puts the curve's identity and version into the save's composition, so a different curve refuses resume by name instead of diverging. |
| **QFGA-4** | AC-FG-6 says "baselines unchanged". The genesis journal row carries the composition (F-FGA-8), so `journal-300s` moves in all four worlds while every summary and fact stays. Accept FA-1's refinement: summaries and facts unchanged; journal rows ≥ 2 proven identical; `journal-300s` re-recorded with a reason line. | **Accept.** Same mechanism and handling as IL-e (#159) via 13c's F-13c-impl-3. |
| **QFGA-5** | `acquaintance-forgotten` compiled biographical? | **Yes**, mirroring `became-acquainted`; a world may switch it off per class. Inert in the reference worlds. |
| **QFGA-6** | Bundled curves `none`, `linear`, `half-life`, `power` (β = 1); one time constant and floor per value; `*_decay_days` default 365 (uncalibrated, inert while `none`); no per-curve parameter types; data-defined curves later. | **Accept.** Every number a world opts into is its own; no decay rate in the literature was verified (step-24 §6.6). |
| **QFGA-7** | Step-24's "≤ 160 KiB per person in a snapshot": measured on the component payload (≈ 46 KB worst case) or on the snapshot's JSON encoding (≈ 164 KB worst case, over the bound; §3.1)? | **Payload.** The snapshot's byte-array JSON is a persistence encoding (F-FGA-15, routed to FG-d), compressed on disk. FA-3(c) reports both. |
| **QFGA-8** | Keep conversation's `forget_after_days` in FG-a (step scope), though the history is already FIFO-capped? It bumps conversation and moves bodies-yard's journal genesis row too. | **Keep.** It is small, makes aging available without raising `remembered`, and shares the seams. |
| **QFGA-9** | A level crossed by decay is stated only by the next reduction touching the pair, as one net `relationship-changed` from the last stated level. No `relationship-faded` fact. | **Accept** (step-24 §8.2). |
| **QFGA-10** | Disclosure: same `acquaintances` record with current values, vs a record with added `*_now` fields. | **Same record.** No reader changes; the observation is the reading, not the state. |
| **QFGA-11** | Decision records: dated notes on `ARC-62`, `ARC-63`, `ARC-28` (and the `ARC-86` correction if DOC-24 does not make it); no new number. | **Notes only.** `ARC-86` is FG-a's record. |
| **QFGA-12** | One PR, or split: FG-a1 (K1, S1–S4, W1, decay) and FG-a2 (cap, conversation, binary proofs)? | **One PR**, nine commits, each reviewable alone; split only if the operator prefers smaller reviews. |

### 13.1 Rulings (2026-10-12, operator, relayed by the primary session)

| Question | Ruling |
| --- | --- |
| **QFGA-1** | **Yes** (operator): add the read-only `WorldRead::at()` to the kernel (K1). |
| QFGA-2 … QFGA-12 | **Accepted as recommended**: S1–S4 approved; a user curve's pack must be enabled (S4); FA-1's refinement of AC-FG-6; `acquaintance-forgotten` compiled biographical (it enters biographies); §5.1's curve set and defaults; **the 160 KiB per-person bound is measured on component data** (FA-3(c) still reports the snapshot encoding beside it); `forget_after_days` stays in FG-a; net level facts at contact; same-shape disclosure; dated notes only, no new decision number; **one PR**. |
| DOC-24 | The primary session is correcting DOC-24 (#175) per F-FGA-1 and F-FGA-8 (§9's preferred route), so FGA-C1 adds no `ARC-86` correction note unless the merged text still carries either premise. |

Consequence: implementation starts only on the operator's go, after DOC-24 merges (§14).

---

## 14. Execution contract for PR FG-a (filled at the freeze, 2026-10-12)

```text
PROJECT / PR        MVP-0 · S24 / PR FG-a — System-owned forgetting in relationships and conversation
FREEZE              DESIGN FROZEN 2026-10-12 (primary session); operator ruled QFGA-1 yes; QFGA-2 … 12
                    accepted as recommended (§13.1)
START CONDITION     the operator's explicit go AND DOC-24 (#175) merged with F-FGA-1/F-FGA-8 corrected;
                    neither alone starts implementation
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-fg-a-relationship-forgetting.md; evidence §15
                    (E-FGA-<n>); deviations §16
RELATED / BINDING   mvp1/step-24-decision-and-memory.md §§1.2, 2.4, 3.2, 8.2, 10, 11.2, 14.1; overall.md;
                    pr-il-e-social.md (as merged); DECISIONS ARC-5, ARC-23, ARC-25, ARC-28, ARC-61, ARC-62,
                    ARC-63, ARC-64, ARC-65, ARC-86, DEP-10; MODULE_SPEC §4.2; CLAUDE.md §§2–4
IMPLEMENTATION BASE main after DOC-24 (#175) merges (exact commit in FGA-C1's evidence; re-audit §3);
                    branch mvp0/pr-fg-a from main; worktree /Users/yuema137/mineworld-worktrees/impl-fg-a;
                    one worktree, one session, a fresh one
COMMANDS            cargo (fmt, check, clippy -D warnings, test, build --release -p mineworld-cli), the
                    built `mineworld` binary, git, gh (never merge), python3 scripts/*, sqlite3 (read-only
                    queries on scratch saves), shasum, mkdir -p, sed -n; no python3 -c, sed -i, awk, xargs,
                    curl or heredoc writes; files edited with the editor tools; no other worktree; no
                    .claude/settings*
APPROVED SCOPE      §1, §4, §5 as answered by §13.1; FGA-C1 … FGA-C9; kernel change limited to K1
                    (read-only WorldRead::at()), SDK changes limited to S1–S4
FROZEN INVARIANTS   no fact, Process, deferred event or journal row for decay; forgetting only lowers a value
                    toward its floor; known.len() ≤ known_at_most after every reduction; edges = entries;
                    disclosure holder-only and same-shaped; no model in this layer; unconfigured: every
                    summary and fact byte-identical (FA-1); integer arithmetic only; §11's no-diff list
SEQUENCE            E-FGA-0 before FGA-C2; C1 → C9 (C2 and C3 in either order; C6 after C2/C3), each
                    committed and pushed when coherent
VALIDATION BUDGET   two long parity records (E-FGA-0 base, FA-1 head), release profile; eight 30-day saves
                    (four worlds × base/head); 30-day scratch runs uncounted; FA-10's six 30-day walls; one
                    full workspace gate on the final head; total ≤ ~1 hour of runs. Real-model NOT REQUIRED
LIVE DOCUMENTATION  §10 checkboxes; §15 ledger; §16 deviations; handoff-fg-a.md
ENDPOINT AUTHORITY  implementation + local validation: on the operator's go only (source: primary session's
                    freeze message, 2026-10-12: "implementation starts only on the operator's go");
                    semantic commits, branch push, PR creation/update marked READY FOR OPERATOR REVIEW, CI
                    repair: authorized once started (source: working rules §§14, 21, confirmed by the
                    freeze); merge: operator only, merge commit (source: CLAUDE.md §3.1; no value changes it)
POST-MERGE SYNC     this session: the PR document, merge identity, evidence; the primary session: step-24
                    and overall.md
NORMAL STOP         PR FG-a READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       an edit in §11's no-diff list (contracts, persistence, server, clients, cognition,
                    authoring, worlds, another pack); any summary or fact change with default content not
                    explained by main moving; an SDK or kernel change beyond §5.6; a falsified invariant
                    above; an answer to QFGA-1 … QFGA-12 other than §13.1's; DOC-24 merging with text that
                    contradicts §5 (stop and report, do not reconcile silently)
```

---

## 15. Evidence ledger (E-FGA)

```text
E-FGA-d0  2026-10-12, planning session. §3.1's measurements: binary /Users/yuema137/MineWorld/target/debug/
          mineworld (built 21:50 from main; no non-Markdown change 0de1de4 → cda4c13); social-cafe,
          market-town, lakeside 30 days seed 7 --save into /Users/yuema137/mineworld-worktrees/scratch-fg-a;
          sqlite3 counts of became-acquainted / relationship-changed; newest snapshot extracted (sqlite3
          writefile, zstd -d) and read with jq. Results as tabulated in §3.1. Run walls 5.6 / 9.6 / 4.2 s,
          faults 0; fingerprints af1927a1e2182bcd / 346dcaa6291381a0 / 60fd886043b1436a (FNV, for reading).
E-FGA-0   (implementation) base capture: long parity record and four 30-day saves, base release binary.
          Base: origin/main @ 6306977 (DOC-24 #175 merged at baa0d33; §3 re-audit: none of §1's base paths
          changed between cda4c13 and 6306977 except Markdown). Binary `cargo build --release -p
          mineworld-cli` (1 m 58 s) copied to /Users/yuema137/mineworld-worktrees/scratch-fg-a-impl/base/.
E-FGA-1   2026-10-13, FGA-C1. Merged ARC-86 audit: F-FGA-1 corrected by DOC-24 ("no component encoding
          changes"); F-FGA-8 still unqualified ("Defaults change no reference world's bytes") and the
          clock described as "available only during disclosure" → one dated note on ARC-86 (§13.1's
          DOC-24 row allows it). Doc scripts: 223 sections / 122 ids, exit 0.
E-FGA-0   (cont.) Long record: `uv run --locked python scripts/ci_parity.py record --binary <base>/mineworld
          --profile long --out <base>/long.record`, 995 s (bodies-yard 322, lakeside 111, market-town 332,
          social-cafe 229), Darwin/arm64, rustc 1.97.1. `baseline check` → FAIL on every key of every
          world (F-FGA-16 below). 30-day saves, base binary, seed 7: social-cafe (5.2 s), market-town (8.0
          s), bodies-yard (9.2 s), lakeside with `--packs entities --packs presentation/mineworld-default`
          (3.9 s), into <base>/<world>-30.
F-FGA-16  baselines.txt is stale on main. The base record's digests equal, key for key, the drift printed
          by the nightly of the same commit (run 38440079845 on 6306977; issue #170 "Nightly CI is red on
          main", "step baseline failed"): e.g. social-cafe summary-300 341 9500d0c8…, facts-300s 837953
          c0d3050f… (baseline 338 ad49c723…, 365330 9ffcfcb1…). Cause outside FG-a (walking towns, 12n-2,
          merged without a re-record). Consequence: FA-1(a)'s oracle is E-FGA-0's base record, not
          baselines.txt (DV-9); who re-records baselines.txt follows §9's rule (the second of FG-a and 12d
          to merge), decided at FGA-C9 against main at that time.
E-FGA-2   FGA-C2: `cargo test -p mineworld-kernel` (all suites ok, incl. NEW read_instant 3/3);
          `cargo clippy -p mineworld-kernel --all-targets -- -D warnings` clean. Commit 7d6c0c4.
E-FGA-3   FGA-C3: `cargo test -p mineworld-sdk` (lib 11 incl. 2 new; integration suites ok);
          `cargo test -p mineworld-acceptance --test interaction_schema --test configuration_seam` 4 + 8 ok;
          `cargo test -p mineworld-worldpack` all ok; `cargo test -p mineworld-cli --test interactions` 4 ok;
          `cargo check --workspace --all-targets` clean; SDK clippy clean.
E-FGA-4   FGA-C4 mutations, each applied in the working tree, run, reverted:
          M-FA4  HALF[32] 46 341 → 46 340: decay::tests::half_life_… (the 65 536 case, tests.rs:29) and
                 …the_table_has_its_literal_… FAILED; the 1 000-at-T/2 literal alone survives (F-FGA-17)
          M-FA5a disclosure encodes the stored record: forgetting.rs:118 left 300, right 150 — FAILED
          M-FA5b no rebase before adjust: forgetting.rs:132 left (310, 140, …), right (160, 140, …) — FAILED
          M-FA5c stated level read after the rebase: forgetting.rs:143 left [] (no crossing) — FAILED
          After: `git grep MUTATION -- '*.rs'` empty; relationships suites ok.
E-FGA-5   FGA-C5 mutations:
          M-FA3a rank by highest familiarity: the_cap_evicts_… FAILED at forgetting.rs:348 (p1 never
                 evicted, so no second became-acquainted)
          M-FA3b no unrelate: a_crowd_of_300_… FAILED at :218 (edges ≠ entries) and the_cap_evicts_…
                 FAILED at :353
          M-FA3c the newcomer eligible: the_cap_evicts_… FAILED at :351, victims [p1, p7, p1] for
                 [p1, p3, p7]
          After: `git grep MUTATION -- '*.rs'` empty; `cargo test -p mineworld-relationships` all ok.
E-FGA-6   FGA-C6: M-FA9 (disclosure not filtered): conversation forgetting.rs:201 "older than a day: never
          shown" left ["first"], right [] — FAILED; reverted, `git grep MUTATION` empty.
E-FGA-7   FGA-C7: M-FA7 (S2's `field_roles` call skipped): forgetting_sections.rs:133 "fa7-place-decay:
          accepted" — FAILED (with `place: cafe` the copy was refused for an undefined class instead,
          still red but for another reason; the case was changed to the implicit class `place` so the
          mutation makes it load). M-FA8 (host clamp `.min(excess)` → `.min(u32::MAX)`):
          curve_catalog.rs:117 left 1000 right 300 (re-basing with greedy inflates every contact) and
          decay::tests::the_host_clamps_… left 5600 right 600 — FAILED. Reverted; `git grep MUTATION`
          empty; relationships and CLI suites ok.
```

---

## 16. Deviations from the step design (recorded, bounded)

| ID | Step-24 says | This design | Why |
| --- | --- | --- | --- |
| DV-1 | "Storing `last_contact` changes the `acquaintances` component's encoding … pack version bumped" (§8.2) | The component's encoding is unchanged; the version is bumped for the section's payload and the new fact | F-FGA-1 |
| DV-2 | AC-FG-6: "`scripts/baselines.txt` is unchanged by FG-a" | Summaries and facts unchanged; `journal-300s` re-recorded at the genesis row | F-FGA-8; QFGA-4 |
| DV-3 | `DecayCurve` with `type Parameters`, `const ID`, `decayed(p, value, floor, elapsed)` | Object-safe `name`/`remaining`/`curve_of`; floor and time constant in the section; host clamps | D-FGA-3 |
| DV-4 | `familiarity_half_life_days` | `*_decay_days`, one time constant per value for every curve | D-FGA-3, D-FGA-4 |
| DV-5 | "A synthetic 300-person world (test-only pack)" | A scripted 300-person world in relationships' tests | D-FGA-10 |
| DV-6 | Parameters "scoped by class like every IL parameter" | Scoped by actor/target class only; place and regions refused for these fields | F-FGA-6 |

Implementation deviations (bounded, recorded during execution):

| ID | Plan says | Done | Why / impact |
| --- | --- | --- | --- |
| DV-7 | FGA-C1 updates the `docs/MVP_STATUS.md` S24 row | No S24 row exists on main; the row is added in FGA-C9, when the PR's state is known | A status row written before the code would state an untrue state; one file, no scope change |
| DV-8 | FGA-C3's SDK unit tests decode a test section (named choice, place entry, region, `requires`) | The SDK unit tests cover the bound, `set_names` and `field_roles` directly; decoding through YAML at line and column is proven with the real sections through the loader (FGA-C7, FA-7) | The SDK has no YAML or JSON decoder among its dependencies; adding one changes `Cargo.lock` (FA-11). Its existing tests say the same: "decoding with line and column is proven through the loader" |
| DV-10 | §11 lists `systems/installed/src/lib.rs` only | `systems/installed/tests/resolution.rs` pins the number of extension lines; 2 → 3 with its message | A pinned fact that I1 moves on purpose (D-IB-12 precedent); no behaviour |
| DV-11 | FA-3: "person i talks to 200 distinct others" (all 300) | 100 speakers each talk to the same 200 listeners, once per pair | 300 people × 200 distinct partners is 60 000 talks over 44 850 pairs, so pairs repeat and the forgotten count could only be predicted by re-implementing the ranking; the chosen layout keeps 300 people, 200 contacts per speaker, 100 days and the default cap, and gives the hand-derived literal 5 000 |
| DV-12 | FA-7: "a catalog curve whose pack the world does not enable: refused by name (S4)" through the loader | FA-8 shows the section's `requires()` (what `check_requires` reads) naming the curve's pack; the refusal by name is `check_requires`'s existing, tested behaviour | The real installed set's curve line is empty and a process registers one catalog; a real-loader run with a user curve needs a test-only installed set, which `ARC-62`'s per-process rule makes a separate binary with its own `installed!` — more machinery than the claim needs |
| DV-9 | FA-1(a): head summaries and facts "equal to scripts/baselines.txt" | Equal to E-FGA-0's base record (main @ 6306977) | F-FGA-16: `baselines.txt` is stale on main for every key (nightly red, issue #170); the base record equals the nightly's own record of the same commit digest for digest |

---

## 17. Process notes (this planning session)

- Measurements used the main checkout's prebuilt debug binary read-only; nothing in
  `/Users/yuema137/MineWorld` was edited. The scratch saves in
  `/Users/yuema137/mineworld-worktrees/scratch-fg-a` were removed at the freeze (primary session's
  instruction); §3.1 and E-FGA-d0 keep the commands and results, and FGA-C9's E-FGA-0 re-measures.
