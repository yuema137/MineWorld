class_name MineWorldDelta
extends RefCounted

## Applies a `delta` frame's `ObservationDelta` to the whole observation a client holds
## (`server/PROTOCOL.md` §5.3, `docs/DECISIONS.md` `DEP-15`).
##
## [MineWorldClient] does this itself and emits whole observations, so a client never sees a delta
## (step-12 R-S11-7). This file is the field-by-field table of §5.3 and nothing else:
##
## [codeblock]
## at                 always present; replaces
## self_location      present only if it changed; replaces, `null` included
## entities.upsert    each replaces the entity with its id, or is added; then sorted by id
## entities.remove    ids no longer perceived; one the held observation does not list is an error
## relations          present only if changed; replaces the whole list
## affordances        present only if changed; replaces the whole list, in its order
## events             always present; replaces (a since-the-last-frame stream)
## [/codeblock]
##
## Identities stay strings. Ids are sorted as the numbers they are — by length, then by digits —
## without ever being parsed into Godot's one number type, which would merge two above 2^53.


## The observation `delta` describes, from `held`; `null` when the delta removes an entity `held`
## does not list (the module then drops the connection and resumes, which yields a whole one).
static func apply(held: Dictionary, delta: Dictionary) -> Variant:
	var by_id := {}
	for entity in held.get("entities", []):
		by_id[String(entity.get("id", ""))] = entity
	var changes: Dictionary = delta.get("entities", {})
	for id in changes.get("remove", []):
		if not by_id.has(String(id)):
			return null
		by_id.erase(String(id))
	for entity in changes.get("upsert", []):
		by_id[String(entity.get("id", ""))] = entity
	var ids: Array = by_id.keys()
	ids.sort_custom(_id_before)
	var entities := []
	for id in ids:
		entities.append(by_id[id])

	var next := held.duplicate()
	next["at"] = delta.get("at", held.get("at"))
	if delta.has("self_location"):
		next["self_location"] = delta["self_location"]
	next["entities"] = entities
	if delta.has("relations"):
		next["relations"] = delta["relations"]
	if delta.has("affordances"):
		next["affordances"] = delta["affordances"]
	next["events"] = delta.get("events", [])
	return next


## Ascending identity order, on the decimal strings: a shorter one is smaller.
static func _id_before(left: String, right: String) -> bool:
	if left.length() != right.length():
		return left.length() < right.length()
	return left < right
