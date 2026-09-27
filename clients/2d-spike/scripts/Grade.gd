extends CanvasLayer

## The paper-and-grade pass: a full-screen quad running art/grade.gdshader
## over the finished frame.
##
## It sits on its own CanvasLayer above the world and below the HUD, so the
## scene is graded and the interface is not.

var _rect: ColorRect
var _mat: ShaderMaterial


func _ready() -> void:
	layer = 50
	_mat = ShaderMaterial.new()
	_mat.shader = load("res://art/grade.gdshader")
	_rect = ColorRect.new()
	_rect.material = _mat
	_rect.set_anchors_preset(Control.PRESET_FULL_RECT)
	_rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_rect)
	get_viewport().size_changed.connect(_fit)
	_fit()


func _fit() -> void:
	if _rect:
		_rect.size = get_viewport().get_visible_rect().size


## The coarse grain is anchored to the world so it does not swim across the
## ground as the camera moves; the camera tells it where the world is.
func set_world_offset(p: Vector2) -> void:
	if _mat:
		_mat.set_shader_parameter("world_off", p)
