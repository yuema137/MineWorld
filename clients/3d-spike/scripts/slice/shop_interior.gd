## Inside The Flower Room: the slice's second enterable building.
##
## `VISUAL_SLICE.md` sec.4.1 (amended 2026-10-06) requires exactly one secondary
## shop the player walks into, held to completeness rather than to the café's
## density: four walls with thickness, a walked floor, a treated ceiling, the
## shop glass shared with the street, one opening besides the front, and a
## fit-out that names the trade from inside at the three object sizes of sec.6.2.
##
## The trade is chosen so the room cannot be mistaken for the café's: a
## florist's is a cool, plant-filled room with a tiled floor and staging rather
## than a warm timber one with a counter and tables.
##
##   room scale       stepped plant staging in the window, display shelving on the
##                    west wall, a work counter and shelving at the back
##   furniture scale  a central display table, a row of flower buckets, planters
##   hand scale       vases, small pots, a till, a roll of wrapping paper, twine,
##                    price cards
##
## FRAME: the terrace unit's own -- x across the frontage (-w/2 .. w/2), y 0 at
## the pavement, z 0 at the façade and negative into the shop. The unit's
## façade skin (`terrace.gd`, 0 .. -SKIN) carries the door and the window.
class_name SliceShopInterior
extends RefCounted

const FLOOR_Y := 0.12              ## one step up, the door's own step
const CEIL_Y := 3.40               ## floor to ceiling 3.28 m
const WALL := 0.30
## The back room's doorway: width, height.
const BACK_DOOR := Vector2(1.00, 2.10)

## The slice's Place volume for the room, in world space, filled in by `build`.
static var room_box := AABB()


static func build(g: Node3D, u: SliceTerrace.Unit, ground_h: float, top: float,
		skin: float) -> void:
	var w := u.width()
	var d := u.depth
	var xi0 := -w * 0.5 + WALL
	var xi1 := w * 0.5 - WALL
	var zf := -skin
	var zb := -d + WALL
	var zm := (zf + zb) * 0.5
	var n := Node3D.new()
	n.name = "FlowerRoom"
	g.add_child(n)

	# --- shell -------------------------------------------------------------------
	# floor: terracotta quarry tiles, the commonest florist's floor and unlike any
	# other floor in the slice
	Build.box(n, Vector3(0, FLOOR_Y - 0.06, zm), Vector3(w - 0.02, 0.12, zf - zb),
		Mats.pbr("long_white_tiles", 2.60, Color(0.80, 0.52, 0.40), 0.66), 0.0, true)
	# side and back walls, full height, solid; the back wall with the back
	# room's doorway cut through it
	for sx in [-1.0, 1.0]:
		Build.box(n, Vector3(sx * (w * 0.5 - WALL * 0.5), top * 0.5, (zf - d) * 0.5),
			Vector3(WALL, top, d + zf), u.wall, 0.0, true)
	var back_door := Rect2(xi0 + 1.05 - BACK_DOOR.x * 0.5, FLOOR_Y, BACK_DOOR.x, BACK_DOOR.y)
	Profile.wall_with_holes(n, -w * 0.5, w * 0.5, 0.0, top, zb, WALL, [back_door], u.wall)
	# the storey above is solid mass, behind the façade's upper windows
	Build.box(n, Vector3(0, (CEIL_Y + 0.14 + top) * 0.5, (zf + zb) * 0.5),
		Vector3(w - WALL * 2.0, top - CEIL_Y - 0.14, zf - zb), u.wall)
	# ceiling: painted boards on dark joists, front to back
	Build.box(n, Vector3(0, CEIL_Y + 0.07, zm), Vector3(xi1 - xi0, 0.14, zf - zb),
		SlicePalette.painted(Color(0.80, 0.78, 0.70), 0.86))
	var joist := SlicePalette.painted(Color(0.27, 0.21, 0.15), 0.72)
	for i in range(6):
		var jx := xi0 + 0.55 + i * (xi1 - xi0 - 1.1) / 5.0
		Build.box(n, Vector3(jx, CEIL_Y - 0.08, zm), Vector3(0.14, 0.16, zf - zb), joist)

	# linings: sage-painted plaster above a dark green boarded dado
	var plaster := SlicePalette.painted(Color(0.60, 0.65, 0.56), 0.92)
	var dado := SlicePalette.painted(u.front_c.darkened(0.15), 0.60)
	var rail := SlicePalette.painted(Color(0.84, 0.82, 0.74), 0.55)
	for sx in [-1.0, 1.0]:
		var wx: float = xi1 if sx > 0.0 else xi0
		Build.box(n, Vector3(wx - sx * 0.02, (FLOOR_Y + CEIL_Y) * 0.5 + 0.5, zm),
			Vector3(0.04, CEIL_Y - FLOOR_Y - 1.0, zf - zb), plaster)
		Build.box(n, Vector3(wx - sx * 0.03, FLOOR_Y + 0.55, zm), Vector3(0.06, 1.10, zf - zb), dado)
		Build.box(n, Vector3(wx - sx * 0.05, FLOOR_Y + 1.11, zm), Vector3(0.10, 0.05, zf - zb), rail)
		# the boards' joints, so the dado reads as boarding and not paint
		var k := int((zf - zb) / 0.14)
		for j in range(k):
			Build.box(n, Vector3(wx - sx * 0.062, FLOOR_Y + 0.55, zf - 0.07 - j * 0.14),
				Vector3(0.006, 1.08, 0.012), joist)
	Profile.wall_with_holes(n, xi0, xi1, FLOOR_Y, CEIL_Y, zb + 0.04, 0.04, [back_door], plaster,
		false)

	_back_room(n, xi0 + 1.05, zb)
	_window_staging(n, -w * 0.5 + 2.40, w * 0.5 - 1.00)
	_west_shelving(n, xi0, zf)
	_counter(n, 0.55, xi1 - 0.25, zb)
	_buckets(n, xi1, zf)
	_display_table(n, 0.10, -3.70)
	_floor_plants(n, xi0, xi1, zf)
	_hanging(n, xi0, xi1)
	_lighting(n, xi0, xi1, zf, zb)
	_probe(n, w, d, zf, zb)

	room_box = AABB(g.to_global(Vector3(xi0, FLOOR_Y - 0.5, zb)),
		Vector3(xi1 - xi0, CEIL_Y - FLOOR_Y + 0.5, zf - zb))


# --- the back room, the opening besides the front ---------------------------------

## A lit doorway into a back room: a real recess through the back wall, a
## cold store of buckets on steel shelving, as the café's back room is a glimpse
## of its store. Blocked honestly at the doorway; the back room is scenery.
static func _back_room(n: Node3D, dx: float, zb: float) -> void:
	var dw: float = BACK_DOOR.x
	var dh: float = BACK_DOOR.y
	var frame := SlicePalette.painted(Color(0.84, 0.82, 0.74), 0.55)
	for sx in [-1.0, 1.0]:
		Build.box(n, Vector3(dx + sx * (dw * 0.5 + 0.045), FLOOR_Y + dh * 0.5, zb + 0.06),
			Vector3(0.09, dh, 0.14), frame)
	Build.box(n, Vector3(dx, FLOOR_Y + dh + 0.045, zb + 0.06), Vector3(dw + 0.18, 0.09, 0.14), frame)
	# the recess behind the wall: floor, walls and a ceiling, 1.4 m deep
	var back := SlicePalette.painted(Color(0.70, 0.74, 0.70), 0.90)
	var rd := 1.40
	var rz := zb - WALL - rd * 0.5
	var rw := dw + 0.80
	Build.box(n, Vector3(dx, FLOOR_Y - 0.03, zb - WALL * 0.5 - rd * 0.5), Vector3(rw, 0.06, rd + WALL),
		Mats.paint(Color(0.50, 0.50, 0.48), 0.8))
	Build.box(n, Vector3(dx, FLOOR_Y + dh * 0.5 + 0.1, rz - rd * 0.5), Vector3(rw, dh + 0.2, 0.06), back)
	for sx in [-1.0, 1.0]:
		Build.box(n, Vector3(dx + sx * rw * 0.5, FLOOR_Y + dh * 0.5 + 0.1, rz), Vector3(0.06, dh + 0.2, rd),
			back)
	Build.box(n, Vector3(dx, FLOOR_Y + dh + 0.2, rz), Vector3(rw, 0.06, rd), back)
	SliceProps.put(n, "steel_frame_shelves_02", Vector3(dx - 0.30, FLOOR_Y, rz - 0.40), PI,
		Color(0.82, 0.82, 0.80))
	for i in range(3):
		var y := FLOOR_Y + 0.02
		Build.cyl(n, Vector3(dx + 0.20, y, rz + 0.25 - i * 0.36), 0.15, 0.13, 0.32, Mats.paint(Color(0.66, 0.68, 0.70), 0.30, 0.75), 12)
		Props.flowerbed(n, Vector3(dx + 0.20, y + 0.42, rz + 0.25 - i * 0.36), 0.26, 0.26, 77 + i)
	var l := OmniLight3D.new()
	l.light_color = Color(0.92, 0.96, 1.0)
	l.light_energy = 2.2
	l.omni_range = 2.6
	l.position = Vector3(dx, FLOOR_Y + 2.0, rz)
	n.add_child(l)
	Build.box_blocker(n, Vector3(dx, FLOOR_Y + 1.0, zb - WALL * 0.5), Vector3(dw, 2.0, 0.14))


# --- room scale -------------------------------------------------------------------

## Stepped staging behind the shop window: three timber tiers of pots and
## buckets, so from the pavement the window is full of plants, which is what
## names a florist from across the street.
static func _window_staging(n: Node3D, x0: float, x1: float) -> void:
	var timber := SlicePalette.painted(Color(0.40, 0.30, 0.20), 0.70)
	var rng := RandomNumberGenerator.new()
	rng.seed = 7717
	var z0 := -0.62
	for t in range(3):
		var y := FLOOR_Y + 0.30 + t * 0.30
		var z := z0 - t * 0.26
		Build.box(n, Vector3((x0 + x1) * 0.5, y, z), Vector3(x1 - x0, 0.04, 0.26), timber)
		var k := int((x1 - x0) / 0.36)
		for i in range(k):
			var px := x0 + 0.18 + i * (x1 - x0 - 0.36) / float(maxi(k - 1, 1))
			if (i + t) % 3 == 0:
				SliceProps.put(n, "planter_pot_clay", Vector3(px, y + 0.02, z), rng.randf() * TAU,
					Color(0.96, 0.90, 0.84))
				SliceProps.put(n, "shrub_03", Vector3(px, y + 0.20, z), rng.randf() * TAU,
					Color(0.92, 1.00, 0.86), 0.28, 0.86, ["_a"])
			else:
				_pot_of_flowers(n, Vector3(px, y + 0.02, z), 0.09, 0.16, rng.randi())
	# the staging's frame: legs, and a solid footprint the player cannot walk into
	for lx in [x0 + 0.05, (x0 + x1) * 0.5, x1 - 0.05]:
		Build.box(n, Vector3(lx, FLOOR_Y + 0.46, z0 - 0.26), Vector3(0.05, 0.92, 0.80), timber)
	Build.box_blocker(n, Vector3((x0 + x1) * 0.5, FLOOR_Y + 0.6, z0 - 0.26),
		Vector3(x1 - x0, 1.2, 0.84))


## Display shelving down the west wall: tall timber units of pots and vases.
static func _west_shelving(n: Node3D, xi0: float, zf: float) -> void:
	var warm := Color(1.0, 0.95, 0.88)
	for i in range(2):
		var z := -2.90 - i * 1.25
		SliceProps.put_solid(n, "wooden_display_shelves_01", Vector3(xi0 + 0.22, FLOOR_Y, z),
			PI * 0.5, warm)
		# what stands on them: the unit's shelves are at about 0.35 / 0.75 / 1.15 m
		for s in range(3):
			var y := FLOOR_Y + 0.37 + s * 0.40
			SliceProps.put(n, ["ceramic_vase_02", "antique_ceramic_vase_01", "jug_01"][s],
				Vector3(xi0 + 0.22, y, z + 0.28), 0.4 + s, warm)
			_pot_of_flowers(n, Vector3(xi0 + 0.22, y, z - 0.25), 0.07, 0.12, 300 + i * 10 + s)


## The work counter at the back: a timber bench with a zinc top, the till, a
## roll of brown paper on its bracket, twine, scissors and a jar of stems --
## where the trade happens, and the most hand-scale corner of the room.
static func _counter(n: Node3D, x0: float, x1: float, zb: float) -> void:
	var z := zb + 1.25
	var h := 0.95
	var timber := SlicePalette.painted(Color(0.36, 0.27, 0.19), 0.62)
	var zinc := Mats.paint(Color(0.62, 0.64, 0.64), 0.34, 0.7)
	Build.box(n, Vector3((x0 + x1) * 0.5, FLOOR_Y + h * 0.5, z), Vector3(x1 - x0, h, 0.70),
		timber, 0.0, true)
	Build.box(n, Vector3((x0 + x1) * 0.5, FLOOR_Y + h + 0.02, z), Vector3(x1 - x0 + 0.06, 0.04, 0.76),
		zinc)
	var y := FLOOR_Y + h + 0.04
	SliceCafeInterior._till(n, x1 - 0.45, y, z - 0.05)
	# the paper roll on its bracket, at the counter's front edge
	Build.box(n, Vector3(x0 + 0.70, y + 0.10, z + 0.25), Vector3(0.04, 0.20, 0.10),
		SlicePalette.iron())
	Build.box(n, Vector3(x0 + 1.70, y + 0.10, z + 0.25), Vector3(0.04, 0.20, 0.10),
		SlicePalette.iron())
	var paper := Build.cyl(n, Vector3(x0 + 0.72, y + 0.14, z + 0.25), 0.075, 0.075, 0.96,
		Mats.paint(Color(0.62, 0.48, 0.34), 0.90), 14)
	paper.rotation.z = PI * 0.5
	paper.position = Vector3(x0 + 1.20, y + 0.14, z + 0.25)
	# a sheet pulled off it across the counter, a ball of twine, scissors
	Build.box(n, Vector3(x0 + 1.20, y + 0.004, z + 0.02), Vector3(0.80, 0.006, 0.42),
		Mats.paint(Color(0.66, 0.52, 0.38), 0.92))
	Build.cyl(n, Vector3(x0 + 2.10, y, z + 0.10), 0.05, 0.06, 0.09,
		Mats.paint(Color(0.76, 0.66, 0.48), 0.95), 10)
	Build.box(n, Vector3(x0 + 2.35, y + 0.006, z + 0.18), Vector3(0.16, 0.012, 0.05),
		Mats.paint(Color(0.12, 0.12, 0.13), 0.3, 0.8))
	_pot_of_flowers(n, Vector3(x0 + 2.75, y, z - 0.10), 0.07, 0.20, 9137)
	# price cards on the counter front
	for i in range(3):
		Build.box(n, Vector3(x0 + 0.6 + i * 0.9, FLOOR_Y + 0.62, z + 0.36), Vector3(0.18, 0.12, 0.01),
			Mats.paint(Color(0.92, 0.90, 0.84), 0.9))
	# shelving behind it, on the back wall, with vases and stock
	for i in range(2):
		var sx := x0 + 0.85 + i * 1.30
		SliceProps.put_solid(n, "Shelf_01", Vector3(sx, FLOOR_Y, zb + 0.16), 0.0,
			Color(1.0, 0.94, 0.86))
		for s in range(3):
			var sy := FLOOR_Y + 0.46 + s * 0.52
			SliceProps.put(n, "ceramic_vase_02", Vector3(sx - 0.25, sy, zb + 0.18), s * 0.7,
				Color(1.0, 0.96, 0.90))
			if s == 0:
				SliceProps.put(n, "wicker_basket_01", Vector3(sx + 0.20, sy, zb + 0.18), s * 0.9,
					Color(1.0, 0.96, 0.90))
			else:
				_pot_of_flowers(n, Vector3(sx + 0.20, sy, zb + 0.18), 0.07, 0.12, 500 + s + i * 7)
	# a chalkboard over the counter: what is in today
	Build.box(n, Vector3((x0 + x1) * 0.5, FLOOR_Y + 2.45, zb + 0.05), Vector3(1.30, 0.70, 0.05),
		SlicePalette.painted(Color(0.30, 0.23, 0.16), 0.8))
	Build.box(n, Vector3((x0 + x1) * 0.5, FLOOR_Y + 2.45, zb + 0.08), Vector3(1.18, 0.58, 0.01),
		SlicePalette.slate_board())
	Profile.text(n, Vector3((x0 + x1) * 0.5, FLOOR_Y + 2.47, zb + 0.09),
		"IN TODAY\nsweet peas . ranunculus\npeonies . eucalyptus", 0.055,
		Color(0.90, 0.89, 0.85), SlicePalette.serif_font())


# --- furniture scale --------------------------------------------------------------

## Galvanised buckets of cut flowers on a low step down the east wall, the image
## a florist's interior is recognised by.
static func _buckets(n: Node3D, xi1: float, zf: float) -> void:
	var zinc := Mats.paint(Color(0.66, 0.68, 0.70), 0.30, 0.75)
	var x := xi1 - 0.36
	Build.box(n, Vector3(x, FLOOR_Y + 0.11, -3.20), Vector3(0.62, 0.22, 3.40),
		SlicePalette.painted(Color(0.32, 0.25, 0.18), 0.7), 0.0, true)
	for i in range(7):
		var z := -1.70 - i * 0.48
		var y := FLOOR_Y + 0.22
		Build.cyl(n, Vector3(x, y, z), 0.17, 0.14, 0.34, zinc, 14)
		Props.flowerbed(n, Vector3(x, y + 0.42, z), 0.30, 0.30, 811 + i * 37)
		# stems: a few leaf cards standing up out of the bucket
		Build.card(n, Vector3(x, y + 0.40, z), 0.26, 0.46,
			Mats.card(ProcGen.leaf_card(901 + i, Color(0.27, 0.42, 0.20)), 0.9), i * 0.7, 0.0, false)
	# a high shelf above them on brackets, with trailing pots: the east wall was
	# a bare plane of plaster above the buckets
	var timber := SlicePalette.painted(Color(0.40, 0.30, 0.20), 0.70)
	Build.box(n, Vector3(xi1 - 0.14, FLOOR_Y + 1.95, -3.20), Vector3(0.26, 0.04, 3.40), timber)
	for k in range(4):
		Build.box(n, Vector3(xi1 - 0.06, FLOOR_Y + 1.84, -1.70 - k * 1.0), Vector3(0.10, 0.20, 0.04),
			SlicePalette.iron())
	for k in range(6):
		var pz := -1.75 - k * 0.58
		Build.cyl(n, Vector3(xi1 - 0.14, FLOOR_Y + 1.97, pz), 0.08, 0.06, 0.13,
			Mats.paint(Color(0.66, 0.42, 0.30), 0.9), 10)
		SliceCafe._trailing(n, Vector3(xi1 - 0.16, FLOOR_Y + 2.08, pz), 0.24, 0.55, 3001 + k)


static func _display_table(n: Node3D, x: float, z: float) -> void:
	SliceProps.put_solid(n, "round_wooden_table_02", Vector3(x, FLOOR_Y, z), 0.3,
		Color(1.02, 0.96, 0.88))
	var y := FLOOR_Y + 0.75
	# a big arrangement in the middle, and small pots round it
	_pot_of_flowers(n, Vector3(x, y, z), 0.10, 0.28, 6211, 0.55)
	SliceProps.put(n, "antique_ceramic_vase_01", Vector3(x + 0.26, y, z + 0.14), 0.9,
		Color(1.0, 0.96, 0.90))
	SliceProps.put(n, "planter_pot_clay", Vector3(x - 0.24, y, z - 0.12), 0.5,
		Color(0.96, 0.90, 0.84))
	SliceProps.put(n, "shrub_03", Vector3(x - 0.24, y + 0.18, z - 0.12), 1.3,
		Color(0.92, 1.00, 0.86), 0.28, 0.86, ["_a"])
	SliceProps.put(n, "wicker_basket_01", Vector3(x - 0.10, y, z + 0.26), 2.2,
		Color(1.0, 0.96, 0.90))


static func _floor_plants(n: Node3D, xi0: float, xi1: float, zf: float) -> void:
	var g := Color(0.92, 1.02, 0.88)
	Props.place(n, "potted_plant_02", Vector3(xi0 + 0.45, FLOOR_Y, -5.60), 0.6, g)
	Props.place(n, "potted_plant_02", Vector3(xi1 - 0.45, FLOOR_Y, -5.90), 2.0, g)
	SliceProps.put(n, "planter_pot_clay", Vector3(xi0 + 0.40, FLOOR_Y, -1.30), 0.2,
		Color(0.96, 0.90, 0.84))
	SliceProps.put(n, "calathea_orbifolia_01", Vector3(xi0 + 0.40, FLOOR_Y + 0.20, -1.30), 0.5,
		g, 0.28, 0.86, ["_a"])
	SliceProps.put_solid(n, "wooden_crate_01", Vector3(xi1 - 0.50, FLOOR_Y, -0.95), 0.1,
		Color(1.0, 0.95, 0.88))
	_pot_of_flowers(n, Vector3(xi1 - 0.30, FLOOR_Y + 0.35, -0.95), 0.08, 0.14, 4441)
	_pot_of_flowers(n, Vector3(xi1 - 0.68, FLOOR_Y + 0.35, -0.98), 0.08, 0.14, 4447)


## Trailing plants hung from the joists, which every florist has and which gives
## the ceiling something besides light fittings.
static func _hanging(n: Node3D, xi0: float, xi1: float) -> void:
	for p in [Vector3(xi0 + 1.6, 0, -2.2), Vector3(-0.6, 0, -5.4), Vector3(xi1 - 1.7, 0, -2.6)]:
		var y := CEIL_Y - 0.95
		Build.cyl(n, Vector3(p.x, y + 0.20, p.z), 0.006, 0.006, CEIL_Y - y - 0.36,
			SlicePalette.iron(), 4)
		Build.cyl(n, Vector3(p.x, y, p.z), 0.20, 0.12, 0.20,
			Mats.paint(Color(0.62, 0.42, 0.30), 0.9), 12)
		SliceCafe._trailing(n, Vector3(p.x, y + 0.16, p.z), 0.42, 0.70, int(absf(p.z) * 977.0))


# --- light ------------------------------------------------------------------------

## Three visible pendants and a dim fill. Cooler and brighter than the café --
## a florist works in daylight-coloured light -- but still lit by its fittings.
static func _lighting(n: Node3D, xi0: float, xi1: float, zf: float, zb: float) -> void:
	var warm := Color(1.0, 0.86, 0.66)
	for p in [Vector3(-1.6, 0, -2.6), Vector3(1.6, 0, -2.6), Vector3(1.9, 0, zb + 1.30)]:
		var fit := SliceProps.hang(n, "modern_ceiling_lamp_01", p.x, CEIL_Y - 0.02, p.z, 0.0,
			Color(0.80, 0.80, 0.76))
		var box := SliceProps.placed_box(fit)
		fit.position.y += (FLOOR_Y + 2.25) - box.position.y
		box = SliceProps.placed_box(fit)
		Build.cyl(n, Vector3(p.x, box.position.y + box.size.y, p.z), 0.008, 0.008,
			maxf(CEIL_Y - box.position.y - box.size.y, 0.02), Mats.paint(Color(0.1, 0.1, 0.1), 0.7), 5)
		Build.sphere(n, Vector3(p.x, box.position.y + 0.10, p.z), 0.045, Mats.emissive(warm, 8.0), 8)
		var l := OmniLight3D.new()
		l.light_color = warm
		l.light_energy = 3.2
		l.omni_range = 6.0
		l.omni_attenuation = 1.1
		l.shadow_enabled = true
		l.position = Vector3(p.x, box.position.y + 0.04, p.z)
		n.add_child(l)
	var fill := OmniLight3D.new()
	fill.light_color = Color(0.96, 0.94, 0.90)
	fill.light_energy = 0.8
	fill.omni_range = 12.0
	fill.omni_attenuation = 0.6
	fill.shadow_enabled = false
	fill.position = Vector3(0, FLOOR_Y + 2.3, (zf + zb) * 0.5)
	n.add_child(fill)


## An interior reflection probe, as the café has: inside it the room's ambient
## replaces the sky's, which a closed room cannot see.
static func _probe(n: Node3D, w: float, d: float, zf: float, zb: float) -> void:
	var p := ReflectionProbe.new()
	p.name = "FlowerRoomProbe"
	p.size = Vector3(w - WALL * 2.0, CEIL_Y - FLOOR_Y, zf - zb)
	p.position = Vector3(0, (FLOOR_Y + CEIL_Y) * 0.5, (zf + zb) * 0.5)
	p.box_projection = true
	p.interior = true
	p.update_mode = ReflectionProbe.UPDATE_ONCE
	p.ambient_mode = ReflectionProbe.AMBIENT_COLOR
	p.ambient_color = Color(0.42, 0.42, 0.38)
	p.ambient_color_energy = 0.6
	n.add_child(p)


# --- hand scale -------------------------------------------------------------------

## A small pot or vase with flowers in it: a glazed cylinder and a flower head
## cluster. The hand-scale object a florist has hundreds of.
static func _pot_of_flowers(n: Node3D, base: Vector3, r: float, h: float, seed_v: int,
		spread := 0.0) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	var c: Color = [Color(0.80, 0.78, 0.72), Color(0.36, 0.44, 0.40), Color(0.66, 0.42, 0.30),
		Color(0.30, 0.34, 0.46)][rng.randi() % 4]
	Build.cyl(n, base, r * 0.92, r * 0.78, h, Mats.paint(c, 0.34), 10)
	# a cluster of small crossed flower cards -- Props.flowerbed's are 0.4-0.6 m,
	# the size of a bed, not of what stands in a vase
	var s := maxf(spread, r * 2.6)
	var fm := Props._flower_mat(seed_v % 4)
	var k := 3 if spread == 0.0 else 7
	for i in range(k):
		var off := Vector3(rng.randf_range(-0.3, 0.3) * s, 0, rng.randf_range(-0.3, 0.3) * s) \
			if i > 0 else Vector3.ZERO
		var cs := s * rng.randf_range(0.75, 1.0)
		Build.card(n, base + off + Vector3(0, h + cs * 0.42, 0), cs, cs * 0.95, fm,
			rng.randf() * TAU, 0.0, false)
