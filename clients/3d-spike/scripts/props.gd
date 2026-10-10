## Street furniture, vegetation and shop dressing.
##
## Two sources, deliberately only two (mixing eight libraries is what makes a
## scene read as a pile of assets rather than a town):
##   1. CC0 Poly Haven glTF props, loaded at runtime through GLTFDocument so
##      nothing has to go through Godot's importer.
##   2. Primitives built here, for everything no open licence covers -- trees,
##      planters, awnings, signage, bikes, boats, fountains.
##
## Every imported prop goes through `retint`. Poly Haven's street furniture
## comes from their `hidden_alley` collection and arrives grimy and back-alley
## grey; the references are a cared-for lakeside promenade. Taking material
## authority back from the asset author is the single step that makes borrowed
## props stop announcing that they were borrowed.
class_name Props
extends RefCounted

const MODEL_DIR := "res://assets/models"

const LEAF_VARIANTS := 5

static var _scenes: Dictionary = {}
static var _rng := RandomNumberGenerator.new()
static var _card_mats: Dictionary = {}


## Generating a 256x256 cut-out in GDScript costs real time, so a handful of
## variants are shared across every instance rather than one per tree. Five
## canopies over thirty trees is invisible; thirty canopies cost minutes.
static func _cached(key: String, maker: Callable) -> Material:
	if not _card_mats.has(key):
		_card_mats[key] = maker.call()
	return _card_mats[key]

static func _leaf_mat(v: int) -> Material:
	var greens := [
		Color(0.27, 0.43, 0.15), Color(0.33, 0.47, 0.18), Color(0.23, 0.39, 0.16),
		Color(0.37, 0.50, 0.20), Color(0.30, 0.45, 0.22),
	]
	return _cached("leaf%d" % v, func(): return Mats.card(ProcGen.leaf_card(11 + v * 97, greens[v]), 0.85))

static func _needle_mat(v: int) -> Material:
	return _cached("needle%d" % v, func(): return Mats.card(ProcGen.needle_card(31 + v * 53), 0.45))

static func _flower_mat(v: int) -> Material:
	return _cached("flower%d" % v, func(): return Mats.card(ProcGen.flower_card(7 + v * 41), 0.5))

static func _grass_mat(v: int) -> Material:
	return _cached("grass%d" % v, func(): return Mats.card(ProcGen.grass_card(17 + v * 29), 0.6))


static func gltf(slug: String) -> Node3D:
	if not _scenes.has(slug):
		var path := "%s/%s/%s.gltf" % [MODEL_DIR, slug, slug]
		var doc := GLTFDocument.new()
		var st := GLTFState.new()
		var err := doc.append_from_file(path, st)
		if err != OK:
			push_warning("gltf load failed: %s (%d)" % [path, err])
			_scenes[slug] = null
		else:
			_scenes[slug] = doc.generate_scene(st)
	var src: Node3D = _scenes[slug]
	if src == null:
		return Node3D.new()
	var n := src.duplicate() as Node3D
	# the slice's frame-cost breakdown finds props by this (RL-b M-7)
	n.set_meta("mw_category", "gltf")
	return n


## Re-point an imported prop's materials at our palette.
static func retint(root: Node, tint: Color, rough_lo := 0.30, rough_hi := 0.80,
		metal_max := 1.0) -> void:
	for child in root.get_children():
		retint(child, tint, rough_lo, rough_hi, metal_max)
	if root is MeshInstance3D:
		var mi := root as MeshInstance3D
		var mesh := mi.mesh
		if mesh == null:
			return
		for i in range(mesh.get_surface_count()):
			var m := mi.get_active_material(i)
			if m is BaseMaterial3D:
				var d := (m as BaseMaterial3D).duplicate() as BaseMaterial3D
				d.albedo_color = d.albedo_color * tint
				d.roughness = clampf(d.roughness, rough_lo, rough_hi)
				d.metallic = minf(d.metallic, metal_max)
				mi.set_surface_override_material(i, d)


static func place(parent: Node3D, slug: String, pos: Vector3, yaw := 0.0,
		tint := Color.WHITE, scale := 1.0) -> Node3D:
	var n := gltf(slug)
	n.transform = Transform3D(Basis(Vector3.UP, yaw).scaled(Vector3.ONE * scale), pos)
	parent.add_child(n)
	if tint != Color.WHITE:
		retint(n, tint)
	return n


# --- vegetation ----------------------------------------------------------------

static func bark_mat() -> Material:
	return Mats.paint(Color(0.24, 0.20, 0.16), 0.95)


## Broadleaf street tree. No CC0 library ships one light enough to plant thirty
## of, so: a tapered trunk, a few boughs, and a canopy shell of alpha-cut leaf
## cards. Reads correctly from about four metres out, which is where the
## references put their street trees.
static func broadleaf(parent: Node3D, pos: Vector3, h := 7.5, seed_v := 0) -> Node3D:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	var t := Node3D.new()
	t.position = pos
	parent.add_child(t)
	var bark := bark_mat()

	var trunk_h := h * 0.42
	Build.cyl(t, Vector3.ZERO, h * 0.014, h * 0.023, trunk_h, bark, 9)
	Build.blocker(t, Vector3.ZERO, h * 0.030, 2.2)

	var boughs := 4
	var tops: Array[Vector3] = []
	for i in range(boughs):
		var a := TAU * float(i) / boughs + rng.randf_range(-0.4, 0.4)
		var lean := rng.randf_range(0.30, 0.52)
		var blen := h * rng.randf_range(0.26, 0.36)
		var from := Vector3(0, trunk_h * rng.randf_range(0.82, 1.0), 0)
		var dir := Vector3(cos(a) * lean, 1.0, sin(a) * lean).normalized()
		var to := from + dir * blen
		var mi := Build.cyl(t, Vector3.ZERO, h * 0.006, h * 0.012, blen, bark, 7)
		# In the tree's LOCAL frame. look_at_from_position takes global
		# positions, and these are local: it put every tree's boughs around the
		# world origin, where they drew as one splayed fan of sticks.
		mi.transform = Transform3D(Basis.looking_at(to - from, Vector3.UP), (from + to) * 0.5)
		mi.rotate_object_local(Vector3.RIGHT, PI * 0.5)
		tops.append(to)

	# canopy: a flattened shell of leaf cards, denser at the rim than the core
	var cr := h * 0.40
	var cy := trunk_h + h * 0.30
	var base_green := Color(0.28, 0.44, 0.16).lerp(Color(0.36, 0.48, 0.18), rng.randf())
	var leaf_mat := _leaf_mat(seed_v % LEAF_VARIANTS)
	var n_cards := 18
	for i in range(n_cards):
		var u := rng.randf() * TAU
		var v := acos(clampf(rng.randf_range(-0.75, 1.0), -1.0, 1.0))
		var rad := cr * (0.62 + 0.38 * pow(rng.randf(), 0.4))
		var p := Vector3(sin(v) * cos(u), cos(v) * 0.72, sin(v) * sin(u)) * rad
		p.y += cy
		var sz := h * rng.randf_range(0.30, 0.44)
		var q := Build.card(t, p, sz, sz, leaf_mat,
			rng.randf() * TAU, rng.randf_range(-0.9, 0.9))
		q.rotate_object_local(Vector3.FORWARD, rng.randf() * TAU)
	# a couple of low cards so the canopy is not hollow when seen from beneath
	for i in range(5):
		var a2 := TAU * float(i) / 5.0
		Build.card(t, Vector3(cos(a2) * cr * 0.4, cy - cr * 0.42, sin(a2) * cr * 0.4),
			h * 0.34, h * 0.34, leaf_mat, a2, -1.3)
	return t


static func conifer(parent: Node3D, pos: Vector3, h := 9.0, seed_v := 0) -> Node3D:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	var t := Node3D.new()
	t.position = pos
	parent.add_child(t)
	Build.cyl(t, Vector3.ZERO, h * 0.007, h * 0.020, h * 0.92, bark_mat(), 8)
	Build.blocker(t, Vector3.ZERO, h * 0.028, 2.2)
	var nm := _needle_mat(seed_v % 3)
	var whorls := 9
	for w in range(whorls):
		var f := float(w) / float(whorls - 1)
		var y := h * (0.16 + f * 0.80)
		var r := h * 0.20 * (1.0 - f) + h * 0.02
		var per := 5
		for i in range(per):
			var a := TAU * float(i) / per + f * 1.3 + rng.randf_range(-0.2, 0.2)
			var sz := r * 2.1
			Build.card(t, Vector3(cos(a) * r * 0.55, y, sin(a) * r * 0.55),
				sz, sz, nm, a, rng.randf_range(-0.35, 0.05), pos.z > -40.0)
	return t


static func flowerbed(parent: Node3D, pos: Vector3, w: float, d: float, seed_v: int) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	var fm := _flower_mat(seed_v % 4)
	var n := int(w * d * 5.0) + 3
	for i in range(n):
		var p := pos + Vector3(rng.randf_range(-w, w) * 0.5, 0, rng.randf_range(-d, d) * 0.5)
		var s := rng.randf_range(0.38, 0.62)
		p.y += s * 0.45
		for k in range(2):
			Build.card(parent, p, s, s * 0.95, fm, rng.randf() * TAU + k * 1.57, 0.0, false)


static func grass_patch(parent: Node3D, pos: Vector3, w: float, d: float, seed_v: int,
		density := 3.0) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	var gm := _grass_mat(seed_v % 3)
	var n := int(w * d * density) + 2
	for i in range(n):
		var s := rng.randf_range(0.30, 0.55)
		var p := pos + Vector3(rng.randf_range(-w, w) * 0.5, s * 0.46, rng.randf_range(-d, d) * 0.5)
		Build.card(parent, p, s * 1.5, s, gm, rng.randf() * TAU, 0.0, false)


## Boulders: a sphere pushed around by noise. Cheaper than the 6 MB CC0 scan
## and, triplanar-textured with the same stone as the kerbs, more coherent.
static func boulder(parent: Node3D, pos: Vector3, r: float, seed_v: int) -> void:
	var noise := FastNoiseLite.new()
	noise.seed = seed_v
	noise.frequency = 0.55
	var sm := SphereMesh.new()
	sm.radius = 1.0
	sm.height = 2.0
	sm.radial_segments = 14
	sm.rings = 8
	var arr := sm.get_mesh_arrays()
	var verts: PackedVector3Array = arr[Mesh.ARRAY_VERTEX]
	for i in range(verts.size()):
		var v := verts[i]
		var n := noise.get_noise_3d(v.x * 2.2, v.y * 2.2, v.z * 2.2)
		verts[i] = v * (1.0 + n * 0.30)
	arr[Mesh.ARRAY_VERTEX] = verts
	var am := ArrayMesh.new()
	am.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arr)
	var st := SurfaceTool.new()
	st.create_from(am, 0)
	st.generate_normals()
	var mi := MeshInstance3D.new()
	mi.mesh = st.commit()
	mi.material_override = Mats.pbr("stone_brick_wall_001", 1.4, Color(1.05, 1.02, 0.96))
	mi.transform = Transform3D(
		Basis(Vector3.UP, float(seed_v)).scaled(Vector3(r, r * 0.72, r * 0.9)),
		pos + Vector3(0, r * 0.40, 0))
	parent.add_child(mi)
	Build.blocker(parent, pos, r * 0.8, r * 1.4)


# --- street furniture ----------------------------------------------------------

static func planter(parent: Node3D, pos: Vector3, w: float, d: float, seed_v: int,
		yaw := 0.0) -> void:
	var stone := Mats.cutstone()
	var h := 0.55
	var t := 0.14
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(g)
	Build.slab(g, 0, (d - t) * 0.5, w, t, 0, h, stone)
	Build.slab(g, 0, -(d - t) * 0.5, w, t, 0, h, stone)
	Build.slab(g, (w - t) * 0.5, 0, t, d - t * 2.0, 0, h, stone)
	Build.slab(g, -(w - t) * 0.5, 0, t, d - t * 2.0, 0, h, stone)
	Build.slab(g, 0, 0, w - t * 1.6, d - t * 1.6, h - 0.16, 0.1,
		Mats.paint(Color(0.20, 0.15, 0.11), 0.98))
	flowerbed(g, Vector3(0, h - 0.06, 0), w - t * 1.8, d - t * 1.8, seed_v)
	Build.box_blocker(parent, pos + Vector3(0, h * 0.5, 0), Vector3(w, h, d), yaw)


static func bollard(parent: Node3D, pos: Vector3) -> void:
	var m := Mats.paint(Color(0.10, 0.11, 0.11), 0.45, 0.65)
	Build.cyl(parent, pos, 0.075, 0.085, 0.86, m, 10)
	Build.sphere(parent, pos + Vector3(0, 0.90, 0), 0.075, m, 10)
	Build.blocker(parent, pos, 0.11, 0.95)


static func bin(parent: Node3D, pos: Vector3, yaw := 0.0) -> void:
	var m := Mats.paint(Color(0.13, 0.15, 0.14), 0.52, 0.55)
	var wood := Mats.planks()
	Build.cyl(parent, pos, 0.27, 0.24, 0.82, m, 12)
	for i in range(8):
		var a := TAU * float(i) / 8.0 + yaw
		Build.box(parent, pos + Vector3(cos(a) * 0.27, 0.42, sin(a) * 0.27),
			Vector3(0.10, 0.62, 0.035), wood, -a)
	Build.cyl(parent, pos + Vector3(0, 0.82, 0), 0.30, 0.29, 0.07, m, 12)
	Build.blocker(parent, pos, 0.32, 0.9)


## Chalkboard A-frame. Every single reference has at least one; they are most
## of what makes those streets read as "small independent shops".
static func sandwich_board(parent: Node3D, pos: Vector3, yaw: float, lines: Array,
		accent := Color(0.93, 0.90, 0.82)) -> void:
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(g)
	var frame := Mats.paint(Color(0.34, 0.24, 0.16), 0.80)
	var slate := Mats.paint(Color(0.10, 0.11, 0.11), 0.92)
	for s in [-1.0, 1.0]:
		var lean: float = 0.16 * s
		var panel := Build.box(g, Vector3(0, 0.55, 0.11 * s), Vector3(0.62, 0.86, 0.035), slate)
		panel.rotation.x = lean
		for e in [-1.0, 1.0]:
			var post := Build.box(g, Vector3(0.32 * e, 0.55, 0.11 * s),
				Vector3(0.055, 0.98, 0.05), frame)
			post.rotation.x = lean
		var top := Build.box(g, Vector3(0, 1.0, 0.09 * s), Vector3(0.68, 0.07, 0.05), frame)
		top.rotation.x = lean
	var y := 0.85
	for line in lines:
		var l := Build.label(g, Vector3(0, y, 0.145), str(line), 0.088, accent, 0.0, 48)
		l.rotation.x = 0.16
		var l2 := Build.label(g, Vector3(0, y, -0.145), str(line), 0.088, accent, PI, 48)
		l2.rotation.x = 0.16
		y -= 0.145
	Build.box_blocker(parent, pos + Vector3(0, 0.5, 0), Vector3(0.7, 1.0, 0.55), yaw)


static func hanging_sign(parent: Node3D, pos: Vector3, yaw: float, txt: String,
		body := Color(0.22, 0.28, 0.24), ink := Color(0.90, 0.85, 0.70)) -> void:
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(g)
	var iron := Mats.paint(Color(0.09, 0.09, 0.09), 0.42, 0.7)
	Build.box(g, Vector3(0, 0.48, 0), Vector3(0.05, 0.05, 0.86), iron)
	Build.box(g, Vector3(0, 0.26, 0.40), Vector3(0.05, 0.48, 0.05), iron)
	Build.box(g, Vector3(0, 0.30, 0.22), Vector3(0.04, 0.38, 0.04), iron)
	var board := Build.box(g, Vector3(0, -0.28, 0.40), Vector3(0.035, 0.62, 0.78),
		Mats.paint(body, 0.62))
	for s in [-1.0, 1.0]:
		Build.label(g, Vector3(0.026 * s, -0.24, 0.40), txt, 0.13, ink,
			PI * 0.5 * s, 56)


static func awning(parent: Node3D, pos: Vector3, w: float, yaw: float, c: Color,
		depth := 1.45, txt := "") -> void:
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(g)
	var fab := Mats.paint(c, 0.88)
	var top := Build.box(g, Vector3(0, 0.26, depth * 0.5), Vector3(w, 0.05, depth + 0.1), fab)
	top.rotation.x = 0.36
	Build.box(g, Vector3(0, -0.05, depth * 0.96), Vector3(w, 0.34, 0.05), fab)
	for s in [-1.0, 1.0]:
		var side := Build.box(g, Vector3(w * 0.5 * s, 0.10, depth * 0.5),
			Vector3(0.03, 0.5, depth), fab)
		side.rotation.x = 0.0
	if txt != "":
		Build.label(g, Vector3(0, -0.06, depth * 0.99), txt, 0.19,
			Color(0.95, 0.93, 0.87), 0.0, 56)


static func bicycle(parent: Node3D, pos: Vector3, yaw: float, frame_c := Color(0.55, 0.20, 0.18)) -> void:
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(g)
	var rubber := Mats.paint(Color(0.07, 0.07, 0.08), 0.92)
	var metal := Mats.paint(Color(0.62, 0.63, 0.65), 0.32, 0.85)
	var frame := Mats.paint(frame_c, 0.38, 0.30)
	for zx in [-0.54, 0.54]:
		var tm := TorusMesh.new()
		tm.inner_radius = 0.30
		tm.outer_radius = 0.345
		tm.rings = 20
		tm.ring_segments = 6
		var mi := MeshInstance3D.new()
		mi.mesh = tm
		mi.material_override = rubber
		mi.transform = Transform3D(Basis(Vector3.RIGHT, PI * 0.5), Vector3(0, 0.345, zx))
		g.add_child(mi)
		for k in range(8):
			var a := PI * float(k) / 8.0
			var sp := Build.box(g, Vector3(0, 0.345, zx), Vector3(0.008, 0.60, 0.008), metal)
			sp.rotation.z = 0.0
			sp.rotation.x = a
	var pts := [
		[Vector3(0, 0.345, 0.54), Vector3(0, 0.98, 0.10)],
		[Vector3(0, 0.345, -0.54), Vector3(0, 0.90, -0.12)],
		[Vector3(0, 0.98, 0.10), Vector3(0, 0.90, -0.12)],
		[Vector3(0, 0.345, 0.54), Vector3(0, 0.40, -0.10)],
		[Vector3(0, 0.40, -0.10), Vector3(0, 0.90, -0.12)],
		[Vector3(0, 0.40, -0.10), Vector3(0, 0.345, -0.54)],
	]
	for seg in pts:
		var a3: Vector3 = seg[0]
		var b3: Vector3 = seg[1]
		var mid := (a3 + b3) * 0.5
		var mi2 := Build.cyl(g, Vector3.ZERO, 0.022, 0.022, a3.distance_to(b3), frame, 6)
		# local frame -- see broadleaf(): look_at_from_position is global
		mi2.transform = Transform3D(Basis.looking_at(b3 - a3, Vector3.UP), mid)
		mi2.rotate_object_local(Vector3.RIGHT, PI * 0.5)
	Build.box(g, Vector3(0, 1.02, 0.12), Vector3(0.46, 0.035, 0.035), metal)
	Build.box(g, Vector3(0, 0.94, -0.16), Vector3(0.09, 0.05, 0.26),
		Mats.paint(Color(0.12, 0.10, 0.09), 0.6))
	Build.box_blocker(parent, pos + Vector3(0, 0.5, 0), Vector3(0.5, 1.0, 1.3), yaw)


static func wayfinder(parent: Node3D, pos: Vector3, yaw: float, arms: Array) -> void:
	var g := Node3D.new()
	g.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(g)
	var wood := Mats.paint(Color(0.38, 0.28, 0.19), 0.85)
	Build.cyl(g, Vector3.ZERO, 0.075, 0.09, 2.55, wood, 8)
	Build.blocker(g, Vector3.ZERO, 0.12, 2.5)
	var y := 2.24
	for a in arms:
		var dirn: float = 1.0 if (a as String).begins_with("<") else -1.0
		var txt: String = (a as String).lstrip("<>")
		var blade := Build.box(g, Vector3(0, y, 0.52 * -dirn), Vector3(0.045, 0.24, 1.0), wood)
		Build.label(g, Vector3(0.03 * -dirn, y, 0.52 * -dirn), txt, 0.135,
			Color(0.96, 0.93, 0.85), PI * 0.5 * -dirn, 48)
		# arrow head
		Build.box(g, Vector3(0, y, (1.02) * -dirn), Vector3(0.045, 0.17, 0.17), wood, 0.0)
		y -= 0.34


static func railing(parent: Node3D, from: Vector3, to: Vector3, h := 1.05) -> void:
	var iron := Mats.paint(Color(0.10, 0.11, 0.11), 0.40, 0.70)
	var seg := to - from
	var n := int(seg.length() / 1.5)
	var yaw := atan2(seg.x, seg.z)
	for i in range(n + 1):
		var p := from + seg * (float(i) / float(maxi(n, 1)))
		Build.cyl(parent, p, 0.035, 0.04, h, iron, 8)
	var mid := (from + to) * 0.5
	for hy in [h - 0.05, h * 0.55, h * 0.18]:
		var bar := Build.box(parent, mid + Vector3(0, hy, 0),
			Vector3(0.035, 0.035, seg.length()), iron, yaw)
	Build.box_blocker(parent, mid + Vector3(0, h * 0.5, 0),
		Vector3(0.25, h, seg.length()), yaw)


static func parasol(parent: Node3D, pos: Vector3, c := Color(0.80, 0.78, 0.70)) -> void:
	var pole := Mats.paint(Color(0.25, 0.22, 0.19), 0.7)
	Build.cyl(parent, pos, 0.035, 0.04, 2.35, pole, 8)
	var cm := CylinderMesh.new()
	cm.top_radius = 0.05
	cm.bottom_radius = 1.35
	cm.height = 0.42
	cm.radial_segments = 8
	var mi := MeshInstance3D.new()
	mi.mesh = cm
	mi.material_override = Mats.paint(c, 0.9)
	mi.position = pos + Vector3(0, 2.28, 0)
	parent.add_child(mi)
	Build.blocker(parent, pos, 0.1, 2.2)
