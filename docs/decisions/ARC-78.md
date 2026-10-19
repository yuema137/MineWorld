## ARC-78 — Players get exported clients started by a windowless supervisor, `mineworld-launch`

**Date** 2026-10-10 · **Approved by** the operator's rulings on S23 (QR-1, QR-8, QR-10 as recommended;
step-23 §15), the step frozen by the primary session on 2026-10-10; number allocated by the primary session
· **Resolves** the open question of `ARC-6` · **Relates to** `ARC-6`, `ARC-44`, `ARC-80`, `DEP-29`,
[`MVP.md`](MVP.md) §7 · **Design** `.structured-coding/plans/mvp0/step-23-release.md` §6, §19;
specification [`tools/launch/LAUNCHER.md`](../tools/launch/LAUNCHER.md)

**Problem.** A downloadable demo must start with a double-click on macOS, Windows and Linux, with no
terminal and nothing to install: a local server on a world, the 2D or 3D client joined to it, and the
server stopped when the game closes. The bash launchers at the repository root (`mineworld-2d`,
`mineworld-slice`) need `bash`, `cargo` and a Godot on `PATH`; they are the contributors' path and cannot be
the players'.

**Decided.**

1. **Players get exported clients started by a supervisor; contributors keep the bash launchers over an
   installed Godot.** This answers `ARC-6`'s "exported native binaries or thin launchers over an installed
   Godot" for each audience.
2. The supervisor is **`mineworld-launch`** (`tools/launch`), a Rust crate on the standard library alone,
   with three entry points: "MineWorld 2D", "MineWorld 3D" and "MineWorld 2D + 3D" (both clients on one
   server — `AC-15`'s showcase, QR-10). Windows binaries use the GUI subsystem and start the server with
   `CREATE_NO_WINDOW`; macOS wraps each in a minimal `.app`; Linux ships `.desktop` files.
3. It is **a process supervisor and nothing else**: find the bundle, start `mineworld server` on a world
   with a save in the per-user `MineWorld` folder, read the server's join line (`server/PROTOCOL.md`
   §4.1), start the exported client with the join arguments, wait for the client(s),
   stop the server, report a failure in a dialog naming the log. It reads nothing from the server but the
   join line, holds no world state and no world rule, opens no window of its own, and stops only processes
   it started.
4. **The server is stopped through its standard input.** `mineworld server --stop-on-stdin-eof` treats
   end of input exactly like Ctrl-C. The launcher holds the pipe and closes it when the clients have
   exited, then waits 10 s before killing its own child. A launcher that dies closes the pipe by dying, so
   the server never outlives it — on every OS, with no signal, Job Object or `PR_SET_PDEATHSIG`, and with no
   admin surface (`ARC-44`) needed.

**Not the launcher `MVP.md` §7 excludes.** That launcher is a GUI over the runtime (world browser,
settings, accounts). `mineworld-launch` has no window, no choices beyond its command line and an optional
`launch.toml`, and adds no product surface; it is the root launchers' hosting logic made portable.

**Rejected** (step-23 §6.2). Shell scripts per OS (`.command` and `.bat` open a terminal); the Godot client
spawning its own server (puts hosting logic and the server's path into both clients, twice in GDScript,
and the clients would stop being pure protocol clients); a launcher framework such as Tauri or Electron (a
window toolkit to start two processes is a dependency larger than the problem, `REUSE_POLICY.md` §12).
No dependency is added: `MessageBoxW` is one `user32` declaration, as `DEP-29`'s note did for
`GenerateConsoleCtrlEvent`, so no `windows-sys` feature is needed.
