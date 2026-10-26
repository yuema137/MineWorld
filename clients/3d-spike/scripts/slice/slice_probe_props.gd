## The drawn props, dumped for the scene check (S36-P5 commit 5;
## `.structured-coding/plans/mvp1/pr-s36-p5-scene-layout.md` §5).
##
##   --slice-props      in the café, on the street (walked out on foot) and in the
##                      store (walked into on foot), the same walk as
##                      `--slice-geometry`: one `props-<place>.json` per place under
##                      `--props-dump=<dir>` (default `artifacts/props`, relative to
##                      the repository), holding
##                        the disclosed `place-shape` as observed (mm, place frame)
##                          and the same in the scene's frame (`SliceLayout.to_scene`,
##                          so the check never re-implements the mapping);
##                        the body band (`SliceProbeGeometry.BAND_LOW`/`BAND_HIGH`);
##                        every prop node `Props.gltf` tagged (`mw_prop`) whose
##                          bounds' centre lies on the place's floor: its model, the
##                          members `SliceProps.keep` left (`mw_parts`, `nodes`), its
##                          global 4 x 4 and nine support samples.
##
## A support sample is the height of the drawn surface under one of nine points of
## the prop's footprint (its bounds' corners, edge midpoints and centre), cast down
## from 50 mm above the prop's lowest point (above its declared rest plane for a
## `ground` mount in `contact.json`). It is cast against DRAWN geometry, not
## colliders: a `put_solid` blocker is 0.92 x the prop's height, so a cup would meet
## it under the tabletop. The probe therefore builds temporary trimesh colliders from
## every drawn, unskinned `MeshInstance3D` near the place on a probe-only physics
## layer, excludes the prop's own, and frees them after the dump.
##
## The probe judges nothing: `cast.py contact-scene <dir>` does (`mwcast/scene_check.py`).
## It fails only when a place is not reached or a file cannot be written.
class_name SliceProbeProps
extends SliceProbeGeometry

const PROPS_MODES := ["props"]
## The probe-only physics layer of the temporary colliders (layer 32), which nothing else uses.
const PROBE_LAYER := 1 << 31
## Rays start this far above the prop's lowest point (or its rest plane), metres. The design's 5 mm
## could not see a prop sunk deeper than 5 mm (the ray would start inside what holds it); 50 mm sees
## a sink up to 45 mm and still starts under a table a chair is tucked beneath.
const RAY_START := 0.05
## And look this far down, metres.
const RAY_DEPTH := 3.0
## A prop belongs to a place when its bounds' centre lies on the place's floor grown by this, metres,
## and between these heights above the place's floor.
const MEMBER_GROW := 0.1
const MEMBER_BELOW := 1.0
const MEMBER_ABOVE := 4.0
## Meshes near the place become support colliders: the floor grown by this, metres.
const SUPPORT_GROW := 1.0
const DEFAULT_DIR := "artifacts/props"
const MOUNTS_FILE := "res://assets/models/contact.json"

var _props_total := 0
var _places_dumped := 0
## slug -> its ground rest plane, metres in its own frame (only `ground` mounts).
var _rest := {}


static func requested() -> bool:
	return requested_of(PROPS_MODES) != ""


func _modes() -> Array:
	return PROPS_MODES


func _run(mode: String) -> void:
	match mode:
		"props": await _props_dump()
		_: await super(mode)


func _props_dump() -> void:
	print("== S36-P5 -- the drawn props, dumped for contact-scene ==\n")
	_load_rest_planes()
	print("dump directory %s" % _dump_dir())
	if not await _walk_places(_dump_place):
		return
	print("\n%d places dumped, %d props" % [_places_dumped, _props_total])
	print("all props checks pass" if _geo_fails == 0 else "%d PROPS CHECKS FAILED" % _geo_fails)
	slice.link.client.disconnect_from_world("probe done")
	await _hold(0.2)


static func _dump_dir() -> String:
	var dir := DEFAULT_DIR
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--props-dump="):
			dir = a.trim_prefix("--props-dump=")
	if dir.is_absolute_path():
		return dir
	return ProjectSettings.globalize_path("res://").path_join("../..").path_join(dir).simplify_path()


func _load_rest_planes() -> void:
	var f := FileAccess.open(MOUNTS_FILE, FileAccess.READ)
	if f == null:
		return
	var parsed: Variant = JSON.parse_string(f.get_as_text())
	if typeof(parsed) != TYPE_DICTIONARY or typeof(parsed.get("items")) != TYPE_DICTIONARY:
		return
	for slug in parsed["items"]:
		var e: Variant = parsed["items"][slug]
		if typeof(e) == TYPE_DICTIONARY and String(e.get("mount", "")) == "ground":
			_rest[slug] = float(e.get("rest_mm", 0.0)) / 1000.0


## One place: its shape, its props and their support samples, written to `props-<place>.json`.
func _dump_place(link: SliceLink, place: String, label: String) -> void:
	var shape: Variant = link.client.latest.component_value(place, "place-shape")
	if typeof(shape) != TYPE_DICTIONARY:
		_geo_fail(label, "the place discloses no place-shape")
		return
	var base := link.to_scene(place, {"x": 0, "y": 0}).y
	var floor_lo := _flat(link, place, shape["floor"]["min"])
	var floor_hi := _flat(link, place, shape["floor"]["max"])
	var lo := Vector2(minf(floor_lo.x, floor_hi.x), minf(floor_lo.y, floor_hi.y))
	var hi := Vector2(maxf(floor_lo.x, floor_hi.x), maxf(floor_lo.y, floor_hi.y))
	var solids := []
	for s in shape.get("solids", []):
		var a := _flat(link, place, s["min"])
		var b := _flat(link, place, s["max"])
		solids.append({"min": [minf(a.x, b.x), minf(a.y, b.y)], "max": [maxf(a.x, b.x), maxf(a.y, b.y)],
			"height": float(s["height"]) / 1000.0})
	# the props of this place, and the drawn meshes near it as temporary colliders
	var props: Array[Node3D] = []
	_tagged(slice, props, lo, hi, base)
	var probe_root := Node3D.new()
	probe_root.name = "PropsProbeSupport"
	slice.add_child(probe_root)
	var bodies := {}   # MeshInstance3D instance id -> its temporary body's RID
	_support_colliders(slice, probe_root, bodies, lo - Vector2.ONE * SUPPORT_GROW,
		hi + Vector2.ONE * SUPPORT_GROW, base)
	await get_tree().physics_frame
	await get_tree().physics_frame
	var space := player.get_world_3d().direct_space_state
	var rows := []
	for n in props:
		rows.append(_prop_row(space, n, bodies))
	probe_root.queue_free()
	var out := {
		"format": 1,
		"place": place,
		"label": label,
		"place_shape": shape,
		"scene": {"base_y": base, "floor": {"min": [lo.x, lo.y], "max": [hi.x, hi.y]}, "solids": solids},
		"band": {"low": BAND_LOW, "high": BAND_HIGH},
		"props": rows,
	}
	var dir := _dump_dir()
	DirAccess.make_dir_recursive_absolute(dir)
	var path := dir.path_join("props-%s.json" % place.validate_filename())
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		_geo_fail(label, "cannot write %s" % path)
		return
	f.store_string(JSON.stringify(out, " ") + "\n")
	f.close()
	_places_dumped += 1
	_props_total += rows.size()
	print("-- %s: place %s, %d props, %d support meshes, %d solids -> %s" % [label, place, rows.size(),
		bodies.size(), solids.size(), path])


## A place-frame point (mm) -> the scene's (x, z), metres.
static func _flat(link: SliceLink, place: String, p: Variant) -> Vector2:
	var v := link.to_scene(place, p)
	return Vector2(v.x, v.z)


## Every visible node `Props.gltf` tagged whose bounds' centre lies on the floor (grown) and near its height.
func _tagged(n: Node, out: Array[Node3D], lo: Vector2, hi: Vector2, base: float) -> void:
	if n is Node3D and n.has_meta("mw_prop"):
		var node := n as Node3D
		if node.is_visible_in_tree():
			var box := _global_box(node)
			var c := box.position + box.size * 0.5
			if c.x >= lo.x - MEMBER_GROW and c.x <= hi.x + MEMBER_GROW and c.z >= lo.y - MEMBER_GROW \
					and c.z <= hi.y + MEMBER_GROW and c.y >= base - MEMBER_BELOW and c.y <= base + MEMBER_ABOVE:
				out.append(node)
		return   # a prop's own children are its parts, never another prop
	for ch in n.get_children():
		_tagged(ch, out, lo, hi, base)


static func _global_box(n: Node3D) -> AABB:
	var parent := n.get_parent() as Node3D
	var local := n.transform * SliceProps.aabb_of(n)
	return parent.global_transform * local if parent != null else local


## A temporary trimesh body, on PROBE_LAYER only, for every drawn, unskinned mesh whose bounds meet
## the region (scene x/z, and from 1 m below the floor to 4 m above it).
func _support_colliders(n: Node, root: Node3D, bodies: Dictionary, lo: Vector2, hi: Vector2,
		base: float) -> void:
	if n == root:
		return
	if n is MeshInstance3D:
		var mi := n as MeshInstance3D
		if mi.mesh != null and mi.skin == null and mi.is_visible_in_tree():
			var box := mi.global_transform * mi.mesh.get_aabb()
			if box.end.x >= lo.x and box.position.x <= hi.x and box.end.z >= lo.y and box.position.z <= hi.y \
					and box.end.y >= base - MEMBER_BELOW and box.position.y <= base + MEMBER_ABOVE:
				var shape := mi.mesh.create_trimesh_shape()
				if shape != null:
					var body := StaticBody3D.new()
					body.collision_layer = PROBE_LAYER
					body.collision_mask = 0
					var cs := CollisionShape3D.new()
					cs.shape = shape
					body.add_child(cs)
					root.add_child(body)
					body.global_transform = mi.global_transform
					bodies[mi.get_instance_id()] = body.get_rid()
	if n is Skeleton3D:
		return   # people: never a prop's support
	for ch in n.get_children():
		_support_colliders(ch, root, bodies, lo, hi, base)


## One prop's row: its model, kept members, global transform and support samples.
func _prop_row(space: PhysicsDirectSpaceState3D, n: Node3D, bodies: Dictionary) -> Dictionary:
	var slug := String(n.get_meta("mw_prop"))
	var own: Array[RID] = []
	_own_bodies(n, bodies, own)
	var box := _global_box(n)
	var start := box.position.y + RAY_START
	if _rest.has(slug):
		start = maxf(start, (n.global_transform * Vector3(0.0, float(_rest[slug]), 0.0)).y + RAY_START)
	var samples := []
	for fx in [0.0, 0.5, 1.0]:
		for fz in [0.0, 0.5, 1.0]:
			var x: float = box.position.x + box.size.x * fx
			var z: float = box.position.z + box.size.z * fz
			var q := PhysicsRayQueryParameters3D.create(Vector3(x, start, z), Vector3(x, start - RAY_DEPTH, z),
				PROBE_LAYER, own)
			var hit := space.intersect_ray(q)
			samples.append([_r(x), _r(z), _r(float(hit["position"].y)) if not hit.is_empty() else null])
	var nodes := []
	for ch in n.get_children():
		nodes.append(String(ch.name))
	var t := n.global_transform
	var b := t.basis
	return {
		"slug": slug,
		"name": String(slice.get_path_to(n)),
		"parts": n.get_meta("mw_parts") if n.has_meta("mw_parts") else null,
		"nodes": nodes,
		"transform": [
			_r(b.x.x), _r(b.y.x), _r(b.z.x), _r(t.origin.x),
			_r(b.x.y), _r(b.y.y), _r(b.z.y), _r(t.origin.y),
			_r(b.x.z), _r(b.y.z), _r(b.z.z), _r(t.origin.z),
			0.0, 0.0, 0.0, 1.0],
		"lowest_y": _r(box.position.y),
		"support": samples,
	}


func _own_bodies(n: Node, bodies: Dictionary, out: Array[RID]) -> void:
	if bodies.has(n.get_instance_id()):
		out.append(bodies[n.get_instance_id()])
	for ch in n.get_children():
		_own_bodies(ch, bodies, out)


static func _r(v: float) -> float:
	return snappedf(v, 0.00001)
