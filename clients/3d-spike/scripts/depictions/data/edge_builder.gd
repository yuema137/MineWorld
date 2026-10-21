## The line-drawn edges of a depiction file's `edges` section (`DECISIONS.md` ARC-177; S33-d sec.5.4):
## kerbs, quays, revetments, rails, low walls and gates, each drawn along its line on the drawn ground,
## with a gap at the doorway of every place tagged in its `open_at` -- so a gap is never a coordinate
## written twice.
##
##   kerb, wall, gate   a block `height_m` tall and `width_m` thick, standing on the ground
##   quay, revetment    a wall from `drop_m` below the ground to `height_m` above it (the fall to the water)
##   rail               posts every `posts_m`, `width_m` square, and a top and a middle rail, `height_m` tall
##
## Each edge collides on `Build.LAYER_ART` (ARC-52) as drawn (a rail as one box over its posts), so where an edge
## stands on a place's boundary the geometry probe finds it at the server's wall (12e); the server's
## walls still decide who passes. Nothing here names a world, a place or a tag.
class_name DepictionEdges
extends RefCounted

## A texture's repeat on an edge or a shell, metres (`PRESENTATION.md` sec.6's schema default).
const TEXTURE_M := 2.0


## Draws every edge of `edges` under `parent`, on `d`'s ground, with gaps at `d`'s places' doorways.
static func build(parent: Node3D, edges: Array, d: SliceDataDepiction) -> void:
	var g := Node3D.new()
	g.name = "Edges"
	parent.add_child(g)
	for e in edges:
		var tint := Color.html(e["tint"]) if e.get("tint") != null else Color.WHITE
		var mat := Mats.pbr(e["material"], TEXTURE_M, tint)
		var doorways := _doorways(d, e.get("open_at", []) if e.get("open_at") != null else [])
		var line: Array = e["line_mm"]
		for k in line.size() - 1:
			var a := Vector2(float(line[k][0]), float(line[k][1]))
			var b := Vector2(float(line[k + 1][0]), float(line[k + 1][1]))
			for span in _spans(a, b, doorways):
				_draw(g, e, d.ground_at(span[0]), d.ground_at(span[1]), mat)


## The doorways (plan mm) of every place carrying one of `tags`.
static func _doorways(d: SliceDataDepiction, tags: Array) -> Array:
	var out := []
	for p in d.places():
		for t in tags:
			if t in p["tags"]:
				out.append(p["doorway"])
				break
	return out


## Segment a-b (plan mm) less a DOOR_W gap centred on every doorway within the box depiction's
## DOOR_GAP_MATCH_MM of it: [[from, to], ...].
static func _spans(a: Vector2, b: Vector2, doorways: Array) -> Array:
	var length := a.distance_to(b)
	if length <= 0.0:
		return []
	var dir := (b - a) / length
	var half := SliceBoxDepiction.DOOR_W * 500.0
	var cuts := []
	for door in doorways:
		var along: float = (door - a).dot(dir)
		var across: float = absf((door - a).cross(dir))
		if across <= SliceBoxDepiction.DOOR_GAP_MATCH_MM and along >= -half and along <= length + half:
			cuts.append(along)
	cuts.sort()
	var out := []
	var start := 0.0
	for at in cuts:
		if at - half > start:
			out.append([a + dir * start, a + dir * (at - half)])
		start = maxf(start, at + half)
	if start < length:
		out.append([a + dir * start, b])
	return out


## One span of edge `e`, from scene point `p` to `q` on the ground.
static func _draw(g: Node3D, e: Dictionary, p: Vector3, q: Vector3, mat: Material) -> void:
	var run := Vector2(q.x - p.x, q.z - p.z)
	var length := run.length()
	if length <= 0.0:
		return
	var yaw := atan2(run.x, run.y)
	var h := float(e["height_m"])
	var w := float(e["width_m"])
	var base := minf(p.y, q.y)
	var mid := Vector3((p.x + q.x) * 0.5, base, (p.z + q.z) * 0.5)
	match e["kind"]:
		"quay", "revetment":
			var drop := float(e["drop_m"])
			Build.box(g, mid + Vector3(0.0, (h - drop) * 0.5, 0.0), Vector3(w, h + drop, length), mat, yaw,
				true)
		"rail":
			var posts := maxi(int(ceil(length / float(e["posts_m"]))), 1)
			for k in posts + 1:
				var at := p.lerp(q, float(k) / float(posts))
				Build.box(g, at + Vector3(0.0, h * 0.5, 0.0), Vector3(w, h, w), mat)
			for rail_y in [h - w * 0.5, h * 0.5]:
				Build.box(g, mid + Vector3(0.0, rail_y, 0.0), Vector3(w * 0.8, w * 0.8, length), mat, yaw)
			Build.box_blocker(g, mid + Vector3(0.0, h * 0.5, 0.0), Vector3(w, h, length), yaw)
		_:
			Build.box(g, mid + Vector3(0.0, h * 0.5, 0.0), Vector3(w, h, length), mat, yaw, true)
