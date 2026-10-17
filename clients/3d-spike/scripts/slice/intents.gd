## Every interaction request the 3D client sends (step-15 §4.4, D-16a-4).
##
## With `slice_link.gd`, which reports the body's strides as `move`, this is
## the only file of the client that names an action; the no-rule scan holds
## that (`tests/acceptance/tests/client_rules.rs`). It knows what a request is
## called and how its payload is shaped -- never whether it is allowed. A
## request is sent whatever the latest observation says about it, so the
## server, not the client, says why it cannot happen (`ADOPTION.md` §3.3).
##
## PR 16a: `talk`. S15 PR 12e adds kick, throw and shove here, and nowhere else. PR 16d sends every
## complete affordance a menu offers unchanged (`submit_offered`), naming none of them.
class_name SliceIntents
extends RefCounted

const TALK := "talk"
## What the player says when they press E. A presentation default: the words are
## the player's, and the server keeps whatever was said.
const DEFAULT_UTTERANCE := "Hello! A coffee, please."

## How each request reads in a sentence ("can't talk to Alice Moreau"): the
## catalog key of its verb (S20; wording is the client's, `ADOPTION.md` §4). An
## action without an entry reads as its own name, made readable.
const VERBS := { TALK: "link.verb.talk" }

## The verb as the log says it, in English whatever the language (INV-SET-6).
const LOG_VERBS := { TALK: "talk to" }


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


## Whether this file composes an incomplete affordance of `action`: talk only (Q-16d-7). Anything else
## incomplete is listed by the menus as not supported and never sent.
static func composes(action: String) -> bool:
	return action == TALK


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
