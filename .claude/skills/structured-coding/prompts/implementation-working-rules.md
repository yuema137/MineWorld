# Implementation Working Rules

These rules govern autonomous implementation after a design has been approved  
for implementation.

They are intended to be reusable across different PRs, design documents, and  
projects. Fill in the project-specific block below before implementation starts.

## Project-specific implementation contract

```text
PROJECT / PR:  
  <e.g. Step 01a — Contract-derived Proposer Prompt Extraction>

PRIMARY DESIGN DOC:  
  <path to the authoritative PR-level detailed design / live ledger>

RELATED / BINDING DOCS:  
  <parent step design, overall roadmap, architecture contracts, policies>

IMPLEMENTATION BASE:  
  <branch / base commit / merged prerequisites>

APPROVED SCOPE:  
  <short statement of what this PR is allowed to change>

FROZEN INVARIANTS:  
  - <invariant 1>  
  - <invariant 2>  
  - <metric/scorer/schema/ownership constraints>  
  - <things explicitly forbidden>

APPROVED IMPLEMENTATION SEQUENCE:  
  <semantic milestones / commits / Gates / terminal validation>

AUTONOMOUS VALIDATION BUDGET:  
  - ordinary unit/integration/static/pseudo tests: unrestricted  
  - real LLM calls: allowed when bounded  
  - real inference / real training / Gate 1 / Gate 2:  
      allowed autonomously when bounded, non-destructive, and expected  
      total wall time <= <default: ~1 hour>  
  - GPU / CPU / API / monetary limits:  
      <project-specific limits if any>  
  - anything materially beyond these limits:  
      stop with a cost/runtime projection before launching

REQUIRED LIVE DOCUMENTATION:  
  <usually PRIMARY DESIGN DOC>

CONTEXT HANDOFF / MEMORY FILE:  
  <usually repository-root before_end_memory.md>

PR CONTEXT SCOPE:  
  This context belongs to THIS PR only.

  The implementation context begins when this PR is authorized and ends when  
  the PR reaches READY FOR OPERATOR REVIEW / merge closure.

  Do not carry unfinished implementation state from a previous PR into this  
  PR's context.

  Cross-PR knowledge must come from merged repository state and the  
  RELATED / BINDING DOCS, not from the previous PR's memory handoff.

PR CONTEXT INITIALIZATION:  
  Begin this PR in a fresh implementation session.

  Before editing:

  1. inspect git branch / HEAD / status / recent history;  
  2. read PRIMARY DESIGN DOC in full;  
  3. read RELATED / BINDING DOCS needed to understand prerequisites;  
  4. verify IMPLEMENTATION BASE and merged prerequisites;  
  5. inspect the source/tests relevant to the first implementation milestone;  
  6. initialize CONTEXT HANDOFF / MEMORY FILE for THIS PR.

  The newly initialized handoff must record at minimum:

    - PROJECT / PR;  
    - PRIMARY DESIGN DOC;  
    - RELATED / BINDING DOCS;  
    - implementation branch;  
    - implementation base;  
    - current HEAD;  
    - working-tree fingerprint;  
    - approved scope;  
    - frozen invariants;  
    - endpoint authority, with each line's source;  
    - implementation sequence;  
    - validation budget;  
    - current checkpoint;  
    - exact next actions;  
    - stop conditions.

  Any memory content describing a previous PR is stale and must be replaced,  
  not appended as the active state.

PR CONTEXT AUTHORITY ORDER:  
  When recovering or resuming this PR, use:

    1. current repository / git / process truth;  
    2. PRIMARY DESIGN DOC;  
    3. parent / binding design documents;  
    4. CONTEXT HANDOFF / MEMORY FILE;  
    5. emergency mechanical rescue snapshot, if one exists;  
    6. conversational memory.

  A lower item may never override a higher one.

  An explicit operator instruction in the CURRENT session outranks a recorded  
  restriction, including one carried in the handoff. Quote the instruction,  
  update ENDPOINT AUTHORITY with it as the source, and say plainly that it  
  replaces the earlier line. Do not silently drop either side.

  Merge authority is outside this: it is never inherited, never widened, and  
  always requires the operator's explicit authorization for this merge.

PR CONTEXT CONTINUITY:  
  During THIS PR:

  - PRIMARY DESIGN DOC is the semantic implementation authority;  
  - CONTEXT HANDOFF / MEMORY FILE is the compact/resume continuation state;  
  - update the design continuously during implementation;  
  - refresh the handoff at semantic milestones and whenever the current  
    checkpoint materially changes;  
  - the handoff should normally be no more than one semantic milestone stale.

  Compaction does NOT start a new PR context.  
  A compacted/resumed session remains the same PR and resumes from the same  
  PRIMARY DESIGN DOC.

PR CONTEXT COMPACTION:  
  Manual compact:  
    stale design/handoff/HEAD/fingerprint -> BLOCK until synchronized.

  Automatic compact:  
    must not deadlock the session;  
    if semantic handoff is stale, create only a deterministic mechanical  
    emergency snapshot, allow compact, and require repository/design recovery  
    immediately on resume.

  Never fabricate semantic summaries inside a hook.

PR CONTEXT RESUME:  
  On SessionStart(compact/resume):

  dynamically recover the ACTIVE PR from the handoff.

  Inject:  
    - PROJECT / PR;  
    - PRIMARY DESIGN DOC;  
    - current checkpoint;  
    - exact next actions;  
    - stop conditions;  
    - emergency recovery warning if the last auto-compact occurred with a  
      stale handoff.

  Never hardcode a historical project phase, PR name, or design path in the  
  resume hook.

PR CONTEXT CLOSEOUT:  
  At READY FOR OPERATOR REVIEW:

  - synchronize PRIMARY DESIGN DOC;  
  - record final executable HEAD;  
  - record final PR HEAD;  
  - record validation / CI state;  
  - mark the PR context CLOSED / AWAITING OPERATOR ACTION;  
  - do not begin the next PR in the same implementation context.

  After merge:  
    perform the required parent/roadmap/status synchronization.

  The NEXT PR starts from a fresh filled Implementation Working Rules contract  
  and a freshly initialized context.

ENDPOINT AUTHORITY:  
  Each endpoint is a separate decision. Record the decision and its SOURCE:  
  an explicit operator instruction, an applicable repository restriction, or  
  `unresolved`. Caution is not a source: an endpoint you narrowed yourself is  
  `unresolved`, not the operator's decision, and is settled before freeze.

  - implementation + local validation:  <default: authorized>  
      source: <...>  
  - semantic commits:                   <default: authorized; section 14 —  
                                         no approval before each commit>  
      source: <...>  
  - branch push:                        <default: authorized; section 21>  
      source: <...>  
  - PR creation / update:               <default: authorized; section 21>  
      source: <...>  
  - CI repair to review readiness:      <default: authorized; section 21>  
      source: <...>  
  - merge:                              explicit operator authorization only;  
                                         section 22. No source value changes  
                                         this line.

  A planning-only or explicitly local-only request restricts publication, and  
  that restriction has a source: the operator's instruction. The absence of any  
  instruction does not.

POST-MERGE SYNCHRONIZATION OWNER:  
  <default: the planning session owns the step and overall updates; THIS  
   implementation session owns the PR document, the merge identity, the  
   evidence, the deviations and the remaining issues>

  One conversation may own both. Record which, so neither session assumes the  
  other did it. Only the owner writes the parent documents; the other reports  
  to the owner. Do not overwrite parent content that has moved on — reconcile.

  Owning this is not merge authority and does not confer it.

NORMAL STOP CONDITION:  
  <e.g. PR 01a READY FOR OPERATOR REVIEW — DO NOT MERGE>

STOP CONDITION:  
  PR ready for operator review:  
    implementation complete  
    + required validation complete  
    + PR opened/updated  
    + terminal CI green on exact final HEAD

MERGE AUTHORITY:  
  NEVER merge without explicit operator approval.  
```

If a field is not applicable, mark it `N/A` rather than inventing a value.

The project-specific contract overrides generic defaults below where it is  
more restrictive **and the restriction records a source**: an explicit operator  
instruction or an applicable repository restriction.

A restriction with no source is not a project decision and does not override  
anything. Resolve it with the operator before freeze instead of acting on it.  
Without this, choosing caution and writing the result into the contract presents  
an agent's decision as the operator's — and every later session that reads the  
contract correctly then inherits it as a frozen operator boundary.

This does not work in reverse. An unsourced line is resolved, not widened past  
the shipped defaults, and merge authority is never widened at all.

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
