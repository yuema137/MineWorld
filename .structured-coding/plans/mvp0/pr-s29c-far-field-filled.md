# PR S29-c — The far field's `filled` slot: a town's station and landmark recorded where they stand

## DRAFT — not frozen

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29c, branch docs/pr-design-s29c,
                        planning base origin/main @ 44f050e3, fast-forwarded to origin/main @ 70508683
                        (#281; nothing between them touches a file this design names)
Approved by / evidence: none yet. The step's rulings (Q-S29-1 … Q-S29-10, step-29 §13.1), S29-a's rulings
                        (Q-S29a-1 … 7) and S33-a's Q-S33a-1 bind and are not re-asked
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S29-a,
                        RL-e0b and S33-a
Start condition:        step-29 §11.2: nothing in S29 starts until Milestone F's checklist has been handed to
                        the operator; then (1) RL-e0 merged (MET: #220, merge 621ea0e2); (2) S29-a merged
                        (NOT MET at 2026-10-18: pr-s29a-town-contract.md is DRAFT revision 3, not frozen);
                        (3) this design frozen; (4) the dispatch
Execution contract:     §12 (ends at MERGE-READY: pushed, `fast` green on the exact head, reported with the
                        PR number and head SHA; the coordinator merges)
Implementer:            Haiku-class (step-29 §11.1, the S29-c row: "Haiku (additive schema on a frozen
                        pattern)"), one commit at a time from §9
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-29-station-landmark.md`](step-29-station-landmark.md) (APPROVED 2026-10-15)
§§0.2, 2.1 (A-10), 6.3 (the far field bullet), 6.7 (all), 9.3 (`packages` far-field tests), 9.5 (X-4), 10
(I-S29-1, I-S29-6, I-S29-7), 11.1 (the S29-c row), 11.3 (AC-S29-1, AC-S29-4), 14.2 (§6.1.1), 14.5 (`ARC-107`) ·
**Parent:** [`overall.md`](overall.md) · **Depends on:** [`pr-s29a-town-contract.md`](pr-s29a-town-contract.md)
(DRAFT; `settlement:`, `WorldPack::settlement()`, the halt's places) · **Consumes:**
[`../mvp1/pr-rl-e0-far-field-mandatory.md`](../mvp1/pr-rl-e0-far-field-mandatory.md) (merged as #220; format 1)
· **Coordinates with:** [`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) (FROZEN; rewrites
Lakeside's far fields after this PR), [`../mvp1/pr-rl-e0b-far-field-polish.md`](../mvp1/pr-rl-e0b-far-field-polish.md)
(FROZEN, in implementation at `impl-rl-e0b`; edits `social-cafe.yaml` and `far_field_view.gd`), S29-d (re-shoots
every `ff_*`; designed in parallel at `design-s29de`) · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 3, 4, 7,
8, 9, 13, 14); `docs/MODULE_SPEC.md` §6.1.1; `docs/decisions/ARC-102.md`, `ARC-105.md`, `ARC-107.md`.

**Decision ids.** None new. `ARC-107` already exists as a record (`docs/decisions/ARC-107.md`). C1 adds one dated
note inside it, `## ARC-107 note — 2026-MM-DD: where the town rule runs, and Lakeside's content` (§5.8). The
reserved `ARC-191` is **not used**.

Local ids: `A-n` audit anchors, `F-S29c-n` findings, `SD-S29c-n` bounded decisions, `I-c-n` invariants, `AC-c-n`
acceptance, `M-c-n` planted mutations beyond the step's X-4, `R-c-n` risks, `Q-S29c-n` operator questions.

No hosted model, image model or paid API is called by this design or by anything it specifies.

---

## 0. Binding inputs, exactly restated

- **Scope (step-29 §11.1, S29-c row).** "RL-e0 schema `filled` in Rust and GDScript; `FAR_FIELD_SLOT_TOWN`;
  Social Café 3D and Lakeside 3D content; Social Café `reach_m` 60 → 100; corpus files; MODULE_SPEC §6.1.1
  amendment; `ARC-107`."
- **Integration checkpoint (same row).** "`mineworld validate worlds/lakeside --packs …` passes; the slice loads;
  RL-e0's rendered check is still green (the footprint is clear of massing)."
- **Adversarial criteria (same row).** "X-4; a `filled` slot under `kind: none` is refused."
- **X-4 (step §9.5).** "set Social Café's 3D `station` slot back to `reserved`" → "`FAR_FIELD_SLOT_TOWN { slot:
  station }` at validate and at slice load".
- **The slot (step §6.7).** `{"state": "filled", "place": PlaceKey, "bearing_deg": f64, "distance_m": f64 > 0,
  "footprint_m": f64 > 0, "source": string}`. A filled slot "names near content"; its `distance_m` "**may** be ≤
  `boundary.reach_m`"; "the massing keeps clear of its footprint"; in a 3D far field of a town "both slots must be
  `filled`", with `station.place == settlement.station` and `landmark.place == settlement.landmark`, otherwise
  `FAR_FIELD_SLOT_TOWN { pack, world, slot }`; under `kind: none`, `filled` is refused (`FAR_FIELD_SLOT`).
- **Default content (step §6.7, §6.3).** Social Café: both slots filled, place `station`, 270°, 70 m, footprint
  60 m, each `source` citing §2.2 and the compression; `reach_m` 60 → 100 "to contain the platform and its fence".
  Lakeside: `station` filled with the halt's design position, `landmark` with `pier`'s; "design content; lakeside
  declares no real location"; S33-a replaces it with cited positions.
- **Operator principles (2026-10-18).** Every world has a far field, no empty horizon. Mechanisms are generic and
  data-driven; every acceptance criterion names a second world or a generated variant. Speed first: per-PR
  validation is the build, the tests the commit writes, and `fast` CI; acceptance runs and planted mutations run
  at the milestone.

---

## 1. Goal

A far field today can only say where the station *will* stand far away (`reserved`, beyond the boundary). After
step-29's ruling (Q-S29-1), a town's station stands at the street's end, inside the boundary, and the player walks
into it. The far field must record that fact, and it must agree with what the world declares.

Plainly: the 3D far-field file gains a third slot state, `filled`. A filled slot says "this slot's building is a
real Place of the world, named here, standing at this bearing and distance". The validators then hold one rule:
**a town's 3D far field fills both slots, with the places the town declares; a world that is not a town fills
none.** A world that drops its station from its far field, or a far field that names the wrong place, is refused
by name.

---

## 2. Scope

### 2.1 In scope

1. **The schema (both validators).** `Slot::Filled` in `packages/src/far_field_schema.rs`; the same state in
   `clients/shared/environment/far_field.gd`. Its citations, ranges and the inside-boundary exception.
2. **The file-level town rule (both validators, client load included).** A file with exactly one filled slot is
   refused `FAR_FIELD_SLOT_TOWN`. This needs no world (SD-S29c-1).
3. **The world-level town rule (Rust, the authoring gate).** `mineworld validate <world>` and `mineworld packs
   validate <world>` check every 3D far field of the world against its `settlement:`: `FAR_FIELD_SLOT_TOWN` and,
   under `kind: none`, `FAR_FIELD_SLOT` (SD-S29c-2, Q-S29c-1).
4. **The drawing.** The 3D view keeps massing clear of a filled slot's footprint, as it does for a reserved one.
5. **Content.** `presentation/mineworld-default/3D/far_field/social-cafe.yaml` (both slots filled; `reach_m` 100;
   the massing `from_m` that RL-e0b's rule then needs, Q-S29c-2) and the `skyline.slots` object of
   `presentation/mineworld-default/3D/far_field/lakeside.yaml` (design content, Q-S29c-4).
6. **Corpus files** for the agreement test, one per new refusal and one per new shape.
7. **Specification.** `docs/MODULE_SPEC.md` §6.1.1 amended; a dated note in `docs/decisions/ARC-107.md`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `settlement:`, the station places, `WorldPack::settlement()` | S29-a |
| The depot, the forecourt in `street.gd`, re-shooting every `ff_*`, `--slice-landmark` | S29-d |
| Lakeside's cited far field (Lake Cuyamaca, the lake moved south, cited slot positions) | S33-a, which rewrites both Lakeside files after this PR |
| The ring fill, the sightline cap and the band overlaps of `social-cafe.yaml` | RL-e0b |
| The 2D far fields | none: format 1 has no slots in 2D (`skyline` is a 3D section) |
| A client that knows the world's `settlement:` at load | none: clients read no World Pack (F-S29c-3) |
| Any change under `kernel/`, `contracts/`, `systems/`, `server/`, `cognition/` | never (I-S29-1) |

### 2.3 Dependencies and order

- **RL-e0 (#220): merged** (621ea0e2). Format 1, both validators and the corpus are on `main` (A-1 … A-9).
- **S29-a: required, not merged.** C4 calls `WorldPack::settlement()` and needs `settlement:` in
  `worlds/{social-cafe,lakeside,market-town,bodies-yard}`. C3's content needs the place `station` in Social Café
  and the halt and `pier`'s `landmark` tag in Lakeside. S29-a missing at C0 → material stop.
- **RL-e0b: in implementation** (`impl-rl-e0b`, two commits, not merged). It moves `social-cafe.yaml`'s massing
  `from_m` from 250 to 70 (its SD-5) and edits `far_field_view.gd`. C0 records which of the two lands first; C3
  follows §5.6's rule either way (F-S29c-4).
- **S33-a: frozen, waits for this PR** (its start condition (4)). It rewrites both Lakeside files whole, slots
  included. This PR writes only Lakeside's `skyline.slots` object (F-S29c-5).
- **S29-d: after this PR** (step §11.1). It re-shoots all seven `ff_*`; this PR's frame change is shown to the
  operator with that re-shoot (Q-S29c-3).

---

## 3. Audit (`origin/main @ 70508683`, 2026-10-18)

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `Slot` has two variants, `Unplaced` and `Reserved { bearing_deg, distance_m, footprint_m, source }`. `Slots` is decoded by a hand-written map visitor that refuses an unknown slot key or a missing one with `SLOT_MARK`. `SlotFields` holds the written fields; `slot()` matches the state: `("unplaced", none)`, `("reserved", all four)`, anything else `"'<s>' is not a slot state (unplaced, reserved)"` | `packages/src/far_field_schema.rs:312-337, 453-483, 485-529` | `Filled` is one more variant and one more match arm; `place` is one more `Option<Key>` field. Mixed states are seen in the `Slots` visitor, after both slots are decoded |
| A-2 | Twelve `FarFieldCode`s in one shared order, `Slot` last; `ALL: [Self; 12]`; `as_str`; `SLOT_MARK = "far-field slot: "` maps a parser message to `FAR_FIELD_SLOT` | `packages/src/far_field.rs:42-111` | `SlotTown` is the 13th, after `Slot`, in both languages; its parser mark must not contain `SLOT_MARK` |
| A-3 | `parse` maps a parser error to `Slot` when the message contains `SLOT_MARK`, else `Malformed`; then shape, scope, sections, citations, ranges, boundary, profiles, coverage | `packages/src/far_field_check.rs:17-37` | The mark check gains the town mark first |
| A-4 | Citations, ranges and the boundary each read a slot only `if let Slot::Reserved` | `far_field_check.rs:181-188` (citations), `263-276` (ranges), `356-363` (boundary) | Each gains a `Filled` arm: cited, ranged; **not** pushed to the boundary list (the inside-boundary exception) |
| A-5 | `check_pack` reads every file of a pack in isolation; `require_for_world` checks only that `far_field/<world>.yaml` exists; `packages::resolve` calls it for each **required** Presentation Pack with a directory source and its module doc says it reads no file's content ("a presence check") | `packages/src/far_field.rs:200-256`; `packages/src/resolve.rs:1-7, 143-152` | No current validator sees a far field and its world together. The world-level rule needs a new call that has both (SD-S29c-2) |
| A-6 | `mineworld validate <world>` = `WorldPack::read_with` + `load` + printing; `mineworld packs validate` on a world = `read_with` + `load` + `world_identity` + the licence policy; on a Presentation Pack = `check_style_manifest` + `far_field::check_pack` | `tools/cli/src/main.rs:592-640`; `tools/cli/src/packs.rs:260-321` | S29-a adds `require_settlement()` to both world paths (its §5.4). The world-level far-field call goes right after it, in both |
| A-7 | `PackError::Requirements { path, refusal: Box<PackageError> }` already carries a package refusal about a world, e.g. `FAR_FIELD_MISSING` from resolve | `worldpack/src/error.rs:188-195` | The world-level refusal reuses this wrapper; no new `PackError` variant |
| A-8 | `worldpack/src/requirements.rs` gathers every data pack in the pack roots with its identity (`found`, private) for the resolver | `worldpack/src/requirements.rs:77` | The world-level rule reuses `found` (made `pub(crate)`) to see Presentation Packs the world does not `require` |
| A-9 | Only Lakeside `requires` the presentation packs (`mineworld-default-2d`, `-3d`). Social Café, Market Town and bodies-yard state no `requires:`. There is no default pack root: a pack is seen only through `--packs` or `MINEWORLD_PACKS` | `worlds/lakeside/world.yaml:35-39`; `worlds/social-cafe/world.yaml` (no `requires`); `packages/src/roots.rs:11-60` | Through `resolve` alone, Social Café's 3D far field is never checked against its world. The world-level rule must also look at Presentation Packs found in the roots (F-S29c-2) |
| A-10 | The GDScript validator keeps the same order. `CODES` (12 entries, `FAR_FIELD_SLOT` last); `_Shape.slots` refuses a foreign or missing slot key; `_Shape.slot` counts the four position fields and matches `unplaced` / `reserved`; citations, ranges and boundary read only `"reserved"` | `clients/shared/environment/far_field.gd:18-31, 201-222, 231-281, 283-305, 437-441, 634-670` | Mirror of A-1 … A-4, line for line |
| A-11 | The 3D slice loads `far_field/social-cafe.yaml` before drawing; `WORLD_KEY` is the constant `"social-cafe"`; on a refusal it prints the code and, in a harness run, exits 3 | `clients/3d-spike/scripts/slice/slice_main.gd:80-110` | The slice knows a world **key**, never the world's `settlement:` (F-S29c-3) |
| A-12 | `_build_skyline` keeps massing clear of `[centre, footprint/2]` for each slot whose state is `"reserved"` | `clients/3d-spike/scripts/slice/far_field_view.gd:482-492` | One condition grows to `reserved` or `filled` |
| A-13 | `_near_ground` draws a walkable grass disc of radius `reach_m` and a collision box of `2·reach_m`; the foreground falls from `reach_m` outward | `far_field_view.gd:191-215, 346-393` | `reach_m` 100 widens the walkable disc by 40 m and moves the fall outward: the frames change (Q-S29c-3) |
| A-14 | Social Café's 3D file: `reach_m` 60, `beyond` `drop` (line 4); bay `shore_m` 1 000; massing sectors `from_m` 250 (lines 101, 106); `station` `reserved` 277°, 790 m, footprint 120; `landmark` `unplaced` (lines 112-118) | `presentation/mineworld-default/3D/far_field/social-cafe.yaml` | Every distance except `from_m` already exceeds 100 m |
| A-15 | Lakeside's 3D file: `reach_m` 80 (line 4); both slots `unplaced` (line 52); every source "design content; lakeside declares no real location" | `presentation/mineworld-default/3D/far_field/lakeside.yaml` | Once S29-a makes Lakeside a town and C4's rule lands, this file is refused unless both slots are filled |
| A-16 | The corpus: one file per code, named by code; `valid-*` must pass; stems ending `-2d` are 2D. The Rust test requires a file for every file-level code (all but `MISSING` and `WORLD_UNKNOWN`). The name-to-code match takes the **longest** code whose name prefixes the stem. `slot-state.yaml` plants the unknown state `"filled"` | `packages/tests/far_field.rs:1-48, 117-142`; `packages/tests/fixtures/far_field/slot-state.yaml:21` | `slot-town-*.yaml` maps to `FAR_FIELD_SLOT_TOWN`, not `FAR_FIELD_SLOT`. `slot-state.yaml` must plant another unknown word, or it stops testing an unknown state |
| A-17 | The agreement test runs the GDScript checker over the corpus and the shipped packs and compares every code with Rust's | `tools/cli/tests/client_far_field.rs:211-260`; `clients/shared/checks/far_field_check.gd` | New corpus files are picked up with no test change |
| A-18 | A change to `social-cafe.yaml` (3D) triggers the per-PR Linux `farfield` job | `scripts/ci_changes.py:251` | This PR's CI runs RL-e0's rendered check once on Linux (Q-S29c-3) |
| A-19 | RL-e0b SD-5 sets both massing sectors' `from_m` to "**70 m** (`reach_m` 60 + 10)"; its FF-RING check bins from `reach_m + 20` | `pr-rl-e0b-far-field-polish.md` §4.5, §5.6, FF-RING | With `reach_m` 100, `from_m` 70 is refused `FAR_FIELD_INSIDE_BOUNDARY` (F-S29c-4) |
| A-20 | S33-a §5.4 writes Lakeside's slots `filled` "from §5.1's translation of the halt's and the pier's floor centres (footprint: the longest side)", in step-33's scene frame (origin the lane's centre, +x east, +z south): a lane point (x, y) mm is at scene (x/1000 − 15, ·, 3 − y/1000) | `pr-s33a-setting-far-field.md` A-17, §5.1, §5.4; its §2.2 row "S29-c (this PR only fills Lakeside's values)" | This PR uses the same frame and the same footprint rule, so S33-a's numbers replace these without a change of meaning |
| A-21 | S29-a: Social Café and Lakeside get `settlement: { kind: town, station: station, landmark: station }` (Lakeside: `landmark: pier`); bodies-yard `kind: none`. Lakeside's waiting room: `lane` passage here (400, 2500), there (29600, 3000), about 8 × 5 m. `pier`: here (2000, 19800), there (18000, 200), about 4 × 20 m running south, foot on its north edge | `pr-s29a-town-contract.md` §5.5; `worlds/lakeside/places/pier.yaml` | The Lakeside content of §5.7 is computed from these |
| A-22 | `ARC-107`'s Consequences end: "Lakeside's 3D slots carry design content until 16f draws Lakeside in 3D" | `docs/decisions/ARC-107.md` | Stale since S33 (Q-S33-10): S33-a replaces the content. The C1 note corrects it |

### 3.2 Findings

- **F-S29c-1. The rule "a town fills both slots" has two halves, and only one needs the world.** A town has both
  slots filled. A world that is not a town has none filled. So a file with **exactly one** filled slot is wrong in
  every world. That half is a property of the file alone, and both validators and every client can hold it. The
  other half (both reserved in a town; a wrong place; filled in a non-town) needs the world's `settlement:`.
- **F-S29c-2. `packages::resolve` cannot carry the world-level half as the step words it.** The step says
  "`mineworld validate` through `packages::resolve`". But `resolve` is presence-only by design (A-5, `ARC-102`):
  it reads no file's content. And it sees only **required** packs, and Social Café requires none (A-9). X-4 is
  planted on Social Café. So the world-level half runs beside `require_settlement()` in the two validate
  commands, over required packs **and** packs found in the roots (SD-S29c-2, Q-S29c-1).
- **F-S29c-3. A client cannot hold the world-level half.** The slice knows the key `social-cafe` and nothing of
  `world.yaml` (A-11). Clients observe the world through Observations, which carry no `settlement:`. Making the
  slice read a World Pack would break `CLAUDE.md` §4 item 3. So "at slice load" means the file-level half. X-4 as
  the step writes it (station back to `reserved`, landmark still filled) **is** a file-level fault, so it fails at
  slice load as the step requires. Both slots set back to `reserved` fails only at validate. This is stated, not
  hidden (Q-S29c-1).
- **F-S29c-4. RL-e0b and this PR collide on one number.** RL-e0b puts the massing at `from_m` 70 = `reach_m` 60 +
  10. This PR raises `reach_m` to 100. Then `from_m` 70 lies inside the boundary and the file is refused. Whichever
  PR lands second must set `from_m` to `reach_m` + 10 = **110**, RL-e0b's own rule. Visible effect: the ring
  between 60 and 110 m is near grass, except where S29-d's depot stands in the west (Q-S29c-2).
- **F-S29c-5. Two PRs write Lakeside's 3D far field, in a fixed order.** Once this PR's world-level rule lands and
  Lakeside is a town (S29-a), Lakeside's file must have both slots filled, or `mineworld validate worlds/lakeside`
  fails. So this PR must write them. S33-a, which starts only after this PR merges, rewrites the whole file. The
  boundary is: this PR writes only the `skyline.slots` object; S33-a owns every other byte and then the slots too.
  The two never edit at the same time (Q-S29c-4).
- **F-S29c-6. Step-29 §6.3 says "the massing (`from_m` 250) and the silhouettes are untouched".** That held on the
  step's base. RL-e0b (frozen 2026-10-18) changed it. The step text is stale, not wrong in intent; §14 proposes a
  dated note.

### 3.3 What does not exist

No `filled`, `FAR_FIELD_SLOT_TOWN`, `SlotName`, or settlement-aware far-field check anywhere in the tree
(`git grep -n -e filled -e SLOT_TOWN -- packages clients worldpack tools` returns only the corpus's planted
`"filled"` word, A-16). No `settlement` outside plan files (S29-a not merged).

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

Nothing is adopted and nothing is built that a library would give. The change extends RL-e0's own validators
(serde with a hand-written visitor in Rust; the `_Shape` checker in GDScript). A JSON Schema validator was
already declined by RL-e0 for format 1 (`DEP-16`); adding one for one more state would be a second validator to
keep in step. No `DEP` record.

---

## 5. Design

### 5.1 The slot, in format 1 (additive; no format bump)

```text
Slot
  {"state": "unplaced"}
  | {"state": "reserved", "bearing_deg": f64, "distance_m": f64 > reach_m, "footprint_m": f64 > 0,
     "source": string}
  | {"state": "filled", "place": Key, "bearing_deg": f64 in [0, 360), "distance_m": f64 > 0,
     "footprint_m": f64 > 0, "source": string}
```

- **`place`** is the key of a Place of the world (`Key`: 1–48 of `a-z`, `0-9`, `-`). Only a filled slot names a
  place.
- **`bearing_deg` and `distance_m`** locate the place's building from the far field's origin (the scene origin), in
  true bearing. **`footprint_m`** is the diameter of the circle around that point that the massing keeps clear of:
  the building's longest side (S33-a's rule, A-20).
- **A filled slot is the one far-field entry allowed at or inside `boundary.reach_m`.** It records where a
  reachable building stands; it draws nothing. A filled slot beyond the boundary is also allowed (a town whose
  station lies outside the presented area, a later S21 region).
- **A filled slot is cited** (`source` non-empty), like every far-field number.

Rust (`far_field_schema.rs`):

```rust
pub enum Slot {
    Unplaced,
    Reserved { bearing_deg: f64, distance_m: f64, footprint_m: f64, source: String },
    /// A slot whose building is a Place of the world, standing here (ARC-107).
    Filled { place: Key, bearing_deg: f64, distance_m: f64, footprint_m: f64, source: String },
}

/// The two slot names, for a refusal that names one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotName { Station, Landmark }   // as_str(): "station", "landmark"; Display = as_str()
```

`SlotFields` gains `#[serde(default)] place: Option<Key>`. `SlotFields::slot()` matches, in this order:

| State | Fields stated | Result |
| --- | --- | --- |
| `unplaced` | none | `Unplaced` |
| `unplaced` | any | `FAR_FIELD_SLOT`: "an unplaced slot states no position" (unchanged) |
| `reserved` | the four, no `place` | `Reserved` |
| `reserved` | otherwise | `FAR_FIELD_SLOT`: "a reserved slot states bearing_deg, distance_m, footprint_m and source, and no place" |
| `filled` | all five | `Filled` |
| `filled` | otherwise | `FAR_FIELD_SLOT`: "a filled slot states place, bearing_deg, distance_m, footprint_m and source" |
| other | — | `FAR_FIELD_SLOT`: "'<s>' is not a slot state (unplaced, reserved, filled)" |

An invalid `place` key fails inside serde with the `Key` message and no mark, so it is `FAR_FIELD_MALFORMED`, as
any other bad key in format 1 is. GDScript's `_Shape.slot` declares `"place": ["key", false]` and gives the same.

### 5.2 The codes

`FarFieldCode` gains **`SlotTown`** (`"FAR_FIELD_SLOT_TOWN"`), the 13th code, after `Slot`, in `ALL` and in
GDScript's `CODES` at the same index. Doc: "A town's slots disagree with the town: exactly one slot filled
(file); or, against the world's `settlement:`, a slot not filled or naming another place (validate)."

`FAR_FIELD_SLOT` keeps its meaning and gains one case: a filled slot in a world whose `settlement:` is
`kind: none` (world-level; §5.4).

### 5.3 The file-level rules (both validators; every client at load)

In the order RL-e0 fixed, unchanged except where marked:

1. **Parsing (MALFORMED, SLOT, SLOT_TOWN).** After both slots decode, the `Slots` visitor (Rust) and
   `_Shape.slots` (GDScript) count the filled slots. **Exactly one filled → `FAR_FIELD_SLOT_TOWN`**, detail
   "only <name> is filled: a town fills both slots and any other world fills none". Rust marks the message with
   `SLOT_TOWN_MARK = "far-field town slot: "` (it does not contain `SLOT_MARK`, so neither mark can be mistaken
   for the other); `parse` looks for the town mark first.
2. Scope, sections: unchanged.
3. **Citations.** A filled slot's `source` joins the list as `slot <name>`.
4. **Ranges.** A filled slot: `bearing_deg` in [0, 360); `distance_m` > 0; `footprint_m` > 0 (`slot <name>
   distance_m`, `slot <name> footprint_m`).
5. **Boundary.** A filled slot's `distance_m` is **not** added to the list `FAR_FIELD_INSIDE_BOUNDARY` checks.
   A reserved slot's still is.
6. Profiles, coverage: unchanged.

Why the mixed case is a parse-stage refusal: it is about the slots' states, as `FAR_FIELD_SLOT` is, and it must be
caught before any range check so both validators give it the same code whatever else is wrong.

### 5.4 The world-level rule (Rust; the authoring gate)

**Pure function** (`packages/src/far_field.rs`):

```rust
/// What a world says about its slots, as the town rule reads it (`ARC-105`, `ARC-107`). The World Pack
/// loader builds it from `settlement:`; this crate does not read worlds.
#[derive(Debug, Clone, Copy)]
pub enum SlotRule<'a> {
    /// `kind: town`: both slots filled, naming these places.
    Town { station: &'a str, landmark: &'a str },
    /// `kind: none`: no slot filled.
    NotATown,
}

/// Checks a far field that already passed `read` against what its world declares. Only a 3D far field
/// has slots; a 2D one passes.
pub fn check_slots(field: &FarField, rule: SlotRule<'_>) -> Result<(), (FarFieldCode, SlotName, String)>;
```

It walks `station`, then `landmark`, and returns the first fault:

| Rule | Slot state | Result |
| --- | --- | --- |
| `Town` | `Filled { place }`, `place` = the declared key | ok |
| `Town` | `Filled { place }`, another key | `SlotTown`, "names the place `<p>`; the world declares `<k>`" |
| `Town` | `Unplaced` or `Reserved` | `SlotTown`, "is `<state>`; a town's far field fills it with `<k>`" |
| `NotATown` | `Filled` | `Slot`, "is filled, but the world is not a town (`settlement: { kind: none }`)" |
| `NotATown` | `Unplaced` or `Reserved` | ok |

**The refusal** (`packages/src/error.rs`), typed so tests assert fields, not prose:

```rust
/// A far field that disagrees with its world's `settlement:` (MODULE_SPEC.md §6.1.1, ARC-107).
#[error("{code}: {path}: pack {pack}, world {world}, slot {slot}: {detail}")]
FarFieldSlots {
    code: FarFieldCode,   // SlotTown or Slot
    path: PathBuf,        // the far-field file
    pack: String,         // the Presentation Pack's id
    world: String,        // the world's id
    slot: SlotName,
    detail: String,
},
```

**The gathering** (`worldpack/src/far_field_slots.rs`, new; `pub use` from `lib.rs`):

```rust
impl WorldPack {
    /// Checks every 3D far field presenting this world against its `settlement:` (ARC-107). Does nothing
    /// when the world declares no settlement: `require_settlement` refuses that first at the gate.
    pub fn check_far_field_slots(&self, roots: &PackRoots) -> Result<(), PackError>;
}
```

1. `self.settlement()` is `None` → `Ok(())`. Otherwise build the `SlotRule` from it.
2. The packs, in a fixed order with no repeats (compared by canonical directory): first each required pack in
   `self.composition().required` (id order) whose kind is Presentation Pack and whose source is a directory; then
   each Presentation Pack found in `roots` (`requirements::found`, now `pub(crate)`), in root order.
3. For each: `check_style_manifest(dir)`; skip unless it presents `3d`. Skip when `far_field/<world-id>.yaml` does
   not exist (presence of a **required** pack's file is `resolve`'s, already checked; a found pack that presents
   other worlds is no concern of this world).
4. `far_field::read(path, &dimensions)` (a file-level refusal is returned as it is), then `check_slots`. A fault
   becomes `PackageError::FarFieldSlots { code, path, pack: identity.id, world: self.id(), slot, detail }`.
5. Every refusal is wrapped `PackError::Requirements { path: <world>/world.yaml, refusal }` (A-7).

**Who calls it.** `tools/cli/src/main.rs` `validate` and `tools/cli/src/packs.rs` (the world branch), each right
after S29-a's `require_settlement()`. Neither `run`, `server` nor the loader calls it: the loader stays
presence-only (`ARC-102`), and a town's far field is an authoring fact like `settlement:` itself (`ARC-53`
precedent, S29-a §5.4). CI runs the gate on every `worlds/*` through S29-a's `every_reference_world_validates`
with `--packs entities --packs presentation/mineworld-default`, which finds both presentation packs (A-9).

*Example.* `mineworld validate worlds/social-cafe --packs presentation/mineworld-default` with the station slot
set back to `reserved` and the landmark filled → the file-level rule refuses first:
`worlds/social-cafe/world.yaml: FAR_FIELD_SLOT_TOWN: …/3D/far_field/social-cafe.yaml: only landmark is filled: …`.
With both set back to `reserved` → the world-level rule refuses: `… FAR_FIELD_SLOT_TOWN: …/social-cafe.yaml: pack
mineworld-default-3d, world social-cafe, slot station: is reserved; a town's far field fills it with station`.

### 5.5 The 3D drawing

`far_field_view.gd` `_build_skyline`: the clear list takes each slot whose state is `reserved` **or** `filled`.
Nothing else in either client reads a slot. A filled slot is not drawn by the far field: the building is near
content, drawn by the scene that presents the place (S29-d for Social Café, S33-e for Lakeside). The far field
only keeps its ground free of distant massing.

### 5.6 Social Café's 3D content (`presentation/mineworld-default/3D/far_field/social-cafe.yaml`)

| Field | Was | Becomes | Source text |
| --- | --- | --- | --- |
| `boundary.reach_m` | 60 | **100** | (no source field) — step-29 §6.3: contains the depot, the arcade, the platform and its fence |
| `skyline.slots.station` | reserved 277°, 790 m, 120 m | `{"state": "filled", "place": "station", "bearing_deg": 270, "distance_m": 70, "footprint_m": 60, "source": …}` | "Santa Fe Depot, San Diego (NRHP 72000248; HABS CA-1965; waiting room 51.8 × 16.8 m, step-29 §2.2), compressed from 790 m to the street's west end so that it is reachable on foot (Q-S29-1); bearing and distance of the hall's centre from the scene origin" |
| `skyline.slots.landmark` | unplaced | the same five values, `place` `station` | "The station is the town's landmark (step-29 SD-S29-3, Q-S29-1); " followed by the station's text |
| both massing sectors' `from_m` | 250, or 70 if RL-e0b merged first | **unchanged at 250** if RL-e0b has not merged; **110** if it has | RL-e0b's sector source already states its rule; append "; from `reach_m` + 10 (S29-c, Q-S29c-2)" when changed |

Every other value is unchanged. S29-d may correct the slot's bearing and distance to the depot as built (step
§6.3: "S29-d C1 records the measured bearing") as its own bounded correction.

If RL-e0b lands **after** this PR, its implementer meets `from_m` 70 ≤ `reach_m` 100 at `packs validate` and
writes 110 instead (its rule "`reach_m` + 10" read with the new `reach_m`). §14 hands RL-e0b this item.

### 5.7 Lakeside's 3D slots (`presentation/mineworld-default/3D/far_field/lakeside.yaml`, `skyline.slots` only)

Computed in step-33's scene frame (A-20: origin the lane's centre; scene (x/1000 − 15, ·, 3 − y/1000); north is
−z), from S29-a's passages (A-21):

| Slot | Point | Scene (m) | Bearing, distance | Footprint | Value |
| --- | --- | --- | --- | --- | --- |
| `station` | the waiting room's centre: its frame origin is lane (29 200, 500), its centre (4 000, 2 500) → lane (33 200, 3 000) | (18.2, 0.0) | 90.0°, 18.2 m | 8 (the room's longest side) | `{"state": "filled", "place": "station", "bearing_deg": 90, "distance_m": 18.2, "footprint_m": 8, "source": "design content; lakeside declares no real location: the halt's waiting room at the lane's east end (S29-a, Q-S33a-1), step-33 scene frame; replaced by S33-a"}` |
| `landmark` | the pier's centre: its frame origin is lane (16 000, −19 600), its centre (2 000, 10 000) → lane (18 000, −9 600) | (3.0, 12.6) | 166.6°, 13.0 m | 20 | `{"state": "filled", "place": "pier", "bearing_deg": 166.6, "distance_m": 13, "footprint_m": 20, "source": "design content; lakeside declares no real location: the pier's centre (worlds/lakeside/places/pier.yaml), step-33 scene frame; replaced by S33-a"}` |

Bearing from the scene point (e, s): `atan2(e, −s)` in degrees, mod 360 (east positive, north is −z). Both lie
inside `reach_m` 80, which stays. No other byte of the file changes. C0 recomputes both rows from S29-a as merged;
a difference is a bounded correction recorded in §15.

### 5.8 Specification text (C1)

**`docs/MODULE_SPEC.md` §6.1.1**, four edits:

1. The `Slot` block becomes §5.1's three-state block, and its note reads: "A reserved or filled slot keeps its
   footprint clear of massing. A filled slot names a Place of the world (`place`) whose building stands at that
   bearing and distance; it is the one far-field entry allowed at or inside `boundary.reach_m`. `filled` was
   added by S29-c (`ARC-107`) as an additive change to format 1."
2. The `FAR_FIELD_INSIDE_BOUNDARY` row: "… a reserved slot's `distance_m` (never a filled slot's) ≤
   `boundary.reach_m`".
3. The `FAR_FIELD_SLOT` row: "… a state other than `unplaced`/`reserved`/`filled`, a state's fields missing or
   foreign, or (validate) a filled slot in a world whose `settlement:` is `kind: none`". A new row after it:
   `FAR_FIELD_SLOT_TOWN` | "exactly one slot is filled (file); or (validate) in a world with `settlement: { kind:
   town }`, a slot that is not filled or whose `place` is not the declared `station` / `landmark`".
4. Under "Where a missing far field is refused", a third item: "**Town slots (validate).** `mineworld validate
   <world>` and `mineworld packs validate <world>` check every 3D far field of the world — in each Presentation
   Pack the world requires and each one found in the pack roots — against the world's `settlement:`
   (`FAR_FIELD_SLOT_TOWN`, `FAR_FIELD_SLOT`). A client holds only the file-level half (exactly one filled slot),
   because a client reads no World Pack."

**`docs/decisions/ARC-107.md`**, a dated note appended:

```text
## ARC-107 note — 2026-MM-DD: where the town rule runs, and Lakeside's content

Implemented by S29-c (pr-s29c-far-field-filled.md). Point 3's rule runs in two halves. A file with exactly one
filled slot is refused FAR_FIELD_SLOT_TOWN by both validators and at client load: it is wrong in every world.
The rest needs the world's settlement:, which no client reads, so it runs at the authoring gate (mineworld
validate and packs validate), over every Presentation Pack the world requires or the pack roots hold, not
through packages::resolve, which stays presence-only (ARC-102). Lakeside's slots carry design content until
S33-a (not 16f) replaces it with cited positions. With reach_m 100, Social Café's massing starts at reach_m + 10
(RL-e0b's rule).
```

C1 then runs `python3 scripts/check_decision_ids.py --write-index` (a note changes no index row; the run proves
it).

---

## 6. Invariants this PR holds

| Id | Invariant | Held by |
| --- | --- | --- |
| I-c-1 (I-S29-1) | The diff under `kernel/`, `contracts/`, `systems/`, `server/`, `cognition/` is empty | `git diff --name-only` at C5 |
| I-c-2 (I-S29-7) | A filled slot is the only far-field entry at or inside `reach_m`; every other distance stays beyond it | the boundary rule (§5.3 item 5); corpus `inside-boundary*` |
| I-c-3 | Both validators give every corpus file and every shipped far field the same code | `client_far_field.rs::the_two_validators_agree` |
| I-c-4 | Every world keeps a far field with no gap: coverage, sections and the 3D rendered check are unchanged by this PR except for the drawn effect of `reach_m` and `from_m` | `packs validate`; RL-e0's `three_d_far_field_closes_every_open_sector` |
| I-c-5 | No world-specific code: no file of `packages/`, `worldpack/`, `tools/cli/src/` or `clients/` names `station`, `pier`, `social-cafe` or `lakeside` as a value the rule depends on; the place keys come from `settlement:` | review question of C2 and C4; the generated `depot` variant (AC-c-2) |
| I-c-6 (I-S29-6) | The one compression is stated in the Social Café slot's `source` | §5.6 |
| I-c-7 | The far field stays decoration: nothing here reaches an `ActionIntent`, an Observation or world state; no replay digest or golden moves | the reference worlds' digest tests untouched (no file under `worlds/` changes) |

---

## 7. Test ownership

| Test | Where | Asserts |
| --- | --- | --- |
| The corpus table | `packages/tests/far_field.rs` (rows grow 13 → 17) | each new file's code and the words its detail must carry |
| The corpus completeness | the same file (unchanged code) | a file exists for `FAR_FIELD_SLOT_TOWN` |
| `check_slots` table | the same file, new test | every row of §5.4's table on an in-memory far field, for a town whose keys are `depot` and `clock-tower` (generated) and for `NotATown` |
| The world-level gate | `tools/cli/tests/requirements.rs`, new tests | §8's AC-c-2 … AC-c-4 on generated worlds and on copies of the reference worlds |
| The agreement | `tools/cli/tests/client_far_field.rs::the_two_validators_agree` (unchanged code; Godot, milestone) | the new corpus files and both shipped 3D files agree |
| The rendered check | `client_far_field.rs::three_d_far_field_closes_every_open_sector` (unchanged; Linux per PR by A-18, three OSes at the milestone) | Social Café's far field still closes every view |

New corpus files (`packages/tests/fixtures/far_field/`), each a copy of `valid-3d.yaml` with one change:

| File | Change | Code |
| --- | --- | --- |
| `valid-3d-filled.yaml` | both slots filled, `place` `hall` / `tower`, `distance_m` 30 (inside `reach_m`) | passes |
| `slot-town.yaml` | station filled, landmark `unplaced` | `FAR_FIELD_SLOT_TOWN` |
| `slot-filled.yaml` | station `{"state": "filled", "bearing_deg": 90, "distance_m": 30, "footprint_m": 10, "source": "x"}` (no `place`) | `FAR_FIELD_SLOT` |
| `out-of-range-filled.yaml` | both filled, landmark `footprint_m` 0 | `FAR_FIELD_OUT_OF_RANGE` |
| `uncited-filled.yaml` | both filled, station `source` `" "` | `FAR_FIELD_UNCITED` |
| `slot-state.yaml` (edit) | the planted unknown state `"filled"` → `"built"` | `FAR_FIELD_SLOT` (unchanged) |

---

## 8. Acceptance criteria (fixed before measuring)

Per-PR (speed-first charter): C2 … C4's own tests and `fast` CI. **Run at the milestone:** AC-c-5, AC-c-6 on three
OSes and every planted mutation of §8.1, failures fixed in one batch then.

| Id | Criterion | Step id |
| --- | --- | --- |
| AC-c-1 | Both validators accept `filled` and refuse every new corpus file with its named code; the shipped `social-cafe.yaml` **and** `lakeside.yaml` (3D) pass `mineworld packs validate presentation/mineworld-default/3D` | AC-S29-1 (format) |
| AC-c-2 | `mineworld validate` and `mineworld packs validate` pass on `worlds/social-cafe` **and** `worlds/lakeside` with `--packs entities --packs presentation/mineworld-default`; and on a **generated town variant** (a copy of Social Café whose station key is `depot` and landmark `clock-tower`, with a generated 3D pack filled to match), which proves no key is assumed | AC-S29-1; row checkpoint |
| AC-c-3 | **X-4** on Social Café (station → `reserved`, landmark filled): `validate` → `FAR_FIELD_SLOT_TOWN` (file-level) and the slice refuses the same code at load and exits 3. **The same on Lakeside** (landmark → `unplaced`): `validate` → `FAR_FIELD_SLOT_TOWN`. **Both slots reserved** on Social Café: `validate` → `FarFieldSlots { code: SlotTown, slot: Station, world: "social-cafe" }` | X-4, AC-S29-1 |
| AC-c-4 | **`filled` under `kind: none`**: a copy of `bodies-yard` with a generated 3D pack whose far field fills both slots → `FarFieldSlots { code: Slot, slot: Station }`; the same pack with a generated non-town variant (a `kind: none` world named `test-yard-2`) refuses the same way | row adversarial criterion |
| AC-c-5 | The slice loads the new Social Café file; RL-e0's 3D rendered check passes (Linux per PR when the job runs; Linux, macOS, Windows at the milestone). Lakeside is held by the validators only until S33-d draws it | AC-S29-4 (the RL-e0 half); row checkpoint |
| AC-c-6 | The two validators agree over the corpus and both shipped 3D files (`the_two_validators_agree`) | step §9.3, `packages` far-field row |
| AC-c-7 | I-c-1 and I-c-7: no path under the forbidden directories or under `worlds/` changes | I-S29-1 |

### 8.1 Planted mutations (milestone; each must fail by name, then be restored from a byte copy)

| Id | Plant | Must fail with |
| --- | --- | --- |
| X-4 | AC-c-3's first plant | `FAR_FIELD_SLOT_TOWN` at validate and at slice load |
| M-c-1 | Lakeside's `landmark.place` `pier` → `station` | `FarFieldSlots { code: SlotTown, slot: Landmark, world: "lakeside" }` |
| M-c-2 | `check_boundary` (Rust) pushes a filled slot's distance too | `valid-3d-filled` red in `every_corpus_file_gives_the_code_its_name_says` (`FAR_FIELD_INSIDE_BOUNDARY`) |
| M-c-3 | GDScript `_check_citations` ignores filled slots | `the_two_validators_agree` red on `uncited-filled` |
| M-c-4 | `check_far_field_slots` reads only required packs (drops the roots) | AC-c-3's Social Café both-reserved test passes validate: red |
| M-c-5 | `check_slots` compares `place` with a constant `"station"` instead of the rule's key | the generated `depot` variant (AC-c-2) red |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s29c`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full.
2. Make the numbered steps, in order. Do not do a later commit's work.
3. Run the commit's validation, one `--test <file>` at a time (R-LOAD-1: never `cargo test --workspace`, never all
   of `tools/cli`). Record per command: test count, PASS / FAIL / INCONCLUSIVE.
4. Static: `cargo fmt --all --check`; `cargo clippy -p <crate> --all-targets -- -D warnings` for each touched
   crate. Markdown: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
5. Review: answer the commit's review question in §15, one or two sentences.
6. `git status --short`: every changed path is in the commit's **Files**, or revert it, or stop and record why.
7. Stage by explicit path (never `git add -A`); commit with the line
   `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` (or the implementer's own model line).
   No heredoc writes, no `sed -i`, no `python3 -c`, no `curl`.

#### C0 — Base and ledger (Markdown only)

- **Files:** this file (§15).
- **Steps:**
  1. Record the base commit and the dispatch confirming Milestone F's checklist was handed over (missing →
     material stop).
  2. Record the merge state of S29-a (required: missing → material stop), RL-e0b and S33-a (S33-a merged first →
     material stop: its start condition was broken).
  3. Record `social-cafe.yaml`'s massing `from_m` on the base (250 or 70) and choose §5.6's row.
  4. Re-read A-1 … A-22 on the base; record moved lines (bounded) or changed statements (stop if one changes a
     decision here). Recompute §5.7's two rows from S29-a as merged.
- [ ] Implementation. [ ] Validation: the two doc scripts. [ ] Review: §2.3 holds on the base.
- **Commit message:** `docs(plan): S29-c — open the ledger at the base`

#### C1 — Specifications first

- **Files:** `docs/MODULE_SPEC.md`, `docs/decisions/ARC-107.md`, `docs/decisions/INDEX.md` (only if the script
  rewrites it).
- **Steps:** §5.8's four MODULE_SPEC edits; §5.8's note at the end of `ARC-107.md`; `python3
  scripts/check_decision_ids.py --write-index`.
- **Validation:** `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Implementation. [ ] Validation. [ ] Review: every code and every rule of §§5.1–5.4 is in the spec text; no
  new ontology term (`SlotRule`, `SlotName` are code names, not concepts).
- **Commit message:** `docs(spec): the far field's filled slot and the town rule (S29-c, ARC-107 note)`

#### C2 — `filled` and the file-level rule, in both validators, with the corpus

- **Files:** `packages/src/far_field_schema.rs`, `packages/src/far_field.rs`, `packages/src/far_field_check.rs`,
  `packages/src/lib.rs` (re-export `SlotName`), `clients/shared/environment/far_field.gd`,
  `clients/3d-spike/scripts/slice/far_field_view.gd` (§5.5's one condition),
  `packages/tests/far_field.rs`, `packages/tests/fixtures/far_field/{valid-3d-filled, slot-town, slot-filled,
  out-of-range-filled, uncited-filled, slot-state}.yaml`.
- **Steps:**
  1. Rust schema (§5.1): `Slot::Filled`, `SlotName`, `SlotFields.place`, `slot()`'s table.
  2. Rust codes (§5.2): `SlotTown`, `ALL: [Self; 13]`, `as_str`; `SLOT_TOWN_MARK`; the mixed check in the `Slots`
     visitor; `parse` maps the town mark first.
  3. Rust rules (§5.3): the `Filled` arms of citations and ranges; the boundary list skips `Filled`. Update the
     module doc's order line to read "MALFORMED (or SLOT / SLOT_TOWN, during parsing) → …".
  4. GDScript, line for line: `CODES` gains `"FAR_FIELD_SLOT_TOWN"` at index 12; `_Shape` gains
     `slot_town_fault`; `slot()` gains `place` and the `filled` arm; `slots()` counts filled after both slots;
     citations and ranges gain `filled`; the boundary skips it. Same header order line.
  5. `far_field_view.gd`: the clear list takes `reserved` or `filled`.
  6. The corpus files of §7; four rows in the table test.
- **Validation:** `cargo test -p mineworld-packages --test far_field`; `cargo clippy -p mineworld-packages
  --all-targets -- -D warnings`. The Godot agreement test is milestone (it needs Godot; R-LOAD-1).
- [ ] Implementation. [ ] Validation. [ ] Review: the two validators keep the same order and give each new file
  the same code (read both diffs side by side); no world key appears in code (I-c-5).
- **Commit message:** `feat(packages, clients): the far field's filled slot and the one-filled refusal (S29-c)`

#### C3 — Content: Social Café and Lakeside

- **Files:** `presentation/mineworld-default/3D/far_field/social-cafe.yaml`,
  `presentation/mineworld-default/3D/far_field/lakeside.yaml` (`skyline.slots` only).
- **Steps:** §5.6 (with C0's `from_m` choice) and §5.7 (with C0's recomputed rows).
- **Validation:** `target/debug/mineworld packs validate presentation/mineworld-default/3D` (build `-p
  mineworld-cli` first); `cargo test -p mineworld-packages --test far_field` (the shipped-shape test).
- [ ] Implementation. [ ] Validation. [ ] Review: every number has a source; Lakeside's file differs from the base
  only inside `skyline.slots` (`git diff` shows it).
- **Commit message:** `content(3d): Social Café's and Lakeside's station and landmark slots filled (S29-c)`

#### C4 — The world-level rule at the authoring gate

- **Files:** `packages/src/far_field.rs` (`SlotRule`, `check_slots`), `packages/src/error.rs`
  (`FarFieldSlots`), `packages/src/lib.rs`, `packages/tests/far_field.rs` (the `check_slots` table),
  `worldpack/src/far_field_slots.rs` (new), `worldpack/src/requirements.rs` (`found` → `pub(crate)`),
  `worldpack/src/lib.rs`, `tools/cli/src/main.rs`, `tools/cli/src/packs.rs`, `tools/cli/tests/requirements.rs`.
- **Steps:** §5.4 in order; the calls after `require_settlement()` in both commands; the tests of AC-c-2 … AC-c-4
  with generated worlds and packs in temporary directories (the file's existing helpers).
- **Validation:** `cargo test -p mineworld-packages --test far_field`; `cargo test -p mineworld-worldpack` (crate,
  unit tests only if any; else `N/A`); `cargo test -p mineworld-cli --test requirements`; then
  `target/debug/mineworld validate worlds/social-cafe --packs entities --packs presentation/mineworld-default` and
  the same for `worlds/lakeside`, `worlds/market-town` and `worlds/bodies-yard` (all pass); clippy on the three
  crates.
- [ ] Implementation. [ ] Validation. [ ] Review: `run` and `server` do not call the check (I-a-3 of S29-a holds);
  the rule reads keys only from `settlement:` (I-c-5); the packs are visited in a fixed order.
- **Commit message:** `feat(validate): a town's far field fills its slots with the declared places (S29-c)`

#### C5 — Close-out

- **Files:** this file (§15).
- **Steps:** `git diff --name-only origin/main...HEAD` checked against I-c-1 and AC-c-7; push `mvp0/pr-s29c`;
  `gh pr create` (body ends with the Claude Code line; it names Q-S29c-3: the frame change is presented with
  S29-d); wait for `fast`; record the PR number and head SHA; report MERGE-READY.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: §15 complete.
- **Commit message:** `docs(plan): S29-c — close the ledger (MERGE-READY)`

---

## 10. Requirements this PR places on other PRs

| PR | Requirement |
| --- | --- |
| S29-a | `WorldPack::settlement()` returns the declared keys; `require_settlement()` runs in both validate paths; `every_reference_world_validates` passes `--packs presentation/mineworld-default` (its AC-a-1 already says so) |
| RL-e0b | If it lands after this PR: massing `from_m` = `reach_m` + 10 = 110, not 70 (F-S29c-4) |
| S33-a | Reads this PR's Lakeside slots as design content and replaces the whole file, slots included (it already plans to, A-20) |
| S29-d | Presents this PR's changed `ff_*` frames to the operator with its own re-shoot (Q-S29c-3); may correct the Social Café slot's bearing and distance to the depot as built |
| S33-d … j | A 3D Lakeside scene keeps massing clear of the filled footprints through the same `_build_skyline` rule |

---

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S29c-1 | The mixed state (exactly one filled) is a file-level `FAR_FIELD_SLOT_TOWN` | F-S29c-1: wrong in every world, so every validator and client can refuse it |
| SD-S29c-2 | The world-level half runs in `mineworld validate` and `packs validate`, over required **and** found Presentation Packs, not in `resolve` | F-S29c-2, F-S29c-3; Q-S29c-1 asks the operator to confirm |
| SD-S29c-3 | `place` is a format-1 `Key`, not a typed `EntityKey` | `packages` does not depend on `worldpack`; the world-level rule compares strings that both sides validated |
| SD-S29c-4 | A filled slot may lie beyond `reach_m` too | the step says "may be ≤"; forbidding the far case would block a station in a later S21 region |
| SD-S29c-5 | `footprint_m` = the named place's longest side | S33-a's rule (A-20); one meaning for both authors |
| SD-S29c-6 | No `PackError` variant; the refusal rides `PackError::Requirements` | A-7: the same wrapper already carries `FAR_FIELD_MISSING` for a world |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S29-c — the far field's filled slot (schema in Rust and GDScript,
                         FAR_FIELD_SLOT_TOWN, the town rule at validate, Social Café and Lakeside content)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s29c-far-field-filled.md
RELATED / BINDING DOCS:  step-29-station-landmark.md (§§6.3, 6.7, 9.3, 9.5, 10, 11); pr-s29a-town-contract.md;
                         pr-s33a-setting-far-field.md (§§2.2, 5.1, 5.4); ../mvp1/pr-rl-e0-far-field-mandatory.md;
                         ../mvp1/pr-rl-e0b-far-field-polish.md (§§4.5, 5.6); docs/MODULE_SPEC.md §6.1.1;
                         docs/decisions/ARC-102.md, ARC-105.md, ARC-107.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md; .structured-coding/standards.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s29c, its own, held by one session
BRANCH:                  mvp0/pr-s29c, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-29 §11.2) AND S29-a merged AND
                         this design frozen by the operator AND the dispatch. Not met at 2026-10-18 (S29-a is
                         DRAFT)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–7, as §14's rulings shape them                     (operator, pending)
FROZEN INVARIANTS:       step-29 I-S29-1, I-S29-6, I-S29-7; §6 I-c-1 … I-c-7; §8 as written
SEQUENCE:                C0 … C5 (§9)
ALLOWED COMMANDS:        cargo build / test -p <crate> [--test <file>] / clippy -p / fmt; target/debug/mineworld
                         validate | packs validate; git; gh (pr create/view/checks; never merge);
                         python3 scripts/check_doc_headings.py, check_decision_ids.py (--write-index);
                         mkdir -p; cp; cmp; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or all
                         of tools/cli (R-LOAD-1); Godot runs outside cast-slots/with-slot.sh; editing any path
                         under kernel/, contracts/, systems/, server/, cognition/ or worlds/; editing any byte
                         of lakeside.yaml outside skyline.slots; reading a key or .env; calling any model;
                         merging
MATERIAL STOPS:          Milestone F not handed over; S29-a not merged, or merged without settlement() /
                         require_settlement(); S33-a merged first; a needed change to a forbidden path; the
                         town rule needing a client to read a World Pack; RL-e0's rendered check red on the
                         new Social Café file with a cause other than reach_m / from_m; any change to a frozen
                         step invariant or acceptance criterion
PLATFORMS:               Linux, macOS, Windows (CI `fast` per PR; the full matrix and the Godot checks at the
                         milestone)
VALIDATION BUDGET:       per commit, §9's named test files only; per PR, `fast` CI (and the Linux `farfield`
                         job that A-18 triggers). Acceptance runs (§8 AC-c-5, AC-c-6) and planted mutations
                         (§8.1) run at the milestone. No paid call
LIVE DOCUMENTATION:      this document (§15)
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                         semantic commits: authorized
                         branch push, PR creation / update, CI repair to green: authorized
                         endpoint: MERGE-READY — pushed, `fast` green on the exact head, reported with the
                         PR number and head SHA
                         merge: NOT by the implementation session; the coordinator merges
POST-MERGE SYNC OWNER:   primary / lane lead: step-29, overall.md, the progress page; implementation session:
                         this ledger, the merge identity, evidence, deviations
STOP CONDITION:          S29-c MERGE-READY (READY FOR OPERATOR REVIEW) — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-c-1 | RL-e0b and this PR both edit `social-cafe.yaml` and `far_field_view.gd` | Different lines (RL-e0b: bands, `from_m`, drawing constants; this PR: `reach_m`, slots, one condition in `_build_skyline`); §5.6's rule decides `from_m` either way; the second to land rebases |
| R-c-2 | `reach_m` 100 widens the walkable grass disc to 100 m in the slice before S29-d builds the forecourt (A-13), and the 60–110 m ring reads as grass from above | It is what `reach_m` means (every reachable point lies within it). The operator sees it once, with S29-d's re-shoot (Q-S29c-3) |
| R-c-3 | A Presentation Pack found in a root that carries an unrelated `far_field/<same-id>.yaml` is checked against this world | World ids are a pack's identity and must be distinct in one composition (`distinct`); the check runs only at the authoring gate, where the author chose the roots |
| R-c-4 | S29-a's halt moves when it merges, so Lakeside's numbers drift | C0 recomputes §5.7 from S29-a as merged; S33-a replaces them anyway |
| R-c-5 | The GDScript and Rust validators drift on the new state | The agreement test (milestone) and C2's side-by-side review question; M-c-3 proves the test sees a drift |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29c-1** | Where does the town rule run? The step says "at validate (through `packages::resolve`) and at client load". `resolve` reads no file content and sees only required packs (Social Café requires none), and a client never reads `world.yaml` | **Split it.** The client and both validators refuse a file with exactly one filled slot (this covers X-4 as written). The full rule (both reserved, a wrong place, filled in a non-town) runs at `mineworld validate` / `packs validate`, over required and found presentation packs. *Reason:* it keeps `resolve` presence-only and clients free of World Packs, and still fails X-4 at both places |
| **Q-S29c-2** | RL-e0b puts Social Café's massing at 70 m (`reach_m` 60 + 10). With `reach_m` 100, 70 is refused. Use 110? | **Yes, 110 (`reach_m` + 10, RL-e0b's own rule).** *Reason:* the depot needs `reach_m` 100; the 60–110 m ring becomes grass, and the depot fills the west |
| **Q-S29c-3** | This PR changes Social Café's far-field frames (wider grass, later drop, massing start). RL-e0's V-gate says a far-field change is shown to the operator. Show it now, or with S29-d's re-shoot of all `ff_*`? | **With S29-d.** *Reason:* one look at the final frames with the depot in place; this PR's frames are an intermediate state |
| **Q-S29c-4** | Lakeside's 3D file is written by this PR (slots only, design content) and then rewritten whole by S33-a. Accept two PRs writing one file, in that fixed order? | **Yes.** *Reason:* once the town rule lands, Lakeside fails validate unless its slots are filled; S33-a already plans to replace them, and the two never edit at the same time |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C5 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; X-4, M-c-1 … M-c-5 run at the milestone, §8.1)

### Deviations and findings

F-S29c-1 … F-S29c-6 (§3.2) and SD-S29c-1 … 6 (§11), recorded at design time.

### Proposed edits elsewhere (made by the PR named)

| File | Edit | By |
| --- | --- | --- |
| `step-29-station-landmark.md` §6.3, "The far field" bullet | dated note: "the massing `from_m` 250" is superseded by RL-e0b (70) and this PR (110, Q-S29c-2) | the lane lead, at this design's freeze |
| `step-29-station-landmark.md` §6.7, "Town cross-check" | dated note: the rule runs in two halves (Q-S29c-1) | the lane lead, at freeze |
| `pr-rl-e0b-far-field-polish.md` §5.6 | a bounded-correction note: `from_m` = `reach_m` + 10 = 110 if S29-c merged first | RL-e0b's implementer, in its ledger |
| `docs/MODULE_SPEC.md` §6.1.1, `docs/decisions/ARC-107.md` | §5.8 | this PR, C1 |
