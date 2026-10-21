## The platform, the track, the cutting and the fence (S29-e §5.3, §5.5; `ARC-192`'s note of
## 2026-10-20). Like `StationPieces`, each is a function of sizes in metres, scene points and a style; none
## reads a kit, a place or a world, so S33-e's halt builds its one track with a `buffer` end from the same
## calls. Every node is categorized `station.platform`, so `SliceBatch` keeps it and the landmark check
## hides it with the rest of the station.
##
## style = { slab, coping, strip, rail, tie, ballast, bank, iron: Material } (`style_of`).
## Nothing here draws a train, a carriage or a signal aspect (step-29 §5.2).
class_name StationTrack
extends RefCounted

const CATEGORY := "station.platform"
## Ballast below the tie's underside, and its shoulder beyond the tie ends (design).
const BALLAST := 0.30
const SHOULDER := 0.45
## The air wall on a platform's edge: the server never lets a body over it (the disclosed floor ends
## there); 12e's geometry probe wants a collider on every disclosed edge (I-e-5).
const AIR_WALL := Vector2(0.10, 1.60)


static func style_of(_name := "") -> Dictionary:
	return {"slab": Mats.pbr("concrete_pavement", 2.0, Color(0.92, 0.88, 0.82), 0.95),
		"coping": SlicePalette.limestone(), "strip": Mats.paint(Color(0.86, 0.68, 0.16), 0.8),
		"rail": Mats.paint(Color(0.36, 0.33, 0.30), 0.45, 0.6), "tie": Mats.pbr("weathered_brown_planks", 1.5,
		Color(0.55, 0.46, 0.38), 0.95), "ballast": Mats.pbr("cobblestone_floor_08", 1.2, Color(0.62, 0.60, 0.56), 1.0),
		"bank": Mats.pbr("leafy_grass", 2.2, Color(0.46, 0.58, 0.34), 0.97), "iron": SlicePalette.iron()}


static func _root(parent: Node3D, nm: String) -> Node3D:
	var n := Node3D.new()
	n.name = nm
	n.set_meta("mw_category", CATEGORY)
	parent.add_child(n)
	return n


## A platform over `rect` (scene x, z), its top at `top_y`, its edge on the side `edge` points to (a unit
## axis): the slab down `drop` to the trackbed, a stone coping `coping` wide on the edge, the detectable
## warning strip `strip` wide inboard of the coping (2010 ADA §705), and the edge's air wall.
static func platform(parent: Node3D, rect: Rect2, edge: Vector3, top_y: float, drop: float, strip: float,
		coping: float, style: Dictionary) -> Node3D:
	var n := _root(parent, "platform")
	var c := Vector3(rect.get_center().x, top_y - drop * 0.5, rect.get_center().y)
	Build.box(n, c - Vector3(0, 0.005, 0), Vector3(rect.size.x, drop - 0.01, rect.size.y), style["slab"], 0.0, true)
	var across := absf(edge.x) > 0.5
	var half := (rect.size.x if across else rect.size.y) * 0.5
	var length := rect.size.y if across else rect.size.x
	var line := c + edge * half
	var cope_size := Vector3(coping, 0.06, length) if across else Vector3(length, 0.06, coping)
	Build.box(n, Vector3(line.x, top_y - 0.025, line.z) - edge * coping * 0.5, cope_size, style["coping"])
	var strip_size := Vector3(strip, 0.012, length) if across else Vector3(length, 0.012, strip)
	Build.box(n, Vector3(line.x, top_y + 0.002, line.z) - edge * (coping + strip * 0.5), strip_size, style["strip"])
	var wall := Vector3(AIR_WALL.x, AIR_WALL.y, length) if across else Vector3(length, AIR_WALL.y, AIR_WALL.x)
	Build.box_blocker(n, Vector3(line.x, top_y + AIR_WALL.y * 0.5, line.z) + edge * AIR_WALL.x * 0.5, wall)
	n.set_meta("edge_line", line)
	return n


## One track from `from` to `to` (scene points on its centreline at rail-head height): two rails
## `gauge` apart (inside faces), ties `tie` (length, width) at `pitch` as one MultiMesh, and ballast.
## `end_kind` "through": the line runs on (the caller adds the cutting); "buffer": a buffer stop at `to`.
static func track(parent: Node3D, from: Vector3, to: Vector3, gauge: float, tie: Vector2, pitch: float,
		rail: Vector2, end_kind: String, style: Dictionary) -> Node3D:
	var n := _root(parent, "track")
	var dir := (to - from).normalized()
	var side := dir.cross(Vector3.UP).normalized()
	var length := from.distance_to(to)
	var mid := (from + to) * 0.5
	var yaw := atan2(dir.x, dir.z)
	var tie_h := 0.18
	var tie_top := from.y - rail.y
	for s: float in [-1.0, 1.0]:
		var p := mid + side * s * (gauge + rail.x) * 0.5 - Vector3(0, rail.y * 0.5, 0)
		Build.box(n, p, Vector3(rail.x, rail.y, length), style["rail"], yaw)
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	var bm := BoxMesh.new()
	bm.size = Vector3(tie.x, tie_h, tie.y)
	bm.material = style["tie"]
	mm.mesh = bm
	var count := int(floor(length / pitch))
	mm.instance_count = count
	var placed := PackedVector3Array()
	for i in count:
		var p := from + dir * (pitch * (i + 0.5)) - Vector3(0, rail.y + tie_h * 0.5, 0)
		mm.set_instance_transform(i, Transform3D(Basis(Vector3.UP, yaw), p))
		placed.append(p)
	# the instances' points as placed: a headless (dummy) renderer keeps no MultiMesh buffer to read back
	n.set_meta("tie_points", placed)
	var mmi := MultiMeshInstance3D.new()
	mmi.name = "ties"
	mmi.multimesh = mm
	mmi.set_meta("mw_category", CATEGORY)
	n.add_child(mmi)
	var bed_top := tie_top - tie_h + 0.06
	Build.box(n, mid + Vector3(0, bed_top - from.y - BALLAST * 0.5, 0),
		Vector3(tie.x + SHOULDER * 2.0, BALLAST, length), style["ballast"], yaw)
	if end_kind == "buffer":
		var b := to - dir * 0.6
		for s: float in [-1.0, 1.0]:
			Build.box(n, b + side * s * gauge * 0.5 + Vector3(0, 0.2, 0), Vector3(0.25, 1.0, 0.25), style["iron"], yaw)
		Build.box(n, b + Vector3(0, 0.55, 0), Vector3(gauge + 0.9, 0.35, 0.30), style["iron"], yaw)
	n.set_meta("count", count)
	return n


## A cutting from `from` to `to` (scene points on the line, at the trackbed's level): the trackbed
## `width` wide between two planted banks rising `bank` metres, and at its far end a stone overbridge
## that closes the view of the line without ending it (Q-S29e-1). Banks and bridge carry colliders.
static func cutting(parent: Node3D, from: Vector3, to: Vector3, width: float, bank: float, style: Dictionary) -> Node3D:
	var n := _root(parent, "cutting")
	var dir := (to - from).normalized()
	var side := dir.cross(Vector3.UP).normalized()
	var length := from.distance_to(to)
	var mid := (from + to) * 0.5
	var yaw := atan2(dir.x, dir.z)
	Build.box(n, mid - Vector3(0, 0.25, 0), Vector3(width, 0.5, length), style["ballast"], yaw, true)
	for s: float in [-1.0, 1.0]:
		# a bank in three terraces, so it reads as a slope
		for k in 3:
			var w := bank * 1.2
			var h := bank * float(k + 1) / 3.0
			var off := width * 0.5 + w * (0.5 + k)
			Build.box(n, mid + side * s * off + Vector3(0, h * 0.5 - 0.3, 0), Vector3(w + 0.02, h + 0.3, length),
				style["bank"], yaw, k == 0)
	var deck := to - dir * 2.0
	var span := width + bank * 7.2
	Build.box(n, deck + Vector3(0, bank + 0.5, 0), Vector3(span, 1.0, 4.0), style["coping"], yaw)
	for s: float in [-1.0, 1.0]:
		Build.box(n, deck + side * s * (width * 0.5 + 0.6) + Vector3(0, bank * 0.5, 0), Vector3(1.2, bank + 0.02, 4.0),
			style["coping"], yaw, true)
	# behind the bridge the line is closed to the eye by a planted bank across it (it is not reached)
	Build.box(n, to + dir * 6.0 + Vector3(0, bank * 0.6, 0), Vector3(span, bank * 1.2 + 0.6, 6.0), style["bank"], yaw)
	return n


## A fence of painted iron palings from `from` to `to` (scene, on the ground), `height` tall, with a
## closed gate centred at the fraction `gate_at` of its length. It collides along its whole length.
static func fence(parent: Node3D, from: Vector3, to: Vector3, height: float, gate_at: float, style: Dictionary) -> Node3D:
	var n := _root(parent, "fence")
	var dir := (to - from).normalized()
	var length := from.distance_to(to)
	var yaw := atan2(dir.x, dir.z) + PI * 0.5
	var mid := (from + to) * 0.5
	for y: float in [0.12, height - 0.10]:
		Build.box(n, mid + Vector3(0, y, 0), Vector3(length, 0.04, 0.03), style["iron"], yaw)
	var count := int(length / 0.13)
	for i in count + 1:
		var p := from + dir * (length * i / float(count))
		Build.box(n, p + Vector3(0, height * 0.5, 0), Vector3(0.018, height, 0.018), style["iron"])
	var g := from + dir * length * gate_at
	for s: float in [-0.6, 0.6]:
		Build.box(n, g + dir * s + Vector3(0, (height + 0.15) * 0.5, 0), Vector3(0.08, height + 0.15, 0.08), style["iron"])
	Build.box(n, g + Vector3(0, height * 0.55, 0), Vector3(1.1, 0.03, 0.03), style["iron"], yaw)
	Build.box_blocker(n, mid + Vector3(0, height * 0.5, 0), Vector3(length, height, 0.06), yaw)
	n.set_meta("count", count + 1)
	return n
