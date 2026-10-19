## DEP-36 — Noto Sans SC Regular bundled as the clients' CJK fallback font

**Date** 2026-10-09 · **Status** selected; one font file added under the DEP-8 font exception ·
**Approved by** the operator (QSET-1, 2026-10-08) · **Relates to** `DEP-8` (its font exception and table
row), `ARC-55`, `ARC-76` · **Design** step-20 §3.7, §4.4

**Problem.** Godot's default font has no CJK glyphs. Without a bundled font, Chinese text renders only
through whatever font the operating system supplies — PingFang on macOS, Microsoft YaHei on Windows,
nothing at all (tofu boxes) on a minimal Linux — so the same build looks different per machine and a
capture cannot be compared.

**Choice.** `clients/shared/settings/fonts/NotoSansSC-Regular.otf`, the unmodified upstream file
(`notofonts/noto-cjk`, `Sans/SubsetOTF/SC/NotoSansSC-Regular.otf`, commit
`165c01b46ea533872e002e0785ff17e44f6d97d8`, 8,331,336 bytes, SHA-256
`faa6c9df652116dde789d351359f3d7e5d2285a2b2a1f04a2d7244df706d5ea9`), beside its `OFL.txt`, named in
`NOTICE`. It is loaded at runtime and appended to the **fallbacks** of the default theme font, so Latin
text keeps its current font glyph for glyph and only glyphs the default font lacks come from Noto Sans
SC, before the system is asked. A check proves every character of every shipped catalog is in the
bundled font with system fallback off (`clients/shared/checks/glyph_check.gd`).

**Options compared** (step-20 §4.4): the variable Noto Sans SC (twice the size for weights the UI does
not use); a GB 2312 subset (a build step, rare characters become tofu, and a Modified Version needs a
new decision); Source Han Sans SC (identical glyphs, no advantage); WenQuanYi Micro Hei through its
Apache-2.0 branch (the fallback had OFL been refused); Droid Sans Fallback (dated); the OS's own fonts
(rejected as the mechanism: different on every machine).

**Accepted cost.** 8.3 MB in Git history, once.
