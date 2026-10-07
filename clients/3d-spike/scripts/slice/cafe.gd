## The Daily Bean: the slice's one enterable building.
##
## Built to `presentation/mineworld-default/3D/references/03_cafe_frontage.png`,
## read as pixels at magnification. What that plate actually contains, in the
## order a viewer notices it:
##
##   a sage-green painted timber shopfront with serif cream capitals on the
##   fascia; a projecting moulded cornice on dark corbels above the fascia;
##   a circular dark-timber sign on a wrought-iron scroll bracket; glazing
##   divided by slim green mullions with white script painted on the glass;
##   a low stallriser; a panelled door with brass furniture and a glass
##   lantern over it; rough coursed limestone above, with a timber window box
##   spilling greenery; a hanging basket; a framed slate board on the stone;
##   an A-board on the paving; bistro tables with black metal frames and
##   timber slats; planters.
##
## Everything in that list is here. The interior is `cafe_interior.gd`.
##
## FRAME. Local, with the building node placed by `slice_world.gd`:
##   x  along the frontage, -W/2 .. +W/2
##   y  0 at the pavement outside the door
##   z  0 at the facade plane, +z out toward the street, -z into the shop
class_name SliceCafe
extends RefCounted

const W := 9.00                ## frontage width
const DEPTH := 11.00           ## front wall to back wall, outside faces
const WALL_T := 0.34
const FLOOR_Y := 0.15          ## one step up from the pavement
const CEIL_Y := 3.45           ## floor to underside of the ceiling boards: 3.30 m
const GLAZE_HEAD := 2.86
const STALL_H := 0.42
const FASCIA_Y := 2.98
const FASCIA_H := 0.92
const STOREY := 3.30
const DOOR_X := -2.55
const DOOR_W := 0.98
const DOOR_H := 2.22

## Where the shopfront joinery stops and the masonry piers begin.
const PIER := 0.62

const NAME := "The Daily Bean"


static func build(parent: Node3D, origin: Vector3) -> Node3D:
	var g := Node3D.new()
	g.name = "DailyBean"
	g.position = origin
	parent.add_child(g)

	var stone := SlicePalette.limestone()
	_shell(g, stone)
	_shopfront(g, stone)
	_upper_storeys(g, stone)
	_roof(g, stone)
	_frontage_dressing(g)
	SliceCafeInterior.fit_out(g, W, DEPTH, WALL_T, FLOOR_Y, CEIL_Y)
	return g


# --- structure ----------------------------------------------------------------

## Walls, floor and ceiling. Four real walls with thickness and a real opening,
## assembled in pieces because Godot's runtime path has no CSG and a solid box
## cannot have a hole cut in it. Assembling the front wall out of piers, a
## stallriser split around the door, a lintel and a spandrel is also how the
## real thing is built, so the proportions come out right instead of being
## fought.
static func _shell(g: Node3D, stone: Material) -> void:
	var back := -DEPTH + WALL_T
	var inner_w := W - WALL_T * 2.0
	var top := STOREY * 3.0 + 0.6

	# floor, one step above the pavement, and a ceiling of exposed boards
	Build.box(g, Vector3(0, FLOOR_Y - 0.06, -DEPTH * 0.5),
		Vector3(inner_w, 0.12, DEPTH), SlicePalette.floor_boards(), 0.0, true)
	Build.box(g, Vector3(0, CEIL_Y + 0.07, -DEPTH * 0.5),
		Vector3(inner_w, 0.14, DEPTH), SlicePalette.ceiling_boards())
	# joists under the boards: the ceiling is the largest interior surface after
	# the shelving wall and a flat plane there is what made the old room read as
	# a box with a lid.
	var beam := SlicePalette.painted(SlicePalette.COUNTER_TIMBER, 0.72)
	var n_beams := int(DEPTH / 1.05)
	for i in range(n_beams):
		var z := -0.9 - i * (DEPTH - 1.6) / float(n_beams - 1)
		Build.box(g, Vector3(0, CEIL_Y - 0.09, z), Vector3(inner_w, 0.18, 0.15), beam)

	# side walls and back wall, full height
	for sx in [-1.0, 1.0]:
		Build.box(g, Vector3(sx * (W * 0.5 - WALL_T * 0.5), top * 0.5, -DEPTH * 0.5),
			Vector3(WALL_T, top, DEPTH), stone, 0.0, true)
	Build.box(g, Vector3(0, top * 0.5, back - WALL_T * 0.5),
		Vector3(W, top, WALL_T), stone, 0.0, true)
	# the storeys above the shop are solid: nobody goes up there, and a hollow
	# volume would leak the interior's own lights into the upper windows.
	# It starts behind the front wall, not at its face: a mass flush with the
	# façade buried every upper window inside it.
	Build.box(g, Vector3(0, (CEIL_Y + 0.14 + top) * 0.5, -(DEPTH + WALL_T) * 0.5),
		Vector3(inner_w, top - CEIL_Y - 0.14, DEPTH - WALL_T), stone, 0.0, true)


## The shopfront: the wall that is almost entirely joinery and glass.
static func _shopfront(g: Node3D, stone: Material) -> void:
	var green := SlicePalette.painted(SlicePalette.CAFE_GREEN, 0.48)
	var green_dk := SlicePalette.painted(SlicePalette.CAFE_GREEN_DARK, 0.46)
	var glass := SlicePalette.shop_glass()

	# masonry piers at each end, running the full height
	for sx in [-1.0, 1.0]:
		Build.box(g, Vector3(sx * (W * 0.5 - PIER * 0.5), (STOREY * 3.0 + 0.6) * 0.5,
			-WALL_T * 0.5), Vector3(PIER, STOREY * 3.0 + 0.6, WALL_T), stone, 0.0, true)

	var x_l := -W * 0.5 + PIER          # inside face of the left pier
	var x_r := W * 0.5 - PIER

	# --- the joinery frame ----------------------------------------------------
	# pilasters at each end of the shopfront, then a mullion splitting the
	# window, and the door opening between the left pilaster and the window
	Profile.pilaster(g, x_l + 0.13, 0.0, FASCIA_Y, 0.26, 0.13, green, green_dk)
	Profile.pilaster(g, x_r - 0.13, 0.0, FASCIA_Y, 0.26, 0.13, green, green_dk)

	var win_l := DOOR_X + DOOR_W * 0.5 + 0.19
	var win_r := x_r - 0.29
	var mull := win_l + (win_r - win_l) * 0.46

	# stallriser: a panelled plinth under the glass, split by the door
	_stallriser(g, x_l + 0.26, DOOR_X - DOOR_W * 0.5 - 0.09, green, green_dk)
	_stallriser(g, win_l - 0.09, x_r - 0.26, green, green_dk)

	# door reveal, lintel and threshold
	_doorway(g, green, green_dk, stone)

	# The sidelight between the left pilaster and the door. The first build
	# left this bay as an open hole -- interior visible straight through, and
	# 0.8 m of nothing solid above the stallriser, wide enough for a body. The
	# frontage ray scan in --measure found it; frame 03 shows it.
	var s0 := x_l + 0.26
	var s1 := DOOR_X - DOOR_W * 0.5 - 0.15
	Build.box(g, Vector3((s0 + s1) * 0.5, STALL_H + 0.045, 0.055),
		Vector3(s1 - s0 + 0.10, 0.09, 0.19), green_dk)                         # cill
	Build.box(g, Vector3((s0 + s1) * 0.5, GLAZE_HEAD + 0.06, 0.035),
		Vector3(s1 - s0 + 0.10, 0.12, 0.17), green_dk)                         # head
	Build.box(g, Vector3((s0 + s1) * 0.5, (STALL_H + GLAZE_HEAD) * 0.5, -0.045),
		Vector3(s1 - s0, GLAZE_HEAD - STALL_H - 0.05, 0.016), glass)
	Build.box_blocker(g, Vector3((x_l + DOOR_X - DOOR_W * 0.5) * 0.5, GLAZE_HEAD * 0.5, -0.06),
		Vector3(DOOR_X - DOOR_W * 0.5 - x_l, GLAZE_HEAD, 0.10))

	# window frame: cill, jambs, transom, head
	var y0 := STALL_H
	var y1 := GLAZE_HEAD
	Build.box(g, Vector3((win_l + win_r) * 0.5, y0 + 0.045, 0.055),
		Vector3(win_r - win_l + 0.18, 0.09, 0.19), green_dk)                  # cill
	for jx in [win_l, win_r]:
		Build.box(g, Vector3(jx, (y0 + y1) * 0.5, 0.025),
			Vector3(0.11, y1 - y0, 0.15), green)
	Build.box(g, Vector3(mull, (y0 + y1) * 0.5, 0.020),
		Vector3(0.085, y1 - y0, 0.13), green)                                 # mullion
	var transom := y0 + (y1 - y0) * 0.80
	Build.box(g, Vector3((win_l + win_r) * 0.5, transom, 0.020),
		Vector3(win_r - win_l, 0.085, 0.13), green)
	Build.box(g, Vector3((win_l + win_r) * 0.5, y1 + 0.06, 0.035),
		Vector3(win_r - win_l + 0.24, 0.12, 0.17), green_dk)                  # head

	# the glass, one pane per bay, set back inside the frame
	for bay in [[win_l, mull], [mull, win_r]]:
		Build.box(g, Vector3((bay[0] + bay[1]) * 0.5, (y0 + transom) * 0.5, -0.045),
			Vector3(bay[1] - bay[0] - 0.05, transom - y0 - 0.05, 0.016), glass)
		Build.box(g, Vector3((bay[0] + bay[1]) * 0.5, (transom + y1) * 0.5, -0.045),
			Vector3(bay[1] - bay[0] - 0.05, y1 - transom - 0.09, 0.016), glass)

	# The glass is solid. Without this the only barrier was the 0.42 m
	# stallriser, and from inside the window bench is a step up onto it: the
	# drive walked straight out through the pane.
	# (to the pier, not the jamb: the scan found a 0.3 m sliver between them)
	var bl := DOOR_X + DOOR_W * 0.5 + 0.08     # overlapping the right jamb
	Build.box_blocker(g, Vector3((bl + x_r) * 0.5, GLAZE_HEAD * 0.5, -0.06),
		Vector3(x_r - bl, GLAZE_HEAD, 0.10))

	# Cream sign-writing on the left pane, as 03 has it: an upright serif, each
	# line centred, opaque. Was a thin script at 92% alpha, left-aligned, which
	# over the lit room was barely legible (preview miss #5). A faint dark
	# outline is the sign-writer's shade line that holds it against the room.
	var script := Profile.text(g, Vector3(win_l + 0.80, 1.80, -0.030),
		"Better\nCoffee\nBrighter\nDays", 0.150, Color(0.97, 0.95, 0.89, 1.0),
		SlicePalette.serif_font(), 0.0, HORIZONTAL_ALIGNMENT_CENTER, 0.06)
	script.line_spacing = 6.0

	# spandrel above the glazing, then the fascia
	Build.box(g, Vector3(0, (GLAZE_HEAD + FASCIA_Y) * 0.5 + 0.06, -0.02),
		Vector3(W - PIER * 2.0, FASCIA_Y - GLAZE_HEAD - 0.12, 0.30), green_dk)
	_fascia(g, green, green_dk)


static func _stallriser(g: Node3D, x0: float, x1: float, green: Material,
		dark: Material) -> void:
	if x1 - x0 < 0.2:
		return
	Build.box(g, Vector3((x0 + x1) * 0.5, STALL_H * 0.5, -0.06),
		Vector3(x1 - x0, STALL_H, WALL_T - 0.10), dark, 0.0, true)
	# a moulded capping and a recessed panel, so it is joinery and not a kerb
	Build.box(g, Vector3((x0 + x1) * 0.5, STALL_H + 0.025, 0.045),
		Vector3(x1 - x0 + 0.08, 0.05, 0.20), green)
	var n := maxi(1, int((x1 - x0) / 1.15))
	var pw := (x1 - x0) / float(n)
	for i in range(n):
		Profile.panel(g, x0 + pw * (float(i) + 0.5), STALL_H * 0.52, pw - 0.15,
			STALL_H - 0.17, 0.055, green)


static func _doorway(g: Node3D, green: Material, dark: Material, stone: Material) -> void:
	var d0 := DOOR_X - DOOR_W * 0.5
	var d1 := DOOR_X + DOOR_W * 0.5

	# jambs and head, in painted timber, standing proud of the wall face
	for jx in [d0 - 0.075, d1 + 0.075]:
		Build.box(g, Vector3(jx, DOOR_H * 0.5, 0.045), Vector3(0.15, DOOR_H + 0.20, 0.19), green,
			0.0, true)
	Build.box(g, Vector3(DOOR_X, DOOR_H + 0.075, 0.045),
		Vector3(DOOR_W + 0.42, 0.15, 0.19), green)
	# lintel and spandrel over the door up to the fascia
	Build.box(g, Vector3(DOOR_X, (DOOR_H + 0.15 + FASCIA_Y) * 0.5, -0.06),
		Vector3(DOOR_W + 0.30, FASCIA_Y - DOOR_H - 0.15, WALL_T - 0.10), dark, 0.0, true)
	# a small fanlight over the door
	Build.box(g, Vector3(DOOR_X, DOOR_H + 0.42, -0.04), Vector3(DOOR_W - 0.06, 0.40, 0.014),
		SlicePalette.door_glass())

	# reveal linings, so the opening reads as a thickness and not a cut
	for jx in [d0 - 0.025, d1 + 0.025]:
		Build.box(g, Vector3(jx, DOOR_H * 0.5, -WALL_T * 0.5),
			Vector3(0.05, DOOR_H, WALL_T), dark)
	Build.box(g, Vector3(DOOR_X, DOOR_H + 0.025, -WALL_T * 0.5),
		Vector3(DOOR_W + 0.10, 0.05, WALL_T), dark)

	# the step and the threshold
	Build.box(g, Vector3(DOOR_X, FLOOR_Y * 0.5, 0.26), Vector3(DOOR_W + 0.70, FLOOR_Y, 0.52),
		SlicePalette.kerbstone())
	Build.box(g, Vector3(DOOR_X, FLOOR_Y - 0.02, -WALL_T * 0.5),
		Vector3(DOOR_W + 0.10, 0.05, WALL_T), SlicePalette.kerbstone())
	# a hidden wedge so crossing the step is a walk, not a hop
	var ramp := StaticBody3D.new()
	ramp.collision_layer = Build.LAYER_WORLD
	var cs := CollisionShape3D.new()
	var bs := BoxShape3D.new()
	bs.size = Vector3(DOOR_W + 0.60, 0.30, 0.62)
	cs.shape = bs
	ramp.add_child(cs)
	ramp.transform = Transform3D(Basis(Vector3.RIGHT, deg_to_rad(15.0)),
		Vector3(DOOR_X, FLOOR_Y - 0.13, 0.30))
	g.add_child(ramp)

	_door_leaf(g, d0)
	SliceTerrace.doors.append([g.to_global(Vector3(DOOR_X, 0.0, 0.0)), g.global_rotation.y,
		"The Daily Bean -- enterable"])


## An open door leaf, hinged at the left jamb and swung inward.
static func _door_leaf(g: Node3D, hinge_x: float) -> void:
	var leaf := Node3D.new()
	leaf.position = Vector3(hinge_x, FLOOR_Y, -WALL_T * 0.45)
	# +1.92 rad, not -1.92: Basis(UP, t) sends the local +x axis to
	# (cos t, 0, -sin t), so a negative angle swings the leaf out over the
	# pavement. It opens inward.
	leaf.rotation.y = 1.92
	g.add_child(leaf)
	var green := SlicePalette.painted(SlicePalette.CAFE_GREEN, 0.46)
	var dark := SlicePalette.painted(SlicePalette.CAFE_GREEN_DARK, 0.44)
	var h := DOOR_H - FLOOR_Y
	Build.box(leaf, Vector3(DOOR_W * 0.5, h * 0.5, 0.0), Vector3(DOOR_W, h, 0.055), green)
	# a glazed upper half and two raised panels below, which is what 03 shows
	Build.box(leaf, Vector3(DOOR_W * 0.5, h * 0.70, -0.005),
		Vector3(DOOR_W - 0.24, h * 0.42, 0.018), SlicePalette.door_glass())
	for i in range(2):
		Profile.panel(leaf, DOOR_W * (0.28 + i * 0.44), h * 0.26, DOOR_W * 0.38,
			h * 0.30, 0.032, dark, 0.045, 0.014)
	# brass lever, escutcheon and kick plate
	var brass := SlicePalette.brass()
	Build.box(leaf, Vector3(DOOR_W - 0.13, h * 0.47, -0.05), Vector3(0.05, 0.22, 0.045), brass)
	Build.cyl(leaf, Vector3(DOOR_W - 0.13, h * 0.47, -0.09), 0.016, 0.016, 0.12, brass, 8)
	Build.box(leaf, Vector3(DOOR_W * 0.5, 0.11, -0.033), Vector3(DOOR_W - 0.10, 0.18, 0.012), brass)
	Build.box_blocker(leaf, Vector3(DOOR_W * 0.5, h * 0.5, 0.0), Vector3(DOOR_W, h, 0.07))


static func _fascia(g: Node3D, green: Material, dark: Material) -> void:
	var x0 := -W * 0.5 + PIER * 0.45
	var x1 := W * 0.5 - PIER * 0.45
	# the signboard itself, standing proud of the wall
	Build.box(g, Vector3(0, FASCIA_Y + FASCIA_H * 0.5, 0.075),
		Vector3(x1 - x0, FASCIA_H, 0.15), green)
	Build.box(g, Vector3(0, FASCIA_Y + 0.035, 0.105),
		Vector3(x1 - x0 + 0.07, 0.07, 0.21), dark)
	Profile.text(g, Vector3(-0.35, FASCIA_Y + FASCIA_H * 0.53, 0.165), NAME, 0.40,
		SlicePalette.SIGN_CREAM, SlicePalette.serif_font())
	# the strapline block to the right, as 03 has it
	Profile.text(g, Vector3(x1 - 1.30, FASCIA_Y + FASCIA_H * 0.55, 0.165),
		"COFFEE\nGOOD PEOPLE\nBRIGHTER DAYS", 0.088, SlicePalette.SIGN_CREAM,
		SlicePalette.serif_font())

	# the cornice over the fascia, on corbels: the deepest shadow line on the
	# whole frontage and the thing that most makes it read as built
	var top := FASCIA_Y + FASCIA_H
	var timber := SlicePalette.painted(Color(0.196, 0.152, 0.116), 0.70)
	for i in range(7):
		Profile.corbel(g, x0 + (x1 - x0) * float(i) / 6.0, top + 0.02, 0.30, timber, 0.11, 0.22)
	Profile.cornice(g, x0 - 0.10, x1 + 0.10, top, 0.40,
		SlicePalette.painted(Color(0.780, 0.726, 0.618), 0.80), 3, 0.085)


# --- masonry above -------------------------------------------------------------

static func _upper_storeys(g: Node3D, stone: Material) -> void:
	var base := FASCIA_Y + FASCIA_H + 0.35
	var x0 := -W * 0.5
	var x1 := W * 0.5
	for s in range(2):
		var y := base + s * STOREY
		# three windows per storey, on the rhythm of the shopfront below, cut
		# THROUGH the wall -- see Profile.wall_with_holes
		var holes: Array = []
		for i in range(3):
			var hx := -W * 0.5 + W * (float(i) + 0.5) / 3.0
			holes.append(Rect2(hx - 0.53, y + 0.72, 1.06, 1.82))
		Profile.wall_with_holes(g, -W * 0.5, W * 0.5, y, y + STOREY, 0.0, WALL_T, holes, stone)
		for i in range(3):
			var cx := -W * 0.5 + W * (float(i) + 0.5) / 3.0
			Profile.punched_window(g, cx, y + 0.72, 1.06, 1.82,
				SlicePalette.CAFE_GREEN_LIGHT, SlicePalette.dead_glass(), stone,
				0.19, 1, 2)
		if s == 0:
			# 03 puts timber window boxes of trailing greenery on the first floor;
			# one box under three windows read as an exception, not a habit
			for i in range(3):
				var wx := -W * 0.5 + W * (float(i) + 0.5) / 3.0
				var anchor := Profile.window_box(g, wx, base + 0.60, 1.42,
					SlicePalette.painted(Color(0.360, 0.268, 0.184), 0.82))
				Props.flowerbed(g, anchor + Vector3(0, 0.10, 0.02), 1.30, 0.22, 4409 + i * 31)
				_trailing(g, Vector3(wx, base + 0.58, 0.28), 1.30, 1.25 - i * 0.25, 8821 + i * 7)
			Profile.band(g, x0, x1, base + STOREY - 0.22, 0.20, 0.10,
				SlicePalette.painted(Color(0.796, 0.744, 0.640), 0.86))


static func _roof(g: Node3D, stone: Material) -> void:
	var y := FASCIA_Y + FASCIA_H + 0.35 + STOREY * 2.0
	Profile.dentils(g, -W * 0.5 + 0.2, W * 0.5 - 0.2, y - 0.16, 0.02,
		SlicePalette.painted(Color(0.762, 0.710, 0.606), 0.86), 0.28, 0.13, 0.11, 0.13)
	Profile.cornice(g, -W * 0.5, W * 0.5, y, 0.42,
		SlicePalette.painted(Color(0.796, 0.744, 0.640), 0.84), 4, 0.075)
	Profile.parapet(g, -W * 0.5, W * 0.5, y + 0.30, DEPTH, stone,
		SlicePalette.kerbstone(), 0.68)
	# two chimney stacks, because a flat parapet with nothing on it is the
	# silhouette VISUAL_FIDELITY sec.5 calls a hard fail
	for sx in [-1.0, 1.0]:
		var cx: float = sx * W * 0.28
		Build.box(g, Vector3(cx, y + 1.55, -DEPTH * 0.66), Vector3(0.95, 2.4, 0.78),
			SlicePalette.brick_red())
		Build.box(g, Vector3(cx, y + 2.80, -DEPTH * 0.66), Vector3(1.10, 0.14, 0.92),
			SlicePalette.kerbstone())
		for i in range(2):
			Build.cyl(g, Vector3(cx - 0.22 + i * 0.44, y + 2.87, -DEPTH * 0.66),
				0.085, 0.10, 0.30, SlicePalette.painted(Color(0.46, 0.30, 0.22), 0.9), 9)
	Profile.downpipe(g, -W * 0.5 + 0.22, y, 0.0, SlicePalette.iron(0.62))
	Profile.gutter(g, -W * 0.5, W * 0.5, y + 1.06, SlicePalette.iron(0.62))


# --- the frontage, at arm's length ---------------------------------------------

static func _frontage_dressing(g: Node3D) -> void:
	var iron := SlicePalette.iron()

	# the hanging sign on its scroll bracket
	var sign := Profile.hanging_sign(g, -W * 0.5 + 0.30, 3.62, 0.96, 0.46,
		SlicePalette.painted(SlicePalette.SIGN_TIMBER, 0.78), iron)
	var bx := -W * 0.5 + 0.30 + 0.96
	var by := 3.62 + 0.09 - 0.17 - 0.46
	for sz in [1.0, -1.0]:
		Profile.text(sign, Vector3(bx, by - 0.24, 0.10 + sz * 0.035), NAME, 0.085,
			SlicePalette.SIGN_CREAM, SlicePalette.serif_font(),
			0.0 if sz > 0.0 else PI)
		_cup_glyph(sign, Vector3(bx, by + 0.08, 0.10 + sz * 0.035), 0.30, sz > 0.0)

	# the wall lantern over the door
	Props.place(g, "industrial_pipe_lamp", Vector3(DOOR_X + DOOR_W * 0.5 + 0.55, 2.52, 0.10),
		0.0, Color(0.55, 0.56, 0.54))
	var lamp := OmniLight3D.new()
	lamp.light_color = SlicePalette.LAMP_WARM
	lamp.light_energy = 2.6
	lamp.omni_range = 4.2
	lamp.position = Vector3(DOOR_X + DOOR_W * 0.5 + 0.55, 2.34, 0.28)
	g.add_child(lamp)
	Build.sphere(g, lamp.position, 0.055, Mats.emissive(SlicePalette.LAMP_WARM, 7.0), 8)

	# the framed slate board on the right-hand pier, as 03 has it
	var px := W * 0.5 - PIER * 0.5
	Build.box(g, Vector3(px, 1.78, 0.055), Vector3(0.74, 1.05, 0.07),
		SlicePalette.painted(Color(0.320, 0.244, 0.172), 0.80))
	Build.box(g, Vector3(px, 1.78, 0.095), Vector3(0.62, 0.93, 0.02),
		SlicePalette.slate_board())
	Profile.text(g, Vector3(px, 1.80, 0.112), "GREAT\nCOFFEE\nKINDER\nPEOPLE\nBRIGHTER\nDAYS",
		0.068, Color(0.86, 0.85, 0.82), SlicePalette.serif_font())

	# hanging baskets, spilling over. 03 has two: a small one by the sign and a
	# large one on the right-hand pier, above the slate board, whose trailing
	# stems reach down past the board's top corner.
	_basket(g, -W * 0.5 + PIER * 0.5, 2.66, 0.30, 0.95, 1213)
	_basket(g, W * 0.5 - PIER * 0.5, 2.62, 0.30, 0.60, 1907)

	# ivy on both piers, which are the plate's strongest soft edges
	_trailing(g, Vector3(-W * 0.5 + 0.12, 5.9, 0.10), 0.55, 3.2, 5501)
	_trailing(g, Vector3(W * 0.5 - 0.20, 5.6, 0.10), 0.50, 2.2, 5519)


## A hanging basket on a wall bracket: a coir bowl, flowers in its top, and
## trailing stems below. `r` is the bowl's rim radius.
static func _basket(g: Node3D, hx: float, y: float, r: float, drop: float,
		seed_v: int) -> void:
	var iron := SlicePalette.iron()
	var out := 0.14 + r
	Build.cyl(g, Vector3(hx, y + 0.22, out), 0.012, 0.012, 0.52, iron, 5)
	Build.box(g, Vector3(hx, y + 0.50, out * 0.5 + 0.03), Vector3(0.09, 0.09, out), iron)
	# a bowl, not a cone: a wide rim course over a rounded underside
	var coir := SlicePalette.painted(Color(0.372, 0.294, 0.212), 0.92)
	Build.cyl(g, Vector3(hx, y + r * 0.30, out), r, r * 0.86, r * 0.30, coir, 14)
	Build.cyl(g, Vector3(hx, y, out), r * 0.86, r * 0.40, r * 0.30, coir, 14)
	Props.flowerbed(g, Vector3(hx, y + r * 0.62, out), r * 1.75, r * 1.40, seed_v)
	_trailing(g, Vector3(hx, y + 0.04, out), r * 1.6, drop, seed_v + 2094)


## A curtain of trailing foliage cards, for baskets, window boxes and climbers.
static func _trailing(g: Node3D, top: Vector3, spread: float, drop: float,
		seed_v: int) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	var mat := Mats.card(ProcGen.leaf_card(seed_v, Color(0.286, 0.430, 0.184)), 0.9)
	var n := int(spread * drop * 26.0) + 6
	for i in range(n):
		var t := pow(rng.randf(), 0.7)
		var p := top + Vector3(rng.randf_range(-0.5, 0.5) * spread, -t * drop,
			rng.randf_range(-0.22, 0.22))
		var s := 0.30 * (1.0 - 0.35 * t)
		Build.card(g, p, s, s, mat, rng.randf() * TAU, rng.randf_range(-0.4, 0.4), false)


## The cup on the hanging sign. Drawn from primitives rather than a texture so
## it stays legible at the two metres a person actually reads a sign from.
static func _cup_glyph(parent: Node3D, c: Vector3, size: float, front: bool) -> void:
	var m := SlicePalette.painted(SlicePalette.SIGN_CREAM, 0.80)
	var z := c.z + (0.004 if front else -0.004)
	Build.box(parent, Vector3(c.x, c.y - size * 0.06, z),
		Vector3(size * 0.62, size * 0.40, 0.012), m)
	Build.box(parent, Vector3(c.x, c.y - size * 0.32, z),
		Vector3(size * 0.86, size * 0.09, 0.012), m)
	Build.box(parent, Vector3(c.x + size * 0.40, c.y - size * 0.04, z),
		Vector3(size * 0.16, size * 0.18, 0.012), m)
	for i in range(3):
		Build.box(parent, Vector3(c.x - size * 0.16 + i * size * 0.16,
			c.y + size * 0.30, z), Vector3(size * 0.05, size * 0.26, 0.012), m)
