## The ground: carriageway, kerbs, pavements, crossing, verges and backdrop.
##
## THE FRAME, and every other slice file depends on it:
##
##     +x   along the street, west to east. The slice runs x = -34 .. +34.
##     +z   south, across the street. z = 0 is the carriageway centreline.
##     +y   up. y = 0 is the carriageway surface.
##
##     z = -8.1   north facade line   -- the cafe and the shops
##     z = -3.6   north kerb face
##     z =  0     centreline
##     z = +3.6   south kerb face
##     z = +8.1   south facade line   -- the lower opposite terrace
##
##  frontage to frontage 16.2 m, carriageway 7.2 m, both pavements 4.5 m --
##  all inside VISUAL_SLICE.md sec.3.3, and `--slice-measure` checks it rather
##  than this comment being trusted.
##
## WHY THE SOUTH SIDE IS LOWER. The sun is 19 degrees up and roughly west-south-
## west, which is what puts the long raking shadows of `05` on the paving. At
## that elevation a three-storey terrace on the south side would lay its own
## shadow across the whole carriageway and both pavements, and the slice would
## be a golden-hour scene with no golden hour in it -- the exact defect ARC-13
## records under the name `21_street_golden`. Two storeys on the south side
## lands that shadow in the middle of the road, keeps the north pavement in
## sun, and lets the sun reach about 4.8 m into the cafe through its glazing.
## The massing is a lighting decision, not an arbitrary one.
class_name SliceStreet
extends RefCounted

const X_MIN := -34.0
const X_MAX := 34.0
const HALF_ROAD := 3.60
const KERB_H := 0.14
const NORTH_FACE := -8.10
const SOUTH_FACE := 8.10
## The pavement surface, everywhere.
const WALK_Y := KERB_H

## Where the zebra crossing sits, and how wide.
const CROSSING_X := -13.0
const CROSSING_W := 3.20


static func build(parent: Node3D) -> Node3D:
	var g := Node3D.new()
	g.name = "Street"
	parent.add_child(g)

	_carriageway(g)
	_pavement(g, NORTH_FACE, -HALF_ROAD, "north")
	_pavement(g, HALF_ROAD, SOUTH_FACE, "south")
	_crossing(g)
	_surrounds(g)
	return g


static func _carriageway(g: Node3D) -> void:
	var w := X_MAX - X_MIN
	# setts, not asphalt: 05's carriageway is the same grey stone as its
	# pavements, laid larger and darker (SlicePalette.road_setts)
	Build.box(g, Vector3((X_MIN + X_MAX) * 0.5, -0.06, 0.0),
		Vector3(w, 0.12, HALF_ROAD * 2.0), SlicePalette.road_setts(), 0.0, true)
	# the gutter channel: one course of setts laid flat against each kerb, a
	# shade darker and slightly dished. It is the line that tells the eye where
	# the road stops, and 05 has it on both sides.
	for sz in [-1.0, 1.0]:
		Build.box(g, Vector3((X_MIN + X_MAX) * 0.5, -0.012, sz * (HALF_ROAD - 0.22)),
			Vector3(w, 0.10, 0.44), SlicePalette.setts_grey())


static func _pavement(g: Node3D, z_far: float, z_near: float, nm: String) -> void:
	# z_near is the kerb side, z_far the building side; north has z_far < z_near.
	var z0 := minf(z_far, z_near)
	var z1 := maxf(z_far, z_near)
	var kerb_z := z_near if absf(z_near) == HALF_ROAD else z_far
	var w := X_MAX - X_MIN
	var cx := (X_MIN + X_MAX) * 0.5

	# the walking surface
	Build.box(g, Vector3(cx, KERB_H - 0.09, (z0 + z1) * 0.5),
		Vector3(w, 0.18, z1 - z0), SlicePalette.setts(), 0.0, true)

	# the kerbstone itself: a separate, greyer, smoother stone standing 0.14 m
	# proud, with its top face flush with the pavement
	var sgn := signf(kerb_z)
	Build.box(g, Vector3(cx, KERB_H - 0.11, kerb_z - sgn * 0.14),
		Vector3(w, 0.30, 0.28), SlicePalette.kerbstone())

	# A hidden ramp so stepping up the kerb is a walk and not a wall. The visible
	# kerb keeps its square 0.14 m face; the collider is a 22-degree wedge over
	# 0.35 m, well inside the controller's 52-degree floor limit. Without it a
	# 0.30 m capsule catches on a square edge and the player has to hunt for the
	# crossing to get onto the pavement, which reads as a bug and is one.
	var ramp := StaticBody3D.new()
	ramp.collision_layer = Build.LAYER_WORLD
	var cs := CollisionShape3D.new()
	var box := BoxShape3D.new()
	box.size = Vector3(w, 0.42, 0.40)
	cs.shape = box
	ramp.add_child(cs)
	ramp.transform = Transform3D(Basis(Vector3.RIGHT, -sgn * deg_to_rad(21.0)),
		Vector3(cx, KERB_H - 0.20, kerb_z - sgn * 0.18))
	g.add_child(ramp)
	ramp.name = "KerbRamp_" + nm


static func _crossing(g: Node3D) -> void:
	# A zebra: seven bars across the carriageway, laid in the same pale stone as
	# the pavement rather than as painted white, which is how 05 reads.
	var mat := Mats.pbr("rectangular_paving", 1.90, Color(1.45, 1.42, 1.34), 0.80)
	for i in range(7):
		var x := CROSSING_X - CROSSING_W * 0.5 + 0.22 + i * 0.44
		Build.box(g, Vector3(x, 0.006, 0.0), Vector3(0.30, 0.02, HALF_ROAD * 1.85), mat)
	Props.place(g, "water_manhole_cover", Vector3(-4.2, 0.005, -1.7), 0.3,
		Color(0.72, 0.70, 0.68))


## Everything outside the street proper: the ground plane the town sits on, the
## ends of the frontage, and the backdrop that closes the view.
static func _surrounds(g: Node3D) -> void:
	var ground := SlicePalette.kerbstone()
	# a wide apron under everything so no frame ever shows the void
	Build.ground(g, Vector3(0, -0.14, 0), 900.0, 900.0,
		Mats.pbr("leafy_grass", 2.2, Color(0.50, 0.62, 0.38), 0.97), 0, true)

	# West end: the street meets a low stone parapet and rises out of sight.
	# A termination you can walk up to and see past, not an invisible wall.
	for sz in [-1.0, 1.0]:
		Build.box(g, Vector3(X_MIN - 1.0, 0.44, sz * 6.2), Vector3(3.0, 0.88, 5.0),
			ground, 0.0, true)
	Build.box(g, Vector3(X_MIN - 5.5, 0.30, 0.0), Vector3(9.0, 0.6, 19.0),
		SlicePalette.setts(), 0.0, true)

	# East end: the carriageway curves away behind a planted bank.
	Build.box(g, Vector3(X_MAX + 5.0, 0.9, -10.0), Vector3(12.0, 1.8, 14.0),
		Mats.pbr("leafy_grass", 2.2, Color(0.46, 0.58, 0.34), 0.97), 0.0, true)
	Build.box(g, Vector3(X_MAX + 8.0, 0.55, 6.0), Vector3(16.0, 1.1, 14.0),
		Mats.pbr("leafy_grass", 2.2, Color(0.46, 0.58, 0.34), 0.97), 0.0, true)
	# The far backdrop is `skyline.gd` (RL-b): real summits at real angles. The
	# sine-sum ridges that stood here were removed with it.
