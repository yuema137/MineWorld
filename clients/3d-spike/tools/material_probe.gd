@tool
extends SceneTree
## Priority-A diagnostic: dump what is actually bound on the character, and the
## import flags behind each texture. A face albedo that never reaches the
## material, or a normal map imported as colour, both look like "the face is a
## smear" and neither is fixed by changing a colour.
func _init() -> void:
	var h := Human.build(Human.CANONICAL_HEIGHT, NPC.REF_SKIN, NPC.REF_HAIR,
		NPC.REF_TEE, NPC.REF_JEANS, NPC.REF_SHOE, NPC.REF_HOODIE, NPC.REF_PACK)
	root.add_child(h)
	for mi: MeshInstance3D in h.skeleton.find_children("*", "MeshInstance3D", true, false):
		for i in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(i)
			var key := src.resource_name if src else "?"
			var m := mi.get_surface_override_material(i)
			if m == null:
				print("%-8s %-14s OVERRIDE MISSING" % [mi.name, key])
				continue
			var line := "%-8s %-14s %s" % [mi.name, key, m.get_class()]
			if m is StandardMaterial3D:
				var sm := m as StandardMaterial3D
				line += "  albedo=%s" % _t(sm.albedo_texture)
				line += " normal=%s(%s)" % [_t(sm.normal_texture), sm.normal_enabled]
				line += " rough=%s" % _t(sm.roughness_texture)
				line += " color=%s" % sm.albedo_color
			elif m is ShaderMaterial:
				var shm := m as ShaderMaterial
				for p in ["tex_diffuse", "tex_opacity"]:
					line += " %s=%s" % [p, _t(shm.get_shader_parameter(p))]
				line += " tint=%s cutoff=%s" % [shm.get_shader_parameter("tint"),
					shm.get_shader_parameter("cutoff")]
			print(line)
	print("\n-- import flags --")
	var d := DirAccess.open("res://assets/characters/vitruvian/textures")
	for f in d.get_files():
		if not f.ends_with(".import"):
			continue
		var cfg := ConfigFile.new()
		cfg.load("res://assets/characters/vitruvian/textures/" + f)
		print("%-20s mipmaps=%s compress=%s normal_map=%s srgb=%s" % [
			f.trim_suffix(".import"),
			cfg.get_value("params", "mipmaps/generate", "<default>"),
			cfg.get_value("params", "compress/mode", "<default>"),
			cfg.get_value("params", "compress/normal_map", "<default>"),
			cfg.get_value("params", "flags/srgb", "<default>")])
	quit()

func _t(t: Variant) -> String:
	if t == null:
		return "NONE"
	var tex := t as Texture2D
	return "%s %dx%d" % [tex.resource_path.get_file(), tex.get_width(), tex.get_height()]
