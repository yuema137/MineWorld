# Playable dialogue and persistent contextual memory — PR design

## DESIGN FROZEN

Design revision: DM-20261127-3 (frozen sections: §0 rulings, §1, §3, §4, §5 goals/scope/acceptance, §6; live
sections: §5 checkboxes and evidence, §7 status column, §9, §10).
Approved by / evidence: operator, 2026-11-27, relayed by the primary coordinator: "同意三个都按推荐冻结"
("agree, freeze all three as recommended"), covering PR #535 (this design), #536 and #537; every question
Q-D1…Q-D5 answered as recommended. Pre-freeze red-team review
`/Users/yuema137/mineworld-worktrees/_reviews/design-dialogue.md` (verdict FREEZE-AFTER-FIXES) applied as bounded
corrections in DM-20261127-3 (§9); none changes an approved answer.
Implementation base: PR-A (C1) — `origin/main` @ `8c971f0f7b469f05d1bf6f21fc933501eb22c404`. PR-B (C2–C5) —
**dependent on the recovered client integration stack**: `recovered/integration-d586754a` @ `e37ce9fd` exists
locally (not yet on `origin`, not merged). PR-B starts only after that stack and PR-A have merged to `main`, and
its base is the exact `main` SHA recorded then (§0 Q-D4).
Execution contract: [`pr-playable-dialogue-memory-contract.md`](pr-playable-dialogue-memory-contract.md)
Lifecycle: FROZEN

Parent requirement: [`operator-playable-mvp-20261123.md`](operator-playable-mvp-20261123.md) §§1–4, check 2: "a
player speaks, receives a relevant response, and later can verify a remembered prior exchange after
leaving/rejoining and after save/restart". This file is the step design and the PR design for both PRs of this
lane (PR-A = C1, PR-B = C2–C5); it is the one authority for the lane (CLAUDE.md §3.2), and each PR keeps its own
ledger section in §10. Owners: planning — primary coordinator; drafting — design lane L3; execution — one fresh
delegated session per PR; independent red-team — the coordinator's assigned reviewer.

## 0. Operator rulings (2026-11-27) and bounded resolutions

| # | Question | Ruling (operator, 2026-11-27: as recommended) |
| --- | --- | --- |
| **Q-D1** | Which Python runtime runs the talking resident? | **R1.** The launcher uses a locally installed Python ≥ 3.12 environment holding the project's locked cognition packages and talks to local Ollama `qwen3.5:4b`. Missing pieces ⇒ the game starts with rule-driven residents and a message. R2 (private Python inside the app) later, as its own prototype PR. R1 is labelled "requires local runtime", never "self-contained". |
| **Q-D2** | Who talks, who plays? | **Market Town.** Alice Moreau (`alice`) on the LM; the operator plays Vera Lindgren (`visitor`); two-window mode is 2D `visitor` + 3D Wes Calloway (`wanderer`); everyone else on rules. |
| **Q-D3** | May the final check use the local model? | **Yes**, one serial local session: ≤ 8 model calls total, ≤ 180 s inference, ≤ 20 s per reply, ≤ 256 output tokens per reply, 5 scripted player turns, each real-process run ≤ 10 min, ≤ 60 min for all real-process runs, one heavy-job slot, $0. All other tests use scripted/recorded answers. |
| **Q-D4** | Which code does the work start from? | **C1 as its own PR on `main` now; C2–C5 on the recovered integration stack.** |
| **Q-D5** | How far may the implementer go? | **Commit, push, open/update the PR, repair CI up to READY FOR OPERATOR REVIEW. Never merge.** |

**Corrected facts under the approved answers (no answer changes).**

- *Q-D4.* When asked, the design said the integration branch did not exist. It now exists locally at
  `e37ce9fd` (six recovered commits on `003d3543`: S34-e `59b442cc`, `301e3c28`, `eacb816d`, `ff4e41e3`,
  startup `bae264be`, launcher `e37ce9fd`). The approved answer still holds; the merge order it implies is
  explicit: **client-stack recovery PR(s) → `main`; PR-A (C1) → `main`; PR-B (C2–C5) branches from the `main` SHA
  that contains both and targets `main`.** PR-B never stacks on an unmerged branch. Launcher seams were re-audited
  on `e37ce9fd` (§2) and are re-checked on PR-B's actual base before C2.
- *Q-D1 provisioning, in plain words.* "Installed" means: in a MineWorld source checkout on this Mac, run
  `uv sync --locked`, then the cognition command that writes a runtime manifest (§3.4.2); put that environment's
  interpreter path and the manifest path in your cognition profile (§3.3). If the checkout is later updated
  (`git pull`, `uv sync`), the manifest no longer matches and the app falls back to rule-only with a message until
  you regenerate the manifest. System Python alone is not enough.
- *Q-D3 consequences, in plain words.* The local 4B model may still fail to use the correction (it ignored prior
  lines once before: F-P7-5, recorded in `pr-p6c-fgd-memory-coherence.md` §1). PR-B may therefore reach READY FOR
  OPERATOR REVIEW with the live gate recorded as FAIL or INCONCLUSIVE. In that case the PR body and the progress
  page say **"check 2 NOT MET"**, nothing claims conversation works, and the lane stops for the operator to choose
  one of: a larger local model, a reviewed prompt/recall amendment, or accepting rule-only conversation for now.
  No cap is raised and no model is swapped without that choice.

**Bounded resolutions recorded at freeze** (not among Q-D1…Q-D5; resolved by the conservative option that
changes no existing public contract; the operator may overrule any of them, which is then a recorded amendment):

- **B1 — `--fresh` and memories.** `--fresh` already deletes `saves/<world>/` (LAUNCHER.md §5). It now also
  deletes that world's cognition store `cognition/<world>/` in the same per-user folder, so no private
  conversation outlives the world it belongs to and on-disk growth stays bounded by the live save. Old memories
  never cross into a fresh world. (The DM-2 draft kept them indefinitely; reverted per review F-4.)
- **B2 — Where conversation is switched on.** The cognition profile lives in the per-user folder
  (`cognition/profile.toml`, beside `saves/`), which survives package upgrades and read-only bundles. The package
  ships only an inactive commented template. No profile file ⇒ conversation off, rule-only. Delete the file to
  switch it off again (review F-5).
- **B3 — Platforms.** The new launcher child-lifecycle code compiles and is tested on macOS, Linux and Windows
  in CI (scripted backend). The live gate and the R1 delivery are this Mac only; Windows/Linux live play is N/A
  for this PR because the operator's R1 runtime exists only here (review F-6).

What the operator gets when PR-B is done: open the app, walk to Alice, tell her something, leave, come back,
change one detail, quit, reopen the same save, and she answers with the updated detail and the unchanged
original detail. Other residents keep walking and trading while the model thinks or is off. With no model the
world still runs.

## 1. Delivery outcome and scope

The same authoritative town supports headless, 2D, 3D and outside-agent (MCP) clients. One explicitly selected
resident is driven by the existing LM Controller and can be taken over by rule or human controllers, keeping
Person identity and authoritative history across release/rejoin. The player has a relevant exchange, corrects
an earlier fact, closes the application, and after reopening the same save receives an answer that uses the
corrected fact and the retained earlier detail. Other residents keep acting during slow/absent inference. The
launcher owns cognition lifetime and uses one per-user cognition store per world that persists across restarts
and package upgrades and is deleted with the save by `--fresh` (B1).

Reuse `SeatRunner`, `Router`, `SeatHomes`, `mineworld_sdk`, existing memory/retry/breaker/scope entry points,
rule fallback (`took_over: "hosted"`) and the OpenAI-compatible backend. Memory-framework reuse is already
decided by `ARC-59` and `DEP-58` (Letta, mem0, Zep, Graphiti, LangGraph memory declined; SQLite home); this lane
does not reopen it. Not in scope: a new cognition framework, database, provider SDK, kernel Person field, client
game logic, model-service launcher, installer, paid/cloud provider, full-town LM rollout, backup/restore feature,
atomic world+cognition snapshot, hot memory-policy migration, semantic truth checker, observer privileges for
controllers, or any scale claim. Models may be absent and renderers detached; rule-only validity is required,
and rule template replies are never accepted as contextual LM conversation.

Not closed by this lane: shopping/nutrition, shop hours, residences, sleep advance, localization, and the
self-contained runtime (R2).

**LM-concept boundary (CLAUDE.md §4.3).** Provider endpoint, model name, tokens and tiers stay inside the
cognition Controller Pack's operator configuration (`CognitionConfig`) and gateway. The launcher handles only
opaque paths and the versioned status line of §3.4.1, whose `reason` is a closed enum carrying no endpoint,
model or token data. Nothing here adds an LM concept to a kernel, System, protocol or World Pack contract.

## 2. Source audit

Audit base: `origin/main` @ `8c971f0f` (#524). The DM-2 audit was on `7718d952`; `git diff --stat 7718d952
8c971f0f -- cognition tools/launch scripts/package.py worlds/market-town sdk server` touches none of the seams
below (only `clients/3d-spike` and docs; MODULE_SPEC line numbers shifted). The recovered draft's base
`b457a94d` is also seam-identical. Launcher seams were additionally audited on `recovered/integration-d586754a`
@ `e37ce9fd`. Prior evidence is preserved byte-exact in
[`dialogue-memory-evidence/integration-audit-20261123.md`](dialogue-memory-evidence/integration-audit-20261123.md)
(SHA-256 `73b73fef…`) and
[`dialogue-memory-evidence/recovered-draft-DM-20261123-1.md`](dialogue-memory-evidence/recovered-draft-DM-20261123-1.md)
(SHA-256 `4ec9ae42…`).

| Concern | Verified | Design consequence |
| --- | --- | --- |
| Launch chain (main) | `lib.rs::run` → `children.rs::Server::start` → join → clients → `Server::stop`; `children.rs:59` passes `--agent alice --stop-on-stdin-eof`; no cognition child | Superseded by the integration stack below |
| Launch chain (`e37ce9fd`) | New `hosting.rs`: `Hosting::Played{admin}` / `Smoke`. `configure_server` adds `--town` (played) or `--agent alice` (smoke), then calls `configure_client`, which **sets** `MINEWORLD_ADMIN_TOKEN` for played runs. Server start removes `MINEWORLD_INVITE`, uses `CREATE_NO_WINDOW` on Windows, no `--decisions` | PR-B adds `--decisions` to played hosting; the cognition child **never** goes through `configure_client`/`configure_server`; its environment is built by a dedicated function that removes `MINEWORLD_ADMIN_TOKEN` and sets the invite variable (§4.11) |
| Join | `join.rs` reads only address and invite from the server's join line | The launcher cannot see the world instance; instance binding is cognition's (§3.4.4) |
| Launch profile | `config.rs`: strict string-only TOML subset (`ARC-78`), keys `world`, `seat`, `seat-2d`, `seat-3d`; defaults 2D `carol`/market-town, 3D `visitor`/social-cafe, both `visitor`+`wanderer` | Cognition profile reuses this parser in the per-user folder (B2); the profile sets the mode seats of Q-D2 |
| Per-user folder | LAUNCHER.md §5: `saves/<world>/`, `logs/` (five runs kept), `scratch/<run>/`; bundle may be read-only; `--fresh` deletes the save | Add `cognition/` beside `saves/` (B1, B2) |
| Assignment | `mineworld decisions <world> --decisions FILE --json` → `controller/assignment.py::read_assignment` (`assigned == "lm"`) | Server and resolver get the same decisions file |
| Cognition entry | `__main__.py` (181 lines) accepts only `--config` (l.44); one `SeatHomes`, one `SeatRunner` per LM seat; no world handle | Supervised flags in a separate module (§3.4.3) |
| Deadline authority | `docs/reference/cognition-toml.md` l.83: `consult_deadline_s` = "Wall seconds from a trigger's receipt to the submit" | The code contradicts its spec (CLAUDE.md §2.1 rule 4): code is wrong |
| Deadline defect | `triggers.py::Pending.offer` (l.214) stores `received_at`; `seat.py::_consults` (l.537–543) only discards stale triggers; `_consult_once` (l.562, l.627) passes a fresh `deadline_s`; `consult.py::consult` starts its own clock (l.149); `_submit` takes `_submit_lock` (l.487) with no recheck | C1 |
| Gateway defect | `gateway.py::ModelGateway.complete` takes `now` before `async with budget.limiter`; `asyncio.timeout` wraps only the backend; `CancelledError` releases the reservation and charges nothing. `ARC-57` fixes this order and its 2026-10-14 note accepts that outside cancellation skips the charge | C1 changes both ⇒ dated amendment to `ARC-57` (+ `ARC-73` cross-reference) in C1 scope |
| Size debt | `config.py` 732, `seat.py` 765, `scripts/package.py` 1309 lines (existing) | §4.14 commitments |
| Memory home | `homes.py::home_path` → `<store>/<instance>/memory.sqlite` (`home`); `SCHEMA_VERSION = 6` | Store = `cognition/<world>/` |
| Recall | `memory/retrieve.py::_with` (l.175), `_recent` (l.323); `validate.py` rejects forged recall tokens, not false prose | Citations necessary, not sufficient; rubric in §6 |
| Triggers | `triggers.py`: T1 spoken-to; T3 remembered person entered my place (once per counterpart per `greet_cooldown_s`; the greeted set is process memory); `upcoming` within `horizon_minutes` (≥ 15) of a punctual part, re-asked every `reask_minutes` | §6 pins a window with no punctual part ahead |
| Alice | `worlds/market-town/people/alice.yaml`: café 05:30 (punctual), park 14:00, apartments 18:00; Market Town `time_scale: 12` | Gate window 07:00–13:00 world time, Alice at the café counter |
| Output bound | `controller/prompt.py:32` `MAX_OUTPUT_TOKENS = 256` (existing) | Q-D3's 256 is the existing constant; no change |
| Budget keys | `[budgets] calls_per_wall_hour`, `tokens_per_wall_day`, `ledger` (SQLite, restart-safe) | Gate cap enforced by a gate-specific ledger (§6) |
| Sample config | `examples/local.toml` l.13 "format 1", l.21 "FG-c is not merged" | Stale; fixed in C2 |
| Spec drift | `docs/MODULE_SPEC.md` §5 LM Controller bullet: "It never chooses movement"; `seat.py` runs `mineworld_sdk.pacer` walking | Stale sentence; fixed in C2 |
| Package | `scripts/package.py::layout` (l.599) ships no Python/cognition | R1 ships only the inactive template and the decisions file |
| Seats | `worlds/market-town/world.yaml` `seats:` l.228: `visitor`, `wanderer`, `alice`, … | Q-D2 profile is real |
| MCP | `clients/agent/src/mineworld_agent/mcp_server.py` | C4 deterministic MCP case |
| Tests | `tools/cli/tests/milestone_d.rs::she_remembers_across_the_two_views` (`#[ignore]`, python layer), `record_the_milestone_d_cassette` (operator-run); `speech_both_ways.rs`; cognition `test_budget.py` (two named gateway tests), `test_controller_{seat,consult,homes}.py`, `test_home_isolation.py`, `test_memory_isolation.py`, `test_milestone_d_{replay,fallback}.py`, `milestone_d_cognition.py` | Extend; no parallel harness |
| Local model | `ollama list`: `qwen3.5:4b` (`7f3251fa6878`, 4.0 GB). Not run | Size proves nothing about latency or headroom |

Historical Milestone D evidence (scripted earliest-line quotation; the 2026-10 local qwen recording,
`pr-p6c-fgd-memory-coherence.md` §9, AP6c-3) is fixture material, not acceptance.

## 3. Runtime route, profile and new contracts

### 3.1 R1 (Q-D1)

The launcher runs the profile's interpreter directly (no shell, no `uv run`) as `<python> -I -m
mineworld_cognition --supervised …`. Before that it runs the same interpreter with `--preflight` (§3.4.3): Python
≥ 3.12; installed distributions match the runtime manifest (§3.4.2), including installed source-file hashes;
the `mineworld.forgetting`, `.retry`, `.breaker`, `.memory_scopes` entry-point groups resolve; the configured
backend URL is loopback. Any failure ⇒ degraded rule-only play with a named reason. Never an implicit fallback
interpreter, never a conversation PASS, never a self-contained claim. No Python dependency enters the Rust build.
The launcher never downloads a model or runtime and never starts or stops Ollama.

One new decision record (allocated under `docs/DECISIONS.md` §3 in C2) records the R1 route and names the R2
candidates to prototype later (python-build-standalone relocatable CPython + locked closure; PyInstaller;
Briefcase), with provenance, redistribution, relocatability, native-wheel, universal-macOS and size questions
left open for that prototype. It cites `ARC-59`/`DEP-58` for memory instead of re-deciding them.

### 3.2 Profile (Q-D2)

Market Town for all three modes. LM seat `alice`; human `visitor` in 2D and 3D solo modes; both-mode 2D
`visitor`, 3D `wanderer`; all other seats rule. EntityIds come from the world/welcome, never hardcoded.
Same-player recall across clients: release `visitor` from one client and reacquire from the other. `wanderer` is
a distinct Person and never inherits `visitor`'s private exchanges. Social Café fixtures stay deterministic test
fixtures only.

### 3.3 Configuration and data flow

`<user-dir>/cognition/profile.toml` (B2) uses the existing `launch.toml` grammar and parser (`ARC-78`) with
per-mode sections `[2d]`, `[3d]`, `[both]`, each holding `world`, the seat keys, and four cognition keys that are
all present or all absent (partial ⇒ named configuration failure): `cognition-python` (absolute interpreter),
`cognition-config` (absolute operator-owned `CognitionConfig` TOML), `cognition-manifest` (absolute runtime
manifest), `cognition-decisions` (bundle-relative decisions file; the bundle ships
`runtime/decisions/market-town-alice-lm.yaml`). When the profile exists, its mode section replaces that mode's
`launch.toml` section. The package ships `runtime/cognition-profile.template.toml`, fully commented, never read.
Smoke runs ignore the profile.

Flow: the launcher reads the profile; runs `--preflight`; on success starts the town server with `--town
--decisions <file>`; runs the owned server binary as a bounded short child to emit the format-3 assignment for the
same world and decisions file into `cognition/<world>/run/assignment.json`; rejects an LM seat that is also a
human seat of the mode; starts the cognition child (§4.11). Python `read_assignment` validates exactly one LM seat.
In supervised mode the provider configuration cannot override server URL, store or assignment.

### 3.4 New contracts (shapes fixed at freeze)

**Owning specifications.** The cognition side is specified in a new `docs/reference/cognition-supervised.md`
(flags, status line, manifest, store layout, instance binding), linked from the LM Controller bullet of
`docs/MODULE_SPEC.md` §5 and from `docs/reference/cognition-toml.md`. The launcher side is specified in
`tools/launch/LAUNCHER.md` (new §§ on the cognition profile, the cognition child and `--fresh`). Both are
written in C2 (shapes) and C3 (lifecycle) before or with the code.

#### 3.4.1 Status line (cognition → launcher)

In `--supervised` mode the child's **stdout carries only status lines**; every other output goes to stderr
(captured to the run's cognition log). One JSON object per line, UTF-8, ≤ 512 bytes:

```json
{"status": 1, "state": "starting", "seat": "alice", "reason": null}
```

- `status`: integer format version, `1`. An unknown version or malformed line ⇒ launcher state
  `degraded(status_invalid)` and the child is stopped.
- `state`: one of `starting`, `ready`, `degraded`, `stopped`. `ready` means the seat's welcome was received and
  its memory home opened; nothing earlier.
- `seat`: the LM seat key, or `null` before assignment is read.
- `reason`: `null` unless `state` is `degraded` or `stopped`; then exactly one of the closed set
  `config_invalid`, `assignment_invalid`, `runtime_mismatch`, `backend_not_loopback`, `home_busy`,
  `home_identity_mismatch`, `instance_mismatch`, `server_unreachable`, `seat_unavailable`, `model_unavailable`,
  `stopping`. Launcher-generated reasons (never sent by the child): `readiness_timeout`, `child_exited`,
  `status_invalid`, `preflight_failed`, `profile_invalid`.
- No other field. No endpoint, model, token, invite or path ever appears.

#### 3.4.2 Runtime manifest

File chosen by the operator (`cognition-manifest`), written by `python -m mineworld_cognition
--write-runtime-manifest PATH` run with the checkout's interpreter. JSON:

```json
{"manifest": 1, "python": "3.12.7", "source_revision": "<40-hex git sha>", "lock_sha256": "<uv.lock sha256>",
 "distributions": [{"name": "mineworld-cognition", "version": "0.1.0",
                    "files": [{"path": "mineworld_cognition/gateway.py", "sha256": "<hex>"}]}],
 "entry_point_groups": ["mineworld.forgetting", "mineworld.retry", "mineworld.breaker", "mineworld.memory_scopes"]}
```

`distributions` covers `mineworld-cognition` and `mineworld-sdk` source files as imported (editable installs
resolve to their real files). Third-party distributions are checked by name+version against the lock. Owner:
cognition. `--preflight` recomputes and compares; any difference ⇒ `runtime_mismatch`.

#### 3.4.3 Supervisor command-line flags

Implemented in a new module (`mineworld_cognition/supervised.py`), not in `config.py` (§4.14):

| Flag | Value | Rule |
| --- | --- | --- |
| `--config FILE` | existing | still required |
| `--supervised` | — | stdin EOF ⇒ Stopper; stdout = status lines only |
| `--server-url URL` | `ws://127.0.0.1:<port>/ws` or `ws://[::1]:<port>/ws` | supervised only; loopback only; overrides top-level `server` |
| `--store-dir DIR` | absolute | supervised only; `cognition/<world>/`; overrides `store` |
| `--assignment FILE` | absolute | supervised only; format-3 JSON; overrides `decisions_json` |
| `--preflight MANIFEST` | absolute | checks §3.1 and exits 0 / named non-zero; prints one status line |
| `--write-runtime-manifest PATH` | absolute | writes §3.4.2 and exits |

The overrides are applied as typed fields through `CognitionConfig` (pydantic `model_copy(update=…)` after
validation), never by text concatenation. Without `--supervised` the CLI behaves exactly as today. The invite is
read from the variable named in the config (`MINEWORLD_INVITE` in the template); never a flag.

#### 3.4.4 Store layout and instance binding

`<user-dir>/cognition/<world>/` is the store: `<instance>/memory.sqlite` (existing `home_path`), resume and
decision-log files (existing names), `budget.sqlite` (the ledger), `run/assignment.json`, and `instance`
(text: the world-instance id). Owner of the directory's lifecycle: the launcher (create; delete with `--fresh`,
B1). Owner of the instance binding: **cognition** — on the first welcome it writes `instance` if absent; on a
later welcome with a different instance it refuses with `instance_mismatch`, touches nothing, and exits.
Never two writers (existing home lock ⇒ `home_busy`). Smoke runs use `scratch/<run>/cognition/`, removed with the
scratch.

Restore caveat (review F-13): MineWorld has no backup/restore feature, and this PR adds none. Restoring an older
copy of a save by hand while keeping a newer store can leave the store ahead of the world; this is documented as
unsupported in LAUNCHER.md, and a test proves that an `instance` mismatch is refused and never silently erased.
Same-instance rollback detection is out of scope.

## 4. Invariants (frozen)

1. Server/System ownership unchanged. Controllers see only permitted Observations and perceived streams; every
   change is an offered `ActionIntent` revalidated by the server. Renderers grant no perception.
2. Rule, human, MCP-agent and LM takeover preserve Person EntityId, inventory, holdings, relations and history.
   `take_over=False` never steals a human seat. Cognition stops submitting after seat loss. Re-entry rebuilds only
   what that Person legitimately perceived during absence; no transfer of human thoughts or model internals.
3. One process / Router / SeatHomes owns all LM seats of a run; this profile selects exactly one. Shared storage
   is not shared knowledge. No observer/global channel enters a Controller prompt.
4. **Absolute deadline** (authority: `cognition-toml.md` l.83). At `Pending.offer` receipt R, expiry
   E = R + `consult_deadline_s`. Pending wait, first-Observation wait, preparation, gateway admission (limiter),
   HTTP, every retry delay, `_submit_lock` acquisition and dispatch spend this one budget; no stage resets E. No
   backend request or protocol submit begins at or after E; the check repeats after acquiring `_submit_lock`,
   immediately before the socket send.
5. A consult ends by E plus a measured scheduler allowance. Expired pending triggers are counted and discarded
   without model work. Existing Pending supersession stays; no promise to answer each line of a burst.
6. A dispatch sent before E may be acknowledged after E: an uncertain acknowledgement, never auto-resubmitted;
   reconciled via SDK history. Acceptance proves no new dispatch after E and no duplicate action.
7. Gateway order becomes: key-material check → budget pre-check → (timeout starts) in-flight slot → recorder →
   backend → charge. Every exit releases exactly what was acquired. A request that never reached the backend
   costs no call; a started backend call costs exactly one call on success, failure, timeout or cancellation
   (actual usage, else the existing failure-token convention), charged at invocation time. Ledger never reset on
   restart. `CancelledError` propagates after cleanup; `CassetteMiss` stays loud. Recorded as a dated `ARC-57`
   note.
8. Store per world in the per-user folder (§3.4.4); retained across restart and package upgrade; deleted with the
   save by `--fresh` (B1); instance binding owned by cognition; never two writers.
9. No backup/restore promise; instance mismatch refused, never silently erased (§3.4.4).
10. Launcher states `disabled`, `starting`, `ready`, `degraded(reason)`, `stopped` follow §3.4.1. Spawn or endpoint
    liveness is never `ready`. World and clients never wait for inference. Readiness timeout (bounded, launcher
    side) stops the child and leaves hosted rule fallback.
11. The launcher is the single shutdown owner and builds the cognition child's command and environment itself:
    direct interpreter, explicit argument list, `MINEWORLD_ADMIN_TOKEN` removed, invite set in the configured
    variable, never via `Hosting::configure_client`/`configure_server`; on Windows `CREATE_NO_WINDOW` and a
    `python.exe` path check. Final client exit or launch error closes cognition stdin (stdin EOF works the same on
    Windows); the Stopper cancels work, leaves seats (5 s), exits within 15 s; then server stdin closes (10 s).
    First exit of two clients preserves the other. Launcher death gives both children EOF. Only owned child
    handles are killed/reaped (no process groups, no process-name search); Ollama is never stopped.
12. Model absence, budget exhaustion, open breaker and malformed or slow answers never block ticks, other residents
    or saving. Rule handback need not answer the timed-out line; no fabricated reply or recall. No hidden cloud
    tier, retry or install. Non-social tiers `none`; retention `consolidate`. Time scale does not change wall
    budgets.
13. An outside agent holding `visitor` through `mineworld_agent` (MCP) talks to LM-Alice through the same talk
    intent and server path and is remembered under the same access rules as a 2D/3D player.
14. Size: deadline logic in a new cohesive `controller/deadline.py`; `seat.py` has no net line growth across C1+C3;
    supervised flags in `supervised.py`, `config.py` no net growth; `scripts/package.py` (1309 lines) growth in C5 is
    recorded as existing debt with its delta, no new responsibilities beyond shipping two files.

## 5. Ordered semantic commits

Each commit carries its tests and a §10 ledger update. All boxes are open. `shuffle`/`file_order`/training:
N/A (no such interface in this lane). R-LOAD-1: targeted local tests only; CI on the exact head is the gate.

### C1 — absolute deadline end to end (PR-A, base `main` @ `8c971f0f`)

**Goal.** Make the code meet `cognition-toml.md` l.83 before cognition joins the packaged lifecycle.

**Scope.** New `controller/deadline.py`; `controller/triggers.py`; `controller/seat.py::_consults/_consult_once/
_submit`; `controller/consult.py::ConsultLimits/consult`; `gateway.py::ModelGateway.complete`; targeted tests in
`test_budget.py`, `test_controller_consult.py`, `test_controller_seat.py`; dated note on `docs/decisions/ARC-57.md`
with an `ARC-73` cross-reference; affected cognition docs. No retry redesign, no default budget change, no
launcher change.

**Implementation.**
- [ ] One absolute expiry from `Pending` receipt through the caller chain; remove the consult-start reset.
- [ ] Gateway admission+backend and consult awaitables bounded by the remaining budget; expiry recheck at backend
  invocation and after `_submit_lock`; normalized `deadline` terminal record.
- [ ] Release on every exit; charge exactly the started backend calls, including cancellation, at invocation time;
  `CancelledError` not swallowed.
- [ ] `ARC-57` dated note (new order, cancellation charge, source: this design + QP6-6 intent).

**Validation.**
- [ ] Two seats, `max_in_flight=1`, first backend blocked: second expires in the queue without invoking the
  backend; a third obtains the freed slot.
- [ ] Pending wait and late first Observation consume the deadline; a backend finishing after E cannot submit;
  retry delay and `_submit_lock` wait cannot dispatch late.
- [ ] Cancel before admission vs. during backend: correct accounting, nothing held, no duplicate action after an
  uncertain acknowledgement.
- [ ] Existing consult/retry/frame-rejection/pacer tests green; failing-before-fix run captured.

**Acceptance.** Captured timestamps show no backend start or submit at ≥ E; no fresh deadline after
pending/Observation wait; slot and budget usable after every exit; one charge per started call; `seat.py` no net
growth.

**Failure/edges.** Expired trigger → discard and count; gateway timeout → existing fallback/handback; socket loss →
`disconnected`; caller stop → cancel and clean; malformed text → existing bounded content failure; no reask.

**Commands.** `uv run --locked pytest cognition/lm-controller/tests/test_budget.py
cognition/lm-controller/tests/test_controller_consult.py cognition/lm-controller/tests/test_controller_seat.py`;
ruff format/check; strict pyright on the cognition member. NOT RUN.

**Boundary/review.**
- [ ] Independent logic review of every gateway caller (including the summary gateway), charge paths,
  timeout-vs-sent uncertainty, stop/pacer locks, size commitments.
- [ ] Staged-diff inspection; commit only this correction, its tests and the ARC-57 note.

### C2 — profile, store and contracts (PR-B, base: `main` containing the client stack and PR-A)

**Goal.** One validated operator profile, one store per world, and the §3.4 contracts written down.

**Scope.** `tools/launch/src/{config,bundle,user}.rs` (paths re-audited on PR-B's base), `tools/launch/LAUNCHER.md`
(profile, store, `--fresh` text per B1, restore caveat); `docs/reference/cognition-supervised.md` (new);
MODULE_SPEC §5 LM Controller bullet (movement sentence + link); `cognition-toml.md` link;
`cognition/lm-controller/examples/local.toml` stale comments; `mineworld_cognition/supervised.py` (flags,
preflight, manifest writer); `runtime/cognition-profile.template.toml`; `runtime/decisions/market-town-alice-lm.yaml`;
the R1 decision record. No new TOML parser, no shell command interface.

**Implementation.**
- [ ] Profile read from the per-user folder with the existing parser; partial/unknown/duplicate keys fail by name.
- [ ] Assignment resolution with the same decisions file as the server; LM seat ≠ human seat; exactly one LM seat.
- [ ] Store creation, `--fresh` deletion of `saves/<world>/` and `cognition/<world>/`, smoke scratch store;
  cognition-side `instance` binding.
- [ ] `--preflight`, `--write-runtime-manifest` and supervised overrides in `supervised.py`; loopback check.
- [ ] Specs (§3.4) and decision record written with the code.

**Validation.**
- [ ] No profile ⇒ behaviour identical to today; partial keys, invalid seat, stale manifest, non-loopback backend,
  wrong world/format, missing interpreter each give their named outcome.
- [ ] Store tests: restart reuse, `--fresh` deletes both, smoke isolation, read-only bundle, upgrade keeps
  per-user files, log pruning never touches `cognition/`, instance mismatch refused without modification.

**Acceptance.** Server and resolver agree; resident selection checked; no secret on disk or argv; missing runtime ⇒
actionable degraded rule-only launch; specs match code.

**Failure/edges.** Busy home or identity/schema mismatch fails cognition by name with the store untouched;
malformed profile fails before any child starts.

**Commands.** `cargo test -p mineworld-launch` selected cases (one heavy slot); cognition config/assignment/home/
supervised tests; doc link and decision-ID checks. CI runs all three OSes. NOT RUN.

**Boundary/review.**
- [ ] Review: path escapes, save/store collision, writer isolation, grammar reuse, secret flow, spec/code match.
- [ ] Commit configuration, specs and tests together; no child supervision before C3.

### C3 — supervise one cognition child

**Goal.** Cognition reachable from packaged play and stopped with its world, including launcher death and
two-window lifetime, on all three OSes.

**Scope.** `tools/launch/src/{lib,children}.rs` plus a cohesive `cognition.rs` child module;
`mineworld_cognition/{__main__,shutdown,supervised}.py`; readiness hook in `seat.py` (no net growth);
`tools/launch/tests/launcher/` real-process fixtures. No server/client domain change; no model-service ownership.

**Implementation.**
- [ ] Start town server with `--decisions`, resolve assignment, start the child per §4.11; own stdin and handle.
- [ ] `--supervised` EOF ⇒ Stopper; status lines per §3.4.1; standalone CLI unchanged.
- [ ] Bounded status reading during play; unexpected exit ⇒ `degraded(child_exited)`, world continues.
- [ ] Ordered shutdown on final client exit and error/drop paths; kill/reap only the owned child after the bound.

**Validation.**
- [ ] Real child (scripted backend): `ready` after welcome+home; unreachable endpoint; malformed answer; killed
  child; launch failure after server start; shutdown during an in-flight request; malformed status line.
- [ ] Two clients: first closes, second still acts; last closes, server persists, all owned children reaped.
  Killing the launcher stops server and cognition via EOF.
- [ ] Other residents act during blocked inference; no cloud attempt; child environment has no
  `MINEWORLD_ADMIN_TOKEN`; a planted invite never appears in the **cognition child's** argv, stderr log, status
  lines or store files (the server log and client argv hold it by existing design, LAUNCHER.md).
- [ ] Same lifecycle tests on Windows and Linux in CI (B3).

**Acceptance.** Direct interpreter child; status reflects held seat/home; no orphaned owned child; only owned
processes stopped; world responsive under model failure.

**Failure/edges.** Human-held seat never taken; busy store refuses a second writer; readiness expiry ⇒ degraded;
HTTP cancellation may not stop Ollama GPU work: measured and reported, never claimed.

**Commands.** Targeted launcher real-process fixtures; cognition entry/shutdown tests; `speech_both_ways` and
Milestone D protocol path where applicable. Scripted backend only; ≤ 10 min, one heavy slot. NOT RUN.

**Boundary/review.**
- [ ] Red-team: EOF inheritance, kill/reap, status bounds, stdin ownership, drop order, partial-launch exits,
  environment construction, Windows behaviour.
- [ ] Commit lifecycle atomically.

### C4 — corrected memory and controller substitution through a real save

**Goal.** Replace earliest-line quotation with observable corrected recall across restart and non-omniscient
controller substitution.

**Scope.** Extend `tools/cli/tests/milestone_d.rs`, `speech_both_ways.rs`, and in `cognition/lm-controller/tests/`:
`milestone_d_cognition.py`, `test_milestone_d_replay.py`, `test_home_isolation.py`, `test_memory_isolation.py`,
`test_controller_seat.py`; one deterministic case through `clients/agent` (`mineworld_agent`). The historical
cassette stays untouched; the new cassette comes from the §6 live session. Production recall/prompt changes only for
a demonstrated missing relevant perceived line or newer/older confusion, recorded first; changing retention,
scopes or bounds is a material deviation.

**Implementation.**
- [ ] Scripted sequence: runtime nonce code + initial location; leave/rejoin question; correction (new location,
  same code); unrelated exchange; another resident's recorded action; graceful full stop and restart; current
  location-and-code question. Questions never contain the answers.
- [ ] Link accepted `spoke` Event IDs to shown memory, valid recalls, request/action IDs and the caused saved fact;
  prove store reuse on restart.
- [ ] Private undisclosed nonce and a second seat in the same home: no leak. Rule → human → LM and MCP-agent →
  LM release with unchanged identity and holdings.

**Validation.**
- [ ] Scripted backend answers only from shown records; removing the correction or its access fails the test; a
  hidden fact never appears in shown memory.
- [ ] Retained-store restart differs from a null-store rebuild; old triggers not replayed; after `--fresh` the old
  nonce is unrecallable and the old store is gone.

**Acceptance.** After restart the caused NPC utterance carries the corrected location and the original code with
correct source references; no superseded location as current; no unknown private nonce; same Person across
controller swaps; no seat theft or stale concurrent action. A SQLite file alone fails.

**Failure/edges.** Cited but wrong = FAIL. Correction absent from shown memory = retrieval defect; present but
ignored = model/prompt failure. Missing runtime or stall before answer = INCONCLUSIVE.

**Commands.** Selected cognition tests; `cargo test -p mineworld-cli --test milestone_d -- --include-ignored
--exact she_remembers_across_the_two_views` plus the new correction test (named in §10 once it exists). No live
model. NOT RUN.

**Boundary/review.**
- [ ] Review shown prompt, rendered text, correction chronology, access predicates, restart identity, oracle
  provenance.
- [ ] Commit acceptance and any bounded owner fix together.

### C5 — package, observe, live gate

**Goal.** The playable path with honest prerequisites and package-level evidence.

**Scope.** `scripts/package.py::layout/assemble/check_contents` (ship the template and decisions file only),
`packaging/PLAY.txt` (how to enable conversation, plain words), launcher/package contract text, smoke/play
artifacts, the live-gate cassette.

**Implementation.**
- [ ] Ship `runtime/cognition-profile.template.toml` and the decisions file; default play unchanged and model-free.
- [ ] Package/source entry-point parity; user save and store paths.
- [ ] Packaged 2D, 3D and both modes on the same save with `alice` on the LM, configured through the per-user
  profile on this Mac.

**Validation.**
- [ ] `--self-test`, standard smoke, R1 missing-prerequisite degraded path, configured-runtime success.
- [ ] Rendered dialogue and walk/interact in both clients; both-window exit; packaged model-absent run; no
  download or remote connection.
- [ ] Checksums, source/export identity, redacted profile/config, manifest, licences, size report, cleanup evidence,
  `package.py` line delta.
- [ ] **Live gate (§6)**, the only live-model run in this lane.

**Acceptance.** The relocated package runs the declared route; live gate PASS per §6 rubric, or FAIL/INCONCLUSIVE
recorded with "check 2 NOT MET" and the operator choice of §0; no owned-child leak; R1 labelled
prerequisite-dependent.

**Failure/edges.** Missing interpreter/arch/metadata fails preflight by name; read-only bundle and paths with
spaces/Unicode work; a fresh world cannot recall a prior nonce.

**Commands.** `python3 scripts/package.py --self-test`; `assemble`, `budget`, `smoke` in one heavy slot; GUI
sessions via UI tools. NOT RUN.

**Boundary/review.**
- [ ] Review manifest, prerequisite disclosure, fingerprints, window evidence, performance, process lifetime,
  maintainability (CLAUDE.md §3.4).
- [ ] Final ledger/diff review; one semantic commit; READY FOR OPERATOR REVIEW. Never merge.

## 6. Bounded local live gate (Q-D3)

**Setup.** Single serial session on this Mac after all deterministic checks pass. `qwen3.5:4b` at
`http://127.0.0.1:11434/v1`; empty `key_env`; no secrets file; no cloud tier; no download; no service start.
`structured_output="none"`, `reasoning="off"` are starting candidates whose compatibility is measured. Social tier
local; other tiers `none`; retention `consolidate`; `max_in_flight=1`; `max_calls=1`; `consult_deadline_s=20`;
`MAX_OUTPUT_TOKENS` 256 (existing); `greet_cooldown_s=86400`.

**Pinned scenario.** A dedicated Market Town save started at world time **07:00**, Alice at the café counter (her
05:30 punctual part has begun; her next part, 14:00 park, is not punctual), so no `upcoming` or `complaint`
trigger can fire. All turns finish before world 13:00 (30 wall minutes at `time_scale: 12`). The player stands at
the counter within `conversation` reach. The harness waits for each consult to end (decision log) before the next
line, so no in-flight consult is superseded.

**Enforcement.** A gate-specific ledger file (`[budgets] ledger`) with `calls_per_wall_hour = 7` and
`tokens_per_wall_day = 30000`; the ledger itself refuses an eighth consult call (expected: six). The harness counts the one warm-up
separately and stops the run if the total (ledger calls + warm-up) would exceed 8.

**Expected calls (cap 8).**

| # | Cause | Calls |
| --- | --- | --- |
| 0 | Warm-up: one Ollama load request with an empty prompt before t0 (loads the 4 GB model; generates nothing) | 1 |
| 1 | Player states code + location (T1) | 1 |
| 2 | Player leaves and returns; T3 suppressed by `greet_cooldown_s` within the process | 0 |
| 3 | Player asks where/what (T1) | 1 |
| 4 | Player corrects the location (T1) | 1 |
| 5 | Unrelated short line (T1) | 1 |
| 6 | Restart; player re-enters; T3 fires once (greeted set is process memory) | 1 |
| 7 | Player asks current location and code (T1) | 1 |
| | **Total expected / cap** | **7 / 8** (one spare for a single failure) |

**Other bounds.** Inference wall ≤ 180 s total; ≤ 20 s trigger-to-submit per reply; each real-process run
≤ 10 min; all C3–C5 real-process runs ≤ 60 min; $0.

**PASS rubric (fixed now, before measuring).** All must hold for turn 7: (a) the saved NPC utterance was caused by
that consult's accepted `ActionIntent`; (b) it names the corrected location; (c) it names the original code
exactly; (d) it does not present the superseded location as current; (e) both source `spoke` events (correction
and original code) are in the shown context and cited by valid recalls; (f) turn 1 and turn 3 replies are relevant
to the player's words (not a generic template). Plus: at least one other resident's action is recorded during
inference, and disk growth of `cognition/market-town/` and the save is reported.

**Judges.** First the independent red-team reviewer scores (a)–(f) from the saved transcript, events and consult
log; then the operator's manual acceptance, recorded separately (operator requirement §6). Machine checks own
(a), (c), (e); (b), (d), (f) are reviewer judgments against the rubric.

**Outcome handling.** Any cap reached ⇒ stop, save, keep evidence. Wrong answer ⇒ FAIL; prerequisite unavailable or
stall before turn 7 ⇒ INCONCLUSIVE. Either ⇒ §0 Q-D3 consequence: "check 2 NOT MET", operator choice. No cap raise,
model swap or retry for a lucky answer.

**Cassette.** The live session is recorded as a new replay cassette (calls already counted above) and becomes the
C4 replay fixture for PR-B; the 2026-10 Milestone D cassette stays unchanged. Refreshing it later is an
operator-run recording under its own bounded approval.

**Baseline comparison.** Rule-only vs one-LM profile on the same save: usable frame, other-resident action/tick
latency, reply latency, CPU and unified-memory pressure — reported as observations, never capacity claims.

## 7. Verification ownership

| Layer | Unique claim | Status |
| --- | --- | --- |
| Static | types, format/lint, provider/ownership scans, doc links, decision IDs | NOT RUN |
| Deterministic integration | absolute deadline, accounting, assignment/store isolation, causality, access, replay, identity, MCP parity | NOT RUN |
| Cross-platform | launcher child lifecycle on macOS/Linux/Windows CI (scripted backend) | NOT RUN |
| Live gate (Gate 1) | §6 rubric within §6 caps, this Mac only | NOT RUN |
| Real process + package (Gate 2) | EOF/cleanup, save+store restart, exported windows with R1 runtime | NOT RUN |
| Terminal CI | exact final head of each PR | NOT RUN |

Report exact commands, counts, wall time, configuration, HEAD + diff fingerprint, decisive observations; classify
PASS/FAIL/INCONCLUSIVE only after inspection.

## 8. Corrections in DM-20261127-2 (vs recovered DM-20261123-1)

1. Audit base moved forward with an empty seam diff. 2. Implementation base split into two PRs (DM-3 corrects the
integration-branch fact). 3. Execution endpoints follow Structured Coding defaults (Q-D5). 4. Live gate made
concrete. 5. Decisions at top in plain language. 6. Tracked location; recovered sources preserved byte-exact.
7. Verified test inventory. 8. LM-concept boundary stated. 9. R-LOAD-1 applied.

## 9. Pre-freeze review dispositions (DM-20261127-3)

Review: `_reviews/design-dialogue.md`, verdict FREEZE-AFTER-FIXES.

| Finding | Disposition |
| --- | --- |
| F-1 integration branch exists (`e37ce9fd`); merge order | Fixed: header, §0 Q-D4 corrected fact + merge order, §2 rows for `hosting.rs`/`configure_client`, §4.11 child environment rule. Approved answer unchanged |
| F-2 contracts unspecified | Fixed: §3.4.1–3.4.4 shapes, closed reason enum, owning specs, cognition owns instance binding |
| F-3 ARC-57 amendment | Fixed: C1 scope + §4.7; authority `cognition-toml.md` l.83 cited |
| F-4 `--fresh` retention | Resolved as B1 (delete with save; matches existing public contract); operator may overrule |
| F-5 profile location | Resolved as B2 (per-user `cognition/profile.toml`, template shipped inactive) |
| F-6 provisioning and platforms | Fixed: Q-D1 plain-words provisioning and checkout coupling; B3 platforms; §4.11 Windows items |
| F-7 live gate by construction | Fixed: pinned window, trigger table, ledger enforcement, counted warm-up, rubric, judges — within the approved 8-call cap |
| F-8 READY with check 2 unmet | Fixed: §0 Q-D3 consequence, §6 outcome handling, contract stop condition |
| F-9 contradictions | Fixed: two-PR framing, per-PR stop conditions, invite claim scoped to the cognition child, section-based MODULE_SPEC citation, p6c §9 citation, audit base `8c971f0f` |
| F-10 reuse record | Fixed: cite `ARC-59`/`DEP-58`; new record limited to the runtime route with R2 candidates |
| F-11 size growth | Fixed: §4.14 |
| F-12 MCP | Fixed: §1, §4.2, §4.13, C4 case |
| F-13 restore detection | Reduced to a documented caveat + refusal test (§3.4.4) |
| F-14 cassette refresh | Fixed: §6 Cassette |

## 10. Ledgers (live)

### PR-A (C1)

Base `8c971f0f`. Branch, owner and evidence: not started.

### PR-B (C2–C5)

Base: pending the client-stack merge and PR-A merge; the exact `main` SHA and the launcher seam re-audit result
are recorded here before C2. Not started.
