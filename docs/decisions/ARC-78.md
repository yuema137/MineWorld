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


### Amendment — played towns and private host controls (2026-11-23)

Actual operator trial exposed a stale packaged launch path: only Alice had a reactive
controller, and the existing clock controls had no local host capability. Played runs now
use the existing `--town` policy (ARC-345), leaving the explicit Alice fixture for smoke.
This corrects launch integration without changing worlds, controllers or saved composition.

On 2026-11-23 the operator explicitly approved, in the Codex coordination session (relayed by the coordinator; recorded in `.structured-coding/plans/mvp0/handoff-coordinator.md` "November 23 continued execution"), “同意复用现有 getrandom 库” ("agree to reuse the existing getrandom library"). Item 2's standard-library-only
restriction is amended solely to reuse workspace `getrandom` for a per-run 128-bit OS-random
admin capability. Existing server code already uses that dependency for secrets. Handwritten
platform randomness, subprocess-generated secrets and a new server secret-output handshake
were rejected as unnecessary additional surface. The launcher keeps the capability in memory
and passes it only through owned child environments; failures fail closed before launch.
Inherited capabilities are replaced/removed and the secret never enters argv, logs or files.

The admin capability enables existing local World-tab clock controls. Item 4's stdin-EOF
shutdown remains unchanged and does not depend on the admin surface. No `--hosted-here`,
Keep Running or `--solo` is added; both clients may coexist and closing one does not stop
the other's world. Design/evidence: `.structured-coding/plans/mvp0/pr-launcher-living-correction.md`;
specification: `tools/launch/LAUNCHER.md` §7.1.

### Note — single-player play runs start `--solo`; Market Town by default (2026-11-27)

This note supersedes the amendment's sentence "No `--hosted-here`, Keep Running or `--solo` is added" for
single-client play runs. It changes nothing else in that amendment.

- **`--solo`.** A 2D or 3D play run (one player alone) now starts the server `--solo`, the single-player
  declaration of `ARC-69` item 5, as the source shell launchers already do. Both mode seats two players and
  never passes it; nor does smoke. `--solo` restricts no admission. It widens what `ARC-69` item 5 grants a
  single-player world: scales above 24×, and the sleep advance once nightlife N2's amending record lands.
- **Scale ceiling.** The shipped World tab still offers at most 24×, and the capability reaches only the
  launcher's own children, so the product ceiling is unchanged.
- **Defaults.** 2D plays Market Town as `visitor` instead of `carol`. 3D and Both keep Social Café until the Q-LS7 packaged timing gate passes; then one constant (`config.rs` `WORLD_3D`) switches them to Market Town.
- **Smoke.** Smoke plays a fixed fixture and no longer reads `launch.toml`.
- **No `--hosted-here`.** Still none is passed.
- **Approval.** The operator approved every recommendation of `pr-launcher-solo.md` (Q-LS1 … Q-LS7) on
  2026-11-27.
- **Specification:** `tools/launch/LAUNCHER.md` §§2, 6, 7, 7.1.
