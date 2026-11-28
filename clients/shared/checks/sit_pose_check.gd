extends SceneTree

## S34-B: exact pre-amendment scalar corpus, versioned vector loading and pose precedence.
## Run from clients/2d: godot --headless --path clients/2d --script ../shared/checks/sit_pose_check.gd
var failures := 0
var scratch := ""

func check(ok: bool, label: String) -> void:
	if not ok:
		failures += 1
		print("[FAIL] ", label)

func load_rows(rows: Array, format := 2) -> MineWorldSitRule:
	var file := FileAccess.open(scratch.path_join("sit_points/check.yaml"), FileAccess.WRITE)
	file.store_string(JSON.stringify({"format": format, "scope": {"world": "check"}, "points": rows}))
	file.close()
	return MineWorldSitRule.load_for(scratch, "check")

func same_pose(actual: Dictionary, expected: Dictionary) -> bool:
	var normalized := expected.duplicate()
	normalized.facing = int(expected.facing)
	for key in ["at", "pelvis"]:
		normalized[key] = Vector2(expected[key][0], expected[key][1])
	return actual == normalized

func _init() -> void:
	var repo := ProjectSettings.globalize_path("res://").path_join("../..").simplify_path()
	var fixture: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(
		repo.path_join("tools/cli/tests/fixtures/seat-surfaces.json")))
	for path in fixture.source_sha256:
		check(FileAccess.get_sha256(repo.path_join(path)) == fixture.source_sha256[path], "measurement source unchanged: " + path)
	var rule := MineWorldSitRule.load_for(repo.path_join("presentation/mineworld-default/3D"), "lakeside")
	check(rule.problem == "" and rule.points.size() == 29, "all legacy rows load")
	for row in fixture.legacy.rows:
		var tags := PackedStringArray(row.tags)
		# S34-c2 (ARC-172): no fallback position; the world's own point, disclosed, gives the pose.
		check(rule.pose_of(tags, null, row.point).is_empty(), "no fallback position " + row.point)
		check(same_pose(rule.pose_of(tags, row.world, row.point), row.world_expected), "legacy world point " + row.point)
		check(same_pose(rule.pose_of(tags, row.disclosed, row.point), row.disclosed_expected), "legacy disclosed " + row.point)

	scratch = OS.get_user_data_dir().path_join("s34b-pose-%d" % OS.get_process_id())
	DirAccess.make_dir_recursive_absolute(scratch.path_join("sit_points"))
	var p := {"point": "chair", "tags": ["room"], "prop": "chair-model", "seat_height_m": 0.6893,
		"seat_displacement_m": {"east": 0.125, "north": -0.375},
		"fallback": {"at": {"x": 1000, "y": 2000}, "facing": 37123}}
	rule = load_rows([p])
	check(rule.problem == "", "vector file loads")
	var pose := rule.pose_of(["room"], null, "chair")
	check(pose.pelvis == Vector2(1125, 1625) and pose.facing == 37123, "non-cardinal facing does not rotate place-local displacement")
	var disclosed := [{"key": "chair", "at": {"x": -200, "y": 400}, "facing": 222222}]
	pose = rule.pose_of(["room"], disclosed, "chair")
	check(pose.pelvis == Vector2(-75, 25) and pose.at == Vector2(-200, 400) and pose.facing == 222222,
		"disclosed at/facing override fallback while vector remains place-local")
	var mirrored := p.duplicate(true)
	mirrored.point = "renamed-chair"
	mirrored.tags = ["renamed-room"]
	mirrored.fallback.at.x = -1000
	mirrored.fallback.facing = 360000 - 37123
	mirrored.seat_displacement_m.east = -0.125
	pose = load_rows([mirrored]).pose_of(["renamed-room"], null, "renamed-chair")
	check(pose.pelvis == Vector2(-1125, 1625), "renamed/mirrored vector pose")
	var no_position := p.duplicate(true)
	no_position.erase("fallback")
	check(load_rows([no_position]).pose_of(["room"], null, "chair").is_empty(), "no position stays empty")
	pose = rule.pose_of(["other-room"], disclosed, "chair")
	check(not pose.depicted and pose.pelvis == Vector2(-200, 400), "undepicted disclosed point stays at semantic position")
	var scalar := p.duplicate(true)
	scalar.erase("seat_displacement_m")
	check(load_rows([scalar]).pose_of(["room"], null, "chair") == load_rows([scalar], 1).pose_of(["room"], null, "chair"),
		"format 2 scalar defaults equal format 1")

	var bad_vectors: Array = [null, [], {}, {"east": 0}, {"east": 0, "north": 0, "up": 0},
		{"east": true, "north": 0}, {"east": 0, "north": "1"}, {"east": 0, "north": null},
		{"east": 1.0e100, "north": 0}]
	for v in bad_vectors:
		var bad := p.duplicate(true)
		bad.point = "invalid-row"
		bad.seat_displacement_m = v
		rule = load_rows([p, bad])
		check(rule.points.is_empty() and rule.problem.contains("invalid-row"), "malformed vector rejects entire file: " + str(v))
	var mixed := p.duplicate(true)
	mixed.seat_offset_m = 0.4
	check(not load_rows([mixed]).problem.is_empty(), "mixed scalar/vector rejected")
	check(not load_rows([p], 1).problem.is_empty(), "format 1 vector rejected")
	check(not load_rows([scalar], 3).problem.is_empty(), "unknown format rejected")
	for token in ["1e999", "-1e999"]:
		var file := FileAccess.open(scratch.path_join("sit_points/check.yaml"), FileAccess.WRITE)
		file.store_string('{"format":2,"scope":{"world":"check"},"points":[{"point":"overflow","seat_displacement_m":{"east":%s,"north":0}}]}' % token)
		file.close()
		rule = MineWorldSitRule.load_for(scratch, "check")
		check(rule.points.is_empty() and rule.problem != "", "parser overflow rejected")
	DirAccess.remove_absolute(scratch.path_join("sit_points/check.yaml"))
	DirAccess.remove_absolute(scratch.path_join("sit_points"))
	DirAccess.remove_absolute(scratch)
	print("sit_pose_check: %s (29 legacy rows, disclosed + fallback; vector schema/precedence)" % ("PASS" if failures == 0 else "FAIL"))
	quit(0 if failures == 0 else 1)
