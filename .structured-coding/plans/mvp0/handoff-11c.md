# Handoff — PR 11c (complete affordances)

A continuation aid, never a design authority. The authority is
[`step-10-market.md`](step-10-market.md) §4.3 (design, checkboxes), §9.3 (evidence), §12.0 (freeze
record, binding) and §14 (execution contract).

```text
PROJECT / PR        MVP-0 · Step 10 / PR 11c — complete affordances (F-3)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-10-market.md §4.3, §9.3
RELATED / BINDING   step-10 §§1.3, 2.3, 3 (SD-9 … SD-12, SD-12 as amended by QS-20), 8.4, 10 (QS-4,
                    QS-20 … QS-25), 12.0, 14; CORE_CONCEPTS §15; MODULE_SPEC §5; server/PROTOCOL.md
                    §§5–6; DECISIONS ARC-23, ARC-26, ARC-27, ARC-33, ARC-35
BRANCH / WORKTREE   mvp0/pr-11c-affordances in /Users/yuema137/mineworld-worktrees/s9-11c (this session
                    only; 11b runs in parallel in s9-11b — never touched)
IMPLEMENTATION BASE main @ da31613 (merge of #38, the freeze of 11b and 11c)
SCOPE               C-C1 … C-C6; contracts/ only observation.rs + one pub(crate) constructor in
                    action.rs; no kernel/, worldpack/, authoring/, tools/cli/src, GDScript change
FROZEN INVARIANTS   I-2 (no 11c allow-list entry), I-4 (sha ad49c723…c64b; AC-13/AC-15 transcripts),
                    I-5, I-8, I-9 (constants fixed by C-C6's criterion); §12.0 merge rules
ENDPOINT AUTHORITY  (source: the coordinator's kickoff for this session, 2026-10-07)
  implementation + local validation   authorized
  semantic commits, branch push       authorized
  PR creation / update                authorized, marked READY FOR OPERATOR REVIEW
  scratch branch for chimes (C-C6)    authorized, local; deleted after evidence
  rebase onto main if 11b merges first, then force-push this branch   authorized (§12.0), only then
  CI repair                           N/A — no CI workflow
  merge                               operator only, with a merge commit; never inherited
SEQUENCE            C-C1 → C-C2 → C-C3 → C-C4 → C-C5 → C-C6, each committed and pushed
VALIDATION BUDGET   targeted tests; 300-day social-cafe at C-C4 and final; scratch chimes 300-day run
                    (≤ 2 if rate lowered); one full workspace gate on the final head (background)
STOP CONDITIONS     market word needed; contracts/ beyond the field + constructor; kernel/ change;
                    RuleController / --agent change; GDScript change; social-cafe sha or AC-13/AC-15
                    transcript change; C-C6 failing at every rate ≥ 5
```

## Current checkpoint

**READY FOR OPERATOR REVIEW — CLOSED / AWAITING OPERATOR ACTION.**

Commits: C-C1 ca2e460, C-C2 7922cb2, C-C3 d2f6c8c, C-C4 fd8c9cb, C-C5 6486d5b, C-C6 20cf29b (final
executable head), then this Markdown-only close. All six mutations fail as designed and are reverted.
Band measured on a scratch install: PASS at 20 — frozen (E-C6). Full gates on 20cf29b PASS
(E-C-final: 443 passed, 0 failed; kill_and_resume PASS; sha = E-0; validate identical). The scratch
branch was local only and is deleted. The PR head and number are in the PR itself.

If 11b merges first (§12.0): rebase this branch onto the new main (never merge main in), move 11c's
`PRECURSORS` base to that main in the same change, keep the rows in order 11a, 11b, 11c with 11b's
structure, re-run the sha, the planted violations and the full gate on the rebased head, record them
in §9.3, then force-push this branch — only then.

Merge: operator only, with a merge commit.

## Notes for a resumed session

- Logs live in `/tmp/11c/`; durable values are copied into §9.3.
- `cargo` is `$HOME/.cargo/bin/cargo`.
- Run anything over ~2 minutes in the background.
