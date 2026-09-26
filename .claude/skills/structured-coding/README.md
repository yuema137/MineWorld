<!-- Generated file. Source: docs/content.en.json in the structured-coding repository, built by scripts/build_human_docs.py. Direct edits here are overwritten by the next build. -->

# Structured Coding

[Chinese mirror](README.zh-CN.md)

Agree on the change. Let the agent build it. Bring the result back into the plan.

> **When this is worth it, and when it is not**
>
> This is built for sustained work on a large codebase. It earns its overhead when a change spans several PRs or sessions, when the code is big enough that an agent must audit before it edits, and when someone has to pick the work up later. For a typo, a one-file bug fix, or a throwaway prototype it is overkill: the planning documents will cost more than the change itself. Use your agent directly for those. Plans, tests, and LLM reviews can still be wrong; what this workflow adds is that their assumptions and evidence are written down where you can inspect them.

[What using it looks like](#example) · [Step-by-step tutorial](#tutorial) · [Going further](#further) · [Optional hooks: installation, coverage, and limits](#hooks)

## You already have this installed

This guide ships inside the skill, so you are reading it in a project that already has it. Invoke the skill explicitly and describe what you want built; ask for planning first, not implementation.

Prefix your request with $structured-coding in Codex or /structured-coding in Claude Code. Describe the feature and its constraints; ask for planning first, not implementation.

<a id="example"></a>

## What using it looks like

Three messages carry one feature from an idea to a reviewed PR. You approve twice; the agent does the work in between.

### 1. Plan the feature

```text
Use the structured-coding skill for this feature. Read its SKILL.md
entrypoint first and load the complete resources its table lists for the
current phase.
First agree with me on requirements, module-level direction, and overall
step boundaries; then detail the current step. Work on planning for now.
Requirements: ...
```

The agent asks what it cannot infer, inspects your actual code, and writes the overall plan with step boundaries. Nothing is implemented yet.

### 2. Prepare the next PR

```text
Read the overall and step documents, audit the current code, and prepare the
PR 01a design doc and filled execution contract. Use structured-coding:
start from its SKILL.md entrypoint and load what the PR design row lists.
Follow the original PR requirements for the commit checklist. Separate
implementation, validation, and review, and prepare the design for my
approval.
```

You get a PR design with an audited commit plan and a filled execution contract. Read it, ask for changes, and approve it once it describes what you actually want built.

### 3. Execute after approval, in a fresh session

```text
Execute PR 01a. The approved DESIGN FROZEN document is
.structured-coding/plans/order-flag/pr-01a.md, and the filled contract is
.structured-coding/plans/order-flag/pr-01a-contract.md.
Use structured-coding: read its SKILL.md entrypoint, then the complete
resources its execute row lists, including Implementation Working Rules and
TEST / CI / GATE in full. Reconcile actual state, and begin.
Continue autonomously to READY FOR OPERATOR REVIEW under the contract.
Do not merge.
```

A fresh session implements, validates, reviews its own logic, commits, and stops at a review handoff. You read the diff and decide whether to merge.

That is the whole loop. After a confirmed merge the agent updates the plans with what it learned, and the next PR starts from there.

## Why use Structured Coding?

A plan describes what you intend to build. It does not, on its own, tell an agent how to execute, what counts as evidence, when to ask you, or how to resume after losing context. This skill connects those decisions into one repeatable workflow.

## Where you step in.

You do not need to approve every commit. You do need to own the decisions that change the agreement.

After design approval, the agent investigates, implements, validates, reviews, records, and commits. If PR and CI work are authorized, it completes those too without waiting for you to prompt each step. The contract records which endpoints you authorized — commit, push, opening the PR, CI repair — and merge is never one of them. The agent may not quietly narrow that list either: a stopping point you did not ask for is a question for you, not a cautious default it can adopt on your behalf.

A material scope change or an action outside existing authorization comes back to you with evidence and a proposal.

<a id="roles"></a>

## What do you, the specification, the agent, and the hooks each do?

Think of the specification as the written requirements, not a program watching every action. The agent reads those requirements and applies them to your project. A hook is a small program the host runs at a supported event, such as compact or a tool call. It can check or remind only where its implementation actually provides that behavior.

| Who or what | What it is responsible for | What it does not replace |
| --- | --- | --- |
| You | You define the intended behavior, approve the current PR design, decide material changes, and explicitly authorize merge after review. | You do not have to approve each ordinary fix, test, or commit that is already inside the agreed scope. |
| Specifications and prompt templates | They define what belongs in a PR design, how the agent executes, what counts as validation, and where authorization is required. | Written rules do not automatically block tools. The hook contract includes future requirements, not only features that are shipped. |
| The agent | It reads the full applicable rules, inspects code, drafts the plan, implements, tests, reviews logic, and keeps the design and handoff current. | A completed checklist or confident answer cannot replace test evidence or your approval. The agent remains responsible for following rules even without hooks. |
| Optional hooks | Continuity checks a recorded checkpoint before manual compact, attempts a rescue snapshot for automatic compact, and supplies recovery instructions. Checkpoints supplies commit-preparation advice and a review-intent notice. | They do not understand every design decision, prove tests passed, enforce design freeze, or prevent every merge. They do not run an automatic continuation loop. |

For example, the specification requires the PR design to record validation evidence. The agent runs the test and records its result. A checkpoints reminder can tell it to inspect missing evidence, but cannot decide that the test passed. You review the result before authorizing merge. Installing more hooks does not remove any of those responsibilities.

<a id="sessions"></a>

## How many conversations do you need?

A practical arrangement is one planning conversation and one fresh implementation conversation for each PR. The planning conversation can cover requirements, the overall plan, the current step, and the next PR design; these do not each need their own chat. The workflow requires a fresh implementation context for each new PR, not a separate chat for every commit or test. This separation matters because planning often contains rejected proposals and superseded assumptions. A fresh implementation session starts from the approved files and current code, reducing the chance that an old discussion is mistaken for the final requirement.

| Conversation | What you do there | When to switch |
| --- | --- | --- |
| Planning conversation | Discuss requirements, ask the agent to inspect the repo and write the plans, review the current PR design, and approve its execution contract. | When that PR is approved, ask for a kickoff with the actual document paths and open a fresh implementation conversation. |
| Implementation conversation for PR A | Give the approved design and contract to the agent. Let it implement, validate, review, commit, and handle authorized PR/CI work. | Keep ordinary fixes, commits, and compact/resume in the same PR context. Do not start PR B here. |
| Review of PR A | Read the diff and handoff. Request repairs in the same implementation conversation, or explicitly authorize merge after you are satisfied. | A separate reviewer conversation is optional, not required. Repairs need updated evidence and CI for the final HEAD. |
| Planning and implementation for PR B | After A is confirmed merged, the implementation conversation records A's merge identity and evidence; the synchronization owner — by default this planning conversation — updates A's parent step and the overall plan, then uses those records to detail and approve B. | Finishing A does not finish the plan: the overall stays open until every step it listed is delivered or you drop it. You can return to the planning conversation or open a replacement that reads the saved plans. Start B in another fresh implementation conversation. |

For a feature with two PRs, that usually means three working conversations: planning, implementation A, and implementation B. This is an example, not a fixed quota. A long planning conversation may need replacement, and an interrupted implementation may need recovery. Saved project documents, rather than another chat's memory, carry the agreement between sessions. A handoff is the agent's saved continuation note: what is done, what is still running, and what to do next.

<a id="tutorial"></a>

## Your first feature, from installation to the next PR

Follow this example in order. Suppose your application reads files and you want an optional alphabetical order while preserving the current default. The paths and PR label in the sample messages are placeholders, not files supplied by this toolkit. Ask the planning agent to create the real documents first, then replace the placeholders before execution. The English entry messages below are identical in both language versions; they do not replace the full original prompts.

### 1. Open the planning conversation and explain the result you want.

Open your target project in your chosen agent host. Invoke $structured-coding in Codex or /structured-coding in Claude Code, then send the planning request below with your actual requirements.

For the example, say that alphabetical mode must visit [c, a, b] as [a, b, c], and that leaving the option off must preserve the old behavior. Explain what is outside scope, such as changing the resume mechanism in this PR. The agent should inspect the repo, ask about unresolved product decisions, and propose an overall direction. It should not start implementing merely because you asked for a plan.

```text
Use the structured-coding skill for this feature. Read its SKILL.md
entrypoint first and load the complete resources its table lists for the
current phase.
First agree with me on requirements, module-level direction, and overall
step boundaries; then detail the current step. Work on planning for now.
Requirements: ...
```

**Before you move on**

Before moving on, you should be able to explain the goal and main steps in your own words. Ask the agent to rewrite anything you cannot review; you do not have to author its design document yourself.

### 2. Ask the same planning agent to detail only the next PR.

The overall plan describes the feature and its main steps. A step plan explains which PRs fit together. The current PR design goes deeper: the agent reads real code and callers before naming the files, functions, commits, and checks. Later PRs can remain less detailed because this implementation may reveal new facts. The PR requirements define the detailed format. Ask the agent to follow that complete specification; you do not need to reconstruct the template yourself.

For the alphabetical-order PR, acceptance must observe the reader visiting [a, b, c], not just a configuration value being stored. Ask for a check that would fail if a caller silently dropped the option. Each commit needs separate implementation, validation, and logic-review items. If a step needs only one PR, the agent can expand the step document in place instead of maintaining a duplicate.

```text
Read the overall and step documents, audit the current code, and prepare the
PR 01a design doc and filled execution contract. Use structured-coding:
start from its SKILL.md entrypoint and load what the PR design row lists.
Follow the original PR requirements for the commit checklist. Separate
implementation, validation, and review, and prepare the design for my
approval.
```

**Before you move on**

The agent should give you the real PR design path and a filled execution contract. Agree where these records live and which, if any, belong in Git. Private planning notes and raw logs do not automatically belong in the published product.

### 3. Review the agreement and explicitly approve this PR.

Check what changes, what stays unchanged, what is excluded, and what observable result will count as done. The execution contract should also say whether the agent may commit, push a branch, open or update a PR, and repair CI, and where it must stop. These are separate permissions; asking for local implementation alone does not authorize publication.

When the design matches your intent, explicitly approve that concrete design and contract. The agent records DESIGN FROZEN and the approval reference. Freeze protects the agreed scope, invariants, and acceptance, not the whole file: the agent must still update discoveries, progress, and evidence. Approval of implementation is not approval to merge.

For authorized validation, existing subscription-covered usage does not need another provider/account or billing question. Metered API calls and separately charged usage need applicable spending authorization. Existing time limits, quotas, and explicit restrictions still apply; the agent must not switch accounts or enable paid fallback to evade them. Separate approval requirements for real training still apply when the task has them.

**Before you move on**

Ask for a kickoff that identifies the approved design, filled contract, implementation base, next action, and stopping condition. Do not copy a template containing unresolved paths into execution and assume it is ready.

### 4. Open a fresh implementation conversation for this PR.

Start a genuinely new conversation in the same target project. Do not just rename the planning conversation. Invoke the skill again and give it the kickoff with the actual paths. The new agent does not need the entire planning chat: it needs the durable agreement and the source files that establish current state.

Before editing, the agent must re-read the skill entrypoint and its execute row, then the approved design, filled contract, and complete execution and test rules. It checks the branch, HEAD, existing edits, merged prerequisites, and relevant running jobs. It preserves unrelated work. If you explicitly enabled a preset, it also reads that preset's interface and binds this session to the current PR; it must not assume installation selected an active PR for it.

```text
Execute PR 01a. The approved DESIGN FROZEN document is
.structured-coding/plans/order-flag/pr-01a.md, and the filled contract is
.structured-coding/plans/order-flag/pr-01a-contract.md.
Use structured-coding: read its SKILL.md entrypoint, then the complete
resources its execute row lists, including Implementation Working Rules and
TEST / CI / GATE in full. Reconcile actual state, and begin.
Continue autonomously to READY FOR OPERATOR REVIEW under the contract.
Do not merge.
```

**Before you move on**

Confirm that the agent is working on the intended PR and has loaded the approved paths. The kickoff delegates execution within the contract; it does not grant extra host permissions or enable hooks.

### 5. Let the agent complete the agreed implementation loop.

The agent implements a coherent piece, runs relevant checks, reviews the logic and callers, records what happened, and commits. A Unit test failure or an extra caller inside the agreed scope normally means investigate, fix, and continue. It should not ask you to approve every commit. If branch publication and PR/CI work are authorized, it continues through those steps too.

You step in when the proposed solution changes a frozen requirement, public interface, material scope, or approved budget. The agent should bring evidence and a concrete choice, not merely say it is blocked. Review that choice before the dependent work proceeds.

If the conversation reaches compact, you are still working on the same PR. Before manual compact, the agent updates its design and handoff. After compact or resume, it rereads the full rules and checks actual Git and process state. A running test must be checked before launching a duplicate. Continuity helps with mechanical checks and recovery instructions; it does not write a correct semantic handoff for the agent.

**Before you move on**

You should be able to ask for the current milestone, evidence, and next action and get an answer grounded in saved records. A hook notice is not proof that the agent performed a review or completed a test.

### 6. Review the finished PR, then decide whether to merge.

For an authorized PR workflow, READY FOR OPERATOR REVIEW means the agreed implementation, validation, and logic review are complete, the PR is published or updated, and required CI passes on its exact final HEAD. HEAD identifies the current commit. A green result for an earlier commit does not prove a later edit passed.

Read the diff alongside the promised behavior, deviations, test evidence, and remaining limits. If something is wrong, request repairs in the same implementation conversation. The agent should update the evidence and final-head CI before handing it back. You can use a separate reviewer agent, but the workflow does not require another conversation for review.

When satisfied, explicitly authorize merging the specific PR and reviewed candidate. Without that approval, the agent stops at review readiness. The shipped presets do not provide a merge guard, so this boundary remains an instruction and any separately configured host/repository protection. A local-only contract has a local endpoint; the agent must not pretend it created or validated a remote PR.

**Before you move on**

After an authorized merge, require confirmation of the actual remote result and merge commit. A merge command being requested is not the same as a completed merge.

### 7. Update the plans before starting the next PR.

After merge is confirmed, ask the agent to mark the current PR merged, update its parent step, and update the overall plan. The updates should record both completed work and discoveries that change what comes next. Hooks do not currently verify this post-merge planning work.

For example, the alphabetical-order implementation may reveal that resume stores only a filename. A later PR may need a clearer way to identify the next occurrence of a repeated filename. Bring that discovery into the next PR's design instead of continuing from an old assumption.

Return to your planning conversation, or open a new planning conversation that reads the updated files. Detail and approve the next PR, then start another fresh implementation conversation. The previous PR's agent may finish its records and prepare a handoff; it must not quietly begin implementing the next PR in the old context.

**Before you move on**

One PR is finished when its result and implications are recorded, not merely when a merge notification appears. You now repeat the same cycle with a smaller amount of uncertainty.

## What documents does the agent maintain?

| Document | Purpose |
| --- | --- |
| Overall | It records the overall goal, requirements, and main steps so each PR has a clear purpose. |
| Step | It explains which PRs complete this step, their dependencies, and how to observe their integration. |
| PR design | It records audited code and the commit plan, then keeps decisions, progress, and evidence current during implementation. |
| Execution contract | It records this PR's approved scope, permitted actions, budget, and stopping conditions. |
| Handoff | It identifies the current PR, branch and HEAD, running jobs, logs, and the exact next action after recovery. |

DESIGN FROZEN protects the requirements you approved, not the entire document. The agent still records discoveries and progress. Implementation, validation, and review stay separate; each item is checked only when that work has actually been completed. If one step needs one PR, expand the step document instead of keeping two copies.

## What happens at compact or resume?

Compact is the host's process of shortening conversation history to free context; it does not create a new PR. A new PR starts in a fresh implementation session, while compact or resume continues the current PR. The optional continuity preset checks recorded mechanical freshness, attempts snapshots, and supplies recovery instructions. The agent still has to keep the design and handoff accurate and actually reconcile state after recovery.

On resume, reload the current design, filled contract, and full execution rules; check the repo and existing jobs before editing. A snapshot cannot invent decisions or test results. Automatic compact must not get stuck waiting for a perfect handoff.

<a id="hooks"></a>

## Optional hooks: installation, coverage, and limits

Choose continuity for compact recovery, checkpoints for commit/review reminders, standards to run your declared checks after a commit, or any combination. Both are off by default. Checkpoints advises; it does not enforce commits or certify readiness. Protocol tests do not establish native event delivery or model compliance. The agent does not dynamically register its own hooks.

```sh
./structured-coding/scripts/install codex --project /path/to/project --hooks checkpoints --dry-run
./structured-coding/scripts/install codex --project /path/to/project --hooks checkpoints
./structured-coding/scripts/install codex --project /path/to/project --hooks continuity checkpoints
./structured-coding/scripts/install codex --project /path/to/project --check-hooks
./structured-coding/scripts/install codex --project /path/to/project --upgrade-registration
./structured-coding/scripts/install codex --project /path/to/project --remove-hooks checkpoints
./structured-coding/scripts/install codex --project /path/to/project --remove-hooks
```

Run these commands from the same parent directory used for the basic installation. Replace /path/to/project with the target project's exact Git root; use claude-code instead of codex for Claude Code. Preview with --dry-run, then run the installation you chose; this list is a set of alternatives, not a sequence you must run in full. Installation adds selected presets, and removing one leaves the other usable. Bare --remove-hooks removes all hooks owned by this installer. Restart the host and inspect registration and trust in /hooks; the installer does not grant trust. Registered commands contain no path specific to your machine, so a registration committed to a shared settings file also works for your teammates, and each of them still reviews and trusts the hooks on their own machine. A registration made by an older release keeps its absolute paths and stays usable; --upgrade-registration rewrites it, which changes every command and so requires trusting the hooks again. The agent must still bind the current session, inspect staged changes before commit, and explicitly prepare its review handoff. Reminders cannot turn pending, inconclusive, or unrun checks into passes. Existing settings, skill files, and session data are preserved.

| Function | Candidate event | Intended behavior | Shipped support |
| --- | --- | --- | --- |
| H1 · Before implementation | PreToolUse | Check approved design, contract, repo state, and recovery. Deny dependent mutations when they do not match; still allow audit/design preparation. | Not implemented |
| H2 · Before commit | PreToolUse | Check or surface diff inspection, ledger, evidence, and deviations. Repair and retry autonomously; no per-commit human gate. | Checkpoints: explicit preparation and advice for direct git commit via Bash; no enforcement |
| H3 · Before merge | PreToolUse + merge-route coverage | Require trusted explicit approval for the exact PR, target branch, and candidate HEAD, plus readiness and CI/Gate evidence. Cover CLI, API, auto-merge, and direct target-branch bypasses. | Not implemented |
| H4 · Manual compact | PreCompact: manual | Block a stale handoff until synchronized, then allow compact. | Continuity: mechanical checkpoint freshness only |
| H5 · Automatic compact | PreCompact: auto | Allow compact; save a mechanical snapshot when needed. Preserve failures as warnings and require recovery. | Continuity: bounded snapshot attempt; never intentionally blocks automatic compact |
| H6 · Compact/resume | SessionStart + mutation guard | Resolve the current PR dynamically. Reload full rules and reconcile repo/process state before continuing; do not duplicate jobs. | Continuity: session-bound PR and full-read instructions; no mutation guard or proof of recovery |
| H7 · Review readiness | Stop / completion event | Verify real completion conditions, exact-final-HEAD CI, evidence, and handoff. Readiness does not authorize merge. | Checkpoints: explicit intent and at-most-once operator notice; no automatic continuation or evidence verdict |
| H7 · After merge | Observed result / remote-state check | Confirm merge, then request and verify PR → step → overall updates. The agent writes the lessons; the next PR uses a fresh session. | Not implemented |

An event name alone does not guarantee blocking. Adapters must handle each host's protocol, trusted approval source, and tool-coverage gaps. A check after merge cannot prevent it. Read the contract's acceptance scenarios before implementing an adapter.

Example: approval for PR 12 at HEAD A does not authorize merging a later HEAD B. The implementing agent cannot turn its own “approved” field into human authorization.

<a id="further"></a>

## Going further

| Resource | What it covers |
| --- | --- |
| [Workflow reference](references/agent-workflow.md) | What the agent does in each phase, and where its authority stops. |
| [PR specification](prompts/pr-design-requirements.md) | What a PR design must contain before it can be approved. |
| [Optional hooks](references/platforms.md) | Installation, coverage, and the limits of what a hook can enforce. |
| [Behavior contract](references/hook-contract.md) | The complete target behavior, including the parts not yet shipped. |

<a id="standards"></a>

## Project standards (optional)

A project can state, once, what is true of its whole codebase, so you do not repeat it in every planning conversation. Copy the template to .structured-coding/standards.md and edit one block. This file is optional: without it the defaults below apply and nothing changes. Requirements specific to a single PR stay in that PR's conversation, so this file remains a stable repository asset.

| What you set | Where | Default |
| --- | --- | --- |
| Conventions an LLM judges | review.conventions | none; you write plain sentences |
| When those conventions are raised | review.trigger | at PR review readiness |
| Commands with a pass or fail result | checks.tools | ruff and pyright on changed files; pytest listed and off |
| When those checks apply | checks.trigger | at PR review readiness |
| Your own additions | .structured-coding/standards.local.md | may add and tighten only, never relax the team's |

Scope is set per tool because the right answer differs by tool: changed files suit ruff and pyright, and mislead for pytest, whose covering tests usually sit in files the change never touched. Tools are grouped by whether they execute your code, so pytest and mypy are recognized and still need a deliberate approval before anything runs them. Running the checks is an explicit command: each one comes back as PASS, FAIL, INCONCLUSIVE or NOT RUN, and a tool that is not installed is INCONCLUSIVE rather than a pass, because it examined nothing. Nothing runs automatically yet; there is no hook, so no commit and no merge is blocked.

English is the authoritative source; Chinese is a synchronized mirror with English technical terms. The human explanation follows DongbeiGPT. Specifications and reusable prompts are in English.
