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
## An attribute block is `{ schema: 3, words?, entries }`: the payload of the `traits` component, or the
## member `attributes` of any other component's payload (an owner's). Any other shape is skipped whole.
##
## Languages (S30-k §5.6, `ARC-129`): the server renders every locale the world's label packs offer, an
## entry carrying `texts` (and, in the holder's view, `labels`) per locale and a block carrying the
## reader `words` per locale. A reader picks among them with `reading(locale, offered)`: its own locale
## when offered, else the one offered locale of its language, else English. An entry without the
## reading's locale is shown in English, whole. No attribute text passes through `tr()` (INV-SET-10).

## The component whose payload is a block.
const TRAITS := "traits"
## The member of an owner's payload that carries its block.
const OWNER_MEMBER := "attributes"
## The block schema this module reads.
const SCHEMA := 3
## The fields of an entry of that schema: an entry with any other is a newer shape, and its block is
## skipped whole.
const FIELDS := ["attribute", "kind", "label", "facets", "rank", "value", "text", "labels", "texts"]
## The words of a reader composing in English (S30-f's constants).
const ENGLISH_WORDS := {"join": ", ", "more": " …", "pair": ": ", "yes": "yes", "no": "no"}
## The English reading: no locale, English words.
const ENGLISH := {"locale": "", "words": ENGLISH_WORDS}
## What joins two English texts.
const SEPARATOR := ", "
## What a cut English composition ends with.
const CUT_MARK := " …"
## The glance's facet and budget, in UTF-8 bytes: who I see.
const GLANCE_FACET := "observed"
const GLANCE_BYTES := 160


## The entries of every block an entity record (`MineWorldObservation.entity(id)`) carries, merged in
## rank order: `(rank, attribute)`, an absent rank last.
static func blocks(entity: Dictionary) -> Array:
	return merged(_components(entity))


## The reader words per offered locale an entity record's blocks carry (the first readable block's).
static func words(entity: Dictionary) -> Dictionary:
	return merged_words(_components(entity))


## How a reader wanting `locale` (the player's language) reads an entity record.
static func reading_of(entity: Dictionary, locale: String) -> Dictionary:
	return reading(locale, words(entity))


static func _components(entity: Dictionary) -> Array:
	var components: Variant = entity.get("components", [])
	return components if typeof(components) == TYPE_ARRAY else []


## The block a component record carries, or null.
static func _candidate(record: Variant) -> Variant:
	if typeof(record) != TYPE_DICTIONARY:
		return null
	var payload: Variant = record.get("payload")
	if typeof(payload) != TYPE_DICTIONARY:
		return null
	return payload if String(record.get("component_type", "")) == TRAITS else payload.get(OWNER_MEMBER)


## The same from a list of component records `{component_type, payload}`.
static func merged(components: Array) -> Array:
	var entries: Array = []
	for record in components:
		var block: Variant = _candidate(record)
		if _readable(block):
			entries.append_array(block["entries"].duplicate())
	entries.sort_custom(_before)
	return entries


## The reader words of the first readable block among `components`; `{}` when none offers a locale.
static func merged_words(components: Array) -> Dictionary:
	for record in components:
		var block: Variant = _candidate(record)
		if _readable(block):
			return block.get("words", {})
	return {}


## Whether `block` is a block this module reads.
static func _readable(block: Variant) -> bool:
	if typeof(block) != TYPE_DICTIONARY:
		return false
	var schema: Variant = block.get("schema")
	if (typeof(schema) != TYPE_INT and typeof(schema) != TYPE_FLOAT) or int(schema) != SCHEMA:
		return false
	if not _words_readable(block.get("words", {})):
		return false
	var entries: Variant = block.get("entries")
	if typeof(entries) != TYPE_ARRAY:
		return false
	for entry in entries:
		if typeof(entry) != TYPE_DICTIONARY or typeof(entry.get("attribute")) != TYPE_STRING \
				or typeof(entry.get("label")) != TYPE_STRING:
			return false
		for key in entry:
			if not FIELDS.has(key):
				return false
		for member in ["labels", "texts"]:
			if not _strings_by_locale(entry.get(member, {})):
				return false
	return true


static func _strings_by_locale(value: Variant) -> bool:
	if typeof(value) != TYPE_DICTIONARY:
		return false
	for locale in value:
		if typeof(value[locale]) != TYPE_STRING:
			return false
	return true


static func _words_readable(value: Variant) -> bool:
	if typeof(value) != TYPE_DICTIONARY:
		return false
	for locale in value:
		var words: Variant = value[locale]
		if typeof(words) != TYPE_DICTIONARY or words.size() != ENGLISH_WORDS.size():
			return false
		for key in ENGLISH_WORDS:
			if typeof(words.get(key)) != TYPE_STRING:
				return false
	return true


## How a reader wanting `locale` reads blocks offering `offered` (locale → words): `locale` when it is
## offered; else the one offered locale with the same language, when exactly one has it; else English.
## `zh-Hans` is read as `zh_Hans`.
static func reading(locale: String, offered: Dictionary) -> Dictionary:
	var wanted := locale.replace("-", "_")
	if wanted == "" or wanted == "en":
		return ENGLISH
	if offered.has(wanted):
		return {"locale": wanted, "words": offered[wanted]}
	var language := wanted.get_slice("_", 0)
	var same: Array = []
	for candidate in offered:
		if String(candidate).get_slice("_", 0) == language:
			same.append(candidate)
	if same.size() == 1:
		return {"locale": same[0], "words": offered[same[0]]}
	return ENGLISH


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


## `entry`'s text in `reader`: its text in the reading's locale, else its English text; "" when none.
static func _text(entry: Dictionary, reader: Dictionary) -> String:
	var locale := String(reader["locale"])
	if locale != "":
		var texts: Dictionary = entry.get("texts", {})
		if texts.has(locale):
			return String(texts[locale])
	var text: Variant = entry.get("text")
	return text if typeof(text) == TYPE_STRING else ""


## The non-empty texts of `entries` carrying `facet` (every entry when `facet` is empty), in order.
static func _texts(entries: Array, facet: String, reader: Dictionary) -> PackedStringArray:
	var out := PackedStringArray()
	for entry in entries:
		if facet != "":
			var facets: Variant = entry.get("facets", [])
			if typeof(facets) != TYPE_ARRAY or not facets.has(facet):
				continue
		var text := _text(entry, reader)
		if text != "":
			out.append(text)
	return out


## Select by `facet` ("" for every entry), join with the reading's `join`, and cut at `budget` UTF-8
## bytes (-1: no cut) at an entry boundary with its `more`; a first text longer than the budget alone
## is cut at a character boundary and ends with "…".
static func compose(entries: Array, facet: String, budget: int, reader: Dictionary = ENGLISH) -> String:
	var separator := String(reader["words"]["join"])
	var mark := String(reader["words"]["more"])
	var texts := _texts(entries, facet, reader)
	var joined := separator.join(texts)
	if budget < 0 or _bytes(joined) <= budget:
		return joined
	var length := 0
	var kept := 0
	for index in texts.size():
		var next := length + _bytes(texts[index]) + (0 if index == 0 else _bytes(separator))
		if next + _bytes(mark) > budget:
			break
		length = next
		kept = index + 1
	if kept > 0:
		return separator.join(texts.slice(0, kept)) + mark
	var first: String = texts[0] if texts.size() > 0 else ""
	var room := budget - _bytes("…")
	var cut := ""
	for character in first:
		if _bytes(cut + character) > room:
			break
		cut += character
	return cut + "…"


## Who I see: the glance of `entity` in the player's `locale`, as the view and every client show it.
static func glance(entity: Dictionary, locale: String = "en") -> String:
	return compose(blocks(entity), GLANCE_FACET, GLANCE_BYTES, reading_of(entity, locale))


## The inspect list: every text in `reader`, one line each.
static func inspect(entries: Array, reader: Dictionary = ENGLISH) -> PackedStringArray:
	return _texts(entries, "", reader)


## A typed value as the own sheet shows it when it renders no text, with `words`' flag words.
static func _typed(value: Variant, words: Dictionary) -> String:
	if typeof(value) != TYPE_DICTIONARY:
		return ""
	for kind in ["category", "level", "quantity", "text"]:
		if value.has(kind):
			return String(value[kind])
	if value.has("flag"):
		return String(words["yes"]) if bool(value["flag"]) else String(words["no"])
	if value.has("instant"):
		return "%d" % int(value["instant"])
	if value.has("set") and typeof(value["set"]) == TYPE_ARRAY:
		var members := PackedStringArray()
		for member in value["set"]:
			members.append(String(member))
		members.sort()
		return String(words["join"]).join(members)
	return ""


## The own sheet in `reader`: `label<pair>text`, or `label<pair><typed value>` when there is no text; an
## entry that shows nothing is left out. An entry the holder's view carries in the reading's locale is
## shown in it; any other in English, whole.
static func sheet(entries: Array, reader: Dictionary = ENGLISH) -> PackedStringArray:
	var out := PackedStringArray()
	var locale := String(reader["locale"])
	for entry in entries:
		var labels: Dictionary = entry.get("labels", {})
		var localised := locale != "" and labels.has(locale)
		var shown_with: Dictionary = reader if localised else ENGLISH
		var label := String(labels[locale]) if localised else String(entry["label"])
		var text := _text(entry, shown_with)
		var shown: String = text if text != "" \
				else _typed(entry.get("value"), shown_with["words"])
		if shown != "":
			out.append("%s%s%s" % [label, String(shown_with["words"]["pair"]), shown])
	return out
