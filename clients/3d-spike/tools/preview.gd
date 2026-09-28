extends SceneTree
## Diagnostic turntable for the rigged human. Not part of the demo.
##   godot --path clients/3d-spike --resolution 1000x1000 --script res://tools/preview.gd -- Walk
func _initialize() -> void:
	var anim := "Walk"
	var a := OS.get_cmdline_user_args()
	if a.size() > 0: anim = a[0]
	var env := Environment.new()
	var sky := Sky.new()
	var pano := PanoramaSkyMaterial.new()
	pano.panorama = load("res://assets/hdri/kloofendal_48d_partly_cloudy_puresky_2k.hdr")
	sky.sky_material = pano
	env.sky = sky
	env.background_mode = Environment.BG_SKY
	env.ambient_light_source = Environment.AMBIENT_SOURCE_SKY
	env.reflected_light_source = Environment.REFLECTION_SOURCE_SKY
	env.tonemap_mode = Environment.TONE_MAPPER_AGX
	env.tonemap_exposure = 1.18
	env.tonemap_white = 6.0
	env.ssao_enabled = true
	var we := WorldEnvironment.new(); we.environment = env; root.add_child(we)
	var sun := DirectionalLight3D.new()
	sun.light_color = Color(1.0, 0.88, 0.71); sun.light_energy = 2.35
	sun.shadow_enabled = true
	sun.rotation_degrees = Vector3(-26.0, 128.0, 0.0)
	root.add_child(sun)
	var h := Human.build(1.72, NPC.REF_SKIN, NPC.REF_HAIR, NPC.REF_TEE,
		NPC.REF_JEANS, NPC.REF_SHOE, NPC.REF_HOODIE, NPC.REF_PACK)
	root.add_child(h)
	if anim == "Sit":
		h.sit()
	elif anim in ["Walk", "Idle"]:
		h.set_gait(1.45 if anim == "Walk" else 0.0)
	else:
		h.debug_clip(anim)
	var ground := MeshInstance3D.new()
	var pm := PlaneMesh.new(); pm.size = Vector2(8, 8); ground.mesh = pm
	var gm := StandardMaterial3D.new(); gm.albedo_color = Color(0.62, 0.60, 0.56)
	ground.material_override = gm
	root.add_child(ground)
	var cam := Camera3D.new(); root.add_child(cam); cam.fov = 45.0
	print("bones=", h.skeleton.get_bone_count(), " motion_scale=", h.skeleton.motion_scale)
	var runner := Node.new()
	runner.set_script(preload("res://tools/preview_runner.gd"))
	runner.set("cam", cam)
	runner.set("anim", anim)
	root.add_child(runner)
