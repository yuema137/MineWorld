extends SceneTree
## Close-up frames of props placed the way a world places them, for judging ground contact and
## part overlap by eye next to `cast.py contact-items` (step-36). Not part of any scene.
##
##   godot --path clients/3d-spike --resolution 640x480 --script res://tools/prop_shots.gd -- \
##       <shots.json> <out_dir>
##
## shots.json is a list of {"name", "slug", "placer", "pos": [x, y, z], "yaw", "focus": [x, y, z],
## "from": [dx, dy, dz], "fov"?, "parts"?}. `placer` names the world's own placing call:
## "props" (`Props.place`: the file's origin at `pos`), "slice" (`SliceProps.put`: the bounding
## box's bottom centre at `pos`) or "hang" (`SliceProps.hang`: the box's top centre at `pos`).
## `focus` is a point in the prop's own space (moved with it), or `focus_world` a point in the
## world; the camera sits at focus + `from`.
## The floor is a grey plane at y = 0 with shadows on, so a gap or a sunk foot reads.

var shots: Array = []
var out_dir := ""
var cam: Camera3D
var stage: Node3D


func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	shots = JSON.parse_string(FileAccess.get_file_as_string(a[0]))
	out_dir = a[1]
	DirAccess.make_dir_recursive_absolute(out_dir)
	var env := Environment.new()
	var sky := Sky.new()
	var pano := PanoramaSkyMaterial.new()
	pano.panorama = load("res://assets/hdri/qwantani_puresky_2k.hdr")
	sky.sky_material = pano
	env.sky = sky
	env.background_mode = Environment.BG_SKY
	env.ambient_light_source = Environment.AMBIENT_SOURCE_SKY
	env.reflected_light_source = Environment.REFLECTION_SOURCE_SKY
	env.tonemap_mode = Environment.TONE_MAPPER_AGX
	env.ssao_enabled = true
	var we := WorldEnvironment.new()
	we.environment = env
	root.add_child(we)
	var sun := DirectionalLight3D.new()
	sun.light_energy = 2.2
	sun.shadow_enabled = true
	sun.rotation_degrees = Vector3(-38.0, 128.0, 0.0)
	root.add_child(sun)
	var ground := MeshInstance3D.new()
	var pm := PlaneMesh.new()
	pm.size = Vector2(12, 12)
	ground.mesh = pm
	var gm := StandardMaterial3D.new()
	gm.albedo_color = Color(0.62, 0.60, 0.56)
	ground.material_override = gm
	root.add_child(ground)
	cam = Camera3D.new()
	root.add_child(cam)
	cam.current = true
	_run.call_deferred()


func _v(x: Variant) -> Vector3:
	var a: Array = x
	return Vector3(a[0], a[1], a[2])


func _run() -> void:
	for s in shots:
		if stage != null:
			stage.queue_free()
		stage = Node3D.new()
		root.add_child(stage)
		var pos := _v(s["pos"])
		var yaw: float = s.get("yaw", 0.0)
		var n: Node3D
		match s["placer"]:
			"props":
				n = Props.place(stage, s["slug"], pos, yaw)
			"slice":
				n = SliceProps.put(stage, s["slug"], pos, yaw, Color.WHITE, 0.28, 0.86, s.get("parts", []))
			"hang":
				n = SliceProps.hang(stage, s["slug"], pos.x, pos.y, pos.z, yaw)
		var focus := _v(s["focus_world"]) if s.has("focus_world") else n.global_transform * _v(s["focus"])
		cam.fov = s.get("fov", 35.0)
		cam.look_at_from_position(focus + _v(s["from"]), focus)
		for i in 12:
			await process_frame
		var img := root.get_viewport().get_texture().get_image()
		img.save_png("%s/%s.png" % [out_dir, s["name"]])
		print("shot %s" % s["name"])
	quit()
