extends CanvasLayer

## The join screen (PR 13e, `ARC-344`): an address, an invite, a nickname and a person to play, chosen
## from the seats the world lists (`GET /status`, `server/PROTOCOL.md` §5.7), in its order.
##
## It decides nothing. It does not know which seats are free — `/status` names no player — so it lists
## them all, and the server's answer says when one is taken; it offers "Take over" only after that
## answer, and only the player's press sends it. It does not open the connection either: it emits
## `join_requested`, and the composition root hands that to `link.gd`, the one script that joins
## (`ARC-47` R5). The invite field is secret and its text is never printed, logged or stored; nothing
## here is written to the settings file (`clients/shared/SETTINGS.md` INV-SET-2).
##
## Its interface is the one a 3D client would take (Q-13e-6): `prefill`, `open(code)`, `close`, the
## `join_requested` signal.

## The player asked to join: where, as whom, with which credentials, and whether to take the seat over.
signal join_requested(address: String, seat: String, invite: String, nickname: String, take_over: bool)
## What the screen shows changed: the seats listed, the answer, the buttons (for the harness).
signal changed

const Words := preload("res://scripts/hud/words.gd")

## The refusal that makes "Take over" a choice (`PROTOCOL.md` §4.2 rule 4).
const OCCUPIED := "seat_occupied"

## The seats `/status` listed, in its order; empty until it answered.
var seats := PackedStringArray()
## The code shown as the last answer, or "" (or "unreachable" when `/status` could not be read).
var answer_code := ""

var _address: LineEdit
var _invite: LineEdit
var _nickname: LineEdit
var _list: ItemList
var _answer: Label
var _join: Button
var _take_over: Button
var _http: HTTPRequest
var _preselect := ""
var _asked := ""


func _ready() -> void:
	layer = 110
	visible = false
	var holder := CenterContainer.new()
	holder.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(holder)
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.97)
	style.set_corner_radius_all(12)
	style.set_content_margin_all(22)
	panel.add_theme_stylebox_override("panel", style)
	panel.custom_minimum_size = Vector2(440, 0)
	holder.add_child(panel)
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	panel.add_child(column)
	column.add_child(_label("ui.join.title", 18))
	_address = _field(column, "ui.join.address")
	_address.text_submitted.connect(func(_t: String) -> void: refresh())
	_invite = _field(column, "ui.join.invite")
	_invite.secret = true
	_nickname = _field(column, "ui.join.nickname")
	_nickname.text_submitted.connect(func(_t: String) -> void: press_join())
	column.add_child(_label("ui.join.person", 14))
	_list = ItemList.new()
	_list.custom_minimum_size = Vector2(0, 180)
	_list.item_selected.connect(func(_i: int) -> void: _update())
	_list.item_activated.connect(func(_i: int) -> void: press_join())
	column.add_child(_list)
	_answer = Label.new()
	_answer.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_answer.add_theme_color_override("font_color", Color("8a3b1e"))
	column.add_child(_answer)
	var buttons := HBoxContainer.new()
	buttons.alignment = BoxContainer.ALIGNMENT_END
	column.add_child(buttons)
	_take_over = Button.new()
	_take_over.pressed.connect(press_take_over)
	buttons.add_child(_take_over)
	_join = Button.new()
	_join.pressed.connect(press_join)
	buttons.add_child(_join)
	_http = HTTPRequest.new()
	_http.timeout = 5.0
	add_child(_http)
	_http.request_completed.connect(_on_status)
	_reword()
	_update()


func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSLATION_CHANGED and _join != null:
		_reword()


## The fields as the arguments gave them (the launcher passes the invite of a world it hosts);
## `seat` is preselected once the list arrives.
func prefill(address: String, invite: String, nickname: String, seat: String) -> void:
	_address.text = address
	_invite.text = invite
	_nickname.text = nickname
	_preselect = seat


## Shows the screen with `code` as the answer ("" for none) and asks the world for its seats again.
func open(code: String) -> void:
	visible = true
	show_answer(code)
	refresh()


func close() -> void:
	visible = false


## The last answer, worded by its code (`reason.<code>`); "Take over" appears only for `seat_occupied`.
func show_answer(code: String) -> void:
	answer_code = code
	_answer.text = "" if code == "" else Words.reason(code)
	_update()


## Asks `GET /status` of the address in the field for the seats it lists.
func refresh() -> void:
	_http.cancel_request()
	# The person chosen stays chosen when the list is asked for again (after an answer, Take over
	# applies to the seat that was refused).
	if chosen() != "":
		_preselect = chosen()
	seats = PackedStringArray()
	_list.clear()
	_asked = _address.text.strip_edges()
	_answer.text = Words.text("ui.join.loading", {"address": _asked})
	_update()
	if _http.request(status_url(_asked)) != OK:
		_unreachable()


## The seat chosen, or "".
func chosen() -> String:
	var picked := _list.get_selected_items()
	return seats[picked[0]] if picked.size() > 0 and picked[0] < seats.size() else ""


## Selects the seat at `index` of the list, as a click would.
func pick(index: int) -> void:
	if index >= 0 and index < seats.size():
		_list.select(index)
		_update()


func join_enabled() -> bool:
	return not _join.disabled


func take_over_offered() -> bool:
	return _take_over.visible and not _take_over.disabled


## Join: the chosen seat, the fields as typed. Disabled until the world listed its seats.
func press_join() -> void:
	if join_enabled():
		_request(false)


## Take over: the same join, asking for the seat another connection holds (only after `seat_occupied`).
func press_take_over() -> void:
	if take_over_offered():
		_request(true)


## Replaces the invite field's text, as typing would (the harness's wrong-invite check).
func type_invite(text: String) -> void:
	_invite.text = text


func _request(take_over: bool) -> void:
	join_requested.emit(_asked, chosen(), _invite.text, _nickname.text.strip_edges(), take_over)


func _on_status(result: int, code: int, _headers: PackedStringArray, body: PackedByteArray) -> void:
	if result != HTTPRequest.RESULT_SUCCESS or code != 200:
		_unreachable()
		return
	var parsed: Variant = JSON.parse_string(body.get_string_from_utf8())
	var listed: Variant = parsed.get("seats") if typeof(parsed) == TYPE_DICTIONARY else null
	if typeof(listed) != TYPE_ARRAY:
		_unreachable()
		return
	for seat in listed:
		seats.append(String(seat))
		_list.add_item(String(seat))
	var at := seats.find(_preselect)
	if at >= 0:
		_list.select(at)
	_answer.text = "" if answer_code == "" else Words.reason(answer_code)
	_update()


func _unreachable() -> void:
	seats = PackedStringArray()
	_list.clear()
	answer_code = "unreachable"
	_answer.text = Words.text("ui.join.unreachable", {"address": _asked})
	_update()


func _update() -> void:
	_join.disabled = seats.is_empty() or chosen() == ""
	_take_over.visible = answer_code == OCCUPIED
	_take_over.disabled = _join.disabled
	changed.emit()


func _reword() -> void:
	_join.text = Words.text("ui.join.join")
	_take_over.text = Words.text("ui.join.take-over")
	_address.placeholder_text = Words.text("ui.join.address")
	_invite.placeholder_text = Words.text("ui.join.invite")
	_nickname.placeholder_text = Words.text("ui.join.nickname")


## `host:port`, `ws://host:port` or `ws://host:port/ws` → the world's `GET /status` URL.
static func status_url(address: String) -> String:
	var host := address.trim_prefix("ws://").trim_prefix("wss://").trim_prefix("http://")
	host = host.trim_suffix("/").trim_suffix("/ws")
	return "http://%s/status" % host


func _label(key: String, size: int) -> Label:
	var label := Label.new()
	label.text = Words.text(key)
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", Color("3b2c1e"))
	return label


func _field(column: VBoxContainer, key: String) -> LineEdit:
	var row := HBoxContainer.new()
	var label := _label(key, 14)
	label.custom_minimum_size = Vector2(90, 0)
	row.add_child(label)
	var edit := LineEdit.new()
	edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(edit)
	column.add_child(row)
	return edit
