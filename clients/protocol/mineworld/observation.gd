class_name MineWorldObservation
extends RefCounted

## One observation, read the way a client is allowed to read it.
##
## This is a **reader**, not a model. Every method here looks something up in the frame the server
## sent and returns it; not one of them decides anything. In particular [method may] reports the
## server's verdict about an action and never computes one: a distance comparison in a renderer is a
## defect (`docs/ENGINEERING_RULES.md` §§8-9), and the reason a client does not need one is that the
## answer is already in this frame.
##
## An observation is exhaustive by definition. It is not a cache, a page or a first result set, and
## there is no second call that would return more (`contracts/src/observation.rs`): when [method
## entity] answers `{}`, that means *this observer was not shown that entity*, which a client must
## draw as ignorance rather than resolve by asking again — there is nothing else to ask.
##
## Identities are **strings** everywhere in this file. Parsing one as a number silently merges two
## entities above 2^53; see `MineWorldClient` for the measurement.

## The frame exactly as it arrived, for a client that wants something this reader does not expose.
var frame: Dictionary


func _init(observation: Dictionary) -> void:
	frame = observation


## Whose view this is.
func observer() -> String:
	return String(frame.get("observer", ""))


## The world's clock when it was taken, in whole simulated seconds.
func at() -> int:
	return int(frame.get("at", 0))


## Where the observer itself is, or `{}` when the world does not model a position for it.
##
## This is what lets a client decide to move before deciding to act.
func self_location() -> Dictionary:
	var location: Variant = frame.get("self_location")
	return location if typeof(location) == TYPE_DICTIONARY else {}


## The place the observer is in, as an identity string, or `""`.
func place() -> String:
	var location := self_location()
	if location.is_empty() or typeof(location.get("place")) != TYPE_DICTIONARY:
		return ""
	return String(location["place"].get("entity", ""))


## Everything the observer perceives, in the order the server listed it.
func entities() -> Array:
	var listed: Variant = frame.get("entities", [])
	return listed if typeof(listed) == TYPE_ARRAY else []


## The identities of everything perceived.
func ids() -> PackedStringArray:
	var found := PackedStringArray()
	for entity in entities():
		found.append(String(entity.get("id", "")))
	return found


## One perceived entity, or `{}` when this observer was not shown it.
func entity(id: String) -> Dictionary:
	for perceived in entities():
		if String(perceived.get("id", "")) == id:
			return perceived
	return {}


## The identities of everything perceived that carries this tag.
##
## Tags are an open vocabulary and they are world data: a client uses them to choose a sprite, a
## colour or a label, which is an appearance decision made from what the world said rather than from
## a rule (`spike/FINDINGS.md` F8.4).
func tagged(tag: String) -> PackedStringArray:
	var found := PackedStringArray()
	for perceived in entities():
		var tags: Variant = perceived.get("tags", [])
		if typeof(tags) == TYPE_ARRAY and tags.has(tag):
			found.append(String(perceived.get("id", "")))
	return found


## Where a perceived entity is: its `location`, or `{}`.
func location_of(id: String) -> Dictionary:
	var perceived := entity(id)
	var location: Variant = perceived.get("location")
	return location if typeof(location) == TYPE_DICTIONARY else {}


## The payload of one component the world disclosed about one entity, or `{}`.
##
## A component is in an observation because its owning System Pack chose to disclose it to *this*
## observer. Most are not: an observation is a list of what was exposed deliberately, and a client
## that finds nothing here has been told nothing, not lied to (`INV-13`).
func component(id: String, component_type: String) -> Dictionary:
	var perceived := entity(id)
	var components: Variant = perceived.get("components", [])
	if typeof(components) != TYPE_ARRAY:
		return {}
	for record in components:
		if String(record.get("component_type", "")) == component_type:
			var payload: Variant = record.get("payload")
			return payload if typeof(payload) == TYPE_DICTIONARY else {}
	return {}


## The payload of one component the world disclosed about the observer itself.
##
## What a client reads to show *what I have been told*: a pack that discloses a person's own state
## does it in that person's own observation and nowhere else.
func own_component(component_type: String) -> Dictionary:
	return component(observer(), component_type)


## The edges the observer was shown. Never the world's relation graph.
func relations() -> Array:
	var listed: Variant = frame.get("relations", [])
	return listed if typeof(listed) == TYPE_ARRAY else []


## The facts the observer is entitled to. Empty in revision 1 of the protocol.
func events() -> Array:
	var listed: Variant = frame.get("events", [])
	return listed if typeof(listed) == TYPE_ARRAY else []


## Everything the observer may attempt, each with the server's answer.
func affordances() -> Array:
	var listed: Variant = frame.get("affordances", [])
	return listed if typeof(listed) == TYPE_ARRAY else []


## The server's answer about one action against one target, or `{}` when it was not offered.
##
## `target` is `""` for an action directed at nobody.
func affordance(action_type: String, target: String = "") -> Dictionary:
	for offered in affordances():
		if String(offered.get("action_type", "")) != action_type:
			continue
		var against: Variant = offered.get("target")
		var named := "" if against == null else String(against)
		if named == target:
			return offered
	return {}


## Whether the **server** says this may be attempted right now.
##
## Read, never computed. This is the whole reason a 2D client and a 3D client can offer the same
## interactions without either implementing a rule, and the reason neither of them measures a
## distance.
func may(action_type: String, target: String = "") -> bool:
	var offered := affordance(action_type, target)
	return bool(offered.get("available", false))


## Why not, when the server says it may not: a `Rejection` as the protocol carries one.
##
## A string for the kernel's closed reasons — `too_far_away`, `busy`, `permission_denied`,
## `no_supported_interaction`, `target_unavailable`, `unavailable`, `precondition_failed` — or a
## dictionary for a system's own code. A client maps it to its own wording, because the contract
## carries no display text on purpose (`DD-13`), and needs a fallback for a code it does not know.
func unavailable_reason(action_type: String, target: String = "") -> Variant:
	return affordance(action_type, target).get("unavailable_reason")


## What the owning system declared the action needs of space, unevaluated.
##
## For *showing* a requirement — "needs 3 m" — without checking it. The server has already checked
## it; this is the declaration, passed through.
func requirement(action_type: String, target: String = "") -> Dictionary:
	var declared: Variant = affordance(action_type, target).get("requirement")
	return declared if typeof(declared) == TYPE_DICTIONARY else {}


## Every action type offered against one target, whether available or not.
func offered_against(target: String = "") -> PackedStringArray:
	var found := PackedStringArray()
	for offered in affordances():
		var against: Variant = offered.get("target")
		var named := "" if against == null else String(against)
		if named == target:
			found.append(String(offered.get("action_type", "")))
	return found
