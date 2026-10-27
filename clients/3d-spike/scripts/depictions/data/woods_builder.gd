## The woods of a depiction file's `trees` section (`DECISIONS.md` ARC-177, DEP-79; S33-d sec.5.6): a
## species table, stands planted by Poisson-disc sampling (Bridson's algorithm) with the stand's seed and
## spacing, thinned by density, the species chosen by weight and by the ground they prefer; single
## planted trees; and invisible air walls behind the first rank.
##
## Each species is drawn by a `Props` form with a foliage card (`Props.FORMS`, `Props.CARDS`): one tree of
## each (form, card) is built once at unit height and merged into a mesh, and every tree is an instance of
## it, scaled to its height, in one `MultiMesh` per (form, card) and LOD band. Within `lod_m` of the scene's
## origin a tree is the full form; beyond, two crossed cards of its foliage (the impostor band).
##
## Trunks keep off every place's footprint and the street's (`keep_clear.places`), `keep_clear.paths_m`
## from every path, off every building's footprint, and out of the near water. Trees build no collider: the
## server's floors stop every walk; the air walls keep the camera rig and a free capsule inside.
## Nothing here names a world, a place, a tag or a species.
class_name DepictionWoods
extends RefCounted

## Bridson's candidates per active point (the algorithm's usual k).
const BRIDSON_K := 30
## An air wall's height, thickness and the length of one of its boxes, metres (schema defaults).
const AIR_WALL_H := 8.0
const AIR_WALL_T := 0.5
const AIR_WALL_SEG := 4.0
## The impostor's width, as a share of the tree's height (schema default).
const IMPOSTOR_W := 0.55


## Draws the woods `t` under `parent`, on `d`'s ground.
static func build(parent: Node3D, t: Dictionary, d: SliceDataDepiction) -> void:
	var started := Time.get_ticks_msec()
	var g := Node3D.new()
	g.name = "Woods"
	parent.add_child(g)
	var species := {}
	for s in t["species"]:
		species[s["name"]] = s
	var lod := float(t["lod_m"])
	# (form|card|band) -> [transforms], and the species that names each key
	var bands := {}
	var count := 0
	var clear := _keep_clear(t, d)
	for stand in (t["stands"] if t.get("stands") != null else []):
		for planted in _stand(stand, species, clear, d):
			_add(bands, species[planted[0]], planted[1], planted[2], planted[3], lod)
			count += 1
	for single in (t["singles"] if t.get("singles") != null else []):
		var at := d.ground_at(Vector2(float(single["at_mm"][0]), float(single["at_mm"][1])))
		_add(bands, species[single["species"]], at, float(single["height_m"]), 0.0, lod)
		count += 1
	for key in bands:
		_draw_band(g, key, bands[key])
	for wall in (t["air_walls"] if t.get("air_walls") != null else []):
		_air_wall(g, wall, d)
	print("[depiction] woods: %d trees in %d multimeshes, %d ms" % [count, bands.size(),
		Time.get_ticks_msec() - started])


static func _add(bands: Dictionary, s: Dictionary, at: Vector3, h: float, yaw: float, lod: float) -> void:
	var far := Vector2(at.x, at.z).length() > lod
	var key := "%s|%s|%s" % [s["form"], s["card"], "far" if far else "near"]
	if not bands.has(key):
		bands[key] = []
	bands[key].append(Transform3D(Basis(Vector3.UP, yaw).scaled(Vector3(h, h, h)), at))


# --- placement -------------------------------------------------------------------------------------

## What trunks keep clear of: plan rectangles (mm) and polygons, and lines with their clearance.
static func _keep_clear(t: Dictionary, d: SliceDataDepiction) -> Dictionary:
	var rects := []
	if bool(t["keep_clear"]["places"]):
		rects.append(d.street()["footprint"])
		for p in d.places():
			rects.append(p["footprint"])
	# a station's trackbed, through its cutting (ARC-179): no trunk stands on the line
	for c in DepictionStations.cuts(d._section("stations"), d):
		rects.append(c["rect"])
	var polygons := []
	for b in d._section("buildings"):
		polygons.append(_polygon(b["footprint_mm"]))
	var water: Variant = d._data().get("water")
	if water != null:
		polygons.append(_polygon(water["polygon_mm"]))
	var lines := []
	for path in d._section("paths"):
		lines.append(_polygon(path["line_mm"]))
	return {"rects": rects, "polygons": polygons, "lines": lines,
		"paths_mm": float(t["keep_clear"]["paths_m"]) * 1000.0}


static func _clear(p: Vector2, clear: Dictionary) -> bool:
	for r in clear["rects"]:
		if (r as Rect2).has_point(p):
			return false
	for polygon in clear["polygons"]:
		if Geometry2D.is_point_in_polygon(p, polygon):
			return false
	for line in clear["lines"]:
		for k in (line as PackedVector2Array).size() - 1:
			if p.distance_to(Geometry2D.get_closest_point_to_segment(p, line[k], line[k + 1])) < clear["paths_mm"]:
				return false
	return true


## One stand's trees: [[species name, scene position, height, yaw], ...].
static func _stand(stand: Dictionary, species: Dictionary, clear: Dictionary, d: SliceDataDepiction) -> Array:
	var rng := RandomNumberGenerator.new()
	rng.seed = int(stand["seed"])
	var area: Dictionary = stand["area"]
	var spacing := float(stand["spacing_m"]) * 1000.0
	var centre := Vector2.ZERO
	var box := Rect2()
	var inside: Callable
	if area.get("ring") != null:
		var ring: Dictionary = area["ring"]
		centre = Vector2(float(ring["centre_mm"][0]), float(ring["centre_mm"][1]))
		var inner := float(ring["inner_m"]) * 1000.0
		var outer := float(ring["outer_m"]) * 1000.0
		box = Rect2(centre - Vector2(outer, outer), Vector2(outer, outer) * 2.0)
		inside = func(p: Vector2) -> bool:
			var r := p.distance_to(centre)
			return r >= inner and r <= outer
	else:
		var polygon := _polygon(area["polygon_mm"])
		box = Rect2(polygon[0], Vector2.ZERO)
		for p in polygon:
			box = box.expand(p)
			centre += p
		centre /= float(polygon.size())
		inside = func(p: Vector2) -> bool: return Geometry2D.is_point_in_polygon(p, polygon)
	var density: Dictionary = stand["density"]
	var rise: Array = density["rise_m"]
	var candidates := []
	for p: Vector2 in _poisson(box, spacing, rng):
		if not inside.call(p) or not _clear(p, clear):
			continue
		var r := p.distance_to(centre) / 1000.0
		var f := clampf((r - float(rise[0])) / maxf(float(rise[1]) - float(rise[0]), 0.001), 0.0, 1.0)
		if rng.randf() > lerpf(float(density["from"]), float(density["to"]), f):
			continue
		candidates.append([p, d.ground_at(p)])
	if candidates.is_empty():
		return []
	var lo := INF
	var hi := -INF
	for c in candidates:
		lo = minf(lo, (c[1] as Vector3).y)
		hi = maxf(hi, (c[1] as Vector3).y)
	var out := []
	for c in candidates:
		var at: Vector3 = c[1]
		var up := 0.0 if hi - lo < 0.001 else (at.y - lo) / (hi - lo)
		var chosen := _choose(stand["mix"], up, rng)
		var s: Dictionary = species[chosen]
		var h := rng.randf_range(float(s["height_m"]["min"]), float(s["height_m"]["max"]))
		out.append([chosen, at, h, rng.randf() * TAU])
	return out


## A species from the mix, its weight raised on the ground it prefers (`up` 0 lowest, 1 highest).
static func _choose(mix: Array, up: float, rng: RandomNumberGenerator) -> String:
	var weights := []
	var total := 0.0
	for m in mix:
		var w := float(m["weight"])
		match m.get("prefer", ""):
			"low":
				w *= 2.0 - up
			"rise":
				w *= 1.0 + up
		weights.append(w)
		total += w
	var pick := rng.randf() * total
	for i in mix.size():
		pick -= weights[i]
		if pick <= 0.0:
			return mix[i]["species"]
	return mix[-1]["species"]


## Bridson's Poisson-disc sampling over `box` (mm) with minimum distance `r`, seeded.
static func _poisson(box: Rect2, r: float, rng: RandomNumberGenerator) -> Array:
	var cell := r / sqrt(2.0)
	var cols := int(ceil(box.size.x / cell)) + 1
	var rows := int(ceil(box.size.y / cell)) + 1
	var grid := PackedInt32Array()
	grid.resize(cols * rows)
	grid.fill(-1)
	var points := []
	var active := []
	var first := box.position + Vector2(rng.randf() * box.size.x, rng.randf() * box.size.y)
	points.append(first)
	active.append(0)
	grid[int((first.y - box.position.y) / cell) * cols + int((first.x - box.position.x) / cell)] = 0
	while not active.is_empty():
		var a := rng.randi_range(0, active.size() - 1)
		var base: Vector2 = points[active[a]]
		var found := false
		for k in BRIDSON_K:
			var angle := rng.randf() * TAU
			var dist := r * (1.0 + rng.randf())
			var p := base + Vector2(cos(angle), sin(angle)) * dist
			if not box.has_point(p):
				continue
			var gx := int((p.x - box.position.x) / cell)
			var gy := int((p.y - box.position.y) / cell)
			var ok := true
			for yy in range(maxi(gy - 2, 0), mini(gy + 3, rows)):
				for xx in range(maxi(gx - 2, 0), mini(gx + 3, cols)):
					var other := grid[yy * cols + xx]
					if other >= 0 and (points[other] as Vector2).distance_to(p) < r:
						ok = false
			if ok:
				points.append(p)
				active.append(points.size() - 1)
				grid[gy * cols + gx] = points.size() - 1
				found = true
				break
		if not found:
			active.remove_at(a)
	return points


# --- drawing ---------------------------------------------------------------------------------------

## One MultiMesh: the (form, card) tree at unit height, or its impostor.
static func _draw_band(g: Node3D, key: String, transforms: Array) -> void:
	var parts := key.split("|")
	var mesh := _template(parts[0], parts[1]) if parts[2] == "near" else _impostor(parts[1])
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.mesh = mesh
	mm.instance_count = transforms.size()
	for i in transforms.size():
		mm.set_instance_transform(i, transforms[i])
	var mi := MultiMeshInstance3D.new()
	mi.name = key.replace("|", "_")
	mi.multimesh = mm
	g.add_child(mi)


## A `form` tree with `card` foliage at unit height, merged into one mesh: a bark surface and a foliage
## surface.
static func _template(form: String, card: String) -> ArrayMesh:
	var scratch := Node3D.new()
	Props.tree(form, scratch, Vector3.ZERO, 1.0, 1)
	var bark := SurfaceTool.new()
	bark.begin(Mesh.PRIMITIVE_TRIANGLES)
	var leaves := SurfaceTool.new()
	leaves.begin(Mesh.PRIMITIVE_TRIANGLES)
	var has := [false, false]
	for mi in scratch.find_children("*", "MeshInstance3D", true, false):
		var m := mi as MeshInstance3D
		var xf := _relative(m, scratch)
		var is_bark := m.material_override == Props.bark_mat()
		(bark if is_bark else leaves).append_from(m.mesh, 0, xf)
		has[0 if is_bark else 1] = true
	scratch.free()
	var mesh := ArrayMesh.new()
	if has[0]:
		bark.commit(mesh)
		mesh.surface_set_material(mesh.get_surface_count() - 1, Props.bark_mat())
	if has[1]:
		leaves.commit(mesh)
		mesh.surface_set_material(mesh.get_surface_count() - 1, Props.card_mat(card, 0))
	return mesh


## A node's transform relative to `root` (the scratch tree is not in the scene tree).
static func _relative(n: Node3D, root: Node3D) -> Transform3D:
	var xf := n.transform
	var p := n.get_parent()
	while p != null and p != root:
		xf = (p as Node3D).transform * xf
		p = p.get_parent()
	return xf


## Two crossed cards of `card` foliage, unit height.
static func _impostor(card: String) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var w := IMPOSTOR_W * 0.5
	for axis in [Vector3(1, 0, 0), Vector3(0, 0, 1)]:
		var a: Vector3 = -axis * w
		var b: Vector3 = axis * w
		var quad := [[a, Vector2(0, 1)], [b, Vector2(1, 1)], [b + Vector3.UP, Vector2(1, 0)],
			[a, Vector2(0, 1)], [b + Vector3.UP, Vector2(1, 0)], [a + Vector3.UP, Vector2(0, 0)]]
		for v in quad:
			st.set_uv(v[1])
			st.add_vertex(v[0])
	st.generate_normals()
	var mesh := st.commit()
	mesh.surface_set_material(0, Props.card_mat(card, 1))
	return mesh


## Invisible boxes along a circle or a line, on the world layer.
static func _air_wall(g: Node3D, wall: Dictionary, d: SliceDataDepiction) -> void:
	var points := []
	if wall.get("ring") != null:
		var c := d.to_scene(Vector2(float(wall["ring"]["centre_mm"][0]), float(wall["ring"]["centre_mm"][1])))
		var radius := float(wall["ring"]["radius_m"])
		var n := maxi(int(ceil(TAU * radius / AIR_WALL_SEG)), 8)
		for k in n + 1:
			var a := TAU * float(k) / float(n)
			points.append(Vector2(c.x + cos(a) * radius, c.z + sin(a) * radius))
	else:
		for p in wall["line_mm"]:
			var s := d.to_scene(Vector2(float(p[0]), float(p[1])))
			points.append(Vector2(s.x, s.z))
	for k in points.size() - 1:
		var a: Vector2 = points[k]
		var b: Vector2 = points[k + 1]
		var run := b - a
		if run.length() <= 0.0:
			continue
		var mid := (a + b) * 0.5
		var ground := d.terrain.height_at(mid.x, mid.y) if d.terrain != null else 0.0
		var body := StaticBody3D.new()
		body.collision_layer = Build.LAYER_WORLD
		body.add_to_group(SliceProbeGeometry.GROUND_GROUP)
		var cs := CollisionShape3D.new()
		var box := BoxShape3D.new()
		box.size = Vector3(AIR_WALL_T, AIR_WALL_H, run.length())
		cs.shape = box
		body.add_child(cs)
		body.transform = Transform3D(Basis(Vector3.UP, atan2(run.x, run.y)),
			Vector3(mid.x, ground + AIR_WALL_H * 0.5, mid.y))
		g.add_child(body)


static func _polygon(points: Array) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in points:
		out.append(Vector2(float(p[0]), float(p[1])))
	return out
