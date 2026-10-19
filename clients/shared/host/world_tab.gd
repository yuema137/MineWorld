## The settings menu's World tab (`clients/shared/HOST.md` §4, `ARC-267`): the world clock's state for
## everybody, and Pause/Resume and the day length for the host. It shows only what the server said
## (`INV-WC-1`); without the admin token its rows are read-only.
class_name MineWorldWorldTab
extends VBoxContainer

var _menu: MineWorldSettingsMenu
var _host: MineWorldHostClock
var _state: Label
var _length: Label
var _toggle: Button
var _lengths: Array[Button] = []
var _note: Label
var _message: Label
var _titles: Array[Label] = []
var _title_keys: Array[String] = []


func _init(menu: MineWorldSettingsMenu, host: MineWorldHostClock) -> void:
	name = "World"
	_menu = menu
	_host = host
	add_theme_constant_override("separation", 10)
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 18)
	add_child(grid)
	_state = _row(grid, "ui.world.clock")
	_length = _row(grid, "ui.world.day_length")
	var buttons := HBoxContainer.new()
	add_child(buttons)
	_toggle = Button.new()
	_toggle.pressed.connect(func() -> void: _host.set_paused(not _host.paused))
	buttons.add_child(_toggle)
	for scale in MineWorldHostClock.DAY_SCALES:
		var button := Button.new()
		button.pressed.connect(_host.set_scale.bind(scale))
		buttons.add_child(button)
		_lengths.append(button)
	_note = Label.new()
	add_child(_note)
	_message = Label.new()
	add_child(_message)
	_menu.rendered.connect(render)
	_host.changed.connect(render)
	render()


func _row(grid: GridContainer, key: String) -> Label:
	var title := Label.new()
	grid.add_child(title)
	_titles.append(title)
	_title_keys.append(key)
	var value := Label.new()
	grid.add_child(value)
	return value


## Every text again from the catalogs, and the controls from the last frame's state.
func render() -> void:
	for k in _titles.size():
		_titles[k].text = MineWorldText.text(_title_keys[k])
	_state.text = MineWorldText.text("ui.world.state.paused" if _host.paused else "ui.world.state.running")
	var length_key := "ui.world.day_length." + _host.day_length()
	_length.text = MineWorldText.text(length_key, {"scale": _host.time_scale})
	var host := _host.has_token()
	_toggle.visible = host
	_toggle.text = MineWorldText.text("ui.world.resume" if _host.paused else "ui.world.pause")
	_toggle.disabled = _host.is_pending()
	for k in _lengths.size():
		var scale: int = MineWorldHostClock.DAY_SCALES[k]
		var key: String = "ui.world.day_length." + ["4h", "2h", "1h"][k]
		_lengths[k].visible = host
		_lengths[k].text = MineWorldText.text(key)
		_lengths[k].disabled = _host.is_pending() or _host.time_scale == scale
	_note.visible = not host
	_note.text = MineWorldText.text("ui.world.read_only")
	_message.text = MineWorldText.text(_host.message) if _host.message != "" else ""
