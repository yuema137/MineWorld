# PR S34-a — the `posture` System Pack

## DESIGN FROZEN

```text
Design revision:        revision 1 (2026-10-18), §§1–6 and §8
Approved by / evidence: operator ruling of 2026-10-18 in the primary session, relayed by the coordinator:
                        S34 frozen with the recommended answers (Q-S34-4 auto-stand, Q-S34-5 not pushable),
                        and S34-a's PR design, written to follow the frozen step, recorded as frozen
Implementation base:    origin/main after #264 merged (C0 records the exact commit)
Execution contract:     §9
Lifecycle:              FROZEN
```

Scope (§1), invariants (§3), decisions SD-a-1…SD-a-10 (§4), acceptance (§5) and the commit plan's intent
(§6) are frozen. Progress, evidence (§10), findings and bounded corrections (§11) stay writable.

**Effort:** `mvp0` · **Step:** [`step-34-posture.md`](step-34-posture.md) §§5–6, 10, 12–14 (the S34-a row) ·
**Parent:** [`overall.md`](overall.md) · **Record:** `ARC-147` (reserved for posture; written by this PR).
Spare ids from the lane's range `ARC-260…ARC-265` are not used unless a bounded discovery needs one.

**Working name:** S34-a. **Branch:** `mvp0/pr-s34a`. **Worktree:**
`/Users/yuema137/mineworld-worktrees/s34a-posture`. One session holds it.

---

## 1. Scope

**In:**

1. A new crate `systems/posture` (`mineworld-posture`), a System Pack with id `posture`, VERSION 1:
   - the place section `sit_points:` (`ARC-31`), carried by Place files only, seeded as `place-sit-points`
     and reduced into `SitPoints` (component type `sit-points`) on the Place;
   - `posture-state` on a Person: `Seated { point, since }`; standing is the absence of the row;
   - actions `sit { point }` and `stand {}`, validated in the step's order (§4);
   - facts `sat-down` and `stood-up { cause: requested | moved }`;
   - the auto-stand reaction to presence's facts (Q-S34-4);
   - offers (complete) and disclosure (`PerceptionProvider`);
   - its section of the World's Interaction List, `configure/posture.yaml` (`ARC-63`): rule on `sit`,
     parameter `reach`, the two facts' audiences.
2. Registration: one line in `systems/installed/src/lib.rs` and its `Cargo.toml`, the workspace member and
   dependency entry.
3. Tests: the integration path (compose → genesis with sit points → sit → bystander observation →
   walk away → auto-stand → persist → drop → resume → still seated), every refusal row against its
   offer, contention, decoding refusals of the section, the Interaction List section, byte identity of a
   world without the pack.
4. Specs first: `docs/CORE_CONCEPTS.md` (sit point), `docs/MODULE_SPEC.md` (the section and the list
   section), `ARC-147`; the generated reference regenerated (`mineworld reference --out docs/reference`).

**Out (later PRs of the step):** bodies' `Anchoring` catalog (S34-b, after 12d #171); sit points in shipped
worlds and enabling the pack there (S34-c); controllers' evaluation, the stillness gate, the agent view row
(S34-d); every client change (S34-e, S34-f). No shipped world's `systems:` list names `posture` in this PR,
so every shipped world stays byte-identical.

## 2. Source audit (`origin/main @ b85f2804`)

The step's audit (§1, A-1…A-27) holds. Additional facts this PR's plan rests on:

| Fact | Where |
| --- | --- |
| A pack with an authored section implements `AuthoredSection` (`SECTION`, `CARRIED_BY`, `Authored`, `seed`) and says `mineworld_sdk::owns_section!()` in `impl SystemPack` | `systems/appearance/src/section.rs`, `system.rs` |
| A pack with an Interaction List section implements `InteractionSection` (`Parameters` via `mineworld_sdk::parameters!`, `Knobs`, `CONFIGURED`, `COMPONENT`, `ACTIONS`, `FACTS`, `REFERENCE`) and wraps its declaration and install in `interactions::declare` / `interactions::install`, saying `mineworld_sdk::interactions!()` | `systems/group-activity/src/interactions.rs`, `system.rs:40-110` |
| A pack with both says both macros (check that the SDK allows `owns_section!()` and `interactions!()` together; if not, follow the pack that already does, or record the finding) | `sdk/src/` (to be audited at C2) |
| The list's carrier is `configure/<pack>.yaml`, not `interactions/<pack>.yaml` (QIL-2: "the carrier stays `configure:`") — the step's §6.6 file name is corrected here (BC-1) | `step-18-interaction-list.md` lifecycle line; `docs/reference/interaction-list.md` |
| Offers: `Offer::complete(&action, requirement)`, `.refused(reason)`; `target: None` for actions directed at nobody; perception iterates `None` first, then each person present | `systems/presence/src/interaction.rs:146-220`; `observe.rs:236-250` |
| Places are perceived subjects (the observer's place is the first entity), so `discloses(world, observer, place)` is asked | `systems/presence/src/observe.rs:104-125` |
| Relocating a person exactly: `mineworld_presence::arrival(world, person, location) -> Result<Emission, Rejection>`; refuses with `resolution-refused` when a resolver would change the placement | `systems/presence/src/event.rs:85-112` |
| `Arrived { person, location }` accessors; `PersonEnteredPlace`, `StoppedShort` | `systems/presence/src/event.rs` |
| `mineworld_movement::is_walking(world, person) -> bool` | `systems/movement/src/system.rs:350` |
| Component writes in reductions: `WorldView::insert` / `remove` | `kernel/src/view.rs:276, 292` |
| Persisted resume test pattern (`PersistentWorld::create`, `SqliteBackend`, `mineworld_test_support::scratch!`) | `systems/group-activity/tests/persisted.rs` |
| The installed set's tests hold the list and the manifest equal | `systems/installed/tests/installed.rs` |
| Generated reference and schemas, regenerated by `mineworld reference --out docs/reference` | `docs/reference/README.md` |

## 3. Invariants (from the step, as they bind this PR)

- **I-S34-1** exclusivity; **I-S34-2** seated means at the point (position; see BC-2 for facing);
  **I-S34-3** only posture writes posture, the relocation is presence's own `Arrived`; **I-S34-6** byte
  identity without the pack; **I-S34-8** no height, mesh or clip in any posture type.
- **I-a-1.** Offer and dispatch answer the same reason, in the same order, for every row of §4.
- **I-a-2.** No `HashMap`/`HashSet`, no float anywhere in the crate; sit points sorted by key; occupancy
  scanned in `PersonId` order.
- **I-a-3.** The pack depends only on contracts, kernel, authoring, sdk, presence and movement.

## 4. Decisions

- **SD-a-1. Types.** `SitPointKey` and `SitGroupKey`: slugs of 1–48 bytes `a`–`z`, `0`–`9`, `-`, `.`
  (the dot allows `window-bench.3`). `SitPoint { key, at: {x, y}, facing: yaw mdeg, group? }`. `SitPoints`
  holds at most 64, sorted by key, keys unique, spacing ≥ `2·PERSON_RADIUS + GAP` measured with the
  constants **posture declares itself** (`SIT_POINT_SPACING` = 510 mm, the 12d value; posture does not
  depend on bodies). Coordinates within ± 100 000 mm. Invalid values cannot be constructed (`try_from`).
- **SD-a-2. Posture.** `Posture::Seated { point: SitPointKey, since: WorldTime }`, component type
  `posture-state`, schema 1, serialized as an object `{ "seated": { "point": …, "since": … } }`.
- **SD-a-3. `sit` validation order** (the step's §6.1, exactly): not a person / no presence →
  `PreconditionFailed`; list forbids → `PermissionDenied`; unknown point in the actor's place →
  `NoSupportedInteraction`; already seated → `Busy`; walking → `Busy`; farther than `reach` from `at` →
  `TooFarAway`; another seated there → `TargetUnavailable`; another standing person within 510 mm of `at` →
  `TargetUnavailable`; and, last, `presence::arrival` to the point refused → `TargetUnavailable` (BC-3).
  A malformed payload → the pack's own `malformed-payload` code, as group-activity.
- **SD-a-4. `stand` validation:** not a person / no presence → `PreconditionFailed`; not seated →
  `PreconditionFailed`. Never governed by the list.
- **SD-a-5. Emissions.** `sit`: `arrival(...)` (omitted when the person already stands exactly at the
  point's position and facing), then `sat-down`. `stand`: `stood-up { cause: requested }`. Audiences:
  `Place` by default, narrowable to `Participants`; not biographical (`BIOGRAPHICAL = &[]`).
- **SD-a-6. Auto-stand (Q-S34-4).** Posture subscribes to presence's `arrived` (and `person-entered-place`
  if an arrival into another place is stated without `arrived` — audited at C4). Reacting to an `arrived`
  whose person holds `Seated { point }` and whose new position or place differs from the point's →
  `stood-up { cause: moved }`. The reaction reads the **fact's payload**, not `Presence`, so it is
  independent of reduction order. A `sat-down` in the same request follows its `arrived`, so the sitter's
  own relocation never stands them up (they are not yet seated when it is reacted to).
- **SD-a-7. Offers.** Standing observer in a place with sit points: one complete `sit { point }` per point
  in key order, `target: None`, `SpatialRequirement` place `SamePlaceAsActor` with no range, carrying the
  pack's refusal per SD-a-3 (the arrival dry-run is asked only for points within `reach`, so an
  observation costs at most one resolver fold per nearby point). Seated observer: one complete `stand {}`.
- **SD-a-8. Disclosure.** `SitPoints` about the place, to every observer who perceives it; `posture-state`
  about a person, to every observer who perceives them.
- **SD-a-9. List section.** `configure/posture.yaml`: actions `sit` (roles actor, place; regional);
  parameter `reach: u32 = 750, 300 ..= 1_500`; facts `sat-down`, `stood-up` (actor = participant 0, place;
  default audience `Place`, narrowest `Participants`, biography not configurable). `stand` is not declared,
  so a list naming it is refused at load.
- **SD-a-10. People without positions.** A place has no "models positions" flag to refuse a section
  against, so the check is on the **person**: a person with no `local` position is `TooFarAway` for every
  point (the step's row 6), and the offer says so (BC-4).

## 5. Acceptance (decided before measuring, `ARC-23`)

| Id | Criterion | Test |
| --- | --- | --- |
| AC-a-1 | AC-S34-1 in a test world (presence + movement + posture): sit within reach; `Presence` at `at`/`facing`; `sat-down` recorded; observer and bystander see `posture-state`; `stand` removes it, location unchanged | `tests/sitting.rs` |
| AC-a-2 | AC-S34-2: each refusal row is produced by a request built for it, and the observer's offer for the same point carries the same reason first | `tests/refusals.rs` |
| AC-a-3 | AC-S34-3: `move` from a seated person emits `stood-up { cause: moved }` in the same dispatch; turning in place (same position, new facing) does not (BC-2) | `tests/sitting.rs` |
| AC-a-4 | Contention: two requests for one point at one instant → one `sat-down`, one `TargetUnavailable` | `tests/refusals.rs` |
| AC-a-5 | AC-S34-4's resume half: seated before a drop, seated after a resume from SQLite with the same `since`; `verify` passes | `tests/persisted.rs` |
| AC-a-6 | Section decoding: duplicate key, 65 points, points 509 mm apart, a coordinate past 100 000 mm, an unknown field, the section in a person file — each refused, naming the problem | `src/section.rs` tests, `tests/section.rs` |
| AC-a-7 | List: `configure/posture.yaml` forbidding `sit` for a class → offer and dispatch `PermissionDenied`; `reach: 300` makes a 400 mm sit `TooFarAway`; naming `stand` is refused | `tests/interactions.rs` |
| AC-a-8 | Byte identity: the shipped worlds' existing digest/golden tests pass unchanged; a world without `posture` gets no `sit` affordance and `sit` is `Unavailable` | existing suites; `tests/sitting.rs` |
| AC-a-9 | With bodies installed and no S34-b yet: a walker who nudges a seated person stands them up (`cause: moved`), so I-S34-2 holds even before S34-b | `tests/sitting.rs` |

**Planted mutations (each turns a named test red; evidence in §10):** X-S34-1 (drop the occupancy check),
X-S34-2 (drop the auto-stand reaction), X-S34-4 (offer measures `reach`, `validate` does not).

## 6. Commit plan

Each commit: **Implementation**, **Validation**, **Review**, each `[ ]` until done with evidence.

### C0 — Design (this file), docs only
- [ ] Implementation: this file; the base commit recorded in §10.
- [ ] Validation: `scripts/check_decision_ids.py` (no new id yet); links resolve.
- [ ] Review: scope matches the step's S34-a row; no client or shipped-world edit planned.

### C1 — Specs before code
- [x] Implementation: `docs/CORE_CONCEPTS.md` §6.4 "Sit point" (semantic place data, free floor; "seat" is
  not the term); `docs/MODULE_SPEC.md`: the `sit_points:` place section (after the places example) and the
  `posture` section of the Interaction List (§4.2); `ARC-147` in `docs/DECISIONS.md` (`docs/decisions/`
  does not exist on this base, so the existing layout is followed), the record of step §17.
- [x] Validation: `python3 -I scripts/check_decision_ids.py` → `169 decision ids, all distinct` (PASS);
  `python3 -I scripts/check_doc_headings.py` → `242 numbered sections across 41 documents, none duplicated`
  (PASS).
- [x] Review: self-review by the implementer, not independent. Terminology: "sit point" throughout, "seat"
  only in the explicit exclusion; `posture-state` and `posture` kept distinct from S30-e's attribute; the
  record states Q-S34-1…14 and SD-a-1…10's shape. Findings in §11 (BC-5, BC-6) are recorded, not patched.

### C2 — Crate skeleton: types, components, facts, the section
- [x] Implementation: `systems/posture/{Cargo.toml, src/lib.rs, key.rs, point.rs, component.rs, event.rs,
  codec.rs, section.rs, system.rs}`; workspace member and dependency; `SitPointKey`, `SitPoint`,
  `SitPoints` (validated), `Posture`; `PlaceSitPoints`, `SatDown`, `StoodUp` events; `AuthoredSection`
  for `sit_points:` (Place only) seeding `place-sit-points`; reduction into `SitPoints`.
- [x] Validation: `cargo test -p mineworld-posture` → `tests/section.rs` 9 passed, 0 failed (AC-a-6 by
  decoding through the section's own type, not unit tests in `src/`: BC-8); `cargo clippy -p mineworld-posture
  --all-targets -- -D warnings` → exit 0; `cargo fmt -p mineworld-posture -- --check` → clean after
  `cargo fmt`. The section's file-kind refusal (a `sit_points:` in a person file) waits for the loader test at C7.
- [x] Review: self-review. No float, no `HashMap`/`HashSet` in the crate; every refusal names its cause
  (`InvalidSitPoints`, `InvalidSitKey`, `Written` with `deny_unknown_fields`); serialization is objects
  (`Posture` as `{ "seated": … }`, `SitPoint` as `{ key, at: {x,y}, facing, group? }`). Finding BC-7 (the
  spacing constant) is recorded in §11 and not applied.

### C3–C7 — implemented as one commit in the recovery session (2026-10-19)

The previous session stopped after C2; the recovery session implemented C3–C7 together under the
charter's speed-first rule (one commit, not five). Each item below names its evidence.

- [x] Implementation (C3): `src/action.rs` (`Sit { point }`, `Stand {}`, `REACH`), `validate` per SD-a-3/4
  through one function `sit_verdict` shared with `resolve` and the offer, `resolve` per SD-a-5 (presence's
  `arrival`, never a `Presence` write), reductions of `sat-down` / `stood-up`.
- [x] Implementation (C4): subscriptions to presence's `arrived` and movement's `walk-started`; the
  reaction per SD-a-6 as corrected by BC-6 (compares the fact's place and position with the row).
- [x] Implementation (C5): `src/perception.rs` per SD-a-7/8 (BC-5 requirement).
- [x] Implementation (C6): `src/interactions.rs` per SD-a-9, wrapped in `interactions::declare` /
  `install` / `reduce`; `owns_section!()` and `interactions!()` compose in one `impl SystemPack`.
- [x] Implementation (C7): `systems/installed` line and manifest; `tests/persisted.rs`; `docs/reference`
  regenerated; `systems/posture/README.md`.
- [x] Validation: `cargo test -p mineworld-posture --no-fail-fast` → PASS, 24 tests (interactions 5,
  persisted 1, refusals 3, section 9, sitting 6), 0 failed; `cargo clippy -p mineworld-posture
  -p mineworld-installed-systems --all-targets -- -D warnings` → exit 0; `cargo fmt` applied. All
  through `cast-slots/with-slot.sh`. Workspace-wide clippy, the shipped worlds' golden suites and the
  CLI reference test are left to the train/milestone CI (charter: no full local runs per PR); AC-a-8's
  shipped-world half is therefore INCONCLUSIVE locally — no shipped world names `posture`, and the
  posture-less scratch world test passes.
- [x] Mutations: X-S34-1 (occupancy check disabled) → `two_requests_for_one_point_at_one_instant_seat_one_person`
  FAILED; X-S34-2 (auto-stand on `arrived` disabled) → `a_seated_person_who_steps_away_…`,
  `…_leaves_the_place_or_sets_off_…`, `…_shoved_off_their_point_…` FAILED; both reverted, suite green
  again. X-S34-4 is N/A: offer and validate call the same `sit_verdict`, so a reach check in one and not
  the other cannot be planted without restructuring.
- [x] Review: self-review (the charter's speed-first rule replaces the independent reviewer per PR).
  Check order equals the step's table; the sitter's own `arrived` cannot stand them up (the row does not
  exist yet, and the position equals the point); a facing-only arrival keeps them seated; offers in key
  order; disclosure only about the subject. No float, no hash map in the crate.

## 7. Test ownership

| Failure class | Owner |
| --- | --- |
| Construction and decoding refusals | unit tests in `src/` |
| Validation order, offers = dispatch, contention, auto-stand, list | integration tests in `systems/posture/tests/` |
| Persistence across resume | `tests/persisted.rs` |
| Byte identity of shipped worlds | the existing golden/digest suites (unchanged) |
| Cross-platform replay | the train's CI matrix (macOS, Linux, Windows) |
| Real-LLM Gate | N/A: no model involved |
| Clients | N/A in this PR (S34-e, S34-f) |

## 8. Is any of this material?

No. Everything here is the frozen step's S34-a row. A discovery that would need a presence, movement,
kernel or contracts change stops and is escalated.

## 9. Execution contract

```text
Repository / worktree:  yuema137/MineWorld · /Users/yuema137/mineworld-worktrees/s34a-posture
Branch / base:          mvp0/pr-s34a from origin/main after #264
Authorized:             implement, validate locally (cast-slots/with-slot.sh), commit, push, open the PR,
                        update it; component-PR pushes run only `fast` (merge-train rule)
Validation budget:      local crate tests and clippy per commit; one workspace clippy + targeted tests at C7
Stop condition:         train-ready: head SHA, local validation, reviewer APPROVE, mutation evidence
Merge boundary:         never merged directly; merged through a train (LANE_LEAD_CHARTER.md)
Escalate:               any change to a frozen invariant, a public contract outside this crate, ownership, or a
                        new dependency
```

## 10. Evidence ledger

- **Base.** `9304fdd6` (C0, the frozen design) on `mvp0/pr-s34a`; its parent is `370b9da0`
  (merge of #267). The C0 commit does not record a separate `origin/main` SHA; the session did not
  verify one.
- **C1.** `python3 -I scripts/check_decision_ids.py` → PASS, `169 decision ids, all distinct`.
  `python3 -I scripts/check_doc_headings.py` → PASS, `242 numbered sections across 41 documents, none
  duplicated`. Files: `docs/CORE_CONCEPTS.md`, `docs/MODULE_SPEC.md`, `docs/DECISIONS.md` (ARC-147),
  this design. Self-review only (see C1 Review).

## 11. Bounded corrections of the step (recorded, not material)

- **BC-1.** The Interaction List file is `configure/posture.yaml` (QIL-2), not `interactions/posture.yaml`
  (step §6.6).
- **BC-2.** I-S34-2 binds position and place; facing may change while seated (a seated person turns their
  head). Auto-stand fires only on a change of place or position. The step's I-S34-2 text "facing = the
  point's facing" holds at the instant of sitting.
- **BC-3.** A last validation row: if presence's `arrival` to the point is refused (a resolver would change
  it, e.g. a solid between the person and the point), `sit` is `TargetUnavailable`.
- **BC-4.** "`sit_points:` in a place without positions is refused at load" (step §6.1) has no fact to test
  a place against; the refusal is per person (`TooFarAway` without a `local` position).
- **BC-5 (found at C2 audit, ESCALATED — not applied).** `SpatialRequirement::evaluate` answers
  `PreconditionFailed` for `SamePlaceAsActor` when the target is `None` (`contracts/src/spatial.rs`,
  `evaluate`: `target.ok_or(Rejection::PreconditionFailed)`). Presence's `verdict` evaluates every offer
  without a refusal. So SD-a-7's requirement `SamePlaceAsActor` would make every permitted, reachable
  `sit` offer report `PreconditionFailed`. The only fix inside this crate is to declare
  `SpatialRequirement::NONE` for `sit` and let posture's own refusal carry the same-place rule (points are
  looked up in the actor's place by construction). That changes the declared requirement a client is
  shown, which SD-a-7 freezes; it needs the operator's ruling.
- **BC-6 (found at C3 design audit, ESCALATED — C4 blocked).** SD-a-6 asks the auto-stand reaction to
  compare the arriving location with "the point's place and `at`". The seated row names only the key
  (SD-a-2, `I-S34-2`: the place is the Presence place), and the reaction cannot read the place a person
  left: presence's `arrived` is reduced before posture reacts in the same generation
  (`kernel/src/dispatch.rs` `reduce`, registration order), so `Presence` already holds the new location
  and the old place is lost. Keyed-by-point lookup is not unique across places, so a place change to a
  place with a same-named, same-coordinate point would wrongly keep the person seated. The step's
  "reads the fact's payload" cannot supply the old place either. The smallest revision is to store the
  place in the row: `Seated { place, point, since }`, with I-S34-2 checked at the reaction. That reverses
  SD-a-2's "the place is not stored twice". It is a frozen decision, so C4 waits for the operator's
  ruling; C1–C3 do not depend on the choice beyond the row's shape.
- **BC-7 (found at C2, ESCALATED — not applied).** SD-a-1 and Q-S34-8 take the spacing from "the 12d value"
  (`SIT_POINT_SPACING` = 510 mm, i.e. `2 × 250 + 10`). The bodies pack on `origin/main` states
  `PERSON_RADIUS` = 300 mm and `GAP` = 10 mm (`systems/bodies/src/geometry.rs`), so `2R + GAP` = 610 mm, and
  bodies' own invariant (`2 × PERSON_RADIUS − 5 mm`) is 595 mm. Two seated people would overlap in the 3D
  capsule at 510 mm. The 600 mm per-person bench default of Q-S34-8 would then be refused. The code uses the
  frozen 510 mm, as the design says; the operator decides between 510 (and a new bench default) and 610.
- **Resolution of BC-5, BC-6, BC-7 (2026-10-19, recovery session, under the operator's 2026-10-18
  ruling "recommended answers" and the charter's speed-first rule; recorded in `ARC-147`'s
  "Bounded corrections" paragraph).** None changes a contract outside this crate.
  - BC-5: `sit` declares `SpatialRequirement::at_place(<observer's place>)` (an existing constructor,
    `PlaceRequirement::Specific`), which perception evaluates without a target. Reach, occupancy and
    walking stay the pack's refusal, so I-a-1 holds.
  - BC-6: the row is `Seated { place, point, since }` (the smallest revision BC-6 proposed). The
    reaction compares the `arrived` payload's place and position with the row's place and point. It
    also stands a seated person up on movement's `walk-started`, so Q-S34-4 ("a seated person who
    walks stands up") holds from the request that starts the walk.
  - BC-7: void. `origin/main` after 12d (#171) states `PERSON_RADIUS` = 250 mm, so 510 mm = `2R + GAP`.
- **BC-9 (C3–C6).** The tests assemble scratch World Packs through the real loader (`mineworld-worldpack`
  as a dev-dependency, as weather's tests do) instead of hand-built worlds, so the section, the list and
  genesis are exercised as a world author writes them. Two layouts (a café with a street; a pier with
  bodies) satisfy the charter's second-world rule.
- **BC-10 (C3, found at the first build).** The section key is `sit-points:`, not `sit_points:`.
  `SectionName` accepts only `a`–`z`, `0`–`9` and `-` (`authoring/src/section.rs`); the C2 skeleton's
  `from_static("sit_points")` panics at compile time. Widening `SectionName` would be an authoring
  contract change, so the key follows the existing rule (and matches the component type `sit-points`).
  The step and the S33 plans (`step-33-lakeside-3d.md`, `pr-s33c-world-seam.md`) still write
  `sit_points:`; their owners update them when they author sit points (S34-c).
- **BC-8 (C2).** The step's and the design's AC-a-6 name "`src/section.rs` tests". The decoding refusals are
  tested in `tests/section.rs` instead, through the section's own `Deserialize`, because a unit test in `src/`
  would construct the values rather than decode them as the loader does. The behaviour is the same; the
  location differs.
