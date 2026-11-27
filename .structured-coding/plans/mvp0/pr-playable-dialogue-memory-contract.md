# Playable dialogue and persistent memory — filled execution contract

Lifecycle: **FROZEN** with [`pr-playable-dialogue-memory.md`](pr-playable-dialogue-memory.md) revision
DM-20261127-3. Approval: operator, 2026-11-27, "同意三个都按推荐冻结" (Q-D1…Q-D5 as recommended), relayed by
the primary coordinator. The working-rule body below is copied verbatim from
`.claude/skills/structured-coding/prompts/implementation-working-rules.md` (skill VERSION 0.1.3, blob
`63fa26b0`, lines 242–1044; verified verbatim by the pre-freeze review) and is not replaced by this block. The
execution owner also reads `prompts/test-ci-gate-rules.md` in full.

## Project-specific implementation contract

```text
PROJECT / PR:
  MineWorld MVP-0 — Playable dialogue and persistent contextual memory (lane L3).
  Two PRs under one frozen design: PR-A = C1; PR-B = C2–C5.

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/pr-playable-dialogue-memory.md (§10 holds each PR's ledger)

RELATED / BINDING DOCS:
  CLAUDE.md §§1.1, 2, 3.1–3.4, 4; docs/ENGINEERING_RULES.md; docs/ENGINEERING_STANDARDS.md;
  docs/ARCHITECTURE.md §§8–9; docs/MODULE_SPEC.md §5 (LM Controller); docs/reference/cognition-toml.md;
  docs/decisions/ARC-57.md, ARC-59.md, ARC-73.md, ARC-78.md, DEP-58.md; tools/launch/LAUNCHER.md;
  .structured-coding/plans/mvp0/operator-playable-mvp-20261123.md §§1–4;
  .structured-coding/plans/mvp0/{step-17-cognition,step-23-release,step-28-shared-memory}.md;
  .structured-coding/standards.md; dialogue-memory-evidence/integration-audit-20261123.md.

IMPLEMENTATION BASE:
  PR-A: origin/main @ 8c971f0f7b469f05d1bf6f21fc933501eb22c404 (or newer main at dispatch; record the SHA).
  PR-B: dependent on the recovered client integration stack (recovered/integration-d586754a @ e37ce9fd,
  local only on 2026-11-27). Starts from the main SHA that contains the merged client-stack recovery and
  merged PR-A; launcher seams re-audited on that SHA and recorded in design §10 before C2.
  Source: operator ruling Q-D4, 2026-11-27.

APPROVED SCOPE:
  Design §1, §3, commits C1–C5 (§5), live gate §6.

FROZEN INVARIANTS:
  Design §4 items 1–14 and §0 bounded resolutions B1–B3, plus:
  - no LM/provider concept in kernel, System, protocol, World Pack or launcher contracts;
  - world runs with no model and no renderer; core tests use scripted/recorded cognition only;
  - no new cognition framework, database, provider SDK, kernel Person field, client game logic;
  - launcher never downloads a model/runtime and never starts or stops Ollama;
  - R1 never described as self-contained (Q-D1);
  - profile: Market Town; LM alice; human visitor (2D/3D solo); both-mode 2D visitor + 3D wanderer;
    all others rule (Q-D2).

APPROVED IMPLEMENTATION SEQUENCE:
  PR-A: C1 → terminal CI on exact head → READY FOR OPERATOR REVIEW.
  PR-B: C2 → C3 → C4 → C5 (live gate inside C5) → terminal CI on exact head → READY FOR OPERATOR REVIEW.
  Each PR starts in a fresh execution session.

AUTONOMOUS VALIDATION BUDGET (source: operator ruling Q-D3, 2026-11-27):
  - ordinary targeted unit/integration/static tests: allowed; no local full-workspace suite
    (R-LOAD-1: CI on the exact head is the gate);
  - real-process lifecycle runs (scripted backend): each <= 10 min; one shared heavy-job slot;
  - real model calls (PR-B C5 only): local Ollama qwen3.5:4b, design §6: <= 8 calls total including
    the counted warm-up, enforced by a gate ledger (calls_per_wall_hour = 7) plus harness count;
    <= 180 s inference total; <= 20 s per reply; 256 output tokens per call; 5 scripted player turns;
  - total real-process wall time across C3–C5: <= 60 min;
  - monetary: 0. No cloud/paid endpoint, no download, no model-service start;
  - PR-A: no live model at all;
  - anything beyond these limits: stop with a projection and ask the operator.

REQUIRED LIVE DOCUMENTATION:
  PRIMARY DESIGN DOC (§5 checkboxes, §7 status, §10 ledger, deviations).

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff-playable-dialogue-memory.md (created by each execution owner;
  never a second design authority; replaced, not appended, at the PR-A → PR-B boundary).

PR CONTEXT SCOPE / INITIALIZATION / AUTHORITY ORDER / CONTINUITY / COMPACTION / RESUME / CLOSEOUT:
  As in the working-rule body below. One worktree under /Users/yuema137/mineworld-worktrees/ per
  branch, held by one session; never /tmp; never the root checkout.

ENDPOINT AUTHORITY (source for each line: operator ruling Q-D5, 2026-11-27):
  - implementation + local validation: authorized.
  - semantic commits: authorized, no approval per commit (working rules §14).
  - branch push: authorized for the PR's own branch only (working rules §21).
  - PR creation / update: authorized (working rules §21).
  - CI repair to review readiness: authorized (working rules §21; CLAUDE.md §3.4).
  - merge: explicit operator authorization only; section 22. No source value changes this line.

POST-MERGE SYNCHRONIZATION OWNER:
  Execution session: design §10 ledger, merge identity, evidence, deviations, remaining issues.
  Primary coordinator: operator-playable-mvp-20261123.md, overall.md, progress page.

NORMAL STOP CONDITION:
  PR-A: C1 READY FOR OPERATOR REVIEW — DO NOT MERGE.
  PR-B: playable dialogue+memory READY FOR OPERATOR REVIEW — DO NOT MERGE.

STOP CONDITION:
  PR-A ready for operator review:
    C1 implemented + its validation complete + PR opened/updated + terminal CI green on exact final HEAD.
  PR-B ready for operator review:
    C2–C5 implemented + required validation complete
    + live gate run with result PASS, or FAIL/INCONCLUSIVE recorded with "check 2 NOT MET" in the PR body
      and design §10, followed by the operator choice of design §0 (Q-D3 consequence)
    + PR opened/updated + terminal CI green on exact final HEAD.

MERGE AUTHORITY:
  NEVER merge without explicit operator approval.
```

---

# 1. Inspect before asking, guessing, or changing the design

When anything is uncertain, do not guess from memory and do not immediately  
ask the operator.

Use this order:

1. re-read the relevant section of the authoritative design document;  
2. inspect the actual implementation;  
3. inspect callers and downstream consumers;  
4. inspect tests, schemas, fixtures, records, and preserved evidence;  
5. inspect relevant git history when behavior may be historical or partially  
   migrated;  
6. inspect runtime logs/artifacts if the question concerns observed behavior;  
7. when the issue depends on external software or platform behavior, inspect  
   authoritative external sources as described in §2;  
8. reason from the gathered evidence;  
9. resolve the question autonomously when the evidence supports one bounded  
   answer;  
10. stop for operator input only when a genuinely material decision remains.

A question that can be answered through source/code/evidence audit is not an  
operator decision point.

Never implement an interface, control-flow assumption, schema field, runtime  
property, or library behavior merely because it "probably works that way."

---

# 2. Autonomous external research / web-search policy

External research is allowed and encouraged when repository evidence alone  
cannot establish an external technical fact.

Examples include:

* Python/runtime semantics;  
* PyTorch / CUDA behavior;  
* signal / subprocess / OOM behavior;  
* operating-system behavior;  
* GitHub Actions / CI behavior;  
* compiler/runtime behavior;  
* third-party library contracts;  
* upstream API semantics.

Use this priority:

1. repository source and tests;  
2. vendored/upstream source code;  
3. official documentation;  
4. primary technical references;  
5. secondary sources only when necessary.

Do not use web search as a substitute for reading the local codebase.

When external research materially affects implementation or interpretation,  
record in the relevant design doc:

* what question required external research;  
* what authoritative source was consulted;  
* the conclusion;  
* how it changes or confirms the implementation;  
* whether it creates any limitation or follow-up.

Do not silently introduce an externally-derived assumption.

---

# 3. Re-read before editing an existing file

Before modifying any existing file:

1. open and re-read the relevant section;  
2. inspect nearby code/documentation constraining the change;  
3. inspect important callers/consumers when applicable;  
4. identify the exact invariant being preserved or corrected;  
5. then edit.

Never edit an existing file solely from memory of an earlier read.

This applies to:

* production code;  
* tests;  
* schemas;  
* fixtures;  
* design docs;  
* CLI/docs;  
* generated/evidence readers;  
* CI configuration.

For a repeated edit to the same file during one tightly-coupled change, it is  
not necessary to ceremonially reread the entire file, but the relevant current  
section must remain in context.

---

# 4. The design document is a LIVE implementation ledger

The relevant design document must be updated continuously while implementation  
is happening.

Do not postpone important discoveries until the end of a commit or PR.

Whenever a planned item is completed:

* mark it `[x]`;  
* record the concrete implementation;  
* record the actual files/functions involved;  
* record validation evidence;  
* update affected status/milestone sections.

Whenever implementation reveals something not fully anticipated by the  
design, record it immediately.

This includes:

* source behavior differing from the original assumption;  
* corrected call graphs;  
* hidden consumers;  
* unexpected control flow;  
* stale comments/contracts;  
* newly discovered edge cases;  
* test failures and their diagnosis;  
* mutations that survive and why;  
* runtime/LLM/GPU observations;  
* failed or abandoned approaches;  
* implementation deviations;  
* additional tests introduced;  
* limitations;  
* follow-up debt;  
* changes in validation strategy;  
* decisions not to implement something and why.

Use this pattern whenever an original premise changes:

```text  
Previous assumption:  
  ...

Audit evidence:  
  ...

Corrected understanding:  
  ...

Implementation consequence:  
  ...

Validation consequence:  
  ...  
```

Do not silently rewrite the design as though the earlier assumption never  
existed when preserving the history is useful for future reviewers.

---

# 5. Decision log discipline

Material implementation decisions should be recoverable from the design doc,  
not only from terminal/chat history.

For each non-trivial decision, record enough information to answer:

* What was the question?  
* What source/evidence was inspected?  
* What options were realistically available?  
* What option was chosen?  
* Why?  
* What invariant does it preserve?  
* What validation proves it?  
* Did it differ from the original plan?

Small local implementation choices do not require a formal decision entry,  
but anything that changes the interpretation of the approved design does.

---

# 6. Autonomous modification policy

Within the approved design and frozen invariants, implementation should  
continue autonomously.

The agent is expected to:

* inspect;  
* reason;  
* edit;  
* refactor narrowly;  
* add tests;  
* repair routine failures;  
* improve local implementation structure;  
* update docs;  
* run bounded validation;  
* create semantic commits;  
* push/update the branch;  
* open/update the PR;  
* inspect CI failures;  
* fix them;  
* iterate until the PR is ready for review.

Do not stop merely because an implementation detail was not literally spelled  
out in the design document.

If code/evidence provides a defensible bounded answer consistent with approved  
principles, proceed and document it.

---

# 7. Design ambiguity policy

The design document guides implementation but is not assumed to have predicted  
every implementation detail.

When an ambiguity or omission appears:

1. inspect the relevant source;  
2. inspect callers/consumers;  
3. inspect design sections;  
4. inspect tests/fixtures/evidence;  
5. inspect history when relevant;  
6. determine whether the issue is bounded by already-approved principles.

If yes:

* resolve it autonomously;  
* implement the narrowest correct solution;  
* document the discovery and resolution.

Stop for operator input only when the remaining choice is material, such as  
when it would:

* change a frozen invariant;  
* alter scorer/metric semantics;  
* alter ownership between subsystems;  
* change a public schema/interface;  
* introduce a new artifact or record kind;  
* add a new dependency;  
* create a new abstraction layer with non-local consequences;  
* reorder major phases;  
* materially change runtime/admission/retry semantics beyond the approved  
  change;  
* contradict preserved evidence;  
* substantially expand scope;  
* choose between genuinely different architectures;  
* invalidate the approved product/release claim.

Do not silently choose a material interpretation.

---

# 8. Autonomous validation policy

Validation should use the minimum sufficient evidence needed to establish the  
property under test.

Use this ladder where applicable:

```text  
source/static audit  
    ->  
focused unit test  
    ->  
negative/edge-case test  
    ->  
mutation/adversarial test  
    ->  
pseudo/integration test  
    ->  
bounded real Gate 1  
    ->  
bounded real Gate 2  
    ->  
broad/full suite  
    ->  
terminal CI  
```

Not every change requires every level.

Do not escalate merely for ceremony.

---

# 9. Gate 1 / Gate 2 autonomy

Small-scale Gate 1 and Gate 2 validation may be launched autonomously.

The names "Gate 1" and "Gate 2" are project-specific; the important distinction  
is cost/risk, not the label.

Unless the project-specific contract says otherwise, the following are  
allowed without asking:

* real LLM calls;  
* real inference;  
* pseudo-training;  
* small real-training runs;  
* bounded GPU runs;  
* bounded CPU runs;  
* cold-start integration tests;  
* one/few-model end-to-end tests;  
* small real-data validation;  
* bounded combinations of real LLM + real training;

provided that all of the following are true:

1. the test directly validates an approved invariant;  
2. it is non-destructive;  
3. it does not overwrite preserved evidence;  
4. expected total runtime is approximately <= 1 hour;  
5. compute/API/monetary cost is modest for the project;  
6. it does not create significant interference for shared infrastructure;  
7. the result can be cleanly attributed to the tested change.

Before launching a non-trivial real Gate:

* estimate expected wall time;  
* identify hardware/API usage;  
* identify the exact question the Gate answers;  
* record the planned Gate in the live design doc.

After it finishes:

* record actual runtime/cost/context;  
* record exact result;  
* record whether it passed, failed, or was inconclusive;  
* record deviations from the expected execution.

Do not turn a small Gate into an empirical campaign without a new reason.

---

# 10. When approval is required for validation

Stop before launching a validation run when:

* expected runtime materially exceeds the project-specific autonomous limit  
  (default ~1 hour);  
* projected API/monetary cost is material;  
* a large GPU/CPU campaign is proposed;  
* the run could disrupt a shared machine/service;  
* preserved evidence could be overwritten or invalidated;  
* external state would be destructively changed;  
* the test changes production/customer state;  
* the test scope has expanded beyond what the approved design needs.

Before asking, estimate from source or prior evidence:

* expected wall time;  
* CPU/GPU requirements;  
* LLM/API usage;  
* monetary cost if applicable;  
* what additional information the larger run would provide.

Always apply:

> **minimum sufficient evidence**

Do not spend hours of compute merely to turn an already-established property  
into an exhaustive census.

---

# 11. Test-failure discipline

When any test or Gate fails:

1. read the complete traceback/log/output;  
2. inspect the failing test;  
3. inspect the production path it exercises;  
4. reproduce/narrow when useful;  
5. classify the likely source:

   * implementation;  
   * test;  
   * fixture;  
   * environment;  
   * external dependency;  
   * pre-existing baseline failure;  
   * unresolved;  
6. establish evidence before editing.

Do not immediately:

* weaken an assertion;  
* add a retry;  
* increase a timeout;  
* skip the test;  
* patch production code based only on the final error line.

If the fix is a routine correction within the frozen design:

* fix it autonomously;  
* add/strengthen targeted validation;  
* document the diagnosis and fix;  
* rerun the appropriate validation.

If the honest fix changes approved semantics or scope, stop for operator input.

For a suspected pre-existing failure, prove it at the relevant merge base /  
clean baseline before classifying it as pre-existing.

---

# 12. Mutation / adversarial testing discipline

For behavior-changing logic, actively test that the intended contract is  
actually pinned.

Where useful, construct mutations such as:

* deleting a transport hop;  
* reversing precedence;  
* falling back silently;  
* changing a threshold/bound;  
* collapsing distinct dispositions;  
* changing exact evidence into censored evidence;  
* ignoring a newly-threaded value;  
* reintroducing a stale default;  
* changing a guard predicate.

Record:

* mutation attempted;  
* expected failure;  
* observed result;  
* whether behavior-changing / equivalent / invalid;  
* any surviving mutation;  
* how the test architecture was strengthened.

A survived mutation is a signal to inspect the test architecture, not merely  
to add a superficial assertion.

---

# 13. Preserve evidence and frozen evaluation semantics

Preserved measurement/scientific evidence is authoritative.

Do not alter historical evidence merely to make implementation or prose agree.

Do not modify a frozen scorer/metric/aggregation rule in response to surprising  
results unless the approved design explicitly changes it.

When evidence and documentation disagree:

* audit;  
* preserve the evidence;  
* correct the prose or implementation if appropriate;  
* record the discrepancy and resolution.

If existing evidence reveals that the approved design itself is materially  
wrong, stop and report rather than silently redefining the task.

---

# 14. Commit policy

Commits are autonomous.

Do not stop for operator approval before each commit.

Create semantic commits at meaningful milestones.

A design's listed "commit" may be split into multiple smaller semantic commits  
when that improves reviewability.

Good commit boundaries include:

* inert seam/API;  
* live behavior wiring;  
* validation/contract pin;  
* evidence/report;  
* documentation reconciliation;  
* CI-specific fix.

Avoid:

* one giant opaque commit;  
* trivial one-line noise commits;  
* commits that mix unrelated changes.

Before each semantic commit:

* inspect the diff;  
* confirm no accidental files are included;  
* update the live design doc with the state being committed;  
* record validation relevant to that milestone.

Then commit and continue automatically.

---

# 15. Continuous deviation tracking

Any deviation from the approved implementation plan must be classified.

### Bounded deviation

Examples:

* actual function lives in a different module;  
* an extra caller must be threaded;  
* one planned test is better implemented at another seam;  
* a stale comment requires correction;  
* a small helper is needed;  
* one additional negative case is required.

Proceed autonomously and record:

```text  
Deviation:  
Reason:  
Source evidence:  
Impact:  
Validation:  
```

### Material deviation

Examples:

* approved behavior is impossible as designed;  
* phase ordering must change;  
* scorer/metric semantics would change;  
* public contract/schema must change;  
* new dependency/artifact/architecture is required;  
* validation reveals the central design assumption is false.

Stop before implementing the material deviation.

---

# 16. Live test/evidence log in the design document

The design doc should accumulate the actual validation record.

For each meaningful validation, record when applicable:

```text  
Validation:  
  command / Gate:  
  purpose:  
  environment:  
  runtime:  
  data/model:  
  expected result:  
  actual result:  
  pass/fail/inconclusive:  
  artifact/log:  
  interpretation:  
```

For cheap unit tests, a compact count is sufficient.

For real LLM/GPU/training/Gate runs, record enough context to make the result  
auditable and reproducible.

Do not leave important evidence only in `/tmp`, shell history, or chat output.

---

# 17. Context-compaction / session-continuity policy

Do not allow important implementation state to exist only in the current model  
context.

If the repository defines a handoff/memory file, keep it current.

Before context becomes tight or before expected compaction, update the handoff  
with at least:

* branch;  
* HEAD/base;  
* active PR;  
* authoritative design doc;  
* frozen invariants;  
* completed implementation stages;  
* current stage;  
* semantic commits;  
* important source discoveries;  
* deviations;  
* test/mutation results;  
* real Gate status;  
* running/background processes;  
* CI status;  
* unresolved issues;  
* exact next action.

After a compaction or fresh session:

1. inspect git state;  
2. read the authoritative design doc;  
3. read the handoff;  
4. inspect active/background processes;  
5. re-open the relevant source seam;  
6. only then continue.

Do not restart completed work merely because conversational memory was lost.

---

# 18. Background-process discipline

Before launching a new long-running/background job:

* inspect whether an equivalent job is already running;  
* know where stdout/stderr/results are being written;  
* know its expected stop condition.

Periodically audit long-lived shells/processes.

Classify them as:

* active + required;  
* active + obsolete;  
* passive watcher;  
* stale/orphaned;  
* unknown.

Do not kill unknown work blindly.

Clean up obsolete processes after confirming useful output is persisted.

This is particularly important when benchmark measurements are sensitive to  
host load.

---

# 19. Documentation synchronization

The detailed design document is the live authority during implementation.

Higher-level ledgers/status documents should be synchronized according to the  
repository's established convention.

Do not prematurely rewrite a canonical planning ledger when the repository  
normally updates it only after merge.

When the detailed design supersedes an old ledger premise:

* record the supersession explicitly;  
* preserve the audited reason;  
* update the higher-level ledger at the appropriate lifecycle point.

---

# 20. When to pause

Pause only when:

* code/doc/evidence audit cannot resolve a required material decision;  
* a materially different architecture is required;  
* implementation must substantially depart from the frozen plan;  
* a new public schema/interface/artifact/dependency is necessary;  
* frozen evidence contradicts a required invariant;  
* scorer/metric semantics would change;  
* subsystem ownership would change;  
* a destructive/irreversible action is required;  
* a validation exceeds the approved autonomous cost/time envelope;  
* credentials/access are genuinely unavailable;  
* work is otherwise genuinely blocked.

Do NOT pause for:

* routine implementation details;  
* bounded source audits;  
* bounded web research;  
* ordinary refactors preserving frozen behavior;  
* additional unit/integration tests;  
* bounded Gate 1 / Gate 2 runs;  
* routine test failures;  
* mutations;  
* documentation updates;  
* semantic commits;  
* PR creation;  
* ordinary CI failures that can be diagnosed and fixed;  
* waiting for a bounded test/CI job if other useful audit/documentation work  
  can be done safely in parallel.

---

# 21. PR / CI completion policy

Implementation continues until the PR is READY FOR OPERATOR REVIEW.

After implementation:

1. run the required terminal validation from the appropriate clean/final  
   executable head;  
2. run static/type/format checks;  
3. push the branch;  
4. create or update the PR;  
5. inspect the PR diff/body;  
6. wait for CI;  
7. inspect failures autonomously;  
8. fix routine problems;  
9. push the new head;  
10. wait for CI again;  
11. verify CI success on the EXACT final HEAD SHA;  
12. verify clean working tree.

Do not stop merely because:

* local tests are green;  
* a semantic commit was created;  
* the PR was opened;  
* CI started;  
* the first CI failed;  
* a watcher is waiting.

The stopping condition is:

> **PR READY FOR OPERATOR REVIEW**

unless a material operator decision is required earlier.

---

# 22. Final operator review and merge authority

Before merge:

STOP and present a complete handoff.

Include:

* PR number/link;  
* branch/base/final HEAD;  
* semantic commits;  
* implementation summary;  
* changed files;  
* source discoveries;  
* deviations from design;  
* design-doc updates;  
* targeted tests;  
* integration/pseudo tests;  
* Gate 1 / Gate 2 evidence;  
* real LLM/training/inference evidence when applicable;  
* mutation results;  
* full-suite result and actual return code;  
* static/type/format results;  
* known limitations/follow-ups;  
* CI run and exact `headSha`;  
* clean working-tree state;  
* why the PR is ready.

Then wait for explicit operator permission to merge.

Never merge first and report afterward.

---

# 23. Intended autonomous workflow

The default workflow is:

```text  
recover actual state  
    ->  
re-read design + relevant code  
    ->  
implement  
    ->  
audit  
    ->  
update design doc  
    ->  
run cheapest sufficient validation  
    ->  
diagnose/fix autonomously  
    ->  
update design doc  
    ->  
semantic commit  
    ->  
continue to next milestone  
    ->  
bounded real Gate(s) when needed  
    ->  
terminal validation  
    ->  
PR  
    ->  
CI  
    ->  
fix/iterate  
    ->  
exact-final-head CI green  
    ->  
STOP FOR OPERATOR REVIEW  
    ->  
explicit merge approval  
```

In short:

> **implement continuously → audit continuously → research when needed →  
> test continuously → update the design ledger continuously → commit  
> autonomously → iterate through PR/CI → stop only at a genuine material  
> decision or the final pre-merge operator review.**
