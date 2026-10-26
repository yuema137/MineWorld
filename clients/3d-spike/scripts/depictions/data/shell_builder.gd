## The building shells of a depiction file's `buildings` section (`DECISIONS.md` ARC-177; S33-d sec.5.5):
## each shell raised from its massing -- footprint, storeys, plinth, walls with their openings, a roof of
## one of four forms, a chimney -- on the drawn ground. A shell's room is `interior_builder.gd`'s, and
## furniture is never drawn from this file (`ARC-307`).
##
## The rules, all from the file or `PRESENTATION.md` sec.6's schema defaults:
##   - walls are `SliceBoxDepiction.WALL_T` thick (the disclosed walls' thickness), from the lowest ground
##     under the footprint to the eaves, `storeys` x `storey_m` higher: centred on the footprint's edge for
##     a shell that depicts a place (the place's floor edge, where the server's wall is), inside any other
##     shell's;
##   - an opening is a hole through its wall with depth (the wall's reveal), a sill under a window and a
##     head over every opening; `dark` puts a dark interior card at the wall's inner face, `closed` a door
##     leaf, `glazed` clear glass, `open` nothing (`DepictionInteriors` draws the room behind both, from
##     the shell's `interior`). `at_doorway` centres a door on its place's
##     doorway; `offset_m` counts from the facade's left end, seen from outside;
##   - a roof covers the footprint's bounding rectangle with `overhang_m` eaves: `gable` and `hip` rise at
##     `pitch_deg` to a ridge along `ridge` (the longer side when unset), `mono` rises away from `falls`,
##     `flat` is a slab;
##   - every shell builds colliders on `Build.LAYER_ART` (ARC-52) for its walls, with a gap at least
##     `SliceBoxDepiction.DOOR_W` wide at each `at_doorway` opening (the server decides who passes; the
##     geometry probe compares these walls with the disclosed ones); a door leaf has no collider.
## Nothing here names a world, a place or a tag.
class_name DepictionShells
extends RefCounted

## A door leaf's timber and the dark interior card's colour (`PRESENTATION.md` sec.6's schema defaults).
const LEAF_SLUG := "weathered_brown_planks"
const DARK := Color(0.035, 0.035, 0.04)
## How far a sill, a head and a plinth stand proud of the wall, and a sill's and a head's depth, metres
## (VISUAL_SLICE sec.5: "every opening has depth"; schema defaults).
const PROUD := 0.05
const TRIM := 0.08
## A flat roof's slab, metres (schema default).
const SLAB := 0.25


## Draws every building of `buildings` under `parent`, on `d`'s ground.
static func build(parent: Node3D, buildings: Array, d: SliceDataDepiction) -> void:
	var g := Node3D.new()
	g.name = "Shells"
	parent.add_child(g)
	for b in buildings:
		_shell(g, b, d)


static func _shell(g: Node3D, b: Dictionary, d: SliceDataDepiction) -> void:
	var node := Node3D.new()
	node.name = String(b["name"])
	g.add_child(node)
	var footprint := PackedVector2Array()
	var base := INF
	for p in b["footprint_mm"]:
		var on := d.ground_at(Vector2(float(p[0]), float(p[1])))
		footprint.append(Vector2(on.x, on.z))
		base = minf(base, on.y)
	# A depicted place's footprint is the floor its walls enclose: its walls stand centred on the floor's
	# edge, where the server's wall is (12e), so two places that share an edge share one wall. Any other
	# shell's footprint is its outer extent, its walls inside it.
	if b.get("depicts") != null:
		var grown := Geometry2D.offset_polygon(footprint, SliceBoxDepiction.WALL_T * 0.5, Geometry2D.JOIN_MITER)
		if not grown.is_empty():
			footprint = grown[0]
	var storey := float(b["storey_m"])
	var eaves := base + float(b["storeys"]) * storey
	var walls := _mat(b["walls"])
	var plinth := float(b["plinth_m"])
	var doorway: Variant = _doorway(b, d)
	var openings: Array = b["openings"] if b.get("openings") != null else []
	var centre := Vector2.ZERO
	for p in footprint:
		centre += p
	centre /= float(footprint.size())
	for k in footprint.size():
		var a := footprint[k]
		var c := footprint[(k + 1) % footprint.size()]
		var inward := _inward(a, c, centre)
		var side := _side(inward)
		var here := []
		for o in openings:
			if o["side"] == side:
				here.append(o)
		_wall(node, a, c, inward, base, eaves, storey, here, walls, doorway)
		if plinth > 0.0:
			_plinth(node, a, c, inward, base, plinth, walls)
	_roof(node, b, footprint, base, eaves, walls, d)


static func _mat(surface: Dictionary) -> Material:
	var tint := Color.html(surface["tint"]) if surface.get("tint") != null else Color.WHITE
	return Mats.pbr(surface["material"], DepictionEdges.TEXTURE_M, tint)


## The doorway (scene x, z) of the place the building depicts, or null.
static func _doorway(b: Dictionary, d: SliceDataDepiction) -> Variant:
	if b.get("depicts") == null:
		return null
	for p in d.places():
		var all := true
		for t in b["depicts"]:
			all = all and (t in p["tags"])
		if all:
			var s := d.to_scene(p["doorway"])
			return Vector2(s.x, s.z)
	return null


## The inward unit normal (scene x, z) of wall a-c of a footprint centred on `centre`.
static func _inward(a: Vector2, c: Vector2, centre: Vector2) -> Vector2:
	var normal := (c - a).normalized().orthogonal()
	return normal if normal.dot(centre - (a + c) * 0.5) > 0.0 else -normal


## The compass side (n, s, e, w) a wall faces, from its inward normal; true north is -z.
static func _side(inward: Vector2) -> String:
	var out := -inward
	if absf(out.x) > absf(out.y):
		return "e" if out.x > 0.0 else "w"
	return "s" if out.y > 0.0 else "n"


## One wall from a to c (scene x, z), its openings cut through it.
static func _wall(g: Node3D, a: Vector2, c: Vector2, inward: Vector2, base: float, eaves: float,
		storey: float, openings: Array, mat: Material, doorway: Variant) -> void:
	var length := a.distance_to(c)
	if length <= 0.0:
		return
	var dir := (c - a) / length
	var t := SliceBoxDepiction.WALL_T
	var yaw := atan2(dir.x, dir.y)
	# Seen from outside (looking along `inward`), the facade's left end is a when `dir` points right.
	var right := Vector2(-inward.y, inward.x)
	var left_is_a := dir.dot(right) > 0.0
	var holes := []
	for o in openings:
		var w := float(o["width_m"])
		var along := 0.0
		if bool(o.get("at_doorway", false)) and doorway != null:
			along = clampf(((doorway as Vector2) - a).dot(dir), w * 0.5, length - w * 0.5)
		else:
			var off := float(o.get("offset_m", 0.0)) + w * 0.5
			along = off if left_is_a else length - off
		var sill := float(o["sill_m"]) if o.get("sill_m") != null else 0.0
		var bottom := base + float(o["storey"]) * storey + sill
		holes.append({"from": along - w * 0.5, "to": along + w * 0.5, "y0": bottom,
			"y1": minf(bottom + float(o["height_m"]), eaves), "o": o})
	var cuts := [0.0, length]
	for h in holes:
		cuts.append(clampf(h["from"], 0.0, length))
		cuts.append(clampf(h["to"], 0.0, length))
	cuts.sort()
	for k in cuts.size() - 1:
		var s0: float = cuts[k]
		var s1: float = cuts[k + 1]
		if s1 - s0 <= 0.001:
			continue
		var spans := [[base, eaves]]
		for h in holes:
			if h["from"] < s1 - 0.001 and h["to"] > s0 + 0.001:
				spans = _less(spans, h["y0"], h["y1"])
		var m := a + dir * ((s0 + s1) * 0.5) + inward * (t * 0.5)
		for sp in spans:
			Build.box(g, Vector3(m.x, (sp[0] + sp[1]) * 0.5, m.y), Vector3(t, sp[1] - sp[0], s1 - s0), mat, yaw)
	for h in holes:
		_dress(g, a, dir, inward, h, yaw, mat)
	_colliders(g, a, dir, inward, length, base, eaves, holes, yaw)


## `spans` ([[y0, y1], ...]) less the interval y0..y1.
static func _less(spans: Array, y0: float, y1: float) -> Array:
	var out := []
	for sp in spans:
		if y1 <= sp[0] or y0 >= sp[1]:
			out.append(sp)
			continue
		if y0 > sp[0]:
			out.append([sp[0], y0])
		if y1 < sp[1]:
			out.append([y1, sp[1]])
	return out


## An opening's sill (windows), head, and fill: a dark interior card, a closed leaf, or nothing.
static func _dress(g: Node3D, a: Vector2, dir: Vector2, inward: Vector2, h: Dictionary, yaw: float,
		mat: Material) -> void:
	var t := SliceBoxDepiction.WALL_T
	var o: Dictionary = h["o"]
	var y0: float = h["y0"]
	var y1: float = h["y1"]
	var w: float = h["to"] - h["from"]
	var centre := a + dir * ((float(h["from"]) + float(h["to"])) * 0.5)
	var mid := centre + inward * (t * 0.5)
	var trim := Vector3(t + PROUD * 2.0, TRIM, w + TRIM * 2.0)
	if o["kind"] == "window" or o["kind"] == "shop_window":
		Build.box(g, Vector3(mid.x, y0 - TRIM * 0.5, mid.y), trim, mat, yaw)
	Build.box(g, Vector3(mid.x, y1 + TRIM * 0.5, mid.y), trim, mat, yaw)
	var depth := 0.0
	var fill: Material = null
	match o["state"]:
		"dark":
			depth = t
			fill = Mats.paint(DARK, 0.9)
		"closed":
			depth = t * 0.5
			fill = Mats.pbr(LEAF_SLUG, DepictionEdges.TEXTURE_M)
		"glazed":
			# clear glass in the wall's middle onto the room drawn behind it (S33-f)
			depth = t * 0.5
			fill = Mats.clear_glass()
		_:
			return
	var at := centre + inward * depth
	Build.box(g, Vector3(at.x, (y0 + y1) * 0.5, at.y), Vector3(0.04, y1 - y0, w), fill, yaw)


## The wall's colliders: the whole wall, less a gap of at least DOOR_W round every door at a doorway.
static func _colliders(g: Node3D, a: Vector2, dir: Vector2, inward: Vector2, length: float, base: float,
		eaves: float, holes: Array, yaw: float) -> void:
	var t := SliceBoxDepiction.WALL_T
	var gaps := []
	for h in holes:
		if bool(h["o"].get("at_doorway", false)):
			var c := (float(h["from"]) + float(h["to"])) * 0.5
			var half := maxf(float(h["to"]) - float(h["from"]), SliceBoxDepiction.DOOR_W) * 0.5
			gaps.append([c - half, c + half])
	gaps.sort_custom(func(x, y): return x[0] < y[0])
	var start := 0.0
	var pieces := []
	for gp in gaps:
		if gp[0] > start:
			pieces.append([start, gp[0]])
		start = maxf(start, gp[1])
	if start < length:
		pieces.append([start, length])
	for pc in pieces:
		var m := a + dir * ((float(pc[0]) + float(pc[1])) * 0.5) + inward * (t * 0.5)
		Build.box_blocker(g, Vector3(m.x, (base + eaves) * 0.5, m.y),
			Vector3(t, eaves - base, float(pc[1]) - float(pc[0])), yaw)


## A plinth course on wall a-c, `PROUD` outside it, `plinth` tall.
static func _plinth(g: Node3D, a: Vector2, c: Vector2, inward: Vector2, base: float, plinth: float,
		mat: Material) -> void:
	var length := a.distance_to(c)
	var dir := (c - a) / length
	var m := (a + c) * 0.5 - inward * (PROUD * 0.5)
	Build.box(g, Vector3(m.x, base + plinth * 0.5, m.y), Vector3(PROUD + 0.02, plinth, length + PROUD * 2.0),
		mat, atan2(dir.x, dir.y))


## The roof over the footprint's bounding rectangle, its gable walls, and the chimney.
static func _roof(g: Node3D, b: Dictionary, footprint: PackedVector2Array, base: float, eaves: float,
		walls: Material, d: SliceDataDepiction) -> void:
	var r: Dictionary = b["roof"]
	var box := Rect2(footprint[0], Vector2.ZERO)
	for p in footprint:
		box = box.expand(p)
	var over := float(r["overhang_m"]) if r.get("overhang_m") != null else 0.0
	var roof_mat := _mat(r)
	var slope := tan(deg_to_rad(float(r["pitch_deg"])))
	var x0 := box.position.x - over
	var x1 := box.end.x + over
	var z0 := box.position.y - over
	var z1 := box.end.y + over
	var roof := SurfaceTool.new()
	roof.begin(Mesh.PRIMITIVE_TRIANGLES)
	var gables := SurfaceTool.new()
	gables.begin(Mesh.PRIMITIVE_TRIANGLES)
	var top := eaves
	match String(r["form"]):
		"flat":
			Build.box(g, Vector3((x0 + x1) * 0.5, eaves + SLAB * 0.5, (z0 + z1) * 0.5),
				Vector3(x1 - x0, SLAB, z1 - z0), roof_mat)
			top = eaves + SLAB
		"mono":
			top = _mono(roof, gables, String(r["falls"]), box, x0, x1, z0, z1, eaves, slope)
		_:
			top = _pitched(roof, gables, r, box, x0, x1, z0, z1, eaves, slope)
	var form := String(r["form"])
	if form != "flat":
		_commit(g, roof, roof_mat, "Roof")
	if form == "gable" or form == "mono":
		_commit(g, gables, walls, "Gables")
	if b.get("chimney") != null:
		var ch: Dictionary = b["chimney"]
		var s := d.to_scene(Vector2(float(ch["at_mm"][0]), float(ch["at_mm"][1])))
		var y1 := top + float(ch["above_ridge_m"])
		Build.box(g, Vector3(s.x, (eaves + y1) * 0.5, s.z),
			Vector3(float(ch["size_m"][0]), y1 - eaves, float(ch["size_m"][1])), walls)


## A mono-pitch roof rising away from its `falls` side; returns its high edge's height.
static func _mono(roof: SurfaceTool, gables: SurfaceTool, falls: String, box: Rect2, x0: float, x1: float,
		z0: float, z1: float, eaves: float, slope: float) -> float:
	# the low edge is on the `falls` side (north is -z, east is +x); h(u) rises with u from 0 there
	var along_z := falls == "n" or falls == "s"
	var lo := z0 if falls == "n" else (z1 if falls == "s" else (x1 if falls == "e" else x0))
	var run := (z1 - z0) if along_z else (x1 - x0)
	var hi := eaves + run * slope
	var h := func(x: float, z: float) -> float:
		var u := absf((z if along_z else x) - lo)
		return eaves + u * slope
	_quad(roof, Vector3(x0, h.call(x0, z0), z0), Vector3(x1, h.call(x1, z0), z0),
		Vector3(x1, h.call(x1, z1), z1), Vector3(x0, h.call(x0, z1), z1))
	# the two triangular walls under the slope, on the walls' line
	if along_z:
		for x in [box.position.x, box.end.x]:
			var high_z: float = box.end.y if h.call(x, box.end.y) > h.call(x, box.position.y) else box.position.y
			_tri(gables, Vector3(x, eaves, box.position.y), Vector3(x, eaves, box.end.y),
				Vector3(x, h.call(x, high_z), high_z))
	else:
		for z in [box.position.y, box.end.y]:
			var high_x: float = box.end.x if h.call(box.end.x, z) > h.call(box.position.x, z) else box.position.x
			_tri(gables, Vector3(box.position.x, eaves, z), Vector3(box.end.x, eaves, z),
				Vector3(high_x, h.call(high_x, z), z))
	return hi


## A gable or hip roof; returns its ridge's height.
static func _pitched(roof: SurfaceTool, gables: SurfaceTool, r: Dictionary, box: Rect2, x0: float, x1: float,
		z0: float, z1: float, eaves: float, slope: float) -> float:
	var ridge_x: bool = (r["ridge"] == "x") if r.get("ridge") != null else box.size.x >= box.size.y
	var hip: bool = r["form"] == "hip"
	var half := ((z1 - z0) if ridge_x else (x1 - x0)) * 0.5
	var ridge_y := eaves + half * slope
	var inset := minf(half, ((x1 - x0) if ridge_x else (z1 - z0)) * 0.5) if hip else 0.0
	# the gable's apex on the walls' line, under the ridge (the eaves' overhang lowers it by over * slope)
	var over := box.position.x - x0
	var apex := ridge_y - over * slope
	if ridge_x:
		var zm := (z0 + z1) * 0.5
		var ra := Vector3(x0 + inset, ridge_y, zm)
		var rb := Vector3(x1 - inset, ridge_y, zm)
		_quad(roof, Vector3(x0, eaves, z0), Vector3(x1, eaves, z0), rb, ra)
		_quad(roof, Vector3(x1, eaves, z1), Vector3(x0, eaves, z1), ra, rb)
		if hip:
			_tri(roof, Vector3(x0, eaves, z1), Vector3(x0, eaves, z0), ra)
			_tri(roof, Vector3(x1, eaves, z0), Vector3(x1, eaves, z1), rb)
		else:
			for x in [box.position.x, box.end.x]:
				_tri(gables, Vector3(x, eaves, box.position.y), Vector3(x, eaves, box.end.y), Vector3(x, apex, zm))
	else:
		var xm := (x0 + x1) * 0.5
		var ra := Vector3(xm, ridge_y, z0 + inset)
		var rb := Vector3(xm, ridge_y, z1 - inset)
		_quad(roof, Vector3(x0, eaves, z1), Vector3(x0, eaves, z0), ra, rb)
		_quad(roof, Vector3(x1, eaves, z0), Vector3(x1, eaves, z1), rb, ra)
		if hip:
			_tri(roof, Vector3(x0, eaves, z0), Vector3(x1, eaves, z0), ra)
			_tri(roof, Vector3(x1, eaves, z1), Vector3(x0, eaves, z1), rb)
		else:
			for z in [box.position.y, box.end.y]:
				_tri(gables, Vector3(box.position.x, eaves, z), Vector3(box.end.x, eaves, z), Vector3(xm, apex, z))
	return ridge_y


static func _quad(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3, e: Vector3) -> void:
	_tri(st, a, b, c)
	_tri(st, a, c, e)


## One triangle, both faces (a roof seen from under its eaves shows its underside).
static func _tri(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3) -> void:
	for v in [a, b, c, a, c, b]:
		st.add_vertex(v)


static func _commit(g: Node3D, st: SurfaceTool, mat: Material, nm: String) -> void:
	st.generate_normals()
	var mesh := st.commit()
	if mesh == null or mesh.get_surface_count() == 0:
		return
	var mi := MeshInstance3D.new()
	mi.name = nm
	mi.mesh = mesh
	mi.material_override = mat
	g.add_child(mi)
