# PR S33-b — Lakeside gets walls: the `bodies` System, a `body:` on every place, one recorded re-baseline

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: freeze with the recommended answers)

```text
Design revision:        revision 1 (2026-10-17), FROZEN 2026-10-18
Author:                 S33-a/b/c PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s33abc, branch docs/pr-designs-s33abc,
                        planning base origin/main @ da80316f. Designed against PR #171 (12d) head dd6f7db5
Approved by / evidence: the step's rulings (Q-S33-1…11) bind; Q-S33-8 (one recorded re-baseline in S33-b,
                        rebased last, after S29-a's) is not re-asked; §14's Q-S33b-1 and Q-S33b-2 (and S33-a's
                        Q-S33a-1) ruled 2026-10-18 by the operator in the primary session, as recommended. The
                        operator also noted: the ticket room grows to at least 4.6 × 4.0 m (F-S33b-5), and S29
                        must change to carry it (§11)
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of 12d, S26-a,
                        S29-a, S30-g, S32-f
Start condition:        (1) Milestone F's checklist handed to the operator (Q-S33-10); (2) 12d merged (#171:
                        OPEN at 2026-10-18); (3) S29-a merged (NOT met at 2026-10-18; S26-a #206 has merged);
                        (4) Q-S33a-1, Q-S33b-1 and Q-S33b-2 ruled (met: 2026-10-18); (5) this design frozen
                        (met: 2026-10-18); (6) the primary session's dispatch, which also orders the
                        re-baseline after every other open Lakeside change (R-S33-9)
Execution contract:     §12
Implementer:            Haiku-class (step-33 §12.1: "Haiku from a commit plan (12d's pattern)"); reviewer the
                        primary session, plus an Opus logic review of C2 (the geometry) and C5 (the re-baseline)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) (APPROVED
2026-10-15) §§2.3 (F-S33-3, F-S33-4), 4.0, 4.6, 4.7, 10 (I-S33-1, I-S33-7, I-S33-9), 12.1 (the S33-b row), 13
(R-S33-9), 14 (Q-S33-8) · **Parent:** [`overall.md`](overall.md) · **Follows the pattern of:**
`step-11-bodies.md` §19 (12d, "towns get walls"; PR #171) · **Consumes:**
[`pr-s29a-town-contract.md`](pr-s29a-town-contract.md) (the halt's three places, as merged) · **Followed by:**
S33-c (the stub's doors at these doorway points), S33-d…f (drawn walls within 50 mm of these solids, X-8) ·
**Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 3, 5, 8, 9, 10, 14, 15); `docs/CORE_CONCEPTS.md` §6.1;
`docs/MODULE_SPEC.md` §4.1 (the `body:` section, as 12d leaves it); `docs/DECISIONS.md` `ARC-36`, `ARC-39`,
`ARC-77`.

**Decision ids.** None (step §15 assigns S33-b no record). Dated notes under `ARC-39` and `ARC-77` (C5), as 12d
did for the towns.

No hosted model is called by this design or by anything it specifies.

---

> **Revision 2 (2026-10-19) — furnishings (`ARC-307`; operator ruling Q-FD-2 in
> [`../mvp1/pr-vf-furnishings-from-data.md`](../mvp1/pr-vf-furnishings-from-data.md) §9: "Lakeside's free-standing
> pieces become furnishings with solids, its table chairs become solids").** A bounded revision applied before
> S33-b starts; scope, invariants and acceptance otherwise unchanged. What changes:
>
> 1. Every free-standing piece of §5.3 is written as a **furnishing** (`body.furnishings`: key, kind, at, facing,
>    `CORE_CONCEPTS.md` §6.5), and its solid(s) name it (`furnishing: <key>`). The place's shell — walls, the
>    oven wall, the fitted counters, the gated stair, the lime tree's trunk, the pavilion's posts, the station's
>    window-counter wall — stays plain solids. The table is §5.3.1.
> 2. The bakery's long table gets **five chairs as solids** on its east side (§5.3.1), superseding S33-f's
>    F-S33f-5 ("chairs are drawn props, not solids").
> 3. Every kind S33-b writes must resolve in both default Presentation Packs: `tools/cli/tests/furnishings.rs`
>    (`every_kind_every_world_names_is_bound_in_each_default_pack`) lists every world and fails otherwise. S33-b
>    adds the 3D binding (`presentation/mineworld-default/3D/furniture/furnishings.yaml`, a library model whose
>    `contact.json` mount is `floor`) and the 2D role (`furnishing.<kind>`) for each new kind, using a
>    `.`-prefix kind where an existing binding fits (`bench.pier` → `bench`); where no model exists yet, it binds
>    the nearest library model and S33-f replaces it. A kind left unbound is drawn as plain boxes and fails that
>    check, so it is not shippable.
> 4. Starts (A-11) and doorway points are re-checked against the added chair solids at genesis (C2's loading
>    check, unchanged).

## 0. Binding inputs, exactly restated

- **Scope (step-33 §12.1, S33-b row).** "`bodies` in `systems:`; `body:` on all Lakeside places at real dimensions
  (the halt's from S29-a); NW-3 walks; one recorded re-baseline of Lakeside's digests and Milestone E
  expectations." Depends on **12d merged**; S29-a.
- **Integration checkpoint.** "`mineworld run worlds/lakeside 30d` seeded: no person in the lake or off the pier;
  save, restart and resume identical; `milestone_e` green with recorded new values."
- **Adversarial.** "a stride off the pier deck is stopped by `bodies`; removing `bodies` from `systems:` restores
  the old digests" (see F-S33b-3 and Q-S33b-1).
- **Step §4.6.** "The server decides where a person can be": the lane, yard, square, shore and pier floors; the
  bakery, boathouse and halt walls and furniture as solids (≤ 64 per place); the pier floor ending at its deck
  edges; the shore floor ending at the water line. "No navmesh"; NW-3's bound (≤ 10 % of walks stopped by
  geometry) on Lakeside's walks.
- **Q-S33-8 (ruled).** One recorded re-baseline in S33-b, rebased last, after S29-a's.
- **I-S33-1.** Lakeside gains the existing `bodies` System and `body:` sections; no kernel, contract, protocol or
  System change.
- **R-LOAD-1.** Locally only the commit's named test files, one at a time (`cargo test -p mineworld-cli --test
  <file>`), plus `cargo test -p mineworld-worldpack` where named. Never `cargo test --workspace`. One long parity
  record, in the foreground, only in C5 (§9 C5 says when it may instead come from CI). Local timings are
  INCONCLUSIVE.

---

## 1. Goal

Make the server, not the scene, decide where a person in Lakeside can stand. Lakeside enables the `bodies`
System that 12d gave the two towns, and every one of its places — the seven village places and the halt's three
— carries a `body:` floor and its solids at stated dimensions, so that a stride into the lake, off the pier deck
or through the bakery's wall is stopped by `bodies` (F-S33-3), and so that the places' floors, translated by the
doorway rule the clients use, never overlap and can be drawn (I-S33-9). Lakeside's history moves once, recorded
(Q-S33-8).

---

## 2. Scope

### 2.1 In scope

1. `worlds/lakeside/world.yaml`: `bodies` in `systems:` (after `schedule`, before `item`: Market Town's
   position, 12d SD-D1), with its comment.
2. `worlds/lakeside/places/{lane,cottages,boathouse,bakery,square,shore,pier,station,station-tickets,
   station-platform}.yaml`: a `body:` section each (§5.3); every `passages:` point moved to 400 mm inside its
   floor edge (§5.2, F-S33b-1); header comments state the drawn size.
3. `tools/cli/tests/lakeside_bodies.rs` (new) and helpers in `tools/cli/tests/lakeside/mod.rs` (§7).
4. The re-baseline: `scripts/baselines.txt` `[world lakeside]` with one `reason S33-b …` line; any Lakeside
   literal in a test that the move shifts (C0 lists them; only Lakeside's).
5. `docs/DECISIONS.md`: dated notes under `ARC-39` (Lakeside has bodies) and `ARC-77` (Lakeside's history moved
   once, S33-b); `docs/MVP_STATUS.md`: the evidence row (12d's pattern, `MVP_STATUS.md:165` on 12d).

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| Any change to the `bodies` crate, its constants, its codes, or any other System | never (I-S33-1) |
| A water concept in `bodies` | not needed: a floor's edge is a wall (F-S33b-4) |
| Drawing anything; the client's disclosed geometry | S33-c (stub), S33-d…f; 12e |
| The halt's place keys, tags, routines, `settlement:` | S29-a |
| Heights (the lane above the water, the deck above the lake) | presentation (F-S33-4); S33-d |
| Other worlds' bodies or baselines | unchanged (AC-b-9) |

### 2.3 Dependencies

**Required before C0:** 12d merged (R 250 mm, `VERSION` 5, the `bodies-doorway` refusal, `town_bodies.rs`'s
helpers; on `main` the crate is still R 300, `VERSION` 4, no doorway check — S33-b on `main` today would move
twice). S29-a merged (the halt's places). Q-S33a-1 ruled (the halt's end and the platform's frame: §5.3's halt
rows follow what S29-a merges). The primary orders this PR's re-baseline after S29-a's and after S30-g's and
S32-f's Lakeside changes if they are open (R-S33-9).

---

## 3. Audit (`origin/main @ da80316f`; 12d at `origin/mvp0/pr-12d-towns @ dd6f7db5`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | The place form: `body: { floor: { min: {x,y}, max: {x,y} }, solids: [ { min, max, height } ] }`, integer mm in the place's frame; unknown fields refused. Each coordinate within ±100 000; each floor side ≥ 2(R+GAP) = 520; ≤ 64 solids; solid height 1…10 000 | 12d `systems/bodies/src/section.rs:1–10, 51–80`; `component.rs:15, 18, 162–168, 182–229`; `geometry.rs:74` |
| A-2 | Constants on 12d: `PERSON_RADIUS` 250 (`geometry.rs:15`), GAP 10 (:22), CLEARANCE 2R − 5 = 495 (:30), `CAPACITY_GRID` 650 (:70); `VERSION` 5 (`system.rs:70`). On `main`: R 300 (`geometry.rs:13`), `VERSION` 4 (`system.rs:67`) | 12d; main |
| A-3 | Genesis checks in order: **capacity** (≥ 4·(population − 1) + 1 points of the 650 mm lattice anchored at floor.min + (R, R), each inside the floor shrunk by R and ≥ R from every solid), outside, in-solid, overlap, then **`doorways_fit`**: every `here` out of a place and every `there` into it ≥ R + GAP = 260 mm from the floor's edge and from every solid | 12d `genesis.rs:67–141, 143–202`; `geometry.rs:284` (`capacity`, `lattice`) |
| A-4 | Codes: `bodies-overlap`, `-outside`, `-in-solid`, `-capacity`, `-doorway` (:38), `-unshaped`, … | 12d `genesis.rs:33–47` |
| A-5 | 12d's authoring rule SD-D3: every doorway point stands **400 mm** in front of its door on each side (F:5984); the 3D client's layout assumes the same: `const DOOR_INSET := 0.40` ("The authoring rule 12d adopted for every doorway (step-11 SD-D3)") | `step-11-bodies.md` on 12d :5984; 16c `clients/3d-spike/scripts/slice/slice_layout.gd:39–44` |
| A-6 | 12d's place example: `worlds/social-cafe/places/apartments.yaml:15–32` (the door "0.4 m into the hall, and on the street 0.4 m in front of the … door"; `body:` with the floor 8 000 × 6 000 and one solid, the stairs, height 3 000) | 12d |
| A-7 | 12d put `bodies` last in Social Café (`world.yaml:51`) and after `schedule`, before `item`, in Market Town (`world.yaml:62–64`) | 12d |
| A-8 | **Every Lakeside doorway point is 200 mm from its edge**: `here` (2000, 200) in cottages, boathouse, bakery; (6000, 200) in square; shore (2000, 9800) on a 10 m-deep place; pier (2000, 19800) on a 20 m place; every `there` at lane y 200 or 5800 on a 6 m lane. Under 12d these are `bodies-doorway` refusals (< 260) | `worlds/lakeside/places/*.yaml` (main) |
| A-9 | Lakeside's stated sizes are comments only: lane ≈ 30 × 6 m, cottages ≈ 14 × 10, boathouse ≈ 10 × 8, bakery ≈ 8 × 6, square ≈ 12 × 12, shore ≈ 15 × 10, pier ≈ 4 × 20 (`*.yaml:1–6`); "No place here has drawn geometry yet; positions are semantic" (`lane.yaml:6`) | main |
| A-10 | The north doors on the lane are 6 m apart (lane x 3 000, 9 000, 15 000, 21 000); the south doors at 12 000 (shore) and 18 000 (pier) | main |
| A-11 | Starts: cottages alba (3000, 2000), bruno (4000, 2500), celia (2500, 3000), elin (3500, 1500), greta (2000, 2000), ines (4500, 2000); boathouse dario (2500, 2000), finn (3000, 2500), hugo (3500, 1500); pier nils (2000, 10000) | `worlds/lakeside/people/*.yaml` |
| A-12 | S29-a rev. 2 (DRAFT) Lakeside halt: `station` here (4000, 400) / lane there (400, 3000), ≈ 8 × 5 m; `station-tickets` here (1500, 400) / station there (6000, 400), "a window bay ≈ 3 × 2 m"; `station-platform` here (0, 3600) / station there (4000, 4600), ≈ 40 × 4 m, "origin on the track edge, mid-length"; "Lakeside's halt has positions but no `body:`" | `pr-s29a-town-contract.md:153, 410–416` |
| A-13 | NW-3 as ruled (M-3 ruling 2): geometry-caused stopped-short strides (`by: None`) ≤ 10 % of strides; measured by `assert_bodies_happen_and_stay_rare` (`geometry * 10 <= strides`), counted in `bodies/mod.rs:580–592`. 12d measured 2.8 % / 2.2 % | 12d `tools/cli/tests/town_bodies.rs:414–434`; F:10718–10719, 7218–7223 |
| A-14 | 12d's save scan (`town_bodies.rs:438–459`) and no-bodies counterfactual (`:473–520`); `bodies::keys_of` (`bodies/mod.rs:956–975`) reads with no pack roots and `bodies::run` (:178–194) passes no `--packs` — Lakeside needs `lakeside::read_world` (`lakeside/mod.rs:198–201`) and `lakeside::run` (:96–106). `rooms_of` (`bodies/mod.rs:89–153`) reads the inline `{ min: { x:, y: }, …, height: }` form as text, in that order | 12d; main |
| A-15 | No Rust test pins a Lakeside digest. `milestone_e.rs` M-5…M-7 (:392–508): 300 days, a 30-day twin, SIGKILL at day 15 and resume, seed 8, replay, all relative; M-2 (:134–195) reads the system lines from the world; M-3's edits (:229–322) do not touch `systems:`; M-9 (:515–550) compares fact types with and without `fishing`. `lakeside/mod.rs` `disabled_copy` (:385–403) removes the fishing lines only | main |
| A-16 | `scripts/baselines.txt` `[world lakeside]` (main :16–25): six keys; the file's rule (:2–5): a PR that changes a world's behaviour updates its section with a `reason` line; regenerated by `ci_parity.py baseline write <long record>`. `record` runs every world (no per-world option); the long profile adds 1 000 days and a 300-day save | main `scripts/ci_parity.py:1–40, 391–447` |
| A-17 | 12d re-baselines Lakeside once already (item names: summaries and `sealed-300s` move, facts and journal do not), and S29-a re-baselines it again (three places, routines, ids +3) | 12d `scripts/baselines.txt`; `pr-s29a-town-contract.md` C4 |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33b-1** | Every Lakeside doorway point is 200 mm from its edge (A-8): under 12d each is a `bodies-doorway` refusal, and the 3D layout's 400 mm rule (A-5) would place every Lakeside wall 0.2 m wrong | Move every point to 400 mm inside its floor's edge (§5.2), 12d's SD-D3. Only the coordinate across the wall changes; positions along the lane stay |
| **F-S33b-2** | **The comment sizes overlap.** The north doors are 6 m apart (A-10), but the places beside them are 8–14 m wide (A-9): under the doorway rule (the wall midway between `here` and `there`, A-5), cottages (lane x 1–15 m), boathouse (7–17), bakery (13–21) and square (15–27) overlap each other, and the shore (10–25) overlaps the pier (16–20). The server does not care (each place is its own frame), but no client can draw overlapping rooms, and I-S33-9 ("walls are the server's") needs drawable walls | Q-S33b-2 (§14): fit each place in its frontage (recommended, §5.3), or lengthen the lane and move the doors. A new check (C3, AC-b-6) holds the translated floors disjoint |
| **F-S33b-3** | The step's adversarial "removing `bodies` from `systems:` restores the old digests" cannot hold literally: F-S33b-1's move changes where every arrival lands, with or without bodies | Q-S33b-1 (§14): restate as "removing `bodies` gives exactly the digests of the same world with the doorway move alone", captured at C1 from a no-bodies copy, and the no-bodies copy's facts differ from the base only in positions |
| **F-S33b-4** | Nothing in `bodies` knows water, and a floor is one axis-aligned rectangle (A-1) | None needed: the shore's floor ends at the water line and the pier's at its deck edges; a floor's edge is a wall (12d: "its edge is its walls"), so a stride into the lake is a `StoppedShort` with `by: None`, as a stride into a wall |
| **F-S33b-5** | **The halt's ticket bay fails capacity.** With 10 people, every shaped place needs 37 lattice points (A-3); a 3 × 2 m bay has 4 × 3 = 12 → `bodies-capacity` | The bay's floor is at least 4 600 × 4 000 mm (7 × 6 = 42 points); its comment and S33-e's window bay follow (R-S33e-b-1). The rule is the System's and is not changed (I-S33-1) |
| **F-S33b-6** | The platform's `here` (0, 3600) lies on x = 0. Read as S29-a states the frame (x along the 40 m, y across the 4 m, origin on the track edge mid-length), the floor is x −20 000…20 000, y 0…4 000 and the point is 400 mm from the back edge: it fits. If Q-S33a-1 gives Lakeside a north–south platform (x across, y along), S29-a writes the points for that frame | §5.3 follows the frame S29-a merges; C0 records it |
| **F-S33b-7** | Lakeside's history moves three times on the way (12d, S29-a, S33-b; A-17) | Q-S33-8 ruled one re-baseline **in S33-b**; the other two are their own PRs'. This PR rebases last and records only its own move |

### 3.3 What does not exist

No Lakeside `body:`; no Lakeside bodies test; no test that translated floors are disjoint in any world.

---

## 4. Reuse

Nothing new is adopted or built: the `bodies` System (12d), its route planner, the `town_bodies.rs` helpers, the
`lakeside` test module and `ci_parity.py` are reused unchanged.

---

## 5. Design

### 5.1 Frames and the doorway rule

Each place's frame is its own (x east, y north, mm; CORE_CONCEPTS §6.1). A passage `here` (in the place) and
`there` (on the lane) each stand 400 mm inside their floors; the wall between them is midway, so in the lane's
frame a north place's south wall lies on the lane's north edge (y 6 000) and a south place's north edge on the
lane's south edge (y 0). A place's floor in the lane's frame is `floor + (there − here) ± 800 mm across the wall`.

### 5.2 Doorway points after the move (only the cross-wall coordinate changes)

| Place | `here` before → after | `there` (lane) before → after |
| --- | --- | --- |
| cottages | (2000, 200) → (2000, 400) | (3000, 5800) → (3000, 5600) |
| boathouse | (2000, 200) → (2000, 400) | (9000, 5800) → (9000, 5600) |
| bakery | (2000, 200) → (2000, 400) | (15000, 5800) → (15000, 5600) |
| square | (6000, 200) → (6000, 400) | (21000, 5800) → (21000, 5600) |
| shore | (2000, 9800) → (2000, 9600) | (12000, 200) → (12000, 400) |
| pier | (2000, 19800) → (2000, 19600) | (18000, 200) → (18000, 400) |
| halt | as S29-a merges them (its points are already at 400 mm, A-12); checked by genesis | — |

### 5.3 The floors and solids (Q-S33b-2 option A, recommended; mm, each place's frame)

Each north place keeps its door and takes the frontage between its neighbours' doors' midpoints (6 m; the
square, last before the halt, takes 12 m), and its depth northward from the comments. Party walls lie on the
midpoints (lane x 6 000, 12 000, 18 000).

| Place | Floor | Solids (≤ 64) | Lane-frame footprint | Capacity (≥ 37) |
| --- | --- | --- | --- | --- |
| `lane` | (0, 0)–(30 000, 6 000) | none (the lane's edges are the houses' fronts and, on the south, the railing over the shore) | itself | 46 × 9 |
| `cottages` (the yard) | (−1 000, 0)–(5 000, 14 000) | the pump (1 700, 6 700)–(2 300, 7 300) h 1 200 | x 0–6 000, y 6 000–20 000 | ≥ 180 |
| `boathouse` (the boat shed) | (−1 000, 0)–(5 000, 12 000) | the boat on its slip (−600, 3 200)–(1 200, 11 000) h 1 100 (clear of the three starts, A-11, all at y ≤ 2 500); the workbench (3 900, 6 000)–(4 700, 9 000) h 900; the stair, gated (3 000, 10 400)–(5 000, 12 000) h 3 000 | x 6 000–12 000, y 6 000–18 000 | ≥ 100 |
| `bakery` | (−1 000, 0)–(5 000, 8 000) | the oven wall (−1 000, 7 000)–(2 500, 8 000) h 2 200; the counter (2 500, 4 600)–(4 600, 5 200) h 950; the bread racks (4 400, 5 600)–(5 000, 7 800) h 1 900; **the long table by the window** (−700, 1 400)–(800, 4 600) h 750 | x 12 000–18 000, y 6 000–14 000 | ≥ 60 |
| `square` | (3 000, 0)–(15 000, 12 000) | the well (8 400, 5 400)–(9 600, 6 600) h 900; the lime tree's trunk (11 800, 8 800)–(12 200, 9 200) h 4 000; two benches under it (10 900, 9 600)–(12 700, 10 000) and (12 600, 8 100)–(13 000, 9 900), h 450 | x 18 000–30 000, y 6 000–18 000 | ≥ 300 |
| `shore` | (−10 000, 0)–(5 600, 10 000); the south edge (y 0) is the water line | three boulders, h 800, C2 places them ≥ 1 m from the door and from the path to the water | x 0–15 600, y −10 000–0 | ≥ 300 |
| `pier` (the deck) | (0, 0)–(4 000, 20 000); its edges are the deck's | the bench at the head (400, 300)–(3 600, 750) h 450; the pavilion's four posts, 200 × 200, at (200, 200), (3 600, 200), (200, 3 600), (3 600, 3 600) min corners, h 2 600 | x 16 000–20 000, y −20 000–0 | 6 × 31 |
| `station` (waiting room) | ≈ 8 000 × 5 000 in S29-a's frame | the bench row along the far wall, h 450; the window counter's wall to the ticket bay, h 1 100 | per Q-S33a-1 | ≥ 70 |
| `station-tickets` (the bay) | ≥ 4 600 × 4 000 (F-S33b-5) | the counter, h 1 100 | per S29-a | ≥ 42 |
| `station-platform` | 40 000 × 4 000 in S29-a's frame (F-S33b-6) | the bench(es) and the board's post, h 450 / 2 400; the buffer-stop end is the floor's end | per Q-S33a-1 | ≥ 180 |

- Every solid is a real object the scene will draw at the same place (X-8's check in S33-d…f compares drawn walls
  with these). Dimensions are approximate design values within VISUAL_SLICE's ranges (bench seat 450 mm,
  §3.4; counter 950–1 100 mm) and are marked so in each place's comment (I-S33-7).
- Starts (A-11) all lie ≥ 250 mm from every edge and solid in these floors (C2 checks by loading).
- If Q-S33b-2 rules option B, C2's table is rewritten from the ruled lane length before any file is edited.

### 5.3.1 Furnishings (revision 2, `ARC-307`)

`at` is the centre of the piece's drawn footprint, inside its solids; `facing` is millidegrees clockwise from
north (0 north, 90 000 east). Keys are unique per place. Solids are §5.3's, now naming their furnishing.

| Place | Furnishing (key → kind) | Solid(s) naming it | at | facing |
| --- | --- | --- | --- | --- |
| `cottages` | `pump` → `pump.hand` | (1 700, 6 700)–(2 300, 7 300) h 1 200 | (2 000, 7 000) | 180 000 |
| `boathouse` | `boat` → `boat.rowing` | (−600, 3 200)–(1 200, 11 000) h 1 100 | (300, 7 100) | 0 |
| `boathouse` | `workbench` → `table.work` | (3 900, 6 000)–(4 700, 9 000) h 900 | (4 300, 7 500) | 270 000 |
| `bakery` | `bread-racks` → `shelves.bread` | (4 400, 5 600)–(5 000, 7 800) h 1 900 | (4 700, 6 700) | 270 000 |
| `bakery` | `long-table` → `table.long` | (−700, 1 400)–(800, 4 600) h 750 | (50, 3 000) | 90 000 |
| `bakery` | `long-table.chair-1` … `-5` → `chair.dining` | 5 boxes (900, y)–(1 400, y + 470) h 900, y = 1 565, 2 165, 2 765, 3 365, 3 965 | each box's centre | 270 000 (west, to the table) |
| `square` | `well` → `well` | (8 400, 5 400)–(9 600, 6 600) h 900 | (9 000, 6 000) | 0 |
| `square` | `bench.west`, `bench.east` → `bench.park` | §5.3's two benches, one each | each box's centre | toward the trunk |
| `shore` | `boulder.1` … `.3` → `boulder` | §5.3's three boulders (C2 places them) | each box's centre | 0 |
| `pier` | `bench.head` → `bench.pier` | (400, 300)–(3 600, 750) h 450 | (2 000, 525) | 180 000 |
| `station`, `station-platform` | each bench → `bench.platform` | §5.3's benches | each box's centre | away from its wall / track |

The chairs stand 100 mm off the table's east edge (x 800) and 600 mm apart centre to centre, 500 × 470 mm each
(a dining chair, the café's `dining_chair_02` collider rule within 50 mm). They leave the doorway point
(2 000, 400) more than 260 mm clear. S33-f's sit points move behind them (S33-f revision 2).

### 5.4 The re-baseline (C5)

One long record (`ci_parity.py record --binary target/release/mineworld --profile long`) on the PR's final
executable head; `baseline check` against the committed file fails **only** in `[world lakeside]` (every other
world's keys equal: AC-b-9); `baseline write`; one line:
`reason S33-b #<PR>: Lakeside gets bodies (step-33 F-S33-3) and its doorways move to 400 mm (12d SD-D3); every key moves; recorded on <sha>`.
Before and after values go in §15 and in `MVP_STATUS.md`, never in code (12d TD-1).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-b-1 (I-S33-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `clients/`, `presentation/` changes | C6 diff scan |
| I-b-2 (I-S33-9) | No person's recorded position lies outside its place's floor shrunk by R, in any run | AC-b-2 |
| I-b-3 | Every place's floor, translated to the lane's frame by §5.1, is disjoint from every other's | AC-b-6 |
| I-b-4 | Lakeside's history moves once, in C5, recorded; no other world's baselines move | AC-b-8, AC-b-9 |
| I-b-5 | Only the cross-wall coordinate of a passage point changes (§5.2) | C1 review (diff) |
| I-b-6 | Determinism: twin, kill-and-resume and replay identical (M-5…M-7) | AC-b-4 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Loader / genesis | doorway insets, capacity, starts inside floors, ≤ 64 solids — the System's own refusals, not re-tested | `mineworld validate` / `run` at genesis |
| Integration (new) `lakeside_bodies.rs` | (1) `the_lake_and_the_deck_edges_are_walls`: Nils (on the pier) and a person on the shore each commanded past an edge (east of the deck, south of the water line) → `StoppedShort` with `by: None`, the end position inside the floor; (2) `nobody_stands_in_the_lake_in_thirty_days`: seed 7, 30 days, `--save`, every recorded position inside its floor shrunk by R (12d's save scan, with `lakeside::read_world`); (3) `bodies_happen_and_stay_rare_in_lakeside`: NW-3 over those 30 days; every routine's place entered each day by its person ≥ 90 % as often as in the no-bodies copy; (4) `the_village_floors_do_not_overlap`: I-b-3 from the world's place files; (5) `without_bodies_only_the_doorways_moved`: the counterfactual as Q-S33b-1 rules it | `tools/cli/tests/lakeside_bodies.rs` (CI `core`) |
| Existing | `milestone_e.rs` M-1…M-9 and `lakeside/*` stay green with bodies (twin, kill/resume, replay, seed 8, fishing-disabled fact types) | CI `core` |
| Nightly | the long record equals `baselines.txt` | nightly `parity` |
| Gate 1 / 2 | NOT REQUIRED | — |

The oracles are the world's own files and the save, never a value the System reports about itself.

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-b-1 | `mineworld validate worlds/lakeside --packs entities --packs presentation/mineworld-default` exits 0 and `mineworld run … --headless --seed 7 --days 1` starts (genesis accepts every place) | §12.1 |
| AC-b-2 | 30 days, seed 7, saved: no recorded position outside its floor shrunk by 250 mm — nobody in the lake, off the pier or inside a solid | §12.1 checkpoint |
| AC-b-3 | A stride past the deck's edge and one past the shore's water line each end `StoppedShort` with `by: None` inside the floor | §12.1 adversarial |
| AC-b-4 | `milestone_e` M-5…M-7 (twin, SIGKILL at day 15 and resume, replay, seed 8) pass with bodies | §12.1 checkpoint |
| AC-b-5 | NW-3: geometry stops ≤ 10 % of strides over AC-b-2's run; every routine place entered ≥ 90 % of the no-bodies copy's entries; the percentages recorded | §4.6 |
| AC-b-6 | The ten floors, translated to the lane's frame (§5.1), are pairwise disjoint | F-S33b-2 |
| AC-b-7 | The counterfactual as ruled (Q-S33b-1): removing `bodies` from `systems:` gives the C1 "doorways only" digests | §12.1 adversarial |
| AC-b-8 | `scripts/baselines.txt` `[world lakeside]` re-recorded once with one `reason S33-b` line; before/after in §15 and `MVP_STATUS.md` | Q-S33-8 |
| AC-b-9 | `baseline check` of the C5 record against the base file fails in `[world lakeside]` only | I-b-4 |
| AC-b-10 | The diff touches only §2.1's paths | I-b-1 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; inapplicable → `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s33b`. Never `cd` elsewhere. Steps 1–10 of
[`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md) §9.0 apply, with these checks: `cargo fmt --all
--check`; `cargo clippy -p mineworld-cli --tests -- -D warnings`; tests one file at a time, `cargo test -p
mineworld-cli --test <file>` (never `--workspace`: R-LOAD-1); doc scripts.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s33b.md` (new).
- Steps:
  1. Record the base. Confirm 12d merged (A-2's 12d values on `main`; `bodies-doorway` exists), S29-a merged (the
     halt's three places and their passages; record them and the platform's frame), Q-S33a-1/Q-S33b-1/Q-S33b-2
     rulings. Any missing → material stop.
  2. Record which of S30-g, S32-f have merged or are open (R-S33-9) and the primary's ordering.
  3. Re-read A-1…A-17 on the base; record moved lines (bounded) or changed statements (stop).
  4. List every Lakeside literal in tests that §5.2's move could shift (`git grep -n "5800\|19800\|9800" -- tools/
     worldpack/ tests/`, restricted to Lakeside's files); expected: none outside `worlds/lakeside`.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: start conditions with evidence.
- **Commit message:** `docs(plan): S33-b — open the ledger at the base`

#### C1 — The doorway move alone; the counterfactual capture

- **Files:** the seven village place files (`passages:` only, §5.2), `tools/cli/tests/lakeside_bodies.rs` (test (5)
  only, ignored-free), `tools/cli/tests/lakeside/mod.rs` (a `summary_digest(world_dir, days)` helper if none
  exists), this file.
- Steps:
  1. §5.2's points; nothing else in the files.
  2. Capture the "doorways only" digest: the 30-day, seed-7 summary digest of `worlds/lakeside` **without** bodies
     at this commit (the summary lines `ci_parity.py` keeps, hashed the same way); write it into test (5) as the
     pinned expectation with a comment naming this commit (the expectation is evidence from before C2's bodies,
     not from the code under test: test rules §25). Any later Lakeside change the primary orders before C5
     (R-S33-9) re-captures it on the rebased branch at C1's state, recorded.
- Validation: `cargo test -p mineworld-cli --test milestone_e` (green: positions moved, nothing relative breaks);
  `--test lakeside_bodies` (test (5) green).
- [ ] Implementation. [ ] Validation. [ ] Review: the diff changes only cross-wall coordinates (I-b-5).
- **Commit message:** `chore(worlds): Lakeside's doorways stand 400 mm inside their doors (12d SD-D3; S33-b)`

#### C2 — Bodies: `systems:` and the ten `body:` sections

- **Files:** `worlds/lakeside/world.yaml`, the ten place files, this file.
- Steps:
  1. `bodies` after `schedule` (A-7, Market Town's position), with a comment citing `ARC-39` and step-33 F-S33-3.
  2. Each `body:` per §5.3, in the inline form `rooms_of` reads (A-14: `{ min: { x:, y: }, max: { x:, y: },
     height: }`), with one comment line per solid naming the object; header comments give the drawn size.
  3. `cargo build -p mineworld-cli`; AC-b-1.
- Validation: AC-b-1; `--test milestone_e`; `--test lakeside_bodies` (test (5) still pins the C1 digest with
  `bodies` removed from a copy — `lakeside::disabled_copy`'s pattern extended by a `without_bodies` copy that
  removes the system line only and keeps the `body:` sections, which the System alone reads).
  Mutations **M-b-1** (bakery `here` back to (2000, 200)) → genesis `bodies-doorway` names bakery; **M-b-2**
  (`station-tickets` floor 3 000 × 2 000) → `bodies-capacity` names it.
- [ ] Implementation. [ ] Validation. [ ] Review (**Opus**): every solid is an object S33-d…f will draw; the
  long table is by the window (the lane side, `bakery.yaml:11`); the stair is drawn and gated (step §4.5).
- **Commit message:** `feat(worlds): Lakeside gets walls — bodies and a body on every place (S33-b)`

#### C3 — The lake, the deck and the floors (tests (1)–(4))

- **Files:** `tools/cli/tests/lakeside_bodies.rs`, `tools/cli/tests/lakeside/mod.rs`, this file.
- Steps: tests (1)–(4) of §7, using `town_bodies.rs`'s helpers through `lakeside::read_world` and
  `lakeside::run` (A-14); (4) computes §5.1's translation from the files' `passages:` and `body:` (it reads the
  YAML with the worldpack loader, not by text).
- Validation: `--test lakeside_bodies` (record wall time; INCONCLUSIVE locally). Mutations **M-b-3** (pier
  floor max x 6 000) → test (1) red; **M-b-4** (bakery floor min x −3 000) → test (4) red naming bakery and
  boathouse; **M-b-5** (`bodies` removed from `systems:`) → test (1) red.
- [ ] Implementation. [ ] Validation. [ ] Review: no test pins a value the System computes; (4) would catch a
  half-metre overlap.
- **Commit message:** `test(cli): Lakeside's lake, deck and floors are walls, and nobody leaves them (S33-b)`

#### C4 — NW-3 on Lakeside

- **Files:** `tools/cli/tests/lakeside_bodies.rs`, this file.
- Steps: test (3) (AC-b-5); record the geometry-stop share and the entry ratio.
- Validation: the file. If NW-3 fails: 12d's remedy ladder (step-11 §19.4) applies to **this PR's geometry only**
  (widen a passage-side clearance, move a solid); a fix needing the System → material stop.
- [ ] Implementation. [ ] Validation. [ ] Review: person-caused stops printed, not bounded (M-3 ruling 2).
- **Commit message:** `test(cli): NW-3 holds on Lakeside's walks (S33-b)`

#### C5 — The recorded re-baseline

- **Files:** `scripts/baselines.txt`, `docs/DECISIONS.md` (notes under `ARC-39`, `ARC-77`), `docs/MVP_STATUS.md`,
  this file.
- Precondition: the branch is rebased on every Lakeside change the primary ordered before it (R-S33-9); C0–C4
  green on the rebased head.
- Steps: §5.4. The long record is made once, locally, in the foreground, only after the coordinator confirms
  the machine is free (R-LOAD-1); otherwise the nightly is dispatched on the branch and its `parity-long`
  artifact is used (recorded as the exception, test rules §10).
- Validation: `python3 scripts/ci_parity.py baseline check <record> scripts/baselines.txt` before (fails in
  Lakeside only: AC-b-9) and after `write` (PASS).
- [ ] Implementation. [ ] Validation. [ ] Review (**Opus**): the before/after table; every key moved for the
  stated reason; no other world moved.
- **Commit message:** `chore(baselines): Lakeside's history moves once — bodies (S33-b)`

#### C6 — Close-out

- **Files:** this file (§15), `handoff-s33b.md`.
- Steps: AC-b-10's diff scan; push `mvp0/pr-s33b`; `gh pr create`; CI on the exact head; repair to green. A head
  change after C5 that alters behaviour repeats C5.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S33-b close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S33c-b-1 (on S33-c).** The Lakeside stub's doors stand at §5.2's points and its boxes at §5.3's lane-frame
  footprints; whichever of S33-b and S33-c merges second updates the stub's one door table.
- **R-S33e-b-1 (on S33-e).** The ticket bay is ≥ 4.6 × 4.0 m (F-S33b-5); the pavilion's posts and the bench stand
  at §5.3's solids.
- **R-S33f-b-1 (on S33-f).** Sit points on the pier bench, the square's benches and the halt's benches lie at
  these solids' front edges.
- **R-S33d-b-1 (on S33-d).** Drawn walls within 50 mm of these floors' edges and solids (X-8).

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §12.1 S33-b row, adversarial | "removing `bodies` … restores the old digests" → as Q-S33b-1 rules (F-S33b-3) |
| `step-33-lakeside-3d.md` §4.0 plan | the village's frontages per Q-S33b-2 |
| `step-29-station-landmark.md` §4.3; `pr-s29a-town-contract.md` | the ticket bay ≥ 4.6 × 4.0 m (F-S33b-5) |
| `overall.md` | S33-b frozen / merged; Lakeside's history moved (S33-b) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-b — Lakeside bodies, one recorded re-baseline
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33b-lakeside-bodies.md
RELATED / BINDING DOCS:  step-33-lakeside-3d.md (§§2.3, 4.0, 4.6, 4.7, 10, 12, 13, 14); step-11-bodies.md §19
                         (12d) and its rulings; pr-s29a-town-contract.md (as merged); docs/MODULE_SPEC.md §4.1;
                         docs/CORE_CONCEPTS.md §6.1; docs/DECISIONS.md ARC-36, ARC-39, ARC-77; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s33b, its own, held by one session
BRANCH:                  mvp0/pr-s33b, created from origin/main
START CONDITION:         Milestone F's checklist handed over; 12d and S29-a merged (neither met at
                         2026-10-18); Q-S33a-1, Q-S33b-1, Q-S33b-2 ruled (met: 2026-10-18); this design frozen
                         (met: 2026-10-18); the primary's dispatch and re-baseline order
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–5
FROZEN INVARIANTS:       step-33 I-S33-1, I-S33-7, I-S33-9; §6 I-b-1 … I-b-6; the bodies System unchanged;
                         Q-S33-8 as ruled
SEQUENCE:                C0 … C6 (§9)
ALLOWED COMMANDS:        cargo build -p mineworld-cli [--release]; cargo test -p mineworld-cli --test <file>;
                         cargo fmt; cargo clippy -p mineworld-cli --tests; target/*/mineworld validate / run;
                         python3 scripts/ci_parity.py (C5 only); python3 scripts/*; git; gh (never merge);
                         mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace;
                         a second long record without a reason; edits under systems/, kernel/, contracts/,
                         server/, clients/, presentation/; another world's files or baselines
MATERIAL STOPS:          a start condition missing; a needed change to the bodies System or its constants;
                         NW-3 unattainable by this PR's geometry; another world's baselines move; the halt's
                         merged frame contradicts §5.3 in a way Q-S33a-1's ruling does not cover; a test
                         outside Lakeside's needs a literal edit
PLATFORMS:               Linux, macOS, Windows (CI core; nightly parity)
VALIDATION BUDGET:       targeted test files locally, one at a time; one long parity record (C5); CI: the PR's
                         runs. Gate 1/2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s33b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                  (source: the operator's freeze of this design, 2026-10-18)
                         semantic commits: authorized                   (working rules §14)
                         branch push: authorized                        (working rules §21)
                         PR creation / update: authorized               (working rules §21)
                         CI repair to review readiness: authorized      (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — only with the operator's explicit authorization (§22)
POST-MERGE SYNC OWNER:   primary: step-33, step-29, overall.md, MVP_STATUS; implementation session: this ledger
STOP CONDITION:          S33-b READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-b-1 | Six-metre frontages make the bakery and the boathouse narrow (Q-S33b-2 A) | realistic for a lakeside terrace; depth carries the room; option B is the alternative the operator may choose |
| R-b-2 | The 300-day Milestone E run slows with bodies | 12d's cost bound (TD-12a) as evidence only; recorded from CI, not local (R-LOAD-1) |
| R-b-3 | NW-3 fails in the narrow bakery or on the 4 m deck | 12d's remedy ladder on this geometry (C4); the System is never changed |
| R-b-4 | The re-baseline collides with S29-a's, S30-g's or S32-f's (R-S33-9) | rebased last; C5 repeated on any behavioural head change |
| R-b-5 | The halt merges in another frame than §5.3 assumes | C0 records it; the halt rows follow it; a contradiction is a stop |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

Q-S33a-1 (the halt's end and the platform's frame) is asked in [`pr-s33a-setting-far-field.md`](pr-s33a-setting-far-field.md)
§14 and binds this PR's halt rows.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33b-1 [operator — the step's adversarial criterion]** | The step says "removing `bodies` from `systems:` restores the old digests". Bodies require every doorway point 260 mm or more inside its edge (400 mm by 12d's rule, which the 3D client also assumes); Lakeside's are at 200 mm, so the points must move, and that alone changes the digests with or without bodies (F-S33b-3). Restate the criterion? | **Yes:** "removing `bodies` from `systems:` gives exactly the digests of the same world with only the doorway move", captured at C1 from that world before bodies are added and pinned in a test. It keeps the criterion's purpose — bodies are the only behavioural change beyond a forced 200 mm move — and stays one recorded re-baseline (Q-S33-8). **Ruled 2026-10-18 (operator): yes, as recommended — the criterion becomes "removing `bodies` gives exactly the world after only moving the door points".** |
| **Q-S33b-2 [operator — the village's plan]** | The north doors are 6 m apart, but the places' stated sizes are 8–14 m wide, so the cottages, boathouse, bakery and square overlap once drawn, and the shore overlaps the pier (F-S33b-2). (A) Keep every door where it is and give each place its 6 m frontage (the square 12 m), with depth to the north: cottage yard 6 × 14 m, boat shed 6 × 12, bakery 6 × 8, square 12 × 12, shore 15.6 × 10; or (B) lengthen the lane to about 50 m and move the doors so each place keeps its stated width? | **(A).** A lakeside terrace of 6 m frontages is realistic, the walk lengths, the 2D layout and `reach_m` stay as they are, and only the cross-wall doorway coordinate moves. (B) changes every walk and the village's proportions for widths that were never measured. **Ruled 2026-10-18 (operator): (A), as recommended — door positions kept, 6 m frontages, depth to the north.** |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C6 | — | [ ] | [ ] | [ ] |

### C0 — base and start conditions (2026-10-20)

- Base: `origin/main @ 97d43fe3` (after S33-c #381 and S30-f #373). Implementer session: worktree
  `impl-s33b`, branch `mvp0/pr-s33b`, dispatched by the lane lead (SPEED FIRST charter: no independent
  reviewer, mutations optional, no full test runs; merges by the coordinator).
- 12d merged: `PERSON_RADIUS` 250, `CAPACITY_GRID` 650, bodies `VERSION` 5, `bodies-doorway` and
  `bodies-capacity` in `genesis.rs`. Furnishings (`ARC-307`, #343) merged: `body.furnishings`,
  `Solid.furnishing`, `tools/cli/tests/furnishings.rs`.
- S29-a merged (#348). The halt as merged, with the **north–south platform frame** (Q-S33a-1):
  `station` floor (0, 0)–(8 000, 5 000), west door here (400, 2 500) / lane there (29 600, 3 000);
  `station-tickets` 4 600 × 4 600, here (2 600, 400) / station there (6 000, 4 600);
  `station-platform` floor x −4 000…0, y −20 000…20 000, here (−3 600, −16 000) / station there
  (7 600, 2 500). Every halt point already stands 400 mm inside its edge (no move). §5.3's halt rows
  follow this frame (F-S33b-6 resolved by the merged frame).
- Q-S33a-1, Q-S33b-1, Q-S33b-2 ruled 2026-10-18 (§14); Q-FD-2 ruled 2026-10-19 (revision 2).
- Other Lakeside changes: S30-g, S32-f not open; S29-a2 #375 (station re-records, baselines) OPEN;
  S33-c #381 MERGED (its stub's door table holds §5.2's "before" points, R-S33c-b-1: listed in the PR
  body for the lead). Re-baseline recorded on this branch's head regardless (dispatch).
- `git grep "5800\|19800\|9800" -- tools/ worldpack/ tests/`: no Lakeside literal outside
  `worlds/lakeside`.

### Evidence

(none yet; C5's before/after table goes here)

### Mutations

(none yet; M-b-1 … M-b-5 as §9 lists them)

### Deviations and findings

F-S33b-1 … F-S33b-7 (§3.2), recorded at design time.
