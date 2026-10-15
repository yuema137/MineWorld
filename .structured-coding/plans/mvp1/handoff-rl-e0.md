# Handoff — S22: PR RL-e0 implementation (the far field is mandatory)

A continuation aid, never a design authority. The authority is
[`pr-rl-e0-far-field-mandatory.md`](pr-rl-e0-far-field-mandatory.md) (DESIGN FROZEN 2026-10-15, revision 2),
with its live ledger in §18.

```text
PROJECT / PR        MineWorld mvp0 visual track (S22), RL-e0 — the far field is mandatory
PRIMARY DESIGN DOC  .structured-coding/plans/mvp1/pr-rl-e0-far-field-mandatory.md; ledger §18
RELATED / BINDING   step-22-realism.md §§6.6, 7, 8, 9; pr-rl-b-3d-budget.md §§0.2, 3, 5; pr-rl-b2 §§0, 10;
                    docs/MODULE_SPEC.md §6; docs/VISUAL_SLICE.md; docs/ART_DIRECTION.md §§12, 21;
                    docs/DECISIONS.md DEP-8, DEP-16, DEP-45, ARC-17, ARC-23; CLAUDE.md;
                    docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
BRANCH / WORKTREE   mvp1/pr-rl-e0 in /Users/yuema137/mineworld-worktrees/impl-rl-e0 (sole writer).
                    The design's §13 names `mvp0/pr-rl-e0`; the dispatch of 2026-10-15 created
                    `mvp1/pr-rl-e0` explicitly ("git worktree add … -b mvp1/pr-rl-e0 origin/main") and
                    replaces that line (working rules, "PR CONTEXT AUTHORITY ORDER").
BASE                origin/main @ 2d07c6c0 (Merge #215, the RL-e0 design freeze)
HEAD                see `git log --oneline -1`; this file is refreshed at each semantic milestone
WORKING TREE        clean at C0
APPROVED SCOPE      design §2.1 items 1–11 (frozen)
FROZEN INVARIANTS   I-FF-1 … I-FF-8; §6's viewpoints, renders, bands and thresholds (N_below 0.90 over
                    [e_h−1.0°, e_h−0.1°], N_above 0.25 over [e_h, e_h+0.25°], 8/255, 2 % exemptions);
                    §8's caps (≤ +24 draws, ≤ +100 000 primitives, ≤ +32 MB video memory per view,
                    ≤ +1.0 ms p95)
ENDPOINT AUTHORITY  implementation + local validation: authorized (dispatch 2026-10-15; §13)
                    semantic commits: authorized (dispatch; §13)
                    branch push: authorized (dispatch: "Push, open the PR")
                    PR creation / update: authorized (dispatch)
                    CI repair to review readiness: authorized (dispatch: "get CI green on the exact head")
                    merge: NEVER by this session ("Never merge"); explicit operator authorization only
SEQUENCE            C0, C1, C2, C3, C3b, C4, C5, C6, C7, C8, C9, C10 (§12)
VALIDATION BUDGET   R-LOAD-1: targeted per-crate tests only; one Godot process at a time; local timings
                    INCONCLUSIVE (counts only); CI on the exact head gates. Two dispatched nightly
                    clients runs (C6 step 3, C8) + the PR's runs
ALLOWED COMMANDS    cargo -p <crate> *; git; gh (never merge); python3 scripts/*; godot --headless
                    --import / --script; ./mineworld-slice *; ./mineworld-2d *; ./mineworld-3d *;
                    target/*/mineworld *; ln -s; mkdir -p; sed -n
NEVER               python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace
                    locally; tuning N_below/N_above or the viewpoints after a run; edits outside scope
MATERIAL STOPS      a hosted runner cannot render the check (R-FF-1); a count cap of §8 exceeded; an
                    operator-measured p95 increment > +1.0 ms; V-1 fails; a needed change to
                    camera_rig.gd (FAR), the protocol, a kernel/contract crate, or a frozen RL-b2 file
                    beyond §14.1's lines; the 2D surround needs camera clamping after all
STOP CONDITION      READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Rulings (design §17.1, verbatim)

Answered 2026-10-15; relayed by the coordinator. **Binding.** The operator answered in the primary session:
"全部按推荐" (everything as recommended).

| Question | Ruled by | Ruling | Applied in |
| --- | --- | --- | --- |
| Q-FF-1 | operator | **N_below = 90 %** over `[e_h − 1.0°, e_h − 0.1°]`, **N_above = 25 %** over `[e_h, e_h + 0.25°]` (land sectors only); the seven viewpoints of §6.1; 8/255 per pixel | §6 (now frozen content) |
| Q-FF-2 | operator | The bay uses `water.gdshader` with San Diego Bay colours, **Fresnel ≤ 0.6**, **no reflections of the town** | §5.4, SD-FF-5 |
| Q-FF-3 | operator | **(a)** The promenade spike stops being walkable; `--drive`, `--shots`, `--portrait` stay | §2.1 item 9, C9 step 4, AC-FF-11 |
| Q-FF-4 | primary | `WorldSummary` pack id **later, with S21**; RL-e0 uses `--world-key` | §2.2, SD-FF-7 |
| Q-FF-5 | operator | **Yes:** a per-PR Linux far-field job, filtered to client and presentation changes (not required until the operator says so) | §2.1 item 11, §6.5, C8 step 4 |
| Q-FF-6 | operator | **Low-rise** distant massing (6–14 m) | §5.6, Appendix B.1 |
| Q-FF-7 | operator | **No fallback** far field; the 2D `--presentation=none` diagnostic mode is **exempt** and says so | §5.1.4, SD-FF-8, I-FF-1 |
| Q-FF-8 | operator | RL-e0 **reserves `station`** at the Santa Fe Depot (277°, 790 m) and leaves **`landmark` unplaced**. The station-and-landmark PR comes **after F**; **the station is the landmark** | §5.1.6, Appendix B.1 |
| Q-FF-9 | primary | **Yes:** the slice (`social-cafe`) uses Market Town's San Diego location | §5.6, Appendix B.1 |
| Q-FF-10 | primary | **Accept:** the slice keeps its natural terminations; RL-e0 adds no invisible wall | Appendix B.1 (`boundary: natural`) |
| Ids | primary | ARC-RLe0-a/b/c → **ARC-102, ARC-103, ARC-104**; next free ARC-105; DEP-59 is taken by the F-tick-1 fix, so a DEP here would be DEP-60 (none is needed) | header, C1, Appendix A.5 |

## Current checkpoint

C0 — ledger and handoff opened against origin/main @ 2d07c6c0. PR #200 (S25 25a) is OPEN, not merged:
C2 step 7 (the generated reference entry) is N/A in RL-e0 and moves to whichever of #200 / RL-e0 merges
second (§14.2). RL-b2 (#212) is an open draft: overlaps per §14.1, mechanical.

### 2026-10-15, after C4 (HEAD d8c7482b)

C0 80afda22 · C1 2b554ed5 · C2 2b6ff714 · C3 2edc512e · C3b 152eba80 · C4 d8c7482b. Working tree clean.
Findings to carry to the report: F-FF-1 (X-2a literal survives: skyline sectors close 360° under the frozen
GAP rule; X-2a′ fails by name), D-FF-1…6 (ledger). Host load 30–120 from other sessions: cargo runs in the
background; Godot one process at a time; a `--script` that fails to compile hangs, so wrap Godot runs in a
kill-after loop (no `timeout` binary on this Mac).

## Next actions

C5 (3D far-field view; design note: proxies must lie beyond the 1 450 m foreground or the near ribbon hides
the bay — record as a deviation), then C6 (with the R-FF-1 hosted-runner dispatch), C7, C8, C9, C10, push,
PR, CI.
