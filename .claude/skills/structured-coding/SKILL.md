---
name: structured-coding
description: "Plan and execute substantial coding work through overall, step, and PR design documents, frozen implementation contracts, autonomous execution, and post-merge plan updates. Use for the Structured Coding workflow, detailed PR commit plans, approved PR execution or recovery, and backward planning updates; do not impose the full workflow on unrelated small edits or reviews."
---

# Structured Coding

Use progressively detailed planning, a live PR design ledger, and autonomous implementation within an approved contract. Keep human decisions at requirements, design freeze, material deviations, and merge review.

## Start with the current phase

Identify the requested phase from the user, existing documents, and repository state. Continue an existing workflow at its current checkpoint. A planning request does not authorize implementation; loading this skill does not authorize a merge.

Read [agent-workflow.md](references/agent-workflow.md) and [adaptation.md](references/adaptation.md). Then load the complete resources for the current phase:

| Phase | Additional reading | Deliverable / boundary |
| --- | --- | --- |
| Overall planning | None | Agreed requirements, module-level approach, step boundaries |
| Step planning | None | PR scopes, dependencies, integration checkpoints and adversarial criteria |
| PR design / freeze | [PR requirements](prompts/pr-design-requirements.md); [test rules](prompts/test-ci-gate-rules.md); the project-specific contract in [working rules](prompts/implementation-working-rules.md) | Audited commit plan; user-approved frozen design and execution contract |
| Execute / resume | Full [working rules](prompts/implementation-working-rules.md) and [test rules](prompts/test-ci-gate-rules.md); filled contract; current PR design in full | Implement, validate, review, commit, and iterate through authorized PR/CI work |
| Operator review / merge | Working rules §§21–22; current PR design and exact-head evidence | Review-ready handoff; merge only with explicit operator authorization |
| After merge | Confirmed merge result; current PR, step, and overall documents | Update PR → step → overall; detail the immediate next PR |
| Project standards | [standards contract](references/standards.md), only when `.structured-coding/standards.md` exists in the target project | Apply the project's declared review conventions and check ownership; the helper reports, it does not enforce |
| Hook design / integration | [hook contract](references/hook-contract.md); [platform notes](references/platforms.md); [continuity preset](references/continuity.md) and [checkpoints preset](references/checkpoints.md), only when explicitly enabled | Read only explicitly enabled preset interfaces; further integration only when requested |

This table is the required resource list for the phase; do not assemble one from the prompt names a request happens to mention. Before stating that work does or does not comply with this workflow, read the rule being cited, in this session: the routing rules live here and in [agent-workflow.md](references/agent-workflow.md), not in the long prompts, and recalling them from an earlier session is not evidence.

The original long prompts are deliberately preserved. Do not replace them with this entrypoint or a summary during execution. Resolve relative resource links against this skill directory, not the target repository.

## Essential operating rules

1. Audit real source, callers, tests, and relevant evidence before specifying file/function-level work. Do not invent repository structure.
2. Plan overall → step → PR. If a step needs one PR, expand the step document in place into the PR design; keep one authority for that work.
3. Every PR supplies a meaningful integration checkpoint. Each planned commit tracks implementation, deterministic validation, and LLM logic review separately with `[ ]` / `[x]` and evidence.
4. Require a user-approved `DESIGN FROZEN` header and an authorized implementation contract before starting implementation. Freeze scope, invariants, and acceptance; keep progress, evidence, and bounded discoveries live.
5. Start each new PR in a fresh implementation session. Resume after compaction as the same PR. Recover repository/process truth and re-read the current PR design and execution rules before editing. A session that is new, replacing another, or a delegated agent re-reads this entrypoint and its current phase row first; that is a session boundary, not something to repeat every turn.
6. Within the approved contract, investigate, implement, run sufficient validation, review, update the ledger, and create semantic commits autonomously. Commit inspection is a checkpoint, not a request for approval.
7. Audit uncertainty before asking. Resolve bounded details and ordinary failures autonomously. Record departures immediately. Escalate material changes or work outside existing authorization with concrete evidence and a proposed next step.
8. Respect test ownership and the approved cost envelope. Real Gates require real evidence. CI evidence must match the exact final PR head; do not repeat expensive full suites without an independent reason.
9. Continue authorized execution until the PR is ready for operator review, including CI repair when needed. Present the complete handoff and retain the explicit merge-approval boundary.
10. After confirmed merge, update progress and implications in the step and overall documents. Completing a PR does not complete its overall; record what remains. Detail one step ahead; do not begin the next PR in the old implementation context.

## Authority and packaging boundaries

The filled, approved project contract specializes these reusable rules. User instructions and existing authorization control task scope. Repository reality establishes what exists; it does not authorize changing frozen intent. A lower-level document or stale handoff cannot silently override a binding invariant.

If the target project declares standards, read [the standards contract](references/standards.md) and apply the resolved result; that file states codebase-wide rules only, and it reports rather than enforces. A project without one is unaffected.

`SIDERIUS`, `shuffle`, `file_order`, Gate labels, tool names, and scientific examples in the prompts retain their source wording. Apply them where relevant using [adaptation.md](references/adaptation.md); do not create those systems in an unrelated project.

Hooks are **optional, not registered by default**. If the operator has enabled `continuity`, read [its interface](references/continuity.md), explicitly bind the current execution session, and maintain its mechanical checkpoint after semantic handoff updates. Never enable/register hooks yourself merely because the skill was invoked. This preset supplies compact freshness checks, snapshot attempts, and recovery instructions, not freeze/merge guards or enforced semantic recovery. If the operator has enabled `checkpoints`, read [its interface](references/checkpoints.md) and use the same explicit session binding. Before selecting a semantic commit, run `checkpoints.py inspect`, inspect the actual staged diff, synchronize the design/handoff and evidence/deviations, then refresh the mechanical checkpoint. Before composing the review handoff, run `prepare-review` for its checklist; use `cancel-review` to cancel pending intent. Keep pending/inconclusive/not-run evidence explicit. These are advisory reminders, with no commit gate, test attestation, merge permission or automatic Stop continuation. Without the selected presets, follow the same checks procedurally; missing hooks do not block authorized work. Report actual enforcement status accurately.

For the human explanation and interaction examples, use [README.md](README.md). It is not required reading during routine execution.
