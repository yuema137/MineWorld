extends SceneTree
const OUT = "/tmp/mineworld-s34e-seat-measure/"
func vec(v: Vector3) -> Array:
	return [v.x, v.y, v.z]
func box(b: AABB) -> Dictionary:
	return {"min": vec(b.position), "max": vec(b.end), "center": vec(b.get_center())}
func read_json(path: String) -> Variant:
	return JSON.parse_string(FileAccess.get_file_as_string(path))
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var survey: Dictionary = read_json(OUT + "mesh-survey.json")
	var surfaces: Dictionary = read_json(OUT + "candidate-surfaces.json")
	var result := {"engine": Engine.get_version_info(), "models": {}, "placements": []}
	for slug in survey:
		var n := Props.gltf(slug)
		result.models[slug] = {"aabb": box(SliceProps.aabb_of(n)), "true_box": box(SliceProps.true_box(n)), "root_origin": vec(n.position), "root_scale": vec(n.scale), "mount": SliceSettle.mount_of(slug)}
		n.free()
	var furnishings := SliceFurnishings.load_pack("../../presentation/mineworld-default/3D")
	assert(furnishings.problems.is_empty())
	var worlds: Dictionary = read_json(OUT + "engine-world-inputs.json")
	var parsed := {"place": "cafe", "world": "social-cafe", "place-shape": worlds["social-cafe"].body}
	assert(worlds["social-cafe"].body.furnishings == worlds["market-town"].body.furnishings)
	var parent := Node3D.new()
	root.add_child(parent)
	furnishings.draw_parsed(parent, parsed, Vector3.ZERO)
	var placed: Node3D = furnishings.nodes[parsed.place]
	for f in parsed["place-shape"]["furnishings"]:
		var binding: Dictionary = furnishings.kinds[f.kind]
		var slug: String = binding.get("model", "")
		if not survey.has(slug):
			continue
		var n := placed.get_node(NodePath(String(f.key).validate_node_name())) as Node3D
		var o: Array = survey[slug].centering_and_floor_subtraction_m
		var c: Array = surfaces[slug].area_weighted_center_m
		var raw_anchor := Vector3(c[0]+o[0], float(surfaces[slug].center_ray_seat_top_m)+o[1], c[2]+o[2])
		var anchor := n.transform * raw_anchor
		var meshes := []
		SliceSettle._collect(n, n.transform, null, SliceProps.true_box(n), anchor.y + 0.015, meshes)
		var hit := SliceSettle._cast(meshes, anchor + Vector3.UP * 0.015)
		var yaw := SliceFurnishings.yaw_of(binding, int(f.facing))
		var at := MineWorldSpace.to_3d(f.at)
		var rot := Basis(Vector3.UP, yaw)
		var raw_aabb: Dictionary = result.models[slug].aabb
		var a: Array = raw_aabb.center
		var predicted := at - rot * Vector3(a[0], float(result.models[slug].true_box.min[1]), a[2])
		result.placements.append({"key": f.key, "model": slug, "front_md": binding.front_md, "facing_md": f.facing, "yaw_rad": yaw, "at_scene_m": vec(at), "placed_origin_m": vec(n.position), "stand_only_origin_m": vec(predicted), "settling_translation_m": vec(n.position-predicted), "actual_true_box": box(SliceProps.true_box(n)), "anchor_scene_m": vec(anchor), "seat_ray_height_m": null if is_nan(hit) else hit, "anchor_ray_gap_mm": null if is_nan(hit) else (hit-anchor.y)*1000, "support_plane_m": SliceSettle.plane_of(n, slug), "mesh_ray_count": meshes.size()})
	result["imported_models"] = Props.imported
	result["fallback_models"] = Props.fallback
	var file := FileAccess.open(OUT + "engine-results.json", FileAccess.WRITE)
	file.store_string(JSON.stringify(result, "\t") + "\n")
	print("ENGINE_MEASUREMENT models=", result.models.size(), " seats=", result.placements.size(), " imported=", Props.imported, " fallback=", Props.fallback)
	parent.free()
	for cached in Props._scenes.values():
		if is_instance_valid(cached):
			cached.free()
	Props._scenes.clear()
	SliceSettle._tris.clear()
	await process_frame
	quit()
