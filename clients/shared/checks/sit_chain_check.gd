extends SceneTree

## The walk-then-sit chain, headless, over hand-built frames (`clients/shared/sitting/sit_chain.gd`; S34-e C1).
##
##   godot --headless --path clients/shared --script res://checks/sit_chain_check.gd
##
## The chain names no action: the client's types are neutral (`act-sit`, `act-stand`). Each case builds the
## frame the server would send and checks the chain's one answer. Prints `[FAIL]` lines and
## `sit_chain_check: PASS|FAIL`; exits non-zero on a failure.

const SIT := "act-sit"
const STAND := "act-stand"
const PLACE := "p-cafe"
const AT := {"x": 3648, "y": 3060}

var _fails := 0


func _init() -> void:
	var chain := MineWorldSitChain.new(SIT, STAND)
	var near := _frame(PLACE, [_sit("stool-1", true, "")], [])
	var choice := chain.choose(near, "stool-1")
	_check(choice.has("submit") and choice["submit"] == near.complete_affordances(SIT, "")[0],
		"in reach: the offer is sent exactly as offered", str(choice))

	var far := _frame(PLACE, [_sit("stool-1", false, "too_far_away")], [])
	choice = chain.choose(far, "stool-1")
	_check(choice.has("walk") and choice["walk"]["place"] == PLACE and choice["walk"]["local"] == AT,
		"too far: walk to the point's disclosed position, not a sit", str(choice))

	var arrived := _frame(PLACE, [_sit("stool-1", true, "")], [])
	choice = chain.walk_ended(arrived, true)
	_check(choice.has("submit") and choice["submit"] == arrived.complete_affordances(SIT, "")[0],
		"walk completed in the same place: the newest offer is sent", str(choice))

	chain.choose(far, "stool-1")
	_check(chain.walk_ended(arrived, false).is_empty(), "walk stopped short: nothing is sent")

	chain.choose(far, "stool-1")
	var elsewhere := _frame("p-street", [_sit("stool-1", true, "")], [])
	_check(chain.walk_ended(elsewhere, true).is_empty(), "walk ended in another place: nothing is sent")

	chain.choose(far, "stool-1")
	chain.cancel()
	_check(chain.walk_ended(arrived, true).is_empty(), "cancelled walk: nothing is sent")

	var busy := _frame(PLACE, [_sit("stool-1", false, "busy")], [])
	choice = chain.choose(busy, "stool-1")
	_check(choice.has("submit") and not choice.has("walk"), "another refusal: sent, never walked to", str(choice))

	var none := _frame(PLACE, [_sit("stool-2", true, "")], [])
	_check(chain.choose(none, "stool-1").is_empty(), "no offer for the point: nothing")

	var seated := _frame(PLACE, [_sit("stool-1", true, "")], [_stand(true)])
	choice = chain.leave(seated)
	_check(choice.has("submit") and choice["submit"]["action_type"] == STAND, "leave: the offered stand is sent", str(choice))
	_check(chain.leave(near).is_empty(), "leave with no offered stand: nothing")

	print("sit_chain_check: %s" % ("PASS" if _fails == 0 else "FAIL"))
	quit(0 if _fails == 0 else 1)


## A complete sit offer of point `point`: reachable, or refused with `reason`.
func _sit(point: String, available: bool, reason: String) -> Dictionary:
	var offer := {"action_type": SIT, "target": null, "payload": {"point": point}, "available": available}
	if reason != "":
		offer["unavailable_reason"] = reason
	return offer


func _stand(available: bool) -> Dictionary:
	return {"action_type": STAND, "target": null, "payload": {}, "available": available}


## A stand-in for `MineWorldObservation`: the three calls the chain makes, over one frame (this project cannot
## see the protocol module; `clients/2d` runs the same chain over the real reader).
class Seen:
	extends RefCounted
	var frame: Dictionary

	func _init(the_frame: Dictionary) -> void:
		frame = the_frame

	func place() -> String:
		return String(frame["self_location"]["place"]["entity"])

	func component_value(id: String, component_type: String) -> Variant:
		for entity in frame["entities"]:
			if String(entity["id"]) == id:
				for record in entity.get("components", []):
					if String(record["component_type"]) == component_type:
						return record["payload"]
		return null

	## Every complete affordance of `action_type`, directed at `target` ("" is nobody), in list order.
	func complete_affordances(action_type: String, target: String = "") -> Array:
		var found := []
		for offered in frame["affordances"]:
			var against: Variant = offered.get("target")
			var directed := "" if against == null else String(against)
			if String(offered["action_type"]) == action_type and directed == target and offered.has("payload"):
				found.append(offered)
		return found


## A frame of one observer at `place`, with the place's disclosed sit points, and `offers` (plus `extra`).
func _frame(place: String, sits: Array, extra: Array) -> Seen:
	var points := [{"key": "stool-1", "at": AT, "facing": 81406}]
	var frame := {"observer": "1", "at": 100,
		"self_location": {"place": {"entity": place}, "local": {"x": 3000, "y": 3000}},
		"entities": [
			{"id": place, "entity_type": "place", "tags": ["cafe"],
				"components": [{"component_type": MineWorldSitChain.DISCLOSED, "payload": points}]},
			{"id": "1", "entity_type": "person", "location": {"place": {"entity": place}, "local": {"x": 3000, "y": 3000}}},
		],
		"affordances": sits + extra}
	return Seen.new(frame)


func _check(ok: bool, what: String, detail: String = "") -> void:
	if not ok:
		_fails += 1
		print("[FAIL] %s  %s" % [what, detail])
