# PR design — CI: targeted PR runs, merge trains, full matrix on main

**DESIGN FROZEN — operator ruling 2026-10-18 (CI policy).** Three rulings, all given in the primary session
on 2026-10-18 and relayed by the coordinator:

1. "为了节省时间，我们尽量不要跑全量 ci，而是只跑必要的。只有到了一个新验收标准完成，我们再跑 ci"
2. "ci 资源有限，我们不应该给每个 pr 都跑，这样会造成大量重复。我们应该攒几个比较相关的，然后一起跑"
3. `removability` becomes a required status check on `main`. The operator changes the protection rule
   after this PR merges. This PR does not change it.

The scope, invariants and acceptance below are frozen by those rulings. The details are the implementer's.
The decision record is [`docs/DECISIONS.md`](../../../docs/DECISIONS.md) ARC-154.

- **Branch:** `ci/targeted-pr-runs`, from `origin/main` at `b85f2804`.
- **Worktree:** `/Users/yuema137/mineworld-worktrees/ci-targeted`.
- **Merge:** the operator's. This PR is never merged by its implementer.

## 1. Goal and scope

Scale CI to the change, and group related PRs into merge trains.

| Mode | Trigger | Runs |
| --- | --- | --- |
| `docs` | documentation only (ARC-48 13x) | `docs` layer, unchanged |
| `component` | an ordinary PR | `fast`; the gates pass with a deferral note; `merge-gate` is red |
| `targeted` | a `train/*` head branch, or the label `ci` | `fast`, the affected Linux test layers, and `removability` |
| `full` | a push to `main`, a dispatch, a scratch push, or the label `full-ci` | everything, on every OS |

**Non-goals:**
- The nightly (ARC-83).
- Branch protection.
- The AC-8 legs.
- Any layer's commands.
- The stale `ci/windows-test-split` branch.

## 2. Invariants

- **I-1:** The required checks (`fast`, `test`, `test-macos`) and the newly required `removability` always
  report, on every non-draft PR, push to `main`, scratch push and dispatch. A gate is red when `changes`
  failed. When the mode deferred its work, the gate passes and writes why into its job summary.
- **I-2:** Fail closed. A path in no class, a `cargo metadata` failure, or an uncomputable change set makes
  the run `full`.
- **I-3:** A targeted Rust set is closed over reverse dependencies, including dev and build dependencies.
- **I-4:** A push to `main` always runs the full matrix.
- **I-5:** A layer's commands stay in `scripts/ci_layer.py`. The workflow names layers, and a package scope
  is a parameter of a shard.
- **I-6:** `scripts/train.py` never touches the caller's working tree.

## 3. Source audit (2026-10-18)

- **`.github/workflows/ci.yml`:**
  - `changes` produced `code` and `clients`.
  - The gates `test`, `test-macos` and `test-windows` were skipped together with their shards on a
    docs-only change.
  - Every code PR ran 3 Linux shards, 4 Windows shards, 2 macOS jobs, `platforms`, `python` on Linux and
    Windows, `removability`, and `farfield` by path.
- **`scripts/ci_changes.py`:** `decide` makes the docs/code decision and fails closed; `touches_clients`
  decides the far field.
- **`scripts/ci_shard.py`:** the ARC-110 partition of the units of `cargo test --workspace`.
- **`scripts/ci_layer.py`:**
  - `core --shard K/N`;
  - the `python` layer builds `mineworld-cli` and drives it;
  - `removability` runs `ci_removability.py`.
- **`.github/actions/layer`:** runs `ci_layer.py` in the toolchain container.
- **Branch protection on 2026-10-18:** `required_status_checks.checks` holds `fast`, `test` and
  `test-macos` (app 15368), with `strict: false`. `test-windows` is not required.
- **Rust tests that read non-crate trees** (`git grep` over `*.rs`):
  - `worlds/` is read by 10 crates; `spike/` by 11; `entities/` by `tools/cli`.
  - `clients/` and `presentation/` are read by `tools/cli`, `tests/acceptance`, `packages` and
    `worldpack`.
  - `cognition/lm-controller/` is read by `tools/cli`.
  - `docs/reference/` is read by `tools/cli` (ARC-90).
  - No Rust test reads `sdk/python/`, `clients/agent/`, `tools/character_pipeline/` or
    `tools/asset_generation/`.
- **cargo target filters** (probed in a toy workspace with cargo 1.97.1):
  - `-p bin_only --lib` and `-p bin_only --lib --bins` fail ("no library targets").
  - `-p a -p b --test X` succeeds when only `a` has `X`.
  - `--doc` fails when no selected package has a library.
  - So a scoped shard passes only the filters that some selected package can satisfy.

## 4. Acceptance

- **A-1:** `ci_changes.py --self-test` covers each of these with its stated plan:
  - kernel (it and every package above it);
  - a leaf crate (itself, one shard);
  - Python only (no Rust; the `test` gate notes "nothing affected");
  - a client only (far field and the client readers);
  - documentation only;
  - the undecidable cases (a path in no class, a `cargo metadata` failure, an unknown change set);
  - each mode's trigger.
- **A-2:** `ci_shard.py --self-test` covers the package scopes: bin-only, lib and proc-macro packages,
  multi-package commands, and an unknown package.
- **A-3:** `train.py --self-test` covers:
  - the body round-trip;
  - name validation;
  - a real throwaway repository: merges in order with one merge commit per component, a conflict that
    names the PR, and a clean tree after the abort.
- **A-4:** This PR's own CI is a `component` run: `fast` runs, the gates pass with their notes, and the test
  shards do not run.
- **A-5:** Labelled `ci`, this PR runs `targeted`. It touches the CI machinery, so that means every Linux
  layer, with no macOS or Windows.
- **A-6:** Labelled `full-ci`, this PR runs the full matrix.
- **A-7:** A real two-PR train of trivial docs PRs, built by `train.py build` against a throwaway base
  branch, shows two things: the train PR body, and `status`. Merging the train into that throwaway base
  shows whether GitHub marks both components merged. Nothing lands on `main`.

## 5. Commit plan

### C1 — `ci: targeted PR runs and merge trains (ARC-154)`

Code and workflow, in one commit, because the workflow reads the scripts' new outputs.

- [x] **Implementation:**
  - `scripts/ci_shard.py`: `Scope`, `scope_of`, `units_of(packages)`, scoped `commands`, `parse_packages`,
    `weight_of`, `--plan N --packages`.
  - `scripts/ci_layer.py`: `--packages`, passed through to the shard.
  - `scripts/ci_changes.py`: `Workspace`, `affected`, `Plan`, `plan_of`, `--head-ref` and `--labels`, and
    every output.
  - `scripts/train.py`: `build`, `status`, `--self-test`, wired into `fast`.
  - `.github/actions/layer`: the `packages` input.
  - `.github/workflows/ci.yml`: the modes, the always-reporting gates, `merge-gate`, the required
    `removability`, `labeled` and `unlabeled`, and the header comment.
- [x] **Deterministic validation:**
  - The three self-tests pass locally: `ci_changes` 54 cases, `ci_shard` passed, `train` passed.
  - `check_ci_pins`, `check_doc_headings` and `check_decision_ids` pass.
  - The workflow parses as YAML (19 jobs).
  - The classifier, run on main's last 8 merges as trains, gave Python-only for #260 and docs for 5. The 2
    remaining merges hit `docs/reference/` and were then mapped to `tools/cli`.
- [x] **LLM logic review:** the review checked each of these.
  - Gates stay red when `changes` failed.
  - No matrix is ever empty: `["none"]` is a placeholder in a skipped job.
  - A reader package does not count as a changed binary.
  - Scratch pushes keep the far field by path.
  - Draft PRs still skip the gates, as before.

### C2 — `docs: ARC-154, ENGINEERING_STANDARDS §§15–16, this design`

- [x] **Implementation:** ARC-154 in the single `docs/DECISIONS.md` (#258 has not merged); §15's rules and
  §16's schedule table.
- [x] **Deterministic validation:** `check_decision_ids` (169 distinct ids) and `check_doc_headings` pass.
- [x] **LLM logic review:** terminology is checked against ARC-48, ARC-110 and ARC-83. The trade-off and the
  `full-ci` rule are stated in both documents.

### C3 — evidence (this ledger)

Validation. Its items are completed on the PR.

- [ ] A-4: the component run on the PR head.
- [ ] A-5: the `ci`-labelled run.
- [ ] A-6: the `full-ci` run.
- [ ] A-7: the docs train.

## 6. Evidence

Recorded on the PR as the runs complete. See §7.

## 7. Progress and deviations

- **D-1 (coordinator, option (c)):** `test-macos` is required, so it and `test-windows` always run as short
  gates and pass with a note, instead of being skipped. No protection change is needed.
- **D-2 (ruling 2):** merge trains and the `component` mode are added to the same PR.
- **D-3 (ruling 3):** `removability` always reports. It runs for real on targeted and full runs.
- **D-4:** `merge-gate` is the protection against merging a component PR alone. It is red on components
  and not required. Enforcing it is a protection change, which is the operator's.
