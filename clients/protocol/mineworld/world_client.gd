class_name MineWorldClient
extends Node

## A connection to a MineWorld server: the three frames a client may send (join, submit, leave), and
## the five things it can be told (welcome, observation, result, refused, closing) — protocol
## revision 2.
##
## Drop this node into a scene, connect the signals, call [method connect_to_world], and render what
## arrives. `server/PROTOCOL.md` is the specification this implements and governs where the two
## disagree; `clients/protocol/ADOPTION.md` is how a client takes this module.
##
## [codeblock]
## var world := MineWorldClient.new()
## add_child(world)
## world.observed.connect(_on_observed)
## world.connect_to_world("127.0.0.1:7878", "visitor", invite, "Yue")
## ...
## world.submit("talk", alice_id, { "utterance": "hello" })
## [/codeblock]
##
## [b]What this module knows, and what it must never learn.[/b]
##
## [codeblock]
## it knows    the frames, the handshake, the seat, the sequence number, the persisted revision a
##             frame names, the correlation token, that an identity is a string, that every other
##             number is an integer, that `action_type` appears twice and must agree, that the
##             actor is this connection's observer, and how to send a complete affordance as it
##             was offered
## you know    which action you are submitting and what its payload means, how the world looks,
##             and what a key press or a click means
## NEITHER     whether an action is allowed. That answer is in the observation, computed by the
##             server. A distance check in a renderer is a defect.
## [/codeblock]
##
## There is no method here that sets a value, names another observer, widens a scope or asserts a
## fact, because there is no such frame: the authority model is the absence of the vocabulary
## (`docs/NETWORKING.md` §2). A client may act, and may never assert.

## Which revision of the protocol this module speaks. A server that answers with another number is
## refused rather than guessed at.
const PROTOCOL := 2

## The connection got a seat: this is the observer it sees the world as, and what the world is.
##
## `observer` is an identity **string**. `world` is the summary the server sent, including
## `instance` — which running world this is.
signal welcomed(seat: String, observer: String, world: Dictionary)

## A fresh view of the world, for this observer only.
signal observed(observation: MineWorldObservation)

## The world's answer to one submitted request: the token this client chose, the identity the
## **server** allocated, and an `ActionResult`.
signal resolved(token: String, action_id: String, result: Dictionary)

## The frame was not something the protocol accepts, and nothing happened.
##
## `code` is one of the codes in `PROTOCOL.md` §5 — a client branches on it and never on `detail`.
signal refused(code: String, token: String, detail: String)

## The server said it is closing this connection, and why — emitted before [signal disconnected].
##
## `reason` is one of `PROTOCOL.md` §5.6's: `left`, `unauthorized`, `protocol_mismatch`,
## `world_stopped`, `taken_over` (another connection took the seat), `superseded` (this player's own
## newer connection resumed it), and from later S11 pull requests `kicked`, `server_stopping`. A client
## branches on `reason` (a wrong invite is `unauthorized`) and never on `detail`.
signal closing(reason: String, detail: String)

## The connection ended, or could not be made. `reason` is for a developer and a status line.
signal disconnected(reason: String)

## The socket dropped without a `closing`, [member reconnect] is on, and this module is about to try
## again: `attempt` counts from 1. A successful attempt ends in the usual [signal welcomed], with
## [member took_over] `"held"` when the seat was still held for this player (`PROTOCOL.md` §4.2).
signal reconnecting(attempt: int)

## What this client just sent, after it was sent: the token and the whole request frame.
##
## For a log, a transcript or a test fixture. Emitted by [method submit] so that what is recorded is
## what went out, rather than a second construction of it that could drift.
signal submitted_request(token: String, request: Dictionary)

## What a connection is doing.
enum State {
	IDLE,      ## nothing yet
	OPENING,   ## the socket is connecting
	JOINING,   ## connected; a seat has been asked for and not yet granted
	SEATED,    ## observations are arriving and requests may be submitted
	CLOSED,    ## finished, for any reason
	RECONNECTING,  ## the socket dropped; waiting to open it again ([member reconnect])
}

## Opt-in: when the socket drops without the server saying `closing`, rejoin the same seat with the
## stored [member resume] — after 1 s, 2 s, 4 s … while within [member hold_seconds] of the drop, then
## once without it — emitting [signal reconnecting] before each attempt. Off by default, so a client
## that never sets it behaves exactly as before. A `closing` from the server (left, taken over,
## superseded, kicked …) never triggers it, and neither does [method disconnect_from_world].
var reconnect := false

## Where this connection is.
var state: State = State.IDLE

## The seat that was asked for.
var seat: String = ""

## The observer the **server** resolved that seat to. Empty until the welcome arrives.
##
## A client never chooses this and never sends it: there is no frame that names an observer, which is
## what makes `INV-13` structural rather than checked.
var observer: String = ""

## What the server said the world is, as of the welcome: `instance`, `at`, `entities`, `systems`,
## `seats`, `clients`, and the three counters `PROTOCOL.md` §5 documents.
var world: Dictionary = {}

## The newest observation received, or `null`.
##
## The newest, deliberately. A client that falls behind loses observations rather than delaying the
## world, and a client should render the latest view it has rather than accumulating a backlog
## (`PROTOCOL.md` §8).
var latest: MineWorldObservation = null

## The sequence number of [member latest]. Frames are numbered from 1 on each connection.
var sequence: int = 0

## The persisted revision of the world that [member latest] was computed from: an `int`, or `null`
## for a world that is not persisted (`PROTOCOL.md` §5).
##
## Set from the welcome's `world.revision`, then from every observation frame that is not stale, so it
## always belongs with [member latest]. Two clients told the same instance and the same revision are
## looking at one committed state of one world — `AC-15`'s fourth line of evidence.
var revision: Variant = null

## How many observations arrived out of order and were dropped. Normally zero.
var stale_observations: int = 0

## The player's nickname as the server accepted it (trimmed), from the welcome. Shown to this
## connection only: no other player is ever told it (`PROTOCOL.md` §4.1).
var nickname: String = ""

## Which connection this is, from the welcome: an identity string, for the operator's admin surface.
## Not a credential.
var session: String = ""

## Whether control of the Person changed hands when this connection joined: `"none"` (the seat was
## free), `"hosted"` (an in-server controller was driving it), `"held"` (this player's own dropped
## connection's seat, resumed) or `"connection"` (taken from another connection with `take_over`).
var took_over: String = ""

## How long the server holds this seat after the socket drops, in wall seconds; `0` holds none.
var hold_seconds: int = 0

## The secret that re-takes this seat after a dropped socket — a fresh one on every welcome — or
## `null` from a server older than S11-B. Used by [member reconnect]; never printed, logged or emitted.
var resume: Variant = null

## Why the server last closed this connection (`closing.reason`), or `""`.
var close_reason: String = ""

var _socket := WebSocketPeer.new()
var _invite := ""
var _nickname_asked := ""
var _url := ""
var _tokens := 0
var _take_over := false
## The resume the next join presents, or `null`: set only while reconnecting.
var _rejoin_with: Variant = null
var _attempt := 0
var _dropped_msec := 0
var _retry_msec := 0
var _last_try := false


## Opens a connection and asks for a seat.
##
## `address` may be `"host:port"`, `"ws://host:port"` or a whole `"ws://host:port/ws"` — a client
## should not have to remember the route. `seat_name` is a seat the world offers; `GET /status` lists
## them, and one the roster does not contain is refused `unknown_seat`.
##
## `invite` is the server's invite: required on every server, loopback included. A server started
## without one prints it on its `[mineworld] invite <token> — join with: …` line. A wrong invite is
## answered `refused unauthorized`, then [signal closing] with `"unauthorized"`. This module never
## prints, logs or emits it. `nickname` names the *player* (1 to 32 characters, trimmed by the
## server); nobody else is shown it.
##
## `take_over` (optional, default `false`) takes the seat from another connection that holds it, or
## from a dropped player's hold; that connection is told `closing` `"taken_over"`. A seat an in-server
## controller drives needs no flag (`PROTOCOL.md` §4.2). Every four-argument call is unchanged.
func connect_to_world(
	address: String, seat_name: String, invite: String, nickname_asked: String,
	take_over := false
) -> void:
	if state != State.IDLE and state != State.CLOSED:
		push_warning("[mineworld] already connected; ignoring connect_to_world")
		return
	seat = seat_name
	_invite = invite
	_nickname_asked = nickname_asked
	_take_over = take_over
	_rejoin_with = null
	_attempt = 0
	_last_try = false
	nickname = ""
	session = ""
	took_over = ""
	hold_seconds = 0
	resume = null
	close_reason = ""
	observer = ""
	world = {}
	latest = null
	sequence = 0
	revision = null
	_url = _websocket_url(address)
	var opened := _socket.connect_to_url(_url)
	if opened != OK:
		_close("cannot open %s (error %d)" % [_url, opened])
		return
	state = State.OPENING


## Submits one request, and returns the correlation token the answer will carry.
##
## [codeblock]
## var token := world.submit("talk", alice_id, { "utterance": "hello" })
## [/codeblock]
##
## What this fills in, so that a client cannot get it wrong:
## [codeblock]
## actor            this connection's observer. A request naming anybody else is refused.
## action_type      written in BOTH of the places the contract requires it, which a hand-built
##                  frame can otherwise make disagree (FINDINGS.md F8.1)
## no action_id     and no issued_at: the server allocates identity and the instant (INV-6). A
##                  client that invented an ActionId would collide with the other client on its
##                  first action (FINDINGS.md F4)
## [/codeblock]
##
## `payload` is the owning system's own shape — normally a Dictionary, but any JSON value the system
## accepts — and its numbers are yours to make integers — use
## [method MineWorldSpace.millimetres] and friends. `actor_location` is a *report* and may be `null`:
## a 3D client sends the position it walked to, a 2D client that models no position sends nothing,
## and the server evaluates its own authoritative state either way.
##
## Returns `""` without sending anything when the connection has no seat yet.
func submit(
	action_type: String,
	target: Variant = null,
	payload: Variant = {},
	actor_location: Variant = null,
) -> String:
	if state != State.SEATED:
		push_warning("[mineworld] not seated; a request needs a seat first")
		return ""
	_tokens += 1
	var token := "c%d" % _tokens
	var request := {
		"actor": observer,
		"action_type": action_type,
		"target": null if target == null else String(target),
		"payload": { "action_type": action_type, "payload": payload },
		"actor_location": actor_location,
	}
	_send({ "t": "submit", "token": token, "request": request })
	submitted_request.emit(token, request)
	return token


## Submits a complete affordance exactly as the world offered it, and returns the token.
##
## [codeblock]
## for offered in world.latest.complete_affordances():
##     ...show it; when chosen:
##     world.submit_affordance(offered)
## [/codeblock]
##
## The request is the affordance's own `action_type`, `target` and `payload`, labelled and sent by
## [method submit], so a client submits it without knowing what the action is
## (`docs/DECISIONS.md` `ARC-34`). It is sent **whether or not the affordance is available**: an
## offer is not a permission, the world may have changed since the observation, and the server
## answers either way. Deciding not to ask is the client implementing the rule.
##
## "Unchanged" means the JSON the server sent, which includes its integers. Godot parses every JSON
## number as a double, so an offered `"count": 1` arrives as `1.0`, would go back out as `1.0`, and
## would be refused by a contract that declares an integer (`ADOPTION.md` §3.2). Every whole number
## in the payload is therefore sent as an integer again — which is what it was on the wire; a field
## the system declares as a float accepts an integer just as well.
##
## Returns `""` and sends nothing when the affordance is not complete — it carries no `payload`, so
## there is nothing to send unchanged — or names no action type, or when the connection has no seat.
func submit_affordance(affordance: Dictionary, actor_location: Variant = null) -> String:
	var action_type: Variant = affordance.get("action_type")
	if typeof(action_type) != TYPE_STRING or String(action_type).is_empty():
		push_warning("[mineworld] submit_affordance: not an affordance (no action_type)")
		return ""
	if not MineWorldObservation.is_complete(affordance):
		push_warning("[mineworld] submit_affordance: '%s' is not complete; compose and submit()" % [
			action_type,
		])
		return ""
	return submit(
		action_type, affordance.get("target"), _as_sent(affordance["payload"]), actor_location
	)


## A parsed JSON value with every whole-number double turned back into the integer it was on the
## wire. Identities are strings and pass through untouched (`PROTOCOL.md` §7).
static func _as_sent(value: Variant) -> Variant:
	match typeof(value):
		TYPE_FLOAT:
			var number: float = value
			if is_finite(number) and number == floorf(number) and absf(number) <= 9007199254740992.0:
				return int(number)
			return number
		TYPE_DICTIONARY:
			var copied := {}
			for key in value:
				copied[key] = _as_sent(value[key])
			return copied
		TYPE_ARRAY:
			var listed := []
			for entry in value:
				listed.append(_as_sent(entry))
			return listed
		_:
			return value


## Closes the connection. The world keeps running: a client is a spectator with a request channel,
## never a participant the world waits for.
func disconnect_from_world(reason: String = "closed by the client") -> void:
	if state == State.CLOSED:
		return
	_socket.close()
	# One more poll, so the close frame is actually written before this node stops polling: a client
	# that vanishes without one is handled by the server anyway — it reaps a subscription whose
	# channel has closed — but saying goodbye is cheap and makes the server's client count drop at
	# once rather than at its next sweep.
	_socket.poll()
	_close(reason)


## Gives the seat up at once and ends the connection: sends `leave`, and the server answers
## [signal closing] with `"left"` and closes. Unlike [method disconnect_from_world], which only drops
## the socket, this tells the server the player is gone rather than interrupted — from S11-B a dropped
## socket's seat is held for a reconnect, and a left one is not.
func leave_world() -> void:
	if state == State.IDLE or state == State.CLOSED:
		return
	_send({ "t": "leave" })


## Whether this connection is seated and can act.
func is_seated() -> bool:
	return state == State.SEATED


## Which running world this is, as the welcome named it, or `""`.
##
## The same value `GET /status` reports. Two connections that are told the same instance are
## connected to one world — which is what `AC-15`'s evidence is made of (`MVP.md` §9.1).
func world_instance() -> String:
	return String(world.get("instance", ""))


func _process(_delta: float) -> void:
	if state == State.IDLE or state == State.CLOSED:
		return
	if state == State.RECONNECTING:
		if Time.get_ticks_msec() >= _retry_msec:
			_reopen()
		return
	_socket.poll()
	match _socket.get_ready_state():
		WebSocketPeer.STATE_OPEN:
			if state == State.OPENING:
				state = State.JOINING
				_send_join(_rejoin_with)
			_drain()
		WebSocketPeer.STATE_CLOSING:
			_drain()
		WebSocketPeer.STATE_CLOSED:
			# What arrived with the close is still read first: a server that refuses a join sends
			# `refused` and `closing` and closes at once, and both frames can land in the same poll.
			_drain()
			if state == State.CLOSED:
				return
			var code := _socket.get_close_code()
			var note := _socket.get_close_reason()
			if close_reason != "":
				_close("the server closed the connection: %s" % close_reason)
			elif reconnect and resume != null and (state == State.SEATED or _attempt > 0):
				# Dropped without a word from the server: a blink, not a goodbye (`PROTOCOL.md` §4.2).
				_schedule_reconnect()
			else:
				_close("the connection closed (%d %s)" % [code, note])
		_:
			pass


## `PROTOCOL.md` §2's join. `protocol` is the int constant, so JSON writes it as an integer (§7).
func _send_join(resuming: Variant) -> void:
	_send({
		"t": "join", "protocol": PROTOCOL, "invite": _invite,
		"nickname": _nickname_asked, "seat": seat, "resume": resuming, "take_over": _take_over,
	})


## The next reconnect attempt: 1 s after the drop, then 2 s, 4 s … while within the hold, with the
## resume; then once without it; then the connection is over.
func _schedule_reconnect() -> void:
	var now := Time.get_ticks_msec()
	if _attempt == 0:
		_dropped_msec = now
	if _last_try:
		_close("the connection dropped and could not be resumed")
		return
	_attempt += 1
	var wait_msec := 1000 * (1 << mini(_attempt - 1, 10))
	if now + wait_msec - _dropped_msec <= hold_seconds * 1000:
		_rejoin_with = resume
	else:
		_rejoin_with = null
		_last_try = true
	_retry_msec = now + wait_msec
	state = State.RECONNECTING
	reconnecting.emit(_attempt)


func _reopen() -> void:
	_socket = WebSocketPeer.new()
	close_reason = ""
	# A new connection numbers its frames from 1 again (`PROTOCOL.md` §5).
	sequence = 0
	var opened := _socket.connect_to_url(_url)
	if opened != OK:
		_schedule_reconnect()
		return
	state = State.OPENING


## Every frame waiting on the socket, in order, until there are none or the connection was closed.
func _drain() -> void:
	while _socket.get_available_packet_count() > 0:
		_receive(_socket.get_packet().get_string_from_utf8())
		if state == State.CLOSED:
			return


## One text frame from the server.
func _receive(text: String) -> void:
	var parsed: Variant = JSON.parse_string(text)
	if typeof(parsed) != TYPE_DICTIONARY:
		push_error("[mineworld] a server frame is a JSON object, and this was: %s" % text)
		return
	var frame: Dictionary = parsed
	match String(frame.get("t", "")):
		"welcome":
			_welcome(frame)
		"observation":
			_observation(frame)
		"result":
			resolved.emit(
				String(frame.get("token", "")),
				String(frame.get("action_id", "")),
				frame.get("result", {}),
			)
		"refused":
			var code := String(frame.get("code", ""))
			refused.emit(code, String(frame.get("token", "")), String(frame.get("detail", "")))
			if state == State.JOINING and _attempt > 0:
				_refused_while_reconnecting(code)
		"closing":
			# The connection is over; the reason is the only thing a client branches on. The client
			# closes the socket itself, which is what the server waits for (`PROTOCOL.md` §5.6).
			close_reason = String(frame.get("reason", ""))
			closing.emit(close_reason, String(frame.get("detail", "")))
			disconnect_from_world("the server closed the connection: %s" % close_reason)
		var unknown:
			# A frame from a future revision. Reported and ignored rather than guessed at: a client
			# that invented a meaning for it would be a client acting on something it cannot read.
			push_warning("[mineworld] ignoring a frame of an unknown kind: %s" % unknown)


## A reconnecting join was refused. The hold had ended (`invalid_resume`): the one plain join is made
## at once, on the same connection, which stays in the handshake. Anything else — somebody else has
## the seat — ends the attempt.
func _refused_while_reconnecting(code: String) -> void:
	if code == "invalid_resume" and _rejoin_with != null:
		_rejoin_with = null
		_last_try = true
		_send_join(null)
	else:
		disconnect_from_world("the seat could not be taken back: %s" % code)


func _welcome(frame: Dictionary) -> void:
	var spoken := int(frame.get("protocol", 0))
	if spoken != PROTOCOL:
		# Refused rather than attempted. A client that guesses at an unknown revision is a client
		# rendering a world it has misread (`PROTOCOL.md` §9).
		disconnect_from_world(
			"the server speaks protocol %d and this client speaks %d" % [spoken, PROTOCOL]
		)
		return
	seat = String(frame.get("seat", seat))
	# Kept as a string, always. Godot has one number type and it is a double, so
	# `9007199254740995` and `9007199254740997` parse to the same value and two entities become one
	# — measured, not theoretical (`spike/FINDINGS.md` F1, F2, `PROTOCOL.md` §7).
	observer = String(frame.get("observer", ""))
	nickname = String(frame.get("nickname", ""))
	session = String(frame.get("session", ""))
	took_over = String(frame.get("took_over", "none"))
	hold_seconds = int(frame.get("hold_seconds", 0))
	resume = frame.get("resume")
	world = frame.get("world", {})
	revision = _revision_of(world)
	_rejoin_with = null
	_attempt = 0
	_last_try = false
	state = State.SEATED
	welcomed.emit(seat, observer, world)


func _observation(frame: Dictionary) -> void:
	var seq := int(frame.get("seq", 0))
	if seq < sequence:
		# A lower sequence number is a stale frame (`PROTOCOL.md` §5). A gap is not an error: frames
		# dropped for a client that was not reading are dropped before they are numbered.
		stale_observations += 1
		return
	sequence = seq
	var observation: Variant = frame.get("observation", {})
	if typeof(observation) != TYPE_DICTIONARY:
		push_error("[mineworld] an observation frame carried no observation")
		return
	latest = MineWorldObservation.new(observation)
	revision = _revision_of(frame)
	observed.emit(latest)


func _send(frame: Dictionary) -> void:
	var sent := _socket.send_text(JSON.stringify(frame))
	if sent != OK:
		_close("cannot send on the connection (error %d)" % sent)


func _close(reason: String) -> void:
	state = State.CLOSED
	disconnected.emit(reason)


## A frame's `revision`, as an `int`, or `null` when the world is not persisted.
##
## A revision is a counter, not an identity, so reading it through JSON's double is exact far beyond
## any revision a world reaches (`PROTOCOL.md` §7 keeps only identities as strings).
static func _revision_of(holder: Dictionary) -> Variant:
	var named: Variant = holder.get("revision")
	return null if named == null else int(named)


## `host:port`, `ws://host:port` or a full `ws://host:port/ws`, as the one route the server upgrades
## on (`PROTOCOL.md` §1).
static func _websocket_url(address: String) -> String:
	var url := address
	if not url.begins_with("ws://") and not url.begins_with("wss://"):
		url = "ws://" + url
	if url.ends_with("/ws"):
		return url
	return url.trim_suffix("/") + "/ws"
