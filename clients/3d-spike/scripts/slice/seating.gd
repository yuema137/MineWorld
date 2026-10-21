## Who this client draws seated, and how (S33-f; `ARC-145`): the shared decision, `MineWorldSitRule`
## (`clients/shared/sitting/`), drawn with the world's sit-point depictions
## (`<pack>/sit_points/<world>.yaml`, step-34 sec.8.2).
##
## A person the rule seats at a point is drawn with the pelvis at the point's `at` moved by the
## depiction's `seat_offset_m` against its facing (positive: back onto a bench; negative: forward onto
## a chair, Q-S34c-2), facing the way the point faces, the body's own seat on the depiction's
## `seat_height_m` (`Human.sit_on`). The point's `at` and facing are the world's where it discloses its
## sit points, else the depiction's fallback. Nothing here decides who sits; it names no world, place or
## point.
class_name SliceSeating
extends RefCounted

var rule: MineWorldSitRule
## id -> the point key the person is drawn seated at, from the last observation.
var seated := {}


func _init(pack_dir: String, world_key: String) -> void:
	rule = MineWorldSitRule.load_for(pack_dir, world_key)


## The decision for one observation, drawn on `figures` (null: decided, not drawn -- the checks).
func observe(obs, figures: SliceFigures, layout: SliceLayout) -> Dictionary:
	seated = rule.decide_observed(obs)
	if figures == null or layout == null:
		return seated
	var here: String = obs.place()
	var tags := MineWorldSitRule.tags_of(obs)
	for id in figures.nodes.keys():
		if not seated.has(id):
			figures.unseat(id)
			continue
		var pose := pose_of(tags, obs.component_value(here, "sit-points"), String(seated[id]))
		if pose.is_empty():
			figures.unseat(id)
			continue
		var at: Vector3 = layout.to_scene(here, {"x": pose["pelvis"].x, "y": pose["pelvis"].y})
		# a facing as a figure's yaw, as `SliceLink` turns a disclosed facing
		var yaw := MineWorldSpace.yaw_to_3d_radians({"yaw": pose["facing"]}) + PI
		figures.seat(id, at, yaw, float(pose["seat_height_m"]))
	return seated


## Where a person seated at `point` is drawn: { pelvis: Vector2 (mm, the place's frame), facing (mdeg),
## seat_height_m, prop }, or {} when the point has no depiction or no position.
func pose_of(tags: PackedStringArray, disclosed: Variant, point: String) -> Dictionary:
	var dep := rule.depiction(tags, point)
	if dep.is_empty():
		return {}
	var at: Variant = dep["at"]
	var facing := int(dep["facing"])
	if typeof(disclosed) == TYPE_ARRAY:
		for p in disclosed:
			if typeof(p) == TYPE_DICTIONARY and String(p.get("key", "")) == point:
				at = Vector2(float(p["at"]["x"]), float(p["at"]["y"]))
				facing = int(p.get("facing", 0))
	if at == null:
		return {}
	return {"pelvis": pelvis(at, facing, float(dep["seat_offset_m"])), "facing": facing,
		"seat_height_m": float(dep["seat_height_m"]), "prop": String(dep["prop"])}


## `at` (mm) moved by `offset_m` against `facing_md` (millidegrees from north toward east).
static func pelvis(at: Vector2, facing_md: int, offset_m: float) -> Vector2:
	var yaw := deg_to_rad(float(facing_md) / 1000.0)
	return at - Vector2(sin(yaw), cos(yaw)) * offset_m * 1000.0
