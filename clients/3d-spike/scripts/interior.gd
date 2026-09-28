## Buildings you can walk into.
##
## The shop row used to be solid: one collidable box per unit with joinery stuck
## on the front, and a warm emissive card behind the glass standing in for a
## room. That reads acceptably from across the street and fails the moment
## anyone walks up to a door, which is the thing the spike now has to answer --
## what it feels like to *be inside* the space.
##
## So an enterable unit is built as a shell instead of a mass: four walls with
## real thickness, an opening where the door is, a floor, a ceiling, and a
## fit-out. Nothing here is projected, billboarded or faked. If you can see it
## through the window you can walk to it.
##
## Why a shell and not a subtracted box: Godot has no CSG in the runtime path we
## use, and a solid box cannot have a hole cut in it. The front wall is
## therefore assembled from pieces -- two piers, a spandrel split around the
## doorway, and a header over the glazing -- which is also how the real thing is
## built, so the proportions come out right rather than being fought.
##
## LIGHTING: the exterior rig is untouched. An interior only adds its own local
## lights, because the one sun and one HDRI that light the street cannot reach
## into a room and the alternative -- brightening the environment -- would wreck
## the street to fix the room.
class_name Interior
extends RefCounted

const WALL_T := 0.30
const CEIL_H := 3.40


static func _wood(c: Color) -> Material:
	return Mats.paint(c, 0.62)


## The shell: everything structural, with a doorway left open at `dx`.
##
## Local space matches `Town.shopfront`: x is centred on the unit, z runs from 0
## at the pavement face to `depth` at the back, y from the pavement.
static func shell(g: Node3D, w: float, depth: float, h: float, top: float,
		gw: float, dx: float, door_w: float, wall: Material) -> void:
	var inner_w := w - WALL_T * 2.0
	var back := depth - WALL_T

	# floor and ceiling
	Build.slab(g, 0, depth * 0.5, inner_w, depth, 0.0, 0.08, _floor_mat(), 0.0, true)
	Build.slab(g, 0, depth * 0.5, w, depth, CEIL_H, 0.22,
		Mats.paint(Color(0.55, 0.53, 0.50), 0.95))

	# side and back walls
	Build.slab(g, -w * 0.5 + WALL_T * 0.5, depth * 0.5, WALL_T, depth, 0, h, wall, 0.0, true)
	Build.slab(g, w * 0.5 - WALL_T * 0.5, depth * 0.5, WALL_T, depth, 0, h, wall, 0.0, true)
	Build.slab(g, 0, back + WALL_T * 0.5, w, WALL_T, 0, h, wall, 0.0, true)
	# the storeys above the shop are solid -- nobody goes up there
	if h > CEIL_H + 0.5:
		Build.slab(g, 0, depth * 0.5, w, depth, CEIL_H + 0.22, h - CEIL_H - 0.22, wall, 0.0, true)

	# --- front wall, in pieces so the openings are real ----------------------
	var pier := (w - gw) * 0.5
	var head_y := top - 0.25
	Build.slab(g, -w * 0.5 + pier * 0.5, WALL_T * 0.5, pier, WALL_T, 0, h, wall, 0.0, true)
	Build.slab(g, w * 0.5 - pier * 0.5, WALL_T * 0.5, pier, WALL_T, 0, h, wall, 0.0, true)
	Build.slab(g, 0, WALL_T * 0.5, gw, WALL_T, head_y, h - head_y, wall, 0.0, true)

	# stallriser below the glazing, split around the doorway
	var l0 := -gw * 0.5
	var r1 := gw * 0.5
	var d0 := dx - door_w * 0.5
	var d1 := dx + door_w * 0.5
	if d0 > l0:
		Build.slab(g, (l0 + d0) * 0.5, WALL_T * 0.5, d0 - l0, WALL_T, 0, 0.70,
			wall, 0.0, true)
	if r1 > d1:
		Build.slab(g, (d1 + r1) * 0.5, WALL_T * 0.5, r1 - d1, WALL_T, 0, 0.70,
			wall, 0.0, true)
	# lintel over the doorway, up to the glazing line
	Build.slab(g, dx, WALL_T * 0.5, door_w, WALL_T, 2.30, maxf(0.05, head_y - 2.30),
		wall, 0.0, true)

	# reveal linings, so the opening reads as a thickness rather than a cut
	var lin := _wood(Color(0.30, 0.27, 0.24))
	Build.slab(g, d0 - 0.03, WALL_T * 0.5, 0.06, WALL_T, 0, 2.30, lin)
	Build.slab(g, d1 + 0.03, WALL_T * 0.5, 0.06, WALL_T, 0, 2.30, lin)
	Build.slab(g, dx, WALL_T * 0.5, door_w + 0.12, WALL_T, 2.30, 0.06, lin)


static func _floor_mat() -> Material:
	return Mats.pbr("weathered_brown_planks", 2.4, Color(1.18, 1.06, 0.92), 0.72)


## An open door leaf, hinged at the jamb and swung inward, plus a threshold.
static func open_door(g: Node3D, dx: float, door_w: float, accent: Color) -> void:
	var leaf := Node3D.new()
	leaf.position = Vector3(dx - door_w * 0.5, 0, WALL_T * 0.5)
	leaf.rotation.y = -1.85
	g.add_child(leaf)
	var j := _wood(accent)
	Build.slab(leaf, door_w * 0.5, 0, door_w, 0.06, 0.02, 2.24, j)
	Build.slab(leaf, door_w * 0.5, -0.04, door_w - 0.26, 0.03, 0.95, 1.05, Mats.glass())
	Build.sphere(leaf, Vector3(door_w - 0.14, 1.05, -0.08), 0.04,
		Mats.paint(Color(0.72, 0.60, 0.33), 0.30, 0.85), 8)
	Build.slab(g, dx, WALL_T * 0.5, door_w + 0.2, WALL_T + 0.1, 0.0, 0.03, Mats.cutstone())


## Warm interior light. Kept local: the street rig is not touched.
## An interior has to be bright enough to read from the sunlit pavement, which
## is a higher bar than being bright enough to stand in. The exterior exposure
## is tuned for a late afternoon street, so a room lit to a comfortable interior
## level shows through the glass as a flat black panel -- the first version of
## this looked exactly like the emissive card it replaced. The room is therefore
## lit hot on purpose, and the ceiling is a mid grey so it does not blow out.
static func _lights(g: Node3D, depth: float, w: float, warm: Color, n := 3) -> void:
	for i in n:
		var z := depth * (float(i) + 0.7) / (n + 0.4)
		var l := OmniLight3D.new()
		l.light_color = warm
		l.light_energy = 6.5
		l.omni_range = 9.5
		l.omni_attenuation = 1.1
		# one shadow-caster is enough to ground the furniture; the rest are fill,
		# and shadowed omnis in a room this small cost more than they show
		l.shadow_enabled = (i == 0)
		l.position = Vector3(0, CEIL_H - 0.95, z)
		g.add_child(l)
		# the pendant it is supposed to be coming from
		Build.cyl(g, Vector3(0, CEIL_H - 0.02, z), 0.012, 0.012, 0.42,
			Mats.paint(Color(0.16, 0.15, 0.14), 0.5))
		Build.cyl(g, Vector3(0, CEIL_H - 0.56, z), 0.20, 0.09, 0.16,
			Mats.paint(Color(0.18, 0.20, 0.19), 0.35, 0.6))
		Build.sphere(g, Vector3(0, CEIL_H - 0.60, z), 0.055,
			Mats.emissive(warm, 6.0), 8)


static func _chair(g: Node3D, pos: Vector3, yaw: float, c: Color) -> void:
	var m := _wood(c)
	var n := Node3D.new()
	n.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	g.add_child(n)
	Build.slab(n, 0, 0, 0.42, 0.42, 0.44, 0.04, m)
	for sx in [-0.17, 0.17]:
		for sz in [-0.17, 0.17]:
			Build.cyl(n, Vector3(sx, 0, sz), 0.018, 0.018, 0.44, m, 6)
	Build.slab(n, 0, -0.19, 0.40, 0.04, 0.48, 0.46, m)
	Build.box_blocker(n, Vector3(0, 0.45, 0), Vector3(0.44, 0.9, 0.44))


static func _table(g: Node3D, pos: Vector3, r: float, c: Color) -> void:
	var m := _wood(c)
	Build.cyl(g, pos + Vector3(0, 0.70, 0), r, r, 0.05, m, 16)
	Build.cyl(g, pos, 0.04, 0.04, 0.70, Mats.paint(Color(0.15, 0.15, 0.15), 0.4, 0.7), 10)
	Build.cyl(g, pos, 0.22, 0.24, 0.03, Mats.paint(Color(0.15, 0.15, 0.15), 0.4, 0.7), 12)
	Build.blocker(g, pos, 0.34, 0.75)


## A cafe: counter along one side, a back bar, loose tables, and the window
## seat that makes `03_cafe_frontage` work.
static func cafe(g: Node3D, w: float, depth: float, accent: Color, seed_v: int) -> void:
	var r := RandomNumberGenerator.new()
	r.seed = seed_v
	var warm := Color(1.0, 0.82, 0.58)
	_lights(g, depth, w, warm, 4)

	var counter_c := accent.lightened(0.10)
	var cx := w * 0.5 - 1.6
	# counter
	Build.slab(g, cx, depth * 0.62, 1.0, depth * 0.46, 0.0, 1.02, _wood(counter_c), 0.0, true)
	Build.slab(g, cx, depth * 0.62, 1.14, depth * 0.46 + 0.12, 1.02, 0.06,
		Mats.paint(Color(0.22, 0.21, 0.20), 0.30))
	# back bar and shelves
	Build.slab(g, w * 0.5 - WALL_T - 0.16, depth * 0.62, 0.30, depth * 0.5, 0.0, 2.1,
		_wood(counter_c.darkened(0.25)))
	for s in 3:
		var sy := 1.25 + s * 0.42
		Build.slab(g, w * 0.5 - WALL_T - 0.30, depth * 0.62, 0.26, depth * 0.44, sy, 0.03,
			_wood(counter_c))
		for i in 7:
			var z := depth * 0.62 - depth * 0.19 + i * depth * 0.062
			Build.cyl(g, Vector3(w * 0.5 - WALL_T - 0.30, sy + 0.03, z),
				0.035, 0.035, r.randf_range(0.10, 0.20),
				Mats.paint(Color(r.randf_range(0.4, 0.8), r.randf_range(0.3, 0.6),
					r.randf_range(0.2, 0.5)), 0.4), 8)
	# espresso machine, the one object everyone looks for
	Build.slab(g, cx, depth * 0.48, 0.78, 0.46, 1.08, 0.44,
		Mats.paint(Color(0.55, 0.52, 0.50), 0.25, 0.85))
	Build.cyl(g, Vector3(cx - 0.2, 1.52, depth * 0.48), 0.05, 0.05, 0.18,
		Mats.paint(Color(0.25, 0.24, 0.23), 0.3, 0.9), 8)
	# menu board over the counter
	Build.slab(g, w * 0.5 - WALL_T - 0.34, depth * 0.62, 0.05, 1.7, 2.25, 0.8,
		Mats.paint(Color(0.14, 0.15, 0.14), 0.85))
	Build.label(g, Vector3(w * 0.5 - WALL_T - 0.40, 2.80, depth * 0.62), "TODAY", 0.22,
		Color(0.92, 0.90, 0.84), -PI * 0.5, 48)

	# loose tables down the open side
	var tx := -w * 0.5 + 1.9
	for i in 3:
		var z := 2.6 + i * 3.0
		if z > depth - 1.6:
			break
		_table(g, Vector3(tx, 0, z), 0.42, Color(0.42, 0.30, 0.21))
		_chair(g, Vector3(tx - 0.72, 0, z), PI * 0.5, Color(0.34, 0.26, 0.20))
		_chair(g, Vector3(tx + 0.72, 0, z), -PI * 0.5, Color(0.34, 0.26, 0.20))

	# window bench, facing the street
	Build.slab(g, 0, 0.95, w - 2.4, 0.42, 0.44, 0.05, _wood(Color(0.40, 0.29, 0.20)))
	Build.slab(g, 0, 0.95, w - 2.4, 0.10, 0.0, 0.44, _wood(Color(0.33, 0.24, 0.17)))

	# a rug, because a bare plank floor reads as unfinished
	Build.ground(g, Vector3(tx, 0.10, 5.4), 2.6, 5.0,
		Mats.paint(Color(0.38, 0.29, 0.26), 0.95), 0, false)


## A grocer: aisles you can walk between, a counter by the door, produce at the
## window.
static func shop(g: Node3D, w: float, depth: float, accent: Color, seed_v: int) -> void:
	var r := RandomNumberGenerator.new()
	r.seed = seed_v
	_lights(g, depth, w, Color(1.0, 0.93, 0.82), 4)

	var shelf_c := Color(0.52, 0.50, 0.47)
	# two gondola aisles down the middle, leaving a walkable gap
	for a in 2:
		var ax: float = -1.7 + a * 3.4
		for s in 4:
			var sy := 0.35 + s * 0.52
			Build.slab(g, ax, depth * 0.55, 0.76, depth * 0.5, sy, 0.04,
				Mats.paint(shelf_c, 0.7))
			for i in 12:
				var z := depth * 0.55 - depth * 0.24 + i * depth * 0.043
				var c := Color(r.randf_range(0.25, 0.85), r.randf_range(0.25, 0.8),
					r.randf_range(0.2, 0.75))
				Build.slab(g, ax + r.randf_range(-0.2, 0.2), z, r.randf_range(0.10, 0.18),
					r.randf_range(0.10, 0.20), sy + 0.04, r.randf_range(0.12, 0.26),
					Mats.paint(c, 0.75))
		Build.slab(g, ax, depth * 0.55, 0.08, depth * 0.5, 0.0, 2.1,
			Mats.paint(shelf_c.darkened(0.2), 0.7))
		Build.box_blocker(g, Vector3(ax, 1.05, depth * 0.55), Vector3(0.8, 2.1, depth * 0.5))

	# wall shelving down both sides
	for sx in [-1.0, 1.0]:
		for s in 4:
			var sy := 0.4 + s * 0.5
			Build.slab(g, sx * (w * 0.5 - WALL_T - 0.22), depth * 0.6, 0.40, depth * 0.55,
				sy, 0.04, Mats.paint(shelf_c, 0.7))
			for i in 10:
				var z := depth * 0.6 - depth * 0.26 + i * depth * 0.055
				Build.slab(g, sx * (w * 0.5 - WALL_T - 0.22), z, 0.16, 0.16, sy + 0.04,
					r.randf_range(0.14, 0.24),
					Mats.paint(Color(r.randf_range(0.3, 0.9), r.randf_range(0.3, 0.8),
						r.randf_range(0.25, 0.7)), 0.75))

	# checkout by the door
	Build.slab(g, w * 0.5 - 1.7, 2.2, 1.3, 0.7, 0.0, 0.95,
		Mats.paint(Color(0.58, 0.55, 0.50), 0.6), 0.0, true)
	Build.slab(g, w * 0.5 - 1.7, 2.2, 1.42, 0.82, 0.95, 0.05,
		Mats.paint(Color(0.26, 0.25, 0.24), 0.3))
	Build.slab(g, w * 0.5 - 1.45, 2.2, 0.32, 0.30, 1.0, 0.24,
		Mats.paint(Color(0.16, 0.17, 0.18), 0.35, 0.4))

	# produce crates in the window, angled to the street
	for i in 3:
		var bx := -w * 0.5 + 1.5 + i * 1.25
		Build.slab(g, bx, 1.35, 1.05, 0.72, 0.0, 0.62,
			_wood(Color(0.48, 0.36, 0.24)), 0.12)
		for k in 9:
			Build.sphere(g, Vector3(bx + r.randf_range(-0.36, 0.36), 0.70,
				1.35 + r.randf_range(-0.22, 0.22)), r.randf_range(0.05, 0.075),
				Mats.paint(Color(r.randf_range(0.4, 0.9), r.randf_range(0.3, 0.7),
					r.randf_range(0.1, 0.35)), 0.55), 8)
		Build.box_blocker(g, Vector3(bx, 0.31, 1.35), Vector3(1.05, 0.62, 0.72))
