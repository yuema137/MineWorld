class_name MineWorldAttributes
extends RefCounted

## What a perceived person's disclosed attributes say, as text a client shows (S30-f §§5.4, 5.9;
## `docs/MODULE_SPEC.md` §4.1 "Disclosure").
##
## Data only, like `observation.gd`: the server renders every phrase, and this module selects, orders,
## joins and cuts those phrases — it renders no value and decides nothing. The same composition is
## implemented in Rust (`authoring/attributes/src/disclosed.rs`) and Python (`mineworld_sdk.attributes`);
## one hand-written golden (`authoring/attributes/tests/fixtures/composition.json`) holds all three
## equal (`checks/attributes_check.gd`).
##
## An attribute block is `{ schema: 2, entries }`: the payload of the `traits` component, or the member
## `attributes` of any other component's payload (an owner's). Any other shape is skipped whole.

## The component whose payload is a block.
const TRAITS := "traits"
## The member of an owner's payload that carries its block.
const OWNER_MEMBER := "attributes"
## The block schema this module reads.
const SCHEMA := 2
## The fields of an entry of that schema: an entry with any other is a newer shape, and its block is
## skipped whole.
const FIELDS := ["attribute", "kind", "label", "facets", "rank", "value", "text"]
## What joins two texts.
const SEPARATOR := ", "
## What a cut composition ends with.
const CUT_MARK := " …"
## The glance's facet and budget, in UTF-8 bytes: who I see.
const GLANCE_FACET := "observed"
const GLANCE_BYTES := 160


## The entries of every block an entity record (`MineWorldObservation.entity(id)`) carries, merged in
## rank order: `(rank, attribute)`, an absent rank last.
static func blocks(entity: Dictionary) -> Array:
	var components: Variant = entity.get("components", [])
	return merged(components if typeof(components) == TYPE_ARRAY else [])


## The same from a list of component records `{component_type, payload}`.
static func merged(components: Array) -> Array:
	var entries: Array = []
	for record in components:
		if typeof(record) != TYPE_DICTIONARY:
			continue
		var payload: Variant = record.get("payload")
		if typeof(payload) != TYPE_DICTIONARY:
			continue
		var block: Variant = payload if String(record.get("component_type", "")) == TRAITS \
				else payload.get(OWNER_MEMBER)
		entries.append_array(_block(block))
	entries.sort_custom(_before)
	return entries


## A block's entries, or `[]` when it is not a block this module reads.
static func _block(block: Variant) -> Array:
	if typeof(block) != TYPE_DICTIONARY:
		return []
	var schema: Variant = block.get("schema")
	if (typeof(schema) != TYPE_INT and typeof(schema) != TYPE_FLOAT) or int(schema) != SCHEMA:
		return []
	var entries: Variant = block.get("entries")
	if typeof(entries) != TYPE_ARRAY:
		return []
	for entry in entries:
		if typeof(entry) != TYPE_DICTIONARY or typeof(entry.get("attribute")) != TYPE_STRING \
				or typeof(entry.get("label")) != TYPE_STRING:
			return []
		for key in entry:
			if not FIELDS.has(key):
				return []
	return entries.duplicate()


static func _before(left: Dictionary, right: Dictionary) -> bool:
	var a: Variant = left.get("rank")
	var b: Variant = right.get("rank")
	if (a == null) != (b == null):
		return b == null
	if a != null and int(a) != int(b):
		return int(a) < int(b)
	return String(left["attribute"]) < String(right["attribute"])


static func _bytes(text: String) -> int:
	return text.to_utf8_buffer().size()


## The non-empty texts of `entries` carrying `facet` (every entry when `facet` is empty), in order.
static func _texts(entries: Array, facet: String) -> PackedStringArray:
	var out := PackedStringArray()
	for entry in entries:
		if facet != "":
			var facets: Variant = entry.get("facets", [])
			if typeof(facets) != TYPE_ARRAY or not facets.has(facet):
				continue
		var text: Variant = entry.get("text")
		if typeof(text) == TYPE_STRING and text != "":
			out.append(text)
	return out


## Select by `facet` ("" for every entry), join with ", ", and cut at `budget` UTF-8 bytes (-1: no cut)
## at an entry boundary with " …"; a first text longer than the budget alone is cut at a character
## boundary and ends with "…".
static func compose(entries: Array, facet: String, budget: int) -> String:
	var texts := _texts(entries, facet)
	var joined := SEPARATOR.join(texts)
	if budget < 0 or _bytes(joined) <= budget:
		return joined
	var length := 0
	var kept := 0
	for index in texts.size():
		var next := length + _bytes(texts[index]) + (0 if index == 0 else _bytes(SEPARATOR))
		if next + _bytes(CUT_MARK) > budget:
			break
		length = next
		kept = index + 1
	if kept > 0:
		return SEPARATOR.join(texts.slice(0, kept)) + CUT_MARK
	var first: String = texts[0] if texts.size() > 0 else ""
	var room := budget - _bytes("…")
	var cut := ""
	for character in first:
		if _bytes(cut + character) > room:
			break
		cut += character
	return cut + "…"


## Who I see: the glance of `entity`, as the view and every client show it.
static func glance(entity: Dictionary) -> String:
	return compose(blocks(entity), GLANCE_FACET, GLANCE_BYTES)


## The inspect list: every text, one line each.
static func inspect(entries: Array) -> PackedStringArray:
	return _texts(entries, "")


## A typed value as the own sheet shows it when it renders no text.
static func _typed(value: Variant) -> String:
	if typeof(value) != TYPE_DICTIONARY:
		return ""
	for kind in ["category", "level", "quantity", "text"]:
		if value.has(kind):
			return String(value[kind])
	if value.has("flag"):
		return "yes" if bool(value["flag"]) else "no"
	if value.has("instant"):
		return "%d" % int(value["instant"])
	if value.has("set") and typeof(value["set"]) == TYPE_ARRAY:
		var members := PackedStringArray()
		for member in value["set"]:
			members.append(String(member))
		members.sort()
		return SEPARATOR.join(members)
	return ""


## The own sheet: `label: text`, or `label: <typed value>` when there is no text; an entry that shows
## nothing is left out.
static func sheet(entries: Array) -> PackedStringArray:
	var out := PackedStringArray()
	for entry in entries:
		var text: Variant = entry.get("text")
		var shown: String = text if typeof(text) == TYPE_STRING and text != "" \
				else _typed(entry.get("value"))
		if shown != "":
			out.append("%s: %s" % [String(entry["label"]), shown])
	return out
