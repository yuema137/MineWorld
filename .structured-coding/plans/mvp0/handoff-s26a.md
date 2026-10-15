# Handoff — mvp0: PR S26-a implementation (the `appearance` System Pack and the cast binding contract)

A continuation aid, never a design authority. The authority is
[`pr-s26a-appearance.md`](pr-s26a-appearance.md) (DESIGN FROZEN 2026-10-14, revision 2), with its live
ledger in §15.

```text
PROJECT / PR        MVP-0 · step-26 / PR S26-a — systems/appearance (mineworld-appearance), installed,
                    looks in social-cafe, market-town, lakeside; cast binding contract (ARC-92), check_cast.py
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s26a-appearance.md; ledger §15
RELATED / BINDING   step-26-cast.md (§§4.1–4.4, 8, 9, 12, 12.1, 12.2, 13, 15, 16.1); overall.md;
                    docs/MODULE_SPEC.md §§4.1, 6.1; docs/DECISIONS.md ARC-31, ARC-33, ARC-35, ARC-46;
                    systems/naming (pattern); clients/2d/PRESENTATION.md; scripts/baselines.txt;
                    .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
BRANCH / WORKTREE   mvp0/pr-s26a in /Users/yuema137/mineworld-worktrees/impl-s26a (sole writer)
BASE                origin/main @ adf6d49 (PR #195, design frozen at 9302ef5)
CARGO_TARGET_DIR    /tmp/impl-s26a-target (operator instruction); release binary there
APPROVED SCOPE      design §2.1 items 1–9 as ruled in §14.1; diff limited to §9's file lists
FROZEN INVARIANTS   §0 rulings; §6 I-S26a-1 … I-S26a-10; §8 AS26a-1 … AS26a-13
ENDPOINT AUTHORITY  implementation + local validation, semantic commits, branch push, PR create/update,
                    CI repair bounded by F-S26a-3: authorized (design §12). Merge: NEVER by this session
SEQUENCE            C0 … C9 (§9)
STOP CONDITION      READY FOR OPERATOR REVIEW — DO NOT MERGE
BASELINES           main's scripts/baselines.txt is stale (F-12d-1); base-vs-new comparison is authoritative.
                    Q-S26a-5: whichever of P6a and S26-a merges second re-records baselines.

RULINGS (§14.1, copied verbatim)
  Q-S26a-1  Yes. appearance goes directly after schedule in market-town; AC-1's allow-list stays unchanged
  Q-S26a-2  Yes. Add "look" to the 2D HUD's SHOWN_ELSEWHERE (C7), pinned by the stub panels test
  Q-S26a-3  Yes. scripts/check_cast.py and its self-test run in the fast layer (C8)
  Q-S26a-4  Keep the generic rendering; no appeared renderer now (C7 is a test only)
  Q-S26a-5  Yes. Whichever of P6a and S26-a merges second rebases and re-records (market-town keeping
            Social Café's list as its prefix)

LOAD POLICY         R-LOAD-1 (coordinator, 2026-10-14): no local workspace test; run only touched crates + fmt +
                    clippy -D warnings; full gate is CI on the exact head, recorded as
                    "workspace gate: CI run <id> on <sha> (R-LOAD-1)"; long records one at a time with load recorded
BASE CAPTURES       C0 done: target/scratch-s26a/{social-cafe,market-town,lakeside}.base.{txt,behaviour};
                    behaviour lines 20 / 24 / 24; genesis 53 / 134 / 82 (bodies-yard 47)
CURRENT CHECKPOINT  C0 committed (00a5a50); C1 done (b9ef144: MODULE_SPEC §4.1 appearance row and example,
                    ARC-91); origin/main merged into mvp0/pr-s26a (ee7a361, no conflicts); C2 done
                    (systems/appearance crate, 4 tests green, M-S26a-1/2/3 red-then-green, commit 7496847);
                    C3 done (installed + refusals tests green; M-S26a-1 red-then-green; commit 318f6ff);
                    C4 done (looks in three worlds, F-S26a-3 literals, golden re-captured; counts 65/146/92/47);
                    C5 next
NEXT ACTIONS        C5 … C9 per §9.0 procedure; C9 pushes and opens the PR
