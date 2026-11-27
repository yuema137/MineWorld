# Launcher P1 + P3 — packaged launchers default to Market Town; packaged single-player worlds start `--solo`

## DRAFT — FREEZE-READY (awaiting operator rulings Q-LS1 … Q-LS6, §0)

```text
Design revision:        revision 1 (2026-11-27), drafted by the startup/launcher lane on coordinator dispatch
Approved by / evidence: NOT YET APPROVED. Freeze requires the operator's answers to §0; recommendations are not rulings
Implementation base:    #549 head fcba4c1d6b40acb63cdfdd04e141674f20d5d4ba (fix/launcher-living-world), which owns
                        the `Hosting` seam this PR edits. Code starts on the main SHA that contains #544, #548 and
                        #549 (PR stacks on no unmerged branch); seams are re-checked on that SHA before C1
Start condition:        #549 merged to main and this design frozen
Execution contract:     §7 of this document (one authority; no separate contract file)
Parent:                 operator-playable-mvp-20261123.md; pr-nightlife.md §1.3 (P1, P3);
                        pr-playable-dialogue-memory.md §§2, 3.2 (PR-B launcher seams and profile)
Lifecycle:              DRAFT — FREEZE-READY; implementation not started
```

## 0. Questions for the operator (freeze gate)

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-LS1** | Which world does each packaged launcher play by default? Today 2D plays `market-town`; 3D and 2D+3D play `social-cafe`, which has no sleep/needs packs, so the town's night, shops and sleep never appear in 3D. | **Market Town for all three modes.** Social Café stays installed and selectable with `world = "social-cafe"` in `launch.toml` (LAUNCHER.md §4). Saves are per world (`saves/<world>/`), so an existing Social Café save is neither touched nor resumed. |
| **Q-LS2** | Which packaged runs start the host `--solo` (the single-player declaration of `ARC-69` item 5, which nightlife N2's sleep advance requires)? | **The 2D and 3D play runs: yes. The 2D+3D run: no**, because it seats two human players (`AC-15`) and is a shared world by definition; nightlife Q-L4 A would refuse its advance anyway. **Smoke runs: no** (fixtures keep their current hosting). This aligns the packaged path with the source shell launchers, which already pass `--solo` (`clients/shared/HOST.md` §6 item 3). |
| **Q-LS3** | `--solo` also lets the server accept day-length scales above 24× (`ARC-69` item 5). Should the packaged game offer them? | **No. Keep the product ceiling at 24×.** The World tab keeps its 6/12/24 presets and is unchanged. The admin token is private to the launcher's own clients (#549), so no other caller can ask for more. No server change: the wider server permission is recorded as such. Offering a scale above 24× later is its own ruling and its own PR. Rejected alternative: a server flag that caps `--solo` at 24×. It adds a host code path to forbid something no shipped client requests. |
| **Q-LS4** | How does a player opt into multiplayer/hosting for others? | **Not in this PR.** The packaged launcher stays loopback-only (`127.0.0.1:0`), as `ARC-78` states. Local two-player play remains the 2D+3D mode (non-solo). Hosting for other machines remains `mineworld server <world> --listen <addr>` without `--solo` (`PROTOCOL.md`, `HOST.md` §6). A launcher "Host for others" mode needs its own design: a LAN listen address, an invite shown to the player, never `--solo`, and firewall/platform notes for all three OSes. |
| **Q-LS5** | Do `--smoke` checks follow the new played default? | **No. Smoke keeps its fixture worlds**: 2D `market-town`, 3D and 2D+3D `social-cafe`, with `--agent alice`. The 3D `--slice-link` verdict and `scripts/package.py` probes are recorded against those fixtures. The new 3D Market Town default is proven instead by the real packaged Gate 2 run in C1, not by moving a fixture. |
| **Q-LS6** | The 2D launcher plays the resident `carol`, so her routine stops while the player drives her. Dialogue PR-B's profile assumes the human is `visitor` in the single-client modes (`pr-playable-dialogue-memory.md` §3.2). Change the 2D default seat? | **Yes: 2D plays `visitor`.** This matches 3D and PR-B, so every resident keeps its routine and `visitor` is the same Person in 2D and 3D (release and reacquire, as PR-B describes). `seat = "carol"` stays available through `launch.toml`. |

## 1. Requirement, scope and non-goals

### 1.1 Requirement

The operator's 2026-11-23 play trial found a stationary town, no replies, no clock controls and no night. #549 (P2)
restores town controllers and private clock controls. Two prerequisites remain, both owned by the launcher lane
(`pr-nightlife.md` §1.3):

- **P1.** The packaged launcher starts Market Town with its own save and matching depiction.
- **P3.** Packaged single-player worlds start `--solo`, so that nightlife N2-C4's sleep advance is permitted. Its
  product consequence (scales above 24×) is surfaced here as Q-LS3.

### 1.2 In scope

- `tools/launch/src/config.rs`: the per-mode default world, and the 2D default seat if Q-LS6 is approved.
- `tools/launch/src/hosting.rs`: `Hosting::Played` carries whether the run is single-player. `configure_server`
  adds `--solo` when it is.
- `tools/launch/src/lib.rs`: builds `Hosting` from the mode (2D or 3D play run → solo; 2D+3D → shared). It also
  separates the smoke fixture world from the played default (Q-LS5).
- `tools/launch/tests/launcher.rs` and `tools/launch/tests/launcher/living.rs`: the oracles of §5.
- Specification: `tools/launch/LAUNCHER.md` (§2 table, §7.1 packaged path) and `clients/shared/HOST.md` §6.1.
  Its sentence "adds no `--solo` permission" is rewritten to say that single-client play runs are `--solo` and
  the 2D+3D and smoke runs are not.
- A dated note on `docs/decisions/ARC-78.md` recording the hosting change. `ARC-69` itself is unchanged: this PR
  uses `--solo` exactly as `ARC-69` item 5 defines it.

### 1.3 Non-goals

- No server, kernel, System, controller, World Pack, schema or client-script change.
- No new scale offer (Q-LS3). No LAN hosting (Q-LS4). No sleep advance: that is nightlife N2.
- No change to the Social Café or Bodies Yard content.
- No cognition child and no `--decisions` (dialogue PR-B).
- No startup-performance work: that investigation stays in the startup lane, #548.

## 2. Audit (origin/main c161c120 + #549 fcba4c1d)

| Seam | Current behaviour | This PR |
| --- | --- | --- |
| `tools/launch/src/config.rs::choice` | Defaults: 2D `market-town`/`carol`; 3D `social-cafe`/`visitor`; both `social-cafe`/`visitor`+`wanderer`. `launch.toml` overrides per mode. | 3D and both default to `market-town`; 2D seat `visitor` (Q-LS6). |
| `tools/launch/src/lib.rs::run` | `let played = options.smoke.unwrap_or(mode); let choice = chosen(&bundle, played)?`. Smoke uses the same defaults as play. | Smoke resolves its fixture world independently (Q-LS5). |
| `tools/launch/src/hosting.rs` (#549) | `Played { admin }` → `--town` plus a private token; `Smoke` → `--agent alice` with the token removed. | `Played { admin, solo: bool }`. `--solo` is added only when `solo`. |
| `tools/cli/src/serve.rs` l. 164–190 | `--solo` requires a loopback listen address. Above 24× is refused unless `--solo`. | Unchanged. The launcher already listens on `127.0.0.1:0`. |
| `server/src/admin.rs` l. 328–335 | `POST /admin/clock` above 24× → 409 `fast_forward_solo_only` unless solo. | Unchanged. Used as the oracle that the flag reached the server (§5). |
| `clients/shared/HOST.md` §6 item 3 | Source shell launchers start `--solo`. §6.1 (in #549) says the packaged supervisor adds no `--solo`. | §6.1 is rewritten per Q-LS2. |
| `presentation/mineworld-default/3D/depictions/market-town.yaml`, `3D/far_field/market-town.yaml` | The 3D Market Town depiction and far field exist on main. `scripts/package.py` `WORLDS` already bundles `market-town`. | No change. The C1 Gate 2 run proves the packaged 3D client draws it. |
| `worlds/market-town/world.yaml` seats | `visitor` and `wanderer` exist (`AC-15`); `carol` is a resident seat. | No change. |
| #549 living test | Expects `both` to answer 48× with 409 ("both is not solo"). | Kept as the non-solo oracle. A 2D or 3D played case adds the solo oracle. |

The nightlife N2 advance gate reads the host's `solo` (`pr-nightlife.md` N2-I3, `server/src/host.rs`). Under
Q-L4 A it also refuses while another controller is attached. So in the 2D+3D mode, and under PR-B's LM Alice, the
advance is unavailable with a named reason. That is consistent with Q-LS2 and needs nothing from this PR.

Dialogue PR-B also edits `Hosting::Played` (it adds `--decisions`). Whichever lands second merges a two-line
`configure_server` change; the two are not otherwise coupled.

## 3. Frozen invariants (on approval)

- **LS-I1.** A packaged 2D or 3D play run starts `mineworld server <world> --listen 127.0.0.1:0 … --town --solo`. A
  2D+3D play run and every smoke run never pass `--solo`.
- **LS-I2.** The admin token stays as #549 defines it: per run, private, never in argv or logs. Smoke never
  carries one.
- **LS-I3.** The clients' offered scales stay 6/12/24. No shipped client requests more than 24× (Q-LS3).
- **LS-I4.** The default world change creates, moves or deletes no existing save. `saves/social-cafe/` stays
  readable by an explicit `world = "social-cafe"`.
- **LS-I5.** Smoke fixture worlds and verdicts stay byte-identical in behaviour to before this PR (Q-LS5).
- **LS-I6.** Platform parity: the change is argument and environment construction only, compiled and tested on
  macOS, Linux and Windows by the existing launcher tests in CI.

## 4. Reuse decision

Nothing new is adopted. The PR reuses `--solo` (`ARC-69`), the `Hosting` seam (#549), the existing `launch.toml`
grammar (`ARC-78`) and the packaged smoke (`scripts/package.py smoke`). No new dependency.

## 5. Commit plan

### C1 — Market Town by default (P1), 2D seat `visitor`, smoke fixtures decoupled

- [ ] **Implementation.**
  - `config.rs` defaults (3D and both → `market-town`; 2D seat → `visitor` per Q-LS6).
  - A smoke fixture table in `lib.rs` (or `config.rs`), keeping today's smoke worlds.
  - `LAUNCHER.md` §2 table and §4.
- [ ] **Deterministic validation (Gate 2, real server, stub renderer).**
  - Each mode's play run joins `market-town` with the expected seats; the save folder is `saves/market-town/`.
  - A pre-existing `saves/social-cafe/` is untouched (byte-compare) and still resumes when `launch.toml` names it.
  - `--smoke=3d` and `--smoke=both` still host `social-cafe` with `--agent alice`.
  - Config unit cases for the new defaults and for overrides.
- [ ] **Gate 2, packaged.** `scripts/package.py` assemble, then a real packaged 3D launcher play run on Market
  Town to the first drawn world view.
  - Record the measured first-view time. No speed claim; compare with the startup lane's Social Café numbers only
    as context.
  - If the 3D Market Town first view does not arrive within a 600 s run, stop and escalate as a material
    discovery; do not narrow scope silently.
  - Run under `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one heavy job.
- [ ] **LLM logic review.** Ownership (launcher only), no fixture drift, readable defaults.

### C2 — single-player runs start `--solo` (P3)

- [ ] **Implementation.**
  - `Hosting::Played { admin, solo }`, where `solo` is true for 2D and 3D play runs and false for both.
  - `configure_server` adds `--solo`.
  - Specification updates: `LAUNCHER.md` §7.1, `HOST.md` §6.1, and the `ARC-78` dated note.
- [ ] **Deterministic validation (Gate 2, real server).**
  - A 3D (or 2D) played run: an authenticated `POST /admin/clock {"time_scale":25}` answers 200. That proves the
    server is `--solo`; the test then restores 12. No shipped client sends 25 (LS-I3).
  - The #549 2D+3D case still answers 48× with 409.
  - Smoke still has no admin surface and is not solo (its server log).
  - Restart of a solo world resumes the day-length scale and drops any fast-forward scale (`ARC-69` item 8;
    existing server behaviour, asserted at the launcher boundary).
  - Token not in argv or logs (the #549 checks extended to the solo case).
- [ ] **LLM logic review.** Single owner of the solo decision (`Hosting`), no inference from connection count
  (`ARC-69` item 5 rejected alternative), and no client changes.

Integration checkpoint: a packaged 3D launch of Market Town, `--solo`, with working 6/12/24 controls. This is the
state that nightlife N1-C3's packaged night check and N2-C4 wait on.

## 6. Test ownership, review and load

- **Launcher crate tests:** `cargo test -p mineworld-launch`, after `cargo build -p mineworld-cli`. They own the
  argument, world and token oracles.
- **Packaged run:** owned by this PR's Gate 2 (C1).
- **Required remote check:** `fast` on the exact head. Full workspace NOT RUN locally (R-LOAD-1).
- **Review:** independent red-team review of this design before freeze, and of the exact head before READY FOR
  OPERATOR REVIEW.

## 7. Execution contract (filled on freeze)

```text
Owner / worktree:      one implementation session; new worktree under /Users/yuema137/mineworld-worktrees/
Affected paths:        tools/launch/**, clients/shared/HOST.md, docs/decisions/ARC-78.md, this ledger
Authorized:            implement C1–C2, targeted tests, one packaged 3D Market Town run, semantic commits, push, PR
Not authorized:        server/kernel/System/client-script/world-content changes; scale offers >24x; LAN hosting;
                       merge; `milestone` label
Budgets:               <=600 s per heavy run, one shared heavy slot, no paid API calls
Escalate:              any need to change the server's --solo semantics, a 3D Market Town first view that cannot be
                       reached, or a smoke verdict that changes
```

## 8. Evidence and readiness

None yet. This is a design-only draft.
