class_name MineWorldNeeds
extends RefCounted

## What the packs disclose about my needs, my sleep, my next part of the day and my employer's
## complaints about lateness (S35 N-e; `pr-s35-ne-presentation.md` §4.1).
##
## Data only, like `observation.gd`: no wording and no thresholds. A client shows the words the packs
## chose and never decides whether a person is hungry, tired or may sleep (`ARC-47`, I-Ne-1). A need is
## named by the `words` map the `needs` record discloses, never by a name written here, so a third need
## needs no change in this file. Every function returns `{}` or `[]` when nothing is disclosed, and a
## field that is missing or of an odd shape skips its record without an error.

## The component types the packs disclose (`needs` and `asleep` are N-b's and N-a's; `agenda` is LE-a's).
const NEEDS := "needs"
const ASLEEP := "asleep"
const AGENDA := "agenda"
## The event a lateness is told by (LE-b).
const LATE := "late-for-shift"
## The deficit scale of `MODULE_SPEC.md` §4.1, for a bar's range only: a level is shown, never computed.
const LEVEL_MAX := 1000


## My needs in the `words` map's order: `[{need, word, level}]`. `level` is the record's number under the
## need's name, or -1 when that number is missing (no bar is drawn for -1).
static func own_needs(obs: MineWorldObservation) -> Array:
	var record := obs.own_component(NEEDS)
	var words: Variant = record.get("words")
	if typeof(words) != TYPE_DICTIONARY:
		return []
	var out: Array = []
	for need in words:
		var word: Variant = words[need]
		if typeof(word) != TYPE_STRING:
			continue
		var level: Variant = _int(record.get(need))
		out.append({"need": String(need), "word": String(word), "level": -1 if level == null else level})
	return out


## Whether `id` is asleep as the observer was told, and since when: `{since, alarm}` where `alarm` is
## null unless the observer is the sleeper (the pack discloses the alarm to the sleeper only). `{}` when
## nothing is disclosed about `id`.
static func asleep_of(obs: MineWorldObservation, id: String) -> Dictionary:
	var record := obs.component(id, ASLEEP)
	var since: Variant = _int(record.get("since"))
	if since == null or record.get("asleep") == false:
		return {}
	return {"since": since, "alarm": _int(record.get("alarm"))}


## My next part of the day, as my agenda names it: `{label, place, from, punctual}` with `place` an
## entity id and `from` a world instant. `{}` when the agenda discloses no next part.
static func next_part(obs: MineWorldObservation) -> Dictionary:
	var next: Variant = obs.own_component(AGENDA).get("next")
	if typeof(next) != TYPE_DICTIONARY:
		return {}
	var from: Variant = _int(next.get("from"))
	var label: Variant = next.get("label")
	var place := _ref(next.get("place"))
	if from == null or typeof(label) != TYPE_STRING or place == "":
		return {}
	return {"label": String(label), "place": place, "from": from, "punctual": next.get("punctual") == true}


## Every `late-for-shift` fact in this observation about me, oldest first:
## `[{id, employer, late_by, count, level, absent}]`. `id` is the fact's event id, so a client shows each
## once; `late_by` is seconds; `level` is the pack's word.
static func lateness(obs: MineWorldObservation) -> Array:
	var me := obs.observer()
	var out: Array = []
	for event in obs.events_of(LATE):
		var late := _payload_of(event)
		var employer := _ref(late.get("employer"))
		var late_by: Variant = _int(late.get("late_by"))
		var count: Variant = _int(late.get("count"))
		var level: Variant = late.get("level")
		if _ref(late.get("employee")) != me or employer == "" or late_by == null or count == null:
			continue
		if typeof(level) != TYPE_STRING:
			continue
		out.append({"id": String(event.get("id", "")), "employer": employer, "late_by": late_by,
			"count": count, "level": String(level), "absent": late.get("absent") == true})
	return out


## The payload of an event envelope: the pack's own fields, inside the envelope's `payload.payload`.
static func _payload_of(event: Variant) -> Dictionary:
	if typeof(event) != TYPE_DICTIONARY:
		return {}
	var outer: Variant = event.get("payload")
	if typeof(outer) != TYPE_DICTIONARY:
		return {}
	var inner: Variant = outer.get("payload")
	return inner if typeof(inner) == TYPE_DICTIONARY else {}


## A whole number as disclosed (JSON numbers arrive as doubles), or null.
static func _int(value: Variant) -> Variant:
	if typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT:
		return int(value)
	return null


## An identity as a string, from a typed reference `{entity, entity_type}` or a plain identity; "" when
## there is none.
static func _ref(value: Variant) -> String:
	if typeof(value) == TYPE_DICTIONARY:
		return _ref(value.get("entity"))
	if typeof(value) == TYPE_STRING:
		return value
	if typeof(value) == TYPE_INT or typeof(value) == TYPE_FLOAT:
		return str(int(value))
	return ""
