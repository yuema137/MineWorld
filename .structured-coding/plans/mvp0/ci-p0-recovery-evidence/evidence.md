# PR523 P0 evidence packet

This packet preserves actual bounded failure-visibility evidence for design
[CI test economy design #520](https://github.com/yuema137/MineWorld/pull/520) §1.1 F-1 / §5 P0 / Q-4.
It is evidence and continuation state, not a second design or the P1–P5 ledger.

## Identity, authority and scope

Public source: `ce69e9e5210c1ca11ed8637b943cc72aea38d5a0`, branch `ci/no-fail-fast`,
[PR523](https://github.com/yuema137/MineWorld/pull/523), original base
`315a50a9cdcc7d9a0472a647844eea342faa2a0f`. Reused docstring correction:
`2adf50427e920dad2284c7e484ae310146bee529`. The closeout branch descends directly
from these commits, preserving exact public ancestry without a rebase.

The 2026-11-23 coordinator dispatch authorizes the existing bounded correction,
cheap validation, local semantic commits and this minimal durable proof packet.
No public push, comment, PR body edit, workflow dispatch or merge is authorized
in this lane. Coordinator owns publication, independent final review and shared
planning/parent synchronization. No paid calls or child agents; initial budget
30 minutes, no heavy/broad reruns. Local endpoint: prepared candidate awaiting
independent review and concrete operator publication decision.

No executable behavior changes during closeout. Cargo test execution and doctests
continue after failure; builds/non-test failures stop the layer; first nonzero
status remains authoritative. Docker composite publishes the failure summary.
Test selection, assertions, expected values, worlds and seeds remain unchanged.
P1 nextest/archive work is excluded. Never publish scratch proof commit
`b05305bbb9a04f9fa9e733bbbd99526acef4f6dc`.

## Actual proof and provenance

- [prove.py](prove.py) is the original external fixture driver, preserved byte for
  byte, including historical absolute paths. It is an archival script, not a CI
  entry point. Original setup exported the two Python scripts from public source
  into `/tmp/mineworld-ci-p0-audit`; the driver ran with that directory on the
  Python import path and created its `fixture` child. Cargo used a dependency-free
  tiny package, through the shared cast-slot wrapper, on macOS.
- [proof.log](proof.log) records the successful proof: integration binaries a/b
  deliberately fail, doctest deliberately fails, following marker command runs,
  layer returns 101; separate non-test exit 7 stops; success removes stale summary;
  repeat mode has exactly one `--no-fail-fast`. These intentional test failures
  are the expected evidence, not a passing MineWorld full suite.
- [proof-attempt1.log](proof-attempt1.log) preserves the initial INCONCLUSIVE
  harness result. Cargo had exercised all three failures/continuation, but the
  external assertion expected literal `Doc-tests`; the parser reports crate name.
  Only the harness formatting expectation was corrected for the final run.
  The archived initial log has one extra trailing blank line removed for diff
  hygiene; every non-whitespace byte is preserved. The original remains untouched.
- [red-team-review.md](red-team-review.md) is the original independent functional
  PASS at public source, with the required module-docstring correction. The
  correction is included; that review does not certify this later packet commit.
- [docstring-parity.txt](docstring-parity.txt) records original correction parity.
  [closeout-checks.txt](closeout-checks.txt) records fresh cheap checks and source
  identity. Final candidate hash and independent disposition belong in external
  handoff/PR metadata to avoid a self-referential commit hash.

Proof source SHA256 (also printed in the actual final log):

| Source at ce69e9e5 | SHA256 |
| --- | --- |
| scripts/ci_layer.py | `2afa010d43d797ff20ed3578c2f28b710aa61f1977d0e1f641413629a2569c2f` |
| scripts/ci_shard.py | `5d76c15cb290be01bfdcc24903f77a8eda7bf89a9d6025d52703a4043b86c122` |

The correction changes only the opening docstring. Fresh AST equality excluding
that docstring and byte-identical source from `from __future__` establish inherited
executable proof applicability. No real Cargo fixture rerun in this closeout.
Gate 1 N/A: no LLM-facing semantics. Full workspace/all-platform suite NOT RUN;
no full-suite return code exists. No mutation run is claimed.

## Base audit and remote evidence limits

Audited local main `b457a94d`: since original base, the only change in the three
P0 production paths is the `ci_gdparse.py --self-test` fast-layer entry. It is
nonoverlapping. Cheap merge-tree inspection confirms the merge is conflict-free
and retains that entry; no gratuitous main integration is included here.

The prior read-only audit captured [run 40586369072](https://github.com/yuema137/MineWorld/actions/runs/40586369072)
as successful at exact old public head ce69e9e5. Fast ran; full test matrices were
SKIPPED. Deferred aggregate gates are not suite passes. This is old-head evidence,
not CI for the prepared closeout. A fresh GitHub read in this closeout could not
connect to api.github.com; current remote state was not reverified. No final-head
remote CI exists for this unpublished candidate. Required remote checks remain
coordinator-owned after authorized publication.

## Maintainability and review disposition

Original independent review evaluated debt, boundaries, readability, robustness,
extensibility, size, inheritance and coupling. Existing centralized command table
is size debt (~729 lines); runner ~50 lines and helpers retain conceptual locality.
No new runtime coupling, inheritance or speculative abstraction. Newly introduced
whole-log retention uses memory proportional to output; disclosed nonblocking P0
debt, with incremental/JUnit collection assigned to P1. Broad membership-based
command classification covers the audited closed forms and repeat wrapper.
Native Markdown publication is absent; native failures remain visible/nonzero.

Closeout adds only documentation/evidence; no new executable maintainability debt.
The first local-link check found that this older PR base predates the design file;
the packet now links to the public design PR instead. No production change resulted.
Independent final-candidate review remains PENDING and must not be self-approved.

## Closeout checklist

- [x] Implementation: reuse existing docstring correction and preserve actual proof.
- [x] Deterministic validation: AST/suffix parity, shard self-test, diff/doc/link
  checks, source hashes and merge-tree gdparse preservation; see closeout-checks.txt.
- [ ] Independent final-candidate logic review: root to route to review_sol.
- [ ] Publication and final-head remote checks: not authorized in this lane.
- [ ] Merge: explicit operator approval only; not performed.
