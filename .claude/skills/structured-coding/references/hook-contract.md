# Hook behavior contract

Status: **BEHAVIOR SPECIFICATION — optional [continuity preset](continuity.md) implements mechanical H4 checks, H5 snapshot attempts, and the H6 recovery-instruction subset.** It is not registered by default. The optional [checkpoints preset](checkpoints.md) adds H2 commit-preparation advice and an H7 review-intent notice, with zero automatic continuations. Full H1/H2/H3/H7 enforcement and H6 mutation enforcement remain unimplemented. The optional [standards preset](standards.md) is **not** an adapter for anything specified here: it reports a project's own declared checks and review conventions after a direct `git commit`, which is a different axis from the freeze, merge and continuity behaviour this contract describes, and it blocks nothing. The requirements below remain the complete target, not a claim of full adapter coverage.

This contract translates the workflow's freeze, merge, and context-continuity requirements into observable behavior. Event names below describe the workflow; actual host adapters are separate work. Supported host events and their sources are listed in [platforms.md](platforms.md).

## State and trust

Maintain one active PR record per repository/worktree/session association. It references the primary design and filled contract; it must not duplicate their semantic content as another authority.

The record needs: repository/worktree identity, PR identity, branch/base/HEAD, design and contract paths, frozen-design revision, implementation authorization reference, lifecycle, handoff path, current checkpoint/next actions, validation budget, and recovery status. Runtime state also tracks the working-tree fingerprint, relevant job IDs/artifact paths, and CI head/result.

Merge authorization must identify the operator, repository/PR, target branch, approved candidate HEAD, and supporting approval event. It must come from a trusted operator/host channel, not a boolean the implementing agent can write into Markdown. Do not infer approval from a frozen design, passing CI, or an agent-written review note. Do not ask again when matching authorization already exists.

Use a stable revision/fingerprint for frozen semantic sections. Exclude live evidence/status sections from that freeze fingerprint. A working-tree fingerprint should cover tracked modifications, staged state, and relevant untracked contents; define workflow metadata exclusions explicitly so updating the handoff does not recursively invalidate itself. A timestamp or list of filenames alone is insufficient to identify tested content.

No universal file format or host configuration is prescribed here. Select the smallest representation that supports the implemented adapter and test it against these cases.

## Lifecycle

```text
DRAFT → FROZEN → EXECUTING → READY FOR OPERATOR REVIEW → MERGED
                        ↘ MATERIAL DECISION → revised, approved FROZEN

Same PR compact/resume: EXECUTING → RECOVERY REQUIRED → EXECUTING
Review repair: READY FOR OPERATOR REVIEW → EXECUTING (same PR)
Next PR: new DRAFT and a new implementation session
```

Record `CLOSED / AWAITING OPERATOR ACTION` for the autonomous implementation context at readiness. This is distinct from merging or closing the remote PR.

## H1. Before implementation starts or mutates project behavior

Check the active PR identity, actual branch/base and prerequisites, operator-approved freeze revision, approved implementation contract, and recovery status. If they match, permit work within the approved contract. If evidence is missing or mismatched, deny the dependent implementation mutation and explain the exact missing condition and recovery action.

Continue to allow source audit and design/contract preparation before freeze. Do not block the edits required to complete a design or record a material decision. Do not let an unrestricted documentation exception authorize modification of the frozen semantic sections.

The guard must cover the actual mutation routes offered by the host: native edits, shell/script writes, and relevant external tools. A filename rule on one edit tool is not comprehensive enforcement. Reading a file can be recorded mechanically; whether the agent understood it remains an agent responsibility.

## H2. Before a semantic commit

No new human approval is required. Confirm or surface the milestone's diff/staged-file inspection, ledger synchronization, relevant evidence, and deviations. If a mechanical prerequisite is stale, return the precise repair action and let the agent repair and retry autonomously.

Do not infer test success or review completion from a commit message. Tests and review have separate evidence records. Do not turn this checkpoint into the outdated per-commit operator gate.

## H3. Before merge

Resolve the exact remote PR, base branch, and candidate HEAD. Check readiness, required CI on that candidate, applicable Gate evidence, and matching explicit operator authorization. Verify the candidate again at the merge operation, using a head-bound operation where available.

Deny a merge with missing or mismatched authorization; return the concrete candidate and missing condition. An allowed merge consumes/applies only the matching authorization. Reuse on another PR or a changed candidate must not succeed unless the operator's explicit authorization covers it.

Cover all enabled merge routes, including host tools, CLI/API calls, scheduled or auto-merge requests, and direct target-branch operations that would bypass the boundary. A regex matching only `gh pr merge` cannot claim complete coverage. For ambiguous operations, require a supported guarded route rather than guessing that no merge occurs.

## H4. Before manual compact

Compare repository/HEAD/fingerprint, primary-design checkpoint, and handoff synchronization. If current, permit compact. If stale, block the manual compact and report the exact synchronization work. After the agent updates the state, the same check should pass without asking for additional operator approval.

Do not perform an LLM summary inside the hook. The agent maintains semantic state before invoking manual compact.

## H5. Before automatic compact

Permit compact even when the semantic handoff is stale. Capture a deterministic emergency snapshot of observable state: repository/worktree/branch/HEAD, status/diff fingerprint, design/handoff identities and hashes, and known relevant job/log references. Store it separately from the semantic handoff and mark `RECOVERY REQUIRED`.

Do not invent completed work, decisions, test results, or next actions. Do not copy credentials, environment dumps, or unrelated process contents into the snapshot. If snapshot capture fails, preserve the failure warning through an available host channel and still avoid blocking unavoidable compaction. Subsequent execution must recover from actual repository state before mutation.

## H6. On compact/resume session start

Dynamically resolve the active PR from the handoff and verify it against repository/worktree identity. Never hardcode a historical PR or phase. If identity is missing, stale, closed, or inconsistent, inject the mismatch and enter recovery; do not auto-start a new PR from an old handoff.

Inject the active PR, primary design and filled contract, checkpoint, exact next actions, stop conditions, and any emergency warning. Replay the filled execution contract and both execution prompts, or require the agent to read the complete files before further implementation. If host output limits truncate the payload, report that explicitly and require full file reads; never claim a partial injection restored the whole prompt.

Require the agent to inspect git/process state, read the primary design in full, reconcile the handoff, and reopen the source seam. Preserve completed work and reuse active jobs where applicable. Clear recovery state only after reconciliation; an agent-supplied `recovered: true` alone is not evidence of mechanical identity checks.

## H7. At readiness and after merge

At readiness, verify the contract's actual terminal conditions and preserve the final executable HEAD, final PR HEAD, evidence pointers, CI state, limitations, and operator handoff. Do not allow a generic completion marker to stand in for missing checks. Record outstanding blockers honestly instead of declaring readiness.

After an observed merge, request/track PR → step → overall synchronization, including progress and implications for the next PR. Verify recorded updates; the hook must not author semantic lessons itself. Do not begin the next PR in the old implementation session.

## Outcomes and failure behavior

The conceptual outcomes are `ALLOW`, `DENY_DEPENDENT_ACTION`, `ALLOW_WITH_RECOVERY_WARNING`, and `REQUEST_STATE_RECONCILIATION`. These are contract terms, not copyable host JSON. An adapter must translate them to that event's supported return format.

For freeze/merge enforcement, unavailable or untrusted required evidence must not count as authorization. For automatic compact, favor successful compaction and mandatory recovery. Hook timeouts, malformed state, and duplicate events must have bounded handling and clear messages. Observational events cannot be used to claim an action was prevented before it happened.

Where a host/tool path cannot enforce a requirement, disclose the coverage gap and use procedural checks or an explicitly integrated boundary. Do not describe a best-effort hook as a sandbox. The current instruction-only edition does not make adapter availability a prerequisite for authorized work.

## Acceptance scenarios for future adapters

| Scenario | Expected behavior |
| --- | --- |
| Draft exists, implementation not approved | Allow audit/design edits; deny implementation mutation |
| Matching frozen design and authorized contract | Allow bounded implementation |
| Only ledger/progress changes after freeze | Preserve freeze validity |
| Frozen acceptance changes without approval | Block execution of changed semantics; require design reconciliation |
| Normal semantic commit with current evidence | Allow without a new operator question |
| Merge requested without explicit authorization | Deny; report exact PR/head and missing approval |
| Another PR's authorization or a different candidate HEAD | Deny mismatched merge |
| Matching authorization and required evidence | Allow head-bound merge; confirm actual result |
| Manual compact with stale fingerprint/handoff | Block until synchronized; retry then passes |
| Automatic compact with stale handoff | Mechanical snapshot/warning; allow compact; require recovery |
| Automatic snapshot write failure | Allow compact with warning; recover from repository truth |
| Resume while an existing Gate is still running | Recover its identity and logs; do not duplicate the run |
| Resume points to another worktree or a closed prior PR | Reconcile identity; do not execute stale next actions |
| Prompt injection exceeds output limits | Require complete on-disk prompt reads before implementation |
| Required Gate is inconclusive, process exits zero | Do not mark readiness as PASS |
| CI is green for an older HEAD | Require applicable current-head evidence |
| Agent writes its own merge-approval field | Do not accept it as operator authorization |
| Alternative tool/script performs a merge | Guard the enabled route or document the enforcement gap |
| Duplicate lifecycle event | No duplicate snapshot/job/merge or destructive state reset |
| Single document serves as step and PR | Update it once, then overall; no self-parent loop |

These scenarios are specified acceptance criteria, not a claim that runtime tests have passed.
