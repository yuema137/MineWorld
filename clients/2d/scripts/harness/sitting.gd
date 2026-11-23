extends RefCounted
## The interaction drive's sitting steps (S34-e C4): `{"click_sit": "<point>" | "last" | "nearest-far"}`,
## `{"report_posture": true}`, `{"floor_near_seat": true}` and `{"record_seating": true}`.
var harness: Node
var last_point := ""

## Prints `SIT_SENT` for every sit the app submits, with the offers of the frame it was sent from.
func _init(owner: Node) -> void:
	harness = owner
	harness.app.link.submitted.connect(func(token: String, request: Dictionary) -> void:
		if request.get("action_type") == harness.app.chain.sit_type:
			print("SIT_SENT ", JSON.stringify({"token": token, "point": request["payload"]["payload"]["point"],
				"offers": harness.app.latest.complete_affordances(harness.app.chain.sit_type, "")})))

## One sitting step; "" when it completed, else why not.
func step(request: Dictionary) -> String:
	var app: Node = harness.app
	if request.has("report_posture"):
		print("POSTURE ", JSON.stringify({"seated": app.people.seated_of(app.latest.observer()),
			"point": last_point}))
		return ""
	if request.has("record_seating"):
		print("SEATING_FRAME ", JSON.stringify(app.latest.frame))
		return ""
	if request.has("floor_near_seat"):
		var pose: Dictionary = app.seating.rule.pose_of(MineWorldSitRule.tags_of(app.latest),
			MineWorldSitRule.disclosed_of(app.latest), last_point)
		if pose.is_empty():
			return "no last seat to leave"
		# 700 mm sideways of the last seat: beyond its picking area, within the server's reach.
		var yaw := deg_to_rad(float(pose["facing"]) / 1000.0)
		var at: Vector2 = pose["at"] + Vector2(cos(yaw), -sin(yaw)) * 700.0
		var failed: String = await harness._click_floor({"x": roundi(at.x), "y": roundi(at.y)})
		if failed != "":
			return failed
		if app.menu.is_open():
			return "near-seat destination hit a figure instead of the floor"
		return "" if await harness.drive._until(func() -> bool: return not app.walker.is_walking(),
			harness.STEP_LIMIT_S) else "near-seat walk did not finish"
	var wanted := String(request["click_sit"])
	var key := last_point if wanted == "last" else wanted
	var offered: Array = app.latest.complete_affordances(app.chain.sit_type, "")
	if wanted == "nearest-far":
		key = ""
		var best := INF
		var here: Vector2 = app.walker.body_plan
		for offer in offered:
			if offer.get("unavailable_reason") != "too_far_away":
				continue
			var point := String(offer["payload"]["point"])
			var pose: Dictionary = app.seating.rule.pose_of(MineWorldSitRule.tags_of(app.latest),
				MineWorldSitRule.disclosed_of(app.latest), point)
			if pose.is_empty():
				continue
			var plan: Vector2 = app.town.to_plan(app.latest.place(), {"x": pose["pelvis"].x, "y": pose["pelvis"].y})
			var distance := here.distance_to(plan)
			if distance < best:
				best = distance
				key = point
	if key == "":
		return "no disclosed point matches " + wanted
	var pose: Dictionary = app.seating.rule.pose_of(MineWorldSitRule.tags_of(app.latest),
		MineWorldSitRule.disclosed_of(app.latest), key)
	if pose.is_empty():
		return "no pose for " + key
	last_point = key
	print("SIT_OFFER ", JSON.stringify({"point": key, "offers": offered}))
	var plan: Vector2 = app.town.to_plan(app.latest.place(), {"x": pose["pelvis"].x, "y": pose["pelvis"].y})
	var center: Vector2 = app.projection.to_screen(plan)
	var clicked := false
	# The seat's exposed area: a point over the seat that no figure or object covers.
	for delta in [Vector2.ZERO, Vector2(0, 10), Vector2(10, 0), Vector2(-10, 0)]:
		var at: Vector2 = center + delta
		if app.people.pick(at) == "" and app.objects.pick(at) == "" \
				and app.seating.pick(app.latest, app.town, app.projection, at) == key:
			harness._push_click(at)
			clicked = true
			break
	if not clicked:
		return "no exposed click area for seat " + key
	await harness.drive._seconds(0.2)
	if not await harness.drive._until(func() -> bool:
		return app.intents.pending.is_empty() and not app.walker.is_walking(), harness.STEP_LIMIT_S):
		return "seat click did not finish"
	await harness.drive._seconds(0.3)
	return ""
