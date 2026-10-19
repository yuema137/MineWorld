# The MineWorld host client module

**Audience:** whoever adopts, extends or reviews the shared host module — the 2D and 3D reference clients,
a third client, the reference launchers, or a later PR (S21 fast-forward, S23 a LAN host launcher). This is
a specification, not an introduction; [`README.md`](README.md) is the orientation.
**Authority:** `docs/DECISIONS.md` `ARC-267` (this module and the World tab), `ARC-268` (`POST
/admin/stop`, the launcher contract and closing the game), `ARC-44` (the admin surface), `ARC-69` (pacing
is a host command), `ARC-76` (the settings module, which this module is not). The wire is
`server/PROTOCOL.md` §§5.9, 11. The design and its rulings are
`.structured-coding/plans/mvp0/pr-tw-e-set-c-world-clock-clients.md` §§5.4–5.8. Where this document and a
decision disagree, the decision governs and this document is the defect.

---

## 1. What this module is

A **host command** is a request to the server's admin surface: pause, resume, a day length, stop. It
changes the world's pacing (or ends the process) for every player, it is sent with the host's bearer token,
and it is never a setting (`SETTINGS.md` §1, `INV-SET-3`). This module is the only shared client code that
sends HTTP. It holds the admin token in memory, sends the commands, and shows the World tab of the settings
menu: the clock's state to everybody, its controls to the token holder.

## 2. Layout and adoption

```text
clients/shared/host/       THE MODULE — what a client takes, as res://mineworld_host
  host_clock.gd            MineWorldHostClock   token, base URL, POST /admin/clock and /admin/stop,
                           pending and confirmed state, error codes
  world_tab.gd             MineWorldWorldTab    the World tab: state, Pause/Resume, Day length 4 h · 2 h · 1 h
clients/shared/checks/
  host_check.gd            the module's headless check, against a scripted transport
```

```text
clients/2d/mineworld_host        -> ../shared/host
clients/3d-spike/mineworld_host  -> ../shared/host
```

**Dependencies (`INV-WC-3`, `INV-SET-3`).** `host/` reads the protocol module's `clock_changed` (through
calls its composition root makes) and the settings module's `MineWorldText` for words. It never names
`MineWorldSettingsStore` or `store.gd`, never reads or writes the settings file, and never names
`MineWorldWorldTime`. `client_text.rs` holds all three.

## 3. `MineWorldHostClock`

A `Node` the composition root adds to its tree.

### 3.1 The token

`setup(base_url: String)` reads the token once from the environment variable `MINEWORLD_ADMIN_TOKEN`.
Never from the command line (a process list shows arguments), never from a file, never printed, never in a
log, an evidence line or a capture. An empty variable is **no token**: `has_token()` is false, the World
tab is read-only, and **no `HTTPRequest` node is ever created** (`AC-WC-11`).

`base_url` is `http://<the address the client joined>`: the admin surface is on the same listener as
`/status` (`PROTOCOL.md` §11.1). The composition root passes it; the module parses no join argument.

### 3.2 State, from the server only (`INV-WC-1`)

`on_welcome(world: Dictionary)` and `on_clock(at, time_scale, paused)` (from `MineWorldClient.clock_changed`)
set `paused` and `time_scale`. Nothing else does: a click, a sent request and a `200` answer change neither.
`day_length()` names the state for the tab: `4h`, `2h`, `1h` for scales 6, 12, 24, else `custom` (a
`--time-scale 1` world, or a solo world fast-forwarding in S21).

### 3.3 Commands

| Call | Request |
| --- | --- |
| `set_paused(paused: bool)` | `POST /admin/clock` `{"paused": true}` or `{"paused": false}` |
| `set_scale(scale: int)` | `POST /admin/clock` `{"time_scale": 6 | 12 | 24}`; any other value is refused locally and sends nothing (the tab offers no fast-forward) |
| `stop_world()` | `POST /admin/stop` `{}` (§5) |

Each carries `Authorization: Bearer <token>` and `Content-Type: application/json`. One request is in
flight at a time; a command while one is pending is ignored. Without a token every call does nothing.

### 3.4 Pending, confirmed, unconfirmed

After a `200` to a clock command the module is **pending** (`ui.world.pending`) until a `clock` frame
whose fields match the request arrives — then it is confirmed and shows nothing — or until 5 wall seconds
pass, when it shows `ui.world.unconfirmed` and the tab returns to the last frame's state. A command whose
value is already in force (`200`, and no frame, `PROTOCOL.md` §11.2 "already in force") is confirmed at
once because the state already matches.

### 3.5 Errors

Each is worded by its code, `ui.world.error.<code>`, in the current language; the controls return to the
server's state:

| Code | When |
| --- | --- |
| `unauthorized` | `401` (answered no sooner than 500 ms after the request, by design) |
| `malformed` | `400` |
| `fast_forward_solo_only` | `409` (the tab never asks for more than 24×; worded for completeness, TW-c F-TWc-5) |
| `world_stopped` | `503` |
| `timeout` | no answer within 5 s (not 1 s: the `401` path alone takes 500 ms) |
| `unreachable` | the connection failed, or any other status |

Signals: `changed` whenever `paused`, `time_scale`, the pending state or the message changes; `stopped`
when `/admin/stop` was answered `202`.

### 3.6 The test seam

`transport` (a `Callable`, default empty) replaces the `HTTPRequest` when set: it is called with
`(method, path, headers, body)` and the test answers by calling `answered(status, body)`. The check passes
headers through it, so the bearer header is asserted without a network.

## 4. `MineWorldWorldTab`

`MineWorldWorldTab.new(menu, host_clock)` is a `VBoxContainer` a client adds with
`menu.add_tab("ui.settings.tab.world", tab)` — in both clients, always, so a player who is not the host
learns why the world stopped.

- **State** (`ui.world.clock`): `ui.world.state.running` or `ui.world.state.paused`.
- **Day length** (`ui.world.day_length`): `ui.world.day_length.4h`, `.2h`, `.1h`, or
  `ui.world.day_length.custom` (`{scale}`) for any other scale.
- **Controls**, only with a token: Pause/Resume (`ui.world.pause`, `ui.world.resume`) and one button per
  day length, the current one disabled. Without a token the same rows are read-only and
  `ui.world.read_only` says why.
- **Message line:** pending, unconfirmed or the last error's words.
- Every text comes from the catalogs at each redraw, on `menu.rendered` and on `host_clock.changed`.

The HUD's `Paused` label (`hud.paused`, `WORLD_TIME.md` §6) is the clients' own and reads the `clock`
frame through the world-time module; the tab never shows a state the server has not sent.

## 5. Stopping the host, and closing the game

`POST /admin/stop` (`PROTOCOL.md` §11.2, `ARC-268`) answers `202 {"stopping": true}` and takes the same
graceful path as Ctrl-C: the server stops accepting, the world checkpoints, writes the host journal's
`Stopped` record and prints its statistics, and the process exits 0. It changes no world state. It is the
only graceful stop that reaches a server started in the background by a Git Bash launcher on Windows,
which receives no console control event.

A client started by its local launcher is told so by the join argument `--hosted-here` (never saved). On a
window close (`NOTIFICATION_WM_CLOSE_REQUEST`) or the menu's Quit:

```text
keep_running = false   →  stop_world(); quit with exit code 0 when the connection ends, or 5 wall seconds
                          after the request, whichever is first
keep_running = true    →  quit with exit code 10 ("leave the host running"); the server keeps running
not --hosted-here      →  quit with exit code 0, as before
```

`keep_running` is the settings module's one launcher preference (`[launcher] keep_running`, default false,
`SETTINGS.md` §3), shown in the General tab only to a client started `--hosted-here`
(`MineWorldSettingsMenu.CAP_HOSTED_HERE`).

## 6. The launcher contract

When a reference launcher (`./mineworld-2d`, `./mineworld-slice --world`) hosts the world itself:

1. It generates a token of 32 hexadecimal digits from `/dev/urandom` with `od` (POSIX, present in Git Bash
   on Windows) and refuses to start if it has fewer.
2. It gives the token to the server and the client processes only, as an environment prefix
   (`MINEWORLD_ADMIN_TOKEN=… command`): never exported to its own shell, never echoed, never in a log line,
   never under `set -x`.
3. It starts `mineworld server <world> --listen 127.0.0.1:0 --solo --save <save> …` with no
   `--time-scale`: the world's `hosting:` or the save's journal gives the scale (TW-c §5.5).
4. It passes `--hosted-here` to the client.
5. After the client exits: on exit code 10 it leaves the server running, prints its address, its log and
   how to stop it (Ctrl-C in the server's console, or `POST /admin/stop` with the token), and removes its
   own `EXIT` trap. Otherwise it waits up to 10 s for the server to exit by itself; a server still running
   (the client crashed, or an older client) gets `SIGINT` on macOS and Linux (graceful), then, after 5 more
   seconds, the existing kill as a last resort, which loses at most the time since the last checkpoint —
   and the launcher says which it did.

When joining a world hosted elsewhere (`--server … --invite …`) a launcher passes neither a token nor
`--hosted-here`; the World tab is read-only unless the user exported `MINEWORLD_ADMIN_TOKEN` themselves
(a LAN host playing on their own machine).

The 3D launcher's `--world` keeps a save under `clients/3d-spike/.save/social-cafe` (`--fresh` starts it
over), as `./mineworld-2d` does, so reopening resumes the same minute. Its scripted modes keep running
on a world without a save, as before.

## 7. Per platform

| Concern | macOS | Linux (X11, Wayland) | Windows (Git Bash launchers) |
| --- | --- | --- | --- |
| `HTTPRequest` to `127.0.0.1` | yes | yes | yes; Windows Firewall does not prompt for loopback |
| Token from `/dev/urandom` via `od` | yes | yes | Git Bash provides both |
| Environment hand-over | env prefix | env prefix | env prefix reaches native `mineworld.exe` and `godot.exe` |
| Graceful close | `/admin/stop`; `SIGINT` as the fallback | same | `/admin/stop` only |
| Exit code 10 from Godot (`get_tree().quit(10)`) | yes | yes | yes; `$?` in Git Bash |
| Symlinked module | yes | yes | needs `core.symlinks`; the launchers refuse with the existing message |

No file under `host/` asks which platform it runs on (`client_text.rs`).

## 8. Checks

| Check | What it holds | Runs |
| --- | --- | --- |
| `clients/shared/checks/host_check.gd` | no token ⇒ no request and no `HTTPRequest`; pending until the matching frame; unconfirmed after 5 s; a repeat confirmed at once; every error code worded; the bearer header | `godot --headless --path clients/shared --script res://checks/host_check.gd` |
| `tests/acceptance/tests/client_text.rs` | `HTTPRequest` only under `host/` and `clients/protocol/checks/`; no `store.gd` or `MineWorldSettingsStore` in `host/`; no literal UI string; no platform branch | `cargo test`, in CI |
| `tools/cli/tests/client_world_time.rs` | both clients against a stub and a real server (`AC-WC-7`, `-9` … `-13`) | `cargo test -p mineworld-cli --test client_world_time -- --ignored --test-threads=1` (needs Godot) |
| `server/tests/admin.rs`, `tools/cli/tests/host_clock.rs` | `POST /admin/stop`: auth, `202`, the graceful path and the `Stopped` record | `cargo test`, in CI |
