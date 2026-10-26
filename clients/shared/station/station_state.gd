class_name MineWorldStationState
extends RefCounted

## ARC-108: a station fixture shows the state the world disclosed about the place the observer stands
## in, and nothing else. Pure: no clock, no file, no node, no request; a client draws the result.
##
## Both reference clients take this module (`res://mineworld_station`), so the rule is written once
## (pr-s29f §5.1). The result always has three keys:
##
##     { "board":  { "state": "none" } | { "state": "lines", "lines": Array },
##       "window": "open" | "shut",
##       "kiosk":  "open" | "shut" }
##
## Example: a hall tagged [station, public, landmark] whose observation discloses a `shop` gets the
## kiosk open; the same hall with no `shop` gets it shut. Nothing not disclosed is ever drawn open.

const NONE := "none"      # board: nothing disclosed
const LINES := "lines"    # board: a departures record was disclosed
const OPEN := "open"
const SHUT := "shut"
const TICKET_OFFICE := "ticket-office"   # the role tag (CORE_CONCEPTS §6.6, ARC-105)
const TICKET := "ticket"                 # the item-kind tag a fare carries (S21 QRT-10)


## The fixtures of the observer's place, from one observation (a `MineWorldObservation`, or anything
## with its `place`, `entity`, `entities` and `component_value` calls).
static func of(observation) -> Dictionary:
	var here: String = observation.place()
	if here == "":
		return nothing_disclosed()
	var place_tags := PackedStringArray()
	for tag in observation.entity(here).get("tags", []):
		place_tags.append(String(tag))
	var item_tags := {}
	for perceived in observation.entities():
		if typeof(perceived) != TYPE_DICTIONARY or String(perceived.get("entity_type", "")) != "item":
			continue
		var tags := PackedStringArray()
		for tag in perceived.get("tags", []):
			tags.append(String(tag))
		item_tags[String(perceived.get("id", ""))] = tags
	return evaluate(place_tags, observation.component_value(here, "departures"),
		observation.component_value(here, "shop"), item_tags)


## The fixtures of a place nothing was disclosed about: board blank, window and kiosk shut.
static func nothing_disclosed() -> Dictionary:
	return evaluate(PackedStringArray(), null, null, {})


## The same rule from its parts.
##   place_tags  the tags the world lists for the observer's place
##   departures  the place's disclosed `departures` payload, or null
##   shop        the place's disclosed `shop` payload, or null
##   item_tags   item id string → PackedStringArray of that kind's disclosed tags (absent: none known)
static func evaluate(place_tags: PackedStringArray, departures: Variant, shop: Variant,
		item_tags: Dictionary) -> Dictionary:
	return {"board": _board(departures), "window": _window(place_tags, shop, item_tags),
		"kiosk": _kiosk(place_tags, shop)}


static func _board(departures: Variant) -> Dictionary:
	if departures == null:
		return {"state": NONE}
	var lines: Array = []
	if typeof(departures) == TYPE_ARRAY:
		lines = departures
	elif typeof(departures) == TYPE_DICTIONARY and typeof(departures.get("lines")) == TYPE_ARRAY:
		lines = departures["lines"]
	return {"state": LINES, "lines": lines}


static func _window(place_tags: PackedStringArray, shop: Variant, item_tags: Dictionary) -> String:
	if not place_tags.has(TICKET_OFFICE) or typeof(shop) != TYPE_DICTIONARY \
			or typeof(shop.get("listed")) != TYPE_ARRAY:
		return SHUT
	for entry in shop["listed"]:
		if typeof(entry) != TYPE_DICTIONARY:
			continue
		var item: Variant = entry.get("item")
		var id := String(item.get("entity", "")) if typeof(item) == TYPE_DICTIONARY else str(item)
		var tags: Variant = item_tags.get(id, PackedStringArray())
		if (typeof(tags) == TYPE_PACKED_STRING_ARRAY or typeof(tags) == TYPE_ARRAY) and tags.has(TICKET):
			return OPEN
	return SHUT


static func _kiosk(place_tags: PackedStringArray, shop: Variant) -> String:
	return OPEN if not place_tags.has(TICKET_OFFICE) and shop != null else SHUT
