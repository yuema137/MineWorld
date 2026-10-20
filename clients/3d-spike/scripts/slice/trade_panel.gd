## The trade panel beside the slice's interaction menus (S31-e §§5.5, 5.6; `clients/shared/TRADE.md` §5).
##
## Opened and closed with `SliceOfferMenu`: on a person's menu (Tab), what that person sells, with *Buy*; on
## your own (B), what the shop you stand in sells, your stall, and the shop's shelf behind one button. The
## rows are the shared trade module's (`MineWorldTrade`), the same rows the 2D client draws; this file draws
## them in the HUD's light-on-dark panel and presses their buttons through `SliceIntents` alone.
##
## Verdicts are read only where rows are drawn: a row the server marked unavailable is dimmed, never hidden,
## and its buttons still send. A price is typed into the panel's own field (16d's input gate holds the game
## while a menu is open); Enter sends, Esc cancels the field before it closes the menu.
class_name SliceTradePanel
extends CanvasLayer

## The rows shown changed: the subject ("" for the shop here) and the rows as data, for the probe.
signal shown(subject: String, rows: Array)

const PANEL_BG := Color(0, 0, 0, 0.45)
const ROW_PX := 15
const TITLE_PX := 18
const UNAVAILABLE_ALPHA := 0.55
const EDGE_PX := 12.0

var link: SliceLink
var menu: SliceOfferMenu
var trade: MineWorldTrade

var subject := ""
var rows: Array = []
var shelf_open := false
var _self := false
## What a button still waits for: {row, button, values, need: "item" | "amount", error: bool}, or {}.
var _waiting := {}
var _signature := ""
var _box: PanelContainer
var _column: VBoxContainer
var _field: LineEdit


func _ready() -> void:
	layer = 21
	_box = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = PANEL_BG
	style.set_corner_radius_all(6)
	style.set_content_margin_all(10)
	_box.add_theme_stylebox_override("panel", style)
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 2)
	_box.add_child(_column)
	add_child(_box)
	_box.visible = false


func is_open() -> bool:
	return _box.visible


## The menu opened about `id` (the observer for your own).
func open(id: String) -> void:
	subject = id
	_self = id == link.client.observer
	shelf_open = false
	_waiting = {}
	_signature = ""
	refresh(link.client.latest)


func close() -> void:
	subject = ""
	rows = []
	_waiting = {}
	_box.visible = false


## Esc: drops a price field or a picker that is waiting. True when something was dropped.
func cancel_input() -> bool:
	if _waiting.is_empty():
		return false
	_waiting = {}
	_signature = ""
	refresh(link.client.latest)
	return true


## Rebuilds the rows from a newer observation while the menu is open.
func refresh(obs: MineWorldObservation) -> void:
	if subject == "" or obs == null:
		return
	var currency := MineWorldMoney.currency_of(obs)
	rows = trade.answer_rows(obs, currency)
	if _self:
		rows.append_array(trade.sale_rows(obs, "", currency))
		rows.append_array(trade.stall_rows(obs, currency))
		if shelf_open:
			rows.append_array(trade.shelf_rows(obs, currency))
	else:
		rows.append_array(trade.sale_rows(obs, subject, currency))
	var signature := JSON.stringify([shelf_open, _waiting.get("need", ""), _waiting.get("error", false),
		MineWorldTrade.report(rows)])
	if signature == _signature:
		return
	_signature = signature
	_draw(obs)
	shown.emit("" if _self else subject, rows)


func has_shelf() -> bool:
	var obs := link.client.latest
	return _self and obs != null and not trade.shelf_rows(obs, {}).is_empty()


func toggle_shelf() -> void:
	shelf_open = not shelf_open
	_signature = ""
	refresh(link.client.latest)


## The player pressed button `b` of row `r` (a click, Enter, or the probe with `given`: `amount` as typed
## text, `item` as a reference). One path; reads no verdict.
func press(r: int, b: int, given: Dictionary = {}) -> void:
	if r < 0 or r >= rows.size() or b < 0 or b >= rows[r]["buttons"].size():
		return
	_continue(rows[r], rows[r]["buttons"][b], given.duplicate())


func _continue(row: Dictionary, button: Dictionary, values: Dictionary) -> void:
	var offered: Dictionary = button["affordance"]
	var ask: PackedStringArray = button["ask"]
	if offered.has("payload") and ask.is_empty():
		link.intents.submit_offered(link.client, offered, link.actor_location())
		return
	if ask.has("item") and not values.has("item"):
		_wait(row, button, values, "item", false)
		return
	if ask.has("amount") and not values.has("price"):
		if values.has("amount"):
			var price: Variant = MineWorldMoney.parse(String(values["amount"]), MineWorldMoney.currency_of(link.client.latest))
			if price != null:
				values["price"] = price
				_continue(row, button, values)
				return
		_wait(row, button, values, "amount", values.has("amount"))
		return
	var input: Dictionary = button["fill"].duplicate()
	input.merge(values, true)
	link.intents.compose_trade(link.client, offered, input, link.actor_location())


func _wait(row: Dictionary, button: Dictionary, values: Dictionary, need: String, error: bool) -> void:
	_waiting = {"row": row, "button": button, "values": values, "need": need, "error": error}
	_signature = ""
	refresh(link.client.latest)


## The picker chose a kind: the waiting button goes on with it.
func pick(item_ref: Variant) -> void:
	if _waiting.get("need", "") != "item":
		return
	var waiting := _waiting
	_waiting = {}
	waiting["values"]["item"] = item_ref
	_continue(waiting["row"], waiting["button"], waiting["values"])
	_signature = ""
	refresh(link.client.latest)


## The price field's text was sent (Enter, or the probe typing): the waiting button goes on with it.
func submit_price(said: String) -> void:
	if _waiting.get("need", "") != "amount":
		return
	var waiting := _waiting
	_waiting = {}
	waiting["values"]["amount"] = said
	_continue(waiting["row"], waiting["button"], waiting["values"])
	if _waiting.is_empty():
		_signature = ""
		refresh(link.client.latest)


func _draw(obs: MineWorldObservation) -> void:
	for child in _column.get_children():
		_column.remove_child(child)
		child.queue_free()
	_field = null
	var shelf := has_shelf()
	_box.visible = not rows.is_empty() or shelf
	if not _box.visible:
		return
	_column.add_child(_label(MineWorldText.text("panel.stall" if _self else "panel.trade"), TITLE_PX))
	if _waiting.get("need", "") == "item":
		_draw_picker(obs)
	elif _waiting.get("need", "") == "amount":
		_draw_field(obs)
	else:
		_draw_rows(shelf)
	_place.call_deferred()


func _draw_rows(shelf: bool) -> void:
	for r in rows.size():
		var row: Dictionary = rows[r]
		var line := HBoxContainer.new()
		var said := _label(row["text"], ROW_PX)
		# The server's verdict, shown: an unavailable row is dimmed, never hidden and never disabled.
		if row["available"] == false:
			said.modulate = Color(1, 1, 1, UNAVAILABLE_ALPHA)
		line.add_child(said)
		for b in row["buttons"].size():
			line.add_child(_button(row["buttons"][b]["label"], press.bind(r, b)))
		_column.add_child(line)
	if shelf:
		_column.add_child(_button(MineWorldText.text("trade.shop-prices"), toggle_shelf))


func _draw_picker(obs: MineWorldObservation) -> void:
	_column.add_child(_label(MineWorldText.text("trade.item.pick"), ROW_PX))
	for kind in MineWorldTrade.catalogue_of(obs):
		_column.add_child(_button(kind["name"], pick.bind(kind["ref"])))
	_column.add_child(_button(MineWorldText.text("trade.cancel"), func() -> void: cancel_input()))


func _draw_field(obs: MineWorldObservation) -> void:
	var currency := MineWorldMoney.currency_of(obs)
	var row: Dictionary = _waiting["row"]
	var values: Dictionary = _waiting["values"]
	var item := String(row["item"]) if row["item"] != "" else MineWorldTrade.ref_id(values.get("item"))
	var prompt := MineWorldText.text("trade.price.prompt", {"item": link.things.name_of(item),
		"currency": currency.get("name", "")})
	if _waiting["error"]:
		prompt = MineWorldText.text("trade.price.not-an-amount", {"text": values.get("amount", ""),
			"currency": currency.get("name", "")})
	_column.add_child(_label(prompt, ROW_PX))
	_field = LineEdit.new()
	_field.custom_minimum_size = Vector2(220, 0)
	if row["amount"] != null:
		_field.placeholder_text = MineWorldMoney.format(row["amount"], currency)
	_field.text_submitted.connect(submit_price)
	_column.add_child(_field)
	_column.add_child(_button(MineWorldText.text("trade.cancel"), func() -> void: cancel_input()))
	_field.grab_focus.call_deferred()


## Beside the menu's box, kept inside the window.
func _place() -> void:
	var view := get_viewport().get_visible_rect().size if get_viewport() != null else Vector2(1600, 900)
	_box.reset_size()
	var beside := menu.box_rect() if menu != null else Rect2(view * 0.5, Vector2.ZERO)
	var at := beside.position + Vector2(beside.size.x + 16.0, 0.0)
	var room := (view - _box.size - Vector2(EDGE_PX, EDGE_PX)).max(Vector2(EDGE_PX, EDGE_PX))
	_box.position = at.clamp(Vector2(EDGE_PX, EDGE_PX), room)


func _button(said: String, on_press: Callable) -> Button:
	var button := Button.new()
	button.text = said
	button.flat = true
	button.add_theme_font_size_override("font_size", ROW_PX)
	for colour in ["font_color", "font_pressed_color"]:
		button.add_theme_color_override(colour, Color(1, 1, 1, 0.88))
	button.add_theme_color_override("font_hover_color", Color(1, 0.93, 0.78, 1.0))
	button.add_theme_color_override("font_focus_color", Color(1, 0.93, 0.78, 1.0))
	button.pressed.connect(on_press)
	return button


func _label(said: String, size: int) -> Label:
	var l := Label.new()
	l.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
	l.text = said
	ControlsHud._style(l, size)
	return l
