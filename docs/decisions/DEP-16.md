## DEP-16 — The 2D reference client: Godot built-ins and the shared protocol module, no addon

**Date** 2026-10-08 · **Status** selected · **Approved by** the primary session at S12 PR 13a's design
freeze · **Relates to** `DEP-4`, `DEP-9`, `ARC-46`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design**
step-13 §5 (the comparison), §14.4

**Problem.** The 2D client needs drawing, UI widgets, a connection, routing and pack-file reading.
Each has mature candidates; some would put MineWorld inside another framework's execution model.

**Choice, per piece** (step-13 §5 holds the full comparison and sources):

```text
drawing          Node2D + Sprite2D, y-sorted, Camera2D, canvas shaders     adopt (built-in)
                 TileMapLayer                                              only if ground drawing is too slow
                 Tiled + YATI, LDtk + importer                             reject: a second source of layout
widgets          Control nodes + Theme                                     adopt (built-in)
                 Dialogue Manager, Dialogic 2                              reject: a second source of conversation
connection       clients/protocol/mineworld over WebSocketPeer             adopt (ours, already shared with 3D)
                 Godot MultiplayerAPI, Nakama                              reject: another authority model
                 godot-rust (gdext)                                        reject for now: native per platform
routing          straight strides through disclosed doorways               adopt
                 NavigationServer2D from a disclosed place-shape           adopt when a place discloses solids
pack files       Godot's JSON parser over YAML's JSON-compatible subset    adopt
                 fimbul-works/godot-yaml, KoBeWi/Godot-YAML (GDExtension,  reject: native per platform
                 RapidYAML)
                 YAML.gd (pure GDScript)                                   reject: an unvetted parser for two
                                                                           small files we author
test harness     a scripted scene printing PASS/FAIL, driven by Rust tests adopt; GUT/gdUnit4 not needed yet
```

**Why not ourselves.** Nothing here is built that a built-in provides; the client's own code is
acquisition, drawing and the protocol module that already exists.

**Isolating interface.** Pack files are read in one script (`clients/2d/scripts/presentation.gd`); a
YAML reader, if a pack author ever needs full YAML, replaces that one call.

**Limitation accepted.** Pack files cannot carry comments; their documentation is a specification
file beside the client. Revisit when a pack author needs full YAML.

Sources checked 2026-10-08: [fimbul-works/godot-yaml](https://github.com/fimbul-works/godot-yaml) ·
[KoBeWi/Godot-YAML](https://github.com/KoBeWi/Godot-YAML) ·
[YAML.gd](https://godotengine.org/asset-library/asset/4120), plus step-13 §5's list.
