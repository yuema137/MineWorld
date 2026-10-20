# PR S29-a — The town contract: `settlement:`, the station group, the loader's checks, and the three towns' stations

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session 2026-10-19: every open question answered as recommended)

```text
Design revision:        revision 4 (2026-10-19: the frames the clients can draw, §14.5), DRAFT;
                        revision 3 (2026-10-18: the S33 rulings of 2026-10-18 applied, §14.2);
                        revision 2 (2026-10-15: §14's rulings applied)
Author:                 S29-a / S30-a / S30-b PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29a-s30ab, branch docs/pr-designs-s29a-s30ab,
                        planning base origin/main @ e6030a36 (#225 merged; the task named dd357ca6, and the
                        only commit between them is the S30 amendment, which this file does not touch).
                        Revision 3: S31-b PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31b-s29a, branch
                        docs/pr-design-s31b-s29a-rev, base origin/main @ b85f2804 (#255, the S33-a/b/c
                        designs, merged); it changes only what §14.2's rulings require, each change marked
                        "rev. 3" with its source.
                        Revision 4: S31/S29 lane lead (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29a-rev4, branch docs/pr-design-s29a-rev4,
                        base origin/main @ 5a8c6f28; it carries the requirements the S29-b … S29-g and S33-e
                        designs place on this PR (§14.5), each change marked "rev. 4" with its source
Approved by / evidence: §14 ruled 2026-10-15 (operator answered directly in the primary session), every
                        question as recommended (§14.1). Not frozen: the freeze follows 12d's merge and
                        the hand-over of Milestone F's checklist (coordinator, 2026-10-15)
Implementation base:    origin/main at C0; C0 records the commit and the merge state of 12d (#171) and S26-a (#206)
Start condition:        (1) Milestone F's checklist handed to the operator (step-29 §11.2, operator 2026-10-15);
                        (2) 12d merged (#171: OPEN at e6030a36); (3) this design frozen; (4) the primary
                        session's dispatch. C4 additionally waits for S26-a (#206: OPEN at e6030a36), so the
                        re-baseline happens once (Q-S29-7 as ruled). NONE of (1)–(3) holds today
Execution contract:     §12
Implementer:            Haiku-class (step-29 §11.1, the S29-a row: "Haiku, from a commit-level plan"); reviewer
                        the primary session, plus an Opus logic review of C2 and C4
Lifecycle:              DESIGN FROZEN 2026-10-19 — (revision 2's rulings, the S33 rulings and Q-S29a-5 … 7 (operator, 2026-10-18,
                        as recommended) applied; revision 4 applies Q-S33e-3 (ruled) and the recommended
                        answers of Q-S29a-8 … Q-S29a-11 (§14.6, open). 12d (#171) and S26-a (#206) merged
                        2026-10-18, and 16d (#250, Milestone F's last item) 2026-10-19; the freeze waits for
                        §14.6's rulings and the coordinator's record that F's checklist was handed over)
```

**Effort:** `mvp0` · **Step:** [`step-29-station-landmark.md`](step-29-station-landmark.md) (APPROVED 2026-10-15;
Q-S29-1…10 ruled as recommended, §13.1) §§1, 2, 3, 4, 5.2, 7.1, 9.3, 9.5 (X-1, X-2, X-3), 10 (I-S29-1, I-S29-2,
I-S29-4, I-S29-6), 11 (the S29-a row), 14.1–14.3 · **Parent:** [`overall.md`](overall.md) · **Followed by:**
S29-b (newsstand), S29-c (far-field `filled`), S29-f (2D), S29-g (agent view), all on this PR's contract ·
**Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 2, 5, 7, 8, 9, 10, 13, 14, 15); `docs/CORE_CONCEPTS.md`
§§6, 6.1, 6.2; `docs/MODULE_SPEC.md` §4.1; `docs/DECISIONS.md` `ARC-31`, `ARC-33`, `ARC-36`, `ARC-53`, `ARC-75`,
`ARC-82`, `ARC-105`.

**Decision ids.** None new. `ARC-105` already exists on `main` (`docs/DECISIONS.md:8305`); this PR adds one dated
note under it (C1) recording how §14's rulings shaped item 3. `ARC-127`…`ARC-129` and `DEP-69` are not used.

No hosted model, image model or paid API is called by this design or by anything it specifies.

---

## 0. Binding inputs, exactly restated

- **Scope (step-29 §11.1, S29-a row).** "C1, specification first: MODULE_SPEC §4.1 and §4.3, CORE_CONCEPTS §6
  role tags, VISUAL_SLICE §4 amendment, `ARC-105`. C2: the `worldpack` `settlement:` key and the §4.2 checks.
  C3: `mineworld validate` requires the key. C4: the three station groups in three worlds, with bodies,
  passages and routines; `bodies-yard` `kind: none`. C5: the one recorded re-baseline of digests and golden
  fixtures (Q-S29-7)."
- **Integration checkpoint (same row).** "`mineworld run worlds/market-town` 30 d: `station_visit` (social half)
  passes; save, restart and resume are identical; `every_reference_world_validates`."
- **Adversarial criteria (same row).** "X-1, X-2, X-3; a `station` tag on a place not joined to the hall →
  `TOWN_STATION_SCATTERED`."
- **Rulings that bind here (step-29 §13).** Q-S29-2 (the `settlement:` key, enforced by `mineworld validate` /
  `packs validate`, as with `ARC-53`); Q-S29-3 (Lakeside gets a small halt; `pier` is its landmark); Q-S29-4
  (the real hall, 51.8 × 16.8 m); Q-S29-6 (an existing person runs the newsstand — S29-b's); Q-S29-7 (one
  recorded re-baseline, in this PR, after 12d and S26-a); Q-S29-9 (all three towns get a station). None is
  re-asked.
- **S33 rulings that bind here (rev. 3; operator, 2026-10-18, in the primary session; recorded in the S33
  designs merged as #255).**
  - **Q-S33a-1** ([`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) §14): "east end, north–south,
    as recommended — the small station at the lane's east end, a north–south platform with its end against the
    shore; S29-a's next revision carries it (R-S29a-a-1)."
  - **The ticket room's capacity** ([`pr-s33b-lakeside-bodies.md`](pr-s33b-lakeside-bodies.md) header and §11,
    F-S33b-5): "the ticket room grows to at least 4.6 × 4.0 m (F-S33b-5), and S29 must change to carry it". The
    rule behind it is `bodies`' genesis capacity check (A-24).
  - Applied in §5.5 only; the open details they leave are §14's Q-S29a-5 … Q-S29a-7.
- **Rev. 4 inputs (2026-10-19).**
  - **Q-S33e-3, ruled.** The operator froze S33-d … S33-j as recommended (#285) and ruled that "the Lakeside
    ticket bay's coordinates are corrected in the S29-a revision" (coordinator's record,
    `pending-decisions.md`, 2026-10-19). The recommendation: "Move it north of the waiting room:
    `station-tickets` here (2 600, 400), `station` there (6 000, 4 600)" (`pr-s33e-pier-halt.md` §14,
    R-S29a-e-1).
  - **R-S29a-d-1, open as Q-S29a-8.** S29-d (`pr-s29d-depot-exterior.md` §10, F-S29d-1 … F-S29d-3, its
    Q-S29d-1): the towns' bay door on its north edge with the counter on the south wall; the towns' platform
    north–south, 11 × 60 m, with ten arcade piers as solids.
  - **Q-S29f-1, open as Q-S29a-9.** S29-f (`pr-s29f-station-2d.md` F-S29f-1): no `halt` tag; the 2D client tells
    the depot from the halt by the ruled `landmark` tag.
  - **Q-S29g-2, open as Q-S29a-10.** S29-g (`pr-s29g-agent-view.md` F-S29g-1): I-a-7's grep is wider than its
    meaning. **R-S29a-g-1** (same file, §10): `CORE_CONCEPTS.md`'s role-tag section (§6.5 since rev. 4; S29-g wrote §6.4) keeps the
    role tags on one line.
  - **The decision log is per file** (#258, merged 2026-10-18): `ARC-105`'s note goes into
    `docs/decisions/ARC-105.md`, and the index is regenerated with `python3 scripts/check_decision_ids.py
    --write-index`.
- **Start condition (step-29 §11.2, operator 2026-10-15).** "Nothing starts before Milestone F's checklist is
  handed to the operator. … S29-a goes first, once 12d has merged."
- **R-LOAD-1 (coordinator rule, in force).** Locally, only the targeted test files of a commit, plus `cargo fmt`
  and `cargo clippy` on the crates touched. Never `cargo test --workspace` or a whole-member suite while other
  agents run. CI on the exact head is the gate. Local timings are INCONCLUSIVE; counts are recorded.

---

## 1. Goal

Make "every town has a train station and a signature landmark" a property the framework checks, not a
convention. A World Pack declares in `world.yaml` whether it is a town (`settlement:`). A town names its station
hall and its landmark. The World Pack loader refuses, by name and file, a town whose station group lacks a hall,
a ticket office or a platform stop, whose station places are scattered, or whose station cannot be walked to from
a seat's starting place. `mineworld validate` and `mineworld packs validate` refuse a world that does not say
whether it is a town. The three reference towns get their station groups as plain Places with role tags,
passages, bodies (where `bodies` is enabled) and routines that give people a reason to go there; `bodies-yard`
declares that it is not a settlement. Adding the places shifts allocated ids, so every digest and golden fixture
of the three towns is re-recorded once, with a reason. Nothing changes in `kernel/`, `contracts/` or `systems/`
(I-S29-1).

---

## 2. Scope

### 2.1 In scope

1. **Specification, first (C1).** `docs/MODULE_SPEC.md` §4.1 (the `settlement:` key in `world.yaml`) and a new
   §4.3 "Towns: station and landmark" (the refusal table, the group rule and its stated skips);
   `docs/CORE_CONCEPTS.md` §6.5 "Role tags" (rev. 4: §6.4 is "Sit point"; the text of step-29 §14.1, as §14's rulings shape it);
   `docs/VISUAL_SLICE.md` §4 dated amendment (step-29 §14.3); a dated note under `ARC-105`.
2. **The loader (C2).** `worldpack/src/settlement.rs` (new): the `Settlement` type and its decoding, the
   `SettlementCode` enum, and the checks of §5.3. `worldpack/src/format.rs`: `WorldManifest.settlement`.
   `worldpack/src/error.rs`: one variant, `PackError::Settlement`. `worldpack/src/read.rs`: the call (read
   step 9b), the field, `settlement()` and `require_settlement()`. `worldpack/src/lib.rs`: the module and its
   re-exports. `worldpack/tests/settlement.rs` (new): the refusal table.
3. **Non-town declarations (C3).** `worlds/bodies-yard/world.yaml` (`settlement: { kind: none }`); the
   `mineworld create` template (§14 Q-S29a-3); every inline fixture world that a CLI test runs through
   `mineworld validate` or `mineworld packs validate` (§3.3's list).
4. **The three towns (C4, the re-baseline commit).** `worlds/{social-cafe,market-town,lakeside}`: three new
   place files each, the `settlement:` key, the street's (or lane's) passage and floor, three routines each,
   `pier`'s `landmark` tag; every in-tree golden and pinned fixture the shifted ids move; `scripts/baselines.txt`
   with one `reason S29-a:` line per world.
5. **The authoring gate (C5).** `tools/cli/src/main.rs` (`validate` requires and prints the settlement);
   `tools/cli/src/packs.rs` (`world_identity` requires it); `tools/cli/tests/packs.rs`
   (`every_reference_world_validates`, X-1).
6. **The scenario (C6).** `tools/cli/tests/station_visit.rs` (new): the social half of step-29 §9.3's
   `station_visit` on `market-town`, `social-cafe` and `lakeside`; the planted mutations X-2, X-3 and the
   scattered tag on copies of the reference worlds; the geometry-caused stop bound on the station walks.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The newsstand (`station-news`, its shop, the `newspaper` routine label, one person's job, E-9) | S29-b |
| The far-field `filled` slot, `FAR_FIELD_SLOT_TOWN`, Social Café's `reach_m` 60 → 100 | S29-c |
| The 3D depot, the forecourt drawn in `street.gd`, the role-tag binding, every rendered shot | S29-d, S29-e |
| `clients/shared/station/station_state.gd`, the board, window and kiosk rules | S29-e (S29-f if first) |
| 2D sprites, the `station` interior spec, i18n words | S29-f |
| `PlaceLine.roles` and the agent scenario | S29-g |
| `MODULE_SPEC.md` §6.1.1, `ART_DIRECTION.md` §4 | S29-c, S29-d |
| `overall.md` (S29 entry; the second re-baseline beside 12d's "Digests" rule) | the primary session (§11) |
| Any change under `kernel/`, `contracts/`, `systems/`, `server/`, `clients/`, `cognition/` | never (I-S29-1); the one allowed exception, a content-only fixture in a System's test, is not needed here (§3.3) |

### 2.3 Dependencies

- **12d (#171, OPEN at `e6030a36`).** It gives `social-cafe` and `market-town` their `bodies` system and every
  place a `body:` section, adds item names, and re-baselines both towns once (`origin/mvp0/pr-12d-towns`). The
  station places' `body:` sections use 12d's format, and the street's floor and its west parapets are 12d's
  (§3.1 A-9). **Required before C0.**
- **S26-a (#206, OPEN).** It adds the `appearance` pack and re-records every world's baselines and the
  `decisions` golden (step-30 A-19). Q-S29-7 ruled the re-baseline "after 12d and S26-a have merged, in one
  commit". **Required before C4.** If it has not merged when C3 is committed, the session stops and reports
  (a wait, not a material stop).
- **Milestone F's checklist handed over** (operator ruling). Recorded by the primary in its dispatch.
- Not required: RL-e0 (#220), 16c, RL-b2 — those gate S29-c and S29-d.

---

## 3. Audit (`origin/main @ e6030a36`, 2026-10-15)

### 3.1 Source anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `WorldManifest` is a derived `Deserialize` with `deny_unknown_fields`; its fields are `world`, `systems`, `places`, `population`, `items`, `organizations`, `seats`, `mineworld`, `requires`, `configure`, `decisions` | `worldpack/src/format.rs:41-93` | `settlement` is one more `#[serde(default)] Option<…>` field. An old binary refuses a `settlement:` key as unknown, which is correct |
| A-2 | `version` and `license` are "Optional to the loader, checked when stated; required by `mineworld packs validate`" | `format.rs:186-194`; enforced only in `packages::Identity::of_world` (`packages/src/identity.rs:368-388`), reached through `world_identity` (`tools/cli/src/packs.rs:157-168`) | **`mineworld validate` does not enforce `ARC-53` today** (`tools/cli/src/main.rs:548-599` calls only `read_with` and `load`). The ruled gate "`mineworld validate` / `packs validate`" therefore needs a new call in both commands (F-S29a-2) |
| A-3 | `read_with` runs the checks in a fixed order: manifest, id, `mineworld:` range, systems, decisions file, licence policy, requirements, configuration, keys, content, undeclared files, Entity Packs, seats, locations, passages, then `check_sections` and `configure::check_references` | `worldpack/src/read.rs:8-35` (the documented order), `115-224` | The settlement checks need places, passages, people and seats, so they run after `check_passages` (`read.rs:196`) and before `check_sections` (`read.rs:220`): read step **9b** |
| A-4 | `AuthoredPlace { tags, note, passages, sections }`; `AuthoredPassage { to, here, there }`; a passage is stated once, in either file, and holds both ways | `format.rs:265-279`, `310-321`; `check_passages` `read.rs:687-737` | The group search builds an **undirected** adjacency from every place's `passages` |
| A-5 | `AuthoredPerson { tags, note, location, sections }`; `location` is optional | `format.rs:246-262` | A seat whose person has no `location` has no starting place; the reachability check skips it, and the skip is stated in §4.3's text |
| A-6 | `Tags::contains(&Tag)` exists; `Tag` validates its text | `contracts/src/entity.rs:26, 78-92` | Role tags are checked with `Tags::contains`; no tag vocabulary is added to `contracts/` |
| A-7 | `PackError` variants are CamelCase, each naming the file and carrying the offending value as a field; tests assert the variant and its fields, not the prose (`worldpack/tests/refusals.rs:1-11`) | `worldpack/src/error.rs:1-14`, `133-143` | One variant `PackError::Settlement { code, path, key, detail }` with a `SettlementCode` enum whose `as_str()` is the upper-snake code, the pattern RL-e0 uses (`origin/mvp1/pr-rl-e0:packages/src/far_field.rs:93-110`) |
| A-8 | A malformed `world.yaml` arrives as `PackError::Malformed { path, kind: "world", detail }`, the detail carrying the parser's line and column | `error.rs:133-143`; `read.rs:430-455` | `SETTLEMENT_MALFORMED` is raised **while decoding**, as a `de::Error::custom` whose text begins with the code, so it keeps its line and column |
| A-9 | 12d's `social-cafe` street: `body.floor` `min: { x: -37450, y: -13000 }`, `max: { x: 30550, y: 3200 }`; 62 solids, among them "the west parapets" `{ min: { x: -37450, y: -1200 }, max: { x: -36950, y: 3200 } }` and `{ min: { x: -37450, y: -13000 }, max: { x: -36950, y: -8600 } }`; a place file has `body: { floor, solids }`, the floor one axis-aligned rectangle | `origin/mvp0/pr-12d-towns:worlds/social-cafe/places/street.yaml`; `systems/bodies/src/section.rs:1-110`; `worlds/bodies-yard/places/hall.yaml` | The forecourt is a westward growth of the street's floor and the removal of the two parapet solids. **Re-read at C0 from 12d as merged**; market-town's street is audited the same way |
| A-10 | `bodies`' section is opaque to the loader: a decoded section is held as `Arc<dyn AuthoredContent>`, which answers only `section`, `owner`, `references` and `seed` | `authoring/src/content.rs:18-41`; `ARC-31` ("the loader never learns what a section means") | **The loader cannot read a stop's `body.floor`**, so step-29 §4.2's `TOWN_PLATFORM_ANCHOR` cannot be a loader check without breaking `ARC-31` (F-S29a-1, Q-S29a-1) |
| A-11 | `mineworld validate` runs on inline fixture worlds in at least: `tools/cli/tests/{commands.rs:91, configure.rs:94, configure.rs:187, content_kinds.rs:55, walking.rs:134, decisions.rs:276, packs.rs:333, entity_packs.rs:155, entity_packs.rs:186, interactions.rs:235, requirements.rs:114, requirements.rs:256-300, milestone_e.rs:310, milestone_e.rs:369, milestone_e.rs:520, third_party.rs:396, create.rs:17}`; `packs validate` on worlds at `packs.rs:233`, `packs.rs:335`, `requirements.rs:333`, `configure.rs:188` | `grep -rn '"validate"' tools/cli/tests/` | Each fixture world that is not a copy of a reference world gains `settlement: { kind: none }` in C3 (F-S29a-2). Copies of reference worlds inherit C4's key |
| A-12 | `mineworld create` writes `tools/cli/templates/new-world/{world.yaml, places/home.yaml, people/first.yaml, people/second.yaml}` and its test runs `mineworld validate` on the result | `tools/cli/src/create.rs:16-34`; `tools/cli/tests/create.rs:8-34` | The template must declare a `settlement:` or the gate refuses every new world (Q-S29a-3) |
| A-13 | Reference worlds: `social-cafe` enables presence, movement, conversation, group-activity, relationships, naming, schedule (and `bodies` after 12d); `market-town` adds item, inventory, item-transfer, economy, employment, consumption, calendar, weather (and `bodies` after 12d); `lakeside` enables movement and schedule, no `bodies`, no `economy`; `bodies-yard` has no `schedule` and no `naming` | `worlds/*/world.yaml`; `origin/mvp0/pr-12d-towns:worlds/*/world.yaml` | Bodies sections for the station only in the two towns. Lakeside's halt has positions but no `body:`. `bodies-yard` declares `none` |
| A-14 | Every place in `social-cafe`/`market-town` joins the street and nothing else; the street's file states no passages (each other place states its doorway onto it) | `worlds/social-cafe/places/street.yaml` header (12d) | The hall states its own passage to `street`; the ticket office and the platform state theirs to `station` |
| A-15 | Routines: `{ from, place, label }`; labels `a-z0-9-`, 1–48 bytes; every boundary within 05:00–23:00 (a restart-test fixture dependency); every part of a day ≥ 4 h, ≥ 5 h "where the way there is long" | `worlds/market-town/people/alice.yaml` header; `systems/schedule/src/label.rs` | Station segments are ≥ 5 h (the station is the farthest place in both towns) and keep every boundary within 05:00–23:00 |
| A-16 | `market-town` people: only `alice` and `felix` have a `job:`; routines equal `social-cafe`'s for the same keys | `worlds/market-town/people/*.yaml` | The three station visitors are jobless in both towns (§5.6), so no wage or `produces` moves |
| A-17 | Ids are allocated places → people → items → organizations, each in key order; `station` < `station-platform` < `station-tickets` sort between `park` and `store` | `worldpack/src/load.rs:12-30`; `worldpack/tests/social_cafe.rs:159-176` (ids pinned 1…18) | In `social-cafe` and `market-town` every id from `store` on moves by +3 (`station…` < `store`); in `lakeside` the three keys sort after `square` (`sq` < `st`), so the places keep their ids and every person id moves by +3 |
| A-18 | Long baselines: `scripts/baselines.txt`, one section per world, `reason <PR>:` lines kept by `ci_parity.py baseline write`; regenerated "from any platform's long record (they are equal, ARC-49)" | `scripts/baselines.txt:1-5`; `scripts/ci_parity.py:11-40, 739-767` | C4 records one long record locally (one bounded run, §9 C4 step 9) and writes the baselines in the same commit; the nightly's `baseline check` on the final head is the canonical gate |
| A-19 | In-tree goldens that pin these worlds: `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt` (byte-for-byte `run` output, `decisions.rs:120-131`); `worldpack/tests/social_cafe.rs:81, 159-176` (key and id lists); `clients/agent/tests/fixtures/market-town-{start,street,walking}.json`; place lists in `tools/cli/tests/{client_2d.rs, milestone_c.rs, walking.rs, facts.rs, walking_pace.rs, run_restart.rs}`, `server/tests/support/mod.rs`, `clients/agent/tests/{test_two_handles.py, test_play_market_town.py}`, `worldpack/tests/refusals.rs` | `grep -rln "apartments, cafe\|park, store\|bakery, boathouse"` and the files named | C4's re-baseline list. C0 re-runs the grep on the base; C4 handles exactly that list |
| A-20 | `the_data_packs_in_named_directories_are_listed_and_validate` runs `mineworld packs validate` on every `worlds/*` with `--packs entities --packs presentation/mineworld-default` | `tools/cli/tests/packs.rs:178-238` | The `packs validate` half of `AC-S29-1` is already enforced for every reference world once C5 lands; C5 adds the `mineworld validate` half |
| A-21 | Scenario patterns: a 30-day seed-7 run with a save, read back through `headless::Tables`; `PersonEnteredPlace`, `Spoke`, `arrived` decoded from stored facts | `tools/cli/tests/rhythm.rs:1-60`; `tools/cli/tests/routines.rs:39-60, 143-200` | `station_visit` reads its save the same way; no new harness |
| A-22 | 12d's walk harness counts stopped-short walks (NW-3, restated by 12n-2 M-3 as "geometry-caused stops ≤ 10 %, person-caused printed") | `tools/cli/tests/bodies/mod.rs` (as 12d merges it); `step-11-bodies.md:8777`; `overall.md:1050` | C6 applies that counter to the station's walks (step-29 §6.5's "no stop caused by geometry") |
| A-23 | `mineworld interactions <world> --place <key> --json` prints the resolution at one place | `tools/cli/src/main.rs:255-266` | "`buy` is never offered in `station-tickets`" is checked statically, without a run |
| A-24 *(rev. 3)* | **`bodies`' capacity rule.** At genesis every shaped place must hold `4·(population − 1) + 1` points of the 650 mm lattice anchored at `floor.min + (R, R)`, each inside the floor shrunk by R and at least R from every solid (inclusive, `distance² ≥ R²`), else `bodies-capacity`. R = 250 mm on 12d (300 on `main`) | 12d head `841805ee`: `systems/bodies/src/geometry.rs:15, 64-70, 218-228, 276-295`, `genesis.rs:52-90`; `pr-s33b-lakeside-bodies.md` A-2, A-3 | Social Café and Market Town (12 people) need **45** points in every shaped place; Lakeside (10 people) needs **37** (F-S29a-8, F-S29a-9) |
| A-25 *(rev. 3)* | Lakeside's `lane` frame: origin its south-west corner, x east, y north, ≈ 30 × 6 m; the houses open onto its north side, the shore and the pier onto its south side; the lake lies south | `worlds/lakeside/places/lane.yaml:1-6`; `step-33-lakeside-3d.md` §4.0 | The lane's east edge is x = 30 000; 400 mm inside it, on its axis, is (29 600, 3 000) (12d's inset, `pr-s33b-lakeside-bodies.md` A-5) |
| A-26 *(rev. 4)* | **A client places a place only by translation.** "The world frame is fixed (+x east, +y north), so a binding only translates": a room's origin is `in_point(door) − here`; doors are inset 400 mm (`DOOR_INSET`) | 16c `clients/3d/scripts/slice/slice_layout.gd` (as cited by `pr-s29d-depot-exterior.md` A-3); `docs/CORE_CONCEPTS.md` §6.1 | Every station place's frame must already face the way it is drawn. A passage's `here` and `there` each stand 400 mm inside **opposite** sides of the shared wall, so the wall lies midway and `here` is 400 mm inside the edge of its own floor that faces the neighbour |
| A-27 *(rev. 4)* | **The doorway rule, written once.** "A place's floor in the lane's frame is `floor + (there − here) ± 800 mm across the wall`"; S33-b checks that Lakeside's translated floors are pairwise disjoint (I-b-3, AC-b-6) | `pr-s33b-lakeside-bodies.md` §5.1, I-b-3, AC-b-6 | No test checks it for the towns. C6 step 8 adds the same rule for the station group and its street (Q-S29a-11) |
| A-28 *(rev. 4)* | **The arcade is real and 8.2 m deep.** "27 × 650 ft (8.2 × 198.1 m), connecting the passenger terminal with the baggage and express rooms"; "A covered concourse (west side)"; the tracks lie west of the depot | Wikipedia (read 2026-10-19); NRHP 72000248 §7 and its maps (`pr-s29d-depot-exterior.md` §3.2) | The platform lies west of the hall and runs north–south; it is deep enough for the arcade plus an open strip, and the arcade's outer piers are solids (12e: a drawn collider needs a disclosed solid) |

### 3.2 Findings

- **F-S29a-1. `TOWN_PLATFORM_ANCHOR` is not a loader check.** The rule reads `body.floor` of the stop, and
  `body:` is the `bodies` pack's section, opaque to the loader by `ARC-31` (A-10). The loader could only check it
  by decoding `bodies`' format a second time (two decoders of one format) or by teaching `AuthoredContent` a
  geometry query (a domain word in the authoring seam). Neither is acceptable. **Q-S29a-1** proposes the
  smallest honest relocation; C2 implements every other code, and the anchor rule's placement follows the ruling.
- **F-S29a-2. "Required by `mineworld validate`" is new for that command.** `ARC-53`'s precedent is enforced by
  `packs validate` only (A-2). The ruling names both commands, so both call `require_settlement` (C5), and every
  inline fixture world validated through either command declares `settlement: { kind: none }` (C3, A-11). The
  churn is ≈ 17 call sites in ≈ 13 test files plus the `create` template; it is recorded rather than avoided,
  because the ruling is explicit. (`ARC-105` option (c), "the loader requires it", stays rejected: `run`,
  `server` and every in-crate `WorldPack::read` fixture are untouched.)
- **F-S29a-3. Invariant I-S29-4 names the landmark; §4.2's table checks only the station.** "The landmark is
  reachable on foot from every seat's starting place (a load-time check where `movement` is enabled)". In two
  towns the landmark is the hall, so the station check covers it; in Lakeside the landmark is `pier`, which no
  code checks. **Q-S29a-2.**
- **F-S29a-4. The step's C3 and C5 cannot keep `main` green in that order.** Requiring the key (step C3) before
  the towns declare it (step C4) turns every reference-world test red; adding places (step C4) before the
  re-baseline (step C5) does the same, because ids shift at once (A-17). This design keeps the step's content
  and orders it so that every commit is green: the towns' content and the whole re-baseline are **one commit**
  (C4, which is the ruled "one commit"), and the gate follows it (C5). The mapping is in §9.
- **F-S29a-5. Station segments must be long.** The hall's doorway is ≈ 25 m west of the street's present west
  end, so it is the farthest place in both towns. A-15's rule gives ≥ 5 h segments; C6 measures arrival.
- **F-S29a-6. The 3D slice cannot draw the station before S29-d.** After C4, Social Café people walk to
  `station` on the server, and the slice (which binds `cafe` and `street` only, step-29 A-13) shows them leave
  the street at its west end. The server's place change is correct; the picture is incomplete until S29-d.
  **Q-S29a-4.**
- **F-S29a-7. Q-S29-7's "one re-baseline" also names S29-b's organization.** The organization moves no id
  (organizations are allocated last and `station-news` sorts last), but its sales move `market-town`'s facts in
  S29-b. This design reads the ruling as one **id shift**, done here; S29-b records its own behaviour change
  (R-S29b-a-1). The reading is listed for the primary in §11 rather than asked, because the baselines file's own
  rule already requires every behaviour change to be recorded.
- **F-S29a-8 (rev. 3). The towns' ticket bay of revision 2 fails `bodies-capacity`.** Its floor, 6 000 × 3 000
  mm with the counter at y 2 400 … 3 000, admits 9 columns × 3 rows = **27** lattice points (rows at y 250, 900,
  1 550; y 2 200 is 200 mm from the counter); 12 people need 45 (A-24). The operator's minimum, 4.6 × 4.0 m, is
  stated for Lakeside's 10 people; for the towns the same rule needs more. A 6 000 × 4 000 floor with a 600 mm
  counter on its far wall admits 9 × 5 = **45**, exactly the need; 6 000 × 4 600 admits 9 × 6 = **54**.
  → **Q-S29a-5**.
- **F-S29a-9 (rev. 3). A 4.6 × 4.0 m bay holds the population only while it has no counter.** Empty, it admits
  7 × 6 = 42 points (≥ 37). S33-b's bay has "the counter, h 1 100" (`pr-s33b-lakeside-bodies.md` §5.3): a 600 mm
  counter on a 4.6 m wall leaves 7 × 5 = **35**, on a 4.0 m wall 6 × 6 = **36**, both under 37; only a counter
  at most 250 mm deep keeps 42. A 4 600 × 4 600 floor with a 600 mm counter on its far wall admits 7 × 6 = 42.
  → **Q-S29a-6** (S33-b's frozen table says "≥ 4 600 × 4 000", so 4 600 × 4 600 satisfies it unchanged).
- **F-S29a-10 (rev. 3). "The track on its west edge" puts the track between the waiting room and the
  platform.** Q-S33a-1's recommendation, ruled "as recommended", gives the platform "4 m (x) × 40 m (y) with the
  track on its west edge". The waiting room is joined to the lane's east edge, so it lies west of the platform
  (step-33 §4.0's plan: "waiting room 8×5 ┐ platform 40×4, track N–S"); a track on the platform's west edge
  would run between them, and a passenger would cross it to board. The ruling's own words — "a north–south
  platform with its end against the shore" — are met either way. → **Q-S29a-7** (recommended: the track on the
  platform's **east** edge, the waiting room's door on its west edge).
- **F-S29a-11 (rev. 4). The towns' ticket bay faces the wrong way.** Its door `here` (3 000, 400) is 400 mm inside
  its **south** edge, and the hall's `there` (8 400, 400) is 400 mm inside the hall's south edge. Translated
  (A-26), the bay's floor runs north from the hall's south wall, into the hall. Example: the bay would cover hall
  x 5.4 … 11.4 m, y 0 … 4.6 m, the hall's own floor south of the benches. The bay must lie south of the hall, so its door is
  400 mm inside its **north** edge, (3 000, 4 200), and its counter on the far (south) wall, y 0 … 600. The
  mirror keeps the capacity: 9 × 6 = 54 points (F-S29a-8's arithmetic, rows mirrored). Found by S29-d
  (F-S29d-1). → Q-S29a-8.
- **F-S29a-12 (rev. 4). The towns' platform runs east–west.** Its floor is x −30 000 … 30 000, y 0 … 6 000 and its
  door `here` (0, 5 600) is on its north edge; the hall's `there` (400, 25 900) is on the hall's **west** wall.
  Translated, the platform lies across that wall and 30 m into the hall. The real platform runs north–south along
  the hall's west side (A-28). So the frame is x across, y along: floor x 0 … 11 000, y −30 000 … 30 000; the
  track edge on the **west** (x = 0), the anchor (0, 0) on it at mid-length; the door `here` (10 600, 0), 400 mm
  inside the east edge, facing the hall. Found by S29-d (F-S29d-2). → Q-S29a-8.
- **F-S29a-13 (rev. 4). The arcade needs room and solids.** The real arcade is 8.2 m deep (A-28). On revision 3's
  6 m platform it would overhang the track by 2.2 m, and its outer piers would stand on walkable floor with no
  solid under them. The platform therefore grows to 11.0 m (the 8.2 m arcade, then a 2.8 m open strip to the
  edge), and this PR writes the ten outer piers as solids: 10 piers + 2 benches = 12 ≤ 64 (S21 A-12). Found by
  S29-d (F-S29d-3). → Q-S29a-8.
- **F-S29a-14 (rev. 4). Lakeside's ticket bay cannot be drawn either.** Revision 3 keeps revision 2's bay points,
  `here` (1 500, 400) in the bay and `there` (6 000, 400) in the waiting room: both 400 mm inside each room's
  south edge, so no shared wall exists. Drawn either side, the bay overlaps the platform by 1.1 m or stands over
  the water. Q-S33e-3, ruled, moves the bay north of the waiting room: `here` (2 600, 400), `there` (6 000,
  4 600). Then the bay is waiting-room x 3 400 … 8 000, y 5 000 … 9 600, clear of the lane and of the platform
  (x ≥ 8 000). Found by S33-e (F-S33e-1).
- **F-S29a-15 (rev. 4). Nothing caught F-S29a-11, -12 or -14 at design time.** All three are one defect: a pair
  of passage points that do not face each other across one wall. S33-b checks Lakeside's translated floors
  (A-27); no check covers the towns. A content test with the same rule, over the station group and its street,
  catches the class in any world with bodies, not only these frames. → Q-S29a-11.
- **F-S29a-16 (rev. 4). I-a-7's grep is wider than its meaning.** "A role tag grants no behaviour" is the rule.
  Its check, "`git grep '"ticket-office"\|"landmark"' -- systems clients cognition` empty", would also refuse the
  clients that bind a **drawing** or order **lines** by role tag (S29-d/e bind the 3D depot by tag, S29-f binds
  `facade:station+landmark`, S29-g's `roles.py` orders the agent's place lines). Choosing a picture or an order
  is not behaviour. Found by S29-g (F-S29g-1). → Q-S29a-10.
- **F-S29a-17 (rev. 4). No `halt` tag.** Step-29 §8.1 picks Lakeside's 2D sprite by an extra world tag `halt`.
  This PR never authored it, and it would be a picture hint living in world data. S29-f tells the two apart by the
  ruled `landmark` tag: a station that is its town's landmark is drawn as the depot, any other as the halt.
  Lakeside's hall is not its landmark (Q-S29-3: the pier is), so no world change is needed. Found by S29-f
  (F-S29f-1). → Q-S29a-9.

### 3.3 What does not exist

No `settlement` anywhere in the tree (`git grep -n settlement` returns only plan files). No station place in
any world. No System or kernel code reads a role tag, and none will (I-S29-1). No test asserts that a world
is a town.

---

## 4. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-a-1 (I-S29-1) | `git diff --name-only origin/main...HEAD` lists nothing under `kernel/`, `contracts/`, `systems/`, `server/src/`, `clients/`, `cognition/`, `sdk/` | C7 diff scan (recorded) |
| I-a-2 (I-S29-2) | Every town declares a station group (hall, ticket office, stop) and a landmark; a world that declares neither `town` nor `none` is refused by `mineworld validate` and `packs validate` with `SETTLEMENT_UNDECLARED`, naming `world.yaml` | `worldpack/tests/settlement.rs`; X-1 |
| I-a-3 | The key is optional to the loader: `run`, `server` and `WorldPack::read` accept a world without it, unchanged | `worldpack/tests/settlement.rs::a_world_without_a_settlement_still_loads` |
| I-a-4 | When present, `settlement:` is validated in full by the loader, so `run`, `server` and `validate` refuse the same malformed town with the same code | the table test (the loader path) and X-2 through `mineworld run` |
| I-a-5 (I-S29-4) | Where `movement` is enabled, every seat with a starting place reaches the hall by passages (and the landmark, `TOWN_LANDMARK_UNREACHABLE`, Q-S29a-2 ruled) | `TOWN_STATION_UNREACHABLE`; X-3 |
| I-a-6 (I-S29-6) | Every station dimension is a real or a stated design value with its source in the place file's comment; the one compression (the platform's length) is stated | review of C4's files |
| I-a-7 *(rev. 4, Q-S29a-10)* | A role tag grants no behaviour: no System or cognition code reads `station`, `stop`, `platform`, `ticket-office` or `landmark`, and no client decides what is possible by one. A client may choose a drawing or order lines by a role tag (S29-d, S29-e, S29-f, S29-g); what a person can do there still comes only from Systems | I-a-1's diff scan (this PR changes no client); `git grep -n '"ticket-office"\|"landmark"' -- systems cognition` empty |
| I-a-8 | Nothing pretends (step-29 §5.2): no shop at `station-tickets`, no routine label about trains, no train | `station_visit`'s interaction check; review of C4's routines |
| I-a-9 | One re-baseline: every moved digest and golden is re-recorded in C4 and nowhere else, each with a reason; no summary or fact moves except by the added places, routines and ids | C4's evidence table (§15) |

---

## 5. Design

### 5.1 The key (MODULE_SPEC §4.1, written by C1)

```yaml
settlement:            # required by `mineworld validate` and `packs validate`; validated whenever present
  kind: town           # town | none
  station: station     # kind town: required. A key of `places`: the station hall
  landmark: station    # kind town: required. A key of `places`; may equal `station`
```

```yaml
settlement:
  kind: none           # not a settlement: a test yard, a lone cabin, a void arena
```

```rust
// worldpack/src/settlement.rs (the exact shape C2 writes)
pub enum SettlementKind { Town { station: EntityKey, landmark: EntityKey }, None }
pub struct Settlement { kind: SettlementKind }      // pub fn kind(&self) -> &SettlementKind
pub enum SettlementCode {
    Undeclared, Malformed, StationUnknown, LandmarkUnknown, StationUntagged, LandmarkUntagged,
    StationNoStop, StationNoTicketOffice, StationScattered, StationUnreachable,
    LandmarkUnreachable,   // Q-S29a-2 (ruled); no PlatformAnchor: Q-S29a-1 moved it to its consumer
}
impl SettlementCode { pub const ALL: [Self; N]; pub const fn as_str(self) -> &'static str }
//   "SETTLEMENT_UNDECLARED", "SETTLEMENT_MALFORMED", "TOWN_STATION_UNKNOWN", "TOWN_LANDMARK_UNKNOWN",
//   "TOWN_STATION_UNTAGGED", "TOWN_LANDMARK_UNTAGGED", "TOWN_STATION_NO_STOP",
//   "TOWN_STATION_NO_TICKET_OFFICE", "TOWN_STATION_SCATTERED", "TOWN_STATION_UNREACHABLE", …
```

**Decoding.** `Settlement` implements `Deserialize` by hand with a map `Visitor` (not a derive), so that every
malformation is a `de::Error::custom` message beginning `SETTLEMENT_MALFORMED: ` and keeping the parser's line
and column (A-8):

| Input | Message after `SETTLEMENT_MALFORMED: ` |
| --- | --- |
| a key other than `kind`, `station`, `landmark` | ``unknown key `<k>`; expected `kind`, `station`, `landmark` `` |
| a key stated twice | ```<k>` is stated twice `` |
| `kind` missing | ```kind` is required: `town` or `none` `` |
| `kind` not `town` / `none` | ``kind `<v>` is neither `town` nor `none` `` |
| `kind: none` with `station` or `landmark` | ``kind `none` names no `<k>` `` |
| `kind: town` without `station` or `landmark` | ``kind `town` requires `<k>`: a key of `places` `` |
| `station` / `landmark` not a valid key | the `EntityKey` error, verbatim |
| not a map | `settlement is a map with `kind`` |

### 5.2 The refusal (`PackError::Settlement`)

```rust
/// A town that breaks the station-and-landmark contract (DECISIONS.md ARC-105, MODULE_SPEC.md §4.3).
#[error("{code}: {path}: {detail}")]
Settlement {
    code: SettlementCode,       // Display = as_str()
    path: PathBuf,              // world.yaml, or the place file the refusal is about
    key: Option<EntityKey>,     // the place or seat named
    detail: String,             // one sentence, naming the key and what is missing
},
```

`SETTLEMENT_MALFORMED` stays a `PackError::Malformed` (A-8), its detail beginning with the code.

### 5.3 The checks (read step 9b), in this order, first failure refuses

Run only when `manifest.settlement` is `Some(Town { station, landmark })`. `Some(None)` and absent run none.
`movement` enabled means `systems.contains(&catalog::PASSAGE_OWNER)` (`worldpack/src/catalog.rs`).

1. `station` is a key of `places`, else `TOWN_STATION_UNKNOWN` (path `world.yaml`; detail lists the places,
   `names()`).
2. `landmark` is a key of `places`, else `TOWN_LANDMARK_UNKNOWN`.
3. The hall carries `station` and `public`, else `TOWN_STATION_UNTAGGED` (path: the hall's file; detail names
   the missing tag(s)).
4. The landmark carries `landmark`, else `TOWN_LANDMARK_UNTAGGED` (path: the landmark's file).
5. **The group.** With `movement`: breadth-first from the hall over the undirected passage adjacency (A-4),
   entering only places tagged `station`; the group is every place reached, the hall included. Without
   `movement`: every place tagged `station`. Keys are visited in `BTreeMap` order, so the result and every
   refusal are deterministic.
6. Some place of the group carries both `stop` and `platform`, else `TOWN_STATION_NO_STOP` (path `world.yaml`;
   key the hall).
7. Some place of the group carries `ticket-office`, else `TOWN_STATION_NO_TICKET_OFFICE`.
8. With `movement`: every place tagged `station` is in the group, else `TOWN_STATION_SCATTERED` (path: the
   first such place's file in key order; key that place). Without `movement`: skipped (the group is the tagged
   set by definition).
9. With `movement`: for every seat in key order whose person has a `location`, breadth-first over **all**
   passages from that place reaches the hall, else `TOWN_STATION_UNREACHABLE` (path: the person's file; key the
   seat; detail names the starting place). A seat without a location is skipped. Without `movement`: skipped.
10. *(Q-S29a-2, ruled.)* When the landmark is not the hall, the same search reaches the landmark, else `TOWN_LANDMARK_UNREACHABLE`.

The skips (no `movement`; a seat with no location) are written into MODULE_SPEC §4.3's notes, not hidden.

`WorldPack` gains `settlement: Option<Settlement>`, `pub fn settlement(&self) -> Option<&Settlement>` and
`pub fn require_settlement(&self) -> Result<&Settlement, PackError>` (`SETTLEMENT_UNDECLARED`, path
`world.yaml`, key `None`, detail "world.yaml does not say whether this world is a town: add `settlement: {
kind: town, station: <place>, landmark: <place> }` or `settlement: { kind: none }` (MODULE_SPEC §4.3)").
`WorldPack::in_memory` sets `settlement: None`.

### 5.4 The authoring gate (C5)

- `tools/cli/src/main.rs` `validate`: after `read_with` and `load`, `pack.require_settlement().map_err(described)?`;
  print `  settlement town (station <k>, landmark <k>)` or `  settlement none` after the `seats` line.
- `tools/cli/src/packs.rs` `world_identity`: `world.require_settlement().map_err(described)?` before
  `Identity::of_world`.
- Neither `run` nor `server` nor `create` calls it (I-a-3).

### 5.5 The station places (C4)

Frames follow `CORE_CONCEPTS.md` §6.1: millimetres from the place's own origin, `x` east, `y` north. Values marked
*design* are this PR's and are re-measured by S29-d; values marked *real* cite step-29 §2.2.

**`station`** (the hall; `social-cafe`, `market-town`)

```yaml
# The station hall: the waiting room of the town's railway station, after the Santa Fe Depot, San Diego
# (1915; NRHP 72000248; HABS CA-1965). 51.8 m by 16.8 m inside (real: the main waiting room, 170 × 55 ft),
# its long axis north–south, its east façade on the street's axis. Millimetres from its inner south-west
# corner; x east, y north (docs/CORE_CONCEPTS.md §6.1).
tags: [station, public, landmark]
note: >-
  The station's waiting hall: oak benches in rows, a high timber ceiling, a tiled wainscot. The ticket window
  is off the south end; the platform is through the doors on the west side.
passages:
  # The arch on the east façade, 0.4 m inside; in the street, 0.4 m in front of the façade (design).
  - to: street
    here: { x: 16400, y: 25900 }
    there: { x: -62050, y: -4900 }
body:
  floor: { min: { x: 0, y: 0 }, max: { x: 16800, y: 51800 } }
  solids:   # 34 bench rows (seat 450 mm; 6 000 × 600 mm each; 1 800 mm between rows; aisles 1 200 / 2 400) and the kiosk
    # bank west, rows k = 0 … 16 at y = 6000 + 2400 k
    - { min: { x: 1200, y: 6000 }, max: { x: 7200, y: 6600 }, height: 450 }
    # … one line per row, k = 1 … 16 (y + 2400 k) …
    # bank east, the same rows
    - { min: { x: 9600, y: 6000 }, max: { x: 15600, y: 6600 }, height: 450 }
    # … k = 1 … 16 …
    # the kiosk (design), south-west of the benches
    - { min: { x: 1200, y: 1500 }, max: { x: 3200, y: 3500 }, height: 2400 }
```

C4 writes all 35 solids out (no `…`), each row on its own line, generated by hand from the rule above and
checked by the reviewer against it: row k's `y` is `6000 + 2400·k` to `6600 + 2400·k`, for k = 0…16, in both
banks. The doorways (16400, 25900), (400, 25900) and (8400, 400) lie in aisles (the rows nearest them end at
25 800 and begin at 27 600; the side aisles are 1 200 mm). 35 ≤ 64 solids (S21 A-12).

**`station-tickets`** (the ticket window bay)

*Rev. 3 (the S33-b capacity ruling, 2026-10-18; F-S29a-8; size per Q-S29a-5's recommendation).* Revision 2's
floor was 6 000 × 3 000 with the counter at y 2 400 … 3 000; it fails `bodies-capacity` (27 < 45). The floor
grows north to 4 600 and the counter moves with the far wall; nothing else in the file changes.

*Rev. 4 (F-S29a-11, Q-S29a-8).* The bay lies **south** of the hall, so its door is 400 mm inside its **north**
edge and the counter stands on its far, **south** wall. The floor, the counter's size and the capacity (54) are
revision 3's, mirrored.

```yaml
# The ticket window: a counter bay off the south end of the hall, 6 m by 4.6 m (design; at least 4.6 × 4.0 m by
# the operator's capacity ruling of 2026-10-18, and deep enough that bodies' capacity rule holds the town's 12
# people with the counter in place: 54 points of the 650 mm lattice, 45 needed). The bay lies south of the hall:
# its door is on its north edge, the counter on its south wall. Shut: no shop is declared here until S21's
# `fares` composes one (step-29 §5.2), so nothing is sold and nothing pretends to be.
tags: [station, ticket-office, public]
note: A timber counter screen with two grilles. The shutters are down.
passages:
  - to: station
    here: { x: 3000, y: 4200 }
    there: { x: 8400, y: 400 }
body:
  floor: { min: { x: 0, y: 0 }, max: { x: 6000, y: 4600 } }
  solids:
    - { min: { x: 0, y: 0 }, max: { x: 6000, y: 600 }, height: 1050 }   # the counter, 1.05 m (VISUAL_SLICE §3)
```

**`station-platform`** (S21's stop)

*Rev. 4 (F-S29a-12, F-S29a-13, Q-S29a-8).* Revision 3's frame ran east–west (floor x −30 000 … 30 000, y 0 …
6 000, door `here` (0, 5 600), two benches); drawn by translation it lay across the hall. The platform now runs
north–south along the hall's west wall, 11 m deep for the real arcade, with the arcade's ten outer piers as solids.

```yaml
# The platform: S21's stop for this town (a Place a line serves). 11 m by 60 m, running north–south along the
# hall's west wall (real: the tracks lie west of the depot; NRHP 72000248 §7). 60 m long: the real platforms are
# about 198 m, compressed to the street's scale (step-29 §6.2). 11 m deep: the arcade, 8.2 m (real: 27 × 650 ft),
# then a 2.8 m open strip to the edge (design). Its origin is the platform anchor: the track edge, mid-length
# (docs/CORE_CONCEPTS.md §6.5); x = 0 is the edge, the track lies at x < 0, the hall's wall at x = 11 000.
tags: [station, stop, platform, public, outdoor]
note: A long platform under a tiled arcade on round arches, beside two tracks. No train has come yet.
passages:
  - to: station
    here: { x: 10600, y: 0 }
    there: { x: 400, y: 25900 }
body:
  floor: { min: { x: 0, y: -30000 }, max: { x: 11000, y: 30000 } }
  solids:
    # the arcade's outer piers, 600 mm square, 4 m high (design), at y = ±2700, ±8100, ±13500, ±18900, ±24300
    - { min: { x: 2200, y: -24600 }, max: { x: 2800, y: -24000 }, height: 4000 }
    # … one line per pier, ten in all (y centre ± 300) …
    # two benches against the hall wall (seat 450 mm)
    - { min: { x: 10100, y: -10000 }, max: { x: 10700, y: -8100 }, height: 450 }
    - { min: { x: 10100, y: 8100 }, max: { x: 10700, y: 10000 }, height: 450 }
```

C4 writes all ten piers out (no `…`): centres y = ±2 700, ±8 100, ±13 500, ±18 900, ±24 300, each spanning
y centre − 300 … centre + 300 and x 2 200 … 2 800. The door (10 600, 0) is 7.8 m east of the piers' line and
8.1 m from each bench, so the way in is clear. 12 ≤ 64 solids (S21 A-12). Capacity: the floor holds far more than
45 lattice points.

The platform's origin (0, 0) lies on its floor's west edge and on no other edge (the anchor rule, whatever
now held by C6's content test and by RT-c, Q-S29a-1).

**The street** (`social-cafe`, `market-town`). `body.floor.min.x` moves from `-37450` to `-62450` (the forecourt,
25 m, step-29 §6.3); the two "west parapets" solids are deleted, and the comment says why ("the street opens into
the station forecourt; step-29 §6.3"). The hall's `there` point (−62 050, −4 900) is 0.4 m inside the new west
edge, on the street's axis (midway between the façades at y 3 200 and −13 000). C0 confirms both towns' street
floors and parapets against 12d as merged; a different value is a bounded correction recorded in §15.

*Rev. 4 — the towns' station laid out by the doorway rule (A-27), in the hall's frame.* The street x 16 800 …
109 800, y 17 800 … 34 000 (east of the hall's east wall); the bay x 5 400 … 11 400, y −4 600 … 0 (south of the
hall's south wall); the platform x −11 000 … 0, y −4 100 … 55 900 (west of the hall's west wall). The hall itself
is x 0 … 16 800, y 0 … 51 800. All four are pairwise disjoint; C6 step 8 holds this (Q-S29a-11).

**Lakeside's halt** (`lakeside`: no `bodies`, positions semantic, A-13):

| Key | Tags | Passages | Comment's size |
| --- | --- | --- | --- |
| `station` | `station`, `public` | `lane`: here (400, 2500), there (29600, 3000) | waiting room ≈ 8 × 5 m (design), x east, y north, **at the lane's east end** (Q-S33a-1): its west door onto the lane's east edge, its east door onto the platform |
| `station-tickets` | `station`, `ticket-office`, `public` | `station`: here **(2600, 400)**, there **(6000, 4600)** *(rev. 4, Q-S33e-3)* | a window bay **4.6 × 4.6 m** (the capacity ruling, ≥ 4.6 × 4.0 m; F-S33b-5; F-S29a-9, Q-S29a-6): 42 lattice points with a 600 mm counter on its far wall, 37 needed. **North of the waiting room** (F-S29a-14): its door 400 mm inside its south edge, the counter on its far (north) wall |
| `station-platform` | `station`, `stop`, `platform`, `public`, `outdoor` | `station`: here (−3600, −16000), there (7600, 2500) | **4 m × 40 m, running north–south** (Q-S33a-1): floor x −4 000 … 0, y −20 000 … 20 000; origin on the track edge (the east edge, x = 0; Q-S29a-7), mid-length; the buffer stop at the south end (y −20 000), against the shore; the door from the waiting room 400 mm inside the west edge, 4 m from the buffer stop |

*Rev. 3 (Q-S33a-1 and the S33-b capacity ruling, both 2026-10-18).* Revision 2 put the halt at the lane's
**west** end (`station` here (4000, 400), lane there (400, 3000)), the bay at ≈ 3 × 2 m, and the platform's frame
40 m along x with the track on its south edge (here (0, 3600), there (4000, 4600)). Only those three rows change;
the keys, tags, `pier`'s `landmark` tag, the routines (§5.6) and the id arithmetic (A-17) do not. The waiting
room's own size and its bay door are unchanged. Positions stay semantic until S33-b adds Lakeside's bodies, which
follow these frames (`pr-s33b-lakeside-bodies.md` R-b-5, F-S33b-6: "If Q-S33a-1 gives Lakeside a north–south
platform (x across, y along), S29-a writes the points for that frame").

*Rev. 4 (Q-S33e-3, ruled; F-S29a-14).* Only the bay's two points change. Laid out by the doorway rule (A-27), in
the waiting room's frame: the bay x 3 400 … 8 000, y 5 000 … 9 600; the platform x 8 000 … 12 000, y −1 500 …
38 500; the lane x −30 000 … 0. The three are pairwise disjoint (the bay and the platform touch at x 8 000), and in
the lane's frame the buffer stop lies at y −1 000, against the shore (S33-e's plan: platform scene x 23 … 27,
z −36 … 4).

**No `halt` tag** *(rev. 4, F-S29a-17, Q-S29a-9).* No station place carries a tag that only steers a picture.
Lakeside's hall is `[station, public]` and the towns' halls `[station, public, landmark]`; a client tells the
depot from the halt by `landmark`, which is already ruled data.

`pier.yaml` gains the tag `landmark` (its other tags unchanged) and a comment line ("the village's landmark,
step-29 Q-S29-3").

**`world.yaml`** of each town: `places:` gains the three keys in key order; the new block after `seats:`:

```yaml
# This world is a town: its station hall and its signature landmark (docs/MODULE_SPEC.md §4.3, ARC-105).
settlement:
  kind: town
  station: station
  landmark: station        # lakeside: pier
```

### 5.6 Routines (C4; owner `schedule`; labels true before trains, step-29 §7.1)

| World | Person | Routine after C4 (before → after) |
| --- | --- | --- |
| social-cafe, market-town | `ivan` | `07:00 street stroll, 11:00 cafe chat, 15:00 park walk, 19:00 apartments home` → `07:00 station meet, 12:00 cafe chat, 16:00 park walk, 20:00 apartments home` |
| social-cafe, market-town | `erin` | `08:00 park read, 14:00 cafe lunch, 18:00 apartments home` → `07:30 station meet, 13:00 cafe lunch, 18:00 apartments home` |
| social-cafe, market-town | `bob` | `07:00 cafe coffee, 11:00 park read, 15:00 store errands, 19:00 apartments home` → `07:00 cafe coffee, 11:00 station sit, 16:00 store errands, 20:00 apartments home` |
| lakeside | `celia` | `06:00 square walk` → `06:00 station meet` (rest unchanged) |
| lakeside | `greta` | `07:00 square well` → `07:00 station meet` (rest unchanged) |
| lakeside | `hugo` | `11:00 square news` → `11:00 station sit` (rest unchanged) |

- Every station segment is ≥ 5 h in the towns and ≥ 4 h in Lakeside (A-15); every boundary is within
  05:00–23:00; every other part of each day stays ≥ 4 h; every Lakeside person still spends one part of the day
  at the water.
- Overlaps: `ivan` and `erin` share the hall 07:30–12:00, `bob` joins from 11:00; `celia` and `greta` share
  07:00–10:00. So at least one pair overlaps every day.
- No visitor is jobless-by-change: none of the six has a `job:` (A-16). `alice`, `felix`, `otto`, `visitor`,
  `wanderer` are untouched (pinned by `routines.rs`, the restart tests and the agent fixtures).
- Each person file's routine comment gains one line: "the station: the town's meeting point (step-29 §7.1)".

---

## 6. Reuse

Nothing here is commodity infrastructure. The group search is a dozen lines of breadth-first search over a
`BTreeMap` adjacency; a graph crate (`petgraph`) would be a dependency larger than the problem and would need a
deterministic visiting order this code gets from `BTreeMap` for free. No `DEP` record is needed.

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | `cargo fmt --check`; `cargo clippy -p mineworld-worldpack -p mineworld-cli --all-targets -- -D warnings`; the doc scripts | locally per commit; CI `fast` |
| Unit / crate | every `SettlementCode` from a planted fixture, asserting the variant, the code, the path and the key (not prose); `SETTLEMENT_MALFORMED` keeps line and column; `kind: none` accepted; a world without the key loads | `worldpack/tests/settlement.rs` |
| CLI | the gate on both commands; `every_reference_world_validates`; X-1 on a copy of `market-town` | `tools/cli/tests/packs.rs` |
| Integration (headless, real binary) | `station_visit` on three towns, 30 days, seed 7: arrivals, a conversation or group activity, save → restart → resume identical; no `buy` at `station-tickets`; X-2, X-3, scattered via `mineworld run` on copies; the geometry-caused stop bound on station walks | `tools/cli/tests/station_visit.rs` |
| Re-baseline | every pinned golden of §3.1 A-19 and the long baselines, re-recorded once with reasons | C4 |
| Nightly | the long record's `baseline check` on the final head (the canonical evidence that only C4's columns moved) | `nightly.yml` (dispatched once, §9 C7) |
| Gate 1 (real LLM) | NOT REQUIRED: no LM-facing surface changes | — |
| Gate 2 (rendered) | NOT REQUIRED here: no client changes; rendered checks are S29-d/e/f | — |

Oracles are the authored files and the stored facts, never a value the loader reports about itself. No expected
digest is computed by the code under test: the re-baselined digests are evidence of the new history, checked by
the reviewer against the reason line's claim (only the expected columns moved), not proof of correctness.

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-a-1 | `mineworld validate` and `mineworld packs validate` pass on every `worlds/*` (with `--packs entities --packs presentation/mineworld-default`), and each prints the settlement | AC-S29-1 |
| AC-a-2 | **X-1:** a copy of `market-town` without `settlement:` → both commands fail with `SETTLEMENT_UNDECLARED` naming `world.yaml`; `mineworld run` of the same copy still runs (I-a-3) | X-1, AC-S29-1 |
| AC-a-3 | **X-2:** a copy without `stop` on `station-platform` → `TOWN_STATION_NO_STOP`, by `validate` and by `run` | X-2 |
| AC-a-4 | **X-3:** a copy without the street ↔ station passage → `TOWN_STATION_UNREACHABLE` naming a seat | X-3 |
| AC-a-5 | **Scattered:** a copy whose `park` also carries `station` → `TOWN_STATION_SCATTERED` naming `park` | S29-a row |
| AC-a-6 | `worldpack/tests/settlement.rs`: one planted world per code of §5.3 (and per §14's rulings), each asserting the variant, the code, the path and the key; every malformation of §5.1's table refused at its line | §4.2 |
| AC-a-7 | **`station_visit`, `market-town`, 30 days, seed 7:** on every day ≥ 3 distinct people enter `station`; on every day ≥ 1 `spoke` fact (or `group-activity` fact) whose speaker is in `station`; a run saved at day 15, restarted and resumed to day 30 has a save digest equal to the uninterrupted run's | AC-S29-2 (social half), the S29-a checkpoint |
| AC-a-8 | The same arrivals and meeting on `social-cafe` and `lakeside` | AC-S29-2 |
| AC-a-9 | `mineworld interactions market-town --place station-tickets --json` lists no `buy` | step-29 §9.3 |
| AC-a-10 | On the station walks of AC-a-7's run, geometry-caused stops ≤ 10 % of the walkers' accepted arrivals, person-caused stops printed (NW-3 as restated) | step-29 §6.5 |
| AC-a-11 | `routines.rs` (unedited in claim) still finds every person at ≥ 90 % of their segments, the six station visitors included | NW-1; F-S29a-5 |
| AC-a-12 | The re-baseline: every file of A-19 and every world section of `scripts/baselines.txt` that moved is re-recorded in C4 only, each with one reason line naming S29-a and the columns that moved; `bodies-yard`'s section does not move; the nightly's `baseline check` on the final head passes | I-a-9, Q-S29-7 |
| AC-a-13 | I-a-1's diff scan is empty | I-S29-1 |
| AC-a-14 *(rev. 4)* | `the_station_places_lay_out_without_overlap` passes on every town with `bodies` and on the moved-station variant; M-a-5 and M-a-6 each turn it red, naming the place | F-S29a-15, Q-S29a-11 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and its recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted
validation per commit; the PR's CI is the one full run.

**Mapping to the step's commits (F-S29a-4):** step C1 → C1; step C2 → C2; step C4 and step C5 → C3 + C4 (C3 the
non-town declarations, C4 the towns and the whole re-baseline, one commit); step C3 → C5; C6 is the integration
checkpoint the step's row names; C0 and C7 are the ledger's.

### 9.0 The procedure for every commit (read it before each commit)

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s29-a`. Never `cd` elsewhere; use absolute
paths.

1. Re-read this commit's section and every design section it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered implementation steps, in order. Do no work that belongs to a later commit.
3. Run the commit's validation commands, one at a time. Record for each: the command, the test count, the wall
   time, and PASS / FAIL / INCONCLUSIVE (a test not run is not a pass; an exit code alone is not a pass — read
   the output).
4. For each planted mutation the commit lists: copy the file to a backup with `cp`, plant the mutation, run the
   named test, record that it turned red with the failing test's name and the refusal's code, restore with `cp`
   from the backup, check `cmp` reports no difference, run the test again green.
5. Static checks: `cargo fmt --all --check`; `cargo clippy -p <each crate you touched> --all-targets -- -D
   warnings`. Markdown: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
   **R-LOAD-1:** never `cargo test --workspace`, never a whole member's suite; one `--test <file>` at a time.
6. Review: answer each review question of the commit in §15, one or two sentences each.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**. Otherwise
   revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) and `handoff-s29a.md` in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the message given plus the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`.
10. Stop and report (a material stop, §12) instead of improvising.

#### C0 — Base, prerequisites and ledger (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s29a.md` (new).
- Steps:
  1. Record the base commit (`git rev-parse origin/main`). Record from the primary's dispatch that Milestone F's
     checklist was handed over (quote it). Missing → stop.
  2. `gh pr view 171 --json state,mergeCommit` must show `MERGED` (12d). Not merged → stop. Record
     `gh pr view 206 --json state` (S26-a): C4 waits for it.
  3. Re-read A-1 … A-23 on the base; record every line number that moved (bounded) and every statement that
     changed (stop). In particular: the two towns' street `body.floor` and the two west parapet solids (A-9); the
     place-list grep of A-19; `grep -rn '"validate"' tools/cli/tests/` (A-11); the `routines.rs` and
     `bodies/mod.rs` helpers (A-21, A-22).
  4. Confirm §14's rulings are recorded in §14.1 (the primary writes them). Absent → stop.
  5. Write the handoff (the contract's fields; the sequence C0 … C7).
- [ ] Implementation: steps 1–5. [ ] Validation: the two doc scripts PASS. [ ] Review: every start condition
  holds, with its evidence quoted.
- **Commit message:** `docs(plan): S29-a — open the ledger at the base`

#### C1 — The specification, first (Markdown only)

- **Files:** `docs/MODULE_SPEC.md`, `docs/CORE_CONCEPTS.md`, `docs/VISUAL_SLICE.md`, `docs/DECISIONS.md`, this
  file (§15).
- Steps:
  1. `MODULE_SPEC.md` §4.1, in the ```` ```yaml ```` block `# world.yaml` (`MODULE_SPEC.md:486-543` at the
     audit base): after `decisions:`, add the `settlement:` lines of §5.1 with these comments: `# required by
     mineworld validate and packs validate (ARC-105); validated by the loader whenever present`, `# town | none`,
     `# kind town: a key of places, the station hall`, `# kind town: a key of places; may equal station`.
  2. `MODULE_SPEC.md`: a new `## 4.3 Towns: station and landmark` after §4.2 and before `# 5.`. Its content:
     (a) what a town is (step-29 §3's `town`, `station`, `station hall`, `stop`, `ticket office`, `landmark`,
     `platform anchor`, word for word, with §14's rulings applied); (b) the two shapes of §5.1; (c) "optional to
     the loader; required by `mineworld validate` and `mineworld packs validate`; validated in full whenever
     present, so `run`, `server` and `validate` refuse the same town alike"; (d) the refusal table: one row per
     code of §5.3 as ruled, columns *Code*, *Refused when*, *Names*, *Checked by* (`validate` only, or the
     loader); (e) the group rule (§5.3 step 5) and the stated skips (no `movement`: the group is the tagged set,
     scattering and reachability are not checked; a seat without a location is not checked); (f) "a role tag
     grants no behaviour; behaviour comes only from Systems (`transport` attaches `StopInfo` to the stop and
     `fares` its shop to the ticket office, S21)".
  3. `CORE_CONCEPTS.md`: a new `## 6.5 Role tags` after `## 6.4 Sit point` (*rev. 4:* §6.4 was taken by "Sit
     point" after revision 2, so role tags are §6.5 everywhere; if C0 finds §6.5 taken too, the next free number
     is a bounded correction, recorded; `check_doc_headings.py` must pass), with
     step-29 §14.1's quoted text, amended for Q-S29a-1's ruling in the sentence about the platform anchor and for
     Q-S29a-10's wording of what a client may do with a role tag. *Rev. 4 (R-S29a-g-1):* the role tags stand on
     **one line** beginning `The role tags are:`, each back-quoted, because S29-g's drift test reads that line.
  4. `VISUAL_SLICE.md` §4: a dated amendment paragraph ("Amended 2026-10-15 (step-29 §14.3, `ARC-105`,
     `ARC-106`)") in §4.1's amendment style: a third enterable building is **required**, the town's station; the
     street's west termination is "the station forecourt closing the view"; and in §12 a view 11, "the station
     from the street and in the hall". The out-of-scope sentence about a third building is struck through
     with `~~…~~` and kept visible.
  5. `docs/decisions/ARC-105.md` *(rev. 4: the log is per file since #258)*: a dated note `**Note (2026-10-??,
     S29-a).**` recording §14's rulings: where the platform-anchor rule is checked (Q-S29a-1), whether a
     landmark-reachability code exists (Q-S29a-2), the `create` template's settlement (Q-S29a-3), that `mineworld
     validate` now enforces the key (F-S29a-2), that no `halt` tag exists (Q-S29a-9), and what a client may do
     with a role tag (Q-S29a-10). Then `python3 scripts/check_decision_ids.py --write-index`.
- Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`; read the four
  diffs once against step-29 §§3, 4, 14.
- [ ] Implementation: steps 1–5. [ ] Validation: both scripts PASS. [ ] Review: no term of `CLAUDE.md` §2.1(3)
  is given a synonym; every code in §4.3's table is one C2 implements; the skips are stated.
- **Commit message:** `docs(spec): towns declare a settlement — station, landmark and role tags (S29-a)`

#### C2 — The loader: `settlement:` and its checks

- **Files:** `worldpack/src/settlement.rs` (new), `worldpack/src/format.rs`, `worldpack/src/error.rs`,
  `worldpack/src/read.rs`, `worldpack/src/lib.rs`, `worldpack/tests/settlement.rs` (new), this file (§15).
- Steps:
  1. `settlement.rs`: module doc (what a town is, the order of §5.3, why the loader owns it and no System does);
     `SettlementKind`, `Settlement`, `SettlementCode` with `ALL` and `as_str()` and `Display`; the hand-written
     `Deserialize` of §5.1 (a `Visitor` with `visit_map`, every message prefixed `SETTLEMENT_MALFORMED: `);
     `pub(crate) fn check(manifest_path: &Path, root: &Path, settlement: &Settlement, places: &BTreeMap<EntityKey,
     AuthoredPlace>, people: &BTreeMap<EntityKey, AuthoredPerson>, seats: &BTreeSet<EntityKey>, systems:
     &[Capability]) -> Result<(), PackError>` doing §5.3 steps 1–9 (and 10 if ruled) in order; private helpers
     `adjacency(places) -> BTreeMap<&EntityKey, BTreeSet<&EntityKey>>` (undirected, from every place's
     `passages`), `reach(from, adjacency, admit: impl Fn(&EntityKey) -> bool) -> BTreeSet<&EntityKey>` (BFS with
     a `VecDeque`, visiting neighbours in set order), and `tag(text) -> Tag` for the five role tags plus `public`.
     Place file paths come from `read::content_path` (make it `pub(crate)`).
  2. `format.rs`: `WorldManifest` gains `/// … (ARC-105, MODULE_SPEC §4.3). Optional to the loader; required by
     validate. Never world state. #[serde(default)] pub settlement: Option<Settlement>`.
  3. `error.rs`: the variant of §5.2 with doc comments on every field; `use crate::settlement::SettlementCode`.
  4. `read.rs`: the module doc's order gains `9b  a town's settlement: its station hall and landmark exist and
     are tagged, its station group has a stop and a ticket office and is one group, and every seat can walk to
     it (ARC-105)`; in `read_with` after `check_passages(...)?;` (A-3) call `if let Some(settlement) =
     &manifest.settlement { settlement::check(&manifest_path, &root, settlement, &places, &people, &seats,
     &systems)?; }`; the field `settlement` in `WorldPack` and in `in_memory`; `settlement()` and
     `require_settlement()` of §5.3.
  5. `lib.rs`: `pub mod settlement;` and `pub use settlement::{Settlement, SettlementCode, SettlementKind};`.
  6. `tests/settlement.rs`: a `Fixture` like `worldpack/tests/refusals.rs`'s (a scratch directory named after the
     pack id with `places/` and `people/`), a builder `town()` writing a minimal valid town (systems presence,
     movement; places `street`, `station`, `station-tickets`, `station-platform` with §5.5's tags and passages,
     no bodies; one seat `ada` located in `street`; `settlement: town station/station`) and one test per row:
     - `a_town_with_a_station_group_reads` (the builder as is);
     - `kind_none_reads` and `a_world_without_a_settlement_still_loads` (`read` then `load(WorldTime::EPOCH)`);
     - one test per malformation of §5.1's table, asserting `PackError::Malformed`, `detail` containing
       `SETTLEMENT_MALFORMED` and a line number;
     - one test per code of §5.3: `TOWN_STATION_UNKNOWN` (station: `depot`), `TOWN_LANDMARK_UNKNOWN`,
       `TOWN_STATION_UNTAGGED` (hall without `public`), `TOWN_LANDMARK_UNTAGGED` (landmark `street`),
       `TOWN_STATION_NO_STOP` (platform without `stop`), `TOWN_STATION_NO_TICKET_OFFICE`,
       `TOWN_STATION_SCATTERED` (`street` tagged `station` but the platform's passage moved to `street`),
       `TOWN_STATION_UNREACHABLE` (no `station`↔`street` passage), and the ruled extra code(s); each asserts
       `code`, the `path`'s file name and `key`;
     - `without_movement_the_group_is_the_tagged_set` (systems presence only, no passages: reads; with one tag
       removed: `TOWN_STATION_NO_STOP`);
     - `a_seat_without_a_location_is_not_checked`;
     - `every_code_is_reached` (a structural guard: the set of codes the tests above produce equals
       `SettlementCode::ALL` minus `Undeclared`, which C5's CLI test covers).
- Validation: `cargo test -p mineworld-worldpack --test settlement`; `cargo test -p mineworld-worldpack --test
  refusals` and `--test social_cafe` (unchanged, still green); `cargo clippy -p mineworld-worldpack --all-targets
  -- -D warnings`. Mutations **M-a-1** (step 8's check removed) → `scattered…` red; **M-a-2** (`reach` admits
  every place, not only `station`-tagged ones) → the scattered test red, because the street joins the group;
  **M-a-3** (the adjacency built one way only) → `a_town_with_a_station_group_reads` red with
  `TOWN_STATION_UNREACHABLE` (the hall states the street passage, not the street).
- [ ] Implementation: steps 1–5. [ ] Validation: step 6, M-a-1…3. [ ] Review: the checks never read a `body:` or
  any section; `BTreeMap`/`BTreeSet` only (no `HashMap`, `clippy.toml`); every refusal names a file; the order is
  §5.3's.
- **Commit message:** `feat(worldpack): towns declare a settlement, refused by name when the station is missing (S29-a)`

#### C3 — Non-town worlds say so

- **Files:** `worlds/bodies-yard/world.yaml`, `tools/cli/templates/new-world/world.yaml`, each test file of A-11
  that writes a fixture `world.yaml` (re-listed at C0), this file (§15).
- Steps:
  1. `bodies-yard/world.yaml`: after `seats:`, the comment "# Not a settlement: a yard for testing bodies
     (docs/MODULE_SPEC.md §4.3)." and `settlement:` / `  kind: none`.
  2. The template, as Q-S29a-3 ruled (`kind: none` with the comment "# Not yet a town. A town names
     its station hall and its landmark: docs/MODULE_SPEC.md §4.3.").
  3. In each test that writes a `world.yaml` into scratch and then runs `validate` or `packs validate` on it, add
     `settlement: { kind: none }` to the written text (a literal edit; no claim changes). A test that copies a
     reference world is left alone.
- Validation: `cargo test -p mineworld-cli --test <file>` for each touched file, one at a time, and `--test
  create` and `--test bodies_yard`; each PASS with the same test count as on the base.
- [ ] Implementation: steps 1–3. [ ] Validation: every touched test file. [ ] Review: no fixture claim changed;
  `bodies-yard`'s ids and genesis are unchanged (a key in `world.yaml` allocates nothing).
- **Commit message:** `chore(worlds): worlds that are not towns say so (S29-a)`

#### C4 — The three towns' stations, and the one recorded re-baseline

- **Precondition:** S26-a (#206) merged and this branch rebased on it. Otherwise stop and report.
- **Files:** `worlds/{social-cafe,market-town}/{world.yaml, places/station.yaml, places/station-tickets.yaml,
  places/station-platform.yaml, places/street.yaml, people/ivan.yaml, people/erin.yaml, people/bob.yaml}`;
  `worlds/lakeside/{world.yaml, places/station.yaml, places/station-tickets.yaml, places/station-platform.yaml,
  places/pier.yaml, people/celia.yaml, people/greta.yaml, people/hugo.yaml}`; every file of A-19 that moves;
  `scripts/baselines.txt`; this file (§15).
- Steps:
  1. The nine place files of §5.5, with every solid written out.
  2. The two streets' floor and parapets (§5.5).
  3. `world.yaml` × 3: the place keys and the `settlement:` block (§5.5).
  4. `pier.yaml`: the `landmark` tag.
  5. The routines of §5.6.
  6. `cargo build -p mineworld-cli` then `target/debug/mineworld validate worlds/<w> --packs entities --packs
     presentation/mineworld-default` for each of the four worlds: each reads (the gate is not yet required; the
     settlement is checked because it is present).
  7. The in-tree re-baseline: run each test file of A-19 alone; for each failure, confirm that the difference is
     only (a) the three new keys in a list, (b) ids shifted by the new places (A-17), (c) the six people's
     routines and their consequences in a recorded run. Re-record the golden by the method its file states
     (the `decisions` golden: the base binary's `run … --seed 7 --days 30` output, `without_wall`, from **this**
     head; `social_cafe.rs`: the id list by A-17's arithmetic; the agent JSON fixtures: re-captured by the method
     their test module documents). A difference of another kind is a material stop.
  8. `cargo build --release -p mineworld-cli`; then once: `python3 scripts/ci_parity.py record --binary
     target/release/mineworld --profile long --out <scratch>/long-record.txt` (bounded: ≤ 25 min; record the
     wall time as INCONCLUSIVE for timing, R-LOAD-1). If it exceeds the bound, stop the run, record it, and use
     the fallback of step 10.
  9. `python3 scripts/ci_parity.py baseline write <scratch>/long-record.txt scripts/baselines.txt`; then add to
     each moved world's section one line `reason S29-a (Q-S29-7, ARC-105): the station group (three places), its
     passages and bodies, and three routines added; place and person ids from <first moved key> on shifted by +3;
     <columns> moved, <columns> did not` with the columns read from `git diff scripts/baselines.txt`.
     `bodies-yard`'s section must not move; if it does, material stop.
  10. *Fallback (only if step 8's bound is exceeded):* commit steps 1–7 as C4, push, dispatch `nightly.yml` on the
      branch (`gh workflow run nightly.yml --ref mvp0/pr-s29a`), download the Linux long record, and commit step 9
      as C4b. Record the deviation from "one commit" in §15.
- Validation: step 6; step 7's files, each PASS after re-recording; `cargo test -p mineworld-cli --test routines`
  (AC-a-11), `--test run_restart`, `--test market_town`; `python3 scripts/ci_parity.py baseline check
  <scratch>/long-record.txt scripts/baselines.txt` PASS.
- [ ] Implementation: steps 1–5, 7, 9. [ ] Validation: steps 6, 8 and the files. [ ] Review: each moved golden's
  difference is one of 7(a)–(c), stated per file in §15; no summary moved that the routines do not explain; every
  dimension's comment cites its source or says *design*.
- **Commit message:** `feat(worlds): every town gets its station — hall, ticket office, platform — and the one re-baseline (S29-a)`

#### C5 — The authoring gate

- **Files:** `tools/cli/src/main.rs`, `tools/cli/src/packs.rs`, `tools/cli/tests/packs.rs`, this file (§15).
- Steps:
  1. `main.rs` `validate`: §5.4's call and the printed line.
  2. `packs.rs` `world_identity`: §5.4's call.
  3. `tests/packs.rs`: `every_reference_world_validates` (each `worlds/*` through `mineworld validate` with the two
     roots; asserts success and the `settlement` line: `town (station station, landmark station)` for the two
     towns, `town (station station, landmark pier)` for Lakeside, `none` for `bodies-yard`);
     `a_world_without_a_settlement_is_refused_at_the_gate` (X-1: a copy of `market-town` in scratch with the
     `settlement:` block removed; `validate` and `packs validate` both fail and print `SETTLEMENT_UNDECLARED` and
     `world.yaml`; `run <copy> --headless --seed 1 --days 1` succeeds).
- Validation: `cargo test -p mineworld-cli --test packs`; `--test create`; `--test commands`; `--test bodies_yard`.
  Mutation **M-a-4** (the `validate` call removed) → `a_world_without_a_settlement_is_refused_at_the_gate` red.
- [ ] Implementation: steps 1–2. [ ] Validation: step 3, M-a-4. [ ] Review: `run`, `server`, `create` do not call
  the gate (I-a-3); the printed line is after `seats` and changes no other line of `validate`'s output.
- **Commit message:** `feat(cli): validate requires every world to say whether it is a town (S29-a)`

#### C6 — `station_visit`, the mutations, and the walks

- **Files:** `tools/cli/tests/station_visit.rs` (new), `tools/cli/Cargo.toml` (only for step 7's dev-dependency,
  if absent), this file (§15).
- Steps:
  1. `market_town_people_meet_at_the_station`: `mineworld run worlds/market-town --headless --seed 7 --days 30
     --save <scratch>`; read the save (`headless::Tables`, A-21); per day count distinct people with a
     `person-entered-place { station }` (≥ 3) and the `spoke`/group-activity facts whose speaker's place, from
     the replayed presence facts at that instant, is `station` (≥ 1). Print the per-day counts before asserting.
  2. `the_station_resumes_identically`: the same run saved at day 15, then resumed to day 30 (the
     `run_restart.rs` pattern); the final save's digest (`ci_parity.py`'s `save_digests` semantics, or the
     helper `run_restart.rs` uses) equals the uninterrupted run's.
  3. `social_cafe_and_lakeside_people_meet_at_the_station`: step 1's arrivals and meeting on both.
  4. `nothing_is_sold_at_the_ticket_window`: AC-a-9.
  5. `the_station_walks_are_not_stopped_by_geometry`: AC-a-10, with `bodies/mod.rs`' stopped-short counter on
     the walks whose destination is in the station group (market-town and social-cafe).
  6. `a_broken_town_is_refused_by_run_as_by_validate`: X-2, X-3 and the scattered tag, each planted in a scratch
     copy; both `validate` and `run` fail with the code and the key.
  7. *(Q-S29a-1, ruled.)* `every_platform_anchor_lies_on_one_edge`: for `market-town` and
     `social-cafe`, `WorldPack::read_with` + `load(WorldTime::EPOCH)`, find the genesis `place-shaped` fact whose
     place is `station-platform`, decode it as `mineworld_bodies::PlaceShaped`, and assert that (0, 0) lies on
     exactly one edge of its floor. Add `mineworld-bodies` as a dev-dependency of `mineworld-cli` only if it is
     not one already (check `tools/cli/Cargo.toml`; a new dev-dependency on a workspace crate is not a new
     external dependency).
  8. *(rev. 4, Q-S29a-11.)* `the_station_places_lay_out_without_overlap`: for **every** `worlds/*` that declares
     `kind: town` and enables `bodies` (today `market-town` and `social-cafe`; the list is found, not written), read
     every genesis `place-shaped` fact as in step 7, and lay the station group and the places it joins out by the
     doorway rule (A-27), written once as a helper over the stored facts and the passages: starting from the
     hall at (0, 0), each neighbour's origin is `there − here`, moved 800 mm across the shared wall, the wall being
     the edge of the neighbour's floor that `here` stands 400 mm inside. Assert (a) each passage's `here` is
     400 mm inside exactly one edge of its floor and `there` 400 mm inside the opposite-facing edge of the other's;
     (b) the laid-out floors are pairwise disjoint (touching edges allowed). Print each place's laid-out rectangle
     before asserting. A second world is a generated variant: the same test on a scratch copy of `market-town`
     whose hall joins the street 2 m further north (the street-side `there` moved by (0, 2 000)) still passes,
     so the check reads the frames, not these numbers. Mutation
     **M-a-5:** the towns' bay `here` set back to revision 3's (3 000, 400) → (a) red, naming `station-tickets`;
     **M-a-6:** the platform's floor set back to revision 3's east–west rectangle → (b) red, naming
     `station-platform` and `station`.
- Validation: `cargo test --release -p mineworld-cli --test station_visit` (release: the 30-day runs; record wall
  time as INCONCLUSIVE). If a run-based test needs `--ignored` by the file's own convention (as `rhythm.rs`),
  CI must still run it: confirm it is in a CI layer's command list (`python3 scripts/ci_layer.py --list core`)
  and record where.
- [ ] Implementation: steps 1–8. [ ] Validation: the file; the planted copies and M-a-5, M-a-6 are the mutations. [ ] Review: the
  oracle is the stored facts; no threshold was tuned to the measured value (record the measured counts beside
  the thresholds).
- **Commit message:** `test(cli): people meet at the station, and a broken town is refused alike (S29-a)`

#### C7 — Close-out

- **Files:** this file (§15), `handoff-s29a.md`.
- Steps: AC-a-13's diff scan (`git diff --name-only origin/main...HEAD`); fill §8's evidence; push
  `mvp0/pr-s29a`; `gh pr create` (body ending with the generated-with line); CI on the exact head (Linux, macOS,
  Windows) green, repair to green; dispatch `nightly.yml` once on the final head (justified: the PR's CI does not
  run the long profile, test rules §10 item 3) and record `baseline check`'s verdict (AC-a-12).
- [ ] Implementation. [ ] Validation: CI green on the exact head; nightly `baseline check` PASS. [ ] Review:
  deviations recorded; §11's parent edits listed for the primary.
- **Commit message:** `docs(plan): S29-a close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S29b-a-1 (S29-b).** The newsstand's seller is chosen among the people still jobless after C4 (`bob`,
  `carol`, `dev`, `erin`, `grace`, `hana`, `ivan`, `otto`, `visitor`, `wanderer` minus anyone S29-b's own audit
  excludes). `station-news` sorts after `corner-store` and organizations are allocated last, so it moves no place,
  person or item id. Its shop, its job and its sales do move `market-town`'s facts and summaries: that is a
  behaviour change, recorded by S29-b with its own `reason S29-b:` line, as `scripts/baselines.txt`'s header
  requires of every behaviour-changing PR. Q-S29-7's "one re-baseline" is read here as the one **id shift**; the
  step's wording ("the station's places, the organization and the routines") also names the organization, so
  §11 asks the primary to state that reading in step-29 (F-S29a-7).
- **R-S29c-a-1 (S29-c).** `FAR_FIELD_SLOT_TOWN` reads `WorldPack::settlement()` (§5.3) — the station and the
  landmark keys — never the world files a second time.
- **R-S29d-a-1 (S29-d).** The forecourt's 3D geometry matches C4's street floor (min.x −62 450) and the hall's
  passage point; S29-d re-measures the hall's height and the towers and may move a *design* value only with a
  recorded reason. *Rev. 4:* the bay and platform frames are S29-d's R-S29a-d-1, carried in §5.5.
- **R-S29f-a-1 (S29-f, rev. 4).** The platform's 2D interior is 60 × 11 m with the door mid-length on the long
  side, and the ten arcade piers are drawn where §5.5 puts them (a solid with no drawing reads as an invisible
  wall).
- **R-S33b-a-1 (S33-b, rev. 4).** Lakeside's bay bodies follow §5.5's rev. 4 frame: the door near its south edge,
  the counter on its north wall.
- **R-RTc-a-1 (S21 RT-c; Q-S29a-1, ruled).** `transport` validates that the stop's platform
  anchor lies on exactly one edge of the stop's `body.floor` when it attaches `StopInfo`, refusing by name.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-29-station-landmark.md` §4.2, §9.5, §11.1 | the platform-anchor row moved (Q-S29a-1) and the `TOWN_LANDMARK_UNREACHABLE` row (Q-S29a-2), **applied in PR #234**; the S29-a row's commit mapping (F-S29a-4) |
| `step-29-station-landmark.md` §2.1 | A-18 corrected: `mineworld validate` did not enforce `ARC-53`; the gate is new for it (F-S29a-2). **Recorded in step-29 §14.8, PR #234** |
| `step-29-station-landmark.md` §13.1 (Q-S29-7) | the reading of F-S29a-7: the one re-baseline is the id shift; S29-b records its sales as a behaviour change. **Recorded in step-29 §14.8, PR #234** |
| `overall.md` | S29 in §3 (step-29 §14.7); the second, recorded re-baseline beside 12d's "Digests" rule; S29-a frozen / merged |
| `step-29-station-landmark.md` §4.3, §14.9 *(rev. 3)* | the counter bay "about 6 000 × 3 000 mm" → at least 4.6 × 4.0 m, sized by the capacity rule; Lakeside's halt at the lane's east end with a north–south platform ending against the shore (Q-S33a-1). **Applied in the rev. 3 PR** with dated notes |
| `step-29-station-landmark.md` §4.3, §8.1, §14.10 *(rev. 4)* | the towns' bay south of the hall and the platform north–south, 11 × 60 m with the arcade (Q-S29a-8); Lakeside's bay north of the waiting room (Q-S33e-3); no `halt` tag (Q-S29a-9); role tags are `CORE_CONCEPTS` §6.5. **Applied in the rev. 4 PR** with dated notes |
| `pr-s33b-lakeside-bodies.md` §5.3 halt rows *(rev. 3, for the S33-b implementer at C0)* | the bay's floor 4 600 × 4 600 if Q-S29a-6 is ruled as recommended (F-S29a-9: a 600 mm counter in a 4 600 × 4 000 bay leaves 35 < 37 points); the platform's frame x −4 000 … 0, y −20 000 … 20 000 (Q-S29a-7). No edit to the frozen file: S33-b's C0 records what S29-a merged (its R-b-5) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S29-a — the town contract (settlement:, station group, loader checks,
                         the three towns' stations, one re-baseline)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s29a-town-contract.md
RELATED / BINDING DOCS:  step-29-station-landmark.md (§§1–5, 7.1, 9.3, 9.5, 10, 11, 13, 14); overall.md;
                         docs/MODULE_SPEC.md §4.1; docs/CORE_CONCEPTS.md §6; docs/VISUAL_SLICE.md §4;
                         docs/DECISIONS.md ARC-31, ARC-36, ARC-53, ARC-82, ARC-105; .structured-coding/standards.md;
                         CLAUDE.md; docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s29-a, its own, held by one session
BRANCH:                  mvp0/pr-s29a, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator AND 12d (#171) merged AND this design
                         frozen, then the primary session's dispatch. C4 also needs S26-a (#206) merged
                                                                     (operator 2026-10-15; step-29 §11.2, Q-S29-7)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–6, as §14's rulings shape them
FROZEN INVARIANTS:       step-29 I-S29-1, I-S29-2, I-S29-4, I-S29-6; §4 I-a-1 … I-a-9; §8 as written
SEQUENCE:                C0 … C7 (§9)
IMPLEMENTER:             Haiku-class, from this plan (step-29 §11.1); reviewer: the primary session, with an
                         Opus logic review of C2 and C4
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace); target/*/mineworld *;
                         python3 scripts/check_doc_headings.py, check_decision_ids.py, ci_parity.py
                         (record --profile long once in C4, baseline write/check), ci_layer.py --list;
                         git; gh (pr create/view, workflow run nightly.yml once in C7 and in C4's fallback, run
                         download; never merge); cp; cmp; mkdir -p; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or a
                         member's whole suite locally (R-LOAD-1); reading any key or .env; any model call;
                         edits outside APPROVED SCOPE; a golden re-recorded for a difference C4 step 7 does not list
MATERIAL STOPS:          a start condition missing; a needed change under kernel/, contracts/, systems/, server/src/,
                         clients/, cognition/ or sdk/; a re-baseline difference of a kind C4 step 7 does not list;
                         bodies-yard's baseline moving; AC-a-7, AC-a-8 or AC-a-11 failing after one bounded content
                         correction (segment length or predecessor place, recorded); a new dependency
PLATFORMS:               Linux, macOS, Windows (CI's test jobs on all three)
VALIDATION BUDGET:       crate and CLI test files unrestricted one at a time; the C4 long record once, ≤ 25 min;
                         station_visit's runs ≤ 10 min on Linux CI; one nightly dispatch (C7). Gate 1 and Gate 2:
                         NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s29a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                                     (task brief 2026-10-15: "endpoint: PR ready
                                                                      for operator review; no merge")
                         semantic commits: authorized                (working rules §14)
                         branch push: authorized                     (working rules §21)
                         PR creation / update: authorized            (working rules §21)
                         CI repair to review readiness: authorized   (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — only with the operator's explicit authorization (working rules
                         §22)
POST-MERGE SYNC OWNER:   primary: step-29, overall.md; implementation session: this ledger and the merge identity
STOP CONDITION:          S29-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-a-1 | The station is far: visitors arrive late and NW-1's 90 % falls | ≥ 5 h segments (A-15); C6 prints arrival times; one bounded content correction, then a material stop |
| R-a-2 | The re-baseline collides with in-flight PRs that touch the same worlds (S29-b, S30-g, S31) | C4 is one commit, rebased last; the primary schedules it in a quiet window (step-29 R-S29-2) |
| R-a-3 | A golden moves for a reason other than ids and routines | C4 step 7's classification is recorded per file; any other kind is a material stop |
| R-a-4 | The local long record takes too long under load | ≤ 25 min bound; the nightly fallback (C4 step 10), recorded |
| R-a-5 | The 3D slice shows Social Café people walking off the street's west end until S29-d | Q-S29a-4; S29-d is on the critical path |
| R-a-6 | The 2D client lays out three new places with no station art until S29-f; RL-e0's 2D far-field check runs on a larger hub | the 2D client draws places from passages generically; S29-f re-runs the check (step-29 §8.2); recorded as an expected visual change |
| R-a-7 | A Haiku implementer misreads a re-baseline difference as expected | the Opus logic review of C4 checks every §15 row of step 7 |
| R-a-9 *(rev. 4)* | A station place's frame faces the wrong way and no one sees it until a client draws it | C6 step 8 (AC-a-14) checks every town with bodies by the doorway rule; S33-b's AC-b-6 covers Lakeside |
| R-a-8 *(rev. 3)* | A station place passes capacity with no margin, and a later PR that adds a resident fails every town's genesis | the bays are sized with a margin (54 for 45, 42 for 37; Q-S29a-5, Q-S29a-6); C4 step 6's `validate` of each town is the first check; `bodies-capacity` names the place |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

Q-S29a-5 … Q-S29a-7 were ruled by the operator on 2026-10-18 in the primary session, each as recommended (town
ticket bay 6.0 × 4.6 m, Lakeside bay 4.6 × 4.6 m, track on the east edge).

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29a-1 [operator — step §4.2's refusal table, `ARC-105` item 3]** | `TOWN_PLATFORM_ANCHOR` reads the stop's `body.floor`. `body:` is the `bodies` pack's section and opaque to the loader (`ARC-31`), so the loader can check it only by decoding `bodies`' format a second time or by adding a geometry query to the authoring seam (F-S29a-1). Where should the rule be enforced? | **Move it to its consumer, and hold it in content now.** (a) The loader drops `TOWN_PLATFORM_ANCHOR`; the rule stays in `CORE_CONCEPTS` §6.4 as the meaning of a stop's origin. (b) S29-a's C6 adds a content test, `every_platform_anchor_lies_on_one_edge`, that loads each reference town and reads the `place-shaped` genesis fact `bodies` states for `station-platform` (its public event type, `mineworld_bodies::PlaceShaped`, `systems/bodies/src/event.rs:21`; the section module itself is private), so the test reads the owner's own decoding of the floor (a test of the content, not a loader rule). (c) S21's RT-c, which owns `StopInfo` and its door anchors along that edge, refuses a stop whose anchor is off-edge when it attaches `StopInfo` (R-RTc-a-1). Alternatives rejected: a second decoder of `bodies`' format in the loader (two authorities for one format); a geometry method on `AuthoredContent` (a domain word in the seam, `CLAUDE.md` §4 rule 2); a check inside `bodies` keyed on the `stop` tag (a System reading a role tag, against I-S29-1) |
| **Q-S29a-2 [operator — I-S29-4 vs step §4.2]** | I-S29-4 says the **landmark** is reachable on foot from every seat's starting place, but §4.2's table has only `TOWN_STATION_UNREACHABLE`. In Lakeside the landmark is `pier`, which no code would check (F-S29a-3). Add `TOWN_LANDMARK_UNREACHABLE`? | **Yes.** One more code, the same search (§5.3 step 10), checked only when the landmark is not the hall (otherwise step 9 already covers it). It costs a dozen lines and one test, and it makes I-S29-4 a load-time fact as the invariant says |
| **Q-S29a-3 [operator — the `create` template]** | `mineworld create` writes a one-place world that `mineworld validate` checks at once (A-12). It now needs a `settlement:`. Declare `kind: none`, or make the template a minimal town (three more places: hall, ticket office, platform)? | **`kind: none`, with a comment pointing to MODULE_SPEC §4.3.** A one-room starter world with a station would add three places the author did not ask for and would make the guide's first world a station rather than a home. The comment makes becoming a town one step away. (The guide, S25, shows the town contract in its own chapter) |
| **Q-S29a-4 [operator — interim 3D picture]** | After C4, Social Café people walk to `station` on the server; the 3D slice cannot draw it until S29-d and shows them leaving the street's west end (F-S29a-6). Land Social Café's station routines in S29-a, or hold them for S29-d? | **Land them in S29-a.** The station group, its checks and `station_visit` need them (AC-S29-2 covers all three towns), and holding them would need a second re-baseline of Social Café in S29-d, against Q-S29-7's "one". S29 starts after Milestone F's checklist and ships before the release (step-29 §11.2), so no milestone the operator walks shows the gap; S29-d is the critical path |

### 14.1 Rulings, 2026-10-15 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| Q-S29a-1 | **The platform-anchor rule moves to its consumer.** The loader drops `TOWN_PLATFORM_ANCHOR`; C6 step 7 adds the content test; RT-c refuses an off-edge anchor (R-RTc-a-1). Applied: §5.1, §5.5, §9 C1/C6, §10; step-29 §§4.2, 14.6, 14.8 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S29a-2 | **Add `TOWN_LANDMARK_UNREACHABLE`.** Applied: §5.1, §5.3 step 10, I-a-5, C2's table test; step-29 §4.2, I-S29-4, §14.8 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S29a-3 | **The `create` template declares `kind: none`, with a comment pointing to MODULE_SPEC §4.3.** Applied: C3 step 2 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S29a-4 | **Social Café's station routines land in S29-a.** Applied: §5.6, C4 | Ruled 2026-10-15 (operator answered directly in the primary session) |

Revision 2: no question remained open. Revision 3 opens Q-S29a-5 … Q-S29a-7 (§14.3), the details the S33
rulings leave. The design stays DRAFT until they are ruled, 12d (#171) has merged and Milestone F's checklist has
been handed to the operator; then the primary writes the freeze header.

### 14.2 The S33 rulings applied in revision 3 (operator, 2026-10-18; binding)

| Ruling | Source | Change in this file |
| --- | --- | --- |
| **Q-S33a-1:** "east end, north–south, as recommended — the small station at the lane's east end, a north–south platform with its end against the shore; S29-a's next revision carries it (R-S29a-a-1)" | [`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) §14 (frozen 2026-10-18, #255) | §5.5's Lakeside rows `station` (the lane passage, at the east edge) and `station-platform` (the north–south frame, the buffer stop at the south end, against the shore), with the note under the table; A-25; F-S29a-10 |
| **The ticket room is at least 4.6 × 4.0 m (capacity rule):** "the ticket room grows to at least 4.6 × 4.0 m (F-S33b-5), and S29 must change to carry it (§11)" | [`pr-s33b-lakeside-bodies.md`](pr-s33b-lakeside-bodies.md) header ("Approved by"), F-S33b-5, §11 (frozen 2026-10-18, #255) | §5.5's towns' `station-tickets` file (floor and counter) and Lakeside's `station-tickets` row; A-24; F-S29a-8, F-S29a-9 |

Nothing else changes: the loader contract (§5.1 … §5.4), the codes, the hall, the towns' platforms and streets,
the routines, the commit plan, the acceptance criteria and the execution contract are revision 2's. The parent
edits are listed in §11 and made in step-29 by the same revision (§14.9 there).

### 14.3 Questions opened by revision 3 (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29a-5 [operator — the towns' ticket bay, F-S29a-8]** | The towns have 12 people, so `bodies` needs 45 free lattice points in the bay; revision 2's 6 × 3 m bay has 27. At least 4.6 × 4.0 m is ruled. Which size for Social Café's and Market Town's bay: **6.0 × 4.6 m** (54 points with the counter) or **6.0 × 4.0 m** (exactly 45)? | **6.0 × 4.6 m.** It is the smallest floor that keeps a margin (room for two more residents) with a real 600 mm counter; 6.0 × 4.0 passes with no margin at all, so the next PR that adds one person to either town would fail genesis in a place it never touched (R-a-8). Both meet the ruled minimum; the width (6 m, two grilles) is unchanged |
| **Q-S29a-6 [operator — Lakeside's bay, F-S29a-9]** | A 4.6 × 4.0 m bay holds Lakeside's 10 people only without a counter: S33-b's 600 mm counter leaves 35 or 36 points of the 37 needed. Write Lakeside's bay **4.6 × 4.6 m** (42 points with the counter), or keep **4.6 × 4.0 m** and require S33-b's counter to be a screen at most 250 mm deep? | **4.6 × 4.6 m**, the same depth as the towns' bay. It satisfies S33-b's frozen "≥ 4 600 × 4 000" with no edit to S33-b, keeps a real counter, and gives one rule for every bay (4.6 m deep, the counter on the far wall). The alternative constrains a later PR's furniture to rescue a floor size |
| **Q-S29a-7 [operator — which edge the track runs along, F-S29a-10]** | Q-S33a-1's recommendation says "the track on its west edge", but the waiting room stands west of the platform, so the track would lie between them. Put the track on the platform's **east** edge (the waiting room opens onto the west edge), or keep the west edge and move the waiting room to the platform's north end? | **The east edge.** The ruling's substance — east end, north–south, the end against the shore — is unchanged; passengers step from the waiting room straight onto the platform, as at a real branch-line halt; the anchor rule (`CORE_CONCEPTS` §6.4: the origin on exactly one edge, the track edge) holds with the floor at x −4 000 … 0. Moving the waiting room to the platform's north end would take the halt 40 m away from the lane, against "at the lane's east end" |

### 14.4 Rulings on revision 3

| Question | Ruling | By |
| --- | --- | --- |
| Q-S29a-5 | **6.0 × 4.6 m** for the towns' bay | Operator, 2026-10-18 (sixth batch, item 17: "S29-a 售票间小镇 6.0×4.6、湖畔村 4.6×4.6，轨道改到东边"), as recommended |
| Q-S29a-6 | **4.6 × 4.6 m** for Lakeside's bay | the same ruling |
| Q-S29a-7 | **The track on the platform's east edge** | the same ruling |

### 14.5 What revision 4 changes, and why

| Change | Source | Where |
| --- | --- | --- |
| Lakeside's bay north of the waiting room: here (2 600, 400), there (6 000, 4 600) | Q-S33e-3, **ruled** (S33-d … j frozen as recommended, #285; "湖畔村售票间坐标修正并入 S29-a 修订", 2026-10-19) | §5.5 Lakeside table and note; F-S29a-14 |
| The towns' bay door on its north edge, the counter on its south wall | R-S29a-d-1 (S29-d F-S29d-1), **Q-S29a-8 open** | §5.5 `station-tickets`; F-S29a-11 |
| The towns' platform north–south, 11 × 60 m, ten arcade piers as solids | R-S29a-d-1 (S29-d F-S29d-2, -3), **Q-S29a-8 open** | §5.5 `station-platform`; A-28; F-S29a-12, -13 |
| No `halt` tag | S29-f Q-S29f-1, **Q-S29a-9 open** | §5.5 note; F-S29a-17 |
| I-a-7 says what it means | S29-g Q-S29g-2, **Q-S29a-10 open** | §4 I-a-7; F-S29a-16 |
| A layout test for every town with bodies | F-S29a-15, **Q-S29a-11 open** | C6 step 8; AC-a-14; M-a-5, M-a-6; R-a-9 |
| Role tags are `CORE_CONCEPTS` §6.5, on one line | §6.4 is "Sit point" on `main`; R-S29a-g-1 | C1 step 3 |
| The `ARC-105` note goes into `docs/decisions/ARC-105.md` | #258 (per-file decisions) | C1 step 5 |

Nothing else changes: the loader contract (§5.1 … §5.4), the codes, the hall, the streets, the routines, the id
arithmetic (A-17: the keys are unchanged) and the commit order are revision 3's. No change moves an id or adds a
re-baseline: every edit is inside files C4 writes for the first time.

### 14.6 Freeze questions opened by revision 4 (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29a-8 [operator — the towns' bay and platform frames; S29-d's Q-S29d-1]** | A client places a place by moving its frame, never by turning it (A-26). Revision 3's towns' bay would be drawn inside the hall, and its platform across the hall. Write the bay with its door on its north edge and the counter on its south wall, and the platform north–south, 11 × 60 m (the real 8.2 m arcade plus a 2.8 m open strip), with the ten arcade piers as solids? | **Yes.** Nothing is built yet, so it costs two place files and no re-baseline. The alternative, teaching both clients to rotate frames, changes 16c's and S33-c's frozen binding. *中文：小镇售票间门改在北边、柜台在南墙；站台改成南北向 11×60 m（含真实 8.2 m 拱廊），10 根拱廊柱写成 solids。同意吗？* |
| **Q-S29a-9 [operator — no `halt` tag; S29-f's Q-S29f-1]** | Step-29 §8.1 picks Lakeside's 2D sprite by an extra world tag `halt`. Author no such tag, and let a client draw a station that is its town's `landmark` as the depot and any other as the halt? | **Yes.** It uses only ruled tags and works for any generated town; a `halt` tag would be a picture hint in world data. *中文：不加 `halt` 标签；客户端按已有的 `landmark` 区分大车站和小站。同意吗？* |
| **Q-S29a-10 [operator — I-a-7's wording; S29-g's Q-S29g-2]** | Reword I-a-7 as "a role tag grants no behaviour; a client may choose a drawing or order lines by it", and narrow its grep to `systems/` and `cognition/`? | **Yes.** Choosing a picture or an order is not behaviour; S29-d/e/f/g all need it, and Systems still never read a role tag. *中文：I-a-7 改成"角色标签不带来行为；客户端可以按它选画法或排序"，检查只扫 systems/ 和 cognition/。同意吗？* |
| **Q-S29a-11 [operator — a layout test; F-S29a-15]** | Three wrong frames reached design review unseen. Add one content test that lays every town's station out by the doorway rule and fails on a misfacing door or overlapping floors, for every town with bodies (and a generated variant)? | **Yes.** One test, one rule, already used by S33-b for Lakeside; it catches the class in any future world. *中文：加一个通用检查：按门口规则把车站各处摆出来，门朝向错或地板重叠就失败，适用于所有带 bodies 的小镇。同意吗？* |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | this commit | [x] base, prerequisites, ledger | [x] doc scripts PASS | [x] start conditions quoted below |
| C1 | see `git log` | [x] MODULE_SPEC §4.1 key + §4.3; CORE_CONCEPTS §6.5 (free at the base); VISUAL_SLICE §4.2 + view 12 (view 11 is the far field); `ARC-105` note | [x] `check_doc_headings.py`, `check_decision_ids.py` PASS | [x] no synonym of a defined term; every code in §4.3 is one C2 implements; both skips stated |
| C2 … C7 | — | [ ] | [ ] | [ ] |

### Evidence

**C0 (2026-10-19, lane lead implementing directly: the machine-wide sub-agent limit refused a dispatch).**
- Base: `origin/main @ 5b88827f` (#336), worktree `/Users/yuema137/mineworld-worktrees/impl-s29-a`, branch
  `mvp0/pr-s29a`.
- Start conditions: the design is frozen (header, operator 2026-10-19, relayed by the coordinator: "operator
  froze all S29 designs (2026-10-19, every question as recommended) — #330–#335 merged to main. … Now implement
  as lane lead in dependency order starting with S29-a"). 12d `#171` MERGED; S26-a `#206` MERGED 2026-10-18;
  16d `#250` (Milestone F's last item) MERGED 2026-10-19.
- SPEED FIRST (charter, operator 2026-10-18) overrides §9's per-commit mutations, the Opus logic review and the
  nightly dispatch: "No independent reviewer, no mutation, no train wait, no full or targeted test run per PR."
  Each commit runs only what it needs to work (build, the test files it writes or re-records, fmt, clippy on the
  crates touched); the mutations and the nightly `baseline check` move to the milestone. Recorded as deviation
  D-a-1.
- Furnishings (`ARC-307`, `pr-vf-furnishings-from-data.md`, frozen, not implemented at the base:
  `git grep -n furnishings -- systems/bodies/src` is empty). C4 therefore writes the station's benches and kiosk
  as plain solids, as §5.5 says; follow-up F-S29a-impl-1 converts them once vf-furnishings merges.

### Deviations (implementation)

- **D-a-1.** SPEED FIRST: mutations, the C2/C4 logic review and the nightly dispatch deferred to the milestone.
- **F-S29a-impl-1.** The hall's 34 bench rows (`bench.w.<k>`, `bench.e.<k>`, k = 0 … 16, kind `bench.station`),
  the kiosk (`kiosk`, kind `kiosk.news`) and the platform's two benches (`bench.platform.s`, `bench.platform.n`,
  kind `bench.station`) become `body.furnishings` with their solids naming them once vf-furnishings merges
  (`ARC-307`); the bay counter and the arcade piers stay shell solids (vf-furnishings §3.8).
  **Resolved in this PR (2026-10-20):** vf-furnishings merged to `main` while S29-a was open; the merge of
  `origin/main` brought `body.furnishings`, and the hall and platform now write those furnishings with their
  solids naming them. The kinds `bench.station` and `kiosk.news` have no Presentation Pack binding yet, so
  both clients draw their solids as plain boxes until S29-e (3D) and S29-f (2D) bind them.
- **D-a-2.** `main`'s vf-furnishings took `CORE_CONCEPTS.md` §6.5 ("Furnishing"), so role tags are **§6.6**
  (C1 step 3's bounded rule). S29-g's drift test reads the line `The role tags are:`, not the number.
- **D-a-3.** The local build slots were saturated for over two hours (48 waiters), so the first compile check
  is CI `fast` on the PR head (R-LOAD-1: CI on the exact head is the gate).
- **D-a-4.** The 30-day `station_visit` runs (`people_meet_at_the_station`, `the_station_resumes_identically`)
  are `#[ignore]`d, following `rhythm.rs`; the milestone runs them with `--ignored`.

### Re-baseline (C4 step 7: one row per moved file, with its difference class)

(none yet)

### Mutations

(none yet; M-a-1 … M-a-6, X-1, X-2, X-3 and the scattered tag as §9 lists them)

### Deviations and findings

F-S29a-1 … F-S29a-6 (§3.2), recorded at design time.

Revision 3 (2026-10-18, design time): A-24, A-25 and F-S29a-8 … F-S29a-10 added; §5.5 changed only as §14.2's
two rulings require; Q-S29a-5 … Q-S29a-7 opened; R-a-8 added; §11 gains two rows.

Revision 4 (2026-10-19, design time): A-26 … A-28 and F-S29a-11 … F-S29a-17 added; §14.4 records the rulings on
revision 3; §14.5 lists every change; Q-S29a-8 … Q-S29a-11 opened (§14.6); I-a-7, §5.5, C1 steps 3 and 5, C6
step 8, AC-a-14, R-a-9, §10 and §11 changed as §14.5 lists.
