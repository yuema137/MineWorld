extends SceneTree

# Revision 2 from the far side, in one run (step-12 §9.5 CP-E2, §20.3 SE-3; risk R-9).
#
#   bash clients/protocol/run.sh rev2
#
# which starts `mineworld server worlds/market-town --agent alice --town --hold 10` with a generated
# admin token in MINEWORLD_ADMIN_TOKEN (the launcher path) and runs, headless, with the same variable:
#
#   godot --headless --path clients/protocol --script res://checks/rev2_check.gd -- \
#       --address 127.0.0.1:7878 --seat visitor --invite <the server's invite>
#
# Seated as `visitor` with `reconnect = true`, in order:
#   join     invite and nickname; welcomed on protocol 2, taken from the town ("hosted")
#   spoke    walks to Alice (`walk-to` her, then `walk-step` until the walk is over, one request per
#            [constant PACE] — the per-seat budget, ARC-97), says a line, and receives her answer: a
#            `spoke` this client did not cause, in its own observation.events
#   deltas   [constant WINDOW] seconds of live frames, applied by the module with no refusal; at
#            [constant PAUSES] host pauses inside the window (POST /admin/clock, the host's call), the
#            reconstruction the module held — accumulated from live deltas since the last keyframe —
#            equals the next keyframe the server sends: nothing changes while the clock is stopped, so
#            the two describe one state (I-10 from the far side; step-12 D-SE5). Each pause is asked
#            once [constant LIVE_DELTAS] live deltas have been applied since the last keyframe, so the
#            reconstruction compared was built by them; meanwhile the visitor paces half a metre back
#            and forth once a second, so its own entity is among what those deltas change
#   resume   drops its socket without leave; the module rejoins with its resume, is welcomed "held",
#            the same Person
#   kick     the host kicks the session (POST /admin/sessions/<id>/kick): closing "kicked"
#
# Driven by signals and frame counts, not sleeps; gives up after [constant BUDGET] seconds. The admin
# token is read from the environment and printed nowhere. Prints `[check] PASS|FAIL <claim>: <observed>`
# lines and exits 1 on any failure.

const Client := preload("res://mineworld/world_client.gd")

## The whole run may take this long, in seconds, before it is a failure.
const BUDGET := 150.0
## The live delta window, in seconds.
const WINDOW := 60.0
## How many host pauses inside it, each compared at one keyframe.
const PAUSES := 3
## The spacing of this client's requests, in seconds: under the default budget of ten a second.
const PACE := 0.11
## A frame counts as "after the pause" only once this long has passed since the paused clock frame:
## a frame swept before the pause may still be queued behind it.
const SETTLE := 0.5
## A pause is asked only once this many live deltas have been applied since the last keyframe, so
## that the reconstruction compared at the next keyframe was built by live deltas, not by empty ones.
const LIVE_DELTAS := 25
## While the window runs, the visitor paces half a metre back and forth this often, in seconds, so
## that its own entity changes in the deltas the pauses compare.
const SHUFFLE := 1.0
## What the visitor says to Alice.
const LINE := "hello Alice, this is the far side"

enum Step { JOIN, WALK, TALK, HEAR, DELTAS, PAUSING, PAUSED, RESUMING, DROP, KICK, DONE }

var _world: Node
var _http: HTTPRequest
var _address := "127.0.0.1:7878"
var _seat := "visitor"
var _invite := ""
var _token := ""
var _elapsed := 0.0
var _failures := 0
var _done := false
var _step := Step.JOIN

var _observer := ""
var _alice := ""
var _own_actions: Array = []
var _talk_token := ""
var _next_request_at := 0.0
var _pending_request: Array = []

var _window_started := 0.0
var _pauses_done := 0
var _paused_at := -1.0
var _held_frame: Dictionary = {}
var _held_at := -1.0
var _deltas_seen := 0
var _want_pause := false
var _last_keyframe_at := -1.0
var _since_keyframe := 0
var _live_at_pause := 0
var _keyframe_when_asked := -1.0
var _retries := 0
var _anchor: Dictionary = {}
var _shuffles := 0
var _shuffled := 0
var _next_shuffle_at := 0.0
var _compared := 0
var _closing_reason := ""
var _welcomes := 0

var _pending_http: Array = []
var _http_busy := false


func _initialize() -> void:
	_read_arguments()
	_token = OS.get_environment("MINEWORLD_ADMIN_TOKEN")
	if _token == "":
		_fail("an admin token in MINEWORLD_ADMIN_TOKEN", "none")
		_finish()
		return
	_world = Client.new()
	_world.reconnect = true
	_http = HTTPRequest.new()
	root.add_child(_world)
	root.add_child(_http)
	_http.request_completed.connect(_on_answered)
	_world.welcomed.connect(_on_welcomed)
	_world.observed.connect(_on_observed)
	_world.resolved.connect(_on_resolved)
	_world.clock_changed.connect(_on_clock)
	_world.refused.connect(func(code, _t, detail): _note("refused %s: %s" % [code, detail]))
	_world.closing.connect(func(reason, _detail): _closing_reason = reason)
	_world.disconnected.connect(_on_disconnected)
	_note("connecting to %s as seat '%s'" % [_address, _seat])
	_world.connect_to_world(_address, _seat, _invite, "rev2-check")


func _process(delta: float) -> bool:
	_elapsed += delta
	if not _pending_request.is_empty() and _elapsed >= _next_request_at:
		var request: Array = _pending_request
		_pending_request = []
		_next_request_at = _elapsed + PACE
		var token: String = _world.submit(request[0], request[1], request[2])
		if request[0] == "talk":
			_talk_token = token
	if _step == Step.DELTAS and _elapsed >= _next_shuffle_at and _pending_request.is_empty():
		_next_shuffle_at = _elapsed + SHUFFLE
		_shuffle()
	if _step == Step.DELTAS:
		var into := _elapsed - _window_started
		if into > WINDOW:
			_conclude_window()
		elif _pauses_done < PAUSES and into > WINDOW * float(_pauses_done + 1) / float(PAUSES + 1):
			_want_pause = true
	if not _done and _elapsed > BUDGET:
		_fail("the run finished within %d s" % int(BUDGET), "stuck at step %d" % _step)
		_finish()
	return false


## One request, sent no sooner than [constant PACE] after the previous one.
func _request(action_type: String, target: Variant, payload: Dictionary) -> void:
	_pending_request = [action_type, target, payload]


func _on_welcomed(_seat_name: String, observer: String, summary: Dictionary) -> void:
	_welcomes += 1
	if _welcomes == 1:
		_observer = observer
		_expect("welcomed on protocol 2", int(summary.get("protocol", 0)), 2)
		_expect("taken from the town", _world.took_over, "hosted")
		_expect("the nickname is this player's", _world.nickname, "rev2-check")
		_step = Step.WALK
	elif _step == Step.DROP:
		_expect("welcomed again: the seat was held for this player", _world.took_over, "held")
		_expect("the same Person", observer, _observer)
		_step = Step.KICK
		_post("/admin/sessions/%s/kick" % _world.session, {})
	else:
		_fail("one welcome per join", "a welcome at step %d" % _step)


func _on_observed(observation) -> void:
	match _step:
		Step.WALK:
			if _alice == "":
				var baristas: PackedStringArray = observation.tagged("barista")
				if baristas.size() == 0:
					return
				_alice = baristas[0]
				_request("walk-to", null, {"to": {"person": {"entity": _alice, "entity_type": "person"}}})
		Step.HEAR:
			for spoke in observation.events_of("spoke"):
				var decided: Variant = spoke.get("provenance", {}).get("controller_decision")
				if decided != null and not _own_actions.has(String(decided)):
					_note("heard spoke #%s in its own observation.events" % String(spoke.get("id", "")))
					_expect("a spoke this client did not cause arrived", true, true)
					_start_window()
					return
		Step.PAUSED:
			_compare_at_keyframe(observation)
	_track(observation)
	if _step == Step.DELTAS and _want_pause and _since_keyframe >= LIVE_DELTAS:
		_want_pause = false
		_live_at_pause = _since_keyframe
		_keyframe_when_asked = _last_keyframe_at
		_step = Step.PAUSING
		_post("/admin/clock", {"paused": true})


## Remembers the frame the module now holds, and whether a delta built it.
func _track(observation) -> void:
	if _world.deltas_applied == _deltas_seen:
		_last_keyframe_at = _elapsed
		_since_keyframe = 0
	else:
		_since_keyframe += 1
	_held_frame = observation.frame
	_held_at = _elapsed
	_deltas_seen = _world.deltas_applied


## Inside a pause: once a frame received after [constant SETTLE] is held, the next keyframe must equal it.
func _compare_at_keyframe(observation) -> void:
	var keyframe: bool = _world.deltas_applied == _deltas_seen
	var settled: bool = _held_at - _paused_at >= SETTLE
	if keyframe and settled:
		_step = Step.RESUMING
		if _last_keyframe_at != _keyframe_when_asked or _live_at_pause < LIVE_DELTAS:
			# A keyframe arrived after the pause was asked, so the frame held was not built by live
			# deltas: nothing to compare. Not counted; the pause is asked again.
			_retries += 1
			_note("a keyframe arrived after the pause was asked; this pause is not counted")
			_post("/admin/clock", {"paused": false})
			return
		var equal: bool = _held_frame == observation.frame
		_expect("pause %d: the reconstruction held, %d live deltas past a keyframe, equals the keyframe"
				% [_pauses_done + 1, _live_at_pause], equal, true)
		if not equal:
			_note("held:     %s" % JSON.stringify(_held_frame))
			_note("keyframe: %s" % JSON.stringify(observation.frame))
		_compared += 1
		_pauses_done += 1
		_post("/admin/clock", {"paused": false})


## One `move` half a metre east of where the window began, or back: the visitor's own entity changes.
func _shuffle() -> void:
	if _anchor.is_empty():
		return
	var local: Dictionary = _anchor["local"]
	var east := 500 if _shuffles % 2 == 0 else 0
	_shuffles += 1
	_request("move", null, {"to": {
		"place": _anchor["place"],
		"local": {"x": int(local["x"]) + east, "y": int(local["y"]), "z": 0},
		"facing": null,
	}})


func _start_window() -> void:
	_step = Step.DELTAS
	_anchor = _world.latest.self_location() if _world.latest != null else {}
	_window_started = _elapsed
	_note("%d s of deltas, with %d host pauses" % [int(WINDOW), PAUSES])


func _conclude_window() -> void:
	_expect("deltas were applied", _world.deltas_applied > 300, true)
	_expect("none refused (no base mismatch, no unknown remove)", _world.deltas_refused, 0)
	_expect("every pause was compared at a keyframe", _compared, PAUSES)
	_expect("the visitor moved while the window ran", _shuffled >= 20, true)
	_note("applied %d deltas; %d pause(s) not counted" % [_world.deltas_applied, _retries])
	_step = Step.DROP
	_note("dropping the socket without leave")
	_world._socket.close()


func _on_resolved(token: String, action_id: String, result: Dictionary) -> void:
	_own_actions.append(action_id)
	match _step:
		Step.WALK:
			if result.has("accepted"):
				_request("walk-step", null, {})
			elif token != "" and _alice != "":
				# The walk is over (precondition_failed): in reach. Say the line.
				_step = Step.TALK
				_request("talk", _alice, {"utterance": LINE})
		Step.DELTAS, Step.PAUSING:
			if result.has("accepted"):
				_shuffled += 1
		Step.TALK:
			if token == _talk_token:
				_expect("the talk was accepted", result.has("accepted"), true)
				_step = Step.HEAR


func _on_clock(_at: int, _time_scale: int, paused: bool) -> void:
	match _step:
		Step.PAUSING:
			if paused:
				_paused_at = _elapsed
				_step = Step.PAUSED
		Step.RESUMING:
			if not paused:
				_step = Step.DELTAS


func _on_disconnected(reason: String) -> void:
	if _done or _step == Step.DROP:
		return
	if _step == Step.KICK:
		_expect("kicked: closing said why before the connection ended", _closing_reason, "kicked")
		_step = Step.DONE
	else:
		_fail("the connection lasted until the kick", reason)
	_finish()


## One admin request, as the host makes it: HTTP POST, the bearer token, a JSON body, one at a time.
func _post(path: String, body: Dictionary) -> void:
	_pending_http.append([path, body])
	if not _http_busy:
		_send_next()


func _send_next() -> void:
	if _pending_http.is_empty():
		_http_busy = false
		return
	_http_busy = true
	var next: Array = _pending_http.pop_front()
	var headers := PackedStringArray([
		"Authorization: Bearer %s" % _token,
		"Content-Type: application/json",
	])
	var error := _http.request(
		"http://%s%s" % [_address, next[0]], headers, HTTPClient.METHOD_POST, JSON.stringify(next[1])
	)
	if error != OK:
		_fail("POST %s was sent" % next[0], str(error))
		_finish()


func _on_answered(_result: int, code: int, _headers: PackedStringArray, _body: PackedByteArray) -> void:
	if code != 200:
		_fail("the admin surface answered 200", str(code))
		_finish()
		return
	_send_next()


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


func _finish() -> void:
	if _done:
		return
	_done = true
	if _world != null and _world.is_seated():
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
