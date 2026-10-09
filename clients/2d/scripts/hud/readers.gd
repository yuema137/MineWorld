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

## Components shown elsewhere or not at all: names label the figures, participation is the marker
## over each participant, passages are the drawn doorways.
const SHOWN_ELSEWHERE := ["display-name", "participation", "passages"]

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


## Remembers every name the observation discloses.
func learn(observation: MineWorldObservation) -> void:
	_latest = observation
	for id in observation.ids():
		var name := observation.display_name(id)
		if name != "":
			_names[id] = name


## What a person is called: the name disclosed now, else the last one disclosed, else the id.
func person(id: String) -> String:
	var name := _known_name(id)
	return name if name != "" else id


## What a thing is called: its disclosed name if any, else "item <id>" (QS13b-3: item kinds carry no
## name until R-PK-2's catalogue is disclosed; this is the one function that will read it).
func thing(id: String) -> String:
	var name := _known_name(id)
	return name if name != "" else Words.text("ui.item", {"id": id})


## A place: its tags as the world lists them now, else as they were last seen, else its id.
func place(id: String) -> String:
	if _latest != null:
		var tags: Variant = _latest.entity(id).get("tags", [])
		if typeof(tags) == TYPE_ARRAY and not tags.is_empty():
			return ", ".join(PackedStringArray(tags))
	if town != null and town.tags.has(id) and not town.tags[id].is_empty():
		return ", ".join(town.tags[id])
	return id


func _known_name(id: String) -> String:
	if _latest != null:
		var now := _latest.display_name(id)
		if now != "":
			return now
	return String(_names.get(id, ""))


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
			out.append(_panel(kind, record.get("payload")))
	var shop: Variant = observation.component_value(observation.place(), "shop")
	if shop != null:
		out.append(_panel("shop", shop))
	return out


## One component as a panel: its reader's rows, or the raw JSON when it has no reader or the payload
## is not the shape the reader knows.
func _panel(kind: String, payload: Variant) -> Dictionary:
	var rows: Variant = _rows(kind, payload)
	var known := rows != null
	if not known:
		rows = PackedStringArray([JSON.stringify(_whole(payload))])
	var title_key := ("panel." + kind) if known else "panel.other"
	var title := Words.text(title_key)
	if not known:
		title = "%s  ·  %s" % [title, kind]
	return {"component": kind, "title": title, "rows": rows, "payload": payload, "known": known}


## The rows a known component reads as, or null.
func _rows(kind: String, payload: Variant) -> Variant:
	if typeof(payload) != TYPE_DICTIONARY:
		return null
	match kind:
		"wallet":
			return PackedStringArray([Words.money(payload.get("balance", 0))]) if payload.has("balance") else null
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
			return PackedStringArray([Words.text("ui.row.agenda", {"label": str(payload.get("label", "")),
				"place": place(_ref(payload.get("place"))), "from": Words.clock(int(payload.get("since", 0))),
				"until": Words.clock(int(payload.get("until", 0)))})])
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
