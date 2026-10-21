## The ground: carriageway, kerbs, pavements, crossing, verges and the street's two ends.
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


## `site`: a station site the depiction declares (`StationDepot`), whose façade closes the west end; empty,
## the west end is the parapet it has always been, built exactly as before (S29-d I-d-8).
static func build(parent: Node3D, site: Dictionary = {}, towers_forward := 0.0) -> Node3D:
	var g := Node3D.new()
	g.name = "Street"
	parent.add_child(g)

	_carriageway(g)
	_pavement(g, NORTH_FACE, -HALF_ROAD, "north")
	_pavement(g, HALF_ROAD, SOUTH_FACE, "south")
	_crossing(g)
	if site.is_empty():
		_surrounds(g)
	else:
		_forecourt(g, float(site["facade_x"]), float(site["axis_z"]), towers_forward)
		_east_end(g)
	return g


static func _carriageway(g: Node3D) -> void:
	var w := X_MAX - X_MIN
	# setts, not asphalt: 05's carriageway is the same grey stone as its
	# pavements, laid larger and darker (SlicePalette.road_setts)
	Build.box(g, Vector3((X_MIN + X_MAX) * 0.5, -0.06, 0.0),
		Vector3(w, 0.12, HALF_ROAD * 2.0), SlicePalette.road_setts(), 0.0, true, Build.LAYER_WORLD)
	# the gutter channel: one course of setts laid flat against each kerb, a
	# shade darker and slightly dished. It is the line that tells the eye where
	# the road stops, and 05 has it on both sides.
	for sz in [-1.0, 1.0]:
		Build.box(g, Vector3((X_MIN + X_MAX) * 0.5, -0.012, sz * (HALF_ROAD - 0.22)),
			Vector3(w, 0.10, 0.44), SlicePalette.setts_grey(), 0.0, true)


static func _pavement(g: Node3D, z_far: float, z_near: float, nm: String) -> void:
	# z_near is the kerb side, z_far the building side; north has z_far < z_near.
	var z0 := minf(z_far, z_near)
	var z1 := maxf(z_far, z_near)
	var kerb_z := z_near if absf(z_near) == HALF_ROAD else z_far
	var w := X_MAX - X_MIN
	var cx := (X_MIN + X_MAX) * 0.5

	# the walking surface
	Build.box(g, Vector3(cx, KERB_H - 0.09, (z0 + z1) * 0.5),
		Vector3(w, 0.18, z1 - z0), SlicePalette.setts(), 0.0, true, Build.LAYER_WORLD)

	# the kerbstone itself: a separate, greyer, smoother stone standing 0.14 m
	# proud of the road (its top 40 mm above the pavement's), a drawn step with
	# its own collider, so a foot stands on what is drawn (PR S36-P4b). Walked
	# onto from the road: `Build.step` puts a walk aid in front of its road face,
	# a 21-degree slope up to the kerbstone's top edge on `LAYER_WALK_AID`, which
	# only the player's capsule meets. Without it a 0.25 m capsule catches on a
	# square edge and the player has to hunt for the crossing to get onto the
	# pavement, which reads as a bug and is one.
	var sgn := signf(kerb_z)
	Build.step(g, Vector3(cx, KERB_H - 0.11, kerb_z - sgn * 0.14),
		Vector3(w, 0.30, 0.28), SlicePalette.kerbstone(), -int(sgn), 0.0)


static func _crossing(g: Node3D) -> void:
	# A zebra: seven bars across the carriageway, laid in the same pale stone as
	# the pavement rather than as painted white, which is how 05 reads.
	var mat := Mats.pbr("rectangular_paving", 1.90, Color(1.45, 1.42, 1.34), 0.80)
	for i in range(7):
		var x := CROSSING_X - CROSSING_W * 0.5 + 0.22 + i * 0.44
		Build.box(g, Vector3(x, 0.006, 0.0), Vector3(0.30, 0.02, HALF_ROAD * 1.85), mat, 0.0, true,
			Build.LAYER_WORLD)
	# set flush: its declared rest plane (`contact.json`, the flange's top) on the road, not its
	# origin 5 mm up (S36-P5c)
	var cover := Props.place(g, "water_manhole_cover", Vector3(-4.2, 0.0, -1.7), 0.3,
		Color(0.72, 0.70, 0.68))
	SliceSettle.settle(g, cover, "water_manhole_cover")


## The ends of the frontage. The ground the town sits on, out to the boundary, and everything
## beyond it are the far field's (`far_field_view.gd`, RL-e0), drawn from the Presentation Pack.
static func _surrounds(g: Node3D) -> void:
	var ground := SlicePalette.kerbstone()

	# West end: the street meets a low stone parapet and rises out of sight.
	# A termination you can walk up to and see past, not an invisible wall.
	for sz in [-1.0, 1.0]:
		Build.box(g, Vector3(X_MIN - 1.0, 0.44, sz * 6.2), Vector3(3.0, 0.88, 5.0),
			ground, 0.0, true)
	Build.box(g, Vector3(X_MIN - 5.5, 0.30, 0.0), Vector3(9.0, 0.6, 19.0),
		SlicePalette.setts(), 0.0, true)

	_east_end(g)


## East end: the carriageway curves away behind a planted bank.
static func _east_end(g: Node3D) -> void:
	Build.box(g, Vector3(X_MAX + 5.0, 0.9, -10.0), Vector3(12.0, 1.8, 14.0),
		Mats.pbr("leafy_grass", 2.2, Color(0.46, 0.58, 0.34), 0.97), 0.0, true)
	Build.box(g, Vector3(X_MAX + 8.0, 0.55, 6.0), Vector3(16.0, 1.1, 14.0),
		Mats.pbr("leafy_grass", 2.2, Color(0.46, 0.58, 0.34), 0.97), 0.0, true)


## The station forecourt (S29-d §5.6): setts flush with the pavements from the street's west end to the
## station's façade at `facade_x`, over exactly the floor the world discloses (z -8.1 .. +8.1); the
## carriageway ends at a dropped kerb, so the forecourt reads as a plaza, not a road. Its edges are low
## stucco sitting walls, their inner faces on the disclosed edges, as far as the towers, which stand
## `towers_forward` in front of the façade and are the edge between them. Behind the walls, planted
## beds and gravel on ground nobody walks.
static func _forecourt(g: Node3D, facade_x: float, axis_z: float, towers_forward: float) -> void:
	var length := X_MIN - facade_x
	var cx := (X_MIN + facade_x) * 0.5
	Build.box(g, Vector3(cx, WALK_Y - 0.09, axis_z), Vector3(length, 0.18, SOUTH_FACE - NORTH_FACE),
		SlicePalette.setts(), 0.0, true)
	# the dropped kerb across the carriageway's end, with the street's hidden 21-degree ramp
	Build.box(g, Vector3(X_MIN + 0.14, KERB_H - 0.11, axis_z), Vector3(0.28, 0.30, HALF_ROAD * 2.0),
		SlicePalette.kerbstone())
	var ramp := StaticBody3D.new()
	ramp.collision_layer = Build.LAYER_WORLD
	var cs := CollisionShape3D.new()
	var box := BoxShape3D.new()
	box.size = Vector3(0.40, 0.42, HALF_ROAD * 2.0)
	cs.shape = box
	ramp.add_child(cs)
	ramp.transform = Transform3D(Basis(Vector3.BACK, -deg_to_rad(21.0)),
		Vector3(X_MIN + 0.18, KERB_H - 0.20, axis_z))
	g.add_child(ramp)
	ramp.name = "KerbRamp_forecourt"
	var wall_end := facade_x + towers_forward
	var stucco := StationDepot.stucco()
	for s: float in [-1.0, 1.0]:
		var edge: float = axis_z + (NORTH_FACE if s < 0.0 else SOUTH_FACE)
		var wx := (X_MIN + wall_end) * 0.5
		var wl := X_MIN - wall_end
		Build.box(g, Vector3(wx, WALK_Y + 0.075, edge + s * 0.20), Vector3(wl, 0.75, 0.40),
			stucco, 0.0, true)
		Build.box(g, Vector3(wx, WALK_Y + 0.48, edge + s * 0.20), Vector3(wl + 0.06, 0.06, 0.48),
			SlicePalette.kerbstone())
		# the beds and the gravel behind the wall, to the hall's ends
		Build.box(g, Vector3(cx, WALK_Y - 0.10, edge + s * 9.4), Vector3(length, 0.20, 18.0),
			Mats.pbr("cobblestone_floor_08", 1.20, Color(0.92, 0.86, 0.76), 0.95), 0.0, true)
		Props.flowerbed(g, Vector3(wx, WALK_Y, edge + s * 1.6), wl - 1.0, 1.8, 41 + int(s))
		for k in 2:
			var px := wall_end + (X_MIN - wall_end) * (0.3 + 0.4 * k)
			Props.broadleaf(g, Vector3(px, WALK_Y, edge + s * 3.4), 7.0, 17 + k + int(s * 3.0))
			Props.place(g, "street_lamp_01", Vector3(px + 3.0, WALK_Y, edge + s * 0.9), 0.0,
				Color(0.44, 0.45, 0.46))
