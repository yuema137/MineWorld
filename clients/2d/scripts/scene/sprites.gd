extends RefCounted

## Turning a pack's sprite into a node that stands on the ground, and the plain shapes drawn when a
## role is unbound. Shared by `people.gd` and `places.gd`; it knows art mechanics (anchors, scale,
## the ink contour) and nothing about the world.

const PlainShape := preload("res://scripts/scene/plain_shape.gd")

var presentation
var _ink_materials: Dictionary = {}


func _init(the_presentation) -> void:
	presentation = the_presentation


## A Sprite2D for a loaded sprite, offset so its anchor (or, with `by_door`, its door) sits on the
## node's origin. Returns null for an empty sprite.
func make(sprite: Dictionary, by_door := false, flip := false) -> Sprite2D:
	if sprite.is_empty():
		return null
	var s := Sprite2D.new()
	s.texture = sprite["texture"]
	s.centered = false
	s.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS
	var pin: Vector2 = sprite["anchor"]
	if by_door and sprite.get("door") != null:
		pin = sprite["door"]
	s.offset = -pin
	var k: float = sprite["scale"]
	s.scale = Vector2(-k if flip else k, k)
	if flip:
		s.offset.x = -s.offset.x - float(s.texture.get_width()) + 2.0 * pin.x
	s.material = ink(k, String(sprite["ink"]))
	return s


## The ink contour for a sprite drawn at scale `k` (the spike's rule: about the same thickness on
## screen everywhere, shared between sprites that land on the same width). null when the pack has no
## ink effect or the sprite carries its own line.
func ink(k: float, ink_class: String) -> ShaderMaterial:
	if ink_class == "none" or presentation.is_plain():
		return null
	var shader: Shader = presentation.shader(String(presentation.setting(["effects", "ink", "shader"], "")))
	if shader == null:
		return null
	var px := float(presentation.setting(["effects", "ink", "px"], 1.35))
	var bite := float(presentation.setting(["effects", "ink", "bite", ink_class], 0.24))
	var texels := clampf(px / maxf(k, 0.02), 1.5, 14.0)
	var width := roundf(texels * 2.0) / 2.0
	var key := "%s:%.1f" % [ink_class, width]
	if not _ink_materials.has(key):
		var m := ShaderMaterial.new()
		m.shader = shader
		m.set_shader_parameter("width", width)
		m.set_shader_parameter("strength", bite)
		_ink_materials[key] = m
	return _ink_materials[key]


## A plain shape (`person`, `player`, `block`, `prop`) for an unbound role.
static func plain(kind: String, size := Vector2(1, 1), colour := Color(0.55, 0.5, 0.45)) -> Node2D:
	var shape: Node2D = PlainShape.new()
	shape.kind = kind
	shape.size = size
	shape.colour = colour
	return shape
