# PR S33-g — Lakeside's own people in 3D: the cast, the extras, and only the activities the world discloses

## PR DESIGN (DRAFT) — not frozen

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874. Read-only: the
                        fishing pack at its pinned commit 26221378 (~/.cargo/git/checkouts/mineworld-pack-fishing-*),
                        S32-p (#259, origin/mvp0/pr-s32p)
Approved by / evidence: none yet. Steps 26, 32 and 33 bind; Q-S26-1 (as superseded by S33), Q-S26-8 (E-2),
                        Q-S32-*, Q-S33-* are not re-asked; §14 is open
Implementation base:    origin/main at C0
Start condition:        S33-c merged (the seam); S26-a merged (Lakeside's `appearance:` lines); S26-c2 merged
                        (connected figures resolve look → cast → character); S32-c merged (modular characters);
                        S32-d's Lakeside batch accepted by the operator (VIS gates); S32-f merged (Lakeside's
                        starting wardrobes); §14 ruled; this design frozen; the lane lead's dispatch. NONE holds
                        at 2026-10-18. S33-d…f are not required (the people walk the stub if needed), but the
                        crowd perf view is judged on S33-d's village
Execution contract:     §12
Implementer:            Haiku-class from this commit plan (step-33 §12.1); Opus frame review of the crowd view
Lifecycle:              PR DESIGN (DRAFT)
```

**Effort:** `mvp0`, release-blocking — **the critical path with S32** (step §12.2) · **Step:**
[`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §§1 (outcome 5), 2.3 (F-S33-5, F-S33-6), 5, 8.1 (`lakeside
crowd`), 10 (I-S33-5), 12.1 (the S33-g row) · **Binding steps:** [`step-26-cast.md`](step-26-cast.md) (§4.4 binding
and fallback, §8 E-2, X-16), [`step-32-wardrobe.md`](step-32-wardrobe.md) (S32-c, S32-d, S32-f rows),
[`pr-s32p-wardrobe-pilot.md`](pr-s32p-wardrobe-pilot.md) (route B, template v2) · **Binding parents:**
`CLAUDE.md` §§1, 4 (items 3, 15); `docs/ENGINEERING_RULES.md` §§2–3; operator rule "NPCs are made to the
protagonist's standard".

**Decision ids.** `ARC-173` (§5.4: a client draws an activity only from disclosed state; an undisclosed Process
is not drawn, for any pack), written by C1. No other id.

No model is called. No Meshy credit is spent by this PR (the bodies are S32-d's).

---

## 0. Binding inputs, exactly restated

- **Scope (step §12.1, S33-g row).** "Lakeside's cast bound in its own world; extras = café cast; ≤ 6 decor
  figures; the fishing pose tied to the disclosed Process; the `lakeside crowd` perf view." Depends on **S32-c
  merged**; S32-d's Lakeside batch accepted; S26-c2. Checkpoint: "every Lakeside person is their own character in
  their outfit; an unknown person is an extra". Adversarial: "S26 X-16 on Lakeside; no Lakeside person drawn as an
  extra".
- **People (step §5).** "each person's `look` (S26-a) binds their modular body (S32-c), wearing their starting
  wardrobe (S32-f). The binding, fallback and extras rule are S26 §4.4's, unchanged." "No Lakeside person ever
  appears as an extra in Lakeside (the pools are disjoint). Decor figures in Lakeside are few (≤ 6 …) and sit only
  if they passed G-13." "No placeholder figure … never CharMorph, never a capsule (S26 X-16)."
- **Activities (step §5).** "A person fishing is drawn at the water's edge facing the water, with a rod prop,
  **only while** the world discloses their fishing Process … S33-g audits the exact disclosure. No one is shown
  rowing (F-S33-6)."
- **The standard (operator).** Template v2, realistic; every NPC by the protagonist's pipeline and bar.
- **R-S33-4.** If S32's route B fails the pilot, "S33-g is the only PR that waits; the environment lands; the
  release waits for people".

---

## 1. Goal

In 3D Lakeside, Alba bakes, Nils stands on the pier and Elin paints — each in their own body and their own clothes,
made to Vera's standard. People who are not Lakeside's are the café town's cast, never a capsule. Nobody is drawn
doing something the world has not said they are doing.

---

## 2. Scope

### 2.1 In scope

1. `scripts/worlds/lakeside/depiction.gd`: the extras pool (the café town's cast keys, by S26's `cast.yaml`), the
   decor figures (5, within the step's ≤ 6; none sits, §5.3).
2. `clients/3d-spike/scripts/slice/` figure path (S26-c2's): no Lakeside-specific code; C2 verifies the path binds
   all ten by look and outfit.
3. The `lakeside crowd` perf view (step §8.1).
4. The activity rule (§5.4; `ARC-173`): a check that no Lakeside figure plays a fishing or rowing clip without a
   disclosed state that names it.
5. `scripts/ci_layer.py`: `clients-probes` gains `slice --world=lakeside --cast`.
6. `docs/DECISIONS.md` (or `docs/decisions/`): `ARC-173`.

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| Generating or accepting Lakeside's bodies and garments | S32-d (batches), S26-d (B2) |
| Lakeside's starting wardrobes in world data | S32-f |
| A disclosed fishing state | the fishing pack's own repository (Q-S33g-1) |
| Seated drawing | S33-f / S34 |
| 2D people | S26-e, S32-e |

---

## 3. Audit (`origin/main @ a1e05874`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | Ten Lakeside people, all seats. Homes: Alba, Bruno, Celia, Greta, Ines, Elin (cottages); Dario, Nils, Finn, Hugo (boathouse). Starts: the six in the cottages; Dario, Finn and Hugo in the boathouse; Nils on the pier (`nils.yaml:21`, his routine "05:00 pier fishing … 19:00 boathouse home") | `worlds/lakeside/people/*.yaml`; step A-18; `pr-s33b-lakeside-bodies.md` A-11 |
| A-2 | S26's binding: `look` → `cast.yaml` → character; fallback §4.4 (unknown look → an extra); E-2: the other world's cast is the extras pool; S26 X-16: no placeholder figure | `step-26-cast.md` §§4.4, 8; rows S26-a, S26-c2 (:1312, :1317) |
| A-3 | S32's chain for bodies: S32-p (route B on Vera, Ivan, Hana; #259 open) → S32-c (modular characters) → S32-d (every cast body by the ruled route) → S32-f (Lakeside's starting wardrobes) | `step-32-wardrobe.md` :1172–1176 |
| A-4 | **The fishing pack discloses nothing.** Its header: "action `fish` … resolve `fishing-started` to the angler; its reduction starts one `catch` process … discloses nothing". The `Angler` component is the pack's own; `fishing-spot` is `SystemInternal`; the only fact others may hear is none | `mineworld-pack-fishing@26221378 src/lib.rs:9–31`, `src/event.rs:11–12` |
| A-5 | An `Observation` lists entities, relations, events and affordances; it has no Process field. A Process is visible to an observer only through a pack's disclosed components or events | `contracts/src/observation.rs:334–384` |
| A-6 | The pack is third-party, "installed by ARC-33's two lines pinned to a commit (ARC-66) and never edited here" | `worlds/lakeside/world.yaml:6–7`; `Cargo.lock:6–8` |
| A-7 | The 3D client's crowd perf view exists for Social Café (S26 §11) | `scripts/slice/slice_perf.gd` (S26-c1) |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33g-1** | **No observer can see that someone is fishing.** The step's rule "drawn fishing only while the world discloses their fishing Process" can never fire for another person (A-4, A-5). The angler alone learns `fishing-started` | **Q-S33g-1.** Recommended: no fishing pose for others until the fishing pack discloses its `Angler` to persons present; the pack's owner releases that (a new pinned commit is a dependency change, `ARC-66`, recorded under its own PR). Meanwhile the angler's own client may draw its own player fishing from its own `fishing-started` |
| **F-S33g-2** | **Bodies are on the critical path** (A-3): S32-p is open; S32-d's Lakeside batch follows S32-c | Nothing here waits for anything but bodies; the PR's start condition names them (R-S33-4) |
| **F-S33g-3** | **Decor figures are presentation people.** They have no world entity. "Nothing pretends" allows them only as passers-by doing nothing a world person would be credited with | ≤ 5 decor figures, drawn from the extras pool, walking the shore trail or standing on the square; never at a sit point; never fishing; never entering a place |
| **F-S33g-4** | **A decor figure cannot sit honestly** (§5.3): the seat's occupancy is the server's (S34) | No decor sitter; the step's "a person on the square's bench" is dropped (§11) |

### 3.3 What does not exist

No Lakeside cast binding in 3D, no Lakeside extras pool, no crowd view for Lakeside, no activity rule.

---

## 4. Reuse

S26's binding and fallback, S32-c's modular character, S26's crowd view: reused unchanged. Nothing new is built
or adopted.

---

## 5. Design

### 5.1 The cast in its own world

The connected figure path (S26-c2) already resolves each disclosed person's `look` to their character and their
disclosed outfit (S32-c). Lakeside adds nothing to that path. C2 proves it: all ten people, each drawn as their own
character in their starting outfit, by a probe that prints every figure's `look` key, character id and garment ids
and compares them with world data and `cast.yaml`.

### 5.2 Extras

The Lakeside depiction declares its extras pool as **the café town's cast** (E-2): S26's `cast.yaml` group for
`social-cafe`. A disclosed person whose look is unbound draws an extra from this pool (S26 §4.4). The pools are
disjoint by construction: a test reads both groups and fails if any key is in both (AC-g-3).

### 5.3 Decor figures (5, within the step's ≤ 6)

| Figure | Where | Does |
| --- | --- | --- |
| 2 walkers | the shore trail (S33-d §5.4), west of the shore door | walk out and back at 1.3 m/s (a walking pace; S26 clips) |
| 1 walker | the lane, east to west | walks through once a minute |
| 2 standing | the square, near the well | stand and talk (the idle-talk clip; no speech) |

**No decor figure sits.** Under S34 whether a sit point is taken is the server's, and in the fallback rule ties
are broken by entity id, which a decor figure does not have. A decor figure drawn on a sit point would show a seat
as taken that the world says is free (I-S33-5). Every Lakeside bench carries sit points (S33-f §5.2), so there is
no bench a decor figure could use. The step's "a person on the square's bench" is therefore not drawn (a recorded
deviation, F-S33g-4); only world persons sit.

Every decor figure is drawn from the extras pool, never a Lakeside person (AC-g-3).

### 5.4 Activities only from disclosure (`ARC-173`)

- A client draws an activity (fishing, rowing, baking at the oven) only from a disclosed component or event that
  names it. An undisclosed Process is not drawn.
- **Fishing (Q-S33g-1, recommended A):** until the fishing pack discloses its angler to persons present, no figure
  of another person is drawn fishing. The player's own figure may be drawn fishing from its own `fishing-started`
  fact while its `catch` is running (the angler's own observation), with a rod prop at the water's edge.
- **Rowing:** never (F-S33-6).
- The check (AC-g-4): the figures' animation log in the probe shows no `fish` or `row` clip on any figure whose
  disclosure does not name it; a planted call (M-g-2) fails by name.

### 5.5 The crowd perf view

`lakeside crowd`: the ten people and the decor figures half in frame at 3–15 m on the lane, from (−6, 1.66, 2) at
yaw 60°. Counts recorded locally; the timing is the operator's (S33-j).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-g-1 (I-S33-1) | No kernel, contract, System or world change | diff scan |
| I-g-2 (S26 X-16) | No placeholder figure: never CharMorph, never a capsule | AC-g-2 |
| I-g-3 | The Lakeside cast and the extras pool are disjoint | AC-g-3 |
| I-g-4 (I-S33-5) | No activity drawn without a disclosed state naming it | AC-g-4 |
| I-g-5 | One standard: every Lakeside body is S32-d's accepted, template-v2 character | AC-g-1 (the character's manifest names its batch and gate verdict) |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Probe | AC-g-1, -2, -4 (`--world=lakeside --cast`) | nightly `clients-probes`; local one at a time |
| Data | AC-g-3 (pools disjoint) | `tools/cli/tests/cast_pools.rs` or S26's cast test (CI `core`) |
| Perf | AC-g-5 counts | local counts; the operator's timing in S33-j |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-g-1 | All ten Lakeside people are drawn as their own S32-d characters in their S32-f starting outfits | §12.1 checkpoint |
| AC-g-2 | A planted person with an unknown look is drawn as an extra from the café cast; S26 X-16's placeholder plant fails by name | S26 X-16 |
| AC-g-3 | No Lakeside person is in the extras pool or drawn as a decor figure | §12.1 adversarial |
| AC-g-4 | No fishing or rowing clip on a figure without a disclosure naming it; M-g-2 fails by name | I-S33-5 |
| AC-g-5 | `lakeside crowd` counts recorded under step §8.1's advisory figures | §8.1 |

---

## 9. Commit plan

Procedure: [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md) §9.0, from
`/Users/yuema137/mineworld-worktrees/impl-s33g`.

- **C0 — Base, prerequisites, ledger.** S26-a, S26-c2, S32-c, S32-d (Lakeside batch accepted), S32-f merged; the
  fishing pack's pinned commit and whether a disclosing release exists (Q-S33g-1). `docs(plan): S33-g — open the
  ledger at the base`.
- **C1 — `ARC-173`.** `docs: ARC-173 — draw an activity only when the world discloses it (S33-g)`.
- **C2 — The cast probe.** `--cast` mode; AC-g-1. Mutation **M-g-1** (Nils's look key changed in a planted copy) →
  `--cast` names Nils as drawn as an extra. `test(3d): Lakeside's ten people in their own bodies and outfits (S33-g)`.
- **C3 — Extras and decor.** §5.2, §5.3; AC-g-2, AC-g-3. `feat(3d): Lakeside's extras are the café town's cast (S33-g)`.
- **C4 — The activity rule.** §5.4; AC-g-4; M-g-2 (a `fish` clip on Dario) red. `feat(3d): no activity without
  disclosure; the player's own catch (S33-g)`.
- **C5 — The crowd view.** AC-g-5. `test(3d): the Lakeside crowd perf view (S33-g)`.
- **C6 — Close-out.** Push; PR; CI; repair; train-ready. `docs(plan): S33-g close-out`.

Each commit tracks implementation, deterministic validation and LLM review separately (S33-d §9).

---

## 10. Requirements this PR places on other PRs

- **R-fishing-g-1 (on the fishing pack's owner, through the coordinator; Q-S33g-1 A).** A release of
  `mineworld-pack-fishing` that discloses `Angler` (the spot, no due time) to persons present at the spot, and a
  MineWorld PR re-pinning it under `ARC-66`, with Lakeside's re-baseline recorded there.
- **R-S32d-g-1 (on S32-d).** Lakeside's batch includes all ten people, each with a starting outfit.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §5 | the fishing rule as ruled in Q-S33g-1 (F-S33g-1); "a person on the square's bench" dropped from the decor figures (F-S33g-4) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-g — Lakeside's people in 3D
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33g-people.md
WORKTREE / BRANCH:       /Users/yuema137/mineworld-worktrees/impl-s33g, mvp0/pr-s33g, one session
START CONDITION:         per the header (NONE met at 2026-10-18)
APPROVED SCOPE:          §2.1 items 1–6
FROZEN INVARIANTS:       step-33 I-S33-1, -5; S26 §4.4, E-2, X-16; §6 I-g-1…I-g-5
NEVER:                   as pr-s33d §12; a placeholder figure; a Meshy call; editing the fishing pack
MATERIAL STOPS:          a body missing or below the gate (the fix is S32's); a needed world change
ENDPOINT AUTHORITY:      implementation, commits, push, PR, CI repair: authorized once started;
                         merge: through a merge train under the charter
STOP CONDITION:          train-ready
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-g-1 | S32's route B fails; bodies late (R-S33-4) | only this PR waits; the release waits for people |
| R-g-2 | The fishing pack never discloses | Lakeside's people are seen standing on the pier and the shore, not fishing; honest, and the operator rules it (Q-S33g-1) |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33g-1 [operator — fishing you can see]** | The fishing pack tells only the angler that they are fishing; nobody else can see it (F-S33g-1). (A) Ask the pack's owner for a release that discloses the angler to people present, re-pinned under `ARC-66` in its own MineWorld PR; until then draw nobody fishing but yourself. (B) Accept that others are never seen fishing. (C) Infer fishing from someone standing still at the water | **(A).** A fishing village whose fishermen are never seen fishing is a hole, and the fix belongs to the pack that owns the activity. (C) draws a behaviour the world never stated, which the project forbids. *Chinese:* 钓鱼包目前只告诉钓鱼的人自己在钓鱼，别人看不到。请包的维护者出一版"向在场的人公开钓鱼状态"，在那之前只画自己钓鱼——同意吗？ |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C6 | — | [ ] | [ ] | [ ] |

Mutations: M-g-1, M-g-2 (none run yet). Findings: F-S33g-1…F-S33g-4 (§3.2).

**Design review (2026-10-18).** One independent Opus reviewer, verdict REQUEST CHANGES, no blockers; the fishing
pack's "discloses nothing" confirmed. Applied before the PR opened: no decor figure sits (F-S33g-4); A-1 separates
homes from starts.
