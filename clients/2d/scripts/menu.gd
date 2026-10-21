extends CanvasLayer

## The interaction menu: the newest observation's affordances for one subject, in the server's order,
## nothing removed and nothing invented (step-13 §15.4 D-b-1 … D-b-4; `ARC-47` note 1).
##
## A subject is a perceived person (their offers), the observer itself (the target-less offers), or a
## drawn loose object (my target-less offers whose payload refers to it, ARC-342). One entry per
## affordance:
##
## ```text
## complete      enabled; chosen → sent exactly as offered (intents.submit_offered)
## composable    enabled; chosen → the input it needs, then intents.compose
## aim           after an offered complete affordance whose type intents aims (an object's menu only):
##               chosen → `aim_chosen`; the next floor click sends it with that point (intents.aim_throw)
## other         listed disabled, "not supported by this client", never sent
## unavailable   greyed but still choosable, with the server's reason and declared range, unevaluated
## ```
##
## The one entry the client adds is "walk to <person>", which is the offered `move` and is listed
## only when the frame offers one. The menu is rebuilt from every observation while it is open, and
## each entry carries the affordance it was built from, so choosing sends what was shown.
##
## Verdicts are read only where entries are built, never where one is chosen (`R3'`): choosing an
## entry the server marked unavailable submits it, and the answer is the server's.

const Words := preload("res://scripts/hud/words.gd")

## The entries shown changed (on opening, and when an observation changes them): the subject and the
## entries as data — the "one world, two views" report.
signal listed(subject: String, entries: Array, offered: Array)
## The player chose an "… at…" entry: the offered affordance to aim, sent on the next floor click.
signal aim_chosen(affordance: Dictionary)

var intents: Node
var walker: Node
var readers
var talk_line: CanvasLayer

## The subject's id ("" when closed) and whether it is the observer itself.
var subject := ""
var entries: Array = []
## Who I see, about the subject (S30-f §5.9): its glance, under the title, and its inspect list, below the
## entries — the server's phrases, selected and cut by `MineWorldAttributes`; empty for my own menu.
var glance := ""
var inspect := PackedStringArray()
var _self := false
## Whether the subject is a drawn loose object.
var _object := false
var _observer := ""
var _signature := ""
var _box: PanelContainer
var _list: VBoxContainer
var _title: Label
var _glance: Label
var _inspect: VBoxContainer


func _ready() -> void:
	layer = 105
	_box = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.97)
	style.border_color = Color("b89a74")
	style.set_border_width_all(1)
	style.set_corner_radius_all(8)
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	_box.add_theme_stylebox_override("panel", style)
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 2)
	_box.add_child(column)
	_title = Label.new()
	_title.add_theme_font_size_override("font_size", 15)
	_title.add_theme_color_override("font_color", Color("6b4e2e"))
	column.add_child(_title)
	_glance = Label.new()
	_glance.add_theme_font_size_override("font_size", 12)
	_glance.add_theme_color_override("font_color", Color("7a6650"))
	_glance.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_glance.custom_minimum_size = Vector2(260, 0)
	column.add_child(_glance)
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 1)
	column.add_child(_list)
	_inspect = VBoxContainer.new()
	_inspect.add_theme_constant_override("separation", 0)
	column.add_child(_inspect)
	add_child(_box)
	_box.visible = false


func is_open() -> bool:
	return subject != ""


## Opens the menu about `id` (a perceived person, the observer for its own menu, or with `is_object` a
## drawn loose object) at a screen point.
func open(id: String, observation: MineWorldObservation, at: Vector2, is_object := false) -> void:
	subject = id
	_observer = observation.observer()
	_object = is_object
	_self = id == _observer and not is_object
	_signature = ""
	_box.position = at + Vector2(14, -10)
	_box.visible = true
	refresh(observation)


func close() -> void:
	subject = ""
	_object = false
	entries = []
	glance = ""
	inspect = PackedStringArray()
	_box.visible = false


## Rebuilds the entries from a newer observation; closes when the subject is no longer perceived.
func refresh(observation: MineWorldObservation) -> void:
	if subject == "":
		return
	if _object and not readers.loose_shapes(observation).has(subject):
		close()
		return
	if not _self and not _object and observation.entity(subject).is_empty():
		close()
		return
	var offered: Array = about_object(observation, subject) if _object \
		else observation.affordances("", "" if _self else subject)
	entries = _build(observation, offered)
	var seen := {} if _self or _object else observation.entity(subject)
	glance = MineWorldAttributes.glance(seen)
	inspect = MineWorldAttributes.inspect(MineWorldAttributes.blocks(seen))
	var signature := JSON.stringify([glance, inspect, entries.map(func(e: Dictionary) -> Array:
		return [e["label"], e["enabled"], e["available"]])])
	if signature == _signature:
		return
	_signature = signature
	_draw()
	listed.emit(subject, entries, offered)


## The player chose entry `index` (a click, Enter, or the harness — one path). Reads no verdict.
func choose(index: int) -> void:
	if index < 0 or index >= entries.size():
		return
	var entry: Dictionary = entries[index]
	if not entry["enabled"]:
		return
	var affordance: Dictionary = entry["affordance"]
	var target := subject
	close()
	match String(entry["kind"]):
		"walk":
			walker.approach(target)
		"complete":
			intents.submit_offered(affordance)
		"aim":
			aim_chosen.emit(affordance)
		"composed":
			var input: Dictionary = intents.input_for(entry["action_type"])
			if not input.has("prompt"):
				intents.compose(affordance, null)
				return
			var suggestions := PackedStringArray()
			if input.has("suggest"):
				# One key, whose text lists the suggestions separated by commas, or several keys, each a suggestion.
				var keys: Array = input["suggest"] if input["suggest"] is Array else [input["suggest"]]
				for key in keys:
					for s in Words.text(key).split(","):
						suggestions.append(s.strip_edges())
			talk_line.ask(Words.text(input["prompt"], {"target": readers.person(target)}), suggestions,
				func(said: String) -> void: intents.compose(affordance, said))


## Entries for the offers, in their order, after "walk to" when it applies. Asks `intents` what it can
## compose; reads no verdict itself (`_entry` does).
func _build(observation: MineWorldObservation, offered: Array) -> Array:
	var built: Array = []
	if not _self and not _object:
		var walk: Dictionary = intents.offered_walk(observation)
		if not walk.is_empty():
			built.append(_entry(walk, "walk", Words.text("ui.walk-to", {"target": readers.person(subject)})))
	built.append_array(_offer_entries(offered, _object))
	for i in built.size():
		built[i]["index"] = i
	return built


## The entries the menu lists for `id` on `observation`, built as `open` builds them, without opening
## it: the scripted checks' report of what each menu offers (PR 13f §4.2). The "walk to" entry a
## person's menu leads with is left out; an object's "… at…" entries are kept, marked `aim`.
func entries_for(observation: MineWorldObservation, id: String, is_object := false) -> Array:
	var as_self := id == observation.observer() and not is_object
	var offered: Array = about_object(observation, id) if is_object \
		else observation.affordances("", "" if as_self else id)
	return _offer_entries(offered, is_object)


## One entry per offer, in the server's order; after an object's complete aiming offer, its "… at…".
func _offer_entries(offered: Array, is_object: bool) -> Array:
	var built: Array = []
	for affordance in offered:
		var type := String(affordance.get("action_type", ""))
		var kind := "unsupported"
		var aims: bool = intents.input_for(type).has("aim")
		if MineWorldObservation.is_complete(affordance):
			kind = "complete"
		elif intents.composes(type) and not aims:
			kind = "point" if intents.input_for(type).has("point") else "composed"
		built.append(_entry(affordance, kind, Words.action(type, _names(affordance))))
		if is_object and kind == "complete" and aims:
			built.append(_entry(affordance, "aim", Words.text("ui.throw-at", _names(affordance))))
	return built


## One entry: what the affordance is, the server's verdict on it as data, and its label.
func _entry(affordance: Dictionary, kind: String, label: String) -> Dictionary:
	var available := bool(affordance.get("available", false))
	var reason: Variant = affordance.get("unavailable_reason")
	var shown := label
	if kind == "unsupported" or kind == "point":
		shown = Words.text("ui.unsupported", {"label": label}) if kind == "unsupported" else label
	elif not available:
		shown = Words.text("ui.unavailable", {"label": label, "reason": Words.reason(reason)})
		var declared: Variant = affordance.get("requirement")
		if typeof(declared) == TYPE_DICTIONARY and declared.get("within_range") != null:
			var metres := float(declared["within_range"]) / 1000.0
			shown += "  ·  " + Words.text("ui.needs-range",
				{"metres": str(int(metres)) if metres == floorf(metres) else String.num(metres, 1)})
	var target: Variant = affordance.get("target")
	return {"action_type": String(affordance.get("action_type", "")), "kind": kind,
		"target": null if target == null else String(target), "about": _about(affordance),
		"complete": MineWorldObservation.is_complete(affordance), "available": available,
		"reason": reason, "label": shown, "enabled": kind != "unsupported" and kind != "point",
		"affordance": affordance}


## The names an affordance concerns, for its label: its target's and its payload's first entity's.
func _names(affordance: Dictionary) -> Dictionary:
	var target: Variant = affordance.get("target")
	var about := _about(affordance)
	return {"target": "" if target == null else readers.person(String(target)),
		"item": "" if about == "" else readers.thing(about)}


## My target-less offers whose payload refers to the loose object `id`, in the server's order: the
## object menu's offers (presentation only — my own menu still lists every one of them).
static func about_object(observation: MineWorldObservation, id: String) -> Array:
	return observation.affordances("", "").filter(func(a: Dictionary) -> bool: return _about(a) == id)


## The first entity a payload refers to by a typed reference, or "".
static func _about(affordance: Dictionary) -> String:
	var payload: Variant = affordance.get("payload")
	if typeof(payload) != TYPE_DICTIONARY:
		return ""
	for value in payload.values():
		if typeof(value) == TYPE_DICTIONARY and typeof(value.get("entity")) == TYPE_STRING:
			return value["entity"]
	return ""


func _draw() -> void:
	_title.text = Words.text("ui.self") if _self else (readers.thing(subject) if _object else readers.person(subject))
	_glance.text = glance
	_glance.visible = glance != ""
	for child in _inspect.get_children():
		child.queue_free()
	if not inspect.is_empty():
		_inspect.add_child(HSeparator.new())
		var heading := Label.new()
		heading.text = Words.text("panel.subject")
		heading.add_theme_font_size_override("font_size", 12)
		heading.add_theme_color_override("font_color", Color("6b4e2e"))
		_inspect.add_child(heading)
		for line in inspect:
			var row := Label.new()
			row.text = line
			row.add_theme_font_size_override("font_size", 12)
			row.add_theme_color_override("font_color", Color("3b2c1e"))
			_inspect.add_child(row)
	for child in _list.get_children():
		child.queue_free()
	if entries.is_empty():
		var none := Label.new()
		none.text = Words.text("ui.menu.empty")
		none.add_theme_color_override("font_color", Color("7a6650"))
		_list.add_child(none)
		return
	# A separator where the kind of thing offered changes between neighbours: presentation only, the
	# order stays the server's.
	var previous := ""
	var first: Button = null
	for entry in entries:
		var group: String = "\n" if entry["kind"] == "walk" else entry["action_type"]
		if previous != "" and group != previous:
			_list.add_child(HSeparator.new())
		previous = group
		var button := Button.new()
		button.text = entry["label"]
		button.flat = true
		for colour in ["font_color", "font_hover_color", "font_focus_color", "font_pressed_color"]:
			button.add_theme_color_override(colour, Color("3b2c1e"))
		button.add_theme_color_override("font_disabled_color", Color("9a8a78"))
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.disabled = not entry["enabled"]
		if entry["enabled"] and not entry["available"]:
			button.modulate = Color(1, 1, 1, 0.55)
		var index: int = entry["index"]
		button.pressed.connect(func() -> void: choose(index))
		_list.add_child(button)
		if first == null and entry["enabled"]:
			first = button
	if first != null:
		first.grab_focus.call_deferred()
	_keep_on_screen.call_deferred()


## Moves the box back inside the window once its size is known: a menu opened near an edge.
func _keep_on_screen() -> void:
	_box.reset_size()
	var room: Vector2 = _box.get_viewport_rect().size - _box.size - Vector2(12, 12)
	_box.position = _box.position.clamp(Vector2(12, 12), room.max(Vector2(12, 12)))
