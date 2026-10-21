extends Node

## The client's one connection to a world: which address, which seat, with which invite, and what
## state it is in.
##
## The only script that constructs a `MineWorldClient` or calls `connect_to_world` (`ARC-47` R5), so
## how a client joins (protocol revision 2: invite and nickname, S11-A) is stated here and in the
## launcher, nowhere else. It forwards the module's signals and adds nothing to them: it decides no
## rule, holds no world state, and keeps only what a status line and a reconnect need. The invite is
## held to re-join and never printed.
##
## Seats (PR 13e, `ARC-344`): a dropped socket resumes the same seat with the module's `resume`
## (`reconnect`), within the server's hold; a code that would be answered the same way again ends the
## connection for good (`ended`), and a takeover is sent only when the player asked for one.

signal welcomed(observer: String, world: Dictionary)
signal observed(observation: MineWorldObservation)
signal resolved(token: String, action_id: String, result: Dictionary)
signal refused(code: String, token: String, detail: String)
signal submitted(token: String, request: Dictionary)
## The connection's state changed: `connecting`, `seated`, `reconnecting` or `closed`.
signal state_changed(state: String, reason: String)
## The connection ended for good with this refusal or closing code (one of `FINAL`): asking again
## would be answered the same, so the client shows the code and lets the player choose again.
signal ended(code: String)
## A join is about to be sent by this script (not the module's own resumes): the seat and whether it
## takes the seat over. For a status line and the harness; never the invite.
signal joining(seat: String, take_over: bool)

## Back-off between attempts: from the first, doubling, capped (step-13 §4.6 point 1).
const RETRY_FIRST_S := 0.5
const RETRY_MAX_S := 8.0
## Refusals and closing reasons that end the attempt for good: asking again would be refused again
## (§4.6 point 4). `world_stopped` is not among them: a world that stopped may be started again.
## `not_granted` (S27-a): a credential that does not admit this seat will not admit it on retry.
## `seat_occupied`, `taken_over`, `superseded`, `kicked` (S11-B, `ARC-344`): somebody else holds the
## seat, or the operator removed this player; only the player may choose to take it back.
const FINAL := ["unknown_seat", "seat_not_in_world", "already_joined", "unauthorized",
	"protocol_mismatch", "invalid_nickname", "left", "not_granted", "seat_occupied", "taken_over",
	"superseded", "kicked"]

var address := ""
var seat := ""
var nickname := ""
## The module's client. Read by `intents.gd`, which is the only script that submits through it.
var client: MineWorldClient
## What the welcome said this world is; kept across a reconnect so the next welcome can be compared.
var instance := ""
## The revision the latest welcome named (`PROTOCOL.md` §5), or null for a world that is not saved.
var welcome_revision: Variant = null
var state := "closed"

var _invite := ""
var _take_over := false
var _retry_s := RETRY_FIRST_S
var _retry_at := -1.0
var _final := false
var _final_code := ""


func _ready() -> void:
	client = MineWorldClient.new()
	client.name = "MineWorldClient"
	# The module's resume within the server's hold (ADOPTION.md §6.1, ARC-344).
	client.reconnect = true
	add_child(client)
	client.welcomed.connect(_on_welcomed)
	client.observed.connect(func(o: MineWorldObservation) -> void: observed.emit(o))
	client.resolved.connect(func(t: String, a: String, r: Dictionary) -> void: resolved.emit(t, a, r))
	client.refused.connect(_on_refused)
	client.closing.connect(_on_closing)
	client.disconnected.connect(_on_disconnected)
	client.reconnecting.connect(func(_attempt: int) -> void: _set_state("reconnecting", "resuming"))
	client.submitted_request.connect(func(t: String, r: Dictionary) -> void: submitted.emit(t, r))


## Opens the connection and asks for the seat, presenting the server's invite. `take_over` is the
## player's explicit choice after `seat_occupied` (PROTOCOL.md §4.2 rule 4); it is sent once.
func open(at_address: String, as_seat: String, invite: String, as_nickname: String,
		take_over := false) -> void:
	address = at_address
	seat = as_seat
	_invite = invite
	nickname = as_nickname
	_take_over = take_over
	_final = false
	_final_code = ""
	_retry_s = RETRY_FIRST_S
	_retry_at = -1.0
	_set_state("connecting", "")
	_join()


func is_seated() -> bool:
	return client != null and client.is_seated()


## Whether control of the Person changed hands at the latest welcome (`hosted`, `held`, `connection`,
## `none`), as the server said.
func took_over() -> String:
	return client.took_over if client != null else ""


func _join() -> void:
	joining.emit(seat, _take_over)
	client.connect_to_world(address, seat, _invite, nickname, _take_over)
	# Never again by itself: a retry or a resume after this one asks for the seat, not for a takeover.
	_take_over = false


func _on_welcomed(_seat: String, observer: String, world: Dictionary) -> void:
	instance = String(world.get("instance", ""))
	welcome_revision = world.get("revision")
	_retry_s = RETRY_FIRST_S
	_set_state("seated", "")
	welcomed.emit(observer, world)


func _on_refused(code: String, token: String, detail: String) -> void:
	if state != "seated" and FINAL.has(code):
		_final = true
		_final_code = code
	refused.emit(code, token, detail)


func _on_closing(reason: String, _detail: String) -> void:
	if FINAL.has(reason):
		_final = true
		_final_code = reason


## The connection ended. The module has already tried its resume when it had one; the rest of the
## policy is the client's (`ADOPTION.md` §6.1): retry the same seat with capped back-off, unless the
## world answered with a code that ends it for good.
func _on_disconnected(reason: String) -> void:
	if _final or address == "":
		_set_state("closed", reason)
		if _final:
			ended.emit(_final_code)
		return
	_retry_at = Time.get_ticks_msec() / 1000.0 + _retry_s
	_retry_s = minf(_retry_s * 2.0, RETRY_MAX_S)
	_set_state("reconnecting", reason)


func _process(_delta: float) -> void:
	if _retry_at >= 0.0 and Time.get_ticks_msec() / 1000.0 >= _retry_at:
		_retry_at = -1.0
		_join()


func _set_state(next: String, reason: String) -> void:
	state = next
	state_changed.emit(next, reason)
