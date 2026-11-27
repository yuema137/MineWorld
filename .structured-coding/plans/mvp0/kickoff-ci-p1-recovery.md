# Fresh-session kickoff — CI economy P1 recovery

Prepared 2026-11-22 by the P0 recovery owner. This is a continuation aid under the existing frozen design, not a new design authority or a claim of implementation/merge completion. Start a fresh execution session; do not implement P1 in the P0 recovery session.

## Required reading and authority

Read AGENTS.md and CLAUDE.md in full from the coordinator checkout, including current §2.1/§3.4 standing rulings. Read `.claude/skills/structured-coding/SKILL.md`, its complete execute/resume row resources (agent-workflow, adaptation, implementation-working-rules, test-ci-gate-rules, standards contract), `.structured-coding/standards.md`, and applicable engineering/CI policies before editing.

Primary design: `.structured-coding/plans/mvp0/pr-ci-test-economy.md`, approved 2026-10-27. Prior frozen design is preserved in `docs/ci-test-economy` and current main. Filled execution contract: `git show da9a5d2835d5743978c1a19b7326d62468a69172:.structured-coding/plans/mvp0/pr-ci-test-economy-contract.md` (PR #526, not on current main). Read it in full and specialize to P1 only; do not inherit its old multi-PR session scope. Design governs where ledger wording is narrower. Binding documents: pr-ci-milestone-runs, step-14-ci, pr-test-hygiene I-TH-1, ARC-110, engineering standards §§15–18/22, reuse policy, DEP-163 on the preserved P1 branch.

Actual operator approval: primary JSONL `/Users/yuema137/.claude/projects/-Users-yuema137-MineWorld/64c4f5c0-38d3-474d-bf6e-27af3609b29c.jsonl` lines 26848 and 26860, copied in `ci-p0-recovery-evidence/approval.txt`. The CI-specific questions immediately precede `全按推荐` at 2026-10-27T21:23:54.135Z. Q-3 approves nextest; Q-4 approves P0. The stale “Not frozen” text does not revoke this ruling. Current dispatch explicitly authorizes recovery and local-first review, forbids public writes/merge/dispatch/paid calls/subagents, and leaves independent review to primary's red_team. A newer explicit coordinator/operator dispatch controls endpoints; do not infer publication permission from old defaults.

## Exact source inventory and base

- Preserved P1 checkout: `/Users/yuema137/mineworld-worktrees/ci-econ-nextest`, branch `ci/econ-nextest`, HEAD `631045e4161d59de89a5958cb751973ff427d8b2`.
- Audited clean: no staged, tracked dirty, or untracked files. Inventory: `ci-p0-recovery-evidence/p1-inventory.json`.
- One P1 commit on P0: `631045e4` introduces nextest execution/JUnit/weights/dependency record. Parent P0 is `ce69e9e5210c1ca11ed8637b943cc72aea38d5a0`; original common base `315a50a9cdcc7d9a0472a647844eea342faa2a0f`.
- Current origin/main was read live at `2b4d36fac345b9325a91fd4313d40e3633e728b0`. P0 #523 remains OPEN at ce69e9e5; do not state merged or drop its prerequisite.
- Start fresh owned worktree `/tmp/mineworld-recover-ci-p1`, new branch `recover/ci-p1-nextest`, from exact `631045e4161d59de89a5958cb751973ff427d8b2` after confirming no owner/path collision. This preserves the reviewed P0 stack and avoids rewriting old worktrees. Re-audit origin refs and P0 review status first; integrate current main only deliberately, preserving its ci_gdparse self-test addition. Local stacked work may proceed without claiming P0 merged. If P0 changes in review, reconcile that prerequisite before final P1 validation.
- Snapshot old committed/dirty/untracked source again before recovery; never reset or overwrite the old source checkout. Do not copy scratch/no-fail-fast-proof (`b05305bb`), whose intentional failures must never ship.

## P1 scope, gaps, and exclusions

Own `.config/nextest.toml`, `.github/actions/layer/action.yml`, `.github/actions/native/action.yml`, Dockerfile, scripts/ci_layer.py, scripts/ci_shard.py, DEP-163 and decision index. The approved archive-per-OS requirement may also require narrow changes to the owning CI workflows; audit those actual consumers before design amendment. Coordinator retains root shared docs/dashboard and integration ownership. Keep one CI writer.

Preserved implementation includes nextest profile, no-fail-fast, timing group, JUnit upload, Cargo fallback for doctests and custom harnesses, and JUnit-derived weight generation. It is NOT code-complete for the approved P1: DEP-163 explicitly says archive-per-OS build-once/shards-from-archive is unfinished. Do not accept that deferral as permission to silently remove an approved requirement. Complete within the frozen P1 design or surface a concrete material departure to the operator.

Audit these risks in the fresh session before implementation:

1. Preserve P0 layer continuation, failure exit status and summaries across nextest, custom harnesses, doctests and core-repeat. Existing P1 is_test_run recognizes only direct cargo prefixes, whereas repeat_commands wraps cargo in a Python command. Verify wrapped-repeat failure behavior; do not assume P0 semantics survived.
2. Archive production/consumption, OS matching, executable/runtime paths and auxiliary binaries, required environment and source remapping. Preserve all discovered tests and ensure build failure never becomes an empty green run.
3. Coverage parity across regular tests, doctests, three recorded custom harness targets, and newly discovered targets; validate the actual manifests rather than retaining a stale list.
4. Pinned nextest version/toolchain availability and pin consistency. Consult installed upstream help/source or official docs when relying on external command semantics; do not guess archive flags.
5. JUnit identities, missing/malformed results, failure/abort/timeout reporting, artifact uniqueness and weight aggregation. Failure summaries are diagnostic; they must not define pass/fail.
6. Timing-sensitive test isolation versus available threads and subprocess concurrency, scratch cleanup, and immutable per-OS artifact sharing. Avoid speculative abstractions and whole-log memory growth when structured results suffice.

I-TH-1: no assertion, expected value, seed or world changes. Never weaken required tests or manufacture passing CI. P2 long-tier moves, P3 shared runs, P4 impact selection, and P5 rerun-of are separate future PRs. Do not implement any of them in this session. Preserve existing guide work #525; Python splitting is separate. P3/P4/P5 depend on P1; P2 shares CI ownership and must be sequenced.

## Local validation and evidence budget

Read current shared-slot state before heavy work. Use `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`; at most two heavy jobs for this lane, preferably one during recovery. Respect PAUSE/PRIORITY_ONLY. No unbounded full-workspace tests or duplicate jobs. No API spending. Current dispatch prohibits workflow dispatch/public writes. Ordinary bounded local static/script checks are authorized; meaningful real fixture runs target approximately ten minutes each and stay within the one-hour outer envelope unless the actual contract narrows it.

Start with existing script self-tests, pin checks, config parsing and exact command plans. Then use a tiny real Cargo/nextest fixture to validate archive build once, multiple shard consumption, two intentional failures, doctest/custom-harness fallbacks, preserved failed exit, required-binary reachability, and JUnit records. Compare selected test identities against the pre-change exact baseline. Use a separate dependency-free fixture for counterfactuals, never intentional failures in the shipping branch. Follow with the smallest real MineWorld targets that exercise paths/cwd/env/scratch assumptions. Full milestone verification remains a separately recorded remote obligation under the frozen design; local readiness is not a full-suite PASS. Do not wait for GitHub to start independent review.

Keep implementation, deterministic validation and logic review separately checked in the canonical ledger. Record exact source SHA plus dirty fingerprint, commands, counts, actual outcomes, duration, artifact paths and limitations. Independent red_team reviews contract fidelity and maintainability; reserve its slot and send a concrete exact-head packet promptly.

## Endpoints and closeout

Implementation/local checks and bounded recovery commits are authorized by recovery dispatch and CLAUDE.md §3.1; do not publish, merge, dispatch workflows, spawn agents or spend on paid APIs under this kickoff. Stop at local code/evidence readiness and send the primary exact SHA, base, diff, approval source, evidence and debt findings for independent review. Release all owned heavy slots/processes. Any production change after validation needs affected checks again. One PR per fresh session; next PR kickoff is a handoff only.
