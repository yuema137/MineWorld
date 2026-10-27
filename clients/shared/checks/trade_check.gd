extends SceneTree

## The trade readers and the row model, headless, over hand-built frames (`clients/shared/TRADE.md` §§4–5;
## S31-e C2, C6).
##
##   godot --headless --path clients/shared --script res://checks/trade_check.gd
##
## The client's table uses neutral type names (`act-*`): the module matches offers to rows by the table's
## tags, never by a type it knows. Prints `[FAIL]` lines and `trade_check: PASS|FAIL`; exits non-zero on a
## failure.

const CROWN := {"name": "crown", "symbol": "cr", "exponent": 0, "symbol_at": "after"}
const INPUTS := {
	"act-list": {"ask": ["amount"], "row": ["held", "note"], "label": "trade.list"},
	"act-price": {"ask": ["amount"], "row": ["ask", "shelf"], "label": "trade.change-price"},
	"act-drop": {"ask": [], "row": ["ask", "shelf"], "label": "trade.withdraw"},
	"act-haggle": {"ask": ["amount"], "row": ["sale"]},
}
const NAMES := {"41": "Flowers", "42": "Scarf", "43": "Bread", "7": "Grace", "8": "Hana"}

var _fails := 0


## A stand-in for `MineWorldObservation`: the calls the module makes, over one frame.
class Seen:
	extends RefCounted
	var me := "1"
	var here := "3"
	var records := {}            ## id -> {component type -> payload}
	var offers: Array = []

	func observer() -> String:
		return me

	func place() -> String:
		return here

	func ids() -> PackedStringArray:
		return PackedStringArray(records.keys())

	func own_component(type: String) -> Dictionary:
		var payload: Variant = component_value(me, type)
		return payload if typeof(payload) == TYPE_DICTIONARY else {}

	func component_value(id: String, type: String) -> Variant:
		return records.get(id, {}).get(type)

	func affordances(action_type: String = "", target: Variant = null) -> Array:
		var out: Array = []
		for offered in offers:
			var at: Variant = offered.get("target")
			var against := "" if at == null else String(at)
			if action_type != "" and offered["action_type"] != action_type:
				continue
			if target != null and against != String(target):
				continue
			out.append(offered)
		return out


static func item(id: String) -> Dictionary:
	return {"entity": id, "entity_type": "item"}


func _init() -> void:
	MineWorldText.load_layers("")
	MineWorldText.set_language("en")
	var trade := MineWorldTrade.new(INPUTS, func(id: String) -> String: return NAMES.get(id, "something"),
		func(reason: Variant) -> String: return "refused " + JSON.stringify(reason))
	_sales(trade)
	_stall(trade)
	_shelf(trade)
	_board(trade)
	_answers()
	print("trade_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


func _sales(trade: MineWorldTrade) -> void:
	var seen := Seen.new()
	var bread := {"action_type": "act-buy", "target": null, "available": true, "payload": {"item": item("43"), "price": 3}}
	var refused := {"action_type": "act-buy", "target": null, "available": false,
		"unavailable_reason": {"system": {"code": "insufficient-funds"}}, "payload": {"item": item("41"), "price": 450}}
	var unpriced := {"action_type": "act-other", "target": null, "available": true, "payload": {"object": "9"}}
	var grace := {"action_type": "act-buy", "target": "7", "available": true, "payload": {"item": item("41"), "price": 450.0}}
	var haggle := {"action_type": "act-haggle", "target": "7", "available": true}
	seen.offers = [bread, unpriced, refused, grace, haggle]
	var rows := trade.sale_rows(seen, "", CROWN)
	_check(rows.size() == 2, "one sale row per priced target-less offer, unpriced ones left out", str(rows.size()))
	if rows.size() == 2:
		_check(rows[0]["text"] == "Bread  ·  3 cr" and rows[0]["available"] == true, "a priced row in crowns", rows[0]["text"])
		_check(rows[0]["buttons"][0]["affordance"] == bread, "Buy carries the offer exactly as offered")
		_check(rows[1]["available"] == false and rows[1]["buttons"].size() == 1, "a refused offer is kept, still pressable")
		_check(String(rows[1]["text"]).contains("refused"), "a refused row carries the server's reason", rows[1]["text"])
	var hers := trade.sale_rows(seen, "7", CROWN)
	_check(hers.size() == 1 and hers[0]["amount"] == 450, "a person's priced offer, by target", str(hers))
	if hers.size() == 1:
		_check(hers[0]["buttons"].size() == 2 and hers[0]["buttons"][1]["fill"] == {"item": item("41")}
			and hers[0]["buttons"][1]["ask"] == PackedStringArray(["amount"]),
			"an incomplete offer tagged `sale` adds a button with the row's item", str(hers[0]["buttons"]))
	seen.records = {"7": {"asks": {"asks": [{"item": item("41"), "price": 450, "available": true},
		{"item": item("42"), "price": 12, "available": false}]}}}
	_check(trade.tag_of(seen, "7", CROWN) == "Flowers 450 cr · +1", "a seller's tag", trade.tag_of(seen, "7", CROWN))
	_check(trade.tag_of(seen, "8", CROWN) == "", "no asks, no tag")


func _stall(trade: MineWorldTrade) -> void:
	var seen := Seen.new()
	seen.records = {"1": {"asks": {"asks": [{"item": item("41"), "price": 450, "available": true}]},
		"holdings": {"held": [{"item": item("41"), "count": 2}, {"item": item("42"), "count": 1}]}}}
	var list := {"action_type": "act-list", "target": null, "available": true}
	var reprice := {"action_type": "act-price", "target": null, "available": true}
	var drop := {"action_type": "act-drop", "target": null, "available": true}
	seen.offers = [list, reprice, drop]
	var rows := trade.stall_rows(seen, CROWN)
	_check(rows.size() == 2, "one ask row and one held row (the asked kind is not offered twice)", str(rows.size()))
	if rows.size() == 2:
		_check(rows[0]["kind"] == "ask" and rows[0]["buttons"].size() == 2, "an ask row: change price, withdraw")
		_check(rows[0]["buttons"][0]["affordance"] == reprice and rows[0]["buttons"][0]["fill"] == {"item": item("41")},
			"change price fills the item reference as disclosed")
		_check(rows[1]["kind"] == "held" and rows[1]["item"] == "42" and rows[1]["buttons"][0]["affordance"] == list,
			"a held row: list it", str(rows[1]))
	seen.offers = []
	_check(trade.stall_rows(seen, CROWN).is_empty(), "no seller offers, no stall")


func _shelf(trade: MineWorldTrade) -> void:
	var seen := Seen.new()
	seen.records = {"3": {"shop": {"operator": {"entity": "20", "entity_type": "organization"},
		"listed": [{"item": item("43"), "price": 3, "in_stock": 6}]}}}
	seen.offers = [{"action_type": "act-list", "target": null, "available": true},
		{"action_type": "act-price", "target": null, "available": true}]
	var rows := trade.shelf_rows(seen, CROWN)
	_check(rows.size() == 2, "one shelf row and one note row", str(rows.size()))
	if rows.size() == 2:
		_check(rows[0]["kind"] == "shelf" and rows[0]["buttons"][0]["fill"] == {"item": item("43"), "shop": true},
			"a shelf row fills the shop side", str(rows[0]["buttons"]))
		_check(rows[1]["kind"] == "note" and rows[1]["buttons"][0]["ask"] == PackedStringArray(["item", "amount"]),
			"the note row asks for the item, then the amount", str(rows[1]["buttons"]))
	var lines := trade.shop_board(MineWorldTrade.listing_of(seen), CROWN)
	_check(lines == PackedStringArray(["Bread 3 cr"]), "the shop's board", str(lines))
	seen.records = {}
	_check(trade.shelf_rows(seen, CROWN).is_empty(), "no listing here, no shelf")


func _board(trade: MineWorldTrade) -> void:
	var seen := Seen.new()
	seen.records = {"1": {"notice-board": {"notices": [
		{"id": 1, "topic": "news.street", "publisher": "8", "body": {"text": "The market opens late."},
			"issued_at": 0, "valid_until": 60},
		{"id": 2, "topic": "prices.street", "publisher": "8", "body": {"prices": [{"item": "43", "price": 280, "basis": 6}]},
			"issued_at": 0, "valid_until": 999999}]}}}
	var rows := trade.board_rows(seen, CROWN, func(at: int) -> String: return "t%d" % at)
	# The first notice's valid_until is long past any clock: the record is the authority, so it is shown.
	_check(rows.size() == 2, "every notice the record holds is shown", str(rows.size()))
	if rows.size() == 2:
		_check(rows[0]["lines"] == PackedStringArray(["The market opens late."]), "a text body", str(rows[0]["lines"]))
		_check(rows[1]["lines"] == PackedStringArray(["Bread 280 cr (6 seen)"]), "a prices body in crowns", str(rows[1]["lines"]))
		_check(rows[1]["title"] == "prices.street  ·  Hana  ·  until t999999", "the title names the publisher", rows[1]["title"])
	_check(MineWorldTrade.has_board(seen), "a board is disclosed")
	seen.records = {}
	_check(not MineWorldTrade.has_board(seen) and trade.board_rows(seen, CROWN, func(at: int) -> String: return "").is_empty(),
		"no board record, no board")


## A hand-built `answer` row (S31-h's, §5.6) goes through the report like any other, with any buttons.
func _answers() -> void:
	var accept := {"action_type": "act-accept", "target": "8", "available": true}
	var decline := {"action_type": "act-decline", "target": "8", "available": false}
	var row := {"kind": "answer", "subject": "8", "item": "43", "text": "Hana offers 2 cr for Bread", "amount": 2,
		"available": null, "reason": null, "buttons": [
			{"label": "Accept", "affordance": accept, "fill": {"proposal": 5}, "ask": PackedStringArray()},
			{"label": "Decline", "affordance": decline, "fill": {"proposal": 5}, "ask": PackedStringArray()}]}
	var shown: Array = MineWorldTrade.report([row])
	_check(shown[0]["buttons"].size() == 2 and shown[0]["buttons"][1]["action_type"] == "act-decline"
		and shown[0]["buttons"][1]["complete"] == false, "an answer row reports both buttons", str(shown))


func _check(ok: bool, what: String, detail: String = "") -> void:
	if not ok:
		_fails += 1
		print("[FAIL] %s  %s" % [what, detail])
