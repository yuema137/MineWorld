# PR S34-b — bodies' `Anchoring` catalog: a seated person is not pushed

## Lifecycle

```text
Design revision:        revision 1 (2026-10-19), DRAFT
Approved by / evidence: none yet — awaiting the operator's freeze (questions in §12)
Implementation base:    origin/main @ 5a8c6f28 (after #301, S34-a, and #323)
Execution contract:     §10
Lifecycle:              DRAFT — awaiting freeze
```

**Effort:** `mvp0` · **Step:** [`step-34-posture.md`](step-34-posture.md) §7.1, §§12–14 (the S34-b row;
AC-S34-5, I-S34-7, X-S34-3, X-S34-10) · **Previous PR:** [`pr-s34a-posture-pack.md`](pr-s34a-posture-pack.md)
(merged as #301) · **Record:** a dated note on `ARC-147` (`docs/decisions/ARC-147.md`) and on `ARC-62`
(the fifth catalog line). No new id unless the operator rules otherwise.

**Working name:** S34-b. **Branch:** `mvp0/pr-s34b`. **Worktree:**
`/Users/yuema137/mineworld-worktrees/s34b-anchoring` (created at implementation). One session holds it.

**In one sentence.** Today a walker who bumps into a seated person nudges them off their sit point, and
posture then stands them up (S34-a AC-a-9). The operator ruled that a seated person cannot be pushed
(Q-S34-5). This PR lets any pack tell bodies "this person is held in place", and bodies then treats that
person like a fixed post: walkers stop short, `shove` is refused, and a held point that lies inside
furniture is refused when the world loads.

---

## 1. Scope

**In:**

1. **The catalog** in `systems/bodies` (`ARC-62`'s mechanism, the same shape as presence's
   `ArrivalResolver` and movement's `Wayfinder`):
   - `pub trait Anchoring: Send + Sync` with `anchoring_of`, `anchored`, `fixed_points` (§4 SD-b-1);
   - `register_anchorings(Vec<Box<dyn Anchoring>>)`, write-once, sorted by id, duplicate and
     different-list panics, exactly as `register_wayfinders`;
   - `require_anchoring(&SystemId)`, called first in an implementer's `System::install`.
2. **Bodies consults it** in three places:
   - the nudge pass (`stride::nudge`, also reached from `entry`'s E2): an anchored person is never
     moved; touching one fails the pass, so the walker is blocked at first contact — the existing
     degrade path, no new route kind;
   - `shove`: an anchored target is `TargetUnavailable`, in `validate` and in the offer alike;
   - the load-time fixed-point check (`BODIES_FIXED_POINT_IN_SOLID`, §4 SD-b-5).
3. **Posture implements it:** `anchored` = the person has a `Seated` row; `fixed_points` = the place's
   `SitPoints`. `mineworld-posture` gains the `mineworld-bodies` dependency.
4. **Registration:** one `extension` line in `systems/installed/src/lib.rs`.
5. **Versions:** posture `VERSION` 1 → 2 (§4 SD-b-7; the step said bodies — see Q-S34b-1).
6. **Tests** (§6) on two layouts: the café-like layout and the pier layout S34-a's tests already build
   through the real loader (BC-9 of S34-a), plus a generated variant (§6, AC-b-6).
7. **Docs:** `ARC-147` note, `ARC-62` note, `systems/bodies/README.md` and `systems/posture/README.md` one
   paragraph each, `docs/MODULE_SPEC.md` §3.1 catalog list.

**Out:** sit points in shipped worlds and enabling posture there (S34-c); NPCs and agents (S34-d); every
client change (S34-e, S34-f); ball flights (`kick`, `throw`): an anchored person is an obstacle to a flying
object exactly as any person is today (step §7.1, last paragraph).

## 2. Source audit (`origin/main @ 5a8c6f28`)

| Fact | Where |
| --- | --- |
| Catalog pattern: trait + `OnceLock<Vec<Box<dyn …>>>` + `register_…` (sorted, duplicate panic, different-list panic) + `require_…` | `systems/presence/src/resolve.rs:157-230`; `systems/movement/src/wayfinder.rs:145-200` |
| The installed set lists catalogs as `extension <trait> => <register fn>: [<types>];`; four lines today | `systems/installed/src/lib.rs:41-44`; `docs/decisions/ARC-62.md` items 1–6 |
| Bodies' resolver builds `Here { room, standing, objects }` per arrival from `WorldRead`; so a catalog asked with `&WorldRead` fits | `systems/bodies/src/resolve.rs:146-150, 209-238` |
| `standing_in(world, place)` lists the place's people in `EntityId` order | `systems/bodies/src/system.rs:269-279` |
| The nudge pass: returns `Err(())` when a nudge exceeds its bounds; the caller then degrades (blocked at first contact → halvings → stay) | `systems/bodies/src/stride.rs:355-431`, header `:1-11` |
| `entry` E2 nudges people off an entry point through the same `nudge` | `systems/bodies/src/entry.rs:47-75` |
| `shove`: `has_body` decides `available`; `validate` and the offer both evaluate `shove_requirement()` with it, so one change answers both | `systems/bodies/src/shove.rs:41-62`; `systems/bodies/src/offer.rs:65-89` |
| Bodies reduces `place-shaped` after `genesis::fits` (people, capacity, solids, doorways) | `systems/bodies/src/system.rs:130-133`; `systems/bodies/src/genesis.rs:57-200` |
| The doorway check is the precedent for "a fixed point of the place must hold a person": it reads movement's `Passages` | `systems/bodies/src/genesis.rs:143-200` |
| Within one place file, sections are seeded in the world's `systems:` order, so `body:` and `sit-points:` arrive in either order | `worldpack/src/load.rs:404-420, 457-481` |
| Posture reduces its own `place-sit-points` into `SitPoints`; its row is `Seated { place, point, since }` | `systems/posture/src/system.rs:60-75`, S34-a BC-6 |
| Bodies `VERSION` = 5; `tests/rapier_pin.rs` pins `(VERSION, rapier)` | `systems/bodies/src/system.rs` (`const VERSION`); `systems/bodies/tests/rapier_pin.rs:9, 109` |
| Posture `VERSION` = 1 | `systems/posture/src/system.rs:53` |
| Worlds with `bodies` in `systems:`: bodies-yard, social-cafe, market-town; their long-run digests are in `scripts/baselines.txt` and move whenever a pack's version in the composition moves | `worlds/*/world.yaml`; `scripts/baselines.txt` header |
| S34-a's AC-a-9 test asserts a nudged seated person stands up (`cause: moved`); this PR replaces that behaviour | `systems/posture/tests/sitting.rs` |

## 3. Invariants

- **I-S34-7** (step): with bodies and posture installed, no fact moves a seated person except their own
  request.
- **I-S34-6** (step): a world without `posture` is byte-identical before and after this PR.
- **I-b-1. Inert without state.** With no implementer registered, or no anchored person in the place,
  every bodies result is byte-identical to `5a8c6f28` (the existing bodies suites run unchanged).
- **I-b-2. Offer equals dispatch.** `shove` of an anchored person answers `TargetUnavailable` in the offer
  and in `validate`, from one function.
- **I-b-3. Pure.** `anchored` and `fixed_points` read only the world they are handed; no cache, no float,
  no hash map; results in `EntityId` / key order.
- **I-b-4. Dependency direction.** Bodies names no implementer. Posture depends on bodies (the trait),
  never the reverse. A future pack that pins a person (a bed, a vehicle) adds an `impl` and one entry on
  the catalog line, and edits nothing in bodies.
- **I-b-5. No interpenetration (operator 2026-10-19).** Every fixed point is on the floor and clear of
  every Solid by `PERSON_RADIUS + GAP` (250 + 10 = 260 mm on main), or the world does not load.

## 4. Decisions

- **SD-b-1. The trait** (the step's sketch, kept):

  ```rust
  /// Asked by bodies, in ascending `anchoring_of` order; pure; inert where its pack's state is absent.
  pub trait Anchoring: Send + Sync {
      fn anchoring_of(&self) -> SystemId;
      /// Whether `person` is held where they stand: never nudged, never shoved.
      fn anchored(&self, world: &WorldRead<'_>, person: PersonId) -> bool;
      /// The points of `place` a person must be able to occupy, with a name an author recognizes
      /// (posture: the sit point's key), in key order.
      fn fixed_points(&self, world: &WorldRead<'_>, place: PlaceId) -> Vec<(String, LocalPosition)>;
  }
  ```

  A person is anchored when **any** implementer says so. Bodies computes one `Vec<bool>` per `Here`,
  parallel to `standing`, so each arrival asks each implementer once per person in the place (tens).
- **SD-b-2. Nudge.** In `nudge`, an index whose person is anchored is never moved: a pusher that would
  touch them (`between < touching²`) makes the pass return `Err(())`. The caller's existing degrade then
  blocks the walker at first contact. The contact sweep already treats every person as a disc, so the
  walker never overlaps the seated person.
- **SD-b-3. Entry.** `entry`'s E2 uses the same `nudge`, so an arrival onto a point that would need an
  anchored person moved falls through to E3 (nearest free point), unchanged code.
- **SD-b-4. Shove.** `has_body` becomes "a position, in a shaped place, and not anchored". `validate` and
  the offer keep calling it, so both answer `TargetUnavailable` (I-b-2). The other caller of `has_body`
  is the offer, so no other action changes.
- **SD-b-5. Load-time fixed-point check, independent of section order.** A fixed point is refused unless
  a person fits there: inside the floor by `PERSON_RADIUS`, clear of every Solid by
  `PERSON_RADIUS + GAP` — the same test `genesis::doorways_fit` applies to doorway points — refused as
  `Rejection::System { code: "bodies-fixed-point-in-solid", detail }`, the detail naming the place, the
  key, the point and the Solid it meets. Because `body:` and `sit-points:` of one place are seeded in the
  world's `systems:` order (audit row 10), the check runs in **whichever reduction comes second**:
  - bodies' `place-shaped` reduction asks every implementer's `fixed_points(place)` (non-empty only if
    the sit points were stated first);
  - posture's `place-sit-points` reduction calls `mineworld_bodies::fixed_points_fit(world, place,
    &points)` (non-empty work only if the place is already shaped).

  Both call one bodies function, so the rule and the message are the same either way. With bodies not
  enabled there is no `PlaceShape`, and the check is skipped — sitting still works (step §13.3 scenario).
  People and loose objects are not part of this check: a person authored onto a sit point is legal (they
  will be asked to move by the sitter's `TargetUnavailable`, S34-a row 8), and objects move.
- **SD-b-6. Registration.** `extension mineworld_bodies::Anchoring => mineworld_bodies::register_anchorings:
  [mineworld_posture::PostureSystem,];`. Posture's `install` calls `mineworld_bodies::require_anchoring`
  first, so a host that forgot the line fails loudly instead of letting walkers push seated people.
- **SD-b-7. Version.** Posture `VERSION` 1 → 2, bodies stays 5. Reason: bodies' results change **only**
  where an implementer reports an anchored person, and the only implementer is posture; a world without
  posture produces the same facts as before, so its saves and digests stay valid. A world with posture
  produces different facts (stopped short instead of nudged), and posture's version is in its composition,
  so its old saves are refused by name (`ARC-25`). This keeps bodies-yard, Social Café and Market Town
  byte-identical (I-S34-6) and avoids re-baselining three worlds in a PR that changes none of them. The
  step's §7.1 said "bodies' VERSION bumps"; this is Q-S34b-1. **Rule stated for future implementers:** a
  pack that starts or changes anchoring bumps its own version.
- **SD-b-8. S34-a's AC-a-9 changes meaning.** The test "a walker who nudges a seated person stands them
  up" becomes "a walker aimed through a seated person is stopped short; the seated person keeps `Seated`
  and `Presence`". The auto-stand on `arrived` stays (a seated person's own `move` still stands them up).

## 5. Acceptance (decided before measuring, `ARC-23`)

| Id | Criterion | Test |
| --- | --- | --- |
| AC-b-1 | AC-S34-5 (walker): in the café layout and in the pier layout, a walker whose stride passes through a seated person stops short of them (no overlap beyond `TOLERANCE`), the seated person's `Presence` and `Seated` row are unchanged, and no `stood-up` is recorded | `systems/posture/tests/anchoring.rs` |
| AC-b-2 | Entry: a person arriving onto a point that only a nudge of a seated person would free lands at the nearest free point instead | `tests/anchoring.rs` |
| AC-b-3 | AC-S34-5 (shove): `shove` of a seated person is `TargetUnavailable` in dispatch, and the observer's offer for the same target carries `TargetUnavailable`; after `stand`, the same shove succeeds | `tests/anchoring.rs` |
| AC-b-4 | AC-S34-5 (load): a sit point 100 mm from a Solid's edge is refused at load naming the place, the key and the Solid, **in both `systems:` orders** (bodies before posture, posture before bodies); a point outside the floor likewise; a point exactly 260 mm clear loads | `tests/anchoring.rs` (two scratch worlds per case) |
| AC-b-5 | I-b-1 / I-S34-6: every existing bodies test passes unchanged; bodies-yard, Social Café and Market Town genesis digests and `scripts/baselines.txt` are unchanged | existing suites (milestone CI) |
| AC-b-6 | I-S34-7 seeded run: a generated variant (a shaped place with N sit points from a seeded generator, N walkers on random walks, half the sit points occupied at start) runs 2 000 requests; after every request, every seated person stands at their point | `tests/anchoring.rs` |
| AC-b-7 | Posture without bodies: in a world that enables posture but not bodies, sit and stand work, the load check is skipped, and nothing panics | `tests/anchoring.rs` |
| AC-b-8 | Registration: a process whose catalog lacks posture refuses to install posture, naming `register_anchorings` and `ARC-62` | `systems/installed/tests/installed.rs` (the existing guard) and one `#[should_panic]` test |

**Planted mutations** (each must turn a named test red; evidence in §11): X-S34-3 (remove posture from
the catalog line → AC-b-1, AC-b-6 red); X-S34-10 (skip the check in posture's reduction → AC-b-4's
"bodies before posture" case red); X-b-1 (anchoring ignored in `has_body` → AC-b-3 red).

## 6. Commit plan

Each commit has **Implementation**, **Validation**, **Review**, each `[ ]` until done with evidence.

### C0 — Design (this file), docs only
- [ ] Implementation: this file.
- [ ] Validation: `python3 -I scripts/check_decision_ids.py`; `python3 -I scripts/check_doc_headings.py`.
- [ ] Review: scope equals the step's S34-b row, apart from Q-S34b-1.

### C1 — The catalog in bodies, inert
- [ ] Implementation: `systems/bodies/src/anchoring.rs` (trait, catalog, `register_anchorings`,
  `require_anchoring`, `registered_anchorings`, `anchored_in(world, standing) -> Vec<bool>`,
  `fixed_points_fit`); `Here` gains `anchored`; `nudge` and `has_body` honour it; `place-shaped`
  reduction asks `fixed_points`. Exported from `lib.rs`.
- [ ] Validation: `cargo test -p mineworld-bodies` (through `with-slot.sh`) passes unchanged (I-b-1);
  clippy `-D warnings`.
- [ ] Review: no world name, no float, no hash map; the empty catalog path allocates nothing new per
  arrival beyond one empty `Vec`.

### C2 — Posture implements it; registration; version
- [ ] Implementation: `systems/posture/src/anchoring.rs` (`impl Anchoring for PostureSystem`), the
  `fixed_points_fit` call in the `place-sit-points` reduction, `require_anchoring` in `install`, posture
  `VERSION` 2, `mineworld-bodies` dependency; the `extension` line in `systems/installed`.
- [ ] Validation: `cargo test -p mineworld-posture -p mineworld-installed-systems`.
- [ ] Review: dependency direction (I-b-4); install order.

### C3 — Tests and mutations
- [ ] Implementation: `systems/posture/tests/anchoring.rs` (AC-b-1…8), AC-a-9 rewritten (SD-b-8).
- [ ] Validation: the suite green; X-S34-3, X-S34-10, X-b-1 planted one at a time, each red, reverted.
- [ ] Review: each AC names its test; the generated variant is seeded and its seed printed on failure.

### C4 — Docs
- [ ] Implementation: `ARC-147` note (anchoring, SD-b-5, SD-b-7), `ARC-62` note (fifth line), READMEs,
  `MODULE_SPEC.md` §3.1, `python3 -I scripts/check_decision_ids.py --write-index`.
- [ ] Validation: both doc scripts PASS.
- [ ] Review: terms ("sit point", "anchored") consistent with `CORE_CONCEPTS.md`.

## 7. Test ownership

| Failure class | Owner |
| --- | --- |
| Catalog registration and its panics | `systems/installed/tests`, one `should_panic` in bodies |
| Nudge, entry, shove, load check | `systems/posture/tests/anchoring.rs` (needs both packs) |
| Byte identity of worlds without posture | existing bodies suites and baselines (milestone CI) |
| Cross-platform | the milestone CI matrix |
| Clients, LLM | N/A in this PR |

## 8. Is any of this material?

Two points touch frozen text and are put to the operator (§12): the version choice (SD-b-7) and the
check living in two reductions (SD-b-5). Everything else is the frozen step's S34-b row.

## 9. Risks

| Risk | Mitigation |
| --- | --- |
| A crowd blocked by a seated person at a doorway jams the walk planner | Movement already replans after `stopped-short`; the wayfinder (bodies' `route.rs`) could treat anchored people as discs — measured in AC-b-6, added only if the seeded run shows stuck walkers |
| Two order-dependent code paths drift | Both call one bodies function; AC-b-4 runs both orders |

## 10. Execution contract

```text
Repository / worktree:  yuema137/MineWorld · /Users/yuema137/mineworld-worktrees/s34b-anchoring
Branch / base:          mvp0/pr-s34b from origin/main after this design freezes
Authorized (on freeze): implement, validate locally (cast-slots/with-slot.sh), commit, push, open the PR
Validation budget:      crate tests and clippy per commit (speed-first charter); full suites at milestone
Stop condition:         MERGE-READY: PR number and head SHA reported to the coordinator
Merge boundary:         the coordinator merges
Escalate:               any change to presence, movement, kernel or contracts; a bodies VERSION bump
```

## 11. Evidence ledger

(empty until implementation)

## 12. Operator questions (each with a recommended answer)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34b-1** | The step said bodies' version bumps (5 → 6), which re-baselines bodies-yard, Social Café and Market Town although none of them has a seated person. Bump posture's version instead? | **Yes, bump posture (1 → 2), keep bodies at 5.** Only worlds with posture behave differently, and posture's version is in their saves; three worlds stay byte-identical |
| **Q-S34b-2** | The "sit point inside furniture" load check: run it in whichever of the two reductions comes second (order-free), or require `bodies` before `posture` in `systems:`? | **Order-free, one shared bodies function.** No new ordering rule for world authors |
| **Q-S34b-3** | Should the walk planner route around seated people (treat them as fixed discs), or only stop walkers short? | **Stop short now; add routing only if AC-b-6's seeded run shows walkers stuck.** Smallest change first |
