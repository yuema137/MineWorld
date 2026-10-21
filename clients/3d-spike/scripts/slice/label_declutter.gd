## Overhead names that never cover one another in the 3D client (promo v2: Bob's and Alice's names
## overlapped behind the counter, after #384 made them fixed-size captions). The placement itself is the
## shared `MineWorldLabelPlacement` (`clients/shared/labels/`, also used by the 2D client); this file
## projects each figure's Label3D stack to screen rectangles, hands them over, and glides the result.
##
## Presentation only. Nothing here reads or decides world state: it moves the drawn caption, never the
## person, and it is deterministic -- the same frame places the same way.
class_name SliceLabelDeclutter
extends RefCounted

const Placement := preload("res://mineworld_labels/label_placement.gd")

## One frame for the captions of `figures` (Node3D each; their Label3D children with `fixed_size`),
## seen by `camera`: each figure's visible captions are one stack, placed by its distance and moved
## together. A caption's own offset when first seen (the Zz one line above the name) is kept as its
## base, and the lift is added to it.
static func settle(figures: Array, camera: Camera3D, delta: float) -> void:
	if camera == null:
		return
	var viewport_h := camera.get_viewport().get_visible_rect().size.y
	var tan_half := tan(deg_to_rad(camera.fov) * 0.5)
	var stacks: Array = []
	var items: Array = []
	for figure in figures:
		if not is_instance_valid(figure):
			continue
		var stack := _stack(figure as Node3D, camera, viewport_h, tan_half)
		if stack.is_empty():
			continue
		stacks.append(stack)
		items.append({"rect": stack["rect"], "priority": stack["distance"]})
	var placement := Placement.place(items)
	var step := Placement.step(delta)
	for s in stacks.size():
		var lift_px: float = placement[s]["lift"]
		var shown: bool = placement[s]["shown"]
		for label in stacks[s]["labels"]:
			var l := label as Label3D
			var base: Vector2 = l.get_meta("base_offset")
			# a fixed-size caption's pixel unit is `pixel_size` metres at 1 m from the camera
			var per_px := 2.0 * tan_half / (l.pixel_size * viewport_h)
			var want := base.y + lift_px * per_px
			l.offset.y = lerpf(l.offset.y, want, step)
			l.transparency = lerpf(l.transparency, 0.0 if shown else 1.0, step)


## A figure's captions that are drawn now, and their union on screen where they belong; {} when none.
static func _stack(figure: Node3D, camera: Camera3D, viewport_h: float, tan_half: float) -> Dictionary:
	var labels: Array = []
	var rect := Rect2()
	var distance := INF
	for child in figure.get_children():
		var l := child as Label3D
		if l == null or not l.fixed_size or not l.is_visible_in_tree() or l.text == "":
			continue
		if not l.has_meta("base_offset"):
			l.set_meta("base_offset", l.offset)
		var d := camera.global_position.distance_to(l.global_position)
		if camera.is_position_behind(l.global_position) or (l.visibility_range_end > 0.0 and d > l.visibility_range_end):
			continue
		var px_per_unit := l.pixel_size * viewport_h / (2.0 * tan_half)
		var font := l.font if l.font != null else ThemeDB.fallback_font
		var text_size := font.get_multiline_string_size(l.text, HORIZONTAL_ALIGNMENT_CENTER, -1, l.font_size)
		var size := (text_size + Vector2.ONE * 2.0 * l.outline_size) * px_per_unit
		var base: Vector2 = l.get_meta("base_offset")
		var centre := camera.unproject_position(l.global_position) - Vector2(0, base.y * px_per_unit)
		var r := Rect2(centre - size * 0.5, size)
		rect = r if labels.is_empty() else rect.merge(r)
		labels.append(l)
		distance = minf(distance, d)
	if labels.is_empty():
		return {}
	return {"labels": labels, "rect": rect, "distance": distance}
