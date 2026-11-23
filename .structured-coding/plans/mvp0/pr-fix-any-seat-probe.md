# F-RC-7: seat-independent link probe and truthful failed-check exit

## Status and authority

## DESIGN FROZEN

Approved execution dispatch: primary bb9217c8-5c5f-4fde-b599-33442d8c3257, message
e0737728-aae2-4d43-839e-2634983d0f9e (2026-11-22), reports independent plan PASS and
explicitly orders this fresh implementation session. Scope/invariants/acceptance below are unchanged.
Lifecycle: implementation active. The original prepared status was stale at dispatch.

Execution contract: this PR only; base and ownership below. Implementation, bounded local tests and
semantic commits authorized by that dispatch. Push/PR/merge/tag require primary approval of concrete
payload. No subagents or paid APIs. Primary owns independent review and parent/shared documents.
Live evidence and continuation state live in this ledger; no separate shared handoff is edited.
Static owner: Godot script registration; deterministic owner: focused Logger regression; real lifecycle
owner: exported server/client link probes; real-LLM Gate N/A (no LM semantics). Full suites/remote CI
NOT RUN by explicit local-first dispatch. Integration budget 900 seconds total, each client <=180s.

Source audit: Cafe branch retains movement/jumps/far/near-talk assertions; Lakeside takes `_route_check`
before that branch. Shared `_through_door` stays unchanged: link-only departure routing prepares each
outward door through authoritative walk-to, then uses existing traversal. Old WIP is recipe only.
Official Logger 4.7 documentation checked: callbacks may run concurrently, no logging in callbacks.
https://docs.godotengine.org/en/4.7/classes/class_logger.html .
Branch `fix/any-seat-probe`, isolated checkout `/tmp/mineworld-any-seat-probe`, base
`4cd160bbbdc3f7244a0fd6b65bcf468138b198df` (PR #530 prerequisite). Do not amend #530.
The coordinator owns merges and shared step/overall updates; this document owns this bounded repair.

The operator authorized takeover and local-first validation, preserving structured coding and independent
red-team review. Coordinator explicitly delegated F-RC-7 recovery. Frozen authority is step-23-release.md
§19.5 F-RC-7 and `tools/launch/LAUNCHER.md` §2: standalone 3D uses visitor; both uses 2D visitor and 3D
wanderer. This repair selects the documented client-probe correction, without changing launcher seats.
Historical coordinator dispatch is corroborating evidence, not a newly invented user approval:
Claude session `64c4f5c0-38d3-474d-bf6e-27af3609b29c`, message
`e7e37d89-a1ae-42f4-9893-938fad2cba10`, 2026-10-27T21:08:45.987Z, Task invocation requests any-seat
routing, nonzero exit for ANY failed probe check, unchanged launcher seats, three-world nondefault-seat
checks, and a deliberately failing negative check. Its obsolete authorship instructions do not apply.

Structured-coding SKILL §Workflow rules 5 and 10 require a fresh implementation session for this PR.
The current #530 implementation agent prepares this design and evidence handoff only; the coordinator
assigns a fresh implementation context after independent design review.

## Immutable WIP evidence and audit

Old `/Users/yuema137/mineworld-worktrees/fix-probe-exit` remains read-only, branch
`fix/probe-any-seat-exit`, HEAD `8a3412abf8ea3db8ad15a7cdac9876231ed98fd3`.
Snapshot `/tmp/mineworld-frc7-wip-snapshot` contains binary tracked.patch, status.txt, manifest.json,
and copies of all five changed/untracked files. SHA-256 of each copied file:

| File (relative to repository) | SHA-256 |
| --- | --- |
| clients/3d-spike/scripts/slice/slice_probe.gd | 25aad9f0c1a2e390dfbe7d5190b6dfe9dede6848ecfa9e31c3c99036cacb0fb9 |
| clients/3d-spike/scripts/slice/slice_probe_world.gd | 4879cf401283e42597a1f6212fe3b30ae399f0df3049fb3414a8a120921826a4 |
| clients/3d-spike/scripts/slice/slice_probe_verdict.gd | 6e54773686eae5622ab915a897e200dbac390c3b34b40fa4b769f63b17c01eb3 |
| clients/3d-spike/scripts/slice/slice_probe_verdict.gd.uid | c21886ddcd7e86f88cc055dd5501267fee778630c32108a2a74c0399cba536a8 |
| scripts/ci_godot.py | 143799c94769701b584bdc97919654c2cf1959f6c61b3d008135ac3cf6e691b0 |

The WIP is a design input, not a verified patch. Its movement change reuses `_cross` and `_route_to`
(authoritative walk intents, never teleport) and selects the depiction's first enterable room, matching
`_room(0)`. The perceived-people assertion moves after entering that room but is not weakened.
Its shared `_through_door` outward change can affect other modes and requires targeted parity evidence.
Its Logger has unsynchronized mutable state and a shared RegEx, despite callbacks being concurrent.
Official Godot Logger documentation explicitly requires synchronization of callback state:
https://docs.godotengine.org/en/4.7/classes/class_logger.html . Never log from a callback.
Its classifier misses `CONVERSATION CHECK FAILED (empty caption or an id on screen)` and falsely claims
identical semantics to Python `judge_probe`. Existing Python classifier remains semantic PASS owner;
process exit 0 alone must never become PASS.

## Invariants and scope

No gameplay/server rules, launcher seat mapping, release workflow, asset content, or production defaults
change. Keep every existing link assertion: seated/body placement, doorway transitions, jogging,
two jumps, accepted movement, exact last authoritative position, far rejection and near talk/reply.
A seat whose place is not drawn, a missing enterable room, unreachable route, or refused intent fails
with a diagnostic and nonzero status. Missing topology must not cause a crash, silent success, or teleport.
No broad refactor of the existing oversized probe files. Introduce at most one small verdict helper
plus its focused regression test; no new gameplay state owner or third-party dependency.

## Commit 1 — seat-independent movement

Goal: repair the concrete wanderer route while preserving assertions; independent of exit collection.
Scope: `clients/3d-spike/scripts/slice/slice_probe_world.gd` and this ledger.

- [x] Reinspect current `_link_check`, `_cross`, `_through_door`, `_route_to`, depiction/door mapping.
- [x] Port only audited WIP topology routing: current place to street to the first enterable room.
- [x] Fail explicitly on absent/unusable mapping; inspect route helpers for invalid door indexing.
- [x] Preserve actual controller traversal and authoritative intent path, all verdict checks and stdout.
- [x] Keep shared outward route adjustment only if required; document affected sibling modes.
- [ ] Validate default visitor and wanderer in social-cafe, and documented valid nondefault seats in
  market-town (`wanderer`) and lakeside (`bruno`, with `alba` as declared first/default seat).
  These are current `worlds/*/world.yaml` seats; inspect invite plumbing before invoking.
- [ ] For shared doorway changes, run bounded direct outward/inward traversal coverage from room and street.
- [ ] Record command, source/server hashes, terminal verdict, failed checks, exit status and wall time.

Acceptance: all requested positive cases terminate with exact `all link checks pass`, no check/script/
parse/asset errors, actual place transition sequence and accepted movement; baseline wanderer failure
is reproduced where practical. Invalid place/topology yields a diagnostic failure, not a success line.
At this commit historical false-zero behavior may remain, explicitly pending commit 2.
No unit test mirroring route implementation; real server/controller integration is the validation owner.

## Commit 2 — failed-check exit propagation

Goal: every completed probe run that reports a failed check exits nonzero, without fabricating PASS.
Scope: `slice_probe.gd`, one runtime-owned verdict helper and UID, one focused Godot regression driver,
this ledger. Change Python prose only if needed for accurate exit-versus-verdict wording; do not claim
identical classifiers or expand CI behavior without separate rationale/review.

- [x] Inventory current failure output in all slice probe modes and record the vocabulary/failure owners.
- [ ] Use a small Logger collector for distributed prints, explicitly preload its runtime script; keep
  it exportable under existing presets. Preserve mode-owned nonzero codes.
- [ ] Guard callback writes AND reads with a per-instance Mutex; avoid shared mutable static RegEx;
  do not print/log from callbacks. Use a synchronized final snapshot after unregistering the logger.
- [ ] Collect FAIL/[FAIL] prefixes, numbered failure/out-of-range summaries, and singular conversation
  failure; handle whitespace and multiline callbacks. Do not classify explanatory/pass lines as failures.
- [ ] Set status directly in the central row-style `_verdict` where FAIL is a suffix; keep stdout unchanged.
- [ ] Register before probe execution; unregister before diagnostic/quit and ensure cleanup on exit.
- [ ] Negative regression executes real Logger callbacks with each inventoried failure shape, multiline,
  success-only controls, repeated/concurrent failures and stable first-failure capture. Assert child
  process nonzero for an intentional probe failure and zero for completed success.
- [ ] Verify existing nonzero mode status is preserved and absence of a success verdict never becomes PASS.
- [ ] Export and run the fixed 3D PCK outside checkout with clean class registration, wanderer seat,
  terminal verdict plus exit and error screening. Also execute a deliberate failure from that package.

Acceptance: focused regression controls prove correct exit propagation, thread-safe collection, no
logger recursion, no masking of existing status. A real failing packaged invocation exits nonzero;
a real nondefault-seat packaged run reports all link checks pass and exits 0. No manual per-mode
replacement of the 174 failure-related sites or unbounded inheritance/reference coupling.

## Bounded validation and commands

Use one heavy slot at a time through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`; no full suite or full asset import.
Before execution inspect project-local script usage (`scripts/ci_godot.py --help`, `scripts/package.py
--help`) and current world invites to fill exact seat commands; do not invent unsupported flags.
Godot scripts-only check and focused logger regression should cost seconds; three-world real link
checks approximately 2–3 minutes each. Budget 15 minutes for the integration batch; stop/report
inconclusive rather than silently expand import/route timeouts. Each client timeout <=180 seconds;
kill owned process groups and stop server in finally. Independent failures may be diagnosed locally
within scope, but substantial budget or invariant changes return to coordinator.

Cache acceleration previously verified in #530: copy old WIP `.godot` only after byte matching actual
assets+sidecars, remove copied script/class metadata, rebuild registration, use ordinary preset/ASTC.
Do not mutate old trees. Prior preflight matched 321 source assets+sidecars; previous provenance field
`assets_count:387` was POST-import sidecar count, not 387 verified assets. These counts must be recomputed.
Cached server SHA-256 `14d32964f17b7ba63678ca70f929d39892871c44bc8576113f0086fb345a48e0`
from `impl-s23-rb/target/release/mineworld` is diagnostic only: binary exact-head provenance unproven.
Reuse current packaging APIs; `/tmp/mineworld-release-recovery.md` and `/tmp/mineworld-local-link.py`
provide concrete successful bundle commands, but copy decisive commands/results into this durable
ledger. Package logs must reject script/parse/asset/check errors in addition to `judge_client`.
A single-seat probe does not close both-mode launcher, windowed, cross-platform, or full release gates.

## Ready-for-review and handoff gates

- [x] Independent plan review and fresh implementation-session reading complete.
- [ ] Each commit records staged files, exact diff summary, actual tests/count/time, deviations.
- [ ] Explicit technical-debt review: modularity, maintainability, readability, robustness, extension
  cost, file/function size, inheritance/reference coupling; preserve stronger repository size triggers.
- [ ] Existing oversized slice_probe files identified as inherited debt; introduced delta bounded,
  verdict helper cohesive, any new topology complexity justified by real route evidence.
- [ ] Independent final red-team review of exact final executable SHA and actual logs complete.
- [ ] Public push/PR permission resolved for concrete reviewed artifact; no assumption from #530 consent.
- [ ] Coordinator receives exact head, dependency, evidence boundaries and remaining release gates.

No implementation/test is claimed complete by this design. Local baseline #530 default-seat packaged
link has independent PASS, but is not F-RC-7 evidence. Historical WIP tests are not adopted as ours.

## Execution checkpoint — 2026-11-22

Fresh owner read root AGENTS/CLAUDE and execute/resume routing. Source base verified at 4cd160bb;
old WIP never edited or executed. No other owner was assigned to this checkout by dispatch.
`ps` is blocked by the sandbox; no duplicate task jobs were started. Shared slot wrapper owns heavy jobs.

Implementation milestone 1 is in the working tree: only `_link_check` and two local helpers change.
The shared `_cross`, `_through_door`, `_route_to` remain byte-identical to base, so sibling doorway
regression obligation for a shared behavioral edit is N/A. The first room is selected in depiction
order; door index is bounded. Departure detects missing doors/cycles and routes around furnishings
before traversing the existing door. People-count assertion moves after room entry, unchanged >=2.
Lakeside's `probe_route` is unchanged and does not run Cafe jump/talk assertions.

Milestone 2 working implementation: runtime preload of a 35-line per-run Logger collector; instance
Mutex serializes callbacks, regex use and snapshots. No callback prints/logs. First means first
observed under this mutex, never wall-clock order between threads. Base probe unregisters before its
exit diagnostic, keeps the collector member alive through quit, and unregisters idempotently on tree
exit. All probe producers are awaited coroutines on the scene tree; audit found no Thread/WorkerThread
producer in the probe inheritance chain. The callback's locked snapshot drains an active critical
section; it does not promise to collect future messages after unregister. Modes must finish their
producers before returning. External semantic/error screening remains required.

Vocabulary audit: FAIL/FAIL:/FAIL-space and [FAIL] prefixes; positive numbered uppercase CHECKS
FAILED/OUT OF RANGE summaries; singular CONVERSATION CHECK FAILED and CAST CHECK FAILED. Central
row `_verdict` sets status directly, preserving a preexisting nonzero. Town/body/geometry rows keep
their existing mode counters and terminal summaries; market retains its own exit handling. No global
ERROR classifier/bypass was added. Python judge_probe is unchanged and still owns semantic PASS.

Initial static/regression/export command:
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh python3 /tmp/mineworld-any-seat-prepare.py`.
Driver uses ordinary package APIs/presets, copied matching cache, current pack export. Evidence under
`/tmp/mineworld-any-seat-evidence`: 1.log registration PASS 4.96s, 2.log focused callback and subprocess
regression PASS 3.79s, 3.log runtime import PASS 1.27s, 4.log runtime export PASS 6.46s,
5.log 3D macOS export PASS 2.53s. Exactly 321 source assets and sidecars were byte-compared pre-import;
post-import sidecar totals are not claimed as matched input counts. No full texture import was needed.
The regression has since gained an explicit held callback across unregister; its rerun is pending.

Real lifecycle run command:
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh python3 /tmp/mineworld-any-seat-run.py`.
Gate claim: actual exported PCK under shared runtime outside checkout, real server, default and alternate
seats in three worlds; deliberate no-server invocation must report failure and exit nonzero without
success. Maximum 180s/client, 900s batch including retries; no parallel heavy runs. Current matrix is
running. Source hashes and PCK hash are in prepare.json; copy decisive evidence into this ledger at
completion. No result is inferred from output in progress or solely from a zero exit.

Bounded review correction during initial matrix (the exported PCK is immutable for that batch):
initial departure failure previously continued into the normal inward `_cross` call, risking another
bad index after an already diagnosed missing door. It now terminates the failed scenario immediately.
The target-room ancestry is checked for missing/out-of-range doors and cycles before shared traversal;
the eight-place bound matches `_cross`'s existing bound. Refused departure intents now fail explicitly
instead of allowing `_route_to`'s fallback leg to hide them. These alter failure guards, not valid route
movement. Final export/retest is required for executable provenance; initial batch remains recorded
against prepare.json hashes and is not represented as byte-identical final-head evidence.

Initial runner screening typo: its case-sensitive allowlist did not match the actual documented
`ERROR: Not supported by this display server.` / `ERROR: Could not create directory: 'res://shots'.`
lines. Raw logs are retained; postprocessing will allow only these exact messages and reject all other
ERROR/script/parse/asset/check failures. This is a harness correction, not a production error bypass.

Final bounded run will use existing `--watch=1`. Source audit of `_street_watch` shows the default 60s
is an unasserted people-observation diagnostic; it performs no acceptance checks. The flag only shortens
that final observation interval, retaining the outward traversal and every original assertion. It avoids
spending most of the remaining 900s integration envelope idling and permits final-head default/alternate
seat coverage. Per-client 180s remains unchanged; initial attempts count against the total.

Final export static/regression: `/tmp/mineworld-any-seat-finalize.py` under the same slot wrapper.
Scripts-only registration PASS 2.08s; expanded Logger regression PASS 4.14s, including a real callback
held on a semaphore while OS.remove_logger runs, then released/joined before synchronized status read.
This proves the retained reference permits completion across unregister; it does not claim removal
itself joins producers. Incremental macOS PCK export PASS 2.59s. Final PCK SHA-256:
`8b2613482d231cf33b0b5ad715f703b509f043573ed1a04ce6c7117da3fe2486`.
No source edits after this export (test content included in recorded source hashes; tools excluded
from the pack). `ci_godot.py` prose corrected; classifier executable unchanged; its 26 self-checks PASS.

Preliminary evidence: Cafe visitor PASS 112.84s, 28/28 moves; wanderer PASS 153.72s, 214/214 moves.
Both exact terminal `all link checks pass`, exit 0, two jumps, 2→8→2 place transitions, identical reported
and observed position, rejected-too-far door talk, accepted counter talk and reply. Original raw
runner result misclassified the two exact documented headless messages; reviewed-results.json records
corrected screening without overwriting results.json. The next Market visitor startup was intentionally
interrupted to switch packages; INCONCLUSIVE, no acceptance claim. Owned driver/client/server stopped.
Preliminary batch charged 269.76s. Final batch has at most 630.24s left, still <=180s/client.

### Maintainability and logic review (implementer, before final ready review)

Inherited size debt: `slice_probe.gd` was 1366 lines, now 1393 (+27); `slice_probe_world.gd` was
1735, now 1792 (+57 net). `_link_check` remains an oversized ~191-line scenario; its existing assertion
sequence is deliberately kept together under the no-broad-refactor contract. New topology helpers are
24 and 25 lines, each one operation (select/validate destination and leave initial place). New Logger
is 35 lines, the focused driver 100 lines. No new deep inheritance, dependency, authoritative world
state, launcher coupling or server rule. Existing probe inheritance is unchanged; native Logger is
used through one explicit preload. This is bounded growth in already oversized files, not resolved
size debt. Splitting their unrelated historical modes is deferred outside this repair.

Ownership: probe owns status, collector owns only its first observed failure under a per-instance lock;
server owns all movement and interaction. Logger reads and writes use the same mutex; no static mutable
regex and no callback logging. Completion/unregister/retained-reference lifecycle is explicit. Future
failure vocabulary changes need a collector case and regression; zero status intentionally proves no
semantic success. Exported runtime owns its helper rather than importing an excluded tools class.

Review fixes: invalid/cyclic target ancestry rejected before shared indexing; failed departure stops;
refused departure intent cannot be hidden by later movement. Valid path integration is the regression
owner. No sibling traversal behavior changed. Removing the helper changes would reintroduce the
historically recorded wanderer failure; we did not spend budget rerunning an obsolete PCK baseline.
The negative child/regression and packaged no-server case discriminate the previous false-zero defect.
No material new maintainability debt identified; existing void `_route_to` and mode-specific output
vocabulary remain inherited constraints, handled locally rather than broad refactoring.

Static evidence: `git diff --check` PASS; `python3 scripts/check_doc_headings.py` PASS (254 numbered
sections across 337 documents); Python AST comparison excluding module docstring proves `ci_godot.py`
executable AST unchanged. Recomputed hashes of all four exported/test source files match final
prepare.json. All heavy checks used one slot; full suites and remote CI deliberately NOT RUN.

### Reproducible final commands

From this checkout, under `cast-slots/with-slot.sh`, Godot 4.7.2:

```sh
godot --headless --path clients/3d-spike --editor --quit
godot --headless --path clients/3d-spike --script res://tools/test_probe_verdict.gd
# package.etc2_astc_for(project, TARGETS['macos-universal']) restores settings afterward:
godot --headless --path clients/3d-spike --export-pack macOS /tmp/mineworld-any-seat-final-evidence/clients/mineworld-3d.pck
```

The ordinary `package.export_runtime`, `layout`, `client_settings`, `write_bundle_toml`, `check_contents`
and `file_digest` APIs assemble/verify a non-distributable diagnostic bundle outside the checkout at
`/tmp/mineworld-any-seat-final-evidence/MineWorld 世界/diagnostic`. Current tracked worlds and packs,
including station data, are copied by `layout`. `check_contents` and manifest digest PASS before runs.
Server invocation per fresh case (actual credentials/ephemeral port stay only in untracked local logs):

```text
<runtime>/mineworld server <runtime>/worlds/<world> --listen 127.0.0.1:0 --save <case>/save
  --agent alice  (Café and Market only; Lakeside has no Alice and its route has no talk assertion)
<runtime>/clients/MineWorld.app/Contents/MacOS/MineWorld --headless --log-file <case>/godot.log --
  --client=3d --root=<runtime> --settings=none --world-key=<world> --slice-link
  --aniso=off --render-cost=low --watch=1 --server=<join-address> --invite=<join-token> --seat=<seat>
```

Use `package.runtime_executable` for the actual executable basename (do not assume the displayed
example name on another export). The deliberate failure uses social-cafe with the same client flags,
omitting server/invite/seat. Each subprocess has its own group and bounded timeout; finally stops the
owned server and closes its log. `_street_watch` still exits to the street, enumerates doorway/person
observations and reports start/end; only its final `while t < secs: await _hold(1.0)` diagnostic interval
is 1 instead of 60 seconds. That interval has no assertion/counter, creates no worker, and changes no
quit/logger lifecycle guarantee. All existing semantic checks precede it and remain unchanged.

### Final-package results (live)

Every row in this section uses PCK SHA-256
`8b2613482d231cf33b0b5ad715f703b509f043573ed1a04ce6c7117da3fe2486`, source hashes in final
prepare.json, and cached server SHA-256
`14d32964f17b7ba63678ca70f929d39892871c44bc8576113f0086fb345a48e0` (exact-head server build
provenance unproven; not release acceptance). All positive cases require exactly one literal
`all link checks pass`, zero process exit, no failed-check/script/parse/asset errors, and no ERROR
except the two exact documented headless lines above.

| World / seat | Exercised path | Wall seconds | Terminal / exit | Movement and interaction evidence | Result |
| --- | --- | --- | --- | --- | --- |
| social-cafe / visitor | Café `_link_check` | 53.80 | `all link checks pass` / 0 | 2→8→2, 28/28 moves, two jumps, sent=observed (1598,1257,0), far rejected, near accepted, reply | PASS |
| social-cafe / wanderer | Café `_link_check` | 95.59 | `all link checks pass` / 0 | 2→8→2, 214/214 moves, two jumps, sent=observed (1985,1243,0), far rejected, near accepted, reply | PASS |

The other final cases are still running; no result is claimed yet. Snapshot comparison of `_cross`,
`_through_door`, `_route_to`, `_route_check` against base confirms these functions are unchanged.

Final PCK inspection: 243,257,776 bytes, 931 entries, zero tools entries. Includes
`scripts/slice/slice_probe_verdict.gd.remap` and `.gdc`; focused test is excluded as intended.
Market Town visitor final-package PASS 52.36s: same Café path, 2→8→2, 28/28 accepted moves, two jumps,
position (1598,1257,0) exact, far rejected-too-far, near accepted and reply, terminal success / exit0.

If budget remains after the final matrix, `/tmp/mineworld-any-seat-negative.py` (one slot) owns two
additional cheap discrimination checks: current exported social-cafe `carol` starts in apartments,
which this depiction does not draw, and must produce diagnostic failure/nonzero without a crash;
the preserved #530 diagnostic package (not old WIP) run without a server should reproduce historical
FAIL text/exit0. Each <=30s and all elapsed time charged against the same900s. These are pending, not
passes. No scene/server asset is mutated for either case.

Bounded harness discovery: `package.layout` copies only `WORLDS = (social-cafe, market-town,
bodies-yard)`, not Lakeside. Contrary to the earlier shorthand "current tracked worlds", it copies
those selected worlds and all required pack categories. Lakeside server startup therefore reported
"is not a World Pack directory" before the client launched: INCONCLUSIVE, not a probe failure.
Preserved at `lakeside-alba-setup-inconclusive/server.log`. No production packaging scope is changed.
Resume uses the same package.copy_tracked API for `worlds/lakeside` into this diagnostic bundle, then
regenerates/verifies BUNDLE.toml. Same PCK/runtime/server; only the requested fixture is supplied.
Initial final-matrix batch elapsed296.70s; preliminary269.76s; remaining333.54s, no timeout expansion.
`/tmp/mineworld-any-seat-resume.py` under one slot runs the mandatory no-server negative first, then
Lakeside alba/bruno. Results append in resume-results.json without overwriting initial final results.
Market wanderer final PASS94.77s, 205/205 moves, two jumps, 2→8→2, position(1992,1226,0) exact,
far rejected, near accepted/reply, exact terminal success / exit0, no unexpected errors.

### Semantic commit 1 preparation

- [x] Implementation: scoped Cafe route and failure guards in slice_probe_world.gd; shared helpers unchanged.
- [x] Deterministic/static validation: Godot registration PASS; actual exported Cafe/Market default and
  wanderer cases PASS on final package as recorded above. Lakeside (unchanged route) remains running.
- [x] LLM logic self-review: original assertion sequence preserved; missing topology and refused departure
  fail; bounded ancestry matches shared traversal; no launcher or world-state changes.

Staged payload for commit1: `slice_probe_world.gd` plus this live ledger only. Other working-tree
Logger/test/prose changes are intentionally separate commit2. Tested source is explicitly the combined
working tree identified by final prepare.json; commit1 alone is not claimed to have new failure-exit
behavior. Local final payload/independent review is still pending, not READY.
