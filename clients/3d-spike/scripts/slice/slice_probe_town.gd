## The slice's evidence harness for the town's layout (step-15 §20, PR 16c): the
## connected slice against a real `mineworld server`, walked on the real
## controller, never teleported (`ENGINEERING_RULES.md` §19).
##
##   --slice-layout   L-1 every doorway on a real door; L-2 the florist is its
##                    shop; L-3 what is shown (SHOWN); L-4 only the world's
##                    people (§20.4)
##   --slice-street   S-1 people walk; S-2 no jumps; S-3 into doors, not walls;
##                    S-4 nobody in a wall; S-5 SHOWN (§20.4), against a server
##                    hosting the town's paced controllers (`--town`).
##                    `--watch=<s>` (default 60); `--frames` saves three frames
##                    of people walking and one of a person going into a door
##
## A sibling of `slice_probe_world.gd`, as 16a split the connected modes off
## `slice_probe.gd` (§20.7 C3: that file is past ~800 lines). It names no world
## content: every place is found through the layout and the disclosure, every
## expected value is a literal of the plan or a value the world disclosed.
class_name SliceProbeTown
extends SliceProbeWorld

const TOWN_MODES := ["layout", "street"]
## The plan's doorway rule, held here independently of `SliceLayout.DOOR_INSET`
## so that the measurement does not take its yardstick from the code under test
## (step-11 SD-D3; step-15 A16c-6, M16c-2): a doorway point stands 0.40 m in
## front of its drawn door's sill.
const PLAN_INSET := 0.40
## L-1's bound (step-15 D-16c-5), likewise the plan's literal.
const PLAN_MATCH := 0.15
## L-3: once the still world has settled, a drawn position is within this of the
## observed one (step-15 §20.4).
const SHOWN_MM := 50

## place id -> the passages the world disclosed on it, as received, wherever the
## observer stood: L-3's reference, recorded from the observations themselves.
var _disclosed := {}
var _fails := 0


static func requested() -> bool:
	return requested_of(TOWN_MODES) != ""


func _modes() -> Array:
	return TOWN_MODES


func _run(mode: String) -> void:
	match mode:
		"layout": await _layout_check()
		"street": await _street_check()
		_: await super(mode)


func _verdict_line(id: String, ok: bool, detail: String) -> void:
	if not ok:
		_fails += 1
	print("%s %s  %s" % [id, "PASS" if ok else "FAIL", detail])


# --- --layout -----------------------------------------------------------------------

func _layout_check() -> void:
	print("== step-15 §20.4 -- the town's layout, from the server ==\n")
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	link.client.observed.connect(_record_disclosure)
	var waited := 0.0
	while waited < 10.0 and (not link.client.is_seated() or link.here == "" or link._reconcile):
		await get_tree().process_frame
		waited += get_process_delta_time()
	await _hold(0.8)
	var lay := link.layout
	print("seated   in %s; the anchor (tag %s) is %s, the street %s"
		% [_where(link), SliceWorld.ANCHOR_TAG, lay.anchor, lay.street])
	if lay.anchor == "" or link.here != lay.anchor:
		_verdict_line("L-0", false, "the seat starts in the place tagged %s, and the layout binds it"
			% SliceWorld.ANCHOR_TAG)
		_finish(link)
		return
	await _settled_report(link)
	_check_shown(link, "the café")
	_check_people(link, "the café")

	# out through the café's door onto the pavement, on foot
	await _walk_to_x(6.0 + SliceCafe.DOOR_X)
	player.rotation.y = PI
	await _walk_dist(3.2, 6.0)
	var t := 0.0
	while t < 5.0 and link.here != lay.street:
		await _hold(0.1)
		t += 0.1
	await _hold(0.8)
	print("street   body at %s, server place %s" % [player.global_position, _where(link)])
	if link.here != lay.street:
		_verdict_line("L-1", false, "the server did not put the player on the street")
		_finish(link)
		return
	_check_doorways(link)
	await _settled_report(link)
	_check_shown(link, "the street")
	_check_people(link, "the street")
	await _florist(link)
	_finish(link)


## L-1: every passage the street discloses is bound to a distinct drawn door, the
## doorway's scene point within PLAN_MATCH of that door's out-point by the plan's
## rule; the café's passage to the café's door; exactly one to The Flower Room's.
func _check_doorways(link: SliceLink) -> void:
	var lay := link.layout
	var street: String = lay.street
	var ok := true
	var to_florist := 0
	var used := {}
	var ps: Array = _disclosed.get(street, [])
	for p in ps:
		var to := String(p["to"].get("entity", ""))
		var at := link.to_scene(street, p["here"])
		var nearest := _nearest_door(at)
		if not lay.doors.has(to) or lay.misses.has(to):
			ok = false
			print("doorway  to place %s at street (%d, %d) -> scene (%.2f, %.2f): no drawn door within %.2f m; nearest %s at %.2f m"
				% [to, int(p["here"]["x"]), int(p["here"]["y"]), at.x, at.z, PLAN_MATCH,
				nearest[0], nearest[1]])
			continue
		var i: int = lay.doors[to]
		var d: Array = SliceTerrace.doors[i]
		var dist := _plan_distance(at, d)
		var distinct := not used.has(i)
		used[i] = to
		var volume: String = d[3] if d.size() > 3 else ""
		if volume == SliceWorld.FLORIST_PLACE:
			to_florist += 1
		var anchor_ok := to != lay.anchor or volume == SliceWorld.CAFE_PLACE
		print("doorway  to place %s (tags %s) at street (%d, %d) -> scene (%.2f, %.2f): bound to %s at %s, %.2f m%s"
			% [to, lay.tags.get(to, []), int(p["here"]["x"]), int(p["here"]["y"]), at.x, at.z, d[2], d[0],
			dist, "" if distinct else "  (DOOR ALREADY BOUND)"])
		if dist > PLAN_MATCH or not distinct or not anchor_ok:
			ok = false
			if dist > PLAN_MATCH:
				print("doorway  to place %s at street (%d, %d) -> scene (%.2f, %.2f): no drawn door within %.2f m; nearest %s at %.2f m"
					% [to, int(p["here"]["x"]), int(p["here"]["y"]), at.x, at.z, PLAN_MATCH,
					nearest[0], nearest[1]])
			if not anchor_ok:
				print("doorway  the café's passage is not bound to the café's door")
	_verdict_line("L-1", ok and to_florist == 1 and ps.size() > 0,
		"%d street passages, each on a distinct drawn door within %.2f m; %d to The Flower Room"
		% [ps.size(), PLAN_MATCH, to_florist])


## The drawn door whose plan out-point is nearest a scene point: [label, metres].
func _nearest_door(at: Vector3) -> Array:
	var best := ["none", INF]
	for d in SliceTerrace.doors:
		var dd := _plan_distance(at, d)
		if dd < best[1]:
			best = [d[2], dd]
	return best


## Horizontal metres from a scene point to a door's out-point by the plan's rule.
static func _plan_distance(at: Vector3, d: Array) -> float:
	var yaw: float = d[1]
	var out: Vector3 = (d[0] as Vector3) + Vector3(sin(yaw), 0.0, cos(yaw)) * PLAN_INSET
	return Vector2(at.x - out.x, at.z - out.z).length()


## L-2: from the pavement on foot into The Flower Room, to the middle of its
## floor's depth and back out. The server's place goes street -> P -> street,
## every move accepted, the server's position inside equal to the last report,
## someone perceived inside P. P is whichever place L-1 bound to its door.
func _florist(link: SliceLink) -> void:
	var lay := link.layout
	var shop := lay.place_of_volume(SliceWorld.FLORIST_PLACE)
	if shop == "":
		print("L-2 does not run: no place's doorway is bound to The Flower Room's door")
		_verdict_line("L-2", false, "The Flower Room depicts no place of this world")
		return
	print("florist  The Flower Room depicts place %s, tagged %s" % [shop, lay.tags.get(shop, [])])
	var door: Array = SliceTerrace.doors[lay.doors[shop]]
	var sill: Vector3 = door[0]
	var changes_before := link.place_changes.size()
	var moves_before := _moves(link).size()
	# the 16a T-2 route along the door's centre line, then to the middle of the
	# floor's depth in the west lane (the display table stands at its centre)
	await _walk_to(Vector3(sill.x, 0.0, SliceStreet.NORTH_FACE + 2.70), 8.0)
	await _walk_to(Vector3(sill.x, 0.0, SliceStreet.NORTH_FACE - 1.40), 6.0)
	var room: AABB = SliceShopInterior.room_box
	var middle := Vector3(sill.x + 0.25, 0.0, room.position.z + room.size.z * 0.5)
	await _walk_to(middle, 8.0)
	await _hold(0.6)
	var tok := link.report_position()
	var sent: Dictionary = link.last_sent_local.duplicate()
	var t := 0.0
	while t < 3.0 and not _answered(link, tok):
		await get_tree().process_frame
		t += get_process_delta_time()
	await _hold(0.8)
	var inside := link.here
	var seen: Variant = link.client.latest.self_location().get("local")
	var same := typeof(seen) == TYPE_DICTIONARY and int(seen.get("x", -1)) == int(sent["x"]) \
		and int(seen.get("y", -1)) == int(sent["y"])
	print("inside   body at %s, server place %s; position sent %s, server says %s"
		% [player.global_position, _where(link), sent, seen])
	var others := 0
	for id in link.figures:
		others += 1
	await _settled_report(link)
	_check_shown(link, "the shop")
	# and back out onto the pavement
	await _walk_to(Vector3(sill.x, 0.0, SliceStreet.NORTH_FACE - 1.40), 8.0)
	await _walk_to(Vector3(sill.x, 0.0, SliceStreet.NORTH_FACE + 2.70), 6.0)
	await _hold(1.0)
	var changes := link.place_changes.slice(changes_before)
	var moves := _moves(link).slice(moves_before)
	var bad := moves.filter(func(a): return a["result"] != "accepted")
	for b in bad:
		print("         NOT ACCEPTED: %s" % JSON.stringify(b))
	print("places   the server's place, as observed: %s" % [changes])
	var path_ok: bool = changes.size() == 2 and changes[0] == [lay.street, shop] \
		and changes[1] == [shop, lay.street]
	_verdict_line("L-2", path_ok and inside == shop and bad.is_empty() and moves.size() > 0 and same
		and others >= 1,
		"street -> %s -> street %s; %d moves, %d not accepted; position inside %s; %d perceived inside"
		% [shop, "yes" if path_ok else "NO", moves.size(), bad.size(), "equal" if same else "DIFFERS",
		others])


static func _moves(link: SliceLink) -> Array:
	return link.answers.filter(func(a): return a["action"] == SliceLink.MOVE_ACTION)


## Report where the body stands, and wait for the answer and the next
## observation, so the server's view of the observer has caught up.
func _settled_report(link: SliceLink) -> void:
	var tok := link.report_position()
	var t := 0.0
	while t < 3.0 and not _answered(link, tok):
		await get_tree().process_frame
		t += get_process_delta_time()
	await _hold(0.8)


## Every observation's own place's passages, as received (L-3's reference).
func _record_disclosure(obs: MineWorldObservation) -> void:
	var leads: Variant = obs.component(obs.place(), "passages").get("leads_to", [])
	if typeof(leads) == TYPE_ARRAY:
		_disclosed[obs.place()] = (leads as Array).duplicate(true)


## L-3, on the report the scene makes (`SliceLink.shown`), printed as a SHOWN line.
## The same checks run on a counterfactual report whose people are read from the
## observation instead of the drawn figures: printed, never asserted (M16c-8).
func _check_shown(link: SliceLink, where: String) -> void:
	var report := _print_shown(link)
	var problems := _shown_problems(link, report)
	for p in problems:
		print("shown    %s" % p)
	_verdict_line("L-3", problems.is_empty(), "SHOWN in %s: %d places, %d doorways, %d people"
		% [where, report["places"].size(), report["doorways"].size(), report["people"].size()])
	var observed := report.duplicate(true)
	var obs := link.client.latest
	for who in observed["people"]:
		var loc: Variant = obs.self_location().get("local") if who["id"] == obs.observer() \
			else obs.location_of(who["id"]).get("local")
		if typeof(loc) == TYPE_DICTIONARY:
			who["local"] = {"x": int(loc.get("x", 0)), "y": int(loc.get("y", 0)), "z": 0}
	var cf := _shown_problems(link, observed)
	print("shown    counterfactual, people read from the observation instead of the scene: %s"
		% ("would pass" if cf.is_empty() else "would fail: %s" % "; ".join(cf)))


func _shown_problems(link: SliceLink, report: Dictionary) -> PackedStringArray:
	var out := PackedStringArray()
	var obs := link.client.latest
	var here := obs.place()
	if report["place"] != here:
		out.append("place %s, the observation says %s" % [report["place"], here])
	var places := {}
	for p in report["places"]:
		if typeof(p["place"]) != TYPE_STRING:
			out.append("a place id that is not a string: %s" % [p["place"]])
		if places.has(p["place"]):
			out.append("place %s listed twice" % p["place"])
		places[p["place"]] = true
	# every doorway is a disclosed passage, field for field; every disclosed
	# passage whose both sides the report lists is listed
	var listed := {}
	for d in report["doorways"]:
		var key := "%s>%s" % [d["from"], d["to"]]
		listed[key] = true
		var found := false
		for p in _disclosed.get(d["from"], []):
			if String(p["to"].get("entity", "")) == d["to"] \
					and d["here"] == [int(p["here"]["x"]), int(p["here"]["y"])] \
					and d["there"] == [int(p["there"]["x"]), int(p["there"]["y"])]:
				found = true
		if not found:
			out.append("doorway %s is not a passage the world disclosed: %s" % [key, JSON.stringify(d)])
	for from in _disclosed:
		for p in _disclosed[from]:
			var key := "%s>%s" % [from, String(p["to"].get("entity", ""))]
			if not listed.has(key):
				out.append("the world disclosed the passage %s and the report does not list it" % key)
	# people: every one the observation lists here, once, and the observer
	var expected := {obs.observer(): true}
	for id in obs.ids():
		var loc := obs.location_of(id)
		if typeof(loc.get("place")) == TYPE_DICTIONARY and String(loc["place"].get("entity", "")) == here:
			expected[id] = true
	var people := {}
	for who in report["people"]:
		var id: Variant = who["id"]
		if typeof(id) != TYPE_STRING:
			out.append("a person id that is not a string: %s" % [id])
			continue
		if people.has(id):
			out.append("person %s listed twice" % id)
		people[id] = true
		if not expected.has(id):
			out.append("person %s is drawn and the observation does not list them here" % id)
		if String(who["label"]) == String(id) or expected.has(String(who["label"])):
			out.append("person %s is labelled with an id" % id)
		var local: Variant = who["local"]
		if typeof(local) != TYPE_DICTIONARY or typeof(local.get("x")) != TYPE_INT \
				or typeof(local.get("y")) != TYPE_INT:
			out.append("person %s: local %s is not integer millimetres" % [id, JSON.stringify(local)])
			continue
		var stated: Variant = obs.self_location().get("local") if id == obs.observer() \
			else obs.location_of(id).get("local")
		if typeof(stated) != TYPE_DICTIONARY:
			continue
		var off := Vector2(int(local["x"]) - int(stated.get("x", 0)),
			int(local["y"]) - int(stated.get("y", 0))).length()
		if off > SHOWN_MM:
			out.append("person %s (%s) drawn at local (%d, %d), the observation says (%d, %d): %.0f mm"
				% [id, who["label"], local["x"], local["y"], int(stated.get("x", 0)),
				int(stated.get("y", 0)), off])
	for id in expected:
		if not people.has(id):
			out.append("person %s is in the observation and not drawn" % id)
	return out


## L-4: every NPC node in the scene, other than the player's own body, stands for
## a person the latest observation lists in the observer's place.
func _check_people(link: SliceLink, where: String) -> void:
	var obs := link.client.latest
	var here := obs.place()
	var others := []
	var count := 0
	for n in _npcs(slice):
		count += 1
		var id := String(n.get_meta("entity_id")) if n.has_meta("entity_id") else ""
		var loc := obs.location_of(id) if id != "" else {}
		var listed := id != "" and typeof(loc.get("place")) == TYPE_DICTIONARY \
			and String(loc["place"].get("entity", "")) == here
		if not listed:
			others.append("%s at %s" % [n.name, n.global_position])
	for o in others:
		print("people   not a person of the world here: %s" % o)
	_verdict_line("L-4", others.is_empty(), "in %s: %d figures drawn, %d not the world's"
		% [where, count, others.size()])


## Every NPC under `n`, except the player's own body.
func _npcs(n: Node) -> Array:
	var out := []
	for c in n.get_children():
		if c == player:
			continue
		if c is NPC:
			out.append(c)
		out.append_array(_npcs(c))
	return out


func _finish(link: SliceLink, what := "layout") -> void:
	print("\n%s" % ("all %s checks pass" % what if _fails == 0 else "%d %s CHECKS FAILED" % [_fails,
		what.to_upper()]))
	link.client.disconnect_from_world("probe done")
	await _hold(0.2)


# --- --street -----------------------------------------------------------------------

## S-2's bound: a figure's drawn speed in any frame, outside a placement, is at
## most the player's jog with 5 % for the frame clock (step-15 §20.4).
const SPEED_BOUND := Player.JOG_SPEED * 1.05
## S-1: a person counts as walking after an observed change of position of this.
const WALKED_M := 1.0
## S-3: a figure that leaves view is removed this close to a bound door's point.
const AT_DOOR_M := 0.15
## S-4: the capsule a person is (r 0.30, h 1.72, as the player's), less this
## depth: an overlap deeper than it is a figure inside a wall. The lowest
## STEP_M is left out, because kerbs, steps and the café's ramp are walked on.
const DEPTH_M := 0.05
const STEP_M := 0.15
## S-5: a figure counts as standing once its observed position has not changed
## for this long; L-3's 50 mm applies to the people standing (a walking figure
## is drawn behind the person it follows, by design).
const STANDING_S := 1.0


## S-1 ... S-5 from a fixed spot on the pavement, standing still, while the
## hosted town walks (step-15 §20.4).
func _street_check() -> void:
	print("== step-15 §20.4 -- the living street ==\n")
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	link.client.observed.connect(_record_disclosure)
	var secs := 60.0
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--watch="):
			secs = float(a.substr(8))
	var frames := "--frames" in OS.get_cmdline_user_args()
	var waited := 0.0
	while waited < 10.0 and (not link.client.is_seated() or link.here == "" or link._reconcile):
		await get_tree().process_frame
		waited += get_process_delta_time()
	await _hold(0.8)
	var lay := link.layout
	if lay.anchor == "" or link.here != lay.anchor:
		_verdict_line("S-0", false, "the seat starts in the place tagged %s" % SliceWorld.ANCHOR_TAG)
		_finish(link, "street")
		return
	await _walk_to_x(6.0 + SliceCafe.DOOR_X)
	player.rotation.y = PI
	await _walk_dist(3.2, 6.0)
	await _hold(1.5)
	if link.here != lay.street:
		_verdict_line("S-0", false, "the server did not put the player on the street")
		_finish(link, "street")
		return
	player.rotation.y = -PI * 0.5
	player.set_camera(FP)
	await _settled_report(link)
	var figs: SliceFigures = link._figures
	var placed0 := figs.placements.size()
	var placed_start := placed0
	var leaves0 := figs.leaves.size()
	print("watch    %.0f s from %s, facing east, in %s; %d people in view"
		% [secs, player.global_position, _where(link), link.figures.size()])

	var first := {}          # id -> the first observed local position on the street
	var farthest := {}       # id -> the largest observed change of position, metres
	var last_drawn := {}     # id -> the drawn position last frame
	var fastest := {}        # id -> the largest drawn speed outside a placement
	var over: Array = []     # [id, speed, at] frames over SPEED_BOUND
	var walls: Array = []    # S-4 overlaps
	var samples := 0
	var seen_leaving := {}
	var shots := 0
	var door_shot := false
	var placed_at := {}
	var t := 0.0
	var next_sample := 0.0
	var next_shot := secs * 0.25
	var wall0 := Time.get_ticks_msec()
	while t < secs:
		await get_tree().process_frame
		var dt := get_process_delta_time()
		t += dt
		var obs := link.client.latest
		for id in obs.ids():
			var loc := obs.location_of(id)
			if id == obs.observer() or typeof(loc.get("place")) != TYPE_DICTIONARY \
					or String(loc["place"].get("entity", "")) != lay.street:
				continue
			var p := MineWorldSpace.to_3d(loc.get("local"))
			if not first.has(id):
				first[id] = p
			farthest[id] = maxf(farthest.get(id, 0.0), _flat_len(p - (first[id] as Vector3)))
		for i in range(placed0, figs.placements.size()):
			placed_at[figs.placements[i]["id"]] = true
		placed0 = figs.placements.size()
		for id in link.figures:
			var at: Vector3 = (link.figures[id] as Node3D).global_position
			if last_drawn.has(id) and not placed_at.has(id) and dt > 0.0:
				var v := _flat_len(at - (last_drawn[id] as Vector3)) / dt
				fastest[id] = maxf(fastest.get(id, 0.0), v)
				if v > SPEED_BOUND:
					over.append([id, v, at])
			last_drawn[id] = at
		placed_at.clear()
		# S-4, at 1 Hz
		if t >= next_sample:
			next_sample += 1.0
			samples += 1
			for id in link.figures:
				var hit := _in_wall(link.figures[id] as Node3D)
				if hit != "":
					walls.append("%s (%s) at %s: %s" % [link.display_label(id), id,
						(link.figures[id] as Node3D).global_position, hit])
		if frames:
			for id in link.figures:
				if shots < 3 and t >= next_shot and fastest.get(id, 0.0) > 0.6 \
						and _flat_len((last_drawn.get(id, Vector3.ZERO) as Vector3) - player.global_position) < 14.0:
					await _aim_at((link.figures[id] as Node3D).global_position + Vector3.UP * 1.2)
					await _save("street_%d_walking" % (shots + 1))
					print("frame    street_%d_walking: %s (%s) at %s" % [shots + 1, link.display_label(id), id,
						(link.figures[id] as Node3D).global_position])
					shots += 1
					next_shot += secs * 0.25
					break
			if not door_shot:
				for id in figs._going:
					var n := figs.going(id)
					if n != null and not seen_leaving.has(id):
						seen_leaving[id] = true
						await _aim_at(n.global_position + Vector3.UP * 1.2)
						await _save("street_4_into_a_door")
						print("frame    street_4_into_a_door: %s going into %s, at %s"
							% [id, figs._going[id]["door"] if figs._going.has(id) else "?", n.global_position])
						door_shot = true
						break
	print("watch    %.0f s in %.1f s wall; %d S-4 samples" % [secs, (Time.get_ticks_msec() - wall0) / 1000.0,
		samples])

	# S-1: people walk
	var walkers := []
	for id in farthest:
		print("person   %s (%s): observed moving up to %.2f m; drawn at most %.2f m/s"
			% [link.display_label(id), id, farthest[id], fastest.get(id, 0.0)])
		if farthest[id] >= WALKED_M:
			walkers.append(id)
	var active := walkers.size() >= 3
	print("S-1 %s  %d distinct people on the street moved %.0f m or more (3 needed)"
		% ["PASS" if active else "INCONCLUSIVE", walkers.size(), WALKED_M])
	# S-2: no jumps
	var watched: Array = figs.placements.slice(placed_start)
	var later := watched.filter(func(p): return not p["first"])
	for o in over:
		print("jump     %s drawn at %.2f m/s at %s" % [o[0], o[1], o[2]])
	for p in watched:
		print("placed   %s %s, gap %.2f m, at %s" % [p["id"], "on first sight" if p["first"] else "after a gap",
			p["gap"], p["at"]])
	_verdict_line("S-2", over.is_empty() and later.size() <= 2,
		"%d frames over %.2f m/s outside a placement; %d placements after first sight (at most 2)"
		% [over.size(), SPEED_BOUND, later.size()])
	# S-3: into doors, not walls
	var leaves := figs.leaves.slice(leaves0)
	var at_doors := 0
	for l in leaves:
		var near := _nearest_point(link, l["at"])
		print("leave    %s removed at %s, %.2f m from %s%s" % [l["id"], l["at"], near[1], near[0],
			" (walked in)" if l["walked"] else " (removed where it stood)"])
		if near[1] <= AT_DOOR_M:
			at_doors += 1
	if leaves.is_empty():
		print("S-3 INCONCLUSIVE  nobody left view during the watch")
	else:
		_verdict_line("S-3", at_doors == leaves.size(), "%d of %d leaves at a bound door, within %.2f m"
			% [at_doors, leaves.size(), AT_DOOR_M])
	# S-4: nobody in a wall
	for w in walls:
		print("wall     %s" % w)
	_verdict_line("S-4", walls.is_empty(), "%d overlaps deeper than %.2f m in %d samples"
		% [walls.size(), DEPTH_M, samples])
	# S-5: what is shown at the end
	await _settled_report(link)
	_check_shown_standing(link)
	if not active:
		print("S-1 found fewer than 3 walking: S-2 ... S-4 are not claimed (step-15 §20.4)")
	_finish(link, "street")


## S-3's yardstick: the nearest in- or out-point of a door the layout bound, by
## the plan's 0.40 m rule (the registry gives each room's depth and floor).
func _nearest_point(link: SliceLink, at: Vector3) -> Array:
	var best := ["none", INF]
	for place in link.layout.doors:
		var d: Array = SliceTerrace.doors[link.layout.doors[place]]
		var yaw: float = d[1]
		var n := Vector3(sin(yaw), 0.0, cos(yaw))
		var points := [(d[0] as Vector3) + n * PLAN_INSET]
		if d.size() > 5 and d[3] != "":
			points.append((d[0] as Vector3) - n * (float(d[4]) + PLAN_INSET))
		for q in points:
			var dd := _flat_len(at - q)
			if dd < best[1]:
				best = [d[2], dd]
	return best


## S-4: whether the person's capsule, less DEPTH_M and above STEP_M, overlaps
## the scene's geometry; the colliders met, or "".
func _in_wall(fig: Node3D) -> String:
	var shape := CapsuleShape3D.new()
	shape.radius = Player.CAPSULE_RADIUS - DEPTH_M
	shape.height = Player.CAPSULE_HEIGHT - STEP_M - DEPTH_M
	var q := PhysicsShapeQueryParameters3D.new()
	q.shape = shape
	q.collision_mask = Build.LAYER_WORLD
	q.transform = Transform3D(Basis(), fig.global_position + Vector3(0, STEP_M + shape.height * 0.5, 0))
	var hits := fig.get_world_3d().direct_space_state.intersect_shape(q, 4)
	var names := PackedStringArray()
	for h in hits:
		var c := h["collider"] as Node
		if c != null:
			names.append(String(c.get_path()))
	return ", ".join(names)


## S-5: L-3's checks, with the 50 mm bound on the people who stand.
func _check_shown_standing(link: SliceLink) -> void:
	var report := _print_shown(link)
	var problems := _shown_problems(link, report)
	var obs := link.client.latest
	var kept := PackedStringArray()
	for p in problems:
		var moving := false
		for id in link.figures:
			if p.begins_with("person %s (" % id) and p.contains("drawn at local") \
					and not _standing(link, id):
				moving = true
		if moving:
			print("shown    (walking, not held to %d mm) %s" % [SHOWN_MM, p])
		else:
			kept.append(p)
	for p in kept:
		print("shown    %s" % p)
	_verdict_line("S-5", kept.is_empty(), "SHOWN at the end: %d places, %d doorways, %d people"
		% [report["places"].size(), report["doorways"].size(), report["people"].size()])


## Whether a perceived person's observed position has stood still for STANDING_S.
func _standing(link: SliceLink, id: String) -> bool:
	var g: Variant = link._figures._goal.get(id)
	return g != null and Time.get_ticks_msec() / 1000.0 - float(g["changed"]) >= STANDING_S


static func _flat_len(v: Vector3) -> float:
	return Vector2(v.x, v.z).length()
