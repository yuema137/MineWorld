class_name MineWorldStatus
extends Node

## `GET /status` of a MineWorld server: what the world is — its composition, never its state
## (`server/PROTOCOL.md` §5.7). Public and credential-free; a client reads it before joining, to list
## the seats a world offers (PR 13e's join screen).
##
## [codeblock]
## var status := MineWorldStatus.new()
## add_child(status)
## status.answered.connect(func(ok, world): ...)
## status.fetch("127.0.0.1:7878")
## [/codeblock]
##
## It only reads. It names no player and decides nothing; which seat is free is the server's answer to
## a join (`seat_occupied`), never this summary's.

## The answer: `ok` false when the server could not be reached or did not answer with a summary;
## `world` is the summary as the server wrote it (`seats`, `instance`, `systems`, …), or `{}`.
signal answered(ok: bool, world: Dictionary)

## Wall seconds before an unanswered request counts as unreachable.
const TIMEOUT_S := 5.0

var _http: HTTPRequest


func _ready() -> void:
	_http = HTTPRequest.new()
	_http.timeout = TIMEOUT_S
	add_child(_http)
	_http.request_completed.connect(_on_completed)


## Asks `address` (`host:port`, `ws://host:port` or `ws://host:port/ws`) for its summary; a request
## still in flight is abandoned. The answer arrives as [signal answered].
func fetch(address: String) -> void:
	_http.cancel_request()
	if _http.request(url(address)) != OK:
		answered.emit(false, {})


## The `GET /status` URL of an address in any of the forms [method MineWorldClient.connect_to_world]
## accepts.
static func url(address: String) -> String:
	var host := address.trim_prefix("ws://").trim_prefix("wss://").trim_prefix("http://")
	host = host.trim_suffix("/").trim_suffix("/ws")
	return "http://%s/status" % host


func _on_completed(result: int, code: int, _headers: PackedStringArray, body: PackedByteArray) -> void:
	if result != HTTPRequest.RESULT_SUCCESS or code != 200:
		answered.emit(false, {})
		return
	var parsed: Variant = JSON.parse_string(body.get_string_from_utf8())
	if typeof(parsed) != TYPE_DICTIONARY:
		answered.emit(false, {})
		return
	answered.emit(true, parsed)
