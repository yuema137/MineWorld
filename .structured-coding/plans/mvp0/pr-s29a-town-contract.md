# PR S29-a — The town contract: `settlement:`, the station group, the loader's checks, and the three towns' stations

## DRAFT — not frozen

```text
Design revision:        revision 1 (2026-10-15), DRAFT
Author:                 S29-a / S30-a / S30-b PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29a-s30ab, branch docs/pr-designs-s29a-s30ab,
                        planning base origin/main @ e6030a36 (#225 merged; the task named dd357ca6, and the
                        only commit between them is the S30 amendment, which this file does not touch)
Approved by / evidence: none yet. The freeze header is the primary session's to write after the operator
                        answers §14
Implementation base:    origin/main at C0; C0 records the commit and the merge state of 12d (#171) and S26-a (#206)
Start condition:        (1) Milestone F's checklist handed to the operator (step-29 §11.2, operator 2026-10-15);
                        (2) 12d merged (#171: OPEN at e6030a36); (3) this design frozen; (4) the primary
                        session's dispatch. C4 additionally waits for S26-a (#206: OPEN at e6030a36), so the
                        re-baseline happens once (Q-S29-7 as ruled). NONE of (1)–(3) holds today
Execution contract:     §12
Implementer:            Haiku-class (step-29 §11.1, the S29-a row: "Haiku, from a commit-level plan"); reviewer
                        the primary session, plus an Opus logic review of C2 and C4
Lifecycle:              DRAFT
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
   `docs/CORE_CONCEPTS.md` §6.4 "Role tags" (the text of step-29 §14.1, as §14's rulings shape it);
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
| I-a-5 (I-S29-4) | Where `movement` is enabled, every seat with a starting place reaches the hall by passages (and the landmark, if Q-S29a-2 is ruled yes) | `TOWN_STATION_UNREACHABLE`; X-3 |
| I-a-6 (I-S29-6) | Every station dimension is a real or a stated design value with its source in the place file's comment; the one compression (the platform's length) is stated | review of C4's files |
| I-a-7 | A role tag grants no behaviour: no System, client or cognition code branches on `station`, `stop`, `platform`, `ticket-office` or `landmark` | I-a-1's diff scan; `git grep -n '"ticket-office"\|"landmark"' -- systems clients cognition` empty |
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
    // per Q-S29a-1 / Q-S29a-2: PlatformAnchor and/or LandmarkUnreachable (§14)
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
10. *(Per Q-S29a-2.)* The same search reaches the landmark, else `TOWN_LANDMARK_UNREACHABLE`.

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

```yaml
# The ticket window: a counter bay off the south end of the hall, 6 m by 3 m (design). Shut: no shop is
# declared here until S21's `fares` composes one (step-29 §5.2), so nothing is sold and nothing pretends to be.
tags: [station, ticket-office, public]
note: A timber counter screen with two grilles. The shutters are down.
passages:
  - to: station
    here: { x: 3000, y: 400 }
    there: { x: 8400, y: 400 }
body:
  floor: { min: { x: 0, y: 0 }, max: { x: 6000, y: 3000 } }
  solids:
    - { min: { x: 0, y: 2400 }, max: { x: 6000, y: 3000 }, height: 1050 }   # the counter, 1.05 m (VISUAL_SLICE §3)
```

**`station-platform`** (S21's stop)

```yaml
# The platform: S21's stop for this town (a Place a line serves). 60 m by 6 m (the real platforms are about
# 198 m; compressed to the street's scale, step-29 §6.2). Its origin is the platform anchor: the track edge,
# mid-length (docs/CORE_CONCEPTS.md §6.4); y = 0 is the edge, the track lies at y < 0, the hall at y = 6 000.
tags: [station, stop, platform, public, outdoor]
note: An open platform beside two tracks, with a fence at each end. No train has come yet.
passages:
  - to: station
    here: { x: 0, y: 5600 }
    there: { x: 400, y: 25900 }
body:
  floor: { min: { x: -30000, y: 0 }, max: { x: 30000, y: 6000 } }
  solids:
    - { min: { x: -10000, y: 4800 }, max: { x: -8100, y: 5400 }, height: 450 }    # a bench
    - { min: { x: 8100, y: 4800 }, max: { x: 10000, y: 5400 }, height: 450 }      # a bench
```

The platform's origin (0, 0) lies on its floor's south edge and on no other edge (the anchor rule, whatever
Q-S29a-1 decides about where it is checked).

**The street** (`social-cafe`, `market-town`). `body.floor.min.x` moves from `-37450` to `-62450` (the forecourt,
25 m, step-29 §6.3); the two "west parapets" solids are deleted, and the comment says why ("the street opens into
the station forecourt; step-29 §6.3"). The hall's `there` point (−62 050, −4 900) is 0.4 m inside the new west
edge, on the street's axis (midway between the façades at y 3 200 and −13 000). C0 confirms both towns' street
floors and parapets against 12d as merged; a different value is a bounded correction recorded in §15.

**Lakeside's halt** (`lakeside`: no `bodies`, positions semantic, A-13):

| Key | Tags | Passages | Comment's size |
| --- | --- | --- | --- |
| `station` | `station`, `public` | `lane`: here (4000, 400), there (400, 3000) | waiting room ≈ 8 × 5 m (design), at the lane's west end |
| `station-tickets` | `station`, `ticket-office`, `public` | `station`: here (1500, 400), there (6000, 400) | a window bay ≈ 3 × 2 m |
| `station-platform` | `station`, `stop`, `platform`, `public`, `outdoor` | `station`: here (0, 3600), there (4000, 4600) | ≈ 40 × 4 m, origin on the track edge, mid-length |

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
  3. `CORE_CONCEPTS.md`: a new `## 6.4 Role tags` after `## 6.3 SpatialRequirement` (the next free number;
     `check_doc_headings.py` must pass), with step-29 §14.1's quoted text, amended for Q-S29a-1's ruling in the
     sentence about the platform anchor.
  4. `VISUAL_SLICE.md` §4: a dated amendment paragraph ("Amended 2026-10-15 (step-29 §14.3, `ARC-105`,
     `ARC-106`)") in §4.1's amendment style: a third enterable building is **required**, the town's station; the
     street's west termination is "the station forecourt closing the view"; and in §12 a view 11, "the station
     from the street and in the hall". The out-of-scope sentence about a third building is struck through
     with `~~…~~` and kept visible.
  5. `DECISIONS.md`: under `ARC-105`, a dated note `**Note (2026-10-??, S29-a).**` recording §14's rulings: where
     the platform-anchor rule is checked (Q-S29a-1), whether a landmark-reachability code exists (Q-S29a-2), the
     `create` template's settlement (Q-S29a-3), and that `mineworld validate` now enforces the key (F-S29a-2).
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
  2. The template, as Q-S29a-3 rules (recommended: `kind: none` with the comment "# Not yet a town. A town names
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
  7. *(If Q-S29a-1 is ruled as recommended.)* `every_platform_anchor_lies_on_one_edge`: for `market-town` and
     `social-cafe`, `WorldPack::read_with` + `load(WorldTime::EPOCH)`, find the genesis `place-shaped` fact whose
     place is `station-platform`, decode it as `mineworld_bodies::PlaceShaped`, and assert that (0, 0) lies on
     exactly one edge of its floor. Add `mineworld-bodies` as a dev-dependency of `mineworld-cli` only if it is
     not one already (check `tools/cli/Cargo.toml`; a new dev-dependency on a workspace crate is not a new
     external dependency).
- Validation: `cargo test --release -p mineworld-cli --test station_visit` (release: the 30-day runs; record wall
  time as INCONCLUSIVE). If a run-based test needs `--ignored` by the file's own convention (as `rhythm.rs`),
  CI must still run it: confirm it is in a CI layer's command list (`python3 scripts/ci_layer.py --list core`)
  and record where.
- [ ] Implementation: steps 1–6. [ ] Validation: the file; the planted copies are the mutations. [ ] Review: the
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
  recorded reason.
- **R-RTc-a-1 (S21 RT-c, if Q-S29a-1 is ruled as recommended).** `transport` validates that the stop's platform
  anchor lies on exactly one edge of the stop's `body.floor` when it attaches `StopInfo`, refusing by name.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-29-station-landmark.md` §4.2, §9.5, §11.1 | the platform-anchor row as Q-S29a-1 rules; a landmark-reachability row if Q-S29a-2 is yes; the S29-a row's commit mapping (F-S29a-4) |
| `step-29-station-landmark.md` §2.1 | A-18 corrected: `mineworld validate` did not enforce `ARC-53`; the gate is new for it (F-S29a-2) |
| `step-29-station-landmark.md` §13.1 (Q-S29-7) | the reading of F-S29a-7: the one re-baseline is the id shift; S29-b records its sales as a behaviour change |
| `overall.md` | S29 in §3 (step-29 §14.7); the second, recorded re-baseline beside 12d's "Digests" rule; S29-a frozen / merged |

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

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29a-1 [operator — step §4.2's refusal table, `ARC-105` item 3]** | `TOWN_PLATFORM_ANCHOR` reads the stop's `body.floor`. `body:` is the `bodies` pack's section and opaque to the loader (`ARC-31`), so the loader can check it only by decoding `bodies`' format a second time or by adding a geometry query to the authoring seam (F-S29a-1). Where should the rule be enforced? | **Move it to its consumer, and hold it in content now.** (a) The loader drops `TOWN_PLATFORM_ANCHOR`; the rule stays in `CORE_CONCEPTS` §6.4 as the meaning of a stop's origin. (b) S29-a's C6 adds a content test, `every_platform_anchor_lies_on_one_edge`, that loads each reference town and reads the `place-shaped` genesis fact `bodies` states for `station-platform` (its public event type, `mineworld_bodies::PlaceShaped`, `systems/bodies/src/event.rs:21`; the section module itself is private), so the test reads the owner's own decoding of the floor (a test of the content, not a loader rule). (c) S21's RT-c, which owns `StopInfo` and its door anchors along that edge, refuses a stop whose anchor is off-edge when it attaches `StopInfo` (R-RTc-a-1). Alternatives rejected: a second decoder of `bodies`' format in the loader (two authorities for one format); a geometry method on `AuthoredContent` (a domain word in the seam, `CLAUDE.md` §4 rule 2); a check inside `bodies` keyed on the `stop` tag (a System reading a role tag, against I-S29-1) |
| **Q-S29a-2 [operator — I-S29-4 vs step §4.2]** | I-S29-4 says the **landmark** is reachable on foot from every seat's starting place, but §4.2's table has only `TOWN_STATION_UNREACHABLE`. In Lakeside the landmark is `pier`, which no code would check (F-S29a-3). Add `TOWN_LANDMARK_UNREACHABLE`? | **Yes.** One more code, the same search (§5.3 step 10), checked only when the landmark is not the hall (otherwise step 9 already covers it). It costs a dozen lines and one test, and it makes I-S29-4 a load-time fact as the invariant says |
| **Q-S29a-3 [operator — the `create` template]** | `mineworld create` writes a one-place world that `mineworld validate` checks at once (A-12). It now needs a `settlement:`. Declare `kind: none`, or make the template a minimal town (three more places: hall, ticket office, platform)? | **`kind: none`, with a comment pointing to MODULE_SPEC §4.3.** A one-room starter world with a station would add three places the author did not ask for and would make the guide's first world a station rather than a home. The comment makes becoming a town one step away. (The guide, S25, shows the town contract in its own chapter) |
| **Q-S29a-4 [operator — interim 3D picture]** | After C4, Social Café people walk to `station` on the server; the 3D slice cannot draw it until S29-d and shows them leaving the street's west end (F-S29a-6). Land Social Café's station routines in S29-a, or hold them for S29-d? | **Land them in S29-a.** The station group, its checks and `station_visit` need them (AC-S29-2 covers all three towns), and holding them would need a second re-baseline of Social Café in S29-d, against Q-S29-7's "one". S29 starts after Milestone F's checklist and ships before the release (step-29 §11.2), so no milestone the operator walks shows the gap; S29-d is the critical path |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C7 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Re-baseline (C4 step 7: one row per moved file, with its difference class)

(none yet)

### Mutations

(none yet; M-a-1 … M-a-4, X-1, X-2, X-3 and the scattered tag as §9 lists them)

### Deviations and findings

F-S29a-1 … F-S29a-6 (§3.2), recorded at design time.
