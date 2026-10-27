## Small geometry helpers. Everything the town is made of funnels through here
## so scale, collision and material handling stay consistent.
class_name Build
extends RefCounted

## What the feet stand on: the ground, the terrain, a floor, a ramp. The server's plan has no height,
## so the drawn ground is what a body stands on, everywhere (S15 12e; 16f-b, `ARC-52`).
const LAYER_WORLD := 1
## Perceived people's colliders (step-15 §4.2). The targeting ray meets them
## (PR 16a) and, since S15 PR 12e, the player's body collides with them (step-11
## §22 SD-E3, SD-E5); the camera boom does not.
const LAYER_BODIES := 2
## Loose objects (boxes and balls) drawn from the place's `loose-objects` listing
## (step-11 §22 SD-E7): the player collides with them and the ray meets them.
const LAYER_OBJECTS := 4
## The current place's disclosed walls and solids, invisible (step-11 §22
## SD-E4): the player collides with them; neither the camera nor the targeting
## ray masks them, because an invisible wall must not hide what the eye sees
## (SD-E3, QE-8). A disclosed solid is drawn, so it also carries LAYER_CAMERA.
const LAYER_DISCLOSED := 8
## Art that stands on the floor: every wall, façade, jamb, counter, lamp or tree a builder makes
## collide (16f-b, `docs/decisions/ARC-52.md`: art only decorates). Where the current place discloses
## its `place-shape`, the player's body ignores this layer and only the disclosed walls stop it
## (`Player.set_art_blocks`); where nothing is disclosed (offline, or a world without bodies) art still
## blocks, because nothing else would. The camera boom, the targeting ray and the geometry probe still
## meet it, so the camera does not pass through a drawn wall and the probe measures art against
## disclosure.
const LAYER_ART := 16
## Drawn things the camera boom must not pass into but nothing else meets (promo v2: the rear camera
## at the café door went through a hanging basket's foliage). A prop placed by `SliceDressing.put` /
## `hang` or `Props.place`, and a foliage curtain, gets a box hull here from its drawn extents; a
## disclosed solid is drawn by `SliceFurnishings` (an undrawn solid is never left, I-FD-5), so its
## collider carries this layer beside LAYER_DISCLOSED. The player's body and the targeting ray do not
## mask it: it is the eye's, like SpringArm3D's own collision mask.
## Its own bit, distinct from LAYER_ART (the 16f-b / S33-d merge: both had taken 16).
const LAYER_CAMERA := 32
## What the camera boom is swept against: the built world, the art (it never passes through a drawn
## wall, 16f-b `ARC-52`), loose objects, and LAYER_CAMERA.
const CAMERA_MASK := LAYER_WORLD | LAYER_ART | LAYER_OBJECTS | LAYER_CAMERA
## Hidden walk aids (PR S36-P4b, Q-S36-P4-4): the slope in front of a drawn step that lets the player's capsule walk
## up it instead of catching on its square edge. Only the player's body masks this layer (and its press cast, which
## bounds a stride against what the body collides with). The feet's rays, the camera boom, targeting and the probes'
## scene rays mask `LAYER_WORLD` (the camera also `LAYER_CAMERA`), so none of them ever stands on, stops at or aims
## at an aid.
const LAYER_WALK_AID := 64
## The walk aid's slope: the street kerb's, inside the controller's 52-degree floor limit (`Player`).
const WALK_AID_DEG := 21.0
## A walk aid's thickness under its sloped face (a size of a collider, not a rule).
const WALK_AID_THICK := 0.30
## The group of every drawn step's collider (`step`): floor that is walked onto, not a wall (`--slice-geometry`).
const STEP_GROUP := &"build_steps"


## The drawn extents of `n` and everything under it, in `n`'s own frame (empty when nothing is drawn).
static func drawn_aabb(n: Node) -> AABB:
	var out := AABB()
	var have := false
	if n is MeshInstance3D and (n as MeshInstance3D).mesh != null:
		out = (n as MeshInstance3D).mesh.get_aabb()
		have = true
	for c in n.get_children():
		var sub := drawn_aabb(c)
		if sub.size == Vector3.ZERO:
			continue
		if c is Node3D:
			sub = (c as Node3D).transform * sub
		out = sub if not have else out.merge(sub)
		have = true
	return out


## A camera hull: an invisible box on LAYER_CAMERA over `box` (in `parent`'s frame), colliding with
## nothing. An empty box adds nothing.
static func camera_hull(parent: Node3D, box: AABB, nm := "CameraHull") -> void:
	if box.size.x <= 0.0 or box.size.y <= 0.0 or box.size.z <= 0.0:
		return
	var sb := StaticBody3D.new()
	sb.name = nm
	sb.collision_layer = LAYER_CAMERA
	sb.collision_mask = 0
	var cs := CollisionShape3D.new()
	var sh := BoxShape3D.new()
	sh.size = box.size
	cs.shape = sh
	sb.add_child(cs)
	sb.position = box.get_center()
	parent.add_child(sb)


static func _mi(mesh: Mesh, mat: Material, xf: Transform3D, parent: Node3D, nm := "m") -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	mi.name = nm
	mi.mesh = mesh
	if mat != null:
		mi.material_override = mat
	mi.transform = xf
	parent.add_child(mi)
	return mi


## Axis-aligned box, positioned by its centre. A colliding box is art (`LAYER_ART`) unless the caller
## says it is a floor the feet stand on (`LAYER_WORLD`).
static func box(parent: Node3D, centre: Vector3, size: Vector3, mat: Material,
		yaw := 0.0, collide := false, layer := LAYER_ART) -> MeshInstance3D:
	var bm := BoxMesh.new()
	bm.size = size
	var xf := Transform3D(Basis(Vector3.UP, yaw), centre)
	var mi := _mi(bm, mat, xf, parent, "box")
	if collide:
		var sb := StaticBody3D.new()
		sb.collision_layer = layer
		var cs := CollisionShape3D.new()
		var sh := BoxShape3D.new()
		sh.size = size
		cs.shape = sh
		sb.add_child(cs)
		sb.transform = xf
		parent.add_child(sb)
	return mi


## A drawn step (a kerbstone, a door step, a threshold): the box drawn, with its own box collider on `LAYER_WORLD`, so
## a foot stands on what is drawn (PR S36-P4b). With `approach` +1 or -1, the step is walked onto across its local +z
## or -z face from a floor at `low_y` (the parent's frame), and a walk aid is added in front of that face: a slope of
## `WALK_AID_DEG` from `low_y` up to the face's top edge, its sloped face ending exactly at that edge (no lip above
## the step's top), on `LAYER_WALK_AID`. With `sides`, its two local x faces get one each as well (a door step stands
## free of the pavement on three sides). Everything is derived from the step's own box; no scene places an aid.
static func step(parent: Node3D, centre: Vector3, size: Vector3, mat: Material, approach := 0,
		low_y := 0.0, yaw := 0.0, sides := false) -> MeshInstance3D:
	var mi := box(parent, centre, size, mat, yaw)
	var xf := Transform3D(Basis(Vector3.UP, yaw), centre)
	var sb := StaticBody3D.new()
	sb.name = "Step"
	sb.collision_layer = LAYER_WORLD
	sb.collision_mask = 0
	var cs := CollisionShape3D.new()
	var sh := BoxShape3D.new()
	sh.size = size
	cs.shape = sh
	sb.add_child(cs)
	sb.transform = xf
	sb.add_to_group(STEP_GROUP)
	parent.add_child(sb)
	var rise := centre.y + size.y * 0.5 - low_y
	if approach == 0 or rise <= 0.0:
		return mi
	# each face walked onto, as the turn about y that brings it to the step's local +z
	var faces: Array[float] = [0.0 if approach > 0 else PI]
	if sides:
		faces.append_array([PI * 0.5, -PI * 0.5])
	for turn in faces:
		var across := size.x if is_zero_approx(sin(turn)) else size.z
		var depth := size.z if is_zero_approx(sin(turn)) else size.x
		_walk_aid(parent, xf * Transform3D(Basis(Vector3.UP, turn), Vector3.ZERO), across, depth, rise,
			low_y - centre.y)
	return mi


## One walk aid in front of the local +z face of a step box (`across` wide, `depth` deep, its top `rise` above the
## floor at `low` -- both in the frame `xf`, whose origin is the box's centre).
static func _walk_aid(parent: Node3D, xf: Transform3D, across: float, depth: float, rise: float,
		low: float) -> void:
	var run := rise / tan(deg_to_rad(WALK_AID_DEG))
	var length := Vector2(run, rise).length()
	# from the floor in front of the face up to the face's top edge
	var foot := Vector3(0.0, low, depth * 0.5 + run)
	var edge := Vector3(0.0, low + rise, depth * 0.5)
	# the slope's upward normal leans toward the floor it rises from
	var n := Vector3(0.0, run, rise) / length
	var aid := StaticBody3D.new()
	aid.name = "WalkAid"
	aid.collision_layer = LAYER_WALK_AID
	aid.collision_mask = 0
	var cs := CollisionShape3D.new()
	var sh := BoxShape3D.new()
	sh.size = Vector3(across, WALK_AID_THICK, length)
	cs.shape = sh
	aid.add_child(cs)
	aid.transform = xf * Transform3D(Basis(Vector3.RIGHT, atan2(rise, run)),
		(foot + edge) * 0.5 - n * WALK_AID_THICK * 0.5)
	parent.add_child(aid)


## Box given by its footprint and a base height -- how buildings are actually
## thought about, and it keeps the caller out of half-height arithmetic.
static func slab(parent: Node3D, x: float, z: float, w: float, d: float,
		y0: float, h: float, mat: Material, yaw := 0.0, collide := false, layer := LAYER_ART) -> MeshInstance3D:
	return box(parent, Vector3(x, y0 + h * 0.5, z), Vector3(w, h, d), mat, yaw, collide, layer)


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
		sb.collision_layer = LAYER_ART
		var cs := CollisionShape3D.new()
		var sh := CylinderShape3D.new()
		sh.radius = maxf(radius_top, radius_bot)
		sh.height = h
		cs.shape = sh
		sb.add_child(cs)
		sb.transform = xf
		parent.add_child(sb)
	return mi


## Triangular prism, positioned by the centre of its bounding box. The ridge
## runs along X, so this is a gable end when `size.z` is the building depth.
static func prism(parent: Node3D, centre: Vector3, size: Vector3, mat: Material,
		yaw := 0.0) -> MeshInstance3D:
	var pm := PrismMesh.new()
	pm.size = size
	pm.left_to_right = 0.5
	return _mi(pm, mat, Transform3D(Basis(Vector3.UP, yaw), centre), parent, "prism")


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
## through things whose geometry is cheaper to keep non-colliding. Art (`LAYER_ART`).
static func blocker(parent: Node3D, pos: Vector3, r: float, h: float) -> void:
	var sb := StaticBody3D.new()
	sb.collision_layer = LAYER_ART
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
	sb.collision_layer = LAYER_ART
	var cs := CollisionShape3D.new()
	var sh := BoxShape3D.new()
	sh.size = size
	cs.shape = sh
	sb.add_child(cs)
	sb.transform = Transform3D(Basis(Vector3.UP, yaw), centre)
	parent.add_child(sb)
