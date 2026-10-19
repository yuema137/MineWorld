# PR S33-f — Lakeside's interiors and its sit points, drawn the S34 way

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00)

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874. Read-only:
                        step-34-posture.md (FROZEN, #264), S34-a's frozen PR design in the S34 lane's worktree
                        (/Users/yuema137/mineworld-worktrees/s34a-posture, pr-s34a-posture-pack.md)
Approved by / evidence: the operator, 2026-10-19, in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00
                        §14 is open
Implementation base:    origin/main at C0
Start condition:        S33-d merged; S33-e2 merged (the halt's benches) or the halt rows deferred (§2.3);
                        S34-a merged (the sit-point shape and validation); the bodies of S26-c1 / S32-c merged for
                        the `--seat` probe (each missing body is reported, not faked); §14 ruled; this design
                        frozen; the lane lead's dispatch. NONE holds at 2026-10-18
Execution contract:     §12
Implementer:            Haiku-class from this commit plan; Opus frame review (step-33 §12.1)
Lifecycle:              DESIGN FROZEN 2026-10-19
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §§1 (outcome
4), 4.5, 9.3 (`--seat`), 9.4 (X-4), 10, 11 (AC-S33-3), 12.1 (the S33-f row), 14 (Q-S33-5, Q-S33-6), 15 (`ARC-145`)
· **Binding sibling step:** [`step-34-posture.md`](step-34-posture.md) (FROZEN 2026-10-18) §§5.1, 8.1, 8.2, 14,
16 (Q-S34-3, Q-S34-9) · **Consumes:** [`pr-s33b-lakeside-bodies.md`](pr-s33b-lakeside-bodies.md) §5.3 (the
solids), [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md) (the shells),
[`pr-s33e-pier-halt.md`](pr-s33e-pier-halt.md) (the pier bench, the halt's benches) · **Followed by:** S34-c (sit
points in world data), S34-e (the shared override), S34-f (3D sit/stand) · **Binding parents:** `CLAUDE.md` §§1,
2, 4 (items 1, 3, 14, 15); `docs/VISUAL_SLICE.md` §§3.4, 6.2.

**Decision ids.** `ARC-145` (step-assigned: sit points as presentation data and the shared rule), **rewritten**
to S34's shape (§5.1), written by C1. `ARC-172` is reserved to this PR for Q-S33f-1's outcome (who writes
Lakeside's `sit_points:` into world data), written by C1. No other id.

---

## 0. Binding inputs, exactly restated

- **Scope (step §12.1, S33-f row).** "bakery and boathouse-shed interiors; sit points in all seats; the shared sit
  rule in `clients/shared/`; `--seat` probe; threshold light at the bakery door". Adversarial: "X-4; two clients
  agree on a contested seat". Checkpoint: "AC-S33-3 on the bodies available; interiors walkable, server place
  changes".
- **Interiors (Q-S33-5, ruled).** "interiors only where the world has an indoor place (bakery, boathouse shed,
  halt); cottage doors closed."
- **The step's sit rule (step §4.5, SD-S33-7).** A person is drawn seated at sit point s iff within 0.40 m of the
  seat's front edge, still ≥ 2 s, not already drawn occupied (ties by entity id). "A future `posture` System that
  discloses sitting overrides the rule."
- **S34 (FROZEN 2026-10-18), what changes for S33-f.**
  - SD-S34-1…4: a **sit point is semantic place data** owned by `posture`: `{ key, at (mm, on free floor at the
    seat's front edge), facing (millidegrees), group? }`, authored in a place file's `sit_points:`; two points
    ≥ 510 mm apart; the default 600 mm per person on a bench.
  - SD-S34-3 / Q-S34-3 (ruled): the server holds the person on the floor in front of the seat; "the **pelvis over
    the seat** is presentation: the depiction places it at `at` moved back along `facing` by the depiction's
    `seat_depth_m` and up to its `seat_height_m`".
  - §8.1 SD-S34-8: one shared decision — where a place discloses `sit-points`, a person is drawn seated iff their
    disclosed `posture-state` is `Seated { point }`; elsewhere "S33's SD-S33-7 rule, unchanged".
  - §8.2: presentation holds **depictions** bound by `{place tag, point key}` with `seat_height_m` and
    `seat_depth_m` (defaults 0.45, 0.45); a drawn-vs-disclosed check fails beyond 150 mm.
  - Q-S34-9 (ruled): "S34-a before S33-f. Then S33-f authors Lakeside's points once as world data and only their
    depictions in presentation; S33's rule stays as the fallback for worlds without posture."
  - §14.2: S34-c "sit points in every world … Lakeside; `posture` enabled in each `world.yaml`; … re-baselines
    recorded".
- **Q-S33-8 (ruled).** One recorded re-baseline in S33-b.
- **Realistic defaults (operator).** Every seat height, depth and table height cites a source.

---

## 1. Goal

A player can walk into the bakery and the boat shed, see rooms fitted out at hand scale, and sit — on the pier
bench, under the lime tree, at the bakery's long table, in the halt — with their body resting on the seat. Every
seat a person can use is one sit point, named once, stored once as world data, and drawn by both clients through
the same shared decision. Until the world discloses sitting, the step's fallback rule draws it.

---

## 2. Scope

### 2.1 In scope

1. `clients/3d-spike/scripts/worlds/lakeside/interiors.gd` (`LakesideInteriors`): the bakery and the boat shed
   (§5.3); the bakery and shed doors drawn open (the server already lets people through).
2. `presentation/mineworld-default/3D/sit_points/lakeside.yaml` (new): the **depictions** of Lakeside's sit
   points (S34 §8.2): `{ place (tag), point (key), seat_height_m, seat_depth_m, prop }` — and, under Q-S33f-1's
   option A only, the fallback `at` / `facing` copied from §5.2's table, marked for deletion by S34-c.
3. `clients/shared/sitting/sit_rule.gd` (new, `MineWorldSitRule`): the shared decision of S34 §8.1 — the
   disclosed-posture branch and the step's fallback branch (SD-S33-7) — used by both clients. If S34-e has landed
   the module first, this PR uses it unchanged (Q-S34-9's "or S34-e creates the module").
4. `scripts/worlds/lakeside/depiction.gd`: `sit_points()` returns the depictions; the bakery and shed entries
   drawn as ROOM.
5. `clients/3d-spike/scripts/slice/slice_probe_world.gd`: the `--seat` probe (§5.5).
6. `clients/2d`: the 2D client calls `MineWorldSitRule` for Lakeside (drawing seated is S34-e's; this PR wires
   the decision so both clients agree, AC-f-4).
7. `scripts/ci_layer.py`: `clients-probes` gains `slice --world=lakeside --seat`.
8. Threshold light at the bakery door: the café's threshold pattern (one `OmniLight3D` inside, the exposure check
   VISUAL_SLICE §6.2), measured by `--slice-measure`.
9. `docs/DECISIONS.md` (or `docs/decisions/`): `ARC-145` (rewritten), `ARC-172`.
10. Under Q-S33f-1 option C only: `worlds/lakeside/places/{pier,square,bakery,station,station-platform}.yaml`
    `sit_points:`, `worlds/lakeside/world.yaml` (`posture` enabled), `scripts/baselines.txt` (a second recorded
    re-baseline).

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| The posture System, its actions and disclosure | S34-a |
| `sit_points:` in Lakeside's world data, `posture` enabled there (unless Q-S33f-1 = C) | S34-c |
| Drawing seated people in 2D; the player's E to sit; sit/stand transitions | S34-e, S34-f |
| The halt's building and benches | S33-e2 |
| People, extras | S33-g |
| The boathouse's rooms above; cottage interiors | never (no such places; Q-S33-5) |

### 2.3 Dependencies

S34-a merged (C0 stop otherwise): it fixes the shape §5.2's table must satisfy. S33-e2 merged for the halt rows;
if S33-e2 is later, the halt rows are added by a bounded follow-up commit on S33-e2's merge (recorded), not
invented on undrawn benches.

---

## 3. Audit (`origin/main @ a1e05874`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | Lakeside's indoor solids (each place's frame, mm): bakery floor (−1 000, 0)–(5 000, 8 000), oven wall (−1 000, 7 000)–(2 500, 8 000) h 2 200, counter (2 500, 4 600)–(4 600, 5 200) h 950, racks (4 400, 5 600)–(5 000, 7 800) h 1 900, long table (−700, 1 400)–(800, 4 600) h 750; boat shed floor (−1 000, 0)–(5 000, 12 000), boat (−600, 3 200)–(1 200, 11 000) h 1 100, workbench (3 900, 6 000)–(4 700, 9 000) h 900, gated stair (3 000, 10 400)–(5 000, 12 000) h 3 000 | `pr-s33b-lakeside-bodies.md` §5.3 |
| A-2 | Outdoor seats: the pier bench (400, 300)–(3 600, 750) h 450 (the pier's head is y 0); the pavilion posts at y 200…400 and 3 600…3 800; the square's benches (10 900, 9 600)–(12 700, 10 000) and (12 600, 8 100)–(13 000, 9 900) h 450, the lime trunk (11 800, 8 800)–(12 200, 9 200) | `pr-s33b-lakeside-bodies.md` §5.3 |
| A-3 | The halt's benches: "the bench row along the far wall, h 450" (waiting room) and "the bench(es) … h 450" (platform), without coordinates; S33-b's C2 places them | `pr-s33b-lakeside-bodies.md` §5.3 |
| A-4 | Bodies' clearances on 12d: R 250 mm, R + GAP 260 mm from a floor edge or solid for a doorway point; a start or a seated person ≥ R from every edge and solid | `pr-s33b-lakeside-bodies.md` A-2, A-3 |
| A-5 | A section of a pack not enabled in `systems:` is refused at read ("recorded, not decoded: `read` refuses it naming the pack", `ARC-121` item 1) | `worldpack/src/content.rs:12`; `worldpack/src/error.rs:483` |
| A-6 | No `clients/shared/sitting/` exists; the 3D client's `scripts/posture.gd` is an art correction of the standing idle (a `SkeletonModifier3D`), unrelated to sitting | `clients/shared/`; `scripts/posture.gd:1–20` |
| A-7 | The spike's seat constants: `CAFE_SEAT_Y` 0.45, `BENCH_SEAT_Y` 0.44 | `scripts/town.gd:26–27` |
| A-8 | S34-a's frozen scope: `systems/posture`, `sit_points:` (place files only), `posture-state`, `sit`/`stand`; "no shipped world enables it" | S34-a design §1 |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33f-1** | **S34 replaces the step's sit-point model.** Step §4.5 made a sit point presentation data with a client-side rule; S34 (frozen) makes it semantic world data with server-owned posture, and keeps the step's rule only as the fallback for worlds without posture | `ARC-145` is rewritten to S34's shape (§5.1). The step's §4.5 gets a dated note (§11). No requirement of the step is dropped: AC-S33-3 and X-4 hold under either branch |
| **F-S33f-2** | **Q-S34-9's ruling collides with Q-S33-8 and S34-c's scope.** Q-S34-9 says "S33-f authors Lakeside's points once as world data". A `sit_points:` section is refused unless `posture` is enabled (A-5), and enabling it moves Lakeside's digests: a second re-baseline, where Q-S33-8 ruled one (in S33-b). S34-c's own row already enables posture in Lakeside with a recorded re-baseline | **Q-S33f-1** (§14). Recommended A: S33-f fixes the values once (§5.2) and lands only presentation; S34-c writes §5.2's table into the place files verbatim, inside its own re-baseline |
| **F-S33f-3** | **The pier bench cannot face the lake.** It stands 300 mm from the head's edge (A-2). A sitter facing south would stand 40 mm from the edge, under the 250 mm bodies needs | The pier's sit points face **north**, back to the lake, looking at the village across the water — the view AC-S33-11 asks for ("look back at the village … sit on the pier bench"). Facing the lake would need S33-b's bench moved **≥ 220 mm** north, so that its front edge stands ≥ 520 mm from the head's edge (260 mm to `at`, then 260 mm to the edge) (Q-S33f-2) |
| **F-S33f-4** | **One square bench is too close to the lime tree.** The north bench's south face is 400 mm from the trunk (A-2); a sitter facing south would stand 140 mm from the trunk | The north bench's points face north, the east bench's face east: both backs to the tree, which is how benches under a village tree stand |
| **F-S33f-5** | **The long table's west side is unusable.** It stands 300 mm from the bakery's west wall (A-1); a sitter there would stand 40 mm from the wall | Five places on the east side only, facing west along the table (the west wall is the party wall with the boathouse, so there is no window there; the bakery's window is on its lane side); chairs are drawn props, not solids (S33-b names none) |
| **F-S33f-7** | **A chair at a table needs a short pelvis offset.** S34 places the pelvis at `at` moved back along `facing` by the depiction's `seat_depth_m` (SD-S34-3). With `at` 260 mm from the table's edge, a bench-like 0.42 m would put a diner's pelvis 680 mm from the table — too far to eat | For the long table's chairs `seat_depth_m` is **0.15**: the pelvis sits ≈ 410 mm from the table's edge, and the chair is drawn under the pelvis with its front edge tucked into the 260 mm band (a chair is a prop, not a solid, so bodies is unaffected). The dining distance is cited at C1 from an anthropometric source (Panero and Zelnik, *Human Dimension & Interior Space*, 1979, the dining-clearance plates), or, if C1 cannot quote one, marked approximate with S34-f as its replacer (S34 R-S34-6 records the related chair-point risk) |
| **F-S33f-6** | The halt's benches have no coordinates yet (A-3), and F-S33e-1 moves the ticket bay onto the waiting room's north wall | §5.2's halt rows are proposed and fixed at C0 from S33-b's and S29-a's merged files |

### 3.3 What does not exist

No Lakeside interior, no sit point anywhere in world data or presentation, no shared sit decision, no `--seat`
probe.

---

## 4. Reuse, both ways

| Need | Decision | Why |
| --- | --- | --- |
| Interiors | the slice's interior builders' patterns (`cafe_interior.gd`, `shop_interior.gd`), the shipped props (`wooden_display_shelves_01`, `wicker_basket_01`, `wooden_crate_01`, `round_wooden_table_02`, `dining_chair_02`, bread and food models) | one method; already shipped |
| The sit decision | S34 §8.1's shared function in `clients/shared/` | one rule for both clients (step §4.5; S34 SD-S34-8) |
| The seat-contact probe | S26's G-13 method (pelvis within ± 20 mm) | the gate already exists for characters |

---

## 5. Design

### 5.1 `ARC-145`, rewritten to S34's shape

A Lakeside sit point is S34's semantic `SitPoint` (key, `at` on free floor, `facing`, group). Its **depiction**
is presentation: `seat_height_m`, `seat_depth_m` and the drawn prop. Both clients decide "seated" through one
shared function: from the disclosed `posture-state` where the place discloses `sit-points`; else by the step's
fallback rule (SD-S33-7), whose positions come from the depiction file (option A) until S34-c lands.

### 5.2 Lakeside's sit points (the one table; S34 SD-S34-1…4)

Frames are each place's (mm, x east, y north); `facing` in millidegrees per `CORE_CONCEPTS.md` §6.1
("yaw measured from +y toward +x, so 0 mdeg faces north and 90 000 mdeg faces east", `CORE_CONCEPTS.md:440`).
Every `at` is 260 mm in front of its seat's front edge (SD-S34-3) and ≥ 260 mm from every other edge and solid
("exactly like a doorway point", SD-S34-3); points on one bench are 600 mm apart (SD-S34-4's default; S34-c cites
the seating-allowance source). These clearances assume 12d's radius R = 250 mm; R-f-4 covers a different ruling.

| Place | Group | Keys | `at` (x, y) | Facing | Seat h / depth (m) | Prop |
| --- | --- | --- | --- | --- | --- | --- |
| `pier` | `pier-bench` | `pier-bench.1…5` | (800, 1 010), (1 400, 1 010), (2 000, 1 010), (2 600, 1 010), (3 200, 1 010) | 0 (north, the village) | 0.45 / 0.45 | slatted bench, back to the lake (F-S33f-3) |
| `square` | `tree-bench-north` | `tree-bench-north.1…3` | (11 200, 10 260), (11 800, 10 260), (12 400, 10 260) | 0 (north) | 0.45 / 0.40 | `painted_wooden_bench` |
| `square` | `tree-bench-east` | `tree-bench-east.1…3` | (13 260, 8 400), (13 260, 9 000), (13 260, 9 600) | 90 000 (east) | 0.45 / 0.40 | `painted_wooden_bench` |
| `bakery` | `long-table` | `long-table.1…5` | (1 060, 1 900), (1 060, 2 500), (1 060, 3 100), (1 060, 3 700), (1 060, 4 300) | 270 000 (west, along the table) | 0.45 / **0.15** (F-S33f-7) | `dining_chair_02`, drawn under the pelvis |
| `station` | `wall-bench` | `wall-bench.1…7` | proposed (1 100…4 700 step 600, 4 290) under a bench (800, 4 550)–(5 000, 5 000); fixed at C0 | 180 000 (south) | 0.45 / 0.45 | timber bench row (S33-e2) |
| `station-platform` | `platform-bench-n`, `platform-bench-s` | `.1…3` each | proposed x −3 190, y 6 500 / 7 100 / 7 700 and −7 700 / −7 100 / −6 500, under benches (−3 900, ±6 200…±8 000)–(−3 450, …); fixed at C0 | 90 000 (east, the track) | 0.45 / 0.45 | timber bench (S33-e2) |

**Sources.** Bench seats 0.45 m: within `VISUAL_SLICE.md` §3.4 (0.42–0.47 m) and the 2010 ADA Standards §903.5
(bench seat "17 inches (430 mm) minimum and 19 inches (485 mm) maximum above the finish floor"). Bench depth
0.40–0.45 m: approximate. ADA §903.3 (bench size) asks a compliant bench for 42 in (1 065 mm) length and 20–24 in
(510–610 mm) depth, and §903.4 for back support; **none of Lakeside's 0.40–0.45 m deep benches meets §903.3's
depth**, which is recorded, not claimed (the world's benches are S33-b's solids). Table 750 mm: within ADA §902.3's dining
surfaces (28–34 in, 710–865 mm). Chair seat 0.45 m: as the bench. C1 quotes each clause from the Access Board's
text with the date.

**Totals:** 29 points (pier 5, square 6, bakery 5, halt 13). None in the boat shed, the cottages' yard, the lane
or on the shore's boulders (no world seat there; adding one is a content change for later).

### 5.3 The interiors (Q-S33-5)

| Room | Fit-out (every solid at S33-b's footprint within 50 mm) | Hand scale (VISUAL_SLICE §6.2) |
| --- | --- | --- |
| Bakery | the oven wall (a brick bread oven, its iron door, a peel on hooks), the counter (glazed front, a till drawn **shut** — no economy), bread racks with loaves and baskets, the long table with five chairs on its east side, a window seat light from the lane | loaves, baskets, the peel, flour sacks |
| Boat shed | a timber rowing boat on its cradle (the boat solid), the workbench with tools and clamps, oars racked on the wall, nets hung from the beams, the stair to the rooms above with a closed, latched gate at its foot (no upstairs place) | planes, chisels, a pot of tar, rope coils |
| Halt | S33-e2's room; this PR adds only the sit points' depictions | — |

Both doors (bakery, shed) are drawn open in this PR, because the server lets people through and the rooms now
exist (S33-d §5.5). The threshold light follows the café's pattern and is measured.

### 5.4 The shared decision (`MineWorldSitRule`)

```text
seated_at(person, place, disclosure, depictions, still_since) -> point_key | none
  if place discloses `sit-points`:
      return person's disclosed posture-state Seated.point, else none          (S34 §8.1)
  else (fallback, SD-S33-7):
      candidates = depictions of this place with fallback `at`
      p = the candidate whose `at` lies within 0.40 m of the person's disclosed position
      return p if the person has not moved for ≥ 2 s of world time and no person with a lower entity id
             is already drawn at p; else none
```

The function holds no rule beyond these; S14's rule-constant scan gains its constants' names, so neither client
holds a copy (S34 §8.1).

### 5.5 The `--seat` probe

For every depiction in `sit_points/lakeside.yaml` and every body available (the Lakeside cast's bodies and the
extras' pool: S26-c1 / S32-c as merged; each missing body is reported as `SKIP <person> no body`, never as a
pass), place the body seated at the point (pelvis at `at` moved back by `seat_depth_m`, up to `seat_height_m`,
S34 SD-S34-3) and measure the pelvis' contact height against the drawn seat's surface: within **± 20 mm** (G-13,
AC-S33-3). Verdict `all seat checks pass` / `<n> SEAT CHECKS FAILED`, failures naming the point and the body.

### 5.6 Two clients agree on a contested seat

- **Posture branch:** the server decides (S34: a taken point refuses `TargetUnavailable`); both clients draw the
  disclosed occupant. Checked once S34-c has enabled posture in Lakeside (S34-c's AC, recorded here as R-S34c-f-1).
- **Fallback branch (this PR):** two people still within 0.40 m of `long-table.3` → both the 2D and the 3D client
  draw the lower entity id seated and the other standing; the probe prints both clients' decisions and compares
  them (AC-f-4).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-f-1 (I-S33-1) | Under option A: no path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/` changes | diff scan |
| I-f-2 | Every sit point satisfies S34's validation (spacing ≥ 510 mm, `at` clear of every solid and every floor edge by ≥ 260 mm — or the values of S34-a as merged) | a test loading §5.2's table into a copy of Lakeside with `posture` enabled, under the scratchpad, through S34-a's loader (AC-f-1) |
| I-f-3 | One table: the depiction file's fallback positions equal §5.2's values byte for byte; S34-c's world lines equal them too | AC-f-5; R-S34c-f-1 |
| I-f-4 (I-S33-5) | No seated pose without a person at a point; no till open; no upstairs | review; AC-f-4 |
| I-f-5 (I-S33-9) | Every drawn interior solid is within 50 mm of a disclosed face | S33-d's `--geometry` on the bakery and the shed |
| I-f-6 | Neither client holds a sit rule outside `clients/shared/` | the rule-constant scan |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Data | I-f-2 (the table under S34-a's loader), I-f-3 | `tools/cli/tests/lakeside_sit_points.rs` (CI `core`) |
| Probe | `--seat` (AC-S33-3), the contested seat (AC-f-4), the interiors' route (bakery and shed entered, place changes) | nightly `clients-probes`; local one at a time |
| Rendered | threshold light; the interiors' review frames | `--slice-measure`; `shots/lakeside/review/` |
| Static | I-f-6 | `client_rules.rs` |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-f-1 | §5.2's table loads under S34-a's validation in a posture-enabled copy of Lakeside (no refusal) | I-f-2 |
| AC-f-2 | `--seat`: every point × every available body within ± 20 mm; missing bodies listed | AC-S33-3 |
| AC-f-3 | X-4 (one depiction's `seat_height_m` + 0.05) → `--seat` red naming the point and the body | X-4 |
| AC-f-4 | The contested seat: the 2D and 3D decisions are equal for the scripted pair | §12.1 adversarial |
| AC-f-5 | The depiction file's fallback `at`/`facing` equal §5.2's table (a test reads both) | I-f-3 |
| AC-f-6 | `--world=lakeside --link` enters the bakery and the shed: place changes, 0 refusals | §12.1 checkpoint |
| AC-f-7 | The bakery threshold measures within VISUAL_SLICE §6.2's exposure band | step §12.1 |
| AC-f-8 | Bytes added ≤ 2 MB | §8.2 |

---

## 9. Commit plan

Procedure: [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md) §9.0, from
`/Users/yuema137/mineworld-worktrees/impl-s33f`.

#### C0 — Base, prerequisites, ledger

- S33-d, S34-a merged (stop otherwise); S33-e2's state; the halt's benches as merged (fix §5.2's halt rows); the
  bodies available for `--seat`; the spacing and clearance of S34-a as merged, and BC-7's ruling (R-f-4: re-space
  to 610 mm if so ruled).
- **Commit message:** `docs(plan): S33-f — open the ledger at the base`

#### C1 — `ARC-145` rewritten, `ARC-172`, the clause quotes

- **Files:** `docs/DECISIONS.md` (or `docs/decisions/`), this file.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: `ARC-145` says nothing S34-e/S34-f must undo.
- **Commit message:** `docs: ARC-145 and ARC-172 — Lakeside's sit points in S34's shape (S33-f)`

#### C2 — The sit-point table and its test

- **Files:** `presentation/mineworld-default/3D/sit_points/lakeside.yaml`, `tools/cli/tests/lakeside_sit_points.rs`,
  this file.
- Validation: AC-f-1, AC-f-5. Mutation **M-f-1** (`pier-bench.2` moved 200 mm toward `.1`) → AC-f-1 red with S34's
  spacing refusal.
- **Commit message:** `feat(presentation): Lakeside's sit points, one table in S34's shape (S33-f)`

#### C3 — The shared decision

- **Files:** `clients/shared/sitting/sit_rule.gd` (or S34-e's, unchanged), the 2D wiring, the 3D wiring,
  `client_rules.rs` (the constants), this file.
- Validation: AC-f-4. Mutation **M-f-2** (the 2D client's tie-break reversed) → AC-f-4 red.
- **Commit message:** `feat(shared): one sit decision for both clients, posture first (S33-f)`

#### C4 — The interiors

- **Files:** `scripts/worlds/lakeside/interiors.gd`, `depiction.gd`, this file.
- Validation: AC-f-6, AC-f-7; `--geometry` on both rooms. Frame review VR-10 (the bakery from the door), VR-11 (the
  shed).
- **Commit message:** `feat(3d): Lakeside's bakery and boat shed interiors (S33-f)`

#### C5 — The `--seat` probe

- **Files:** `slice_probe_world.gd`, `scripts/ci_layer.py`, this file.
- Validation: AC-f-2, AC-f-3 (X-4).
- **Commit message:** `test(3d): every Lakeside seat holds every body within 20 mm (S33-f)`

#### C6 — Close-out

- Push; PR; CI on the exact head; repair; train-ready.
- **Commit message:** `docs(plan): S33-f close-out`

---

## 10. Requirements this PR places on other PRs

- **R-S34c-f-1 (on S34-c).** Under Q-S33f-1 A: write §5.2's table into Lakeside's place files verbatim, enable
  `posture`, delete the fallback positions from `sit_points/lakeside.yaml` in the same PR, and run the contested
  seat in the posture branch.
- **R-S34f-f-1 (on S34-f).** Bind Lakeside's depictions by `{place, point}` and keep `--seat` green.
- **R-S33b-f-1 (on S33-b, only if Q-S33f-2 is ruled B).** Move the pier bench ≥ 220 mm farther from the head's
  edge (F-S33f-3).

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §4.5, §14.2 | a dated note: sit points are S34's semantic data; the step's rule is the fallback (F-S33f-1) |
| `step-34-posture.md` §14.2 S34-c row | Lakeside's values from `pr-s33f` §5.2 (Q-S33f-1 A) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-f — Lakeside's interiors and sit points
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33f-interiors-sit-points.md
RELATED / BINDING DOCS:  step-33-lakeside-3d.md; step-34-posture.md; pr-s34a-posture-pack.md;
                         pr-s33b-lakeside-bodies.md; pr-s33d…; pr-s33e…; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s33f, one session
BRANCH:                  mvp0/pr-s33f, from origin/main
START CONDITION:         per the header (NONE met at 2026-10-18)
APPROVED SCOPE:          §2.1 items 1–9 (item 10 only under Q-S33f-1 C)
FROZEN INVARIANTS:       step-33 I-S33-1, -3, -5, -9; S34 SD-S34-1…4, SD-S34-8; §6 I-f-1…I-f-6
SEQUENCE:                C0 … C6
ALLOWED COMMANDS:        as pr-s33d §12, plus cargo test -p mineworld-cli --test lakeside_sit_points
NEVER:                   as pr-s33d §12; a sit rule in either client; a sit point that fails S34's validation
MATERIAL STOPS:          S34-a missing; §5.2's table refused by S34-a's validation in a way a bounded move
                         (≤ 100 mm) cannot fix; a body failing ± 20 mm whose fix is in the body (S32/S26)
ENDPOINT AUTHORITY:      implementation, commits, push, PR, CI repair: authorized once started;
                         merge: through a merge train under the charter
STOP CONDITION:          train-ready
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-f-1 | Two copies of the positions (fallback file, world data) drift | AC-f-5 now; S34-c deletes the fallback copy (R-S34c-f-1) |
| R-f-2 | Bodies arrive late (S32's critical path) | `--seat` reports missing bodies; S33-g re-runs it |
| R-f-3 | The pier's sitters face away from the lake and the operator wants the lake view | Q-S33f-2 |
| R-f-4 | **The spacing basis changes.** §5.2 assumes 12d's R = 250 mm (510 mm spacing, 260 mm clearance). `main` still has R 300 mm (12d #171 is open), and S34-a's ledger has escalated 510 versus 610 mm to the operator (its finding BC-7). At 610 mm every 600 mm bench row is refused | C0 reads the spacing and clearance from S34-a as merged; if 610 mm is ruled, the rows are re-spaced to 610 mm (a bounded correction: pier 5 → 5 points at 610 mm fits 3.2 m; the square's 1.8 m benches keep 3; the bakery's 3.2 m table keeps 5), recorded in §15 |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33f-1 [operator — who writes Lakeside's sit points into world data]** | Q-S34-9 said S33-f "authors Lakeside's points once as world data". A `sit_points:` section only loads when `posture` is enabled, and enabling it moves Lakeside's digests — a second re-baseline, where Q-S33-8 allowed one in S33-b; S34-c already plans to enable posture in every world with its own recorded re-baseline (F-S33f-2). (A) S33-f fixes the values once (§5.2) and lands presentation only; S34-c copies the table into the place files verbatim. (B) S33-f waits for S34-c and lands depictions only. (C) S33-f writes the world data and enables posture in Lakeside, with a second recorded re-baseline | **(A).** The values are authored once, in this design; the world change and its re-baseline stay where S34 planned them; Lakeside shows sitting from S33-f on (fallback), and from the server once S34-c lands. (B) delays seats behind 12d's chain; (C) splits S34's re-baseline. *Chinese:* 湖边的座位点由 S33-f 一次定好数值、只落表现层，世界数据和重算基线交给 S34-c 照抄——同意吗？ |
| **Q-S33f-2 [operator — the pier bench's view]** | The pier bench stands 0.3 m from the head's edge, so a sitter can only face north, toward the village (F-S33f-3). Keep that (A), or move the bench at least 0.22 m north in S33-b so people can sit facing the lake (B)? | **(A).** Sitting at the pier's end looking back at the village across the water is the view the operator's walk asks for, and nothing frozen changes. *Chinese:* 码头尽头的长椅朝北看村子（不朝湖）——保持现状吗？ |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C6 | — | [ ] | [ ] | [ ] |

Mutations: M-f-1, M-f-2, X-4 (none run yet). Findings: F-S33f-1…F-S33f-7 (§3.2).

**Design review (2026-10-18).** One independent Opus reviewer, verdict REQUEST CHANGES, no blockers; §5.2's
clearances and spacings confirmed point by point. Applied before the PR opened: the bakery's facing reads "along
the table" (no window on the party wall); the chairs' pelvis offset 0.15 m (F-S33f-7); the spacing basis made a
risk and a C0 check (R-f-4, S34-a's BC-7); one figure for the pier bench's move (≥ 220 mm); ADA §903.3/§903.4
attributed correctly; the facing convention cited from `CORE_CONCEPTS.md:440`; I-f-2's clearance aligned with
SD-S34-3.
