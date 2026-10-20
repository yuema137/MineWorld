## The slice, assembled: which buildings stand where, and what each one is.
##
## Everything here is the scope `docs/VISUAL_SLICE.md` sec.4 fixes, and nothing
## beyond it. One cafe, five other north-side facades, four lower south-side
## ones, street furniture, planting, two ends, and the far field the Presentation
## Pack declares (RL-e0). Adding a second
## street or a second interior here is what would make the Godot and Unreal
## slices stop comparing, so the list is short on purpose.
##
## The semantic side of the frontage lives here too. Each building carries the
## `Place` identity the authoritative world knows it by, so that entering the
## cafe changes *where the player is in the world* and not only where they are
## on the ground (`VISUAL_SLICE.md` sec.9). The client never decides what a
## Place means; it only knows which volume corresponds to which id.
class_name SliceWorld
extends RefCounted

## The slice's place volumes: where a body is in THIS scene. Which of the
## world's places each depicts is learned from the server (`SliceLayout`), never
## listed: the café room depicts the place tagged ANCHOR_TAG, the street volume
## the place its door opens onto, and the florist the place whose disclosed
## doorway lands on The Flower Room's door (after S15 12d, the world's store).
const CAFE_PLACE := "cafe.main"
const STREET_PLACE := "street.main"
## The second enterable building (`VISUAL_SLICE.md` sec.4.1).
const FLORIST_PLACE := "florist.main"
## The one depiction the scene declares (step-15 D-16c-3): the café room is the
## world's place with this tag, as a presentation selects a place's art by its
## tag. No other place key or tag is named by this client.
const ANCHOR_TAG := "cafe"

## Where the player starts: on the pavement, west of the cafe, facing east
## along the street so the first frame is the approach.
const SPAWN := Vector3(-11.5, 0.45, -5.55)
const SPAWN_YAW := -78.0

## The far field's fixed viewpoints on this street (RL-e0 sec.6.1, RL-e0b sec.6.1): [id, scene
## position, roles]. Roles: "protected" -- a reachable end or the spawn, whose sightlines to the far
## field the ring's roofs keep clear (RL-e0b SD-5) and from which the bay must stay visible
## (FF-BAY); "judged" -- also rendered in ID colours for the position, order, seam and texture
## criteria; "high" -- the review's elevated eye, also rendered looking down at the ring (FF-RING).
## They belong to the street, not to the check: another street names its own.
const FAR_FIELD_VIEWPOINTS := [
	["VP-spawn", Vector3(-11.5, 1.80, -5.55), ["protected"]],
	["VP-street", Vector3(0.0, 1.66, 0.0), []],
	["VP-cafe", Vector3(3.45, 1.80, -5.60), []],
	["VP-florist", Vector3(11.84, 1.80, -5.60), []],
	["VP-west", Vector3(-33.5, 1.80, 0.0), ["protected", "judged"]],
	["VP-east", Vector3(33.5, 1.66, 0.0), ["protected", "judged"]],
	["VP-high", Vector3(0.0, 40.0, 0.0), ["judged", "high"]],
]

## The far field's regression views (RL-e0 sec.7, RL-e0b sec.7): [name, position, yaw (0 = north,
## -90 = east), pitch].
const FAR_FIELD_SHOTS := [
	["ff_1_north_street", Vector3(0.0, 1.8, -1.0), 0.0, 8.0],
	["ff_2_north_aerial", Vector3(0.0, 40.0, 25.0), 0.0, -4.0],
	["ff_3_south_aerial", Vector3(0.0, 40.0, -25.0), 180.0, -4.0],
	["ff_4_east_aerial", Vector3(-30.0, 40.0, 0.0), -90.0, -4.0],
	["ff_5_west_aerial", Vector3(30.0, 40.0, 0.0), 90.0, -4.0],
	["ff_6_west_street_end", Vector3(-24.0, 1.8, 0.0), 90.0, 2.0],
	["ff_7_east_street_end", Vector3(24.0, 1.8, 0.0), -90.0, 2.0],
	["ff_8_southwest_aerial", Vector3(0.0, 40.0, 0.0), 135.0, -4.0],
	["ff_9_ring_aerial", Vector3(0.0, 40.0, 0.0), 0.0, -27.0],
]


## The viewpoints with `role`, as [id, position].
static func far_field_viewpoints(role: String) -> Array:
	var out := []
	for vp in FAR_FIELD_VIEWPOINTS:
		if role == "" or role in vp[2]:
			out.append([vp[0], vp[1]])
	return out


## The protected eyes' scene positions (RL-e0b SD-5).
static func protected_eyes() -> PackedVector3Array:
	var out := PackedVector3Array()
	for vp in far_field_viewpoints("protected"):
		out.append(vp[1])
	return out


## `connected`: the slice draws a world's people (`SliceLink`), so the
## streetscape's decorative townspeople are not built (D-16c-6). Standalone
## builds exactly as before.
static func build(parent: Node3D, far_field: MineWorldFarField, connected := false) -> Node3D:
	var g := Node3D.new()
	g.name = "Slice"
	parent.add_child(g)

	var t0 := Time.get_ticks_msec()
	SliceStreet.build(g);                                        _tick("street", t0)
	SliceFarFieldView.build(g, far_field, protected_eyes());     _tick("far field", t0)
	_north_frontage(g);                                          _tick("north", t0)
	_south_frontage(g);                                          _tick("south", t0)
	SliceCafe.build(g, Vector3(6.0, SliceStreet.WALK_Y, SliceStreet.NORTH_FACE))
	_tick("cafe", t0)
	SliceStreetscape.build(g, not connected);                    _tick("streetscape", t0)
	_place_volumes(g)
	# Static batching, after everything is placed and before anything is drawn.
	# See batch.gd: the slice is thousands of small boxes, and a frame that
	# issues one draw call per moulding is not a frame anyone will wait for.
	var m := SliceBatch.merge(g, SliceTrial.cell_size())
	print("batch  %d primitive instances -> %d meshes, %d triangles"
		% [m["merged_from"], m["merged_into"], m["triangles"]])
	print("props  %d imported, %d fallback" % [Props.imported, Props.fallback])
	var b := SliceBudget.apply(g)
	print("budget %d instances ranged by screen size; widest at its range end %.2f px; %d over"
		% [b["ranged"], b["max_px"], b["violations"]])
	var o := SliceOccluders.build(g)
	print("occluders %d boxes from opaque massing; %d overlap glazing or a door"
		% [o["occluders"], o["overlaps"]])
	_tick("batched", t0)
	return g


## Build timing, printed on every run. A scene that takes a minute to assemble
## is a defect the operator would meet as "it does not start".
static func _tick(stage: String, t0: int) -> void:
	print("build %-14s %6d ms" % [stage, Time.get_ticks_msec() - t0])


## The five secondary facades on the cafe's side. Each differs in storey count,
## roof, material and shopfront treatment, which `VISUAL_SLICE.md` sec.4 makes a
## requirement rather than a nicety.
static func _north_frontage(g: Node3D) -> void:
	var books := SliceTerrace.Unit.new(-34.0, -25.5)
	books.storeys = 3
	books.wall = SlicePalette.brick_red()
	books.roof = SliceTerrace.Roof.PARAPET
	books.shopfront = true
	books.name_text = "RIVERSTONE BOOKS"
	books.strap = "BOOKS . IDEAS . PEOPLE"
	books.front_c = SlicePalette.BOOKS_GREEN
	books.bays = 3
	SliceTerrace.build(g, books, SliceStreet.NORTH_FACE, 0.0)

	var bakery := SliceTerrace.Unit.new(-25.5, -17.5)
	bakery.storeys = 2
	bakery.wall = SlicePalette.render_cream()
	bakery.roof = SliceTerrace.Roof.PITCHED
	bakery.roof_mat = SlicePalette.pantile()
	bakery.shopfront = true
	bakery.name_text = "SUNRISE BAKERY"
	bakery.strap = "FRESH BREAD . HAPPIER DAYS"
	bakery.front_c = Color(0.760, 0.702, 0.588)
	bakery.awning = true
	bakery.stripes = true
	bakery.awning_c = SlicePalette.AWNING_BLUE
	bakery.bays = 3
	SliceTerrace.build(g, bakery, SliceStreet.NORTH_FACE, 0.0)

	var flats := SliceTerrace.Unit.new(-17.5, -6.5)
	flats.storeys = 4
	flats.storey_h = 3.20
	flats.wall = SlicePalette.brick_buff()
	flats.roof = SliceTerrace.Roof.PARAPET
	flats.shopfront = false
	flats.front_c = Color(0.226, 0.254, 0.238)
	flats.bays = 3
	flats.depth = 11.0
	SliceTerrace.build(g, flats, SliceStreet.NORTH_FACE, 0.0)

	var maple := SliceTerrace.Unit.new(-6.5, 1.5)
	maple.storeys = 2
	maple.wall = SlicePalette.limestone()
	maple.roof = SliceTerrace.Roof.PITCHED
	maple.roof_mat = SlicePalette.slate_roof()
	maple.shopfront = true
	maple.name_text = "MAPLE & CO."
	maple.strap = "BOOKS . GIFTS . LOCAL"
	maple.front_c = Color(0.184, 0.286, 0.252)
	maple.awning = false
	maple.bays = 2
	SliceTerrace.build(g, maple, SliceStreet.NORTH_FACE, 0.0)

	# east of the cafe, a gable end turned to the street -- the roofline step
	# that stops the frontage reading as one continuous parapet
	var florist := SliceTerrace.Unit.new(10.5, 19.0)
	florist.storeys = 2
	florist.storey_h = 3.10
	florist.wall = SlicePalette.limestone()
	florist.roof = SliceTerrace.Roof.GABLE
	florist.roof_mat = SlicePalette.pantile()
	florist.shopfront = true
	florist.name_text = "THE FLOWER ROOM"
	florist.strap = "GROWN NEARBY"
	florist.front_c = Color(0.336, 0.244, 0.288)
	florist.awning = true
	florist.awning_c = Color(0.344, 0.268, 0.296)
	florist.bays = 2
	florist.depth = 8.0
	# VISUAL_SLICE.md sec.4.1: the second enterable building. Connected, it is
	# the room of whichever place's disclosed doorway lands on its door
	# (`SliceLayout`); since S15 12d that is the world's store.
	florist.enterable = true
	florist.volume = FLORIST_PLACE
	SliceTerrace.build(g, florist, SliceStreet.NORTH_FACE, 0.0)

	# the frontage stops against a stone retaining wall and the planted bank
	Build.box(g, Vector3(24.0, SliceStreet.WALK_Y + 0.95, SliceStreet.NORTH_FACE + 1.1),
		Vector3(11.0, 1.90, 2.2), SlicePalette.kerbstone(), 0.0, true)
	Build.box(g, Vector3(24.0, SliceStreet.WALK_Y + 1.95, SliceStreet.NORTH_FACE + 1.1),
		Vector3(11.2, 0.12, 2.4), SlicePalette.kerbstone())


## The opposite side: two storeys, lower detail, and deliberately lower so the
## sun reaches the north pavement. See the note in `street.gd`.
static func _south_frontage(g: Node3D) -> void:
	var specs := [
		[-32.0, -21.0, SliceTerrace.Roof.PITCHED, false, "", 2],
		[-21.0, -11.0, SliceTerrace.Roof.PARAPET, true, "EVERYDAY MART", 3],
		[-11.0, -1.0, SliceTerrace.Roof.PITCHED, false, "", 3],
		[-1.0, 9.5, SliceTerrace.Roof.PARAPET, true, "LAKESIDE DELI", 3],
		[9.5, 20.0, SliceTerrace.Roof.PITCHED, false, "", 3],
		[20.0, 31.0, SliceTerrace.Roof.PITCHED, false, "", 2],
	]
	var walls := [SlicePalette.render_cream(), SlicePalette.brick_buff(),
		SlicePalette.limestone(), SlicePalette.brick_red()]
	for i in range(specs.size()):
		var s: Array = specs[i]
		var u := SliceTerrace.Unit.new(s[0], s[1])
		u.storeys = 2
		u.storey_h = 3.15
		u.wall = walls[i % walls.size()]
		u.roof = s[2]
		u.roof_mat = SlicePalette.pantile() if i % 2 == 0 else SlicePalette.slate_roof()
		u.shopfront = s[3]
		u.name_text = s[4]
		u.strap = "A BRIGHTER TOMORROW" if s[3] else ""
		u.front_c = Color(0.214, 0.258, 0.236) if i % 2 == 0 else Color(0.290, 0.226, 0.192)
		u.bays = s[5]
		u.depth = 8.5
		SliceTerrace.build(g, u, SliceStreet.SOUTH_FACE, PI)


## Semantic `Place` volumes. An `Area3D` per place, tagged with the id the
## authoritative world uses. The client reports which volume the player is in;
## it never decides what that means, what is allowed there, or who else is
## present -- that is the server's, and `ADOPTION.md` sec.3.3 makes deciding it
## here a defect rather than an optimisation.
##
## Both volumes reach 0.5 m BELOW the pavement. A player's position is their
## feet, and a volume whose floor is exactly the walking surface puts a body
## standing on that surface on the boundary: the first run reported the player
## back on the pavement as being in no place at all.
static func _place_volumes(g: Node3D) -> void:
	var y := SliceStreet.WALK_Y + 1.25
	_place(g, CAFE_PLACE,
		Vector3(6.0, y, SliceStreet.NORTH_FACE - SliceCafe.DEPTH * 0.5),
		Vector3(SliceCafe.W - SliceCafe.WALL_T * 2.0, 3.5, SliceCafe.DEPTH - SliceCafe.WALL_T * 2.0))
	var fb := SliceShopInterior.room_box
	_place(g, FLORIST_PLACE, fb.position + fb.size * 0.5, fb.size)
	_place(g, STREET_PLACE, Vector3(0.0, y, 0.0),
		Vector3(SliceStreet.X_MAX - SliceStreet.X_MIN, 3.5,
			SliceStreet.SOUTH_FACE - SliceStreet.NORTH_FACE))


static func _place(g: Node3D, id: String, centre: Vector3, size: Vector3) -> void:
	var a := Area3D.new()
	a.name = "Place_" + id
	a.monitorable = false
	a.collision_layer = 0
	a.collision_mask = 0
	a.set_meta("place_id", id)
	var cs := CollisionShape3D.new()
	var b := BoxShape3D.new()
	b.size = size
	cs.shape = b
	a.add_child(cs)
	a.position = centre
	g.add_child(a)


## Which place a world-space point is in, innermost first. The cafe volume sits
## inside the street volume's x range, so order matters and is stated rather
## than relied on.
static func place_at(root: Node3D, p: Vector3) -> String:
	var best := ""
	var best_vol := INF
	for c in root.get_children():
		if not (c is Area3D) or not c.has_meta("place_id"):
			continue
		var a := c as Area3D
		var cs := a.get_child(0) as CollisionShape3D
		var box := (cs.shape as BoxShape3D).size
		var local := p - a.position
		if absf(local.x) <= box.x * 0.5 and absf(local.y) <= box.y * 0.5 \
				and absf(local.z) <= box.z * 0.5:
			var vol := box.x * box.y * box.z
			if vol < best_vol:
				best_vol = vol
				best = str(a.get_meta("place_id"))
	return best
