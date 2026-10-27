## The small scattered things of a depiction file's `scatter` section (`DECISIONS.md` ARC-177; S33-d
## sec.5.4, sec.5.6): granite boulders (`Props.boulder`), and reeds, grass and wildflower clumps as one
## `MultiMesh` of cards per entry. Each entry is placed by its own seed in its polygon, or in a strip of
## `width_m` along its line, on the drawn ground. The validator keeps every area off every place's
## floor (`DEPICTION_ON_FLOOR`): what people walk among is the world's (furnishings), never this file's
## (`ARC-307`). Nothing here names a world, a place or a tag.
class_name DepictionScatter
extends RefCounted

## How many placements are tried per thing asked for before an area is given up as too small.
const TRIES := 8


## Draws every entry of `scatter` under `parent`, on `d`'s ground.
static func build(parent: Node3D, scatter: Array, d: SliceDataDepiction) -> void:
	var g := Node3D.new()
	g.name = "Scatter"
	parent.add_child(g)
	for entry in scatter:
		var rng := RandomNumberGenerator.new()
		rng.seed = int(entry["seed"])
		var points := _points(entry, rng)
		var lo := float(entry["size_m"]["min"])
		var hi := float(entry["size_m"]["max"])
		if entry["kind"] == "boulder":
			for i in points.size():
				var size := rng.randf_range(lo, hi)
				Props.boulder(g, d.ground_at(points[i]), size * 0.5, int(entry["seed"]) * 1000 + i)
		else:
			_cards(g, entry, points, rng, lo, hi, d)


## `count` plan points (mm) in the entry's area, seeded.
static func _points(entry: Dictionary, rng: RandomNumberGenerator) -> Array:
	var area: Dictionary = entry["area"]
	var count := int(entry["count"])
	var out := []
	if area.get("polygon_mm") != null:
		var polygon := PackedVector2Array()
		for p in area["polygon_mm"]:
			polygon.append(Vector2(float(p[0]), float(p[1])))
		var box := Rect2(polygon[0], Vector2.ZERO)
		for p in polygon:
			box = box.expand(p)
		var tries := count * TRIES
		while out.size() < count and tries > 0:
			tries -= 1
			var at := Vector2(rng.randf_range(box.position.x, box.end.x), rng.randf_range(box.position.y, box.end.y))
			if Geometry2D.is_point_in_polygon(at, polygon):
				out.append(at)
		return out
	var line: Array = area["along_mm"]
	var half := float(area["width_m"]) * 500.0
	var lengths := []
	var total := 0.0
	for k in line.size() - 1:
		var length := Vector2(float(line[k][0]), float(line[k][1])).distance_to(
			Vector2(float(line[k + 1][0]), float(line[k + 1][1])))
		lengths.append(length)
		total += length
	if total <= 0.0:
		return out
	for i in count:
		var s := rng.randf() * total
		var k := 0
		while k < lengths.size() - 1 and s > lengths[k]:
			s -= lengths[k]
			k += 1
		var a := Vector2(float(line[k][0]), float(line[k][1]))
		var b := Vector2(float(line[k + 1][0]), float(line[k + 1][1]))
		var dir := (b - a).normalized()
		out.append(a + dir * clampf(s, 0.0, lengths[k]) + dir.orthogonal() * rng.randf_range(-half, half))
	return out


## Reeds, grass or wildflowers: one MultiMesh of upright cards, each turned and sized by the seed.
## Reeds stand twice as tall as they are wide; grass and flowers as wide as tall.
static func _cards(g: Node3D, entry: Dictionary, points: Array, rng: RandomNumberGenerator,
		lo: float, hi: float, d: SliceDataDepiction) -> void:
	var quad := QuadMesh.new()
	quad.size = Vector2.ONE
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.mesh = quad
	mm.instance_count = points.size()
	var reeds: bool = entry["kind"] == "reeds"
	for i in points.size():
		var size := rng.randf_range(lo, hi)
		var width := size * 0.5 if reeds else size
		var at := d.ground_at(points[i]) + Vector3(0.0, size * 0.5, 0.0)
		var turn := Basis(Vector3.UP, rng.randf() * TAU).scaled(Vector3(width, size, 1.0))
		mm.set_instance_transform(i, Transform3D(turn, at))
	var mi := MultiMeshInstance3D.new()
	mi.name = String(entry["name"])
	mi.multimesh = mm
	var variant := int(entry["seed"]) % 3
	mi.material_override = Props._flower_mat(variant) if entry["kind"] == "wildflower" else Props._grass_mat(variant)
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	if entry.get("range_m") != null:
		mi.visibility_range_end = float(entry["range_m"])
	g.add_child(mi)
