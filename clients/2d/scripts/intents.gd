extends Node

## THE ONLY script that names an action type or submits a request (`ARC-47` R1, R2).
##
## Each composer says how to *ask* for an action — what its payload is made of — and nothing else.
## Whether the action exists, is available, is near enough or would succeed is the server's answer,
## read from the result; nothing here reads an affordance's verdict (R3), and nothing here refuses to
## send what the player chose. A request whose result never arrives is not resent: the server
## allocates identity, so a retry could do a thing twice (`INV-6`).
##
## A complete affordance is sent exactly as it was offered, through the shared module's
## `submit_affordance` (`ARC-34`); this file composes only the incomplete ones it knows (step-13
## §15.4 D-b-2). Every request carries `actor_location: null`: this client models no authoritative
## position (D-b-11, `PROTOCOL.md` §6.1).

## The action types this client composes. `scripts/check_client_rules.py` reads this list: no other
## script may hold one of these as a string literal.
const COMPOSED := ["move", "talk", "invite", "accept-invitation", "decline-invitation", "join-group-activity", "leave-group-activity"]

## What the player supplies for a composed type that needs more than a choice: a point on the floor
## (the walker asks), or a line of text with the wording keys of its prompt and suggestions.
const INPUTS := {
	"move": {"point": true},
	"talk": {"prompt": "ui.talk.prompt"},
	"invite": {"prompt": "ui.invite.prompt", "suggest": "suggest.invite-kind"},
}

## The link whose `MineWorldClient` the requests go through.
var link: Node

## token → `{action_type, target, about}`: what was asked for, until the world answers it. A move is
## the walker's and is not kept here.
var pending: Dictionary = {}


## Asks to stand at `local` (integer millimetres, `+y` north) in `place`, facing `yaw_mdeg`
## (`PROTOCOL.md` §6.2). Returns the request's token, or "" when there is no seat to ask from.
func move(place: String, local: Dictionary, yaw_mdeg: Variant) -> String:
	var to := MineWorldSpace.location(place, local, yaw_mdeg)
	return link.client.submit("move", null, {"to": to})


## Whether this client knows how to ask for an incomplete affordance of `action_type`.
func composes(action_type: String) -> bool:
	return COMPOSED.has(action_type)


## What the player supplies for `action_type`: `{}` (nothing: a choice is enough), `{point: true}`, or
## `{prompt, suggest?}` for a typed line.
func input_for(action_type: String) -> Dictionary:
	return INPUTS.get(action_type, {})


## The offered target-less `move`, which "walk to <person>" is (D-b-3), or `{}` when the world
## offers no walking.
func offered_walk(observation: MineWorldObservation) -> Dictionary:
	return observation.affordance("move", "")


## Sends a complete affordance exactly as it was offered, whatever the world said about it.
func submit_offered(affordance: Dictionary) -> String:
	var token: String = link.client.submit_affordance(affordance)
	_remember(token, affordance)
	return token


## Composes and sends an incomplete affordance with the player's `input` (the typed line, or null),
## sent exactly as given. Returns the token, or "" for a type this file does not compose here.
func compose(affordance: Dictionary, input: Variant) -> String:
	var action_type := String(affordance.get("action_type", ""))
	var payload: Dictionary
	match action_type:
		"talk":
			payload = {"utterance": String(input)}
		"invite":
			payload = {"kind": String(input)}
		"accept-invitation", "decline-invitation", "join-group-activity", "leave-group-activity":
			payload = {}
		_:
			return ""
	var token: String = link.client.submit(action_type, affordance.get("target"), payload)
	_remember(token, affordance)
	return token


## What `token` asked for, forgotten as it is read; `{}` for a move or an unknown token.
func take(token: String) -> Dictionary:
	var asked: Dictionary = pending.get(token, {})
	pending.erase(token)
	return asked


## The connection dropped: every unanswered request, forgotten and never resent (§4.6 point 5).
func drop_pending() -> Array:
	var unanswered := pending.values()
	pending.clear()
	return unanswered


func _remember(token: String, affordance: Dictionary) -> void:
	if token == "":
		return
	var about := ""
	var payload: Variant = affordance.get("payload")
	if typeof(payload) == TYPE_DICTIONARY:
		for value in payload.values():
			if typeof(value) == TYPE_DICTIONARY and typeof(value.get("entity")) == TYPE_STRING:
				about = value["entity"]
				break
	pending[token] = {"action_type": String(affordance.get("action_type", "")),
		"target": affordance.get("target"), "about": about}
