# PR vf-furniture-solids — people walk through the café's chairs in the 3D client

**Status.** The content fix (§3) is inside the frozen route (`ARC-39`, step-11 §19.3.1's `put_solid`
rule) and ships in this PR, with `ARC-306` amending one row of §19.3.1. §4 is a contract question for
the operator; nothing in §4 is implemented.

**Branch** `fix/vf-furniture-solids` from `origin/main @88ce5cf9`. **Decision** `ARC-306`.

## 1. The defect, as recorded

`/Users/yuema137/mineworld-demos/video/footage/run-a/a-3d.avi` (recorded from `origin/main @370b9da0`,
before 12d), 20–23 s: the visitor walks the window lane of the café and her legs pass through a window
stool; NPCs stand and walk among chairs no collider holds. `a-3d.log`: the server accepted the visitor at
café (1 596, 1 139), which is on the drinks ledge's line (y 950 … 1 290).

## 2. Findings

### 2.1 Check 1 — is the furniture a solid, and where do the drawn positions come from?

| Object | Drawn by (main) | Slice collider | `places/cafe.yaml` solid |
| --- | --- | --- | --- |
| three round tables | `cafe_interior.gd:482` `put_solid` | yes | yes (step-11 §19.3.1) |
| six table chairs | `cafe_interior.gd:485,487` `SliceProps.put` | **no** | **no** |
| three window stools | `cafe_interior.gd:458` `SliceProps.put` | **no** | **no** (§19.3.1: "the stools … are visual") |
| drinks ledge | `cafe_interior.gd:454` `Build.box(…)`, no `collide` | **no** | **no** |

- The furniture positions are hand-placed in GDScript (`_loose_seating`, `_window_bench`), not read
  from the world. The world's `body.solids` are a hand transcription of the slice's colliders (step-11
  §19.3.1, E-TD1), so whatever the slice draws without a collider is missing on both sides.
- Where both exist, they agree: table east is `put_solid` at scene (4.86, −3.50), yaw 0.15, glTF bounds
  0.796 m → 0.833 m box, which is the server's (4 443, 2 743)–(5 277, 3 577).
- `market-town`'s café is the same room (`town_bodies.rs` TD-3) and has the same gap.

### 2.2 Check 2 — does the 3D client ignore the server's resolved positions?

On `main`, yes, partly:

- The player's body is moved to the server's position only on the first view and after a refused
  `move` (`slice_link.gd:338`, `:551`, `:582`). A stride the server *accepts but stops short*
  (`stopped-short`, 12d) is never reconciled.
- Perceived people are snapped to each observed position (`slice_link.gd:366`): no straight-line
  interpolation through furniture on `main`; their positions are the server's, which walk through the
  chairs only because the server holds no chair.

Both are 12e's scope and are fixed there (PR #254, not touched here): `correction.gd` (`ARC-50`, the
150 mm correction), `client_bodies.gd` (colliders built from the disclosed `place-shape`, so the client
body meets every server solid, including the ten this PR adds), `figures.gd` (figures walk toward each
observed position). One residue for 12e to know: `figures.gd` walks a figure in a straight line between
two observed positions; both are server-legal, and a stride is ≤ 2 m, so a figure can at most graze a
solid's corner between two observations. No change to 12e is required by this PR.

### 2.3 The 2D lane's hint (`--drive=walk`, seat `visitor`, stops short at the café shopfront)

Most likely not a data mismatch (read from the code, not re-run here). The street's terrace tables (`street.yaml` lines 81–86: x 530 … 5 870,
y 780 … 2 020) are solids, as 12d authored them from the slice. The 2D drive's `walk` scenario
(`clients/2d/scripts/harness/drive.gd:181–230`) goes out of the café door, 1.5 m toward the street's
middle, then in a straight leg to `FRONT_M` in front of the *nearest other* doorway; in social-cafe that
is the store's at x 8 390, so the straight leg runs along the frontage through the terrace and the
server stops it short. Fix belongs to the 2D drive (route its leg via the street's middle, or around
the frontage), not to the data. Recorded for the 2D lane; not changed here.

## 3. What ships (inside the frozen route)

1. `worlds/social-cafe/places/cafe.yaml` and `worlds/market-town/places/cafe.yaml`: ten solids — six
   chairs, three stools, the ledge — sized by §19.3.1's `put_solid` rule (glTF POSITION bounds, yawed,
   × 0.92, centred where the slice places them). Both towns still validate (`mineworld validate`).
2. `clients/3d-spike/scripts/slice/cafe_interior.gd`: the chairs and stools with `put_solid`, the ledge
   with `collide`, so the slice's own physics stops the body where the server does (before and after
   12e).
3. `tools/cli/tests/town_bodies.rs`: in each town, a stride into a chair stops at the chair
   (x ≤ 3 648), not at the table behind it (x 4 183). Fails on the base (both towns at 4 183), passes
   here.
4. `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt` re-captured: the town's 30-day
   run changes because people now walk around the chairs (123 233 facts, was 122 707; 0 faults).

## 4. Operator question (not implemented)

**Q-VF-F1. Should the furniture the 3D client draws come from the world's data?** Today the slice draws
the café's furniture from GDScript constants and the world's solids are copied from them by hand. Any
world whose café differs (a generated variant, a third town) is drawn with the slice's furniture where
its solids are not, and this defect returns.

Options:

- (a) **Keep the transcription**, and make 12e's geometry probe (`--world --geometry`) fail CI when a
  slice collider and a disclosed solid disagree by more than 150 mm. Cheap; still one world's layout.
- (b) **Solids carry a furniture kind.** Extend a place's `body.solids` entries with an optional
  `kind` (e.g. `chair`, `table`, `stool`) and `yaw`; `bodies` discloses it in `place-shape`; the 3D
  client draws a kind's asset at each disclosed solid (and draws nothing it was not told), and the 2D
  client its glyph. The server stays authoritative and unaware of rendering (a kind is a semantic
  label, like a tag; the asset choice is the client's).
- (c) Furniture as `Item` entities with bodies. Heavier; furniture that never moves does not need an
  entity.

**Recommended: (b)**, as its own small PR design (contract change to `PlaceShape`'s written form and
its disclosure, `ARC-39` amended), after 12e lands, with acceptance in social-cafe and a generated
variant whose café layout differs.

**Ruled (operator, 2026-10-19, relayed):** the furniture the 3D client draws comes from world data, as its
own PR after 12e. Designed in [`pr-vf-furnishings-from-data.md`](pr-vf-furnishings-from-data.md) (DRAFT),
which refines option (b) into a `furnishings` list that solids name (Q-FD-1 there).
