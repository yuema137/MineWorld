## Every interaction request the 3D client sends (step-15 §4.4, D-16a-4).
##
## With `slice_link.gd`, which reports the body's strides as `move`, this is
## the only file of the client that names an action; the no-rule scan holds
## that (`tests/acceptance/tests/client_rules.rs`). It knows what a request is
## called and how its payload is shaped -- never whether it is allowed. A
## request is sent whatever the latest observation says about it, so the
## server, not the client, says why it cannot happen (`ADOPTION.md` §3.3).
##
## PR 16a: `talk`. S15 PR 12e adds kick, throw and shove here, and nowhere else.
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


## token -> { "action", "target", "words" }, until its answer arrives
var _pending := {}


## Ask to talk to `target`. Returns the request's token.
func talk(client: MineWorldClient, target: String, utterance: String,
		actor_location: Dictionary) -> String:
	var tok := client.submit(TALK, target, { "utterance": utterance }, actor_location)
	_pending[tok] = { "action": TALK, "target": target, "words": utterance }
	return tok


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
