## The stations of a depiction file's `stations` section (`DECISIONS.md` ARC-179; S33-e sec.5.2): each
## station's fixtures, its platform and the track beside it, drawn by S29-e's shared station pieces
## (`StationPieces`, `StationTrack`) from the file's sizes. The station's building is a `buildings` shell
## (`DepictionShells`, its doors into the places off it by `doorway_of`); its benches and kiosk are the
## world's furnishings (`ARC-307`), drawn from the disclosure.
##
##   fixtures   a board, a clock or a ticket window bay, at `at_mm`, `up_m` above the place's floor, its
##              face toward `facing_deg` (clockwise from north); a board or a clock on an optional post
##              (`post_m`), which collides; a window bay's counter collides (it is the world's counter).
##              Each is registered with `StationFixtures` by the scene volume of the place `in` names, so
##              it shows only what the world disclosed about that place (`ARC-108`): offline, or for a
##              place never observed, the defaults -- board "No service yet", window shut, clock no hands.
##   platform   the slab over the depicted place's floor at `surface_m`, its edge on side `edge` standing
##              `height_m` above the rail head, the coping and the warning strip on it, the edge's air
##              wall, a fence with a closed gate across each end, and an optional canopy cantilevered from
##              its back on brackets (no post on the floor: nothing stands there the world does not know)
##   track      one track along the edge, `near_mm` from it: with `buffer`, it stops at that end of the
##              platform at a buffer stop and runs on `beyond_m` past the other into a cutting
##              `cutting_m` long; without, it runs on at both ends. No train, no signal aspect (step-29
##              sec.5.2); nothing moves (ARC-171).
##
## The platform's slab is ground (the group `SliceProbeGeometry.GROUND_GROUP`, `LAYER_WORLD`): a body
## stands on it. The terrain is cut below the slab and along the trackbed (`cuts`), so the drawn ground
## never covers either. Every node is categorized `station.<name>` (the pieces keep their own
## `station.*` categories), so the landmark check's `station.*` hides a station. Nothing here names a
## world, a place or a tag.
class_name DepictionStations
extends RefCounted

## The category prefix the landmark check hides for a town's station (S29 sec.9.2).
const CATEGORY := "station"
## A tie's depth (`StationTrack.track`'s), metres.
const TIE_H := 0.18
## The terrain under the slab and the trackbed stands this far below what covers it, metres.
const CLEARANCE := 0.02
## The trackbed's margin beyond the ballast's shoulder, millimetres.
const BED_MARGIN_MM := 300.0
## A canopy's roof sheet and its brackets, metres; brackets stand at most this far apart.
const SHEET_M := 0.06
const BRACKET_M := 0.1
const BRACKET_PITCH_M := 2.0


## Draws every station of `stations` under `parent`, in `d`'s frame.
static func build(parent: Node3D, stations: Array, d: SliceDataDepiction) -> void:
	var g := Node3D.new()
	g.name = "Stations"
	parent.add_child(g)
	for s in stations:
		var node := Node3D.new()
		node.name = String(s["name"])
		node.set_meta("mw_category", "%s.%s" % [CATEGORY, s["name"]])
		g.add_child(node)
		var style := StationPieces.style_of(String(s["style"]))
		for f in s.get("fixtures", []) if s.get("fixtures") != null else []:
			_fixture(node, f, d, style)
		if s.get("platform") != null:
			_platform(node, s["platform"], d, style)


# --- geometry the terrain and the woods also read ------------------------------------------------

## The platform's place footprint (plan mm), or an empty rect when its tags name no one place.
static func platform_rect(p: Dictionary, d: SliceDataDepiction) -> Rect2:
	var i := d.place_index(p["depicts"])
	return d.place_footprint(i) if i >= 0 else Rect2()


## The rail head's height above the street's surface, metres.
static func rail_head(p: Dictionary) -> float:
	return float(p["surface_m"]) - float(p["height_m"])


## The trackbed's top (where the ballast's ties sit), metres above the street's surface.
static func bed_top(p: Dictionary) -> float:
	return rail_head(p) - float(p["track"]["rail_mm"][1]) / 1000.0 - TIE_H + 0.06


## The plan rect (mm) the track's bed takes beside the platform's edge, along the platform and on
## through the run and the cutting at each running end.
static func trackbed(p: Dictionary, r: Rect2) -> Rect2:
	var t: Dictionary = p["track"]
	var reach := float(t["near_mm"]) + float(t["tie_mm"][0]) * 0.5 + StationTrack.SHOULDER * 1000.0 \
		+ BED_MARGIN_MM
	var run := (float(t["beyond_m"]) + float(t["cutting_m"])) * 1000.0
	var buffer: Variant = t.get("buffer")
	var bed := Rect2()
	match String(p["edge"]):
		"e": bed = Rect2(r.end.x, r.position.y, reach, r.size.y)
		"w": bed = Rect2(r.position.x - reach, r.position.y, reach, r.size.y)
		"n": bed = Rect2(r.position.x, r.end.y, r.size.x, reach)
		_: bed = Rect2(r.position.x, r.position.y - reach, r.size.x, reach)
	var along_y := String(p["edge"]) in ["e", "w"]
	# the running ends: both, or the one away from the buffer
	for end in (["s", "n"] if along_y else ["w", "e"]):
		if buffer != null and String(buffer) == end:
			continue
		match end:
			"s": bed = Rect2(bed.position.x, bed.position.y - run, bed.size.x, bed.size.y + run)
			"n": bed.size.y += run
			"w": bed = Rect2(bed.position.x - run, bed.position.y, bed.size.x + run, bed.size.y)
			"e": bed.size.x += run
	return bed


## The terrain cuts every station asks for: [{ rect: Rect2 (plan mm), to_m: float, strict: bool }]. Under
## the platform's slab the ground is cut strictly inside its footprint, so the rooms beside it keep
## their ground at their walls; along the trackbed, to just under the ballast's top.
static func cuts(stations: Array, d: SliceDataDepiction) -> Array:
	var out := []
	for s in stations:
		var p: Variant = s.get("platform")
		if p == null:
			continue
		var r := platform_rect(p, d)
		if r.size == Vector2.ZERO:
			continue
		var low := bed_top(p) - CLEARANCE
		out.append({"rect": r, "to_m": low, "strict": true})
		out.append({"rect": trackbed(p, r), "to_m": low, "strict": false})
	return out


## The walked surface (metres above the street) of the platform covering plan point `mm`, or null.
static func surface_at(stations: Array, d: SliceDataDepiction, mm: Vector2) -> Variant:
	for s in stations:
		var p: Variant = s.get("platform")
		if p != null and platform_rect(p, d).has_point(mm):
			return float(p["surface_m"])
	return null


# --- fixtures -----------------------------------------------------------------------------------------

static func _fixture(g: Node3D, f: Dictionary, d: SliceDataDepiction, style: Dictionary) -> void:
	var size := Vector2(float(f["size_m"][0]), float(f["size_m"][1]))
	var piece: Node3D = null
	match String(f["piece"]):
		"board":
			piece = StationPieces.board(g, size.x, size.y, style)
		"clock":
			piece = StationPieces.clock(g, size.x, style)
		"window_bay":
			piece = StationPieces.window_bay(g, size.x, float(f["counter_m"]), size.y, float(f["screen_m"]),
				int(f["grilles"]), style)
	if piece == null:
		return
	var at := d.to_scene(SliceDataDepiction._point(f["at_mm"]))
	var floor_y := d.ground_height(at.x, at.z)
	var yaw := -deg_to_rad(float(f["facing_deg"]))
	var front := Vector3(-sin(yaw), 0.0, -cos(yaw))
	# on a post, the piece hangs on the post's face: the post stands at `at_mm`, where the world's solid is
	var ahead := front * (0.07 + float(f["post_m"]) * 0.5) if f.get("post_m") != null else Vector3.ZERO
	piece.transform = Transform3D(Basis(Vector3.UP, yaw), Vector3(at.x, floor_y + float(f["up_m"]), at.z) + ahead)
	if f["piece"] == "window_bay":
		# the counter is the world's solid: it collides as drawn
		var counter := float(f["counter_m"])
		Build.box_blocker(g, Vector3(at.x, floor_y + counter * 0.5, at.z), Vector3(size.x, counter, size.y), yaw)
	if f.get("post_m") != null:
		var w := float(f["post_m"])
		var top := float(f["up_m"]) + size.y * 0.5
		Build.box(g, Vector3(at.x, floor_y + top * 0.5, at.z), Vector3(w, top, w), style["wood"], yaw, true)
	StationFixtures.register(piece, d.volume_of(f["in"]))


# --- the platform and its track ---------------------------------------------------------------------

static func _platform(g: Node3D, p: Dictionary, d: SliceDataDepiction, style: Dictionary) -> void:
	var r := platform_rect(p, d)
	if r.size == Vector2.ZERO:
		return
	var ts := StationTrack.style_of()
	var o := d.frame_origin()
	var top := o.y + float(p["surface_m"])
	var lo := d.to_scene(r.position)
	var hi := d.to_scene(r.end)
	var rect := Rect2(minf(lo.x, hi.x), minf(lo.z, hi.z), absf(hi.x - lo.x), absf(hi.z - lo.z))
	var out := _outward(String(p["edge"]))
	var along := Vector3(absf(out.z), 0.0, absf(out.x))
	var height := float(p["height_m"])
	var plat := StationTrack.platform(g, rect, out, top, height + 0.5, float(p["strip_mm"]) / 1000.0,
		float(p["coping_mm"]) / 1000.0, ts)
	_ground(plat, rect, top)
	var line: Vector3 = plat.get_meta("edge_line")
	var across_x := absf(out.x) > 0.5
	var half := (rect.size.y if across_x else rect.size.x) * 0.5
	var width := rect.size.x if across_x else rect.size.y
	for s: float in [-1.0, 1.0]:
		var a := Vector3(line.x, top, line.z) + along * s * (half - 0.05)
		StationTrack.fence(g, a - out * 0.05, a - out * (width - 0.05), float(p["end_fence_m"]), 0.5, ts)
	_track(g, p, d, line, out, along, half, ts)
	if p.get("canopy") != null:
		_canopy(g, p["canopy"], d, r, out, top, style)


## The slab's top as ground: a body stands on the platform (ARC-178's rule, for a platform).
static func _ground(plat: Node3D, rect: Rect2, top: float) -> void:
	var body := StaticBody3D.new()
	body.name = "PlatformGround"
	body.collision_layer = Build.LAYER_WORLD
	body.add_to_group(SliceProbeGeometry.GROUND_GROUP)
	var cs := CollisionShape3D.new()
	var sh := BoxShape3D.new()
	sh.size = Vector3(rect.size.x, 0.2, rect.size.y)
	cs.shape = sh
	body.add_child(cs)
	body.position = Vector3(rect.get_center().x, top - 0.1, rect.get_center().y)
	plat.add_child(body)


static func _track(g: Node3D, p: Dictionary, d: SliceDataDepiction, line: Vector3, out: Vector3, along: Vector3,
		half: float, ts: Dictionary) -> void:
	var t: Dictionary = p["track"]
	var o := d.frame_origin()
	var rail_y := o.y + rail_head(p)
	var rail := Vector2(float(t["rail_mm"][0]), float(t["rail_mm"][1])) / 1000.0
	var tie := Vector2(float(t["tie_mm"][0]), float(t["tie_mm"][1])) / 1000.0
	var near := float(t["near_mm"]) / 1000.0
	var run := float(t["beyond_m"])
	var cut := float(t["cutting_m"])
	var c := Vector3(line.x, rail_y, line.z) + out * near
	# `along` points south or east in the scene; a buffer end is where the line stops
	var buffer: Variant = t.get("buffer")
	var stop := 0.0
	if buffer != null:
		stop = 1.0 if String(buffer) in ["s", "e"] else -1.0
	# the cutting's trackbed, at the depot's level under the rails (`StationDepotPlatform`)
	var bed_y := rail_y - rail.y - 0.12
	var bed_w := near * 2.0
	var mid := Vector3(line.x, bed_y, line.z) + out * near
	if stop == 0.0:
		StationTrack.track(g, c - along * (half + run + cut), c + along * (half + run + cut),
			float(t["gauge_mm"]) / 1000.0, tie, float(t["tie_pitch_mm"]) / 1000.0, rail, "through", ts)
	else:
		StationTrack.track(g, c - along * stop * (half + run + cut), c + along * stop * half,
			float(t["gauge_mm"]) / 1000.0, tie, float(t["tie_pitch_mm"]) / 1000.0, rail, "buffer", ts)
	if cut > 0.0:
		for s: float in ([-1.0, 1.0] if stop == 0.0 else [-stop]):
			StationTrack.cutting(g, mid + along * s * (half + run), mid + along * s * (half + run + cut), bed_w,
				float(t["bank_m"]), ts)


## A canopy over the platform from its back edge (opposite `out`), between `along_mm` on the platform's
## long axis: a roof sheet rising toward the back at `pitch_deg`, its outer edge `eaves_m` above the
## surface, carried on brackets from the back.
static func _canopy(g: Node3D, c: Dictionary, d: SliceDataDepiction, r: Rect2, out: Vector3, top: float,
		style: Dictionary) -> void:
	var tint := Color.html(c["tint"]) if c.get("tint") != null else Color.WHITE
	var mat := Mats.pbr(c["material"], DepictionEdges.TEXTURE_M, tint)
	var across_x := absf(out.x) > 0.5
	var a := float(c["along_mm"][0])
	var b := float(c["along_mm"][1])
	# the back edge on plan: the platform side opposite its edge
	var back_mm := (r.position.x if out.x > 0.0 else r.end.x) if across_x \
		else (r.end.y if out.z > 0.0 else r.position.y)
	var p0 := d.to_scene(Vector2(back_mm, a) if across_x else Vector2(a, back_mm))
	var p1 := d.to_scene(Vector2(back_mm, b) if across_x else Vector2(b, back_mm))
	var depth := float(c["depth_m"])
	var slope := tan(deg_to_rad(float(c["pitch_deg"])))
	var eaves := top + float(c["eaves_m"])
	var high := eaves + depth * slope
	var length := p0.distance_to(p1)
	var mid := (p0 + p1) * 0.5
	mid.y = 0.0
	var node := Node3D.new()
	node.name = "Canopy"
	g.add_child(node)
	# the sheet: a thin box tilted about the platform's long axis, its low edge out over the platform
	var long := (p1 - p0).normalized()
	var sheet := MeshInstance3D.new()
	var bm := BoxMesh.new()
	bm.size = Vector3(depth / cos(atan(slope)), SHEET_M, length)
	sheet.mesh = bm
	sheet.material_override = mat
	var tilt := Basis(long, atan(slope) * (1.0 if _turns_up(out, long) else -1.0))
	var basis := tilt * Basis(Vector3.UP, atan2(out.x, out.z) - PI * 0.5)
	sheet.transform = Transform3D(basis, mid + out * depth * 0.5 + Vector3(0.0, (eaves + high) * 0.5, 0.0))
	node.add_child(sheet)
	var count := maxi(2, int(ceil(length / BRACKET_PITCH_M)) + 1)
	for i in count:
		var at := p0.lerp(p1, float(i) / float(count - 1))
		at.y = 0.0
		var reach := depth - 0.2
		# a rafter under the sheet, tilted with it, from the back out
		var under := high - SHEET_M * 0.5 - (reach * 0.5) * slope
		var rafter := MeshInstance3D.new()
		var rm := BoxMesh.new()
		rm.size = Vector3(reach / cos(atan(slope)), BRACKET_M, BRACKET_M)
		rafter.mesh = rm
		rafter.material_override = style["wood"]
		rafter.transform = Transform3D(basis, at + out * (reach * 0.5) + Vector3(0.0, under - BRACKET_M * 0.5, 0.0))
		node.add_child(rafter)
		var strut := MeshInstance3D.new()
		var sm := BoxMesh.new()
		var drop := 0.9
		var span := Vector2(reach * 0.6, drop).length()
		sm.size = Vector3(BRACKET_M * 0.8, span, BRACKET_M * 0.8)
		strut.mesh = sm
		strut.material_override = style["wood"]
		var foot := at + Vector3(0.0, under - drop, 0.0)
		var head := at + out * (reach * 0.6) + Vector3(0.0, under - BRACKET_M, 0.0)
		var up := (head - foot).normalized()
		var side := up.cross(out).normalized()
		if side.length() < 0.5:
			side = long
		strut.transform = Transform3D(Basis(side, up, side.cross(up)), (foot + head) * 0.5)
		node.add_child(strut)


## Whether a tilt about `long` by a positive angle raises the sheet's back (the side opposite `out`).
static func _turns_up(out: Vector3, long: Vector3) -> bool:
	return (Basis(long, 0.1) * (-out)).y > 0.0


static func _outward(edge: String) -> Vector3:
	return {"e": Vector3.RIGHT, "w": Vector3.LEFT, "n": Vector3.FORWARD, "s": Vector3.BACK}[edge]
