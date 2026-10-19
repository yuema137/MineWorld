## ARC-13 — The 3D lighting rig was changed on purpose, and it is one revert

**Date** 2026-09-27 · **Scope** `clients/3d-spike` · **Status under `ARC-11`** default-style
**candidate**, not an accepted default

This records a **reversal of direction**, because a reversal that lives only in an agent report
is not reviewable later, and `CLAUDE.md` §2.2 puts design decisions in documents rather than in
conversation.

**What happened.** The 3D spike's lighting rig was repeatedly protected: it was the best thing in
the spike and the standing instruction was to preserve it unchanged while the character and
buildings were worked on. Later review of the same build against
`presentation/mineworld-default/3D/references/` reversed that: the operator's complaint was
**画风** — the rendering idiom, not the geometry inventory — and the lighting was identified as the
single highest-leverage gap, above massing, weathering and asset detail.

**Why the later direction wins.** The plates are warm, saturated and lit by a low sun. The rig was
a bright overcast sky, a pale key and mild saturation, so every frame read grey regardless of what
the geometry did. No amount of bays, gables or dirt passes changes the register of a frame lit
like an overcast afternoon. The shot named `21_street_golden`, framed against
`05_main_street_golden_hour`, contained no golden hour at all — a filename promising what the
frame did not deliver, which is its own kind of evidence defect.

**What changed, and what deliberately did not.** The *structure* of the rig is untouched, because
the structure is the part that was good: one sky, one sun, one tonemap, one exposure, SSAO, glow,
and every asset judged inside it (`DEP-8`'s coherence procedure, step 3). Only values changed:

| | before | after |
| --- | --- | --- |
| sky HDRI | `kloofendal_48d_partly_cloudy_puresky_2k.hdr` | `qwantani_puresky_2k.hdr` |
| sun elevation | 26° | 17° |
| sun colour | `(1.0, 0.88, 0.71)` | `(1.0, 0.80, 0.56)` |
| sun energy | 2.35 | 2.9 |
| tonemap exposure | 1.18 | 1.06 |
| adjustment saturation | 1.06 | 1.16 |
| adjustment contrast | 1.04 | 1.07 |

All of it is in `scripts/main.gd` `_environment()` and `_sun()`, plus one filename. **Reverting is
those seven values and the sky file** — deliberately, so that a taste decision the operator has
not yet made stays cheap to undo.

**Authority.** Under `ARC-11` this is taste, so it is a *candidate*. No agent may record it as the
accepted default look; that remains the operator's, finally and always.

**Also fixed while in here, and not a taste decision:** the distant ridge's heightfield was wound
clockwise seen from above, so its generated normals pointed at the ground — lit from underneath,
culled from above, reading as pale angular shards floating over the lake. That was a defect, and
it is fixed regardless of what happens to the lighting.
