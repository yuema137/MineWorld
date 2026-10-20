extends CanvasLayer

## The notice board, toggled by N (S31-e §5.9; `clients/shared/TRADE.md` §5.5): the current notices of the
## topics I follow or publish, exactly as the `notice-board` record lists them, worded by the shared trade
## module. The record is the authority on what is current: nothing here reads a clock against a notice.
##
## The badge counts the notices in the record that were not there when the board was last opened; that set
## is presentation memory, dropped when the world is another one.

const Words := preload("res://scripts/hud/words.gd")

## The board's rows changed (on opening, and when a frame changes them while open), for the harness.
signal shown(rows: Array, badge: int)

var trade: MineWorldTrade
var rows: Array = []
var _seen := {}
var _instance := ""
var _latest: MineWorldObservation = null
var _signature := ""
var _box: PanelContainer
var _column: VBoxContainer


func _ready() -> void:
	layer = 91
	_box = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.94)
	style.set_corner_radius_all(8)
	style.set_content_margin_all(12)
	_box.add_theme_stylebox_override("panel", style)
	_box.position = Vector2(20, 150)
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 4)
	_box.add_child(_column)
	add_child(_box)
	_box.visible = false


func is_shown() -> bool:
	return _box.visible


## A different world: the seen set goes.
func reset(instance: String) -> void:
	if instance != _instance:
		_instance = instance
		_seen.clear()


## N: shows or hides the board; showing it marks every notice in it as seen.
func toggle() -> void:
	_box.visible = not _box.visible
	_signature = ""
	if _latest != null:
		refresh(_latest)


## How many notices the record holds that were not there when the board was last opened.
func badge() -> int:
	var fresh := 0
	for row in rows:
		if not _seen.has(row["id"]):
			fresh += 1
	return fresh


## Rebuilds the rows from a frame; every notice of the record is listed.
func refresh(observation: MineWorldObservation) -> void:
	_latest = observation
	rows = trade.board_rows(observation, Words.currency, func(at: int) -> String: return Words.day_time(at))
	if _box.visible:
		for row in rows:
			_seen[row["id"]] = true
	var signature := JSON.stringify([_box.visible, rows, badge()])
	if signature == _signature:
		return
	_signature = signature
	_draw(observation)
	shown.emit(rows, badge())


func _draw(observation: MineWorldObservation) -> void:
	for child in _column.get_children():
		child.queue_free()
	if not _box.visible:
		return
	_column.add_child(_label(Words.text("panel.board"), 15, Color("6b4e2e")))
	if not MineWorldTrade.has_board(observation) or rows.is_empty():
		_column.add_child(_label(Words.text("board.empty"), 13, Color("7a6650")))
	for row in rows:
		_column.add_child(_label(row["title"], 13, Color("6b4e2e")))
		for line in row["lines"]:
			_column.add_child(_label("    " + line, 13, Color("3b2c1e")))


static func _label(said: String, size: int, colour: Color) -> Label:
	var label := Label.new()
	label.text = said
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size = Vector2(360, 0)
	label.add_theme_font_size_override("font_size", size)
	label.add_theme_color_override("font_color", colour)
	return label
