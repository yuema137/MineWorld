extends Node

## Speech bubbles and activity notices from the facts an observer learned (PR 13e, `ARC-343`).
##
## Every observation carries `events`, the facts its observer learned since the previous frame
## (`server/PROTOCOL.md` §5.2). Which of them are shown, and how, is the Presentation Pack's
## `renderer/godot.yaml` `events` table (`clients/2d/PRESENTATION.md` §4.1): this script names no event
## type and no payload field. A type the table does not name is skipped and counted, never an error.
##
## ```text
## bubble   over the event's first subject (`subjects[0]`), if that person is drawn; the text is the
##          payload field the table names; a newer bubble over the same person replaces it
## notice   a toast worded `ui.event.<event_type>` with {who} (the subjects' names) and the payload's
##          top-level strings and whole numbers
## once     a fact is shown at most once per world instance, by its id, across reconnects
## ```
##
## It decides nothing about the world: the server already chose who perceived the fact.

const Words := preload("res://scripts/hud/words.gd")

## A fact was drawn as a bubble: `{id, event_type, who, text}`.
signal bubbled(record: Dictionary)
## A fact was shown as a notice: `{id, event_type, who, text}`.
signal noticed(record: Dictionary)
## A fact was not shown: `{id, event_type, why}` — `unlisted` (the table does not name its type) or
## `undrawn` (a bubble whose person is not drawn).
signal skipped(record: Dictionary)

## The wording key family of a notice: the key is this prefix and the event type (PRESENTATION.md §7).
const NOTICE_KEY := "ui.event."
## How far above a figure's middle a bubble's bottom sits, in pixels at zoom 1, beyond half its height.
const LIFT_PX := 14.0

var shown := 0
var skipped_count := 0

var _people: Node
var _toasts: CanvasLayer
var _world: Node2D
var _types: Dictionary = {}
var _bubble_s := 6.0
var _notice_s := 5.0
var _instance := ""
var _seen := {}
## Who has a bubble now: person id → {node, until}.
var _bubbles := {}


func setup(people: Node, toasts: CanvasLayer, presentation, world: Node2D) -> void:
	_people = people
	_toasts = toasts
	_world = world
	var table: Variant = presentation.setting(["events", "types"], {})
	_types = table if typeof(table) == TYPE_DICTIONARY else {}
	_bubble_s = float(presentation.setting(["events", "bubble_seconds"], 6.0))
	_notice_s = float(presentation.setting(["events", "notice_seconds"], 5.0))


## A welcome: a new world instance forgets which facts it showed; the same one keeps them, so a
## reconnect never shows a fact twice (I-13e-6).
func reset(instance: String) -> void:
	if instance == _instance:
		return
	_instance = instance
	_seen.clear()
	for who in _bubbles.keys():
		_drop(who)


## Shows what the table says of each fact in `observation` not yet shown.
func present(observation: MineWorldObservation) -> void:
	for event in observation.events():
		if typeof(event) != TYPE_DICTIONARY:
			continue
		var id := String(event.get("id", ""))
		if id == "" or _seen.has(id):
			continue
		_seen[id] = true
		var type := String(event.get("event_type", ""))
		var how: Variant = _types.get(type)
		if typeof(how) != TYPE_DICTIONARY:
			_skip(id, type, "unlisted")
			continue
		var subjects: Array = event.get("subjects", []) if typeof(event.get("subjects")) == TYPE_ARRAY else []
		var payload := _payload(event)
		var who := _names(observation, subjects)
		match String(how.get("show", "")):
			"bubble":
				_bubble(id, type, subjects, who, String(payload.get(String(how.get("text", "")), "")))
			"notice":
				_notice(id, type, who, payload)
			_:
				_skip(id, type, "unlisted")


func _bubble(id: String, type: String, subjects: Array, who: String, text: String) -> void:
	var speaker := String(subjects[0]) if not subjects.is_empty() else ""
	if speaker == "" or text == "" or _people.figure_at(speaker) == null:
		_skip(id, type, "undrawn")
		return
	_drop(speaker)
	var anchor := Node2D.new()
	anchor.z_index = 100
	var box := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(1.0, 0.99, 0.95, 0.96)
	style.border_color = Color("3b2c1e")
	style.set_border_width_all(1)
	style.set_corner_radius_all(8)
	style.set_content_margin_all(6)
	box.add_theme_stylebox_override("panel", style)
	var label := Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size = Vector2(minf(220.0, 10.0 + 7.0 * text.length()), 0)
	label.add_theme_font_size_override("font_size", 12)
	label.add_theme_color_override("font_color", Color("3b2c1e"))
	label.add_to_group(MineWorldText.WORLD_TEXT_GROUP)
	box.add_child(label)
	anchor.add_child(box)
	_world.add_child(anchor)
	_bubbles[speaker] = {"node": anchor, "box": box, "until": _now() + _bubble_s}
	_follow(speaker)
	shown += 1
	bubbled.emit({"id": id, "event_type": type, "who": speaker, "name": who, "text": text})


func _notice(id: String, type: String, who: String, payload: Dictionary) -> void:
	var args := {"who": who}
	for field in payload:
		var value: Variant = payload[field]
		if typeof(value) == TYPE_STRING:
			args[String(field)] = value
		elif typeof(value) == TYPE_INT or (typeof(value) == TYPE_FLOAT and value == floorf(value)):
			args[String(field)] = str(int(value))
	var key := NOTICE_KEY + type
	var said := Words.text(key, args) if Words.has(key) else "%s %s" % [Words.readable(type), who]
	_toasts.toast({"kind": "notice", "action_type": "", "target": null, "event": id}, said, "",
		Callable(), _notice_s)
	shown += 1
	noticed.emit({"id": id, "event_type": type, "who": who, "text": said})


func _skip(id: String, type: String, why: String) -> void:
	skipped_count += 1
	skipped.emit({"id": id, "event_type": type, "why": why})


func _process(_delta: float) -> void:
	var now := _now()
	for who in _bubbles.keys():
		if now > float(_bubbles[who]["until"]) or _people.figure_at(who) == null:
			_drop(who)
		else:
			_follow(who)


## Keeps a bubble over its person as they walk: centred, its bottom above the figure.
func _follow(who: String) -> void:
	var entry: Dictionary = _bubbles[who]
	var figure: Variant = _people.figure_at(who)
	if figure == null:
		return
	var box: PanelContainer = entry["box"]
	var half: float = maxf(_people.drawn_height_px(who), 34.0) * 0.5
	entry["node"].position = (figure as Vector2) - Vector2(box.size.x * 0.5, half + LIFT_PX + box.size.y)


func _drop(who: String) -> void:
	if _bubbles.has(who):
		if is_instance_valid(_bubbles[who]["node"]):
			_bubbles[who]["node"].queue_free()
		_bubbles.erase(who)


## The owning pack's payload, as the wire carries it (`payload.payload`), or {}.
static func _payload(event: Dictionary) -> Dictionary:
	var record: Variant = event.get("payload")
	var inner: Variant = record.get("payload") if typeof(record) == TYPE_DICTIONARY else null
	return inner if typeof(inner) == TYPE_DICTIONARY else {}


## The subjects' display names as the observation gives them, joined; an unnamed subject is left out.
static func _names(observation: MineWorldObservation, subjects: Array) -> String:
	var names := PackedStringArray()
	for subject in subjects:
		var name := observation.display_name(String(subject))
		if name != "":
			names.append(name)
	return ", ".join(names)


func _now() -> float:
	return Time.get_ticks_msec() / 1000.0
