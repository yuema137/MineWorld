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
## would be the client deciding where people can walk (§8.6). The person
## collider -- what the targeting ray meets and, since S15 PR 12e, what stops
## the player's body -- is each figure's child, so it moves with what is drawn.
class_name SliceFigures
extends Node

## A figure further than this from its observed position is placed, not walked.
const PLACE_GAP := 4.0
## How far a person who leaves view may be from a bound door and still be
## walked into it: longer than any one crossing stride.
const DOOR_WALK := 2.5
## A figure never falls further behind its observed position than about this
## long at the person's observed speed: past it, it walks faster to catch up
## (never faster than a jog). Long enough that a person walking in strides is
## drawn walking steadily, not dashing and waiting (E16c-5).
const CATCH_UP := 2.0
## The observed speed is a stride over the time since the previous one, that
## time taken as at least STEP_GAP_SHORTEST and at most STEP_GAP_LONGEST: a
## first stride after standing is not read as a crawl, nor two observations
## close together as a dash.
const STEP_GAP_SHORTEST := 0.1
const STEP_GAP_LONGEST := 1.0
## Closer than this to its goal, a figure stands on it.
const ON_GOAL := 0.01

var world_root: Node3D
var layout: SliceLayout
## The pack's cast binding: which body draws a person, from the look the world
## disclosed (ARC-92). Null draws every person as a town figure.
var cast: SliceCast = null
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
			n = _build(id, s)
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
			speed = minf(moved / clampf(now - changed, STEP_GAP_SHORTEST, STEP_GAP_LONGEST), Player.JOG_SPEED)
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
## figures. Which body draws it comes from the look the world disclosed, through
## the pack's cast binding (`SliceCast`); its label from the disclosed name.
func _build(id: String, s: Dictionary) -> NPC:
	var rng := RandomNumberGenerator.new()
	rng.seed = hash(id)
	var look: String = s.get("look", "")
	var character := cast.character_for(look, id) if cast != null else ""
	var n := NPC.make_body(rng, NPC.Pose.STAND, 1.72, cast.body_for(character) if cast != null else {})
	n.name = "Person_" + id
	n.set_meta("entity_id", id)       # a string, never a number (ADOPTION.md sec.3.1)
	n.set_meta("look", look)
	n.set_meta("character", character)
	print("[link] figure %s  [name %s, look %s, character %s]" % [
		id, s.get("name", ""), look, character if character != "" else "(town figure)"])
	# what the targeting ray meets and the player's body is stopped by; it
	# rides the walking figure (step-11 §22 SD-E5). `--no-people-colliders`
	# builds none: the adversarial run in which the server alone keeps people
	# apart (X-2).
	if not no_people_colliders():
		n.add_child(SliceTargeting.person_collider())
	world_root.add_child(n)
	var label := overhead_label(s["label"], 2.05, 48)
	label.name = "Label"
	# A name or role the world disclosed: world content, never translated (SD-SET-a-9).
	MineWorldText.mark_world_text(label)
	n.add_child(label)
	return n


## OVERHEAD TEXT (names, the Zz). Godot's own screen-space label, not a world-sized sign: a world-
## sized name grows without bound as its person nears the camera (a person 1 m from a third-person
## camera wore a name a fifth of the frame tall) and two names a few metres apart along the view
## overlap at full size. `fixed_size` draws the text at one size on screen at any distance:
## LABEL_FRAME_FRACTION of the frame height, derived from the camera rig's own field of view, not
## copied (Godot sizes a fixed-size label as it would be 1 m from the camera, where the frame is
## 2 tan(fov / 2) metres tall).
##
## A constant-size name on a far person would be larger than the person it names, and the names of a
## crowd across a room pile into one another. So the text is shown only while its person stands at
## least LABEL_BODY_RATIO times the text's height on screen -- the name stays a caption of a person
## you can see, never a sign floating over a speck. That is a distance, from the person's height
## (PERSON_HEIGHT_M), the ratio and the field of view: h / (2 d tan(fov/2)) >= r f, so
## d <= h / (2 tan(fov/2) r f). Godot's visibility range hides it there and fades it over the last
## LABEL_FADE of that distance (Godot measures from the camera to the label's own AABB centre).
const LABEL_FRAME_FRACTION := 0.022
const LABEL_BODY_RATIO := 4.0
## A typical adult's standing height (the 3D client's mannequins are built to it).
const PERSON_HEIGHT_M := 1.70
const LABEL_FADE := 0.2


## A billboarded overhead text `height_m` above its parent's origin, sized and ranged as above.
static func overhead_label(text: String, height_m: float, font_size: int) -> Label3D:
	var tan_half := tan(deg_to_rad(CameraRig.FOV) * 0.5)
	var l := Label3D.new()
	l.text = text
	l.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	l.fixed_size = true
	l.position = Vector3(0, height_m, 0)
	l.font_size = font_size
	l.pixel_size = LABEL_FRAME_FRACTION * 2.0 * tan_half / float(font_size)
	# a caption, not a sign in the room: a pendant lamp or a shelf between the camera and the head
	# cut the name in half. Drawn over the scene (transparent, so back to front: a nearer person's
	# name lies over a farther one's), and only for a person the world discloses as perceived.
	l.no_depth_test = true
	var shown := PERSON_HEIGHT_M / (2.0 * tan_half * LABEL_BODY_RATIO * LABEL_FRAME_FRACTION)
	l.visibility_range_end = shown
	l.visibility_range_end_margin = LABEL_FADE * shown
	l.visibility_range_fade_mode = GeometryInstance3D.VISIBILITY_RANGE_FADE_SELF
	return l


## `--no-people-colliders` on the command line (step-11 §22 X-2).
static func no_people_colliders() -> bool:
	return "--no-people-colliders" in OS.get_cmdline_user_args()


static func _flat(v: Vector3) -> Vector3:
	return Vector3(v.x, 0.0, v.z)
