extends CanvasLayer

## The trade panel beside the interaction menu: the sale rows of the person the menu is about, or, on my
## own menu, the sale rows of the shop I stand in, my stall, and the shop's shelf behind one button (S31-e
## §§5.5, 5.6; `clients/shared/TRADE.md` §5).
##
## Rows come from the shared trade module (`MineWorldTrade`), which reads the frame and words the rows; this
## file draws them in the 2D client's warm panel and presses their buttons. Verdicts are read only where rows
## are drawn: a row the server marked unavailable is greyed and its buttons are still pressed and sent, and
## the server answers (`R3'`). Pressing calls `intents` alone, which builds every request.

const Words := preload("res://scripts/hud/words.gd")

## The rows shown changed: the subject ("" for the shop here) and the rows as data, for the harness.
signal shown(subject: String, rows: Array)

var intents: Node
var readers
var talk_line: CanvasLayer
var trade: MineWorldTrade

## The menu's subject while open ("" when closed), and whether it is the observer.
var subject := ""
var rows: Array = []
var shelf_open := false
## The button waiting for the player to pick an item kind, or {} (`_continue` resumes it).
var _picking := {}
var _self := false
var _latest: MineWorldObservation = null
var _signature := ""
var _box: PanelContainer
var _column: VBoxContainer


func _ready() -> void:
	layer = 106
	_box = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.97)
	style.border_color = Color("b89a74")
	style.set_border_width_all(1)
	style.set_corner_radius_all(8)
	style.set_content_margin_all(10)
	_box.add_theme_stylebox_override("panel", style)
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 2)
	_box.add_child(_column)
	add_child(_box)
	_box.visible = false


func is_open() -> bool:
	return _box.visible


## The menu opened about `id`: show its rows beside `beside` (the menu's box), or nothing when it has none.
func open(id: String, observation: MineWorldObservation, beside: Rect2) -> void:
	_latest = observation
	_self = id == observation.observer()
	subject = id
	shelf_open = false
	_picking = {}
	_signature = ""
	_box.position = beside.position + Vector2(beside.size.x + 12, 0)
	refresh(observation)


func close() -> void:
	subject = ""
	rows = []
	_picking = {}
	_box.visible = false


## Rebuilds the rows from a newer observation while the menu is open.
func refresh(observation: MineWorldObservation) -> void:
	_latest = observation
	if subject == "":
		return
	var currency := MineWorldMoney.currency_of(observation)
	rows = trade.answer_rows(observation, currency)
	if _self:
		rows.append_array(trade.sale_rows(observation, "", currency))
		rows.append_array(trade.stall_rows(observation, currency))
		if shelf_open:
			rows.append_array(trade.shelf_rows(observation, currency))
	else:
		rows.append_array(trade.sale_rows(observation, subject, currency))
	var signature := JSON.stringify([shelf_open, _picking.is_empty(), MineWorldTrade.report(rows)])
	if signature == _signature:
		return
	_signature = signature
	_draw(observation)
	shown.emit("" if _self else subject, rows)


## Whether the shop here has a shelf to open (a listing and seller offers), for the collapsed button.
func has_shelf() -> bool:
	return _self and _latest != null and not trade.shelf_rows(_latest, {}).is_empty()


func toggle_shelf() -> void:
	shelf_open = not shelf_open
	_signature = ""
	refresh(_latest)


## The player pressed button `b` of row `r` (a click, or the harness with what it typed in `given`: `amount`
## as typed text, `item` as a reference). One path; reads no verdict.
func press(r: int, b: int, given: Dictionary = {}) -> void:
	if r < 0 or r >= rows.size() or b < 0 or b >= rows[r]["buttons"].size():
		return
	_continue(rows[r], rows[r]["buttons"][b], given.duplicate())


## Collects what the button still needs — an item kind from the picker, a price from the typed line — then
## hands the request to `intents`.
func _continue(row: Dictionary, button: Dictionary, values: Dictionary) -> void:
	var offered: Dictionary = button["affordance"]
	var ask: PackedStringArray = button["ask"]
	if offered.has("payload") and ask.is_empty():
		intents.submit_offered(offered)
		return
	if ask.has("item") and not values.has("item"):
		_picking = {"row": row, "button": button, "values": values}
		_signature = ""
		refresh(_latest)
		return
	if ask.has("amount") and not values.has("price"):
		if values.has("amount"):
			var price: Variant = MineWorldMoney.parse(String(values["amount"]), MineWorldMoney.currency_of(_latest))
			if price != null:
				values["price"] = price
				_continue(row, button, values)
				return
		_ask_price(row, button, values)
		return
	var input: Dictionary = button["fill"].duplicate()
	input.merge(values, true)
	intents.compose(offered, input)


## The typed line asks for the price; text that is not an amount asks again, saying so (S31-e §5.6).
func _ask_price(row: Dictionary, button: Dictionary, values: Dictionary) -> void:
	var currency := MineWorldMoney.currency_of(_latest)
	var item := String(row["item"]) if row["item"] != "" else MineWorldTrade.ref_id(values.get("item"))
	var prompt := Words.text("trade.price.prompt", {"item": readers.thing(item), "currency": currency.get("name", "")})
	if values.has("amount"):
		prompt = Words.text("trade.price.not-an-amount", {"text": values["amount"], "currency": currency.get("name", "")})
	var suggestions := PackedStringArray()
	if row["amount"] != null:
		suggestions.append(MineWorldMoney.format(row["amount"], currency))
	talk_line.ask(prompt, suggestions, func(said: String) -> void:
		values["amount"] = said
		_continue(row, button, values))


## The picker chose a kind (a click, or the harness): the waiting button goes on with it.
func pick(item_ref: Variant) -> void:
	if _picking.is_empty():
		return
	var waiting := _picking
	_picking = {}
	waiting["values"]["item"] = item_ref
	_signature = ""
	refresh(_latest)
	_continue(waiting["row"], waiting["button"], waiting["values"])


func _draw(observation: MineWorldObservation) -> void:
	for child in _column.get_children():
		child.queue_free()
	var shelf := has_shelf()
	_box.visible = not rows.is_empty() or shelf
	if not _box.visible:
		return
	_column.add_child(_label(Words.text("panel.stall" if _self else "panel.trade"), 15, Color("6b4e2e")))
	if not _picking.is_empty():
		_draw_picker(observation)
		return
	for r in rows.size():
		var row: Dictionary = rows[r]
		var line := HBoxContainer.new()
		var said := _label(row["text"], 13, Color("3b2c1e"))
		# The server's verdict, shown: an unavailable row is greyed, never hidden and never disabled.
		if row["available"] == false:
			said.modulate = Color(1, 1, 1, 0.55)
		line.add_child(said)
		for b in row["buttons"].size():
			var button := Button.new()
			button.text = row["buttons"][b]["label"]
			button.pressed.connect(press.bind(r, b))
			line.add_child(button)
		_column.add_child(line)
	if shelf:
		var toggle := Button.new()
		toggle.text = Words.text("trade.shop-prices")
		toggle.flat = true
		toggle.pressed.connect(toggle_shelf)
		_column.add_child(toggle)
	_keep_on_screen.call_deferred()


func _draw_picker(observation: MineWorldObservation) -> void:
	_column.add_child(_label(Words.text("trade.item.pick"), 13, Color("6b4e2e")))
	for kind in MineWorldTrade.catalogue_of(observation):
		var button := Button.new()
		button.text = kind["name"]
		button.flat = true
		button.pressed.connect(pick.bind(kind["ref"]))
		_column.add_child(button)
	var cancel := Button.new()
	cancel.text = Words.text("trade.cancel")
	cancel.pressed.connect(func() -> void:
		_picking = {}
		_signature = ""
		refresh(_latest))
	_column.add_child(cancel)
	_keep_on_screen.call_deferred()


## Moves the box back inside the window once its size is known.
func _keep_on_screen() -> void:
	_box.reset_size()
	var room: Vector2 = _box.get_viewport_rect().size - _box.size - Vector2(12, 12)
	_box.position = _box.position.clamp(Vector2(12, 12), room.max(Vector2(12, 12)))


static func _label(said: String, size: int, colour: Color) -> Label:
	var label := Label.new()
	label.text = said
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", colour)
	return label
