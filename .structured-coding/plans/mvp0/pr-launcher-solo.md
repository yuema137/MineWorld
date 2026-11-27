# Launcher P1 + P3 — packaged launchers default to Market Town; packaged single-player worlds start `--solo`

## DRAFT — FREEZE-READY (awaiting operator rulings Q-LS1 … Q-LS7, §0)

```text
Design revision:        revision 3 (2026-11-27): revision 2 + delta re-review D1–D4 (§9); revision 2 fixed F1–F10 of
                        _reviews/design-launcher-solo.md. Dispositions in §9. No ruling is assumed.
Approved by / evidence: NOT YET APPROVED. Freeze requires the operator's answers to §0; recommendations are not rulings
Implementation base:    #549 head fcba4c1d6b40acb63cdfdd04e141674f20d5d4ba (fix/launcher-living-world), which owns
                        the `Hosting` seam this PR edits. Code starts on the main SHA that contains #544, #548 and
                        #549 (PR stacks on no unmerged branch); seams are re-checked on that SHA before C1
Start condition:        #549 ratified (frozen) and merged to main, and this design frozen
Execution contract:     §7 of this document (one authority; no separate contract file)
Parent:                 operator-playable-mvp-20261123.md; pr-nightlife.md §1.3 (P1, P3);
                        pr-playable-dialogue-memory.md §§2, 3.2 (PR-B launcher seams and profile)
Lifecycle:              DRAFT — FREEZE-READY; implementation not started
```

## 0. Questions for the operator (freeze gate)

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-LS1** | Which world does each packaged launcher play by default? Today 2D plays `market-town`, while 3D and 2D+3D play `social-cafe`. Social Café has no sleep/needs packs, so the town's night, shops and sleep never appear in 3D. | **Market Town for all three modes**, subject to the Q-LS7 acceptance bound. **Pre-decided fallback:** if 3D Market Town misses Q-LS7, 3D and 2D+3D keep `social-cafe` (2D is unchanged) and the miss is reported with its measurements; P3 still lands. In that branch, P1 is recorded **partial** in this ledger (2D only); LAUNCHER.md §2 rows state `social-cafe` for 3D/2D+3D; nightlife N2-C4's 3D sleep check is noted to need `world = "market-town"` in `launch.toml` (this design records it; the frozen nightlife text is not edited). **P1's resident cycle is proven here, not handed off:** C3 is a dedicated, budgeted packaged run (§5) whose pass condition is a non-Alice resident's home → asleep → awake → commute sequence. Social Café stays selectable with `world = "social-cafe"` in `launch.toml` (LAUNCHER.md §4). Saves are per world (`saves/<world>/`), so an existing Social Café save is neither touched nor resumed. The per-mode defaults are owned here; dialogue PR-B's cognition profile only overrides them per mode (`pr-playable-dialogue-memory.md` §3.3) and does not edit `config.rs` defaults. |
| **Q-LS2** | Which packaged runs start the host `--solo`? `--solo` is the single-player declaration of `ARC-69` item 5, which nightlife N2's sleep advance requires. | **2D and 3D play runs: yes. 2D+3D run: no.** It seats two human players (`AC-15`); the operator's 2026-11-23 bound gives no blanket solo authority to both-client mode (`operator-playable-mvp-20261123.md` §3). **Smoke runs: no.** This aligns the packaged path with the source shell launchers, which already pass `--solo` (`clients/shared/HOST.md` §6 item 3; `ARC-69` Consequences: "a launcher starts a single-player world with `--solo` and a generated admin token"). **Consequences to rule on knowingly:** (a) `--solo` restricts **no** admission. It only requires a loopback listener and widens permissions (§2), so a cognition child, a second client or an MCP agent holding the invite still joins. (b) In dialogue PR-B's LM profile, LM Alice is attached, so nightlife Q-L4 A always refuses the sleep advance ("another controller is connected"; the named message is owned by N2-C4 item 3) until a future N3. |
| **Q-LS3** | `--solo` also lets the server accept day-length scales above 24× (`ARC-69` item 5). Should the packaged game offer them? | **No. Keep the product ceiling at 24×.** The World tab's presets stay 6/12/24 (`host_clock.gd` `DAY_SCALES`). The admin token reaches only the launcher's own child processes (#549). No process of another user can ask for more; a process of the same user can read child environments, as it can today. No server change: the wider server permission is recorded as such. Offering more than 24× is its own ruling and PR. Rejected: a server flag capping `--solo` at 24×, which adds a host code path to forbid something no shipped client requests. |
| **Q-LS4** | How does a player opt into multiplayer/hosting for others? | **Not in this PR.** The packaged launcher stays loopback-only (`127.0.0.1:0`, `ARC-78`). Local two-player play remains the 2D+3D mode (non-solo). Hosting for other machines remains `mineworld server <world> --listen <addr>` without `--solo`. A launcher "Host for others" mode needs its own design: LAN listen, an invite shown to the player, never `--solo`, firewall notes for all three OSes. |
| **Q-LS5** | Do `--smoke` checks follow the new played defaults? | **No.** A fixture table pins smoke **world and seats** per mode, exactly today's values: 2D `market-town`/`carol`; 3D `social-cafe`/`visitor`; 2D+3D `social-cafe`/`visitor`+`wanderer`; all with `--agent alice`. **Smoke ignores `launch.toml`** (a behaviour change: today a player's `launch.toml` also steers smoke). Smoke checks the bundle, not the player's choice. The 3D `--slice-link` verdict and `scripts/package.py` probes stay valid. |
| **Q-LS6** | The 2D launcher plays the resident `carol`, so her routine stops while the player drives her. Dialogue PR-B assumes the human is `visitor` in single-client modes (§3.2). Change the 2D default seat? | **Yes: 2D plays `visitor`.** This matches 3D and PR-B and keeps every resident on its routine. **Consequence:** an existing `saves/market-town/` that was played as Carol resumes with the player as `visitor`. Carol (wallet, holdings and location kept) becomes town-driven. Nothing is reset or deleted. LAUNCHER.md states the opt-out `seat = "carol"` in `[2d]`. Saves are shared by the three modes, which alternate solo and non-solo. This is harmless: a fast-forward scale never survives a restart (`ARC-69` item 8). |
| **Q-LS7** | What must the packaged 3D Market Town run achieve before 3D/2D+3D switch to it (Q-LS1)? Social Café's packaged 3D first view is 145–153 s today, with an open +8.1 s regression (#548). | **On one macOS machine, one session, under matched conditions:** (1) the **median of 3 interleaved runs per world** (café, town, café, town, café, town) of first view is ≤ **1.25 ×** the café median and ≤ **240 s** absolute. A margin under 5 % of either bound is INCONCLUSIVE and takes the fallback. Conditions: a fresh `--user-dir` per run (fresh save); the Godot shader cache is **not** cleared (a warm cache, as a returning player has; the first run of the session is discarded as cold warm-up and recorded); default graphics; window 1600×900. Timer: starts at launcher process start (wall clock taken by the wrapper) and stops at the client's `[startup] … client scene visible` line, which #548 prints only after the first world view has been drawn; the timestamp is read from the client log. (2) the packaged `--slice-market` probe of §5 C1 passes (walk, enter a place, buy; this is headless, so it proves interaction, not drawing). (3) a far-field screenshot at the spawn viewpoint shows no clear-colour or sky-only sector (the far-field checks' criterion); the operator judges the screenshot on the progress page. (4) frame time: median and p95 over 300 frames after first view, same build and window, recorded, not gated. The implementer measures and the red-team reviewer re-derives the numbers from the recorded logs. Any miss of (1)–(3) takes the Q-LS1 fallback. |

## 1. Requirement, scope and non-goals

### 1.1 Requirement

The operator's 2026-11-23 play trial found a stationary town, no replies, no clock controls and no night. #549 (P2)
restores town controllers and private clock controls. Two prerequisites remain, owned by the launcher lane
(`pr-nightlife.md` §1.3):

- **P1.** "The launcher starts Market Town with its own save and matching depiction, **and at least one non-Alice
  resident walks home, sleeps, wakes and commutes**."
  - The launcher half (world, save, depiction) is this PR's C1.
  - The resident-cycle clause is proven **in this PR** by C3, a dedicated packaged run with its own wall budget
    (§5). The frozen nightlife design is not amended. Its N1-C3 checks stay as they are.
- **P3.** Packaged single-player worlds start `--solo`, so nightlife N2-C4's sleep advance is permitted. The
  >24× consequence is surfaced as Q-LS3.

### 1.2 In scope

- `tools/launch/src/config.rs`: per-mode defaults (Q-LS1, Q-LS6) and the smoke fixture table (Q-LS5).
- `tools/launch/src/hosting.rs`: `Hosting::Played { admin, solo }`. `configure_server` adds `--solo` when `solo`.
- `tools/launch/src/lib.rs`: builds `Hosting` from the mode, and resolves smoke from its fixture table without
  reading `launch.toml`.
- `tools/launch/tests/launcher.rs` and `tools/launch/tests/launcher/*.rs`: the oracles of §5.
- Specifications:
  - `tools/launch/LAUNCHER.md`: the §2 table, including its "2D client as `carol`" row, §4 `seat` opt-out, and
    §7.1.
  - `clients/shared/HOST.md` §6.1.
  - A dated note on `docs/decisions/ARC-78.md` that **supersedes** #549's amendment sentence "No `--hosted-here`,
    Keep Running or `--solo` is added" for 2D/3D play runs. The supersession is explicit, never a silent rewrite.
- `ARC-69` wording: this PR uses `--solo` as defined by `ARC-69` item 5 **as amended by the frozen nightlife design**
  (`pr-nightlife.md` §0). After N2-C2's amending record, `--solo` widens two permissions: scales above 24×, and the
  advance operation. LS-I1 names both; the second applies once N2-C2 has landed. No ARC-69 text is edited here.

### 1.3 Non-goals

- No server, kernel, System, controller, World Pack, schema or client-script change.
- No offer above 24× (Q-LS3), no LAN hosting (Q-LS4), no sleep advance (nightlife N2), no Social Café or Bodies Yard
  content change, no cognition child or `--decisions` (dialogue PR-B), no startup-performance work (startup lane, #548).

## 2. Audit (origin/main c161c120 + #549 fcba4c1d)

| Seam | Current behaviour | This PR |
| --- | --- | --- |
| `tools/launch/src/config.rs::choice` | Defaults: 2D `market-town`/`carol`; 3D `social-cafe`/`visitor`; both `social-cafe`/`visitor`+`wanderer`. `launch.toml` overrides per mode. | Played defaults per Q-LS1/Q-LS6, with the Q-LS1 fallback branch. A separate smoke fixture table (Q-LS5). |
| `tools/launch/src/lib.rs::run` l. 173–174 | `let played = options.smoke.unwrap_or(mode); let choice = chosen(&bundle, played)?`, so smoke reads the played defaults **and** `launch.toml`. | Smoke resolves only from the fixture table. |
| `tools/launch/src/children.rs` l. 249–256 | Smoke passes `--seat=<seat> --drive` (2D) or `--slice-link` (3D). | Unchanged; the seat now comes from the fixture table. |
| `tools/launch/src/hosting.rs` (#549) | `Played { admin }` gives `--town` plus a private token. `Smoke` gives `--agent alice` and removes the token. | `Played { admin, solo: bool }`; `--solo` only when `solo`. |
| `tools/cli/src/serve.rs` l. 164–190; `server/src/app.rs` l. 150 | `--solo` requires a loopback listener. A scale above 24× is refused unless `--solo`. | Unchanged. The launcher already listens on `127.0.0.1:0`. |
| `server/src/admin.rs` l. 283, 328–335 | `POST /admin/clock` above 24× → 409 `fast_forward_solo_only` unless solo. Live ceiling 3600. | Unchanged; used as the oracle that the flag reached the server (§5). |
| Admission | `--solo` changes no admission. | Stated in Q-LS2. |
| `clients/shared/host/host_clock.gd` l. 16, 84–86 | `DAY_SCALES` = [6, 12, 24]; `set_scale` sends only those. | Unchanged (Q-LS3). |
| `clients/shared/HOST.md` §6 item 3, §6.1 (#549) | Source shell launchers start `--solo`. §6.1 says the packaged supervisor adds no `--solo`. | §6.1 superseded per Q-LS2 with a dated note. |
| `presentation/mineworld-default/3D/depictions/market-town.yaml`, `3D/far_field/market-town.yaml`; `scripts/package.py` `WORLDS` | The 3D Market Town depiction and far field exist, and the bundle ships `market-town`. The packaged automated 3D probe uses `social-cafe` (`package.py` l. 114). Rendered 3D far-field tests cover the slice default and Lakeside only (`tools/cli/tests/client_far_field.rs`). | Q-LS7's packaged gate is the first 3D Market Town evidence. |
| `worlds/{market-town,social-cafe}/world.yaml` seats | Identical `seats:` lists. `visitor` and `wanderer` are players (`AC-15`); `carol` is a resident. | No change. An existing `launch.toml` that names only a seat stays valid. |

## 3. Frozen invariants (on approval)

- **LS-I1.** A packaged 2D or 3D play run starts `mineworld server <world> --listen 127.0.0.1:0 … --town --solo`. A
  2D+3D play run and every smoke run never pass `--solo`. `--solo` widens exactly the permissions `ARC-69` item 5,
  as amended by nightlife N2-C2, grants: scales above 24×, and (once N2-C2 lands) the advance. It restricts no
  admission.
- **LS-I2.** The admin token stays as #549 defines it: per run, private, never in argv or logs. Smoke never carries
  one.
- **LS-I3.** No shipped client requests more than 24× (Q-LS3).
- **LS-I4.** No save is created, moved, reset or deleted by the default change. `saves/social-cafe/` resumes under
  an explicit `world = "social-cafe"`. A Carol-played `saves/market-town/` resumes with the player as `visitor`
  (Q-LS6).
- **LS-I5.** Smoke world, seats, hosting and verdicts are identical to before this PR, and smoke ignores
  `launch.toml` (Q-LS5).
- **LS-I6.** The change is argument and environment construction only. Platform evidence is listed in §6.

## 4. Reuse decision

Nothing new is adopted. This PR reuses `--solo` (`ARC-69`), the `Hosting` seam (#549), the `launch.toml` grammar
(`ARC-78`), `scripts/package.py` assemble/smoke, and the existing `--slice-market` probe. No new dependency.

## 5. Commit plan

### C1 — played defaults (P1 launcher half, Q-LS6) and the smoke fixture table

- [ ] **Implementation.**
  - `config.rs` played defaults, with the Q-LS1 fallback expressed as a one-line default choice for 3D/2D+3D.
  - The smoke fixture table; smoke no longer reads `launch.toml`.
  - `LAUNCHER.md` §2 table rows (following whichever Q-LS1 branch is taken), the §4 `seat` opt-out, and the
    smoke rule.
- [ ] **Deterministic validation (Gate 2, real server, stub renderer).**
  - Each mode's play run joins its default world with the expected seats; the save folder is `saves/<world>/`.
  - A pre-existing `saves/social-cafe/` is untouched (byte-compare) and still resumes when `launch.toml` names it.
  - **Reseating:** a pre-existing `market-town` save played as `carol` resumes with the stub joined as `visitor`.
    A join on `carol` reports `took_over: "hosted"`, the world instance id is unchanged, and Carol's wallet and holdings are equal before and after the resume, so
    nothing was reset.
  - **Smoke:** `--smoke=2d` drives `carol` on `market-town`; `--smoke=3d` and `--smoke=both` host `social-cafe`
    with `--agent alice`, with unchanged verdict lines. A `launch.toml` naming another world/seat does not change
    any smoke run.
  - Config unit cases for defaults and overrides.
- [ ] **Gate 2, packaged (Q-LS7), macOS, one heavy job at a time under `cast-slots/with-slot.sh`.**
  - `scripts/package.py` assemble from the exact head.
  - In one session, a Social Café 3D first view and a Market Town 3D first view (same flags, default graphics).
  - The packaged `--slice-market` probe, recorded in the ledger before it runs:
    - **Server:** the bundle's `runtime/mineworld server <runtime>/worlds/market-town --listen 127.0.0.1:0
      --save <scratch> --town --solo`, with a test admin token in its environment only.
    - **Client:** `<runtime>/clients/MineWorld.app/Contents/MacOS/MineWorld --headless -- --client=3d
      --root=<runtime> --server=<addr> --invite=<invite> --world-key=market-town --seat=visitor --slice-market
      --steps=<file> --settings=none`.
    - **Steps file:** the exact `3d-market-door-steps` JSON that `tools/cli/tests/client_3d_market.rs` writes for a
      fresh Market Town. It is copied from that test's scratch, and its SHA-256 is recorded.
    - **Pass:** exit 0 and the probe's own pass summary line.
  - A far-field screenshot and a 300-frame frame-time sample (Q-LS7 (3), (4)).
  - Each result is recorded PASS/FAIL/INCONCLUSIVE against Q-LS7.
  - A miss applies the Q-LS1 fallback in the same commit; that is the known branch, not an escalation.
  - Linux and Windows packaged 3D Market Town runs: NOT RUN, because no local machine exists for them. Recorded so.
- [ ] **LLM logic review.** Ownership (launcher only), no fixture drift, readable defaults, honest measurements.

### C2 — single-player play runs start `--solo` (P3)

- [ ] **Implementation.**
  - `Hosting::Played { admin, solo }`: `solo` for 2D/3D play runs, not for 2D+3D.
  - `configure_server` adds `--solo`.
  - `LAUNCHER.md` §7.1, `HOST.md` §6.1, and the `ARC-78` dated note superseding #549's "no `--solo`" sentence.
- [ ] **Deterministic validation (Gate 2, real server).**
  - 3D play run: read the current scale from `welcome`. An authenticated `POST /admin/clock {"time_scale":25}`
    returns 200 (the server is `--solo`). Then restore the observed scale and see the `clock` frame. No shipped
    client sends 25 (LS-I3).
  - The #549 2D+3D case still returns 409 for 48×.
  - Smoke's server log shows no admin surface and no solo.
  - A restart of a solo world resumes the day-length scale and drops a fast-forward scale (`ARC-69` item 8).
  - The token is absent from argv and logs (the #549 checks, extended to solo).
- [ ] **Gate 2, packaged World tab on a `--solo` run.** Exported 3D (or 2D) play run on macOS:
  - the World tab is enabled;
  - 24× is applied and confirmed by the server's `clock` frame (client log or a read-only save host-journal read);
  - recorded PASS/FAIL.
  - This also closes #549's open exported World-tab evidence if that has not been run by then.
- [ ] **LLM logic review.** A single owner of the solo decision (`Hosting`). No inference from connection count
  (`ARC-69` item 5's rejected alternative). No client changes.

### C3 — P1's resident cycle, a dedicated packaged run (evidence and ledger only; run once on the final head)

- [ ] **Run.**
  - Packaged 3D launcher (2D if the Q-LS1 fallback was taken), fresh `--user-dir`, Market Town from genesis
    (00:00).
  - Immediately after the first view, apply 24× in the World tab (the C2 packaged item) and confirm it by the
    `clock` frame.
  - Leave the game running, untouched, until world time 08:00 or the wall budget, whichever comes first, then
    close normally.
  - **Wall budget: 1800 s**, an explicit exception to the 600 s per-run budget, used once. The window 00:00 → 08:00
    at 24× is 20 wall minutes, plus startup.
  - One shared heavy slot.
- [ ] **Pass condition (precise).** From the closed save, `mineworld biography market-town --save <save> --person
  <key> --json` for every non-Alice resident seat. PASS if **at least one** resident shows, in this order with
  increasing world time:
  1. arrival at the `place` of their routine's `home` entry;
  2. `fell-asleep`;
  3. `woke`;
  4. departure from that place;
  5. arrival at the place of their next routine entry.

  For example, Bob starts in the café at 00:00 while his routine says apartments from 20:00. He should walk home,
  sleep and go back to the café after 07:00 (`worlds/market-town/people/bob.yaml`).
  - The resident and the instants are recorded.
  - Missing any step for every resident: **FAIL**.
  - Budget reached before 08:00: **INCONCLUSIVE**.
  - No scale above 24× is used: walking stays at one stride per wall second (`ARC-341`), so a faster run would
    test lag, not routines.
- [ ] **LLM logic review.** The sequence is read from the save, not from client drawing, and the resident is not
  chosen after the fact to fit. Each resident's verdict is listed.

Integration checkpoint: a packaged 3D play run of the C1-chosen world, `--solo`, with the World tab's 24× confirmed
(C2 packaged item) and C3's resident cycle PASS. This is the state nightlife N1-C3's packaged night check and N2-C4
wait on.

## 6. Test ownership, CI scope, review and load

- **Launcher crate tests:** `cargo test -p mineworld-launch`, after `cargo build -p mineworld-cli`, run locally on
  macOS. They own the argument, world, seat, save and token oracles.
- **Remote required check:** `fast` (ubuntu-24.04) on the exact head.
- **Platform evidence:** `test-macos`, `test-windows` and `platforms` run only on a full run (`milestone` label or
  dispatch, `ci.yml` "mode == full"). Applying that label is the operator's choice. Without it, Windows launcher
  tests are recorded NOT RUN with that reason, never as passes.
- **Packaged gates (C1, C2, C3):** macOS only; Linux/Windows packaged runs NOT RUN (reason above).
- **Review:** independent red-team review of this design before freeze, and of the exact head before READY FOR
  OPERATOR REVIEW.

## 7. Execution contract (filled on freeze)

```text
Owner / worktree:      one implementation session; new worktree under /Users/yuema137/mineworld-worktrees/
Affected paths:        tools/launch/**, clients/shared/HOST.md, docs/decisions/ARC-78.md, this ledger
                       (no edit to any frozen design, including pr-nightlife.md)
Authorized:            implement C1–C2, targeted tests, the packaged gates of C1/C2, C3's single 1800 s run on the
                       final head, semantic commits, push, PR
Not authorized:        server/kernel/System/client-script/world-content changes; scale offers >24x; LAN hosting;
                       merge; `milestone` label
Budgets:               <=600 s per heavy run except C3 (one run, <=1800 s), one shared heavy slot, no paid API calls
Escalate:              any need to change the server's --solo semantics, or a smoke verdict that changes.
                       A Q-LS7 miss is NOT an escalation: it applies the pre-decided Q-LS1 fallback
```

## 8. Evidence and readiness

None yet. This is a design-only draft.

## 9. Red-team pre-freeze review — disposition (revision 2)

| Finding | Disposition |
| --- | --- |
| F1 smoke seat drift | Fixed: Q-LS5 pins world and seats; smoke ignores `launch.toml` (stated as a behaviour change); a `--smoke=2d` carol oracle is added. |
| F2 P1 resident-cycle clause dropped | Revision 2 handed it to N1-C3; superseded in revision 3 (D1): proven here by C3. |
| F3 3D Market Town acceptance not fixed before measuring | Fixed: Q-LS7 sets the bound (1.25 × same-session café, ≤ 240 s), the probe, far-field and frame-time checks, and the pre-decided fallback in Q-LS1. |
| F4 silent reseating of 2D saves | Fixed: the consequence is stated in Q-LS6, LS-I4 and the LAUNCHER.md opt-out; a C1 reseating oracle is added. |
| F5 ARC-69 wording; #549 text | Fixed: §1.2 and LS-I1 name both permissions per the nightlife amendment and cite ARC-69 Consequences; an explicit dated supersession of #549's sentence; dependency on #549 ratification stated. |
| F6 admission and LM-profile consequences | Fixed: moved into Q-LS2. |
| F7 Q-LS3 wording; "restores 12" | Fixed: "no process of another user"; the restore uses the observed scale. |
| F8 platform coverage claimed | Fixed: §6 names the full-run jobs and records the NOT RUN cases. |
| F9 overlap with PR-B | Fixed: Q-LS1 states the defaults are owned here and PR-B's profile only overrides. |
| F10 commit-plan completeness | Fixed: the C2 packaged World-tab item; the integration checkpoint is tied to it; the LAUNCHER.md carol row is named. |

### Delta re-review @ 2852969d — disposition (revision 3)

| Finding | Disposition |
| --- | --- |
| D1 P1 hand-off amends frozen N1-C3 | Withdrawn. No frozen design is edited. P1's resident cycle is proven by this PR's C3: one packaged run of at most 1800 s at 24×, from genesis to 08:00, with a precise save-read pass condition. Stated in Q-LS1. |
| D2 Q-LS7 under-specified | Fixed in Q-LS7: median of 3 interleaved runs per world; <5 % margin INCONCLUSIVE; warm shader cache with a discarded warm-up run; fresh `--user-dir`; timer from launcher start to the `client scene visible` log line; far field judged by the operator against the no-sky-only-sector criterion; frame-time build and window named; the reviewer re-derives the numbers. |
| D3 no packaged `--slice-market` recipe | Fixed in C1: exact server and client commands, steps-file origin and hash, pass line. |
| D4 fallback consequences | Fixed in Q-LS1: P1 recorded partial; LAUNCHER.md rows follow the branch; N2-C4's 3D sleep check needs `world = "market-town"` (noted here, nightlife text untouched). |
| F4 nit (nothing reset) | Noted: C1's reseating oracle also compares Carol's wallet and holdings before and after. |
