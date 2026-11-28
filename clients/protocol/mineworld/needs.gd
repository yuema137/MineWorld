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


## The hunger models a schema-2 record names (`ARC-370`), and the one whose rows carry a reserve.
const MODELS := ["unitless", "food-energy"]
const RESERVE_MODEL := "food-energy"
## The unit every reserve number is in, as the record states it.
const RESERVE_UNIT := "kcal"
## The largest kcal number the packs state (`MODULE_SPEC.md` §4.2).
const KCAL_MAX := 100000


## My needs in the `words` map's order: `[{need, word, level}]`. `level` is the record's number under the
## need's name, or -1 when that number is missing (no bar is drawn for -1).
##
## The record's schema is read first (`ARC-370`): schema 1 is read as it always was; schema 2 names its
## hunger model, and in the food-energy model the need the reserve `replaces_need` reads as a reserve row
## `{need, word, level: -1, reserve: {available_kcal, capacity_kcal, use_kcal_per_world_day, bar,
## maximum}}` — the owner's numbers, checked and passed through, never computed. A malformed reserve
## drops that row (never a deficit bar in its place); the other needs still read. Any other schema reads
## nothing.
static func own_needs(obs: MineWorldObservation) -> Array:
	var schema := obs.component_schema(obs.observer(), NEEDS)
	var record := obs.own_component(NEEDS)
	if schema == 1:
		return _rows(record, "", {})
	if schema != 2 or not MODELS.has(record.get("hunger_model")):
		return []
	if record.get("hunger_model") != RESERVE_MODEL:
		return _rows(record, "", {})
	var reserve := _reserve(record.get("food_energy"))
	var replaced: Variant = record.get("food_energy", {}).get("replaces_need") \
			if typeof(record.get("food_energy")) == TYPE_DICTIONARY else null
	if typeof(replaced) != TYPE_STRING or String(replaced) == "":
		return []
	return _rows(record, String(replaced), reserve)


## The rows of a record's `words` map; the need named `replaced` reads as `reserve`, or not at all when
## `reserve` is empty (malformed).
static func _rows(record: Dictionary, replaced: String, reserve: Dictionary) -> Array:
	var words: Variant = record.get("words")
	if typeof(words) != TYPE_DICTIONARY:
		return []
	var out: Array = []
	for need in words:
		var word: Variant = words[need]
		if typeof(word) != TYPE_STRING:
			continue
		if replaced != "" and String(need) == replaced:
			if not reserve.is_empty():
				out.append({"need": String(need), "word": String(word), "level": -1, "reserve": reserve})
			continue
		var level: Variant = _int(record.get(need))
		out.append({"need": String(need), "word": String(word), "level": -1 if level == null else level})
	return out


## A disclosed `food_energy` block, checked by type and equality only: `{}` unless every number is
## whole, within its bound and `unit` is kcal.
static func _reserve(block: Variant) -> Dictionary:
	if typeof(block) != TYPE_DICTIONARY or block.get("unit") != RESERVE_UNIT:
		return {}
	var bar: Variant = block.get("bar")
	if typeof(bar) != TYPE_DICTIONARY:
		return {}
	var capacity: Variant = _whole(block.get("capacity_kcal"))
	var use: Variant = _whole(block.get("use_kcal_per_world_day"))
	var available: Variant = _whole(block.get("available_kcal"))
	var maximum: Variant = _whole(bar.get("maximum"))
	var value: Variant = _whole(bar.get("value"))
	if capacity == null or use == null or available == null or maximum == null or value == null:
		return {}
	if capacity != clampi(capacity, 1, KCAL_MAX) or use != clampi(use, 1, KCAL_MAX):
		return {}
	if available != clampi(available, 0, capacity) or maximum != clampi(maximum, 1, KCAL_MAX):
		return {}
	if value != clampi(value, 0, maximum):
		return {}
	return {"available_kcal": available, "capacity_kcal": capacity, "use_kcal_per_world_day": use,
		"bar": value, "maximum": maximum}


## A whole number as disclosed, or null: an integer, or a double with no fraction (JSON numbers arrive as
## doubles); never a boolean, a string or a fraction.
static func _whole(value: Variant) -> Variant:
	if typeof(value) == TYPE_INT:
		return value
	if typeof(value) == TYPE_FLOAT and value == floorf(value) and is_finite(value):
		return int(value)
	return null


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
