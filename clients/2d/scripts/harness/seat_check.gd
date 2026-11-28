extends SceneTree

## The 2D client's sit decision over a recorded set of observations, headless (S33-f AC-f-4).
##
##   godot --headless --path clients/2d --script res://scripts/harness/seat_check.gd
##
## Feeds each frame of the fixture to this client's own wiring (`scripts/scene/seating.gd`) and prints
## `SEATED <at> <id> <point>` per person drawn seated, in id order, then `seat_check: <n> frames`. It
## judges nothing itself: `tools/cli/tests/client_sit_rule.rs` compares these lines with the 3D client's
## and with the fixture's expectation.
##
##   --fixture=<path>   the observations (default tests/fixtures/observations/sitting/contested-long-table.json)

const Seating := preload("res://scripts/scene/seating.gd")
const People := preload("res://scripts/scene/people.gd")
const Town := preload("res://scripts/town.gd")
const PlanProjection := preload("res://scripts/projection.gd")
const Presentation := preload("res://scripts/presentation.gd")
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
	var seating = Seating.new(repo, String(data["world"]))
	if data.has("depiction_pack"):
		seating.rule = MineWorldSitRule.load_for(data.depiction_pack, String(data.world))
	var town := Town.new()
	var world := Node2D.new()
	root.add_child(world)
	var people := People.new()
	root.add_child(people)
	people.setup(PlanProjection.new(), Presentation.new(), town, world)
	for frame in data["frames"]:
		var observation := MineWorldObservation.new(frame)
		town.learn(observation)
		people.reconcile(observation)
		var seated: Dictionary = seating.reconcile(observation)
		var poses: Dictionary = seating.poses(observation, town)
		for id in people.drawn_ids():
			people.set_seated(id, poses.get(id))
		people._process(0.0)
		for id in people.drawn_ids():
			var pose: Variant = people.seated_of(id)
			if pose == null:
				print("DRAWN %d %s standing" % [int(frame["at"]), id])
			else:
				var local: Dictionary = town.local_in(observation.place(), people.drawn_plan(id))
				print("DRAWN %d %s seated %s %d %d %d" % [int(frame["at"]), id,
					pose["point"], pose["view"], local["x"], local["y"]])
		var ids: Array = seated.keys()
		ids.sort_custom(func(a, b): return MineWorldSitRule._before(a, b))
		for id in ids:
			print("SEATED %d %s %s" % [int(frame["at"]), id, seated[id]])
	print("seat_check: %d frames" % (data["frames"] as Array).size())
	people.free()
	world.free()
	quit(0)
