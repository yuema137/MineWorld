## `--slice-seat` (S33-f AC-f-2, AC-f-3; step-33 AC-S33-3, X-4): every sit point of the world's sit-point
## depiction file, with every body this client can draw, seats the body's own seat on the DRAWN seat
## within SEAT_MM.
##
## Connected, like `--slice-link`: the probe walks the depiction's probe route on the real controller,
## and in each place it reaches, once the place's furnishings are drawn from the disclosure, for every
## point the depiction file binds to the place's disclosed tags:
##   - the drawn seat: a vertical ray at the pelvis' plan position (the point's `at` moved by
##     `seat_offset_m` against its facing, `SliceSeating.pelvis`) against the triangles of the drawn
##     furnishings, the highest hit no higher than `seat_height_m` + WINDOW above the floor;
##   - each body: built as the client draws a person (`NPC.make_body`), stood at the pelvis, seated with
##     `Human.sit_on(seat_height_m)` (the depiction's placement, the one every seated person gets), and
##     its lowest seat vertex measured (`SeatContact.lowest`);
##   - the row: the contact's height against the drawn seat's, within SEAT_MM (G-13's 20 mm).
## A character whose body this checkout cannot build (no manifest, a Git LFS pointer) is reported
## `SKIP <character> no body`, never as a pass. Verdict: `all seat checks pass` or
## `<n> SEAT CHECKS FAILED`, each failure naming the point and the body. The bodies come from the pack's
## cast binding (its looks, its extras, its default) and the town figure.
class_name SliceProbeSeat
extends SliceProbeWorld

const SEAT_MODES := ["seat"]
## G-13's bound (S26 §6.1), the plan's, never widened after a measurement.
const SEAT_MM := 20.0
## How far above `seat_height_m` the ray still takes a surface for the seat, metres: room for X-4's
## +0.05 plant, under any backrest or table top over the pelvis.
const WINDOW := 0.15
## A seat lower than this above the floor is the floor or a stretcher, not a seat, metres.
const LOWEST_SEAT := 0.2

var _rows := 0
var _fails := 0


static func requested() -> bool:
	return requested_of(SEAT_MODES) != ""


func _modes() -> Array:
	return SEAT_MODES


func _run(mode: String) -> void:
	match mode:
		"seat": await _seat_check()
		_: await super(mode)


func _seat_check() -> void:
	print("== S33-f -- every sit point holds every body within %d mm of the drawn seat ==\n" % int(SEAT_MM))
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	var waited := 0.0
	while waited < 10.0 and (not link.client.is_seated() or link.here == "" or link._reconcile):
		await get_tree().process_frame
		waited += get_process_delta_time()
	if not link.client.is_seated():
		print("FAIL: not seated after 10 s")
		return
	await _hold(0.8)
	var rule := link.seating.rule
	print("depictions %d points from %s%s" % [rule.points.size(), rule.path,
		"" if rule.problem == "" else " (%s)" % rule.problem])
	if rule.points.is_empty():
		_fails += 1
		print("FAIL: the world has no sit-point depictions")
	var bodies := _bodies(link.cast)
	var done := {}
	var route: Array = SliceWorlds.current.viewpoints().get("probe_route", [])
	for step in route:
		var target := _place_of_entry(link.layout, _step_tags(step))
		if target == "":
			continue
		if target != link.here and not await _cross(link, target):
			_fails += 1
			print("FAIL: the server did not move the player into %s" % target)
			break
		if done.has(link.here):
			continue
		done[link.here] = true
		await _hold(1.0)
		await _place_rows(link, bodies)
	var unvisited := []
	for p in rule.points:
		if not p.has("_seen_in"):
			unvisited.append(p["point"])
	if not unvisited.is_empty():
		_fails += 1
		print("FAIL: points in no place the route reached: %s" % ", ".join(PackedStringArray(unvisited)))
	print("\nrows     %d" % _rows)
	print("%s" % ("all seat checks pass" if _fails == 0 else "%d SEAT CHECKS FAILED" % _fails))
	link.client.disconnect_from_world("probe done")
	await _hold(0.2)


## The bodies to seat: [[name, spec]] -- every character the cast binding can resolve, and the town
## figure; a character with no buildable body is reported and left out.
func _bodies(cast: SliceCast) -> Array:
	var names := {}
	if cast != null:
		for look in cast.looks:
			names[String(cast.looks[look])] = true
		for e in cast.extras:
			names[String(e)] = true
		if cast.default_character != "":
			names[cast.default_character] = true
	var out := [["town-figure", {}]]
	var keys := names.keys()
	keys.sort()
	for c in keys:
		var spec: Dictionary = cast.body_for(c)
		if spec.is_empty():
			print("SKIP %s no body" % c)
			continue
		out.append([c, spec])
	print("bodies   %s" % ", ".join(PackedStringArray(out.map(func(b): return b[0]))))
	return out


## Every row of the place the body now stands in.
func _place_rows(link: SliceLink, bodies: Array) -> void:
	var obs := link.client.latest
	var tags := MineWorldSitRule.tags_of(obs)
	var furnished := link.furnishings.nodes.get(link.here) as Node3D
	for p in link.seating.rule.points:
		if not MineWorldSitRule.binds(p, tags):
			continue
		p["_seen_in"] = link.here
		var pose := link.seating.pose_of(tags, obs.component_value(link.here, "sit-points"), p["point"])
		if pose.is_empty():
			_fails += 1
			print("FAIL: %s has no position" % p["point"])
			continue
		var at := link.layout.to_scene(link.here, {"x": pose["pelvis"].x, "y": pose["pelvis"].y})
		var height: float = pose["seat_height_m"]
		var top := _drawn_seat(furnished, at, height)
		if top == INF:
			_fails += 1
			print("FAIL: %s: no drawn seat under the pelvis at %s (prop %s)" % [p["point"], at, pose["prop"]])
			continue
		print("seat     %s: drawn seat %.3f m above the floor, depiction %.3f m" % [p["point"], top - at.y,
			height])
		var yaw := MineWorldSpace.yaw_to_3d_radians({"yaw": pose["facing"]}) + PI
		for b in bodies:
			await _row(String(p["point"]), b[0], b[1], at, yaw, height, top)


## One body seated at one point, measured.
func _row(point: String, name: String, spec: Dictionary, at: Vector3, yaw: float, height: float,
		top: float) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = hash(name)
	var n := NPC.make_body(rng, NPC.Pose.STAND, 1.72, spec)
	slice.link.world_root.add_child(n)
	n.global_position = at
	n.rotation.y = yaw
	await n.body.sit_on(height)
	await get_tree().process_frame
	var low := SeatContact.lowest(n.body)
	_rows += 1
	if low == INF:
		_fails += 1
		print("FAIL: %s with %s: no seat vertex measured" % [point, name])
	else:
		var err := (low - top) * 1000.0
		var ok := absf(err) <= SEAT_MM
		if not ok:
			_fails += 1
		print("%s %s with %s: contact %.3f m, drawn seat %.3f m, %+.1f mm" % ["ok      " if ok else "FAIL:",
			point, name, low - at.y, top - at.y, err])
	n.queue_free()
	await get_tree().process_frame


## The drawn seat's height (world y) under `at`: the highest triangle of the drawn furnishings a
## vertical ray at `at` meets between LOWEST_SEAT and `height` + WINDOW above the floor; INF for none.
static func _drawn_seat(furnished: Node3D, at: Vector3, height: float) -> float:
	if furnished == null:
		return INF
	var best := INF
	var from := Vector3(at.x, at.y + height + WINDOW, at.z)
	for mi: MeshInstance3D in furnished.find_children("*", "MeshInstance3D", true, false):
		if mi.mesh == null:
			continue
		var faces := mi.mesh.get_faces()
		var xf := mi.global_transform
		for k in range(0, faces.size(), 3):
			var hit: Variant = Geometry3D.ray_intersects_triangle(from, Vector3.DOWN, xf * faces[k],
				xf * faces[k + 1], xf * faces[k + 2])
			if hit == null:
				continue
			var y := (hit as Vector3).y
			if y - at.y < LOWEST_SEAT:
				continue
			if best == INF or y > best:
				best = y
	return best
