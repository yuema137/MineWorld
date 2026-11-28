extends SceneTree
## S34-e's freeze-legacy.gd, adapted for S34-c2 (ARC-172): Lakeside's depiction file carries no fallback,
## so each row's position is the world's own sit point (out/world-points.json, read from the place files),
## given to pose_of as the place's disclosure. Same encode, same perturbed disclosed case.
func encode(p: Dictionary) -> Dictionary:
	return {"pelvis": [p.pelvis.x,p.pelvis.y], "at": [p.at.x,p.at.y], "facing": p.facing, "seat_height_m":p.seat_height_m,"prop":p.prop,"depicted":p.depicted}
func _init() -> void:
	var rule := MineWorldSitRule.load_for("/Users/yuema137/mineworld-worktrees/impl-s34c2/presentation/mineworld-default/3D", "lakeside")
	assert(rule.points.size()==29)
	var places: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("/Users/yuema137/mineworld-worktrees/_scratch/s34c2/gen/out/world-points.json"))
	var rows := []
	for p in rule.points:
		var world := []
		for key in places:
			if MineWorldSitRule.binds(p, PackedStringArray(places[key].tags)):
				for w in places[key].points:
					if w.key == p.point:
						world = [{"key": w.key, "at": {"x": int(w.at.x), "y": int(w.at.y)}, "facing": int(w.facing)}]
		assert(world.size() == 1)
		assert(rule.pose_of(p.tags, null, p.point).is_empty())
		var w0: Dictionary = world[0]
		var disclosed := [{"key":p.point,"at":{"x":w0.at.x+123,"y":w0.at.y-47},"facing":int(w0.facing+19333)%360000}]
		rows.append({"point":p.point,"tags":Array(p.tags),"world":world,"world_expected":encode(rule.pose_of(p.tags,world,p.point)),"disclosed":disclosed,"disclosed_expected":encode(rule.pose_of(p.tags,disclosed,p.point))})
	var f:=FileAccess.open("/Users/yuema137/mineworld-worktrees/_scratch/s34c2/gen/out/legacy-baseline-head.json",FileAccess.WRITE)
	f.store_string(JSON.stringify({"head":"S34-c2 (worlds/lakeside sit-points; no fallback, ARC-172)","rows":rows},"\t")+"\n")
	print("LEGACY_FROZEN 29 rows, world+disclosed")
	quit()
