extends SceneTree
func encode(p: Dictionary) -> Dictionary:
	return {"pelvis": [p.pelvis.x,p.pelvis.y], "at": [p.at.x,p.at.y], "facing": p.facing, "seat_height_m":p.seat_height_m,"prop":p.prop,"depicted":p.depicted}
func _init() -> void:
	var rule := MineWorldSitRule.load_for("../../presentation/mineworld-default/3D", "lakeside")
	assert(rule.points.size()==29)
	var rows := []
	for p in rule.points:
		var disclosed := [{"key":p.point,"at":{"x":p.at.x+123,"y":p.at.y-47},"facing":int(p.facing+19333)%360000}]
		rows.append({"point":p.point,"tags":Array(p.tags),"disclosed":disclosed,"fallback_expected":encode(rule.pose_of(p.tags,null,p.point)),"disclosed_expected":encode(rule.pose_of(p.tags,disclosed,p.point))})
	var f:=FileAccess.open("/tmp/mineworld-s34e-seat-measure/legacy-baseline.json",FileAccess.WRITE)
	f.store_string(JSON.stringify({"head":"4ce72437c8206826a5d5b5eba9fc3471024032c6","rows":rows},"\t")+"\n")
	print("LEGACY_FROZEN 29 rows, fallback+disclosed")
	quit()
