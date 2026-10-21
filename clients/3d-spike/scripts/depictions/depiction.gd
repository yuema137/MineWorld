## One world, as this client draws it (`DECISIONS.md` ARC-144): the seam between the slice's shared
## machinery and what a world looks like in 3D.
##
## A depiction is a builder with one interface. It builds the scene, and it says what it drew: which
## scene volumes and doors exist, and which TAGGED places each drawn room, door or street depicts. It
## never names a place key. `SliceLayout` binds the places the server discloses to these entries by
## their disclosed tags and by doorway coincidence (16c's D-16c-3 rule, generalised; `slice_layout.gd`).
##
## One depiction per world key, found by `SliceWorlds`: the pack's `depictions/<key>.yaml`, drawn by the
## shared `SliceDataDepiction` (`depictions/data/`, ARC-177; Lakeside is drawn so, with no script of its
## own), else a bespoke script at `res://scripts/depictions/<key>/depiction.gd` (ARC-146 when it is
## absent; no world has one since 16f-d2), else plainly from disclosure (ARC-52).
## 16f keeps this interface and changes only where depictions come from (R-16f-c-1).
##
## THE ENTRIES (`depictions()`), one dictionary per drawn thing that depicts a tagged place:
##
##   tags      PackedStringArray, every one written with `tag()`: all must be among the place's
##             disclosed tags for the entry to be a candidate for it
##   volume    the scene volume's `place_id` (an Area3D of `place_volumes()`), "" for a closed door
##   drawn_as  SliceLayout.ROOM | DOOR | STREET
##   door      the index in `doors()` of the entry's door, or -1
##   origin    null, or the scene position of the place frame's (0, 0) -- for an entry that anchors
##             without a door, such as a street drawn in its own frame
class_name SliceDepiction
extends RefCounted

## The world key this depiction was opened for (set by `SliceWorlds.open`).
var key := ""


## Build the world's scene under `root` and return it. `connected`: a hosted or joined server is
## present, so the world's people are drawn from it and no decorative townspeople are (D-16c-6).
func build(_root: Node3D, _far_field: MineWorldFarField, _connected: bool) -> Node3D:
	push_error("SliceDepiction.build is abstract")
	return null


## Every scene volume (an Area3D whose meta `place_id` is the volume's name) this depiction drew.
func place_volumes() -> Array[Area3D]:
	return []


## What is drawn as which tagged place (the entries above).
func depictions() -> Array[Dictionary]:
	return []


## The drawn doors, in 16c's format (D-16c-4): [sill centre (global), façade yaw, label, the room's
## volume ("" for a closed door), the depth from the sill to the room's inner face, the room floor's
## height above the sill]. Every depiction registers its doors in the scene's one door registry,
## `SliceDoors.rows`, which the probes and the occluders read too.
func doors() -> Array:
	return SliceDoors.rows


## The offline spawn: { position: Vector3, yaw: float (degrees) }. Connected, the body is put where
## the world says (16c, unchanged).
func spawn() -> Dictionary:
	return {"position": Vector3.ZERO, "yaw": 0.0}


## The viewpoints this world is checked and captured from:
##   farfield        [[id, scene position, roles]] (RL-e0b sec.6.1's roles: protected, judged, high)
##   farfield_shots  [[name, position, yaw, pitch]]
##   perf            [[name, position, yaw, pitch]]; perf_breakdown: [name]
##   probe_route     [PackedStringArray of tags, ...]: the link probe's walk, by entries' tags; empty
##                   means the world's own bespoke walk
func viewpoints() -> Dictionary:
	return {}


## The scene's GI volumes and reflection probes: { gi: [{box: AABB, subdiv: 64 | 128 | 256}],
## probes: [{box: AABB, interior: bool}] } (`SliceMain._voxel_gi`, `_reflections`). None by default.
func light() -> Dictionary:
	return {"gi": [], "probes": []}


## Sit points (presentation data, step-33 §4.5); none until S33-f.
func sit_points() -> Array:
	return []


## The far field this depiction draws: the world's own (`far_field/<key>.yaml`) unless the depiction
## draws another world's street, and so that street's far field.
func far_field_key() -> String:
	return key


## One HUD line that says what this depiction is, when it is not the finished world (Q-S33c-1):
## { key: a catalog key, args: its arguments }, or {} for none.
func hud_note() -> Dictionary:
	return {}


## The far-field viewpoints with `role` ("" for all), as [id, position].
func far_field_viewpoints(role: String) -> Array:
	var out := []
	for vp in viewpoints().get("farfield", []):
		if role == "" or role in vp[2]:
			out.append([vp[0], vp[1]])
	return out


## THE ONE PLACE a tag literal is written (X-6, `client_rules.rs` check (d)): a depiction states which
## tags its entries depict, never which place key.
static func tag(name: String) -> String:
	return name


## One entry (the format above).
static func entry(tags: PackedStringArray, volume: String, drawn_as: String, door := -1,
		origin: Variant = null) -> Dictionary:
	return {"tags": tags, "volume": volume, "drawn_as": drawn_as, "door": door, "origin": origin}
