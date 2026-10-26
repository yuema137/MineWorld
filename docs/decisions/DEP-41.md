## DEP-41 — Official Godot 4.7.2 export templates, checksum-pinned

**Date** 2026-10-10 · **Status** selected · **Approved by** the S23 freeze · **Relates to** `DEP-4`,
`ARC-80`, `step-14-ci.md` §5.7 · **Design** step-23 §5.2, §12, §18.2

**Problem.** Exporting the clients needs Godot export templates of exactly the engine version the
projects target.

**Choice.** The official `Godot_v4.7.2-stable_export_templates.tpz` from the `godotengine/godot`
release `4.7.2-stable` (1,281,349,702 bytes), fetched by version and accepted only if its SHA-512 equals
the line in that release's `SHA512-SUMS.txt`
(`ca4d71c4d7b81dfc15d1a98baa07534aa95b03fdda78a0075b06672e1648d2e5f40980c9adc28d23e1b92e732ee7bf3461997aa804af74ec2fcd7a93ccb84079`,
checked 2026-10-10). Installed in the user's Godot templates folder, never in the repository; cached in
CI by version.

**Options compared.** `barichello/godot-ci` (already declined, `step-14-ci.md` §5.7: a third-party image
for a download we can pin ourselves); building templates from source (smaller binaries through feature
stripping and the only way to regain `--main-pack`, but hours of CI and a C++ toolchain to own: declined
until the size budget demands it).

**Limitation accepted.** Official release templates are built with `disable_path_overrides`, so
`--main-pack` is unavailable (step-23 §18.2, P-3) and each client is exported with its own engine
(`ARC-80`). The macOS universal and arm64 exports require the project setting
`rendering/textures/vram_compression/import_etc2_astc=true` (P-5).
