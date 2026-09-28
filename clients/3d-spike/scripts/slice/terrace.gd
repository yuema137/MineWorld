## The rest of the frontage: the buildings you walk past but do not enter.
##
## `VISUAL_SLICE.md` sec.4 allows three to five of these and requires each to
## have its own massing, roofline, material and shopfront treatment -- a
## repeated unit is a defect, and a terrace of one unit stamped five times is
## the most recognisable signature of a scene built in a hurry.
##
## They are solid, not hollow: nobody goes up there, and a hollow volume would
## leak light into windows that have no room behind them. What a lit window has
## instead is a shallow recess with a warm card and a few boxes in it, so that
## looking into it from the pavement shows depth rather than a black pane.
class_name SliceTerrace
extends RefCounted

enum Roof { PARAPET, PITCHED, GABLE }

## One building. `origin` is the facade line; `yaw` turns it to face the street.
class Unit extends RefCounted:
	var x0: float
	var x1: float
	var storeys: int
	var storey_h: float
	var wall: Material
	var roof: Roof
	var roof_mat: Material
	var shopfront: bool
	var name_text: String
	var strap: String
	var front_c: Color
	var awning_c: Color
	var awning: bool
	var bays: int
	var depth: float
	var stripes: bool

	func _init(a: float, b: float) -> void:
		x0 = a
		x1 = b
		storeys = 2
		storey_h = 3.30
		wall = SlicePalette.render_cream()
		roof = Roof.PARAPET
		roof_mat = SlicePalette.pantile()
		shopfront = false
		name_text = ""
		strap = ""
		front_c = SlicePalette.CAFE_GREEN_DARK
		awning_c = SlicePalette.AWNING_GREEN
		awning = false
		bays = 3
		depth = 9.0
		stripes = false

	func width() -> float:
		return x1 - x0


static func build(parent: Node3D, u: Unit, face_z: float, yaw: float) -> Node3D:
	var g := Node3D.new()
	g.name = "Unit_%.0f" % u.x0
	# The unit is authored in a facade-local frame with +z toward the street,
	# then turned to face it. A south-side unit is the same code at yaw = PI.
	g.transform = Transform3D(Basis(Vector3.UP, yaw),
		Vector3((u.x0 + u.x1) * 0.5, SliceStreet.WALK_Y, face_z))
	parent.add_child(g)

	var w := u.width()
	var ground_h := 4.30 if u.shopfront else u.storey_h
	var top := ground_h + (u.storeys - 1) * u.storey_h

	# the mass
	Build.box(g, Vector3(0, top * 0.5, -u.depth * 0.5), Vector3(w, top, u.depth),
		u.wall, 0.0, true)

	if u.shopfront:
		_shopfront(g, u, ground_h)
	else:
		_ground_floor(g, u, ground_h)

	for s in range(u.storeys - 1):
		var y := ground_h + s * u.storey_h
		_storey(g, u, y)
		if s == 0 and u.storeys > 2:
			Profile.band(g, -w * 0.5, w * 0.5, y - 0.16, 0.18, 0.09,
				SlicePalette.painted(Color(0.780, 0.734, 0.640), 0.86))

	_roof(g, u, top)
	Profile.downpipe(g, -w * 0.5 + 0.20, top, 0.0, SlicePalette.iron(0.62))
	return g


# --- the ground storey ---------------------------------------------------------

static func _shopfront(g: Node3D, u: Unit, h: float) -> void:
	var w := u.width()
	var paint := SlicePalette.painted(u.front_c, 0.48)
	var dark := SlicePalette.painted(u.front_c.darkened(0.30), 0.46)
	var pier := 0.52
	var x_l := -w * 0.5 + pier
	var x_r := w * 0.5 - pier
	var glaze_head := 2.80
	var fascia_y := 2.92
	var fascia_h := 0.84

	Profile.pilaster(g, x_l + 0.12, 0.0, fascia_y, 0.24, 0.12, paint, dark)
	Profile.pilaster(g, x_r - 0.12, 0.0, fascia_y, 0.24, 0.12, paint, dark)

	# door to one side, window to the other
	var door_x := x_l + 0.82
	var door_w := 1.02
	for jx in [door_x - door_w * 0.5 - 0.07, door_x + door_w * 0.5 + 0.07]:
		Build.box(g, Vector3(jx, 1.12, 0.04), Vector3(0.14, 2.24, 0.18), paint)
	Build.box(g, Vector3(door_x, 2.30, 0.04), Vector3(door_w + 0.38, 0.14, 0.18), paint)
	Build.box(g, Vector3(door_x, 1.12, -0.05), Vector3(door_w, 2.22, 0.07), dark)
	Build.box(g, Vector3(door_x, 1.44, -0.02), Vector3(door_w - 0.22, 1.10, 0.016),
		SlicePalette.dead_glass())
	Build.box(g, Vector3(door_x + door_w * 0.35, 1.05, -0.09), Vector3(0.045, 0.20, 0.04),
		SlicePalette.brass())
	Build.box(g, Vector3(door_x, 0.06, 0.16), Vector3(door_w + 0.5, 0.12, 0.34),
		SlicePalette.kerbstone())

	var win_l := door_x + door_w * 0.5 + 0.22
	var win_r := x_r - 0.26
	var stall := 0.38
	Build.box(g, Vector3((win_l + win_r) * 0.5, stall * 0.5, -0.05),
		Vector3(win_r - win_l + 0.2, stall, 0.24), dark, 0.0, true)
	Build.box(g, Vector3((win_l + win_r) * 0.5, stall + 0.025, 0.04),
		Vector3(win_r - win_l + 0.26, 0.05, 0.18), paint)
	for jx in [win_l, win_r]:
		Build.box(g, Vector3(jx, (stall + glaze_head) * 0.5, 0.02),
			Vector3(0.10, glaze_head - stall, 0.14), paint)
	var mull := (win_l + win_r) * 0.5
	Build.box(g, Vector3(mull, (stall + glaze_head) * 0.5, 0.018),
		Vector3(0.08, glaze_head - stall, 0.12), paint)
	Build.box(g, Vector3((win_l + win_r) * 0.5, glaze_head + 0.06, 0.03),
		Vector3(win_r - win_l + 0.22, 0.12, 0.16), dark)

	_lit_window(g, (win_l + win_r) * 0.5, stall, glaze_head, win_r - win_l,
		u.name_text)
	Build.box(g, Vector3((win_l + win_r) * 0.5, (stall + glaze_head) * 0.5, -0.02),
		Vector3(win_r - win_l - 0.04, glaze_head - stall - 0.04, 0.014),
		SlicePalette.shop_glass())

	# fascia, lettering and cornice
	Build.box(g, Vector3(0, fascia_y + fascia_h * 0.5, 0.065),
		Vector3(w - pier * 0.8, fascia_h, 0.13), paint)
	if u.name_text != "":
		Profile.text(g, Vector3(0, fascia_y + fascia_h * 0.58, 0.145), u.name_text,
			0.33, SlicePalette.SIGN_CREAM, SlicePalette.serif_font())
	if u.strap != "":
		Profile.text(g, Vector3(0, fascia_y + fascia_h * 0.20, 0.145), u.strap,
			0.078, SlicePalette.SIGN_CREAM.darkened(0.12), SlicePalette.serif_font())
	var top := fascia_y + fascia_h
	for i in range(6):
		Profile.corbel(g, -w * 0.5 + 0.5 + (w - 1.0) * float(i) / 5.0, top + 0.02, 0.26,
			SlicePalette.painted(Color(0.208, 0.164, 0.124), 0.72), 0.10, 0.20)
	Profile.cornice(g, -w * 0.5 + 0.1, w * 0.5 - 0.1, top, 0.34,
		SlicePalette.painted(Color(0.786, 0.734, 0.632), 0.82), 3, 0.08)

	if u.awning:
		_awning(g, win_l - 0.1, win_r + 0.1, fascia_y - 0.10, u)


static func _awning(g: Node3D, x0: float, x1: float, y: float, u: Unit) -> void:
	if not u.stripes:
		var a := Profile.awning(g, x0, x1, y, 1.45, u.awning_c)
		if u.strap != "":
			Profile.text(a, Vector3((x0 + x1) * 0.5, y - 0.45, 1.02),
				u.strap, 0.085, SlicePalette.SIGN_CREAM, SlicePalette.serif_font())
			(a.get_child(a.get_child_count() - 1) as Node3D).rotate_object_local(
				Vector3.RIGHT, -atan2(0.34, 1.45))
		return
	# a striped awning, as `05` gives the bakery: alternating panels rather than
	# a texture, because the stripe pitch then matches the plate's
	var n := int((x1 - x0) / 0.42)
	var pw := (x1 - x0) / float(n)
	for i in range(n):
		var c: Color = u.awning_c if i % 2 == 0 else Color(0.90, 0.88, 0.82)
		Profile.awning(g, x0 + i * pw, x0 + (i + 1) * pw + 0.004, y, 1.45, c)


## A residential or blank ground storey: a doorcase, two windows, a plinth.
static func _ground_floor(g: Node3D, u: Unit, h: float) -> void:
	var w := u.width()
	Build.box(g, Vector3(0, 0.30, 0.045), Vector3(w, 0.60, 0.09),
		SlicePalette.kerbstone())
	var door_x := -w * 0.5 + 1.25
	var paint := SlicePalette.painted(u.front_c, 0.48)
	for jx in [door_x - 0.62, door_x + 0.62]:
		Build.box(g, Vector3(jx, 1.16, 0.075), Vector3(0.20, 2.32, 0.15),
			SlicePalette.kerbstone())
	Build.box(g, Vector3(door_x, 2.40, 0.10), Vector3(1.60, 0.20, 0.20),
		SlicePalette.kerbstone())
	Build.box(g, Vector3(door_x, 1.10, -0.06), Vector3(1.04, 2.20, 0.08), paint)
	for i in range(2):
		Profile.panel(g, door_x, 0.60 + i * 1.10, 0.74, 0.82, -0.015, paint, 0.05, 0.016)
	Build.box(g, Vector3(door_x, 2.36, -0.03), Vector3(1.00, 0.34, 0.014),
		SlicePalette.dead_glass())
	Build.box(g, Vector3(door_x + 0.38, 1.02, -0.10), Vector3(0.05, 0.18, 0.05),
		SlicePalette.brass())
	Build.box(g, Vector3(door_x, 0.07, 0.20), Vector3(1.50, 0.14, 0.40),
		SlicePalette.kerbstone())
	var n := maxi(1, int((w - 3.2) / 2.6))
	for i in range(n):
		var cx := door_x + 1.9 + (w - 3.4 - 1.9) * (float(i) + 0.5) / float(n)
		Profile.punched_window(g, cx, 1.05, 1.08, 1.90, u.front_c,
			SlicePalette.dead_glass(), u.wall, 0.18, 1, 2)


# --- upper storeys and roof ----------------------------------------------------

static func _storey(g: Node3D, u: Unit, y: float) -> void:
	var w := u.width()
	for i in range(u.bays):
		var cx := -w * 0.5 + w * (float(i) + 0.5) / float(u.bays)
		Profile.punched_window(g, cx, y + 0.78, 1.04, 1.76, u.front_c,
			SlicePalette.dead_glass(), u.wall, 0.19, 1, 2)
		# a juliet balcony on the middle bay of the tall block, as `05` has
		if u.storeys >= 4 and i == 1:
			_juliet(g, cx, y + 0.72, 1.40)
		elif i == u.bays - 1 and u.storeys == 3:
			var anchor := Profile.window_box(g, cx, y + 0.66, 1.18,
				SlicePalette.painted(Color(0.348, 0.260, 0.180), 0.82))
			Props.flowerbed(g, anchor + Vector3(0, 0.08, 0.0), 1.06, 0.20,
				int(absf(cx) * 313.0) + 11)


static func _juliet(g: Node3D, cx: float, y: float, w: float) -> void:
	var iron := SlicePalette.iron(0.5)
	Build.box(g, Vector3(cx, y - 0.04, 0.16), Vector3(w + 0.18, 0.07, 0.34),
		SlicePalette.kerbstone())
	Build.box(g, Vector3(cx, y + 0.92, 0.30), Vector3(w, 0.04, 0.04), iron)
	Build.box(g, Vector3(cx, y + 0.46, 0.30), Vector3(w, 0.025, 0.025), iron)
	var n := int(w / 0.14)
	for i in range(n):
		Build.cyl(g, Vector3(cx - w * 0.5 + 0.07 + i * 0.14, y, 0.30), 0.011, 0.011, 0.94,
			iron, 5)


static func _roof(g: Node3D, u: Unit, top: float) -> void:
	var w := u.width()
	match u.roof:
		Roof.PARAPET:
			Profile.dentils(g, -w * 0.5 + 0.2, w * 0.5 - 0.2, top - 0.15, 0.02,
				SlicePalette.painted(Color(0.762, 0.712, 0.612), 0.84), 0.28, 0.12, 0.10, 0.12)
			Profile.cornice(g, -w * 0.5, w * 0.5, top, 0.40,
				SlicePalette.painted(Color(0.792, 0.740, 0.638), 0.84), 4, 0.072)
			Profile.parapet(g, -w * 0.5, w * 0.5, top + 0.29, u.depth, u.wall,
				SlicePalette.kerbstone(), 0.58)
			Build.box(g, Vector3(w * 0.28, top + 1.70, -u.depth * 0.5),
				Vector3(0.88, 2.20, 0.74), SlicePalette.brick_red())
			Build.box(g, Vector3(w * 0.28, top + 2.84, -u.depth * 0.5),
				Vector3(1.02, 0.13, 0.88), SlicePalette.kerbstone())
		Roof.PITCHED:
			Profile.gutter(g, -w * 0.5, w * 0.5, top + 0.02, SlicePalette.iron(0.6))
			Profile.pitched_roof(g, 0.0, w, top + 0.06, u.depth, w * 0.30, u.roof_mat)
			Build.box(g, Vector3(-w * 0.22, top + w * 0.30 * 0.62, -u.depth * 0.42),
				Vector3(0.82, w * 0.30 * 1.1, 0.70), u.wall)
			Build.box(g, Vector3(-w * 0.22, top + w * 0.30 * 1.18, -u.depth * 0.42),
				Vector3(0.96, 0.13, 0.84), SlicePalette.kerbstone())
		Roof.GABLE:
			# the ridge runs front to back, so the street sees a gable end
			Profile.gutter(g, -w * 0.5, w * 0.5, top + 0.02, SlicePalette.iron(0.6))
			var rise := w * 0.36
			Build.prism(g, Vector3(0, top + rise * 0.5, -u.depth * 0.5),
				Vector3(w + 0.2, rise, u.depth), u.wall)
			for sx in [-1.0, 1.0]:
				var mi := Build.box(g, Vector3(sx * (w * 0.25 + 0.16),
					top + rise * 0.5 + 0.08, -u.depth * 0.5 + 0.18),
					Vector3(w * 0.62, 0.16, u.depth + 0.34), u.roof_mat)
				mi.rotate_object_local(Vector3.FORWARD, -sx * atan2(rise, w * 0.5))
				mi.position = Vector3(sx * w * 0.25, top + rise * 0.5, -u.depth * 0.5 + 0.18)


## A recess behind a shop window with a warm card, a shelf and a few boxes on
## it, so a non-enterable shopfront still has depth when you put your face to
## the glass. Not a billboard: it is geometry, just shallow.
static func _lit_window(g: Node3D, cx: float, y0: float, y1: float, w: float,
		nm: String) -> void:
	var d := 1.30
	var back := SlicePalette.painted(Color(0.400, 0.330, 0.268), 0.90)
	Build.box(g, Vector3(cx, (y0 + y1) * 0.5, -d), Vector3(w, y1 - y0 + 0.4, 0.08), back)
	for sx in [-1.0, 1.0]:
		Build.box(g, Vector3(cx + sx * w * 0.5, (y0 + y1) * 0.5, -d * 0.5),
			Vector3(0.08, y1 - y0 + 0.4, d), back)
	Build.box(g, Vector3(cx, y1 + 0.16, -d * 0.5), Vector3(w, 0.08, d), back)
	Build.box(g, Vector3(cx, y0 - 0.02, -d * 0.5), Vector3(w, 0.08, d),
		SlicePalette.joinery_timber())
	var rng := RandomNumberGenerator.new()
	rng.seed = int(absf(cx) * 977.0) + nm.length()
	var shelf := SlicePalette.joinery_timber()
	for s in range(3):
		var sy := y0 + 0.28 + s * 0.62
		if sy > y1 - 0.25:
			break
		Build.box(g, Vector3(cx, sy, -d * 0.62), Vector3(w - 0.2, 0.04, d * 0.5), shelf)
		for i in range(9):
			var bw := rng.randf_range(0.06, 0.15)
			Build.box(g, Vector3(cx - w * 0.5 + 0.2 + i * (w - 0.4) / 8.0,
				sy + rng.randf_range(0.09, 0.17), -d * 0.62 + rng.randf_range(-0.12, 0.12)),
				Vector3(bw, rng.randf_range(0.16, 0.30), rng.randf_range(0.06, 0.16)),
				Mats.paint(Color(rng.randf_range(0.30, 0.86), rng.randf_range(0.26, 0.74),
					rng.randf_range(0.20, 0.62)), 0.62))
	var l := OmniLight3D.new()
	l.light_color = Color(1.0, 0.84, 0.62)
	l.light_energy = 5.0
	l.omni_range = 4.6
	l.shadow_enabled = false
	l.position = Vector3(cx, y1 - 0.30, -d * 0.45)
	g.add_child(l)
