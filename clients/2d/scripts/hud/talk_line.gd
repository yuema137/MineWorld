extends CanvasLayer

## One line of text the player types for a request that needs it: what to say, or what to do
## together (step-13 §15.4 D-b-5). Enter sends exactly what was typed — an empty line too, the server
## decides whether it is acceptable — and Esc cancels. Suggestions only fill the line; nothing typed
## is lower-cased, trimmed or checked here (`B-6`: the server validates an activity kind).

const Words := preload("res://scripts/hud/words.gd")

var _box: PanelContainer
var _prompt: Label
var _line: LineEdit
var _suggestions: HBoxContainer
var _on_done: Callable = Callable()


func _ready() -> void:
	layer = 110
	var holder := Control.new()
	holder.set_anchors_preset(Control.PRESET_FULL_RECT)
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(holder)
	_box = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.97)
	style.set_corner_radius_all(10)
	style.content_margin_left = 14
	style.content_margin_right = 14
	style.content_margin_top = 10
	style.content_margin_bottom = 10
	_box.add_theme_stylebox_override("panel", style)
	_box.anchor_left = 0.5
	_box.anchor_right = 0.5
	_box.anchor_top = 1.0
	_box.anchor_bottom = 1.0
	_box.position = Vector2(-260, -260)
	_box.custom_minimum_size = Vector2(520, 0)
	var lines := VBoxContainer.new()
	_box.add_child(lines)
	_prompt = Label.new()
	_prompt.add_theme_color_override("font_color", Color("4b3826"))
	lines.add_child(_prompt)
	_line = LineEdit.new()
	_line.text_submitted.connect(_on_submitted)
	lines.add_child(_line)
	_suggestions = HBoxContainer.new()
	lines.add_child(_suggestions)
	var hint := Label.new()
	hint.text = Words.text("ui.input.hint")
	hint.add_theme_font_size_override("font_size", 11)
	hint.add_theme_color_override("font_color", Color("7a6650"))
	lines.add_child(hint)
	holder.add_child(_box)
	_box.visible = false


## Asks for a line: shows `prompt`, offers `suggestions`, and calls `on_done` with the text when the
## player presses Enter.
func ask(prompt: String, suggestions: PackedStringArray, on_done: Callable) -> void:
	_on_done = on_done
	_prompt.text = prompt
	_line.text = ""
	for child in _suggestions.get_children():
		child.queue_free()
	for suggestion in suggestions:
		var pick := Button.new()
		pick.text = suggestion
		pick.pressed.connect(func() -> void:
			_line.text = suggestion
			_line.grab_focus())
		_suggestions.add_child(pick)
	_box.visible = true
	_line.grab_focus()


func is_open() -> bool:
	return _box.visible


## Types `said` into the line and presses Enter: the path a keyboard takes (used by the harness).
func type_and_send(said: String) -> void:
	_line.text = said
	_line.text_submitted.emit(said)


func cancel() -> void:
	_box.visible = false
	_on_done = Callable()


func _on_submitted(said: String) -> void:
	if not _box.visible:
		return
	var done := _on_done
	cancel()
	if done.is_valid():
		done.call(said)


func _unhandled_key_input(event: InputEvent) -> void:
	if _box.visible and event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		cancel()
		get_viewport().set_input_as_handled()
