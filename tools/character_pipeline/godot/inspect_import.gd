# SPDX-License-Identifier: MIT
# Prints what Godot 4.7 made of an imported character scene (design §5.9, B10 step 6). The stored-once decision reads
# these lines: the albedo path, the surface count, the bone count with the first three names, and every project file
# whose name contains "base_color".
#
# Run in a scratch project, never in clients/3d-spike:
#   godot --headless --path <project> --import
#   godot --headless --path <project> --script res://inspect_import.gd -- res://<id>.gltf
extends SceneTree


func _init() -> void:
	var scene_path := ""
	for arg in OS.get_cmdline_user_args():
		scene_path = arg
	var packed := load(scene_path) as PackedScene
	if packed == null:
		print("load failed: ", scene_path)
		quit(1)
		return
	var root := packed.instantiate()
	var meshes: Array[MeshInstance3D] = []
	var skeletons: Array[Skeleton3D] = []
	_collect(root, meshes, skeletons)
	var surfaces := 0
	var material_name := ""
	var albedo := ""
	for mesh_node in meshes:
		surfaces += mesh_node.mesh.get_surface_count()
		for surface in range(mesh_node.mesh.get_surface_count()):
			var material := mesh_node.get_active_material(surface)
			if material == null:
				continue
			material_name = material.resource_name
			if material is BaseMaterial3D and material.albedo_texture != null:
				albedo = material.albedo_texture.resource_path
	print("surfaces=", surfaces)
	print("material=", material_name)
	print("albedo=", albedo)
	var bone_count := 0
	var first_bones: Array[String] = []
	if not skeletons.is_empty():
		var skeleton := skeletons[0]
		bone_count = skeleton.get_bone_count()
		for index in range(mini(3, bone_count)):
			first_bones.append(skeleton.get_bone_name(index))
	print("bones=", bone_count, " first=", first_bones)
	var hits: Array[String] = []
	_walk("res://", hits)
	for hit in hits:
		print("base_color file: ", hit)
	quit(0)


func _collect(node: Node, meshes: Array[MeshInstance3D], skeletons: Array[Skeleton3D]) -> void:
	if node is MeshInstance3D:
		meshes.append(node)
	if node is Skeleton3D:
		skeletons.append(node)
	for child in node.get_children():
		_collect(child, meshes, skeletons)


func _walk(dir_path: String, hits: Array[String]) -> void:
	var dir := DirAccess.open(dir_path)
	if dir == null:
		return
	dir.list_dir_begin()
	var entry := dir.get_next()
	while entry != "":
		var full := dir_path.path_join(entry)
		if dir.current_is_dir():
			_walk(full, hits)
		elif "base_color" in entry:
			hits.append(full)
		entry = dir.get_next()
	dir.list_dir_end()
