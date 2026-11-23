# PR525 — bounded guide-output correction (2026-11-23)

## Authorized correction contract

DESIGN FROZEN for the bounded correction only. Authorization: coordinator delegation
`2b75d9de-11b0-431f-b3f1-b5ff04d78413`, conveying the operator-approved old-PR cleanup
and existing purpose (re-record guide output). This records current correction authority;
it does not claim a historical freeze for the original implementation.

Base/public head: `be888b57b41f97f119390f0fa519b5bc4be0bf8b`.
Branch: `fix/pr525-output-boundaries`; sole owned clone:
`/tmp/mineworld-pr525-closeout-source`. Primary ledger and continuation state: this file.
Binding contracts: root CLAUDE.md and Structured Coding execution/test rules;
`docs/ENGINEERING_STANDARDS.md`; `docs/GUIDE_FORMAT.md` §§2, 4.1, 7, 11.2.
Parent planning/status synchronization belongs to coordinator; no parent edits here.

Scope: reproduce and repair R525-1 before publication. Existing output matching,
exit/equals failures, command sequence, directives, prose, other blocks and mirrors'
pre-rewrite equality rule remain intact. No features, grammar extensions, dependencies,
simulation/world changes, assertion weakening or unrelated cleanup.

Endpoint authority (same delegation): implementation, local checks and one semantic commit
authorized; public push/comment/PR mutation/merge forbidden. Deliver commit, exact evidence
and a body file under `/tmp/mineworld-pr525-closeout` for root/review_sol independent review.
Budget: <=10 minutes/check, <=45 minutes initial execution; no paid/model/GPU runs.
Rust compilation uses one root-coordinated slot only; never a mutable shared target.
Root instruction `08a8a6d5-73aa-432d-85ab-8f8220513e14`: wait for release, then use
an owned COW target; no competing standalone Rust compile during profiling.
Stop/escalate if a frozen invariant or actual scope must change; otherwise proceed.

## Source audit and design, before implementation

`page_run::console` calls `rewrite::recorded` after exact exit and output checks.
`recorded` checks snapbox matching, but `$ ` output becomes a new command on next
parse/run, and a closing fence can escape into prose. `run_guide::write_back` invokes
`rewrite::apply` before either file write, providing the rejection boundary.
`fences` already uses the same pulldown-cmark parser as `page::parse`; no new parser needed.

Choose rejection over escaping: the existing grammar defines every `$ ` prefix as a
command and has no output-escape syntax. In `apply`, compare original and replacement
command lines exactly. Reparse the proposed page and compare every fence's info/content
with the intended replacement or original. Validate the proposed mirror as well (its
fence delimiter may differ despite identical content), all before returning to disk writes.
Reject with named `unrepresentable recorded output` and original block line. Preserve
all slices outside console content exactly; do not widen wildcards.

Sequence: add minimal regression fixture; observe old writer accepting corrupting output
and actual parser interpreting changed syntax; implement rejection; run bounded regressions
and runner fixtures; inspect diff and maintainability; record evidence and commit.

Test ownership: deterministic Rust writer/parser regressions own grammar rejection and
unchanged English/mirror files; existing runner fixtures own normal check/write/check,
exit/equals and mirror behavior. Static format/diff/docs checks own formatting. Real LLM,
GPU/training, world lifecycle and full suite NOT REQUIRED for this runner-only correction.
Python duplication of the Rust parser is not authoritative and will not replace its evidence.

- [x] Implementation: reject changed command identities and reparsed fence corruption before writes.
- [x] Validation: old-head reproduction, new regressions, runner fixtures, format/diff/docs checks.
- [x] Logic review: implementing agent inspected exact diff, boundaries, debt/readability/robustness (below).
- [ ] Independent final implementation review: coordinator/review_sol; design review already PASS.

## Live state

Current checkpoint: source audited; design recorded before code. Public head unchanged;
working tree contains only this ledger. No background job. Next: minimal reproduction
fixture, then await coordinated Rust slot. No compile or test pass claimed yet.

Fixture preparation: two real `validate` fixtures use the embedded minimal-world templates,
with a YAML-quoted multiline name. The next-parse diagnostic on the old writer records
actual console bodies after unsafe writing. No compile launched pending coordinator slot.
Specification §11.2 rule 4 now states the rejection boundary before implementation.

Owned APFS COW target prepared at `/tmp/mineworld-pr525-closeout-target` from the
root target (read-only copy, no shared target mutations). Copy completed; no Rust process
launched. Waiting for root slot release. Source audit confirms World Pack names are Strings;
CLI validate prints the name directly, so quoted multiline fixture output follows a real path.

Independent design review: review_sol PASS reported by coordinator in message
`c0f3aef3-4f3f-46b6-9bc5-8574f33e2569`; artifact
`/tmp/mineworld-pr523-pr526-closeout-independent-review.md`. Required mirror-delimiter
case added: English four backticks, mirror three, equal original info/content. Root accepted
owned COW target; explicit compile release remains pending after 521/522.

Heavy-slot release: 521522 owner message `4fb8ad1c-a914-4c37-b00a-f6cd13b5a9a8`
explicitly released all heavy processes; root notification sent before starting. Old writer
reproduction launched using `python3 /tmp/mineworld-pr525-closeout/run_check.py before
cargo test --offline --locked -p mineworld-cli --test guide write_mode_rejects -- --nocapture`.
600s process-group timeout, two build jobs, owned COW target. Executable source remains public
base; added regression patch SHA256 `83e8e4c0981ecdbfc6969e39085b20e0cd3c9a9da8e44a6c92bd0f8740a4ae03`.

Old-head reproduction: expected FAIL, observed 0 passed/3 failed, rc101, 120.877s
(including build; fixture execution 0.92s). `before.log` shows actual next parse:
`$ mineworld --version` became a second command; triple-fence output truncated the
console to the original command plus `before`; four-backtick English retained the output
but the three-backtick mirror gained a sixth block. Old writer reported REWROTE and changed
files before guards complained. This is runtime reproduction, not only source inference.
No timeout. Proceeding with the pre-recorded minimal rejection design.

First corrected runner execution PASS: 44 passed, 0 failed, real-guide test excluded;
2.503s total, 1.08s fixture execution, rc0 (`after.log/json`). All three refusal fixtures
assert named error, no rewritten summary entries, exact unchanged English and mirror bytes.
Static fmt/diff/scratch checks PASS. Added a positive boundary regression: triple backticks
remain representable inside four-backtick English/mirror fences, with exact next-parse content.
Removed only the temporary old-writer parse diagnostic (preserved in before.log and fixture patch).
Final targeted rerun will cover the additional positive case. No production/world source changes.

## Final local evidence and review (2026-11-23)

All commands run in the sole owned clone on macOS with rustc 1.97.1,
`CARGO_TARGET_DIR=/tmp/mineworld-pr525-closeout-target`, `CARGO_BUILD_JOBS=2`.

| Check | Result and decisive evidence |
| --- | --- |
| Old writer, `cargo test --offline --locked -p mineworld-cli --test guide write_mode_rejects -- --nocapture` | Expected FAIL: 3/3 regression cases fail; actual parse corruption above. 120.877s, rc101. |
| Final `cargo test --offline --locked -p mineworld-cli --test guide -- --skip the_guide_in_this_repository_runs_green` | PASS: 45 passed, 0 failed, 1 filtered, 2.832s total/1.08s fixtures, rc0. Includes real CLI output, all rejection cases, normal check/write/check, mirror mismatch, exit and equals failures. |
| `cargo clippy --offline --locked -p mineworld-cli --test guide -- -D warnings` | PASS: 21.625s, rc0. |
| `cargo fmt --all --check`; `git diff --check` | PASS, rc0. |
| `python3 scripts/check_doc_headings.py` | PASS: 255 numbered sections/337 documents, none duplicated. |
| `python3 scripts/check_scratch.py scan` | PASS: 384 test sources; only two established exemptions. |
| `python3 scripts/check_scratch.py left --target-dir /tmp/mineworld-pr525-closeout-target` | FAIL as a global clean-scratch claim: 31 historical directories inherited by the COW target plus one unrelated global temporary directory. Direct root/owned `target/tmp` name comparison gives 31/31 identical, zero new names; no guide scratch remains. Existing files preserved, no cleanup beyond owned test guards. |
| Real guide/full suite/all-platform/remote candidate CI | NOT RUN; intentionally outside bounded runner-fixture authorization. No full-suite return code or passing remote claim. |
| LLM/GPU/training/world lifecycle Gates | N/A: deterministic guide writer/parser boundary only. |

Final tested tracked diff SHA256: `21d5c25c78ec745dbf8081efaccef32c8847afe3dbbdaaedf1fdf57eb9ee6828`
against public base. Subsequent changes only add this ledger/evidence. Exact source hashes:

- `tools/cli/tests/guide.rs`: `db65127783bea85982b7231c4d1f562be546a0d71ce993920df95859110e93fb`

- `tools/cli/tests/guide/rewrite.rs`: `b273f5056e372bb0d85060395c6aa3108808281c066b7b7d7713b3da147086d2`

- `docs/GUIDE_FORMAT.md`: `b9cf77038b9707c20366e833dfeaf00c93742ee08eda2f1165e15fe46c428812`

Counterfactual proof is the actual old-head failing reproduction, not a synthetic mutation.
Complete logs, metadata, preserved reproduction fixture patch and body draft live under
`/tmp/mineworld-pr525-closeout/`; this tracked ledger preserves commands, decisive results,
source identities and limitations independently of temporary logs.

Maintainability review: only guide-test infrastructure changes. `validated_replacement`
(about 49 lines) owns one pre-write representability check and reuses the existing parser,
content renderer and source-slice replacement; no dependency, escaping grammar or abstraction
layer. `rewrite.rs` remains about 350 lines. `guide.rs` was already >1000 lines; it gains
117 lines of related fixtures. The ~70-line fixture helper is cohesive setup/execution/no-write
verification with embedded complete templates; splitting it into trivial helpers would scatter
this boundary test. Existing alignment complexity and sequential file writes remain unchanged.
No ownership/layer leakage, deep inheritance, speculative extension or new coupling. The
original command strings/prose/other block source slices are preserved by the unchanged
replacement writer, and all fences are reparsed against intended content. Both candidate
files validate before caller I/O; an I/O failure after the first actual write can still leave a
partial update (existing limitation, distinct from syntax rejection). Full original
PR525 robustness outside R525-1 is not re-scoped here.

Heavy slot released to root after clippy wrapper exited; no background job remains. Local
implementation and validation complete; final semantic commit and independent exact-head
review are next. Public source remains `be888b57`; no push/comment/merge performed.
