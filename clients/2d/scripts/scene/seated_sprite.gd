extends RefCounted

## Seated atlas selection and the approved hip-crop fallback (S34-e SD-e-4).
## Reconciliation and gait stay in people.gd; this constructs only a drawable sprite.
## Greater trochanter / stature: Drillis and Contini (1966), via Winter, 4th ed., Fig. 4.1.
const HIP_RATIO := 0.530


static func make(sprites, projection, person: Dictionary, pose: Dictionary, view: int) -> Dictionary:
	var views: Dictionary = person["views"]
	if views.is_empty():
		return {}
	var seated: Dictionary = views.get("seated", {})
	if not seated.is_empty() and typeof(seated.get("sheet")) == TYPE_DICTIONARY \
			and seated.get("seat_height_m") != null:
		return _sheet(sprites, projection, seated, pose, view)
	var layout: Dictionary = person["sheet"]
	var source: Dictionary = person["loaded"]
	if layout.is_empty() and view >= 3 and not (views.get("back", {}) as Dictionary).is_empty():
		source = views["back"]
	if source.is_empty():
		return {}
	var columns := int(layout.get("columns", 1))
	var names: Array = layout.get("views", ["down"])
	var texture: Texture2D = source["texture"]
	var cell := Vector2i(texture.get_width() / columns, texture.get_height() / names.size())
	var origin := Vector2i(0, row_of(names, view) * cell.y)
	var image := texture.get_image()
	var used := image.get_region(Rect2i(origin, cell)).get_used_rect()
	var scale := absf(float(source["scale"]))
	# Known stature locates the hip above the authored floor anchor. Without it use opaque height.
	var stature: Variant = source.get("height_m")
	var hip := float(source["anchor"].y) - HIP_RATIO * float(used.size.y)
	if stature != null and float(stature) > 0.0:
		hip = float(source["anchor"].y) - HIP_RATIO * projection.height_px(float(stature)) / scale
	var crop_h := clampf(hip - used.position.y, 1.0, float(cell.y - used.position.y))
	var node: Sprite2D = sprites.make(source, false, sprites.faces_left(source))
	node.region_enabled = true
	node.region_rect = Rect2(0, origin.y + used.position.y, cell.x, crop_h)
	node.offset = Vector2(-float(source["anchor"].x), -crop_h)
	var lift: float = projection.height_px(float(pose["seat_height_m"]))
	node.position = Vector2(0, -lift)
	return {"node": node, "top": lift + crop_h * scale}


static func _sheet(sprites, projection, source: Dictionary, pose: Dictionary, view: int) -> Dictionary:
	var layout: Dictionary = source["sheet"]
	var names: Array = layout["views"]
	var node: Sprite2D = sprites.make(source, false, sprites.faces_left(source))
	node.hframes = int(layout["columns"])
	node.vframes = names.size()
	node.frame = row_of(names, view) * node.hframes
	var lift: float = projection.height_px(float(pose["seat_height_m"]) - float(source["seat_height_m"]))
	node.position = Vector2(0, -lift)
	return {"node": node, "top": float(source["anchor"].y) * absf(node.scale.y) + lift}


static func row_of(names: Array, view: int) -> int:
	var wanted: String = ["down", "down_right", "right", "up_right", "up"][clampi(view, 0, 4)]
	var row := names.find(wanted)
	return row if row >= 0 else clampi(view, 0, names.size() - 1)
