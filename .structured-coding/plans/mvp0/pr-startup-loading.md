# Startup loading correction — 2026-11-23

## DESIGN FROZEN — 2026-11-27 (operator ratification)

```text
Design revision:        revision 1 (2026-11-23, recovered as bae264be) + bounded corrections R1 (2026-11-27,
                        _reviews/pr548.md finding 1: scripted runs no longer wait for first view) and R2
                        (2026-11-27, perf lane: truthful first-camera graphics stage)
Approved by / evidence: the operator, 2026-11-27, relayed by the primary coordinator:
                        "启动器按推荐；启动加载和启动器设计冻结，接受慢 8 秒，3D 实跑放到里程碑"
                        ("launcher as recommended; startup-loading and launcher designs frozen; accept 8 s slower;
                        3D real runs go to the milestone")
Operator rulings:       D1 ratified as a bounded correction to S23-RT §4.1.
                        D2 the measured +8.1 s first-view regression (145.180 s → 153.281 s, matched pair
                           2026-11-23) is ACCEPTED as a recorded deviation, pending the separate shader-cache work.
                        D3 the 3D source/exported runs of the checklist (frame order, input hold, Esc/Quit,
                           failure frame, cold-cache graphics stage) are DEFERRED to the milestone trial; they
                           remain NOT RUN here, never counted as passes.
Implementation base:    ff4e41e3 (client stack), published on #544 (merged); branch fix/startup-loading-ui (PR #548)
Lifecycle:              FROZEN (2026-11-27); implementation complete, under review
```

### Execution contract (filled; binding)

```text
Owner / worktree:       one session; /Users/yuema137/mineworld-worktrees/impl-startup-ui, branch fix/startup-loading-ui
Affected paths:         clients/shared/loading/**, clients/shared/LOADING.md, clients/runtime/runtime/boot.gd,
                        clients/{runtime,3d-spike}/mineworld_loading (symlinks), clients/3d-spike/scripts/slice/slice_main.gd,
                        clients/shared/settings/locale/{en,zh_Hans}.po + messages.pot, this ledger
Authorized:             the correction as designed; bounded fixes from review; targeted checks; semantic commits; push
Not authorized:         world/server/schema/settings-semantics/rendering-quality changes; speed claims; merge; milestone label
Validation owners:      GDScript parse (scripts/ci_gdparse.py incl. clients/runtime); headless runtime boot checks;
                        headless 3D connected-probe join failure (named FAIL inside the probe's own bound);
                        3D source and exported runs (frame order, input hold, Esc/Quit, failure frame); remote `fast`
Budgets:                <=600 s per heavy run, one shared slot (cast-slots/with-slot.sh), no paid API calls
Escalate:               any change to exit statuses, quality settings, or first-view readiness semantics
```

Bounded correction authorized by operator's slow 3D startup/no progress report and coordinator dispatch. This is a new isolated execution context, not a change to simulation or MVP acceptance. Base 703aa8dce66c3fa150942315e74bffd346fcfa8a from reviewed playable source; clone /tmp/mineworld-startup-20261123-source. Independent reviewer /root/delivery_review. Parent owns progress/shared plans. No public writes/merge, no paid calls. Local implementation, targeted validation, semantic commit and immutable local artifact authorized by dispatch. User saves, existing source/caches/bundle are read-only.

## Audit and invariants
Read root CLAUDE, Structured Coding skill/workflow/adaptation/implementation/test/design rules, engineering rules/standards, vision/architecture, 3D PRESENTATION, S23-RT and launcher contract. boot.gd synchronously mounts PCK then deferred synchronous scene load; no UI. SliceMain._ready performs all setup synchronously; _process assumes player exists once world exists. SliceLink.has_first_view is existing arrival authority, said gives translated connection status. No timings inferred from previous smoke/movie total duration.
Preserve settings, rendering quality, setup order, selected packs/worlds, server authority, save schemas, launcher supervision and existing exit statuses. Headless must not await an absent draw. Source launch remains valid. Runtime error status2 preserved; errors in main displayed persistently, scripted refusals status3 unchanged.

## C1: truthful loading UI and staged initialization
Goal: present before blocking resource work, retain meaningful current stage until usable world, keep existing result/quality. Scope shared small CanvasLayer adopted by runtime+3D using existing symlink pattern; boot loader and SliceMain setup gating only. No dependencies or world/server changes.
- [ ] Baseline default packaged startup with timestamped stdout and isolated server/save; preserve existing baseline. Process inventory first; concurrency recorded.
- [ ] Built-in-font screen independent of client resources; actual rendered frame before pack mount. Root ownership survives scene transition, bounded cleanup for 2D and3D.
- [ ] Engine threaded scene resource request/status; stage-specific fraction only; failures remain named and status2.
- [ ] 3D yields actual draws between original setup stages, gates process/input until initialized; retain loading through existing first-view predicate, show existing connection failure notices; quit remains available. Source creates own UI.
- [ ] Log stage monotonic timestamps for actual phase measurement; no fake progress or sleeps.
- [ ] Independent source/diff review including async hazards, headless,2D lifetime, localized wait status.
Validation owners: engine parser/static plus real Gate2 source/export startup,2D and both launcher smoke, failure and no-server waiting. Gate1 N/A; full workspace N/A no Rust/world logic changed. Per run <=600s, initial aggregate <=3600s, one shared heavy slot, no paid API.
Acceptance: actual default packaged frame visible before heavy scene work; stages truthful; first world view actually arrived/rendered before dismissal; no input into half-built world; 2D and both modes finish current smoke; failure visible/status preserved; unchanged quality, user state untouched. Speed improvement claimed only measured at comparable path and load. Synchronous indivisible engine/world phase stalls remain explicitly disclosed if present.
Edge cases: missing pack/invalid scene; resource load fails; headless; source launch; disconnected/refused join; early user exit; no network/offline scene. No new retry policy.
Commands: baseline.py first; existing package.py assemble/export helpers, Godot --headless --editor --quit import/parser, existing package smoke; concrete exact commands and identity in live evidence before runs. Coherent single correction commit with code/tests/contract; inspect exact diff, no unrelated cleanup. Reviewed local handoff endpoint only.

## Initial Gate specification
Baseline.py launches unchanged exported runtime with default graphics and selected social-cafe via existing start_server, own scratch only; timestamp stdout until first-view log, then owned shutdown. <=180s. This identifies pre-world/first-observation latency but stdout alone cannot prove first drawn frame. Existing user process may affect measurement; inventory recorded. Evidence baseline/timed.log and result.json. Next default run will use same path. Visual frame proof is separate; no baseline pass claim without it.

## Live evidence / decisions
- Independent PLAN review PASS by delivery_review. Original S23-RT evidence untouched.
- Baseline first attempt INCONCLUSIVE: stdout reader dropped buffered remaining lines when process ended. Recovered engine log confirms cafe arrival, no crash conclusion. Retry default pack first-view log observed by10.919s, file buffered all early stages together; upper bound only, not cold/warm or engine-phase timing. Own server/save. Initial user client had already exited naturally.
- Source visual initial FAIL: adding root child during ready caused busy-parent and nil controls. Deferred attachment + await ready fixes the actual failure. Historical failed logs retained.
- Corrected source visual PASS100frames35.552s; first UI draw0.739s, construction6.511s, light3.433s, first joined frame11.976s. Default render settings, movie capture overhead explicitly not production-speed proof. Frames1/40 visually inspected loading/world. Source root/catalogs used.
- Reviewer mouse/input findings repaired: visible mouse while waiting, held until final draw; root layer consumes gameplay keys and offers Esc/GUI quit. Sibling offer-menu shortcuts otherwise ignore the main entry handler's initialization gate. Runtime2D finish connected to persistent layer directly before scene transition; boot continuation would not survive.
- Scope: synchronous world/glTF and GI baking identified bottlenecks. These remain unchanged to preserve current depiction contracts/quality. Stage feedback repairs blank splash; no total startup speedup claimed.
- API verification: https://docs.godotengine.org/en/stable/tutorials/io/background_loading.html (request/status/get; get blocks unless LOADED); RenderingServer reference (headless disables rendering; frame_post_draw for draw barrier).
- Local parser/import runtime+3D PASS, no script errors. 42 LFS binary hashes verified against committed oid before hydration; git diff has no asset changes. 3D cache copied; import checked66 presentation reference images, no new art or texture campaign.

## Publication — 2026-11-27 (Claude takeover)

Approval basis: bounded correction from the operator's 2026-11-23 trial report; independent plan PASS by
`delivery_review` (`handoff-coordinator.md` "Operator trial feedback: 3D startup latency"). The `/tmp` source was
lost on reboot; `bae264be` is the byte-verified recovery of `2295aa8d` (`recovered/startup-2295aa8d`). Branch
`fix/startup-loading-ui` merges PR #544 (`origin/mvp0/pr-s34e-seated` @ `02b6fee0`) and then `origin/main`
(`303c0df3`); neither touched `boot.gd`, `slice_main.gd`, `clients/shared/loading/` or the locale catalogs since
`ff4e41e3`, so both merges are clean and the correction's diff is unchanged.

Local evidence on this branch (`mineworld-worktrees/_targets/startup-ui-evidence/`):

- `scripts/ci_gdparse.py clients/shared clients/2d clients/3d-spike clients/runtime` — PASS (35/73/151/2 scripts,
  0 failed; the runtime project is added explicitly because the default list omits it).
- Headless shared-runtime boot against scratch roots with a freshly exported 2D pack (`run-checks.sh`):
  missing arguments → exit 2, `[runtime] FAIL …` and the error shown on the loading layer — PASS; unreadable pack
  settings → exit 2 named — PASS; scene path absent from the pack → threaded request reports failure, exit 2
  `cannot load scene` — PASS; valid 2D scene → stages `Loading scene resources…` → `Preparing scene…` →
  `client scene visible` (scene_changed one-shot on the root layer), exit 0 — PASS. Headless draws are scheduling
  markers only, not visual proof.
- 3D source/exported scene run, input-hold/Esc check and exported failure frames: NOT RUN in this session (the 3D
  project's full asset import is the heavy step); historical 2026-11-23 evidence above stands as historical only.

Performance status (unchanged, not resolved): the matched-condition 2026-11-23 pair measured old first view
145.180 s vs UI first view 153.281 s — an **8.1 s first-view regression**, performance acceptance FAIL; the loader
itself is drawn at ~15 s in that packaged run versus a blank splash before. Later diagnostics (`recovery-plan.md`)
rejected the lazy Metal shader-loading switch and point at path/history-dependent first-camera pipeline
compilation; the next diagnostic (draw-barrier/pipeline counters) is not run. No speed-up is claimed; the
investigation stays open as its own follow-up.

### Bounded correction R1 — 2026-11-27 (`_reviews/pr548.md` finding 1, BLOCK)

- **Defect.** The connected 3D path waited on `link.has_first_view()` without a bound, before `_start_probe()`. A
  scripted probe whose join fails therefore never started, so a harness run hung until killed (CI INCONCLUSIVE)
  instead of printing the probe's own named FAIL.
- **Fix.** Scripted runs (`SliceMain._scripted()`) no longer wait for first view. The probe starts right after
  setup, as it did before this PR, and bounds its own join wait. Players still wait on the loading layer, which
  shows the worded link status and keeps Quit/Esc available.
- **Regression check.** `_targets/startup-ui-evidence/probe-join-fail.sh`:
  `godot --headless --path clients/3d-spike res://scenes/slice.tscn -- --slice-link --server=127.0.0.1:9 …`,
  run against a closed port with the full 3D import.
  - Before the fix: no verdict; killed at the 60 s limit, with the last line "disconnected" → **hang reproduced**.
  - After the fix: `FAIL: not seated after 10 s`, `[probe] exit 1`, 20 s in total → **PASS** (named FAIL inside
    the probe's own bound).
  - The `[startup]` stages in the same log appear in setup order: world build ≈ 6.6 s and lighting ≈ 1.4 s
    headless.
- **Not done.** A tracked automated test for this case; it is a follow-up, `_reviews/pr548.md` finding 4.

### Bounded correction R2 — 2026-11-27 (perf lane root cause)

- **Finding** (perf lane, `perf-startup/target/perf-evidence/`). On a cold macOS Metal shader cache, Godot 4.7.2
  compiles every pipeline variant. The first frame drawn through the player's camera blocks about 117 s while the
  screen still reads "Preparing your character".
- **Fix.** Right after `_spawn_player()`, `_graphics_stage()` draws one frame with `disable_3d` set, showing
  `loading.graphics` (en and zh_Hans, plus the pot). 3D is then restored, so the stalled frame is entered with a
  truthful stage on screen. There is no percentage. The elapsed timer cannot advance inside the stalled frame;
  LOADING.md says so.
- **Evidence.**
  - `ci_gdparse.py clients/3d-spike`: PASS.
  - Headless closed-port probe (`probe-join-fail-graphics.log`): the new stage is printed in order after the
    character stage; named FAIL after 19 s, exit 1, so the probe wait stays bounded. PASS.
  - A windowed Metal cold-cache visual check that the new text is what stays on screen: NOT RUN in this lane.
