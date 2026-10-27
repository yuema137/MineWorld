## A station's exterior, built from a station kit at a site (`DECISIONS.md` ARC-192; S29-d §5.5): the
## arched main building's shell, the towers beside its forecourt, the arch with its doors and glazed
## window, the arcade along the wall its open place lies against, the annex of its walled room, and the
## roofs. The interior's fit-out, the platform's slab and the tracks are S29-e's.
##
## Nothing here names a world, a place key or a role tag. The kit names its places and their tags as
## data; the site says only where the arch stands:
##
##     site = { kit: String, facade_x: float, axis_z: float, facing: "east", floor_y: float }
##
## The main place's frame (0, 0) lands at scene (facade_x - east wall - floor.max.x, floor_y,
## axis_z + arch.y): the arch's doorway point then lies on the axis, 0.40 m inside the wall. Each inner
## door's far place is placed by translation through its wall (`SliceLayout`'s rule: a binding never
## rotates), so a kit point (x, y) mm of place P is scene origin(P) + (x, 0, -y) / 1000.
##
## Categories (`mw_category`): the root `station.depot`; `station.hall`, `station.tower` (one node per
## tower, so X-6 hides exactly them), `station.arcade`, `station.annex`, `station.roof`. `SliceBatch`
## keeps the `station.` prefix through its merge (F-S29d-6), and the landmark check hides by it.
class_name StationDepot
extends RefCounted

## 12d's doorway inset, as the binding uses it.
const INSET := SliceLayout.DOOR_INSET
## A place volume's height and its centre above the floor, as the slice's own volumes (`SliceWorld`).
const VOLUME_H := 3.5
const VOLUME_UP := 1.25
## Vertical strips per arched head, and the widest strip of a curved crest.
const STRIPS := 10
const CREST_STRIP := 0.40
## The widest strip under a round head: a 14 m arch is drawn from 47 strips, so its curve reads as a curve.
const ARC_STRIP := 0.30
## Every box the depot stands on or in is this far below its floor, so no seam shows at the ground.
const FOOTING := 0.30
## The open doorway's transom bar: its depth above the leaves.
const TRANSOM := 0.12
## An occluder box's inset inside the solid it stands for, metres.
const OCCLUDER_INSET := 0.02

## The last build, for the probes, the occluders and the landmark check (one depot per scene):
##   origins    kit place -> scene origin of its frame
##   volumes    kit place -> its volume's name
##   doors      kit door name -> its index in the door registry
##   footprint  Array[Rect2] in scene (x, z): the building's plan, for LM-2
##   towers     Array[AABB]: each tower's massing
##   floor_y    the floors' height
static var report := {}
## Occluder boxes from the same numbers the walls and towers were built from, each inset by OCCLUDER_INSET
## (the occluder pass's inset, `tools/presentation/social_cafe/builders/occluders.gd` since 16f-d2).
static var occluder_boxes: Array[AABB] = []

static var _glass: Material = null


## Builds the depot under `root`; returns its node, or null when the site cannot be drawn.
static func build(root: Node3D, kit: StationKit, site: Dictionary) -> Node3D:
	if StationKit.site_refusal(site) != "":
		return null
	report = {"origins": {}, "volumes": {}, "doors": {}, "footprint": [], "towers": [],
		"floor_y": float(site.get("floor_y", 0.0)), "kit": kit.kit_name(), "site": site}
	occluder_boxes.clear()
	_glass = SlicePalette.dead_glass()
	var g := _node(root, "Station", "station.depot")
	var main := kit.main_place()
	var hall := _node(g, "Hall", "station.hall")
	var o := _main_origin(kit, site)
	report["origins"][main] = o
	_hall(hall, kit, main, o)
	for d in kit.doors():
		if d["from"] != StationKit.OUTSIDE:
			_inner_place(g, kit, d, o)
	for side: float in [-1.0, 1.0]:
		_tower(g, kit, site, side)
	_volumes(root, kit)
	return g


# --- frames ------------------------------------------------------------------------------------------

static func _main_origin(kit: StationKit, site: Dictionary) -> Vector3:
	var r := kit.floor_rect(kit.main_place())
	var arch := kit.arch_door()
	return Vector3(float(site["facade_x"]) - kit.wall_t("east") - r.end.x,
		float(report["floor_y"]), float(site["axis_z"]) + float(arch["at"][1]) / 1000.0)


## A point of a place's frame (metres, x east, y north) in the scene.
static func at(o: Vector3, p: Vector2) -> Vector3:
	return o + Vector3(p.x, 0.0, -p.y)


## The unit vector, in the scene, that points out of a place through its `wall`.
static func outward(wall: String) -> Vector3:
	return {"east": Vector3.RIGHT, "west": Vector3.LEFT, "north": Vector3.FORWARD,
		"south": Vector3.BACK}[wall]


## A door registry yaw whose (sin, cos) is `v`.
static func _yaw(v: Vector3) -> float:
	return atan2(v.x, v.z)


static func _node(parent: Node3D, nm: String, category: String) -> Node3D:
	var n := Node3D.new()
	n.name = nm
	n.set_meta("mw_category", category)
	parent.add_child(n)
	return n


# --- materials ---------------------------------------------------------------------------------------

static func stucco() -> Material:
	return Mats.pbr("painted_plaster_wall", 2.40, Color(1.20, 1.10, 0.94), 0.92)


static func trim() -> Material:
	return Mats.pbr("painted_plaster_wall", 2.40, Color(1.10, 1.00, 0.86), 0.90)


static func timber() -> Material:
	return Mats.pbr("brown_planks_09", 1.60, Color(0.96, 0.70, 0.56), 0.80)


static func door_paint() -> Material:
	return Mats.paint(Color(0.30, 0.19, 0.12), 0.60)


# --- the main building ---------------------------------------------------------------------------------

## The hall's floor, its four walls with their openings, the arch, its ceiling and its roof.
static func _hall(g: Node3D, kit: StationKit, main: String, o: Vector3) -> void:
	var r := kit.floor_rect(main)
	var eaves := kit.m("shell", "eaves_mm")
	var t := {}
	for w in StationKit.WALLS:
		t[w] = kit.wall_t(w)
	var x0: float = o.x + r.position.x - t["west"]
	var x1: float = o.x + r.end.x + t["east"]
	var z0: float = o.z - r.end.y - t["north"]
	var z1: float = o.z - r.position.y + t["south"]
	report["footprint"].append(Rect2(x0, z0, x1 - x0, z1 - z0))
	# the floor, quarry tile at the forecourt's level, under the walls too: a flush threshold
	var fl := Build.box(g, Vector3((x0 + x1) * 0.5, o.y - FOOTING * 0.5, (z0 + z1) * 0.5),
		Vector3(x1 - x0, FOOTING, z1 - z0), StationTile.material(kit.pattern("floor")), 0.0, true)
	fl.name = "floor"
	# the ceiling is the fit-out's: boards, purlins and arches at the eaves (`depot_interior.gd`)
	for w in StationKit.WALLS:
		_hall_wall(g, kit, main, o, w, t)
	_roof(g, kit, x0, x1, z0, z1, o.y + eaves)


## One wall of the hall: the doors through it, its clerestories, and on the arch's wall the arch.
static func _hall_wall(g: Node3D, kit: StationKit, main: String, o: Vector3, wall: String, t: Dictionary) -> void:
	var r := kit.floor_rect(main)
	var eaves := kit.m("shell", "eaves_mm")
	var ridge := kit.m("shell", "ridge_mm")
	var th: float = t[wall]
	# the wall's centre line, from one outer corner to the other, running so `u` grows with the frame's
	# y (east and west walls) or x (north and south walls)
	var a: Vector3
	var b: Vector3
	var span: float
	match wall:
		"east", "west":
			var x := o.x + (r.end.x + th * 0.5 if wall == "east" else r.position.x - th * 0.5)
			a = Vector3(x, o.y, o.z - r.position.y + t["south"])
			b = Vector3(x, o.y, o.z - r.end.y - t["north"])
			span = r.size.y + t["south"] + t["north"]
		_:
			var z := o.z - (r.end.y + th * 0.5 if wall == "north" else r.position.y - th * 0.5)
			a = Vector3(o.x + r.position.x - t["west"], o.y, z)
			b = Vector3(o.x + r.end.x + t["east"], o.y, z)
			span = r.size.x + t["west"] + t["east"]
	var lo: float = t["south"] if wall in ["east", "west"] else t["west"]
	var openings := []
	var closed := []      # [u_from, u_to]: stretches the clerestories and the occluders keep clear of
	for d in kit.doors():
		var host := String(d["to"] if d["from"] == StationKit.OUTSIDE else d["from"])
		if host != main or d["wall"] != wall:
			continue
		var u: float = lo + (float(d["at"][1]) if wall in ["east", "west"] else float(d["at"][0])) / 1000.0
		if d["from"] == StationKit.OUTSIDE:
			closed.append_array(_arch(g, kit, a, b, u, th))
		else:
			openings.append(_doorway(g, kit, d, a, b, u, th))
			closed.append([u - 1.6, u + 1.6])
	if wall == "east":
		for side: float in [-1.0, 1.0]:
			# the towers stand against this wall: no window behind them
			var c: float = lo + (float(kit.arch_door()["at"][1]) / 1000.0) - side * (kit.m("towers", "inner_offset_mm")
				+ kit.pair("towers", "plan_mm").y * 0.5)
			closed.append([c - kit.pair("towers", "plan_mm").y * 0.5, c + kit.pair("towers", "plan_mm").y * 0.5])
	if wall in ["east", "west"]:
		openings.append_array(_clerestories(kit, span, closed))
	var crest := ridge - eaves if wall in ["north", "south"] else 0.0
	_wall(g, a, b, eaves, th, stucco(), openings, true, crest, false, _cuts(closed))
	_plinth(g, kit, a, b, th, closed)
	_wall_occluders(a, b, eaves, th, openings, closed,
		float(kit.group("shell")["clerestory"]["sill_mm"]) / 1000.0)


## The stretches of a wall left to other pieces (the arch's frontispiece), as [u0, u1] pairs that the
## wall itself does not build. Only the arch claims a stretch; the doors' entries are kept for windows.
static func _cuts(closed: Array) -> Array:
	return closed.filter(func(c: Array) -> bool: return c.size() > 2)


## Evenly spaced arched clerestory windows along a long wall, skipping any that a door, the arch or a
## tower keeps clear (NRHP §7: "arched clerestory windows, near the ceiling").
static func _clerestories(kit: StationKit, span: float, closed: Array) -> Array:
	var c: Dictionary = kit.group("shell")["clerestory"]
	var n := int(c["count"])
	var size := Vector2(c["size_mm"][0], c["size_mm"][1]) / 1000.0
	var sill := float(c["sill_mm"]) / 1000.0
	var out := []
	for i in n:
		var u := span * (i + 0.5) / n
		var clear := true
		for k in closed:
			clear = clear and (u + size.x * 0.5 + 0.3 < k[0] or u - size.x * 0.5 - 0.3 > k[1])
		if clear:
			out.append({"u": u, "w": size.x, "sill": sill, "spring": sill + size.y - size.x * 0.5,
				"rise": size.x * 0.5, "glass": true})
	return out


## An inner door through the hall's wall: the opening, a transom bar at the leaves' head, the leaves
## folded back into the reveal, and the registry entry, whose out-point is on the hall's side.
static func _doorway(g: Node3D, kit: StationKit, d: Dictionary, a: Vector3, b: Vector3, u: float,
		th: float) -> Dictionary:
	var leaf := Vector2(d["leaf_mm"][0], d["leaf_mm"][1]) / 1000.0
	var w := leaf.x * int(d["leaves"])
	var dir := (b - a).normalized()
	var sill := a + dir * u
	# the hall's side of the wall: the inner face, toward the hall
	var into_hall := -outward(String(d["wall"]))
	var inner := sill + into_hall * th * 0.5
	Build.box(g, sill + Vector3(0, leaf.y + TRANSOM * 0.5, 0), Vector3(0, TRANSOM, 0) + _along(dir, w, th),
		timber(), 0.0, true)
	for s: float in [-1.0, 1.0]:
		var jamb := inner + dir * s * (w * 0.5 - 0.03)
		Build.box(g, jamb + into_hall * leaf.x * 0.5 + Vector3(0, leaf.y * 0.5, 0),
			Vector3(0, leaf.y, 0) + _along(into_hall, leaf.x, 0.05), door_paint())
	var idx := SliceDoors.rows.size()
	SliceDoors.rows.append([inner, _yaw(into_hall), "%s %s" % [kit.kit_name(), d["name"]],
		kit.place(String(d["to"]))["volume"], th, 0.0])
	report["doors"][d["name"]] = idx
	return {"u": u, "w": w, "sill": 0.0, "spring": leaf.y + TRANSOM + 0.10, "rise": w * 0.5,
		"glass": true}


## A box size whose length `l` runs along `dir` (an axis) and whose thickness `t` runs across it.
static func _along(dir: Vector3, l: float, t: float) -> Vector3:
	return Vector3(absf(dir.x) * l + absf(dir.z) * t, 0.0, absf(dir.z) * l + absf(dir.x) * t)


## The arch in its wall: the frontispiece between the towers with its curvilinear Mission parapet, the
## round-headed opening, the screen of door bays at the façade, the glazed window filling the arch above
## it, and the open door on the axis. Returns the stretch it claims: [[u0, u1, "arch"]].
static func _arch(g: Node3D, kit: StationKit, a: Vector3, b: Vector3, u: float, th: float) -> Array:
	var clear := kit.pair("arch", "clear_mm")
	var half := kit.m("towers", "inner_offset_mm")
	var eaves := kit.m("shell", "eaves_mm")
	var dir := (b - a).normalized()
	var radius := clear.x * 0.5
	var spring := clear.y - radius
	var arch_open := {"u": half, "w": clear.x, "sill": 0.0, "spring": spring, "rise": radius}
	var fa := a + dir * (u - half)
	var fb := a + dir * (u + half)
	_wall(g, fa, fb, eaves, th, stucco(), [arch_open], true, kit.m("arch", "gable_mm") - eaves, true)
	_archivolt(g, fa, fb, half, clear, th, outward(String(kit.arch_door()["wall"])))
	_screen(g, kit, a + dir * u, dir, th, clear, spring)
	return [[u - half, u + half, "arch"]]


## A moulded band around the arch's head, proud of the façade's outer face (`out`).
static func _archivolt(g: Node3D, fa: Vector3, fb: Vector3, half: float, clear: Vector2, th: float,
		out: Vector3) -> void:
	var dir := (fb - fa).normalized()
	var r := clear.x * 0.5 + 0.25
	var spring := clear.y - clear.x * 0.5
	var n := 24
	for i in n:
		var ang := PI * (i + 0.5) / n
		var p := fa + dir * (half - cos(ang) * r) + Vector3(0, spring + sin(ang) * r, 0) + out * (th * 0.5 + 0.06)
		var tangent := (dir * sin(ang) + Vector3.UP * cos(ang)).normalized()
		var bm := BoxMesh.new()
		bm.size = Vector3(PI * r / n + 0.04, 0.5, 0.12)
		var mi := MeshInstance3D.new()
		mi.name = "archivolt"
		mi.mesh = bm
		mi.material_override = trim()
		mi.transform = Transform3D(Basis(tangent, out.cross(tangent), out), p)
		g.add_child(mi)


## The screen across the arch at the façade: door bays under round heads, the bay on the axis open, the
## others closed; the glazed window above, with its mullions and transoms; and the arch door's registry
## entry, its sill on the façade.
static func _screen(g: Node3D, kit: StationKit, centre: Vector3, dir: Vector3, th: float, clear: Vector2,
		spring: float) -> void:
	var d := kit.arch_door()
	var leaf := Vector2(d["leaf_mm"][0], d["leaf_mm"][1]) / 1000.0
	var bays := int(kit.group("arch")["bays"])
	var top := kit.m("arch", "screen_mm")
	var pitch := clear.x / bays
	var out := outward(String(d["wall"]))
	var face := centre + out * th * 0.5            # the façade's outer face, on the axis
	var screen_t := 0.30
	var openings := []
	var w := leaf.x * int(d["leaves"])
	for i in bays:
		var off := (i - (bays - 1) * 0.5) * pitch
		openings.append({"u": clear.x * 0.5 + off, "w": w, "sill": 0.0, "spring": leaf.y + TRANSOM + 0.10,
			"rise": w * 0.5, "glass": true})
		var c := face - out * screen_t * 0.5 + dir * off
		Build.box(g, c + Vector3(0, leaf.y + TRANSOM * 0.5, 0), Vector3(0, TRANSOM, 0) + _along(dir, w, screen_t),
			timber(), 0.0, true)
		if absf(off) > 0.01:
			# a closed bay: its two leaves, flush with the screen
			Build.box(g, c + Vector3(0, leaf.y * 0.5, 0), Vector3(0, leaf.y, 0) + _along(dir, w, 0.08),
				door_paint(), 0.0, true)
	var sa := face - out * screen_t * 0.5 - dir * clear.x * 0.5
	var sb := face - out * screen_t * 0.5 + dir * clear.x * 0.5
	_wall(g, sa, sb, top, screen_t, trim(), openings, true)
	_window(g, face - out * th * 0.5, dir, clear, spring, top)
	var idx := SliceDoors.rows.size()
	SliceDoors.rows.append([face, _yaw(out), "%s %s" % [kit.kit_name(), d["name"]],
		kit.place(String(d["to"]))["volume"], th, 0.0])
	report["doors"][d["name"]] = idx


## The glazed window filling the arch above the screen: glass strips under the curve, iron mullions and
## transoms.
static func _window(g: Node3D, mid: Vector3, dir: Vector3, clear: Vector2, spring: float, bottom: float) -> void:
	var r := clear.x * 0.5
	var n := maxi(STRIPS * 2, ceili(clear.x / ARC_STRIP))
	for i in n:
		var off := -r + clear.x * (i + 0.5) / n
		var head := spring + sqrt(maxf(0.0, r * r - off * off))
		if head <= bottom:
			continue
		Build.box(g, mid + dir * off + Vector3(0, (bottom + head) * 0.5, 0),
			Vector3(0, head - bottom, 0) + _along(dir, clear.x / n + 0.01, 0.03), _glass)
	var iron := SlicePalette.iron()
	var bars := 8
	for i in range(1, bars):
		var off := -r + clear.x * i / bars
		var head := spring + sqrt(maxf(0.0, r * r - off * off))
		Build.box(g, mid + dir * off + Vector3(0, (bottom + head) * 0.5, 0),
			Vector3(0, head - bottom, 0) + _along(dir, 0.08, 0.10), iron)
	var y := bottom + 1.8
	while y < spring + r - 0.5:
		var half := sqrt(maxf(0.0, r * r - maxf(0.0, y - spring) ** 2))
		Build.box(g, mid + Vector3(0, y, 0), Vector3(0, 0.08, 0) + _along(dir, half * 2.0, 0.10), iron)
		y += 1.8


# --- walls -----------------------------------------------------------------------------------------------

## A wall of thickness `t` and height `h` on the centre line a -> b (axis-aligned, at the floor), built
## from boxes around its openings. An opening is { u (centre, from a), w, sill, spring (where its head
## starts), rise (0: a flat head), glass (a pane in it) }. `crest` raises the top toward the middle:
## linearly (a gable end) or, with `curved`, as a Mission parapet. `skip` holds [u0, u1] stretches other
## pieces build. Only what a body can meet collides: the full-height stretches, the sills, and heads
## below 3 m.
static func _wall(g: Node3D, a: Vector3, b: Vector3, h: float, t: float, mat: Material, openings: Array,
		collide: bool, crest := 0.0, curved := false, skip: Array = []) -> void:
	var dir := (b - a).normalized()
	var length := a.distance_to(b)
	var yaw := atan2(-dir.z, dir.x)
	var sorted := openings.duplicate()
	sorted.sort_custom(func(p: Dictionary, q: Dictionary) -> bool: return p["u"] < q["u"])
	var top := func(u: float) -> float:
		if crest <= 0.0:
			return h
		var x := absf(u - length * 0.5) / (length * 0.5)
		return h + crest * ((1.0 - x * x) if curved else (1.0 - x))
	var cursor := 0.0
	for op in sorted:
		var u0: float = op["u"] - op["w"] * 0.5
		_solid(g, a, dir, yaw, cursor, u0, t, mat, collide, top, skip)
		_head(g, a, dir, yaw, op, t, mat, collide, top)
		cursor = op["u"] + op["w"] * 0.5
	_solid(g, a, dir, yaw, cursor, length, t, mat, collide, top, skip)


## The full-height wall between u0 and u1: one box, or strips under a crest.
static func _solid(g: Node3D, a: Vector3, dir: Vector3, yaw: float, u0: float, u1: float, t: float,
		mat: Material, collide: bool, top: Callable, skip: Array) -> void:
	for s in skip:
		if u0 < s[1] and u1 > s[0]:
			_solid(g, a, dir, yaw, u0, minf(u1, s[0]), t, mat, collide, top, [])
			_solid(g, a, dir, yaw, maxf(u0, s[1]), u1, t, mat, collide, top, [])
			return
	if u1 - u0 < 0.005:
		return
	var n := maxi(1, ceili((u1 - u0) / CREST_STRIP)) if top.call(u0) != top.call(u1) else 1
	for i in n:
		var p0 := u0 + (u1 - u0) * i / n
		var p1 := u0 + (u1 - u0) * (i + 1) / n
		var hh: float = top.call((p0 + p1) * 0.5)
		Build.box(g, a + dir * (p0 + p1) * 0.5 + Vector3(0, (hh - FOOTING) * 0.5, 0),
			Vector3(p1 - p0, hh + FOOTING, t), mat, yaw, collide)


## The wall under and over one opening: the sill, then strips from the head's curve to the top.
static func _head(g: Node3D, a: Vector3, dir: Vector3, yaw: float, op: Dictionary, t: float, mat: Material,
		collide: bool, top: Callable) -> void:
	var w: float = op["w"]
	var u0: float = op["u"] - w * 0.5
	if op["sill"] > 0.0:
		Build.box(g, a + dir * op["u"] + Vector3(0, (op["sill"] - FOOTING) * 0.5, 0),
			Vector3(w, op["sill"] + FOOTING, t), mat, yaw, collide)
	var n: int = maxi(STRIPS, ceili(w / ARC_STRIP)) if op["rise"] > 0.0 else 1
	for i in n:
		var p0 := u0 + w * i / n
		var p1 := u0 + w * (i + 1) / n
		var x: float = ((p0 + p1) * 0.5 - op["u"]) / (w * 0.5)
		var bottom: float = op["spring"] + op["rise"] * sqrt(maxf(0.0, 1.0 - x * x))
		var hh: float = top.call((p0 + p1) * 0.5)
		if hh - bottom > 0.005:
			Build.box(g, a + dir * (p0 + p1) * 0.5 + Vector3(0, (bottom + hh) * 0.5, 0),
				Vector3(p1 - p0 + 0.002, hh - bottom, t), mat, yaw, collide and bottom < 3.0)
	if op.get("glass", false) and op["sill"] > 0.0:
		var head: float = op["spring"] + op["rise"] * 0.7
		Build.box(g, a + dir * op["u"] + Vector3(0, (op["sill"] + head) * 0.5, 0),
			Vector3(w, head - op["sill"], 0.03), _glass, yaw)


## A sandstone course along the foot of a wall, 50 mm proud of both faces, leaving its doorways and the
## arch's stretch clear. It never collides: the wall does.
static func _plinth(g: Node3D, kit: StationKit, a: Vector3, b: Vector3, t: float, skip: Array) -> void:
	_wall(g, a, b, kit.m("shell", "plinth_mm"), t + 0.10, SlicePalette.limestone(), [], false, 0.0, false,
		skip)


static func _wall_occluders(a: Vector3, b: Vector3, h: float, t: float, openings: Array, closed: Array,
		sill: float) -> void:
	var dir := (b - a).normalized()
	var length := a.distance_to(b)
	var gaps := []
	for op in openings:
		if op["sill"] < sill - 0.01:
			gaps.append([op["u"] - op["w"] * 0.5 - 1.0, op["u"] + op["w"] * 0.5 + 1.0])
	for c in closed:
		gaps.append([c[0] - 1.0, c[1] + 1.0])
	gaps.sort_custom(func(p: Array, q: Array) -> bool: return p[0] < q[0])
	var cursor := 0.0
	for gp in gaps + [[length, length]]:
		var u1: float = minf(gp[0], length)
		if u1 - cursor > 2.0:
			var p0 := a + dir * cursor
			var p1 := a + dir * u1
			var across := Vector3(absf(dir.z), 0.0, absf(dir.x)) * t * 0.5
			var lo := p0.min(p1) - across
			var hi := p0.max(p1) + across + Vector3(0, minf(h, sill) - 0.3, 0)
			occluder_boxes.append(AABB(lo, hi - lo).grow(-OCCLUDER_INSET))
		cursor = maxf(cursor, gp[1])


# --- roofs -----------------------------------------------------------------------------------------------

## The hall's gable roof in Mission tile, ridge along the long (north-south) axis, with an overhang.
static func _roof(g: Node3D, kit: StationKit, x0: float, x1: float, z0: float, z1: float, eaves_y: float) -> void:
	var roofs := _node(g, "Roof", "station.roof")
	var rise := kit.m("shell", "ridge_mm") - kit.m("shell", "eaves_mm")
	var over := kit.m("shell", "overhang_mm")
	var xm := (x0 + x1) * 0.5
	for s: float in [-1.0, 1.0]:
		var lo := Vector3(x0 if s < 0.0 else x1, eaves_y, 0.0)
		var hi := Vector3(xm, eaves_y + rise, 0.0)
		slope(roofs, lo, hi, Vector3.BACK, z1 - z0 + over * 2.0, (z0 + z1) * 0.5, over)


## A tiled roof slab rising from the edge `lo` to `hi` (both at z = 0 of the slab's run), extruded along
## `run` (an axis) by `length` around `run_centre`, with `over` of overhang past `lo`.
static func slope(g: Node3D, lo: Vector3, hi: Vector3, run: Vector3, length: float, run_centre: float,
		over: float) -> void:
	var v := hi - lo
	var xn := v.normalized()
	var zn := run.normalized()
	var yn := zn.cross(xn).normalized()
	var w := v.length() + over
	var c := (lo - xn * over + hi) * 0.5 + yn * 0.10
	if absf(zn.z) > 0.5:
		c.z = run_centre
	else:
		c.x = run_centre
	var bm := BoxMesh.new()
	bm.size = Vector3(w, 0.20, length)
	var mi := MeshInstance3D.new()
	mi.name = "roof"
	mi.mesh = bm
	mi.material_override = SlicePalette.pantile()
	mi.transform = Transform3D(Basis(xn, yn, zn), c)
	g.add_child(mi)


# --- the places off the hall -----------------------------------------------------------------------------

## The place an inner door leads to: its frame's origin, and its shell (an annex or the arcade).
static func _inner_place(g: Node3D, kit: StationKit, d: Dictionary, o: Vector3) -> void:
	var to := String(d["to"])
	var wall := String(d["wall"])
	var out := outward(wall)
	var th := kit.wall_t(wall)
	var hall_point := at(o, Vector2(d["at"][0], d["at"][1]) / 1000.0)
	# the hall-side doorway point is INSET inside the inner face; the far side's is INSET beyond the outer
	var far := hall_point + out * (INSET + th + INSET)
	var origin := far - (at(Vector3.ZERO, Vector2(d["there"][0], d["there"][1]) / 1000.0))
	origin.y = o.y
	report["origins"][to] = origin
	match String(kit.place(to)["shell"]):
		"annex":
			_annex(_node(g, "Annex", "station.annex"), kit, to, origin, wall)
		"open":
			_arcade(_node(g, "Arcade", "station.arcade"), kit, to, origin, d, far)


## A walled room off the hall: its three outer walls and a shed roof falling away from the hall.
static func _annex(g: Node3D, kit: StationKit, place: String, o: Vector3, door_wall: String) -> void:
	var r := kit.floor_rect(place)
	var t := kit.wall_t("other")
	var h := kit.m("annex", "eaves_mm")
	var x0 := o.x + r.position.x - t
	var x1 := o.x + r.end.x + t
	var z0 := o.z - r.end.y - t
	var z1 := o.z - r.position.y + t
	report["footprint"].append(Rect2(x0, z0, x1 - x0, z1 - z0))
	Build.box(g, Vector3((x0 + x1) * 0.5, o.y - FOOTING * 0.5, (z0 + z1) * 0.5),
		Vector3(x1 - x0, FOOTING, z1 - z0), StationTile.material(kit.pattern("floor")), 0.0, true)
	var shared := outward(door_wall) * -1.0     # the annex's side toward the hall
	var edges := {
		Vector3.FORWARD: [Vector3(x0, o.y, z0 + t * 0.5), Vector3(x1, o.y, z0 + t * 0.5)],
		Vector3.BACK: [Vector3(x0, o.y, z1 - t * 0.5), Vector3(x1, o.y, z1 - t * 0.5)],
		Vector3.LEFT: [Vector3(x0 + t * 0.5, o.y, z0), Vector3(x0 + t * 0.5, o.y, z1)],
		Vector3.RIGHT: [Vector3(x1 - t * 0.5, o.y, z0), Vector3(x1 - t * 0.5, o.y, z1)],
	}
	for side in edges:
		if side.is_equal_approx(shared):
			continue
		_wall(g, edges[side][0], edges[side][1], h, t, stucco(), [], true)
		_wall(g, edges[side][0], edges[side][1], kit.m("shell", "plinth_mm"), t + 0.10, SlicePalette.limestone(),
			[], false)
	# the shed: high against the hall, falling outward
	var far_edge: Vector3 = -shared
	var hi_x := x1 if shared.x > 0.5 else (x0 if shared.x < -0.5 else (x0 + x1) * 0.5)
	var hi_z := z1 if shared.z > 0.5 else (z0 if shared.z < -0.5 else (z0 + z1) * 0.5)
	var lo_x := x1 if far_edge.x > 0.5 else (x0 if far_edge.x < -0.5 else (x0 + x1) * 0.5)
	var lo_z := z1 if far_edge.z > 0.5 else (z0 if far_edge.z < -0.5 else (z0 + z1) * 0.5)
	var run := Vector3.BACK if absf(shared.x) > 0.5 else Vector3.RIGHT
	var length := (z1 - z0) if absf(shared.x) > 0.5 else (x1 - x0)
	var centre := (z0 + z1) * 0.5 if absf(shared.x) > 0.5 else (x0 + x1) * 0.5
	slope(_node(g, "Roof", "station.roof"), Vector3(lo_x, o.y + h, lo_z), Vector3(hi_x, o.y + h + 1.0, hi_z),
		run, length + 0.6, centre, kit.m("shell", "overhang_mm") * 0.6)


## The arcade along the hall's wall, over the open place: piers on round arches, a shed roof of Mission
## tile on timber rafters. Its piers are the place's disclosed solids, so they collide.
static func _arcade(g: Node3D, kit: StationKit, place: String, o: Vector3, d: Dictionary, far: Vector3) -> void:
	var out := outward(String(d["wall"]))
	var along := Vector3(absf(out.z), 0.0, absf(out.x))
	var wall_face := far - out * INSET                # the hall wall's outer face, at the door
	var depth := kit.m("arcade", "depth_mm")
	var pier := kit.m("arcade", "pier_mm")
	var pitch := kit.m("arcade", "pitch_mm")
	var length := kit.m("arcade", "length_mm")
	var eaves := kit.m("arcade", "eaves_mm")
	var line := wall_face + out * (depth + pier * 0.5)     # the piers' centre line
	line.y = o.y
	var n := int(floor((length - pier) / pitch)) + 1
	var first := -(n - 1) * pitch * 0.5
	var a := line - along * (-first + pier * 0.5)
	var b := line + along * (-first + pier * 0.5)
	var openings := []
	for k in n - 1:
		openings.append({"u": pier * 0.5 + pitch * (k + 0.5), "w": pitch - pier, "sill": 0.0,
			"spring": kit.m("arcade", "spring_mm"), "rise": kit.m("arcade", "rise_mm")})
	_wall(g, a, b, eaves, pier, stucco(), openings, true)
	var half := a.distance_to(b) * 0.5
	var corner := [line - along * half + out * pier * 0.5, line + along * half - out * (depth + pier * 0.5)]
	var lo_x: float = minf(corner[0].x, corner[1].x)
	var hi_x: float = maxf(corner[0].x, corner[1].x)
	var lo_z: float = minf(corner[0].z, corner[1].z)
	var hi_z: float = maxf(corner[0].z, corner[1].z)
	report["footprint"].append(Rect2(lo_x, lo_z, hi_x - lo_x, hi_z - lo_z))
	var rise := depth * 0.2
	var roof := _node(g, "Roof", "station.roof")
	slope(roof, line + out * pier * 0.5 + Vector3(0, eaves, 0), wall_face + Vector3(0, eaves + rise, 0),
		along, half * 2.0 + 0.6, line.dot(along), 0.5)
	# timber rafters under the shed, four to a bay
	var step := pitch / 4.0
	var count := int(half * 2.0 / step)
	var v := (wall_face + Vector3(0, eaves + rise, 0)) - (line + Vector3(0, eaves, 0))
	var xn := v.normalized()
	var yn := xn.cross(Vector3.UP).cross(xn).normalized()
	for i in count + 1:
		var c := line + along * (-half + step * i) + v * 0.5 + Vector3(0, eaves - 0.12, 0)
		var bm := BoxMesh.new()
		bm.size = Vector3(v.length(), 0.18, 0.10)
		var mi := MeshInstance3D.new()
		mi.name = "rafter"
		mi.mesh = bm
		mi.material_override = timber()
		mi.transform = Transform3D(Basis(xn, yn, xn.cross(yn)), c)
		g.add_child(mi)
	report["arcade_line"] = line


# --- towers -----------------------------------------------------------------------------------------------

## One tower beside the forecourt (Q-S29d-2): a square stucco shaft, its inner face on the forecourt's
## edge and its back face on the façade line; an open belfry; a drum with the railway's emblem; a
## tiled dome; a tiled lantern and its finial.
static func _tower(g: Node3D, kit: StationKit, site: Dictionary, side: float) -> void:
	var t := _node(g, "Tower%s" % ("N" if side < 0.0 else "S"), "station.tower")
	var plan := kit.pair("towers", "plan_mm")
	var fx := float(site["facade_x"]) + kit.m("towers", "forward_mm")
	var inner := float(site["axis_z"]) + side * kit.m("towers", "inner_offset_mm")
	var cz := inner + side * plan.y * 0.5
	var cx := fx - plan.x * 0.5
	var y0 := float(report["floor_y"])
	var shaft := kit.m("towers", "shaft_mm")
	var body := AABB(Vector3(cx - plan.x * 0.5, y0, cz - plan.y * 0.5), Vector3(plan.x, shaft, plan.y))
	report["footprint"].append(Rect2(body.position.x, body.position.z, plan.x, plan.y))
	Build.box(t, Vector3(cx, y0 + (shaft - FOOTING) * 0.5, cz), Vector3(plan.x, shaft + FOOTING, plan.y),
		stucco(), 0.0, true)
	Build.box(t, Vector3(cx, y0 + kit.m("shell", "plinth_mm") * 0.5 - FOOTING * 0.5, cz),
		Vector3(plan.x + 0.10, kit.m("shell", "plinth_mm") + FOOTING, plan.y + 0.10), SlicePalette.limestone())
	occluder_boxes.append(body.grow(-OCCLUDER_INSET))
	# the tile panel on the street face, and the shaft's cornice
	var panel := Build.card(t, Vector3(fx + 0.02, y0 + shaft * 0.62, cz), 2.6, 2.6, null, PI * 0.5, 0.0, false)
	StationTile.apply(panel, kit.pattern("emblem"))
	Build.box(t, Vector3(cx, y0 + shaft - 0.18, cz), Vector3(plan.x + 0.4, 0.36, plan.y + 0.4), trim())
	var top := _belfry(t, kit, Vector3(cx, y0 + shaft, cz), plan)
	top = _crown(t, kit, Vector3(cx, top, cz))
	report["towers"].append(AABB(body.position, Vector3(plan.x, top - y0, plan.y)))


## The belfry: four walls with an arched opening each, corner pinnacles and a cornice. Returns its top.
static func _belfry(t: Node3D, kit: StationKit, base: Vector3, plan: Vector2) -> float:
	var size := kit.pair("towers", "belfry_mm")
	var wt := 0.55
	var h := size.y - 0.4      # the walls, under the cornice that completes the stage
	var half := size.x * 0.5
	var op := {"u": half, "w": size.x * 0.33, "sill": 0.9, "spring": h * 0.45, "rise": size.x * 0.165}
	for k in 4:
		var ang := PI * 0.5 * k
		var n := Vector3(cos(ang), 0.0, sin(ang))
		var tg := Vector3(-n.z, 0.0, n.x)
		var mid := base + n * (half - wt * 0.5)
		_wall(t, mid - tg * half, mid + tg * half, h, wt, stucco(), [op], false)
	Build.box(t, base + Vector3(0, h + 0.2, 0), Vector3(size.x + 0.5, 0.4, size.x + 0.5), trim())
	for sx: float in [-1.0, 1.0]:
		for sz: float in [-1.0, 1.0]:
			var p := base + Vector3(sx * (plan.x * 0.5 - 0.4), 0.0, sz * (plan.y * 0.5 - 0.4))
			Build.cyl(t, p, 0.05, 0.38, 1.9, trim(), 8)
	return base.y + size.y


## The drum, the dome and the lantern, in glazed tile, and the finial. Returns the finial's top.
static func _crown(t: Node3D, kit: StationKit, base: Vector3) -> float:
	var drum := kit.pair("towers", "drum_mm")
	var dome := kit.pair("towers", "dome_mm")
	var lantern := kit.pair("towers", "lantern_mm")
	Build.cyl(t, base, drum.x * 0.5, drum.x * 0.5, drum.y, stucco(), 8)
	var apothem := drum.x * 0.5 * cos(PI / 8.0)
	for k in 4:
		# CylinderMesh puts its first vertex on +z, so a face's centre is half a segment round
		var ang := PI * 0.5 * k + PI / 8.0
		var n := Vector3(sin(ang), 0.0, cos(ang))
		var card := Build.card(t, base + n * (apothem + 0.02) + Vector3(0, drum.y * 0.5, 0), 1.3, 1.3, null,
			ang, 0.0, false)
		StationTile.apply(card, kit.pattern("emblem"))
	var y := base.y + drum.y
	var sm := SphereMesh.new()
	sm.radius = dome.x * 0.5
	sm.height = dome.y          # a hemisphere SphereMesh spans 0 .. height
	sm.is_hemisphere = true
	sm.radial_segments = 32
	sm.rings = 10
	var mi := Build._mi(sm, null, Transform3D(Basis(), Vector3(base.x, y, base.z)), t, "dome")
	StationTile.apply(mi, kit.pattern("dome"), Vector2(TAU * dome.x * 0.5 / 0.30, dome.y / 0.30 * 2.0))
	y += dome.y
	# the lantern: a ring of columns round a dark core, a tiled cap and the finial
	var lh := lantern.y * 0.55
	Build.cyl(t, Vector3(base.x, y, base.z), lantern.x * 0.30, lantern.x * 0.30, lh, Mats.paint(Color(0.10, 0.10, 0.11)), 8)
	for k in 8:
		var ang := TAU * k / 8.0
		Build.cyl(t, Vector3(base.x + cos(ang) * lantern.x * 0.42, y, base.z + sin(ang) * lantern.x * 0.42),
			0.09, 0.09, lh, stucco(), 6)
	y += lh
	var cap := SphereMesh.new()
	cap.radius = lantern.x * 0.5
	cap.height = lantern.y - lh
	cap.is_hemisphere = true
	cap.radial_segments = 16
	cap.rings = 6
	var cmi := Build._mi(cap, null, Transform3D(Basis(), Vector3(base.x, y, base.z)), t, "lantern")
	StationTile.apply(cmi, kit.pattern("lantern"), Vector2(TAU * lantern.x * 0.5 / 0.20, 6.0))
	y += lantern.y - lh
	var f := kit.m("towers", "finial_mm")
	Build.sphere(t, Vector3(base.x, y + f * 0.5, base.z), f * 0.5, trim(), 10)
	return y + f


# --- volumes -------------------------------------------------------------------------------------------

## One place volume per kit place, its floor rectangle at its frame's origin (`SliceWorld`'s volumes).
static func _volumes(root: Node3D, kit: StationKit) -> void:
	for p in kit.place_names():
		if not report["origins"].has(p):
			continue
		var o: Vector3 = report["origins"][p]
		var r := kit.floor_rect(p)
		var c := at(o, r.get_center()) + Vector3(0, VOLUME_UP, 0)
		var a := Area3D.new()
		var vol := String(kit.place(p)["volume"])
		a.name = "Place_" + vol
		a.monitorable = false
		a.collision_layer = 0
		a.collision_mask = 0
		a.set_meta("place_id", vol)
		var cs := CollisionShape3D.new()
		var b := BoxShape3D.new()
		b.size = Vector3(r.size.x, VOLUME_H, r.size.y)
		cs.shape = b
		a.add_child(cs)
		a.position = c
		root.add_child(a)
		report["volumes"][p] = vol
