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

var address := ""
var seat := ""
## The module's client. Read by `intents.gd`, which is the only script that submits through it.
var client: MineWorldClient
## What the welcome said this world is; kept across a reconnect so the next welcome can be compared.
var instance := ""
## The revision the latest welcome named (`PROTOCOL.md` §5), or null for a world that is not saved.
var welcome_revision: Variant = null
var state := "closed"


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
	_set_state("seated", "")
	welcomed.emit(observer, world)


func _on_refused(code: String, token: String, detail: String) -> void:
	refused.emit(code, token, detail)


func _on_disconnected(reason: String) -> void:
	_set_state("closed", reason)


func _set_state(next: String, reason: String) -> void:
	state = next
	state_changed.emit(next, reason)
