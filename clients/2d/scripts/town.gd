extends RefCounted

## One drawn town, glued from the place frames the world disclosed (`ARC-45`).
##
## Every place has its own frame. A passage disclosed in place `Q` says where its doorway is in `Q`
## (`here`) and in the place it leads to, `P` (`there`), so `P`'s origin, in the frame of the first
## place this client stood in, is `origin(Q) + here − there`: a translation, never a rotation or a
## scale. That is all the layout there is. It is learned from observations, held per world instance
## in memory, and dropped when the instance changes; it holds no world truth, and nothing is ever
## submitted from it except positions the player chose.
##
## Units: positions are kept in integer millimetres on the world's axes (`+y` north), exactly as the
## world states them, and converted to plan metres only by [method to_plan].

## Which running world this layout belongs to.
var instance := ""
## place id string → Vector2i: the place's origin in the root frame, millimetres, `+y` north.
var origins: Dictionary = {}
## place id string → Array of `{to: String, here: Vector2i, there: Vector2i, to_tags: PackedStringArray}`.
var passages: Dictionary = {}
## place id string → PackedStringArray of the tags the world listed for it: when the observer stood
## there, and for each place a disclosed doorway leads to (ARC-82, the door sign).
var tags: Dictionary = {}
## place id string → Rect2 in plan metres: the footprint the presentation drew for it. Decoration
## (`ARC-45` point 5): used only to tell which place a click pointed into, never to refuse a stride.
var footprints: Dictionary = {}
## Bumped whenever something drawable was learned, so the scene knows to rebuild.
var version := 0


## Forgets everything: a different world (or none yet).
func reset(for_instance: String) -> void:
	instance = for_instance
	origins.clear()
	passages.clear()
	tags.clear()
	footprints.clear()
	version += 1


## Learns what one observation shows: the observer's place, its tags and its doorways.
func learn(observation: MineWorldObservation) -> void:
	var place := observation.place()
	if place == "":
		return
	var changed := false
	var listed := observation.entity(place)
	var place_tags := PackedStringArray()
	for tag in listed.get("tags", []):
		place_tags.append(String(tag))
	if not tags.has(place) or tags[place] != place_tags:
		tags[place] = place_tags
		changed = true
	var learned := _passages_of(observation.component(place, "passages"))
	if not passages.has(place) or passages[place] != learned:
		passages[place] = learned
		changed = true
	# A doorway discloses the tags of the place it leads to (ARC-82): the door sign, learned here so the
	# doorway is named and its façade chosen before the player has entered (R-PK-1).
	for p in learned:
		var neighbour: String = p["to"]
		if not tags.has(neighbour) or tags[neighbour] != p["to_tags"]:
			tags[neighbour] = p["to_tags"]
			changed = true
	changed = _place_origins(place) or changed
	if changed:
		version += 1


## The plan-metre point of a position in a place, or null when that place is not placed yet.
func to_plan(place: String, local: Variant) -> Variant:
	if not origins.has(place) or typeof(local) != TYPE_DICTIONARY:
		return null
	var origin: Vector2i = origins[place]
	var mm := {"x": origin.x + int(local.get("x", 0)), "y": origin.y + int(local.get("y", 0))}
	return MineWorldSpace.to_2d(mm)


## A plan-metre point as a position in `place`'s own frame: integer millimetres, `+y` north.
func local_in(place: String, plan: Vector2) -> Dictionary:
	var origin: Vector2i = origins.get(place, Vector2i.ZERO)
	var mm := MineWorldSpace.from_2d(plan)
	return {"x": int(mm["x"]) - origin.x, "y": int(mm["y"]) - origin.y, "z": 0}


## The passage from `from` to `to`, or {}.
func passage(from: String, to: String) -> Dictionary:
	for p in passages.get(from, []):
		if p["to"] == to:
			return p
	return {}


## The hub: the learned place with the most disclosed doorways, and at least two (the street in a
## town). "" while no place opens onto more than one other: a room with one door is not a hub.
func hub() -> String:
	var best := ""
	var most := 1
	for place in passages:
		var count: int = passages[place].size()
		if count > most and origins.has(place):
			best = place
			most = count
	return best


## The plan-metre rectangle the hub's doorways span — the extent dressing is anchored to (`ARC-45`).
func hub_extent() -> Rect2:
	var place := hub()
	var points := PackedVector2Array()
	for p in passages.get(place, []):
		points.append(to_plan(place, _mm_dict(p["here"])))
	if points.is_empty():
		return Rect2()
	var box := Rect2(points[0], Vector2.ZERO)
	for point in points:
		box = box.expand(point)
	return box


## Which place a plan point lies in, for routing a click: a drawn footprint if it lies in one, else
## the hub if known, else `fallback`.
func place_at(plan: Vector2, fallback: String) -> String:
	for place in footprints:
		var box: Rect2 = footprints[place]
		if box.has_point(plan):
			return place
	var h := hub()
	return h if h != "" else fallback


## How many doorways are learned, all places together (AC-W4 reads it).
func passage_count() -> int:
	var n := 0
	for place in passages:
		n += passages[place].size()
	return n


func _passages_of(component: Dictionary) -> Array:
	var out: Array = []
	for entry in component.get("leads_to", []):
		if typeof(entry) != TYPE_DICTIONARY:
			continue
		var to: Variant = entry.get("to")
		var target := String(to.get("entity", "")) if typeof(to) == TYPE_DICTIONARY else String(to)
		var here: Variant = entry.get("here")
		var there: Variant = entry.get("there")
		if target == "" or typeof(here) != TYPE_DICTIONARY or typeof(there) != TYPE_DICTIONARY:
			continue  # A world that models no doorway position cannot be glued; nothing is guessed.
		var to_tags := PackedStringArray()
		for tag in entry.get("to_tags", []):
			to_tags.append(String(tag))
		out.append({"to": target, "here": _mm(here), "there": _mm(there), "to_tags": to_tags})
	return out


## Places the observer's place and its neighbours. Returns whether any origin was added.
func _place_origins(place: String) -> bool:
	var added := false
	if not origins.has(place):
		# Reached without a known way in (the first place, or after the world moved the observer):
		# placed from a known neighbour if one is disclosed, otherwise it becomes a root.
		for p in passages.get(place, []):
			if origins.has(p["to"]):
				origins[place] = origins[p["to"]] + p["there"] - p["here"]
				break
		if not origins.has(place):
			origins[place] = Vector2i.ZERO if origins.is_empty() else _far_root()
		added = true
	for p in passages.get(place, []):
		if not origins.has(p["to"]):
			origins[p["to"]] = origins[place] + p["here"] - p["there"]
			added = true
	return added


## An origin for an unconnected place, well clear of everything placed so far.
func _far_root() -> Vector2i:
	var east := 0
	for place in origins:
		east = maxi(east, origins[place].x)
	return Vector2i(east + 200_000, 0)


static func _mm(local: Dictionary) -> Vector2i:
	return Vector2i(int(local.get("x", 0)), int(local.get("y", 0)))


static func _mm_dict(mm: Vector2i) -> Dictionary:
	return {"x": mm.x, "y": mm.y, "z": 0}
