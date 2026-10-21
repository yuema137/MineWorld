## A world drawn plainly from the server's disclosure (S14 16f-b, `docs/decisions/ARC-52.md`; the
## operator's ruling Q-16f-1 (A), which amends `ARC-146`): the depiction a world gets when this client
## has no depiction of it.
##
## It draws only what every world has: the world's far field from the pack (`far_field/<key>.yaml`;
## a pack without it is refused `FAR_FIELD_MISSING` by the far-field loader) and plain ground under it. It
## draws no place and states no entry. The layout then anchors the first place the body stands in at
## the scene's origin and glues every other place at its passages (`ARC-45` point 1); `SliceShells`
## draws each from its own `place-shape` -- a floor, walls with an opening at each doorway, a box per
## solid -- styled by the pack's `places/shells.yaml`. Every wall drawn is the server's own, so this
## is not the fallback `ARC-146` declined ("looks like the world and is not it"), and the HUD says
## what it is (`hud.depiction.plain`).
##
## Nothing here names a world or a place.
class_name SlicePlainDepiction
extends SliceDepiction

## The plain ground's side, metres: past every world's places, inside the far field's foreground.
const GROUND_SIZE := 400.0
## The ground stands this far below a place's floor, so a shell's floor slab never fights it.
const GROUND_BELOW := 0.02
## Eye height of the one protected far-field viewpoint, metres (`SliceWorld`'s standing eye).
const EYE := 1.66

var _root: Node3D = null


func build(root: Node3D, far_field: MineWorldFarField, _connected: bool) -> Node3D:
	var g := Node3D.new()
	g.name = "Plain"
	root.add_child(g)
	var eyes := PackedVector3Array()
	for vp in far_field_viewpoints("protected"):
		eyes.append(vp[1])
	SliceFarFieldView.build(g, far_field, eyes)
	Build.ground(g, Vector3(0.0, -GROUND_BELOW, 0.0), GROUND_SIZE, GROUND_SIZE,
		Mats.paint(Color(0.50, 0.52, 0.46), 0.95))
	_root = g
	return g


func place_volumes() -> Array[Area3D]:
	var out: Array[Area3D] = []
	if _root == null:
		return out
	for c in _root.get_children():
		if c is Area3D and c.has_meta("place_id"):
			out.append(c as Area3D)
	return out


## Offline only: the scene's origin. Connected, the body is put where the world says.
func spawn() -> Dictionary:
	return {"position": Vector3(0.0, 0.1, 0.0), "yaw": 0.0}


## One protected far-field viewpoint at the origin; no shots, no perf views. The link probe walks the
## world's own passages (`SliceProbeWorld._passages_check`), so no route is named.
func viewpoints() -> Dictionary:
	return {
		"farfield": [["VP-plain-1", Vector3(0.0, EYE, 0.0), ["protected"]]],
		"farfield_shots": [],
		"perf": [],
		"perf_breakdown": [],
		"probe_route": [],
	}


func hud_note() -> Dictionary:
	return {"key": "hud.depiction.plain", "args": {"world": key}}
