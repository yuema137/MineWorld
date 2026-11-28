# Canonical nutrition engine independent review — 2026-11-23

Reviewer: independent_review, replacement independent red-team lane. Read-only source review; root main b457a94d9a1ace6a39005de683cc393886e19288 remains dirty and untouched. No code/build/test/Godot/inference/public write, child worker, or heavy slot. Prior completed review checkpoints recovered from `/tmp/mineworld-review-sol-20261123.md` and `/tmp/mineworld-nutrition-independent-review-20261123.md`; their completed architectural work was retained rather than repeated.

## Verdict

**Architectural proposal PASS; canonical execution design requires three bounded drafting corrections before final READY FOR OPERATOR DESIGN DECISION.** These are source-grounded engineering corrections, not additional user preferences or a redesign. No implementation freeze/grant exists. No code-readiness or validation PASS is asserted.

Reviewed complete canonical primary design (176 lines), proposed execution contract (938 lines including its unchanged working-rule body), revision2 superseding semantics and revision3 correction, source manifest and caller/consumer map. Read CLAUDE in full and Structured Coding PR-design routing/resources. Independently recomputed all 47 manifest source hashes: all match current root. Independently compared preserved contract §1 onward against original implementation-working-rules.md: byte-identical.

Reviewed draft SHA256:
- primary: `5961c55dfb39953bcb8345aaed1a66feabb79b3a00bd1e175c5010684cf7ba40`
- contract: `bf5a83011e6de1dfbcae46121d39d4e432f0ea1abaae21ed333dbf409c1826ff`
- amendment: `0d4696375dfafe1441723d2dc1470a0aca39320c3a13a73a7afb8145294bc748`

These identify this review snapshot; subsequent bounded corrections require a narrow disposition check, not a new whole-design audit.

## Required bounded corrections

### R1 — Preserve member pytest configuration/network guards

Primary C4 verification combines `clients/agent/tests/test_needs_view.py` with `cognition/lm-controller/tests/test_needs_prompt.py` and `test_needs_triggers.py` in one pytest command. `.structured-coding/standards.md` explicitly forbids this: their common ancestor selects repository-root configuration, losing member addopts/network guard. Both member pyprojects and agent conftest confirm the guard lives in pytest configuration.

Replace with separate invocations from repository root:

```
uv run --locked pytest clients/agent/tests/test_needs_view.py
uv run --locked pytest cognition/lm-controller/tests/test_needs_prompt.py cognition/lm-controller/tests/test_needs_triggers.py
```

Any additional no-needs/controller test stays in the appropriate member invocation. Do not run tests in this planning lane.

### R2 — Repair canonical evidence pointer

Primary §4 says `nutrition-source-search-map-20261123.md`; actual preserved raw map is `nutrition-engine-source-searches-20261123.md` in the same directory. Use the actual filename consistently before incorporation. The evidence exists; this is a broken reference, not missing source work.

### R3 — Make C2/C3 integration test ordering executable

C2 promises a failing synthetic owner through the actual CLI before C3 adds the needs InvalidGenesisContent producer. `systems/installed/src/lib.rs` documents and implements a compiled installed set; test-only stub owners do not become available to the production binary. Avoid inserting production test-registry plumbing or claiming a library stub is actual CLI proof.

Use C2 stub-owner persistence factory tests for zero/one invocation, real successful CLI creation/replay plus ordinary assemble-failure regressions, and keep the decisive real CLI invalid model/capacity diagnostic/no-save/no-listener tests in C3, where their production owner exists (already listed there). Make C2 acceptance and pending/deferred status explicit until that integration evidence arrives. Alternatively an audited existing production owner that genuinely emits the new generic error would need to be identified; none is currently mapped. This changes evidence placement, not final engine acceptance.

## Source/consumer assessment

The proposed lazy path fits current PersistentWorld::create: snapshot, single genesis, revision/snapshot encoding and manifest encoding can finish before factory invocation. Existing create can delegate while honestly retaining the preallocated-backend limitation. Only production callers in tools/cli/src are serve.rs and run.rs; both are included. Existing resume/config-drift order is preserved, with logical committed-history invariance rather than unsupported physical file equality. No source facts suggest a need for a second genesis or deletion rollback.

Generic owner schema constants/default-noop validation hook and exhaustive ConfigurationRefusal mapping fit the existing InteractionSection/resolve/HeldSection seams. Model-presence scan precedes erased defaults; errors preserve actual file/EntryAt without fabricated lines. Needs2 table includes its configured event and IL component; unrelated owners inherit1. Generic InvalidGenesisContent carries owner/entity key/code/field, adds no domain rule, and terminates candidate construction. Replay/runtime remain fail closed and never become action rejections or migration.

The four reader families are accounted for: rule/motivated consume owner-normalized policy values; shared Godot decoder uses schema envelope and owner identity/bar; agent uses typed energy/serving records; LM uses bounded owner facts with unchanged word-based trigger. The new replaces_need identity avoids hardcoding need vocabulary into the client. Unknown/malformed v2 never falls back to deficit, schema1 recordings have explicit legacy handling, and owner-provided bars retain no client calorie/threshold arithmetic. Existing R7 remains binding. No-needs deterministic controller behavior, prompt golden and renderer-free lifecycle remain required independent witnesses.

No newly proposed God object, generic resource framework, replacement scheduler/database, duplicate hunger state, controller optimizer, cross-owner currency mutation or renderer-owned domain rule. SDK hook/version extension and persistence factory are small existing-owner seams. Standard integer arithmetic and existing SQLite/dispatch/authoring/observation facilities are reused; a new dependency is unnecessary and no mature wheel is being reimplemented. Future code review must still inspect actual function/file growth and typed error paths; this plan review cannot certify unseen implementation maintainability. The 938-line file is primarily preserved binding procedure, not an oversized production module.

Both supported uses share one authoritative model: headless simulations run without models/renderers; human/rule/LM controllers and 2D/3D views remain replaceable. Synthetic dual-currency/capacity/composition/restart acceptance is appropriately separate from Market Town content. Actual two-client screenshots and lifecycle tests remain decisive requirements, not source-inferred passes.

## Concrete operator decision package after R1–R3

1. Accept optional food-energy mode inside needs: one-day bounded reserve with C=D, configured 2000 default, reserve bar distinct from normalized hunger motivation, explicit one-unit serving and no forced medical/collapse consequences.
2. Accept needs2/new-save-only and retained matching old package/config for existing saves; explicit incompatible-load refusal preserves logical committed history, with no migration/reset or physical byte promise.
3. Accept the explicitly scoped SDK version/validation hook, generic invalid-genesis diagnostic and valid-genesis-first lazy persistence creation in both CLI callers, plus mapped schema-aware displays. Kernel receives no hunger/currency rules.
4. Accept separate engine/model and Market Town content/integration checkpoints. Engine approval does not approve retail values or NPC finances/policies or declare whole playable delivery complete.

Do not re-ask already approved configurable2000 or USD10000 visitor genesis. Before freeze the coordinator still records actual approval/semantic revision, integrated exact base, isolated worktree/session owner and implementation/publication endpoint source. These are dispatch facts, not nutrition preferences. A planning PASS is not execution authority.

Content remains **BLOCKED** until ten verified portion/price rows and concrete unchanged-policy reachable-choice, every-prefix cash/wage/stock/production proof exist. Current controller includes chance/generic/free-hand paths and cap waste; a cheap feasible basket alone cannot prove300-day affordability. Original S35 rhythm/long acceptance retains its bounds. No current NPC endowment/wage/stock adequacy is accepted by this review.

## Remaining capacity

Independent reviewer remains available for startup_recover candidate and memory/night design when their actual artifacts are ready. No duplicate startup experiment was launched; prior startup performance FAIL/inconclusive checkpoints remain as recovered.
