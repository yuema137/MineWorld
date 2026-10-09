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

## Current checkpoint (2026-10-09)

- IB-C1 … IB-C11 implemented and validated; IB-1 … IB-14 PASS (§12.13 E-IB-0 … E-IB-12); town runs
  4 of 4 used; origin/main a30755e merged (35885f2). Next: the full gate (E-IB-13), push, PR marked
  READY FOR OPERATOR REVIEW, CI on the exact head. Do not merge.

## Earlier checkpoint

- E-IB-0 captured on the base (two of four town runs used). Base binary: target/il-b/base-mineworld.
- IB-C1 … IB-C8 committed and pushed (head 81dba89): specs; authoring context/classes/attachments;
  worldpack framework keys/attachments/policy; presence Offer::refused; sdk interactions (IB-6 PASS,
  M-IB6); test-tuning schema proofs (M-IB5a/b/c, M-IB10); cli biography + `mineworld interactions`.
- Deviations D-IB-1 … D-IB-11 recorded in §12.14. No material stop so far.
- The machine is heavily loaded by other sessions (load 100–250): IB-14's walls will need a quiet
  window, else INCONCLUSIVE per §12.5.

## Next actions

- IB-C9 conversation's `gap` section (VERSION 2); IB-4 in worldpack/tests/interaction_sections.rs with
  M-IB4a/b; IB-C10 group-activity `invitation_lifetime` + Invitation.until + rule controller; IB-11's
  configured half in tools/cli/tests/interactions.rs; IB-C11 runs (IB-1 two town runs left; IB-2, IB-3,
  IB-8, IB-14, M-IB1a/b, M-IB2, M-IB3, M-IB8), the gate, merge origin/main, PR.
