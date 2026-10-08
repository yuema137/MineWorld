extends Node

## The client's one connection to a world: which address, which seat, and what state it is in.
##
## The only script that constructs a `MineWorldClient` or calls `connect_to_world` (`ARC-47` R5), so
## a change to how a client joins — S11-A's invite and nickname — is a change here and in the
## launcher, nowhere else. It forwards the module's signals and adds nothing to them: it decides no
## rule, holds no world state, and keeps only what a status line and a reconnect need.

signal welcomed(observer: String, world: Dictionary)
signal observed(observation: MineWorldObservation)
signal resolved(token: String, action_id: String, result: Dictionary)
signal refused(code: String, token: String, detail: String)
signal submitted(token: String, request: Dictionary)
## The connection's state changed: `connecting`, `seated`, `reconnecting` or `closed`.
signal state_changed(state: String, reason: String)

## Back-off between attempts: from the first, doubling, capped (step-13 §4.6 point 1).
const RETRY_FIRST_S := 0.5
const RETRY_MAX_S := 8.0
## Refusals that end the attempt for good: asking again would be refused again (§4.6 point 4).
## `world_stopped` is not among them: a world that stopped may be started again.
const FINAL_REFUSALS := ["unknown_seat", "seat_not_in_world", "already_joined"]

var address := ""
var seat := ""
## The module's client. Read by `intents.gd`, which is the only script that submits through it.
var client: MineWorldClient
## What the welcome said this world is; kept across a reconnect so the next welcome can be compared.
var instance := ""
## The revision the latest welcome named (`PROTOCOL.md` §5), or null for a world that is not saved.
var welcome_revision: Variant = null
var state := "closed"

var _retry_s := RETRY_FIRST_S
var _retry_at := -1.0
var _final := false


func _ready() -> void:
	client = MineWorldClient.new()
	client.name = "MineWorldClient"
	add_child(client)
	client.welcomed.connect(_on_welcomed)
	client.observed.connect(func(o: MineWorldObservation) -> void: observed.emit(o))
	client.resolved.connect(func(t: String, a: String, r: Dictionary) -> void: resolved.emit(t, a, r))
	client.refused.connect(_on_refused)
	client.disconnected.connect(_on_disconnected)
	client.submitted_request.connect(func(t: String, r: Dictionary) -> void: submitted.emit(t, r))


## Opens the connection and asks for the seat.
func open(at_address: String, as_seat: String) -> void:
	address = at_address
	seat = as_seat
	_set_state("connecting", "")
	client.connect_to_world(address, seat)


func is_seated() -> bool:
	return client != null and client.is_seated()


func _on_welcomed(_seat: String, observer: String, world: Dictionary) -> void:
	instance = String(world.get("instance", ""))
	welcome_revision = world.get("revision")
	_retry_s = RETRY_FIRST_S
	_set_state("seated", "")
	welcomed.emit(observer, world)


func _on_refused(code: String, token: String, detail: String) -> void:
	if state != "seated" and FINAL_REFUSALS.has(code):
		_final = true
	refused.emit(code, token, detail)


## The connection ended. The policy is the client's (`ADOPTION.md` §6.1): retry the same seat with
## capped back-off, unless the world refused this seat for good.
func _on_disconnected(reason: String) -> void:
	if _final or address == "":
		_set_state("closed", reason)
		return
	_retry_at = Time.get_ticks_msec() / 1000.0 + _retry_s
	_retry_s = minf(_retry_s * 2.0, RETRY_MAX_S)
	_set_state("reconnecting", reason)


func _process(_delta: float) -> void:
	if _retry_at >= 0.0 and Time.get_ticks_msec() / 1000.0 >= _retry_at:
		_retry_at = -1.0
		client.connect_to_world(address, seat)


func _set_state(next: String, reason: String) -> void:
	state = next
	state_changed.emit(next, reason)
