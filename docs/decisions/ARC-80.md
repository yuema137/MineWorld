## ARC-80 — The downloadable bundle: its layout, per-user data, and `--root` for exported clients

**Date** 2026-10-10 · **Status** selected · **Approved by** the primary session's S23 freeze (QR-1 … QR-13)
and its C0 freeze of R-a's plan (D-Ra-1 … D-Ra-4) · **Relates to** `ARC-6`, `ARC-78`, `ARC-79`, `DEP-36`,
`DEP-41` · **Design** `.structured-coding/plans/mvp0/step-23-release.md` §3, §6.5, §18

**Problem.** A player downloads one archive per operating system and runs MineWorld with no Godot, no
Rust toolchain and no repository. Exported clients cannot find files the way a checkout does: in a
release template `ProjectSettings.globalize_path("res://")` is `""` (step-23 §18.2, P-1), so the 2D
client's "relative to the repository root" and the shared text module's own folder have no meaning, and
Godot never packs the shared module's `.gdignore`'d catalogs and font (P-6). Official release templates
also refuse `--main-pack` (P-3), so one engine cannot run two packs.

**Choice — the layout.** One folder per target, archived as `.zip` (macOS, Windows) or `.tar.gz` (Linux):

```text
MineWorld-<version>-<target>/
  PLAY.txt  LICENSE  NOTICE  LICENSES/            notices (Rust: DEP-38; Godot; fonts; assets)
  <launcher entry points>                         ARC-78
  runtime/
    mineworld[.exe]                               the release server
    clients/mineworld-2d[.exe] + mineworld-2d.pck the exported 2D client (macOS: mineworld-2d.app)
    clients/mineworld-3d[.exe] + mineworld-3d.pck the exported 3D client, slice scene as main (macOS: .app)
    clients/shared/settings/locale/*.po           the shared catalog layer, as plain files
    clients/shared/settings/fonts/NotoSansSC-Regular.otf, OFL.txt
    presentation/mineworld-default/2D/{art,assets,i18n,renderer,manifest.yaml,pack.yaml}
    presentation/mineworld-default/3D/{far_field,characters/cast.yaml,characters/cast,manifest.yaml,pack.yaml}
    worlds/<world>/                               as tracked
    BUNDLE.toml                                   version, commit, target, build date, file-list digest
```

`<target>` is `macos-universal`, `windows-x86_64`, `linux-x86_64` or `linux-arm64`. Linux and Windows
clients carry their pack beside the executable (`mineworld-2d[.exe]` and `mineworld-2d.pck`, which Godot
loads by name), so the packaging checks can read every pack's directory; macOS clients are
ad-hoc-signed `.app`s with the pack in `Contents/Resources`. `BUNDLE.toml` names each client's
executable (`client_2d`, `client_3d`, relative to `runtime/`): a macOS client's program is named after its
Godot project, and the launcher reads the name rather than guessing it. Under `runtime/`, every file the clients read from disk keeps its path relative to
the repository root, so one argument locates all of them.

**Choice — `--root=<dir>`.** A user argument both clients accept: the absolute folder that stands for the
repository root. The 2D client resolves a relative `--presentation=` against it
(`clients/2d/PRESENTATION.md` §1); `MineWorldText.module_dir()` returns `<root>/clients/shared/settings`
(`clients/shared/SETTINGS.md` §2); the 3D client's `SliceMain.pack_dir()` returns
`<root>/presentation/mineworld-default/3D`, where it reads its far field (`ARC-102`) and its cast binding
(step-23 §18, D-Ra-5). Absent, both behave exactly as from a checkout. It changes where files
are read from, never what is read, in which order, or any rule.

**Choice — per-user data.** Saves go to `<per-user MineWorld folder>/saves/<world>/` and logs to
`…/logs/` (the folder of `clients/shared/SETTINGS.md` §4), never beside the executable: a quarantined
macOS app or a Windows install under `Program Files` runs from a read-only folder.

**Not shipped.** Evidence and tooling: `shots/`, `screenshots/`, `references/`, `candidates/`, `tools/`
(which holds the GPL Blender scripts), every `.md`, and any `.env`. No language model and no cognition
package (QR-8).

**Options compared** (step-23 §3.2): one engine with two packs (impossible with official templates,
P-3; it would need templates built from source, declined in `DEP-41`); two full exports (chosen; the
engine is carried twice, measured at 28–60 MB compressed per copy); thin launchers over an installed
Godot (the contributors' path, `ARC-78`, not a player's).

**Limitation accepted.** Each bundle carries the engine twice. `--root` is one more argument a launcher
must pass; a client started by hand from a bundle without it falls back to checkout behaviour and draws
plainly.
