# Handoff — S26-b1 (route D+ in the repository)

This file is a continuation aid. The authority is `pr-s26b1-pipeline.md` (DESIGN FROZEN, revision 2).

- **Project / PR:** MineWorld mvp0, S26-b1 — the protagonist's character pipeline, ported and parameterized.
- **Primary design:** `.structured-coding/plans/mvp0/pr-s26b1-pipeline.md` (§§5, 6, 8, 9, 12, 14.1).
- **Binding docs:** `step-26-cast.md` (§§5, 6, 10, 12, 12.1, 12.2, 15, 16.1); `docs/references/CHARACTER_ROUTE_D_PLUS.md`;
  `docs/DECISIONS.md` (DEP-8, ARC-9, ARC-17, ARC-19, ARC-23, ARC-24); `CLAUDE.md`; `docs/ENGINEERING_STANDARDS.md`;
  `docs/ENGINEERING_RULES.md`; `docs/REUSE_POLICY.md`.
- **Worktree:** `/Users/yuema137/mineworld-worktrees/impl-s26b1` (held by one session).
- **Branch:** `mvp0/pr-s26b1`, created from `origin/main`.
- **Base:** `f772a18` (Merge of PR #197, `origin/main` at the time of B0).
- **HEAD:** recorded in the §15 ledger at each commit.
- **Approved scope:** design §2.1 items 1–9.
- **Frozen invariants (copied from §6):**
  - I-B1-1: no file under `clients/ server/ systems/ kernel/ contracts/ worldpack/ persistence/ cognition/ sdk/ worlds/`
    changes. Nothing outside `tools/character_pipeline/` imports `mwcast`.
  - I-B1-2: No Meshy credit is spent and no OpenAI request is made by any agent in this PR.
  - I-B1-3: No key is read, printed, logged, copied or committed; nothing under the runtime directories names the secrets path.
  - I-B1-4: No GLB, PNG, `.blend`, Meshy download, `request.json` or `task.json` is committed.
  - I-B1-5: Every pipeline output goes to a scratch root outside the repository.
  - I-B1-6: Every gate bound is a literal with a source; none is computed from the candidate.
  - I-B1-7: `meshy_d.glb`, `meshy_d_base_color.png` and their `.import` files are unchanged.
  - I-B1-8: Blender scripts are GPL-2.0-or-later and live only in `tools/character_pipeline/blender/`.
  - I-B1-9: Per-character numbers live in `fit.yaml`; the Blender scripts contain no number typed for one character.
- **Endpoint authority (sources):**
  - implementation and local validation: authorized once frozen (primary's freeze relay; design §12);
  - semantic commits: authorized (working rules §14);
  - branch push: authorized (working rules §21);
  - PR creation and update: authorized (working rules §21);
  - CI repair to review readiness: authorized (working rules §21);
  - merge: never by the implementation session; only with the operator's explicit authorization (working rules §22).
- **Sequence:** B0 … B11 (design §9).
- **Budget:** unit tests unrestricted locally; B7 smoke ≤ 2 min; B10 Gate ≤ 30 min wall in total; 0 Meshy credits;
  $0 OpenAI. Gate 1 not required (no LLM-facing change).
- **Scratch root for B7 and B10:** `/Users/yuema137/mineworld-demos/cast-scratch/s26b1` (outside the repository; created).
- **Checkpoint:** READY FOR OPERATOR REVIEW after CI is green on the final head. DO NOT MERGE.
- **Progress:** B0 (`ff0cab0`), B1 (`f8b64cc`), B2 (`9673735`), B3 (`39df3cd`), B4 (`744626e`), B5 (`88ba3c1`),
  B6 (`a1e8309`), B7 (`c1b5d29`), B8 (`7a58b04`), B9 (this commit) done. B10–B11 open.
- **Open finding for the primary (F-B1-B3-1):** the accepted `meshy_d.glb` fails G-6(b) under the frozen bounds
  (`mixamorig:Spine` 63.5 mm and `mixamorig:Spine2` 46.9 mm outside the body). Bounds are not changed. Decision needed
  before B10 step 5 (AB1-10).
- **Current state (2026-10-14, after ruling F-B1-B10-2 option (b)):** `body_g5.glb` committed under `rig_source/` with
  PROVENANCE; stage 7 reads its front surface. B10 run once: AB1-8 weights FAIL (0.0932; F-B1-B10-3), AB1-10 FAIL on G-6(b)
  (F-B1-B3-1), AB1-11 INCONCLUSIVE on the albedo name (Godot). Determinism, counterfactual, mask and texture PASS. Stopped
  for the primary. No B11, no push, no PR, no CI, no READY.
- **Previously (historical):** B10 step 1 is a MATERIAL STOP (F-B1-B10-1, design §15): the `vitruvian.glb` and `body_g5.glb` armatures
  differ in four foot and toe rolls (worst 2.14e-4 rad against the 1e-5 bound). Waiting for the primary session's
  decision. No further Blender or Godot step, no PR, no push.
- **Ruling F-B1-B10-1 (primary, 2026-10-14) applied:** accept the vitruvian armature; four-bone roll tolerance ≤ 2.5e-4 rad
  (design §14.1). Run r1 then exited 1 at `fit_bind` (F-B1-B10-2: `vitruvian.glb`'s Body has no vertices at the head top,
  z 1.092; `body_g5` has 16). Stopped again, awaiting the primary's decision (options and recommendation in the ledger).
  No push, no PR, no CI. Opus review of B3, B7 and B10 is required before ready.
- **Next action (superseded):** none until the primary decides F-B1-B10-2.
- **State after F-B1-B10-3 (2026-10-14, Opus session, sole writer of this worktree):** cause found (stage 6's transform
  handed to `fit_bind` at full precision; D+ typed the 4-decimal print). Fix `c588946`; AB1-11 fix in the same commit. Gate
  re-run once (`r3`): AB1-7, -8, -9, -11, -12, -13 and AC-S26-3 PASS; AB1-10 FAIL on G-6(b) only, with a measured finding
  for the primary (design §15). Next: B11 close-out, push, PR, CI on the exact head; READY FOR OPERATOR REVIEW, DO NOT MERGE.
- **After rulings D-B1-1, R-B1-G6, F-B1-B10-4 (2026-10-14):** PR #211 open. The one cognition test made robust; G-6(b) is
  the depth-ray inside test and passes on `r3` (AB1-10 PASS); F-B1-B10-4 is an S26-b2 follow-up. #204 (S27-c) edits the same
  test: whichever merges second rebases. Next: CI green on the exact head, gate comment, report.
- **Load rule (coordinator R-LOAD-1, 2026-10-14):** no workspace cargo test locally; the full gate is CI on the exact
  head, recorded as "workspace gate: CI run <id> on <sha> (R-LOAD-1)"; parity and perf runs one at a time with the
  load average recorded; no interim reports; report once at READY or at a real blocker.
- **Standing verification notes (B1, 2026-10-14):** Meshy §2.4 unchanged in substance; Winter INCONCLUSIVE; AccuRIG
  INCONCLUSIVE (404); RigNet is GPL-3.0; these are recorded in the design §15 evidence.
- **Stop conditions (design §12, MATERIAL STOPS):**
  - B1 step 2: Meshy's §2.4 changed in substance, or Winter's figure contradicts step-26's numbers.
  - B10: the armatures of `vitruvian.glb` and `body_g5.glb` differ beyond 1e-5; AB1-8 cannot hold after a localized
    port review; the stored-once rule of §5.9 fails (fallback touches Q-S26-5); any need to spend a credit or a dollar;
    any edit under a forbidden directory (I-B1-1).
- **NEVER (design §12):** `python3 -c`; `sed -i`; heredoc writes; `curl`; `git add -A`; sourcing the secrets file or
  reading any key or `.env` file; `cast.py meshy` without `--dry-run`; any OpenAI request; writing into
  `/Users/yuema137/mineworld-demos/i23d/` or `scratch-character/` (read only); committing any binary; edits outside
  approved scope; changing a bound after a Gate run.
