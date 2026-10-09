extends CanvasLayer

## The pack's full-screen post-process (the `grade` effect: paper tooth and colour grade, the spike's
## `Grade.gd`), on its own layer above the world and below the status line. Absent when the pack
## names no such effect.

var _material: ShaderMaterial


func setup(presentation) -> bool:
	layer = 50
	var shader: Shader = presentation.shader(String(presentation.setting(["effects", "grade", "shader"], "")))
	if shader == null:
		return false
	_material = ShaderMaterial.new()
	_material.shader = shader
	var paper: Texture2D = presentation.texture(String(presentation.setting(["effects", "grade", "paper"], "")))
	if paper != null:
		_material.set_shader_parameter("paper", paper)
		_material.set_shader_parameter("paper_size", Vector2(paper.get_width(), paper.get_height()))
	var rect := ColorRect.new()
	rect.material = _material
	rect.set_anchors_preset(Control.PRESET_FULL_RECT)
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(rect)
	return true


## The coarse grain is anchored to the world, so it does not swim as the camera moves.
func set_world_offset(at: Vector2) -> void:
	if _material != null:
		_material.set_shader_parameter("world_off", at)
