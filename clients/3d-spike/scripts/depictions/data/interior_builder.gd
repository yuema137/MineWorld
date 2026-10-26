## The rooms inside a depiction file's shells (`DECISIONS.md` ARC-177, ARC-307; S33-f sec.5.3): for every
## building with an `interior`, the floor's finish, the ceiling under its first upper storey, the fitted
## parts of the room's shell, and its lamps.
##
## The rules, all from the file:
##   - the room is the building's footprint less half the wall's thickness (`SliceBoxDepiction.WALL_T`),
##     the walls' inner face, so the finishes meet the walls `DepictionShells` drew;
##   - the floor's finish lies `FINISH` above the lowest ground under the footprint, the ceiling one
##     `storey_m` above that ground, both triangulated from the room's polygon;
##   - a fitting is a block from the ground under its centre to `height_m` above it, drawn in its
##     material and with a collider on `Build.LAYER_WORLD` of the same box: it is a solid the world
##     states (a solid naming no furnishing), and the geometry probe compares the two;
##   - a lamp is an `OmniLight3D` `up_m` above the ground at `at_mm`, its energy, range and colour from
##     the file.
## Furniture is never drawn here: every free-standing piece is a furnishing in the world's data, drawn
## from the disclosure (`slice/furnishings.gd`). Nothing here names a world, a place or a tag.
class_name DepictionInteriors
extends RefCounted

## How far the floor's finish stands above the ground, and the ceiling's thickness, metres (schema
## defaults: a finish over the drawn ground, never under the feet by more than the ground check's 50 mm).
const FINISH := 0.006
const CEILING_T := 0.08
## A lamp's falloff (the café's, `cafe_interior.gd`).
const ATTENUATION := 1.15


## Draws the room of every building with an `interior` under `parent`, on `d`'s ground.
static func build(parent: Node3D, buildings: Array, d: SliceDataDepiction) -> void:
	var g: Node3D = null
	for b in buildings:
		if b.get("interior") == null:
			continue
		if g == null:
			g = Node3D.new()
			g.name = "Interiors"
			parent.add_child(g)
		_room(g, b, d)


static func _room(g: Node3D, b: Dictionary, d: SliceDataDepiction) -> void:
	var node := Node3D.new()
	node.name = String(b["name"])
	g.add_child(node)
	var room: Dictionary = b["interior"]
	var plan := PackedVector2Array()
	var base := INF
	for p in b["footprint_mm"]:
		var on := d.ground_at(Vector2(float(p[0]), float(p[1])))
		plan.append(Vector2(on.x, on.z))
		base = minf(base, on.y)
	var inset := Geometry2D.offset_polygon(plan, -SliceBoxDepiction.WALL_T * 0.5, Geometry2D.JOIN_MITER)
	if not inset.is_empty():
		plan = inset[0]
	_slab(node, plan, base + FINISH, 0.0, DepictionShells._mat(room["floor"]), "Floor")
	var ceiling := base + float(b["storey_m"])
	_slab(node, plan, ceiling, CEILING_T, DepictionShells._mat(room["ceiling"]), "Ceiling")
	for f in room.get("fittings", []):
		_fitting(node, f, d)
	for l in room.get("lights", []):
		_lamp(node, l, d)


## A flat slab over `plan` (scene x, z) at height `y`, `thick` deep below it (0: one face up).
static func _slab(g: Node3D, plan: PackedVector2Array, y: float, thick: float, mat: Material,
		nm: String) -> void:
	var tris := Geometry2D.triangulate_polygon(plan)
	if tris.is_empty():
		return
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for k in range(0, tris.size(), 3):
		var a := plan[tris[k]]
		var b := plan[tris[k + 1]]
		var c := plan[tris[k + 2]]
		# both faces: the floor is seen from above, the ceiling from below
		for face in [[a, b, c], [a, c, b]]:
			for v in face:
				st.set_uv(Vector2(v.x, v.y))
				st.add_vertex(Vector3(v.x, y, v.y))
		if thick > 0.0:
			for v in [a, c, b]:
				st.set_uv(Vector2(v.x, v.y))
				st.add_vertex(Vector3(v.x, y + thick, v.y))
	st.generate_normals()
	var mi := MeshInstance3D.new()
	mi.name = nm
	mi.mesh = st.commit()
	mi.material_override = mat
	g.add_child(mi)


## A fitted block, drawn and colliding as the world states it.
static func _fitting(g: Node3D, f: Dictionary, d: SliceDataDepiction) -> void:
	var r: Array = f["box_mm"]
	var lo := d.to_scene(Vector2(float(r[0]), float(r[1])))
	var hi := d.to_scene(Vector2(float(r[0]) + float(r[2]), float(r[1]) + float(r[3])))
	var centre_mm := Vector2(float(r[0]) + float(r[2]) * 0.5, float(r[1]) + float(r[3]) * 0.5)
	var ground := d.ground_at(centre_mm).y
	var h := float(f["height_m"])
	var size := Vector3(absf(hi.x - lo.x), h, absf(hi.z - lo.z))
	var centre := Vector3((lo.x + hi.x) * 0.5, ground + h * 0.5, (lo.z + hi.z) * 0.5)
	var mi := Build.box(g, centre, size, DepictionShells._mat(f))
	mi.name = String(f["name"])
	Build.box_blocker(g, centre, size)


static func _lamp(g: Node3D, l: Dictionary, d: SliceDataDepiction) -> void:
	var at := d.ground_at(Vector2(float(l["at_mm"][0]), float(l["at_mm"][1])))
	var lamp := OmniLight3D.new()
	lamp.light_color = Color.html(l["colour"]) if l.get("colour") != null else Color.WHITE
	lamp.light_energy = float(l["energy"])
	lamp.omni_range = float(l["range_m"])
	lamp.omni_attenuation = ATTENUATION
	lamp.position = at + Vector3(0.0, float(l["up_m"]), 0.0)
	g.add_child(lamp)
