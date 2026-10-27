extends RefCounted

## The fittings of a drawn place that follow disclosed state, and the words drawn over them
## (`clients/2d/PRESENTATION.md` §4, `state` and `label`; pr-s29f §5.6).
##
## The state comes from `MineWorldStationState` (`ARC-108`), the one rule both clients share; this
## file only chooses a picture and a word from it:
##
##     { "role": "prop:kiosk", "state": "kiosk" }   kiosk "shut" -> prop:kiosk:shut (else prop:kiosk)
##     { "label": "board" }                        board "none" -> "No service yet"; lines -> one per line
##     { "label": "window" }                       window "shut" -> "Closed"; open -> no word
##
## A fitting of the place the observer stands in follows that place's observation; any other place's
## shows what nothing disclosed shows (shut, blank). It names no place and no world.

const Words := preload("res://scripts/hud/words.gd")
const State := preload("res://mineworld_station/station_state.gd")

var presentation
var sprites
var projection
## One row per stateful or labelled fitting: {place, item, holder, role, label (Label or null)}.
var rows: Array = []
## The observer's place and its fixture state, from the latest observation.
var here := ""
var here_state: Dictionary = State.nothing_disclosed()


func _init(the_presentation, the_sprites, the_projection) -> void:
	presentation = the_presentation
	sprites = the_sprites
	projection = the_projection


## Whether a fitting spec needs this file (it names a `state` or a `label`).
static func is_stateful(item: Dictionary) -> bool:
	return String(item.get("state", "")) != "" or String(item.get("label", "")) != ""


## The fixture state drawn for `place`.
func state_of(place: String) -> Dictionary:
	return here_state if place == here and place != "" else State.nothing_disclosed()


## The role a fitting is drawn by under `state`: `<role>:<value>` when the pack binds it, else `<role>`.
func role_of(item: Dictionary, state: Dictionary) -> String:
	var role := String(item.get("role", ""))
	var field := String(item.get("state", ""))
	if field != "" and state.has(field):
		var specific := "%s:%s" % [role, state[field]]
		if presentation.is_bound(specific):
			return specific
	return role


## Records a stateful fitting drawn in `holder`, and gives it its label.
func track(place: String, item: Dictionary, holder: Node2D, role: String) -> void:
	var label: Label = null
	if String(item.get("label", "")) != "":
		label = Label.new()
		label.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
		label.add_theme_font_size_override("font_size", 12)
		label.add_theme_color_override("font_color", Color("f4ead2"))
		label.add_theme_color_override("font_outline_color", Color(0.12, 0.09, 0.06, 0.95))
		label.add_theme_constant_override("outline_size", 5)
		label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		label.size = Vector2(180, 18)
		label.position = Vector2(-90, -9 - projection.height_px(float(item.get("lift_m", 0.0)) + 0.5))
		label.z_index = 40
		holder.add_child(label)
	rows.append({"place": place, "item": item, "holder": holder, "role": role, "label": label})
	_word(rows[-1])


func clear() -> void:
	rows.clear()


## Follows a new observation: re-resolves the fittings whose state changed. `make` builds a fitting's
## sprite node for a role (places.gd's drawing), so the picture rules stay in one file.
func follow(observation: MineWorldObservation, make: Callable) -> void:
	here = observation.place()
	here_state = State.of(observation)
	for row in rows:
		var role := role_of(row["item"], state_of(row["place"]))
		if role != row["role"]:
			row["role"] = role
			var holder: Node2D = row["holder"]
			for child in holder.get_children():
				if not child is Label:
					child.queue_free()
			var node: Node2D = make.call(role, row["item"])
			if node != null:
				holder.add_child(node)
		_word(row)


## The word a labelled fitting shows, for the harness's report: "" when none.
func word_of(row: Dictionary) -> String:
	var state := state_of(row["place"])
	match String(row["item"].get("label", "")):
		"board":
			var board: Dictionary = state["board"]
			if board.get("state") != State.LINES:
				return Words.text("sign.no_service")
			var lines := PackedStringArray()
			for entry in board.get("lines", []):
				var line: Variant = entry.get("line") if typeof(entry) == TYPE_DICTIONARY else null
				lines.append(String(line) if typeof(line) == TYPE_STRING else JSON.stringify(entry))
			return "\n".join(lines)
		"window":
			return Words.text("sign.window_closed") if state["window"] == State.SHUT else ""
	return ""


func _word(row: Dictionary) -> void:
	if row["label"] != null:
		row["label"].text = word_of(row)
