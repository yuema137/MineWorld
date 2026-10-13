extends CanvasLayer

## The panels: what the world disclosed to this observer, drawn as the readers read it (step-13
## §15.4 D-b-7). Two groups the player toggles — I for one's things (money, carrying, the shop here,
## the day, work, anything else told), H for conversations (heard lately, people known, invitations).
## Rebuilt from every observation: a panel is drawn only while its component is in the newest frame.

## Which group a component's panel belongs to; any other is a thing.
const TALK := ["conversation-history", "acquaintances", "invitations"]

signal changed(panels: Array)

## The panels of the newest observation, as `readers.gd` built them.
var current: Array = []
var _signature := ""
var _things: VBoxContainer
var _talk: VBoxContainer


func _ready() -> void:
	layer = 90
	_things = _column(Vector2(-336, 18))
	_talk = _column(Vector2(-336, 330))
	_things.visible = false
	_talk.visible = false


func _column(offset: Vector2) -> VBoxContainer:
	var column := VBoxContainer.new()
	column.anchor_left = 1.0
	column.anchor_right = 1.0
	column.position = offset
	column.custom_minimum_size = Vector2(316, 0)
	column.add_theme_constant_override("separation", 8)
	var holder := Control.new()
	holder.set_anchors_preset(Control.PRESET_FULL_RECT)
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	holder.add_child(column)
	add_child(holder)
	return column


## Shows the panels of one observation.
func show_panels(panels: Array) -> void:
	current = panels
	var signature := JSON.stringify(panels.map(func(p: Dictionary) -> Array: return [p["component"], p["title"], p["rows"]]))
	if signature == _signature:
		return
	_signature = signature
	for column in [_things, _talk]:
		for child in column.get_children():
			child.queue_free()
	for panel in panels:
		(_talk if TALK.has(panel["component"]) else _things).add_child(_box(panel))
	changed.emit(panels)


## I: one's things. H: conversations.
func toggle(which: String) -> void:
	var column := _talk if which == "conversations" else _things
	column.visible = not column.visible


func is_shown(which: String) -> bool:
	return (_talk if which == "conversations" else _things).visible


func _box(panel: Dictionary) -> PanelContainer:
	var box := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.9)
	style.set_corner_radius_all(8)
	style.content_margin_left = 12
	style.content_margin_right = 12
	style.content_margin_top = 6
	style.content_margin_bottom = 8
	box.add_theme_stylebox_override("panel", style)
	var lines := VBoxContainer.new()
	box.add_child(lines)
	lines.add_child(_label(panel["title"], 14, Color("6b4e2e")))
	for row in panel["rows"]:
		lines.add_child(_label(row, 13, Color("3b2c1e")))
	return box


static func _label(said: String, size: int, colour: Color) -> Label:
	var label := Label.new()
	label.text = said
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size = Vector2(292, 0)
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", colour)
	return label
