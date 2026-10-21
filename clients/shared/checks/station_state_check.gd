extends SceneTree

## The station fixture rule, headless (`ARC-108`; pr-s29f §5.1, AC-f-5; X-5, X-7).
##
##   godot --headless --path clients/shared --script res://checks/station_state_check.gd
##
## Prints `STATION <case> <board> <window> <kiosk>` per case, `[FAIL]` for a case that differs from
## its expected states, and `all station state checks pass` or `<n> STATION STATE CHECKS FAILED`;
## exits non-zero on any failure.

const State := preload("res://station/station_state.gd")

const HALL := ["station", "public", "landmark"]
const TICKETS := ["station", "ticket-office", "public"]

var _fails := 0


## A stand-in for `MineWorldObservation` built from a literal frame: the four calls `of` makes, read
## the way the protocol module reads them.
class Seen:
	extends RefCounted
	var frame := {}

	func _init(the_frame: Dictionary) -> void:
		frame = the_frame

	func place() -> String:
		return String(frame.get("self_location", {}).get("place", {}).get("entity", ""))

	func entities() -> Array:
		return frame.get("entities", [])

	func entity(id: String) -> Dictionary:
		for e in entities():
			if String(e.get("id", "")) == id:
				return e
		return {}

	func component_value(id: String, component_type: String) -> Variant:
		for record in entity(id).get("components", []):
			if String(record.get("component_type", "")) == component_type:
				return record.get("payload")
		return null


func _init() -> void:
	var news_shop := {"operator": {"entity": "1"}, "listed": [{"item": "9", "price": 2, "in_stock": 4}]}
	_case("hall_nothing", State.evaluate(PackedStringArray(HALL), null, null, {}), "none", "shut", "shut")
	_case("hall_shop", State.evaluate(PackedStringArray(HALL), null, news_shop, {}), "none", "shut", "open")
	_case("tickets_nothing", State.evaluate(PackedStringArray(TICKETS), null, null, {}), "none", "shut", "shut")
	_case("tickets_newspaper", State.evaluate(PackedStringArray(TICKETS), null, news_shop,
		{"9": PackedStringArray(["news"])}), "none", "shut", "shut")
	_case("tickets_ticket", State.evaluate(PackedStringArray(TICKETS), null, news_shop,
		{"9": PackedStringArray(["ticket"])}), "none", "open", "shut")
	_case("tickets_untagged", State.evaluate(PackedStringArray(TICKETS), null, news_shop, {}),
		"none", "shut", "shut")
	_case("departures_planted", State.evaluate(PackedStringArray(HALL), [{"line": "coast"}], null, {}),
		"lines1", "shut", "shut")
	_case("departures_empty", State.evaluate(PackedStringArray(HALL), [], null, {}), "lines0", "shut", "shut")
	_case("variant_tag_order", State.evaluate(PackedStringArray(["public", "ticket-office", "station"]), null,
		news_shop, {"9": PackedStringArray(["ticket"])}), "none", "open", "shut")
	_case("variant_no_landmark", State.evaluate(PackedStringArray(["station", "public"]), null, null, {}),
		"none", "shut", "shut")
	# The typed-reference item form and an observation frame read end to end.
	var frame := {
		"self_location": {"place": {"entity": "5"}},
		"entities": [
			{"id": "5", "entity_type": "place", "tags": TICKETS, "components": [
				{"component_type": "shop", "payload": {"operator": {"entity": "1"},
					"listed": [{"item": {"entity": "9"}, "price": 3, "in_stock": 1}]}},
				{"component_type": "departures", "payload": {"lines": [{"line": "coast"}, {"line": "hills"}]}}]},
			{"id": "9", "entity_type": "item", "tags": ["ticket"], "components": []},
		],
	}
	var seen := Seen.new(frame)
	var by_parts := State.evaluate(PackedStringArray(TICKETS), seen.component_value("5", "departures"),
		seen.component_value("5", "shop"), {"9": PackedStringArray(["ticket"])})
	var whole := State.of(seen)
	_case("of_observation", whole, "lines2", "open", "shut")
	if whole != by_parts:
		_fails += 1
		print("[FAIL] of_observation differs from evaluate on its parts: %s vs %s" % [whole, by_parts])
	_case("unentered", State.nothing_disclosed(), "none", "shut", "shut")
	if _fails == 0:
		print("all station state checks pass")
	else:
		print("%d STATION STATE CHECKS FAILED" % _fails)
	quit(1 if _fails > 0 else 0)


func _case(name: String, got: Dictionary, board: String, window: String, kiosk: String) -> void:
	var b: Dictionary = got.get("board", {})
	var board_text := String(b.get("state", "?"))
	if board_text == "lines":
		board_text += str((b.get("lines", []) as Array).size())
	print("STATION %s %s %s %s" % [name, board_text, got.get("window"), got.get("kiosk")])
	if board_text != board or got.get("window") != window or got.get("kiosk") != kiosk:
		_fails += 1
		print("[FAIL] %s: expected %s %s %s" % [name, board, window, kiosk])
