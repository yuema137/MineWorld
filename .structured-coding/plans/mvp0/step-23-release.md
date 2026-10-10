# Step 23 — S23 (placeholder): a downloadable public demo

**Lifecycle:** `DESIGN FROZEN 2026-10-10 (primary session), on the operator's rulings QR-1, QR-3, QR-4,
QR-8, QR-12 and QR-13 (§15).` The step plan (scope, invariants, PR split, adversarial criteria) is frozen.
**R-a and R-c** carry execution contracts (§17) and may start, each in a fresh implementation session and
its own worktree. Following `overall.md` §5 ruling 9, each implementing session's first commit details its
PR to the commit (Markdown only, in §17's PR section), and the primary session freezes that detail before
code begins. **R-b and R-d** stay at medium scope (§13) and are frozen on their own later. Nothing here
authorizes a merge: every PR merges only after the primary session's review and the operator's
authorization (`CLAUDE.md` §3.1). Progress, evidence and bounded corrections stay writable.
**Author:** the release planning session, 2026-10-10. Draft: worktree
`/Users/yuema137/mineworld-worktrees/plan-release`, branch `docs/release-design`, from
`origin/main @ 5ccc402`, merged as PR #136 (`a926fbf`). Freeze: branch `docs/s23-freeze` from
`origin/main @ a926fbf`.
**Binding parents:** `CLAUDE.md` §§1.1, 2–4; `overall.md` §5 ("Every platform", "Realistic defaults",
"Late evening of 2026-10-09" CI rule, Milestone F, QB-14), the D-1 licence ruling and its publication
decisions; `docs/MVP.md` §7; `docs/REUSE_POLICY.md` §§1–15; `docs/DECISIONS.md` ARC-6, DEP-4, DEP-8,
DEP-17 to DEP-19, DEP-22, DEP-31, DEP-36, ARC-48 and its notes; `step-14-ci.md` §5.7, §13.0.2 (R-SET-10),
W-11 and QB-14; `clients/shared/SETTINGS.md` §4.
**Scope of this file.** It is the only file this session writes. Edits to `overall.md`,
`docs/DECISIONS.md`, `docs/MVP.md` and other step documents are proposed in §16 for the primary session to
apply. Decision numbers were assigned by the primary session on 2026-10-10: ARC-78, ARC-79, ARC-80 and
DEP-38 … DEP-42 (§12). The records themselves are written into `docs/DECISIONS.md` by the implementing PRs,
not by this document. The draft's DEP-R6 (`rcodesign`) has no number, because notarization is deferred
(QR-3).

Every statement about an external service carries the date it was read (2026-10-10) and its source. A
statement this session could not confirm from a primary source is marked **verify**, and the PR that
first relies on it must confirm it before freezing.

---

# 1. The requirement

## 1.1 The operator's words (2026-10-09, relayed by the primary session)

After MVP-0, publish a downloadable public demo. Not Steam. The primary session's starting
recommendation, to be evaluated rather than assumed:

- GitHub Releases as the canonical artefact store, built by CI on a version tag, for macOS (arm64 and
  x86_64, or universal), Windows x86_64 and Linux x86_64 (plus arm64 if cheap);
- itch.io as the player-facing page, uploaded with `butler` (an operator API key in GitHub secrets);
- Zenodo's GitHub integration to archive each release with a DOI (enabled once by the operator);
- Hugging Face (Hub hosting or Spaces), Flathub/AppImage, Homebrew cask, winget and Scoop evaluated
  briefly.

## 1.2 Standing rules this step must satisfy

- **Every platform** (`overall.md` §5): macOS, Linux and Windows each get a bundle, and each bundle is
  smoke-tested on its own OS. A platform that cannot be tested is a recorded finding, not a silent skip.
- **Framework, not demo** (`overall.md` §5, "Framework, not demo"; `ARC-6`): the bundle demonstrates the
  runtime. It ships the server, the two reference clients and sample World Packs, unmodified. Nothing in it
  is a fork of the source for distribution, and no rule moves into a launcher or a client.
- **Kernel ignorance and no leakage** (`CLAUDE.md` §4 items 2–3): packaging adds no concept to a contract.
  The launcher is a process supervisor; it decides nothing about the world.
- **A world with every model removed is valid** (`CLAUDE.md` §1): the demo runs with the rule
  controller only. No language model, no network access beyond loopback, no API key.
- **Reuse before reinvention** (`REUSE_POLICY.md`): every substantial tool adopted or declined here has a
  decision record (§12).
- **Documentation-only changes do not run the build** (ARC-48 note, 13x): this design is docs-only.

## 1.3 The answer in brief

1. **Canonical store: GitHub Releases.** Free, no bandwidth limit, 2 GiB per file, attestable. A tag
   `v*` on `main` builds four bundles in CI, smoke-tests each on its own OS, and creates a **draft**
   release. The operator publishes the draft; publishing is the human checkpoint.
2. **Player page: itch.io**, fed by `butler` from the published release, in a separate workflow guarded by
   a GitHub Environment that holds the only secret (`BUTLER_API_KEY`).
3. **Archive: Zenodo's GitHub integration**, enabled once by the operator, plus `CITATION.cff` and
   `.zenodo.json` in the repository. It mints a DOI per published release.
4. **Hugging Face: declined** for this purpose (§4.5); Flathub, Homebrew, winget and Scoop: deferred (§4.6).
5. **One bundle per OS** holds the release server binary, one Godot 4.7.2 runtime with the 2D and 3D
   clients as two packs, the three sample worlds, the 2D Presentation Pack's runtime files, a small Rust
   launcher with two entry points ("MineWorld 2D", "MineWorld 3D") that starts a local server and the
   client with no terminal, `LICENSES/`, `NOTICE` and `PLAY.txt`.
6. **Unsigned by default** for the pre-alpha demo, with plain per-OS instructions in `PLAY.txt` and on the
   itch page. Signing and notarization are the operator's choice (§8, QR-3, QR-4).
7. **Four PRs:** R-a (export presets, client path fixes, local packaging script), R-b (CI release workflow,
   checksums, attestations, SBOM), R-c (launcher and per-OS smoke tests), R-d (publishing to itch.io and
   Zenodo, changelog and release notes). R-a and R-c may run in parallel; R-b needs both; R-d needs R-b.
8. **Not blocking MVP-0 acceptance.** R-a and R-c can start now; the first public tag waits for Milestone F
   and for the 3D visuals to settle (§10).

---

# 2. Audit (`origin/main @ 5ccc402`, 2026-10-10)

## 2.1 What exists

| Fact | Evidence |
| --- | --- |
| Workspace version is already `0.1.0` (Milestone E, overall §5 "Parallel build-out"); no git tag and no GitHub release exist | `Cargo.toml:27`; `git tag` empty; `gh release list` empty |
| Repository is public, licence MIT (`D-1`, revisited 2026-10-08) | `gh api repos/yuema137/MineWorld` → `public`, `MIT` |
| The server is the `mineworld` binary of `mineworld-cli` (`mineworld server <world> --listen 127.0.0.1:0 --save <dir> --agent alice`); with no `--invite` it prints one join line `[mineworld] invite <token> — join with: <address> seat=… invite=<token>` | `tools/cli/Cargo.toml:12–13`; `mineworld-2d:118–145`; `server/PROTOCOL.md` §4.1 |
| System Packs are compiled in (`ARC-33`, `DEP-12`); a world is a directory read at start. The third-party pack (`DEP-23`) is a pinned git dependency compiled in | `systems/installed`; `ARC-66` |
| Worlds on `main`: `social-cafe` (84 KB), `market-town` (308 KB, includes `data/weather/NOTICE` for the NOAA record, `DEP-31`), `bodies-yard` (128 KB) | `du -sh worlds/*` |
| Three root launchers, all bash, all needing Godot on `PATH` and `cargo`: `mineworld-2d` (hosts a world on a free port, reads the join line, saves under `clients/2d/.save/<world>`, stops only its own server), `mineworld-3d` (the promenade spike, no server), `mineworld-slice` (the 3D slice; `--world` hosts `social-cafe`) | the three files at the repository root |
| Godot projects: `clients/2d` (`run/main_scene="res://scenes/app.tscn"`), `clients/3d-spike` (`res://scenes/main.tscn`; the playable connected client is `res://scenes/slice.tscn`), `clients/protocol`, `clients/shared`; all `config/features` "4.7", Forward Plus for the playable two | `clients/*/project.godot` |
| No `export_presets.cfg` exists in any client | `ls clients/*/export_presets.cfg` → none |
| Shared modules reach the clients through tracked symlinks: `clients/{2d,3d-spike}/mineworld → ../protocol/mineworld`, `…/mineworld_settings → ../shared/settings` | `ls -la clients/2d`; `step-14-ci.md` §13.0.2 |
| **CI installs no Godot at all.** The planned `clients` job (13c, `step-14-ci.md` §5.7: official Linux binary, checksum pinned, cached; `barichello/godot-ci` declined) has not landed | `.github/workflows/ci.yml`; `grep -i godot .github scripts/ci_layer.py` → nothing |
| CI runner labels in use: `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-26`, `macos-15`, `windows-2025`; every action pinned by commit SHA, enforced by `scripts/check_ci_pins.py` | `ci.yml`; `DEP-19` |
| The Linux toolchain container is `rust:1.97.1-slim-trixie` (Debian 13, glibc 2.41) | `Dockerfile:13` |
| Godot 4.7.2-stable was published 2026-08-18. Assets: `Godot_v4.7.2-stable_export_templates.tpz` 1 281 349 702 bytes; `Godot_v4.7.2-stable_linux.x86_64.zip` 77 860 424 bytes; `SHA512-SUMS.txt` | `gh api repos/godotengine/godot/releases/tags/4.7.2-stable`, read 2026-10-10 |
| Sizes of tracked trees: `clients/3d-spike` 187 MB (of which `assets/` 149 MB: models 75, textures 42, characters 27, HDRI 4.4; plus `shots/` 25 and `screenshots/` 11 that are evidence, not runtime); `clients/2d` 21 MB (of which `shots/` 20); `presentation/mineworld-default/2D` 51 MB (runtime: `art/` 7.4 MB, `assets/`, `i18n/`, `renderer/`, `manifest.yaml`, `pack.yaml`; evidence: `candidates/` 33 MB, `references/` 11 MB) | `du -sh` on a fresh worktree |
| Largest runtime files: `characters/meshy_d/meshy_d.glb` 10.1 MB, its base colour PNG 6.1 MB, `characters/vitruvian/vitruvian.glb` 4.6 MB, `hdri/qwantani_puresky_2k.hdr` 4.6 MB | `git ls-files … | xargs ls -l | sort` |
| Licences to carry: MIT (`LICENSE`), `NOTICE` (GPL exception for `clients/3d-spike/tools/blender/`, OFL 1.1 for `NotoSansSC-Regular.otf`, CC0 third-party assets, AI-generated assets, NOAA GHCN-Daily CC0/public domain) | `NOTICE`; `clients/shared/settings/fonts/OFL.txt`; `clients/3d-spike/ASSETS.md`; `presentation/mineworld-default/LICENSES/` |
| Per-user folder already fixed for settings: `%APPDATA%\MineWorld\`, `~/Library/Application Support/MineWorld/`, `$XDG_DATA_HOME/MineWorld/` or `~/.local/share/MineWorld/` | `clients/shared/SETTINGS.md` §4 |
| The server stops gracefully on Ctrl-C, and on Windows also Ctrl-Break, close and shutdown console events; it has no stdin- or parent-based stop | `tools/cli/src/serve.rs:244–315` |
| `ARC-6` left open "whether `mineworld-2d` and `mineworld-3d` are exported native binaries or thin launchers over an installed Godot"; `MVP.md` §7 rules a launcher *GUI* out of MVP-0 | `docs/DECISIONS.md:157–186`; `docs/MVP.md:295–307` |

## 2.2 Findings that block packaging (each owned by a PR in §13)

- **F-R1 — the 2D client finds its Presentation Pack through `res://../..`.** `app.gd:370–373`
  `repo_path()` returns `ProjectSettings.globalize_path("res://")/../../<path>`. In an exported build `res://`
  is inside a pack, so this does not name the bundle's folder (**verify** the exact return value of
  `globalize_path("res://")` in a 4.7.2 release template; either way it is not the bundle root). The pack
  (`presentation/mineworld-default`) is read with `FileAccess` from the file system (`presentation.gd:108,
  145, 186`). Owner: R-a.
- **F-R2 — the shared text module globalizes `res://` paths.** `clients/shared/settings/text.gd:49`
  `module_dir()` globalizes the module's own `res://` folder and reads `locale/` from the file system; the
  `.po` catalogs are non-resource files, so a default export would leave them out. Owner: R-a, in
  coordination with S20 (the module's owner).
- **F-R3 — several 3D resources are loaded by constant string at run time** (for example
  `human.gd:27,33,37`), so Godot's "export selected scenes and dependencies" would miss them. R-a exports all
  resources with exclusion filters and proves completeness by running the exported client (§7).
- **F-R4 — the 3D client's playable scene is not its main scene.** `main.tscn` is the promenade spike;
  the connected client is `slice.tscn`. Release templates do not honour `--scene` (Godot command-line
  reference, read 2026-10-10: `--scene` needs `disable_path_overrides=false`). Recommended fix with no code:
  a feature-tag override `application/run/main_scene.mineworld_release="res://scenes/slice.tscn"` set by the
  export preset's custom feature (**verify** in R-a by probe; fallback: a three-line boot script reading a
  user argument).
- **F-R5 — the Linux server built in the toolchain container needs glibc 2.41**, newer than Ubuntu 24.04
  (2.39) and 22.04 (2.35). A release binary must target an older floor (QR-6).
- **F-R6 — the 3D probes exit 0 on failure** (`slice_probe.gd:160`; `step-14-ci.md` F-5). A smoke test must
  read a verdict line, and a missing line is FAIL in the release smoke test (not INCONCLUSIVE: a release is
  not published on an unknown).
- **F-R7 — the server has no stop that a windowless parent can send on Windows.** A GUI-subsystem launcher
  has no console to send Ctrl-Break into. The launcher needs a portable graceful stop (§6.4); the server
  owner (S11 lane) adds it.
- **F-R8 — the bundle's own folder may be read-only.** macOS App Translocation runs a quarantined app from a
  randomized read-only path (**verify** on macOS 26), and Windows users unpack into `Program Files` or a
  read-only share. Saves must therefore go to the per-user folder, never beside the executable.

---

# 3. What a demo bundle contains

## 3.1 Layout (one archive per target)

```text
MineWorld-0.1.0-<target>/
  PLAY.txt                       start here: how to open, what to do, unsigned-app steps, uninstall
  LICENSE  NOTICE                copied from the repository root, unmodified
  LICENSES/
    THIRD_PARTY_RUST.html        generated by cargo-about from Cargo.lock at the tag (DEP-38)
    godot/LICENSE.txt            Godot's MIT licence, from the 4.7.2-stable source tag
    godot/COPYRIGHT.txt          Godot's third-party notices (FreeType, ICU, mbedTLS, Jolt, …)
    fonts/OFL.txt                Noto Sans SC (DEP-36)
    assets/                      clients/3d-spike/ASSETS.md and presentation/mineworld-default/LICENSES/*
  MineWorld 2D[.exe|.app]        launcher entry point (§6)
  MineWorld 3D[.exe|.app]        launcher entry point (§6)
  runtime/
    mineworld[.exe]              the release server binary
    godot/<engine runtime>       one Godot 4.7.2 release template (macOS: an .app holding it)
    clients/2d.pck               the exported 2D client
    clients/3d.pck               the exported 3D client (slice scene as main, F-R4)
    worlds/social-cafe/  worlds/market-town/  worlds/bodies-yard/      copied as tracked
    presentation/mineworld-default/2D/{art,assets,i18n,renderer,manifest.yaml,pack.yaml}
    BUNDLE.toml                  version, commit, target, build date, the file list's digest
```

- `<target>` is one of `macos-universal`, `windows-x86_64`, `linux-x86_64`, `linux-arm64` (QR-2).
- Archive format: `.zip` for macOS and Windows (what their file managers open), `.tar.gz` for Linux (keeps
  the executable bit). macOS `.dmg` is optional and needs a macOS runner (Godot's macOS export page, read
  2026-10-10: "DMG images can only be created when exporting from macOS"); declined for the first release.
- **Evidence is not shipped:** `shots/`, `screenshots/`, `references/`, `candidates/`, `tools/` (which holds
  the GPL Blender scripts) and every `.md` stay out. Adversarial criterion A-R3 checks this.
- **Cognition is not shipped.** The demo is LM-free (`CLAUDE.md` §1; QR-8). Python, `uv` and the SDK are
  absent.
- **`PLAY.txt` is plain text** (not Markdown, so the documentation law's spec rule does not apply to it):
  about 40 lines in English with a Simplified Chinese section, matching the clients' two languages (`ARC-70`).

## 3.2 One engine, two packs (recommended) versus two full exports

| Option | Size per bundle | Cost |
| --- | --- | --- |
| (a) **One Godot release template plus `2d.pck` and `3d.pck`**, chosen with `--main-pack` by the launcher | engine binary once | `--main-pack` must work in a 4.7.2 release template (**verify** by probe in R-a); macOS app name and icon are the runtime's, not per client |
| (b) Two full exports (`--export-release`), each an executable with its pack embedded | engine binary twice: about +80 to +190 MB uncompressed per bundle (**verify**: measure in R-a) | none beyond size |
| (c) Thin launchers over a Godot the player installs | smallest | fails "no terminal, no prerequisites"; this is what the root launchers do today |

Recommendation: **(a)**, with (b) as the recorded fallback if the probe fails. (c) stays the contributors'
path from source.

## 3.3 Per-user data

- Saves: `<per-user folder>/saves/<world>/` (the same `MineWorld` folder as settings, `SETTINGS.md` §4).
  The launcher passes `--save` with that absolute path (F-R8).
- Logs: `<per-user folder>/logs/` — the server log and the launcher log of the last five runs.
- "Start the world over" is a launcher option (`--fresh`, and a line in `PLAY.txt` naming the folder).
- Uninstall: delete the unpacked folder and, optionally, the per-user folder. `PLAY.txt` says both.
- Risk carried: `F-SAVE-1` (a 30-day Market Town save is 266 MB). Demo players rarely run 30 days, but
  `PLAY.txt` names the folder so a player can see and remove it (§11, R-R5).

---

# 4. Where it is published — comparison (read 2026-10-10)

## 4.1 GitHub Releases — adopt as the canonical store

- Limits (docs.github.com, "About releases"): "Each file included in a release must be under 2 GiB." "Up to
  1000 release assets may be associated with a single release." "There is no limit on the total size of a
  release, nor bandwidth usage."
- Fits: artefacts live beside the tagged source; GitHub artifact attestations bind each file to the
  workflow run (§5.4); `gh release` is scriptable; no account for players; zero cost on a public repository.
- Limits for players: no store page, no screenshots gallery, no installer experience. That is itch.io's role.

## 4.2 itch.io via `butler` — adopt as the player page

- `butler` is itch.io's official, MIT-licensed upload tool, "easily integrated into an automated build/deploy
  pipeline"; in CI it authenticates with the environment variable `BUTLER_API_KEY`
  (itch.io/docs/butler, `login.html`). Latest release `v15.32.0` (`gh api repos/itchio/butler`, 2026-10-10).
- Channels: a channel name containing `windows`/`win`, `mac`/`osx`, `linux` sets the platform tag; channels
  cannot be tagged by architecture; `--userversion` carries our version; "the itch.io backend will reject
  builds with a total uncompressed size that exceeds 30GB" (`pushing.html`). Channels: `windows`, `mac`,
  `linux` (x86_64), `linux-arm64` (**verify** how the itch app treats a second Linux channel; it cannot
  distinguish architectures).
- Terms: hosting free, no ads, open revenue share, minimum price may be 0 (creators FAQ). Per-file size is
  "limited" and can be raised by request — the number is not stated (**verify**; butler builds are diffed and
  are not subject to the web uploader's per-file limit as far as read, **verify**).
- Obligations: the AI Disclosure section must be filled ("We ask that you accurately tag your project if it
  contains materials produced by generative AI"; quality guidelines). The bundle contains AI-generated assets
  (`NOTICE`), so the page **must** declare them. Only platforms actually tested may be ticked.
- Operator actions: create the itch.io project page; create an API key; store it as the secret
  `BUTLER_API_KEY` of a GitHub Environment `itch` with the operator as required reviewer (QR-5).

## 4.3 Zenodo via the GitHub integration — adopt as the archive

- Behaviour (docs.github.com, "Referencing and citing content"): "Zenodo archives your repository and issues a
  new DOI each time you create a new GitHub release." Owner steps: repository public (true), licence present
  (true), log in to Zenodo with GitHub, toggle the repository on.
- What is archived: the source archive of the tag. Whether release assets are included as well is not
  confirmed (**verify**; the design assumes source only, which is what a citation needs).
- Limits: 50 GB per record, 100 files (Zenodo support FAQ as summarized by search, **verify** on the page);
  far above our need.
- Metadata: Zenodo reads `CITATION.cff` and `.zenodo.json` (help.zenodo.org index lists both; precedence
  **verify**). R-d adds `CITATION.cff` (author, title, licence, version, repository URL) — GitHub also renders
  it as "Cite this repository".
- Draft releases do not trigger Zenodo; only publishing does. Pre-releases do trigger it (**verify**); QR-9
  asks whether release candidates should be archived.

## 4.4 Two alternatives for the canonical store, declined

- **itch.io as canonical:** no provenance binding to the source, an account-bound API, and a store page that
  can be delisted; it is the right page for players, the wrong single source of truth.
- **A self-hosted object store (S3, R2) or GitHub Pages:** costs money or has size limits, and adds an
  operational surface GitHub Releases already removes.

## 4.5 Hugging Face — declined for this purpose (revisit for a hosted server)

- **Hub file hosting.** Free public storage is "best-effort", and the Hub asks that uploads be "as useful to
  the community as possible"; its guidance and tooling are for models and datasets (storage-limits page, read
  2026-10-10). A game client is neither. It gives players nothing GitHub Releases does not, adds a second
  token, and has no attestation story. Decline.
- **Spaces.** Static Spaces are free; "Gradio and Docker Spaces run on compute and require a paid plan to
  create"; outbound and served ports are limited to 80, 443 and 8080; free hardware sleeps when unused
  (Spaces overview, read 2026-10-10). A static Space could host a web build, but the 3D client is Forward Plus
  and a Godot web export needs the Compatibility renderer, and MineWorld has no web client (`ARC-18`). A
  Docker Space could host a public `mineworld server` behind `wss://` on 443, which is the right shape for a
  later "join a hosted world" demo, but it needs a paid plan, its disk is not persistent by default, and a
  public server raises moderation and join-secret questions MVP-0 left out (relay and public worlds are
  non-goals). **Revisit with MVP-1's hosted-world work**, not here.

## 4.6 Package managers and Linux formats — deferred

| Channel | Finding (2026-10-10) | Recommendation |
| --- | --- | --- |
| Flathub | Requires building from source inside `flatpak-builder` with vendored, offline sources (Cargo and the Godot export with templates), plus a review; heavy for a pre-alpha that changes weekly (**verify** current submission rules) | defer to a stable release |
| AppImage | One file for Linux, cheap with `appimagetool`; but it needs FUSE 2, which recent Ubuntu does not install by default (**verify**), and it is a second Linux artefact to test | defer; revisit if players ask (QR-7) |
| Homebrew cask (official tap) | Homebrew deprecated casks that fail Gatekeeper and set their disabling for 2026-09-01 (Homebrew 5.0.0 notes via Workbrew; CopyQ issue showing the warning); an unsigned MineWorld cannot enter the official tap | no; an own tap (`yuema137/homebrew-mineworld`) is possible only after notarization |
| winget | A manifest PR to `microsoft/winget-pkgs`; portable zips are supported; unsigned installers are scanned and may be flagged (**verify**) | defer |
| Scoop | An own bucket is a JSON file in a repository; the `extras` bucket wants notability (**verify**) | defer; cheapest of the three when wanted |

---

# 5. Building in CI on a tag

## 5.1 Workflows

- **`.github/workflows/release.yml`** (R-b). Triggers: `push: tags: ['v*']` and `workflow_dispatch` (a dry
  run on any ref: builds, smoke-tests and uploads workflow artifacts; it creates a release only with the
  input `draft: true`, and then only a draft named `test-<run>`, never a published one). It is a new
  file, so `ci.yml`'s `fast`/`test` names and triggers are untouched (ARC-48's "check names are an
  interface").
- **`.github/workflows/publish.yml`** (R-d). Trigger: `release: types: [published]`. Downloads the release's
  assets, verifies `SHA256SUMS` and each attestation, then runs `butler push`. Its job runs in the GitHub
  Environment `itch`, which holds `BUTLER_API_KEY` and requires the operator's approval.
- Neither workflow runs on `pull_request`; neither uses `pull_request_target`; no secret is reachable from a
  PR.

## 5.2 Jobs of `release.yml`

```text
verify-tag    ubuntu-24.04     tag == "v" + workspace version (Cargo.toml) [+ "-rc.N" allowed];
                               the tagged commit is an ancestor of origin/main;
                               the commit's required checks (fast, test) are green (gh api)
server        matrix           cargo build --release --locked -p mineworld-cli
                 macos-26        aarch64-apple-darwin + x86_64-apple-darwin, joined with lipo (universal)
                 windows-2025    x86_64-pc-windows-msvc
                 ubuntu-24.04    linux x86_64 at the floor QR-6 picks
                 ubuntu-24.04-arm linux arm64 at the same floor (QR-2)
launcher      same matrix      cargo build --release --locked -p mineworld-launch (R-c)
notices       ubuntu-24.04     cargo-about → THIRD_PARTY_RUST.html (DEP-38); Godot LICENSE/COPYRIGHT
                               fetched from the 4.7.2-stable tag, SHA-256 pinned
export        ubuntu-24.04     Godot 4.7.2 Linux editor + export templates, both SHA-512-checked
                               against SHA512-SUMS.txt and cached (actions/cache);
                               godot --headless --import, then --export-pack for 2D and 3D, and the
                               engine runtime per target (§3.2); checks the size budget (§7)
assemble      ubuntu-24.04     per target: layout of §3.1, archive with fixed mtimes and sorted
                               entries, SHA256SUMS
smoke         matrix (native)  §9's smoke test on macos-26, windows-2025, ubuntu-24.04, ubuntu-24.04-arm;
                               also macos-15 for an older macOS (QR-2)
draft         ubuntu-24.04     needs every smoke leg green; attest provenance and SBOM; gh release
                               create --draft with notes from the template (§10.3) and every asset
```

- **Why export on Linux for every target.** Godot exports Windows and Linux from Linux; macOS from Linux as
  a `.zip` with the built-in ad-hoc signature (Godot "Exporting for macOS", read 2026-10-10: built-in
  signing "will use an ad-hoc signature"; ".app bundles exported from Windows lack the executable flag",
  which does not apply to Linux). Exporting once also avoids R-SET-10: on a Linux runner the shared-module
  symlinks resolve (whether Godot's exporter follows directory symlinks into the pack is **verify** in R-a;
  fallback: the export job copies the two modules into place in its scratch checkout, never in the
  repository).
- **Pins.** Every action by commit SHA (`check_ci_pins.py`); runner labels never `-latest` (DEP-19); Godot
  and `butler` downloads by version and checksum; Rust from `rust-toolchain.toml`.
- **Minutes.** Public repository: standard hosted runners are free (ARC-48). The template archive is 1.28
  GB; cached once per Godot version (the Actions cache limit per repository is 10 GB, **verify** current
  value). A release run is estimated at 40–60 minutes wall time, dominated by the macOS universal build and
  the Windows smoke leg (**measure** in R-b).

## 5.3 Reproducibility

- **Promise: rebuildable, not bit-for-bit reproducible.** Same tag, same pinned toolchain, same templates,
  `--locked`. Archives are written with sorted entries, a fixed mtime (`SOURCE_DATE_EPOCH` = the tagged
  commit's time) and normalized permissions. Rust builds use `--remap-path-prefix` so no runner path is
  embedded.
- **Measured, not claimed.** R-b runs `workflow_dispatch` twice on one commit and compares per-file SHA-256
  of each bundle. Files that differ (expected candidates: Godot `.pck` ordering or UIDs, MSVC link
  timestamps) are listed in the PR's evidence as known non-reproducible; bit reproducibility is not a goal of
  this step.

## 5.4 Checksums and provenance

- `SHA256SUMS` lists every asset of the release; it is itself an asset. `PLAY.txt` and the release notes say
  how to check it on each OS (`shasum -a 256 -c`, `sha256sum -c`, `Get-FileHash`).
- **GitHub artifact attestations** (`actions/attest@v4`; permissions `id-token: write`, `contents: read`,
  `attestations: write`; GitHub docs read 2026-10-10). Public repositories use the Sigstore Public Good
  Instance. Availability for public repositories on the Free plan is believed but not stated on the pages
  read (**verify** in R-b by a dispatch run). A player or reviewer verifies with
  `gh attestation verify <file> -R yuema137/MineWorld`.
- **SBOM, because it is cheap.** `anchore/sbom-action` (syft) writes an SPDX JSON SBOM of the source tree at
  the tag (Cargo.lock, uv.lock); `actions/attest` with `sbom-path` binds it to each bundle (DEP-40). Its known
  gap: Godot's bundled libraries are not in `Cargo.lock`; they are listed in `LICENSES/godot/COPYRIGHT.txt`.
  An SBOM that omits them is labelled as covering the Rust and Python graphs only.

---

# 6. The launcher

## 6.1 What it is, and what it is not (ARC-78)

The launcher is a **process supervisor for one machine**: find a free port, start the bundled server on a
world with a per-user save, read the join line, start the Godot runtime with the right pack and join
arguments, wait for the client to exit, stop the server gracefully, and show an error if any step fails. It
is the existing `mineworld-2d` hosting logic (`mineworld-2d:96–160`) in a portable, windowless form.

It is **not** the "launcher, world editor GUI" that `MVP.md` §7 and `ARC-6` keep out of MVP-0: it has no
window of its own, no world browser, no settings, no accounts, and no world rule. It never reads world state;
it never parses anything from the server but the join line, which `server/PROTOCOL.md` §4.1 already makes
an interface. This step resolves `ARC-6`'s open question: **players get exported clients started by a
supervisor; contributors keep the bash launchers over an installed Godot.**

## 6.2 Comparison

| Option | No terminal on every OS | Testable headless in `cargo test` | Keeps clients pure protocol clients | Verdict |
| --- | --- | --- | --- | --- |
| (a) Shell scripts per OS (`.command`, `.bat`/`.ps1`, `.sh`) | no: `.command` opens Terminal; `.bat` opens a console | partly | yes | decline |
| (b) The Godot client spawns the server itself (`OS.create_process`) when given no `--server` | yes (**verify** console suppression on Windows) | no (needs Godot) | **no**: hosting logic and the server's path enter both clients, twice in GDScript | decline |
| (c) **A small Rust binary, `mineworld-launch`**, GUI subsystem on Windows, inside an `.app` on macOS, plain ELF plus a `.desktop` file on Linux | yes | yes, with a stub client process | yes | **adopt** |
| (d) An existing launcher framework (Tauri, Electron shell, Godot launcher addons) | yes | partly | yes | decline: a window toolkit to start two processes is a dependency larger than the problem (`REUSE_POLICY.md` §12) |

## 6.3 Shape of (c)

- Crate `tools/launch` (`mineworld-launch`), `std` only plus `windows-sys` on Windows (already in
  `Cargo.lock`, version 0.61.2) for `MessageBoxW` and `CREATE_NO_WINDOW`. No new dependency on macOS or Linux.
- Two entry points per bundle, "MineWorld 2D" and "MineWorld 3D": the same crate's two `[[bin]]` targets with
  one-line `main`s, so no argv[0] trickery. Defaults: 2D on `market-town` as `carol`; 3D on `social-cafe` as
  `visitor`, matching the root launchers. A text file `runtime/launch.toml` may override world and seat;
  editing it is documented in `PLAY.txt` (QR-10 asks whether a "both clients on one server" entry ships,
  the `AC-15` showcase).
- macOS: each entry point is a minimal `.app` (`Info.plist` plus the launcher in `Contents/MacOS`), ad-hoc
  signed in the assemble job; the shared `runtime/` sits beside them.
- Linux: the executables plus `MineWorld-2D.desktop`/`MineWorld-3D.desktop` (`Terminal=false`) that a user may
  copy to `~/.local/share/applications/`.
- Errors: macOS `osascript -e 'display alert …'`; Windows `MessageBoxW`; Linux `zenity` or `kdialog` when
  present, otherwise only the log. Every error names the log file.

## 6.4 Stopping the server (F-R7)

- New server option, owned by the S11 lane: `mineworld server … --stop-on-stdin-eof`. The server treats EOF
  on standard input exactly like Ctrl-C (the same graceful path, `serve.rs:244–253`). The launcher holds the
  pipe; when the client exits it closes it and waits up to 10 s, then kills only its own child.
- This also prevents orphaned servers when the launcher itself is killed, on all three OSes, without Job
  Objects or `PR_SET_PDEATHSIG`. It changes no world state and no protocol (ARC-44's admin surface is not
  needed).
- Test (R-c, in the default suite, all three OSes): a launcher run with a stub client that exits; the server's
  shutdown lines appear; no `mineworld` child remains; a second run resumes the save.

## 6.5 Exported-client changes (R-a)

- `--root=<absolute dir>` user argument for both clients; the launcher passes the bundle's `runtime/`.
  `repo_path()` (F-R1) resolves against it when given, and falls back to today's behaviour otherwise, so the
  bash launchers and every harness keep working unchanged.
- `text.gd` reads its own catalogs through `res://` paths (`DirAccess`/`FileAccess` on `res://` work inside a
  pack), and the export filter includes `*.po` (F-R2). The pack and user layers keep absolute paths.
- No rule, no protocol change, no new setting. `check_client_rules.py` keeps passing unchanged.

---

# 7. Size budget

## 7.1 Budget (fixed before measuring)

| Target | Hard limit (compressed archive) | Goal |
| --- | --- | --- |
| each of the four bundles | 400 MB | ≤ 250 MB |
| `3d.pck` | 250 MB | ≤ 150 MB |
| `2d.pck` plus the 2D pack files | 40 MB | ≤ 20 MB |

Reasons: well inside GitHub's 2 GiB per file and itch.io's limits; a 250 MB download is ordinary for a demo;
the engine runtime alone is roughly 80–190 MB uncompressed per target (**measure** in R-a). The `export` job
fails if a hard limit is exceeded and reports the top 20 files of each pack (a release that grows silently is
how budgets die).

## 7.2 Keeping the 3D client inside it

1. **Export filters, not scene selection** (F-R3): export all resources, excluding `shots/*`,
   `screenshots/*`, `tools/*`, `*.md`, and the promenade-only scenes once R-a's probe shows the slice does not
   reach them.
2. **Texture import limits.** VRAM-compressed (BPTC/S3TC for desktop) with mipmaps, and a per-texture size
   limit of 2048 px (1024 px for props) set in each `.import`'s `process/size_limit`; the HDRI stays 2k. Apple
   Silicon reads BC formats (**verify** that the macOS universal preset needs no ETC2/ASTC copy).
3. **Character meshes.** `meshy_d.glb` (10 MB) and the Vitruvian set (≈ 7 MB with hair strands) are the
   largest runtime files; whichever the slice does not use in the release build is filtered out.
4. **Measure before optimizing further.** Only if the budget fails: mesh compression (`meshoptimizer` in glTF
   import), lower-resolution LODs, or Basis Universal textures — each a visual change that the RL-b ruling
   precedent says goes to the operator.
5. **A budget regression is a red `export` job**, not a warning.

---

# 8. Code signing and notarization — the operator's choice

## 8.1 Options and costs

| Platform | Option | Cost (2026-10-10) | Player experience | Notes |
| --- | --- | --- | --- | --- |
| macOS | **Unsigned, ad-hoc signed** (default) | 0 | Gatekeeper blocks the first open. Since macOS 15 the Control-click override is gone; the player opens System Settings → Privacy & Security → "Open Anyway" (Apple announcement as reported by Macworld and others, 2024; one report says 15.1 removed even that for some apps, **verify** on macOS 26), or runs `xattr -dr com.apple.quarantine <folder>` | Godot: an ad-hoc signature "will make running an exported app easier for the end users" |
| macOS | Developer ID signing plus notarization | Apple Developer Program, US$99 per year (historical price; **verify** on developer.apple.com/programs) | opens normally | needs a Developer ID certificate; `rcodesign` signs and notarizes from Linux (unnumbered; deferred with QR-3), or a macOS runner with `codesign` and `notarytool`; secrets: a `.p12` and an App Store Connect API key, in an Environment |
| Windows | **Unsigned** (default) | 0 | SmartScreen: "Windows protected your PC" → More info → Run anyway; Mark-of-the-Web follows files out of a downloaded zip | |
| Windows | Azure Artifact Signing (formerly Trusted Signing), Basic | about US$9.99 per month (third-party sources; the official page showed no price, **verify**) | publisher name shown; SmartScreen reputation still builds over time (**verify**) | individuals: USA and Canada only (Microsoft Learn summary via search, **verify**); identity validation required |
| Windows | SignPath Foundation (free for open source) | 0 | as above, certificate in SignPath Foundation's name | conditions (signpath.org/terms, draft, read 2026-10-10): OSI licence "for all components", no proprietary components, every release built by CI from the repository, "every release needs manual approval", a code-signing policy on the project page, MFA for the team. Whether AI-generated assets distributed under MIT satisfy "no proprietary components" is a question for SignPath (**verify**) |
| Windows | An OV or EV certificate from a CA | roughly US$200–600 per year plus an HSM or cloud key (**verify**) | as above | heavier operations for one maintainer |
| Linux | none needed | 0 | the archive runs after `tar -xzf`; checksums and attestations are the integrity story | |

## 8.2 Recommendation (ruled 2026-10-10: unsigned for the pre-alpha on all three, QR-3 and QR-4)

- **Pre-alpha: unsigned on all three**, with the per-OS steps in `PLAY.txt`, on the itch page and in the
  release notes, plus `SHA256SUMS` and attestations for anyone who wants to check. This costs nothing and
  blocks nothing.
- **When the demo is shown to non-technical players:** macOS notarization first (the macOS path is the
  harshest without it, and Homebrew needs it), then Windows through SignPath Foundation if accepted, else
  Azure Artifact Signing if the operator is eligible.
- The workflows are written so signing is a later, additive job behind an Environment; nothing in R-a … R-d
  depends on the choice.

---

# 9. Smoke test of each bundle

## 9.1 What every leg does (on the bundle's own OS)

1. Unpack the archive as a player would (zip/tar), into a path with a space and a non-ASCII character
   (`MineWorld 世界/`), because players' folders have both.
2. Check `SHA256SUMS` and the bundle's internal file list (`BUNDLE.toml`).
3. **Server alone:** run `runtime/mineworld run` on each world for a fixed number of simulated days and compare
   its digest with the AC-8 record of the same commit (the release binary must simulate the same world as CI's
   debug-profile parity runs; `ARC-49`). A different digest is FAIL.
4. **Launcher plus 2D client, headless:** the launcher in smoke mode (`--smoke=2d`) starts the server on a
   scratch save, starts the runtime with `2d.pck --headless -- --drive=<scenario> --settings=none`, and
   exits with the client's code (the 2D drive exits non-zero on failure, `mineworld-2d:16`).
5. **Launcher plus 3D client, headless:** `--smoke=3d` runs the slice's `--slice-link` probe; the verdict
   line is required (F-R6) — missing is FAIL.
6. **Clean exit:** after each run, the launcher's log shows the server's shutdown lines, no `mineworld`
   process started by the run remains, and the per-user scratch folder is removed (`check_scratch.py`'s
   rule, `DEP-29`).
7. **Windowed frame (macOS leg only, best effort):** one rendered frame of each client saved as PNG and
   uploaded as evidence (`D-11`'s recipe); a black or missing frame is INCONCLUSIVE and reported, not a
   release blocker, because hosted runners' GPUs vary (**verify** Metal availability on `macos-26` runners).

`--headless` in a release template is **verify** in R-a (Godot documents it for CI use; the release template
should honour it). If it does not, legs 4–5 run windowed on macOS and under `xvfb-run` on Linux, and Windows
is reported INCONCLUSIVE with an owner.

## 9.2 Local equivalent

`python3 scripts/package.py smoke <archive>` runs the same steps on the developer's machine (R-c), so a
release candidate can be checked by hand on hardware CI does not have (the operator's Milestone F machines).

---

# 10. Versioning, changelog and release notes

## 10.1 Versions

- **Framework 0.1.0** (Milestone E). The first public demo is tag `v0.1.0`; release candidates are
  `v0.1.0-rc.N`, marked "pre-release" on GitHub and pushed to hidden itch channels (QR-9).
- **One version.** The tag must equal the workspace version in `Cargo.toml`; `verify-tag` fails otherwise.
  World packs keep their own pack versions (`ARC-53`); the bundle records both.
- **The client reports it.** The runtime's `BUNDLE.toml` holds the version and commit; the clients' settings
  menu may show it later (S20's lane); not required here.
- SemVer meaning while 0.x: a minor bump may break the protocol or saves; `PLAY.txt` says saves are not
  guaranteed to load across 0.x versions.

## 10.2 Changelog

- `CHANGELOG.md` at the root, "Keep a Changelog" sections (Added, Changed, Fixed, Known issues) per version,
  written by hand from the merged PRs at release time. Under `CLAUDE.md` §2.1 it is a non-README Markdown file
  and therefore held to the specification standard: complete and exact, every entry citing its PR. QR-11
  confirms this rather than a `docs/` location.

## 10.3 Release notes template (`.github/release-notes.md`, filled by the `draft` job)

```text
MineWorld {version} — downloadable demo (pre-alpha)

What this is        one paragraph: a framework demo; 2D and 3D clients on one local server; no AI model.
Download            one line per target, with size
First open          macOS / Windows / Linux unsigned-app steps (same text as PLAY.txt)
Verify (optional)   SHA256SUMS; gh attestation verify <file> -R yuema137/MineWorld
What's new          from CHANGELOG.md, this version's section
Known issues        from CHANGELOG.md; always includes "saves may not load across 0.x"
Licences            MIT; exceptions and asset records in NOTICE; AI-generated assets disclosed
Built from          commit {sha}, workflow run {url}
```

---

# 11. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-R1 | The exported clients behave differently from editor runs (paths, missing non-resource files, string-loaded resources) | F-R1–F-R4 fixed in R-a; the smoke test runs the *exported* clients' drives; R-a's acceptance runs them locally on macOS before CI exists |
| R-R2 | macOS players cannot open an unsigned app at all on a future macOS | QR-3; `xattr` instruction as fallback; notarization path ready as an additive job |
| R-R3 | The bundle exceeds its budget as Milestone F and S22 add 3D assets | hard limit in the `export` job; the first tag waits for visuals to settle |
| R-R4 | A secret leaks through a workflow | the only secret lives in an Environment with required reviewer, used only by `publish.yml` on `release: published`; `release.yml` needs none |
| R-R5 | Long play fills a player's disk (`F-SAVE-1`) | the save folder is named in `PLAY.txt`; persistence lane owns retention |
| R-R6 | Release binary simulates differently from CI (release profile, musl, LTO) | smoke step 3 compares digests with the AC-8 record of the same commit |
| R-R7 | Antivirus false positives on unsigned Godot executables on Windows (**verify**; commonly reported for unsigned game binaries) | signed builds (QR-4) are the cure; until then the release notes link the checksums and attestation |
| R-R8 | Publishing duplicates the source of truth (itch shows a build GitHub does not have) | `publish.yml` only pushes assets downloaded from the published GitHub release after checking `SHA256SUMS` |

---

# 12. Decision records (numbers assigned by the primary session, 2026-10-10)

Each record is written into `docs/DECISIONS.md` by the PR named in the last column, in the form
`REUSE_POLICY.md` §11 requires. The draft's placeholder is kept in brackets for traceability.

| Number | Decision | Alternatives compared | Written by |
| --- | --- | --- | --- |
| **ARC-78** [ARC-R1] | Players receive exported clients started by a windowless supervisor (`mineworld-launch`); contributors keep the bash launchers. Resolves `ARC-6`'s open question; does not contradict `MVP.md` §7 (no launcher GUI) | §6.2 (a)–(d) | R-c |
| **ARC-79** [ARC-R2] | Release channels: GitHub Releases canonical (draft by CI, published by the operator), itch.io mirror from the published release only, Zenodo archive; tag equals workspace version | §4 | R-b |
| **ARC-80** [ARC-R3] | The bundle layout, per-user data folder for saves and logs, and the `--root` user argument for exported clients | §3, §6.5 | R-a |
| **DEP-38** [DEP-R1] | `cargo-about` (Embark Studios, MIT/Apache-2.0) generates the Rust third-party notices required by MIT/Apache-2.0 binary redistribution | `cargo-license` (lists names, no licence texts — insufficient); `cargo-bundle-licenses` (less used, **verify** maintenance); hand-written file (drifts from `Cargo.lock`, declined); `cargo-deny` already checks licences (DEP-22) but does not emit notices | R-a |
| **DEP-39** [DEP-R2] | `butler` (itch.io official, MIT), pinned by version and checksum, run directly | the itch.io web uploader (manual, no CI); third-party GitHub Actions wrapping butler (an extra supplier for a five-line step, declined) | R-d |
| **DEP-40** [DEP-R3] | SBOM: `anchore/sbom-action` (syft) SPDX JSON, attested with `actions/attest` | `cargo-cyclonedx` (Rust graph only); `cargo-auditable` (embeds the dependency list in the binary — complementary, cheap, worth a later look); none (acceptable, but the cost is one step) | R-b |
| **DEP-41** [DEP-R4] | Official Godot 4.7.2 export templates from the `godotengine/godot` release, SHA-512 checked against `SHA512-SUMS.txt`, cached | `barichello/godot-ci` (already declined in `step-14-ci.md` §5.7); building templates from source (smaller binaries via feature stripping, but hours of CI and a toolchain to own — declined until the size budget demands it) | R-a |
| **DEP-42** [DEP-R5] | Provenance: `actions/attest@v4` (GitHub artifact attestations, Sigstore) | `slsa-framework/slsa-github-generator` (SLSA L3 reusable workflows, heavier, verifies with `slsa-verifier`); no provenance (declined: public binaries from one maintainer need a checkable origin) | R-b |
| — [DEP-R6] | *Not assigned: notarization is deferred (QR-3).* If it is chosen later: `rcodesign` (apple-codesign, MPL-2.0) to sign and notarize from Linux | Xcode `codesign` plus `notarytool` on a macOS runner (equally fine; choose at that PR) | a later signing PR, with a number from the primary session |

The Rust launcher adds no dependency beyond `windows-sys` (already present), so it needs no DEP record.

---

# 13. PR split, dependencies and checkpoints

## 13.1 Order

```text
R-a  export presets, client path fixes, local packaging script     ─┐
R-c  launcher, server stop-on-EOF (S11), smoke tests               ─┼─► R-b  CI release workflow ─► R-d publishing
(R-a and R-c in parallel, each in its own worktree)                 ─┘
first public tag: after Milestone F's hands-on acceptance and after the 3D visuals settle (16c, 12d, S22 quick wins)
```

None of the four blocks an MVP-0 acceptance criterion; MVP-0's acceptance does not wait for any of them.

## 13.2 R-a — export presets and a local packaging script

- **Files:** `clients/2d/export_presets.cfg`, `clients/3d-spike/export_presets.cfg` (no credentials in them;
  signing fields empty), the feature-tag main-scene override in `clients/3d-spike/project.godot` (F-R4),
  `clients/2d/scripts/app.gd` (`--root`, F-R1), `clients/shared/settings/text.gd` (F-R2, with S20's review),
  `scripts/package.py` (stdlib Python like the other `scripts/*.py`: `build`, `export`, `assemble`,
  `budget`), `PLAY.txt` source under `packaging/`, specs updated first (`clients/2d/README.md` is human; the
  client spec sections that describe arguments).
- **Checkpoint:** on the operator's Mac, `python3 scripts/package.py assemble --target macos-universal`
  produces a bundle whose exported 2D client drives a fresh `market-town` and whose 3D client completes the
  slice link probe, both from the bundle with no Godot on `PATH`; the four probes of §2.2 (globalize, scene
  override, `--main-pack`, `--headless` in a release template) are recorded with output.
- **Adversarial:** A-R1, A-R3, A-R4.

## 13.3 R-c — the launcher and the smoke tests

- **Files:** `tools/launch/` (new crate, two binaries), `Cargo.toml` workspace member, server option
  `--stop-on-stdin-eof` in `tools/cli/src/serve.rs` (S11 lane reviews; protocol unchanged),
  `scripts/package.py smoke`, tests in `tools/launch/tests/` using a stub client executable.
- **Checkpoint:** in the default suite on Linux, macOS and Windows (`test`, `test-macos`, `test-windows`):
  a launcher run hosts a world, starts the stub client with the expected arguments, stops the server
  gracefully when the stub exits, leaves no process and no scratch, and a second run resumes the save.
- **Adversarial:** A-R5, A-R6.

## 13.4 R-b — the CI release workflow

- **Files:** `.github/workflows/release.yml`, `scripts/ci_release.py` if the steps grow past a few lines
  (layers stay in `ci_layer.py`), `.github/release-notes.md`, an ARC-48 note.
- **Checkpoint:** a `workflow_dispatch` dry run on the PR branch produces four bundles, all smoke legs PASS,
  attestations verify with `gh attestation verify`, `SHA256SUMS` checks; a dispatch with the input
  `draft: true` creates a draft release named `test-<run>` (deleted afterwards) and nothing else — no
  publication, no itch push, no Zenodo record.
- **Adversarial:** A-R2, A-R7, A-R8.

## 13.5 R-d — publishing

- **Files:** `.github/workflows/publish.yml`, `CITATION.cff`, `.zenodo.json` (if needed after the precedence
  check), `CHANGELOG.md`, README download section (human, short).
- **Operator actions (outside the PR):** create the itch.io page and API key, create the `itch` Environment
  with the secret and a required reviewer, enable the repository in Zenodo, fill itch.io's AI disclosure.
- **Checkpoint:** publishing a release candidate draft pushes to hidden itch channels after approval, the
  itch app installs and opens it on one OS, Zenodo shows a record with a DOI (or QR-9 rules candidates out).
- **Adversarial:** A-R9, A-R10.

---

# 14. Adversarial criteria (fixed before measuring)

| ID | Criterion | How it bites |
| --- | --- | --- |
| A-R1 | The exported 2D client started from a bundle folder that is *not* inside a repository checkout loads its Presentation Pack and catalogs | move the bundle to `/tmp/x y/`; with F-R1 unfixed the pack is "plain drawing" and the run fails by name |
| A-R2 | A tag that differs from the workspace version produces no release | push `v9.9.9` on a scratch tag in R-b's evidence: `verify-tag` red, no draft |
| A-R3 | No bundle contains a GPL file, an evidence folder, a `.md` file or a secret pattern | the assemble job lists files and fails on `tools/blender`, `shots/`, `references/`, `candidates/`, `*.md`, `.env`; a planted `shots/x.png` must fail it |
| A-R4 | Every bundle is inside its hard size limit | plant a 300 MB file in the 3D export in a scratch run: `export` red |
| A-R5 | Closing the client stops the server gracefully and leaves no process | stub client exits; with `--stop-on-stdin-eof` removed the test fails by name |
| A-R6 | Killing the launcher does not orphan the server | kill the launcher in the test; the server exits on EOF within 10 s |
| A-R7 | A failed smoke leg on any OS prevents the draft | plant a failing drive scenario on a scratch dispatch: `draft` skipped, no release |
| A-R8 | The release server simulates the same world as CI | smoke step 3 against the AC-8 record; a planted one-line change to a world in the bundle must turn it red |
| A-R9 | itch.io receives only bytes that GitHub published | `publish.yml` verifies `SHA256SUMS` and attestations before `butler push`; a modified asset fails it |
| A-R10 | No workflow reachable from a pull request can read `BUTLER_API_KEY` | `publish.yml` triggers only on `release: published` and runs in the `itch` Environment; R-d's evidence shows the Environment's protection rule |

---

# 15. Operator questions (QR-n), ruled 2026-10-10

**[OPERATOR]** marks the ones only the operator could decide. Rulings were relayed by the primary session on
2026-10-10: the operator ruled QR-1, QR-3, QR-4, QR-8, QR-12 and QR-13 explicitly; "all other QRs as you
recommended", with QR-5 an operator action when R-d is ready.

| ID | Question | Recommendation | Ruling (2026-10-10) |
| --- | --- | --- | --- |
| **QR-1 [OPERATOR]** | Adopt the channel set: GitHub Releases canonical, itch.io page, Zenodo archive; Hugging Face declined; package managers deferred? | **Yes** (§4) | **Yes, all three channels**: GitHub Releases as a draft that the operator publishes; itch.io at price 0 with the AI disclosure; Zenodo DOI |
| **QR-2** | Targets: macOS universal, Windows x86_64, Linux x86_64 — and Linux arm64? Smoke on macOS 15 as the older macOS? | **Yes to all four targets**: Linux arm64 costs one matrix leg (the runner already exists for AC-8) and one Godot template already in the archive; **yes** to a `macos-15` smoke leg | as recommended |
| **QR-3 [OPERATOR]** | macOS: ship unsigned (ad-hoc) or join the Apple Developer Program (about US$99/year, **verify**) and notarize? | **Unsigned for the pre-alpha**; notarize before showing it to non-technical players | **Unsigned for the pre-alpha**, with clear "Open Anyway" instructions |
| **QR-4 [OPERATOR]** | Windows: unsigned, SignPath Foundation (free, conditions §8.1), or Azure Artifact Signing (about US$9.99/month, individuals in US/Canada, **verify**)? | **Unsigned for the pre-alpha**; apply to SignPath Foundation when signing is wanted | **Unsigned for the pre-alpha**, with clear "Run anyway" instructions |
| **QR-5 [OPERATOR]** | Create the itch.io project, an API key, and a GitHub Environment `itch` holding `BUTLER_API_KEY` with yourself as required reviewer; and enable the repository in Zenodo? | **Yes**, when R-d is ready; nobody but the operator handles the key | an operator action, when R-d is ready |
| **QR-6** | Linux binary floor: (a) static `musl` (runs on any distribution; a different allocator and libc maths), (b) native build on an `ubuntu-22.04` runner (glibc 2.35; runner retirement date **verify**), (c) `cargo-zigbuild` to a chosen glibc | **(a) musl static**, guarded by smoke step 3's digest comparison; fall back to (b) if the digest differs | as recommended |
| **QR-7** | AppImage in addition to `.tar.gz`? | **Not now** | as recommended |
| **QR-8 [OPERATOR]** | Keep the demo free of language models (rule controller only)? | **Yes**: no key, no network, and it proves the "models removed" rule; a cognition add-on can come with S10 | **Yes**: the demo ships without language models |
| **QR-9** | Release candidates: GitHub pre-release + hidden itch channel; archive them on Zenodo too? | **No Zenodo for candidates** (every published release mints a DOI); if Zenodo archives pre-releases automatically, candidates stay drafts and are tested from workflow artifacts | as recommended |
| **QR-10** | Ship a third entry point "MineWorld 2D + 3D" (both clients on one server, the `AC-15` showcase)? | **Yes**, it is the clearest demonstration of the framework; it costs one more `[[bin]]` | as recommended (R-c builds it) |
| **QR-11** | `CHANGELOG.md` at the root, held to the specification standard, entries citing PRs? | **Yes** | as recommended |
| **QR-12 [OPERATOR]** | First public tag timing: after Milestone F and the 3D visuals settle, not blocking MVP-0? | **Yes** | **Yes**: the first tag comes after Milestone F and after the visuals settle; it does not block MVP-0 |
| **QR-13 [OPERATOR]** | itch.io page: price 0 with optional donation, AI disclosure ticked, platforms only as smoke-tested? | **Yes** | **Yes** |

**Related ruling, the same day (QB-11, `overall.md` §5 "Still open").** `test-windows` and `test-macos` stay
non-required checks until the release. R-a and R-c still run their tests on all three OSes (§17); a red
`test-windows` or `test-macos` leg on their PRs is a finding the session resolves or reports, never ignores.

---

# 16. Edits outside this file

## 16.1 `overall.md` (the primary session)

- §3: add "S23 — Downloadable public demo" after S15, linking this file, with "not an MVP-0 gate".
- §5 "Still open": QB-14 → "planned and frozen in `step-23-release.md`; ruled by QR-1, QR-3, QR-4"; QB-11 →
  "non-required until the release (operator, 2026-10-10)".
- §5 decision-number table: S23 — ARC-78, ARC-79, ARC-80; DEP-38 … DEP-42 (already assigned by the primary
  session).

## 16.2 `docs/DECISIONS.md` (the implementing PRs, never this document)

- The records of §12, each written by the PR in its "Written by" column: ARC-80, DEP-38 and DEP-41 by R-a;
  ARC-78 by R-c; ARC-79, DEP-40 and DEP-42 by R-b; DEP-39 by R-d.
- An `ARC-6` note, written by R-c with ARC-78: its open question is resolved by ARC-78.
- An ARC-48 note, written by R-b: `release.yml` and `publish.yml` exist, need no PR check names, and hold the
  only secret.

## 16.3 `docs/MVP.md` §7

- No change for MVP-0. A sentence under the artefact table: "After MVP-0, the same artefacts are published as
  per-OS bundles (S23); the bundle's supervisor is not the launcher GUI excluded above."

## 16.4 `step-14-ci.md`

- §13.0.2 and W-11: the player path for Windows symlinks is S23's bundles (export on Linux); QB-14 points here.
- §5.7: S23's `export` job downloads the same official Godot build and templates; 13c and R-b share one pinned
  download step (one owner: whichever lands first; the other reuses it).

---

# 17. Execution contracts for R-a and R-c (frozen 2026-10-10, primary session)

These are the filled project contracts of `.claude/skills/structured-coding/prompts/implementation-working-rules.md`
for the two PRs that may start now. Each PR's commit plan is still to be written: the implementing session's
first commit (Markdown only) adds it as §18 (R-a) or §19 (R-c) of this file, re-verifying §2's audit anchors
against its base, and the primary session freezes that commit plan before code begins (`overall.md` §5
ruling 9). A field marked N/A is not applicable.

## 17.1 R-a — export presets, exported-client path fixes, local packaging script

```text
PROJECT / PR:
  MineWorld mvp0 · S23 R-a — export presets, client path fixes, local packaging script

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/step-23-release.md — §§1–16 (frozen step design), §17.1 (this contract),
  §18 (R-a's commit plan and live ledger, written by the implementing session's first commit)

RELATED / BINDING DOCS:
  CLAUDE.md; docs/ENGINEERING_RULES.md; docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md;
  overall.md §5 (Every platform; Parallel build-out rulings 1, 4, 9); docs/DECISIONS.md ARC-6, DEP-4, DEP-8,
  DEP-29, DEP-36, ARC-46, ARC-70, ARC-76; clients/shared/SETTINGS.md; clients/2d/PRESENTATION.md;
  step-20-client-settings.md §3.6 (the text module's layers); step-14-ci.md §13.0.2 (symlinks)

IMPLEMENTATION BASE:
  branch mvp0/pr-s23-ra from origin/main at session start (at or after a926fbf plus this freeze's merge);
  worktree /Users/yuema137/mineworld-worktrees/impl-s23-ra, held by this one session only

APPROVED SCOPE:
  - clients/2d/export_presets.cfg and clients/3d-spike/export_presets.cfg (no credentials; signing empty;
    macOS ad-hoc only), include filters for non-resource files (*.po, the pack files the clients read) and
    exclude filters of §7.2 item 1
  - F-R4: the feature-tag main-scene override in clients/3d-spike/project.godot; if the probe shows it does not
    work in a 4.7.2 release template, the recorded fallback (a minimal boot scene that reads a user argument)
  - F-R1: the `--root=<absolute dir>` user argument in clients/2d (app.gd repo_path), falling back to today's
    behaviour when absent
  - F-R2: clients/shared/settings/text.gd reads its own catalogs through res:// (with S20's semantics kept:
    layer order, override rules, user layer unchanged)
  - scripts/package.py (stdlib Python): build (host target, --target accepted), export, assemble, budget,
    probe (server + exported 2D drive + exported 3D link probe, without a launcher), --self-test
  - packaging/: PLAY.txt source (English + Simplified Chinese), BUNDLE.toml template, cargo-about
    configuration and template (DEP-38)
  - docs/DECISIONS.md records ARC-80, DEP-38, DEP-41 (numbers already assigned); client spec sections that
    describe arguments (clients/2d/PRESENTATION.md or the client's argument spec), SETTINGS.md if text.gd's
    loading description changes
  - one line adding `python3 scripts/package.py --self-test` to scripts/ci_layer.py's `fast` list
    (reported to the S13 lane in the PR body)

FROZEN INVARIANTS:
  - §1.2's standing rules; no rule, protocol field, setting or world-state behaviour changes in any client
  - the root launchers (mineworld-2d, mineworld-3d, mineworld-slice) and every existing harness keep working
    unchanged from a repository checkout (`--root` absent = today's behaviour)
  - bundle content per §3.1; nothing of A-R3's exclusion list is ever in a bundle (tools/blender GPL scripts,
    shots/, screenshots/, references/, candidates/, *.md, .env)
  - size limits of §7.1 are hard; R-a filters files only and changes no texture import setting
  - no credentials or signing identities in any tracked file
  - the shared modules stay symlinks in Git (§13.0.2); no committed copies
  - check_client_rules.py and client_rules.rs pass unchanged

APPROVED IMPLEMENTATION SEQUENCE:
  C0  commit plan (§18), anchors re-verified — Markdown only; primary freezes it
  C1  specs first: argument spec for --root, SETTINGS.md if needed, ARC-80 / DEP-38 / DEP-41 records
  C2  the four engine probes of §13.2 on a scratch project outside the repository, recorded in §18 with output:
      globalize_path("res://") in a release template; feature-tag main_scene override; --main-pack;
      --headless in a release template
  C3  F-R1, F-R2 (client changes) with the existing 2D and settings suites green from the checkout
  C4  export presets and F-R4; exports of 2D and 3D succeed headless
  C5  scripts/package.py and packaging/; --self-test including a planted excluded file (A-R3) and a planted
      oversize file (A-R4)
  C6  gate: on macOS, assemble macos-universal; `package.py probe` passes on the bundle moved to a folder
      outside any checkout with a space and a non-ASCII character in its path (A-R1); budget report recorded;
      LICENSES/ complete

AUTONOMOUS VALIDATION BUDGET:
  - cargo build / test / clippy, Godot headless and windowed runs as the checks need: unrestricted
  - one download of Godot 4.7.2 export templates (1.28 GB) with `gh release download`, SHA-512 checked against
    SHA512-SUMS.txt, installed in the user's Godot templates folder (not in the repository)
  - `cargo install cargo-about --locked` at a pinned version, recorded in DEP-38
  - no long town runs; no network beyond those two downloads; no language model
  - anything materially beyond this: stop with a cost projection

REQUIRED LIVE DOCUMENTATION:
  step-23-release.md §18 (commit plan, ledger, evidence, findings)

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff-s23-ra.md

PR CONTEXT SCOPE / INITIALIZATION / AUTHORITY ORDER / CONTINUITY / COMPACTION / RESUME / CLOSEOUT:
  as in implementation-working-rules.md, unmodified. A fresh implementation session; no state carried from
  the planning session except this document.

ENDPOINT AUTHORITY:
  - implementation + local validation: authorized from the primary session's freeze of the C0 commit plan
      source: primary session, 2026-10-10 (this freeze) and overall.md §5 ruling 9
  - semantic commits: authorized, no approval before each commit
      source: overall.md "Autonomous execution authorization (operator, 2026-09-25)"
  - branch push: authorized          source: overall.md D-12
  - PR creation / update: authorized source: overall.md D-12
  - CI repair to review readiness: authorized   source: overall.md D-12
  - merge: explicit operator authorization only, after the primary session's review. No source changes this.

ALLOWED COMMANDS:
  cargo *; git; gh (pr create/view/checks, release download of godotengine/godot assets, api read-only; never
  merge); python3 scripts/*; godot *; mkdir -p; cp, mv, ln -s inside scratch folders and the worktree;
  sed -n; shasum; unzip; tar; the root launchers; binaries under target/
NOT ALLOWED:
  python3 -c, sed -i, awk, xargs, curl, heredoc file writes; reading any secret; file changes go through
  Read, Edit and Write

MATERIAL STOPS (stop, report evidence and the smallest revision to the primary session):
  1. an edit to clients/protocol/mineworld, the server, contracts, kernel, any System/World/Presentation pack
     content, or any .import file
  2. the size budget fails with file filtering alone
  3. a probe fails and its recorded fallback does not work either
  4. a new runtime dependency, or a tool other than cargo-about and the Godot templates
  5. any existing test, harness or launcher behaviour changes
  6. a licence question: a bundled file whose redistribution status is not already recorded in NOTICE or
     ASSETS.md

POST-MERGE SYNCHRONIZATION OWNER:
  this implementation session: §18, the handoff, the DECISIONS records, the closeout record;
  the primary session: §§1–17 and overall.md

NORMAL STOP CONDITION:
  S23 R-a READY FOR OPERATOR REVIEW — DO NOT MERGE

STOP CONDITION:
  implementation complete + C6 gate evidence + PR opened/updated + fast and test green on the exact final
  head (test-windows / test-macos read and reported; non-required under QB-11's ruling)

MERGE AUTHORITY:
  NEVER merge without explicit operator approval, relayed after the primary session's review.
```

## 17.2 R-c — the launcher, the server's stop-on-EOF, and the smoke tests

```text
PROJECT / PR:
  MineWorld mvp0 · S23 R-c — mineworld-launch, server --stop-on-stdin-eof, smoke tests

PRIMARY DESIGN DOC:
  .structured-coding/plans/mvp0/step-23-release.md — §§1–16, §17.2 (this contract), §19 (R-c's commit plan
  and live ledger, written by the implementing session's first commit)

RELATED / BINDING DOCS:
  CLAUDE.md; docs/ENGINEERING_RULES.md; docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; overall.md §5
  (Every platform); docs/DECISIONS.md ARC-6, ARC-44, DEP-11, DEP-29 and its 2026-10-09 note, ARC-48 notes;
  server/PROTOCOL.md §4.1 (the join line); clients/shared/SETTINGS.md §4 (per-user folder);
  step-14-ci.md §15 (13w, the Windows and macOS default suite)

IMPLEMENTATION BASE:
  branch mvp0/pr-s23-rc from origin/main at session start; worktree
  /Users/yuema137/mineworld-worktrees/impl-s23-rc, held by this one session only. The packaging integration
  commit (C5 below) starts only after R-a has merged; the session merges origin/main first.

APPROVED SCOPE:
  - new crate tools/launch (mineworld-launch): std plus windows-sys (Windows only, the version already in
    Cargo.lock); binaries for "MineWorld 2D", "MineWorld 3D" and "MineWorld 2D + 3D" (QR-10); the launch.toml
    override file; per-user saves and logs; error dialogs per §6.3; --smoke=2d|3d|both and --no-dialog
  - Cargo.toml workspace member; systems/installed is not touched
  - server option `--stop-on-stdin-eof` in tools/cli/src/serve.rs (same graceful path as Ctrl-C), with its
    argument documented where the server's arguments are specified; reviewed with the S11 lane
  - tests: tools/launch/tests/ (stub client, scratch bundle, DEP-29 helpers), and a tools/cli test for the
    server option
  - after R-a merges: scripts/package.py gains the launcher in `assemble` (.app wrappers, .desktop files,
    Windows GUI subsystem binaries) and `smoke` (§9.1 steps 1–6 through the launcher)
  - docs/DECISIONS.md: ARC-78 and the ARC-6 note; tools/launch/README.md (human, short)

FROZEN INVARIANTS:
  - the launcher reads nothing from the server but the join line; it holds no world rule and no world state
  - it stops only processes it started; never anything found by name
  - saves and logs live in the per-user MineWorld folder (§3.3), never beside the executable
  - the server option changes no world state, no protocol and no existing behaviour when absent
  - the bash launchers stay as they are (contributors' path, ARC-78)
  - no language model, no network beyond loopback
  - default suite stays green and scratch-clean on Linux, macOS and Windows

APPROVED IMPLEMENTATION SEQUENCE:
  C0  commit plan (§19), anchors re-verified — Markdown only; primary freezes it
  C1  specs first: ARC-78, ARC-6 note, the server argument's specification
  C2  server --stop-on-stdin-eof with its test (EOF → shutdown lines, exit 0, save resumable)
  C3  tools/launch core: host, join-line read, spawn client, graceful stop, kill fallback after 10 s, logs;
      tests with a stub client (A-R5, A-R6)
  C4  per-OS presentation of errors and windowless start; the three entry points
  C5  (after R-a merges) package.py assemble + smoke integration; smoke on macOS against a real bundle
  C6  gate: default suite green on all three OSes in CI; on macOS, `package.py smoke` PASS for 2d, 3d and
      both on an assembled bundle; evidence in §19

AUTONOMOUS VALIDATION BUDGET:
  - cargo build / test / clippy, Godot headless runs, the CI default suite on three OSes: unrestricted
  - no long town runs; no language model; anything materially beyond: stop with a projection

REQUIRED LIVE DOCUMENTATION:
  step-23-release.md §19

CONTEXT HANDOFF / MEMORY FILE:
  .structured-coding/plans/mvp0/handoff-s23-rc.md

PR CONTEXT SCOPE / INITIALIZATION / AUTHORITY ORDER / CONTINUITY / COMPACTION / RESUME / CLOSEOUT:
  as in implementation-working-rules.md, unmodified. A fresh implementation session.

ENDPOINT AUTHORITY:
  - implementation + local validation: authorized from the primary session's freeze of the C0 commit plan
      source: primary session, 2026-10-10 (this freeze) and overall.md §5 ruling 9
  - semantic commits: authorized    source: overall.md "Autonomous execution authorization (2026-09-25)"
  - branch push: authorized         source: overall.md D-12
  - PR creation / update: authorized source: overall.md D-12
  - CI repair to review readiness: authorized   source: overall.md D-12
  - merge: explicit operator authorization only, after the primary session's review. No source changes this.

ALLOWED COMMANDS:
  cargo *; git; gh (pr create/view/checks, api read-only; never merge); python3 scripts/*; godot *;
  mkdir -p; cp, mv inside scratch folders and the worktree; sed -n; shasum; unzip; tar; codesign -s - (ad-hoc,
  scratch bundles only); the root launchers; binaries under target/
NOT ALLOWED:
  python3 -c, sed -i, awk, xargs, curl, heredoc file writes; reading any secret; file changes go through
  Read, Edit and Write

MATERIAL STOPS:
  1. any server change beyond --stop-on-stdin-eof, or any change to the protocol, contracts, kernel or packs
  2. a dependency beyond std and windows-sys (for example a dialog or GUI crate)
  3. a client change (the clients are R-a's)
  4. a graceful stop that cannot be made to work on one OS without a platform-specific server path
  5. any existing test or launcher behaviour changes
  6. R-a's packaging script needs a structural change rather than an addition

POST-MERGE SYNCHRONIZATION OWNER:
  this implementation session: §19, the handoff, the DECISIONS records, the closeout record;
  the primary session: §§1–17 and overall.md

NORMAL STOP CONDITION:
  S23 R-c READY FOR OPERATOR REVIEW — DO NOT MERGE

STOP CONDITION:
  implementation complete + C6 gate evidence + PR opened/updated + fast and test green on the exact final
  head, test-windows and test-macos green or every red leg explained (non-required under QB-11's ruling,
  but R-c's claims are per OS)

MERGE AUTHORITY:
  NEVER merge without explicit operator approval, relayed after the primary session's review.
```
