extends SceneTree

## The 3D client's sit decision over a recorded set of observations, headless (S33-f AC-f-4).
##
##   godot --headless --path clients/3d-spike --script res://scripts/checks/seat_check.gd
##
## Feeds each frame of the fixture to this client's own wiring (`SliceSeating`, as `SliceLink` calls it)
## and prints `SEATED <at> <id> <point>` per person drawn seated, in id order, then
## `seat_check: <n> frames`. It judges nothing itself: `tools/cli/tests/client_sit_rule.rs` compares
## these lines with the 2D client's and with the fixture's expectation. Each frame first prints a `DRAWN3`
## line per figure, read from the nodes the seating actually placed (S34-e AC-e-1, AC-e-9).
##
##   --fixture=<path>   the observations (default tests/fixtures/observations/sitting/contested-long-table.json)

const DEFAULT_FIXTURE := "tests/fixtures/observations/sitting/contested-long-table.json"


func _init() -> void:
	_run.call_deferred()


func _run() -> void:
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
	var seating := SliceSeating.new(SlicePack.select().dir, String(data["world"]))
	if data.has("depiction_pack"):
		seating.rule = MineWorldSitRule.load_for(data.depiction_pack, String(data.world))
	# Real figure nodes, so the lines report where this client actually draws each person.
	var figures := SliceFigures.new()
	root.add_child(figures)
	var layout := SliceLayout.new()
	for frame in data["frames"]:
		var obs := MineWorldObservation.new(frame)
		for person in MineWorldSitRule.people_of(obs):
			if not figures.nodes.has(person.id):
				var n := NPC.new()
				root.add_child(n)
				n.set_process(false)
				figures.nodes[person.id] = n
		var seated := seating.observe(obs, figures, layout)
		for id in figures.nodes:
			if figures._seated.has(id):
				var n: NPC = figures.nodes[id]
				print("DRAWN3 %d %s seated %s %d %d %d" % [int(frame.at), id, seated[id],
					roundi(n.global_position.x * 1000), roundi(-n.global_position.z * 1000),
					roundi(float(figures._seated[id].height) * 1000000)])
			else:
				print("DRAWN3 %d %s standing" % [int(frame.at), id])
		var ids: Array = seated.keys()
		ids.sort_custom(func(a, b): return MineWorldSitRule._before(a, b))
		for id in ids:
			print("SEATED %d %s %s" % [int(frame["at"]), id, seated[id]])
	print("seat_check: %d frames" % (data["frames"] as Array).size())
	for n in figures.nodes.values():
		n.free()
	figures.free()
	quit(0)
