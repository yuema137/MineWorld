extends SceneTree

# The far side of facts in observations, the perceived stream and acted_through
# (step-12 §17 CA-15, SD-C11; `server/PROTOCOL.md` §§5.2, 5.8).
#
#   bash clients/protocol/run.sh perceived
#
# which starts `mineworld server worlds/social-cafe --agent alice --town --save <scratch> --hold 10`
# and runs, headless:
#
#   godot --headless --path clients/protocol --script res://checks/perceived_check.gd -- \
#       --address 127.0.0.1:7878 --seat wanderer --invite <the server's invite>
#
# Seated as `wanderer` with `perceive_from(null)` and `reconnect = true`, it
#   1  receives perceived frames from the world's first fact, ascending, never repeated;
#   2  walks two strides to Alice (ac15's positions) and says a line to her; its observations reach
#      acted_through == the talk's action_id, and Alice's answer — a `spoke` this client did not
#      cause — arrives in its own observation.events;
#   3  drops its socket without leave; the module rejoins with its resume and its cursor, is welcomed
#      "held", and the stream continues with no repeated id;
#   4  prints every perceived id and the last cursor, which run.sh compares with the server's own
#      `mineworld perceived --json` once the server has stopped (no gap, no duplicate).
#
# Driven by signals; gives up after [constant BUDGET] seconds. Exits 1 on any failure.

const Client := preload("res://mineworld/world_client.gd")

const BUDGET := 60.0
## ac15's positions: the wanderer's table, then next to Alice (two strides of at most 2 m).
const STRIDES := [[7200, 5050], [7300, 6200]]

var _world: Node
var _address := "127.0.0.1:7878"
var _seat := "wanderer"
var _invite := ""
var _elapsed := 0.0
var _failures := 0
var _done := false

var _welcomes := 0
var _ids: Array = []
var _repeated := 0
var _step := 0
var _alice := ""
var _talk_token := ""
var _talk_action := ""
var _own_actions: Array = []
var _acted_seen := false
var _heard := ""
var _dropped := false
var _after_resume := 0.0


func _initialize() -> void:
	_read_arguments()
	_world = Client.new()
	_world.reconnect = true
	_world.perceive_from(null)
	root.add_child(_world)
	_world.welcomed.connect(_on_welcomed)
	_world.observed.connect(_on_observed)
	_world.perceived.connect(_on_perceived)
	_world.resolved.connect(_on_resolved)
	_world.refused.connect(func(code, _token, detail): _note("refused %s: %s" % [code, detail]))
	_world.disconnected.connect(_on_disconnected)
	_note("connecting to %s as seat '%s', perceived from the beginning" % [_address, _seat])
	_world.connect_to_world(_address, _seat, _invite, "perceived-check")


func _process(delta: float) -> bool:
	_elapsed += delta
	if _welcomes >= 2:
		_after_resume += delta
		if _after_resume > 5.0 and not _done:
			_conclude()
	if not _done and _elapsed > BUDGET:
		_fail("the run finished within %d s" % int(BUDGET), "step %d, %d welcome(s)" % [
			_step, _welcomes,
		])
		_finish()
	return false


func _on_welcomed(_seat_name: String, _observer: String, _summary: Dictionary) -> void:
	_welcomes += 1
	if _welcomes == 2:
		_expect("welcomed again: the seat was held", _world.took_over, "held")


func _on_perceived(events: Array, through: String) -> void:
	for event in events:
		var id := String(event.get("id", ""))
		if _ids.size() > 0 and _id_before(id, String(_ids[-1])) or _ids.has(id):
			_repeated += 1
		_ids.append(id)
	if _ids.size() > 0 and _welcomes == 1 and _step == 0:
		_note("perceived %d facts so far, cursor %s" % [_ids.size(), through])


func _on_observed(observation) -> void:
	if _step == 0 and _alice == "":
		var baristas: PackedStringArray = observation.tagged("barista")
		if baristas.size() == 0:
			return
		_alice = baristas[0]
		_stride(observation)
		return
	if _talk_action != "" and not _acted_seen and observation.acted_through() == _talk_action:
		_acted_seen = true
		_note("an observation reflects the talk: acted_through %s" % _talk_action)
	for spoke in observation.events_of("spoke"):
		var decided: Variant = spoke.get("provenance", {}).get("controller_decision")
		if _heard == "" and decided != null and not _own_actions.has(String(decided)):
			_heard = String(spoke.get("id", ""))
			_note("heard spoke #%s in its own observation.events" % _heard)
	if _acted_seen and _heard != "" and not _dropped:
		_dropped = true
		_note("dropping the socket without leave")
		_world._socket.close()


func _stride(observation) -> void:
	var location: Dictionary = observation.self_location()
	var point: Array = STRIDES[_step]
	_world.submit("move", null, { "to": {
		"place": location["place"],
		"local": { "x": point[0], "y": point[1], "z": 0 },
		"facing": null,
	} })


func _on_resolved(token: String, action_id: String, result: Dictionary) -> void:
	_own_actions.append(action_id)
	if token == _talk_token:
		_talk_action = action_id
		_expect("the talk was accepted", result.has("accepted"), true)
		return
	_expect("stride %d was accepted" % (_step + 1), result.has("accepted"), true)
	_step += 1
	if _step < STRIDES.size():
		_stride(_world.latest)
	else:
		_talk_token = _world.submit("talk", _alice, { "utterance": "hello from the far side" })


func _conclude() -> void:
	_expect("the talk's action_id reached acted_through", _acted_seen, true)
	_expect("a spoke this client did not cause was in its observation.events", _heard != "", true)
	_expect("perceived facts arrived", _ids.size() > 0, true)
	_expect("no perceived id repeated or out of order", _repeated, 0)
	print("[check] ids %s" % ",".join(_ids))
	print("[check] cursor %s" % String(_world.perceived_cursor))
	_finish()


static func _id_before(left: String, right: String) -> bool:
	if left.length() != right.length():
		return left.length() < right.length()
	return left < right


func _read_arguments() -> void:
	var arguments := OS.get_cmdline_user_args()
	for index in range(arguments.size() - 1):
		match String(arguments[index]):
			"--address":
				_address = String(arguments[index + 1])
			"--seat":
				_seat = String(arguments[index + 1])
			"--invite":
				_invite = String(arguments[index + 1])


func _on_disconnected(reason: String) -> void:
	if not _done:
		_fail("the connection came back", reason)
		_finish()


func _finish() -> void:
	if _done:
		return
	_done = true
	_world.leave_world()
	print("[check] %s — %d failure(s)" % ["PASS" if _failures == 0 else "FAIL", _failures])
	quit(0 if _failures == 0 else 1)


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
