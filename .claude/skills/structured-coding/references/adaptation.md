# Applying the preserved prompts

[Chinese mirror](adaptation.zh-CN.md)

This file explains how to apply the prompts to different projects. It does not replace their detailed instructions.

## Resolve rules by scope and phase

- Follow the current user's instructions and existing authorization. A filled contract records that authorization; a template with example values does not create it.
- Apply the approved PR contract and binding parent invariants. A more restrictive project limit controls over a generic default. Do not silently relax a binding restriction by editing a child document.
- Use the working rules for implementation autonomy and lifecycle. Use the TEST / CI / GATE rules for validation ownership and cost discipline.
- Use the PR requirements to build an auditable design and commit plan. Commit-time checks do not require a human response; execution continues after recording them.
- If a material conflict remains after source/document audit, identify the incompatible clauses and prepare a concrete resolution before requesting a decision. Do not stop over a routine detail already bounded by approved intent.

Repository/git/process truth establishes actual state, including when a document is stale. It cannot silently redefine the intended behavior. When actual behavior contradicts an invariant, record that fact and determine whether it is an implementation defect or a design decision requiring the operator.

## Project-specific wording retained intentionally

| Source wording | Application in another repository |
| --- | --- |
| `SIDERIUS` | Historical project name in the test prompt. Bind rules to the current `PROJECT / PR`; do not rename the user's project. |
| `shuffle`, `file_order`, visited sequence, random seed, step count | Apply to ordering/default-parity changes. Otherwise mark not applicable; do not introduce these options to satisfy the template. |
| Planner exposure, production-default changes | Keep the source exclusions where relevant. An intentional change to one belongs in explicitly approved scope with the required evidence. |
| Gate 1 / Gate 2 | In this workflow, real-LLM and real-lifecycle evidence. Map to actual project commands, and record `NOT REQUIRED` where the claim does not need that layer. |
| Ruff, Pyright, Pydantic, pytest | Examples of ownership and commands. Use the repository's actual tools; the skill does not require new language/tool dependencies. |
| Scorer, metric, model, training, GPU | Apply when the task has those semantics. Non-ML projects still use deterministic and real-integration evidence where appropriate. |
| `before_end_memory.md` | The handoff filename in the source prompt. The default location is `.structured-coding/plans/<effort>/handoff.md`; reuse an established file if the project has one, and never overwrite unrelated user notes. |
| Full-suite result in final handoff | Report the canonical result if run, or `NOT RUN` with its reason and the actual selected CI evidence. Do not run a new full suite solely to populate this field. |

## Budget and approval interpretation

The working rules describe an approximately one-hour outer autonomous runtime envelope. The more specific test rules target approximately ten minutes **per Gate run** unless the frozen contract authorizes more. Record both per-run and total limits, plus relevant CPU/GPU/API/monetary constraints, in the filled contract. Count failures and retries against the total envelope.

The ten-minute default is not a requirement to fake a real lifecycle. Reduce the real workload while keeping the acceptance claim observable. If that is impossible within the authorized envelope, prepare the smallest meaningful run and its cost projection for the operator.

Real-training approval can be granted in the implementation contract before execution. It need not be requested again before each already-authorized bounded run. A project with no such authorization cannot obtain it merely by inheriting example template language.

For model validation within an authorized task, distinguish the billing route from the test itself:

- **Existing subscription:** Use the current subscription-backed session for bounded validation covered by that subscription. Do not require separate billing approval, a newly designated provider/account, or a monetary cap for included usage. Existing task scope, runtime limits, subscription quotas, and explicit operator restrictions still apply.
- **Metered API or other additional charges:** Ask before incurring charges without an applicable approved spend envelope. This includes separately billed credits and subscription overage. If the run is already covered by an approved envelope, proceed without asking again; count retries toward it.
- **Uncertain billing route:** Reuse known session information or inspect available non-secret configuration first. Ask only if the billing route remains unclear. Do not switch accounts/providers, buy credits, or enable paid fallback to bypass a limit without authorization.

Apply this distinction when filling or resuming a contract: do not make provider/account/budget selection a blanket prerequisite for every real-model test. Subscription access does not authorize unrelated work or waive a task's separate real-training approval requirement.

## Freeze and continuous updates

`DESIGN FROZEN` freezes the agreed objective, scope, invariants, acceptance, and material constraints. Checklists, audit findings, implementation facts, test evidence, and bounded design corrections remain writable. Record changed assumptions and their reasons rather than presenting them as the original plan.

A bounded discovery can change the route to the approved result. It cannot change the result or a binding constraint. Material revisions return the affected design to review; preserve completed work and record renewed approval before executing the revised scope.

## PR review, CI, and merge

The operator can review the diff while CI runs. That overlap does not mean the agent has reached its terminal condition or received merge authorization. `READY FOR OPERATOR REVIEW` requires the agreed implementation/review/validation work and canonical CI on the exact final head.

Opening/updating the PR and repairing CI are autonomous when included in the approved execution contract. A planning-only or local-only request does not inherit permission to publish a branch. If the user explicitly selects a local-only endpoint, record that endpoint before execution and report it as local completion, not a remotely validated PR.

Existing required CI checks remain binding. The prompt's selective-CI and stacked-PR guidance does not authorize disabling checks or rebuilding CI infrastructure as part of an unrelated feature PR.

## Skill instructions and runtime enforcement

This edition supplies instructions and a hook behavior contract, plus explicitly optional [continuity](continuity.md) and [checkpoints](checkpoints.md) presets. Default installation registers no host hooks; none of these modes changes permissions. Continuity covers compact freshness, snapshot attempts, and recovery instructions, not freeze/merge enforcement. Checkpoints supplies commit-preparation and review-intent reminders with zero automatic continuation, not H2/H7 enforcement or test attestation. Follow the remaining checks procedurally. Do not request broad access changes solely to prevent all possible interruptions.
