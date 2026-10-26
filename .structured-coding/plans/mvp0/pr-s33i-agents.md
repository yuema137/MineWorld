# PR S33-i — An MCP agent plays Lakeside like a person, and a 3D client sees it

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00)

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874; the agent surface
                        re-read at origin/main @ 94183574 (#261 merged: one `act` tool)
Approved by / evidence: the operator, 2026-10-19, in the primary session: every question answered with its recommendation; Q-S33h-1 OpenAI image cap US$8.00
Implementation base:    origin/main at C0
Start condition:        S27-d merged (MCP stdio; met in part — the stdio adapter and #261's `act` are on main,
                        C0 confirms); S29-a merged (the halt's places); S29-g merged (`PlaceLine.roles`); S33-c
                        merged (3D Lakeside on the server path); this design frozen; the lane lead's dispatch.
                        S29-a, S29-g, S33-c NOT met at 2026-10-18
Execution contract:     §12
Implementer:            Haiku-class from this commit plan (step-33 §12.1)
Lifecycle:              DESIGN FROZEN 2026-10-19
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §§1 (outcome
6), 7, 9.5, 11 (AC-S33-8), 12.1 (the S33-i row) · **Binding steps:**
[`step-27-agent-players.md`](step-27-agent-players.md), [`pr-s27d-mcp-stdio.md`](pr-s27d-mcp-stdio.md),
[`step-29-station-landmark.md`](step-29-station-landmark.md) (S29-g, §9.4) · **Binding parents:** `CLAUDE.md` §§1,
4 (items 3, 10); operator memory "Agents are first-class players".

**Decision ids.** `ARC-175` (§5.3: every world that ships a 3D depiction ships one recorded MCP scenario with a 3D
observer, run in CI), written by C1. No other id.

No live language model is called: the agent is a deterministic script over MCP (step-27's recorded-agent
pattern). No money is spent.

---

## 0. Binding inputs, exactly restated

- **Scope (step §12.1, S33-i row).** "the §7 scenario in `clients/agent` over MCP with a 3D observer". Depends on
  S27-d; S29-g; S33-c. Checkpoint: AC-S33-8. Adversarial: "an agent away from the halt sees no `roles: station`
  exit leading there".
- **The scenario (step §7).** "an agent seat (Nils) observes on the pier (`roles` contains `landmark`), calls
  `walk_to` the halt, observes `roles` `station`, sees **no offers** at the ticket window, and walks back. A 3D
  client connected to the same server draws Nils's person moving along the lane, and the server's place changes
  match in both."
- **Same path (step §4.8, S27).** The agent uses the same server, the same Observations and the same intents as a
  human; nothing in S33 is agent-specific.
- **Headless first, deterministic, LM-independent** (`CLAUDE.md` §4 item 10).
- **All platforms.** The agent suite runs on Linux, macOS and Windows; the 3D observer runs where the nightly
  Godot layers run.

---

## 1. Goal

A script that knows nothing about Lakeside except what MCP tells it joins as Nils, finds out from its own
observation that it stands on the village's landmark, walks to the place whose role is `station`, finds the
ticket window offers nothing, and walks back. At the same time a 3D client joined as Elin watches Nils walk along
the lane, and every place change the server records is the one the 3D client shows.

---

## 2. Scope

### 2.1 In scope

1. `clients/agent/tests/test_play_lakeside.py` (new): the scenario over MCP stdio against the real server
   (`realserver.py`'s pattern), with oracles from the server (`/status`, the transcript, `mineworld perceived`).
2. `clients/agent/tests/realserver.py`: a `lakeside()` start helper (the world's pack roots), beside
   `market_town()`.
3. `tools/cli/tests/agent_with_3d_observer.rs` (new, `#[ignore]`, Godot): hosts Lakeside, runs the scenario's
   script and `./mineworld-slice --server=… --invite=… --world-key=lakeside --seat=elin --observe-log=…` together,
   then compares the server's place changes for Nils with the 3D client's figure log.
4. `clients/3d-spike/scripts/slice/slice_link.gd`: an `--observe-log=<file>` option that writes, per observation,
   each drawn figure's entity id and bound place (no new behaviour; a log).
5. `scripts/ci_layer.py`: the agent test in its existing Python layer; the observer test in nightly `clients`.
6. `docs/DECISIONS.md` (or `docs/decisions/`): `ARC-175`.

### 2.2 Not in scope

Any new MCP tool or act kind; any agent-specific server path; `roles` itself (S29-g); live models.

---

## 3. Audit (`origin/main @ 94183574` for the agent surface, A-1 … A-3; `a1e05874` for the rest)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | MCP operations: `seats`, `join`, `observe`, `inspect`, `events`, `wait`, `act`, `clock`, `leave`; `act.kind` ∈ `say`, `walk_to`, `move`, `stop`, `attempt` (#261: "one `act` tool instead of five") | `clients/agent/src/mineworld_agent/mcp_server.py:122–141`; `models.py:762–766` |
| A-2 | `PlaceLine` exists; it has **no** `roles` field yet (S29-g adds it) | `models.py:185` |
| A-3 | The real-server test pattern: `realserver.py` (`market_town`, `seated`, `Transcript`, `Wire`, `biography`, `perceived_export`); "No sleep stands in for a stream wait; every wait has a ceiling" | `clients/agent/tests/test_play_market_town.py:1–40` |
| A-4 | Lakeside: every person is a seat; Nils starts on `pier`; Elin in `cottages`; no `economy` (so no offer at a ticket window) | step A-18, A-16 |
| A-5 | S29-g's row: "`PlaceLine.roles`; the §9.4 scenario in `clients/agent` … the adapter must not invent a service state" | `step-29-station-landmark.md:741` |
| A-6 | The 3D link prints `SHOWN` and binds figures by disclosure; it has no per-figure log | `scripts/slice/slice_link.gd` (S33-c / 16c) |
| A-7 | Presence's perception rule: "entities — the place you are in, and everybody this pack knows to be in it". An observer sees nobody in another place | `systems/presence/src/observe.rs:9–14` (at a1e05874) |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33i-1** | **The step's "calls `walk_to` the halt" is now an `act`** (#261, A-1) | The script sends `act { kind: walk_to, … }`; no tool is added |
| **F-S33i-2** | **`roles` does not exist** (A-2) | The scenario needs S29-g; until it merges, nothing here can pass (start condition) |
| **F-S33i-3** | **The 3D client has no machine-readable record of where it draws others** (A-6) | `--observe-log` (§2.1 item 4): a log, not a behaviour |
| **F-S33i-4** | **An observer sees only its own place** (A-7). The step's "the server's place changes match in both" cannot mean every change: Elin's client cannot see Nils on the pier or at the halt | The observer stands in the lane and the comparison covers the lane crossings it can perceive (§5.2) |

### 3.3 What does not exist

No Lakeside agent scenario; no test with an agent and a 3D client on one server.

---

## 4. Reuse

`realserver.py`, the MCP relay and the guarded adapter (`clients/agent/tests`); the 3D link; RL-e0's Godot test
harness (`client_far_field.rs`'s runner). Nothing new is adopted.

---

## 5. Design

### 5.1 The script (deterministic, no model)

```text
seats → join nils
observe                       assert here.roles ∋ "landmark"; assert no exit's roles ∋ "station"   (adversarial)
act walk_to <the exit toward the lane>; wait arrived
observe                       find the exit whose roles ∋ "station"   (from disclosure only)
act walk_to that exit; wait arrived → here.roles ∋ "station"
act walk_to the exit whose roles ∋ "ticket-office"; wait arrived
observe                       assert offers == []                       (no economy: no service)
walk back: tickets → station → lane → pier; at each step wait arrived; assert here at the end = the pier
leave
```

The script reads place keys only from what it observed; it holds no Lakeside key (the same discipline as S33-c's
check (d)). Every wait has a ceiling (A-3).

### 5.2 The 3D observer

An observer perceives only its own place and the people in it (A-7). Elin starts in the cottages' yard, so a 3D
client seated as Elin would never see Nils on the pier, at the halt or in the ticket bay. The test therefore:

1. hosts `worlds/lakeside` and joins the 3D client as Elin with `--observe-log`;
2. walks Elin to the lane with the client's own intents (the link probe's walk, as a human would) and waits there;
3. runs §5.1's script for Nils;
4. after the script leaves, compares two ordered lists: the server transcript's place changes of Nils that **enter or
   leave the lane** (pier → lane, lane → station, station → lane, lane → pier), and the 3D log's appearances and
   disappearances of Nils's figure in the lane. The lists must be equal, each 3D entry within 2 s of world time of
   the server's; while Nils is in the lane, his figure's logged positions must move from the pier's door toward the
   station's door (outbound) and back (inbound).

What the observer cannot perceive is not compared: that is the world's rule, not a gap in the test.

### 5.3 `ARC-175`

Every world that ships a 3D depiction ships one recorded MCP scenario with a 3D observer, run in CI. Agents and
humans are then checked on the same path, world by world.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-i-1 (I-S33-1) | No server, protocol, kernel, contract or System change; no new MCP tool | diff scan; `tests/golden/tools.json` unchanged |
| I-i-2 | The script holds no Lakeside place key | a scan of the test file for `worlds/lakeside/places/*` stems |
| I-i-3 | LM-independent and seeded | the test runs offline |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Agent suite | AC-i-1, AC-i-2, AC-i-3 | `clients/agent` pytest, Linux, macOS, Windows (its existing layer) |
| Godot pair | AC-i-4 | `agent_with_3d_observer.rs` (`#[ignore]`), nightly `clients` |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-i-1 | The script completes: landmark seen, station reached, no offers at the window, back on the pier | AC-S33-8 |
| AC-i-2 | On the pier, no exit carries `roles ∋ station` (the halt is not adjacent) | §12.1 adversarial |
| AC-i-3 | M-i-1 (a planted offer in the fixture's ticket-window observation) → the test fails at the "no offers" step, naming the offer | S29-g "must not invent a service state" |
| AC-i-4 | The 3D observer, standing in the lane, logs Nils's four lane crossings in the server's order and within 2 s of world time, and his figure moves between the two doors (§5.2); M-i-2 (the log drops one crossing) fails naming the step | AC-S33-8 |

---

## 9. Commit plan

Procedure: [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md) §9.0, from
`/Users/yuema137/mineworld-worktrees/impl-s33i`; the agent suite with `uv run --locked pytest clients/agent/tests/…`
(one file).

- **C0 — Base, prerequisites, ledger.** S27-d, S29-a, S29-g, S33-c merged; `roles` present.
  `docs(plan): S33-i — open the ledger at the base`.
- **C1 — `ARC-175`.** `docs: ARC-175 — every 3D world ships an MCP scenario with a 3D observer (S33-i)`.
- **C2 — The scenario.** §5.1; AC-i-1…3. `test(agent): an MCP agent walks Lakeside from the pier to the halt (S33-i)`.
- **C3 — The observer log and the pair test.** §5.2; AC-i-4. `test(3d): a 3D client sees the agent's walk as the
  server records it (S33-i)`.
- **C4 — Close-out.** Push; PR; CI; repair; train-ready. `docs(plan): S33-i close-out`.

---

## 10. Requirements this PR places on other PRs

- **R-S29g-i-1 (on S29-g).** `PlaceLine.roles` on the current place and on each exit's destination, from
  disclosed tags only.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §7 | "calls `walk_to`" → "acts `walk_to`" (#261, F-S33i-1); "the server's place changes match in both" → "the lane crossings the observer can perceive match the server's" (F-S33i-4) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-i — an MCP agent on Lakeside with a 3D observer
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33i-agents.md
WORKTREE / BRANCH:       /Users/yuema137/mineworld-worktrees/impl-s33i, mvp0/pr-s33i, one session
START CONDITION:         per the header (NOT met at 2026-10-18)
APPROVED SCOPE:          §2.1 items 1–6
NEVER:                   a live model; a new MCP tool or act kind; a Lakeside key in the script; as pr-s33d §12
MATERIAL STOPS:          a needed server or protocol change; `roles` absent after S29-g
ENDPOINT AUTHORITY:      implementation, commits, push, PR, CI repair: authorized once started;
                         merge: through a merge train under the charter
STOP CONDITION:          train-ready
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-i-1 | The pair test is slow or flaky on a loaded runner | ceilings on every wait; the comparison tolerates 2 s of world time; nightly only |
| R-i-2 | S29-g slips | the scenario waits; nothing else in S33 does |

---

## 14. Operator questions

None. The scenario is the step's, unchanged except for #261's tool name (F-S33i-1), which is a bounded correction.

---

## 15. Ledger (live during implementation)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | cd7868f5 | [x] base origin/main @ 8be7b293; prerequisites confirmed | [x] S27-d (#239), #261 (`act`), S29-a (#375), S29-g (#358), S33-c (#381, #397, #400) on main; `PlaceLine.roles` at `clients/agent/src/mineworld_agent/models.py:225`, filled by `view.py:177` (`roles_of`) | N/A (ledger only) |
| C1 | 714a770b | [x] `docs/decisions/ARC-175.md`; index rewritten | [x] `check_decision_ids.py --write-index`: 249 ids, all distinct | [x] self-review: matches §5.3; names the files and the comparison rule |
| C2 | 9670e5d0 | [x] `test_play_lakeside.py` (scenario, M-i-1, I-i-2 scan, program entry); `realserver.lakeside()` | [x] PASS: `uv run --locked pytest clients/agent/tests/test_play_lakeside.py` 3 passed (real server, 50 s); ruff, ruff format, pyright clean | [x] self-review: no place key (scan), no model, every wait ceilinged (the `walk` helper); F-S33i-5 |
| C3 | 3234c488 | [x] `--observe-log` (`slice_link.gd`), `--slice-observe` (`slice_probe_world.gd`, launcher `--observe`), `agent_with_3d_observer.rs`, `ci_layer.py` clients-probes | [x] PASS: pair `--ignored` run 64 s locally, server `[Enters(18), Leaves(30), Enters(47), Leaves(58)]` = drawn, figure (18000,400)→(29546,2989) and (29600,3000)→(18080,410); comparison tests 4 passed; `./mineworld-slice --world=lakeside --link` "all link checks pass"; `check_client_rules.py` PASS; clippy `-D warnings` clean | [x] self-review: the log changes nothing drawn or sent; no place named in client code; F-S33i-6 |
| C4 | 1b2017b9 (+ this ledger line) | [x] ledger; push; PR #480 | [x] `fast` PASS on 1b2017b9 (run 40410973498) | N/A (ledger) |

Mutations: M-i-1 run as `test_a_planted_offer_fails_the_window_step` (a planted `buy` fails naming
`(3, 'buy', 'P9')`); M-i-2 run as `a_log_that_drops_a_crossing_fails_naming_it` (fails naming "crossing 4"),
plus a late crossing (names "crossing 2") and a figure that never moves. Findings: F-S33i-1…F-S33i-6.

**F-S33i-5 (bounded correction, C2).** §5.1's `offers == []` cannot hold: the seat's own acts (`move`,
`walk-to`, `put-down`, `eat`, `drink` of what Nils carries) are offered in every place with no target. The
window step asserts that no offer is the window's: none with a target here and no `buy`; and `inspect` shows
no `shop` component on the place (S29-g's rule). M-i-1 still fails it by name.

**F-S33i-6 (bounded correction, C3).** §5.2 step 2's "the link probe's walk" is a new probe mode,
`--slice-observe`: the station walk's `_cross` to the depiction's street, then standing there until a stop file
appears. The pair needs the Python workspace, which the Windows `clients` job does not set up, so it runs in the
nightly `clients-probes` layer (Linux and macOS, which already install uv) instead of `clients`. No workflow
file changes.

**C0 (2026-10-26).** Start condition met. `roles` is present on main: `PlaceLine.roles: list[RoleTag]`
(`models.py:225`), the role vocabulary `station, stop, platform, ticket-office, landmark` (`roles.py`), and
S29-g's own Lakeside run (`test_play_station_mcp.py::test_lakeside`). Lakeside at the base: Nils starts on
`pier` (tags carry `landmark`); the pier's one passage leads to `lane`; `lane` has no passages of its own
and is reached from every place; `station` → `lane`; `station-tickets` and `station-platform` → `station`.
The 3D observer is joined as `elin`, who starts in `cottages`, one door from the lane.

**Design review (2026-10-18).** One independent Opus reviewer, verdict REQUEST CHANGES, no blockers; #261's `act`
confirmed. Applied before the PR opened: the observer stands in the lane and compares only what it can perceive
(§5.2, A-7, F-S33i-4); the audit's two bases named.
