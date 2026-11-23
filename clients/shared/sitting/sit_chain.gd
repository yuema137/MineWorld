## The walk-then-sit chain, shared by both reference clients (S34-e §5.2; I-e-3, I-S34-5): "click a seat"
## and "look at a seat and press E" send the same intents, because each client calls these functions.
##
## It names no action type: the client gives the two it composes (`MineWorldSitChain.new(sit_type,
## stand_type)`), as the trade module takes its table from its client. It reads one server verdict,
## `too_far_away` on the offer of the point, to choose "walk there first". It measures no distance,
## retries nothing, and sends each offer exactly as it was offered. Its answers say what the client does:
##
##   {}                          nothing to send, nothing to walk
##   {walk: {place, local}}      walk to `local` (the point's disclosed `at`, mm, in `place`); then report
##                               [method walk_ended]
##   {submit: affordance}        send this complete affordance as offered; the server answers
class_name MineWorldSitChain
extends RefCounted

## The component a place discloses its sit points in (the posture pack's `sit-points`). Named here, in the
## sitting module, so no client script outside it holds the literal (I-e-4).
const DISCLOSED := "sit-points"

var sit_type: String
var stand_type: String
## The point whose walk is under way: {place, point}, or {} when none is.
var _walking: Dictionary = {}


func _init(the_sit_type: String, the_stand_type: String) -> void:
	sit_type = the_sit_type
	stand_type = the_stand_type


## A click on, or E at, the sit point `point` of the observer's place.
func choose(observation, point: String) -> Dictionary:
	_walking = {}
	var offer := _offer_for(observation, point)
	var reason: Variant = offer.get("unavailable_reason")
	# A point the frame offers nothing for (out of reach of the observation) is walked to as well;
	# the newest offer is read once the walk ends.
	if offer.is_empty() or reason == "too_far_away":
		var at: Variant = _disclosed_at(observation, point)
		if at != null:
			_walking = {"place": observation.place(), "point": point}
			return {"walk": {"place": observation.place(), "local": at}}
	return {} if offer.is_empty() else {"submit": offer}


## The walk started by [method choose] ended. Once it completed, in the same place, the newest offer of that
## point is sent, whatever its verdict now: one try, never a retry (INV-6).
func walk_ended(observation, completed: bool) -> Dictionary:
	var walking := _walking
	_walking = {}
	if walking.is_empty() or not completed or observation.place() != String(walking["place"]):
		return {}
	var offer := _offer_for(observation, String(walking["point"]))
	return {} if offer.is_empty() else {"submit": offer}


## The player leaves the seat: the offered complete stand, if the frame has one.
func leave(observation) -> Dictionary:
	var offered: Array = observation.complete_affordances(stand_type, "")
	return {} if offered.is_empty() else {"submit": offered[0]}


## Forgets any remembered walk (a floor click, a walk key, another choice, a place change, a lost connection).
func cancel() -> void:
	_walking = {}


## The newest complete target-less offer of sit_type whose payload names `point`, or {}.
func _offer_for(observation, point: String) -> Dictionary:
	for offered in observation.complete_affordances(sit_type, ""):
		var payload: Variant = offered.get("payload")
		if typeof(payload) == TYPE_DICTIONARY and String(payload.get("point", "")) == point:
			return offered
	return {}


## The disclosed position of `point` in the observer's place (`{x, y}`, mm), or null when it is not disclosed.
func _disclosed_at(observation, point: String) -> Variant:
	var disclosed: Variant = observation.component_value(observation.place(), DISCLOSED)
	if typeof(disclosed) != TYPE_ARRAY:
		return null
	for p in disclosed:
		if typeof(p) == TYPE_DICTIONARY and String(p.get("key", "")) == point and typeof(p.get("at")) == TYPE_DICTIONARY:
			return p["at"]
	return null
