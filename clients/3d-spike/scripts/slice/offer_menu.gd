## The interaction menus of the connected slice: yours (B) and the person you look at (Tab) (step-15
## §22.3 D-16d-3, D-16d-4a, D-16d-8; PR 16d).
##
## A menu is the newest observation's affordances for one subject, in the server's order, nothing
## removed and nothing invented -- 13b's entry model (`clients/2d/scripts/menu.gd`), so the two
## clients' `MENU` reports compare field for field (§22.4 P-2). The subject is the observer itself
## (the target-less offers) or a perceived person (the offers directed at them). One entry each:
##
## ```text
## complete      enabled; chosen -> sent exactly as offered (intents.submit_offered)
## composed      enabled; chosen -> the talk line asks for the words (talk, Q-16d-7; PR 16g1)
## body          the offer the body answers by walking (SliceLink.MOVE_ACTION); listed, never chosen
## unsupported   listed disabled, "not supported by this client", never sent
## unavailable   greyed but still choosable, with the server's reason and declared range, unevaluated
## ```
##
## Verdicts are read only where entries are built, never where one is chosen: choosing an entry the
## server marked unavailable sends it, and the answer is the server's (`ADOPTION.md` §3.3). The menu is
## rebuilt from every observation while it is open, each entry carries the affordance it was built
## from, and it closes when its person is no longer perceived, on a choice, and on Esc.
##
## Beside your own menu stands what you have: money, what you carry, and the listing of the shop you
## stand in (`SliceThings.panels`). The look is the slice's HUD (`ControlsHud`): light text on the
## captions' dark translucent panel.
class_name SliceOfferMenu
extends CanvasLayer

## The entries shown changed (on opening, and when an observation changes them): the subject, the
## entries as data and the raw offers they were built from -- 13b's "one world, two views" report.
signal listed(subject: String, entries: Array, offered: Array)
## The panels beside your menu changed: `SliceThings.panels`' records.
signal shown_panels(panels: Array)
## A menu opened or closed: the slice holds gameplay input while one is open (D-16d-9).
signal opened
signal closed

## What the menus look like: the captions' panel and the HUD's text (D-16d-8).
const PANEL_BG := Color(0, 0, 0, 0.45)
const ENTRY_PX := 15
const TITLE_PX := 18
const UNAVAILABLE_ALPHA := 0.55
const DISABLED_ALPHA := 0.35
## Where each menu stands, as a share of the window: yours left of centre with what you have to its
## right; a person's just right of the reticle.
const SELF_AT := Vector2(0.18, 0.22)
const PERSON_AT := Vector2(0.53, 0.40)
const EDGE_PX := 12.0

var link: SliceLink
var things: SliceThings

## The subject's id ("" when closed) and whether it is the observer.
var subject := ""
var entries: Array = []
var panels: Array = []
var _self := false
var _holding := false              ## `opened` was said and `closed` not yet
var _signature := ""
var _panel_signature := ""
var _box: PanelContainer
var _title: Label
var _list: VBoxContainer
var _have: PanelContainer
var _have_list: VBoxContainer


func _ready() -> void:
	layer = 20
	_box = _panel()
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 2)
	_box.add_child(column)
	_title = _label("", TITLE_PX)
	column.add_child(_title)
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 0)
	column.add_child(_list)
	add_child(_box)
	_have = _panel()
	_have_list = VBoxContainer.new()
	_have_list.add_theme_constant_override("separation", 2)
	_have.add_child(_have_list)
	add_child(_have)
	_box.visible = false
	_have.visible = false


func is_open() -> bool:
	return subject != ""


## B and Tab, as a player presses them. Held, like every gameplay key, while the settings menu is open.
func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo) or link == null \
			or link.client == null or link.client.latest == null:
		return
	match (event as InputEventKey).physical_keycode:
		KEY_B:
			open(link.client.observer)
			get_viewport().set_input_as_handled()
		KEY_TAB:
			var who := link.facing_person()
			if who == "":
				link.nobody_to_offer()
			else:
				open(who)
			get_viewport().set_input_as_handled()


## Opens the menu about `id`: the observer for your own, or a perceived person.
func open(id: String) -> void:
	var was_open := is_open()
	subject = id
	_self = id == link.client.observer
	_signature = ""
	_panel_signature = ""
	_box.visible = true
	_have.visible = _self
	refresh(link.client.latest)
	if is_open() and not was_open:
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
		_holding = true
		opened.emit()


func close() -> void:
	subject = ""
	entries = []
	_box.visible = false
	_have.visible = false
	if _holding:
		_holding = false
		closed.emit()


## Rebuilds the entries from a newer observation; closes when the subject is no longer perceived.
func refresh(obs: MineWorldObservation) -> void:
	if subject == "" or obs == null:
		return
	if not _self and obs.entity(subject).is_empty():
		close()
		return
	var offered: Array = obs.affordances("", "" if _self else subject)
	entries = _build(offered)
	var signature := JSON.stringify(entries.map(func(e: Dictionary) -> Array:
		return [e["label"], e["enabled"], e["available"]]))
	if signature != _signature:
		_signature = signature
		_draw()
		listed.emit(subject, entries, offered)
	if _self:
		panels = things.panels(obs)
		var shown := JSON.stringify(panels.map(func(p: Dictionary) -> Array: return [p["title"], p["rows"]]))
		if shown != _panel_signature:
			_panel_signature = shown
			_draw_panels()
			shown_panels.emit(panels)


## The player chose entry `index` (a click, Enter, or the harness -- one path). Reads no verdict.
func choose(index: int) -> String:
	if index < 0 or index >= entries.size():
		return ""
	var entry: Dictionary = entries[index]
	if not entry["enabled"]:
		return ""
	var affordance: Dictionary = entry["affordance"]
	var target := subject
	close()
	match String(entry["kind"]):
		"complete":
			return link.intents.submit_offered(link.client, affordance, link.actor_location())
		"composed":
			# The words are the player's: the talk line opens for this person, and sends what is typed.
			link.wants_words.emit(target)
	return ""


## Entries for the offers, in their order. Asks `intents` what it composes; reads no verdict itself
## (`_entry` does).
func _build(offered: Array) -> Array:
	var built: Array = []
	for affordance in offered:
		var type := String(affordance.get("action_type", ""))
		var kind := "unsupported"
		if MineWorldObservation.is_complete(affordance):
			kind = "complete"
		elif type == SliceLink.MOVE_ACTION:
			kind = "body"
		elif link.intents.composes(type):
			kind = "composed"
		var entry := _entry(affordance, kind)
		entry["index"] = built.size()
		built.append(entry)
	return built


## One entry: what the affordance is, the server's verdict on it as data, and its label.
func _entry(affordance: Dictionary, kind: String) -> Dictionary:
	var type := String(affordance.get("action_type", ""))
	var available := bool(affordance.get("available", false))
	var reason: Variant = affordance.get("unavailable_reason")
	var about := about_of(affordance)
	var target: Variant = affordance.get("target")
	var label := things.action_label(type, "" if target == null else String(target), about)
	var shown := label
	if kind == "unsupported":
		shown = MineWorldText.text("ui.unsupported", {"label": label})
	elif kind == "body":
		shown = MineWorldText.text("ui.by-body", {"label": label})
	elif not available:
		shown = MineWorldText.text("ui.unavailable", {"label": label,
			"reason": SliceThings.reason_words(reason)})
		var declared: Variant = affordance.get("requirement")
		if typeof(declared) == TYPE_DICTIONARY and declared.get("within_range") != null:
			var metres := float(declared["within_range"]) / 1000.0
			shown += "  ·  " + MineWorldText.text("ui.needs-range",
				{"metres": str(int(metres)) if metres == floorf(metres) else String.num(metres, 1)})
	return {"action_type": type, "kind": kind, "target": null if target == null else String(target),
		"about": about, "complete": MineWorldObservation.is_complete(affordance), "available": available,
		"reason": reason, "label": shown, "enabled": kind == "complete" or kind == "composed",
		"affordance": affordance}


## The first entity a payload refers to by a typed reference, or "" (13b's `about`).
static func about_of(affordance: Dictionary) -> String:
	var payload: Variant = affordance.get("payload")
	if typeof(payload) != TYPE_DICTIONARY:
		return ""
	for value in payload.values():
		if typeof(value) == TYPE_DICTIONARY and typeof(value.get("entity")) == TYPE_STRING:
			return value["entity"]
	return ""


func _draw() -> void:
	_title.text = MineWorldText.text("ui.self") if _self else link.shown_text(subject)
	for child in _list.get_children():
		_list.remove_child(child)
		child.queue_free()
	if entries.is_empty():
		_list.add_child(_label(MineWorldText.text("ui.menu.empty"), ENTRY_PX))
	var first: Button = null
	for entry in entries:
		var button := Button.new()
		button.text = entry["label"]
		button.flat = true
		button.alignment = HORIZONTAL_ALIGNMENT_LEFT
		button.add_theme_font_size_override("font_size", ENTRY_PX)
		for colour in ["font_color", "font_hover_color", "font_focus_color", "font_pressed_color",
				"font_disabled_color"]:
			button.add_theme_color_override(colour, Color(1, 1, 1, 0.88))
		button.add_theme_color_override("font_hover_color", Color(1, 0.93, 0.78, 1.0))
		button.add_theme_color_override("font_focus_color", Color(1, 0.93, 0.78, 1.0))
		button.disabled = not entry["enabled"]
		if not entry["enabled"]:
			button.modulate = Color(1, 1, 1, DISABLED_ALPHA)
		elif not entry["available"]:
			button.modulate = Color(1, 1, 1, UNAVAILABLE_ALPHA)
		var index: int = entry["index"]
		button.pressed.connect(func() -> void: choose(index))
		_list.add_child(button)
		if first == null and entry["enabled"]:
			first = button
	if first != null:
		first.grab_focus.call_deferred()
	_place.call_deferred()


func _draw_panels() -> void:
	for child in _have_list.get_children():
		_have_list.remove_child(child)
		child.queue_free()
	for p in panels:
		_have_list.add_child(_label(p["title"], TITLE_PX))
		for row in p["rows"]:
			_have_list.add_child(_label("    " + row, ENTRY_PX))
	_place.call_deferred()


## Puts each box where D-16d-8 stands it, kept inside the window (13b's keep-on-screen rule).
func _place() -> void:
	var view := get_viewport().get_visible_rect().size if get_viewport() != null else Vector2(1600, 900)
	_box.reset_size()
	var at := view * (SELF_AT if _self else PERSON_AT)
	_box.position = _inside(at, _box.size, view)
	if _have.visible:
		_have.reset_size()
		_have.position = _inside(_box.position + Vector2(_box.size.x + 16.0, 0.0), _have.size, view)


static func _inside(at: Vector2, size: Vector2, view: Vector2) -> Vector2:
	var room := (view - size - Vector2(EDGE_PX, EDGE_PX)).max(Vector2(EDGE_PX, EDGE_PX))
	return at.clamp(Vector2(EDGE_PX, EDGE_PX), room)


func _panel() -> PanelContainer:
	var box := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = PANEL_BG
	style.set_corner_radius_all(6)
	style.set_content_margin_all(10)
	box.add_theme_stylebox_override("panel", style)
	return box


func _label(text: String, size: int) -> Label:
	var l := Label.new()
	l.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
	l.text = text
	ControlsHud._style(l, size)
	return l
