# Handoff — S32-p, the wardrobe pilot (route B on Vera, Ivan and Hana)

Continuation aid only. The authority is the PR design; this file never overrides it.

| Field | Value |
| --- | --- |
| PROJECT / PR | MineWorld mvp0, S32-p — the wardrobe pilot (route B) on Vera, Ivan and Hana |
| PRIMARY DESIGN DOC | `.structured-coding/plans/mvp0/pr-s32p-wardrobe-pilot.md` (DESIGN FROZEN 2026-10-15; live ledger §16) |
| RELATED / BINDING DOCS | §12 of the design: `step-32-wardrobe.md`, `step-26-cast.md`, `tools/character_pipeline/PIPELINE.md`, `docs/HUMANOID_PROFILE.md`, `docs/VISUAL_FIDELITY.md`, `docs/DECISIONS.md`, `.structured-coding/standards.md`, `CLAUDE.md`, `docs/ENGINEERING_*` |
| Implementation branch | `mvp0/pr-s32p` in `/Users/yuema137/mineworld-worktrees/impl-s32p` (one session) |
| Implementation base | `origin/main @ b0a9d738` (ancestry of #211, `73c5bc2c`, `bd7bc92f`, `d81baeb3` verified, §16) |
| Current HEAD | see `git log -1`; this file is refreshed at each semantic milestone |
| Approved scope | design §2.1 items 1–8 |
| Frozen invariants | design §5 I-p-1 … I-p-13; step-32 §5.2's decision rule; step-26 §5.5; the 52-joint profile |
| Endpoint authority | implementation, local validation, commits, push, PR, CI repair: authorized (design §12; operator freeze "同意冻结并实施"). Heavy local jobs: allowed through `with-slot.sh` (operator, 2026-10-18, "你现在可以跑本机重任务了", relayed; supersedes the same day's "code only" dispatch). Spend: Meshy ≤ 1,200 credits, OpenAI ≤ 70 calls (Q-S32-1, Q-S32p-6). Merge: NOT authorized |
| Sequence | C0 … C8 (design §8) |
| Validation budget | design §12: unit files one at a time; Blender stage ≤ 15 min, pose sheet ≤ 10 min, W-sweep ≤ 30 min; R-LOAD-1 (no local full suites) |
| Stop conditions | design §12 material stops; ledger would pass 1,200 or a phase ceiling; Vera fails her checkpoint; a base-layer pick missing when its spend is due (operator gate, pre-spend page); a licence failing DEP-8 |
| Scratch | `MINEWORLD_CAST_SCRATCH=/Users/yuema137/mineworld-worktrees/cast-scratch-s32p` |
| Keys | `source ~/.config/mineworld/secrets.env && …`; check with `test -n` only; never print |

## Current checkpoint

C0–C7 committed; the live pilot is done. Vera checkpoint FAIL (ledger §16 L-10): route B as built does not reach
the fused Vera's look. `VIS-3D-WARDROBE-P` is delivered at `/Users/yuema137/mineworld-demos/wardrobe-pilot/index.html`
(leads with 不采用, levers L-a…L-e). Meshy 690 of 1,200 charged (account balance 260); image calls 40 of 70.

## Exact next actions

1. Wait for the operator's verdict on `VIS-3D-WARDROBE-P`; it sets `docs/decisions/ARC-138.md`'s status in a
   follow-up commit on this PR before any merge (§8 C8).
2. If a lever is chosen, it is a new design or a bounded revision, not more spend under this PR.

## Background jobs

None. Scratch helpers (not committed): `cast-scratch-s32p/tools/{part.sh,outfit.sh,make_spec.py,build_page.py,
write_report.py,set_pick.py,montage.py}`.
