## What the things the world mentions are called, and what the observer has, read from what the
## world disclosed (step-15 §22.3 D-16d-5, PR 16d).
##
## A reader knows the *shape* of a component the default packs disclose -- as a client knows
## wording -- and nothing about a rule: no arithmetic beyond formatting, no condition on what a
## value means. Every name comes from the disclosure: a person's or a place's from `SliceLink`'s
## one naming rule, an item kind's from the `item-catalogue` its place discloses (12d SD-D10). An
## entity id is never returned as a name: what the world has not named reads as "something".
##
## The 2D client has its own reader for the same shapes (`clients/2d/scripts/hud/readers.gd`). The
## two are compared field for field through their `MENU` reports (§22.4 P-2). Money, worth and the shop
## listing are read and formatted by the shared trade module both use (S31-e, `ARC-184`).
class_name SliceThings
extends RefCounted

## The three panels this client shows (Q-16d-8), by component type: the observer's own wallet and
## holdings, and the listing of the shop the observer stands in.
const OWN_PANELS := ["wallet", "wealth", "holdings"]
const PLACE_PANELS := ["shop"]
## The component a place discloses its kinds of things in (12d SD-D10).
const CATALOGUE := "item-catalogue"

var link: SliceLink
## id -> the last name the world disclosed for it in this instance: a thing named in a result after
## its catalogue has left view is still named as it was seen (13b D-b-8)
var _names := {}
var _instance := ""


func _init(a_link: SliceLink) -> void:
	link = a_link


## Remembers every name `obs` discloses; forgets them all when the world is another one.
func learn(obs: MineWorldObservation) -> void:
	var instance := link.client.world_instance() if link.client != null else ""
	if instance != _instance:
		_instance = instance
		_names.clear()
	for entry in catalogue(obs):
		_names[entry["item"]] = entry["name"]
	for id in obs.ids():
		var shown := obs.display_name(id).strip_edges()
		if shown != "":
			_names[id] = shown


## What `id` is called: a perceived person or place by `SliceLink`'s rule, else an item kind by the
## catalogue of the place in view, else by the last name seen, else "something". Never the id.
func name_of(id: String) -> String:
	if id == "":
		return MineWorldText.text("ui.thing.unnamed")
	var obs := link.client.latest if link.client != null else null
	if obs != null and not obs.entity(id).is_empty():
		return link.shown_text(id)
	if obs != null:
		for entry in catalogue(obs):
			if entry["item"] == id:
				return entry["name"]
	if _names.has(id):
		return _names[id]
	return MineWorldText.text("ui.thing.unnamed")


## An action as a menu entry or a result reads: `action.<type>` (or `.done`) with the names it
## concerns; unworded, the type made readable followed by those names (`MineWorldText.text`).
func action_label(type: String, target: String, about: String, done := false) -> String:
	var said: Variant = action_ref(type, target, about, done)
	return MineWorldText.text(said.key, said.args) if said is MineWorldText.Ref else String(said)


## `action_label` as a catalog reference, worded whenever the message holding it is rendered, so a
## status line kept across a language change is all in the new language. The names stay the world's.
func action_ref(type: String, target: String, about: String, done := false) -> Variant:
	var args := {"target": name_of(target) if target != "" else "",
		"item": name_of(about) if about != "" else ""}
	return MineWorldText.code_ref("action", type + (".done" if done else ""), args)


## The kinds the observer's place discloses: `[{item, category, name}]`, in the catalogue's order.
func catalogue(obs: MineWorldObservation) -> Array:
	var out: Array = []
	var disclosed: Variant = obs.component_value(obs.place(), CATALOGUE)
	if typeof(disclosed) != TYPE_DICTIONARY:
		return out
	var kinds: Variant = (disclosed as Dictionary).get("kinds")
	if typeof(kinds) != TYPE_ARRAY:
		return out
	for kind in kinds:
		if typeof(kind) == TYPE_DICTIONARY and typeof(kind.get("name")) == TYPE_STRING:
			out.append({"item": ref(kind.get("item")), "category": str(kind.get("category", "")),
				"name": String(kind["name"])})
	return out


## Every panel `obs` supports, each `{component, title, rows, payload}`: the observer's wallet and
## holdings, in the order its components arrived, then the shop listing of the place in view. A panel
## exists only while its component is in the newest observation.
func panels(obs: MineWorldObservation) -> Array:
	var out: Array = []
	var records: Variant = obs.entity(obs.observer()).get("components", [])
	if typeof(records) == TYPE_ARRAY:
		for record in records:
			if typeof(record) != TYPE_DICTIONARY:
				continue
			var kind := String(record.get("component_type", ""))
			if OWN_PANELS.has(kind):
				out.append(_panel(kind, record.get("payload"), obs))
	for kind in PLACE_PANELS:
		var payload: Variant = obs.component_value(obs.place(), kind)
		if payload != null:
			out.append(_panel(kind, payload, obs))
	return out


func _panel(kind: String, payload: Variant, obs: MineWorldObservation) -> Dictionary:
	var rows: Variant = _rows(kind, payload, obs)
	return {"component": kind, "title": MineWorldText.text("panel." + kind),
		"rows": rows if rows != null else PackedStringArray(), "payload": payload}


## The rows a known component reads as, or null when the payload is not the shape this reader knows. Money,
## worth and the listing are read by the shared trade module (S31-e).
func _rows(kind: String, payload: Variant, obs: MineWorldObservation) -> Variant:
	if typeof(payload) != TYPE_DICTIONARY:
		return null
	match kind:
		"wallet":
			var held := MineWorldTrade.money_of(obs)
			return PackedStringArray([money(held["balance"])]) if not held.is_empty() else null
		"wealth":
			var worth := MineWorldTrade.wealth_of(obs)
			return PackedStringArray([money(worth["amount"])]) if not worth.is_empty() else null
		"holdings":
			return _list(payload.get("held"), func(h: Dictionary) -> String:
				return MineWorldText.text("ui.row.holding", {"item": name_of(ref(h.get("item"))),
					"count": int(h.get("count", 0))}))
		"shop":
			var listing := MineWorldTrade.listing_of(obs)
			if listing.is_empty():
				return null
			return PackedStringArray(listing["listed"].map(func(l: Dictionary) -> String:
				return MineWorldText.text("ui.row.shop", {"item": name_of(l["item"]),
					"price": money(l["price"]), "stock": l["in_stock"]})))
	return null


static func _list(listing: Variant, row: Callable) -> Variant:
	if typeof(listing) != TYPE_ARRAY:
		return null
	var rows := PackedStringArray()
	for entry in listing:
		if typeof(entry) != TYPE_DICTIONARY:
			return null
		rows.append(row.call(entry))
	return rows


## An amount of minor units in the currency the newest observation disclosed, through the shared formatter
## (S31-e §5.2, `res://mineworld_trade`): no scale is assumed here.
func money(minor: Variant) -> String:
	return MineWorldMoney.format(minor, currency())


## The currency the newest observation disclosed, or {} (no economy, or before the first frame).
func currency() -> Dictionary:
	var obs := link.client.latest if link != null and link.client != null else null
	return MineWorldMoney.currency_of(obs) if obs != null else {}


## An identity as a string, from an identity string or a typed reference `{entity, entity_type}`.
static func ref(value: Variant) -> String:
	if typeof(value) == TYPE_DICTIONARY:
		return str(value.get("entity", ""))
	if typeof(value) == TYPE_FLOAT and value == floorf(value):
		return str(int(value))
	return "" if value == null else str(value)


## The code a rejection carries: a kernel reason as a string; a system's as the contract's
## `{"system": {"code": …}}`; any other dictionary by its first key (13b `words.gd`'s rule).
static func code_of(reason: Variant) -> String:
	if typeof(reason) == TYPE_DICTIONARY and not (reason as Dictionary).is_empty():
		var system: Variant = reason.get("system")
		if typeof(system) == TYPE_DICTIONARY and typeof(system.get("code")) == TYPE_STRING:
			return String(system["code"])
		return str((reason as Dictionary).keys()[0])
	return "" if reason == null else str(reason)


## Why not, in words: `reason.<code>` as a reference worded when rendered, or "" when there is none.
static func reason_words(reason: Variant) -> Variant:
	var said := code_of(reason)
	return MineWorldText.code_ref("reason", said) if said != "" else ""
