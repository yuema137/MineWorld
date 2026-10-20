## The slice's evidence harness for walls (S15 PR 12e, step-11 §22 SD-E14,
## GE-1 … GE-5): the scene's colliders and the server's disclosed geometry agree
## within 150 mm, checked in both directions, in every drawn place the observer
## can stand in.
##
##   --slice-geometry   in the café, on the street (walked out on foot) and in
##                      the store (walked into on foot):
##                      DISCLOSED -> SCENE  along every disclosed solid face and
##                        floor edge, every 100 mm, a thin box spanning the
##                        face's own heights (the floor to the solid's height,
##                        or to the capsule's top at an edge), swept from 0.3 m
##                        on the walkable side toward the face against the scene
##                        only (`Build.LAYER_WORLD`), must meet a collider within
##                        150 mm of the face's plane -- except in a doorway's gap
##                        (two fixed-height rays until 2026-10-17: X12e-22);
##                      SCENE -> DISCLOSED  every scene collider that meets the
##                        floor and the capsule's height band, its footprint
##                        sampled every 50 mm, lies within 150 mm of a disclosed
##                        solid or of the floor's edge.
##                      Every row printed with its place, face or node, distance.
##
## The instrument masks the scene only: measuring against the disclosed
## colliders (`SliceClientBodies`) would pass by construction (M-E7). Expected
## values are the plan's literals and the disclosure's own numbers; the
## tolerance is never widened after a measurement. The two exemptions are named
## once, here, by the footprints 12d authored (step-11 QD-3): the café's and the
## florist's open door leaves, diagonal boards held by their axis-aligned boxes.
## A sibling of `slice_probe_town.gd`, as each S14/S15 PR splits its modes off.
class_name SliceProbeGeometry
extends SliceProbeTown

const GEOMETRY_MODES := ["geometry"]
## The plan's bound (step-11 §22.4), held here, not taken from the code under test.
const AGREE_MM := 150.0
## Disclosed -> scene: sample spacing, ray start on the walkable side, how far
## past the face the ray looks.
const FACE_STEP_MM := 100.0
const RAY_BACK_MM := 300.0
const RAY_PAST_MM := 500.0
## The swept box's thickness, metres: a probe's size, its half added back to the distance.
const SWEEP_THICK := 0.01
## Scene -> disclosed: footprint sample spacing; the capsule's height band.
const FOOT_STEP_MM := 50.0
const BAND_LOW := 0.05
const BAND_HIGH := 1.72
## The plan's doorway gap (step-11 §22.3.1): DOOR_GAP wide at a doorway within
## DOOR_EDGE of an edge.
const PLAN_DOOR_GAP_MM := 2000.0
const PLAN_DOOR_EDGE_MM := 500.0
## The exemptions, by name: [place tag, footprint min, footprint max, name].
const EXEMPT := [
	["cafe", Vector2(751, 0), Vector2(1153, 746), "the café's open door leaf"],
	["store", Vector2(199, 0), Vector2(593, 864), "the florist's open door leaf"],
]

var _geo_fails := 0
var _geo_rows := 0


static func requested() -> bool:
	return requested_of(GEOMETRY_MODES) != ""


func _modes() -> Array:
	return GEOMETRY_MODES


func _run(mode: String) -> void:
	match mode:
		"geometry": await _geometry_check()
		_: await super(mode)


func _geometry_check() -> void:
	print("== step-11 §22.4 GE -- the scene's walls against the server's ==\n")
	var link := slice.link
	if link == null:
		print("FAIL: no --server= given")
		return
	if not await _seated(link):
		return
	await _hold(0.8)
	var lay := link.layout
	_place_rows(link, link.here, "GE-1 café")
	# out through the café's door onto the pavement, on foot
	await _walk_to_x(6.0 + SliceCafe.DOOR_X)
	player.rotation.y = PI
	await _walk_dist(3.2, 6.0)
	await _until_place(link, lay.street)
	await _hold(0.8)
	if link.here == lay.street:
		_place_rows(link, link.here, "GE-2 street")
	else:
		_geo_fail("GE-2", "the server did not put the player on the street (%s)" % _where(link))
	# into the store, the 16c route (§20.4 L-2)
	var shop := lay.place_of_volume(SliceWorld.FLORIST_PLACE)
	if shop == "":
		_geo_fail("GE-3", "no place's doorway is bound to The Flower Room's door")
	else:
		var door: Array = SliceTerrace.doors[lay.doors[shop]]
		var sill: Vector3 = door[0]
		await _walk_to(Vector3(sill.x, 0.0, SliceStreet.NORTH_FACE + 2.70), 8.0)
		await _walk_to(Vector3(sill.x, 0.0, SliceStreet.NORTH_FACE - 1.40), 6.0)
		await _settled_report(link)
		await _until_place(link, shop)
		if link.here == shop:
			_place_rows(link, shop, "GE-3 store")
		else:
			_geo_fail("GE-3", "the server did not put the player in the store (%s)" % _where(link))
	# the summary on a line of its own, in the form CI's probe verdict reads (ci_godot.py)
	print("\n%d rows" % _geo_rows)
	print("all geometry checks pass" if _geo_fails == 0 else "%d GEOMETRY CHECKS FAILED" % _geo_fails)
	link.client.disconnect_from_world("probe done")
	await _hold(0.2)


func _seated(link: SliceLink) -> bool:
	var waited := 0.0
	while waited < 10.0 and not link.has_first_view():
		await get_tree().process_frame
		waited += get_process_delta_time()
	if not link.client.is_seated():
		print("FAIL: not seated after 10 s")
		return false
	return true


func _until_place(link: SliceLink, place: String) -> void:
	var t := 0.0
	while t < 5.0 and link.here != place:
		await _hold(0.1)
		t += 0.1


func _geo_fail(id: String, why: String) -> void:
	_geo_fails += 1
	print("%s FAIL  %s" % [id, why])


## Both directions in one place.
func _place_rows(link: SliceLink, place: String, label: String) -> void:
	var obs := link.client.latest
	var shape: Variant = obs.component_value(place, "place-shape")
	if typeof(shape) != TYPE_DICTIONARY:
		_geo_fail(label, "the place discloses no place-shape")
		return
	var tags: Array = link.layout.tags.get(place, [])
	var lo := _v2(shape["floor"]["min"])
	var hi := _v2(shape["floor"]["max"])
	var solids: Array = []
	for s in shape.get("solids", []):
		solids.append({"min": _v2(s["min"]), "max": _v2(s["max"]), "h": float(s["height"])})
	var doors: Array = []
	for p in link.layout.passages.get(place, []):
		doors.append(_v2(p["here"]))
	print("-- %s: place %s, floor %s..%s, %d solids, %d doorways" % [label, place, lo, hi, solids.size(),
		doors.size()])
	# DISCLOSED -> SCENE
	_ramps.clear()
	for cs in _scene_shapes(slice.world):
		if cs.shape is BoxShape3D and _ramp(cs):
			_ramps.append((cs.get_parent() as CollisionObject3D).get_rid())
	var faces: Array = []
	for i in range(solids.size()):
		var s: Dictionary = solids[i]
		var top: float = s["h"]
		var nm := "solid %d %s..%s h %d" % [i, s["min"], s["max"], s["h"]]
		var exempt := _exempt(tags, s["min"], s["max"])
		faces.append([nm + " south", Vector2(s["min"].x, s["min"].y), Vector2(s["max"].x, s["min"].y),
			Vector2(0, -1), top, exempt, false])
		faces.append([nm + " north", Vector2(s["min"].x, s["max"].y), Vector2(s["max"].x, s["max"].y),
			Vector2(0, 1), top, exempt, false])
		faces.append([nm + " west", Vector2(s["min"].x, s["min"].y), Vector2(s["min"].x, s["max"].y),
			Vector2(-1, 0), top, exempt, false])
		faces.append([nm + " east", Vector2(s["max"].x, s["min"].y), Vector2(s["max"].x, s["max"].y),
			Vector2(1, 0), top, exempt, false])
	faces.append(["floor edge south", Vector2(lo.x, lo.y), Vector2(hi.x, lo.y), Vector2(0, 1), BAND_HIGH * 1000.0, "", true])
	faces.append(["floor edge north", Vector2(lo.x, hi.y), Vector2(hi.x, hi.y), Vector2(0, -1), BAND_HIGH * 1000.0, "", true])
	faces.append(["floor edge west", Vector2(lo.x, lo.y), Vector2(lo.x, hi.y), Vector2(1, 0), BAND_HIGH * 1000.0, "", true])
	faces.append(["floor edge east", Vector2(hi.x, lo.y), Vector2(hi.x, hi.y), Vector2(-1, 0), BAND_HIGH * 1000.0, "", true])
	for f in faces:
		_face_row(link, place, label, f, lo, hi, solids, doors)
	# SCENE -> DISCLOSED
	var base := link.to_scene(place, {"x": 0, "y": 0}).y
	for cs in _scene_shapes(slice.world):
		_collider_row(link, place, label, cs, base, lo, hi, solids)


func _face_row(link: SliceLink, place: String, label: String, f: Array, lo: Vector2, hi: Vector2,
		solids: Array, doors: Array) -> void:
	var a: Vector2 = f[1]
	var b: Vector2 = f[2]
	var n: Vector2 = f[3]
	var tall: float = f[4]
	var length := a.distance_to(b)
	var along := (b - a) / maxf(length, 1.0)
	var worst := 0.0
	var worst_at := Vector2.ZERO
	var over := 0
	var samples := 0
	var spans: Array = []          # [from, to] along the face, mm, where a sample is over
	var worst_met := ""
	var space := player.get_world_3d().direct_space_state
	var base := link.to_scene(place, {"x": 0, "y": 0}).y
	var t := FACE_STEP_MM * 0.5
	while t < length:
		var q := a + along * t
		var at := t
		t += FACE_STEP_MM
		var from := q + n * RAY_BACK_MM
		if not _walkable(from, lo, hi, solids):
			continue
		if bool(f[6]) and _in_gap(q, n, doors):
			continue
		# another disclosed solid between the walkable side and the face (a bollard
		# against a planter): the face is not exposed there, and the sweep would meet
		# the other one -- that solid's own faces are measured on their own
		if _approach_blocked(from, q, solids):
			continue
		var dev := _sweep(space, link, place, base, from, n, tall)
		if dev > worst:
			worst = dev
			worst_at = q
			worst_met = _last_met
		if dev > AGREE_MM:
			over += 1
			if not spans.is_empty() and at - float(spans.back()[1]) <= FACE_STEP_MM * 1.5:
				spans.back()[1] = at
			else:
				spans.append([at, at])
		samples += 1
	if samples == 0:
		return
	var where := ""
	for sp in spans:
		var p0: Vector2 = a + along * float(sp[0])
		var p1: Vector2 = a + along * float(sp[1])
		where += " (%d, %d)..(%d, %d)" % [p0.x, p0.y, p1.x, p1.y]
	_row(label, "disclosed->scene", f[0], worst, f[5], "%d samples, %d over%s%s" % [samples, over,
		"; over at" + where if where != "" else "",
		"; the worst met %s" % worst_met if worst > AGREE_MM and worst_met != "" else ""])


## One sample (step-11 §22.15 X12e-22, the primary session's ruling of 2026-10-17): from 300 mm on
## the walkable side, a thin vertical box spanning the face's own heights -- from just above the
## floor (BAND_LOW) to the solid's height, or to the capsule's top for a floor edge -- is swept toward
## the face against the scene only. It meets a slab, a low kerb or a full-height wall alike, wherever
## along those heights the scene's collider stands, which two fixed-height rays did not. Returns how
## far from the face's plane the scene's collider stands, in millimetres, or INF.
func _sweep(space: PhysicsDirectSpaceState3D, link: SliceLink, place: String, base: float, from: Vector2,
		n: Vector2, tall_mm: float) -> float:
	var top := minf(tall_mm / 1000.0, BAND_HIGH)
	if top <= BAND_LOW:
		top = BAND_LOW + 0.01
	var box := BoxShape3D.new()
	box.size = Vector3(SWEEP_THICK, top - BAND_LOW, SWEEP_THICK)
	var q := PhysicsShapeQueryParameters3D.new()
	q.shape = box
	var p0 := link.to_scene(place, {"x": int(from.x), "y": int(from.y)})
	p0.y = base + (BAND_LOW + top) * 0.5
	q.transform = Transform3D(Basis(), p0)
	q.motion = Vector3(-n.x, 0.0, n.y) * (RAY_BACK_MM + RAY_PAST_MM) / 1000.0
	q.collision_mask = Build.LAYER_WORLD
	q.exclude = _ramps
	var safe := space.cast_motion(q)
	_last_met = ""
	if safe.is_empty() or float(safe[0]) >= 1.0:
		return INF
	# what it met, for the row: the collider at the first unsafe point
	q.transform = Transform3D(Basis(), p0 + q.motion * float(safe[1]))
	for hit in space.intersect_shape(q, 1):
		var c: Variant = hit.get("collider")
		if c is Node:
			_last_met = String((c as Node).get_path()).replace("/root/Slice/Slice/", "")
	return absf(float(safe[0]) * (RAY_BACK_MM + RAY_PAST_MM) + SWEEP_THICK * 500.0 - RAY_BACK_MM)


## The scene collider the last sweep met first ("" if none).
var _last_met := ""
## The scene's ramps (a box tilted no steeper than the controller climbs): floor
## to walk up, never a face, so the sweep passes them (X12e-22).
var _ramps: Array[RID] = []


## Whether a point on the walkable side is on the floor and in no solid.
static func _walkable(p: Vector2, lo: Vector2, hi: Vector2, solids: Array) -> bool:
	if p.x <= lo.x or p.x >= hi.x or p.y <= lo.y or p.y >= hi.y:
		return false
	for s in solids:
		if p.x >= s["min"].x and p.x <= s["max"].x and p.y >= s["min"].y and p.y <= s["max"].y:
			return false
	return true


## Whether the 300 mm approach from `from` to the face point `q` passes through
## a disclosed solid (sampled every 20 mm, the last 10 mm -- the face itself --
## excluded).
static func _approach_blocked(from: Vector2, q: Vector2, solids: Array) -> bool:
	var length := from.distance_to(q)
	var k := 0.0
	while k < length - 10.0:
		var p := from.lerp(q, k / length)
		for sd in solids:
			if p.x > sd["min"].x and p.x < sd["max"].x and p.y > sd["min"].y and p.y < sd["max"].y:
				return true
		k += 20.0
	return false


## Whether a floor-edge point lies in a doorway's gap (the plan's DOOR_GAP at a
## doorway within DOOR_EDGE of the edge).
static func _in_gap(q: Vector2, n: Vector2, doors: Array) -> bool:
	for d in doors:
		var off := absf((d - q).dot(n))
		var side := absf((d - q).dot(Vector2(n.y, -n.x)))
		if off <= PLAN_DOOR_EDGE_MM and side <= PLAN_DOOR_GAP_MM * 0.5:
			return true
	return false


func _exempt(tags: Array, a: Vector2, b: Vector2) -> String:
	for e in EXEMPT:
		if tags.has(e[0]) and a == e[1] and b == e[2]:
			return e[3]
	return ""


func _row(label: String, direction: String, what: String, mm: float, exempt: String, note: String) -> void:
	_geo_rows += 1
	var over := mm > AGREE_MM
	var verdict := "ok" if not over else ("EXEMPT (%s)" % exempt if exempt != "" else "FAIL")
	if over and exempt == "":
		_geo_fails += 1
	print("%s  %s  %-60s  %s mm  %s  %s" % [label.split(" ")[0], direction, what,
		"none within %d" % int(RAY_PAST_MM) if mm == INF else "%.1f" % mm, verdict, note])


## Every collision shape of a static body on the scene's layer under `root`.
func _scene_shapes(root: Node) -> Array:
	var out: Array = []
	for c in root.get_children():
		if c is CollisionShape3D and not (c as CollisionShape3D).disabled and c.get_parent() is StaticBody3D \
				and ((c.get_parent() as StaticBody3D).collision_layer & Build.LAYER_WORLD) != 0:
			out.append(c)
		out.append_array(_scene_shapes(c))
	return out


## One scene collider against the disclosure: its footprint inside the floor, in
## the band, every sample within AGREE_MM of a disclosed solid or the floor's
## edge.
func _collider_row(link: SliceLink, place: String, label: String, cs: CollisionShape3D, base: float,
		lo: Vector2, hi: Vector2, solids: Array) -> void:
	var aabb := _shape_aabb(cs)
	if aabb.size == Vector3.ZERO:
		return
	if aabb.end.y <= base + BAND_LOW or aabb.position.y >= base + BAND_HIGH:
		return
	# the AABB's corners in the place frame, clipped to the floor
	var c0 := _v2(link.to_world(place, aabb.position))
	var c1 := _v2(link.to_world(place, aabb.end))
	var x0 := maxf(minf(c0.x, c1.x), lo.x)
	var x1 := minf(maxf(c0.x, c1.x), hi.x)
	var y0 := maxf(minf(c0.y, c1.y), lo.y)
	var y1 := minf(maxf(c0.y, c1.y), hi.y)
	if x0 >= x1 or y0 >= y1:
		return
	var inv := cs.global_transform.affine_inverse()
	var exact := cs.shape is BoxShape3D or cs.shape is CylinderShape3D
	var worst := -1.0
	var worst_p := Vector2.ZERO
	var samples := 0
	var x := x0 + FOOT_STEP_MM * 0.5
	while x < x1:
		var y := y0 + FOOT_STEP_MM * 0.5
		while y < y1:
			var p := Vector2(x, y)
			y += FOOT_STEP_MM
			var sp := link.to_scene(place, {"x": int(p.x), "y": int(p.y)})
			sp.y = clampf(base + 0.9, aabb.position.y, aabb.end.y)
			if exact and not _inside(cs, inv * sp):
				continue
			# exact for a box: where it stands at this point (a tilted kerb ramp's top is the
			# pavement it leads to, below the band)
			if cs.shape is BoxShape3D and (_box_top(cs, sp) <= base + BAND_LOW or _ramp(cs)):
				continue
			samples += 1
			var dd := _to_disclosed(p, lo, hi, solids)
			if dd > worst:
				worst = dd
				worst_p = p
		x += FOOT_STEP_MM
	if samples == 0:
		return
	_row(label, "scene->disclosed", String(cs.get_parent().get_path()).replace("/root/", ""), worst, "",
		"%d samples%s; worst at (%d, %d)" % [samples, "" if exact else ", AABB (%s)" % cs.shape.get_class(),
		worst_p.x, worst_p.y])


## Distance (mm) from a floor point to the nearest disclosed solid footprint
## (0 inside one) or to the floor's edge.
static func _to_disclosed(p: Vector2, lo: Vector2, hi: Vector2, solids: Array) -> float:
	var best := minf(minf(p.x - lo.x, hi.x - p.x), minf(p.y - lo.y, hi.y - p.y))
	for s in solids:
		var dx := maxf(maxf(s["min"].x - p.x, 0.0), p.x - s["max"].x)
		var dy := maxf(maxf(s["min"].y - p.y, 0.0), p.y - s["max"].y)
		best = minf(best, Vector2(dx, dy).length())
	return best


## A box's top surface height above the scene point `p`'s (x, z): the highest of
## its upward faces' planes there (the lowest of them, the box being convex):
## for an upright box its top, for a tilted one (a ramp) the sloped face.
## Whether a box is a ramp: its top face tilted, but no steeper than the
## controller climbs (`Player.floor_max_angle`, 52°) -- a floor to walk up, not
## an obstacle.
static func _ramp(cs: CollisionShape3D) -> bool:
	var up := 0.0
	for axis in range(3):
		up = maxf(up, absf(cs.global_transform.basis[axis].normalized().y))
	return up < 0.9995 and up >= cos(deg_to_rad(52.0))


static func _box_top(cs: CollisionShape3D, p: Vector3) -> float:
	var e: Vector3 = (cs.shape as BoxShape3D).size * 0.5
	var xf := cs.global_transform
	var best := INF
	for axis in range(3):
		for s in [-1.0, 1.0]:
			var nrm: Vector3 = xf.basis[axis].normalized() * s
			if nrm.y < 0.2:
				continue
			var p0: Vector3 = xf.origin + xf.basis[axis] * e[axis] * s
			best = minf(best, p0.y - (nrm.x * (p.x - p0.x) + nrm.z * (p.z - p0.z)) / nrm.y)
	return best


static func _inside(cs: CollisionShape3D, local: Vector3) -> bool:
	if cs.shape is BoxShape3D:
		var e: Vector3 = (cs.shape as BoxShape3D).size * 0.5
		return absf(local.x) <= e.x + 0.001 and absf(local.z) <= e.z + 0.001
	if cs.shape is CylinderShape3D:
		var r: float = (cs.shape as CylinderShape3D).radius
		return Vector2(local.x, local.z).length() <= r + 0.001
	return true


static func _shape_aabb(cs: CollisionShape3D) -> AABB:
	if cs.shape == null:
		return AABB()
	var local := AABB()
	if cs.shape is BoxShape3D:
		var s: Vector3 = (cs.shape as BoxShape3D).size
		local = AABB(-s * 0.5, s)
	elif cs.shape is CylinderShape3D:
		var c := cs.shape as CylinderShape3D
		local = AABB(Vector3(-c.radius, -c.height * 0.5, -c.radius), Vector3(c.radius * 2, c.height, c.radius * 2))
	elif cs.shape is SphereShape3D:
		var r: float = (cs.shape as SphereShape3D).radius
		local = AABB(Vector3(-r, -r, -r), Vector3(r, r, r) * 2.0)
	elif cs.shape is CapsuleShape3D:
		var k := cs.shape as CapsuleShape3D
		local = AABB(Vector3(-k.radius, -k.height * 0.5, -k.radius), Vector3(k.radius * 2, k.height, k.radius * 2))
	else:
		var mesh := cs.shape.get_debug_mesh()
		if mesh == null:
			return AABB()
		local = mesh.get_aabb()
	return cs.global_transform * local


static func _v2(v: Variant) -> Vector2:
	if typeof(v) == TYPE_VECTOR3:
		return Vector2.ZERO
	if typeof(v) != TYPE_DICTIONARY:
		return Vector2.ZERO
	return Vector2(float(v.get("x", 0)), float(v.get("y", 0)))
