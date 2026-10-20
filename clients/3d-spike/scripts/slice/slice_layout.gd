## Where each of the world's places sits in this scene -- learned from the
## server, never listed here (step-15 §20, D-16c-2, D-16c-3).
##
## The world frame is fixed (+x east, +y north, `CORE_CONCEPTS.md` §6.1), so a
## binding only translates. The scene asserts one thing about a world, once:
## the café room depicts the place tagged `SliceWorld.ANCHOR_TAG`. Everything
## else follows from what the server discloses:
##
##   the anchor   its one passage puts its `here` on the café door's in-point
##                and its `there` -- the place it leads to, whatever its tags --
##                on the door's out-point. That place is drawn as the street.
##   the street   every passage it discloses binds the place it leads to by
##                DOORWAY COINCIDENCE: the drawn door whose out-point lies
##                within DOOR_MATCH of the doorway's scene point. An enterable
##                door binds its room's volume to that place (the florist
##                becomes the world's store, SC-3); a closed door binds the
##                place as a door, with nothing drawn behind it.
##
## A doorway that lands on no drawn door binds nothing and is recorded as a
## miss (`misses`): the probe names it, the HUD says the place is not drawn.
## Nothing is fitted, widened or guessed (R-16c-1).
##
## The drawn doors are the scene's own registry (`SliceTerrace.doors`), filled
## by the code that builds each door: [sill centre (global), façade yaw, label,
## the room's place volume ("" for a closed door), the depth from the sill to
## the room's inner face, the room floor's height above the sill] (D-16c-4).
##
## Presentation only: this decides where things are DRAWN. Which place the body
## is in is still the server's answer; the client reports the body in the place
## whose volume it stands in, and the movement system decides the crossing.
class_name SliceLayout
extends RefCounted

## How each bound place is drawn (the SHOWN report's `drawn_as`, D-16c-7).
const ROOM := "room"
const DOOR := "door"
const STREET := "street"

## The authoring rule 12d adopted for every doorway (step-11 SD-D3): a doorway
## point stands 0.40 m in front of its door, on each side. A door's out-point
## is its sill plus this along the façade's outward normal; its in-point is the
## sill minus (the depth to the room's inner face + this). The client's art
## convention matching the world's, checked end to end by `--layout` (L-1).
const DOOR_INSET := 0.40
## How far, horizontally, a disclosed doorway's scene point may lie from a drawn
## door's out-point and still be that door (D-16c-5): under the narrowest drawn
## door's clearance either side of a body, and the tolerance the project uses
## for scene-against-world geometry. Never widened after a measurement.
const DOOR_MATCH := 0.15

## place id -> its passages as the world disclosed them where the observer
## stood: [{ "to": id, "here": {x, y}, "there": {x, y} }], exactly as received
var passages := {}
## place id -> its tags as the world listed them where the observer stood
var tags := {}
## place id -> where its frame's (0, 0) lands in this scene, at its floor
var origins := {}
## place id -> ROOM | DOOR | STREET, for every bound place
var drawn_as := {}
## place id -> the scene volume that depicts it ("" for a door)
var volumes := {}
## place id -> the index in `SliceTerrace.doors` of the door bound to it
var doors := {}
## place id -> why a doorway to it bound nothing:
## { "at": scene point, "nearest": label, "distance": metres, "why": text }
var misses := {}
## The place the café room depicts, once the observer has stood in it.
var anchor := ""
## The place the café's door opens onto, drawn as the street.
var street := ""


## Learn from one observation of the observer's own place: its tags and
## passages, then whatever binding they make possible.
func learn(obs: MineWorldObservation) -> void:
	var place := obs.place()
	if place == "":
		return
	var t: Variant = obs.entity(place).get("tags", [])
	tags[place] = (t as Array).duplicate() if typeof(t) == TYPE_ARRAY else []
	passages[place] = _passages_of(obs, place)
	if anchor == "" and tags[place].has(SliceWorld.ANCHOR_TAG):
		_bind_anchor(place)
	if place == street and street != "":
		_bind_street_doorways()


## Whether people in `place` can be drawn: its frame is bound to the scene.
func is_drawn(place: String) -> bool:
	return origins.has(place)


## The place a scene volume depicts, or "" if it depicts none yet.
func place_of_volume(volume: String) -> String:
	if volume == "":
		return ""
	for p in volumes:
		if volumes[p] == volume:
			return p
	return ""


## World millimetres in a bound place's frame -> a point in this scene.
func to_scene(place: String, local: Variant) -> Vector3:
	var p := MineWorldSpace.to_3d(local)
	return (origins.get(place, Vector3.ZERO) as Vector3) + Vector3(p.x, 0.0, p.z)


## A point in this scene -> world millimetres in a bound place's frame. Height
## is never reported (`SliceLink.REPORT_HEIGHT`).
func to_world(place: String, p: Vector3) -> Dictionary:
	var rel := p - (origins.get(place, Vector3.ZERO) as Vector3)
	rel.y = 0.0
	return MineWorldSpace.from_3d(rel)


## A drawn door's out-point: on the street side, DOOR_INSET in front of its sill.
static func out_point(door: Array) -> Vector3:
	return (door[0] as Vector3) + _outward(door) * DOOR_INSET


## A drawn door's in-point: inside its room, DOOR_INSET past the inner face, at
## the room's floor.
static func in_point(door: Array) -> Vector3:
	var depth: float = door[4] if door.size() > 4 else 0.0
	var floor_h: float = door[5] if door.size() > 5 else 0.0
	return (door[0] as Vector3) - _outward(door) * (depth + DOOR_INSET) + Vector3(0.0, floor_h, 0.0)


## The façade's outward normal: its local +z (toward the street), turned by its yaw.
static func _outward(door: Array) -> Vector3:
	var yaw: float = door[1]
	return Vector3(sin(yaw), 0.0, cos(yaw))


## What the scene shows of the world, in S12 13a's `SHOWN` keys and meanings
## (D-16c-7): every bound place; every passage of every place the observer
## stood in whose two sides are both bound, exactly as disclosed; and `people`,
## already read back from the DRAWN positions by the caller -- [{ id, label,
## at: scene point }] -- each turned into `local` in the observer's place
## frame. What is not drawn is not reported.
func shown(here: String, people: Array) -> Dictionary:
	var places := []
	for p in drawn_as:
		places.append({"place": p, "drawn_as": drawn_as[p], "tags": tags.get(p, [])})
	var doorways := []
	for from in passages:
		if not drawn_as.has(from):
			continue
		for p in passages[from]:
			if not drawn_as.has(p["to"]):
				continue
			doorways.append({"from": from, "to": p["to"],
				"here": [int(p["here"].get("x", 0)), int(p["here"].get("y", 0))],
				"there": [int(p["there"].get("x", 0)), int(p["there"].get("y", 0))]})
	var shown_people := []
	for who in people:
		var local := to_world(here, who["at"]) if is_drawn(here) else {}
		shown_people.append({"id": who["id"], "label": who["label"],
			"local": null if local.is_empty() else local})
	return {"place": here, "places": places, "doorways": doorways, "people": shown_people}


# --- binding ------------------------------------------------------------------------

## The passages disclosed on `place`, the well-formed ones, as received.
static func _passages_of(obs: MineWorldObservation, place: String) -> Array:
	var out := []
	var leads: Variant = obs.component(place, "passages").get("leads_to", [])
	if typeof(leads) != TYPE_ARRAY:
		return out
	for p in leads:
		if typeof(p) != TYPE_DICTIONARY or typeof(p.get("to")) != TYPE_DICTIONARY \
				or typeof(p.get("here")) != TYPE_DICTIONARY or typeof(p.get("there")) != TYPE_DICTIONARY:
			continue
		var to := String(p["to"].get("entity", ""))
		if to != "":
			out.append({"to": to, "here": p["here"], "there": p["there"]})
	return out


## The one declared depiction: the café room is the anchor place, and its one
## door binds both it and the place that door leads to.
func _bind_anchor(place: String) -> void:
	var ps: Array = passages[place]
	var door := _door_of_volume(SliceWorld.CAFE_PLACE)
	if ps.size() != 1 or door < 0:
		misses[place] = {"at": Vector3.ZERO, "nearest": "", "distance": INF,
			"why": "the anchor discloses %d passages; the café room has one door" % ps.size()}
		return
	var d: Array = SliceTerrace.doors[door]
	var p: Dictionary = ps[0]
	anchor = place
	_bind(place, ROOM, SliceWorld.CAFE_PLACE, door, in_point(d) - _flat(p["here"]))
	street = p["to"]
	_bind(street, STREET, SliceWorld.STREET_PLACE, door, out_point(d) - _flat(p["there"]))


## Standing in the street: every passage it discloses binds its place to the
## drawn door its doorway lands on, or is recorded as a miss.
func _bind_street_doorways() -> void:
	var used := {}
	for p in drawn_as:
		if p != street:
			used[doors[p]] = p
	for p in passages[street]:
		var to: String = p["to"]
		if drawn_as.has(to):
			continue
		var at := to_scene(street, p["here"])
		var best := -1
		var best_d := INF
		for i in SliceTerrace.doors.size():
			var dd := _horizontal(at, out_point(SliceTerrace.doors[i]))
			if dd < best_d:
				best_d = dd
				best = i
		var label: String = SliceTerrace.doors[best][2] if best >= 0 else ""
		if best < 0 or best_d > DOOR_MATCH:
			misses[to] = {"at": at, "nearest": label, "distance": best_d,
				"why": "no drawn door within %.2f m" % DOOR_MATCH}
			continue
		if used.has(best):
			misses[to] = {"at": at, "nearest": label, "distance": best_d,
				"why": "that door is already bound to place %s" % used[best]}
			continue
		var d: Array = SliceTerrace.doors[best]
		var volume: String = d[3] if d.size() > 3 else ""
		used[best] = to
		if volume != "":
			_bind(to, ROOM, volume, best, in_point(d) - _flat(p["there"]))
		else:
			_bind(to, DOOR, "", best, Vector3.INF)


func _bind(place: String, how: String, volume: String, door: int, origin: Vector3) -> void:
	drawn_as[place] = how
	volumes[place] = volume
	doors[place] = door
	if origin != Vector3.INF:
		origins[place] = origin
	misses.erase(place)


## The first drawn door whose room is `volume`, or -1.
static func _door_of_volume(volume: String) -> int:
	for i in SliceTerrace.doors.size():
		var d: Array = SliceTerrace.doors[i]
		if d.size() > 3 and d[3] == volume:
			return i
	return -1


## A disclosed point as a horizontal scene offset (height dropped).
static func _flat(local: Variant) -> Vector3:
	var p := MineWorldSpace.to_3d(local)
	return Vector3(p.x, 0.0, p.z)


static func _horizontal(a: Vector3, b: Vector3) -> float:
	return Vector2(a.x - b.x, a.z - b.z).length()
