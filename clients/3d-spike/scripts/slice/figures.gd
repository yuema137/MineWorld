## The world's perceived people, drawn and moved between observations
## (step-15 §4.6, §8.6, D-16c-8).
##
## Each observation gives a person's position; a figure no longer jumps there.
## It walks toward the newest observed position at the speed the person was
## observed to move (capped at the player's jog, `Player.JOG_SPEED`, never
## faster), with the gait the player's own body uses (`NPC.step`), facing its
## motion, and turns to the observed facing when it arrives. A figure first
## seen, or more than PLACE_GAP behind its observed position (a crossing into
## view, a long correction), is PLACED there instead. A person who leaves view
## walks the last metres into the bound door nearest where they were last seen,
## if one lies within DOOR_WALK, and is then removed; otherwise they are removed
## at once.
##
## Presentation only. Nothing here decides where anyone may be: a figure moves
## only toward a position the world stated, or into a door the world's
## doorways are bound to (`SliceLayout`), and nothing is pathfound -- a navmesh
## would be the client deciding where people can walk (§8.6). The targeting
## pick collider is each figure's child, so it moves with what is drawn.
class_name SliceFigures
extends Node

## A figure further than this from its observed position is placed, not walked.
const PLACE_GAP := 4.0
## How far a person who leaves view may be from a bound door and still be
## walked into it: longer than any one crossing stride.
const DOOR_WALK := 2.5
## A figure behind its observed position catches up within about this long, if
## the person's own observed speed would take longer (never faster than a jog).
const CATCH_UP := 0.5
## Closer than this to its goal, a figure stands on it.
const ON_GOAL := 0.01

var world_root: Node3D
var layout: SliceLayout
## EntityId string -> the NPC drawn for every person perceived now. The link
## shares this dictionary as `SliceLink.figures`; it is mutated, never replaced.
var nodes := {}
## Every placement, in order: { id, gap (metres; 0 on first sight), first, at }
var placements: Array = []
## Every removal of a person who left view, in order:
## { id, at (where the figure was removed), door (label or ""), walked }
var leaves: Array = []

## id -> { at, yaw, speed, changed } -- the newest observed position and facing,
## the observed speed, and when the observed position last changed (seconds)
var _goal := {}
## id -> { node, to, door, speed } -- a person out of view, walking into a door
var _going := {}


## One observation of the observer's place. `seen`: id -> { at: scene point,
## yaw: the observed facing as a node yaw, label: text }. Returns the ids that
## came into view and the ids that left it.
func observe(seen: Dictionary) -> Dictionary:
	var now := Time.get_ticks_msec() / 1000.0
	var arrived := []
	for id in seen:
		var s: Dictionary = seen[id]
		var at: Vector3 = s["at"]
		var n: NPC = nodes.get(id)
		if n == null:
			n = _build(id, s["label"])
			nodes[id] = n
			n.global_position = at
			n.rotation.y = s["yaw"]
			_goal[id] = {"at": at, "yaw": s["yaw"], "speed": 0.0, "changed": now}
			placements.append({"id": id, "gap": 0.0, "first": true, "at": at})
			arrived.append(id)
			continue
		var label := n.get_node_or_null("Label") as Label3D
		if label != null:
			label.text = s["label"]
		var gap := _flat(at - n.global_position).length()
		if gap > PLACE_GAP:
			n.global_position = at
			placements.append({"id": id, "gap": gap, "first": false, "at": at})
		var g: Dictionary = _goal[id]
		var moved := _flat(at - (g["at"] as Vector3)).length()
		var speed: float = g["speed"]
		var changed: float = g["changed"]
		if moved > 0.0005:
			speed = minf(moved / maxf(now - changed, 0.001), Player.JOG_SPEED)
			changed = now
		_goal[id] = {"at": at, "yaw": s["yaw"], "speed": speed, "changed": changed}
	var left := []
	for id in nodes.keys():
		if not seen.has(id):
			left.append(id)
			_leave(id)
	return {"arrived": arrived, "left": left}


## Every figure goes at once: the observer is now somewhere else.
func clear() -> void:
	for id in nodes:
		(nodes[id] as Node).queue_free()
	for id in _going:
		(_going[id]["node"] as Node).queue_free()
	nodes.clear()
	_going.clear()
	_goal.clear()


## Whether a person who left view is still being drawn walking into a door.
func going(id: String) -> NPC:
	return _going[id]["node"] if _going.has(id) else null


func _process(delta: float) -> void:
	if delta <= 0.0:
		return
	for id in nodes:
		var g: Dictionary = _goal[id]
		_walk(nodes[id], g["at"], g["speed"], delta, g["yaw"])
	for id in _going.keys():
		var w: Dictionary = _going[id]
		var n: NPC = w["node"]
		if _walk(n, w["to"], w["speed"], delta, null):
			leaves.append({"id": id, "at": n.global_position, "door": w["door"], "walked": true})
			n.queue_free()
			_going.erase(id)


## One frame of walking toward `to`; true once the figure stands on it.
## `rest_yaw` (or null) is the facing it turns to when it stands there.
func _walk(n: NPC, to: Vector3, speed: float, delta: float, rest_yaw: Variant) -> bool:
	var d := _flat(to - n.global_position)
	var dist := d.length()
	if dist <= ON_GOAL:
		n.global_position = to
		if rest_yaw != null:
			n.rotation.y = lerp_angle(n.rotation.y, float(rest_yaw), minf(1.0, delta * 8.0))
		n.step(delta, 0.0)
		return true
	var v := minf(Player.JOG_SPEED, maxf(speed, dist / CATCH_UP))
	var m := minf(v * delta, dist)
	var p := n.global_position + d / dist * m
	n.global_position = Vector3(p.x, to.y, p.z)
	n.rotation.y = atan2(d.x, d.z)
	n.step(delta, m / delta)
	return m >= dist


## A person no longer perceived: into the nearest bound door if one is close,
## else gone now.
func _leave(id: String) -> void:
	var n: NPC = nodes[id]
	nodes.erase(id)
	var last: Vector3 = (_goal[id]["at"] as Vector3) if _goal.has(id) else n.global_position
	var speed: float = maxf(float(_goal[id]["speed"]) if _goal.has(id) else 0.0, Player.WALK_SPEED)
	_goal.erase(id)
	var best := Vector3.INF
	var label := ""
	var bound: Dictionary = layout.doors if layout != null else {}
	for place in bound:
		var door: Array = SliceTerrace.doors[layout.doors[place]]
		var points := [SliceLayout.out_point(door)]
		if door.size() > 3 and door[3] != "":
			points.append(SliceLayout.in_point(door))
		for q in points:
			if best == Vector3.INF or _flat(q - last).length() < _flat(best - last).length():
				best = q
				label = door[2]
	if best != Vector3.INF and _flat(best - last).length() <= DOOR_WALK:
		_going[id] = {"node": n, "to": best, "door": label, "speed": speed}
		return
	leaves.append({"id": id, "at": n.global_position, "door": "", "walked": false})
	n.queue_free()


## A perceived person, drawn from the same stand-in mannequin as the street's
## figures; its look is chosen from its id, so the same person looks the same.
func _build(id: String, label_text: String) -> NPC:
	var rng := RandomNumberGenerator.new()
	rng.seed = hash(id)
	var n := NPC.make(rng, NPC.Pose.STAND, 1.72, false)
	n.name = "Person_" + id
	n.set_meta("entity_id", id)       # a string, never a number (ADOPTION.md sec.3.1)
	# what the targeting ray meets; nothing collides with it (step-15 §19)
	n.add_child(SliceTargeting.person_collider())
	world_root.add_child(n)
	var label := Label3D.new()
	label.name = "Label"
	# A name or role the world disclosed: world content, never translated (SD-SET-a-9).
	MineWorldText.mark_world_text(label)
	label.text = label_text
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.position = Vector3(0, 2.05, 0)
	label.font_size = 48
	label.pixel_size = 0.004
	n.add_child(label)
	return n


static func _flat(v: Vector3) -> Vector3:
	return Vector3(v.x, 0.0, v.z)
