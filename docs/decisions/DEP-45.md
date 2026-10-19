## DEP-45 — Godot in CI: the official 4.7.2 builds, SHA-512 pinned in the repository; Xvfb and Mesa for windowed tests

**Date** 2026-10-10 · **Status** selected; integrated in S13 PR 13c · **Approved by** the primary session
(13c QC-5, QC-10, design freeze 2026-10-10) · **Relates to** `ARC-83`, `ARC-17`, step-23 §16.4 · **Design**
`.structured-coding/plans/mvp0/pr-13c-nightly.md` §§3.4, 5.3

**Problem.** The nightly's `clients` group needs Godot 4.7.2 — the version the operator runs
(`4.7.2.stable.official.ed1daf0bf`) — on Linux, macOS and Windows runners, fetched reproducibly and
without trusting whatever a release page serves on the night.

**Options considered.**

```text
(a) the official 4.7.2-stable builds from godotengine/godot-builds, downloaded   chosen
    by our script, checked against SHA-512 values pinned in the repository,
    unpacked and cached (MIT)
(b) chickensoft-games/setup-godot (third-party action, MIT)                      declined: holds the
                                                                                 download logic we must
                                                                                 pin anyway, with .NET
                                                                                 options we do not need
(c) barichello/godot-ci container (MIT)                                          declined: Linux only, a
                                                                                 third party's cadence
(d) Godot built from source                                                      declined: hours of CI
(e) Xvfb and Mesa (lavapipe) from the Ubuntu archive for the windowed tests      chosen for Linux
```

**Choice.** (a) and (e). `scripts/ci_godot.py fetch` downloads `Godot_v4.7.2-stable_linux.x86_64.zip`,
`…_win64.exe.zip` or `…_macos.universal.zip` with the standard library, refuses any archive whose SHA-512
differs from the value pinned in the script (cross-checked once against the release's
`SHA512-SUMS.txt` when pinned; the release's own file is not trusted at run time, because it would only
detect corruption, not a replaced asset), unpacks it, and checks that `--version` starts with
`4.7.2.stable.official`. On Windows it uses the archive's `_console.exe`. `.github/actions/godot` caches
the unpacked build keyed on the version and the pinned hash, exports `GODOT`, and puts a `godot` command
on `PATH` for the bash launchers; on Linux it installs `xvfb` and `mesa-vulkan-drivers` from the
runner's Ubuntu mirror and starts a virtual display. One pinned Godot version exists in the repository:
S23's `export` job reuses this action and adds its export templates (step-23 §16.4: one download step,
owned by whichever lands first).

**Why not ourselves / why not the others.** The binary is Godot's own build; only the hundred lines that
fetch and verify it are ours, because any adopted action would still need our pin to be trustworthy.

**Isolating interface.** One constant block in `scripts/ci_godot.py` (version, three SHA-512 values) and
the composite action. The tests and launchers read only `GODOT` or `godot` on `PATH`.

**Revisit triggers.** A Godot upgrade (change the block, re-pin from `SHA512-SUMS.txt`); S23 needing
export templates; a hosted runner gaining a GPU.

**Accepted limitations.** Windowed tests on Linux run on a software Vulkan device under a virtual
display; they judge behaviour, never frames (`ARC-17`).

**Dated note, 2026-10-15 (RL-e0, `ARC-103`).** The accepted limitation "judge behaviour, never frames" is
narrowed: the far-field checks judge frames **only against frames rendered in the same run** (`ARC-103`),
never against a stored image; every other windowed test is unchanged.
