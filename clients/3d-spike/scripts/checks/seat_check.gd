extends SceneTree

## The 3D client's sit decision over a recorded set of observations, headless (S33-f AC-f-4).
##
##   godot --headless --path clients/3d-spike --script res://scripts/checks/seat_check.gd
##
## Feeds each frame of the fixture to this client's own wiring (`SliceSeating`, as `SliceLink` calls it)
## and prints `SEATED <at> <id> <point>` per person drawn seated, in id order, then
## `seat_check: <n> frames`. It judges nothing itself: `tools/cli/tests/client_sit_rule.rs` compares
## these lines with the 2D client's and with the fixture's expectation.
##
##   --fixture=<path>   the observations (default tests/fixtures/observations/sitting/contested-long-table.json)

const DEFAULT_FIXTURE := "tests/fixtures/observations/sitting/contested-long-table.json"


func _init() -> void:
	var repo := ProjectSettings.globalize_path("res://").path_join("../..").simplify_path()
	var fixture := repo.path_join(DEFAULT_FIXTURE)
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--fixture="):
			fixture = arg.trim_prefix("--fixture=")
	var data: Variant = JSON.parse_string(FileAccess.get_file_as_string(fixture))
	if typeof(data) != TYPE_DICTIONARY:
		print("seat_check: cannot read %s" % fixture)
		quit(2)
		return
	var pack := ProjectSettings.globalize_path("res://").path_join(SliceMain.PACK_DIR).simplify_path()
	var seating := SliceSeating.new(pack, String(data["world"]))
	for frame in data["frames"]:
		var seated := seating.observe(MineWorldObservation.new(frame), null, null)
		var ids: Array = seated.keys()
		ids.sort_custom(func(a, b): return MineWorldSitRule._before(a, b))
		for id in ids:
			print("SEATED %d %s %s" % [int(frame["at"]), id, seated[id]])
	print("seat_check: %d frames" % (data["frames"] as Array).size())
	quit(0)
