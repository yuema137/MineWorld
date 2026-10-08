## Placing borrowed props so they sit on the surface they are supposed to sit on,
## and measuring them so scale is checked rather than eyeballed.
##
## Poly Haven ships glTF at real metric scale, so nothing here rescales anything
## -- `DEP-8`'s coherence procedure step 1 says to re-export an asset that
## disagrees rather than to scale it in the node tree, and a prop that needs a
## scale factor is reported by `audit()` instead of being quietly fixed.
##
## What this does do is take the guesswork out of placement: `put()` reads the
## prop's own bounding box and stands its feet on the y you name. Hand-placing
## thirty props by eye and then discovering that half of them float two
## centimetres above the counter is the failure this avoids.
class_name SliceProps
extends RefCounted

## Measured extents of every prop the slice places, filled by `put` and dumped
## by `audit`. Keyed by slug.
static var measured: Dictionary = {}


## The node's extents **in its own local space**, with its own transform left
## out so a caller can position it. Children are folded in through their own
## transforms, once each.
static func aabb_of(n: Node) -> AABB:
	var out := AABB()
	var have := false
	if n is MeshInstance3D and (n as MeshInstance3D).mesh != null:
		out = (n as MeshInstance3D).mesh.get_aabb()
		have = true
	for c in n.get_children():
		var sub := aabb_of(c)
		if sub.size == Vector3.ZERO:
			continue
		if c is Node3D:
			sub = (c as Node3D).transform * sub
		out = sub if not have else out.merge(sub)
		have = true
	return out


## Several Poly Haven props are not one object: they are a **set of variants
## laid out side by side in one file**. `shrub_02` is four shrubs spanning
## 6.57 m; `wine_bottles_01` is four bottle shapes at 0.20 m centres;
## `calathea_orbifolia_01` is five plants; `tea_set_01` is ten pieces of
## crockery arranged on an imaginary table. Placing one of those by its overall
## bounding box puts the whole display where one object was wanted.
##
## This was found by measuring, not by looking: `--slice-measure` prints every
## prop's own extents, and a 6.57 m shrub is not a judgement call. `keep` is how
## a caller asks for one member of such a set, by the suffix the file gives it.
static func keep(n: Node3D, suffixes: Array) -> Node3D:
	var wanted: Array[Node] = []
	for c in n.get_children():
		var nm := str(c.name)
		var take := false
		for s in suffixes:
			if nm.ends_with(str(s)):
				take = true
				break
		if take:
			wanted.append(c)
		else:
			c.queue_free()
			n.remove_child(c)
	if wanted.is_empty():
		push_warning("keep(%s): nothing matched %s" % [n.name, suffixes])
	return n


## Place a prop standing on the surface at `pos.y`, centred on `pos.xz`.
## `parts`, when given, selects members of a variant set -- see `keep`.
static func put(parent: Node3D, slug: String, pos: Vector3, yaw := 0.0,
		tint := Color.WHITE, rough_lo := 0.28, rough_hi := 0.86,
		parts: Array = []) -> Node3D:
	var n := Props.gltf(slug)
	parent.add_child(n)
	# Measured before trimming, so `audit` reports what the FILE contains -- the
	# 6.57 m of shrub_02 is the finding, and recording the trimmed variant
	# instead would hide it.
	if not measured.has(slug):
		measured[slug] = aabb_of(n).size
	if not parts.is_empty():
		keep(n, parts)
	var local := aabb_of(n)
	var b := Basis(Vector3.UP, yaw)
	var box := Transform3D(b, Vector3.ZERO) * local
	n.transform = Transform3D(b, pos - Vector3(
		box.position.x + box.size.x * 0.5, box.position.y,
		box.position.z + box.size.z * 0.5))
	if tint != Color.WHITE:
		Props.retint(n, tint, rough_lo, rough_hi)
	_alpha_swap(n, slug)
	return n


## Poly Haven's glTF ships JPG base colour, so a card-foliage material declared
## alpha MASK cuts nothing and every leaf card renders as an opaque dark blade --
## shrub_02 read as a brown fan of sticks. fetch_slice_assets.sh composes the
## published Alpha map into `<slug>_diff_alpha_1k.png`; where that file exists,
## it replaces the base colour on every alpha-tested surface.
static func _alpha_swap(n: Node, slug: String) -> void:
	var path := "%s/%s/textures/%s_diff_alpha_1k.png" % [Props.MODEL_DIR, slug, slug]
	if not ResourceLoader.exists(path):
		return
	var tex := load(path) as Texture2D
	_alpha_apply(n, tex)


static func _alpha_apply(n: Node, tex: Texture2D) -> void:
	for c in n.get_children():
		_alpha_apply(c, tex)
	if not (n is MeshInstance3D) or (n as MeshInstance3D).mesh == null:
		return
	var mi := n as MeshInstance3D
	for i in range(mi.mesh.get_surface_count()):
		var m := mi.get_active_material(i) as BaseMaterial3D
		if m == null or m.transparency == BaseMaterial3D.TRANSPARENCY_DISABLED:
			continue
		var d := m.duplicate() as BaseMaterial3D
		d.albedo_texture = tex
		d.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA_SCISSOR
		d.alpha_scissor_threshold = 0.5
		d.cull_mode = BaseMaterial3D.CULL_DISABLED
		mi.set_surface_override_material(i, d)


## The placed extents of a prop `put` has already positioned, in the parent's
## frame -- what a collider or a neighbouring prop needs to know.
static func placed_box(n: Node3D) -> AABB:
	return n.transform * aabb_of(n)


## Same as `put`, plus a box collider matching the prop's footprint. For
## anything the player must not walk through.
static func put_solid(parent: Node3D, slug: String, pos: Vector3, yaw := 0.0,
		tint := Color.WHITE) -> Node3D:
	var n := put(parent, slug, pos, yaw, tint)
	var box := placed_box(n)
	Build.box_blocker(parent, box.position + box.size * 0.5, box.size * 0.92)
	return n


## Hang a prop from a ceiling: its bounding box's **top** goes at `ceiling_y`,
## not its bottom. A pendant's model includes its own flex and rose, so standing
## it on the ceiling puts the shade above the plaster.
static func hang(parent: Node3D, slug: String, x: float, ceiling_y: float, z: float,
		yaw := 0.0, tint := Color.WHITE) -> Node3D:
	var n := Props.gltf(slug)
	parent.add_child(n)
	var local := aabb_of(n)
	if not measured.has(slug):
		measured[slug] = local.size
	var b := Basis(Vector3.UP, yaw)
	var box := Transform3D(b, Vector3.ZERO) * local
	n.transform = Transform3D(b, Vector3(x, ceiling_y, z) - Vector3(
		box.position.x + box.size.x * 0.5,
		box.position.y + box.size.y,
		box.position.z + box.size.z * 0.5))
	if tint != Color.WHITE:
		Props.retint(n, tint, 0.28, 0.86)
	return n


## Scatter a prop a few times with slight rotation, for shelves and tables.
static func scatter(parent: Node3D, slug: String, origin: Vector3, along: Vector3,
		count: int, seed_v: int, tint := Color.WHITE, jitter := 0.03,
		variants: Array = []) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	for i in range(count):
		var p := origin + along * float(i)
		p.x += rng.randf_range(-jitter, jitter)
		p.z += rng.randf_range(-jitter, jitter)
		var parts: Array = []
		if not variants.is_empty():
			parts = [variants[rng.randi() % variants.size()]]
		put(parent, slug, p, rng.randf() * TAU, tint, 0.28, 0.86, parts)


## What every placed prop actually measures, in metres, so scale is a recorded
## number and not an impression. Printed by the slice's `--measure` mode.
static func audit() -> Array[String]:
	var out: Array[String] = []
	var keys := measured.keys()
	keys.sort()
	for k in keys:
		var s: Vector3 = measured[k]
		out.append("%-28s %5.2f x %5.2f x %5.2f m" % [k, s.x, s.y, s.z])
	return out
