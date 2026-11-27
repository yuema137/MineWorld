# CI P0 recovery handoff — 2026-11-22

Status: READY FOR COORDINATOR REVIEW. Independent red-team functional review PASS at ce69e9e5; its sole required docstring correction is committed locally at 2adf50427e920dad2284c7e484ae310146bee529 with executable parity verified. No merge or publication authorization is implied. This is a continuation/evidence aid, not a second design authority.

## Identity and authorization

- PR: https://github.com/yuema137/MineWorld/pull/523 (OPEN in live read).
- Public PR head remains `ci/no-fail-fast`, `ce69e9e5210c1ca11ed8637b943cc72aea38d5a0` (not pushed). Final local candidate: `recover/ci-p0-docstring`, `2adf50427e920dad2284c7e484ae310146bee529`, isolated shared clone `/tmp/mineworld-recover-ci-p0`, directly based on ce69e9e5. Executable payload is unchanged from the reviewed/tested ce69e9e5.
- Original base / current merge-base: `315a50a9cdcc7d9a0472a647844eea342faa2a0f`.
- Live origin/main: `2b4d36fac345b9325a91fd4313d40e3633e728b0`; verified using `git ls-remote`, not stale tracking refs. Only intervening change in P0's three paths is the additional `ci_gdparse.py --self-test` entry in `ci_layer.py`; integration must retain it.
- Authority: `pr-ci-test-economy.md` §1.1, §5 P0, §6 Q-4; current recovery dispatch; CLAUDE.md §3.4 local-first review. Original approval context is preserved in [approval.txt](ci-p0-recovery-evidence/approval.txt): primary session line 26848 asks the CI-economy questions and line 26860 (2026-10-27T21:23:54.135Z) says `全按推荐`. The contradictory old “Not frozen” suffix is stale; no missing approval was found.
- Scope: recover/readiness-review P0 only. P1 kickoff prepared separately; no P1 implementation in this session. No subagents, paid calls, push, PR update, merge or workflow dispatch performed.

## Result and preserved source state

Existing P0 already implements the bounded failure-visibility fix: shard cargo commands use `--no-fail-fast`; the layer continues after test failures, reports failed commands/tests, returns the first failing status, and clears a prior summary before a new layer. Docker composite action appends that summary. Repeat mode does not duplicate the flag.

Initial audit required no recovery. The subsequent reviewer-requested repair changes only the opening module docstring in scripts/ci_layer.py, in the isolated clone; no executable changes or rebase were made. Source checkouts were read only. [P0 inventory](ci-p0-recovery-evidence/p0-proof-inventory.json) and [P1 inventory](ci-p0-recovery-evidence/p1-inventory.json) include HEAD, tracked/staged diff and all untracked status; both were clean. Existing commits remain preserved. [P0 patch](ci-p0-recovery-evidence/p0.patch) is the review diff.

DO NOT SHIP `scratch/no-fail-fast-proof` at `b05305bbb9a04f9fa9e733bbbd99526acef4f6dc`. Its deliberate-failure delta is preserved separately in [scratch-proof.patch](ci-p0-recovery-evidence/scratch-proof.patch); none of it entered P0. The old P0 checkout is on this scratch branch, not on the PR head.

## Validation

- PASS: exact-head `ci_shard.py --self-test`; deterministic partition/completeness, doc-only and no-fail-fast checks.
- PASS: `git diff --check ce69e9e5^ ce69e9e5`.
- PASS: bounded real Cargo fixture using byte-identical exported P0 `ci_layer.py` and `ci_shard.py`. Two integration binaries deliberately panic, then a doctest deliberately fails; the later command runs, all three failed tests are reported, and layer returns 101. A non-test command exits 7 and stops the layer; a successful rerun removes stale failure summary; repeat commands carry exactly one no-fail-fast flag. [Proof script](ci-p0-recovery-evidence/prove.py), [final log](ci-p0-recovery-evidence/proof.log).
- First harness attempt was INCONCLUSIVE as a complete check: it expected literal `Doc-tests` in the rendered summary, whereas the audited parser intentionally extracts the crate name. The real three failures and continuation had run correctly. Corrected only the external harness expectation to the exact crate/test identity and reran; [first log](ci-p0-recovery-evidence/proof-attempt1.log) retained. No repository tests/assertions were changed.
- Cost: two dependency-free tiny Cargo fixture runs, each seconds, through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`; no full workspace suite. The wrapper exited and released this task's slot; no process remains owned by this task. Other lanes may hold slots.
- Export fingerprints (SHA256): ci_layer.py `2afa010d43d797ff20ed3578c2f28b710aa61f1977d0e1f641413629a2569c2f`; ci_shard.py `5d76c15cb290be01bfdcc24903f77a8eda7bf89a9d6025d52703a4043b86c122`. Both compared byte-for-byte with `git show ce69e9e5:scripts/<file>` after validation.
- Full suite NOT RUN: narrow CI defect with real bounded failure proof; current dispatch forbids workflow dispatch and requires prompt local-first review. Gate 1 NOT REQUIRED (no model semantics). This fixture is bounded real subprocess evidence, not proof of the MineWorld full suite or all OSes.
- Remote read: https://github.com/yuema137/MineWorld/actions/runs/40586369072 reports SUCCESS at exact `headSha=ce69e9e5210c1ca11ed8637b943cc72aea38d5a0`, event pull_request. Fast passed. Full test matrix jobs SKIPPED; green aggregate test jobs are deferred gates, not executed test-suite passes. PR has no comments despite its body promising a scratch-proof URL there; therefore no remote scratch proof is claimed.

## Logic and maintainability review

Independent review found functional PASS with one required docstring correction; that correction is now complete. See [review](ci-p0-recovery-evidence/red-team-review.md). Module boundaries stay in CI scripts/composite action, without runtime dependencies or simulation changes. Existing shard selection, test sets, seeds, worlds and assertions are unchanged. Non-test/build failures retain fail-stop behavior. Helpers are small and directly own streaming, parsing, summary and classification; no inheritance or speculative abstraction was added.

Existing debt: ci_layer.py was already a large centralized command table; now 729 lines (roughly +70 net), below the strong 800-line warning but above the 500-line review trigger. run() remains about 50 lines; helpers preserve conceptual locality rather than arbitrary splitting.

New nonblocking concerns for red-team disposition: stream() retains each command's complete output in memory; this is proportional to command log size and not bounded. It is not evidence of a failure at observed scope, but P1's JUnit migration is the appropriate opportunity to avoid whole-log retention. is_test() uses membership matching to recognize the existing repeat wrapper, which is broader than a command-prefix predicate; audited existing command shapes are intentional. The stale opening docstring was the reviewer’s sole required correction and is fixed in final local head 2adf5042. Native summary publication is not added by P0 (Docker composite only); tests still fail and print their results on native jobs. These are disclosed review findings, not a claim that the full CI redesign is complete.

## Next action and ownership

Coordinator receives the final local SHA and [repair patch](ci-p0-recovery-evidence/p0-docstring-repair.patch), and records review disposition in the canonical ledger. [Committed parity evidence](ci-p0-recovery-evidence/docstring-parity.txt) proves identical AST excluding the module docstring and byte-identical executable suffix; diff-check passes and clone is clean. Inherit the bounded ce69e9e5 executable proof without claiming new-head remote CI. No broad tests or heavy slots were needed for the correction. Local packet files are uncommitted and belong to the CI lane; root shared documents and dashboard were untouched. Coordinator owns shared planning/integration and any later publication/merge decision. Reopen this P0 session for a bounded review repair if required. Fresh P1 execution uses [kickoff-ci-p1-recovery.md](kickoff-ci-p1-recovery.md); P0 has NOT merged.
