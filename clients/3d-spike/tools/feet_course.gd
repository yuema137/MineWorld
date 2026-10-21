## The foot-grounding probe (PR S36-P4 commit 3, `.structured-coding/plans/mvp1/pr-s36-p4-foot-grounding.md`).
##
##     godot --headless --path clients/3d-spike --fixed-fps 30 -s tools/feet_course.gd -- \
##         --bodies <json> --seed <n> --out <dir> [--no-ik] [--scene course|street]
##     godot --headless --path clients/3d-spike --fixed-fps 30 -s tools/feet_course.gd -- --self-test --out <dir>
##
## `--bodies` is `mwcast/ground.py bodies()`'s listing (`cast.py feet --run` writes it): every body the client
## draws, built exactly as the client builds it. Each body walks a **course** drawn from the seed -- a flat
## stretch, a kerb up and down, a ramp up and down, three stairs up and down, an uneven patch -- at its own
## measured walk speed, its root at the ground under its centre (as a terrain-following figure stands), and then
## stands still for 2 s straddling every kerb and stair edge. `--scene street` builds the slice's street
## (`SliceStreet`, the shipped scene of `social-cafe` and `market-town`) instead, finds its kerb edges by rays (no
## coordinate literal), stands every body across three of them and walks the first body over the kerb and along it.
##
## Every frame, read in `skeleton_updated` (after every modifier): per foot, the least clearance of its support
## points above the drawn surface under each of them (a ray per point), whether it is planted (G-8's drift rule,
## the same one `FootGrounding` uses), and whether its ground is out of reach. Written to `<out>/feet.json`, which
## `cast.py feet` judges against the tolerance file. Every surface is a `Build.LAYER_WORLD` collider, so the
## grounding's rays meet what a scene would have; the clearance rays meet the drawn meshes themselves (a
## judge-only layer of trimesh colliders made from every `MeshInstance3D` of the scene), so an invisible collider
## cannot make a floating foot look grounded.
##
## `--self-test` runs the commit-1 and commit-2 checks and prints `all feet checks pass` or `N FEET CHECKS FAILED`.
extends SceneTree

const JUDGE_LAYER := 1 << 19
const FPS := 30.0
const STRADDLE_S := 2.0
const SETTLE_FRAMES := 20
## The course's plan view: it runs along +z, `HALF_W` either side of x = 0.
const HALF_W := 2.0
## A stair tread: at least 11 in (279 mm, International Building Code section 1011.5.2); 0.30 m.
const TREAD_M := 0.30
## The uneven patch: heights on a 0.25 m grid, at most this far from its mean (design commit 3).
const GRID_M := 0.25
const BUMP_M := 0.030
## Straddling an edge, the lower foot's inner edge stands this far from it, so the `Stand` loop's sway (about a
## centimetre) never carries it over the edge; a narrow stance then leaves the upper foot overhanging the edge.
const STRADDLE_CLEAR_M := 0.03

var _args := {}
var _stage: Node3D
var _rng := RandomNumberGenerator.new()
## [z0, z1, name] for every walked segment of the course, in order
var _segments: Array = []
## [z, yaw-facing note, name] for each edge to straddle
var _edges: Array = []
var _rows: Array = []
var _cur: Dictionary = {}
var _segment := ""
var _frame := 0
var _body_speed := 0.0
var _last_foot := {}
var _fails: Array[String] = []


func _initialize() -> void:
	var raw := OS.get_cmdline_user_args()
	var i := 0
	while i < raw.size():
		var key: String = raw[i]
		if key.begins_with("--") and i + 1 < raw.size() and not (raw[i + 1] as String).begins_with("--"):
			_args[key.substr(2)] = raw[i + 1]
			i += 2
		else:
			_args[key.trim_prefix("--")] = true
			i += 1
	_stage = Node3D.new()
	_stage.name = "FeetCourse"
	root.add_child(_stage)
	_run.call_deferred()


func _run() -> void:
	var out: String = _args.get("out", "")
	if out == "":
		printerr("feet_course: --out is required")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(out)
	if _args.has("self-test"):
		await _self_test(out)
		return
	if not _args.has("bodies"):
		printerr("feet_course: --bodies is required")
		quit(2)
		return
	var listing: Variant = JSON.parse_string(FileAccess.get_file_as_string(String(_args["bodies"])))
	if typeof(listing) != TYPE_ARRAY:
		printerr("feet_course: --bodies is not a JSON array")
		quit(2)
		return
	var seed_n := int(_args.get("seed", "1"))
	_rng.seed = seed_n
	var scene: String = _args.get("scene", "course")
	Human.foot_grounding = not _args.has("no-ik")
	var course := {}
	if scene == "street":
		SliceStreet.build(_stage)
	else:
		course = _build_course()
	_judge_layer(_stage)
	# the colliders join the physics space on the next physics step
	await _frames_n(2)
	if scene == "street":
		course = _find_kerbs()
		if (course["stations"] as Array).is_empty():
			printerr("feet_course: the street has no kerb edge the rays can find")
			quit(1)
			return
	var bodies: Array = []
	var first := true
	for entry: Dictionary in listing:
		var h := _build_body(entry)
		_rows = []
		_cur = {}
		_last_foot = {}
		h.skeleton.skeleton_updated.connect(_on_updated.bind(h))
		if scene == "street":
			await _street_run(h, course, first)
		else:
			await _course_run(h, course)
		h.skeleton.skeleton_updated.disconnect(_on_updated)
		bodies.append({"id": entry["id"], "frames": _rows})
		print("feet_course: %s %d frames" % [entry["id"], _rows.size()])
		h.queue_free()
		await _frames_n(2)
		first = false
	var report := {"format": 1, "seed": seed_n, "ik": Human.foot_grounding, "scene": scene, "fps": FPS,
		"tunables": FootGrounding.tunables(), "course": course.get("plan", {}), "bodies": bodies}
	var f := FileAccess.open(out.path_join("feet.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(report, "  "))
	f.close()
	print("feet_course: wrote %s" % out.path_join("feet.json"))
	quit(0)


# --- bodies ---------------------------------------------------------------------------------------------------------

func _build_body(entry: Dictionary) -> Human:
	var h: Human
	if entry.get("reference", false):
		h = Human.build(Human.CANONICAL_HEIGHT, Color(0.86, 0.70, 0.60), Color(0.30, 0.20, 0.13),
			Color(0.55, 0.20, 0.18), Color(0.22, 0.27, 0.38), Color(0.14, 0.13, 0.12),
			Color(0.36, 0.38, 0.33, 1.0), Color(0.24, 0.25, 0.21, 1.0), Human.Body.REFERENCE)
	else:
		var m: Variant = JSON.parse_string(FileAccess.get_file_as_string(String(entry["manifest"])))
		h = Human.build_character(m)
	h.blink_hold = 0.0
	_stage.add_child(h)
	return h


func _walk_speed(h: Human) -> float:
	return h._walk_mps * h.scale.y


# --- the course -----------------------------------------------------------------------------------------------------

## The seeded course. Every surface is a `LAYER_WORLD` box (or, for the uneven patch, a heightfield) and a drawn
## mesh, so the grounding's rays and the judge's rays meet the same geometry.
func _build_course() -> Dictionary:
	var kerb := _rng.randf_range(0.10, 0.15)
	var slope := deg_to_rad(_rng.randf_range(2.0, 10.0))
	var riser := _rng.randf_range(0.15, 0.178)
	var z := 0.0
	_flat(z, z + 3.0, 0.0)
	_segments.append([z, z + 3.0, "flat"])
	z += 3.0
	# a kerb up, a pavement, a kerb down
	_flat(z, z + 2.5, kerb)
	_edges.append([z, "kerb_up", 1.0])
	_segments.append([z - 0.6, z + 0.6, "kerb_up"])
	_segments.append([z + 0.6, z + 1.9, "pavement"])
	_edges.append([z + 2.5, "kerb_down", -1.0])
	_segments.append([z + 1.9, z + 3.1, "kerb_down"])
	z += 2.5
	_flat(z, z + 2.0, 0.0)
	_segments.append([z + 0.6, z + 2.0, "flat_2"])
	z += 2.0
	# a ramp up, a top, a ramp down
	var run := 3.0
	var top := run * tan(slope)
	_ramp(z, z + run, 0.0, top)
	_segments.append([z, z + run, "ramp_up"])
	z += run
	_flat(z, z + 1.0, top)
	_segments.append([z, z + 1.0, "ramp_top"])
	z += 1.0
	_ramp(z, z + run, top, 0.0)
	_segments.append([z, z + run, "ramp_down"])
	z += run
	_flat(z, z + 1.5, 0.0)
	_segments.append([z, z + 1.5, "flat_3"])
	z += 1.5
	# three stairs up, a landing, three down
	var z_stairs := z
	for k in 3:
		_flat(z, z + TREAD_M, riser * (k + 1))
		_edges.append([z, "stair_up_%d" % (k + 1), 1.0])
		z += TREAD_M
	_flat(z, z + 1.2, riser * 3.0)
	z += 1.2
	for k in 3:
		_edges.append([z, "stair_down_%d" % (k + 1), -1.0])
		_flat(z, z + TREAD_M, riser * (2 - k))
		z += TREAD_M
	_segments.append([z_stairs, z, "stairs"])
	_flat(z, z + 1.5, 0.0)
	_segments.append([z, z + 1.5, "flat_4"])
	z += 1.5
	# the uneven patch
	_uneven(z, z + 4.0)
	_segments.append([z, z + 4.0, "uneven"])
	z += 4.0
	_flat(z, z + 2.0, 0.0)
	_segments.append([z, z + 2.0, "flat_5"])
	z += 2.0
	return {"length": z, "plan": {"kerb_m": kerb, "ramp_deg": rad_to_deg(slope), "riser_m": riser,
		"tread_m": TREAD_M, "grid_m": GRID_M, "bump_m": BUMP_M, "segments": _segments, "edges": _edges}}


func _flat(z0: float, z1: float, top: float) -> void:
	# a slab from 0.4 m under the lowest surface to `top`
	Build.slab(_stage, 0.0, (z0 + z1) * 0.5, HALF_W * 2.0, z1 - z0, -0.4, top + 0.4, null, 0.0, true)


func _ramp(z0: float, z1: float, y0: float, y1: float) -> void:
	var length := Vector2(z1 - z0, y1 - y0).length()
	var tilt := atan2(y1 - y0, z1 - z0)
	var thick := 0.2
	var centre := Vector3(0.0, (y0 + y1) * 0.5, (z0 + z1) * 0.5)
	var b := Basis(Vector3.RIGHT, -tilt)
	centre -= b * Vector3(0, thick * 0.5, 0)
	var size := Vector3(HALF_W * 2.0, thick, length)
	var mi := MeshInstance3D.new()
	var bm := BoxMesh.new()
	bm.size = size
	mi.mesh = bm
	mi.transform = Transform3D(b, centre)
	_stage.add_child(mi)
	var sb := StaticBody3D.new()
	sb.collision_layer = Build.LAYER_WORLD
	var cs := CollisionShape3D.new()
	var sh := BoxShape3D.new()
	sh.size = size
	cs.shape = sh
	sb.add_child(cs)
	sb.transform = mi.transform
	_stage.add_child(sb)
	# a slab under it, so nothing falls through at the ends
	Build.slab(_stage, 0.0, (z0 + z1) * 0.5, HALF_W * 2.0, z1 - z0, -0.4, 0.4 + minf(y0, y1) - 0.01, null, 0.0, true)


## A smooth surface through a seeded height grid (bilinear between grid points), drawn and collided as one trimesh.
func _uneven(z0: float, z1: float) -> void:
	var nx := int(round(HALF_W * 2.0 / GRID_M)) + 1
	var nz := int(round((z1 - z0) / GRID_M)) + 1
	var hgt: Array = []
	for j in nz:
		var row: Array = []
		for i in nx:
			var edge := j == 0 or j == nz - 1
			row.append(0.0 if edge else _rng.randf_range(-BUMP_M, BUMP_M))
		hgt.append(row)
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for j in nz - 1:
		for i in nx - 1:
			var p := func(a: int, b: int) -> Vector3:
				return Vector3(-HALF_W + a * GRID_M, hgt[b][a], z0 + b * GRID_M)
			var a0: Vector3 = p.call(i, j)
			var a1: Vector3 = p.call(i + 1, j)
			var a2: Vector3 = p.call(i, j + 1)
			var a3: Vector3 = p.call(i + 1, j + 1)
			for v in [a0, a1, a2, a1, a3, a2]:
				st.add_vertex(v)
	st.generate_normals()
	var mesh := st.commit()
	var mi := MeshInstance3D.new()
	mi.mesh = mesh
	_stage.add_child(mi)
	var sb := StaticBody3D.new()
	sb.collision_layer = Build.LAYER_WORLD
	var cs := CollisionShape3D.new()
	var shape := mesh.create_trimesh_shape()
	shape.backface_collision = true
	cs.shape = shape
	sb.add_child(cs)
	_stage.add_child(sb)
	Build.slab(_stage, 0.0, (z0 + z1) * 0.5, HALF_W * 2.0, z1 - z0, -0.4, 0.4 - BUMP_M - 0.02, null, 0.0, true)


## The judge's own colliders: one trimesh per drawn mesh, on a layer nothing else masks (see the header).
func _judge_layer(under: Node) -> void:
	for mi: MeshInstance3D in under.find_children("*", "MeshInstance3D", true, false):
		if mi.mesh == null or not mi.is_visible_in_tree():
			continue
		if mi.get_parent() is Skeleton3D:
			continue
		var shape := mi.mesh.create_trimesh_shape()
		if shape == null:
			continue
		shape.backface_collision = true
		var sb := StaticBody3D.new()
		sb.collision_layer = JUDGE_LAYER
		sb.collision_mask = 0
		var cs := CollisionShape3D.new()
		cs.shape = shape
		sb.add_child(cs)
		_stage.add_child(sb)
		sb.global_transform = mi.global_transform


func _ground_at(p: Vector3, mask: int, up := 0.5, down := 1.0) -> Variant:
	var q := PhysicsRayQueryParameters3D.create(p + Vector3.UP * up, p - Vector3.UP * down, mask)
	var hit := root.get_world_3d().direct_space_state.intersect_ray(q)
	return null if hit.is_empty() else (hit["position"] as Vector3).y


func _segment_at(z: float) -> String:
	for s: Array in _segments:
		if z >= float(s[0]) and z < float(s[1]):
			return String(s[2])
	return "between"


## Walk the course along +z at the body's walk speed, root on the ground under its centre; then straddle each edge.
func _course_run(h: Human, course: Dictionary) -> void:
	var speed := _walk_speed(h)
	var z := -1.0
	h.rotation = Vector3.ZERO
	await _stand_at(h, Vector3(0, 0, z), 0.0, SETTLE_FRAMES, "")
	h.set_gait(speed)
	_body_speed = speed
	var length: float = course["length"]
	while z < length - 0.5:
		z += speed / FPS
		_put(h, Vector3(0.0, 0.0, z))
		_segment = _segment_at(z)
		await process_frame
	h.set_gait(0.0)
	_body_speed = 0.0
	# straddling: the body faces +x, so its feet are either side of the edge line (z = const)
	for e: Array in _edges:
		await _straddle(h, Vector3(0.0, 0.0, float(e[0])), Vector3(0, 0, float(e[2])), int(STRADDLE_S * FPS),
			"straddle_" + String(e[1]))
	# a jog over the first flat stretch: at a jog the clip leaves the floor on purpose, and its planted frames (if any)
	# are judged like the walk's (S36-P2's finding: the jog's contact point drifts at about twice the gait speed)
	var jog := h._jog_mps * h.scale.y
	var zj := -1.0
	h.rotation = Vector3.ZERO
	await _stand_at(h, Vector3(0, 0, zj), 0.0, 0, "")
	h.set_gait(jog)
	_body_speed = jog
	_segment = "jog_flat"
	while zj < 2.5:
		zj += jog / FPS
		_put(h, Vector3(0.0, 0.0, zj))
		await process_frame
	h.set_gait(0.0)
	_body_speed = 0.0
	_segment = ""


func _put(h: Human, at: Vector3) -> void:
	var y: Variant = _ground_at(Vector3(at.x, 1.0, at.z), Build.LAYER_WORLD, 0.0, 3.0)
	h.global_position = Vector3(at.x, float(y) if y != null else 0.0, at.z)


func _stand_at(h: Human, at: Vector3, yaw: float, frames: int, segment: String) -> void:
	h.rotation = Vector3(0, yaw, 0)
	_put(h, at)
	_last_foot = {}
	_segment = ""
	await _frames_n(SETTLE_FRAMES)
	_segment = segment
	await _frames_n(frames if segment != "" else 0)
	_segment = ""


## Stand `h` across the edge through `at` whose higher side is `up_dir` (horizontal), as a person stands at a kerb:
## one foot on each side. From the body's own support points (no per-body number), the yaw and the offset are the
## ones that leave the widest gap between its two feet across the edge, with the edge in the middle of the gap.
func _straddle(h: Human, at: Vector3, up_dir: Vector3, frames: int, segment: String) -> void:
	await _stand_at(h, at - up_dir * 1.0, 0.0, 0, "")
	var pts: Dictionary = h._ground.last_points
	var best := -INF
	var best_d := Vector3.BACK
	var best_hi := 0.0
	# people stand side by side at a kerb: the separating direction is within 45 degrees of the body's left (+x)
	for k in 19:
		var d := Vector3.RIGHT.rotated(Vector3.UP, deg_to_rad(5.0 * k - 45.0))
		var lo := INF
		var hi := -INF
		for p: Vector3 in (pts["left"] as PackedVector3Array):
			lo = minf(lo, Vector3(p.x, 0, p.z).dot(d))
		for p: Vector3 in (pts["right"] as PackedVector3Array):
			hi = maxf(hi, Vector3(p.x, 0, p.z).dot(d))
		if lo - hi > best:
			best = lo - hi
			best_d = d
			best_hi = hi
	# the yaw that turns the body's direction `best_d` (left foot ahead along it) onto `up_dir`
	var yaw := atan2(best_d.cross(up_dir).y, best_d.dot(up_dir))
	h.rotation = Vector3(0, yaw, 0)
	# the edge at the inner extreme of the lower foot: it stands wholly on the lower side, and the upper foot
	# rests on the higher one (overhanging the edge if the stance is narrower than the feet)
	var origin := at - up_dir * (best_hi * h.scale.y + STRADDLE_CLEAR_M)
	h.global_position = Vector3(origin.x, h.global_position.y, origin.z)
	_put(h, Vector3(origin.x, 0.0, origin.z))
	# the root stands on the lower side's ground, as a figure following the terrain under its lower foot would
	var lower := Vector3.ZERO
	for p: Vector3 in (pts["right"] as PackedVector3Array):
		lower += p
	lower = h.global_transform * (lower / maxf((pts["right"] as PackedVector3Array).size(), 1.0))
	var low_y: Variant = _ground_at(Vector3(lower.x, 1.0, lower.z), Build.LAYER_WORLD, 0.0, 3.0)
	if low_y != null:
		h.global_position.y = float(low_y)
	_last_foot = {}
	_segment = ""
	await _frames_n(SETTLE_FRAMES)
	_segment = segment
	await _frames_n(frames)
	_segment = ""


# --- the street -----------------------------------------------------------------------------------------------------

## The slice's street (built before the judge layer). Its kerb edges are found by rays across the pavement line at three stations along the
## street, from the street's own extent (no coordinate literal).
func _find_kerbs() -> Dictionary:
	var x0 := SliceStreet.X_MIN
	var x1 := SliceStreet.X_MAX
	var stations: Array = []
	for frac in [0.3, 0.5, 0.7]:
		var x: float = lerpf(x0, x1, frac)
		var top: Variant = _ground_at(Vector3(x, 2.0, SliceStreet.NORTH_FACE + 0.5), Build.LAYER_WORLD, 0.0, 4.0)
		var road: Variant = _ground_at(Vector3(x, 2.0, 0.0), Build.LAYER_WORLD, 0.0, 4.0)
		if top == null or road == null:
			continue
		# from the centreline toward the north frontage: the drawn surface's step from road to pavement
		var z := 0.0
		var edge := INF
		while z > SliceStreet.NORTH_FACE:
			var y: Variant = _ground_at(Vector3(x, 2.0, z), JUDGE_LAYER, 0.0, 4.0)
			if y != null and float(y) > (float(road) + float(top)) * 0.5:
				edge = z
				break
			z -= 0.01
		if edge < INF:
			stations.append({"x": x, "edge_z": edge, "road_y": road, "walk_y": top})
	return {"stations": stations, "plan": {"stations": stations}}


func _street_run(h: Human, course: Dictionary, walker: bool) -> void:
	var stations: Array = course["stations"]
	if stations.is_empty():
		_fails.append("street: no kerb edge found")
		return
	for s: Dictionary in stations:
		# the pavement is the higher side: toward the north frontage, -z
		await _straddle(h, Vector3(s["x"], 0.0, s["edge_z"]), Vector3(0, 0, -1), int(STRADDLE_S * FPS),
			"street_straddle_x%d" % int(round(float(s["x"]))))
	if not walker:
		return
	var speed := _walk_speed(h)
	var s0: Dictionary = stations[0]
	# across: from the road up onto the pavement, then back down, facing the way it walks
	for leg in [[1.5, -1.5], [-1.5, 1.5]]:
		var z := float(s0["edge_z"]) + float(leg[0])
		var z_end := float(s0["edge_z"]) + float(leg[1])
		var dir := signf(z_end - z)
		h.rotation = Vector3(0, 0.0 if dir > 0.0 else PI, 0)
		_put(h, Vector3(s0["x"], 0.0, z))
		await _frames_n(SETTLE_FRAMES)
		h.set_gait(speed)
		_body_speed = speed
		_segment = "street_across"
		while (z_end - z) * dir > 0.0:
			z += dir * speed / FPS
			_put(h, Vector3(s0["x"], 0.0, z))
			await process_frame
		h.set_gait(0.0)
		_body_speed = 0.0
		_segment = ""
	# along: the left foot over the pavement's edge, walking +x
	var zl := float(s0["edge_z"])
	var x := float(s0["x"]) - 2.0
	h.rotation = Vector3(0, PI * 0.5, 0)
	_put(h, Vector3(x, 0.0, zl))
	await _frames_n(SETTLE_FRAMES)
	h.set_gait(speed)
	_body_speed = speed
	_segment = "street_along"
	while x < float(s0["x"]) + 2.0:
		x += speed / FPS
		_put(h, Vector3(x, 0.0, zl))
		await process_frame
	h.set_gait(0.0)
	_body_speed = 0.0
	_segment = ""


# --- measurement ----------------------------------------------------------------------------------------------------

## One row per frame, after every modifier: per foot, its least clearance over the drawn surface.
func _on_updated(h: Human) -> void:
	if _segment == "":
		return
	var t := FootGrounding.tunables()
	var gc := h._ground
	var by := gc.points_by_side()
	var to_world := h.global_transform
	var feet := {}
	var grounds := {}
	for side: String in ["left", "right"]:
		var least := INF
		var under := INF
		var least_at := Vector3.ZERO
		for p: Vector3 in (by[side] as PackedVector3Array):
			var w := to_world * p
			var g: Variant = _ground_at(w, JUDGE_LAYER, 0.4, 0.4)
			if g == null:
				continue
			if w.y - float(g) < least:
				least = w.y - float(g)
				under = float(g)
				least_at = w
		grounds[side] = under
		# planted: G-8's drift rule on the contact point (the lowest support point, the same vertex across frames),
		# as S36-P2's gate and FootGrounding read it; the judge also asks for a run of 3 frames
		var pts: PackedVector3Array = by[side]
		var world := PackedVector3Array()
		var low := 0
		for i in pts.size():
			world.append(to_world * pts[i])
			if pts[i].y < pts[low].y:
				low = i
		var speed := INF
		var prev: PackedVector3Array = _last_foot.get(side, PackedVector3Array())
		if prev.size() == world.size() and not world.is_empty():
			speed = (world[low] - prev[low]).length() * FPS
		_last_foot[side] = world
		var planted := speed < INF and (_body_speed < FootGrounding.STANDING_MPS
			or speed <= float(t["planted_drift_share"]) * _body_speed)
		var row := {"height_mm": snappedf(least * 1000.0, 0.01) if least < INF else 0.0, "planted": planted,
			"influence": 0.0, "raise_mm": 0.0, "hit": least < INF,
			"drift_mps": snappedf(speed, 0.001) if speed < INF else -1.0, "body_mps": snappedf(_body_speed, 0.001)}
		if h._feet != null:
			row["influence"] = snappedf(float(h._feet.feet[side]["influence"]), 0.001)
			row["raise_mm"] = snappedf(float(h._feet.feet[side]["raise"]) * 1000.0, 0.01)
			row["foot_ground_mm"] = snappedf((float(h._feet.feet[side]["y"]) + h.global_position.y) * 1000.0, 0.1)
		if least < INF:
			row["at"] = [snappedf(least_at.x, 0.001), snappedf(least_at.y, 0.001), snappedf(least_at.z, 0.001)]
			row["ground_mm"] = snappedf(under * 1000.0, 0.1)
		if least == INF:
			row["planted"] = false
		feet[side] = row
	if grounds["left"] < INF and grounds["right"] < INF:
		var step := float(t["max_step_m"])
		feet["left"]["unreached"] = grounds["left"] - grounds["right"] > step
		feet["right"]["unreached"] = grounds["right"] - grounds["left"] > step
	_rows.append({"frame": _frame, "segment": _segment, "lift_mm": snappedf(gc.last_lift * 1000.0, 0.01),
		"feet": feet})
	_frame += 1


func _frames_n(n: int) -> void:
	for i in n:
		await process_frame


# --- the self-test (commits 1 and 2) --------------------------------------------------------------------------------

func _check(name: String, ok: bool, detail: String) -> void:
	print("%s %s: %s" % ["PASS" if ok else "FAIL", name, detail])
	if not ok:
		_fails.append(name)


func _self_test(out: String) -> void:
	# the floor: x -6..2, z -5..5, top at y = 0
	Build.slab(_stage, -2.0, 0.0, 8.0, 10.0, -0.4, 0.4, null, 0.0, true)
	# a 0.14 m step (the slice street's kerb height) over x >= 2, its edge along x = 2, for the straddle
	Build.slab(_stage, 4.0, 0.0, 4.0, 10.0, -0.4, 0.4 + SliceStreet.KERB_H, null, 0.0, true)
	_judge_layer(_stage)
	var ref := {"id": "reference", "reference": true}
	# 1. the per-side rule with both grounds 0 is the S26-c-ground rule, over a walk cycle
	Human.foot_grounding = false
	var a := _build_body(ref)
	a.global_position = Vector3(-3.0, 0.0, -4.0)
	Human.foot_grounding = true
	var b := _build_body(ref)
	b.global_position = Vector3(-1.5, 0.0, -4.0)
	await _frames_n(SETTLE_FRAMES)
	var worst_rule := 0.0
	var worst_flat := 0.0
	var closed := 0.0
	var worst_at := ""
	var ramp := int(ceil(FPS / float(FootGrounding.tunables()["influence_per_s"])))
	var hs := {}
	var collect := func(h: Human) -> void:
		var by := h._ground.points_by_side()
		hs[h] = [GroundContact._low(by["left"]), GroundContact._low(by["right"]), by]
	a.skeleton.skeleton_updated.connect(collect.bind(a))
	b.skeleton.skeleton_updated.connect(collect.bind(b))
	a.set_gait(_walk_speed(a))
	b.set_gait(_walk_speed(b))
	# the bodies walk forward at their walk speed (a foot is planted only against the ground, not in place)
	for i in int(FPS * 1.5):
		a.position.z += _walk_speed(a) / FPS
		b.position.z += _walk_speed(b) / FPS
		await process_frame
		var low := minf(hs[a][0], hs[a][1])
		# the old rule's result: the lowest point at 0 (pin 1 at a walk)
		worst_rule = maxf(worst_rule, absf(low))
		# the walk cycle once the start has settled: a foot leaving the floor at the switch from Stand to Walk is
		# released over the influence ramp (1 / influence_per_s)
		if i < ramp:
			continue
		for k in 2:
			# equal, or the clip's floating planted foot closed toward the floor by the IK
			var off_on := absf(float(hs[a][k]) - float(hs[b][k]))
			# (on flat ground the IK may only close a float: lower a foot toward the floor, never into it, never up)
			if float(hs[b][k]) > -0.0005 and float(hs[b][k]) < float(hs[a][k]):
				closed = maxf(closed, off_on)
			else:
				if off_on > worst_flat:
					worst_flat = off_on
					worst_at = "frame %d foot %d (off %.1f mm, on %.1f mm)" % [i, k, float(hs[a][k]) * 1000.0, float(hs[b][k]) * 1000.0]
	_check("flat-rule", worst_rule < 0.0001, "lowest point after the per-side lift, worst |y| %.3f mm" % (worst_rule * 1000.0))
	_check("flat-on-off", worst_flat < 0.0005, "per-foot height, grounding on vs off, worst %.3f mm (planted floats closed toward the floor: up to %.2f mm) at %s" % [worst_flat * 1000.0, closed * 1000.0, worst_at])
	# 2. grounds 0.14 / 0: the hips settle on the lower (right) foot
	a.set_gait(0.0)
	b.set_gait(0.0)
	a._ground.ground_left = SliceStreet.KERB_H
	a._ground.ground_right = 0.0
	await _frames_n(SETTLE_FRAMES)
	# the body's lowest point, against the lower (right) ground: the hips settled there, not on the kerb
	var lh: Dictionary = a._ground.last_heights
	var on_lower := minf(float(lh["left"]) + SliceStreet.KERB_H, float(lh["right"]))
	_check("settle-lower", absf(on_lower) < 0.0005, "lowest point above the lower ground %.3f mm" % (on_lower * 1000.0))
	a._ground.ground_left = 0.0
	# 3. straddling the 0.14 m step: the upper foot on the step top within 1 mm after 0.5 s
	# facing +z, its feet either side of the edge (x = 2), its root on the lower floor
	await _straddle(b, Vector3(2.0, 0.0, 1.0), Vector3(1, 0, 0), int(FPS * 0.5), "")
	# the support points as drawn (read in skeleton_updated, after every modifier)
	var by_b: Dictionary = hs[b][2]
	var res := []
	var xs := []
	for side: String in ["left", "right"]:
		var least := INF
		for p: Vector3 in (by_b[side] as PackedVector3Array):
			var w := b.global_transform * p
			var g: Variant = _ground_at(w, JUDGE_LAYER, 0.4, 0.4)
			if g != null:
				least = minf(least, w.y - float(g))
		res.append(least)
		var xr: Array = []
		for p: Vector3 in (by_b[side] as PackedVector3Array):
			xr.append(snappedf((b.global_transform * p).x, 0.01))
		xs.append([xr.min(), xr.max()])
	var upper := 0 if b._feet.feet["left"]["y"] > b._feet.feet["right"]["y"] else 1
	_check("straddle-step", absf(float(res[upper])) < 0.001 and absf(float(res[1 - upper])) < 0.005,
		"upper foot %.2f mm, lower foot %.2f mm above its ground (feet at x %s)" % [float(res[upper]) * 1000.0, float(res[1 - upper]) * 1000.0, xs])
	# 4. cost: 20 grounded bodies against 20 without, ms per frame per body
	a.queue_free()
	b.queue_free()
	var cost := {}
	for on in [false, true]:
		Human.foot_grounding = on
		var crowd: Array = []
		for k in 20:
			var c := _build_body(ref)
			c.global_position = Vector3(-4.0 + (k % 5) * 1.0, 0.0, -4.0 + floorf(k / 5.0) * 1.0)
			c.set_gait(_walk_speed(c))
			crowd.append(c)
		await _frames_n(SETTLE_FRAMES)
		var t0 := Time.get_ticks_usec()
		await _frames_n(60)
		cost[on] = float(Time.get_ticks_usec() - t0) / 60.0 / 1000.0
		for c: Human in crowd:
			c.queue_free()
		await _frames_n(2)
	var per := (float(cost[true]) - float(cost[false])) / 20.0
	print("feet_course: cost %.3f ms per frame per grounded body (20 bodies: %.2f ms on, %.2f ms off)" % [
		per, cost[true], cost[false]])
	var f := FileAccess.open(out.path_join("selftest.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify({"fails": _fails, "cost_ms_per_body": per, "frame_ms_on": cost[true],
		"frame_ms_off": cost[false]}, "  "))
	f.close()
	if _fails.is_empty():
		print("all feet checks pass")
	else:
		print("%d FEET CHECKS FAILED" % _fails.size())
	quit(0 if _fails.is_empty() else 1)
