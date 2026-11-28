# `mineworld-launch` — the bundle's supervisor

Specification of the launcher that starts a downloadable bundle's local server and client(s)
(`docs/DECISIONS.md` `ARC-78`; design `.structured-coding/plans/mvp0/step-23-release.md` §6 and §19). The
human introduction is [`README.md`](README.md).

## 1. What it is

A process supervisor for one machine. It finds the bundle it belongs to, starts `mineworld server` on a
world with a save in the per-user `MineWorld` folder, reads the server's join line, starts an exported client
with the join arguments, waits for the client(s) to exit, stops the server, and reports
a failure.

It never reads world state; the only thing it reads from the server is the join line
(`server/PROTOCOL.md` §4.1). With conversation on (§10) it also supervises one cognition process and reads
nothing from it but its closed status lines. It holds no world rule, opens no window of its own, makes no network connection,
and stops only the processes it started. The launcher itself invokes no model provider. The bundled
default towns have no decision list and use in-process rule controllers without model or network
services; a separately configured World Pack retains its own authored decision policy.

## 2. Entry points

One crate, `tools/launch` (`mineworld-launch`), using the standard library and the workspace
`getrandom` dependency for a private local host capability (ARC-78 amendment, 2026-11-23).
Three binaries, each a one-line
`main` naming its mode:

| Binary | Mode | Default world | Seats |
| --- | --- | --- | --- |
| `mineworld-2d-launch` | `2d` | `market-town` | 2D client as `carol` |
| `mineworld-3d-launch` | `3d` | `social-cafe` | 3D client as `visitor` |
| `mineworld-both-launch` | `both` | `social-cafe` | 2D client as `visitor`, 3D client as `wanderer` (`AC-15`'s two players) |

Every played mode starts `--town`: unoccupied seats use the world's resolved decision models
(default in-process paced rule controllers for the bundled towns), without pinning Alice to a
reactive-only controller. Scripted smoke retains `--agent alice`, without `--town`. Smoke verifies
its scripted fixture, not living-town behavior. The bundle presents them as "MineWorld 2D", "MineWorld 3D" and "MineWorld 2D + 3D": macOS
`.app` wrappers, Windows `.exe` files, Linux executables with `.desktop` files (the assembling script's
concern, `scripts/package.py`).

On Windows the binaries use the GUI subsystem (`#![windows_subsystem = "windows"]`), so a double-click opens
no console; the server is started with `CREATE_NO_WINDOW`.

## 3. Arguments

All optional. A double-click passes none.

```text
--fresh               delete this mode's world save (saves/<world>/) and its cognition store
                      (cognition/<world>/, §10.3) before starting: the world starts over
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
| `clients/mineworld-client.exe` (Windows), `clients/mineworld-client` (Linux) | the bundle's one Godot runtime, which starts either client |
| `clients/MineWorld.app` (macOS) | the same runtime; the launcher starts the one executable in `Contents/MacOS/`, whatever the export named it |
| `clients/mineworld-2d.pck`, `clients/mineworld-3d.pck` and their `.settings.json` | the clients' packs and run-time settings, which the runtime loads |
| `launch.toml` | optional overrides (§6) |
| `decisions/market-town-alice-lm.yaml` | the decision list a cognition profile names (§10.1); read only then |
| `cognition-profile.template.toml` | an inactive, commented profile to copy (§10.1); never read |

Official Godot release templates are built with `disable_path_overrides` and refuse `--main-pack` (step-23
§18, probe P-3). So the bundle carries one exported runtime (`clients/runtime/`) that mounts the client's pack
itself, chosen by `--client=<2d|3d>` (step-23 PR S23-RT, amending D-Ra-1 and D-RC-4; the bundle layout is
`ARC-80`'s). The launcher checks the runtime, not the packs: the runtime names a missing pack. A missing file — or a macOS `Contents/MacOS/` holding other than exactly one file — is an error
naming its path. The server is checked when the bundle is found; a client when its mode needs it. These paths
are defined in one place, `src/bundle.rs`.

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
- **Cognition:** `cognition/profile.toml`, the optional profile that turns conversation on (§10.1), and
  `cognition/<world>/`, a world's cognition store (§10.3), deleted with its save by `--fresh`. A world
  named `profile.toml` has no store.
- **Logs:** `logs/`, one set of files per run, named `<run>-launch.log`, `<run>-server.log`,
  `<run>-client-2d.log`, `<run>-client-3d.log` and, when conversation is on, `<run>-preflight.log` and
  `<run>-cognition.log`, where `<run>` is the run's start time in Unix seconds and
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

Blank lines, comments and surrounding whitespace are ignored; a comment is a whole line (a `#` after a value
is refused, not read as a comment). A pair before any section, an unknown section
or key, a key given twice in one section, or any other line is an error naming the file, the line number and
what was expected. A `world` is one folder name under `runtime/worlds/` (no separator, not `.` or `..`). A
seat is not checked by the launcher: the server refuses an unknown seat to the client that asks for it.

## 7. A run

```text
1  find the bundle (§4), the per-user folder (§5), read launch.toml (§6)
2  play: saves/<world>/ (deleted first with --fresh); smoke: scratch/<run>/
3  start the server:
     runtime/mineworld server runtime/worlds/<world> --listen 127.0.0.1:0 --save <save>
                      [play: --town; smoke: --agent alice] --stop-on-stdin-eof
     standard input: a pipe the launcher holds
     standard output and error: logs/<run>-server.log
     environment: the launcher's without MINEWORLD_INVITE; private played-run
                  MINEWORLD_ADMIN_TOKEN, or no admin token in smoke
4  read the join line from the server's log, every 100 ms for up to 60 s:
     [mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>
   the token follows "invite " up to the next space; the address follows " join with: " up to the next
   space. The server exiting first, or no line within 60 s, is an error naming the server's log.
5  start each client of the mode:
     <the runtime> [--headless]
              -- --client=<2d|3d> --root=<root>/runtime --server=<address> --seat=<seat> --invite=<token>
                 --world-key=<world> [probe]
     (--world-key names the world the client draws; a joining client refuses to enter without it, RL-e0)
     standard output and error: logs/<run>-client-<2d|3d>.log
     environment: the same private played-run MINEWORLD_ADMIN_TOKEN; removed in smoke
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

### 7.1 Local host capability

Before starting a played server, the launcher generates 16 bytes using the operating system
random source through `getrandom`, encoding them as 32 hex characters. Failure to obtain randomness
fails the run before a child starts. The capability is independent of the server-generated invite.
Only the owned server and clients receive it, in their environment; it is never put in an argument,
log, file or parent environment. Inherited admin capabilities are replaced in played runs and
removed from both child types in smoke runs. The server mounts its existing loopback admin
surface; the client's existing World tab controls pause/resume and day length (4/2/1 hours).
The server remains the clock authority. No world schema or saved composition changes.

The supervisor deliberately passes neither `--hosted-here` nor `--solo`. A bundled client
therefore exposes clock controls but not the shell launcher's Keep Running preference. Both mode
never obtains solo fast-forward permission. Closing one client leaves the other connected;
all clients must exit before the supervisor stops the server. The shutdown contract below
continues to use stdin EOF, independently of the admin surface.

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
Conversation never fails a run: a cognition process that cannot start, or stops, is logged with its reason
(§10.5), and the world plays with its rule controllers.

## 10. Conversation: the cognition profile and the cognition child

Design and evidence: `.structured-coding/plans/mvp0/pr-playable-dialogue-memory.md` (DM-20261127-3) §§3–4;
the runtime route is `docs/decisions/ARC-372.md`; the cognition side of every contract below is
[`docs/reference/cognition-supervised.md`](../../docs/reference/cognition-supervised.md). The launcher handles
only opaque paths and the closed status vocabulary: it never reads the cognition configuration, never names a
model, endpoint or key, never downloads a model or a runtime, and never starts or stops a model service.

### 10.1 The profile

`cognition/profile.toml` in the per-user folder (§5). It survives package upgrades and read-only bundles. **No
profile, no conversation:** the run is exactly §7. Deleting the file turns conversation off again. A smoke run
ignores it. The bundle ships `runtime/cognition-profile.template.toml`, fully commented and never read.

When the profile exists it **replaces** `launch.toml` for every mode: a mode's section gives its world and
seats as `launch.toml` would (§6), and a mode without a section plays its defaults (§2). Its grammar is §6's,
read by the same parser, with four more keys, which a section gives all together or not at all:

| Key | Value |
| --- | --- |
| `cognition-python` | absolute path of the interpreter of a MineWorld checkout's `uv sync --locked` environment (on Windows, a `.exe`) |
| `cognition-config` | absolute path of the operator's cognition configuration (`docs/reference/cognition-toml.md`) |
| `cognition-manifest` | absolute path of the runtime manifest that interpreter wrote (`cognition-supervised.md` §3) |
| `cognition-decisions` | the decision list, relative to the bundle root, with plain components only: `runtime/decisions/market-town-alice-lm.yaml` |

Some but not all four, a relative interpreter, configuration or manifest, a decision list with `..` or an
absolute path, a key given twice, or an unknown key: the run fails before any child starts, naming the file
and line, as for `launch.toml`. A section without the keys plays that mode without conversation.

The operator's profile (design Q-D2): Market Town for every mode; the player is `visitor` in 2D and in 3D, and
`visitor` (2D) with `wanderer` (3D) in 2D + 3D; the shipped list drives only `alice` by the LM controller.

### 10.2 A run with conversation on

```text
1  as §7 step 1, with the profile in place of launch.toml; create cognition/<world>/
2  preflight: <python> -I -m mineworld_cognition --config <config> --preflight <manifest>
     standard output: one status line, read; standard error: logs/<run>-preflight.log; 30 s, then killed
     anything but `starting` and exit 0 ⇒ conversation off for this run (§10.5), and step 3 is §7's
3  start the server as §7 step 3, plus  --decisions <bundle>/<cognition-decisions>
4  read the join line (§7 step 4)
5  the assignment: <runtime>/mineworld decisions <world> --decisions <the same file> --json
     standard output: cognition/<world>/run/assignment.json; 30 s, then killed; failure ⇒ conversation off
6  start the cognition child (§10.4)
7  start the clients (§7 step 5); they never wait for cognition
8  when every client has exited: stop the cognition child (§10.4), then the server (§8)
```

The server and the assignment always read the same decision list. While no cognition process holds Alice's
seat, the list's fallback (`rule`) drives her in the server.

### 10.3 The world's cognition store

`cognition/<world>/` in the per-user folder: the cognition process's memory home, resume and decision-log
files, budget ledger, the `instance` binding (`cognition-supervised.md` §4), and `run/assignment.json`. It
persists across restarts and package upgrades.

- **Lifecycle owner: the launcher.** It creates the folder when conversation is on, and `--fresh` deletes
  it together with `saves/<world>/` (design B1): no private conversation outlives the world it belongs to,
  and disk use stays bounded by the live save. Another world's store and the profile are never touched.
- **Binding owner: cognition.** The first welcome binds the store to the world instance; a store bound to
  another instance is refused (`instance_mismatch`) and left as it is, never silently erased.
- **Restore caveat.** MineWorld has no backup or restore feature. Restoring an older copy of a save by hand
  while keeping a newer store can leave the store ahead of the world: unsupported. A save of another world
  instance is refused as above; a same-instance rollback is not detected.
- Log pruning (§5) never touches `cognition/`. A smoke run starts no cognition and needs no store.

### 10.4 The cognition child

```text
<python> -I -m mineworld_cognition --config <config> --supervised --server-url=ws://<address>/ws
         --store-dir <user>/cognition/<world> --assignment <user>/cognition/<world>/run/assignment.json
standard input:   a pipe the launcher holds
standard output:  status lines (cognition-supervised.md §2), read by the launcher's watcher thread
standard error:   logs/<run>-cognition.log
environment:      the launcher's, without MINEWORLD_ADMIN_TOKEN, with MINEWORLD_INVITE = the join line's invite
Windows:          CREATE_NO_WINDOW; the interpreter must be a .exe
```

The command and its environment are built by `src/cognition.rs` and nowhere else; never through
`Hosting::configure_client` or `configure_server`, so the played run's admin capability (§7.1) never reaches
cognition. The invite is in the child's environment only: never in its arguments, its log, its status lines
or its store (the server's log and the clients' arguments hold it, §7).

**Stopping.** When the last client exits, or the run fails after the child started, the launcher closes the
child's standard input: the process stops serving, leaves its seat (5 s) and exits (its own bound is 15 s).
The launcher waits up to 20 s, then kills that child only; then it stops the server (§8). The first of two
clients to exit stops nothing (§7.1). A launcher that dies closes the pipe by dying, and the child stops the
same way. Only owned child handles are killed or reaped: no process group, no search by name, and a model
service is never touched. An HTTP request in flight is cancelled with the process; a local model service may
still finish that computation on its own.

### 10.5 States

The launcher logs each state of the cognition child, never anything the process says beyond the closed status
vocabulary:

| State | Meaning |
| --- | --- |
| `disabled` | no profile, or no cognition keys in this mode's section: nothing is logged, nothing started |
| `starting` | the child was started and has not yet held its seat |
| `ready` | the child's status line says the seat's welcome was received and its memory home is held; never a spawn or an open endpoint alone |
| `degraded(reason)` | conversation is off for the rest of the run; the world plays with its rule controllers |
| `stopped` | the child exited after its input closed |

The launcher's own reasons, in addition to the process's (`cognition-supervised.md` §2.3):

| Reason | When | The child |
| --- | --- | --- |
| `profile_invalid` | the profile's decision list is not in the bundle; the store cannot be created; the child's status names a seat that is a human seat of this mode | stopped, if started |
| `preflight_failed` | the interpreter is missing or not a `.exe` on Windows, the preflight crashed, printed no valid line, or ran past 30 s | none started |
| `readiness_timeout` | no `ready` within 60 s of the start | stopped |
| `status_invalid` | a malformed status line | stopped |
| `child_exited` | the child could not be started, or exited before it was asked to stop | already gone |

A failure before the server starts (the profile's decision list, the preflight) starts the server exactly as
without a profile: no `--decisions`. Every state change is one line in `<run>-launch.log`, with what the
player can do about it.
