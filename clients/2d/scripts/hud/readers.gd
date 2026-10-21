extends RefCounted

## Readers of what the world disclosed to this observer about itself, and the names it was told
## (step-13 §15.4 D-b-7, D-b-8).
##
## A reader knows the *shape* of a component the default packs disclose — the way a client knows
## wording — and nothing about a rule: no arithmetic beyond formatting, no condition on what the
## values mean. Each panel exists only while its component is in the newest observation (`I-7`). An
## own component with no reader is shown raw in "other", never dropped (`I-3`). Shapes were pinned
## from a real frame (§15.12 E-3).

const Words := preload("res://scripts/hud/words.gd")

## The panels that show what I am told about my needs, my sleep and my day (S35 N-e): their rows come from
## the shared reader, which holds no wording and no threshold (`MineWorldNeeds`).
const NEEDS_PANELS := ["needs", "asleep", "agenda"]

## Components shown elsewhere or not at all: names label the figures, participation is the marker
## over each participant, passages are the drawn doorways, and a look chooses how a figure is drawn
## (`ARC-92`), never text.
const SHOWN_ELSEWHERE := ["display-name", "participation", "passages", "look"]
## The component a place discloses the world's item kinds in: `{kinds: [{item, category, name}]}` (`item`
## pack, `ARC-37` note). Items are never perceived entities, so this is where an item's name comes from.
const CATALOGUE := "item-catalogue"

## id → the last display name the world disclosed for it in this instance (presentation memory,
## like the layout cache: dropped when the instance changes).
var _names: Dictionary = {}
var _instance := ""
var _latest: MineWorldObservation = null
## The layout learned (`town.gd`): the tags of places seen before, for naming a place not in view.
var town


## A different world: forget every name.
func reset(instance: String) -> void:
	if instance != _instance:
		_instance = instance
		_names.clear()


## Remembers every name the observation discloses: people's and places' display names, and the item
## kinds' names in the catalogue of the place in view.
func learn(observation: MineWorldObservation) -> void:
	_latest = observation
	var kinds := catalogue(observation)
	for id in kinds:
		_names[id] = kinds[id]
	var loose := loose_objects(observation)
	for id in loose:
		_names[id] = loose[id]
	for id in observation.ids():
		var name := observation.display_name(id)
		if name != "":
			_names[id] = name


## What a person is called: the name disclosed now, else the last one disclosed, else the id.
func person(id: String) -> String:
	var name := _known_name(id)
	return name if name != "" else id


## What a thing is called: its disclosed name (an item kind's from the place's catalogue, a loose
## object's tags from the place's listing, as a place is shown) if any, else "item <id>" for a thing the
## world names nowhere (QS13b-3).
func thing(id: String) -> String:
	var name := _known_name(id)
	return name if name != "" else Words.text("ui.item", {"id": id})


## A place: its tags as the world lists them now, else as they were last seen (where one stood, or on a
## doorway's sign), else "place <id>" for a place the world has not named to this observer.
func place(id: String) -> String:
	if _latest != null:
		var tags: Variant = _latest.entity(id).get("tags", [])
		if typeof(tags) == TYPE_ARRAY and not tags.is_empty():
			return ", ".join(PackedStringArray(tags))
	if town != null and town.tags.has(id) and not town.tags[id].is_empty():
		return ", ".join(town.tags[id])
	return Words.text("ui.place", {"id": id}) if id != "" else ""


func _known_name(id: String) -> String:
	if _latest != null:
		var now := _latest.display_name(id)
		if now != "":
			return now
		var kinds := catalogue(_latest)
		if kinds.has(id):
			return kinds[id]
		var loose := loose_objects(_latest)
		if loose.has(id):
			return loose[id]
	return String(_names.get(id, ""))


## The loose objects lying in the place in view, id → their tags joined (bodies' `loose-objects`
## listing: `[{object, shape, at, tags}]`); an object without tags is left out. A loose object is not
## a kind, so no catalogue names it.
static func loose_objects(observation: MineWorldObservation) -> Dictionary:
	var out := {}
	var listing: Variant = observation.component_value(observation.place(), "loose-objects")
	if typeof(listing) != TYPE_ARRAY:
		return out
	for lying in listing:
		if typeof(lying) != TYPE_DICTIONARY or typeof(lying.get("tags")) != TYPE_ARRAY:
			continue
		var tags := PackedStringArray()
		for tag in lying["tags"]:
			if typeof(tag) == TYPE_STRING:
				tags.append(tag)
		if not tags.is_empty():
			out[_ref(lying.get("object"))] = ", ".join(tags)
	return out


## The item kinds the place in view names: id → name, from its `item-catalogue` record; empty when the
## place discloses none.
static func catalogue(observation: MineWorldObservation) -> Dictionary:
	var out := {}
	var disclosed: Variant = observation.component_value(observation.place(), CATALOGUE)
	if typeof(disclosed) != TYPE_DICTIONARY:
		return out
	var kinds: Variant = (disclosed as Dictionary).get("kinds")
	if typeof(kinds) != TYPE_ARRAY:
		return out
	for kind in kinds:
		if typeof(kind) == TYPE_DICTIONARY and typeof(kind.get("name")) == TYPE_STRING and kind["name"] != "":
			out[_ref(kind.get("item"))] = String(kind["name"])
	return out


## Every panel this observation supports, in the order the components arrived: the observer's own
## components, then the current place's shop listing. Each is `{component, title, rows, payload}`.
func panels(observation: MineWorldObservation) -> Array:
	var out: Array = []
	var me := observation.observer()
	var records: Variant = observation.entity(me).get("components", [])
	if typeof(records) == TYPE_ARRAY:
		for record in records:
			if typeof(record) != TYPE_DICTIONARY:
				continue
			var kind := String(record.get("component_type", ""))
			if kind == "" or SHOWN_ELSEWHERE.has(kind):
				continue
			out.append(_panel(kind, record.get("payload"), observation))
	var shop: Variant = observation.component_value(observation.place(), "shop")
	if shop != null:
		out.append(_panel("shop", shop, observation))
	return out


## One component as a panel: its reader's rows, or, when it has no reader or the payload is not the
## shape the reader knows, the payload read by its structure (`structured`: field by field, every
## reference by its name, never JSON). `bars` (needs: one level per row, -1 for none) and `button`
## (asleep: true, a "Wake up…" button) are the panel's extras; `_box` draws them.
func _panel(kind: String, payload: Variant, observation: MineWorldObservation) -> Dictionary:
	var rows: Variant = _rows(kind, payload, observation)
	var known := rows != null
	if not known:
		rows = structured(payload)
	var title_key := ("panel." + kind) if known else "panel.other"
	var title := Words.text(title_key)
	if not known:
		title = "%s  ·  %s" % [title, kind]
	var panel := {"component": kind, "title": title, "rows": rows, "payload": payload, "known": known}
	if known and kind == "needs":
		panel["bars"] = MineWorldNeeds.own_needs(observation).map(func(e: Dictionary) -> int: return e["level"])
	if known and kind == "asleep":
		panel["button"] = true
	return panel


## The rows a known component reads as, or null.
func _rows(kind: String, payload: Variant, observation: MineWorldObservation) -> Variant:
	if kind == "needs" and typeof(payload) == TYPE_DICTIONARY:
		return _list(MineWorldNeeds.own_needs(observation), func(e: Dictionary) -> String:
			return Words.text("ui.row.need", {"need": Words.word(e["need"]), "word": Words.word(e["word"])}))
	if kind == "asleep" and typeof(payload) == TYPE_DICTIONARY:
		var asleep := MineWorldNeeds.asleep_of(observation, observation.observer())
		if asleep.is_empty():
			return null
		if asleep["alarm"] == null:
			return PackedStringArray([Words.text("ui.row.asleep.no-alarm", {"since": Words.clock(asleep["since"])})])
		return PackedStringArray([Words.text("ui.row.asleep", {"since": Words.clock(asleep["since"]),
			"alarm": Words.clock(asleep["alarm"])})])
	# The ways out of here (`ways`: `[{place, doors, length}]`, length in millimetres), nearest first as disclosed.
	if kind == "ways":
		return _list(payload, func(w: Dictionary) -> String:
			return Words.text("ui.row.way", {"place": place(_ref(w.get("place"))),
				"metres": _metres(w.get("length", 0)), "doors": int(w.get("doors", 0))}))
	if typeof(payload) != TYPE_DICTIONARY:
		return null
	# My own attributes (S30-f §5.9): the own sheet, `label: text`, from the server's phrases.
	if kind == "traits":
		var sheet := MineWorldAttributes.sheet(MineWorldAttributes.merged([{"component_type": kind,
			"payload": payload}]))
		return sheet if not sheet.is_empty() else null
	match kind:
		"wallet":
			return PackedStringArray([Words.money(payload.get("balance", 0))]) if payload.has("balance") else null
		"wealth":
			if not payload.has("amount"):
				return null
			var rows := PackedStringArray([Words.text("ui.row.wealth", {"amount": Words.money(payload["amount"])})])
			if int(payload.get("unvalued", 0)) > 0:
				rows.append(Words.text("ui.row.wealth.unvalued", {"count": int(payload["unvalued"])}))
			return rows
		"currency":
			if typeof(payload.get("name")) != TYPE_STRING:
				return null
			return PackedStringArray([Words.text("ui.row.currency", {"name": payload["name"],
				"symbol": str(payload.get("symbol", ""))})])
		"sleep-terms":
			if not payload.has("shortest_minutes") or not payload.has("longest_minutes"):
				return null
			return PackedStringArray([Words.text("ui.row.sleep-terms", {
				"shortest": int(payload["shortest_minutes"]), "longest": int(payload["longest_minutes"])})])
		"holdings":
			return _list(payload.get("held"), func(h: Dictionary) -> String:
				return Words.text("ui.row.holding", {"item": thing(_ref(h.get("item"))), "count": int(h.get("count", 0))}))
		"shop":
			return _list(payload.get("listed"), func(l: Dictionary) -> String:
				return Words.text("ui.row.shop", {"item": thing(_ref(l.get("item"))),
					"price": Words.money(l.get("price", 0)), "stock": int(l.get("in_stock", 0))}))
		"conversation-history":
			return _list(payload.get("heard"), func(h: Dictionary) -> String:
				return Words.text("ui.row.heard", {"speaker": person(_ref(h.get("speaker"))),
					"utterance": str(h.get("utterance", ""))}))
		"acquaintances":
			return _list(payload.get("known"), func(k: Dictionary) -> String:
				return person(_ref(k.get("counterpart"))))
		"invitations":
			return _list(payload.get("pending"), func(i: Dictionary) -> String:
				return Words.text("ui.row.invitation", {"inviter": person(_ref(i.get("from"))),
					"kind": str(i.get("kind", ""))}))
		"agenda":
			var rows := PackedStringArray([Words.text("ui.row.agenda", {"label": str(payload.get("label", "")),
				"place": place(_ref(payload.get("place"))), "from": Words.clock(int(payload.get("since", 0))),
				"until": Words.clock(int(payload.get("until", 0)))})])
			var next := MineWorldNeeds.next_part(observation)
			if not next.is_empty():
				rows.append(Words.text("ui.row.agenda-next.punctual" if next["punctual"] else "ui.row.agenda-next",
					{"label": next["label"], "place": place(next["place"]), "from": Words.clock(next["from"])}))
			return rows
		"employment":
			var job: Variant = payload.get("job")
			if typeof(job) != TYPE_DICTIONARY:
				return null
			var rows := PackedStringArray([Words.text("ui.row.employment", {
				"employer": thing(_ref(job.get("employer"))), "place": place(_ref(job.get("workplace"))),
				"from": _plain(job.get("from")), "until": _plain(job.get("until")),
				"wage": Words.money(job.get("wage", 0))})])
			if payload.get("shift") != null:
				rows.append(Words.text("ui.row.on-shift"))
			# The attributes this owner discloses within its own record, by the same sheet.
			rows.append_array(MineWorldAttributes.sheet(MineWorldAttributes.merged([{"component_type": kind,
				"payload": payload}])))
			return rows
	return null


## Each entry of a listing through `row`, or null when the listing is not an array of objects.
static func _list(listing: Variant, row: Callable) -> Variant:
	if typeof(listing) != TYPE_ARRAY:
		return null
	var rows := PackedStringArray()
	for entry in listing:
		if typeof(entry) != TYPE_DICTIONARY:
			return null
		rows.append(row.call(entry))
	return rows


## An identity as a string, from an identity string or a typed reference `{entity, entity_type}`.
static func _ref(value: Variant) -> String:
	if typeof(value) == TYPE_DICTIONARY:
		return str(value.get("entity", ""))
	if typeof(value) == TYPE_FLOAT and value == floorf(value):
		return str(int(value))
	return "" if value == null else str(value)


## A value as plain text: whole numbers without a decimal point, anything structured as JSON.
static func _plain(value: Variant) -> String:
	if typeof(value) == TYPE_DICTIONARY or typeof(value) == TYPE_ARRAY:
		return JSON.stringify(_whole(value))
	if typeof(value) == TYPE_FLOAT and value == floorf(value):
		return str(int(value))
	return str(value)


## Millimetres as whole metres, for a row (formatting only).
static func _metres(millimetres: Variant) -> int:
	return roundi(float(millimetres) / 1000.0)


## A payload no reader knows, as rows read by its structure, never as JSON: a record's fields one row
## each (`Field: value`), a listing one row per entry. Field names are `word.<field>` or the field made
## readable; a typed reference (`{entity, entity_type}`) reads as its name (`named`); an empty listing
## or a null field says nothing. A record's nested record or listing of records reads inline.
func structured(payload: Variant) -> PackedStringArray:
	var rows := PackedStringArray()
	if typeof(payload) == TYPE_ARRAY:
		for entry in payload:
			rows.append(_inline(entry))
	elif typeof(payload) == TYPE_DICTIONARY and not _is_reference(payload):
		for key in payload:
			var value: Variant = payload[key]
			if value == null or (typeof(value) == TYPE_ARRAY and value.is_empty()):
				continue
			rows.append("%s: %s" % [Words.word(str(key)).capitalize(), _inline(value)])
	else:
		rows.append(_inline(payload))
	return rows


## One value as text on one line: a reference by its name, a record as `field value` pairs, a listing
## joined, a whole number without its decimal point.
func _inline(value: Variant) -> String:
	match typeof(value):
		TYPE_DICTIONARY:
			if _is_reference(value):
				return named(value)
			var parts := PackedStringArray()
			for key in value:
				if value[key] != null:
					parts.append("%s %s" % [Words.word(str(key)), _inline(value[key])])
			return "  ·  ".join(parts)
		TYPE_ARRAY:
			return ", ".join(PackedStringArray(value.map(func(entry: Variant) -> String: return _inline(entry))))
		TYPE_BOOL:
			return Words.word("yes" if value else "no")
	return _plain(value)


static func _is_reference(value: Variant) -> bool:
	return typeof(value) == TYPE_DICTIONARY and value.size() == 2 and value.has("entity") \
		and value.has("entity_type")


## A typed reference by what its type is called: a place by its tags, a person by their name, any other
## thing by its disclosed name ("item <id>" when the world names it nowhere).
func named(reference: Dictionary) -> String:
	var id := _ref(reference)
	match str(reference.get("entity_type", "")):
		"place":
			return place(id)
		"person":
			return person(id)
	return thing(id)


## A parsed JSON value shown as it was written: Godot reads every number as a double, so a whole one
## is shown without the ".0" the wire never had.
static func _whole(value: Variant) -> Variant:
	match typeof(value):
		TYPE_FLOAT:
			return int(value) if is_finite(value) and value == floorf(value) and absf(value) < 9.0e15 else value
		TYPE_DICTIONARY:
			var copied := {}
			for key in value:
				copied[key] = _whole(value[key])
			return copied
		TYPE_ARRAY:
			return value.map(func(entry: Variant) -> Variant: return _whole(entry))
	return value
