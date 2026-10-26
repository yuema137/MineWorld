## A station's inside, from its kit's `fitout` group (S29-e §5.4; R-S29e-d-1): the main place's
## transverse arches, purlins and boarded ceiling, the tiled wainscot, the chandeliers and lights, the
## departures board, the clock over the door to the open place, the leaflet rack and the bin; the walled
## room's ticket window; and the open place's platform, tracks, cuttings, fences and board
## (`depot_platform.gd`).
##
## The benches and the kiosk are NOT here: they are the world's furnishings (`body.furnishings`,
## ARC-307), drawn from the disclosure through `furniture/furnishings.yaml` with the same pieces.
##
## Every number is the kit's; every position follows from the kit's places, doors and `StationDepot.report`
## (origins, volumes). Nothing names a world, a place key or a role tag: the main place is the kit's
## `main` shell, the ticket window stands in its `annex` place and the platform is its `open` place.
## Every node is under `station.fitout` or `station.platform`.
class_name StationDepotInterior
extends RefCounted

## Arch segments per transverse arch.
const SEGMENTS := 14

## The last build, for the probes: { arches, chandeliers, fixtures: {name: Node3D}, ... }.
static var report := {}


## Builds the fit-out under `depot` (the node `StationDepot.build` returned), for its last build.
static func build(depot: Node3D, kit: StationKit) -> Node3D:
	report = {"fixtures": {}}
	if depot == null or StationDepot.report.is_empty():
		return null
	var g := StationDepot._node(depot, "Fitout", StationPieces.CATEGORY)
	var style := StationPieces.style_of(String(kit.data.get("style", "")))
	var main := kit.main_place()
	var o: Vector3 = StationDepot.report["origins"][main]
	_ceiling(g, kit, main, o)
	_wainscot(g, kit, main, o)
	_lights(g, kit, main, o)
	_fixtures(g, kit, main, o, style)
	for p in kit.place_names():
		if not StationDepot.report["origins"].has(p):
			continue
		match String(kit.place(p)["shell"]):
			"annex":
				_ticket_window(g, kit, p, style)
			"open":
				StationDepotPlatform.build(g, kit, p, _door_to(kit, p), style)
	StationFixtures.show_defaults()
	return g


## The scene yaw that turns a piece's front (-Z) toward `front`.
static func yaw_front(front: Vector3) -> float:
	return atan2(-front.x, -front.z)


static func _put(n: Node3D, pos: Vector3, front: Vector3) -> Node3D:
	n.transform = Transform3D(Basis(Vector3.UP, yaw_front(front)), pos)
	return n


## The door that leads from the main place to `place`.
static func _door_to(kit: StationKit, place: String) -> Dictionary:
	for d in kit.doors():
		if d["to"] == place and d["from"] != StationKit.OUTSIDE:
			return d
	return {}


## The main place's span along y between the outer faces of its end walls, and where its first arch bay
## starts (frame metres): the clerestories' grid (`StationDepot._clerestories`), so arches stand between
## the windows and chandeliers hang under them.
static func _bays(kit: StationKit, main: String) -> Vector2:
	var r := kit.floor_rect(main)
	return Vector2(r.size.y + kit.wall_t("south") + kit.wall_t("north"), r.position.y - kit.wall_t("south"))


# --- the ceiling -----------------------------------------------------------------------------------

static func _ceiling(g: Node3D, kit: StationKit, main: String, o: Vector3) -> void:
	var r := kit.floor_rect(main)
	var eaves := kit.m("shell", "eaves_mm")
	var rib := kit.pair("fitout", "rib_mm")
	var spring := kit.m("fitout", "spring_mm")
	var proud := kit.m("fitout", "pilaster_mm")
	var wood := StationDepot.timber()
	var c := StationDepot.at(o, r.get_center())
	Build.box(g, Vector3(c.x, o.y + eaves + 0.05, c.z), Vector3(r.size.x, 0.10, r.size.y), wood)
	var purlins := kit.n("fitout", "purlins")
	for i in purlins:
		var x := r.position.x + r.size.x * (i + 1) / float(purlins + 1)
		var p := StationDepot.at(o, Vector2(x, r.get_center().y))
		Build.box(g, Vector3(p.x, o.y + eaves - 0.14, p.z), Vector3(0.22, 0.28, r.size.y), wood)
	var arches := kit.n("fitout", "arches")
	var bays := _bays(kit, main)
	var openings := _openings(kit, main)
	report["arches"] = arches
	for k in arches:
		var y := bays.y + bays.x * (k + 1) / float(arches + 1)
		_arch(g, kit, o, r, y, rib, spring, proud, eaves, wood, openings)


## One transverse arch at frame y: a pilaster on each long wall to `spring` (a corbel where the wall is
## open), then a segmental rib to the ceiling.
static func _arch(g: Node3D, kit: StationKit, o: Vector3, r: Rect2, y: float, rib: Vector2, spring: float,
		proud: float, eaves: float, wood: Material, openings: Dictionary) -> void:
	for wall in ["west", "east"]:
		var x := r.position.x + proud * 0.5 if wall == "west" else r.end.x - proud * 0.5
		var p := StationDepot.at(o, Vector2(x, y))
		var open := _in_opening(openings.get(wall, []), y, rib.x)
		var h := 0.6 if open else spring
		Build.box(g, Vector3(p.x, o.y + spring - h * 0.5, p.z), Vector3(proud, h, rib.x), wood)
	var a := r.size.x * 0.5 - proud
	var crown := eaves - rib.y * 0.5
	var rise := crown - spring
	var radius := (a * a + rise * rise) / (2.0 * rise)
	var phi := asin(clampf(a / radius, -1.0, 1.0))
	var mid := StationDepot.at(o, Vector2(r.get_center().x, y))
	var yc := o.y + crown - radius
	for s in SEGMENTS:
		var t0 := -phi + 2.0 * phi * s / float(SEGMENTS)
		var t1 := -phi + 2.0 * phi * (s + 1) / float(SEGMENTS)
		var tm := (t0 + t1) * 0.5
		var chord := 2.0 * radius * sin((t1 - t0) * 0.5) + 0.02
		var mi := Build.box(g, Vector3(mid.x + radius * sin(tm), yc + radius * cos(tm), mid.z),
			Vector3(chord, rib.y, rib.x), wood)
		mi.basis = Basis(Vector3.BACK, -tm)


## Each wall's openings as [u0, u1] (frame metres along the wall: y on east and west, x on north and
## south): the arch's clear width, and an inner door's leaves plus 0.2 m each side.
static func _openings(kit: StationKit, main: String) -> Dictionary:
	var out := {}
	for d in kit.doors():
		var host := String(d["to"] if d["from"] == StationKit.OUTSIDE else d["from"])
		if host != main:
			continue
		var wall := String(d["wall"])
		var u := float(d["at"][1] if wall in ["east", "west"] else d["at"][0]) / 1000.0
		var w := kit.pair("arch", "clear_mm").x if d["from"] == StationKit.OUTSIDE \
			else int(d["leaves"]) * float(d["leaf_mm"][0]) / 1000.0 + 0.4
		if not out.has(wall):
			out[wall] = []
		out[wall].append([u - w * 0.5, u + w * 0.5])
	return out


static func _in_opening(list: Array, u: float, w: float) -> bool:
	for c in list:
		if u + w * 0.5 > c[0] and u - w * 0.5 < c[1]:
			return true
	return false


# --- walls -----------------------------------------------------------------------------------------

## The glazed-tile wainscot on every inner face to the kit's height, broken at the openings, under a
## frieze course; plaster (S29-d's wall) above.
static func _wainscot(g: Node3D, kit: StationKit, main: String, o: Vector3) -> void:
	var r := kit.floor_rect(main)
	var h := kit.m("fitout", "wainscot_mm")
	var tile := StationTile.material("frieze")
	var course: Material = StationPieces.style_of(String(kit.data.get("style", "")))["accent"]
	var openings := _openings(kit, main)
	for wall in StationKit.WALLS:
		var along_y: bool = wall in ["east", "west"]
		var lo := r.position.y if along_y else r.position.x
		var hi := r.end.y if along_y else r.end.x
		var cuts: Array = openings.get(wall, []).duplicate()
		cuts.sort_custom(func(p: Array, q: Array) -> bool: return p[0] < q[0])
		var start := lo
		for c in cuts + [[hi, hi]]:
			var stop := clampf(c[0], lo, hi)
			if stop - start > 0.05:
				_run(g, o, r, wall, start, stop, h, tile, course)
			start = maxf(start, clampf(c[1], lo, hi))
	report["wainscot_m"] = h


static func _run(g: Node3D, o: Vector3, r: Rect2, wall: String, u0: float, u1: float, h: float,
		tile: Material, course: Material) -> void:
	var along_y: bool = wall in ["east", "west"]
	var fixed: float = {"east": r.end.x - 0.015, "west": r.position.x + 0.015, "north": r.end.y - 0.015,
		"south": r.position.y + 0.015}[wall]
	var c := StationDepot.at(o, Vector2(fixed, (u0 + u1) * 0.5) if along_y else Vector2((u0 + u1) * 0.5, fixed))
	var size := Vector3(0.03, h, u1 - u0) if along_y else Vector3(u1 - u0, h, 0.03)
	Build.box(g, Vector3(c.x, o.y + h * 0.5, c.z), size, tile)
	var top := Vector3(0.06, 0.14, u1 - u0) if along_y else Vector3(u1 - u0, 0.14, 0.06)
	Build.box(g, Vector3(c.x, o.y + h + 0.07, c.z), top, course)


# --- light -----------------------------------------------------------------------------------------

## A bronze-and-glass chandelier under every bay (NRHP §7), lowest point at the kit's height; four of
## them carry a shadowed light, one shadowless fill, and one reflection probe with the room's ambient
## (the shop interior's pattern; a 52 m VoxelGI would be too coarse).
static func _lights(g: Node3D, kit: StationKit, main: String, o: Vector3) -> void:
	var r := kit.floor_rect(main)
	var eaves := kit.m("shell", "eaves_mm")
	var size := kit.pair("fitout", "chandelier_mm")
	var low := kit.m("fitout", "chandelier_low_mm")
	var bronze := SlicePalette.brass(0.4)
	var warm := Color(1.0, 0.84, 0.62)
	var glow := Mats.emissive(warm, 5.0)
	var bays := _bays(kit, main)
	var n := kit.n("fitout", "arches") + 1
	var lit := 0
	for k in n:
		var y := bays.y + bays.x * (k + 0.5) / float(n)
		var p := StationDepot.at(o, Vector2(r.get_center().x, y))
		var base := Vector3(p.x, o.y + low, p.z)
		Build.cyl(g, base + Vector3(0, size.y, 0), 0.015, 0.015, eaves - low - size.y, bronze, 6)
		Build.cyl(g, base + Vector3(0, size.y * 0.35, 0), size.x * 0.5, size.x * 0.5, 0.05, bronze, 16)
		Build.cyl(g, base, 0.10, 0.22, size.y * 0.35, bronze, 10)
		for i in 8:
			var a := TAU * i / 8.0
			Build.sphere(g, base + Vector3(cos(a) * size.x * 0.46, size.y * 0.35 + 0.12, sin(a) * size.x * 0.46),
				0.09, glow, 8)
		if k % 2 == 1 and lit < 4:
			lit += 1
			var l := OmniLight3D.new()
			l.light_color = warm
			l.light_energy = 4.0
			l.omni_range = 15.0
			l.omni_attenuation = 1.0
			l.shadow_enabled = true
			l.position = base + Vector3(0, size.y * 0.35, 0)
			g.add_child(l)
	report["chandeliers"] = n
	var c := StationDepot.at(o, r.get_center())
	var fill := OmniLight3D.new()
	fill.light_color = Color(0.98, 0.94, 0.88)
	fill.light_energy = 1.0
	fill.omni_range = r.size.y * 0.6
	fill.omni_attenuation = 0.5
	fill.shadow_enabled = false
	fill.position = Vector3(c.x, o.y + 3.0, c.z)
	g.add_child(fill)
	var probe := ReflectionProbe.new()
	probe.name = "HallProbe"
	probe.size = Vector3(r.size.x, eaves, r.size.y)
	probe.position = Vector3(c.x, o.y + eaves * 0.5, c.z)
	probe.box_projection = true
	probe.interior = true
	probe.update_mode = ReflectionProbe.UPDATE_ONCE
	probe.ambient_mode = ReflectionProbe.AMBIENT_COLOR
	probe.ambient_color = Color(0.46, 0.42, 0.36)
	probe.ambient_color_energy = 0.7
	g.add_child(probe)


# --- fixtures --------------------------------------------------------------------------------------

## The departures board on the north end wall, at the head of the hall's long axis (design; S29-a's
## benches face north); the clock over the door to the open place;
## a leaflet rack with empty slots (leaflets would be a timetable) and a bin beside that door.
static func _fixtures(g: Node3D, kit: StationKit, main: String, o: Vector3, style: Dictionary) -> void:
	var r := kit.floor_rect(main)
	var volume := String(StationDepot.report["volumes"].get(main, ""))
	var bsize := kit.pair("fitout", "board_mm")
	var bp := StationDepot.at(o, Vector2(r.get_center().x, r.end.y - 0.06))
	var board := _put(StationPieces.board(g, bsize.x, bsize.y, style),
		Vector3(bp.x, o.y + kit.m("fitout", "board_up_mm"), bp.z), Vector3.BACK)
	StationFixtures.register(board, volume)
	report["fixtures"]["board"] = board
	for d in kit.doors():
		if d["from"] != main or String(kit.place(String(d["to"]))["shell"]) != "open":
			continue
		var wall := String(d["wall"])
		var out := StationDepot.outward(wall)
		var at := Vector2(d["at"][0], d["at"][1]) / 1000.0
		var face := StationDepot.at(o, at) + out * (SliceLayout.DOOR_INSET - 0.06)
		var clock := _put(StationPieces.clock(g, kit.m("fitout", "clock_mm"), style),
			face + Vector3(0, kit.m("fitout", "clock_up_mm"), 0), -out)
		StationFixtures.register(clock, volume)
		report["fixtures"]["clock"] = clock
		var along := Vector3(absf(out.z), 0, absf(out.x))
		_rack(g, face - out * 0.10 + along * 2.4, -out, style)
		Build.cyl(g, face - out * 0.30 + along * 3.4, 0.17, 0.15, 0.70, SlicePalette.iron(), 12)
		# one umbrella someone left leaning by the rack (VISUAL_SLICE §6.2: hand scale)
		var u := Build.cyl(g, face - out * 0.18 + along * 1.85 + Vector3(0, 0.02, 0), 0.025, 0.04, 0.86,
			Mats.paint(Color(0.12, 0.16, 0.22), 0.6), 8)
		u.basis = Basis(along.cross(Vector3.UP), 0.18)


static func _rack(g: Node3D, foot: Vector3, front: Vector3, style: Dictionary) -> void:
	var n := _put(StationPieces._root(g, "leaflet_rack"), foot, front)
	Build.box(n, Vector3(0, 0.75, 0.10), Vector3(0.90, 1.50, 0.04), style["wood"])
	for side: float in [-1.0, 1.0]:
		Build.box(n, Vector3(side * 0.43, 0.75, 0.0), Vector3(0.04, 1.50, 0.24), style["wood"])
	for k in 5:
		Build.box(n, Vector3(0, 0.40 + k * 0.25, -0.02), Vector3(0.82, 0.02, 0.18), style["wood"])
		Build.box(n, Vector3(0, 0.46 + k * 0.25, -0.10), Vector3(0.82, 0.10, 0.012), style["wood"])


## The ticket window on the walled room's far wall, facing its door; one pendant light over the bay.
static func _ticket_window(g: Node3D, kit: StationKit, place: String, style: Dictionary) -> void:
	var d := _door_to(kit, place)
	if d.is_empty():
		return
	var o: Vector3 = StationDepot.report["origins"][place]
	var r := kit.floor_rect(place)
	var out := StationDepot.outward(String(d["wall"]))
	var depth := kit.m("fitout", "counter_depth_mm")
	var across_x := absf(out.z) > 0.5
	var width := r.size.x if across_x else r.size.y
	var centre := StationDepot.at(o, r.get_center()) + out * ((r.size.y if across_x else r.size.x) * 0.5 - depth * 0.5)
	var bay := _put(StationPieces.window_bay(g, width, kit.m("fitout", "counter_mm"), depth,
		kit.m("fitout", "screen_mm"), kit.n("fitout", "grilles"), style), centre, -out)
	StationFixtures.register(bay, String(StationDepot.report["volumes"].get(place, "")))
	report["fixtures"]["window"] = bay
	var c := StationDepot.at(o, r.get_center())
	var l := OmniLight3D.new()
	l.light_color = Color(1.0, 0.86, 0.66)
	l.light_energy = 2.2
	l.omni_range = 6.0
	l.shadow_enabled = true
	l.position = Vector3(c.x, o.y + 2.6, c.z)
	g.add_child(l)
	Build.sphere(g, l.position + Vector3(0, 0.05, 0), 0.12, Mats.emissive(Color(1.0, 0.86, 0.66), 4.0), 10)
	Build.cyl(g, l.position + Vector3(0, 0.12, 0), 0.01, 0.01, kit.m("annex", "eaves_mm") - 2.75,
		SlicePalette.brass(), 5)
