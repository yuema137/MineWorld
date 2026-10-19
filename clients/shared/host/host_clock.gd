## Host commands with the admin token: pause, resume, a day length, stop (`clients/shared/HOST.md` §3,
## `ARC-267`, `ARC-268`).
##
## The only shared client code that sends HTTP. The token is read once from `MINEWORLD_ADMIN_TOKEN` and
## kept in memory; it is never printed, logged or stored. The state it reports — paused, the scale — comes
## only from the server's `clock` frame and `welcome` summary, never from a click or a `200`
## (`INV-WC-1`). Without a token it creates no `HTTPRequest` and sends nothing.
class_name MineWorldHostClock
extends Node

signal changed
signal stopped

## How long a request may wait for an answer, and a confirmed change for its `clock` frame.
const WAIT_SECONDS := 5.0
## The day lengths the tab offers, as scales: 4 h, 2 h, 1 h.
const DAY_SCALES: Array[int] = [6, 12, 24]

var paused := false
var time_scale := 1
## The message key to show ("" for none): pending, unconfirmed, or an error's words.
var message := ""
## Replaces the `HTTPRequest` when set (the check's seam): called with (method, path, headers, body);
## the caller answers through `answered(status, body)`.
var transport := Callable()

var _token := ""
var _base_url := ""
var _http: HTTPRequest = null
var _in_flight := ""
var _waiting := 0.0
## What a confirmed request waits to see in a `clock` frame: {"paused": bool} or {"time_scale": n}.
var _expect := {}
var _expect_left := -1.0


## Reads the token from the environment, once. `base_url` is `http://<the address the client joined>`.
func setup(base_url: String) -> void:
	_base_url = base_url
	_token = OS.get_environment("MINEWORLD_ADMIN_TOKEN")


func has_token() -> bool:
	return _token != ""


func is_pending() -> bool:
	return _in_flight != "" or not _expect.is_empty()


## `4h`, `2h`, `1h` for the scales the tab offers, else `custom`.
func day_length() -> String:
	match time_scale:
		6:
			return "4h"
		12:
			return "2h"
		24:
			return "1h"
	return "custom"


func on_welcome(world: Dictionary) -> void:
	time_scale = int(world.get("time_scale", time_scale))
	paused = bool(world.get("paused", paused))
	changed.emit()


## A `clock` frame (`MineWorldClient.clock_changed`): the only source of the state shown.
func on_clock(_at: int, a_time_scale: int, a_paused: bool) -> void:
	time_scale = a_time_scale
	paused = a_paused
	if not _expect.is_empty() and _matches():
		_expect.clear()
		_expect_left = -1.0
		message = ""
	changed.emit()


func set_paused(want: bool) -> void:
	_send("/admin/clock", {"paused": want})


func set_scale(scale: int) -> void:
	if DAY_SCALES.has(scale):
		_send("/admin/clock", {"time_scale": scale})


## Asks the host to stop gracefully (`POST /admin/stop`); `stopped` is emitted on its `202`.
func stop_world() -> void:
	_send("/admin/stop", {})


func _send(path: String, body: Dictionary) -> void:
	if not has_token() or is_pending():
		return
	_in_flight = path
	_waiting = 0.0
	_expect = body.duplicate() if path == "/admin/clock" else {}
	message = "ui.world.pending" if path == "/admin/clock" else ""
	var headers := PackedStringArray(["Authorization: Bearer " + _token, "Content-Type: application/json"])
	var text := JSON.stringify(body)
	changed.emit()
	if transport.is_valid():
		transport.call("POST", path, headers, text)
		return
	if _http == null:
		_http = HTTPRequest.new()
		_http.timeout = WAIT_SECONDS
		add_child(_http)
		_http.request_completed.connect(_on_completed)
	if _http.request(_base_url + path, headers, HTTPClient.METHOD_POST, text) != OK:
		answered(0, "")


func _on_completed(result: int, status: int, _headers: PackedStringArray, body: PackedByteArray) -> void:
	if result == HTTPRequest.RESULT_TIMEOUT:
		_fail("timeout")
		return
	answered(status if result == HTTPRequest.RESULT_SUCCESS else 0, body.get_string_from_utf8())


## The answer to the request in flight: `status` 0 is "no connection".
func answered(status: int, _body: String) -> void:
	var path := _in_flight
	_in_flight = ""
	if status == 202 and path == "/admin/stop":
		message = ""
		changed.emit()
		stopped.emit()
		return
	if status == 200:
		if _matches():
			_expect.clear()
			message = ""
		else:
			_expect_left = WAIT_SECONDS
		changed.emit()
		return
	match status:
		401:
			_fail("unauthorized")
		400:
			_fail("malformed")
		409:
			_fail("fast_forward_solo_only")
		503:
			_fail("world_stopped")
		_:
			_fail("unreachable")


func _fail(code: String) -> void:
	_in_flight = ""
	_expect.clear()
	_expect_left = -1.0
	message = "ui.world.error." + code
	changed.emit()


func _matches() -> bool:
	if _expect.has("paused") and bool(_expect["paused"]) != paused:
		return false
	if _expect.has("time_scale") and int(_expect["time_scale"]) != time_scale:
		return false
	return true


func _process(delta: float) -> void:
	tick(delta)


## Advances the waits by `delta` wall seconds: a request with no answer times out; a confirmed change
## with no frame becomes unconfirmed, and the shown state stays the last frame's.
func tick(delta: float) -> void:
	if _in_flight != "" and not transport.is_valid() and _http == null:
		return
	if _in_flight != "" and transport.is_valid():
		_waiting += delta
		if _waiting >= WAIT_SECONDS:
			_fail("timeout")
	if _expect_left >= 0.0:
		_expect_left -= delta
		if _expect_left < 0.0:
			_expect.clear()
			message = "ui.world.unconfirmed"
			changed.emit()
