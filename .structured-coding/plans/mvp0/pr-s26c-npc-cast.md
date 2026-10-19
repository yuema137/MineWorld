# PR S26-c — every named NPC made with the protagonist's pipeline (route D+, fused)

**Lifecycle:** `DRAFT — implementation started on the coordinator's relay of the operator's order (2026-10-19):
"generate now, handle package size at the milestone"`. Freeze requested; the scope below is what this PR does.
**Parent:** [`step-26-cast.md`](step-26-cast.md) (§§6, 10.1, 12 — this PR is the S26-d batch work for the café
town, run on the S26-b1 pipeline). **Base:** `origin/mvp0/pr-s32p @ b7da622d` (#259), because the only paid command
shape the permission system allows is `cast.py wardrobe meshy …`, which lives there.
**Lane:** S26-c lead. Worktree `/Users/yuema137/mineworld-worktrees/impl-s26c`, branch `mvp0/pr-s26c-npc-cast`.

## 1. What and why

The operator (2026-10-19): in the 3D client the other NPCs still look nothing like the protagonist. Every named
Person must be made the same way as Vera, in template v2's realistic style, from the 21 operator picks of
`cast/PICKS.yaml`. Café town first (the 11 named Persons of `worlds/social-cafe` and `worlds/market-town` other than
Vera), then lakeside (10).

Per Person, the same chain as Vera (step-26 §5.2, `PIPELINE.md` §2):

```text
casting pick (v2, operator) → stage 3 mask → Meshy image-to-3D (30 credits) → raw gate (agent)
→ ortho render → landmark picks (agent) → fit.yaml → stages 6–10 (gameready, fit_bind with the 4-decimal
handoff, finger weights, flecks, export) → G-1…G-6, G-14 → pose sheet (P-1…P-8) beside Vera → scorecard
```

## 2. Scope

- `tools/character_pipeline/recipes/s26c-cast.yaml`: a `"route": "fused"` recipe — the 21 picks with SHA-256s,
  no garments or outfits, its own ledger capped at 1,800 credits.
- `mwcast/wardrobe/recipe.py`: the `route` field (`modular` default; `fused` has no outfits).
- `mwcast/gates.py`, `mwcast/glb.py`: G-6(b) also intersects the depth ray with the triangles (F-S26c-1).
- `blender/fit_bind.py`, `mwcast/records.py`, `mwcast/pipeline.py`: `fit.depth` (`dplus` default, `midspan` for the
  fused cast; F-S26c-2).
- Per Person: `presentation/mineworld-default/3D/characters/cast/<id>/fit.yaml` and `generation_log.yaml`.
- `PIPELINE.md` §17 and the `fit.depth` row of §4.1.
- Review page (not in the repository): `mineworld-demos/casting/npc-generated.html`.

**Not in this PR:** the binaries (GLB, glTF, PNG — `PIPELINE.md` §10 keeps them out; ~10 MB per Person),
`cast.yaml` `looks` entries, `character.yaml`, and the client resolution of `look` (S26-c1/c2). Those wait for
the operator's answer to Q-S26c-2.

## 3. Invariants

- I-1 Vera's re-derivation is unchanged: `fit.depth` defaults to `dplus`; G-6's new hits only add surfaces the
  ray really crosses.
- I-2 No per-Person code: every Person is data (`fit.yaml`, the recipe).
- I-3 Spend: every paid call through `cast.py wardrobe meshy body` under the recipe ledger; S26 total ≤ 3,000.
- I-4 Keys only from `secrets.env`; nothing printed.

## 4. Findings

- F-S26c-1 (fixed): G-6(b) vertex-window miss on decimated fused bodies (`PIPELINE.md` §17).
- F-S26c-2 (fixed): slab-mean depth biased by a fused apron (`PIPELINE.md` §17).
- F-S26c-3 (open): `check-characters` counts `gameready.eye_z_raw`, `centre_band_z_raw` and `eye_from_sole_m` as
  overrides, though they are per-character measurements; every new Person shows 4–5 "overrides". Proposal: add
  them to `OVERRIDE_EXEMPT`.
- F-S26c-4 (open, client): several picks have the palms turned forward at rest (Meshy follows the reference), so
  the hanging arms read supinated in-client. Proposal: a per-character forearm twist in `character.yaml`'s
  `posture` (S26-c1).
- F-S26c-5 (open, client): P-4 is flagged by the protagonist seating rule (Alice's hips 0.39 m over the seat
  against Vera's 0.25 m) while seat contact passes; `human.gd` `sit()` needs the per-character hips-over-seat.
- F-S26c-6 (account): the Meshy account balance (monthly 680 at the start, shared with the S32-p pilot) cannot
  cover lakeside after the café town; see Q-S26c-1.
- F-S26c-7: the ledger charges the balance delta when it exceeds `consumed_credits`; with the pilot running on the
  same account the delta doubled twice (60 for a 30-credit task). Measured cost is 30 per run.

## 5. Questions for the operator (recommended answers)

- Q-S26c-1: Meshy balance. Recommendation: top up (or wait for the monthly reset, about 2026-11-07) for lakeside's
  10 Persons and retries, about 300–450 credits.
- Q-S26c-2: where the finished binaries live. Recommendation: commit each Person's stored-once glTF + PNG
  (~10 MB) under `clients/3d-spike/assets/characters/cast/<id>/` with S26-c1's client resolution, and handle the
  release size at the milestone (S26-f), as ruled for package size.
- Q-S26c-3: the step-32 amendment cancelled the fused phase 2 in favour of route B. The coordinator's order is
  route D+ now. Recommendation: keep the fused characters as the release cast until a route-B outfit passes parity.

## 6. Ledger

| Commit | Implementation | Validation | Review |
| --- | --- | --- | --- |
| C1 fused recipe, G-6 by triangles, `fit.depth` | [x] | [x] 140 tests pass locally (uvx pytest); ruff clean | [ ] |
| C2 café town records (11 Persons) | [ ] | [ ] gates per Person, pose sheets | [ ] |
| C3 lakeside records | [ ] blocked on Q-S26c-1 | [ ] | [ ] |
