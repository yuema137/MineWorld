## Inside The Daily Bean.
##
## This is the room the whole slice exists for, and the one place where the
## previous spike failed hardest: its cafe had a bare brick wall, one box
## counter, three cone pendants and no object smaller than a chair, against a
## reference whose back wall is full-height timber shelving loaded with jars,
## bottles, ceramics and plants, whose counter carries a glazed pastry case,
## and whose menu is a framed chalkboard of handwritten lines.
##
## `VISUAL_SLICE.md` sec.6.2 states the rule that came out of that: **objects
## must exist at three sizes.** Room-scale (shelving, counter, back bar),
## furniture-scale (tables, chairs, bench) and hand-scale (cups, jars, cakes,
## paper). A room with only the first two reads as a model of a cafe, and it is
## the third that is always the one missing.
##
## FRAME. Same as `cafe.gd`: x across the frontage, y 0 at the pavement outside,
## z 0 at the facade and negative into the room.
class_name SliceCafeInterior
extends RefCounted

## Where the counter stands, and the zones either side of it.
const COUNTER_Z := -7.30
const COUNTER_D := 0.78
const COUNTER_TOP := 1.06          ## above the floor
const BENCH_Z := -1.02
const SHELF_Z_OFF := 0.16          ## shelving stands this far off the back wall


static func fit_out(g: Node3D, w: float, depth: float, wall_t: float,
		floor_y: float, ceil_y: float) -> void:
	var n := Node3D.new()
	n.name = "Interior"
	g.add_child(n)

	var x0 := -w * 0.5 + wall_t      # inside face, left
	var x1 := w * 0.5 - wall_t       # inside face, right
	var z_back := -depth + wall_t    # inside face, back

	_wall_linings(n, x0, x1, z_back, floor_y, ceil_y)
	_back_wall(n, x0, x1, z_back, floor_y, ceil_y)
	_counter(n, x0, x1, z_back, floor_y)
	_window_bench(n, x0, x1, floor_y)
	_loose_seating(n, x0, floor_y)
	_back_room(n, x0, z_back, floor_y, ceil_y)
	_decoration(n, x0, x1, z_back, floor_y, ceil_y)
	_lighting(n, x0, x1, z_back, floor_y, ceil_y)


# --- surfaces ------------------------------------------------------------------

static func _wall_linings(n: Node3D, x0: float, x1: float, z_back: float,
		fy: float, cy: float) -> void:
	var plaster := SlicePalette.plaster_interior()
	var dado := SlicePalette.painted(SlicePalette.CAFE_GREEN_DARK, 0.58)
	var rail := SlicePalette.painted(SlicePalette.CEIL_TIMBER, 0.52)
	var z_mid := (z_back + 0.0) * 0.5
	var d := -z_back

	# plaster on both side walls, above a painted dado with a moulded rail and
	# a skirting -- the horizontal that stops a 3.3 m wall reading as a slab
	for sx in [-1.0, 1.0]:
		var wx: float = x1 if sx > 0.0 else x0
		Build.box(n, Vector3(wx - sx * 0.02, (fy + cy) * 0.5 + 0.44, z_mid),
			Vector3(0.04, cy - fy - 0.88, d), plaster)
		Build.box(n, Vector3(wx - sx * 0.035, fy + 0.46, z_mid),
			Vector3(0.07, 0.92, d), dado)
		Build.box(n, Vector3(wx - sx * 0.06, fy + 0.94, z_mid),
			Vector3(0.12, 0.055, d), rail)
		Build.box(n, Vector3(wx - sx * 0.055, fy + 0.07, z_mid),
			Vector3(0.11, 0.14, d), rail)

	# a rug in the seating half: a bare plank floor reads as unfinished, and the
	# reference's floor is broken up by furniture in exactly this zone
	Build.box(n, Vector3(x0 + 1.95, fy + 0.008, -4.30), Vector3(2.70, 0.016, 5.20),
		Mats.paint(Color(0.336, 0.234, 0.196), 0.96))
	Build.box(n, Vector3(x0 + 1.95, fy + 0.010, -4.30), Vector3(2.44, 0.016, 4.94),
		Mats.paint(Color(0.404, 0.292, 0.238), 0.96))


## The back wall: tiling behind the service zone, full-height timber shelving,
## and the framed chalkboard menu. In `03` this wall is most of what the eye
## lands on through the glass, and it is the single densest surface in the room.
static func _back_wall(n: Node3D, x0: float, x1: float, z_back: float,
		fy: float, cy: float) -> void:
	var timber := SlicePalette.painted(SlicePalette.SHELF_TIMBER, 0.62)
	var boards := SlicePalette.joinery_timber()

	# metro tiling behind the machine end
	Build.box(n, Vector3((x0 + x1) * 0.5, (fy + cy) * 0.5, z_back + 0.03),
		Vector3(x1 - x0, cy - fy, 0.06), SlicePalette.bar_tile())

	# the dresser: two tall bays of open shelving with a pilaster between them,
	# standing off the wall so the shelves cast onto the tiling behind
	# The left end of the wall is the back-room opening, so the bays start clear
	# of it; the menu board takes the wall to the right of the second bay.
	var bays := [[x0 + 1.56, x0 + 4.56], [x0 + 4.86, x0 + 7.26]]
	for b in bays:
		var bx0: float = b[0]
		var bx1: float = b[1]
		var cxm := (bx0 + bx1) * 0.5
		var zs := z_back + SHELF_Z_OFF
		# carcass
		Build.box(n, Vector3(cxm, (fy + cy) * 0.5, zs + 0.01),
			Vector3(bx1 - bx0, cy - fy, 0.05), timber)
		for ex in [bx0, bx1]:
			Build.box(n, Vector3(ex, (fy + cy) * 0.5, zs + 0.17),
				Vector3(0.07, cy - fy, 0.34), timber)
		# six shelves at a 0.42 m pitch, which is the plate's rhythm
		for i in range(6):
			var sy := fy + 0.52 + i * 0.42
			if sy > cy - 0.22:
				break
			Build.box(n, Vector3(cxm, sy, zs + 0.17), Vector3(bx1 - bx0, 0.045, 0.34), boards)
			_shelf_contents(n, bx0 + 0.20, bx1 - 0.20, sy + 0.023, zs + 0.17, i, int(cxm * 97.0))
		# a moulded cornice on top of the dresser
		Build.box(n, Vector3(cxm, cy - 0.12, zs + 0.24),
			Vector3(bx1 - bx0 + 0.10, 0.10, 0.46), timber)
		Build.box(n, Vector3(cxm, cy - 0.04, zs + 0.27),
			Vector3(bx1 - bx0 + 0.16, 0.07, 0.52), timber)

	_menu_board(n, x0 + 7.98, fy, cy, z_back)


## The variant sets. Several Poly Haven props are a display of alternatives in
## one file rather than one object (`dressing.gd`), so the slice names which
## member it wants instead of placing the whole display.
const BOTTLES := ["_bordeaux", "_alsace", "_burgundy", "_champagne"]
const PLANT_SMALL := ["_b"]
const PLANT_MED := ["_a"]


## A cup on its saucer, with an optional plate beside it. The single most
## repeated hand-scale object in the reference, and it is placed as two pieces
## of one set rather than as the whole ten-piece service.
static func _cup(n: Node3D, x: float, y: float, z: float, yaw: float,
		plate := false) -> void:
	SliceProps.put(n, "tea_set_01", Vector3(x, y, z), yaw, Color.WHITE, 0.28, 0.86,
		["_saucer_circular_03"])
	SliceProps.put(n, "tea_set_01", Vector3(x, y + 0.016, z), yaw + 0.6,
		Color.WHITE, 0.28, 0.86, ["_cup_small_01"])
	if plate:
		SliceProps.put(n, "tea_set_01", Vector3(x + 0.24, y, z - 0.05), yaw - 0.4,
			Color.WHITE, 0.28, 0.86, ["_plate_large_circular_03"])


## What is actually on the shelves. Every item here is hand-scale, which is the
## size class `VISUAL_SLICE.md` sec.6.2 names as the one that is always absent.
static func _shelf_contents(n: Node3D, x0: float, x1: float, y: float, z: float,
		row: int, seed_v: int) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v + row * 1013
	var warm := Color(1.04, 0.98, 0.90)
	match row:
		0:
			SliceProps.put(n, "wicker_basket_01", Vector3(x0 + 0.28, y, z), 0.4, warm)
			SliceProps.put(n, "wooden_crate_01", Vector3(x0 + 0.95, y, z), -0.2, warm)
			SliceProps.put(n, "pot_enamel_01", Vector3(x1 - 0.30, y, z), 1.1, warm)
		1:
			SliceProps.scatter(n, "wine_bottles_01",
				Vector3(x0 + 0.18, y, z), Vector3(0.24, 0, 0),
				maxi(1, int((x1 - x0) / 0.24)), seed_v, warm, 0.02, BOTTLES)
		2:
			SliceProps.put(n, "jug_01", Vector3(x0 + 0.26, y, z), 0.9, warm)
			SliceProps.put(n, "brass_pot_01", Vector3(x0 + 0.72, y, z), -0.5, warm)
			SliceProps.put(n, "ceramic_vase_02", Vector3(x0 + 1.22, y, z), 0.2, warm)
			SliceProps.put(n, "antique_ceramic_vase_01", Vector3(x0 + 1.72, y, z), 1.4, warm)
			if x1 - x0 > 2.4:
				SliceProps.put(n, "wooden_bowl_01", Vector3(x1 - 0.34, y, z), 0.7, warm)
		3:
			# the row of canisters the plate shows: turned timber, many of them
			var k := int((x1 - x0) / 0.19)
			for i in range(k):
				var h := rng.randf_range(0.13, 0.24)
				var r := rng.randf_range(0.052, 0.075)
				var c := Color(rng.randf_range(0.62, 0.92), rng.randf_range(0.54, 0.82),
					rng.randf_range(0.42, 0.68))
				Build.cyl(n, Vector3(x0 + 0.10 + i * 0.19, y, z + rng.randf_range(-0.05, 0.05)),
					r * 0.86, r, h, Mats.paint(c, 0.52), 10)
				Build.cyl(n, Vector3(x0 + 0.10 + i * 0.19, y + h, z + rng.randf_range(-0.02, 0.02)),
					r * 0.72, r * 0.90, 0.022, Mats.paint(c.darkened(0.35), 0.4), 10)
		4:
			SliceProps.put(n, "calathea_orbifolia_01", Vector3(x0 + 0.42, y, z), 0.3,
				Color(0.94, 1.02, 0.90), 0.28, 0.86, PLANT_SMALL)
			SliceProps.scatter(n, "ceramic_vase_02", Vector3(x0 + 1.20, y, z),
				Vector3(0.40, 0, 0), maxi(1, int((x1 - x0 - 1.2) / 0.40)), seed_v + 7, warm)
		_:
			SliceProps.put(n, "wicker_basket_01", Vector3(x0 + 0.40, y, z), 1.2, warm)
			SliceProps.put(n, "wooden_bowl_01", Vector3(x1 - 0.42, y, z), -0.6, warm)


## The framed chalkboard menu, with handwritten lines. `03` shows it dark, tall
## and dense with text, and a two-word board reads as a prop label.
static func _menu_board(n: Node3D, x: float, fy: float, cy: float, z_back: float) -> void:
	var z := z_back + 0.09
	Build.box(n, Vector3(x, fy + 2.02, z + 0.04),
		Vector3(1.32, 1.82, 0.08), SlicePalette.painted(Color(0.276, 0.204, 0.140), 0.76))
	Build.box(n, Vector3(x, fy + 2.02, z + 0.09),
		Vector3(1.16, 1.66, 0.02), SlicePalette.slate_board())
	var chalk := Color(0.90, 0.89, 0.85)
	Profile.text(n, Vector3(x, fy + 2.66, z + 0.105), "THE DAILY BEAN", 0.072,
		chalk, SlicePalette.serif_font())
	Profile.text(n, Vector3(x - 0.50, fy + 1.94, z + 0.105),
		"espresso\nflat white\ncortado\nfilter\nchai\nhot chocolate", 0.058,
		Color(0.87, 0.86, 0.82), SlicePalette.script_font(), 0.0,
		HORIZONTAL_ALIGNMENT_LEFT)
	Profile.text(n, Vector3(x + 0.44, fy + 1.94, z + 0.105),
		"2.40\n3.10\n2.90\n2.60\n3.20\n3.40", 0.058, Color(0.87, 0.86, 0.82),
		SlicePalette.script_font(), 0.0, HORIZONTAL_ALIGNMENT_RIGHT)
	Profile.text(n, Vector3(x, fy + 1.28, z + 0.105), "good coffee, brighter days",
		0.050, Color(0.82, 0.81, 0.78), SlicePalette.script_font())


# --- the counter ---------------------------------------------------------------

static func _counter(n: Node3D, x0: float, x1: float, z_back: float, fy: float) -> void:
	var cx0 := x0 + 3.86        # the lane in from the door runs to its left
	var cx1 := x1
	var cxm := (cx0 + cx1) * 0.5
	var front := COUNTER_Z + COUNTER_D * 0.5
	var timber := SlicePalette.painted(SlicePalette.COUNTER_TIMBER, 0.56)

	# carcass, panelled front, moulded top
	Build.box(n, Vector3(cxm, fy + COUNTER_TOP * 0.5, COUNTER_Z),
		Vector3(cx1 - cx0, COUNTER_TOP, COUNTER_D), timber, 0.0, true)
	var panels := int((cx1 - cx0) / 0.86)
	var pw := (cx1 - cx0) / float(panels)
	for i in range(panels):
		Profile.panel(n, cx0 + pw * (float(i) + 0.5), fy + COUNTER_TOP * 0.52,
			pw - 0.16, COUNTER_TOP - 0.30, front + 0.005,
			SlicePalette.painted(SlicePalette.CAFE_GREEN_DARK, 0.52), 0.05, 0.018)
	Build.box(n, Vector3(cxm, fy + COUNTER_TOP + 0.025, COUNTER_Z),
		Vector3(cx1 - cx0 + 0.09, 0.05, COUNTER_D + 0.09), SlicePalette.counter_top())
	var top_y := fy + COUNTER_TOP + 0.05

	_pastry_case(n, cx0 + 0.95, top_y, front - 0.06)

	# the machine, the grinder and the till: the three objects a viewer looks
	# for behind a counter, in the order they look for them
	_espresso_machine(n, cx0 + 2.45, top_y, COUNTER_Z - 0.06)
	# Authored, not Poly Haven's CashRegister_01: that model's atlas reproduces a
	# Bank of Canada banknote, which Poly Haven's CC0 cannot relicense (ASSETS.md).
	_till(n, cx1 - 0.52, top_y, COUNTER_Z - 0.02)
	SliceProps.put(n, "jug_01", Vector3(cx0 + 3.35, top_y, COUNTER_Z + 0.10), 0.8)
	SliceProps.put(n, "wooden_bowl_01", Vector3(cx1 - 1.20, top_y, COUNTER_Z + 0.16), 0.3)
	SliceProps.put(n, "food_apple_01", Vector3(cx1 - 1.22, top_y + 0.055, COUNTER_Z + 0.14), 1.1)
	SliceProps.put(n, "food_apple_01", Vector3(cx1 - 1.14, top_y + 0.055, COUNTER_Z + 0.20), 2.4)
	SliceProps.put(n, "calathea_orbifolia_01", Vector3(cx0 + 0.24, top_y, COUNTER_Z), 0.6,
		Color(0.94, 1.02, 0.90), 0.28, 0.86, PLANT_MED)

	# a folded paper on the counter, and a stack of cups: hand scale again
	Build.box(n, Vector3(cx1 - 1.86, top_y + 0.004, COUNTER_Z + 0.14),
		Vector3(0.21, 0.008, 0.29), Mats.paint(Color(0.88, 0.85, 0.78), 0.92), 0.22)
	for i in range(5):
		Build.cyl(n, Vector3(cx0 + 4.15, top_y + i * 0.055, COUNTER_Z - 0.20),
			0.042, 0.033, 0.055, Mats.paint(Color(0.90, 0.88, 0.84), 0.38), 12)

	# the service zone behind: a worktop, a shelf of cups, and an under-counter
	# run, all visible over the counter from the customer side
	var bz := z_back + 0.72
	Build.box(n, Vector3(cxm, fy + 0.44, bz), Vector3(cx1 - cx0, 0.88, 0.62),
		SlicePalette.painted(Color(0.300, 0.324, 0.306), 0.52), 0.0, true)
	Build.box(n, Vector3(cxm, fy + 0.90, bz), Vector3(cx1 - cx0 + 0.06, 0.05, 0.68),
		SlicePalette.counter_top())
	_cup(n, cx0 + 1.10, fy + 0.95, bz, 0.4)
	_cup(n, cx0 + 1.52, fy + 0.95, bz + 0.04, -0.5)
	SliceProps.put(n, "pot_enamel_01", Vector3(cx0 + 2.30, fy + 0.95, bz), -0.6)
	SliceProps.put(n, "brass_pot_01", Vector3(cx1 - 0.60, fy + 0.95, bz), 0.9)


## A glazed pastry case: a timber plinth, two loaded shelves and a glass hood.
## The plate puts croissants and cakes at eye level immediately inside the
## window, and it is the first thing a person walking past actually sees.
static func _pastry_case(n: Node3D, cx: float, y: float, z: float) -> void:
	var steel := Mats.paint(Color(0.62, 0.62, 0.60), 0.30, 0.80)
	var glass := SlicePalette.door_glass()
	var w := 1.34
	var d := 0.62
	var h := 0.62
	Build.box(n, Vector3(cx, y + 0.035, z), Vector3(w, 0.07, d), steel)
	# frame
	for sx in [-1.0, 1.0]:
		for sz in [-1.0, 1.0]:
			Build.box(n, Vector3(cx + sx * (w * 0.5 - 0.02), y + h * 0.5, z + sz * (d * 0.5 - 0.02)),
				Vector3(0.035, h, 0.035), steel)
	Build.box(n, Vector3(cx, y + h, z), Vector3(w, 0.045, d), steel)
	# glazing on three sides and the top
	Build.box(n, Vector3(cx, y + h * 0.5, z + d * 0.5), Vector3(w - 0.06, h - 0.06, 0.012), glass)
	Build.box(n, Vector3(cx, y + h * 0.5, z - d * 0.5), Vector3(w - 0.06, h - 0.06, 0.012), glass)
	for sx in [-1.0, 1.0]:
		Build.box(n, Vector3(cx + sx * w * 0.5, y + h * 0.5, z),
			Vector3(0.012, h - 0.06, d - 0.06), glass)
	# two shelves, loaded
	for s in range(2):
		var sy := y + 0.085 + s * 0.27
		Build.box(n, Vector3(cx, sy, z), Vector3(w - 0.09, 0.016, d - 0.09), glass)
		var rng := RandomNumberGenerator.new()
		rng.seed = 3301 + s * 17
		for i in range(5):
			var px := cx - w * 0.5 + 0.18 + i * (w - 0.36) / 4.0
			var pz := z + rng.randf_range(-0.10, 0.10)
			if s == 0:
				SliceProps.put(n, "croissant", Vector3(px, sy + 0.008, pz),
					rng.randf() * TAU, Color(1.06, 0.96, 0.84))
			elif i % 2 == 0:
				SliceProps.put(n, "carrot_cake", Vector3(px, sy + 0.008, pz),
					rng.randf() * TAU, Color(1.02, 0.96, 0.88))
			else:
				SliceProps.put(n, "croissant", Vector3(px, sy + 0.008, pz),
					rng.randf() * TAU, Color(1.02, 0.92, 0.80))
	# a small warm light inside, so the case glows the way a lit display does
	var l := OmniLight3D.new()
	l.light_color = Color(1.0, 0.90, 0.72)
	l.light_energy = 1.7
	l.omni_range = 1.5
	l.position = Vector3(cx, y + h - 0.06, z)
	n.add_child(l)


static func _espresso_machine(n: Node3D, cx: float, y: float, z: float) -> void:
	var chrome := Mats.paint(Color(0.74, 0.74, 0.72), 0.16, 0.92)
	var body := Mats.paint(Color(0.302, 0.108, 0.096), 0.28, 0.55)
	Build.box(n, Vector3(cx, y + 0.24, z), Vector3(0.86, 0.48, 0.52), body)
	Build.box(n, Vector3(cx, y + 0.50, z), Vector3(0.92, 0.06, 0.56), chrome)
	Build.box(n, Vector3(cx, y + 0.055, z + 0.20), Vector3(0.80, 0.03, 0.16), chrome)
	for sx in [-0.24, 0.24]:
		Build.cyl(n, Vector3(cx + sx, y + 0.10, z + 0.24), 0.030, 0.030, 0.10, chrome, 10)
		Build.box(n, Vector3(cx + sx, y + 0.08, z + 0.30), Vector3(0.09, 0.03, 0.10), chrome)
	Build.cyl(n, Vector3(cx + 0.44, y + 0.12, z + 0.16), 0.016, 0.016, 0.26, chrome, 8)
	# the grinder beside it
	Build.cyl(n, Vector3(cx + 0.64, y, z - 0.02), 0.09, 0.10, 0.34, body, 12)
	Build.cyl(n, Vector3(cx + 0.64, y + 0.34, z - 0.02), 0.11, 0.075, 0.20,
		Mats.paint(Color(0.24, 0.16, 0.10, 0.85), 0.20), 12)
	# cups warming on top
	for i in range(4):
		Build.cyl(n, Vector3(cx - 0.30 + i * 0.20, y + 0.53, z - 0.10),
			0.038, 0.030, 0.052, Mats.paint(Color(0.90, 0.88, 0.84), 0.36), 10)


## A small modern till: cash drawer, angled screen on a stem, card reader.
## ~0.40 x 0.42 x 0.38 m overall, which is the size class of a real one.
static func _till(n: Node3D, cx: float, y: float, z: float) -> void:
	var shell := Mats.paint(Color(0.86, 0.85, 0.82), 0.40)
	var dark := Mats.paint(Color(0.10, 0.10, 0.11), 0.30, 0.20)
	Build.box(n, Vector3(cx, y + 0.06, z), Vector3(0.40, 0.12, 0.38), shell)
	Build.box(n, Vector3(cx, y + 0.06, z + 0.192), Vector3(0.34, 0.006, 0.006), dark)
	Build.cyl(n, Vector3(cx, y + 0.12, z - 0.06), 0.022, 0.022, 0.12, dark, 8)
	Build.box(n, Vector3(cx, y + 0.30, z - 0.06), Vector3(0.30, 0.20, 0.020), dark)
	Build.box(n, Vector3(cx, y + 0.30, z - 0.06), Vector3(0.27, 0.17, 0.028),
		Mats.emissive(Color(0.62, 0.74, 0.78), 0.35))
	Build.box(n, Vector3(cx + 0.27, y + 0.03, z + 0.08), Vector3(0.08, 0.06, 0.15), dark)


# --- seating -------------------------------------------------------------------

## The window bench, which is what makes `03` read as a place people sit in
## rather than a shop they queue in.
static func _window_bench(n: Node3D, x0: float, x1: float, fy: float) -> void:
	# starts clear of the door: a bench across the entrance is a wall
	var bx0 := x0 + 2.36
	var bx1 := x1 - 0.30
	var timber := SlicePalette.painted(SlicePalette.COUNTER_TIMBER, 0.58)
	Build.box(n, Vector3((bx0 + bx1) * 0.5, fy + 0.44, BENCH_Z),
		Vector3(bx1 - bx0, 0.06, 0.44), SlicePalette.joinery_timber(), 0.0, true)
	Build.box(n, Vector3((bx0 + bx1) * 0.5, fy + 0.21, BENCH_Z),
		Vector3(bx1 - bx0, 0.42, 0.10), timber)
	for i in range(5):
		var lx := bx0 + (bx1 - bx0) * float(i) / 4.0
		Build.box(n, Vector3(lx, fy + 0.21, BENCH_Z + 0.16), Vector3(0.07, 0.42, 0.07), timber)
	# cushions
	var rng := RandomNumberGenerator.new()
	rng.seed = 5150
	for i in range(4):
		var cx := bx0 + 0.70 + i * (bx1 - bx0 - 1.4) / 3.0
		Build.box(n, Vector3(cx, fy + 0.50, BENCH_Z), Vector3(0.52, 0.09, 0.40),
			Mats.paint(Color(0.72, 0.66, 0.54).lerp(Color(0.46, 0.50, 0.44), rng.randf()), 0.94))
	# a narrow drinks ledge under the window, and stools facing it
	Build.box(n, Vector3((bx0 + bx1) * 0.5, fy + 0.74, BENCH_Z - 0.44),
		Vector3(bx1 - bx0, 0.05, 0.34), SlicePalette.counter_top())
	for i in range(3):
		var sx := bx0 + 0.80 + i * 1.60
		SliceProps.put(n, "bar_chair_round_01", Vector3(sx, fy, BENCH_Z - 0.98),
			PI + 0.1 * i, Color(0.98, 0.94, 0.88))
	# cups on the ledge
	_cup(n, bx0 + 1.30, fy + 0.768, BENCH_Z - 0.44, 0.5)
	_cup(n, bx0 + 3.05, fy + 0.768, BENCH_Z - 0.44, -1.1)


static func _loose_seating(n: Node3D, x0: float, fy: float) -> void:
	# Placed to leave a clear lane from the door to the counter. The door is at
	# x0 + 1.61 and a person walks in on that line; anything standing on it is
	# an obstacle course, which `--slice-drive` would find and a screenshot
	# would not.
	var seats := [
		[x0 + 4.86, -3.50, 0.15],
		[x0 + 7.11, -4.90, -0.22],
		[x0 + 2.26, -8.90, 0.08],
	]
	var rng := RandomNumberGenerator.new()
	rng.seed = 8123
	for i in range(seats.size()):
		var s: Array = seats[i]
		var tx: float = s[0]
		var tz: float = s[1]
		var ty: float = s[2]
		SliceProps.put_solid(n, "round_wooden_table_02", Vector3(tx, fy, tz), ty,
			Color(1.04, 0.96, 0.86))
		var chair := "dining_chair_02" if i != 1 else "painted_wooden_chair_01"
		SliceProps.put(n, chair, Vector3(tx - 0.66, fy, tz + 0.10), PI * 0.5 + ty,
			Color(0.98, 0.92, 0.84))
		SliceProps.put(n, chair, Vector3(tx + 0.68, fy, tz - 0.08), -PI * 0.5 + ty,
			Color(0.98, 0.92, 0.84))
		# what is on the table -- the size class that is always missing
		_cup(n, tx + rng.randf_range(-0.14, 0.10), fy + 0.748,
			tz + rng.randf_range(-0.10, 0.06), rng.randf() * TAU, i == 0)
		if i == 2:
			SliceProps.put(n, "carrot_cake", Vector3(tx + 0.20, fy + 0.748, tz + 0.16), 1.2)


# --- the back room, and the opening into it ------------------------------------

## `VISUAL_SLICE.md` sec.6.1 requires an opening other than the front glazing,
## so the room is not a sealed box. A lit doorway to a back room does it, and it
## also gives the deepest part of the room something to be lit by.
static func _back_room(n: Node3D, x0: float, z_back: float, fy: float, cy: float) -> void:
	var dx := x0 + 0.72
	var dw := 1.05
	var dh := 2.12
	var frame := SlicePalette.painted(SlicePalette.CAFE_GREEN_DARK, 0.52)
	# the opening is cut by simply not building the dresser there; line it
	for sx in [-1.0, 1.0]:
		Build.box(n, Vector3(dx + sx * (dw * 0.5 + 0.045), fy + dh * 0.5, z_back + 0.12),
			Vector3(0.09, dh, 0.28), frame)
	Build.box(n, Vector3(dx, fy + dh + 0.045, z_back + 0.12), Vector3(dw + 0.18, 0.09, 0.28), frame)
	# a shallow recess behind it, with a shelf and a warm light, seen through
	# the opening only -- it is a glimpse, and a glimpse is what it should be
	var d := 1.20
	Build.box(n, Vector3(dx, fy + dh * 0.5, z_back - d * 0.5),
		Vector3(dw + 0.10, dh, 0.06), SlicePalette.plaster_interior())
	Build.box(n, Vector3(dx, fy - 0.03, z_back - d * 0.5), Vector3(dw, 0.06, d),
		SlicePalette.floor_boards())
	for sx in [-1.0, 1.0]:
		Build.box(n, Vector3(dx + sx * dw * 0.5, fy + dh * 0.5, z_back - d * 0.5),
			Vector3(0.06, dh, d), SlicePalette.plaster_interior())
	Build.box(n, Vector3(dx, fy + dh, z_back - d * 0.5), Vector3(dw, 0.06, d),
		SlicePalette.plaster_interior())
	SliceProps.put(n, "steel_frame_shelves_02", Vector3(dx, fy, z_back - 0.85), PI,
		Color(0.82, 0.82, 0.80))
	SliceProps.put(n, "wooden_crate_01", Vector3(dx - 0.28, fy, z_back - 0.34), 0.5)
	var l := OmniLight3D.new()
	l.light_color = Color(1.0, 0.86, 0.66)
	l.light_energy = 3.4
	l.omni_range = 3.4
	l.position = Vector3(dx, fy + 2.0, z_back - 0.55)
	n.add_child(l)
	# no way through: the back room is scenery, and it is blocked honestly
	Build.box_blocker(n, Vector3(dx, fy + 1.0, z_back + 0.02), Vector3(dw, 2.0, 0.14))


# --- decoration ----------------------------------------------------------------

static func _decoration(n: Node3D, x0: float, x1: float, z_back: float,
		fy: float, cy: float) -> void:
	var warm := Color(1.02, 0.96, 0.88)
	# pictures on the left wall, at two heights
	# hanging_picture_frame_02 was here until its painting turned out to be
	# signed by an artist Poly Haven does not credit -- excluded, see ASSETS.md.
	SliceProps.put(n, "fancy_picture_frame_01", Vector3(x0 + 0.06, fy + 2.10, -5.10),
		PI * 0.5, warm)
	SliceProps.put(n, "wall_clock", Vector3(x1 - 0.06, fy + 2.32, -4.40), -PI * 0.5, warm)
	# a chalk-written strapline on the left wall's plaster, as the frontage has
	Profile.text(n, Vector3(x0 + 0.09, fy + 1.62, -7.10), "people\nplaces\npossibilities",
		0.10, Color(0.44, 0.40, 0.34), SlicePalette.script_font(), PI * 0.5)
	# floor plants at two sizes, and a shelf run on the left wall
	Props.place(n, "potted_plant_02", Vector3(x0 + 0.46, fy, -1.90), 0.7,
		Color(0.92, 1.02, 0.88))
	Props.place(n, "potted_plant_02", Vector3(x0 + 0.52, fy, -8.35), 1.2,
		Color(0.92, 1.02, 0.88))
	SliceProps.put(n, "wooden_display_shelves_01", Vector3(x0 + 0.34, fy, -6.05),
		PI * 0.5, Color(1.02, 0.94, 0.86))
	SliceProps.put(n, "Shelf_01", Vector3(x1 - 0.36, fy, -2.35), -PI * 0.5,
		Color(1.02, 0.94, 0.86))
	SliceProps.put(n, "wicker_basket_01", Vector3(x1 - 0.42, fy + 0.86, -2.35), 0.4, warm)
	SliceProps.put(n, "ceramic_vase_02", Vector3(x0 + 0.40, fy + 1.02, -6.05), 0.9, warm)
	SliceProps.put(n, "calathea_orbifolia_01", Vector3(x0 + 0.44, fy + 1.02, -5.60), 0.5,
		Color(0.92, 1.02, 0.88), 0.28, 0.86, PLANT_SMALL)


# --- light ---------------------------------------------------------------------

## Visible fittings, not invisible sources. `03` hangs six pendants at three
## heights on black cords: a clear-glass lantern over the counter, wide conical
## shades over the tables, and small ones deep in the room. The variation is
## what makes the ceiling read as a ceiling rather than as a lit plane.
##
## Energies are set high on purpose. The exterior exposure is tuned for a
## sunlit street, so a room lit to a comfortable indoor level shows through the
## glass as a flat black panel -- and looking in through the glass is most of
## what makes the plate work. `--threshold` measures whether this is right
## rather than leaving it to impression.
static func _lighting(n: Node3D, x0: float, x1: float, z_back: float,
		fy: float, cy: float) -> void:
	var warm := SlicePalette.LAMP_WARM
	# x, z, lowest point above the FLOOR, slug, energy, range, shadows.
	# VISUAL_SLICE.md sec.3.4 puts a pendant's lowest point between 1.90 and
	# 2.30 m; the three heights are the plate's, which hangs them unevenly.
	var pendants := [
		[x0 + 4.86, -3.50, 2.05, "hanging_industrial_lamp", 5.6, 7.0, true],
		[x0 + 7.11, -4.90, 2.28, "modern_ceiling_lamp_01", 4.2, 6.0, false],
		[x0 + 4.86, -6.10, 1.95, "hanging_industrial_lamp", 4.4, 6.0, false],
		[x1 - 2.40, COUNTER_Z + 0.52, 2.10, "hanging_industrial_lamp", 5.4, 6.5, true],
		[x1 - 0.80, COUNTER_Z + 0.52, 2.26, "modern_ceiling_lamp_01", 4.2, 5.5, false],
		[x0 + 1.80, -1.90, 2.18, "modern_ceiling_lamp_01", 3.8, 5.5, false],
		[x0 + 2.26, -8.90, 2.12, "hanging_industrial_lamp", 3.6, 5.0, false],
	]
	for p in pendants:
		var px: float = p[0]
		var pz: float = p[1]
		var low: float = p[2]
		# The fitting carries its own flex and ceiling rose, so it is hung by
		# the top of its own bounding box rather than stood on the ceiling.
		var fit := SliceProps.hang(n, p[3], px, cy - 0.02, pz, 0.0,
			Color(0.62, 0.60, 0.56))
		var box := SliceProps.placed_box(fit)
		# and then dropped until its lowest point is where it should be
		fit.position.y += (fy + low) - box.position.y
		box = SliceProps.placed_box(fit)
		# the flex, from the ceiling down to the top of the fitting
		var top := box.position.y + box.size.y
		Build.cyl(n, Vector3(px, top, pz), 0.008, 0.008, maxf(cy - top, 0.02),
			Mats.paint(Color(0.10, 0.10, 0.10), 0.7), 5)
		Build.sphere(n, Vector3(px, box.position.y + 0.10, pz), 0.045,
			Mats.emissive(warm, 9.0), 8)
		var l := OmniLight3D.new()
		l.light_color = warm
		l.light_energy = p[4]
		l.omni_range = p[5]
		l.omni_attenuation = 1.15
		l.shadow_enabled = p[6]
		l.position = Vector3(px, box.position.y + 0.06, pz)
		n.add_child(l)

	# a wide, dim, shadowless fill so the corners of a 3.3 m room are not black.
	# One light, stated as a fill, rather than raising every pendant until the
	# room is flat.
	var fill := OmniLight3D.new()
	fill.light_color = Color(1.0, 0.88, 0.74)
	fill.light_energy = 2.3
	fill.omni_range = 16.0
	fill.omni_attenuation = 0.55
	fill.shadow_enabled = false
	fill.position = Vector3((x0 + x1) * 0.5, fy + 2.30, -5.20)
	n.add_child(fill)
