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
- **Progress:** B0 (`ff0cab0`), B1 (`f8b64cc`) done. B2 committed in the next commit.
- **Next action:** B3 — static gates G-1…G-6 and G-14 (design §9, B3). Opus review of B3 is required before ready.
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
