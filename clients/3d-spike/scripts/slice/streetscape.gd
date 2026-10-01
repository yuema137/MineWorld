## Street furniture, planting and people.
##
## `VISUAL_SLICE.md` sec.4 lists the minimum: lamps, a kerbside edge treatment,
## a bench, a litter bin, A-boards, planters, the cafe's own outdoor seating,
## street trees, planting with real foliage, a climbing or trailing plant, and
## at least two other figures. All of it is here, and the density is set from
## `05_main_street_golden_hour` and `01_lakeside_promenade` rather than from a
## feeling about how much is enough -- both plates are crowded with small
## objects along the kerb line, and an empty pavement is the clearest single
## tell that a street was modelled rather than observed.
class_name SliceStreetscape
extends RefCounted

const WALK := SliceStreet.WALK_Y


static func build(parent: Node3D) -> Node3D:
	var g := Node3D.new()
	g.name = "Streetscape"
	parent.add_child(g)

	_lamps(g)
	_kerbline(g)
	_trees(g)
	_cafe_terrace(g)
	_planting(g)
	_seating_and_bins(g)
	_signs(g)
	_bicycle(g, Vector3(12.4, WALK, -7.05), -0.22)
	_people(g)
	return g


# --- lighting the street -------------------------------------------------------

## Cast-iron columns with a hooded head, lit. `05` is a golden-hour plate and
## its lamps are on: a lamp that is visibly a lamp and visibly off reads as a
## prop, and the warm pool under it is one of the few things in the frame that
## is not the sun.
static func _lamps(g: Node3D) -> void:
	for x in [-24.0, -8.0, 9.0, 25.0]:
		_lamp(g, Vector3(x, WALK, -4.30), true)
	for x in [-17.0, 1.0, 18.0]:
		_lamp(g, Vector3(x, WALK, 4.30), false)


static func _lamp(g: Node3D, pos: Vector3, banner: bool) -> void:
	var n := Props.place(g, "street_lamp_01", pos, 0.0, Color(0.44, 0.45, 0.46))
	var box := SliceProps.aabb_of(n)
	var h := box.size.y
	Build.blocker(g, pos, 0.13, 2.4)
	var head := pos + Vector3(0, h * 0.94, 0)
	var l := OmniLight3D.new()
	l.light_color = Color(1.0, 0.80, 0.54)
	l.light_energy = 3.4
	l.omni_range = 9.0
	l.omni_attenuation = 1.3
	l.shadow_enabled = false
	l.position = head
	g.add_child(l)
	Build.sphere(g, head, 0.085, Mats.emissive(Color(1.0, 0.86, 0.62), 6.0), 8)
	if not banner:
		return
	# the banner `05` hangs from its columns, printed both sides
	var iron := SlicePalette.iron()
	for sy in [2.72, 3.94]:
		Build.box(g, pos + Vector3(0.26, sy, 0), Vector3(0.52, 0.045, 0.045), iron)
	var bx := pos + Vector3(0.50, 3.33, 0)
	Build.box(g, bx, Vector3(0.02, 1.16, 0.62),
		SlicePalette.canvas(Color(0.286, 0.336, 0.404)))
	for sz in [1.0, -1.0]:
		Profile.text(g, bx + Vector3(sz * 0.016, 0.0, 0.0),
			"A KINDER\nBRIGHTER\nMINEWORLD", 0.072, Color(0.90, 0.90, 0.88),
			SlicePalette.serif_font(), (PI * 0.5 if sz > 0.0 else -PI * 0.5))


# --- the kerb line -------------------------------------------------------------

## Bollards and a litter bin along the kerb. `05` has fluted cast-iron bollards
## between the paving and the carriageway, and they are the object that most
## establishes the kerb as an edge rather than a texture change.
static func _kerbline(g: Node3D) -> void:
	var iron := SlicePalette.iron(0.48)
	var x := -2.0
	while x < 21.0:
		_bollard(g, Vector3(x, WALK, -3.92), iron)
		x += 2.35
	for bx in [-19.0, -16.6, 7.6]:
		_bollard(g, Vector3(bx, WALK, 3.92), iron)
	_litter_bin(g, Vector3(3.6, WALK, -4.05))
	_litter_bin(g, Vector3(-21.4, WALK, -4.05))


static func _bollard(g: Node3D, pos: Vector3, iron: Material) -> void:
	Build.cyl(g, pos, 0.105, 0.125, 0.78, iron, 12)
	Build.cyl(g, pos + Vector3(0, 0.78, 0), 0.085, 0.115, 0.10, iron, 12)
	Build.sphere(g, pos + Vector3(0, 0.90, 0), 0.085, iron, 10)
	Build.cyl(g, pos, 0.145, 0.160, 0.09, iron, 12)
	Build.blocker(g, pos, 0.16, 1.0)


static func _litter_bin(g: Node3D, pos: Vector3) -> void:
	var iron := SlicePalette.iron(0.54)
	Build.cyl(g, pos + Vector3(0, 0.16, 0), 0.21, 0.185, 0.60, iron, 14)
	Build.cyl(g, pos + Vector3(0, 0.76, 0), 0.235, 0.235, 0.05, iron, 14)
	Build.cyl(g, pos, 0.06, 0.06, 0.18, iron, 8)
	Build.cyl(g, pos, 0.14, 0.16, 0.05, iron, 10)
	Build.blocker(g, pos, 0.24, 0.9)


# --- vegetation ----------------------------------------------------------------

static func _trees(g: Node3D) -> void:
	# North pavement, clear of the cafe frontage so the shopfront stays readable
	var north := [-27.5, -20.5, -12.5, 20.5, 27.0]
	for i in range(north.size()):
		_street_tree(g, Vector3(north[i], WALK, -4.95), 7.4 + (i % 3) * 0.5, 3100 + i * 71)
	var south := [-29.0, -22.0, -13.0, -5.0, 3.5, 12.0, 21.0]
	for i in range(south.size()):
		_street_tree(g, Vector3(south[i], WALK, 4.95), 7.0 + (i % 4) * 0.6, 5200 + i * 53)
	# the planted bank closing the east end
	var rng := RandomNumberGenerator.new()
	rng.seed = 9001
	for i in range(22):
		var p := Vector3(rng.randf_range(30.0, 44.0), 1.8,
			rng.randf_range(-17.0, 2.0))
		if p.z > -2.0:
			p.y = 1.1
		Props.broadleaf(g, p, rng.randf_range(6.5, 10.5), 6100 + i * 37)
	for i in range(16):
		Props.conifer(g, Vector3(rng.randf_range(32.0, 50.0), 1.8,
			rng.randf_range(-26.0, -12.0)), rng.randf_range(8.0, 13.0), 7100 + i * 29)


## A street tree standing in a square tree pit with an iron grille, which is how
## both plates plant them -- a trunk rising straight out of paving reads wrong.
static func _street_tree(g: Node3D, pos: Vector3, h: float, seed_v: int) -> void:
	Build.box(g, Vector3(pos.x, WALK - 0.03, pos.z), Vector3(1.35, 0.06, 1.35),
		Mats.paint(Color(0.140, 0.112, 0.090), 0.98))
	var iron := SlicePalette.iron(0.55)
	for i in range(7):
		var f := -0.58 + i * 0.19
		Build.box(g, Vector3(pos.x + f, WALK + 0.005, pos.z), Vector3(0.05, 0.04, 1.32), iron)
		Build.box(g, Vector3(pos.x, WALK + 0.005, pos.z + f), Vector3(1.32, 0.04, 0.05), iron)
	Build.box(g, Vector3(pos.x, WALK - 0.02, pos.z), Vector3(1.50, 0.10, 1.50),
		SlicePalette.kerbstone())
	Props.broadleaf(g, pos, h, seed_v)
	# a tree guard: three hoops on four stakes
	for i in range(4):
		var a := TAU * float(i) / 4.0 + 0.4
		Build.cyl(g, pos + Vector3(cos(a) * 0.36, 0, sin(a) * 0.36), 0.022, 0.022, 1.35,
			iron, 6)


static func _planting(g: Node3D) -> void:
	# Stone planters along the kerb, deep enough to be furniture
	for x in [-6.2, 0.4, 14.6, 22.2]:
		_stone_planter(g, Vector3(x, WALK, -4.42), 1.55, 0.95)
	for x in [-24.5, -10.0, 6.5]:
		_stone_planter(g, Vector3(x, WALK, 4.42), 1.35, 0.90)
	# the cafe's own pots, either side of its door
	for p in [Vector3(1.05, WALK, -7.35), Vector3(11.10, WALK, -7.35)]:
		SliceProps.put_solid(g, "planter_box_01", p, 0.2, Color(0.96, 0.94, 0.90))
		# one shrub out of the file's four, by name -- see dressing.gd
		SliceProps.put(g, "shrub_02", p + Vector3(0, 0.44, 0), 0.7,
			Color(0.90, 1.00, 0.84), 0.28, 0.86, ["_d"])
	# Flanking the door, never in it: the door opening is x 2.96..3.94, and the
	# first version stood a pot at x 3.55, which the drive found as the thing
	# the body stopped against 1.3 m short of the threshold.
	for p in [Vector3(2.30, WALK, -7.62), Vector3(4.62, WALK, -7.62),
			Vector3(8.70, WALK, -7.25)]:
		SliceProps.put_solid(g, "planter_pot_clay", p, -0.4, Color(0.94, 0.90, 0.86))
		SliceProps.put(g, "shrub_03", p + Vector3(0, 0.22, 0), 1.4,
			Color(0.92, 1.00, 0.86), 0.28, 0.86, ["_a"])


static func _stone_planter(g: Node3D, pos: Vector3, w: float, d: float) -> void:
	var stone := SlicePalette.kerbstone()
	Build.box(g, Vector3(pos.x, WALK + 0.30, pos.z), Vector3(w, 0.60, d), stone, 0.0, true)
	Build.box(g, Vector3(pos.x, WALK + 0.63, pos.z), Vector3(w + 0.10, 0.07, d + 0.10), stone)
	Build.box(g, Vector3(pos.x, WALK + 0.58, pos.z), Vector3(w - 0.16, 0.06, d - 0.16),
		Mats.paint(Color(0.130, 0.100, 0.076), 0.98))
	var seed_v := int(absf(pos.x) * 131.0) + 7
	Props.flowerbed(g, Vector3(pos.x, WALK + 0.70, pos.z), w - 0.22, d - 0.22, seed_v)
	Props.grass_patch(g, Vector3(pos.x, WALK + 0.62, pos.z), w - 0.26, d - 0.26,
		seed_v + 3, 4.0)
	SliceProps.put(g, "shrub_02", Vector3(pos.x - w * 0.22, WALK + 0.62, pos.z), 0.9,
		Color(0.88, 0.98, 0.82), 0.28, 0.86, ["_b"])


# --- the cafe's pavement, and other seating ------------------------------------

## `03` puts three bistro tables under the window with timber-slat chairs on
## black metal frames, a potted plant on one table, and the A-board beside them.
static func _cafe_terrace(g: Node3D) -> void:
	var warm := Color(1.04, 0.96, 0.86)
	for i in range(3):
		var x := 4.60 + i * 2.05
		Props.place(g, "outdoor_table_chair_set_01", Vector3(x, WALK, -6.30),
			1.55 + i * 0.22, warm)
		Build.blocker(g, Vector3(x, WALK, -6.30), 0.62, 0.85)
	SliceProps.put(g, "potted_plant_02", Vector3(5.72, WALK + 0.735, -6.30), 0.4,
		Color(0.92, 1.02, 0.88))
	SliceProps.put(g, "tea_set_01", Vector3(7.80, WALK + 0.735, -6.22), 1.1,
		Color.WHITE, 0.28, 0.86, ["_saucer_circular_03"])
	SliceProps.put(g, "tea_set_01", Vector3(7.80, WALK + 0.751, -6.22), 1.7,
		Color.WHITE, 0.28, 0.86, ["_cup_small_01"])
	# the A-boards
	SliceProps.put_solid(g, "standing_chalkboard_01", Vector3(2.35, WALK, -6.55), 0.55,
		Color(0.92, 0.90, 0.86))
	SliceProps.put_solid(g, "standing_chalkboard_01", Vector3(-27.2, WALK, -6.30), -0.42,
		Color(0.92, 0.90, 0.86))


static func _seating_and_bins(g: Node3D) -> void:
	var warm := Color(1.02, 0.96, 0.88)
	Props.place(g, "painted_wooden_bench", Vector3(-15.0, WALK, -5.10), 0.0, warm)
	Build.box_blocker(g, Vector3(-15.0, WALK + 0.42, -5.10), Vector3(1.9, 0.84, 0.7))
	Props.place(g, "painted_wooden_bench", Vector3(-3.0, WALK, 5.10), PI, warm)
	Build.box_blocker(g, Vector3(-3.0, WALK + 0.42, 5.10), Vector3(1.9, 0.84, 0.7))


## The fingerpost from `01` and `02`, and the street-name plate from `05`.
static func _signs(g: Node3D) -> void:
	var post := Vector3(-0.9, WALK, -4.45)
	var timber := SlicePalette.painted(Color(0.316, 0.236, 0.164), 0.80)
	Build.box(g, post + Vector3(0, 1.30, 0), Vector3(0.14, 2.60, 0.14), timber)
	Build.prism(g, post + Vector3(0, 2.68, 0), Vector3(0.20, 0.16, 0.20), timber)
	Build.blocker(g, post, 0.14, 2.2)
	var labels := ["Lakeside", "Town Square", "Market", "Trails"]
	for i in range(labels.size()):
		var y := 2.36 - i * 0.30
		var dir := 1.0 if i % 2 == 0 else -1.0
		var c := post + Vector3(dir * 0.52, y, 0.0)
		Build.box(g, c, Vector3(0.92, 0.22, 0.035), timber)
		Build.prism(g, c + Vector3(dir * 0.53, 0, 0), Vector3(0.22, 0.22, 0.035), timber,
			(PI * 0.5 if dir > 0.0 else -PI * 0.5))
		for sz in [1.0, -1.0]:
			Profile.text(g, c + Vector3(0, 0, sz * 0.026), labels[i], 0.082,
				Color(0.94, 0.92, 0.88), SlicePalette.serif_font(),
				0.0 if sz > 0.0 else PI)


## A bicycle leaning against the kerb, as `05` has at its right edge. Built from
## primitives: no CC0 library in `DEP-8`'s approved set ships one.
static func _bicycle(g: Node3D, pos: Vector3, yaw: float) -> void:
	var n := Node3D.new()
	n.transform = Transform3D(Basis(Vector3.UP, yaw) * Basis(Vector3.FORWARD, 0.20), pos)
	g.add_child(n)
	var tyre := Mats.paint(Color(0.075, 0.075, 0.078), 0.82)
	var frame := Mats.paint(Color(0.176, 0.212, 0.240), 0.34, 0.55)
	var steel := Mats.paint(Color(0.62, 0.63, 0.62), 0.28, 0.85)
	for sx in [-0.54, 0.54]:
		Profile.disc(n, Vector3(sx, 0.345, 0), 0.345, 0.035, tyre, 22)
		Profile.disc(n, Vector3(sx, 0.345, 0), 0.30, 0.018, Mats.paint(Color(0.10, 0.10, 0.10), 0.9), 22)
		for i in range(10):
			var a := TAU * float(i) / 10.0
			var s := Build.cyl(n, Vector3.ZERO, 0.006, 0.006, 0.58, steel, 4)
			s.transform = Transform3D(Basis(Vector3.FORWARD, a), Vector3(sx, 0.345, 0))
		Profile.disc(n, Vector3(sx, 0.345, 0), 0.055, 0.05, steel, 10)
	for seg in [[Vector3(-0.54, 0.345, 0), Vector3(-0.06, 0.74, 0)],
			[Vector3(-0.06, 0.74, 0), Vector3(0.36, 0.98, 0)],
			[Vector3(-0.06, 0.74, 0), Vector3(0.10, 0.31, 0)],
			[Vector3(0.10, 0.31, 0), Vector3(-0.54, 0.345, 0)],
			[Vector3(0.10, 0.31, 0), Vector3(0.36, 0.98, 0)],
			[Vector3(0.36, 0.98, 0), Vector3(0.54, 0.345, 0)]]:
		var a: Vector3 = seg[0]
		var b: Vector3 = seg[1]
		var mi := Build.cyl(n, Vector3.ZERO, 0.019, 0.019, a.distance_to(b), frame, 7)
		# local frame: look_at_from_position would read these as global
		# positions and draw the frame on the road at the world origin
		mi.transform = Transform3D(Basis.looking_at(b - a, Vector3.UP), (a + b) * 0.5)
		mi.rotate_object_local(Vector3.RIGHT, PI * 0.5)
	Build.box(n, Vector3(-0.06, 0.80, 0), Vector3(0.22, 0.05, 0.10), tyre)     # saddle
	Build.box(n, Vector3(0.40, 1.02, 0), Vector3(0.05, 0.05, 0.44), frame)     # bars
	Build.blocker(g, pos, 0.5, 1.1)


## Two figures on the pavement, so the street is inhabited. Static, deliberately:
## a walking crowd is a simulation question and this slice is a presentation one.
static func _people(g: Node3D) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 40711
	var a := NPC.make(rng, NPC.Pose.STAND, 1.74, false)
	a.transform = Transform3D(Basis(Vector3.UP, 2.1), Vector3(-9.4, WALK, -5.60))
	g.add_child(a)
	var b := NPC.make(rng, NPC.Pose.STAND, 1.68, false)
	b.transform = Transform3D(Basis(Vector3.UP, -0.6), Vector3(-8.5, WALK, -5.95))
	g.add_child(b)
	var c := NPC.make(rng, NPC.Pose.SIT, 1.72, false)
	c.transform = Transform3D(Basis(Vector3.UP, -1.55), Vector3(6.66, WALK, -6.28))
	g.add_child(c)
	var d := NPC.make(rng, NPC.Pose.STAND, 1.76, false)
	d.transform = Transform3D(Basis(Vector3.UP, 1.3), Vector3(19.8, WALK, 5.2))
	g.add_child(d)
