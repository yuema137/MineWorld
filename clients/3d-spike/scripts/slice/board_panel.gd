## The notice board of the connected slice, opened with N (S31-e §5.9; `clients/shared/TRADE.md` §5.5).
##
## The current notices of the topics you follow or publish, exactly as the `notice-board` record lists them,
## worded by the shared trade module, in the HUD's light-on-dark panel. The record is the authority on what
## is current: nothing here reads a clock against a notice. While it is open it holds the game through the
## slice's one input gate, like the menus (D-16d-9).
##
## The badge counts the notices the record holds that were not there when the board was last opened; that
## set is presentation memory, dropped when the world is another one.
class_name SliceBoardPanel
extends CanvasLayer

signal opened
signal closed
## The board's rows changed, for the probe.
signal shown(rows: Array, badge: int)

const PANEL_BG := Color(0, 0, 0, 0.45)
const ROW_PX := 15
const TITLE_PX := 18

var link: SliceLink
var trade: MineWorldTrade
var rows: Array = []
var _seen := {}
var _instance := ""
var _signature := ""
var _box: PanelContainer
var _column: VBoxContainer


func _ready() -> void:
	layer = 21
	_box = PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = PANEL_BG
	style.set_corner_radius_all(6)
	style.set_content_margin_all(12)
	_box.add_theme_stylebox_override("panel", style)
	_box.position = Vector2(24, 160)
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 4)
	_box.add_child(_column)
	add_child(_box)
	_box.visible = false


func is_open() -> bool:
	return _box.visible


## N, as a player presses it. Held, like every gameplay key, while the settings menu is open.
func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo \
			and (event as InputEventKey).physical_keycode == KEY_N and link != null and link.client != null:
		toggle()
		get_viewport().set_input_as_handled()


func toggle() -> void:
	if is_open():
		close()
		return
	_box.visible = true
	Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
	opened.emit()
	_signature = ""
	refresh(link.client.latest)


func close() -> void:
	if not is_open():
		return
	_box.visible = false
	closed.emit()
	_signature = ""
	refresh(link.client.latest)


## How many notices the record holds that were not there when the board was last opened.
func badge() -> int:
	var fresh := 0
	for row in rows:
		if not _seen.has(row["id"]):
			fresh += 1
	return fresh


## Rebuilds the rows from a frame; every notice of the record is listed.
func refresh(obs: MineWorldObservation) -> void:
	if obs == null:
		return
	var instance := link.client.world_instance()
	if instance != _instance:
		_instance = instance
		_seen.clear()
	rows = trade.board_rows(obs, MineWorldMoney.currency_of(obs),
		func(at: int) -> String: return MineWorldClockFormat.day_time(at))
	if is_open():
		for row in rows:
			_seen[row["id"]] = true
	var signature := JSON.stringify([is_open(), rows, badge()])
	if signature == _signature:
		return
	_signature = signature
	_draw(obs)
	shown.emit(rows, badge())


func _draw(obs: MineWorldObservation) -> void:
	for child in _column.get_children():
		_column.remove_child(child)
		child.queue_free()
	if not is_open():
		return
	_column.add_child(_label(MineWorldText.text("panel.board"), TITLE_PX))
	if not MineWorldTrade.has_board(obs) or rows.is_empty():
		_column.add_child(_label(MineWorldText.text("board.empty"), ROW_PX))
	for row in rows:
		_column.add_child(_label(row["title"], ROW_PX))
		for line in row["lines"]:
			_column.add_child(_label("    " + line, ROW_PX))


func _label(said: String, size: int) -> Label:
	var l := Label.new()
	l.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
	l.text = said
	ControlsHud._style(l, size)
	return l
