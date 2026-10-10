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

## The frame's `acted_through`: an `ActionId` string, or `null`.
var _acted_through: Variant = null


func _init(observation: Dictionary, acted_through_value: Variant = null) -> void:
	frame = observation
	_acted_through = acted_through_value


## The newest request this connection submitted that this observation already reflects — the
## `action_id` its `resolved` answer carried — or `null` before the first (`PROTOCOL.md` §5.2). A
## client that moved a body ahead of the server reconciles once this reaches the request it predicted.
func acted_through() -> Variant:
	return null if _acted_through == null else String(_acted_through)


## The facts in this observation of one event type, oldest first: what the observer learned since
## its previous frame (`PROTOCOL.md` §5.2). An event type a client does not know is just "something
## happened" — never an error.
func events_of(event_type: String) -> Array:
	var found := []
	for event in events():
		if String(event.get("event_type", "")) == event_type:
			found.append(event)
	return found


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
##
## Only a payload that is a JSON object comes back; anything else is `{}`. Read a payload of another
## shape — a listing that is an array — through [method component_value].
func component(id: String, component_type: String) -> Dictionary:
	var payload: Variant = component_value(id, component_type)
	return payload if typeof(payload) == TYPE_DICTIONARY else {}


## The payload of one disclosed component exactly as it arrived — an object, an array, a number — or
## `null` when that component was not disclosed about that entity.
##
## The same lookup as [method component], without its Dictionary typing: a pack chooses the shape of
## what it discloses, and a reader that coerced it would lose what it does not expect.
func component_value(id: String, component_type: String) -> Variant:
	var components: Variant = entity(id).get("components", [])
	if typeof(components) != TYPE_ARRAY:
		return null
	for record in components:
		if typeof(record) == TYPE_DICTIONARY \
				and String(record.get("component_type", "")) == component_type:
			return record.get("payload")
	return null


## The payload of one component the world disclosed about the observer itself.
##
## What a client reads to show *what I have been told*: a pack that discloses a person's own state
## does it in that person's own observation and nowhere else.
func own_component(component_type: String) -> Dictionary:
	return component(observer(), component_type)


## What a perceived entity is called, or `""` when the world disclosed no name for it.
##
## The `naming` System Pack's `display-name` record, payload `{ "name": "Alice Moreau" }`
## (`docs/DECISIONS.md` `ARC-31`). A client shows it and never invents one: an empty answer means
## this observer was told no name — the world has no `naming`, or nobody named that entity — and the
## honest thing to show is then something else the frame carries, never a guess.
func display_name(id: String) -> String:
	var name: Variant = component(id, "display-name").get("name", "")
	return name if typeof(name) == TYPE_STRING else ""


## The edges the observer was shown. Never the world's relation graph.
func relations() -> Array:
	var listed: Variant = frame.get("relations", [])
	return listed if typeof(listed) == TYPE_ARRAY else []


## The facts the observer is entitled to. Empty in revision 1 of the protocol.
func events() -> Array:
	var listed: Variant = frame.get("events", [])
	return listed if typeof(listed) == TYPE_ARRAY else []


## What the observer may attempt, each with the server's answer, in the order the server listed it.
##
## With no argument, every affordance. `action_type` narrows the list to one action type (`""`
## matches any). `target` narrows it to one target: `null` matches any target, `""` matches the
## affordances directed at nobody, and an identity string matches exactly that target.
##
## Every match comes back, never only the first: several complete affordances routinely share an
## action type and a target and differ only in `payload`, one per choice the world offers, and a
## client keeps them apart by their position in this list (`server/PROTOCOL.md` §5).
func affordances(action_type: String = "", target: Variant = null) -> Array:
	var listed: Variant = frame.get("affordances", [])
	if typeof(listed) != TYPE_ARRAY:
		return []
	if action_type.is_empty() and target == null:
		return listed
	var found: Array = []
	for offered in listed:
		if not action_type.is_empty() and String(offered.get("action_type", "")) != action_type:
			continue
		if target != null and _target_of(offered) != String(target):
			continue
		found.append(offered)
	return found


## The complete affordances among [method affordances], filtered the same way.
##
## A complete affordance carries the exact request the offering system would accept
## (`docs/DECISIONS.md` `ARC-34`); submit it with [method MineWorldClient.submit_affordance],
## knowing nothing about the action.
func complete_affordances(action_type: String = "", target: Variant = null) -> Array:
	var found: Array = []
	for offered in affordances(action_type, target):
		if is_complete(offered):
			found.append(offered)
	return found


## Whether an affordance is complete: whether it carries a `payload`.
##
## The key's presence decides, not its value. The protocol leaves `payload` out of an affordance that
## has none, so a present `null` is a complete affordance whose payload is `null` — an action that
## takes no arguments.
static func is_complete(affordance: Dictionary) -> bool:
	return affordance.has("payload")


## Every affordance that concerns one entity, complete or not, in list order.
##
## One concerns `id` when it is directed at `id`, or when its `payload` is an object with a top-level
## value that refers to `id` — the identity string itself, or the contract's typed reference
## `{ "entity": id, "entity_type": … }`, which is how a typed identity such as an item's travels
## (`TypedEntityRef`). That is how an affordance directed at nobody but about a thing — one naming an
## item or an object in its payload — is found from that thing without this module or its client
## knowing the action. Nothing deeper is searched.
func affordances_about(id: String) -> Array:
	var found: Array = []
	for offered in affordances():
		if _target_of(offered) == id or _payload_names(offered, id):
			found.append(offered)
	return found


## The server's answer about one action against one target, or `{}` when it was not offered.
##
## `target` is `""` for an action directed at nobody. The **first** match: for complete affordances,
## of which several may share an action type and a target, use [method affordances].
func affordance(action_type: String, target: String = "") -> Dictionary:
	var found := affordances(action_type, target)
	return found[0] if not found.is_empty() else {}


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
	for offered in affordances("", target):
		found.append(String(offered.get("action_type", "")))
	return found


## An affordance's target as an identity string, `""` for one directed at nobody.
static func _target_of(offered: Dictionary) -> String:
	var against: Variant = offered.get("target")
	return "" if against == null else String(against)


## Whether an affordance's payload is an object with a top-level value referring to `id`: the
## identity string, or a typed reference `{ "entity": id, … }`.
static func _payload_names(offered: Dictionary, id: String) -> bool:
	var payload: Variant = offered.get("payload")
	if typeof(payload) != TYPE_DICTIONARY:
		return false
	for value in payload.values():
		if typeof(value) == TYPE_STRING and value == id:
			return true
		if typeof(value) == TYPE_DICTIONARY and typeof(value.get("entity")) == TYPE_STRING \
				and value["entity"] == id:
			return true
	return false
