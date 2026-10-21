# PR 13e — Many players, a living town (2D client)

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 0 (2026-10-20), this file as first committed on docs/pr-design-13e
Approved by / evidence: the operator, primary session 2026-10-20 ("every question as recommended"),
                        relayed by the coordinator to the implementation session.
Implementation base:    origin/main @ c9e0dde0 (#420) at C0 of the implementation session; planned
                        against origin/main @ f92e3cad (#364).
Execution contract:     §12 (filled at C0)
Lifecycle:              FROZEN — READY FOR OPERATOR REVIEW (implementation complete; ledger §13)
```

**Effort:** `mvp0` · **Step:** S12 [`step-13-client-2d.md`](step-13-client-2d.md) §9, the 13e row
(§4.6 reconnect, §4.7 multiplayer, §4.8 a living town, §6.1 R-S11-1 … R-S11-6, §6.2 R-S10-1, §7 M-5, M-6,
§8 I-4) · **Parent:** [`overall.md`](overall.md) S12 · **Builds on:** S11 ([`step-12-server.md`](step-12-server.md),
`server/PROTOCOL.md` revision 2), TW-e + SET-c ([`pr-tw-e-set-c-world-clock-clients.md`](pr-tw-e-set-c-world-clock-clients.md)),
S20 ([`step-20-client-settings.md`](step-20-client-settings.md)), S27 ([`step-27-agent-players.md`](step-27-agent-players.md)),
13a (§14 of step-13), 13b (§15) · **Binding specs:** `CLAUDE.md` §§1.1, 2–4; `docs/ENGINEERING_RULES.md`
§§2–4, 7–12; `docs/ENGINEERING_STANDARDS.md`; `docs/REUSE_POLICY.md`; `server/PROTOCOL.md` §§4.1, 4.2,
5.1, 5.2, 5.5–5.9; `clients/protocol/ADOPTION.md` §§2, 6.1; `clients/2d/PRESENTATION.md` §§4, 7;
`clients/shared/SETTINGS.md` (INV-SET-2) · **Lane charter:** `/Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md`
(speed first; no world-specific hard-coding; tools first).

**Planning worktree:** `/Users/yuema137/mineworld-worktrees/design-13e`, branch `docs/pr-design-13e`. This
session writes this file and one pointer in step-13 §9. It implements nothing.

---

## 0. The short version

When 13e was planned (2026-10-08) almost everything it needed was missing from the server. Since then
S11, TW-e and S27 delivered most of it. **What is left is small and is all in the 2D client:**

1. A **join screen**: address, a seat picker filled from `GET /status`, the invite, a nickname. Today the
   client joins whatever `--seat` says, and the launcher defaults to `carol` and `--agent alice`, which
   do not exist in `worlds/lakeside`.
2. **Seats as the server now runs them**: show who took over whom, offer "Take over" after
   `seat_occupied`, stop the reconnect loop after `taken_over`, and resume a dropped seat with its
   `resume` secret instead of being locked out by the server's 30 s hold.
3. **Speech bubbles and activity notices from `observation.events`**, chosen and worded by the
   Presentation Pack, so the client names no event type.
4. **A living town by default**: the launcher's played mode passes `--town`, as `mineworld-slice` does.

The clock at the server's time scale, M-5 and M-6 are **already delivered** and leave this PR (§2).

---

## 1. Goal, in one paragraph

Several people play one world at once, each in their own `mineworld-2d` window, and the world around them
lives without them. A player opens the client, picks a free person from a list, and joins. They see the
other players and the town's people walk, talk and start activities: a line spoken near them appears as
a bubble over the speaker, and an activity that starts near them appears as a short notice. If they take
over a person the town was driving, they are told so. If two players want the same person, the second is
told the seat is taken and may take it over on purpose; the first is told who took it and is returned to
the join screen. A dropped connection comes back to the same person without anyone else noticing. All of
this works on any world that has seats — `market-town`, `social-cafe`, `lakeside` — because the client
reads the seat list, the events and the wording from the server and the packs, never from its own code.

---

## 2. Audit — what is already on `origin/main @ f92e3cad`

Read in this session. "Delivered" means merged and exercised by a check named in the evidence column.

| 13e item (step-13 §9) | State | Evidence |
| --- | --- | --- |
| **M-5** join credentials in the module | **Delivered** (S11-A, S11-B) | `clients/protocol/mineworld/world_client.gd:219` `connect_to_world(address, seat, invite, nickname, take_over := false)`; commits `a550feed` (S11-A C7, invite + nickname, "every call site and launcher"), `2998fb84` (S11-B, `take_over`, opt-in `reconnect` with `resume`). |
| **M-6** events in observations | **Delivered** (S11-C) | `observation.gd:186` `events()`, `:42` `events_of()`; `delta.gd:52` events replace on every delta; `world_client.gd:56,507` `perceived` stream, `:257` `perceive_from`; commit `1931642b` (C-C9a). Checked by `clients/protocol/checks/perceived_check.gd:114`, `rev2_check.gd:180`. |
| R-S11-1 invite + nickname | **Server delivered**; the 2D client takes them **only as arguments** | `PROTOCOL.md` §4.1; `clients/2d/scripts/app.gd:157-158` `link.open(--server, --seat or "carol", --invite, --nickname or "2d-player")`; `mineworld-2d:52,54,72-73`. No screen collects them. |
| R-S11-2 seat occupancy | **Server delivered** (`seat_occupied`, holds, `taken_over`, `superseded`); **the 2D client mishandles it** | `PROTOCOL.md` §4.2, §5.5 line 490, §5.6 lines 521-523. `clients/2d/scripts/link.gd:26-27` `FINAL` lacks `taken_over`, `superseded`, `kicked`, so a player whose seat was taken retries for ever; `link.gd:73-74,98-106` re-joins **without** `resume`, so during the 30 s hold (`tools/cli/src/main.rs` `--hold`, default 30) its own dropped seat answers `seat_occupied` and the link sits in `JOINING` with no further retry. `tools/cli/tests/client_2d.rs:259-262` works around it with `--hold 0`. |
| R-S11-2 "the seat listing says which seats are free" | **Not delivered, by design**: `/status` is composition only and names no player | `PROTOCOL.md` §5.7 ("never its state"); free/held/hosted is only in `GET /admin/seats` (§11.2, admin token). See Q-13e-2. |
| R-S11-3 takeover of an agent-driven Person | **Server delivered**; the client **shows nothing** | `welcome.took_over` `"hosted" / "held" / "connection"` (`PROTOCOL.md` §5.1 line 299); read by the module (`world_client.gd:159,567`); `grep took_over clients/2d` finds nothing. |
| R-S11-4 events → speech bubbles, activity notices | **Server and module delivered**; **the 2D client reads no event** | `grep -e events -e perceive clients/2d/scripts` finds no use. The only "notice" is S35's lateness, read from a component (`app.gd:391-412`). No bubble exists in either client (`grep -i bubble clients` finds only `clients/agent` tests). |
| R-S11-6 time scale; "the clock at the server's time scale" | **Delivered** (TW-e C4, 2026-10-18) | `app.gd:125,428-431` `clock_changed` → `MineWorldWorldTime.on_clock`; `app.gd:279-283` the HUD clock ticks between frames; commit `2b924baf`. `worlds/market-town/world.yaml:226-227` and `worlds/social-cafe/world.yaml:137-138` `hosting: time_scale: 12`. **Leaves this PR.** |
| R-S10-1 hosted agents that take initiative | **Server delivered** (`--town`, S11-B SD-B9); **the 2D launcher does not ask for it** | `tools/cli/src/main.rs` `--town` "Drive every other seat with the paced rule controller whenever no player holds it"; `mineworld-slice:267` passes `--town` when played; `mineworld-2d:56,157,167` passes only `--agent alice` (reactive, answers talk only). |
| Multiplayer on one machine | Works with hand-typed `--invite` | `mineworld-2d:166-167` hosts `--solo` (loopback only); the invite is read from the server's log line (`:172-187`) and handed to this one client only. A second window must dig the invite out of `clients/2d/.save/server-<world>.log`. |
| Generic worlds | **Defect**: the launcher's defaults name market-town's people | `mineworld-2d:52` `seat="carol"`, `:56` `agent="alice"`; `worlds/lakeside/world.yaml:97-107` seats `alba … nils` (no `carol`, no `alice`). |
| Agents as players (S27) | Delivered; affects 13e only as "another holder of a seat" | `link.gd:25-27` `not_granted` already final (commit `668f6851`). An MCP agent holding a seat is, to a 2D player, a `seat_occupied` like any other. |

**Conclusion.** Two of the six named items (clock, M-5/M-6) are done. The server side of the other four is
done. This PR shrinks to a client-only PR: one new screen, a corrected link policy, an event presenter,
launcher defaults, and one small additive change to the shared module (M-8, §5.5).

---

## 3. Scope

### 3.1 In scope

- **S-1 Join screen** (`clients/2d/scripts/hud/join_screen.gd`, new): address, seat picker from
  `GET /status`, invite, nickname; shown when no `--seat` is given (Q-13e-1) and whenever a join ends for
  good.
- **S-2 Link policy** (`link.gd`): use the module's `reconnect` (resume within the hold); treat
  `taken_over`, `superseded`, `kicked` as final and return to the join screen; after `seat_occupied` offer
  "Take over" (Q-13e-2).
- **S-3 Seat messages**: a toast for `took_over` `hosted` / `connection` / `held`, and for the closing
  reasons, worded by the pack.
- **S-4 Event presenter** (`clients/2d/scripts/scene/speech.gd`, new): bubbles over drawn speakers and
  notices, from `observation.events`, by the Presentation Pack's `events` table (Q-13e-3, Q-13e-4).
- **S-5 Launcher** (`mineworld-2d`): no default seat, no default `--agent`; played mode passes `--town`
  (Q-13e-5); `--join-local` for a second window on the same machine (Q-13e-7).
- **S-6 Shared module M-8**: `closing {lagged}` resumes like a drop when `reconnect` is on (Q-13e-8).
- **S-7 Tests and harness scenarios**: `--drive=join`, `--drive=blink`, `--drive=listen`; a new
  `tools/cli/tests/client_2d_many.rs` with two real clients on one hosted server.

### 3.2 Not in scope

- Any change under `kernel/`, `contracts/`, `systems/`, `server/`, `worldpack/`, `persistence/`,
  `authoring/`, `sdk/`, `cognition/`, `worlds/` (I-4). No protocol change.
- The clock and time scale (delivered, §2). M-5, M-6 (delivered).
- The 3D client's join screen and bubbles (S14 / S15 own them; §5.1 keeps the screen liftable).
- `AC-5` **asserted** (biography, relationships, holdings unchanged across a takeover, checked from the
  save) stays in 13f, as step-13 R-S11-3 says ("13e shown, 13f asserted"). 13e shows it and checks
  identity.
- LAN hosting from the launcher (it hosts `--solo`, loopback only). A world started by hand with
  `mineworld server` is joined from the join screen like any other.
- Remembering the last address or nickname. `clients/shared/SETTINGS.md` INV-SET-2 forbids writing them
  to the settings file; the screen is prefilled from arguments only.
- Object animation along `path` (13d).

### 3.3 Invariants this PR holds

| ID | Invariant |
| --- | --- |
| **I-13e-1** | **No rule in the client.** The client never decides whether a seat is free, whether a takeover is allowed, or who may hear a line. It asks; the server answers by code. `scripts/check_client_rules.py` R1–R7 stay clean, and a new **R8** (C5) holds the event table's names out of the scripts. |
| **I-13e-2** | **No world or pack vocabulary in the scripts.** No `.gd` under `clients/2d/scripts/` holds a seat key, a person's name, an event type (`spoke`, `group-activity-started`, …) or a payload field name (`utterance`, `kind`, …). Seats come from `/status`; event types and fields come from the pack's `events` table. |
| **I-13e-3** | **One path for one player and many.** A single player on `./mineworld-2d` and four players on one server use the same join frame, the same link and the same presenter. No "solo" branch in the client. |
| **I-13e-4** | **The client is removable** (step-13 I-4). Paths changed are inside the allowed set of `check_client_rules.py --scope`, plus the two additions C5 makes to it (`client_2d_many.rs`, `clients/protocol/mineworld/world_client.gd` for M-8, `clients/protocol/ADOPTION.md`). |
| **I-13e-5** | **Secrets stay secret.** The invite and the `resume` secret are never printed, logged, drawn as text or written to a file by the client or the launcher. The invite field is a secret `LineEdit` (`secret = true`). |
| **I-13e-6** | **A bubble is a fact, once.** Each bubble or notice comes from one event the server sent, keyed by its `id`; the same `id` is never shown twice on one world instance, across reconnects included. |

---

## 4. Reuse (`REUSE_POLICY.md`; both halves of the question)

| Need | Options compared | Verdict |
| --- | --- | --- |
| Join form (text fields, list, buttons) | **Godot `Control` nodes** (`LineEdit` with `secret`, `ItemList`, `Button`, `PanelContainer`, `Theme`) · a UI addon (none fits better than core) · our own widgets | **Adopt Godot built-ins** — already step-13 §5.2's verdict and `DEP-S12-a`. |
| Reading `GET /status` | **Godot `HTTPRequest`** (core; `clients/shared/host` already uses it for `/admin`) · a raw `HTTPClient` loop · a WebSocket probe join | **Adopt `HTTPRequest`.** A probe join would take a seat to list seats. |
| Speech bubble | **Godot `PanelContainer` + `Label` (autowrap, `custom_minimum_size`) as a child of the figure**, `Tween` for fade · Dialogue Manager's balloon (nathanhoad, MIT) · small "speech bubble" addons on the Asset Library | **Adopt Godot built-ins.** Dialogue Manager is rejected on fit (step-13 §5.2: it runs authored dialogue; MineWorld speech is a world fact). The small addons are single-author, unmaintained or tied to their own dialogue format, and do nothing a `PanelContainer` does not. |
| Notices | **The client's existing `hud/toasts.gd`** (13b; `kind: "notice"` already used by S35 lateness, `app.gd:408`) | **Reuse.** No new widget. |
| Reconnect with resume | **The shared module's opt-in `reconnect`** (`world_client.gd:109,453-469`; checked by `reconnect_check.gd`) · keep `link.gd`'s own plain re-join | **Adopt the module's.** `link.gd`'s plain re-join cannot pass `resume` (`connect_to_world` clears it, `world_client.gd:237`), which is the hold defect of §2. |
| Event de-duplication | A bounded `Dictionary` of seen ids per instance | Our own: four lines; no library is warranted. |

Are we forcing a wheel where it does not fit? No: every adopted piece is a Godot core node already used
by this client.

---

## 5. Design

### 5.1 The join screen (S-1, SD-13e-1)

**What it shows.**

```text
┌ Join a world ─────────────────────────────┐
│ Address   [127.0.0.1:53712            ]   │
│ Invite    [••••••••••••••••           ]   │
│ Nickname  [player                     ]   │
│ Person    ┌──────────────────────────┐    │
│           │ visitor                  │ ←  │   the seats /status lists, in its order
│           │ wanderer                 │    │
│           │ alice                    │    │
│           └──────────────────────────┘    │
│ (the last answer, worded by its code)     │
│                     [ Take over ] [ Join ]│   Take over appears only after seat_occupied
└───────────────────────────────────────────┘
```

- **Seats** are `GET /status`'s `seats`, as keys, in order (`PROTOCOL.md` §5.7). The list shows the key.
  A key is not a display name, and `/status` carries none; the welcome's observation will name the
  person once joined. (Showing names before joining would need a server change; not needed now.)
- **Prefill** from arguments: `--server`, `--invite`, `--nickname`, `--seat` (preselected). The launcher
  passes the invite of a world it hosts, so a local player only picks a person and presses Enter.
- **When it appears** (Q-13e-1): at start when no `--seat` was given; and after any final answer —
  `unauthorized`, `unknown_seat`, `seat_not_in_world`, `invalid_nickname`, `not_granted`,
  `seat_occupied`, `protocol_mismatch`, and the closings `taken_over`, `superseded`, `kicked`, `left`.
  With `--seat` it does not appear at start, so every existing harness run and test is unchanged.
- **What it does on Join**: calls `link.open(address, seat, invite, nickname)` — the one call path
  (`check_client_rules.py` R5 keeps `connect_to_world` in `link.gd`). Nothing is drawn until the welcome.
- **Unreachable address**: `/status` fails → the answer line says "cannot reach <address>"; the seat list
  stays empty; Join is disabled until a list arrives (a seat must be one the world offers).
- **Wording**: every label is a key in the pack's `i18n/*.po` (`ui.join.*`), codes through
  `MineWorldText.code("refused", code)` as today (`words.gd:64-65`).
- **Placement**: `clients/2d/scripts/hud/join_screen.gd`, a `CanvasLayer` like `talk_line.gd`, with an
  interface the 3D client can take as is (`prefill(options)`, `show_answer(code)`, signal
  `join_requested(address, seat, invite, nickname, take_over)`). It is 2D-local now (Q-13e-6).

### 5.2 The link, as the server now runs seats (S-2, SD-13e-2)

| Event | Today (`link.gd`) | After 13e |
| --- | --- | --- |
| Socket drops, server said nothing | plain re-join every 0.5 → 8 s; refused `seat_occupied` during the hold; stuck | `client.reconnect = true`: the module rejoins with `resume` at 1, 2, 4 s within `hold_seconds`, then once without (`world_client.gd:453-469`). If the module gives up, the link's own capped back-off continues with plain joins (a restarted server has no holds). |
| `closing {taken_over}` / `{superseded}` / `{kicked}` | not final → retries for ever | final; toast worded by the reason; join screen shown with that answer. |
| `closing {lagged}` | not final → plain re-join → `seat_occupied` | M-8: the module resumes as for a drop (§5.5). |
| `refused {seat_occupied}` on a join | not final; the link waits on an open socket | final for this attempt; the join screen shows it and offers **Take over**, which sends the same join with `take_over: true` (`PROTOCOL.md` §4.2 rule 4). |
| `refused {not_granted}` after Take over | final (already) | final; the answer says the credential does not allow taking this seat. |
| `welcome {took_over}` | ignored | `"hosted"`: "You are now <name>; the town was looking after them." `"connection"`: "You took <name> over from another player." `"held"`: nothing (it is this player's own seat coming back). `"none"`: nothing. The name is the observation's `display_name` of the observer, read on the first observation. |

The link still holds no world state and decides no rule: it reacts to codes. `FINAL` stays the one list
of codes it ends on, now with `taken_over`, `superseded`, `kicked` and `seat_occupied`.

**Relaunch after a crash (`AC-3`).** A new process has no `resume` (it is never written anywhere, I-13e-5).
During the hold the server answers `seat_occupied`; the player sees "<seat> is in use" and may press
Take over. After the hold ends a plain join is granted. Both are the server's decisions.

### 5.3 Speech bubbles and activity notices (S-4, SD-13e-3)

**Where the decision lives.** The Presentation Pack says which event types are shown and how, in
`renderer/godot.yaml` (`clients/2d/PRESENTATION.md` §4 gains §4.x):

```json
"events": {
  "bubble_seconds": 6.0,
  "notice_seconds": 5.0,
  "types": {
    "spoke":                   { "show": "bubble", "text": "utterance" },
    "group-activity-started":  { "show": "notice" },
    "group-activity-ended":    { "show": "notice" },
    "joined-group-activity":   { "show": "notice" },
    "happening-began":         { "show": "notice" }
  }
}
```

- **Who.** A bubble is drawn over the event's **first subject** (`subjects[0]` of the envelope — for
  `spoke` the speaker, `PROTOCOL.md` §5.2's example). A notice names every subject through
  `display_name`. The envelope's fields are the contract's, so this reads no pack's payload to find a
  person.
- **What.** A bubble's text is the payload field the table names (`text`). A notice's text is the
  wording key `event.<event_type>` with arguments `{who}` (the subjects' names, joined) and every
  top-level string or integer field of the payload (so `{kind}` for an activity). An event whose type has
  no `event.<type>` wording shows the type made readable, as `words.gd` does for actions.
- **Not listed → not shown.** A type the table does not name is not drawn. The protocol asks a client to
  treat an unknown type as "something happened, never an error" (`PROTOCOL.md` §5.2): it is counted in an
  `EVENTS {shown, skipped}` evidence line, never an error.
- **No speaker drawn → no bubble.** The server already sends only facts the observer perceived; a speaker
  not drawn (another place, or not yet drawn) gets a notice-free skip, counted.
- **Source** (Q-13e-4): `observation.events` of **every** observation frame (`link.observed` fires per
  frame, `app.gd:380`), not the `perceived` stream: bubbles are for now, and a backfill would replay a
  day of old lines. De-duplicated by `id` per world instance (I-13e-6), cleared on a new instance like
  `_noticed` (`app.gd:372-374`).
- **Lifetime** is the pack's `bubble_seconds` (wall seconds; presentation, not a rule). A newer line by the
  same speaker replaces the bubble.
- **Generic by construction.** The same table works on `social-cafe` and `lakeside`: both install
  `conversation` and `group-activity` (`worlds/lakeside/world.yaml` systems). A world without them
  simply sends no such events.

`clients/2d/scripts/scene/speech.gd` (new) owns the bubbles (children of the y-sorted world, positioned
from `people.figure_at(id)`); notices go through the existing `toasts.gd`.

### 5.4 Launcher defaults (S-5, SD-13e-4)

```text
./mineworld-2d                                  hosts worlds/market-town --town --solo; join screen
./mineworld-2d --world worlds/lakeside          the same for lakeside; its own seats listed
./mineworld-2d --seat visitor                   skips the screen (as today)
./mineworld-2d --agent bob                      bob reactive (--agent), the rest --town
./mineworld-2d --no-town                        nobody driven (today's quiet world, for debugging)
./mineworld-2d --join-local                     a second window: joins the world this checkout
                                                hosts for --world, reading its invite itself
./mineworld-2d --drive=…                        unchanged: scripted runs keep explicit seats, no --town
```

- `seat` and `agent` lose their defaults (`mineworld-2d:52,56`). `--town` is added to played hosted runs
  only (as `mineworld-slice:267`). Scripted runs keep today's arguments so every test is unchanged.
- `--join-local` reads `clients/2d/.save/server-<world>.log`'s join line exactly as the hosting branch does
  (`mineworld-2d:172-187`) and passes address and invite to the client; it starts no server and echoes
  nothing. A world that is not running fails at `/status` on the join screen ("cannot reach").

### 5.5 Shared module M-8 (S-6, SD-13e-5)

`world_client.gd` `_process` (`:430-436`): today any `closing` ends the connection. **Change:** when
`reconnect` is on and the reason is `lagged`, schedule the resume reconnect as for a silent drop
(`PROTOCOL.md` §5.8 "rejoins with its resume and its cursor"). Off by default, so no current caller
changes (only `clients/protocol/checks/*` set `reconnect`). `ADOPTION.md` §2 and §6.1 say so.

### 5.6 What does not change

`world_time`, `host_clock`, `daylight`, the HUD clock (TW-e); `intents.gd` (no new request: a takeover
is a join); `walker.gd`; `people.gd` except a figure lookup `speech.gd` already has (`figure_at`).

---

## 6. Acceptance — decided before measuring (`ARC-23`)

Every criterion names at least one world besides `market-town` (charter). "Within" bounds are wall
time on the CI runner, fixed now. A criterion that cannot be measured is `INCONCLUSIVE`, never `PASS`.

| ID | Criterion (observable) | Worlds | Oracle | Test |
| --- | --- | --- | --- | --- |
| **AC-13e-1** | Started with no `--seat`, the client shows the join screen whose seat list equals `/status`'s `seats` exactly, in order; choosing entry 0 and Join yields a welcome whose observer is drawn within 10 s. | `market-town`, `lakeside` | `World::status()` read by the test | `client_2d_many.rs` `joins_from_the_screen` (`--drive=join --pick=0`) |
| **AC-13e-2** | A wrong invite: the screen shows the answer worded from `unauthorized` within 3 s (the server's fixed 500 ms delay plus margin); **zero** people and places were drawn; no `REQUEST` line was printed. | `social-cafe` | the server's code; `SHOWN` counts | `client_2d_many.rs` `a_wrong_invite_draws_nothing` |
| **AC-13e-3** | Client B asks for the seat client A holds: B shows `seat_occupied` and the Take over button; A stays seated and its revision keeps rising for 5 s. B presses Take over: B is welcomed `took_over: "connection"` and shows the "took over from another player" toast; A shows the `taken_over` toast, returns to the join screen, and sends **no** join in the next 10 s. | `market-town`, `social-cafe` | welcome/closing frames as printed; A's `JOIN` lines | `client_2d_many.rs` `an_occupied_seat_is_refused_then_taken_on_purpose` |
| **AC-13e-4** | With `--town`, a client joins a seat the town drives: welcomed `took_over: "hosted"`, the "town was looking after them" toast names the observer's `display_name`; the observer id equals the id client A's observation listed for that person before the join. (AC-5 *shown*; *asserted* in 13f.) | `social-cafe` (seat `bob`), `lakeside` (seat `bruno`) | A's earlier observation | `client_2d_many.rs` `taking_over_a_town_person` |
| **AC-13e-5** | The client closes its own socket mid-session (`--drive=blink`, the module's `WebSocketPeer.close()`, as `reconnect_check.gd:95`): it is welcomed again `took_over: "held"`, same observer, within the 30 s default hold, and **no** `seat_occupied` is printed. | `market-town`, `social-cafe` | welcome frames | `client_2d_many.rs` `a_blink_resumes_the_same_seat` (no `--hold 0`) |
| **AC-13e-6** | Two clients A and B in one place (the test picks two seats whose first observations name the same place). A talks to a person (the line `hello from A`). B shows a bubble over A's figure with exactly that text within 2 s of A's accepted `RESULT`. Over the whole run, **the set of `spoke` event ids B bubbles equals the set of `spoke` facts in B's `mineworld perceived` export whose first subject B drew**, with no id twice. If `/status` reports `events_dropped > 0` the set check is `INCONCLUSIVE`. | `market-town`, `social-cafe` | `mineworld perceived <world> --save DIR --person <B's seat> --json` (the server's own function, `PROTOCOL.md` §5.8), restricted to facts recorded while B was seated (from B's first to last observation `at`) | `client_2d_many.rs` `bubbles_are_the_facts_heard` (`--drive=listen`) |
| **AC-13e-7** | A group activity starts where B is (A invites, a scripted client C accepts, as 13b did): B shows one notice worded from `event.group-activity-started` with every member's name, once. | `social-cafe`, `lakeside` | B's perceived export | `client_2d_many.rs` `an_activity_starting_is_noticed_once` |
| **AC-13e-8** | A living town: hosted with `--town` and the world's `hosting: time_scale`, at least one person B did not drive is drawn at ≥ 1 m from where B first drew them within 120 s, and B's drawn position for them matches the server's within AC-W2's tolerance at the end. | `market-town`, `social-cafe` | the server's observation (SHOWN vs FRAME) | `client_2d_many.rs` `the_town_lives_without_us` |
| **AC-13e-9** | The launcher's played mode on `worlds/lakeside` reaches the join screen (no `carol`, no `alice` assumed) and its server was started with `--town`; on `market-town` the same. | `lakeside`, `market-town` | the server's log line and process arguments | hand check §9, plus a launcher dry-run test if cheap (C4) |

**Adversarial criteria (each must bite).**

| ID | Check | How it bites |
| --- | --- | --- |
| **AD-13e-1** | Rule scan R1–R7 clean; new **R8**: no `.gd` under `clients/2d/scripts/` holds a literal equal to an event type or field named in the default pack's `events.types`. | Plant `"spoke"` in `speech.gd`: R8 fails. |
| **AD-13e-2** | Delete `events` from a copy of the pack: the client runs, draws no bubble and no notice, prints `EVENTS {shown: 0, skipped: n>0}`. | Proves no built-in fallback for `spoke`. |
| **AD-13e-3** | Revert C2's `FINAL` change: AC-13e-3's "no join in 10 s" fails. | Proves the loop fix is tested. |
| **AD-13e-4** | Set `reconnect = false` in `link.gd`: AC-13e-5 fails with `seat_occupied`. | Proves the resume path is tested. |
| **AD-13e-5** | Scope: `check_client_rules.py --scope origin/main` passes; no path outside I-13e-4's set. | Plant a byte in `server/`: scope fails. |
| **AD-13e-6** | Secret scan: the invite string and any 32-hex `resume` from the run appear in no stdout line, no log under `clients/2d/.save/`, no settings file. | A test-only print of the invite fails it. |

---

## 7. Commit plan

Each commit: implementation, deterministic validation, LLM logic review as separate items. Under the
charter's speed rule, local validation is the one test being written plus `check_client_rules.py`; the
Godot-gated tests run at the milestone train (R-LOAD-1), through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.

### C0 — Contract and ledger (Markdown only)

- **Goal:** fill §12 from the freeze ruling; open the ledger (§13).
- [ ] Implementation: §12 filled; §13 opened with the base SHA.
- [ ] Validation: `git diff --stat` shows only this file.
- [ ] Review: N/A — no code.

### C1 — Specs before code

- **Goal:** the pack format and the module's behaviour are written before code (`CLAUDE.md` §2.2).
- **Scope:** `clients/2d/PRESENTATION.md` §4.x `events` (§5.3 of this doc), §7 the `ui.join.*`,
  `ui.seat.*`, `event.*` keys; `clients/protocol/ADOPTION.md` §2 and §6.1 (M-8; the client pattern of §5.2);
  `clients/2d/README.md` (join screen, `--join-local`, `--no-town`, two windows).
- [ ] Implementation: the three files edited.
- [ ] Validation: `python3 scripts/check_decision_ids.py`; links resolve.
- [ ] Review: terms match `CORE_CONCEPTS.md` (`Event`, `Observation`, `Presentation Pack`); no rule
  stated in a README only.

### C2 — Link: resume, final reasons, takeover (S-2) and M-8 (S-6)

- **Goal:** fix the hold and loop defects of §2 before any screen exists, so they are testable alone.
- **Scope:** `clients/2d/scripts/link.gd` (`reconnect = true`, `FINAL` +4, `take_over` passed through
  `open`, signal `ended(code)`), `clients/protocol/mineworld/world_client.gd` (M-8 only),
  `clients/2d/scripts/harness/drive.gd` (`blink` scenario). Non-goal: no UI.
- [ ] Implementation: as above.
- [ ] Validation: `client_2d.rs` `killed_mid_walk_and_relaunched` still passes **with** `--hold 0` (unchanged
  meaning); new `client_2d_many.rs` `a_blink_resumes_the_same_seat` (AC-13e-5); AD-13e-4 planted and
  recorded; `clients/protocol/run.sh` checks (`reconnect_check`, `rev2_check`) still PASS.
- [ ] Review: no rule added; M-8 changes nothing when `reconnect` is off; a `kicked` seat never auto-rejoins.

### C3 — The join screen (S-1) and seat messages (S-3)

- **Scope:** `clients/2d/scripts/hud/join_screen.gd` (new), `app.gd` (shown when no `--seat`; on
  `link.ended`; toasts for `took_over`), `presentation/mineworld-default/2D/i18n/{en,zh_Hans}.po`
  (`ui.join.*`, `ui.seat.*`), `harness/drive.gd` (`join` scenario: `--pick=<index>`, `--invite-override`,
  presses Take over on `--take-over`).
- [ ] Implementation: as above.
- [ ] Validation: AC-13e-1, -2, -3, -4 in `client_2d_many.rs`; AD-13e-3 planted; AD-13e-6 secret scan;
  `check_client_rules.py` R5, R6 clean.
- [ ] Review: the screen never lists a seat `/status` did not; Join disabled with no list; no
  address/nickname written to the settings file (INV-SET-2).

### C4 — Launcher defaults (S-5)

- **Scope:** `mineworld-2d` (no default seat/agent; `--town` on played hosted runs; `--no-town`;
  `--join-local`); `clients/2d/README.md` lines if C1 missed any.
- [ ] Implementation: as above.
- [ ] Validation: AC-13e-9 by hand on macOS (`./mineworld-2d --world worlds/lakeside`); scripted runs
  unchanged (the existing `client_2d*.rs` pass their seats explicitly).
- [ ] Review: invite never echoed; only this script's server is stopped (HOST.md §5 unchanged).

### C5 — Bubbles and notices (S-4), rule R8

- **Scope:** `clients/2d/scripts/scene/speech.gd` (new), `app.gd` (feed every observation's events),
  `presentation/mineworld-default/2D/renderer/godot.yaml` (`events`), the `.po` files (`event.*`),
  `scripts/check_client_rules.py` (R8; `--check-pack` validates `events`; `SCOPE_FILES` +
  `tools/cli/tests/client_2d_many.rs`, `clients/protocol/mineworld/world_client.gd`,
  `clients/protocol/ADOPTION.md`), `harness/drive.gd` (`listen` scenario printing `BUBBLE`, `NOTICE`,
  `EVENTS` lines).
- [ ] Implementation: as above.
- [ ] Validation: AC-13e-6, -7 in `client_2d_many.rs`; AD-13e-1 and AD-13e-2 planted;
  `check_client_rules.py --self-test` covers R8.
- [ ] Review: no event type or field literal in scripts; de-dup survives a blink; an unknown type is not
  an error.

### C6 — A living town and the cross-world gate

- **Scope:** `client_2d_many.rs` `the_town_lives_without_us` (AC-13e-8); stills of a bubble and a notice
  under `clients/2d/shots/preview/` (windowed, `--capture`).
- [ ] Implementation: test and stills.
- [ ] Validation: AC-13e-8 on both worlds; the full `client_2d_many.rs` once on the PR head; AD-13e-5.
- [ ] Review: stills inspected by eye (bubble legible, not clipped by the far field).

### C7 — Close-out

- [ ] Ledger, evidence table, step-13 §9 13e row status; train-ready report to the coordinator.
- [ ] Validation: CI `fast` green on the exact head.
- [ ] Review: N/A beyond the ledger check.

---

## 8. Files touched

| Path | Commit | Kind |
| --- | --- | --- |
| `clients/2d/scripts/link.gd` | C2 | edit |
| `clients/protocol/mineworld/world_client.gd` (M-8 only) | C2 | edit, additive |
| `clients/protocol/ADOPTION.md` | C1 | edit |
| `clients/2d/scripts/hud/join_screen.gd` | C3 | new |
| `clients/2d/scripts/scene/speech.gd` | C5 | new |
| `clients/2d/scripts/app.gd` | C3, C5 | edit |
| `clients/2d/scripts/harness/drive.gd` | C2, C3, C5 | edit |
| `clients/2d/PRESENTATION.md`, `clients/2d/README.md` | C1 | edit |
| `presentation/mineworld-default/2D/renderer/godot.yaml` | C5 | edit |
| `presentation/mineworld-default/2D/i18n/en.po`, `zh_Hans.po` | C3, C5 | edit |
| `mineworld-2d` | C4 | edit |
| `scripts/check_client_rules.py` | C5 | edit (R8, pack check, scope list) |
| `tools/cli/tests/client_2d_many.rs` | C2–C6 | new |
| `tools/cli/tests/godot2d/mod.rs` | C2 | edit (a `Drive` start without `--seat`; `perceived` export helper) |
| this file | C0, C7 | edit |

---

## 9. Test ownership

| Claim | Owner | Not re-tested by |
| --- | --- | --- |
| Server join rules, holds, takeover, `perceived` order | S11 (`server/tests`, `rev2_check.gd`, `perceived_check.gd`) | 13e only consumes them |
| Module resume path | `clients/protocol/checks/reconnect_check.gd` | 13e tests the 2D link using it (AC-13e-5) |
| Clock at the time scale | TW-e (its §8 criteria) | — |
| Join screen, link policy, seat messages | **13e** `client_2d_many.rs` | — |
| Bubbles/notices equal the perceived facts | **13e** AC-13e-6, -7 | — |
| No rule / no vocabulary in the client | **13e** R8 + existing R1–R7 | — |
| `AC-5` asserted from the save | 13f | 13e shows it only |

Hand check for the operator (macOS, then Linux/Windows at the milestone): two windows —
`./mineworld-2d` then `./mineworld-2d --join-local` — pick two people in the café, type a line in one,
see the bubble in the other; take the other's person over from the first window and see both messages.

---

## 10. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| RK-13e-1 | The paced town says little near a given seat, so AC-13e-6's NPC part sees few lines. | The oracle is the perceived export, so a quiet run still checks equality; A's scripted line guarantees one bubble. |
| RK-13e-2 | Two Godot clients plus a `--town` server on a 2-slot machine are slow and flaky. | Gated `#[ignore]` tests run in the nightly clients layer and at the milestone; wall bounds have margin; one `with-slot.sh` run per AC set. |
| RK-13e-3 | `subjects[0]` is not the speaker for some future event type. | The pack table may later name a `who` field; until then `subjects[0]` is the contract's "what the fact is about", which is the drawn person. Recorded as a bounded decision. |
| RK-13e-4 | Lakeside's 2D drawing of its places is weaker than market-town's. | The criteria measure joins, frames and events, not looks. |
| RK-13e-5 | An open PR also edits `app.gd` or `link.gd`. | C0 checks open PRs touching them; rebase before C3. |

---

## 11. Questions for the freeze, each with a recommended answer

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-13e-1** | When does the join screen appear? | **When no `--seat` is given, and after any join that ends for good.** `--seat` skips it, so scripts and tests are unchanged. 推荐：没给 `--seat` 就显示加入界面；给了就直接进。 |
| **Q-13e-2** | `/status` does not say which seats are free (it is composition only). Add a server field? | **No server change.** List every seat; the server's `seat_occupied` answer tells the player, and a "Take over" button retakes it only on purpose. Never automatic, even after a crash. 推荐：不改服务器；被占时提示并给"接管"按钮，只在玩家点了才接管。 |
| **Q-13e-3** | Where is "which events show, as a bubble or a notice" decided? | **In the Presentation Pack** (`renderer/godot.yaml` `events`, wording `event.<type>`). The client names no event type; an unlisted type is not shown. 推荐：放在 Presentation Pack 的配置里，客户端代码里不写任何事件名。 |
| **Q-13e-4** | Bubbles from `observation.events` (best effort) or the reliable `perceived` stream? | **`observation.events`, de-duplicated by id.** Bubbles are for now; the reliable stream would replay old lines after a rejoin. 推荐：用 observation 里的 events，按 id 去重。 |
| **Q-13e-5** | Should `./mineworld-2d` run the town (`--town`) by default, and drop the `carol` / `--agent alice` defaults? | **Yes**, like `mineworld-slice`; `--no-town` for a quiet world. The old defaults break `lakeside`. 推荐：默认开 `--town`，去掉 carol/alice 默认值。 |
| **Q-13e-6** | Join screen in the 2D client only, or a shared module for 3D too? | **2D-local now**, with an interface 3D can take; move it to `clients/shared/` when the 3D client adopts it (then it is a real repeat). 推荐：先放 2D，接口留好，3D 用时再抽到 shared。 |
| **Q-13e-7** | How does a second player on the same machine get the invite? | **`./mineworld-2d --join-local`** reads the hosted world's invite itself and never prints it. 推荐：加 `--join-local`，第二个窗口一条命令加入，不打印邀请码。 |
| **Q-13e-8** | Change the shared module so `closing {lagged}` resumes like a dropped socket (M-8)? | **Yes**, opt-in under `reconnect`; no current caller changes; the protocol already says a lagged client rejoins with its resume. 推荐：同意，小改动，只在开启 reconnect 时生效。 |

---

## 12. Execution contract (filled at freeze)

```text
PROJECT / PR:            MineWorld mvp0, S12 PR 13e — many players, a living town (2D)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-13e-many-players-2d.md
RELATED / BINDING DOCS: step-13-client-2d.md §9 (13e row), server/PROTOCOL.md rev 2,
                         clients/protocol/ADOPTION.md, clients/2d/PRESENTATION.md, CLAUDE.md,
                         LANE_LEAD_CHARTER.md (speed first; no world-specific hard-coding)
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-13e (one session)
BRANCH:                  mvp0/pr-13e (from origin/main)
START CONDITION:         operator freeze 2026-10-20, relayed (met)
IMPLEMENTATION BASE:     origin/main @ c9e0dde0
APPROVED SCOPE:          §3.1 S-1 … S-7; paths of §8 plus harness helpers under clients/2d/scripts/
FROZEN INVARIANTS:       I-13e-1 … I-13e-6 (§3.3)
SEQUENCE:                C0 … C7 (§7)
ALLOWED COMMANDS:        git, gh (PR open/view/checks), python3 scripts/*.py; cargo and godot only
                         through cast-slots/with-slot.sh; CARGO_TARGET_DIR=<worktree>/target
NEVER:                   merge; edit outside I-13e-4's paths; print or log an invite or resume
                         secret; sed -i / perl -pi / inline python -c / curl; paid calls
MATERIAL STOPS:          a protocol or server change; a frozen invariant; scope beyond §3.1
PLATFORMS:               macOS locally; Linux/Windows at the milestone train (R-LOAD-1)
VALIDATION BUDGET:       the build plus the specific tests written; the full client_2d_many.rs and
                         the Godot gated layers at the milestone (charter: speed first)
LIVE DOCUMENTATION:      this file (§13 ledger)
HANDOFF:                 the final report to the coordinator (no separate handoff file)
ENDPOINT AUTHORITY:      push and open the PR (coordinator brief 2026-10-20); merge is the
                         coordinator's after `fast` is green (merge freeze: it queues)
DECISION IDS:            ARC-343, ARC-344, ARC-345 (coordinator, 2026-10-20)
POST-MERGE SYNC OWNER:   the coordinator (step-13 §9 row, progress page)
STOP CONDITION:          PR open, `fast` green on the exact head, report delivered
```

### 12.1 Decision records proposed (ids assigned by the coordinator 2026-10-20)

- **ARC-343 (SD-13e-3)** — *Which events a 2D client shows, and how, is the Presentation Pack's.* The
  `events` table, `subjects[0]` as the bubble's owner, `event.<type>` wording, unlisted types not shown.
- **ARC-344 (SD-13e-2)** — *The 2D link resumes with the module's `resume`, ends on `taken_over`,
  `superseded`, `kicked`, `seat_occupied`, and takes a seat over only on the player's explicit choice.*
- **ARC-345 (SD-13e-4)** — *The 2D launcher hosts a living town by default and assumes no seat or agent.*

---

## 13. Ledger (implementation session)

| Commit | SHA | Implementation | Validation (evidence) | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] §12 filled, ledger opened, base c9e0dde0 | [x] `git diff --stat`: this file only | N/A | |
| C1 | 23f7c65e | [x] PRESENTATION.md §4.1/§7, ADOPTION.md §2/§6.1, README, ARC-343..345 | [x] `check_decision_ids.py` PASS; `check_doc_headings.py` PASS | [x] terms per CORE_CONCEPTS (Event, Observation, Presentation Pack); rules in specs, README links | D-3 |
| C2 | 0163df74 | [x] link.gd (reconnect, FINAL +4, take_over, ended, joining), world_client.gd M-8, `--drive=blink` | [x] `a_blink_resumes_the_same_seat` PASS (market-town, social-cafe), twice; AD-13e-4 not planted (INCONCLUSIVE) | [x] M-8 inert with reconnect off; kicked never rejoins; no rule | |
| C3 | 0820d214 | [x] join_screen.gd, app.gd wiring, took_over toasts, wording, `--drive=join` | [x] `joins_from_the_screen` (market-town, lakeside), `a_wrong_invite_draws_nothing` (social-cafe; AD-13e-6 secret scan inside), `an_occupied_seat_is_refused_then_taken_on_purpose` (market-town, social-cafe), `taking_over_a_town_person` (social-cafe, lakeside) PASS | [x] lists only /status seats; Join disabled without a list; nothing written to settings | D-2, D-5 |
| C4 | aaa98a68 | [x] launcher: no default seat/agent, `--town`, `--no-town`, `--join-local`, pack roots | [x] `bash -n`; AC-13e-9 hand check not run (INCONCLUSIVE: operator checklist) | [x] invite never echoed; only this script's server stopped | |
| C5 | 8aee04fc | [x] speech.gd, events table, R8, `--check-pack` events, `--drive=listen` | [x] `bubbles_are_the_facts_heard` PASS (market-town, social-cafe; set check vs `mineworld perceived`), `an_activity_starting_is_noticed_once` PASS (social-cafe, market-town); AD-13e-1 planted `"spoke"` in speech.gd → R8 FAIL, removed; `--self-test` PASS; AD-13e-2 not planted (INCONCLUSIVE) | [x] no event literal in scripts; de-dup by id per instance; unknown type skipped | D-1, D-4 |
| C6 | 8aee04fc, c86304e9 | [x] `--drive=town`, `the_town_lives_without_us`; stills not taken | [~] first run FAIL on the player's own figure (F-13e-1); after c86304e9 not re-run locally (machine reserved) — INCONCLUSIVE until CI's clients layer; AD-13e-5 `--scope origin/main` PASS | [ ] stills not inspected (none taken) | D-4, F-13e-1 |
| C7 | (this commit) | [x] ledger, step-13 §9 row | [ ] `fast` on the exact head (see PR) | N/A | |

**Fixes found by the local runs** (a57cede7, 595b4b49, c86304e9): bubble owner by `who` (D-1); the link closes the
socket on a final refusal while joining (the server keeps it open after `seat_occupied`); the join screen keeps the
chosen seat across re-listing; lakeside hosted with its pack roots; `/status` read through the shared module (D-2);
AC-13e-8 checks the town's people only. Merged origin/main (c4fdfaf9, #442's shared HUD move) with no conflict.

**Deviations (bounded).**
- **D-1** `spoke` is stated about its listener, so `subjects[0]` is the listener (RK-13e-3 realised). The table
  gains an optional `who` payload field; `spoke` names `speaker`.
- **D-2** `client_text.rs` AC-WC-15 keeps `HTTPRequest` out of client scripts: `GET /status` is read by a new
  additive shared-module node, `clients/protocol/mineworld/status.gd` (`MineWorldStatus`), documented in ADOPTION.md §2.
- **D-3** Notice wording key `ui.event.<type>` (not `event.<type>`): `event` is not a catalog family in `client_text.rs`.
- **D-4** C5 and C6 share one commit; the C6 stills were not taken (windowed capture; machine saturated, then reserved).
- **D-5** AC-13e-4's identity oracle is the World Pack's genesis id (social-cafe); on lakeside, whose pack reads only
  through its pack roots, the SEAT observer is compared with the welcome.

**Finding F-13e-1 (not this PR's scope).** Under `--town`, the player's own drawn figure stayed 0.11 m from the
server's position at the end of a 120 s idle run (market-town, observer 20): the server moved the body without a
request (likely a body nudge), and the walker did not follow. Owner: the walker (13a/13d).
