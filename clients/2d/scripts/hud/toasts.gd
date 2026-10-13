extends CanvasLayer

## Short messages: the world's answer to something the player asked (step-13 §15.4 D-b-6). Each
## stays a few seconds; one may carry a button, which is an ordinary choice the player can make —
## after a `too_far_away` rejection, the same "walk to" entry the menu offers (D-b-3).

## How long a message stays, in seconds.
const SHOWN_S := 5.0

## A message was shown: its kind (`accepted`, `rejected`, `unavailable`, `refused`, `unknown`), what
## it was about, its text and its button's text (or "").
signal toasted(record: Dictionary)

var _column: VBoxContainer
var _live: Array = []


func _ready() -> void:
	layer = 95
	var holder := Control.new()
	holder.set_anchors_preset(Control.PRESET_FULL_RECT)
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(holder)
	_column = VBoxContainer.new()
	_column.anchor_left = 0.5
	_column.anchor_right = 0.5
	_column.anchor_top = 1.0
	_column.anchor_bottom = 1.0
	_column.position = Vector2(-260, -150)
	_column.custom_minimum_size = Vector2(520, 0)
	_column.add_theme_constant_override("separation", 6)
	holder.add_child(_column)


## Shows `said`; with `button` and `on_press`, a button that runs it once.
func toast(record: Dictionary, said: String, button: String = "", on_press: Callable = Callable()) -> void:
	var box := PanelContainer.new()
	var style := StyleBoxFlat.new()
	var good := String(record.get("kind", "")) == "accepted"
	style.bg_color = Color(0.93, 0.97, 0.9, 0.94) if good else Color(0.99, 0.93, 0.88, 0.94)
	style.set_corner_radius_all(8)
	style.content_margin_left = 14
	style.content_margin_right = 14
	style.content_margin_top = 7
	style.content_margin_bottom = 7
	box.add_theme_stylebox_override("panel", style)
	var row := HBoxContainer.new()
	box.add_child(row)
	var label := Label.new()
	label.text = said
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.add_theme_font_size_override("font_size", 14)
	label.add_theme_color_override("font_color", Color("3b2c1e"))
	row.add_child(label)
	if button != "" and on_press.is_valid():
		var press := Button.new()
		press.text = button
		press.pressed.connect(func() -> void:
			on_press.call()
			box.queue_free())
		row.add_child(press)
	_column.add_child(box)
	_live.append({"node": box, "until": _now() + SHOWN_S})
	var shown := record.duplicate()
	shown["text"] = said
	shown["button"] = button
	toasted.emit(shown)


## Removes every message shown: after a language change none is left in the old language (S20).
func clear() -> void:
	for entry in _live:
		if is_instance_valid(entry["node"]):
			entry["node"].queue_free()
	_live.clear()


func _process(_delta: float) -> void:
	var now := _now()
	for entry in _live.duplicate():
		if now > entry["until"]:
			if is_instance_valid(entry["node"]):
				entry["node"].queue_free()
			_live.erase(entry)


func _now() -> float:
	return Time.get_ticks_msec() / 1000.0
