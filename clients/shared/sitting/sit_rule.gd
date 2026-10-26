## Who is drawn seated, and where: the one decision both reference clients make about sitting
## (`DECISIONS.md` ARC-145, rewritten to S34's shape; step-34 sec.8.1 SD-S34-8; step-33 sec.4.5 SD-S33-7).
##
## It decides a POSE, never a fact: the server says where a person is and, where the world has the
## `posture` System Pack, whether they sit. Two branches, in order:
##
##   posture   the place discloses `sit-points`: a person is drawn seated iff their disclosed
##             `posture-state` names a point (`{ seated: { place, point, since } }`), at that point
##   fallback  otherwise, at a sit point of this world's sit-point depiction file whose fallback `at`
##             lies within SIT_NEAR_M of the person's disclosed position, once they have not moved for
##             SIT_STILL_S of world time, and no person with a lower entity id is drawn at that point
##             already (ties by entity id); each person at their nearest such point
##
## The depiction file (`presentation/mineworld-default/3D/sit_points/<world>.yaml`) binds a point by its
## place's tags and its key, and gives the drawn seat (`seat_height_m`, `seat_offset_m`, `prop`) and,
## until S34-c writes the points into world data, the fallback `at` / `facing` (Q-S33f-1 (A)).
##
## Both clients call [method decide] with what they were told: it reads no clock, no scene and no
## renderer state, so the 2D and the 3D client draw the same people seated at the same points. Neither
## client holds a copy of these constants (`client_rules.rs`, I-f-6). Nothing here names a world, a
## place, a point or a tag.
class_name MineWorldSitRule
extends RefCounted

## The fallback branch's two constants (step-33 SD-S33-7): metres from a point's `at`, and seconds of
## world time standing still.
const SIT_NEAR_M := 0.40
const SIT_STILL_S := 2.0
## The pack's folder of sit-point depiction files.
const DIR := "sit_points"
## How far a disclosed position may move between two observations and still count as standing still,
## millimetres (a rounding of the server's integer position, not a rule).
const STILL_MM := 1

## Every point of the file: { tags: PackedStringArray, point, group, prop, seat_height_m,
## seat_offset_m, at: Vector2 (mm) or null, facing: int (millidegrees) }.
var points: Array = []
var path := ""
var problem := ""
## id -> { x, y, since: world seconds the person was last seen to move }
var _still := {}


## Reads `<pack_dir>/sit_points/<world_key>.yaml`. A world with no file has no fallback points; a faulty
## file is named in [member problem] and gives none.
static func load_for(pack_dir: String, world_key: String) -> MineWorldSitRule:
	var rule := MineWorldSitRule.new()
	if world_key == "":
		return rule
	rule.path = pack_dir.path_join(DIR).path_join("%s.yaml" % world_key)
	if not FileAccess.file_exists(rule.path):
		return rule
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(rule.path))
	if typeof(parsed) != TYPE_DICTIONARY or int(parsed.get("format", 0)) != 1 \
			or typeof(parsed.get("points")) != TYPE_ARRAY:
		rule.problem = "%s is not a format-1 sit-point depiction file" % rule.path
		push_warning("sitting: " + rule.problem)
		return rule
	if String(parsed.get("scope", {}).get("world", "")) != world_key:
		rule.problem = "%s: scope.world is not %s" % [rule.path, world_key]
		push_warning("sitting: " + rule.problem)
		return rule
	for p in parsed["points"]:
		var at: Variant = null
		var facing := 0
		var fb: Variant = p.get("fallback")
		if typeof(fb) == TYPE_DICTIONARY:
			at = Vector2(float(fb["at"]["x"]), float(fb["at"]["y"]))
			facing = int(fb.get("facing", 0))
		rule.points.append({"tags": PackedStringArray(p.get("tags", [])), "point": String(p["point"]),
			"group": String(p.get("group", "")), "prop": String(p.get("prop", "")),
			"seat_height_m": float(p.get("seat_height_m", 0.45)),
			"seat_offset_m": float(p.get("seat_offset_m", 0.45)), "at": at, "facing": facing})
	return rule


## The depiction of `point` in a place carrying `place_tags`, or {}.
func depiction(place_tags: PackedStringArray, point: String) -> Dictionary:
	for p in points:
		if p["point"] == point and _tagged(p["tags"], place_tags):
			return p
	return {}


## The decision for one observation of one place.
##
##   place_tags   the place's disclosed tags
##   disclosed    the place's disclosed `sit-points` value, or null when it discloses none
##   people       [{ id: String, x: float, y: float (mm, the place's frame), posture: Variant (the
##                person's disclosed `posture-state`, or null) }]
##   now_s        the world's time of the observation, seconds
##
## Returns id -> the point key each person seated is drawn at. People not listed are forgotten.
func decide(place_tags: PackedStringArray, disclosed: Variant, people: Array, now_s: float) -> Dictionary:
	var seated := {}
	var seen := {}
	for person in people:
		var id := String(person["id"])
		seen[id] = true
		var was: Variant = _still.get(id)
		if was == null or absf(float(was["x"]) - float(person["x"])) > STILL_MM \
				or absf(float(was["y"]) - float(person["y"])) > STILL_MM:
			_still[id] = {"x": float(person["x"]), "y": float(person["y"]), "since": now_s}
	for id in _still.keys():
		if not seen.has(id):
			_still.erase(id)
	if disclosed != null:
		for person in people:
			var state: Variant = person.get("posture")
			if typeof(state) == TYPE_DICTIONARY and typeof(state.get("seated")) == TYPE_DICTIONARY:
				seated[String(person["id"])] = String(state["seated"].get("point", ""))
		return seated
	var candidates := []
	for p in points:
		if p["at"] != null and _tagged(p["tags"], place_tags):
			candidates.append(p)
	var order := people.duplicate()
	order.sort_custom(func(a, b): return _before(String(a["id"]), String(b["id"])))
	var taken := {}
	for person in order:
		var id := String(person["id"])
		if now_s - float(_still[id]["since"]) < SIT_STILL_S:
			continue
		var here := Vector2(float(person["x"]), float(person["y"]))
		var best := ""
		var best_d := INF
		for p in candidates:
			var d := here.distance_to(p["at"]) / 1000.0
			if d <= SIT_NEAR_M and d < best_d and not taken.has(p["point"]):
				best = p["point"]
				best_d = d
		if best != "":
			taken[best] = true
			seated[id] = best
	return seated


## The people of `observation` in the observer's place, in [method decide]'s form, read from the
## observation alone (`MineWorldObservation`): every perceived person with a position in that place.
static func people_of(observation) -> Array:
	var here: String = observation.place()
	var out := []
	for entity in observation.entities():
		if String(entity.get("entity_type", "")) != "person":
			continue
		var location: Variant = entity.get("location")
		if typeof(location) != TYPE_DICTIONARY or typeof(location.get("local")) != TYPE_DICTIONARY:
			continue
		var place: Variant = location.get("place")
		if typeof(place) == TYPE_DICTIONARY and String(place.get("entity", "")) != here:
			continue
		var id := String(entity.get("id", ""))
		out.append({"id": id, "x": float(location["local"].get("x", 0)),
			"y": float(location["local"].get("y", 0)),
			"posture": observation.component_value(id, "posture-state")})
	return out


## The observer's place's disclosed tags.
static func tags_of(observation) -> PackedStringArray:
	var tags: Variant = observation.entity(observation.place()).get("tags", [])
	return PackedStringArray(tags) if typeof(tags) == TYPE_ARRAY else PackedStringArray()


## The whole decision for one observation: [method decide] on what it says about its place.
func decide_observed(observation) -> Dictionary:
	var here: String = observation.place()
	if here == "":
		_still.clear()
		return {}
	return decide(tags_of(observation), observation.component_value(here, "sit-points"),
		people_of(observation), float(observation.at()))


## Every tag of `wanted` is among `have`.
static func _tagged(wanted: PackedStringArray, have: PackedStringArray) -> bool:
	if wanted.is_empty():
		return false
	for t in wanted:
		if not have.has(t):
			return false
	return true


## Entity-id order: shorter decimal ids first, then by text, so "9" comes before "10".
static func _before(a: String, b: String) -> bool:
	if a.length() != b.length():
		return a.length() < b.length()
	return a < b
