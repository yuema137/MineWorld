## A stub depiction built from data (`DECISIONS.md` ARC-144; S33-c §5.5): flat ground, the world's far
## field, and one walled box per place at its footprint, with a gap at its door -- so a world can be
## walked in 3D, on the same server path, before its real depiction exists.
##
## The mechanism is here, shared; a world's stub is only its data table (`places()`, `street()`,
## `frame_origin()`, `spawn()`, `viewpoints()`, `hud_note()`), so the next world's stub is data only.
## Nothing here names a place key: every place is written by its tags (`SliceDepiction.tag()`).
##
## THE FRAME. A stub draws its places in one frame -- the street's, millimetres, x east, y north
## (`CORE_CONCEPTS.md` §6.1) -- whose (0, 0) stands at `frame_origin()` in the scene. A footprint is a
## Rect2 in that frame; a doorway is the point on the STREET side of a place's door, as the world's
## passages state it there, so that the drawn door's out-point is exactly that point and binding by
## doorway coincidence (`SliceLayout`, DOOR_MATCH) holds by construction. The door's sill stands
## `SliceLayout.DOOR_INSET` behind it, toward the place (12d's 0.40 m authoring rule).
class_name SliceBoxDepiction
extends SliceDepiction

const WALL_H := 2.4
const WALL_T := 0.2
## The gap left in a wall for a door, centred on the doorway.
const DOOR_W := 1.4
## The place volumes reach this far below the ground (the slice's rule: a body standing on the floor is
## inside its volume, `SliceWorld._place_volumes`) and this high.
const VOLUME_BELOW := 0.5
const VOLUME_H := 3.5
const GROUND_SIZE := 200.0

var _root: Node3D = null
var _volumes: Array[Area3D] = []
var _entries: Array[Dictionary] = []


# --- the data a stub supplies --------------------------------------------------------------------

## Where the frame's (0, 0) stands in the scene.
func frame_origin() -> Vector3:
	return Vector3.ZERO


## The street every place opens onto: { tags: PackedStringArray, footprint: Rect2 (mm) }.
func street() -> Dictionary:
	return {}


## Every other place: [{ tags: PackedStringArray, label: String, footprint: Rect2 (mm),
## doorway: Vector2 (mm, on the street side) }].
func places() -> Array:
	return []


# --- the shared builder --------------------------------------------------------------------------

func build(root: Node3D, far_field: MineWorldFarField, _connected: bool) -> Node3D:
	var g := Node3D.new()
	g.name = "Stub"
	root.add_child(g)
	draw_far_field(g, far_field)
	draw_ground(g)
	register(g)
	draw_boxes(g)
	_root = g
	return g


## The world's far field, drawn round the protected viewpoints; its flat ground inside the boundary
## `near_ground_drop` below where it stands for a world drawn on it (`SliceFarFieldView.build`).
func draw_far_field(g: Node3D, far_field: MineWorldFarField, near_ground_drop := 0.0) -> void:
	var eyes := PackedVector3Array()
	for vp in far_field_viewpoints("protected"):
		eyes.append(vp[1])
	SliceFarFieldView.build(g, far_field, eyes, near_ground_drop)


## The stub's flat ground.
func draw_ground(g: Node3D) -> void:
	Build.ground(g, Vector3.ZERO, GROUND_SIZE, GROUND_SIZE, Mats.pbr("leafy_grass", 1.4,
		Color(0.62, 0.86, 0.42)))


## What this depiction says it drew, from `street()` and `places()` (S33-d R2-4): one scene volume per
## place, its door in the scene's door registry (with its label over the sill), and one entry per place.
## Shared by every depiction built from this data, whatever else draws the place.
func register(g: Node3D) -> void:
	var s := street()
	var street_volume := _volume(g, "stub.street", s["footprint"])
	_entries.append(SliceDepiction.entry(s["tags"], street_volume, SliceLayout.STREET, -1,
		frame_origin()))
	var i := 0
	for p in places():
		var volume := _volume(g, place_volume(i), p["footprint"])
		var door := _door(g, p, volume)
		_entries.append(SliceDepiction.entry(p["tags"], volume, SliceLayout.ROOM, door))
		i += 1


## The scene volume's name of the `i`-th place of `places()`.
static func place_volume(i: int) -> String:
	return "stub.%d" % i


## The stub's drawing: a floor on the street and a walled grey box on every place, each wall with a
## DOOR_W gap at every doorway it carries.
func draw_boxes(g: Node3D) -> void:
	var wall := Mats.paint(Color(0.62, 0.62, 0.60), 0.85)
	var floor_mat := Mats.paint(Color(0.48, 0.47, 0.44), 0.9)
	_floor(g, street()["footprint"], floor_mat)
	var doorways := []
	for p in places():
		doorways.append(p["doorway"])
	for p in places():
		_walls(g, p["footprint"], doorways, wall)
		_floor(g, p["footprint"], floor_mat)


## The label drawn over a place's door and named in the door registry.
func door_label(label: String) -> String:
	return "%s -- stub" % label


func place_volumes() -> Array[Area3D]:
	return _volumes


func depictions() -> Array[Dictionary]:
	return _entries


## A frame point (mm) in the scene, on the ground.
func to_scene(mm: Vector2) -> Vector3:
	return frame_origin() + MineWorldSpace.to_3d({"x": mm.x, "y": mm.y})


## The scene centre and size of a footprint, at ground level.
func _box(rect: Rect2) -> Array:
	var a := to_scene(rect.position)
	var b := to_scene(rect.end)
	var centre := (a + b) * 0.5
	return [centre, Vector3(absf(b.x - a.x), 0.0, absf(b.z - a.z))]


func _volume(g: Node3D, id: String, rect: Rect2) -> String:
	var box := _box(rect)
	var size: Vector3 = box[1]
	SliceWorld._place(g, id, (box[0] as Vector3) + Vector3(0.0, VOLUME_H * 0.5 - VOLUME_BELOW, 0.0),
		Vector3(size.x, VOLUME_H, size.z))
	_volumes.append(g.get_child(g.get_child_count() - 1) as Area3D)
	return id


func _floor(g: Node3D, rect: Rect2, mat: Material) -> void:
	var box := _box(rect)
	var size: Vector3 = box[1]
	Build.box(g, (box[0] as Vector3) + Vector3(0.0, 0.01, 0.0), Vector3(size.x, 0.02, size.z), mat)


## The outward normal (toward the street) of the footprint's side the doorway faces, in the frame.
static func _side(rect: Rect2, doorway: Vector2) -> Vector2:
	if doorway.y <= rect.position.y:
		return Vector2(0, -1)
	if doorway.y >= rect.end.y:
		return Vector2(0, 1)
	if doorway.x <= rect.position.x:
		return Vector2(-1, 0)
	return Vector2(1, 0)


## The place's door in 16c's format, registered in the scene's door registry; returns its index.
func _door(g: Node3D, p: Dictionary, volume: String) -> int:
	var out_frame := _side(p["footprint"], p["doorway"])
	var out := MineWorldSpace.to_3d({"x": out_frame.x, "y": out_frame.y}).normalized()
	var yaw := atan2(out.x, out.z)
	var at := to_scene(p["doorway"])
	var sill := at - out * SliceLayout.DOOR_INSET
	# the room's floor height (a deck's surface, ARC-178) puts its in-point, and so its frame, on that floor
	SliceTerrace.doors.append([sill, yaw, door_label(p["label"]), volume, 0.0, float(p.get("floor_m", 0.0))])
	Build.label(g, sill + Vector3(0.0, WALL_H + 0.35, 0.0) + out * 0.05, p["label"], 0.45,
		Color(0.95, 0.95, 0.92), yaw)
	return SliceTerrace.doors.size() - 1


## Four walls on the footprint's edges, inside it, with a DOOR_W gap on the street side at the doorway.
## Four walls on the footprint's edges, inside it, each with a DOOR_W gap at every doorway (of any place)
## that stands within DOOR_GAP_MATCH_MM of its centre line and inside its span -- so two boxes that share a
## door, such as a room off another room, both open at it.
func _walls(g: Node3D, rect: Rect2, doorways: Array, mat: Material) -> void:
	var lo := rect.position
	var hi := rect.end
	var t := WALL_T * 1000.0
	var edges := [
		Rect2(lo.x, lo.y, hi.x - lo.x, t),
		Rect2(lo.x, hi.y - t, hi.x - lo.x, t),
		Rect2(lo.x, lo.y + t, t, hi.y - lo.y - 2.0 * t),
		Rect2(hi.x - t, lo.y + t, t, hi.y - lo.y - 2.0 * t),
	]
	for r in edges:
		var along_x: bool = r.size.x > r.size.y
		var line: float = r.get_center().y if along_x else r.get_center().x
		var from: float = r.position.x if along_x else r.position.y
		var to: float = r.end.x if along_x else r.end.y
		# the shared span function (`SliceSpans`, 16f-b), in this depiction's millimetres
		var gaps := SliceSpans.openings(doorways, along_x, line, DOOR_GAP_MATCH_MM, Vector2(from, to))
		for piece in SliceSpans.split(from, to, gaps, DOOR_W * 1000.0):
			_wall_span(g, r, along_x, piece[0], piece[1], mat)


## How far a disclosed doorway may stand from a wall's centre line for this depiction to draw that
## wall's gap there, in millimetres: a drawing tolerance, not a rule (the server decides who passes). The
## doorway rule puts a doorway 0.2 to 0.4 m in front of its door (12d), the wall half its thickness further.
const DOOR_GAP_MATCH_MM := 1000.0


func _wall_span(g: Node3D, r: Rect2, along_x: bool, from: float, to: float, mat: Material) -> void:
	if to <= from:
		return
	if along_x:
		_wall(g, Rect2(from, r.position.y, to - from, r.size.y), mat)
	else:
		_wall(g, Rect2(r.position.x, from, r.size.x, to - from), mat)


func _wall(g: Node3D, r: Rect2, mat: Material) -> void:
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return
	var box := _box(r)
	var size: Vector3 = box[1]
	Build.box(g, (box[0] as Vector3) + Vector3(0.0, WALL_H * 0.5, 0.0), Vector3(size.x, WALL_H, size.z),
		mat, 0.0, true)
