# Handoff — PR IL-b (S17: the Interaction List schema and its first sections)

A continuation aid only. The authority is `step-18-interaction-list.md` §12 (design, contract §12.12,
ledger §12.13, deviations §12.14).

```text
PROJECT / PR         MVP-0 · S17 / PR IL-b — the Interaction List schema (ARC-63, ARC-64, ARC-65, DEP-28)
                     and its first sections
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/step-18-interaction-list.md §12
RELATED / BINDING    same file §§4, 5, 6, 11; overall.md "The World Interaction List"; step-19 §4.5, §6.5,
                     QTW-7, QTW-15; step-11 §19 (12d); DECISIONS ARC-5, 23, 25, 27, 29, 31, 33, 34, 55,
                     61, 62, DEP-10; MODULE_SPEC §§3.1, 4, 4.1; CLAUDE.md §§2–4
BRANCH / WORKTREE    mvp0/pr-il-b-interactions · /Users/yuema137/mineworld-worktrees/impl-il-b (one writer)
BASE                 main @ aa74b32 (code identical to the freeze base 77a8717: only plan documents moved;
                     12d not merged at start)
FROZEN               DESIGN FROZEN 2026-10-08, primary session (#89)
APPROVED SCOPE       §12.1's change set; IB-C1 … IB-C11; SD-IB-1 … SD-IB-17 as answered by QIB-1 … QIB-14
FROZEN INVARIANTS    §12.12: no diff under kernel/ contracts/ persistence/ server/ clients/ worlds/; no System
                     Pack diff beyond presence's offer refusal, conversation and group-activity; root
                     Cargo.toml and Cargo.lock unchanged; IB-1 equal to E-IB-0; ac1_composability,
                     precursor_vocabulary, seam_vocabulary unedited and passing; an unconfigured world seeds
                     exactly what it seeded; a list cannot grant
ENDPOINT AUTHORITY   implementation + local validation: authorized (primary session's freeze; operator's
                     kickoff 2026-10-08)
                     semantic commits, branch push: authorized (same)
                     PR creation/update, marked READY FOR OPERATOR REVIEW: authorized (same)
                     merge: operator only, with a merge commit
SEQUENCE             IB-C1 … IB-C11; E-IB-0 before IB-C2
VALIDATION BUDGET    four 300-day town runs (E-IB-0 ×2 on the base, IB-1 ×2 on the head); M-IB1a/b and
                     IB-14 on 30-day social-cafe runs, uncounted (operator's budget ruling, 2026-10-08);
                     bodies-yard 30-day, long_run, long_run_objects twice each; one full gate on the
                     final head
PLATFORMS            operator requirement 2026-10-08: macOS, Linux and Windows; no Unix-only assumption
                     added; data: attachments and YAML reading handle Windows paths and CRLF; existing
                     Unix-only assumptions recorded as findings with an owner
STOP                 PR IL-b READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Lifecycle: APPROVED ON REVIEW — AWAITING THE PRIMARY SESSION'S MERGE (2026-10-09)

- PR #102 approved by the primary session (its own mutation — `permits` always allowing — was caught by
  name by `a_forbidden_pair_is_refused_at_dispatch_and_shown_unavailable_for_that_reason` and
  `a_forbidding_default_admits_only_what_a_rule_permits`; D-IB-12 and D-IB-13 accepted). The primary
  session merges, with a merge commit; this session does not.
- All IB-1 … IB-15 PASS (§12.13 E-IB-0 … E-IB-15). Town runs: 5 (4 budgeted + the fifth, authorized:
  market-town 300 days = TW-a's baseline 24a95d2a…d270). F-IB-16 is owned by S13's 13w (ruling).
- Code head of the gate (E-IB-14): 196cbc3; CI green on 9862663 (pull_request run 37981060252).
- This commit merges origin/main bc4f8e7 (#98 S10 P3 Python SDK, #104 S11-D admin, #106, #107, #108
  docs): one DECISIONS.md conflict (ARC-63 … DEP-28 against P3's ARC-56 … block), both kept; no Rust
  simulation path changed on main (sdk/python, server/, tools/cli/src/{main,serve}.rs only), so the
  byte-identity evidence stands. Checked after the merge: fmt; clippy -D warnings for cli and server;
  cli --test interactions 4, --test configure 3; doc checks 192 / 82 distinct. CI on the new head is
  reported to the primary session.

## Next actions

- None for this session beyond reporting the new head's CI. After merge: the primary session owns the
  step and overall updates (§12.12's POST-MERGE owner).
