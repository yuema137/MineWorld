extends SceneTree

# The live check for the `clock` frame and the admin surface, from the far side (step-12 §18.5 DA-10).
#
#   bash clients/protocol/run.sh admin
#
# which starts `mineworld server worlds/social-cafe --agent alice` with a generated admin token in
# MINEWORLD_ADMIN_TOKEN (the launcher path, never a command-line argument) and runs, headless, with the
# same variable set:
#
#   godot --headless --path clients/protocol --script res://checks/admin_check.gd -- \
#       --address 127.0.0.1:7878 --seat visitor --invite <the server's invite>
#
# Seated as `visitor`, it expects:
#   1. clock_changed(paused false) right after the welcome;
#   2. after this script's own POST /admin/clock {"paused": true} — the host's call, over HTTP, with
#      the bearer token — clock_changed(paused true), and the module's `paused` set;
#   3. a submit while paused refused `paused`, with its own token;
#   4. after POST {"paused": false}, clock_changed(paused false);
#   5. after POST /admin/sessions/<its session>/kick, closing "kicked" before disconnected.
#
# The module decides nothing here: it reads the frames and emits them. The token is read from the
# environment and printed nowhere. Driven by signals, never by sleeps; gives up after [constant
# BUDGET] seconds. Prints `[check] PASS|FAIL <claim>: <observed>` lines and exits 1 on any failure.

const Client := preload("res://mineworld/world_client.gd")

## The whole run may take this long, in seconds, before it is a failure.
const BUDGET := 40.0

enum Step { WELCOME, FIRST_CLOCK, PAUSING, PAUSED_SUBMIT, RESUMING, KICKING, DONE }

var _world: Node
var _http: HTTPRequest
var _address := "127.0.0.1:7878"
var _seat := "visitor"
var _invite := ""
var _token := ""
var _elapsed := 0.0
var _failures := 0
var _done := false
var _step := Step.WELCOME
var _submitted := ""
var _closing_reason := ""
var _pending: Array = []
var _busy := false


func _initialize() -> void:
	_read_arguments()
	_token = OS.get_environment("MINEWORLD_ADMIN_TOKEN")
	if _token == "":
		_fail("an admin token in MINEWORLD_ADMIN_TOKEN", "none")
		_finish()
		return
	_world = Client.new()
	_http = HTTPRequest.new()
	root.add_child(_world)
	root.add_child(_http)
	_http.request_completed.connect(_on_answered)
	_world.welcomed.connect(_on_welcomed)
	_world.clock_changed.connect(_on_clock)
	_world.refused.connect(_on_refused)
	_world.closing.connect(func(reason, _detail): _closing_reason = reason)
	_world.disconnected.connect(_on_disconnected)
	_note("connecting to %s as seat '%s'" % [_address, _seat])
	_world.connect_to_world(_address, _seat, _invite, "admin-check")


func _process(delta: float) -> bool:
	_elapsed += delta
	if not _done and _elapsed > BUDGET:
		_fail("the run finished within %d s" % int(BUDGET), "stuck at step %d" % _step)
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


func _on_welcomed(_seat_name: String, _observer: String, _summary: Dictionary) -> void:
	_step = Step.FIRST_CLOCK


func _on_clock(_at: int, time_scale: int, paused: bool) -> void:
	match _step:
		Step.FIRST_CLOCK:
			_expect("a clock frame follows the welcome, running", paused, false)
			_expect("the time scale", time_scale, 1)
			_step = Step.PAUSING
			_post("/admin/clock", {"paused": true})
		Step.PAUSING:
			_expect("the host's pause is announced", paused, true)
			_expect("the module holds the paused state", _world.paused, true)
			_step = Step.PAUSED_SUBMIT
			_submitted = _world.submit("talk", null, {"utterance": "is anyone there?"})
		Step.RESUMING:
			_expect("the host's resume is announced", paused, false)
			_step = Step.KICKING
			_post("/admin/sessions/%s/kick" % _world.session, {})
		_:
			_fail("a clock frame only when the host changes the clock", "at step %d" % _step)


func _on_refused(code: String, token: String, _detail: String) -> void:
	if _step != Step.PAUSED_SUBMIT:
		_fail("no refusal outside the paused submit", code)
		return
	_expect("a submit while paused is refused paused", code, "paused")
	_expect("with its own token", token, _submitted)
	_step = Step.RESUMING
	_post("/admin/clock", {"paused": false})


func _on_disconnected(reason: String) -> void:
	if _done:
		return
	if _step == Step.KICKING:
		_expect("kicked: closing said why before the connection ended", _closing_reason, "kicked")
		_step = Step.DONE
	else:
		_fail("the connection lasted until the kick", reason)
	_finish()


## One admin request, as the host makes it: HTTP, the bearer token, a JSON body. One HTTPRequest
## carries one request at a time, and a frame can answer before the HTTP answer has finished
## arriving, so requests wait their turn here.
func _post(path: String, body: Dictionary) -> void:
	_pending.append([path, body])
	if not _busy:
		_send_next()


func _send_next() -> void:
	if _pending.is_empty():
		_busy = false
		return
	_busy = true
	var next: Array = _pending.pop_front()
	var path: String = next[0]
	var body: Dictionary = next[1]
	var headers := PackedStringArray([
		"Authorization: Bearer %s" % _token,
		"Content-Type: application/json",
	])
	var error := _http.request(
		"http://%s%s" % [_address, path], headers, HTTPClient.METHOD_POST, JSON.stringify(body)
	)
	if error != OK:
		_fail("POST %s was sent" % path, str(error))
		_finish()


func _on_answered(_result: int, code: int, _headers: PackedStringArray, _body: PackedByteArray) -> void:
	if code != 200:
		_fail("the admin surface answered 200", str(code))
		_finish()
		return
	_send_next()


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
