## Occluders from the slice's opaque massing (RL-b SD-RLb-5, SC-6).
##
## Godot's occlusion culling rasterises occluder shapes on the CPU (Embree) and
## skips any instance whose bounds are wholly hidden behind them, in the colour
## and depth passes. The scene is built from script, so there is no edit-time
## bake; the occluders are boxes placed here from the same numbers the buildings
## were built from:
##
##   each ordinary terrace unit: its solid mass, which starts `RECESS_DEEP`
##     behind the façade -- behind every window reveal, shop-window recess and
##     door -- and runs to the rear wall, ground to eaves;
##   the café: its two side walls, its back wall and the solid storeys above;
##   the florist (a real room): its two party walls and the solid storey above.
##     Its back wall is left out: it has a doorway into the back room.
##
## Never glazing, never a door, never a shopfront: every box sits inside a solid
## the player cannot see through, and `build` checks that by testing each box
## against every glass triangle in the slice and every street door, and reports
## any overlap as an error rather than trusting the geometry above.
class_name SliceOccluders
extends RefCounted

## Each box is pulled this far inside the solid it stands for.
const INSET := 0.02


static func build(root: Node3D) -> Dictionary:
	var boxes: Array[AABB] = []
	for m in SliceTerrace.massing:
		_unit(m, boxes)
	_cafe(root, boxes)
	var g := Node3D.new()
	g.name = "Occluders"
	root.add_child(g)
	for b in boxes:
		var oi := OccluderInstance3D.new()
		var shape := BoxOccluder3D.new()
		shape.size = b.size
		oi.occluder = shape
		oi.position = b.get_center()
		g.add_child(oi)
	var bad := _overlaps(root, boxes)
	return {"occluders": boxes.size(), "overlaps": bad}


## One terrace unit, from its `SliceTerrace.massing` entry:
## [global transform of the unit, width, height to the eaves, depth, enterable].
static func _unit(m: Array, boxes: Array[AABB]) -> void:
	var xf: Transform3D = m[0]
	var w: float = m[1]
	var top: float = m[2]
	var depth: float = m[3]
	if bool(m[4]):
		_florist(xf, w, top, boxes)
		return
	var z0 := -SliceTerrace.RECESS_DEEP
	var z1 := -depth
	boxes.append(_inset(xf * AABB(Vector3(-w * 0.5, 0.0, z1), Vector3(w, top, z0 - z1))))


## The enterable unit: party walls and the storey above the room.
static func _florist(xf: Transform3D, w: float, top: float, boxes: Array[AABB]) -> void:
	var room: AABB = SliceShopInterior.room_box
	if room.size == Vector3.ZERO:
		return
	var y_floor := xf.origin.y
	var x_l := xf.origin.x - w * 0.5
	var x_r := xf.origin.x + w * 0.5
	var y_ceil := room.end.y + 0.20
	# the storey above the room
	boxes.append(_inset(AABB(Vector3(room.position.x, y_ceil, room.position.z),
		Vector3(room.size.x, y_floor + top - y_ceil, room.size.z))))
	# the party walls, outside the room's side linings
	for span in [[x_l, room.position.x], [room.end.x, x_r]]:
		var a: float = span[0]
		var b: float = span[1]
		if b - a > INSET * 4.0:
			boxes.append(_inset(AABB(Vector3(a, y_floor, room.position.z),
				Vector3(b - a, top, room.size.z))))


## The café's side walls, back wall and the storeys above the shop, from the
## constants `cafe.gd` builds them with.
static func _cafe(root: Node3D, boxes: Array[AABB]) -> void:
	var cafe := root.find_child("DailyBean", true, false) as Node3D
	if cafe == null:
		return
	var xf := cafe.global_transform
	var w := SliceCafe.W
	var d := SliceCafe.DEPTH
	var t := SliceCafe.WALL_T
	var top := SliceCafe.STOREY * 3.0 + 0.6
	for sx in [-1.0, 1.0]:
		var x0: float = sx * (w * 0.5) - (t if sx > 0.0 else 0.0)
		boxes.append(_inset(xf * AABB(Vector3(x0, 0.0, -d), Vector3(t, top, d - t))))
	boxes.append(_inset(xf * AABB(Vector3(-w * 0.5, 0.0, -d), Vector3(w, top, t))))
	var y0 := SliceCafe.CEIL_Y + 0.14
	boxes.append(_inset(xf * AABB(Vector3(-w * 0.5 + t, y0, -d), Vector3(w - 2.0 * t, top - y0,
		d - t))))


static func _inset(b: AABB) -> AABB:
	return b.grow(-INSET)


## Every occluder against every glass triangle (any merged cell whose material is
## alpha-blended: shop, door and upper-window glazing alike) and every street
## door's opening. Returns how many pairs overlap; each is reported.
static func _overlaps(root: Node3D, boxes: Array[AABB]) -> int:
	var glass: Array[AABB] = []
	for mi in root.find_children("*", "MeshInstance3D", true, false):
		var m := (mi as MeshInstance3D).material_override as BaseMaterial3D
		if m == null or m.transparency != BaseMaterial3D.TRANSPARENCY_ALPHA:
			continue
		var mesh := (mi as MeshInstance3D).mesh
		var xf := (mi as MeshInstance3D).global_transform
		for s in range(mesh.get_surface_count()):
			var faces := mesh.surface_get_arrays(s)
			var v: PackedVector3Array = faces[Mesh.ARRAY_VERTEX]
			var idx: PackedInt32Array = faces[Mesh.ARRAY_INDEX]
			for i in range(0, idx.size(), 3):
				var tri := AABB(xf * v[idx[i]], Vector3.ZERO)
				tri = tri.expand(xf * v[idx[i + 1]]).expand(xf * v[idx[i + 2]])
				glass.append(tri)
	var doors: Array[AABB] = []
	for d in SliceTerrace.doors:
		var p: Vector3 = d[0]
		var yaw: float = d[1]
		doors.append(Transform3D(Basis(Vector3.UP, yaw), p) * AABB(
			Vector3(-0.7, 0.0, -1.0), Vector3(1.4, 2.7, 1.5)))
	var bad := 0
	for b in boxes:
		for g in glass:
			if b.intersects(g):
				bad += 1
				push_error("occluder %s overlaps glazing %s" % [b, g])
		for d in doors:
			if b.intersects(d):
				bad += 1
				push_error("occluder %s overlaps a door opening %s" % [b, d])
	return bad
