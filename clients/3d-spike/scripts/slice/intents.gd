## Every interaction request the 3D client sends (step-15 §4.4, D-16a-4).
##
## With `slice_link.gd`, which reports the body's strides as `move`, this is
## the only file of the client that names an action; the no-rule scan holds
## that (`tests/acceptance/tests/client_rules.rs`). It knows what a request is
## called and how its payload is shaped -- never whether it is allowed. A
## request is sent whatever the latest observation says about it, so the
## server, not the client, says why it cannot happen (`ADOPTION.md` §3.3).
##
## PR 16a: `talk`. S15 PR 12e adds kick, throw and shove here, and nowhere else
## (step-11 §22 SD-E6): each looks for the complete offer that matches and
## submits it unchanged; when none matches, it builds the same request by name
## and submits it anyway, so the server says why (`too_far_away`, `unavailable`).
## PR 16d sends every complete affordance a menu offers unchanged (`submit_offered`), naming none of them.
class_name SliceIntents
extends RefCounted

const TALK := "talk"
const KICK := "kick"
const THROW := "throw"
const SHOVE := "shove"
## What the player says when they press E. A presentation default: the words are
## the player's, and the server keeps whatever was said.
const DEFAULT_UTTERANCE := "Hello! A coffee, please."

## How each request reads in a sentence ("can't talk to Alice Moreau"): the
## catalog key of its verb (S20; wording is the client's, `ADOPTION.md` §4). An
## action without an entry reads as its own name, made readable.
const VERBS := { TALK: "link.verb.talk", KICK: "link.verb.kick", THROW: "link.verb.throw",
	SHOVE: "link.verb.shove" }

## The verb as the log says it, in English whatever the language (INV-SET-6).
const LOG_VERBS := { TALK: "talk to", KICK: "kick", THROW: "throw", SHOVE: "shove" }


## The verb, as a catalog reference worded whenever the message holding it is rendered.
static func verb(action: String) -> Variant:
	return MineWorldText.ref(VERBS[action]) if VERBS.has(action) else MineWorldText.readable(action)


static func log_verb(action: String) -> String:
	return LOG_VERBS.get(action, action)


## token -> { "action", "target", "words" } for a talk, { "action", "target", "about" } for an offer
## sent as offered, until its answer arrives
var _pending := {}


## Ask to talk to `target`. Returns the request's token.
func talk(client: MineWorldClient, target: String, utterance: String,
		actor_location: Dictionary) -> String:
	var tok := client.submit(TALK, target, { "utterance": utterance }, actor_location)
	_pending[tok] = { "action": TALK, "target": target, "words": utterance }
	return tok


## PR 16d: a complete affordance, sent exactly as the world offered it, whatever it said about it
## (`submit_affordance`, `ARC-34`). No action is named here: the affordance carries its own type.
func submit_offered(client: MineWorldClient, affordance: Dictionary, actor_location: Dictionary) -> String:
	var tok := client.submit_affordance(affordance, actor_location)
	if tok != "":
		var target: Variant = affordance.get("target")
		_pending[tok] = { "action": String(affordance.get("action_type", "")),
			"target": "" if target == null else String(target),
			"about": SliceOfferMenu.about_of(affordance) }
	return tok


## Whether this file composes an incomplete affordance of `action` from a menu: talk only (Q-16d-7). The
## seller actions are composed from the trade panel (`trades`); anything else incomplete is listed by the
## menus as not supported and never sent.
static func composes(action: String) -> bool:
	return action == TALK


## S31-e: the seller actions this client composes from the trade panel, as the shared trade module reads
## them (`clients/shared/TRADE.md` §5.2): what the player supplies, the rows each serves, its button's words.
const TRADE_INPUTS := {
	"list-for-sale": {"ask": ["amount"], "row": ["held", "note"], "label": "trade.list"},
	"set-price": {"ask": ["amount"], "row": ["ask", "shelf"], "label": "trade.change-price"},
	"withdraw": {"ask": [], "row": ["ask", "shelf"], "label": "trade.withdraw"},
}


## Whether `action` is composed from the trade panel beside the menus.
static func trades(action: String) -> bool:
	return TRADE_INPUTS.has(action)


## Composes a seller action from the trade panel's `input` `{item, price, shop}`: the item reference exactly
## as the world disclosed it, the parsed price, and `for: shop` when the row is the shop's (S31-e §5.6).
## Sent whatever the observation says; the server answers. Returns the token, or "" for another type.
func compose_trade(client: MineWorldClient, affordance: Dictionary, input: Dictionary,
		actor_location: Dictionary) -> String:
	var action := String(affordance.get("action_type", ""))
	if not trades(action):
		return ""
	var payload := {"item": input.get("item")}
	if action != "withdraw":
		payload["price"] = int(input.get("price", 0))
	if input.get("shop", false):
		payload["for"] = "shop"
	var tok := client.submit(action, null, payload, actor_location)
	if tok != "":
		_pending[tok] = {"action": action, "target": "", "about": SliceThings.ref(input.get("item"))}
	return tok


## Composes an incomplete affordance this file knows, for `target`: talk, with the default words.
func compose(client: MineWorldClient, affordance: Dictionary, target: String,
		actor_location: Dictionary) -> String:
	if String(affordance.get("action_type", "")) != TALK:
		return ""
	return talk(client, target, DEFAULT_UTTERANCE, actor_location)


## The connection dropped: every unanswered request, forgotten and never resent (13b §4.6 point 5).
func drop_pending() -> Array:
	var unanswered := _pending.values()
	_pending.clear()
	return unanswered


## The request a token stands for, handed over once with its answer; {} for a
## token this file did not send.
func take(token: String) -> Dictionary:
	var req: Dictionary = _pending.get(token, {})
	_pending.erase(token)
	return req


## Whether the latest observation offers talking to `target` as available, and
## if not, the reason the world stated. Shown to the player; never used to
## decide whether to send.
static func talk_offered(obs: MineWorldObservation, target: String) -> bool:
	return obs == null or obs.may(TALK, target)


static func talk_reason(obs: MineWorldObservation, target: String) -> Variant:
	return obs.unavailable_reason(TALK, target) if obs != null else null


## Kick `object` -- the reference the world disclosed for it, unchanged.
## Target-less, the object in the payload (`bodies`' `kick { object }`).
## Returns the token.
func kick(client: MineWorldClient, object: Variant, actor_location: Dictionary) -> String:
	return _send(client, KICK, null, { "object": object }, SliceObjects.id_of(object), actor_location)


## Throw `object` toward `toward` (`{ x, y }`, integer millimetres in the
## thrower's place frame), or with `toward` null along the default line -- the
## complete form the world offers. Returns the token.
func throw(client: MineWorldClient, object: Variant, toward: Variant, actor_location: Dictionary) -> String:
	return _send(client, THROW, null, { "object": object, "toward": toward }, SliceObjects.id_of(object),
		actor_location)


## Shove `person`, with an empty payload. Returns the token.
func shove(client: MineWorldClient, person: String, actor_location: Dictionary) -> String:
	return _send(client, SHOVE, person, {}, person, actor_location)


## The request every interaction builds, as `{ action, target, payload }` --
## what it sends when no offer matches, and what a matching offer must equal.
static func built(action: String, target: Variant, payload: Dictionary) -> Dictionary:
	return { "action": action, "target": target, "payload": payload }


## The complete offer in `obs` that equals the request this file would build,
## or {}: the same action type, the same target and the same payload. For a
## thrown object only the default line (`toward` null) is ever offered.
static func matching_offer(obs: MineWorldObservation, request: Dictionary) -> Dictionary:
	if obs == null:
		return {}
	var target: Variant = request["target"]
	for offered in obs.complete_affordances(String(request["action"])):
		var against: Variant = offered.get("target")
		if (against == null) != (target == null):
			continue
		if target != null and String(against) != String(target):
			continue
		if _same(offered.get("payload"), request["payload"]):
			return offered
	return {}


## Whether a payload the world offered equals the one built here, value by value
## (an offered whole number arrives as a float and is the same integer).
static func _same(offered: Variant, ours: Variant) -> bool:
	if typeof(ours) == TYPE_DICTIONARY:
		if typeof(offered) != TYPE_DICTIONARY or (offered as Dictionary).size() != (ours as Dictionary).size():
			return false
		for k in ours:
			if not (offered as Dictionary).has(k) or not _same(offered[k], ours[k]):
				return false
		return true
	if ours == null or offered == null:
		return ours == null and offered == null
	if typeof(ours) in [TYPE_INT, TYPE_FLOAT] and typeof(offered) in [TYPE_INT, TYPE_FLOAT]:
		return float(ours) == float(offered)
	return String(ours) == String(offered)


## Every interaction the client built, in order, as `built` says it, with
## whether it went out as the world's offer: { action, target, payload, offered }.
var sent: Array = []


func _send(client: MineWorldClient, action: String, target: Variant, payload: Dictionary, about: String,
		actor_location: Dictionary) -> String:
	var request := built(action, target, payload)
	var offer := matching_offer(client.latest, request)
	var tok := ""
	if not offer.is_empty():
		tok = client.submit_affordance(offer, actor_location)
	else:
		tok = client.submit(action, target, payload, actor_location)
	var record := request.duplicate()
	record["offered"] = not offer.is_empty()
	sent.append(record)
	_pending[tok] = { "action": action, "target": about }
	return tok
