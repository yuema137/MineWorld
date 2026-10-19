# PR design — CI: `fast` only per PR and per merge; the full matrix at milestones

**DESIGN FROZEN — operator ruling 2026-10-18 (CI policy).** The rulings were all made in the primary session on 2026-10-18 and relayed by the coordinator.

1. "为了节省时间，我们尽量不要跑全量 ci，而是只跑必要的。只有到了一个新验收标准完成，我们再跑 ci"
2. "ci 资源有限，我们不应该给每个 pr 都跑，这样会造成大量重复。我们应该攒几个比较相关的，然后一起跑"
3. `removability` becomes a required status check on `main`. The primary session changes the protection rule after this PR merges.
4. **The latest ruling, which governs this design:** speed first. There are no per-PR test runs. The full, acceptance and integration tests run only at milestones, and their failures are fixed then, in one batch. This supersedes the targeted-run and merge-train drafts made for rulings 1–2.

Scope, invariants and acceptance are frozen by these rulings. The details are the implementer's.

- **Decision:** [`docs/DECISIONS.md`](../../../docs/DECISIONS.md) ARC-154.
- **Branch:** `ci/targeted-pr-runs`, from `origin/main` at `b85f2804`.
- **Worktree:** `/Users/yuema137/mineworld-worktrees/ci-targeted`.
- **PR:** #268.
- **Merge:** under the new rule, it merges once `fast` is green, by the coordinator.

## 1. Scope

| Mode | Trigger | Runs |
| --- | --- | --- |
| `docs` | documentation only (ARC-48 13x) | the `docs` layer (unchanged) |
| `deferred` | every other PR, a push to `main`, a dispatch without `milestone` | `fast` only. On a PR, the gates pass with "deferred to milestone (operator policy 2026-10-18)" |
| `full` | a dispatch with `milestone: true`, a PR labelled `milestone`, a scratch push | everything on every OS. A milestone dispatch or a `-scenario` scratch push also runs AC-8 |

**Non-goals:**
- the nightly (unchanged);
- branch protection;
- every layer's commands;
- the ARC-110 partition.

## 2. Invariants

- **I-1.** On every non-draft PR and every dispatch, these four always report: `fast`, `test`, `test-macos`, and `removability` (required after merge). `test-windows` reports the same way.
  - When `changes` failed, a gate is red.
  - When the run is deferred, the gate passes and writes its note into the job summary.
- **I-2.** On a push to `main`, the gates do not run. `ci_nightly.py` `main_push_run` would otherwise read a deferred gate as a passing test sample.
- **I-3.** A change set that cannot be computed is code, so `fast` runs on it, not `docs`.
- **I-4.** A `full` run is every layer, whatever the diff.

## 3. Source audit (2026-10-18)

- **`ci.yml`, before this PR:** every code PR and every push to `main` ran the full matrix. Gates were skipped on a docs-only change. The AC-8 legs ran on main pushes and on dispatches.
- **Branch protection:** requires `fast`, `test` and `test-macos` (app 15368), with `strict: false`.
- **`scripts/ci_nightly.py`:**
  - `main_push_run` and `sample_of_job` count a `success` job of main's push run as a passing sample.
  - A missing job counts as no sample.
  - Hence I-2.

## 4. Acceptance

- **A-1.** `ci_changes.py --self-test` covers every mode trigger:
  - a code PR;
  - a docs PR;
  - a `milestone` label;
  - a docs PR labelled `milestone`;
  - a push to main;
  - a plain dispatch;
  - a milestone dispatch;
  - `milestone` outside a dispatch;
  - a scratch push;
  - an unknown change set.
- **A-2.** This PR's own run is `deferred`:
  - `fast` passes;
  - `test`, `test-macos`, `test-windows` and `removability` pass with the note;
  - no shard runs.

## 5. Commit plan

### C1 `ci: targeted PR runs, merge trains, full matrix on main (ARC-154)`, superseded

C1 and C2 were the first draft, made for rulings 1–3 and pushed to #268. C3 replaces them.

### C2 `docs: ARC-154, ENGINEERING_STANDARDS §§15–16, and the CI policy PR design`, superseded

Replaced by C3.

### C3 `ci: fast only per PR and per merge; the full matrix at milestones (ARC-154)`

- [x] **Implementation**
  - `ci_changes.py`: `mode_of`, `note_of`, `--labels`, `--milestone`, and the outputs `mode` and `note`.
  - `ci.yml`:
    - the `milestone` dispatch input, plus `labeled` and `unlabeled`;
    - shards and layers run on `full` only;
    - the four gates always report on PRs and dispatches;
    - `removability` becomes one gate job;
    - the AC-8 legs run on a milestone dispatch or a `-scenario` scratch push;
    - the header comment.
  - Reverted the draft's `ci_shard.py`, `ci_layer.py` and layer-action changes, and removed `scripts/train.py`.
  - ARC-154 rewritten; ENGINEERING_STANDARDS §§15–16 amended.
- [x] **Deterministic validation**
  - `ci_changes.py --self-test`: 40 cases pass.
  - The workflow parses as YAML (17 jobs).
  - `check_doc_headings` and `check_decision_ids` pass.
- [x] **LLM logic review:**
  - gates on a PR when `changes` failed: red;
  - push to main: gates skipped (I-2);
  - matrices are static, never empty;
  - `inputs.milestone` is false outside a dispatch;
  - a docs PR labelled `milestone` stays `docs`;
  - the AC-8 `ac8` still uses `always()`.
- [ ] **Evidence:** A-2 on the PR head (§6).

## 6. Evidence

- **Merge-train rehearsal (D-2 draft), 2026-10-19.** Train #271 (`train/probe`) merged #269 and #270 into the throwaway base `probe/train-base` with a merge commit. GitHub marked both components `MERGED`. All four branches are deleted.
- **A-2:** recorded on the PR.

## 7. Deviations

- **D-1, coordinator option (c):** `test-macos` is required, so the gates always report rather than skip.
- **D-2:** targeted runs and merge trains were drafted, then superseded by ruling 4. The record is in ARC-154 item 6.
- **D-3:** `removability` always reports on PRs (ruling 3).
