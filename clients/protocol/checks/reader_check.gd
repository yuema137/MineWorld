extends SceneTree

# The reader check for `MineWorldObservation`: the JSON shapes the contract allows and no installed
# pack sends yet, read through the module with no server.
#
#   godot --headless --path clients/protocol --script res://checks/reader_check.gd
#
# Everything a real server sends is checked against a real server by `checks/affordances_check.gd`
# (`run.sh affordances`); this file owns only what that run cannot produce today: an array-valued
# component (12c's listings), a complete affordance whose payload is `null` (a payload-less action),
# an identity nested inside a payload, both reference forms side by side, and target filtering over
# every combination. The frames
# below are synthetic and say so; every expected value is written by hand, never computed by the code
# under test (`step-15-demo-3d.md` §18.4 A-1).
#
# Prints one `[reader] PASS|FAIL <claim>: <observed>` line per claim and exits 1 if any fails.

const Observation := preload("res://mineworld/observation.gd")

# Identities as the protocol carries them: decimal strings, one above 2^53 so that nothing here can
# pass by accident through a float.
const ME := "101"
const ALICE := "9007199254740995"
const BOB := "9007199254740997"
const BOX := "7001"
const COFFEE := "8001"
const TEA := "8002"

var _failures := 0


func _init() -> void:
	var observation = Observation.new(_frame())
	_listing(observation)
	_targets(observation)
	_completeness(observation)
	_components(observation)
	_about(observation)
	print("[reader] %s — %d failure(s)" % ["PASS" if _failures == 0 else "FAIL", _failures])
	quit(0 if _failures == 0 else 1)


## A synthetic observation. Shapes are the contract's (`PROTOCOL.md` §5); the values are invented.
func _frame() -> Dictionary:
	return {
		"observer": ME,
		"at": 0,
		"entities": [
			{ "id": ME, "components": [] },
			{ "id": "500", "components": [
				# A listing whose payload is an array, as 12c's `loose-objects` may be.
				{ "component_type": "listing", "payload": [ { "object": BOX }, { "object": "7002" } ] },
				{ "component_type": "shape", "payload": { "floor": 1 } },
			] },
		],
		"affordances": [
			# 0: incomplete, against Alice
			{ "action_type": "wave", "target": ALICE, "available": true },
			# 1, 2: two complete affordances of one type against one target, differing only in payload
			{ "action_type": "hand", "target": ALICE, "available": false,
				"unavailable_reason": "too_far_away", "payload": { "item": COFFEE, "count": 1 } },
			{ "action_type": "hand", "target": ALICE, "available": false,
				"unavailable_reason": "too_far_away", "payload": { "item": TEA, "count": 1 } },
			# 3: complete and target-less, naming an entity at the top level of its payload
			{ "action_type": "nudge", "target": null, "available": true, "payload": { "object": BOX } },
			# 4: complete, payload null — a payload-less action
			{ "action_type": "shrug", "target": BOB, "available": true, "payload": null },
			# 5: complete and target-less, naming Bob only inside a nested value
			{ "action_type": "point", "target": null, "available": true,
				"payload": { "at": { "who": BOB } } },
			# 6: complete and target-less, naming an item by the contract's typed reference — the
			#    shape every typed identity (ItemId, …) travels in (`TypedEntityRef`)
			{ "action_type": "pick", "target": null, "available": true,
				"payload": { "item": { "entity": TEA, "entity_type": "item" }, "count": 1.0 } },
		],
	}


# A-1a: every match, in order; affordance() still the first.
func _listing(observation) -> void:
	var hands: Array = observation.affordances("hand", ALICE)
	_expect("a. affordances(type, target) returns both, in order",
		_items(hands), [COFFEE, TEA])
	_expect("a. affordance(type, target) is still the first",
		observation.affordance("hand", ALICE).get("payload", {}).get("item"), COFFEE)
	_expect("a. affordances() with no argument is the whole list",
		observation.affordances().size(), 7)


# A-1b: null = any target, "" = target-less, an id = exactly that target.
func _targets(observation) -> void:
	_expect("b. target null matches any", _types(observation.affordances("", null)),
		["wave", "hand", "hand", "nudge", "shrug", "point", "pick"])
	_expect("b. target \"\" matches the target-less", _types(observation.affordances("", "")),
		["nudge", "point", "pick"])
	_expect("b. an id matches exactly that target", _types(observation.affordances("", BOB)),
		["shrug"])


# A-1c: complete means the key is present, even with a null value.
func _completeness(observation) -> void:
	var listed: Array = observation.affordances()
	_expect("c. is_complete per entry", [
		Observation.is_complete(listed[0]), Observation.is_complete(listed[1]),
		Observation.is_complete(listed[4]),
	], [false, true, true])
	_expect("c. complete_affordances() lists the complete subset",
		_types(observation.complete_affordances()),
		["hand", "hand", "nudge", "shrug", "point", "pick"])
	_expect("c. complete_affordances(type, target) narrows like affordances()",
		_types(observation.complete_affordances("wave", ALICE)), [])


# A-1d: a payload as it arrived; component() keeps its Dictionary contract.
func _components(observation) -> void:
	var listing: Variant = observation.component_value("500", "listing")
	_expect("d. component_value returns an array payload as an array",
		[typeof(listing), (listing as Array).size() if typeof(listing) == TYPE_ARRAY else -1],
		[TYPE_ARRAY, 2])
	_expect("d. component_value returns an object payload as itself",
		observation.component_value("500", "shape"), { "floor": 1 })
	_expect("d. component_value is null when undisclosed",
		observation.component_value("500", "nothing"), null)
	_expect("d. component() is {} for an array payload, as before",
		observation.component("500", "listing"), {})


# A-1e: by target, by a top-level payload string, never by a nested one.
func _about(observation) -> void:
	_expect("e. affordances_about(Alice) finds her by target",
		_types(observation.affordances_about(ALICE)), ["wave", "hand", "hand"])
	_expect("e. affordances_about(box) finds a target-less one by its payload",
		_types(observation.affordances_about(BOX)), ["nudge"])
	_expect("e. affordances_about(coffee) finds the one complete choice naming it",
		_items(observation.affordances_about(COFFEE)), [COFFEE])
	_expect("e. affordances_about(tea) finds a typed reference too",
		_types(observation.affordances_about(TEA)), ["hand", "pick"])
	_expect("e. affordances_about(Bob) does not search nested values",
		_types(observation.affordances_about(BOB)), ["shrug"])


func _types(listed: Array) -> Array:
	var found: Array = []
	for offered in listed:
		found.append(String(offered.get("action_type", "")))
	return found


func _items(listed: Array) -> Array:
	var found: Array = []
	for offered in listed:
		var payload: Variant = offered.get("payload")
		found.append(payload.get("item") if typeof(payload) == TYPE_DICTIONARY else null)
	return found


func _expect(claim: String, observed: Variant, expected: Variant) -> void:
	var held: bool = typeof(observed) == typeof(expected) and observed == expected
	if not held:
		_failures += 1
	print("[reader] %s %s: %s%s" % [
		"PASS" if held else "FAIL", claim, JSON.stringify(observed),
		"" if held else " (expected %s)" % JSON.stringify(expected),
	])
