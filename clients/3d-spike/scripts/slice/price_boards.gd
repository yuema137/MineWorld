## Prices you can see from the street: a board above each shop's bound door, and a tag over each person who
## sells (S31-e §5.8; `clients/shared/TRADE.md` §5.5).
##
## Every line is a disclosed price, worded by the shared trade module (`MineWorldTrade.shop_board`, `tag_of`);
## nothing is computed or compared here. A board stands only above a door `SliceLayout` bound to the shop's
## place (16c R-16c-1: nothing is guessed). A board whose place is out of view keeps its last lines, dimmed,
## and every board goes when the world is another one (presentation memory, SD-S31e-10). No geometry is
## added: boards and tags are `Label3D` captions (step-31 §12).
class_name SlicePriceBoards
extends Node

## How high above a door's sill a board's caption stands.
const BOARD_HEIGHT_M := 2.9
## A seller's tag hangs under the name (`SliceFigures`' 2.05 m caption).
const TAG_HEIGHT_M := 1.88
const TAG_PX := 40
## Tags fade out between these camera distances (SD-S31e-7; presentation constants).
const TAG_FADE_FROM_M := 6.0
const TAG_FADE_TO_M := 12.0
const BOARD_PX := 44
const STALE_ALPHA := 0.55

var link: SliceLink
var trade: MineWorldTrade

## place id -> {lines, fresh, node}
var boards := {}
var _instance := ""


func observe(obs: MineWorldObservation) -> void:
	var instance := link.client.world_instance() if link.client != null else ""
	if instance != _instance:
		_instance = instance
		for place in boards:
			if boards[place]["node"] != null:
				(boards[place]["node"] as Node).queue_free()
		boards.clear()
	var currency := MineWorldMoney.currency_of(obs)
	_tags(obs, currency)
	for place in boards:
		boards[place]["fresh"] = false
	for place in MineWorldTrade.shops_in_view(obs):
		var lines := trade.shop_board(MineWorldTrade.listing_of(obs, place), currency)
		var entry: Dictionary = boards.get(place, {"node": null})
		entry["lines"] = lines
		entry["fresh"] = true
		boards[place] = entry
	for place in boards:
		_draw_board(place)


## A tag over every drawn person who asks a price; none over a person who asks nothing.
func _tags(obs: MineWorldObservation, currency: Dictionary) -> void:
	for id in link.figures:
		var figure := link.figures[id] as Node3D
		if figure == null:
			continue
		var said := trade.tag_of(obs, String(id), currency)
		var tag := figure.get_node_or_null("Tag") as Label3D
		if said == "":
			if tag != null:
				tag.queue_free()
			continue
		if tag == null:
			tag = SliceFigures.overhead_label(said, TAG_HEIGHT_M, TAG_PX)
			tag.name = "Tag"
			tag.visibility_range_end = TAG_FADE_TO_M
			tag.visibility_range_end_margin = TAG_FADE_TO_M - TAG_FADE_FROM_M
			tag.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
			figure.add_child(tag)
		tag.text = said


func _draw_board(place: String) -> void:
	var entry: Dictionary = boards[place]
	var door := link.layout.door_of(place)
	if door.is_empty():
		return
	var label: Label3D = entry["node"]
	if label == null:
		label = SliceFigures.overhead_label("", 0.0, BOARD_PX)
		label.name = "PriceBoard_" + place
		label.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
		label.no_depth_test = false
		label.visibility_range_end = 0.0
		link.world_root.add_child(label)
		label.global_position = (door[0] as Vector3) + Vector3(0.0, BOARD_HEIGHT_M, 0.0)
		entry["node"] = label
	label.text = "\n".join(entry["lines"])
	label.modulate = Color(1, 1, 1, 1.0 if entry["fresh"] else STALE_ALPHA)


## Every drawn board and tag, for the probe's HUD report: {boards: [{place, lines, fresh}], tags: [{subject,
## text}]}.
func report() -> Dictionary:
	var shown_boards: Array = []
	for place in boards:
		if boards[place]["node"] != null:
			shown_boards.append({"place": place, "lines": Array(boards[place]["lines"]), "fresh": boards[place]["fresh"]})
	var tags: Array = []
	for id in link.figures:
		var tag := (link.figures[id] as Node3D).get_node_or_null("Tag") as Label3D if link.figures[id] != null else null
		if tag != null:
			tags.append({"subject": String(id), "text": tag.text})
	return {"boards": shown_boards, "tags": tags}
