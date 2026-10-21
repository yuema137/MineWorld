extends RefCounted

## Social Café's street: the AUTHORING SOURCE of its place art (S14 16f-d1, 16f-d2; design
## `pr-16f-presentation-pack-3d.md` §4.5). `../export_place_art.gd` builds the pieces listed here and
## writes them as glTF assets, and the two depiction files, into the default 3D Presentation Pack.
## Nothing at run time reads this file: the 3D client draws the street from the pack.
##
## What is here is the street's composition (the frontage tables, the retaining wall, the café's
## placement, the halt's door), its viewpoints, and the builders under `builders/` (`street.gd`,
## `cafe.gd`, `cafe_interior.gd`, `shop_interior.gd`, `terrace.gd`, `streetscape.gd`, `profile.gd`,
## `occluders.gd`), moved unchanged out of the 3D client in 16f-d2 (finding F-d1-1). They are outside
## the Godot project, so they have no global class names: each is a preload constant of its old name.
## The client's shared library (`Build`, `Mats`, `Props`, `SlicePalette`, `SliceProps`, `SliceLayout`,
## `SliceDoors`) they call stays the client's; the tool runs with `--path clients/3d-spike`.
##
## The asset frame of every piece is Social Café's street frame: the scene point FRAME_ORIGIN, where the
## street place's (0, 0) stands, becomes the asset's (0, 0, 0). So a piece placed `at_mm: [0, 0]` stands
## where it stood when the client drew it, and the café's `front_door` entrance lands on the street's
## doorway (0, 2 800).

const SliceStreet := preload("builders/street.gd")
const SliceStreetscape := preload("builders/streetscape.gd")
const SliceTerrace := preload("builders/terrace.gd")
const SliceCafe := preload("builders/cafe.gd")
const SliceShopInterior := preload("builders/shop_interior.gd")
const Profile := preload("builders/profile.gd")
const SliceCafeInterior := preload("builders/cafe_interior.gd")

## The far field's fixed viewpoints on this street (RL-e0 sec.6.1, RL-e0b sec.6.1): [id, scene
## position, roles]. Roles: "protected" -- a reachable end or the spawn, whose sightlines to the far
## field the ring's roofs keep clear (RL-e0b SD-5) and from which the bay must stay visible
## (FF-BAY); "judged" -- also rendered in ID colours for the position, order, seam and texture
## criteria; "high" -- the review's elevated eye, also rendered looking down at the ring (FF-RING).
## (Moved from the client's `SliceWorld` in 16f-d2; written into the depiction files.)
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

## RL-b's perf views (M-1): [name, position, yaw, pitch]. Each is the pose of an existing shot or perf
## view, except `skyline east`, which looks east along the street above the eaves line so the backdrop
## is in frame. (Moved from the client's `SlicePerf` in 16f-d2.)
const PERF_VIEWS := [
	["street wide", Vector3(-20.0, 0.45, -5.20), -75.0, -1.0],
	["cafe frontage", Vector3(0.30, 0.45, -5.55), -52.0, 3.0],
	["interior", Vector3(2.75, 0.60, -10.60), -38.0, -3.0],
	["doorway", Vector3(3.45, 0.45, -6.90), 0.0, 0.0],
	["street east", Vector3(22.0, 0.45, -5.30), 96.0, -1.0],
	["south side", Vector3(2.0, 0.45, -4.60), 165.0, 1.0],
	["florist interior", Vector3(11.4, 0.45, -9.10), -38.0, -6.0],
	["skyline east", Vector3(-20.0, 0.45, -5.20), -90.0, 4.0],
]
## M-7's views.
const PERF_BREAKDOWN := ["street wide", "cafe frontage", "interior"]

## The street's frame in the slice's scene, where the street's (0, 0) stands.
const FRAME_ORIGIN := Vector3(3.45, 0.0, -4.90)
## The café's façade position.
const CAFE_AT := Vector3(6.0, SliceStreet.WALK_Y, SliceStreet.NORTH_FACE)
## The scene volumes of the two rooms (`SliceWorld`): names the doors' registry rows carry.
const CAFE_VOLUME := SliceCafe.VOLUME
const FLORIST_VOLUME := "florist.main"
## The halt's door (S29-a), as the social-cafe depiction draws it: the street side of the station hall's
## arch, mm in the street's frame, and the outward normal of the hall's east façade there.
const HALT_DOORWAY := Vector2(-62050.0, -4900.0)
const HALT_OUTWARD := Vector3(1.0, 0.0, 0.0)
const HALT_DOOR_W := 1.60
const HALT_DOOR_H := 2.80
## The offline spawn (`depictions/social-cafe/depiction.gd`).
const SPAWN := Vector3(-11.5, 0.45, -5.55)
const SPAWN_YAW := -78.0

## The florist's door out-point in the scene, kept when its piece is built (the store's doorway).
var _florist_out := Vector3.ZERO


## The pieces: {id, build (scene: Node3D), frame_origin, entrances {name: (door row) -> bool},
## spaces {name: () -> AABB}, volume_spaces {scene volume: space}}. A drawn door no entrance names is
## exported as `door_<nn>`.
func pieces() -> Array:
	return [
		{"id": "social-cafe-street", "build": _street, "frame_origin": FRAME_ORIGIN,
			"spaces": {"street": _street_box}},
		{"id": "social-cafe-north-frontage", "build": _north_frontage, "frame_origin": FRAME_ORIGIN},
		{"id": "social-cafe-florist", "build": _florist, "frame_origin": FRAME_ORIGIN,
			"entrances": {"front_door": func(d: Array) -> bool: return d[3] == FLORIST_VOLUME},
			"spaces": {"shop_room": func() -> AABB: return SliceShopInterior.room_box},
			"volume_spaces": {FLORIST_VOLUME: "shop_room"}},
		{"id": "social-cafe-south-frontage", "build": _south_frontage, "frame_origin": FRAME_ORIGIN},
		{"id": "social-cafe-cafe", "build": _cafe, "frame_origin": FRAME_ORIGIN,
			"entrances": {"front_door": func(d: Array) -> bool: return d[3] == CAFE_VOLUME},
			"spaces": {"cafe_room": _cafe_box}, "volume_spaces": {CAFE_VOLUME: "cafe_room"}},
		{"id": "social-cafe-halt", "build": _halt, "frame_origin": FRAME_ORIGIN,
			"entrances": {"front_door": func(d: Array) -> bool: return str(d[2]).begins_with("the halt")}},
	]


## `VISUAL_SLICE.md` sec.3's scale, for what this street's art declares (moved from the client's
## `--measure` in 16f-d2: the client no longer knows these numbers): [what, value, low, high], metres.
## The export prints one SCALE line each and refuses to write when one is out of range.
func scale_rows() -> Array:
	return [
		["cafe floor to ceiling", SliceCafe.CEIL_Y - SliceCafe.FLOOR_Y, 3.00, 3.60],
		["cafe door leaf width", SliceCafe.DOOR_W, 0.85, 1.00],
		["cafe door head height", SliceCafe.DOOR_H - SliceCafe.FLOOR_Y, 2.05, 2.20],
		["entrance step riser", SliceCafe.FLOOR_Y, 0.10, 0.18],
		["shopfront glazing head", SliceCafe.GLAZE_HEAD, 2.60, 3.00],
		["stallriser height", SliceCafe.STALL_H, 0.20, 0.45],
		["fascia band height", SliceCafe.FASCIA_H, 0.70, 1.10],
		["cafe frontage width", SliceCafe.W, 7.00, 11.00],
		["cafe interior depth", SliceCafe.DEPTH, 8.00, 13.00],
		["upper storey height", SliceCafe.STOREY, 3.00, 3.60],
		["north pavement width", -SliceStreet.HALF_ROAD - SliceStreet.NORTH_FACE, 3.00, 4.50],
		["south pavement width", SliceStreet.SOUTH_FACE - SliceStreet.HALF_ROAD, 3.00, 4.50],
		["carriageway width", SliceStreet.HALF_ROAD * 2.0, 6.00, 8.00],
		["frontage to frontage", SliceStreet.SOUTH_FACE - SliceStreet.NORTH_FACE, 14.0, 22.0],
		["kerb upstand", SliceStreet.KERB_H, 0.10, 0.16],
		["walkable street length", SliceStreet.X_MAX - SliceStreet.X_MIN, 45.0, 70.0],
		["counter working height", SliceCafeInterior.COUNTER_TOP, 1.00, 1.10],
	]


## The library roots the pieces may reference, and their licences (`clients/3d-spike/ASSETS.md`).
func licences() -> Array:
	return [
		{"root": "res://assets/textures/", "spdx": "CC0-1.0", "url": "https://polyhaven.com/license",
			"what": "Poly Haven textures, referenced by path from the 3D client's library (clients/3d-spike/ASSETS.md)"},
		{"root": "res://assets/models/", "spdx": "CC0-1.0", "url": "https://polyhaven.com/license",
			"what": "Poly Haven models and their textures, referenced by path from the 3D client's library (clients/3d-spike/ASSETS.md)"},
	]


## The licence of what MineWorld authored: the repository's.
func licence_spdx() -> String:
	return "MIT"


func style_family() -> String:
	return "mineworld-default-3d"


func authoring() -> String:
	return "tools/presentation/social_cafe/source.gd"


# --- the pieces --------------------------------------------------------------------------------------

func _street(g: Node3D) -> void:
	SliceStreet.build(g)
	# No decorative townspeople: people are the world's, drawn from the server (D-16c-6).
	SliceStreetscape.build(g, false)
	# the frontage stops against a stone retaining wall and the planted bank
	Build.box(g, Vector3(24.0, SliceStreet.WALK_Y + 0.95, SliceStreet.NORTH_FACE + 1.1),
		Vector3(11.0, 1.90, 2.2), SlicePalette.kerbstone(), 0.0, true)
	Build.box(g, Vector3(24.0, SliceStreet.WALK_Y + 1.95, SliceStreet.NORTH_FACE + 1.1),
		Vector3(11.2, 0.12, 2.4), SlicePalette.kerbstone())


func _street_box() -> AABB:
	var size := Vector3(SliceStreet.X_MAX - SliceStreet.X_MIN, 3.5,
		SliceStreet.SOUTH_FACE - SliceStreet.NORTH_FACE)
	return AABB(Vector3(0.0, SliceStreet.WALK_Y + 1.25, 0.0) - size * 0.5, size)


## The four secondary façades west of the café (`SliceWorld._north_frontage`, unchanged).
func _north_frontage(g: Node3D) -> void:
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


## East of the café, the second enterable building: a gable end turned to the street
## (`SliceWorld._north_frontage`'s last unit, unchanged).
func _florist(g: Node3D) -> void:
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
	florist.enterable = true
	florist.volume = FLORIST_VOLUME
	SliceTerrace.build(g, florist, SliceStreet.NORTH_FACE, 0.0)
	for d in SliceDoors.rows:
		if d[3] == FLORIST_VOLUME:
			_florist_out = SliceLayout.out_point(d)


## The opposite side (`SliceWorld._south_frontage`, unchanged).
func _south_frontage(g: Node3D) -> void:
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


## The café, without its furniture: furnishings are the world's (ARC-307), drawn from disclosure.
func _cafe(g: Node3D) -> void:
	SliceCafe.build(g, CAFE_AT)


func _cafe_box() -> AABB:
	var size := Vector3(SliceCafe.W - SliceCafe.WALL_T * 2.0, 3.5, SliceCafe.DEPTH - SliceCafe.WALL_T * 2.0)
	var centre := Vector3(CAFE_AT.x, SliceStreet.WALK_Y + 1.25, SliceStreet.NORTH_FACE - SliceCafe.DEPTH * 0.5)
	return AABB(centre - size * 0.5, size)


## The halt's closed door in a short façade at the street's west forecourt
## (`depictions/social-cafe/depiction.gd` `_build_halt_door`, unchanged).
func _halt(g: Node3D) -> void:
	var at := FRAME_ORIGIN + MineWorldSpace.to_3d({"x": HALT_DOORWAY.x, "y": HALT_DOORWAY.y})
	var sill := Vector3(at.x, SliceStreet.WALK_Y, at.z) - HALT_OUTWARD * SliceLayout.DOOR_INSET
	var yaw := atan2(HALT_OUTWARD.x, HALT_OUTWARD.z)
	var f := Node3D.new()
	f.name = "HaltDoor"
	g.add_child(f)
	f.position = sill
	f.rotation.y = yaw
	var stone := SlicePalette.limestone()
	var half := HALT_DOOR_W * 0.5
	var pier := 1.6
	var top := HALT_DOOR_H + 1.2
	for sx in [-1.0, 1.0]:
		Build.box(f, Vector3(sx * (half + pier * 0.5), top * 0.5, -0.20), Vector3(pier, top, 0.40), stone)
	Build.box(f, Vector3(0.0, HALT_DOOR_H + (top - HALT_DOOR_H) * 0.5, -0.20),
		Vector3(HALT_DOOR_W, top - HALT_DOOR_H, 0.40), stone)
	Profile.door(f, 0.0, 0.0, HALT_DOOR_W, HALT_DOOR_H, Color(0.30, 0.20, 0.12), false)
	SliceDoors.rows.append([sill, yaw, "the halt -- door", "", 0.0, 0.0])


# --- the depiction files (the pack's depictions/, 16f-d2) ----------------------------------------------

## The worlds drawn on this street: Social Café, and Market Town on the same geometry (step-15 D-16d-2).
func worlds() -> Array:
	return ["social-cafe", "market-town"]


## A world's depiction file (S33-d's format 1 plus 16f's `models` section), every number derived from
## the client's constants above, so no value is typed twice. Places bind by tag (ARC-144).
func depiction(world: String, _pack: String) -> Dictionary:
	var o := FRAME_ORIGIN
	var florist_box := SliceShopInterior.room_box
	return {
		"format": 1,
		"scope": {"world": world},
		"frame": {"origin_m": [_m(o.x), _m(o.y), _m(o.z)]},
		"street": {"tags": ["street"], "footprint_mm": _rect(_street_box())},
		"places": [
			{"tags": ["cafe"], "label": SliceCafe.NAME, "footprint_mm": _rect(_cafe_box()),
				"doorway_mm": _plan(CAFE_AT + Vector3(SliceCafe.DOOR_X, 0.0, SliceLayout.DOOR_INSET))},
			{"tags": ["store"], "label": "The Flower Room", "footprint_mm": _rect(florist_box),
				"doorway_mm": _florist_doorway()},
			{"tags": ["station"], "label": "The halt", "footprint_mm": _halt_footprint(),
				"doorway_mm": [int(HALT_DOORWAY.x), int(HALT_DOORWAY.y)]},
		],
		"spawn": {"at_mm": _plan(SPAWN), "up_m": _m(SPAWN.y), "yaw_deg": SPAWN_YAW},
		"models": [
			{"asset": "social-cafe-street", "depicts": null, "anchor": {"at_mm": [0, 0], "yaw_deg": 0}},
			{"asset": "social-cafe-north-frontage", "depicts": null, "anchor": {"at_mm": [0, 0], "yaw_deg": 0}},
			{"asset": "social-cafe-south-frontage", "depicts": null, "anchor": {"at_mm": [0, 0], "yaw_deg": 0}},
			{"asset": "social-cafe-cafe", "depicts": ["cafe"],
				"anchor": {"entrance": "front_door", "at_doorway": true}},
			{"asset": "social-cafe-florist", "depicts": ["store"],
				"anchor": {"entrance": "front_door", "at_doorway": true}},
			{"asset": "social-cafe-halt", "depicts": ["station"],
				"anchor": {"entrance": "front_door", "at_doorway": true}},
		],
		"light": _light(),
		"viewpoints": {
			"farfield": _farfield(),
			"farfield_shots": _views(FAR_FIELD_SHOTS),
			"perf": _views(PERF_VIEWS),
			"perf_breakdown": PERF_BREAKDOWN,
			# none: the link probe walks this street by its anchor room's door, as it did when the client drew
			# it (`SliceProbeWorld._link_check`); the route walker goes straight and the terrace's solids stop it
			"probe_route": [],
		},
	}


## The store's doorway: the florist's drawn door's out-point, kept when the piece was built.
func _florist_doorway() -> Array:
	return _plan(_florist_out)


## The station hall's floor in the street's frame: its east façade 0.4 m behind the doorway, its floor
## 16.8 m by 51.8 m with the arch 25.9 m from its south end (`worlds/social-cafe/places/station.yaml`).
func _halt_footprint() -> Array:
	var east := int(HALT_DOORWAY.x) - 400
	return [east - 16800, int(HALT_DOORWAY.y) - 25900, 16800, 51800]


## The café's VoxelGI and the two reflection probes (`SliceMain._voxel_gi`, `_reflections`).
func _light() -> Dictionary:
	var gi_size := Vector3(SliceCafe.W + 5.0, 9.0, SliceCafe.DEPTH + 9.0)
	var gi_at := Vector3(6.0, 4.0, SliceStreet.NORTH_FACE - SliceCafe.DEPTH * 0.5 + 4.5)
	var inside_size := Vector3(SliceCafe.W, 3.4, SliceCafe.DEPTH)
	var inside_at := Vector3(6.0, SliceStreet.WALK_Y + 1.7, SliceStreet.NORTH_FACE - SliceCafe.DEPTH * 0.5)
	var front_size := Vector3(22.0, 9.0, 9.0)
	var front_at := Vector3(6.0, 3.2, SliceStreet.NORTH_FACE + 3.4)
	return {
		"gi": [{"box_mm": _rect(AABB(gi_at - gi_size * 0.5, gi_size)), "height_m": _m(gi_size.y), "subdiv": 128}],
		"probes": [
			{"box_mm": _rect(AABB(inside_at - inside_size * 0.5, inside_size)), "height_m": _m(inside_size.y),
				"centre_up_m": _m(inside_at.y), "update": "once"},
			{"box_mm": _rect(AABB(front_at - front_size * 0.5, front_size)), "height_m": _m(front_size.y),
				"centre_up_m": _m(front_at.y), "update": "once"},
		],
	}


func _farfield() -> Array:
	var out := []
	for vp in FAR_FIELD_VIEWPOINTS:
		var p: Vector3 = vp[1]
		out.append({"id": vp[0], "at_mm": _plan(p), "eye_m": _m(p.y), "roles": vp[2]})
	return out


func _views(views: Array) -> Array:
	var out := []
	for v in views:
		var p: Vector3 = v[1]
		out.append({"name": v[0], "at_mm": _plan(p), "eye_m": _m(p.y), "yaw_deg": v[2], "pitch_deg": v[3]})
	return out


## Metres to the millimetre: a scene float carries single-precision noise the file need not.
func _m(x: float) -> float:
	return snappedf(x, 0.001)


## A scene point -> the street frame's plan millimetres [x east, y north].
func _plan(p: Vector3) -> Array:
	var rel := p - FRAME_ORIGIN
	return [roundi(rel.x * 1000.0), roundi(-rel.z * 1000.0)]


## A scene box -> the plan rectangle [x, y, width, height], (x, y) its south-west corner.
func _rect(b: AABB) -> Array:
	var sw := _plan(Vector3(b.position.x, 0.0, b.end.z))
	return [sw[0], sw[1], roundi(b.size.x * 1000.0), roundi(b.size.z * 1000.0)]
