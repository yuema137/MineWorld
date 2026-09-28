## Architectural profiles: the reason a box stops reading as a box.
##
## The measured gap between the spike's street and `05_main_street_golden_hour`
## was not texture resolution. It was that every opening in the spike is a flat
## rectangle cut in a tiled plane, and every opening in the plate has a reveal,
## a sill, a head and a frame that stand proud of the wall. Depth at the
## openings is what gives a facade its self-shadowing, and self-shadowing under
## a 17-degree sun is most of what "golden hour" actually looks like.
##
## Everything here is small stacked boxes. No CSG, no lathe, no imported kit --
## `DEP-8` records that no semi-realistic modular building kit exists under an
## open licence, and that the route is box geometry at correct proportions
## dressed with CC0 materials. This file is that route made reusable.
##
## FRAME. Every helper works in a facade-local frame:
##
##     +x  along the frontage, left to right facing the building
##     +y  up from the pavement
##     +z  out of the facade, toward the street; z = 0 is the wall face
##
## A caller places the whole building node; nothing here knows where north is.
class_name Profile
extends RefCounted


## A run of text on a wall, in a named font. Label3D rather than a texture:
## the reference plates are full of legible signage, and a sign baked into a
## 1k texture is unreadable at the distance a person actually reads a sign from.
static func text(parent: Node3D, pos: Vector3, txt: String, cap_height: float,
		c: Color, font: Font, yaw := 0.0, align := HORIZONTAL_ALIGNMENT_CENTER,
		outline := 0.0) -> Label3D:
	var l := Label3D.new()
	l.font = font
	l.text = txt
	l.font_size = 96
	# Label3D sizes by line height, which for a serif face is about 1.38x the
	# cap height. Callers think in cap heights, because that is what a sign's
	# proportions are drawn to.
	l.pixel_size = cap_height * 1.38 / 96.0
	l.modulate = c
	l.outline_size = int(outline * 96.0)
	l.outline_modulate = Color(0, 0, 0, 0.55)
	l.shaded = true
	l.double_sided = false
	l.horizontal_alignment = align
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	l.texture_filter = BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS_ANISOTROPIC
	l.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(l)
	return l


## A stepped cornice: several courses, each projecting further than the one
## below, so the whole thing throws a deep shadow line across the facade.
## `proj` is how far the topmost course stands out from the wall.
static func cornice(parent: Node3D, x0: float, x1: float, y: float,
		proj: float, mat: Material, courses := 4, course_h := 0.075) -> float:
	var w := x1 - x0
	var cx := (x0 + x1) * 0.5
	for i in range(courses):
		var f := float(i + 1) / float(courses)
		var p := proj * (0.28 + 0.72 * f)
		var h := course_h * (1.0 if i < courses - 1 else 1.45)
		Build.box(parent, Vector3(cx, y + course_h * i + h * 0.5, p * 0.5),
			Vector3(w + p * 0.6, h, p), mat)
	return y + course_h * courses


## A dentil course: the row of small blocks under a cornice. Cheap, and it is
## one of the strongest "this is a real building" cues at street distance.
static func dentils(parent: Node3D, x0: float, x1: float, y: float, z: float,
		mat: Material, pitch := 0.24, block := 0.11, depth := 0.09,
		height := 0.11) -> void:
	var n := int((x1 - x0) / pitch)
	var start := x0 + ((x1 - x0) - (n - 1) * pitch) * 0.5
	for i in range(n):
		Build.box(parent, Vector3(start + i * pitch, y + height * 0.5, z + depth * 0.5),
			Vector3(block, height, depth), mat)


## A horizontal band standing proud of the wall: a string course, a plinth
## capping, a fascia rail.
static func band(parent: Node3D, x0: float, x1: float, y: float, h: float,
		proj: float, mat: Material) -> void:
	Build.box(parent, Vector3((x0 + x1) * 0.5, y + h * 0.5, proj * 0.5),
		Vector3(x1 - x0, h, proj), mat)


## A pilaster: a flat column standing proud of the facade, with a base and a
## cap. The cafe frontage has one at each end and the plate's shopfront is
## unreadable without them.
static func pilaster(parent: Node3D, x: float, y0: float, h: float, w: float,
		proj: float, mat: Material, cap_mat: Material = null) -> void:
	var cm: Material = cap_mat if cap_mat != null else mat
	Build.box(parent, Vector3(x, y0 + 0.10, proj * 0.5),
		Vector3(w + 0.07, 0.20, proj + 0.035), cm)                       # base
	Build.box(parent, Vector3(x, y0 + h * 0.5, proj * 0.5),
		Vector3(w, h, proj), mat)                                        # shaft
	Build.box(parent, Vector3(x, y0 + h - 0.09, proj * 0.5 + 0.012),
		Vector3(w + 0.055, 0.18, proj + 0.024), cm)                      # cap
	Build.box(parent, Vector3(x, y0 + h + 0.035, proj * 0.5 + 0.02),
		Vector3(w + 0.09, 0.07, proj + 0.04), cm)                        # abacus


## A corbel bracket under a cornice or a window box: a small wedge.
static func corbel(parent: Node3D, x: float, y: float, proj: float,
		mat: Material, w := 0.10, h := 0.26) -> void:
	Build.box(parent, Vector3(x, y - h * 0.5, proj * 0.5), Vector3(w, h, proj), mat)
	Build.prism(parent, Vector3(x, y - h - 0.055, proj * 0.36),
		Vector3(w, 0.11, proj * 0.72), mat, PI)


## A punched masonry window, with everything that makes it read as one:
## a recessed reveal, a projecting sill on two small stops, a head, a painted
## frame set back in the reveal, glazing bars, and the pane.
##
## `depth` is how far the frame sits back from the wall face (0.14-0.22 m is a
## masonry wall; anything under 0.06 reads as a sticker).
static func punched_window(parent: Node3D, cx: float, y0: float, w: float, h: float,
		frame_c: Color, glass: Material, stone: Material,
		depth := 0.17, bars_x := 1, bars_y := 2, sill := true) -> void:
	var frame := SlicePalette.painted(frame_c, 0.50)
	var jamb := 0.055

	# reveal: the four faces of the hole, so the wall shows its thickness
	Build.box(parent, Vector3(cx - w * 0.5 - 0.03, y0 + h * 0.5, -depth * 0.5),
		Vector3(0.06, h, depth), stone)
	Build.box(parent, Vector3(cx + w * 0.5 + 0.03, y0 + h * 0.5, -depth * 0.5),
		Vector3(0.06, h, depth), stone)
	Build.box(parent, Vector3(cx, y0 + h + 0.03, -depth * 0.5),
		Vector3(w + 0.12, 0.06, depth), stone)
	Build.box(parent, Vector3(cx, y0 - 0.03, -depth * 0.5),
		Vector3(w + 0.12, 0.06, depth), stone)

	# head: a flat arch or a lintel standing slightly proud
	Build.box(parent, Vector3(cx, y0 + h + 0.085, 0.022),
		Vector3(w + 0.30, 0.17, 0.045), stone)

	if sill:
		Build.box(parent, Vector3(cx, y0 - 0.07, 0.045),
			Vector3(w + 0.34, 0.09, 0.19), stone)
		Build.box(parent, Vector3(cx - w * 0.5 - 0.06, y0 - 0.17, 0.028),
			Vector3(0.09, 0.11, 0.11), stone)
		Build.box(parent, Vector3(cx + w * 0.5 + 0.06, y0 - 0.17, 0.028),
			Vector3(0.09, 0.11, 0.11), stone)

	# frame, set back in the reveal
	var zf := -depth + 0.045
	Build.box(parent, Vector3(cx, y0 + jamb * 0.5, zf), Vector3(w, jamb, 0.07), frame)
	Build.box(parent, Vector3(cx, y0 + h - jamb * 0.5, zf), Vector3(w, jamb, 0.07), frame)
	Build.box(parent, Vector3(cx - w * 0.5 + jamb * 0.5, y0 + h * 0.5, zf),
		Vector3(jamb, h, 0.07), frame)
	Build.box(parent, Vector3(cx + w * 0.5 - jamb * 0.5, y0 + h * 0.5, zf),
		Vector3(jamb, h, 0.07), frame)

	# glazing bars
	for i in range(bars_x):
		var bx := cx - w * 0.5 + w * float(i + 1) / float(bars_x + 1)
		Build.box(parent, Vector3(bx, y0 + h * 0.5, zf), Vector3(0.032, h, 0.055), frame)
	for j in range(bars_y):
		var by := y0 + h * float(j + 1) / float(bars_y + 1)
		Build.box(parent, Vector3(cx, by, zf), Vector3(w, 0.032, 0.055), frame)

	# the pane, a hair behind the bars
	Build.box(parent, Vector3(cx, y0 + h * 0.5, zf - 0.03),
		Vector3(w - jamb, h - jamb, 0.012), glass)


## A window box on two corbels, with soil. The planting itself is the caller's.
static func window_box(parent: Node3D, cx: float, y: float, w: float,
		timber: Material) -> Vector3:
	var d := 0.30
	Build.box(parent, Vector3(cx, y + 0.115, d * 0.5), Vector3(w, 0.23, d), timber)
	Build.box(parent, Vector3(cx, y + 0.235, d * 0.5), Vector3(w + 0.05, 0.035, d + 0.05), timber)
	for sx in [-1.0, 1.0]:
		corbel(parent, cx + sx * (w * 0.5 - 0.10), y, d * 0.72, timber, 0.07, 0.15)
	Build.box(parent, Vector3(cx, y + 0.20, d * 0.5), Vector3(w - 0.06, 0.05, d - 0.06),
		Mats.paint(Color(0.130, 0.098, 0.074), 0.98))
	return Vector3(cx, y + 0.225, d * 0.5)


## A cast-iron downpipe with its hopper and two brackets.
static func downpipe(parent: Node3D, x: float, y_top: float, y_bot: float,
		mat: Material) -> void:
	var h := y_top - y_bot
	Build.cyl(parent, Vector3(x, y_bot, 0.095), 0.041, 0.041, h, mat, 10)
	Build.box(parent, Vector3(x, y_top + 0.10, 0.10), Vector3(0.20, 0.20, 0.17), mat)
	for f in [0.22, 0.62, 0.92]:
		Build.box(parent, Vector3(x, y_bot + h * f, 0.055), Vector3(0.10, 0.045, 0.06), mat)
	Build.cyl(parent, Vector3(x, y_bot, 0.095), 0.055, 0.062, 0.14, mat, 10)


## A projecting eaves gutter along the head of a wall.
static func gutter(parent: Node3D, x0: float, x1: float, y: float, mat: Material) -> void:
	Build.box(parent, Vector3((x0 + x1) * 0.5, y, 0.135),
		Vector3(x1 - x0, 0.11, 0.13), mat)
	Build.box(parent, Vector3((x0 + x1) * 0.5, y + 0.075, 0.185),
		Vector3(x1 - x0, 0.04, 0.045), mat)


## A pitched roof with eaves that overhang, plus a ridge. `depth` runs in -z.
static func pitched_roof(parent: Node3D, cx: float, w: float, y: float,
		depth: float, rise: float, mat: Material, overhang := 0.32) -> void:
	var d := depth + overhang * 2.0
	var ww := w + overhang * 2.0
	Build.prism(parent, Vector3(cx, y + rise * 0.5, -depth * 0.5),
		Vector3(d, rise, ww), mat, PI * 0.5)
	# a ridge tile so the apex is not a knife edge
	Build.box(parent, Vector3(cx, y + rise, -depth * 0.5),
		Vector3(0.16, 0.10, d), mat)


## A flat roof behind a parapet, with a coping stone on top of the parapet.
static func parapet(parent: Node3D, x0: float, x1: float, y: float, depth: float,
		wall: Material, coping: Material, h := 0.62) -> void:
	Build.box(parent, Vector3((x0 + x1) * 0.5, y + h * 0.5, 0.045),
		Vector3(x1 - x0, h, 0.30), wall)
	Build.box(parent, Vector3((x0 + x1) * 0.5, y + h + 0.045, 0.045),
		Vector3(x1 - x0 + 0.10, 0.09, 0.40), coping)
	Build.box(parent, Vector3((x0 + x1) * 0.5, y + 0.06, -depth * 0.5),
		Vector3(x1 - x0, 0.12, depth), coping)


## A framed panel: a recessed field inside a raised bead. Used on shopfront
## stallrisers, door leaves, counter fronts and pilaster shafts. This one
## detail is most of the difference between "painted timber" and "green box".
static func panel(parent: Node3D, cx: float, cy: float, w: float, h: float, z: float,
		mat: Material, bead := 0.055, relief := 0.022) -> void:
	Build.box(parent, Vector3(cx, cy, z - relief * 0.5), Vector3(w, h, 0.05), mat)
	Build.box(parent, Vector3(cx, cy + h * 0.5 - bead * 0.5, z), Vector3(w, bead, 0.05), mat)
	Build.box(parent, Vector3(cx, cy - h * 0.5 + bead * 0.5, z), Vector3(w, bead, 0.05), mat)
	Build.box(parent, Vector3(cx - w * 0.5 + bead * 0.5, cy, z), Vector3(bead, h, 0.05), mat)
	Build.box(parent, Vector3(cx + w * 0.5 - bead * 0.5, cy, z), Vector3(bead, h, 0.05), mat)


## A straight canvas awning on folding arms, with a valance and printed text.
## `proj` is how far it reaches over the pavement; the plates put it at 1.3-1.6 m.
static func awning(parent: Node3D, x0: float, x1: float, y: float, proj: float,
		c: Color, drop := 0.34) -> Node3D:
	var n := Node3D.new()
	parent.add_child(n)
	var canvas := SlicePalette.canvas(c)
	var w := x1 - x0
	var cx := (x0 + x1) * 0.5
	var slope := atan2(drop, proj)
	var slant := sqrt(proj * proj + drop * drop)
	var mi := Build.box(n, Vector3(cx, y - drop * 0.5, proj * 0.5),
		Vector3(w, 0.035, slant), canvas)
	mi.rotate_object_local(Vector3.RIGHT, -slope)
	# valance: the vertical flap at the front edge
	Build.box(n, Vector3(cx, y - drop - 0.11, proj), Vector3(w, 0.22, 0.03), canvas)
	# arms
	var arm := SlicePalette.iron()
	for sx in [-0.5, 0.5]:
		var ax: float = cx + sx * (w - 0.30)
		var a := Build.cyl(n, Vector3(ax, y - drop, proj), 0.024, 0.024, slant, arm, 6)
		a.transform = Transform3D(Basis(), Vector3(ax, y - drop * 0.5, proj * 0.5))
		a.rotate_object_local(Vector3.RIGHT, PI * 0.5 - slope)
	Build.box(n, Vector3(cx, y + 0.03, 0.06), Vector3(w + 0.10, 0.10, 0.12), arm)
	return n


## A projecting hanging sign on a wrought-iron scroll bracket -- 03's single
## most recognisable object after the fascia itself.
static func hanging_sign(parent: Node3D, x: float, y: float, reach: float,
		radius: float, board: Material, iron: Material) -> Node3D:
	var n := Node3D.new()
	parent.add_child(n)
	# bracket: a wall plate, a horizontal arm, a diagonal stay and two scrolls
	Build.box(n, Vector3(x, y, 0.045), Vector3(0.09, 0.62, 0.09), iron)
	var arm := Build.cyl(n, Vector3.ZERO, 0.026, 0.026, reach, iron, 8)
	arm.transform = Transform3D(Basis(Vector3.FORWARD, PI * 0.5),
		Vector3(x + reach * 0.5, y + 0.26, 0.10))
	var stay := Build.cyl(n, Vector3.ZERO, 0.020, 0.020, reach * 0.86, iron, 6)
	stay.transform = Transform3D(Basis(Vector3.FORWARD, PI * 0.72),
		Vector3(x + reach * 0.42, y - 0.02, 0.10))
	for i in range(5):
		var a := TAU * float(i) / 5.0
		Build.cyl(n, Vector3(x + reach * 0.30 + cos(a) * 0.085,
			y + 0.14 + sin(a) * 0.085 - 0.02, 0.10), 0.014, 0.014, 0.05, iron, 6)
	# hangers and the board
	for sx in [-1.0, 1.0]:
		Build.cyl(n, Vector3(x + reach + sx * radius * 0.62, y + 0.09 - 0.17, 0.10),
			0.011, 0.011, 0.17, iron, 5)
	var bx := x + reach
	var by := y + 0.09 - 0.17 - radius
	# the board is a disc standing vertical, facing the street, so its axis is z
	disc(n, Vector3(bx, by, 0.10), radius, 0.045, board)
	disc(n, Vector3(bx, by, 0.10), radius + 0.028, 0.022, iron)
	return n


## A disc standing vertical in the xy plane, its axis along z. Build.cyl makes
## a y-axis cylinder, so this is the same mesh laid on its side -- worth a
## helper because every sign board, wheel, clock face and lamp shade wants it.
static func disc(parent: Node3D, centre: Vector3, radius: float, thickness: float,
		mat: Material, segs := 28) -> MeshInstance3D:
	var cm := CylinderMesh.new()
	cm.top_radius = radius
	cm.bottom_radius = radius
	cm.height = thickness
	cm.radial_segments = segs
	cm.rings = 1
	var mi := MeshInstance3D.new()
	mi.mesh = cm
	mi.material_override = mat
	mi.transform = Transform3D(Basis(Vector3.RIGHT, PI * 0.5), centre)
	parent.add_child(mi)
	return mi
