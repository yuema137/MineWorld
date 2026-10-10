extends CanvasLayer

## The status line: who you are, where, the world's time, which world and revision, and whether the
## connection holds (step-13 §4.3 point 8). Everything it shows was told to this client; it states
## nothing of its own. Its text is the pack's wording (`ARC-70`).

const Words := preload("res://scripts/hud/words.gd")

var _who: Label
var _when: Label


func _ready() -> void:
	layer = 100
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.84)
	style.set_corner_radius_all(10)
	style.content_margin_left = 14
	style.content_margin_right = 14
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", style)
	panel.position = Vector2(20, 18)
	var lines := VBoxContainer.new()
	panel.add_child(lines)
	_who = _line(lines, 17)
	_when = _line(lines, 13)
	var hint := _line(lines, 12)
	hint.text = Words.text("ui.hint")
	add_child(panel)


func _line(parent: Node, size: int) -> Label:
	var label := Label.new()
	label.add_theme_color_override("font_color", Color("4b3826"))
	label.add_theme_font_size_override("font_size", size)
	parent.add_child(label)
	return label


## Refreshes the lines from the newest observation and the connection's state.
func show_state(observation: MineWorldObservation, link: Node, revision: Variant) -> void:
	var state := Words.text("ui.state." + String(link.state))
	if observation != null:
		var me := observation.observer()
		var name := observation.display_name(me)
		var place := observation.entity(observation.place())
		var tags: Array = place.get("tags", [])
		_who.text = Words.text("ui.status.who", {"name": name if name != "" else me,
			"place": ", ".join(PackedStringArray(tags)) if not tags.is_empty() else observation.place()})
		_when.text = Words.text("ui.status.when", {"clock": Words.clock(observation.at()),
			"revision": "—" if revision == null else str(int(revision)),
			"instance": link.instance.right(8), "state": state})
	else:
		_when.text = state
