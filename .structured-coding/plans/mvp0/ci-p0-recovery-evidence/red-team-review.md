# CI P0 independent exact-payload review — 2026-11-22

Reviewed PR #523 executable HEAD `ce69e9e5210c1ca11ed8637b943cc72aea38d5a0` against base `315a50a9cdcc7d9a0472a647844eea342faa2a0f` through git objects and the recovery packet. Verdict: functional local code review PASS; one small documentation correction remains before declaring final readiness. No merge authorization or full-suite acceptance is implied.

## Required bounded correction

`scripts/ci_layer.py` opening module docstring still says the first failure stops the layer. Update it to distinguish continued Cargo test failures from non-test/build fail-stop behavior. This is a stale contract description introduced by P0's behavioral change, not a new design decision. No executable retest is needed for that prose-only correction; preserve exact-head evidence mapping.

## Functional findings

- `ci_shard.commands` applies `--no-fail-fast` to test executions and doctests, not the initial no-run build. Existing partition and test selection remain intact.
- `run` records each failure and continues recognized test commands, stops on other failures, and returns the first nonzero status. Later successes cannot mask earlier failures. Failure summary includes failing commands even when no test name parses.
- `is_test` recognizes the existing `ci_repeat.py ... -- cargo test` wrapper because it uses membership, so P0's current repeat continuation works. This predicate is broader than necessary but commands are a closed, repository-owned set. P1 must not replace it with a predicate that loses wrapper recognition.
- The line parser follows Running/Doc-tests identity and collects harness FAILED rows. It is a reporting aid, not the exit-status authority. Native summary publication is absent; only the Docker composite appends the Markdown summary. Native process/test failures still remain visible and nonzero. Do not claim cross-platform Markdown publishing.
- The summary is cleared at layer start and replaced on failure; successful reruns do not retain old failures. The action uses always() to publish after a failing Docker test step.

## Evidence and limits

Read the real tiny-Cargo proof script and final log: integration tests a and b fail, then the doctest fails, a following marker command runs, and the layer returns 101. Independent non-test exit 7 stops execution; successful rerun clears summary; repeat flag occurs once. The first inconclusive harness assertion is preserved and its correction changes only expected report formatting.

Recomputed SHA256 from exact git objects: ci_layer.py `2afa010d43d797ff20ed3578c2f28b710aa61f1977d0e1f641413629a2569c2f`; ci_shard.py `5d76c15cb290be01bfdcc24903f77a8eda7bf89a9d6025d52703a4043b86c122`. They match the exported fixture source fingerprints. No broad test rerun was performed in this review.

Remote run 40586369072 is reported as fast-only success at this head; skipped full matrix jobs are not full-suite evidence. This review does not relabel them. Preserve main's intervening gdparse self-test entry when integrating. Never ship the deliberately failing scratch-proof head.

## Explicit debt review

The existing centralized command table is inherited size debt; approximately 729 lines now, with cohesive small streaming/report/classifier helpers and a roughly 50-line runner. No new inheritance, runtime-domain coupling or speculative abstraction is introduced. New full-output retention is proportional to command output and is avoidable memory overhead; no observed failure makes it a blocker for this narrow fix. P1 should collect failures incrementally or through its JUnit path rather than retain entire logs. The broad command classifier is acceptable for the audited current shapes but must remain tested when P1 adds new command forms. These are recorded limitations, not a blanket waiver for P1 regressions.
