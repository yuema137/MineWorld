extends SceneTree

# The live check for the module's affordance readers, `submit_affordance` and `revision`, against a
# real server (`step-15-demo-3d.md` §18.4 A-2 … A-4).
#
#   bash clients/protocol/run.sh affordances
#
# which starts `mineworld server worlds/market-town --agent alice --save <a temporary directory>` and
# runs, headless:
#
#   godot --headless --path clients/protocol --script res://checks/affordances_check.gd -- \
#       --address 127.0.0.1:7878 --seat visitor
#
# Seated as `visitor` at the café door, it reads what the world offers, then:
#   1. submits the incomplete `talk` through submit_affordance — nothing may be sent;
#   2. submits Alice's first complete `give`, which the server says is unavailable — it must be sent,
#      and the server must answer;
#   3. submits one available complete `buy` unchanged — accepted, and the wallet, holdings and
#      revision then move.
#
# Like every client, it decides nothing: it never measures a distance and never reads `available` to
# choose whether to submit. It knows two action names, `buy` and `give`, only to say which of the
# world's affordances this check is about, and `talk` only to pick an incomplete one; the market's
# content (six priced kinds at the café, two kinds held) is what it expects, read from
# `worlds/market-town`. Driven by observations and answers, never by sleeps; it gives up after
# [constant BUDGET] seconds. Prints `[check] PASS|FAIL <claim>: <observed>` lines and exits 1 on any
# failure.

const Client := preload("res://mineworld/world_client.gd")
const Observation := preload("res://mineworld/observation.gd")

## The whole run may take this long, in seconds, before it is a failure.
const BUDGET := 60.0
## `worlds/market-town/organizations/cafe-company.yaml` prices six kinds at the café.
const CAFE_KINDS := 6
## `worlds/market-town/people/visitor.yaml` holds two kinds, `apple` and `scarf`.
const VISITOR_KINDS := 2

var _world: Node
var _address := "127.0.0.1:7878"
var _seat := "visitor"
var _elapsed := 0.0
var _failures := 0
var _done := false

var _step := 0
var _submitted: Array = []           # every submitted_request, as [token, request]
var _revisions: Array = []           # every revision seen, in order
var _give_token := ""
var _buy_token := ""
var _buy: Dictionary = {}
var _buy_item := ""
var _price := -1
var _wallet_before := -1
var _held_before := -1
var _revision_at_buy: Variant = null


func _initialize() -> void:
	_read_arguments()
	_world = Client.new()
	root.add_child(_world)
	_world.observed.connect(_on_observed)
	_world.resolved.connect(_on_resolved)
	_world.refused.connect(_on_refused)
	_world.disconnected.connect(_on_disconnected)
	_world.submitted_request.connect(func(token, request): _submitted.append([token, request]))
	_note("connecting to %s as seat '%s'" % [_address, _seat])
	_world.connect_to_world(_address, _seat)


func _process(delta: float) -> bool:
	_elapsed += delta
	if not _done and _elapsed > BUDGET:
		_fail("the run finished within %d s" % int(BUDGET), "step %d after %.1f s" % [_step, _elapsed])
		_finish()
	return false


func _read_arguments() -> void:
	var arguments := OS.get_cmdline_user_args()
	for index in range(arguments.size() - 1):
		match String(arguments[index]):
			"--address":
				_address = String(arguments[index + 1])
			"--seat":
				_seat = String(arguments[index + 1])


# ------------------------------------------------------------------------------------------------

func _on_observed(observation) -> void:
	_revisions.append(_world.revision)
	if _step == 0:
		_step = 1
		_first_view(observation)
	elif _step == 3:
		_after_buy(observation)


## A-2, then the three submissions of A-3 and A-4.
func _first_view(observation) -> void:
	var me: String = _world.observer
	var alice := _first(observation.tagged("barista"))
	_note("observer %s, Alice %s, revision %s" % [me, alice, JSON.stringify(_world.revision)])

	# A-2: listing.
	for offered in observation.complete_affordances():
		_note("complete: %s" % JSON.stringify(offered))
	var buys: Array = observation.complete_affordances("buy")
	_expect("A-2 one complete buy per kind the café prices", buys.size(), CAFE_KINDS)
	_expect("A-2 every buy is target-less", _targets(buys).filter(func(t): return t != null), [])
	var gives: Array = observation.affordances("give", alice)
	_expect("A-2 two give affordances against Alice, one per kind held", gives.size(), VISITOR_KINDS)
	_expect("A-2 both are complete",
		gives.filter(func(a): return Observation.is_complete(a)).size(), VISITOR_KINDS)
	_expect("A-2 they differ only in payload, in the server's order",
		_order_preserved(observation.affordances(), gives), true)
	if gives.size() == 2:
		_expect("A-2 their payload items differ",
			_ref(gives[0]["payload"].get("item")) != _ref(gives[1]["payload"].get("item")), true)
	var talk: Dictionary = observation.affordance("talk", alice)
	_expect("A-2 talk to Alice is offered and incomplete",
		[talk.is_empty(), Observation.is_complete(talk)], [false, false])
	if not buys.is_empty():
		var item := _ref(buys[0]["payload"].get("item"))
		_expect("A-2 affordances_about(item) contains the buy naming it",
			observation.affordances_about(item).has(buys[0]), true)
	_expect("A-4 revision is an int (a saved world)", typeof(_world.revision), TYPE_INT)

	# A-4: an incomplete affordance is refused without sending anything.
	var before := _submitted.size()
	var token: String = _world.submit_affordance(talk)
	_expect("A-4 submit_affordance(incomplete talk) returns \"\" and sends nothing",
		[token, _submitted.size() - before], ["", 0])

	# A-4: an unavailable complete affordance is sent anyway; the server answers.
	if not gives.is_empty():
		_note("Alice's first give is available=%s, reason %s" % [
			gives[0].get("available"), JSON.stringify(gives[0].get("unavailable_reason"))])
		_give_token = _world.submit_affordance(gives[0])
		_expect("A-4 submit_affordance(unavailable give) is sent", _give_token.is_empty(), false)

	# A-3: one available buy, unchanged.
	for offered in buys:
		if bool(offered.get("available", false)):
			_buy = offered
			break
	if _buy.is_empty():
		_fail("A-3 an available buy exists", "none of %d" % buys.size())
		_finish()
		return
	_buy_item = _ref(_buy["payload"].get("item"))
	_price = _price_of(observation, _buy_item)
	_wallet_before = _balance(observation, me)
	_held_before = _held(observation, me, _buy_item)
	_revision_at_buy = _world.revision
	_buy_token = _world.submit_affordance(_buy)
	var sent: Array = _submitted.back() if not _submitted.is_empty() else ["", {}]
	var request: Dictionary = sent[1]
	_expect("A-3 the request is the affordance, unchanged", [
		sent[0] == _buy_token,
		request.get("action_type") == _buy["action_type"],
		request.get("target") == _buy["target"],
		JSON.stringify(request.get("payload", {}).get("payload")) == JSON.stringify(_buy["payload"]),
		request.get("payload", {}).get("action_type") == _buy["action_type"],
	], [true, true, true, true, true])
	_note("bought item %s at price %d; wallet %d, held %d, revision %s" % [
		_buy_item, _price, _wallet_before, _held_before, JSON.stringify(_revision_at_buy)])
	_step = 2


func _on_resolved(token: String, action_id: String, result: Dictionary) -> void:
	_note("%s -> action %s: %s" % [token, action_id, JSON.stringify(result)])
	if token == _give_token:
		_expect("A-4 the server answers the unavailable give: rejected too_far_away",
			result.get("rejected"), "too_far_away")
	elif token == _buy_token:
		_expect("A-3 the buy is accepted", result.has("accepted"), true)
		_step = 3 if result.has("accepted") else 4
		if _step == 4:
			_finish()


func _after_buy(observation) -> void:
	var me: String = _world.observer
	var wallet := _balance(observation, me)
	if wallet == _wallet_before:
		return  # this observation was computed before the purchase; wait for the next
	_expect("A-3 the wallet fell by the price", _wallet_before - wallet, _price)
	_expect("A-3 holdings of that kind rose by one", _held(observation, me, _buy_item) - _held_before, 1)
	_expect("A-4 revision rose after the accepted buy",
		typeof(_world.revision) == TYPE_INT and typeof(_revision_at_buy) == TYPE_INT
			and int(_world.revision) > int(_revision_at_buy), true)
	_finish()


func _on_refused(code: String, token: String, detail: String) -> void:
	_fail("no frame is refused", "%s %s %s" % [token, code, detail])


func _on_disconnected(reason: String) -> void:
	if not _done:
		_fail("the connection stays open", reason)
		_finish()


func _finish() -> void:
	if _done:
		return
	_done = true
	_expect("A-4 revision never decreased", _never_decreases(_revisions), true)
	_note("revisions seen: %d observations, first %s, last %s" % [
		_revisions.size(), JSON.stringify(_revisions.front()), JSON.stringify(_revisions.back())])
	_world.disconnect_from_world("the check is over")
	print("[check] %s — %d failure(s)" % ["PASS" if _failures == 0 else "FAIL", _failures])
	quit(0 if _failures == 0 else 1)


# ------------------------------------------------------------------------------------------------
# Reading the market's disclosures: what the world told this observer, and nothing else.

## The price the café's disclosed `shop` listing gives for one kind, or -1.
func _price_of(observation, item: String) -> int:
	var shop: Variant = observation.component_value(observation.place(), "shop")
	if typeof(shop) != TYPE_DICTIONARY:
		return -1
	for key in ["listed", "prices"]:
		for entry in shop.get(key, []):
			if _ref(entry.get("item")) == item:
				return int(entry.get("price", -1))
	return -1


func _balance(observation, me: String) -> int:
	var wallet: Variant = observation.component_value(me, "wallet")
	return int(wallet.get("balance", -1)) if typeof(wallet) == TYPE_DICTIONARY else -1


func _held(observation, me: String, item: String) -> int:
	var holdings: Variant = observation.component_value(me, "holdings")
	if typeof(holdings) != TYPE_DICTIONARY:
		return -1
	for entry in holdings.get("held", []):
		if _ref(entry.get("item")) == item:
			return int(entry.get("count", 0))
	return 0


# ------------------------------------------------------------------------------------------------

## The identity a reference names: an identity string as it is, or a typed reference's `entity`.
func _ref(value: Variant) -> String:
	if typeof(value) == TYPE_STRING:
		return value
	if typeof(value) == TYPE_DICTIONARY and typeof(value.get("entity")) == TYPE_STRING:
		return value["entity"]
	return ""


func _first(found: PackedStringArray) -> String:
	return "" if found.is_empty() else String(found[0])


func _targets(listed: Array) -> Array:
	return listed.map(func(a): return a.get("target"))


## Whether `subset` appears in `whole` in the same relative order.
func _order_preserved(whole: Array, subset: Array) -> bool:
	var at := -1
	for entry in subset:
		var found := -1
		for index in range(at + 1, whole.size()):
			if whole[index] == entry:
				found = index
				break
		if found < 0:
			return false
		at = found
	return true


func _never_decreases(seen: Array) -> bool:
	var last := -1
	for value in seen:
		if typeof(value) != TYPE_INT:
			return false
		if int(value) < last:
			return false
		last = int(value)
	return true


func _expect(claim: String, observed: Variant, expected: Variant) -> void:
	var held: bool = typeof(observed) == typeof(expected) and observed == expected
	if not held:
		_failures += 1
	print("[check] %s %s: %s%s" % [
		"PASS" if held else "FAIL", claim, JSON.stringify(observed),
		"" if held else " (expected %s)" % JSON.stringify(expected),
	])


func _fail(claim: String, observed: String) -> void:
	_failures += 1
	print("[check] FAIL %s: %s" % [claim, observed])


func _note(line: String) -> void:
	print("[check] %s" % line)
