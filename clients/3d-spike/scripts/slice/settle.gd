## Placement by support (S36-P5c; `.structured-coding/plans/mvp1/pr-s36-p5c-content-fixes.md` §3.1).
##
## `SliceProps.put` stands a prop's bounding box on a `y` its caller typed. That `y` is wrong
## whenever the caller misjudged the surface (a worktop's top, a saucer's rim) or the bounds are
## loose (a glTF node rotated inside the file stands 0.3 m above its own lowest vertex). The props
## probe found 60 such rows (`slice_probe_props.gd`, `cast.py contact-scene`). This runs the probe's
## own rule once, at placement: nine rays cast down at the prop's bounds' corners, edge midpoints and
## centre, from 50 mm above its plane, against the drawn meshes under the same parent (the prop's own
## excluded); the hit nearest the plane is the support, and the prop moves vertically onto it.
##
## The plane is the prop's true lowest vertex, or for a `ground` mount the rest plane `contact.json`
## declares (`rest_mm`: grass rooted below the soil, a manhole cover set flush). `wall` and `ceiling`
## mounts, and a model with no entry, are left where the caller put them. A move larger than
## `MAX_MOVE` is not made: a prop that far from every surface under it is misplaced data, and dropping
## it to the floor would hide that.
##
## Geometry is asked through the engine's `TriangleMesh` (a BVH ray query, cached per mesh), so no
## physics frame and no collider is needed while the scene is still being built.
class_name SliceSettle
extends RefCounted

const MOUNTS_FILE := "res://assets/models/contact.json"
## The probe's numbers (`SliceProbeProps.RAY_START`, `RAY_DEPTH`): rays start this far above the
## plane, metres, and look this far down.
const RAY_START := 0.05
const RAY_DEPTH := 3.0
## Metres. Beyond this the prop is left where it was, with a warning.
const MAX_MOVE := 0.30
const SETTLED := ["floor", "surface", "ground"]

## slug -> {"mount": String, "rest": metres}
static var _mounts := {}
static var _loaded := false
## "<slug>|<parts>" -> the prop's lowest vertex in its own frame, metres
static var _lowest := {}
## Mesh instance id -> TriangleMesh (null when the mesh has no faces)
static var _tris := {}


## The model's `contact.json` entry: {"mount", "rest"}, or {} when it has none.
static func mount_of(slug: String) -> Dictionary:
	if not _loaded:
		_loaded = true
		_load()
	return _mounts.get(slug, {})


static func _load() -> void:
	var f := FileAccess.open(MOUNTS_FILE, FileAccess.READ)
	if f == null:
		push_warning("settle: cannot read %s; props stay where they are put" % MOUNTS_FILE)
		return
	var parsed: Variant = JSON.parse_string(f.get_as_text())
	if typeof(parsed) != TYPE_DICTIONARY or typeof(parsed.get("items")) != TYPE_DICTIONARY:
		push_warning("settle: %s holds no items" % MOUNTS_FILE)
		return
	for slug in parsed["items"]:
		var e: Variant = parsed["items"][slug]
		if typeof(e) == TYPE_DICTIONARY:
			_mounts[String(slug)] = {"mount": String(e.get("mount", "")),
				"rest": float(e.get("rest_mm", 0.0)) / 1000.0}


## The height of `n`'s plane in its parent's frame: its declared rest plane for a `ground` mount,
## else its true lowest vertex. NAN for a mount that never settles.
static func plane_of(n: Node3D, slug: String, key := "") -> float:
	var m := mount_of(slug)
	var kind := String(m.get("mount", ""))
	if not (kind in SETTLED):
		return NAN
	if kind == "ground":
		return (n.transform * Vector3(0.0, float(m["rest"]), 0.0)).y
	return n.transform.origin.y + n.transform.basis.get_scale().y * _lowest_local(n, slug + "|" + key)


## Puts `n`'s plane at `y`: a caller's height names where the prop meets its support (a planted
## shrub's soil line, not the bottom of its roots or of a loose box).
static func stand(n: Node3D, slug: String, key: String, y: float) -> void:
	var plane := plane_of(n, slug, key)
	if not is_nan(plane):
		n.position.y += y - plane


## Moves `n`, a prop just placed as a direct child of `parent`, vertically onto the drawn surface
## under it. `key` names the kept members (`SliceProps.keep`) so the lowest point is cached per
## variant. Returns the distance moved down, metres (0 when nothing moved).
static func settle(parent: Node3D, n: Node3D, slug: String, key := "") -> float:
	var plane := plane_of(n, slug, key)
	if is_nan(plane):
		return 0.0
	var box := n.transform * SliceProps.aabb_of(n)
	var start := box.position.y + RAY_START
	if String(mount_of(slug)["mount"]) == "ground":
		start = maxf(start, plane + RAY_START)
	var meshes := []
	_collect(parent, Transform3D.IDENTITY, n, box, start, meshes)
	if meshes.is_empty():
		return 0.0
	var gap := INF
	for fx in [0.0, 0.5, 1.0]:
		for fz in [0.0, 0.5, 1.0]:
			var x: float = box.position.x + box.size.x * fx
			var z: float = box.position.z + box.size.z * fz
			var h := _cast(meshes, Vector3(x, start, z))
			if not is_nan(h) and absf(plane - h) < absf(gap):
				gap = plane - h
	if gap == INF:
		return 0.0
	if absf(gap) > MAX_MOVE:
		push_warning("settle: %s (%s) is %.0f mm from every surface under it; left where it was"
			% [slug, n.name, absf(gap) * 1000.0])
		return 0.0
	n.position.y -= gap
	return gap


## The lowest vertex of every mesh under `n`, in `n`'s own frame (its own transform left out), so it
## holds for any yaw the prop is placed at. Cached per model and kept members.
static func _lowest_local(n: Node3D, key: String) -> float:
	if _lowest.has(key):
		return _lowest[key]
	var low := INF
	var stack: Array = [[n, Transform3D.IDENTITY]]
	while not stack.is_empty():
		var item: Array = stack.pop_back()
		var node: Node = item[0]
		var xf: Transform3D = item[1]
		if node is MeshInstance3D and (node as MeshInstance3D).mesh != null:
			var verts: PackedVector3Array = xf * (node as MeshInstance3D).mesh.get_faces()
			for v in verts:
				low = minf(low, v.y)
		for c in node.get_children():
			if c is Node3D:
				stack.append([c, xf * (c as Node3D).transform])
	if low == INF:
		low = SliceProps.aabb_of(n).position.y
	_lowest[key] = low
	return low


## Every drawn, unskinned mesh under `node` (but not under `skip`) whose bounds meet the column
## under `box` from `start` down, as [mesh, transform in `parent`'s frame].
static func _collect(node: Node, xf: Transform3D, skip: Node3D, box: AABB, start: float,
		out: Array) -> void:
	for c in node.get_children():
		if c == skip or c is Skeleton3D or not (c is Node3D) or not (c as Node3D).visible:
			continue
		var cxf: Transform3D = xf * (c as Node3D).transform
		if c is MeshInstance3D:
			var mi := c as MeshInstance3D
			if mi.mesh != null and mi.skin == null:
				var b := cxf * mi.mesh.get_aabb()
				if b.end.x >= box.position.x and b.position.x <= box.end.x \
						and b.end.z >= box.position.z and b.position.z <= box.end.z \
						and b.position.y <= start and b.end.y >= start - RAY_DEPTH:
					out.append([mi.mesh, cxf])
		_collect(c, cxf, skip, box, start, out)


## The highest drawn surface under `from` (a ray straight down, RAY_DEPTH long), or NAN.
static func _cast(meshes: Array, from: Vector3) -> float:
	var best := NAN
	var to := from + Vector3.DOWN * RAY_DEPTH
	for entry in meshes:
		var tm := _tri(entry[0])
		if tm == null:
			continue
		var xf: Transform3D = entry[1]
		var inv := xf.affine_inverse()
		var hit := tm.intersect_segment(inv * from, inv * to)
		if hit.is_empty():
			continue
		var h: float = (xf * (hit["position"] as Vector3)).y
		if is_nan(best) or h > best:
			best = h
	return best


static func _tri(mesh: Mesh) -> TriangleMesh:
	var id := mesh.get_instance_id()
	if not _tris.has(id):
		_tris[id] = mesh.generate_triangle_mesh()
	return _tris[id]
