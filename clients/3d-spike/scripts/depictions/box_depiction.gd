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
	var eyes := PackedVector3Array()
	for vp in far_field_viewpoints("protected"):
		eyes.append(vp[1])
	SliceFarFieldView.build(g, far_field, eyes)
	Build.ground(g, Vector3.ZERO, GROUND_SIZE, GROUND_SIZE, Mats.pbr("leafy_grass", 1.4,
		Color(0.62, 0.86, 0.42)))
	var wall := Mats.paint(Color(0.62, 0.62, 0.60), 0.85)
	var floor_mat := Mats.paint(Color(0.48, 0.47, 0.44), 0.9)
	var s := street()
	var street_volume := _volume(g, "stub.street", s["footprint"])
	_floor(g, s["footprint"], floor_mat)
	_entries.append(SliceDepiction.entry(s["tags"], street_volume, SliceLayout.STREET, -1,
		frame_origin()))
	var i := 0
	for p in places():
		var volume := _volume(g, "stub.%d" % i, p["footprint"])
		var door := _door(g, p, volume)
		_walls(g, p["footprint"], p["doorway"], wall)
		_floor(g, p["footprint"], floor_mat)
		_entries.append(SliceDepiction.entry(p["tags"], volume, SliceLayout.ROOM, door))
		i += 1
	_root = g
	return g


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
	SliceTerrace.doors.append([sill, yaw, "%s -- stub" % p["label"], volume, 0.0, 0.0])
	Build.label(g, sill + Vector3(0.0, WALL_H + 0.35, 0.0) + out * 0.05, p["label"], 0.45,
		Color(0.95, 0.95, 0.92), yaw)
	return SliceTerrace.doors.size() - 1


## Four walls on the footprint's edges, inside it, with a DOOR_W gap on the street side at the doorway.
func _walls(g: Node3D, rect: Rect2, doorway: Vector2, mat: Material) -> void:
	var side := _side(rect, doorway)
	var lo := rect.position
	var hi := rect.end
	var t := WALL_T * 1000.0
	var edges := [
		[Vector2(0, -1), Rect2(lo.x, lo.y, hi.x - lo.x, t)],
		[Vector2(0, 1), Rect2(lo.x, hi.y - t, hi.x - lo.x, t)],
		[Vector2(-1, 0), Rect2(lo.x, lo.y + t, t, hi.y - lo.y - 2.0 * t)],
		[Vector2(1, 0), Rect2(hi.x - t, lo.y + t, t, hi.y - lo.y - 2.0 * t)],
	]
	for e in edges:
		var r: Rect2 = e[1]
		if e[0] != side:
			_wall(g, r, mat)
			continue
		var half := DOOR_W * 500.0
		if side.y != 0:
			_wall(g, Rect2(r.position.x, r.position.y, doorway.x - half - r.position.x, r.size.y), mat)
			_wall(g, Rect2(doorway.x + half, r.position.y, r.end.x - doorway.x - half, r.size.y), mat)
		else:
			_wall(g, Rect2(r.position.x, r.position.y, r.size.x, doorway.y - half - r.position.y), mat)
			_wall(g, Rect2(r.position.x, doorway.y + half, r.size.x, r.end.y - doorway.y - half), mat)


func _wall(g: Node3D, r: Rect2, mat: Material) -> void:
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return
	var box := _box(r)
	var size: Vector3 = box[1]
	Build.box(g, (box[0] as Vector3) + Vector3(0.0, WALL_H * 0.5, 0.0), Vector3(size.x, WALL_H, size.z),
		mat, 0.0, true)
