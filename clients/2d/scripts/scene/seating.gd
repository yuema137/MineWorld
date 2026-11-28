extends RefCounted

## Who this client draws seated, from each observation (S33-f; `ARC-145`): the shared decision,
## `MineWorldSitRule` (`clients/shared/sitting/`), and nothing of its own. S34-e draws the seated people
## at the pelvis points the shared pose function gives (`poses`), so both clients agree on where (AC-f-4).
##
## The world's sit-point depiction file is the 3D Presentation Pack's (`sit_points/<world>.yaml`): only
## its optional fallback positions matter here, for the shared rule's fallback branch in a world that
## does not disclose its sit points. No shipped depiction file carries one since S34-c2 (`ARC-172`): the
## shipped worlds with sit points disclose them, so this client follows the posture branch there.

## The pack whose `sit_points/` the rule reads, relative to the repository.
const SIT_PACK := "presentation/mineworld-default/3D"

var rule: MineWorldSitRule
## id -> the point key the person is drawn seated at, from the last observation.
var seated := {}


func _init(repo_root: String, world_key: String) -> void:
	rule = MineWorldSitRule.load_for(repo_root.path_join(SIT_PACK), world_key)


## The decision for one observation.
func reconcile(observation) -> Dictionary:
	seated = rule.decide_observed(observation)
	return seated


## A click this close to a sit point's drawn pelvis picks that point (drawing only, like PICK_HALF_WIDTH_PX).
const SIT_PICK_PX := 12.0


## The disclosed sit point of the observer's place whose drawn pelvis lies under a click at `at` (the world
## node's frame), the nearest within SIT_PICK_PX, or "" (S34-e §5.5). Reads the place's disclosure through the
## shared module, and places each point with the shared pose function; nothing is decided here.
func pick(observation, town, projection, at: Vector2) -> String:
	var disclosed: Variant = MineWorldSitRule.disclosed_of(observation)
	if typeof(disclosed) != TYPE_ARRAY:
		return ""
	var here: String = observation.place()
	var tags := MineWorldSitRule.tags_of(observation)
	var picked := ""
	var best := SIT_PICK_PX
	for p in disclosed:
		if typeof(p) != TYPE_DICTIONARY:
			continue
		var key := String(p.get("key", ""))
		var pose := rule.pose_of(tags, disclosed, key)
		if pose.is_empty():
			continue
		var pelvis: Vector2 = pose["pelvis"]
		var plan: Variant = town.to_plan(here, {"x": pelvis.x, "y": pelvis.y})
		if plan == null:
			continue
		var d: float = projection.to_screen(plan).distance_to(at)
		if d <= best:
			best = d
			picked = key
	return picked


## Where each person the last decision seats is drawn (S34-e §5.3): id -> `{point, plan, direction,
## seat_height_m, pelvis}`, the pelvis placed by the shared pose function (`MineWorldSitRule.pose_of`, I-e-2) and
## converted into the town's plan frame; `direction` is the point's facing as a plan direction. A seat whose
## point has no position is not drawn. Drawing only: nothing here decides who sits.
func poses(observation, town) -> Dictionary:
	var out := {}
	var here: String = observation.place()
	var tags := MineWorldSitRule.tags_of(observation)
	var disclosed: Variant = MineWorldSitRule.disclosed_of(observation)
	for id in seated:
		var pose := rule.pose_of(tags, disclosed, String(seated[id]))
		if pose.is_empty():
			continue
		var pelvis: Vector2 = pose["pelvis"]
		var at: Vector2 = pose["at"]
		var yaw := deg_to_rad(float(pose["facing"]) / 1000.0)
		var plan: Variant = town.to_plan(here, {"x": pelvis.x, "y": pelvis.y})
		var from: Variant = town.to_plan(here, {"x": at.x, "y": at.y})
		var ahead: Variant = town.to_plan(here, {"x": at.x + 1000.0 * sin(yaw), "y": at.y + 1000.0 * cos(yaw)})
		if plan == null or from == null or ahead == null:
			continue
		out[id] = {"point": String(seated[id]), "plan": plan, "direction": (ahead as Vector2) - (from as Vector2),
			"seat_height_m": float(pose["seat_height_m"]), "pelvis": pelvis}
	return out
