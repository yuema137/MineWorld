## Overhead names that never cover one another (promo v2: Bob's and Alice's names overlapped behind
## the counter, after #384 made them fixed-size captions).
##
## The standard answer of map renderers to the same problem (MapLibre / Mapbox GL symbol placement,
## OpenLayers' `declutter`): place labels greedily in priority order, each against the screen
## rectangles of the labels already placed. Here the priority is distance -- the nearest person's name
## is placed first and never moves; a farther name that would overlap is lifted up the screen, above
## the name it hits, by at most MAX_LIFT_LINES of its own height; one that still overlaps is faded
## out. Godot has no collision placement for Label3D (it sorts and depth-tests labels, nothing more),
## and the greedy pass is a few dozen lines over Rect2, so it is written here rather than adopted.
##
## Presentation only. Nothing here reads or decides world state: it moves the drawn caption, never the
## person, and it is deterministic -- the same frame places the same way.
class_name SliceLabelDeclutter
extends RefCounted

## How far a name may be lifted above where it belongs, in its own line heights; past it, it fades.
const MAX_LIFT_LINES := 2.5
## Clear space kept between two placed captions, screen pixels.
const GAP_PX := 3.0
## How fast a caption glides to its placed offset and fades in or out, per second: a lift is shown
## moving, never a jump, so a crowd passing does not flicker.
const SETTLE_RATE := 10.0
## Bounded work per label: every overlap found lifts it past one placed rectangle.
const MAX_TRIES := 12


## The placement of `items`, each `{rect: Rect2 (screen pixels, y down, where it belongs),
## priority: float (lower is placed first)}`. Returns one `{lift: float (pixels up), shown: bool}` per
## item, in the input order. Ties keep the input order, so the answer is a function of the input.
static func place(items: Array) -> Array:
	var order := range(items.size())
	order.sort_custom(func(a: int, b: int) -> bool:
		var pa: float = items[a]["priority"]
		var pb: float = items[b]["priority"]
		return pa < pb or (pa == pb and a < b))
	var out: Array = []
	out.resize(items.size())
	var placed: Array[Rect2] = []
	for i in order:
		var rest: Rect2 = items[i]["rect"]
		var lift := 0.0
		var shown := true
		for _try in MAX_TRIES:
			var at := Rect2(rest.position - Vector2(0, lift), rest.size)
			var hit: Variant = _first_overlap(at, placed)
			if hit == null:
				break
			lift += at.end.y - (hit as Rect2).position.y + GAP_PX
			if lift > MAX_LIFT_LINES * rest.size.y:
				shown = false
				break
		if shown and _first_overlap(Rect2(rest.position - Vector2(0, lift), rest.size), placed) != null:
			shown = false
		if shown:
			placed.append(Rect2(rest.position - Vector2(0, lift), rest.size))
		out[i] = {"lift": lift if shown else 0.0, "shown": shown}
	return out


static func _first_overlap(rect: Rect2, placed: Array[Rect2]) -> Variant:
	var grown := rect.grow(GAP_PX * 0.5)
	for other in placed:
		if grown.intersects(other.grow(GAP_PX * 0.5)):
			return other
	return null


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
	var placement := place(items)
	var step := 1.0 - exp(-SETTLE_RATE * maxf(delta, 0.0))
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
