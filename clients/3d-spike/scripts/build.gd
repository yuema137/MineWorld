## Small geometry helpers. Everything the town is made of funnels through here
## so scale, collision and material handling stay consistent.
class_name Build
extends RefCounted

const LAYER_WORLD := 1


static func _mi(mesh: Mesh, mat: Material, xf: Transform3D, parent: Node3D, nm := "m") -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	mi.name = nm
	mi.mesh = mesh
	if mat != null:
		mi.material_override = mat
	mi.transform = xf
	parent.add_child(mi)
	return mi


## Axis-aligned box, positioned by its centre.
static func box(parent: Node3D, centre: Vector3, size: Vector3, mat: Material,
		yaw := 0.0, collide := false) -> MeshInstance3D:
	var bm := BoxMesh.new()
	bm.size = size
	var xf := Transform3D(Basis(Vector3.UP, yaw), centre)
	var mi := _mi(bm, mat, xf, parent, "box")
	if collide:
		var sb := StaticBody3D.new()
		sb.collision_layer = LAYER_WORLD
		var cs := CollisionShape3D.new()
		var sh := BoxShape3D.new()
		sh.size = size
		cs.shape = sh
		sb.add_child(cs)
		sb.transform = xf
		parent.add_child(sb)
	return mi


## Box given by its footprint and a base height -- how buildings are actually
## thought about, and it keeps the caller out of half-height arithmetic.
static func slab(parent: Node3D, x: float, z: float, w: float, d: float,
		y0: float, h: float, mat: Material, yaw := 0.0, collide := false) -> MeshInstance3D:
	return box(parent, Vector3(x, y0 + h * 0.5, z), Vector3(w, h, d), mat, yaw, collide)


static func cyl(parent: Node3D, base: Vector3, radius_top: float, radius_bot: float,
		h: float, mat: Material, segs := 12, collide := false) -> MeshInstance3D:
	var cm := CylinderMesh.new()
	cm.top_radius = radius_top
	cm.bottom_radius = radius_bot
	cm.height = h
	cm.radial_segments = segs
	cm.rings = 1
	var xf := Transform3D(Basis(), base + Vector3(0, h * 0.5, 0))
	var mi := _mi(cm, mat, xf, parent, "cyl")
	if collide:
		var sb := StaticBody3D.new()
		sb.collision_layer = LAYER_WORLD
		var cs := CollisionShape3D.new()
		var sh := CylinderShape3D.new()
		sh.radius = maxf(radius_top, radius_bot)
		sh.height = h
		cs.shape = sh
		sb.add_child(cs)
		sb.transform = xf
		parent.add_child(sb)
	return mi


static func sphere(parent: Node3D, centre: Vector3, r: float, mat: Material, segs := 12) -> MeshInstance3D:
	var sm := SphereMesh.new()
	sm.radius = r
	sm.height = r * 2.0
	sm.radial_segments = segs
	sm.rings = int(segs * 0.6)
	return _mi(sm, mat, Transform3D(Basis(), centre), parent, "sph")


## Flat quad in the XZ plane (ground), centred, facing up.
static func ground(parent: Node3D, centre: Vector3, w: float, d: float, mat: Material,
		subdiv := 0, collide := true) -> MeshInstance3D:
	var pm := PlaneMesh.new()
	pm.size = Vector2(w, d)
	pm.subdivide_width = subdiv
	pm.subdivide_depth = subdiv
	var mi := _mi(pm, mat, Transform3D(Basis(), centre), parent, "ground")
	if collide:
		var sb := StaticBody3D.new()
		sb.collision_layer = LAYER_WORLD
		var cs := CollisionShape3D.new()
		var sh := BoxShape3D.new()
		sh.size = Vector3(w, 0.4, d)
		cs.shape = sh
		sb.add_child(cs)
		sb.position = centre - Vector3(0, 0.2, 0)
		parent.add_child(sb)
	return mi


## Vertical quad (a "card"), centred, facing +Z before yaw.
static func card(parent: Node3D, centre: Vector3, w: float, h: float, mat: Material,
		yaw := 0.0, pitch := 0.0, shadow := true) -> MeshInstance3D:
	var qm := QuadMesh.new()
	qm.size = Vector2(w, h)
	var b := Basis(Vector3.UP, yaw) * Basis(Vector3.RIGHT, pitch)
	var mi := _mi(qm, mat, Transform3D(b, centre), parent, "card")
	mi.cast_shadow = (GeometryInstance3D.SHADOW_CASTING_SETTING_DOUBLE_SIDED if shadow
		else GeometryInstance3D.SHADOW_CASTING_SETTING_OFF)
	return mi


static func label(parent: Node3D, pos: Vector3, txt: String, size_m: float, c: Color,
		yaw := 0.0, font_size := 64) -> Label3D:
	var l := Label3D.new()
	l.text = txt
	l.font_size = font_size
	l.pixel_size = size_m / float(font_size)
	l.modulate = c
	l.outline_size = 0
	l.shaded = true
	l.double_sided = false
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	l.transform = Transform3D(Basis(Vector3.UP, yaw), pos)
	parent.add_child(l)
	return l


## A cylindrical collision post with no mesh -- used to stop the player walking
## through things whose geometry is cheaper to keep non-colliding.
static func blocker(parent: Node3D, pos: Vector3, r: float, h: float) -> void:
	var sb := StaticBody3D.new()
	sb.collision_layer = LAYER_WORLD
	var cs := CollisionShape3D.new()
	var sh := CylinderShape3D.new()
	sh.radius = r
	sh.height = h
	cs.shape = sh
	sb.add_child(cs)
	sb.position = pos + Vector3(0, h * 0.5, 0)
	parent.add_child(sb)


static func box_blocker(parent: Node3D, centre: Vector3, size: Vector3, yaw := 0.0) -> void:
	var sb := StaticBody3D.new()
	sb.collision_layer = LAYER_WORLD
	var cs := CollisionShape3D.new()
	var sh := BoxShape3D.new()
	sh.size = size
	cs.shape = sh
	sb.add_child(cs)
	sb.transform = Transform3D(Basis(Vector3.UP, yaw), centre)
	parent.add_child(sb)
