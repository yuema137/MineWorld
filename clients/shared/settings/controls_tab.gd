## The Controls tab of the settings menu (`clients/shared/SETTINGS.md` §7.1; S20 SET-b): mouse sensitivity and
## vertical look where the client has the look capability, and the client's key map, read-only. The tab
## edits the menu's draft only; the menu's Apply saves it and tells the client (`applied`).
##
## The key labels are the engine's (the label the keyboard layout gives each physical key, SD-SETb-11);
## the words around them are catalog text.
class_name MineWorldControlsTab
extends VBoxContainer

## The client's player is looked at with the mouse (3D). A client without a look does not pass it.
const CAP_MOUSE_LOOK := "mouse_look"

var _menu: MineWorldSettingsMenu
var _key_rows: Array[MineWorldKeyRow] = []
## Static labels: label -> catalog key, rendered again on every redraw.
var _texts := {}
var _look_rows: Array[Control] = []
var _sensitivity: OptionButton
var _invert: OptionButton
## Two cells per key row: the action's text, then its key labels.
var _map_cells: Array[Label] = []


func _init(menu: MineWorldSettingsMenu, capabilities: PackedStringArray, key_rows: Array[MineWorldKeyRow]) -> void:
	_menu = menu
	_key_rows = key_rows
	add_theme_constant_override("separation", 12)
	_build(capabilities.has(CAP_MOUSE_LOOK))
	_menu.rendered.connect(_render_draft)
	_render_texts()


## A language change re-renders every text of the tab (the menu's redraw only re-selects the draft).
func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSLATION_CHANGED and is_node_ready():
		_render_texts()


func _build(look: bool) -> void:
	var look_grid := _grid()
	add_child(look_grid)
	_look_rows.append(_text(look_grid, "ui.settings.controls.sensitivity"))
	_sensitivity = _choice(look_grid, _on_sensitivity)
	_look_rows.append(_sensitivity)
	_look_rows.append(_text(look_grid, "ui.settings.controls.invert_y"))
	_invert = _choice(look_grid, _on_invert)
	_look_rows.append(_invert)
	for node in _look_rows:
		node.visible = look
	_text(self, "ui.settings.controls.keys")
	var map_grid := _grid()
	add_child(map_grid)
	for row in _key_rows:
		var action := Label.new()
		var keys := Label.new()
		map_grid.add_child(action)
		map_grid.add_child(keys)
		_map_cells.append(action)
		_map_cells.append(keys)
	var note := _text(self, "ui.settings.controls.read_only")
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART


func _grid() -> GridContainer:
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 18)
	grid.add_theme_constant_override("v_separation", 8)
	return grid


func _text(parent: Node, key: String) -> Label:
	var label := Label.new()
	label.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
	_texts[label] = key
	parent.add_child(label)
	return label


func _choice(grid: GridContainer, on_pick: Callable) -> OptionButton:
	var option := OptionButton.new()
	option.custom_minimum_size = Vector2(260, 0)
	option.item_selected.connect(on_pick)
	grid.add_child(option)
	return option


func _on_sensitivity(index: int) -> void:
	_menu.draft.mouse_sensitivity = MineWorldSettings.MOUSE_SENSITIVITIES[index]


func _on_invert(index: int) -> void:
	_menu.draft.invert_y = index == 0


## Every text of the tab, from its key, in the language in effect (on build and on a language change).
func _render_texts() -> void:
	for label: Label in _texts:
		label.text = MineWorldText.text(_texts[label])
	_sensitivity.clear()
	for percent in MineWorldSettings.MOUSE_SENSITIVITIES:
		_sensitivity.add_item(MineWorldText.text("ui.settings.percent_value", {"percent": percent}))
	_invert.clear()
	_invert.add_item(MineWorldText.text("ui.settings.controls.invert_y.on"))
	_invert.add_item(MineWorldText.text("ui.settings.controls.invert_y.off"))
	for i in _key_rows.size():
		_map_cells[2 * i].text = MineWorldText.text(_key_rows[i].action)
		_set_key_cell(_map_cells[2 * i + 1], _key_rows[i].keys)
	_render_draft()


## The two choices show the menu's draft: on open, Apply, Revert and every redraw of the menu.
func _render_draft() -> void:
	_sensitivity.select(MineWorldSettings.MOUSE_SENSITIVITIES.find(_menu.draft.mouse_sensitivity))
	_invert.select(0 if _menu.draft.invert_y else 1)


## The key labels are engine data (SD-SETb-11): their cell is world text and never translated. An action
## with no key shows the catalog's "no key" instead, which is translated like every other text.
static func _set_key_cell(cell: Label, keys: PackedInt32Array) -> void:
	if keys.is_empty():
		cell.remove_from_group(MineWorldText.WORLD_TEXT_GROUP)
		cell.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
		cell.text = MineWorldText.text("ui.settings.controls.keys.none")
		return
	MineWorldText.mark_world_text(cell)
	var names := PackedStringArray()
	for physical in keys:
		var label := DisplayServer.keyboard_get_label_from_physical(physical)
		names.append(OS.get_keycode_string(label if label != KEY_NONE else physical))
	cell.text = ", ".join(names)
