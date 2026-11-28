# Packaged launcher living-world correction

## DESIGN FROZEN — 2026-11-27 (operator ratification)

```text
Design revision:        revision 1 (2026-11-23, recovered as e37ce9fd); publication notes 2026-11-27
Approved by / evidence: the operator, 2026-11-27, relayed by the primary coordinator:
                        "启动器按推荐；启动加载和启动器设计冻结，接受慢 8 秒，3D 实跑放到里程碑"
                        ("launcher as recommended; startup-loading and launcher designs frozen; accept 8 s slower;
                        3D real runs go to the milestone"). The getrandom reuse was approved separately on 2026-11-23:
                        "同意复用现有 getrandom 库" (handoff-coordinator.md "November 23 continued execution").
Operator rulings:       D1 ratified as a bounded correction to S23/ARC-78 hosting.
                        D2 the exported-bundle World-tab check is DEFERRED to the milestone trial (and to
                           pr-launcher-solo.md C2's packaged item); it remains NOT RUN here, never counted as a pass.
Implementation base:    fix/startup-loading-ui (PR #548), on main after #544; branch fix/launcher-living-world (PR #549)
Lifecycle:              FROZEN (2026-11-27); implementation complete, under review
```

### Execution contract (filled; binding)

```text
Owner / worktree:       one session; /Users/yuema137/mineworld-worktrees/impl-launcher-living, branch fix/launcher-living-world
Affected paths:         tools/launch/{src/hosting.rs,src/children.rs,src/lib.rs,Cargo.toml,LAUNCHER.md,tests/**},
                        Cargo.lock, clients/shared/HOST.md, docs/decisions/ARC-78.md, this ledger
Authorized:             the correction as designed; bounded review fixes; targeted checks; semantic commits; push
Not authorized:         server/world/controller/schema/client changes; --solo/--hosted-here; dependencies beyond the
                        approved getrandom reuse; merge; milestone label
Validation owners:      cargo test -p mineworld-launch (real server built from the head, stub renderer); clippy -D
                        warnings; fmt; exported-bundle World-tab run (macOS); remote `fast`
Budgets:                <=600 s per heavy run, one shared slot (cast-slots/with-slot.sh), no paid API calls
Escalate:               any change to admission, smoke hosting, or token handling beyond this design
```

## Authorization and execution contract

Bounded correction to S23/ARC-78 runtime hosting, under the operator's 2026-11-23 actual trial request (residents stationary, no replies, day-length controls unavailable). Existing played-town policy is ARC-345; existing host capability is clients/shared/HOST.md. Coordinator dispatched a fresh execution lane; independent reviewer delivery_review accepts C1 as bounded integration. C2 dependency amendment explicitly approved by operator: “同意复用现有 getrandom 库”. This permits the existing workspace getrandom dependency for a private per-run admin token, replacing ARC-78's standard-library-only restriction only to this extent. It does not authorize other dependency, world, schema or rendering changes.

Replacement owner launcher_sol; isolated branch fix/launcher-living-world-sol in /tmp/mineworld-launcher-sol-20261123-source; prior tree preserved read-only; base 703aa8dce66c3fa150942315e74bffd346fcfa8a. Local code, targeted validation, independent review and semantic commit authorized by coordinator dispatch. No publication/merge/paid calls. Root owns shared progress and final bundle integration with startup_loading. User saves and prior bundle read-only. Budget <=600 seconds/run, <=3600 seconds initial aggregate, one coordinated heavy job. No hooks enabled.

## Source audit and scope

children.rs currently always passes --agent alice, strips admin token from server and leaves client inherited token untouched. Actual user server log has one reactive seat and no admin surface. Source 2D played launcher uses --town. Default DecisionList::none resolves rule, no model/network. Shared World tab already reads child MINEWORLD_ADMIN_TOKEN; day-length controls are 6/12/24. Neither simulation nor UI logic needs changing. Smoke fixtures require their existing explicit Alice. The Rust supervisor always stops its child after all clients exit, unlike shell --hosted-here/keep-running semantics.

Touched owners: tools/launch process setup/tests/specification; ARC-78 amendment; HOST.md clarification for packaged lifecycle; this ledger. No default world/seat/save/composition change. No startup/client code, public protocol, controller or system change. User expectations about home/sleep/shop/night remain outside this correction and must not be claimed complete.

## Design

Introduce a small private Hosting enum: Played holds a fresh random admin capability; Smoke holds no capability. Server configuration derives from this single mode: Played --town, Smoke --agent alice. Both retain loopback, generated invite, stdin-eof supervision. Hosting applies its capability to server and client environments, overwriting/removing inherited MINEWORLD_ADMIN_TOKEN without mutating parent environment. A played token is 16 OS-random bytes encoded as 32 hex characters through existing getrandom 0.3.4; random failure fails before launching children and names no secret. No Debug/Display derives on the secret owner. The server independently generates invite and rejects equality; do not reuse an invite as admin.

No --hosted-here, no Keep Running exposed, no --solo added. Existing clock presets need neither. In particular Both never gains fast-forward authority. All clients still must exit before server stdin closes. Existing exit semantics unchanged. Smoke retains no admin route and client token is explicitly removed.

## Commit and validation plan

One semantic correction commit plus review/evidence closeout if needed.
- [x] Implementation: Hosting mode, exact child configuration and token lifecycle; module/spec amendment; no controller/domain code.
- [x] Static: cargo fmt and targeted clippy/check for mineworld-launch; doc guards; exact diff audit.
- [x] Gate 2: existing launcher tests use real server with stub process solely replacing renderer; verify lifecycle/resume and smoke. Add actual played two-client test: real WebSocket observations show non-Alice movement; controlled nearby conversation elicits non-Alice reply; explicit seat release returns identity to controller; both closing order preserves server until last closes. Seed/config is deterministic fixture, not proof every arbitrary line is answered.
- [x] Gate 2: child stub consumes token in memory (never writes it), real HTTP missing/wrong token rejected; inherited token does not grant access; real day-length changes 6/12/24 and clock confirmations; pause/resume; journal scale survives restart. Verify token not in argv or logs in child memory before reporting only boolean outcomes. Snapshot/secret values never printed. Parent env unchanged by constructor. Both mode >24 rejected. Smoke token absent.
- [x] Independent review: exact diff and evidence, ownership, robustness, size/coupling, save/settings authority and no new API use.

Real exported UI confirmation belongs final integrated bundle with startup correction, explicitly pending until run. Gate 1 N/A: no real model behavior added. Full workspace/matrix NOT RUN: launcher-only targeted checks first, remote publication unrequested. Existing historical tests/claims are not complete living-world proof.

## Current checkpoint

Design written before implementation; C1 bounded implementation and C2 getrandom exception explicitly approved through coordinator dispatch. Prior owner completed implementation and tests. Replacement resumed the same approved correction in a fresh clone, preserved staged and unstaged patches, and completed remaining static/build checks; independent final source review passed; exact semantic commit identity is confirmed in the external handoff after commit.

## Live findings and evidence

- Independent pre-code plan PASS (delivery_review); C2 explicit getrandom approval relayed by coordinator before implementation.
- Hosting is the single played/smoke owner. Played no longer pins Alice; smoke explicitly strips inherited admin from clients as well as server. No hosted-here/solo flags added.
- Initial build PASS, 7.59 s. First real launcher gate PASS (~55 s), but reviewer found broad serialized-dialogue comparator could admit the player's own echo. Preserve that run at `/tmp/mineworld-launcher-living-20261123-evidence/living-1.log` as historical incomplete dialogue evidence, not final proof. Comparator now resolves Bob via display-name and requires a structured spoke fact with Bob as speaker, player as listener, and the original utterance quoted. Tightened rerun pending.
- Test world is a scratch Social Cafe copy with Bob's night destination kept in Cafe and Carol's changed to Cafe to make the required reply and post-release movement observable. This is real default rule-controller behavior under controlled content, not an assertion of the unmodified world's whole daily rhythm. Production world files unchanged.
- Server fixture executable is copied from reviewed playable package's universal `mineworld` at base703aa8dc (server production is unchanged). Launcher binaries/tests compile from this checkout. This is deliberate reuse, not a claimed fresh server build. Final integrated actual-renderer package remains coordinator-owned.
- Documentation heading and decision-id checks PASS; no full workspace or hosted CI run.

### Final targeted gate specification (before run)

Command: standard shared-slot wrapper runs `/tmp/mineworld-launcher-living-20261123-evidence/final-checks.py`, sequentially `cargo test -p mineworld-launch`, targeted all-target clippy, then release builds for aarch64-apple-darwin and x86_64-apple-darwin. Each command timeout600s; expected combined <150s. This is one owned heavy job, coordinated after startup visual measurement. Server fixture hash recorded in server-provenance.sha256. Tested source = base703aa8dc plus staged patch76609951c309a1b382fc6b2f7551ac29a07f2ec7b973450b875be3e094d62518; subsequent ledger text only does not alter executable/test content. Expected PASS: all crate tests and five real-launch lifecycle cases, including tightened structured non-Alice speech, post-release movement, clock/pause authority, denied wrong/missing tokens and non-solo48x, same-instance restart6x, both client lifetime; build produces portable launcher artifacts. Log/checks.json records actual time/status. Build results alone do not prove graphical integration.

Final targeted test run PASS22.145517s (7unit+5real-launch lifecycle cases), including the corrected structured Bob-reply comparator. Targeted clippy FAIL2.092842542s solely on collapsible_if in the child probe panic guard at tests/launcher.rs128. Release builds NOTRUN because wrapper stopped there. Coordinator requires modelreplacement now; preserveWIP and continue from handoff-launcher-living.md. No job remains active, shared slot released; no further edits/checks were launched by outgoingowner.


### Replacement closeout — 2026-11-23

The replacement session followed Structured Coding execute/resume routing and read the root operating instructions, primary ledger, launcher/host contracts and ARC-78 amendment. New branch is based on the identical703aa8dce66c3fa150942315e74bffd346fcfa8a revision. The previous checkout and original delivered bundle/saves were not modified. Its target cache was copied with filesystem cloning into this checkout before compilation; all resulting writes are owned by the replacement checkout.

The sole post-test code change collapses the nested probe panic guard at tests/launcher.rs128 into the equivalent Rust let-chain requested by clippy. cargo fmt -p mineworld-launch PASS. Existing 7unit +5real-launcher integration PASS22.145517s is retained from the prior exact executable/test content; no full test repeat solely for this equivalent lint correction. Replacement targeted cargo clippy -p mineworld-launch --all-targets -- -D warnings PASS1.206829083s. Release builds for aarch64-apple-darwin PASS1.161644667s and x86_64-apple-darwin PASS0.844727209s. Each log names the replacement source path and rebuilds mineworld-launch. Actual additional measured heavy-job runtime3.213200959s; known aggregate initial build7.59s + historical gate~55s + corrected tests22.145517s + failed clippy2.092843s + closeout3.213201s =~90.04s, within600s/run and3600s initial budget. No paid calls, public push, PR or merge.

Evidence: /tmp/mineworld-launcher-sol-20261123-evidence/checks.json and clippy.log/release-arm.log/release-x86.log. lipo confirms x86_64+arm64 on all three files in its universal/ directory. SHA256: mineworld-2d-launch b2496ee259d104bf59988066fcbc69524abd31c82296ef65239abf3285dbd70d; mineworld-3d-launch d1e64292db6b805f6a1476408ae1c2d959d34280a5343940c06e53c8f4713d42; mineworld-both-launch faf42b00bed64f34a9ec8a04735b0fb0dc1a4f1ae778dad57e9df706445fa2ec.

These launcher artifacts are ready for the coordinator's combined startup/launcher integration candidate. Actual exported-client settings and unmodified MarketTown behavior remain final-integration evidence, not a claim made by stub-renderer tests. Fullworkspace, remoteCI and mutation checks NOTRUN; no server, renderer, default world/controller, saved-world or schema changes. Parent coordinator owns integration, shared progress and subsequent operator delivery.


Independent final review: review_sol inspected the final intended diff, existing Hosting/process implementation and tightened structured dialogue test, new checks.json/logs, and diff whitespace. PASS with no code blocker; the sole new test change is an equivalent let-chain. Review distinguishes source approval from the coordinator-owned actual exported World-tab gate. Maintainability review: Hosting is a small single process-policy owner, no new cross-layer/domain dependency, production files stay scoped; the361-line living test owns one realistic launcher lifecycle scenario with bounded protocol helpers. No material new technical debt identified. Existing supervisor wait/error structure is unchanged. Source-only closeout is READY FOR COORDINATOR INTEGRATION; this is not remote CI, actual renderer acceptance, publication or merge readiness.

## Publication — 2026-11-27 (Claude takeover)

Approval basis: bounded correction requested by the operator's 2026-11-23 play feedback; independent plan PASS and
final source PASS (`handoff-coordinator.md` "November 23 continued execution"); operator approved the getrandom
reuse (ARC-78 amendment). The `/tmp` sources were lost on reboot; `e37ce9fd` is the byte-verified recovery of
`ff94a0ad` (via `d586754a`, patch-id `04bedb59`). Branch `fix/launcher-living-world` is `e37ce9fd` merged onto the
startup-UI branch `fix/startup-loading-ui` (itself PR #544 + `origin/main` `303c0df3`); no `tools/launch` or
workspace-manifest change on either side since `ff4e41e3`, so the merge is clean (only `Cargo.lock` auto-merged).

Local evidence on this head (`mineworld-worktrees/_targets/launcher-living-evidence/checks.log`), under the shared
slot: `cargo build -p mineworld-cli` (the server is built fresh from this head, not borrowed from a package) then
`cargo test -p mineworld-launch` — PASS, 7 unit + 5 real-launch cases, including
`played_town_and_private_clock_controls` with the tightened oracle (a structured `spoke` event whose speaker is Bob,
resolved by display name, whose listener is the player, quoting the original utterance; Carol moves after seat
release; 401 for missing/inherited token; 6/12/24 confirmed by clock frames; 48× refused 409; pause/resume; scale
survives restart; secret absent from argv and process logs). `cargo clippy -p mineworld-launch --all-targets -- -D
warnings` PASS; `cargo fmt -p mineworld-launch --check` PASS. Exported packaged UI check: NOT RUN. Full workspace:
NOT RUN (remote `fast` gates).

Scope reconciliation with later designs (recorded, not expanded): nightlife (`pr-nightlife.md` §1.3) P2 — per-run
admin token — is delivered here. P3 — packaged single-player worlds started `--solo` — is explicitly out of this
correction's scope (Design: "no --solo added") and remains new launcher work needing its own design and the
operator's product decision (unlocks >24× on solo). P1 — launcher Market Town for 3D/both with matching depiction —
is also not here (3D and both still default to `social-cafe`, LAUNCHER.md §2). Dialogue PR-B's `--decisions` and
cognition-child seams build on this `Hosting` shape and are PR-B's work.

### Red-team review disposition — 2026-11-27 (`_reviews/pr549.md`, PASS-WITH-FOLLOWUPS)

- Finding 6 (exported World tab unverified): **NOT RUN** on this head. No exported bundle was assembled from it. The
  stub-renderer test proves the server and token path, not the shipped Godot `host_clock.gd` reading the token.
  Open as operator decision D2; planned as `pr-launcher-solo.md` C2's packaged World-tab item.
- Finding 4 (approval quote undated): date and place recorded in the header above.
- Finding 2 (source 3D launcher `mineworld-slice` still adds `--agent alice` in played mode): pre-existing and out
  of this packaged-only scope; follow-up.
- Finding 8 (`configure_server` calling `configure_client`; `tests/launcher.rs` at 839 lines): follow-up, no
  behaviour risk.
