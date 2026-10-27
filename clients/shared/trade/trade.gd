## The trade records the world discloses, read once for both reference clients, and the rows both draw
## (`clients/shared/TRADE.md` §§4–5; `docs/decisions/ARC-184.md`; S31-e §§5.3, 5.4).
##
## The readers are static: each takes a `MineWorldObservation` (duck-typed, so the module also runs on its
## own) and returns plain data. An instance builds rows: it is made with the client's own table of the
## seller actions it composes (`inputs`), and with two callables the client already has, one that names an
## id and one that words a refusal. So this file names no action, no item, no currency and no place.
##
## Nothing here decides. A row carries the server's verdict on its offer (`available`, `reason`) to be
## shown, and every button carries the offer exactly as the frame did. Two amounts are never compared, a
## reference value is never read, and a notice's expiry is never computed: the record is current.
class_name MineWorldTrade
extends RefCounted

## The components read here (S31-a's `economy`, S31-b's `notices`, 12d's `item`, inventory's holdings).
const CURRENCY := "currency"
const WALLET := "wallet"
const WEALTH := "wealth"
const HOLDINGS := "holdings"
const ASKS := "asks"
const SHOP := "shop"
const CATALOGUE := "item-catalogue"
const BOARD := "notice-board"
const SUBSCRIPTIONS := "subscriptions"
## How many rows a shop's board shows before "+n" (S31-e §5.8).
const BOARD_ROWS := 4

## action type -> `{ask: [what the player supplies], row: [the row kinds it serves], label: wording key}`,
## the client's table (2D `intents.INPUTS`, 3D `SliceIntents.INPUTS`): the only place a type is spelled.
var inputs: Dictionary = {}
## id -> what the world calls it (the client's name memory). Never returns an id.
var names: Callable
## a rejection (a kernel reason or `{system: {code}}`) -> its words.
var reasons: Callable


func _init(an_inputs: Dictionary, a_names: Callable, a_reasons: Callable) -> void:
	inputs = an_inputs
	names = a_names
	reasons = a_reasons


# --- readers -------------------------------------------------------------------------------------------

## The observer's money: `{balance}`, or `{}` with no wallet.
static func money_of(observation: Object) -> Dictionary:
	var wallet: Dictionary = observation.own_component(WALLET)
	return {"balance": int(wallet["balance"])} if typeof(wallet.get("balance")) in [TYPE_INT, TYPE_FLOAT] else {}


## The observer's worth: `{amount, unvalued}`, or `{}` when it is not disclosed.
static func wealth_of(observation: Object) -> Dictionary:
	var worth: Dictionary = observation.own_component(WEALTH)
	if typeof(worth.get("amount")) not in [TYPE_INT, TYPE_FLOAT]:
		return {}
	return {"amount": int(worth["amount"]), "unvalued": int(worth.get("unvalued", 0))}


## What the observer carries: `[{item, ref, count}]`, `item` the id and `ref` the reference as disclosed.
static func held_of(observation: Object) -> Array:
	var holdings: Dictionary = observation.own_component(HOLDINGS)
	var out: Array = []
	for held in _array(holdings.get("held")):
		if typeof(held) == TYPE_DICTIONARY:
			out.append({"item": ref_id(held.get("item")), "ref": held.get("item"), "count": int(held.get("count", 0))})
	return out


## The asks a person discloses: `[{item, ref, price, available}]`, in the record's order.
static func asks_of(observation: Object, person: String) -> Array:
	var record: Variant = observation.component_value(person, ASKS)
	var out: Array = []
	if typeof(record) != TYPE_DICTIONARY:
		return out
	for ask in _array(record.get("asks")):
		if typeof(ask) == TYPE_DICTIONARY:
			out.append({"item": ref_id(ask.get("item")), "ref": ask.get("item"), "price": int(ask.get("price", 0)),
				"available": bool(ask.get("available", false))})
	return out


## The observer's own asks (the `asks` record the world discloses about the observer).
static func own_asks(observation: Object) -> Array:
	return asks_of(observation, observation.observer())


## The listing of the shop at `place` (default: the observer's place): `{operator, listed: [{item, ref,
## price, in_stock}]}`, or `{}` when none is disclosed.
static func listing_of(observation: Object, place: String = "") -> Dictionary:
	var at := place if place != "" else String(observation.place())
	var shop: Variant = observation.component_value(at, SHOP)
	if typeof(shop) != TYPE_DICTIONARY:
		return {}
	var listed: Array = []
	for line in _array(shop.get("listed")):
		if typeof(line) == TYPE_DICTIONARY:
			listed.append({"item": ref_id(line.get("item")), "ref": line.get("item"),
				"price": int(line.get("price", 0)), "in_stock": int(line.get("in_stock", 0))})
	return {"operator": ref_id(shop.get("operator")), "listed": listed}


## The places in view that disclose a shop listing, by id.
static func shops_in_view(observation: Object) -> PackedStringArray:
	var out := PackedStringArray()
	for id in observation.ids():
		if observation.component_value(id, SHOP) != null:
			out.append(id)
	return out


## The kinds the observer's place names: `[{item, ref, category, name}]`.
static func catalogue_of(observation: Object) -> Array:
	var record: Variant = observation.component_value(observation.place(), CATALOGUE)
	var out: Array = []
	if typeof(record) != TYPE_DICTIONARY:
		return out
	for kind in _array(record.get("kinds")):
		if typeof(kind) == TYPE_DICTIONARY and typeof(kind.get("name")) == TYPE_STRING:
			out.append({"item": ref_id(kind.get("item")), "ref": kind.get("item"),
				"category": str(kind.get("category", "")), "name": String(kind["name"])})
	return out


## The notices on the observer's board, in the record's order: `[{id, topic, publisher, body,
## issued_at, valid_until}]`. Every notice the record holds is current; none is dropped here.
static func board_of(observation: Object) -> Array:
	var record: Dictionary = observation.own_component(BOARD)
	var out: Array = []
	for notice in _array(record.get("notices")):
		if typeof(notice) == TYPE_DICTIONARY:
			out.append({"id": ref_id(notice.get("id")), "topic": str(notice.get("topic", "")),
				"publisher": ref_id(notice.get("publisher")), "body": notice.get("body"),
				"issued_at": int(notice.get("issued_at", 0)), "valid_until": int(notice.get("valid_until", 0))})
	return out


## Whether the world discloses a board to the observer at all.
static func has_board(observation: Object) -> bool:
	return observation.own_component(BOARD).has("notices")


## The topics the observer subscribes to.
static func topics_of(observation: Object) -> PackedStringArray:
	var out := PackedStringArray()
	for topic in _array(observation.own_component(SUBSCRIPTIONS).get("topics")):
		out.append(str(topic))
	return out


## An identity as a string, from an identity string, a number or a typed reference `{entity, entity_type}`.
static func ref_id(value: Variant) -> String:
	if typeof(value) == TYPE_DICTIONARY:
		return str(value.get("entity", ""))
	if typeof(value) == TYPE_FLOAT and value == floorf(value):
		return str(int(value))
	return "" if value == null else str(value)


static func _array(value: Variant) -> Array:
	return value if typeof(value) == TYPE_ARRAY else []


# --- the row model (S31-e §5.4) --------------------------------------------------------------------------

## One row per priced offer about `subject` (a person id; "" for the shop where the observer stands), in
## the server's order. A priced offer is a complete affordance whose payload carries an integer price and a
## typed item reference (SD-S31e-3). Its *Buy* sends it unchanged; a refused one is shown, and still sent.
func sale_rows(observation: Object, subject: String, currency: Dictionary) -> Array:
	var rows: Array = []
	var extra := _incomplete(observation, subject, "sale")
	for offered in observation.affordances("", subject):
		if not _priced(offered):
			continue
		var payload: Dictionary = offered["payload"]
		var item := ref_id(payload["item"])
		var price := int(payload["price"])
		var row := _row("sale", subject, item, MineWorldText.text("trade.row.sale", {"item": _name(item),
			"price": MineWorldMoney.format(price, currency)}), price, offered)
		row["buttons"].append(_button(MineWorldText.text("trade.buy"), offered, {}, PackedStringArray()))
		for more in extra:
			row["buttons"].append(_button(_label(more, item), more, {"item": payload["item"]}, _asks(more, "sale")))
		rows.append(row)
	return rows


## My stall: one `ask` row per ask I hold (re-price, withdraw) and one `held` row per kind I carry and do not
## ask for (list it), each with a button per seller offer the client's table tags for that row.
func stall_rows(observation: Object, currency: Dictionary) -> Array:
	var rows: Array = []
	var me := String(observation.observer())
	var asked := {}
	var for_ask := _incomplete(observation, "", "ask")
	var for_held := _incomplete(observation, "", "held")
	for ask in own_asks(observation):
		asked[ask["item"]] = true
		var row := _row("ask", me, ask["item"], MineWorldText.text("trade.row.ask", {"item": _name(ask["item"]),
			"price": MineWorldMoney.format(ask["price"], currency)}), ask["price"], {})
		row["available"] = ask["available"]
		for offered in for_ask:
			row["buttons"].append(_button(_label(offered, ask["item"]), offered, {"item": ask["ref"]}, _asks(offered, "ask")))
		if not for_ask.is_empty():
			rows.append(row)
	for held in held_of(observation):
		if asked.has(held["item"]) or for_held.is_empty():
			continue
		var row := _row("held", me, held["item"], MineWorldText.text("trade.row.held", {"item": _name(held["item"]),
			"count": held["count"]}), null, {})
		for offered in for_held:
			row["buttons"].append(_button(_label(offered, held["item"]), offered, {"item": held["ref"]}, _asks(offered, "held")))
		rows.append(row)
	return rows


## The shop's shelf, for a keeper (SD-S31e-5): one `shelf` row per listed kind and one `note` row to list
## another kind, shown only while a listing is disclosed here and the client's table tags seller offers for
## them. Every button fills `shop: true`; a visitor who presses one is refused by the server.
func shelf_rows(observation: Object, currency: Dictionary) -> Array:
	var rows: Array = []
	var listing := listing_of(observation)
	if listing.is_empty():
		return rows
	var for_shelf := _incomplete(observation, "", "shelf")
	var for_note := _incomplete(observation, "", "note")
	if for_shelf.is_empty() and for_note.is_empty():
		return rows
	for line in listing["listed"]:
		var row := _row("shelf", "", line["item"], MineWorldText.text("trade.row.shelf", {"item": _name(line["item"]),
			"price": MineWorldMoney.format(line["price"], currency), "stock": line["in_stock"]}), line["price"], {})
		for offered in for_shelf:
			row["buttons"].append(_button(_label(offered, line["item"]), offered, {"item": line["ref"], "shop": true},
				_asks(offered, "shelf")))
		rows.append(row)
	if not for_note.is_empty():
		var note := _row("note", "", "", MineWorldText.text("trade.list-for-shop"), null, {})
		for offered in for_note:
			var ask := _asks(offered, "note")
			if not ask.has("item"):
				ask.insert(0, "item")
			note["buttons"].append(_button(MineWorldText.text("trade.list"), offered, {"shop": true}, ask))
		rows.append(note)
	return rows


## Answers to offers made to me, and mine to others: none in S31-e. S31-h fills this from its record
## (`TRADE.md` §5.4); both clients already draw `answer` rows, with any number of buttons, above the rest.
func answer_rows(_observation: Object, _currency: Dictionary) -> Array:
	return []


## A person's price tag: "Flowers 450 cr", "+n" for more asks, sold out when the seller holds none. "" for
## a person who asks nothing.
func tag_of(observation: Object, person: String, currency: Dictionary) -> String:
	var asks := asks_of(observation, person)
	if asks.is_empty():
		return ""
	var first: Dictionary = asks[0]
	var said := MineWorldText.text("trade.tag" if first["available"] else "trade.tag.sold-out",
		{"item": _name(first["item"]), "price": MineWorldMoney.format(first["price"], currency)})
	if asks.size() != 1:
		said = MineWorldText.text("trade.tag.more", {"tag": said, "count": asks.size() - 1})
	return said


## A shop's board: its first BOARD_ROWS listed kinds and prices, one per line, then "+n".
func shop_board(listing: Dictionary, currency: Dictionary) -> PackedStringArray:
	var lines := PackedStringArray()
	var listed: Array = listing.get("listed", [])
	for line in listed.slice(0, BOARD_ROWS):
		lines.append(MineWorldText.text("trade.tag", {"item": _name(line["item"]),
			"price": MineWorldMoney.format(line["price"], currency)}))
	if listed.size() != lines.size():
		lines.append(MineWorldText.text("trade.tag.rest", {"count": listed.size() - lines.size()}))
	return lines


## The board's notices as rows: `{id, topic, publisher, title, lines}`, worded; `clock` words a world time
## (the client's `MineWorldClockFormat` use). Every notice of the record is shown.
func board_rows(observation: Object, currency: Dictionary, clock: Callable) -> Array:
	var rows: Array = []
	for notice in board_of(observation):
		var lines := PackedStringArray()
		var body: Variant = notice["body"]
		if typeof(body) == TYPE_DICTIONARY and typeof(body.get("text")) == TYPE_STRING:
			lines.append(String(body["text"]))
		elif typeof(body) == TYPE_DICTIONARY:
			for line in _array(body.get("prices")):
				if typeof(line) == TYPE_DICTIONARY:
					lines.append(MineWorldText.text("board.row.price", {"item": _name(ref_id(line.get("item"))),
						"price": MineWorldMoney.format(int(line.get("price", 0)), currency),
						"basis": int(line.get("basis", 0))}))
		rows.append({"id": notice["id"], "topic": notice["topic"], "publisher": notice["publisher"],
			"title": MineWorldText.text("board.row.title", {"topic": notice["topic"],
				"publisher": _name(notice["publisher"]), "time": clock.call(notice["valid_until"])}),
			"lines": lines})
	return rows


## Every row of `rows` as a report shows it: no affordance bodies; each button as `{label, action_type,
## complete, ask}` (S31-e §5.12's `TRADE` report).
static func report(rows: Array) -> Array:
	return rows.map(func(row: Dictionary) -> Dictionary:
		var shown := row.duplicate()
		shown["buttons"] = row["buttons"].map(func(b: Dictionary) -> Dictionary:
			return {"label": b["label"], "action_type": String(b["affordance"].get("action_type", "")),
				"complete": b["affordance"].has("payload"), "ask": Array(b["ask"])})
		return shown)


# --- helpers ---------------------------------------------------------------------------------------------

## A complete affordance whose payload is `{item: <typed reference>, price: <integer>, …}`.
static func _priced(offered: Dictionary) -> bool:
	var payload: Variant = offered.get("payload")
	return typeof(payload) == TYPE_DICTIONARY and typeof(payload.get("item")) == TYPE_DICTIONARY \
		and typeof(payload.get("price")) in [TYPE_INT, TYPE_FLOAT]


## The incomplete offers about `subject` whose type the client's table tags for `kind` of row, in order.
func _incomplete(observation: Object, subject: String, kind: String) -> Array:
	var out: Array = []
	for offered in observation.affordances("", subject):
		if offered.has("payload"):
			continue
		var entry: Variant = inputs.get(String(offered.get("action_type", "")))
		if typeof(entry) == TYPE_DICTIONARY and Array(entry.get("row", [])).has(kind):
			out.append(offered)
	return out


func _asks(offered: Dictionary, _kind: String) -> PackedStringArray:
	var entry: Dictionary = inputs.get(String(offered.get("action_type", "")), {})
	return PackedStringArray(entry.get("ask", []))


## A button's words: the table's label key with the item's name, else the action's own wording.
func _label(offered: Dictionary, item: String) -> String:
	var type := String(offered.get("action_type", ""))
	var entry: Dictionary = inputs.get(type, {})
	var args := {"item": _name(item) if item != "" else ""}
	if entry.has("label"):
		return MineWorldText.text(String(entry["label"]), args)
	return MineWorldText.code("action", type, args)


func _name(id: String) -> String:
	return String(names.call(id))


## A row; its verdict is the offer's when it has one (read here, where rows are built, and only shown).
func _row(kind: String, subject: String, item: String, text: String, amount: Variant, offered: Dictionary) -> Dictionary:
	var available: Variant = null
	var reason: Variant = null
	if not offered.is_empty():
		available = bool(offered.get("available", false))
		reason = offered.get("unavailable_reason")
		if not available:
			text = MineWorldText.text("ui.unavailable", {"label": text, "reason": reasons.call(reason)})
	return {"kind": kind, "subject": subject, "item": item, "text": text, "amount": amount,
		"available": available, "reason": reason, "buttons": []}


static func _button(label: String, offered: Dictionary, fill: Dictionary, ask: PackedStringArray) -> Dictionary:
	return {"label": label, "affordance": offered, "fill": fill, "ask": ask}
