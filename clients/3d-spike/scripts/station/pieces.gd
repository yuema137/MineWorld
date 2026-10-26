## The station pieces (S29-e §5.3; `ARC-192`'s note of 2026-10-20): a bench row, a kiosk, a ticket
## window bay, a departures board and a clock. Each is a static function of its SIZES in metres and a
## STYLE, and returns a `Node3D`; none reads a kit, a place or a world, so a second building (S33-e's
## timber halt) builds the same pieces at village scale with its own materials.
##
## A piece is drawn in its own frame: the origin on the floor at its footprint's centre (a wall piece:
## at the foot of the wall face it hangs on), its FRONT toward -Z, length along X. The caller places it.
##
## style = { wood, metal, plaster, glass, accent, dark: Material } (`style_of` makes the depot's and the
## halt's). A fixture -- kiosk, window, board, clock -- carries `mw_fixture` (its field of
## `MineWorldStationState` / `MineWorldStationClock`) and is updated by `show(node, state)`: the state
## decides what it shows, and the piece decides nothing a world owns. Toggled parts are surface-material
## meshes, which `SliceBatch` leaves alone, so batching never bakes a shutter both up and down.
class_name StationPieces
extends RefCounted

const CATEGORY := "station.fitout"
## A bench's iron standards are at most this far apart (design; the NRHP's "long oak benches").
const STANDARD_PITCH := 1.5
## A seat's depth and its back's height above the seat (design; VISUAL_SLICE §3.4 seats 0.42-0.47 m).
const SEAT_DEPTH := 0.42
const BACK_RISE := 0.42
## The ticket window's openings: width and height above the counter (design).
const OPENING := Vector2(0.80, 0.85)
## Text: the board's heading and body size, metres per line (design: legible at 5 m).
const HEAD_M := 0.22
const BODY_M := 0.17


## The depot's materials (oak, bronze, plaster) or the halt's (`timber`: painted boards, black iron).
static func style_of(name: String) -> Dictionary:
	if name == "timber":
		return {"wood": Mats.pbr("brown_planks_09", 1.6, Color(0.78, 0.62, 0.46), 0.85),
			"metal": SlicePalette.iron(), "plaster": SlicePalette.painted(Color(0.86, 0.82, 0.70)),
			"glass": SlicePalette.dead_glass(), "accent": SlicePalette.painted(Color(0.22, 0.30, 0.24)),
			"dark": Mats.paint(Color(0.08, 0.09, 0.09), 0.7)}
	return {"wood": Mats.pbr("brown_planks_09", 1.2, Color(0.98, 0.72, 0.48), 0.70),
		"metal": SlicePalette.brass(), "plaster": SlicePalette.plaster_interior(),
		"glass": SlicePalette.dead_glass(), "accent": SlicePalette.painted(Color(0.36, 0.20, 0.12)),
		"dark": Mats.paint(Color(0.07, 0.08, 0.08), 0.7)}


static func _root(parent: Node3D, nm: String, fixture := "") -> Node3D:
	var n := Node3D.new()
	n.name = nm
	n.set_meta("mw_category", CATEGORY)
	if fixture != "":
		n.set_meta("mw_fixture", fixture)
	parent.add_child(n)
	return n


## A box whose material is on its mesh, not an override: `SliceBatch` does not merge it.
static func live_box(parent: Node3D, centre: Vector3, size: Vector3, mat: Material) -> MeshInstance3D:
	var bm := BoxMesh.new()
	bm.size = size
	bm.material = mat
	var mi := MeshInstance3D.new()
	mi.mesh = bm
	mi.position = centre
	parent.add_child(mi)
	return mi


## A world furnishing drawn by a piece (`furniture/furnishings.yaml`'s `piece`): `size` = (length,
## depth) of its footprint in the piece's own frame, `height` = its solid's height. Null for a name no
## builder has. The piece's footprint is the solid's: nothing is added beyond it but a back's lean.
static func furnishing(parent: Node3D, piece: String, size: Vector2, height: float, style: Dictionary) -> Node3D:
	match piece:
		"bench_row":
			return bench_row(parent, size.x, size.y, height, style)
		"kiosk":
			return kiosk(parent, size, height, style)
	return null


# --- bench row -------------------------------------------------------------------------------------

## A long oak bench `length` x `depth`, seat at `seat_h`, its back on the +Z side: a slatted seat, a
## back to `seat_h + BACK_RISE`, and an iron standard with an armrest every STANDARD_PITCH at most.
static func bench_row(parent: Node3D, length: float, depth: float, seat_h: float, style: Dictionary) -> Node3D:
	var n := _root(parent, "bench_row")
	var seat_z := -depth * 0.5 + SEAT_DEPTH * 0.5 + 0.02
	for k in 3:
		var slat_z := seat_z - SEAT_DEPTH * 0.5 + SEAT_DEPTH * (k + 0.5) / 3.0
		Build.box(n, Vector3(0, seat_h - 0.02, slat_z), Vector3(length - 0.06, 0.035, SEAT_DEPTH / 3.0 - 0.012),
			style["wood"])
	var back_z := seat_z + SEAT_DEPTH * 0.5 + 0.03
	for k in 3:
		var y := seat_h + 0.10 + k * 0.12
		Build.box(n, Vector3(0, y, back_z + k * 0.012), Vector3(length - 0.06, 0.09, 0.025), style["wood"])
	var count := maxi(2, int(ceil(length / STANDARD_PITCH)) + 1)
	for i in count:
		var x := -length * 0.5 + 0.04 + (length - 0.08) * float(i) / float(count - 1)
		Build.box(n, Vector3(x, (seat_h - 0.04) * 0.5, seat_z), Vector3(0.05, seat_h - 0.04, SEAT_DEPTH - 0.04),
			style["metal"])
		Build.box(n, Vector3(x, seat_h + BACK_RISE * 0.5, back_z), Vector3(0.05, BACK_RISE + 0.04, 0.05),
			style["metal"])
		Build.box(n, Vector3(x, seat_h + 0.22, seat_z), Vector3(0.06, 0.04, SEAT_DEPTH), style["metal"])
	n.set_meta("seat_h", seat_h)
	return n


# --- kiosk -----------------------------------------------------------------------------------------

## A timber news kiosk `size.x` x `size.y`, `height` tall, its counter at the front: a panelled body to
## the counter, corner posts, a roof with a shallow awning, and a roll shutter over the front opening.
## Shut: the shutter is down. Open: it is up, papers on the counter and a rack of them inside.
static func kiosk(parent: Node3D, size: Vector2, height: float, style: Dictionary) -> Node3D:
	var n := _root(parent, "kiosk", "kiosk")
	var w := size.x
	var d := size.y
	var counter := 1.05
	var roof_y := height - 0.25
	Build.box(n, Vector3(0, counter * 0.5, 0), Vector3(w - 0.04, counter, d - 0.04), style["wood"])
	Build.box(n, Vector3(0, counter + 0.02, -0.02), Vector3(w, 0.04, d), style["accent"])
	for sx: float in [-1.0, 1.0]:
		for sz: float in [-1.0, 1.0]:
			Build.box(n, Vector3(sx * (w * 0.5 - 0.05), height * 0.5, sz * (d * 0.5 - 0.05)),
				Vector3(0.10, height, 0.10), style["wood"])
	# the back and sides above the counter, closed
	Build.box(n, Vector3(0, (counter + roof_y) * 0.5, d * 0.5 - 0.03), Vector3(w - 0.1, roof_y - counter, 0.04),
		style["wood"])
	for sx: float in [-1.0, 1.0]:
		Build.box(n, Vector3(sx * (w * 0.5 - 0.03), (counter + roof_y) * 0.5, 0), Vector3(0.04, roof_y - counter, d - 0.1),
			style["wood"])
	Build.box(n, Vector3(0, roof_y + 0.12, -0.10), Vector3(w + 0.30, 0.24, d + 0.50), style["accent"])
	var shut := Node3D.new()
	shut.name = "Shut"
	n.add_child(shut)
	live_box(shut, Vector3(0, (counter + roof_y) * 0.5, -d * 0.5 + 0.03), Vector3(w - 0.12, roof_y - counter, 0.03),
		style["plaster"])
	var open := Node3D.new()
	open.name = "Open"
	n.add_child(open)
	live_box(open, Vector3(0, roof_y - 0.10, -d * 0.5 + 0.03), Vector3(w - 0.12, 0.20, 0.06), style["plaster"])
	for i in 4:
		var x := -w * 0.3 + w * 0.2 * i
		live_box(open, Vector3(x, counter + 0.06 + 0.01 * (i % 2), -d * 0.5 + 0.18), Vector3(0.30, 0.04 + 0.02 * (i % 2), 0.22),
			Mats.paint(Color(0.90, 0.88, 0.80), 0.9))
	for k in 3:
		live_box(open, Vector3(0, counter + 0.35 + k * 0.30, d * 0.5 - 0.12), Vector3(w - 0.4, 0.22, 0.06),
			Mats.paint(Color(0.84, 0.80, 0.70), 0.9))
	return n


# --- ticket window bay -------------------------------------------------------------------------------

## A ticket counter `width` wide at `counter_h`, `counter_depth` deep, and a carved timber screen above
## it to `screen_h` with `grilles` brass grilles. Behind each grille a shutter; the plate under the
## middle reads `sign.window_closed` when shut. Open, the shutters are up -- and no clerk is drawn: a
## clerk is a disclosed person, drawn by the cast binding (S26).
static func window_bay(parent: Node3D, width: float, counter_h: float, counter_depth: float, screen_h: float,
		grilles: int, style: Dictionary) -> Node3D:
	var n := _root(parent, "ticket_window", "window")
	Build.box(n, Vector3(0, counter_h * 0.5, 0), Vector3(width, counter_h, counter_depth), style["wood"])
	Build.box(n, Vector3(0, counter_h + 0.025, -0.02), Vector3(width + 0.04, 0.05, counter_depth + 0.06),
		style["accent"])
	var z := counter_depth * 0.5 - 0.06
	var top := counter_h + 0.05
	var pitch := width / float(grilles)
	var shut := Node3D.new()
	shut.name = "Shut"
	n.add_child(shut)
	var open := Node3D.new()
	open.name = "Open"
	n.add_child(open)
	# the screen: a head over the openings, the piers between them, a sill under them
	Build.box(n, Vector3(0, (top + OPENING.y + 0.15 + screen_h) * 0.5, z),
		Vector3(width, screen_h - top - OPENING.y - 0.15, 0.10), style["wood"])
	Build.box(n, Vector3(0, top + 0.075, z), Vector3(width, 0.15, 0.10), style["wood"])
	var edge := -width * 0.5
	for i in grilles + 1:
		var stop := (-width * 0.5 + pitch * (i + 0.5) - OPENING.x * 0.5) if i < grilles else width * 0.5
		Build.box(n, Vector3((edge + stop) * 0.5, top + 0.15 + OPENING.y * 0.5, z),
			Vector3(stop - edge, OPENING.y, 0.10), style["wood"])
		edge = stop + OPENING.x
	for i in grilles:
		var cx := -width * 0.5 + pitch * (i + 0.5)
		var sill := top + 0.15
		for b in 7:
			var bx := cx - OPENING.x * 0.5 + OPENING.x * (b + 0.5) / 7.0
			Build.box(n, Vector3(bx, sill + OPENING.y * 0.5, z - 0.03), Vector3(0.012, OPENING.y, 0.012), style["metal"])
		Build.box(n, Vector3(cx, sill + OPENING.y * 0.62, z - 0.03), Vector3(OPENING.x, 0.015, 0.015), style["metal"])
		live_box(shut, Vector3(cx, sill + OPENING.y * 0.5, z + 0.04), Vector3(OPENING.x, OPENING.y, 0.02),
			style["accent"])
		live_box(open, Vector3(cx, sill + OPENING.y + 0.03, z + 0.04), Vector3(OPENING.x, 0.06, 0.02), style["accent"])
	# the plate, on the screen's sill under the middle of the openings
	live_box(shut, Vector3(0, top + 0.075, z - 0.06), Vector3(0.70, 0.13, 0.01), style["dark"])
	var plate := Build.label(shut, Vector3(0, top + 0.075, z - 0.07), "", 0.09, Color(0.94, 0.88, 0.70), PI, 48)
	plate.name = "Plate"
	plate.set_meta("text_key", "sign.window_closed")
	return n


# --- departures board --------------------------------------------------------------------------------

## A framed board `width` x `height`, its centre at the origin, its face toward -Z: the heading
## `sign.departures` and a body that reads `sign.no_service`, or one row per disclosed line.
static func board(parent: Node3D, width: float, height: float, style: Dictionary) -> Node3D:
	var n := _root(parent, "departures_board", "board")
	Build.box(n, Vector3(0, 0, 0.04), Vector3(width + 0.16, height + 0.16, 0.06), style["wood"])
	Build.box(n, Vector3(0, 0, 0.0), Vector3(width, height, 0.03), style["dark"])
	Build.box(n, Vector3(0, height * 0.5 - HEAD_M * 1.6, -0.018), Vector3(width * 0.9, 0.012, 0.004), style["metal"])
	var head := Build.label(n, Vector3(0, height * 0.5 - HEAD_M * 0.9, -0.02), "", HEAD_M, Color(0.96, 0.86, 0.56), PI, 64)
	head.name = "Heading"
	head.set_meta("text_key", "sign.departures")
	var body := Build.label(n, Vector3(0, -HEAD_M * 0.6, -0.02), "", BODY_M, Color(0.92, 0.92, 0.88), PI, 64)
	body.name = "Body"
	body.width = (width * 0.9) / body.pixel_size
	body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	return n


# --- clock -------------------------------------------------------------------------------------------

## A round wall clock of `diameter`, its centre at the origin, its face toward -Z: a bronze rim, a
## white face with twelve marks and, when the world keeps a calendar, two hands.
static func clock(parent: Node3D, diameter: float, style: Dictionary) -> Node3D:
	var n := _root(parent, "clock", "clock")
	var r := diameter * 0.5
	var face := Build.cyl(n, Vector3.ZERO, r, r, 0.04, Mats.paint(Color(0.94, 0.92, 0.86), 0.5), 32)
	face.transform = Transform3D(Basis(Vector3.RIGHT, PI * 0.5), Vector3(0, 0, 0))
	var rim := Build.cyl(n, Vector3.ZERO, r + 0.04, r + 0.04, 0.03, style["metal"], 32)
	rim.transform = Transform3D(Basis(Vector3.RIGHT, PI * 0.5), Vector3(0, 0, 0.012))
	for k in 12:
		var a := TAU * k / 12.0
		var mark := Build.box(n, Vector3(sin(a) * r * 0.84, cos(a) * r * 0.84, -0.025), Vector3(0.018, r * 0.12, 0.005),
			style["dark"])
		mark.basis = Basis(Vector3.BACK, -a)
	var hands := Node3D.new()
	hands.name = "Hands"
	n.add_child(hands)
	for spec: Array in [["Hour", r * 0.50, 0.035], ["Minute", r * 0.78, 0.022]]:
		var pivot := Node3D.new()
		pivot.name = spec[0]
		pivot.position = Vector3(0, 0, -0.035)
		hands.add_child(pivot)
		live_box(pivot, Vector3(0, spec[1] * 0.5 - 0.02, 0), Vector3(spec[2], spec[1], 0.006), style["dark"])
	return n


# --- the fixtures' state -----------------------------------------------------------------------------

## Shows `state` (MineWorldStationState's dictionary plus `clock`, seconds into the day or -1) on a
## fixture built here. A node with no `mw_fixture` is left alone.
static func show(n: Node3D, state: Dictionary) -> void:
	var kind := String(n.get_meta("mw_fixture", ""))
	match kind:
		"kiosk", "window":
			var open := String(state.get(kind, MineWorldStationState.SHUT)) == MineWorldStationState.OPEN
			(n.get_node("Open") as Node3D).visible = open
			(n.get_node("Shut") as Node3D).visible = not open
		"board":
			(n.get_node("Body") as Label3D).text = board_text(state.get("board", {}))
		"clock":
			var t := int(state.get("clock", MineWorldStationClock.NO_HANDS))
			var hands := n.get_node("Hands") as Node3D
			hands.visible = t >= 0
			if t >= 0:
				# seen from the front (-Z), a positive turn about +Z is clockwise
				(hands.get_node("Hour") as Node3D).basis = Basis(Vector3.BACK, TAU * float(t % 43200) / 43200.0)
				(hands.get_node("Minute") as Node3D).basis = Basis(Vector3.BACK, TAU * float(t % 3600) / 3600.0)
	for label in n.find_children("*", "Label3D", true, false):
		if label.has_meta("text_key"):
			(label as Label3D).text = MineWorldText.text(String(label.get_meta("text_key")))


## The board's body: the catalog's `sign.no_service`, or one row per disclosed line with its next
## times (`{line, next: [world seconds]}`, the provisional shape of Q-S29e-3).
static func board_text(b: Variant) -> String:
	if typeof(b) != TYPE_DICTIONARY or String(b.get("state", "")) != MineWorldStationState.LINES:
		return MineWorldText.text("sign.no_service")
	var rows := PackedStringArray()
	for line in b.get("lines", []):
		if typeof(line) != TYPE_DICTIONARY:
			continue
		var name: Variant = line.get("line", "")
		var label := String(name.get("entity", "")) if typeof(name) == TYPE_DICTIONARY else str(name)
		var times := PackedStringArray()
		for t in line.get("next", []):
			times.append(MineWorldClockFormat.day_time(int(t)))
		rows.append("%s   %s" % [label, "  ".join(times)])
	return MineWorldText.text("sign.no_service") if rows.is_empty() else "\n".join(rows)
