class_name SliceInspectPanel
extends CanvasLayer

## The inspect panel (S30-f §5.9): while the inspect control is held, every phrase the world disclosed
## about the person aimed at (the inspect list), or my own sheet when nobody is aimed at. The lines are
## `MineWorldAttributes`' output only: server phrases, selected and listed, never worded or decided here.

var _box: PanelContainer
var _title: Label
var _lines: VBoxContainer
## The lines shown now, for the probe; empty while hidden.
var shown := PackedStringArray()


func _ready() -> void:
	layer = 104
	_box = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.08, 0.07, 0.06, 0.82)
	style.set_corner_radius_all(6)
	style.content_margin_left = 12
	style.content_margin_right = 12
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	_box.add_theme_stylebox_override("panel", style)
	_box.position = Vector2(24, 120)
	var column := VBoxContainer.new()
	_box.add_child(column)
	_title = Label.new()
	_title.add_theme_font_size_override("font_size", 16)
	column.add_child(_title)
	_lines = VBoxContainer.new()
	column.add_child(_lines)
	add_child(_box)
	_box.visible = false


## Shows `lines` under `title`; nothing to show hides the panel.
func present(title: String, lines: PackedStringArray) -> void:
	var wanted := not lines.is_empty()
	if lines == shown and _box.visible == wanted:
		return
	shown = lines
	_title.text = title
	for child in _lines.get_children():
		child.queue_free()
	for line in lines:
		var row := Label.new()
		row.text = line
		row.add_theme_font_size_override("font_size", 14)
		_lines.add_child(row)
	_box.visible = wanted


func hide_panel() -> void:
	shown = PackedStringArray()
	_box.visible = false


## What the panel lists for `observation`: the inspect list of `aimed` (a person's id), or my own sheet
## when `aimed` is "".
static func lines_for(observation: MineWorldObservation, aimed: String) -> PackedStringArray:
	if observation == null:
		return PackedStringArray()
	if aimed == "":
		return MineWorldAttributes.sheet(MineWorldAttributes.blocks(observation.entity(observation.observer())))
	return MineWorldAttributes.inspect(MineWorldAttributes.blocks(observation.entity(aimed)))
