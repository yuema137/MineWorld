extends SceneTree

# The live check for the module's opt-in reconnect (step-12 §16.4 SB-10, SD-B14).
#
#   bash clients/protocol/run.sh reconnect
#
# which starts `mineworld server worlds/market-town --agent alice --town --hold 10` and runs, headless:
#
#   godot --headless --path clients/protocol --script res://checks/reconnect_check.gd -- \
#       --address 127.0.0.1:7878 --seat visitor --invite <the server's invite>
#
# Seated as `visitor` (the town was driving it: took_over "hosted") with `reconnect = true`, it drops
# its own socket without `leave` after the first observation — closing the module's WebSocketPeer, the
# same event a blinking network produces — and expects the module to say `reconnecting(1)` and be
# welcomed again with took_over "held", as the same observer, inside the server's hold.
#
# It reads no resume secret and prints none: the module keeps it. Driven by signals, never by sleeps;
# gives up after [constant BUDGET] seconds. Prints `[check] PASS|FAIL <claim>: <observed>` lines and
# exits 1 on any failure.

const Client := preload("res://mineworld/world_client.gd")

## The whole run may take this long, in seconds, before it is a failure.
const BUDGET := 30.0

var _world: Node
var _address := "127.0.0.1:7878"
var _seat := "visitor"
var _invite := ""
var _elapsed := 0.0
var _failures := 0
var _done := false

var _welcomes := 0
var _observer := ""
var _dropped := false
var _attempts: Array = []


func _initialize() -> void:
	_read_arguments()
	_world = Client.new()
	_world.reconnect = true
	root.add_child(_world)
	_world.welcomed.connect(_on_welcomed)
	_world.observed.connect(_on_observed)
	_world.reconnecting.connect(func(attempt): _attempts.append(attempt))
	_world.closing.connect(func(reason, _detail): _note("closing %s" % reason))
	_world.disconnected.connect(_on_disconnected)
	_note("connecting to %s as seat '%s', reconnect on" % [_address, _seat])
	_world.connect_to_world(_address, _seat, _invite, "reconnect-check")


func _process(delta: float) -> bool:
	_elapsed += delta
	if not _done and _elapsed > BUDGET:
		_fail("the run finished within %d s" % int(BUDGET), "%d welcome(s) after %.1f s" % [
			_welcomes, _elapsed,
		])
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
			"--invite":
				_invite = String(arguments[index + 1])


func _on_welcomed(_seat_name: String, observer: String, _summary: Dictionary) -> void:
	_welcomes += 1
	if _welcomes == 1:
		_observer = observer
		_expect("the town was driving the seat", _world.took_over, "hosted")
		_expect("the server holds a dropped seat for 10 s", _world.hold_seconds, 10)
		_expect("a resume secret was given (not shown)", _world.resume != null, true)
		return
	_expect("one reconnect attempt, announced", _attempts, [1])
	_expect("welcomed again: the seat was held for this player", _world.took_over, "held")
	_expect("the same Person", observer, _observer)
	_finish()


func _on_observed(_observation) -> void:
	if _welcomes == 1 and not _dropped:
		_dropped = true
		_note("dropping the socket without leave")
		# The module's own socket, closed under it: what a dropped network looks like from here.
		_world._socket.close()


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
