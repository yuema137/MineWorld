## The slice's evidence harness for bodies (S15 PR 12e, step-11 §22 SD-E12):
## the connected slice against a real `mineworld server`, walked on the real
## controller, with extra seats -- PUPPETS -- held on connections of their own.
##
##   --slice-bodies   B-a nudge, B-b crowd, B-c box, B-d kick, B-e throw, B-f
##                    shove, C-2b shoved while walking, B-g end state, B-r the
##                    radius, C-1 a free walk on the pavement; C-2 … C-4 measured
##                    on every correction. `--only=a,b,…` runs a subset (C-5's
##                    runs at two time scales use `--only=d,f`); `--frames`
##                    (windowed) saves the evidence frames, third person rear.
##   --slice-rules    X-1: the same scripted interactions -- talk from the door
##                    and at the counter, shove Bob, kick and throw the café ball
##                    -- printed as REQUEST lines; the launcher runs it against
##                    social-cafe and a copy without bodies and compares.
##   --no-people-colliders   (with --slice-bodies) X-2: no person collider.
##
## Every staged event is a real request on a real connection: the puppets
## (`bob`, `wanderer`, `ivan`; free seats in a scripted run, which hosts no
## `--town`) move by the same `move` request (`SliceLink.MOVE_ACTION`) and shove
## through `SliceIntents.shove`. The oracle never quotes a server rule number:
## it asserts what the client must make true -- drawn = observed, corrections
## converge, colliders match behaviour -- with the plan's own bounds, and reads
## the server's facts from the observation's events. Positions it walks to are
## the authored ones of `worlds/social-cafe` (step-11 §19), as millimetres in
## the café's frame, mapped by the layout like any client point.
class_name SliceProbeBodies
extends SliceProbeGeometry

const BODIES_MODES := ["bodies", "rules"]
## The plan's bounds (step-11 §22.4), held here, not taken from the code under test.
const CAPSULE_R := 0.25                 ## the server's person, the plan's literal
const TOUCH_SLACK := 0.005              ## drawn separation ≥ 2R − 5 mm
const DRAWN_MM := 1.0                   ## drawn = observed, people and objects
const BODY_MM := 150.0                  ## the player within 150 mm
const GLIDE_S := 0.12                   ## the plan's glide, ± one frame
const STILL_MM := 10.0                  ## C-4
const STEP_SLACK_MM := 5.0              ## C-3
## Where things stand, café frame, millimetres (worlds/social-cafe, 12d).
const OPEN_FLOOR := Vector2(3000, 4800)       ## B-a: wanderer, on open floor
const CROWD := [Vector2(5200, 6300), Vector2(5710, 6300), Vector2(6220, 6300)]  ## B-b, the counter's face at 6 570
const LANE_X := 6000.0                        ## between the round tables
const SHOVE_SPOT := Vector2(2000, 3000)       ## B-f: the player stands here
const JOG_LANE_Y := 1900.0                    ## C-2b
const SHOVER_AT := Vector2(4500, 1300)        ## C-2b: Bob, south of the lane
const SEATS := ["bob", "wanderer", "ivan"]

var _b_fails := 0
var _only: PackedStringArray = []
var _frames := false
var puppets := {}
var _p_answers := {}
## Per physics frame: the smallest drawn separation from each figure, by phase.
var _phase := ""
var _min_sep := {}
## The server-stated separations (B-r), and the observer's facts.
var _server_min := INF
var _stopped_short: Array = []
## Body positions per physics frame (C-3), with time.
var _track: Array = []
var _box_overlap := 0.0


static func requested() -> bool:
	return requested_of(BODIES_MODES) != ""


func _modes() -> Array:
	return BODIES_MODES


func _run(mode: String) -> void:
	match mode:
		"bodies": await _bodies_check()
		"rules": await _rules_check()
		_: await super(mode)


func _verdict_b(id: String, ok: bool, detail: String) -> void:
	if not ok:
		_b_fails += 1
	print("%s %s  %s" % [id, "PASS" if ok else "FAIL", detail])


func _physics_process(delta: float) -> void:
	super(delta)
	var link: SliceLink = slice.link if slice != null else null
	if link == null or player == null:
		return
	_track.append([Time.get_ticks_usec() / 1e6, player.global_position, player.velocity, player.gliding()])
	if _phase == "":
		return
	for id in link.figures:
		var d := _flat_len((link.figures[id] as Node3D).global_position - player.global_position)
		var key := "%s/%s" % [_phase, id]
		_min_sep[key] = minf(float(_min_sep.get(key, INF)), d)
	if _phase == "c" and link.objects != null:
		for id in link.objects.nodes:
			if link.objects.shape_of(id) != "box":
				continue
			var b: Node3D = link.objects.nodes[id]
			# the box's half extents as disclosed, never read from the collider under test
			var half := _disclosed_half(link, id)
			var rel := player.global_position - b.global_position
			var dx := maxf(absf(rel.x) - half.x, 0.0)
			var dz := maxf(absf(rel.z) - half.z, 0.0)
			var o := CAPSULE_R - Vector2(dx, dz).length()
			if o > 0.005 and o > _box_overlap:
				print("B-c      overlap %.1f mm at %.2f s: gliding %s, box moving %s, input velocity %s; body %s, box %s, half %s"
					% [o * 1000.0, Time.get_ticks_msec() / 1000.0, player.gliding(), link.objects.moving(id),
					player.velocity, player.global_position, b.global_position, half])
				print("B-c      at that moment the server states %s" % _server_box_gap(link, id))
			_box_overlap = maxf(_box_overlap, o)


func _on_obs(obs: MineWorldObservation) -> void:
	var shape: Variant = obs.component_value(obs.place(), "place-shape")
	if typeof(shape) == TYPE_DICTIONARY:
		_shapes[obs.place()] = shape
	var me := obs.self_location()
	var mine: Variant = me.get("local")
	if typeof(mine) == TYPE_DICTIONARY and _phase in ["a", "b"]:
		for id in obs.ids():
			if id == obs.observer():
				continue
			var loc := obs.location_of(id)
			if typeof(loc.get("place")) != TYPE_DICTIONARY or String(loc["place"].get("entity", "")) != obs.place():
				continue
			var d := Vector2(float(mine["x"]), float(mine["y"])).distance_to(
				Vector2(float(loc["local"]["x"]), float(loc["local"]["y"]))) / 1000.0
			_server_min = minf(_server_min, d)
	for e in obs.events_of("stopped-short"):
		var p: Variant = e.get("payload", {}).get("payload")
		if typeof(p) == TYPE_DICTIONARY:
			_stopped_short.append(p)


# --- puppets ------------------------------------------------------------------------

func _puppet(seat: String) -> MineWorldClient:
	var c := MineWorldClient.new()
	c.name = "Puppet_" + seat
	add_child(c)
	c.resolved.connect(func(token: String, _id: String, result: Dictionary) -> void:
		_p_answers[token] = String(result.keys()[0]) if not result.is_empty() else "?"
		if not result.has("accepted"):
			print("puppet   %s: answered %s" % [seat, JSON.stringify(result)]))
	c.refused.connect(func(code: String, token: String, _d: String) -> void:
		_p_answers[token] = "refused " + code)
	c.connect_to_world(SliceLink.address_from_args(), seat, SliceLink.invite_from_args(), "puppet")
	var t := 0.0
	while t < 10.0 and (not c.is_seated() or c.latest == null):
		await _hold(0.1)
		t += 0.1
	print("puppet   %s seated %s as %s in %s at %s" % [seat, c.is_seated(), c.observer,
		c.latest.place() if c.latest != null else "?", _p_at(c)])
	return c


static func _p_at(c: MineWorldClient) -> Vector2:
	if c.latest == null:
		return Vector2.INF
	var l: Variant = c.latest.self_location().get("local")
	return Vector2(float(l["x"]), float(l["y"])) if typeof(l) == TYPE_DICTIONARY else Vector2.INF


## One stride of a puppet to `to` in `place` (mm), facing `yaw_mdeg`; waits for
## the answer and the next observation. Returns the answer's kind.
func _p_step(c: MineWorldClient, place: String, to: Vector2, yaw_mdeg: int) -> String:
	var loc := MineWorldSpace.location(place, {"x": int(to.x), "y": int(to.y), "z": 0}, yaw_mdeg)
	for attempt in range(5):
		var seq := c.sequence
		var tok := c.submit(SliceLink.MOVE_ACTION, null, {"to": loc}, loc)
		var t := 0.0
		while t < 4.0 and not _p_answers.has(tok):
			await _hold(0.05)
			t += 0.05
		var kind := String(_p_answers.get(tok, "NO ANSWER"))
		if kind.begins_with("refused rate_limited"):
			# the seat's request budget (PROTOCOL.md §8): wait, and ask again
			await _hold(0.3)
			continue
		while t < 6.0 and c.sequence == seq:
			await _hold(0.05)
			t += 0.05
		return kind
	return "refused rate_limited"


## A puppet walks a route in strides of at most 0.9 m.
func _p_walk(c: MineWorldClient, place: String, route: Array, yaw_mdeg := 180000) -> bool:
	for goal in route:
		var g: Vector2 = goal
		var guard := 0
		while _p_at(c).distance_to(g) > 2.0 and guard < 40:
			guard += 1
			var at := _p_at(c)
			var next := at + (g - at).limit_length(900.0)
			var kind := await _p_step(c, place, next, yaw_mdeg)
			if kind != "accepted":
				print("puppet   %s: stride to %s answered %s, at %s" % [c.seat, next, kind, _p_at(c)])
				return false
			if _p_at(c).distance_to(next) > 100.0:
				print("puppet   %s: stride to %s accepted, but stands at %s; %s" % [c.seat, next, _p_at(c),
					JSON.stringify(c.latest.events_of("stopped-short").map(func(e): return e.get("payload", {}).get("payload")))])
				if _p_at(c).distance_to(at) < 10.0:
					return false
	return true


## Ivan walks from the street into the café: south of the pavement's furniture
## to the doorway's line, to the street's side of the café's doorway, then the
## crossing, as any client reports one.
func _p_into_cafe(c: MineWorldClient, link: SliceLink) -> bool:
	var street := link.layout.street
	var cafe := link.layout.anchor
	var door: Dictionary = {}
	# the doorway as Ivan's own view discloses it, read the way the client reads one
	var seen := SliceLayout.new()
	seen.learn(c.latest)
	for p in seen.passages.get(street, []):
		if String(p["to"]) == cafe:
			door = p
	if door.is_empty():
		print("puppet   ivan: the street discloses no doorway to the café")
		return false
	var there := Vector2(float(door["here"]["x"]), float(door["here"]["y"]))
	var inside := Vector2(float(door["there"]["x"]), float(door["there"]["y"]))
	if not await _p_walk(c, street, [Vector2(_p_at(c).x, 650), Vector2(there.x, 650), there], 0):
		return false
	var kind := await _p_step(c, cafe, inside, 0)
	print("puppet   ivan crossed into the café: %s, now in %s at %s" % [kind, c.latest.place(), _p_at(c)])
	return kind == "accepted" and c.latest.place() == cafe


# --- helpers --------------------------------------------------------------------------

func _cafe(link: SliceLink, mm: Vector2) -> Vector3:
	return link.to_scene(link.layout.anchor, {"x": int(mm.x), "y": int(mm.y)})


func _on(id: String) -> bool:
	return _only.is_empty() or _only.has(id)


func _frame(nm: String) -> void:
	if _frames:
		await _save("12e_" + nm)


## Walk into whatever stands at `toward` (scene point) holding the input for
## `secs` after the body first stops moving forward.
func _press_into(toward: Vector3, secs: float) -> void:
	var d := toward - player.global_position
	player.rotation.y = atan2(-d.x, -d.z)
	Input.action_press("move_forward")
	var t := 0.0
	var still := 0.0
	var last := player.global_position
	while t < 12.0 and still < secs:
		await get_tree().physics_frame
		var dt := get_physics_process_delta_time()
		t += dt
		if _flat_len(player.global_position - last) < 0.002 * 60.0 * dt:
			still += dt
		last = player.global_position
	Input.action_release("move_forward")
	player.velocity = Vector3.ZERO
	await _hold(0.6)


## Where the server holds the player, millimetres in its place's frame.
static func _server_at(link: SliceLink) -> Vector2:
	var l: Variant = link.client.latest.self_location().get("local")
	return Vector2(float(l["x"]), float(l["y"])) if typeof(l) == TYPE_DICTIONARY else Vector2.INF


## A listed box's half extents (x east, y up, z south in the scene), metres, as disclosed.
func _disclosed_half(link: SliceLink, box: String) -> Vector3:
	var listing: Variant = link.client.latest.component_value(link.here, "loose-objects")
	if typeof(listing) == TYPE_ARRAY:
		for e in listing:
			if SliceObjects.id_of(e.get("object")) == box:
				var h: Dictionary = e["shape"].get("box", {})
				return Vector3(float(h.get("x", 0)), float(h.get("z", 0)), float(h.get("y", 0))) / 1000.0
	return Vector3.ZERO


## How far the server's own player stands from the server's own box, from the latest
## observation alone: "{gap} mm (player p, box b)".
func _server_box_gap(link: SliceLink, box: String) -> String:
	var me := _server_at(link)
	var listing: Variant = link.client.latest.component_value(link.here, "loose-objects")
	if typeof(listing) != TYPE_ARRAY:
		return "no listing"
	for e in listing:
		if SliceObjects.id_of(e.get("object")) == box:
			var at: Dictionary = e["at"]
			var half: Dictionary = e["shape"].get("box", {})
			var dx := maxf(absf(me.x - float(at["x"])) - float(half.get("x", 0)), 0.0)
			var dy := maxf(absf(me.y - float(at["y"])) - float(half.get("y", 0)), 0.0)
			return "%.1f mm from the box's faces (player %s, box (%s, %s))" % [Vector2(dx, dy).length(), me,
				at["x"], at["y"]]
	return "no box"


func _corrections(link: SliceLink, since: int) -> Array:
	return link.correction.history.slice(since)


## The drawn figure of a puppet, by its observer id.
func _drawn_sep_ok(id_prefix: String) -> Array:
	var worst := INF
	var who := ""
	for k in _min_sep:
		if String(k).begins_with(id_prefix + "/") and float(_min_sep[k]) < worst:
			worst = _min_sep[k]
			who = String(k).split("/")[1]
	return [worst, who]


func _report_and_settle(link: SliceLink) -> void:
	await _settled_report(link)


# --- --bodies -----------------------------------------------------------------------

func _bodies_check() -> void:
	print("== step-11 §22.4 B / C -- bodies in the 3D client ==\n")
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--only="):
			_only = a.substr(7).split(",")
	_frames = "--frames" in OS.get_cmdline_user_args()
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	if not await _seated(link):
		return
	link.client.observed.connect(_on_obs)
	if _frames:
		player.set_camera(REAR)
	print("people colliders %s; capsule r %.3f" % ["OFF (--no-people-colliders)"
		if SliceFigures.no_people_colliders() else "on", Player.CAPSULE_RADIUS])
	await _hold(0.8)
	for seat in SEATS:
		puppets[seat] = await _puppet(seat)
	var cafe := link.layout.anchor
	if _on("a"): await _b_nudge(link, cafe)
	if _on("b"): await _b_crowd(link, cafe)
	if _on("c"): await _b_box(link, cafe)
	if _on("d"): await _b_kick(link, cafe)
	if _on("e"): await _b_throw(link, cafe)
	if _on("f"): await _b_shove(link, cafe)
	if _on("c2b"): await _c_shoved_walking(link, cafe)
	if _on("g"): await _b_end(link)
	if _on("r"):
		_verdict_b("B-r", _server_min >= 2 * CAPSULE_R and _server_min <= 2 * CAPSULE_R + 0.05,
			"smallest server-stated distance to a person over B-a and B-b %.3f m, in [%.2f, %.2f]"
			% [_server_min, 2 * CAPSULE_R, 2 * CAPSULE_R + 0.05])
	_c_summary(link)
	if _on("free"): await _c_free_walk(link)
	print("\nOBSERVER %s" % link.client.observer)
	print("\n%s" % ("all bodies checks pass" if _b_fails == 0 else "%d BODIES CHECKS FAILED" % _b_fails))
	for c in puppets.values():
		(c as MineWorldClient).disconnect_from_world("probe done")
	link.client.disconnect_from_world("probe done")
	await _hold(0.3)


func _b_nudge(link: SliceLink, cafe: String) -> void:
	var w: MineWorldClient = puppets["wanderer"]
	var ok := await _p_walk(w, cafe, [Vector2(_p_at(w).x, 3900), Vector2(OPEN_FLOOR.x, 3900), OPEN_FLOOR])
	await _hold(1.0)
	var before := _p_at(w)
	var c0 := link.correction.history.size()
	var pressed0 := link.press_reports
	await _walk_to(_cafe(link, Vector2(1610, OPEN_FLOOR.y)), 8.0)
	await _walk_to(_cafe(link, Vector2(OPEN_FLOOR.x - 1200, OPEN_FLOOR.y)), 6.0)
	_phase = "a"
	await _press_into(_cafe(link, _p_at(w)), 2.0)
	await _frame("nudge")
	await _hold(1.0)
	_phase = ""
	var after := _p_at(w)
	var sep := _drawn_sep_ok("a")
	var fig: Node3D = link.figures.get(w.observer)
	var drawn_gap := INF
	if fig != null:
		drawn_gap = _flat_len(fig.global_position - _cafe(link, after)) * 1000.0
	var cs := _corrections(link, c0)
	if SliceFigures.no_people_colliders():
		# X-2: nothing local keeps them apart; the server does, and the client draws its answer
		await _settled_report(link)
		var me: Variant = link.client.latest.self_location().get("local")
		var gap := _flat_len(player.global_position - link.to_scene(cafe, me)) * 1000.0
		var kinds := {}
		for a in link.answers:
			kinds[a["action"]] = true
		var allowed := [SliceLink.MOVE_ACTION, SliceIntents.TALK, SliceIntents.KICK, SliceIntents.THROW,
			SliceIntents.SHOVE]
		_verdict_b("X-2", before.distance_to(after) > 10.0 and not cs.is_empty() and gap <= BODY_MM
			and kinds.keys().all(func(k): return allowed.has(k)),
			"no person collider: wanderer moved %.0f mm by the server's answer; %d corrections; body - observed %.0f mm; requests %s"
			% [before.distance_to(after), cs.size(), gap, kinds.keys()])
		return
	# tightened after M-E5 survived the plan's 10 mm alone (§22.15 X12e-18): the nudge must come from
	# a reported wish, not from strides that only reach contact
	var pressed := link.press_reports - pressed0
	_verdict_b("B-a", ok and before.distance_to(after) > 10.0 and float(sep[0]) >= 2 * CAPSULE_R - TOUCH_SLACK
		and drawn_gap <= DRAWN_MM and cs.size() <= 1 and pressed >= 1,
		"wanderer moved %.0f mm (%s -> %s); smallest drawn separation %.3f m (%s); drawn - observed %.1f mm; %d corrections; %d wished strides reported"
		% [before.distance_to(after), before, after, sep[0], sep[1], drawn_gap, cs.size(), pressed])
	await _c4_still(link, cs, "B-a")


func _b_crowd(link: SliceLink, cafe: String) -> void:
	var b: MineWorldClient = puppets["bob"]
	var w: MineWorldClient = puppets["wanderer"]
	var i: MineWorldClient = puppets["ivan"]
	var ok := await _p_walk(b, cafe, [CROWD[0]])
	ok = await _p_walk(w, cafe, [Vector2(CROWD[1].x, _p_at(w).y), CROWD[1]]) and ok
	if i.latest.place() != cafe:
		ok = await _p_into_cafe(i, link) and ok
	ok = await _p_walk(i, cafe, [Vector2(1610, 1500), Vector2(3800, 1500), Vector2(3800, 5600),
		Vector2(CROWD[2].x, 5600), CROWD[2]]) and ok
	# each puppet re-stands on its point (R-E7)
	for k in range(3):
		var c: MineWorldClient = [b, w, i][k]
		if _p_at(c).distance_to(CROWD[k]) > 2.0:
			await _p_step(c, cafe, CROWD[k], 180000)
	print("crowd    bob %s, wanderer %s, ivan %s; the player %s" % [_p_at(b), _p_at(w), _p_at(i),
		_server_at(link)])
	await _hold(1.0)
	var c0 := link.correction.history.size()
	var moves0 := _moves(link).size()
	var ss0 := _stopped_short.size()
	await _walk_to(_cafe(link, Vector2(LANE_X, 4000)), 8.0)
	await _walk_to(_cafe(link, Vector2(CROWD[1].x, 5300)), 6.0)
	_phase = "b"
	await _press_into(_cafe(link, CROWD[1]), 3.0)
	await _frame("crowd")
	_phase = ""
	await _report_and_settle(link)
	var me: Variant = link.client.latest.self_location().get("local")
	var body_gap := _flat_len(player.global_position - link.to_scene(cafe, me)) * 1000.0
	var bad := _moves(link).slice(moves0).filter(func(a): return a["result"] != "accepted")
	var mine := _stopped_short.slice(ss0).filter(func(p): return SliceObjects.id_of(p.get("person")) == link.client.observer)
	var sep := _drawn_sep_ok("b")
	_verdict_b("B-b", ok and not mine.is_empty() and bad.is_empty() and float(sep[0]) >= 2 * CAPSULE_R - TOUCH_SLACK
		and body_gap <= BODY_MM,
		"%d stopped-short of the player; %d moves not accepted; smallest drawn separation %.3f m (%s); body - observed %.0f mm; %d corrections"
		% [mine.size(), bad.size(), sep[0], sep[1], body_gap, _corrections(link, c0).size()])
	await _c4_still(link, _corrections(link, c0), "B-b")


func _object(link: SliceLink, shape: String) -> String:
	for id in link.objects.nodes:
		if link.objects.shape_of(id) == shape:
			return id
	return ""


func _listed_at(link: SliceLink, id: String) -> Vector3:
	var listing: Variant = link.client.latest.component_value(link.here, "loose-objects")
	if typeof(listing) == TYPE_ARRAY:
		for e in listing:
			if SliceObjects.id_of(e.get("object")) == id:
				var at: Dictionary = e["at"]
				return link.to_scene(link.here, at) + Vector3(0, float(at.get("z", 0)) / 1000.0, 0)
	return Vector3.INF


func _b_box(link: SliceLink, cafe: String) -> void:
	var box := _object(link, "box")
	if box == "":
		_verdict_b("B-c", false, "no box drawn in the café")
		return
	var at0 := _listed_at(link, box)
	await _walk_to(_cafe(link, Vector2(LANE_X, 4000)), 8.0)
	var mm := link.to_world(cafe, at0)
	await _walk_to(_cafe(link, Vector2(float(mm["x"]) - 1300, float(mm["y"]))), 6.0)
	_box_overlap = -INF
	_phase = "c"
	await _press_into(at0, 1.5)
	await _frame("box")
	await _hold(1.0)
	_phase = ""
	var at1 := _listed_at(link, box)
	var drawn := link.objects.drawn_at(box)
	# the separation the server itself states, from the observation alone
	var me := _server_at(link)
	var bm := link.to_world(cafe, at1)
	var half: Vector3 = _disclosed_half(link, box) * 1000.0
	var sdx := maxf(absf(me.x - float(bm["x"])) - half.x, 0.0)
	var sdy := maxf(absf(me.y - float(bm["y"])) - half.z, 0.0)
	print("B-c      the server states the player %s and the box at %s: %.1f mm from its faces (the capsule's radius is %d mm)"
		% [me, bm, Vector2(sdx, sdy).length(), int(CAPSULE_R * 1000.0)])
	_verdict_b("B-c", at1 != at0 and drawn.distance_to(at1) * 1000.0 <= DRAWN_MM and _box_overlap <= 0.005,
		"box disclosed %s -> %s; drawn - disclosed %.2f mm; largest capsule overlap %.1f mm"
		% [at0, at1, drawn.distance_to(at1) * 1000.0, _box_overlap * 1000.0])


## Stand `metres` from the ball on the line from `side`, and aim at it.
## Close to it (within a kick's offer), the sides are tried in turn until the
## world offers something about the ball from where the body stands: the ball
## may have come to rest against furniture. The probe reads the offer only to
## choose where to stand; the request is sent whatever it says.
func _face_ball(link: SliceLink, ball: String, metres: float, side: Vector2) -> Vector3:
	var at := _listed_at(link, ball)
	var sides := [side] if metres > 0.8 else [side, Vector2(0, 1), Vector2(1, 0), Vector2(-1, 0), Vector2(0, -1)]
	for s in sides:
		at = _listed_at(link, ball)
		var mm := link.to_world(link.here, at)
		var spot := Vector2(float(mm["x"]), float(mm["y"])) + (s as Vector2).normalized() * metres * 1000.0
		await _walk_to(_cafe(link, spot), 8.0)
		await _hold(0.4)
		await _aim_at(_listed_at(link, ball))
		var offered := link.client.latest.affordances_about(ball).filter(func(o): return bool(o.get("available", false)))
		if metres > 0.8 or not offered.is_empty():
			return at
		print("aim      no offer about the ball from %s of it; trying another side" % s)
	return at


func _answer_of(link: SliceLink, tok: String) -> Dictionary:
	var t := 0.0
	while t < 4.0 and tok != "" and not _answered(link, tok):
		await _hold(0.05)
		t += 0.05
	for a in link.answers:
		if a["token"] == tok:
			return a
	return {}


static func _code(a: Dictionary) -> String:
	if a.is_empty():
		return "NO ANSWER"
	if a.has("code"):
		return "refused " + String(a["code"])
	var d: Variant = a.get("detail")
	var v: Variant = (d as Dictionary).get(a["result"]) if typeof(d) == TYPE_DICTIONARY else null
	return "%s %s" % [a["result"], JSON.stringify(v) if v != null else ""]


## Wherever the last interaction went out as the world's offer, the built
## request equals it, byte for byte.
func _same_as_offer(link: SliceLink) -> String:
	if link.intents.sent.is_empty():
		return "nothing sent"
	var rec: Dictionary = link.intents.sent[link.intents.sent.size() - 1]
	if not rec["offered"]:
		return "not offered"
	var offer := SliceIntents.matching_offer(link.client.latest, rec)
	var a := JSON.stringify(MineWorldClient._as_sent(offer.get("payload")), "", true)
	var b := JSON.stringify(rec["payload"], "", true)
	return "offered, equal byte for byte" if a == b else "offered, DIFFERS: %s vs %s" % [a, b]


func _ball_follows(link: SliceLink, ball: String, before: Vector3) -> Array:
	var t := 0.0
	while t < 2.0 and _listed_at(link, ball) == before:
		await _hold(0.05)
		t += 0.05
	var started := Time.get_ticks_usec() / 1e6
	while link.objects.moving(ball) and t < 8.0:
		await get_tree().process_frame
		t += get_process_delta_time()
	var secs := Time.get_ticks_usec() / 1e6 - started
	await _hold(0.2)
	var now := _listed_at(link, ball)
	return [now != before, link.objects.drawn_at(ball).distance_to(now) * 1000.0, secs, now]


func _b_kick(link: SliceLink, cafe: String) -> void:
	var ball := _object(link, "ball")
	if ball == "":
		_verdict_b("B-d", false, "no ball drawn in the café")
		return
	await _walk_to(_cafe(link, Vector2(LANE_X, 4000)), 8.0)
	var at := await _face_ball(link, ball, 0.6, Vector2(0, -1))
	var looked := link.aim()
	var tok := link.kick_facing()
	var offer := _same_as_offer(link)
	var a := await _answer_of(link, tok)
	var f := await _ball_follows(link, ball, at)
	if _frames:
		await _frame("kicked")
	var m := link.objects.motions.filter(func(x): return x["id"] == ball)
	var how: String = m[m.size() - 1]["how"] if not m.is_empty() else "-"
	var drawn_s: float = float(m[m.size() - 1].get("ended", -1.0)) - float(m[m.size() - 1]["started"]) \
		if not m.is_empty() else -1.0
	print("C5 ball_s %.4f (%s, %s keyframes, first drawn to last, wall seconds)" % [drawn_s, how,
		m[m.size() - 1]["seconds"] if not m.is_empty() else "-"])
	var near_ok: bool = String(a.get("result", "")) == "accepted" and f[0] and f[1] <= DRAWN_MM
	# from 2 m: not offered, still sent, answered too far away
	var at2 := await _face_ball(link, ball, 2.0, Vector2(0, -1))
	var far := link.kick_facing()
	var b := await _answer_of(link, far)
	var far_ok := far != "" and _code(b).contains("too_far_away")
	_verdict_b("B-d", near_ok and far_ok and not offer.contains("DIFFERS"),
		"aimed at %s; near: %s, %s, ball moved %s, drawn - disclosed %.2f mm (%s, %.2f s); from 2 m: %s"
		% [looked["entity"], _code(a), offer, f[0], f[1], how, f[2], _code(b)])


func _b_throw(link: SliceLink, cafe: String) -> void:
	var ball := _object(link, "ball")
	if ball == "":
		_verdict_b("B-e", false, "no ball drawn in the café")
		return
	# G, G: the default line
	var at := await _face_ball(link, ball, 0.6, Vector2(0, -1))
	link.throw_press()
	var tok := link.throw_press()
	var r1 := _same_as_offer(link)
	var a := await _answer_of(link, tok)
	var f := await _ball_follows(link, ball, at)
	var ok1: bool = String(a.get("result", "")) == "accepted" and link.intents.sent.back()["payload"]["toward"] == null
	# G, aim at the floor 2 m beyond, G
	at = await _face_ball(link, ball, 0.6, Vector2(0, -1))
	link.throw_press()
	var beyond := at + (at - player.global_position).normalized() * 2.0
	beyond.y = link.to_scene(cafe, {"x": 0, "y": 0}).y
	await _aim_at(beyond)
	var hit := SliceTargeting.aim(player.get_viewport())
	tok = link.throw_press()
	var sent: Dictionary = link.intents.sent.back()["payload"]
	var b := await _answer_of(link, tok)
	var want := link.to_world(cafe, hit["point"]) if hit["point"] != null else {}
	var ok2: bool = String(b.get("result", "")) == "accepted" and typeof(sent["toward"]) == TYPE_DICTIONARY \
		and int(sent["toward"]["x"]) == int(want.get("x", -1)) and int(sent["toward"]["y"]) == int(want.get("y", -1))
	await _ball_follows(link, ball, at)
	# G, aim beyond the place's floor (out through the shopfront), G
	at = await _face_ball(link, ball, 0.6, Vector2(0, -1))
	link.throw_press()
	await _aim_at(_cafe(link, Vector2(float(link.to_world(cafe, at)["x"]), -4000)) + Vector3(0, 1.0, 0))
	tok = link.throw_press()
	var sent3: Dictionary = link.intents.sent.back()["payload"]
	var c := await _answer_of(link, tok)
	var ok3: bool = tok != "" and not c.is_empty() and String(c.get("result", "")) != "accepted"
	_verdict_b("B-e", ok1 and ok2 and ok3,
		"G,G: %s (%s, moved %s); aimed at the floor: %s toward %s, aim point %s; beyond the floor: %s toward %s"
		% [_code(a), r1, f[0], _code(b), JSON.stringify(sent["toward"]), want, _code(c),
		JSON.stringify(sent3["toward"])])


func _b_shove(link: SliceLink, cafe: String) -> void:
	var b: MineWorldClient = puppets["bob"]
	# between the round tables, south of the first, then west and north to the spot
	await _walk_to(_cafe(link, Vector2(LANE_X, 4000)), 8.0)
	await _walk_to(_cafe(link, Vector2(LANE_X, 2300)), 8.0)
	await _walk_to(_cafe(link, Vector2(SHOVE_SPOT.x, 2300)), 8.0)
	await _walk_to(_cafe(link, SHOVE_SPOT), 8.0)
	await _report_and_settle(link)
	var ok := await _p_walk(b, cafe, [Vector2(4000, 5000), _server_at(link) + Vector2(600, 0)])
	await _hold(1.0)
	print("B-f      the player at %s, Bob at %s" % [_server_at(link), _p_at(b)])
	var c0 := link.correction.history.size()
	var seq := link.client.sequence
	var tok := SliceIntents.new().shove(b, link.client.observer, MineWorldSpace.location(cafe,
		{"x": int(_p_at(b).x), "y": int(_p_at(b).y), "z": 0}, 270000))
	var t := 0.0
	while t < 4.0 and not _p_answers.has(tok):
		await _hold(0.02)
		t += 0.02
	var answered_seq := link.client.sequence
	var started := Time.get_ticks_usec() / 1e6
	while link.correction.history.size() == c0 and link.client.sequence <= answered_seq + 2 and t < 6.0:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
	var corrected_after := link.client.sequence - answered_seq
	var glide_from := Time.get_ticks_usec() / 1e6
	while player.gliding():
		await get_tree().physics_frame
	var glide_s := Time.get_ticks_usec() / 1e6 - glide_from
	await _frame("shoved")
	await _hold(0.6)
	var cs := _corrections(link, c0)
	var me: Variant = link.client.latest.self_location().get("local")
	var gap := _flat_len(player.global_position - link.to_scene(cafe, me)) * 1000.0
	var one: bool = cs.size() == 1 and float(cs[0]["size"]) <= 1.0
	print("C5 glide_s %.3f" % glide_s)
	if not ok:
		print("B-f      Bob did not reach his spot; the shove is judged as answered")
	_verdict_b("B-f", String(_p_answers.get(tok, "")) == "accepted" and one
		and absf(glide_s - GLIDE_S) <= 1.0 / 60.0 + 0.002 and gap <= BODY_MM
		and corrected_after <= 2,
		"shove %s; %d corrections %s; glide %.3f s; body - observed %.0f mm; corrected on observation %d after the answer"
		% [_p_answers.get(tok, "NO ANSWER"), cs.size(), cs.map(func(c): return "%.3f m %s" % [c["size"], c["form"]]),
		glide_s, gap, corrected_after])
	await _c4_still(link, cs, "B-f")


## C-2b: Bob stands south of the lane; the player jogs east along it; Bob shoves
## her as she passes, while her strides are in flight.
func _c_shoved_walking(link: SliceLink, cafe: String) -> void:
	var b: MineWorldClient = puppets["bob"]
	await _walk_to(_cafe(link, Vector2(1610, JOG_LANE_Y)), 10.0)
	var ok := await _p_walk(b, cafe, [Vector2(3000, 4000), Vector2(3000, SHOVER_AT.y + 1000), SHOVER_AT])
	await _hold(1.0)
	print("C-2b     Bob stands at %s; the player starts at %s" % [_p_at(b), _server_at(link)])
	var closest := INF
	var c0 := link.correction.history.size()
	player.rotation.y = -PI * 0.5
	Input.action_press("jog")
	Input.action_press("move_forward")
	var tok := ""
	var t := 0.0
	while t < 6.0:
		await get_tree().physics_frame
		t += get_physics_process_delta_time()
		var mm := link.to_world(cafe, player.global_position)
		var seen := _server_at(link)
		closest = minf(closest, seen.distance_to(_p_at(b)))
		if tok == "" and seen.distance_to(_p_at(b)) <= 1000.0:
			tok = SliceIntents.new().shove(b, link.client.observer, MineWorldSpace.location(cafe,
				{"x": int(_p_at(b).x), "y": int(_p_at(b).y), "z": 0}, 0))
		if float(mm["x"]) >= 6300.0:
			break
	Input.action_release("move_forward")
	Input.action_release("jog")
	player.velocity = Vector3.ZERO
	print("C-2b     closest the server held the player to Bob while jogging: %.0f mm" % closest)
	await _hold(1.5)
	await _report_and_settle(link)
	var cs := _corrections(link, c0)
	var me: Variant = link.client.latest.self_location().get("local")
	var gap := _flat_len(player.global_position - link.to_scene(cafe, me)) * 1000.0
	_verdict_b("C-2b", ok and String(_p_answers.get(tok, "")) == "accepted" and cs.size() == 1 and gap <= BODY_MM,
		"shove while jogging %s; %d corrections %s; body - observed %.0f mm"
		% [_p_answers.get(tok, "NO ANSWER"), cs.size(),
		cs.map(func(c): return "%.3f m %s" % [c["size"], c["form"]]), gap])


## C-4: after a correction, no input for 2 s: the body moves less than 10 mm.
func _c4_still(link: SliceLink, cs: Array, label: String) -> void:
	if cs.is_empty():
		return
	while player.gliding():
		await get_tree().physics_frame
	var from := player.global_position
	await _hold(2.0)
	var moved := _flat_len(player.global_position - from) * 1000.0
	_verdict_b("C-4", moved < STILL_MM, "%s: still for 2 s after the correction, moved %.1f mm" % [label, moved])


## B-g: every drawn person and object where the final observation puts them.
func _b_end(link: SliceLink) -> void:
	await _hold(3.0)
	await _report_and_settle(link)
	await _hold(1.0)
	var obs := link.client.latest
	var worst := 0.0
	var worst_what := ""
	for id in link.figures:
		var loc := obs.location_of(id)
		var d := _flat_len((link.figures[id] as Node3D).global_position - link.to_scene(link.here, loc.get("local"))) * 1000.0
		if d > worst:
			worst = d
			worst_what = id
	for id in link.objects.nodes:
		var d := link.objects.drawn_at(id).distance_to(_listed_at(link, id)) * 1000.0
		if d > worst:
			worst = d
			worst_what = id
	var me: Variant = obs.self_location().get("local")
	var gap := _flat_len(player.global_position - link.to_scene(link.here, me)) * 1000.0
	_verdict_b("B-g", worst <= DRAWN_MM and gap <= BODY_MM,
		"%d people and %d objects drawn: worst drawn - observed %.2f mm (%s); the player %.0f mm"
		% [link.figures.size(), link.objects.nodes.size(), worst, worst_what, gap])


## C-2 and C-3 over every correction this run drew.
func _c_summary(link: SliceLink) -> void:
	var drawn_log := link.correction.history
	print("corrections %d:" % drawn_log.size())
	var smooth := true
	for c in drawn_log:
		var start: float = c["at"]
		var frames: Array = _track.filter(func(f): return f[0] >= start - 0.001 and f[0] <= start + GLIDE_S + 0.05)
		var worst := 0.0
		for k in range(1, frames.size()):
			if not frames[k][3] and not frames[k - 1][3]:
				continue
			# the frame's displacement less what the input's own velocity moved it
			var step := _flat_len((frames[k][1] - frames[k - 1][1])
				- (frames[k][2] as Vector3) * (1.0 / Engine.physics_ticks_per_second))
			worst = maxf(worst, step)
		var allowed := float(c["size"]) / maxf(GLIDE_S * 60.0, 1.0) + STEP_SLACK_MM / 1000.0
		var snap := bool(c.get("snap", false))
		var idle := frames.all(func(f): return _flat_len(f[2]) < 0.01)
		if not snap and idle and worst > allowed:
			smooth = false
		print("  %.3f m %s %s form at %.2f s, C-1 kind %s; largest glide step %.1f mm (bound %.1f)%s" % [c["size"],
			"SNAP" if snap else "glide", c["form"], start, _c1_kind(link, c), worst * 1000.0, allowed * 1000.0,
			"" if idle else "; input live, the walk's own motion in it: reported, not judged"])
	_verdict_b("C-3", smooth, "every glide drawn with the input idle moves the body evenly; snaps listed above")


## C-1: a free walk and jog on the pavement, nobody in the way: no correction.
func _c_free_walk(link: SliceLink) -> void:
	var lay := link.layout
	await _walk_to(_cafe(link, Vector2(1610, 1500)), 10.0)
	# counted from inside the café: the walk out through its door is one crossing
	var c0 := link.correction.history.size()
	var x0 := link.place_changes.size()
	await _walk_to_x(_anchor_sill().x)
	player.rotation.y = PI
	await _walk_dist(3.2, 6.0)
	await _until_place(link, lay.street)
	var t0 := Time.get_ticks_msec()
	var legs := 0
	while Time.get_ticks_msec() - t0 < 60000:
		if legs % 2 == 0:
			Input.action_press("jog")
		else:
			Input.action_release("jog")
		player.rotation.y = -PI * 0.5 if legs % 4 < 2 else PI * 0.5
		await _walk_dist(6.0, 6.0)
		legs += 1
	Input.action_release("jog")
	var cs := link.correction.history.slice(c0)
	var crossings := link.place_changes.size() - x0
	var kinds := {"doorway": 0, "corner": 0, "other": 0}
	for c in cs:
		kinds[_c1_kind(link, c)] += 1
	_verdict_b("C-1", link.here == lay.street and kinds["other"] == 0 and kinds["doorway"] <= 2 * crossings,
		"the walk out through the café's door and 60 s on the pavement (%d legs): %d corrections — %d at a doorway (allowed: at most 2 for each of %d crossings), %d at a convex corner, %d of no allowed kind"
		% [legs, cs.size(), kinds["doorway"], crossings, kinds["corner"], kinds["other"]])


## C-1 AS AMENDED (the operator's ruling of 2026-10-17, step-11 §22.4): the allowed corrections.
## A correction is one of:
##   "doorway"  the world stopped the stride at a wall ("by" nobody), the server's position lies
##              within DOOR_NEAR_MM of a doorway point of that place, and it is a small one, at
##              most SMALL_M: bodies' edge band at a crossing
##   "corner"   the world stopped the stride at a wall or a solid, and the stride started next to
##              a disclosed solid's corner -- the nearest point of some solid is one of its corners,
##              within CORNER_NEAR_MM: the server's controller stopping at a convex corner
##   "other"    anything else, which still fails C-1
## The bounds are the ruling's words and the plan's DOOR_EDGE, never a server rule number.
const DOOR_NEAR_MM := 500.0
const SMALL_M := 0.30
const CORNER_NEAR_MM := 500.0

var _shapes := {}


func _c1_kind(link: SliceLink, c: Dictionary) -> String:
	var stop: Dictionary = c.get("stop", {})
	if stop.is_empty() or String(stop["by"]) != "":
		return "other"
	var place := String(c["place"])
	var to: Vector2 = c["to"]
	if float(c["size"]) <= SMALL_M:
		for p in link.layout.passages.get(place, []):
			var here := Vector2(float(p["here"]["x"]), float(p["here"]["y"]))
			if to.distance_to(here) <= DOOR_NEAR_MM:
				return "doorway"
	var start: Vector2 = stop["start"]
	var shape: Variant = _shapes.get(place)
	if start != Vector2.INF and typeof(shape) == TYPE_DICTIONARY:
		for sd in shape.get("solids", []):
			var lo := Vector2(float(sd["min"]["x"]), float(sd["min"]["y"]))
			var hi := Vector2(float(sd["max"]["x"]), float(sd["max"]["y"]))
			var q := Vector2(clampf(start.x, lo.x, hi.x), clampf(start.y, lo.y, hi.y))
			var at_corner := (q.x == lo.x or q.x == hi.x) and (q.y == lo.y or q.y == hi.y) \
				and q.x != start.x and q.y != start.y
			if at_corner and start.distance_to(q) <= CORNER_NEAR_MM:
				return "corner"
	return "other"


# --- --rules ------------------------------------------------------------------------

## X-1: the scripted interactions, each request printed as the client sent it.
func _rules_check() -> void:
	print("== step-11 §22.4 X-1 -- the same requests, with and without bodies ==\n")
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	if not await _seated(link):
		return
	var given := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--rules-object="):
			given = a.substr(15)
	link.client.submitted_request.connect(func(_tok: String, r: Dictionary) -> void:
		if String(r["action_type"]) != SliceLink.MOVE_ACTION:
			print("REQUEST %s" % JSON.stringify({"action": r["action_type"], "target": r["target"],
				"payload": r["payload"]}, "", true)))
	await _hold(0.8)
	var cafe := link.layout.anchor
	var barista := ""
	for id in link.client.latest.tagged("barista"):
		barista = id
	var bob := ""
	for id in link.client.latest.tagged("regular"):
		bob = id
	print("barista %s, bob %s" % [barista, bob])
	var a := await _talk_to(link, barista)
	print("ANSWER talk from the door: %s" % a[0])
	await _walk_to(Vector3(8.16, 0.0, -10.20), 6.0)
	var bp: Vector3 = (link.figures[barista] as Node3D).global_position
	await _walk_to(Vector3(bp.x, 0.0, bp.z + 1.85), 6.0)
	await _hold(0.8)
	a = await _talk_to(link, barista)
	print("ANSWER talk at the counter: %s" % a[0])
	# shove Bob, from beside him
	if bob != "" and link.figures.has(bob):
		var at: Vector3 = (link.figures[bob] as Node3D).global_position
		await _walk_to(at + Vector3(-0.7, 0, 0), 6.0)
		await _aim_at(at + Vector3.UP * HEAD_Y)
		print("ANSWER shove: %s" % _code(await _answer_of(link, link.shove_facing())))
	# kick and throw the café ball: aimed when the world draws it, else the
	# object the first run named (a world without bodies discloses none)
	var ball := _object(link, "ball")
	print("RULES-OBJECT %s" % (JSON.stringify(link.objects.ref_of(ball)) if ball != "" else given))
	await _walk_to(_cafe(link, Vector2(LANE_X, 4000)), 8.0)
	await _walk_to(_cafe(link, Vector2(2600, 5000)), 8.0)
	if ball != "":
		# from 2 m first: out of reach, not offered, still sent (tightened after M-E6 survived X-1,
		# §22.15 X12e-19)
		await _walk_to(_cafe(link, Vector2(2600, 3600)), 8.0)
		await _aim_at(_listed_at(link, ball))
		print("ANSWER kick from 2 m: %s" % _code(await _answer_of(link, link.kick_facing())))
		await _walk_to(_cafe(link, Vector2(2600, 5000)), 8.0)
		await _aim_at(_listed_at(link, ball))
		print("ANSWER kick: %s" % _code(await _answer_of(link, link.kick_facing())))
		await _hold(1.5)
		await _face_ball(link, ball, 0.6, Vector2(0, -1))
		link.throw_press()
		print("ANSWER throw: %s" % _code(await _answer_of(link, link.throw_press())))
	elif given != "":
		await _walk_to(_cafe(link, Vector2(2600, 3600)), 8.0)
		print("ANSWER kick from 2 m: %s" % _code(await _answer_of(link, link.intents.kick(link.client,
			JSON.parse_string(given), link._here_location()))))
		await _walk_to(_cafe(link, Vector2(2600, 5000)), 8.0)
		print("ANSWER kick: %s" % _code(await _answer_of(link, link.intents.kick(link.client, JSON.parse_string(given),
			link._here_location()))))
		print("ANSWER throw: %s" % _code(await _answer_of(link, link.intents.throw(link.client, JSON.parse_string(given), null,
			link._here_location()))))
	print("\nrules run done")
	link.client.disconnect_from_world("probe done")
	await _hold(0.2)
