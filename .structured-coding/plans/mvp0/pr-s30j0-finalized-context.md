# PR S30-j0 — Final context from subject resolution

## 1. Authority and freeze status

**DESIGN FROZEN — 2026-11-22.**

Coordinator freeze under the operator-approved option A scope and independent detailed
plan PASS: `/tmp/mineworld-s30j0-plan-red-team-review.md`. Reviewed pre-freeze document
SHA-256: `8fee3abdbee0e0acd604d55b1c87994bfcdbb9412a5d39ab776d76ced61c0657`. This records delegated implementation detail finalization,
not a new operator statement. Sections 2–8 are frozen; evidence remains live.
Local integration base is the complete reviewed S31-b2 commit
`f10b1ee8577a3e05f9196cc1387fc297f6f589c6`, not merged main.
Fresh isolated execution is authorized on this local stack.

Binding review conditions: prove drift refusal after real reopen and unchanged saved
state; later-owner refusal publishes no partial pack; label test-adapter limits;
assert nonzero matches for targeted test filters.

On 2026-11-22 the coordinator relayed the operator's exact reply, **“同意你的推荐”**, to the
Inbox proposal recommending S30-j §16 option A: a separate authoring/worldpack foundation
PR retaining the single composed event, acknowledging migration of external public Rust
trait implementers. This approves the scope direction. It does not assert approval of this
subsequently filled detailed plan. No repeated approval of that direction is requested.
The primary records the final plan audit and applicable design-freeze decision before a
fresh foundation owner implements. This document is the sole foundation design authority;
S30-j's ledger remains authority for runtime sampling. Both earlier awaited-event and
OnceLock alternatives are rejected; they must not enter this foundation.

Source audit base: reviewed **local `f10b1ee8` (S31-b2), not merged**. Planning checkout is
`/tmp/mineworld-recover-s30j`, branch `mvp0/recover-s30j`, HEAD `2b4d36fa`. Its recovered
S30-j source changes are NOT the foundation starting patch. New owner starts a clean,
separate worktree after primary integrates S31-b2, verifies its diff against `f10b1ee8`,
and records actual base/head here. No cherry-pick of all recovery changes.

Freeze-ready sections are §§2–8: scope, API, lifecycle, invariants, commit boundaries,
acceptance and execution contract. §9 is live evidence. A freeze records date, authority,
exact reviewed revision and any audit amendments here; drafting completion is not freeze.

## 2. Problem, scope and integration order

`OwnsData::context` runs before subject sections decode. `ResolvesSubjects` sees the
owner's authored subject values but can return only seeds; the world-level context event
therefore cannot include immutable data derived from those values. S30-j needs the complete
authored/locked Fixed map inside its existing `attributes-composed.newcomers` before any
admission. The foundation makes the post-resolution context explicit, without implementing
newcomers or changing persisted events.

Order: S31-b2 integration → foundation audited/frozen → fresh foundation implementation,
local evidence and independent review → authorized integration → fresh/resumed S30-j on
that base → S30-k1. S31-b2's `YieldedFile` authoring export and direct-WorldGenesis filter
in `worldpack/src/configure.rs` must survive unchanged. Notify primary before taking
`authoring/src/lib.rs`; its current owner is S31-b2. Primary owns shared root plans/dashboard.

Owned production paths: `authoring/src/data.rs`, `authoring/src/lib.rs`,
`worldpack/src/resolution.rs`, `worldpack/src/read.rs`, and signature-only migration in
`systems/attributes/src/section.rs`. Documentation: `docs/MODULE_SPEC.md` §3.1 and dated
amendment to `docs/decisions/ARC-121.md`. Existing SDK dispatch signatures remain; update
its documentation only if it describes the old typed contract. Test paths are §6.

Excluded: kernel/contracts, persistence implementation, capability registry expansion,
entity creation, attributes sampling semantics, runtime arrivals, new persisted events,
reference-world files/baselines/goldens, dependency changes, global mutable initialization,
new subject-genesis drift API, S31-b2 comparison implementation. No public writes.
Reuse: existing `Arc`, typed owner erasure and loader lifecycle already solve storage and
ownership. This is an extension of that seam, not commodity infrastructure requiring a
new library. No dependency or general plugin redesign is justified.

## 3. Exact proposed public contract

Add and re-export from `mineworld_authoring`:

```rust
#[derive(Debug)]
pub struct SubjectResolution<C, S> {
    pub context: Arc<C>,
    pub seeds: BTreeMap<EntityKey, S>,
}
```

Replace the typed method (associated types and `seed_resolved` unchanged):

```rust
fn resolve_subjects(
    context: Arc<Self::Context>,
    configuration: Option<&dyn Any>,
    subjects: &[Subject<'_, Self::Authored>],
) -> Result<SubjectResolution<Self::Context, Self::Seed>, ResolveRefusal>;
```

Add required method to public erased `ResolvedSeeds`:

```rust
fn context(&self) -> Arc<dyn OwnerContext>;
```

`ResolvedSeeds::as_any` still exposes `BTreeMap<EntityKey, Seed>`, not the new result.
Its owner/section/seed methods retain meaning. `SystemPack::resolve_subjects`, SDK
`resolves!` dispatch, installed `Capability::resolve_subjects`, and `resolve_erased`
retain their erased result type `Arc<dyn ResolvedSeeds>` (and existing Option where
applicable). Do not describe this as full erased-trait source compatibility: external
implementations of `ResolvedSeeds` must supply the required context method. External
`ResolvesSubjects` implementations must accept Arc and return the named result. There is
no compatibility shim or new `Clone` bound on Context. Unchanged owners return the input
Arc with their existing seeds. Owners enriching context construct a new immutable Context;
they may not change declarations that already validated authored sections.

Five typed implementers at the audited base, all migrated in the foundation:

| Owner | Source | Change |
| --- | --- | --- |
| AttributesSystem | systems/attributes/src/section.rs | Return original Arc and existing seeds; no domain change |
| Probe | authoring/src/data/tests.rs | Migrate; exercise derived context and refusal |
| Owner | sdk/rust/tests/data.rs | Migrate; prove macro dispatch and downcasts |
| Resolver | worldpack/src/load.rs | Migrate; exercise final context seeding and omitted seeds |
| SchemaPack | tests/acceptance/tests/schema/mod.rs | Migrate; add derived-context persistence fixture |

One existing erased implementer: `HeldSeeds<S>` in authoring/src/data.rs. Direct erasure
callers: two authoring tests, three load tests, SDK macro dispatch in sdk/rust/src/pack.rs.
The production loader invokes the installed capability in worldpack/src/resolution.rs.
Audit counts again on integration; additional implementers are bounded migration work,
not permission to omit tests or change semantics.

## 4. Lifecycle and atomicity

1. Collect/decode owned data and configuration, build initial owner contexts, decode and
   validate sections against those contexts. Initial declarations remain authoritative.
2. Retain existing section checks. Resolve each enabled resolving owner once in systems
   order from initial contexts. Input is exclusively its configuration and subjects of
   its carried kinds: key, kind, tags, own typed authored section or None. No name, note,
   location, another owner's sections, world handle or final context of another owner.
3. `resolve_erased` validates owner/type of context, configuration and sections as today,
   calls the typed method with its Arc, then rejects returned seed keys absent from the
   input. BTreeMap guarantees one seed per key; an omitted seed is legal. The loader's
   file/key uniqueness remains the source of input uniqueness; do not add implicit merging.
4. Hold final typed context with the seeds; wrap that SAME Arc in `HeldContext<S>` for the
   erased accessor. Seeding and `WorldPack::context` must observe the same immutable data.
5. Collect all resolutions before installation. Validate each result's owner, section and
   returned context owner against the capability. Report mismatch as `ResolveRefused`
   naming the expected system, without leaking subject values. Collect and validate all
   final contexts in temporary state before mutating the pack. Install the contexts and
   resolutions together in a private, fallible operation. No partially finalized WorldPack
   escapes on any refusal; nonresolving contexts remain unchanged.
6. In `WorldPack::read_with`, installation occurs after `check_sections` and **before
   `configure::check_references`**. Current configure reference checking reads configured
   references, not derived context; preserve the order without inventing a new reference
   API. Resolution failure does not run subsequent validation/seeding. Test constructors
   must use the same installation helper when claiming to model loader finalization.
7. Assembly emits configuration, then final context facts in existing owner order, then
   resolved subject seeds in existing file/section order. A resolving owner NEVER falls
   back to `HeldIn`'s initial-context section seed, including omitted-seed subjects.
   Nonresolving section behavior remains unchanged. Repeated assembly does not re-resolve.
8. Tools reading `WorldPack::context`, notably tools/cli/src/attributes_resolve.rs, receive
   finalized context. Existing AttributesSystem returns identical context; output bytes
   and samples stay unchanged in this PR.

Exact ARC-121 amendment text to append, with actual freeze provenance at implementation:

> An owner resolving subjects may return an immutable final context alongside its subject
> seeds. Resolution receives the initial context and only its own typed subject data. The
> loader installs all successful final contexts and seed holders before configuration
> reference checking and before any seeding. World-level context facts are emitted from the
> final contexts after configuration and before sections; they remain about no subject.
> Resolved subject seeds use the same final context and replace section seeding even when
> a subject has no seed. An owner that does not enrich its context returns its initial Arc.
> Initial declarations used for decoding may not be reinterpreted. Resolution does not
> expose other owners' contexts, subject names, notes or locations. Typed and erased Rust
> implementers migrate to SubjectResolution and ResolvedSeeds::context respectively.

Synchronize MODULE_SPEC §3.1 and trait docs to this lifecycle; fix old wording claiming
“every reference checked” before resolution, distinguishing section checks from subsequent
configuration checks. CONTEXT_FACTS retains world-level/no-subject semantics. S31-b2's
comparison remains restricted to direct WorldGenesis facts, excluding later runtime facts.

## 5. Frozen acceptance candidates

| ID | Observable obligation |
| --- | --- |
| F1 | All five typed implementations compile; macro dispatch, erased downcast and non-Clone Context work |
| F2 | Derived context reaches context emission, subject seeding and tool-facing context access identically |
| F3 | Actual event sequence is configuration → final world context → subject seeds; repeated assembly is equal |
| F4 | No old section fallback for omitted seed; zero subjects and sectionless subjects resolve deterministically |
| F5 | Wrong owner/type/configuration/section, stranger seed and final-context owner mismatch refuse explicitly |
| F6 | Two independent owners cannot see each other's subject sections/final context; failure publishes neither result |
| F7 | Real SQLite create, advance, close, reopen/replay and verify preserves derived context; changed authored-derived context refuses resume as ConfigurationDrift naming owner, before append |
| F8 | Existing attributes CLI resolve golden and salt/authored invariants remain unchanged; real CLI save/resume/verify passes |
| F9 | Same fixed-input short reference-world event bytes on base and head; tracked world/golden/baseline files unchanged |
| F10 | Independent review finds no hidden mutation, domain leakage, unreasonable function/file growth or compatibility omission |

F7 includes altered derived context after a persisted save and a digest of saved facts/journal
before/after refusal, not merely a manually assembled comparison call. Foundation tests
use a generic schema-derived value; authored+locked newcomer drift and complete Fixed map
before admission remain mandatory S30-j acceptance after foundation. This distinction must
not be used to claim runtime newcomers are implemented here. Foundation adds no new event
in production; a test-only schema context fact uses that test pack's existing own codec
pattern and SystemInternal visibility. No private values in diagnostic output.

## 6. Semantic commit checkpoints

### C1 — Explicit result and complete public migration

**Goal:** define one immutable result and migrate all users together so the commit compiles.
**Scope:** authoring data/export/tests, five implementers above, SDK test/documentation,
MODULE_SPEC/ARC-121. No world behavior change or newcomers; no dependency on later commits.
**Implementation:**
- [x] Add documented result and required erased accessor; retain original map downcast.
- [x] Update erasure to retain final Arc and validate existing ownership/stranger rules.
- [x] Migrate all five implementations; production AttributesSystem returns original Arc.
- [x] Record typed AND erased external migration and exact contract amendment.
**Validation:**
- [ ] Deterministic authoring and SDK seam tests: non-Clone context, derived context visible
  to seed, wrong inputs refuse, omitted/zero seeds, unchanged map downcast.
- [x] Focused all-target compilation of affected packages, including acceptance and CLI.
- [ ] Independent LLM logic review recorded separately from deterministic checks.
**Acceptance:** F1 and erasure portions of F2/F4/F5; production payloads unchanged.
**Failures/edges:** foreign context/config/section and stranger seeds return ResolveRefusal;
owner refusal propagates; no fallback or synthetic empty context; no panics on bad erasure.
**Commands/evidence:** §7 commands A/B; record exact head, counts, duration, failures.
**Commit boundary:** one compile-coherent public migration; inspect staged paths/diff and
record deviations before local semantic commit. No recovered S30-j model work included.

### C2 — Install finalized contexts before validation and assembly

**Goal:** connect the result to actual loader/context consumers atomically.
**Scope:** worldpack resolution/read, focused load tests (new `worldpack/src/load/final_context_tests.rs`
may hold cohesive tests instead of growing the already large load.rs), test fixture migration.
Depends on C1; no configure comparison rewrite, capability additions or runtime semantics.
**Implementation:**
- [x] Collect/validate all final context owners and resolutions before publishing either.
- [x] Use one private installation operation from read_with and applicable test builders.
- [x] Preserve unchanged contexts and ordering; ensure initial HeldIn cannot seed resolving owners.
- [x] Add focused final-context tests through production installation/assembly functions.
**Validation:**
- [x] Deterministic F2–F6: changed derived context, two owners, owner mismatch, later refusal,
  zero subjects, absent authored section, omitted seed, repeated assembly and actual event order.
- [x] Existing loader data/resolution/configuration regression filters and real CLI golden.
- [x] Independent logic review verifies install precedes check_references and no partial pack escapes.
**Acceptance:** F2–F6 observable, including tool-facing WorldPack::context and same Arc data.
**Failures/edges:** any resolution/owner mismatch aborts load; earlier successful result is
not installed on failure; nonresolver unchanged; missing seed intentionally emits nothing.
**Commands/evidence:** §7 B/C; test fixtures may construct the closed capability-bound
WorldPack with existing in-crate builders but must call the production installation helper.
Label this accurately: not a public custom-pack loader test.
**Commit boundary:** one loader lifecycle change with integration regressions, no unrelated
load.rs cleanup. Stage inspection and evidence update precede local commit.

### C3 — Persistence, CLI parity and final review

**Goal:** demonstrate restart and no-newcomers compatibility through actual storage/processes.
**Scope:** tests/acceptance/tests/schema/mod.rs and data_seam.rs; a cohesive new
`tests/acceptance/tests/data_seam/persistence.rs` if needed; focused CLI test addition in
`tools/cli/tests/attributes_resolve.rs` or adjacent new `attributes_context_restart.rs`.
Depends on C1/C2. Reuse existing scratch guards, persistence APIs, CLI process helpers.
**Implementation:**
- [x] Extend SchemaPack test context with immutable subject-derived summary and internal
  world-level fact; use its codec/reducer to verify replayed state.
- [x] Add real PersistentWorld create/advance/drop/open/replay/verify flow following the
  configuration_seam.rs SQLite pattern; saved genesis comparison uses reopened disk facts.
- [x] Reject changed derived source before resume append; compare stored rows unchanged.
- [x] Run real CLI on existing sampled world: resolve golden, create/continue/verify saved
  world using supported binary commands; inspect facts and resume report, not exit alone.
- [x] Compare baseline/head reference event bytes; archive commands and digests.
**Validation:**
- [x] F7–F9 deterministic evidence with explicit separation: SchemaPack exercises SDK +
  kernel + persistence seam, because closed Capability prevents public loader installation;
  C2 proves generic loader wiring, real attributes CLI proves installed-loader compatibility.
- [ ] Targeted fmt/check/clippy, documentation and scratch checks, plus independent F10 review.
**Acceptance:** actual persisted state/revision and replay match; changed source produces
named drift and no save mutation; original CLI golden and baseline bytes match exactly.
**Failures/edges:** no manual compare-only stand-in; no silent skipped restart; no new
production test-pack registration; no golden regeneration to conceal drift. Platform/process
limitation is INCONCLUSIVE and blocks the corresponding acceptance claim.
**Commands/evidence:** §7 C/D. Record measured counts/time, output artifacts and exact head.
**Commit boundary:** integration evidence and necessary fixture changes only. Technical-debt
review disposition and exact staged diff recorded before commit and local readiness.

## 7. Validation commands and bounded budget

All cargo test/build/check/clippy commands run from the new owner worktree through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`; inspect its usage before
invoking. Two global slots, at most ONE heavy job owned by this PR. No full workspace
suite, no nightly/client runs, no paid API or model Gate. No GitHub waiting. Formatting
and documentation checks do not need a heavy slot. Use a separate target directory.

Commands below are planned, NOT run foundation evidence:

```sh
# A: smallest seam checks
cargo test -p mineworld-authoring --lib data::tests
cargo test -p mineworld-sdk --test data
# B: migration and affected-target checks
cargo check -p mineworld-authoring -p mineworld-worldpack -p mineworld-attributes -p mineworld-sdk -p mineworld-acceptance -p mineworld-cli --all-targets
cargo test -p mineworld-worldpack --lib resolution
cargo test -p mineworld-worldpack --lib context
# C: real seam/CLI tests
cargo test -p mineworld-acceptance --test data_seam
cargo test -p mineworld-cli --test attributes_resolve
# D: finish
cargo clippy -p mineworld-authoring -p mineworld-worldpack -p mineworld-attributes -p mineworld-sdk -p mineworld-acceptance -p mineworld-cli --all-targets -- -D warnings
cargo fmt --all --check
python3 scripts/check_doc_headings.py
python3 scripts/check_decision_ids.py
python3 scripts/check_scratch.py scan
python3 scripts/check_scratch.py left --target-dir target
```

Confirm package names against Cargo.toml at dispatch and record bounded corrections.
Use explicit new-test names/filter lists for C2 and new CLI restart test so zero selected
tests never count as a pass. Baseline: one seed-7, one-day headless social-cafe run on clean
base and final head, same binary arguments/world inputs; compare canonical fact bytes and
stdout determinism separately from SQLite container bytes. Audit tools/cli tests/headless
helpers and binary help before choosing exact baseline/restart invocations; record exact
commands before executing. No 30-day kill matrix is required for this seam change.

Initial envelope: one baseline capture, one initial pass of each listed target/filter,
one head comparison, one final scoped static pass; up to two focused repair reruns per
failed target. Soft resource stop at 60 minutes aggregate active heavy-job time (slot wait
excluded): record measured work and propose a narrower continuation if exceeded, never
claim pass from budget exhaustion. Additional runs require a new failure/change/question;
no repeat merely for confidence. This is a proposed detailed-plan budget pending audit,
not retroactive authorization for unbounded runs. Shared CI requirements remain separately
NOT RUN until primary later authorizes publication; local scope reduction is the explicit
operator-directed local-first deviation from generic workspace-wide standard commands.

## 8. Filled execution contract for the fresh owner

| Field | Contract |
| --- | --- |
| Objective | Deliver generic immutable final-context resolution and migrate existing consumers, with no production event/behavior change |
| Phase gate | Direction approved; wait for primary's recorded detailed-plan audit/freeze before implementation |
| Owner/worktree | Fresh dedicated foundation agent/branch/worktree; primary assigns exact identifiers at dispatch; never reuse old WIP or root |
| Base | S31-b2 reviewed local f10b1ee8 content, integrated base to be recorded by primary; abort mismatching material seam changes for reconciliation |
| Authorized endpoint | Local implementation, bounded validation, semantic commits, independent review, READY FOR OPERATOR REVIEW |
| Prohibited endpoint | Public push/PR, merge, tags, paid API, additional agents, workspace-wide suites |
| Paths | §2 production/docs and §6 tests; shared authoring export requires ownership coordination before edit |
| Acceptance | F1–F10; no silent deferral; specific not-run checks remain INCONCLUSIVE |
| Budget | §7; one owned heavy job through shared two-slot limiter |
| Review | Separate per-commit deterministic evidence and LLM review; independent red team before readiness |
| Escalation | New event, broader context visibility, registry extension, persistence/kernel change, changed production bytes or added dependency is material; prepare concrete amendment |
| Autonomy | Fix bounded implementation/test failures and migrate callers within frozen contract without repeat permission |
| Publication | Primary obtains exact public disclosure authorization separately; approval of direction is not publication approval |
| Handoff | Exact base/head/branch, owned paths, commands/counts/time, artifacts, failures, debt disposition, process status and integration instructions |

Technical-debt audit explicitly examines: owner-local typed API, errors on erased misuse,
immutability/repeated assembly, installation atomicity, restart source of truth, no hidden
fallback, external compatibility disclosures and no cross-owner dependencies. data.rs and
load.rs already exceed normal review thresholds; quantify added function/file size. Keep
resolution installation cohesive in resolution.rs and move only new coherent tests where
needed. Do not grow read_with into an orchestration monolith or split solely for line count.
Both 2D and 3D remain consumers of identical server-authoritative facts; no spatial or
presentation rule changes. No new client execution is justified by this foundation alone.

## 9. Evidence and checkpoint

2026-11-22: design source-audited against local f10b1ee8; all five typed implementations,
existing erased holder, caller chain and S31-b2 export/comparison diff identified. Existing
`data_seam.rs` documents its closed-capability test limitation; this plan preserves it
explicitly. `configure::check_references` currently checks configured entity references;
no claim that it already reads derived context. Detailed plan awaits independent audit.
Foundation implementation/checks/restart/parity: **NOT RUN**. No API/new-event code added.
Recovered S30-j sampler tests are earlier evidence only, not foundation acceptance.

Pending independent audit: correctness of the installation helper boundary, adequacy of
split loader/SDK-persistence/CLI evidence, final package/test command filters, budget and
public migration text. Primary may finalize bounded details without asking the operator
again for already-approved scope; genuinely material differences require a concrete packet.

Planning validation: doc heading checker PASS (254 numbered sections/337 documents);
`git diff --check` PASS; foundation package names verified from Cargo manifests. New
design local link targets and three eight-part commit checkpoints checked. These checks
validate document structure only, not the proposed implementation.

## 10. Execution ledger — S30-j0 owner

Execution initialized 2026-11-22 in fresh shared clone `/tmp/mineworld-s30j0-finalized-context`,
branch `recover/s30j0-finalized-context`, base/current HEAD
`f10b1ee8577a3e05f9196cc1387fc297f6f589c6` (local reviewed prerequisite, not merged).
The requested worktree creation failed because shared root Git metadata is read-only; the
dispatch explicitly permits the shared-clone fallback. Initial source diff empty. Only this
frozen plan copied from recovery; no S30-j source WIP copied. Primary notified before export
ownership. This section also holds continuation state; shared parent handoff is not edited.
Endpoint authority: dispatch and §8 authorize local implementation, validation, semantic commits;
prohibit public writes, merge, paid APIs, subagents and shared root documentation/dashboard edits.
Sequence C1 → C2 → C3; budget §7, one heavy job through shared slots, 60 minutes active soft limit.
Binding contracts: CLAUDE.md, MODULE_SPEC §3.1, ARC-121; invariants §§2–5 above.
Current checkpoint: C3 implementation and affected validation complete; final independent review PENDING. Historical per-checkpoint entries
below retain their original status; the final receipt supersedes them. Material scope changes stop for review.
Gate 1 NOT REQUIRED: no model-facing semantics. Gate 2 owns actual SQLite and CLI lifecycle.

C1 deterministic evidence (precommit, executable content staged with checkpoint):
- `cargo test --offline --locked -p mineworld-authoring --lib data::tests`: 7 PASS,
  5.53s initial build; strengthened input misuse cases rerun 0.83s, 7 PASS.
- `cargo test --offline --locked -p mineworld-sdk --test data`: 2 PASS, 28.13s.
- Scoped six-package `cargo check --offline --locked ... --all-targets`: PASS, 15.73s.
  Found unused SchemaPack EntityKey import; removed. Test-local Quiet impl moved to
  module scope to remove non_local_definitions warning; final static pass will verify.
- Baseline CLI compiled from unchanged f10b1ee8 in 91s, target
  `/tmp/mineworld-s30j0-target`; run from owned checkout against identical world files:
  `mineworld run worlds/social-cafe --headless --seed 7 --days 1 --save
  /tmp/mineworld-s30j0-baseline-save`. PASS: revision 2834, 4351 facts, zero faults.
  Save preserved for canonical fact-byte comparison; provenance
  `/tmp/mineworld-s30j0-evidence/baseline-provenance.txt`. Initial read-only SQLite
  inspection could not open under sandbox; immutable read succeeded (no data changed).
C1 self logic review: API retains typed Arc through HeldSeeds and wraps that same Arc
for access; seed-map downcast preserved. All five implementers migrated. Production
AttributesSystem returns original Arc and exact existing seed construction. Erasure
now also checks section name; foreign owner/type/config/section and stranger seed refuse.
Configuration payload type remains owning implementation's responsibility, explicitly tested.
No new dependencies, event types, kernel or registry changes. Public migration documents
cover typed and erased source breaks. Independent review PENDING coordinator assignment.
C1 scope inspection: nine source/doc files plus owned plan only; YieldedFile export and
WorldGenesis filter preserved. C1 is not final acceptance; loader installation awaits C2.

C2 implementation discovery: the installed Capability is closed, and old load.rs probes
intentionally rank `r-probe` as unrelated capabilities. Those legacy `with_resolved` tests
remain low-level ordering tests, not finalization evidence. New final_context_tests.rs
uses probes with matching capability owner/section identities and the production private
`finalize_with` dispatch boundary. It receives only each owner's context, configuration
and collected subjects; production `finalize` selects the actual resolving capabilities.
This avoids test-pack registry expansion while exercising later resolver failure before
publication. `final_contexts` validates the complete temporary result; `install_resolutions`
publishes contexts and seeds only after it succeeds. The read path calls finalization after
check_sections and before configure::check_references. Resolving dispatch returning None
now refuses explicitly rather than permitting initial section fallback. No context values
are printed by the new refusal. New tests claim this adapter boundary, not public loading.

C2 validation: initial final_context compile FAIL (Contexts imported from data's private import);
corrected to owning content module. Focused retry PASS: 3 final_context tests, 7.86s build.
Planned regression filters: resolution 6 PASS (0.12s), context 7 PASS (0.08s), all nonzero.
C2 self logic review: no mutation until complete collection and validation succeed; the new
installation method is six lines and preserves nonresolver contexts. final_contexts is 29
lines; resolution.rs grows from 125 to 181 lines. New coherent tests 369 lines; no load.rs
production growth. Tool accessor, context facts and seed emissions share typed Arc identity;
actual assembled sequence includes configuration, both contexts and both owners' seeds;
omitted authored section panics if wrongly used, and repeated assembly is equal.
Late owner refusal, missing result and three owner/section/context mismatch cases preserve
all original context Arcs and empty original resolutions. No newly introduced domain
knowledge, cross-owner context access, mutable cache or registry expansion.
Independent C2 review pending. Real CLI golden is consolidated with C3's planned CLI target
to avoid duplicating that process evidence. C3 tests in progress, unstaged at C2 checkpoint.

Independent checkpoint review: C1 PASS at aa17bace, report
`/tmp/mineworld-s30j0-c1-red-team-review.md`. C2 source logic PASS at 39349ace but
review found overlapping invalid-result cases: the wrong-owner and wrong-section cases
also returned a wrong context. Corrected with a separate wrong_context flag; owner and
section cases now retain inner.context. Focused guard-removal proof requested by primary,
limited to the corrected exact test and restoring source in finally; results pending.
SDK source doc audit also found old 'every reference checked' wording in pack.rs; corrected
without changing erased dispatch signatures.

C3 first SQLite acceptance run: 5 PASS, 25.93s compile, 0.01s tests. Real create/advance/drop,
reopen with one journal revision replayed, advance again and verify: 3 revisions and 2 facts.
Changed authored source refuses named ConfigurationDrift against disk genesis before resume;
11 saved rows remain identical, digest ae5de701e19f6019. The adapter invokes production compare,
not installed custom-pack loading. Advanced must-use warnings fixed by asserting instants == 1.

CLI first attempt: 5 FAIL at pre-load package identity, INCONCLUSIVE for feature claims.
Source audit packages/src/declared.rs: bundled = compiled_under(manifest_dir, PACKAGES_DIR),
with the latter compiled into packages. Reusing the baseline target across checkout roots
retained the baseline root in that dependency while head packs had the new root. This is
validation environment contamination, not a reason to modify package rules/world fixtures.
Corrective run uses fresh head-only `/tmp/mineworld-s30j0-head-target`; baseline target/save
remain preserved. All final CLI/static checks will use head-only target. Earlier pure seam
checks exercised their claims and do not depend on bundled package classification.


C3 final receipt (source staged for the C3 checkpoint; final independent review pending):
- Fresh head-only CLI `attributes_resolve`: 5 PASS, 99s build, 1.08s tests. Includes
  unchanged golden and actual create/reopen/continue/replay verification.
- Final six-package all-target `cargo clippy --offline --locked ... -- -D warnings`:
  PASS, 19.90s. Covers all five migrated implementers, SDK source, and last warning fixes.
- Final acceptance `data_seam -- --nocapture`: 5 PASS, 64s build, 0.01s tests;
  11 unchanged rows, digest ae5de701e19f6019, replay verifies 3 revisions/2 facts.
- Corrected C2 exact test: 1 PASS (5.11s). Owner-guard removal: expected RED (1.33s);
  section-guard removal: expected RED (1.28s). Both ran exactly one test and failed at
  the expected refusal assertion. Production source restored in finally and git diff
  confirms no change; SHA256 44fbc7b7b5c11308abfb5ef00884c678990a91341dbe702b8f198844effa6933.
  Script and three logs: `/tmp/mineworld-s30j0-evidence/check-guards.py`,
  `corrected.log`, `owner-guard-removed.log`, `section-guard-removed.log`.
- Head social-cafe run matches preserved base exactly: 4351 facts, 4101209 bytes,
  SHA256 09af4594486407f3d6b5f3191a44488e62304dc7278051ff7a1bfc9f6fc597a7.
  Deterministic stdout also equal (only existing header/wall exclusions). Canonical
  byte framing, saved artifacts and stdout provenance are in
  `/tmp/mineworld-s30j0-evidence/{compare-head.py,parity-result.txt,baseline-provenance.txt}`;
  baseline/head facts and stdout retained there. No world/golden/baseline edits.
- Final fmt and diff whitespace PASS; headings PASS (254 sections/337 docs), decision
  IDs PASS (257), scratch static PASS (387 sources, 2 established exemptions).
  Scratch-left reports one unowned system-temp entry `mineworld-full-link-honaky1m`
  (863.2 MiB), not created by this PR; left untouched. Owned head target/tmp is empty.
  Global scratch hygiene is not claimed PASS. Baseline/head saves are intentional
  retained review evidence outside test scratch. No broad workspace or remote CI run.

C3 self review: test fixture SchemaPack is 409 lines, coherent persistence adapter 197,
loader fixture 375. Production runtime changes remain confined to C1/C2; C3 adds tests,
corrects independently identified test overlap and public SDK lifecycle wording.
The schema context is non-Clone, derives from resolved values, emits a subjectless
SystemInternal fact and replays actual process state. Its adapter boundary is documented;
it does not claim installation into the closed catalog. Existing attributes supplies real
CLI compatibility evidence. No new registry/kernel/domain facts or runtime newcomers.
No material debt found: no catch-all state bag, mutation cache, duplicate assembly pipeline
or production test hooks. Review packet retains source-level API break documentation,
YieldedFile export and unchanged WorldGenesis filter. Final independent F10 remains pending.

Final loader filters: data::tests 9 PASS (3.73s); configure::tests 16 PASS (0.11s),
including YieldedFile collection/refusals and reaction-during-genesis filtering. Logs:
`/tmp/mineworld-s30j0-evidence/data-regression.log` and `configure-regression.log`.
All heavy work serialized through shared slots; total active time below 10 minutes,
within the 60-minute budget. No broader rerun after these passing checks.
