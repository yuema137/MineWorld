# `mineworld-launch` — the bundle's supervisor

Specification of the launcher that starts a downloadable bundle's local server and client(s)
(`docs/DECISIONS.md` `ARC-78`; design `.structured-coding/plans/mvp0/step-23-release.md` §6 and §19). The
human introduction is [`README.md`](README.md).

## 1. What it is

A process supervisor for one machine. It finds the bundle it belongs to, starts `mineworld server` on a
world with a save in the per-user `MineWorld` folder, reads the server's join line, starts the Godot runtime
with a client's pack and the join arguments, waits for the client(s) to exit, stops the server, and reports
a failure.

It never reads world state; the only thing it reads from the server is the join line
(`server/PROTOCOL.md` §4.1). It holds no world rule, opens no window of its own, makes no network connection,
and stops only the processes it started. No language model is involved anywhere.

## 2. Entry points

One crate, `tools/launch` (`mineworld-launch`), standard library only. Three binaries, each a one-line
`main` naming its mode:

| Binary | Mode | Default world | Seats |
| --- | --- | --- | --- |
| `mineworld-2d-launch` | `2d` | `market-town` | 2D client as `carol` |
| `mineworld-3d-launch` | `3d` | `social-cafe` | 3D client as `visitor` |
| `mineworld-both-launch` | `both` | `social-cafe` | 2D client as `visitor`, 3D client as `wanderer` (`AC-15`'s two players) |

Every mode drives `alice` with the in-server rule controller (`--agent alice`), as the repository's root
launchers do. The bundle presents them as "MineWorld 2D", "MineWorld 3D" and "MineWorld 2D + 3D": macOS
`.app` wrappers, Windows `.exe` files, Linux executables with `.desktop` files (the assembling script's
concern, `scripts/package.py`).

On Windows the binaries use the GUI subsystem (`#![windows_subsystem = "windows"]`), so a double-click opens
no console; the server is started with `CREATE_NO_WINDOW`.

## 3. Arguments

All optional. A double-click passes none.

```text
--fresh               delete this mode's world save (saves/<world>/) before starting: the world starts over
--smoke=2d|3d|both    a scripted, headless check of the bundle (§7); never a player's mode
--no-dialog           report failures only in the log and on stderr, never in a dialog
--bundle=<dir>        the bundle root, instead of the one found from the executable's path (§4)
--user-dir=<dir>      the per-user folder, instead of the platform's (§5)
```

An argument not listed here is an error. `--smoke` replaces the mode's play run; its value need not equal the
binary's mode.

## 4. The bundle it reads

**Root.** The folder that holds the executable; or, when the executable is
`<name>.app/Contents/MacOS/<exe>`, the folder that holds `<name>.app`. `--bundle=<dir>` replaces it.

**Files under `<root>/runtime/`:**

| Path | What |
| --- | --- |
| `mineworld` (`mineworld.exe` on Windows) | the server binary |
| `worlds/<world>/` | a World Pack |
| `clients/2d.pck`, `clients/3d.pck` | the exported clients |
| `godot/godot.exe` (Windows), `godot/godot` (Linux), `godot/Godot.app/Contents/MacOS/Godot` (macOS) | the Godot runtime |
| `launch.toml` | optional overrides (§6) |

A missing file is an error naming its path. These paths are defined in one place, `src/bundle.rs`.

## 5. Per-user folder

The folder the clients' settings already use (`clients/shared/SETTINGS.md` §4):

| OS | Folder |
| --- | --- |
| Windows | `%APPDATA%\MineWorld\` |
| macOS | `~/Library/Application Support/MineWorld/` |
| Linux | `$XDG_DATA_HOME/MineWorld/` when `XDG_DATA_HOME` is set and absolute, else `~/.local/share/MineWorld/` |

`--user-dir=<dir>` replaces it. Nothing is ever written beside the executable: the bundle's folder may be
read-only (step-23 F-R8).

- **Saves:** `saves/<world>/`, passed to the server as `--save`. A play run resumes it; `--fresh` deletes it
  first.
- **Logs:** `logs/`, one set of files per run, named `<run>-launch.log`, `<run>-server.log`,
  `<run>-client-2d.log`, `<run>-client-3d.log`, where `<run>` is the run's start time in Unix seconds and
  the launcher's process id, `<seconds>-<pid>`. After writing its own, the launcher keeps the files of the
  five most recent runs and deletes the rest; a file whose name is not of that form is never touched.
- **Smoke scratch:** `scratch/<run>/`, used as the save of a smoke run and removed when the run ends, pass or
  fail.

## 6. `runtime/launch.toml`

Optional. When present it overrides a mode's world and seats. Its grammar is a strict subset of TOML, read
without a TOML library:

```text
file     = { line }
line     = blank | comment | section | pair
section  = "[" ( "2d" | "3d" | "both" ) "]"
pair     = key ws? "=" ws? '"' text '"'          (no escapes; text holds no '"')
comment  = "#" anything
keys     [2d] and [3d]: world, seat        [both]: world, seat-2d, seat-3d
```

Blank lines, comments and surrounding whitespace are ignored. A pair before any section, an unknown section
or key, a key given twice in one section, or any other line is an error naming the file, the line number and
what was expected. A `world` is one folder name under `runtime/worlds/` (no separator, not `.` or `..`). A
seat is not checked by the launcher: the server refuses an unknown seat to the client that asks for it.

## 7. A run

```text
1  find the bundle (§4), the per-user folder (§5), read launch.toml (§6)
2  play: saves/<world>/ (deleted first with --fresh); smoke: scratch/<run>/
3  start the server:
     runtime/mineworld server runtime/worlds/<world> --listen 127.0.0.1:0 --save <save>
                      --agent alice --stop-on-stdin-eof
     standard input: a pipe the launcher holds
     standard output and error: logs/<run>-server.log
     environment: the launcher's, without MINEWORLD_INVITE and MINEWORLD_ADMIN_TOKEN
4  read the join line from the server's log, every 100 ms for up to 60 s:
     [mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>
   the token follows "invite " up to the next space; the address follows " join with: " up to the next
   space. The server exiting first, or no line within 60 s, is an error naming the server's log.
5  start each client of the mode:
     <engine> --main-pack runtime/clients/<2d|3d>.pck [--headless]
              -- --root=<root>/runtime --server=<address> --seat=<seat> --invite=<token> [probe]
     standard output and error: logs/<run>-client-<2d|3d>.log
6  wait until every client has exited
7  stop the server (§8)
```

**Smoke (`--smoke=<mode>`)** runs the clients with `--headless` and a probe, on a scratch save:

| Client | Probe arguments |
| --- | --- |
| 2D | `--drive --settings=none` (the scripted checks; the client exits non-zero on failure) |
| 3D | `--slice-link --settings=none` (the slice's round trip; its verdict line is read by `package.py smoke`, not by the launcher) |

The invite is passed to the client on its command line, as the root launchers do; it is never written to the
launcher's own log. The server's log holds it, once, in the join line, as `server/PROTOCOL.md` §4.1 allows.

## 8. Stopping

When every client has exited, the launcher closes the server's standard input. With
`--stop-on-stdin-eof` (`docs/MODULE_SPEC.md`, `mineworld server`) the server takes its graceful path —
stops accepting connections, checkpoints the save, prints its statistics — and exits. The launcher waits up to
10 s; a server still running then is killed (that child only) and the run is reported as failed.

If the launcher itself ends early — killed, crashed, logged out — the operating system closes the pipe and
the server stops the same graceful way. The launcher never looks for a process by name.

## 9. Failures and exit status

A failure is one message naming the step that failed and the log to read. It is written to
`<run>-launch.log` and to standard error and, unless `--no-dialog` or `--smoke` is given, shown in a dialog:

| OS | Dialog |
| --- | --- |
| macOS | `osascript` `display alert`, the message passed as an argument, never interpolated into the script |
| Windows | `MessageBoxW` (one `user32` declaration; no `windows-sys` feature) |
| Linux | `zenity --error --no-markup`, else `kdialog --error`, when either is installed; otherwise none |

Exit status: in a smoke run, the clients' — the first non-zero, else 0; in a play run, 0 when the clients
exited and the server stopped gracefully. Any failure of the launcher itself, including a server that had to
be killed, is 1.
