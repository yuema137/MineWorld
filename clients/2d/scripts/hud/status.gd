extends CanvasLayer

## The status line: who you are, where, the world's time, which world and revision, and whether the
## connection holds (step-13 §4.3 point 8). Everything it shows was told to this client; it states
## nothing of its own.

var _who: Label
var _when: Label
var _note: Label
var _note_until := 0.0


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
	_note = _line(lines, 13)
	var hint := _line(lines, 12)
	hint.text = "click to walk  ·  WASD or arrows  ·  Esc quits"
	add_child(panel)


func _line(parent: Node, size: int) -> Label:
	var label := Label.new()
	label.add_theme_color_override("font_color", Color("4b3826"))
	label.add_theme_font_size_override("font_size", size)
	parent.add_child(label)
	return label


## Refreshes the lines from the newest observation and the connection's state.
func show_state(observation: MineWorldObservation, link: Node, revision: Variant) -> void:
	if observation != null:
		var me := observation.observer()
		var name := observation.display_name(me)
		var place := observation.entity(observation.place())
		var tags: Array = place.get("tags", [])
		_who.text = "%s  ·  %s" % [name if name != "" else me,
			", ".join(PackedStringArray(tags)) if not tags.is_empty() else observation.place()]
		_when.text = "%s  ·  revision %s  ·  world …%s  ·  %s" % [
			clock(observation.at()), "—" if revision == null else str(int(revision)),
			link.instance.right(8), link.state]
	else:
		_when.text = link.state


## Shows a short message — the world's answer to something — for a few seconds.
func note(text: String) -> void:
	_note.text = text
	_note_until = Time.get_ticks_msec() / 1000.0 + 4.0


func _process(_delta: float) -> void:
	if _note.text != "" and Time.get_ticks_msec() / 1000.0 > _note_until:
		_note.text = ""


## World seconds as "day N  hh:mm".
static func clock(seconds: int) -> String:
	var day := seconds / 86400 + 1
	var in_day := seconds % 86400
	return "day %d  %02d:%02d" % [day, in_day / 3600, (in_day % 3600) / 60]
