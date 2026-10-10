## The in-game settings menu (`clients/shared/SETTINGS.md` §7): General (language, clock) and Display
## (window mode, size or render scale, VSync, frame cap, the monitor's rate, the measured frame rate),
## Apply / Revert / Close, and Quit. Built in code, like both clients' HUDs.
##
## Language and clock apply at once and are saved on Apply; display changes apply on Apply, followed by
## a confirmation that reverts them unless kept. Close drops what was not applied. The menu pauses
## nothing; while it is open it covers the screen with a control that takes the mouse, and the client
## stops its gameplay input. Per client only the `Theme` and the capabilities differ.
class_name MineWorldSettingsMenu
extends CanvasLayer

signal opened
signal closed
signal quit_requested

## How long a display change waits to be kept before it is reverted.
const CONFIRM_SECONDS := 15.0
const FPS_EVERY := 0.5

var store: MineWorldSettingsStore
var capabilities := PackedStringArray()
## What the controls show: the saved value plus what the player has changed and not yet applied.
var draft := MineWorldSettings.new()
## Seconds left to keep a display change; < 0 when nothing waits.
var confirm_left := -1.0

var _display_before: MineWorldSettings = null
var _root: Control
var _tabs: TabContainer
var _tab_keys: Array[String] = []
var _labels := {}             ## key -> Label whose whole text is that key
var _buttons := {}            ## key -> Button whose whole text is that key
var _language: OptionButton
var _clock: OptionButton
var _window_mode: OptionButton
var _window_size: OptionButton
var _render_scale: OptionButton
var _vsync: OptionButton
var _max_fps: OptionButton
var _size_row: Array[Control] = []
var _scale_row: Array[Control] = []
var _refresh_value: Label
var _fps_value: Label
var _status: Label
var _confirm: PanelContainer
var _confirm_text: Label
var _sizes: Array[Vector2i] = []
var _locales := PackedStringArray()
var _fps_since := 0.0
## The status line's key, kept so it renders again in a new language.
var _status_said := ""


func setup(a_store: MineWorldSettingsStore, a_capabilities: PackedStringArray, theme: Theme) -> void:
	store = a_store
	capabilities = a_capabilities
	draft = store.settings.copy()
	layer = 200
	_build(theme)
	_render()
	visible = false


func is_open() -> bool:
	return visible


func toggle() -> void:
	if visible:
		close()
	else:
		open()


func open() -> void:
	if visible:
		return
	draft = store.settings.copy()
	draft.language = MineWorldText.language()
	draft.clock = MineWorldText.clock_setting()
	visible = true
	_render()
	opened.emit()


## Closes, dropping what was not applied: the live language and clock return to the saved ones.
func close() -> void:
	if not visible:
		return
	if confirm_left >= 0.0:
		_revert_display()
	_set_text_settings(store.settings.language, store.settings.clock)
	visible = false
	closed.emit()


## Adds a tab for a later PR (SET-b, SET-c), titled by `key`'s text; the existing tabs are untouched.
func add_tab(key: String, control: Control) -> void:
	_tabs.add_child(control)
	_tab_keys.append(key)
	_render_tab_titles()


func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSLATION_CHANGED and _root != null:
		_render()


func _process(delta: float) -> void:
	if not visible:
		return
	tick(delta)


## Advances the confirmation countdown and the measured frame rate by `delta` seconds.
func tick(delta: float) -> void:
	if confirm_left >= 0.0:
		confirm_left -= delta
		if confirm_left < 0.0:
			_revert_display()
		else:
			_render_confirm()
	_fps_since += delta
	if _fps_since >= FPS_EVERY:
		_fps_since = 0.0
		_fps_value.text = MineWorldText.text("ui.settings.fps_value", {"fps": int(round(Engine.get_frames_per_second()))})


# --- actions ---------------------------------------------------------------------------------------

## Saves what changed. General settings are saved at once; a display change is applied and waits to be
## kept (`keep_display`) or reverts by itself.
func apply() -> void:
	var saved := store.settings.copy()
	if draft.differs_in(saved, MineWorldSettings.GENERAL):
		var general := saved.copy()
		general.language = draft.language
		general.clock = draft.clock
		_report(store.save(general))
	if draft.differs_in(store.settings, MineWorldSettings.DISPLAY):
		_display_before = store.settings.copy()
		MineWorldDisplay.apply(draft, get_window(), capabilities)
		confirm_left = CONFIRM_SECONDS
		_render_confirm()
	_render()


func keep_display() -> void:
	if confirm_left < 0.0:
		return
	confirm_left = -1.0
	_display_before = null
	var kept := store.settings.copy()
	_copy_display(draft, kept)
	_report(store.save(kept))
	_render()


func _revert_display() -> void:
	confirm_left = -1.0
	if _display_before != null:
		MineWorldDisplay.apply(_display_before, get_window(), capabilities)
		_copy_display(_display_before, draft)
	_display_before = null
	_render()


## Every control, and the live language and clock, back to the saved value.
func revert() -> void:
	if confirm_left >= 0.0:
		_revert_display()
	draft = store.settings.copy()
	_set_text_settings(draft.language, draft.clock)
	_render()


func _set_text_settings(language: String, clock: MineWorldSettings.ClockFormat) -> void:
	var clock_changed := clock != MineWorldText.clock_setting()
	MineWorldText.set_clock(clock)
	if language != MineWorldText.language():
		MineWorldText.set_language(language)
	elif clock_changed and is_inside_tree():
		# The clock is not a language: tell every composed label to render again.
		get_tree().root.propagate_notification(NOTIFICATION_TRANSLATION_CHANGED)


static func _copy_display(from: MineWorldSettings, into: MineWorldSettings) -> void:
	into.window_mode = from.window_mode
	into.window_size = from.window_size
	into.render_scale = from.render_scale
	into.vsync = from.vsync
	into.max_fps = from.max_fps


func _report(error: Error) -> void:
	_status_said = "ui.settings.saved" if error == OK and store.is_persistent() else "ui.settings.session_only"
	_status.text = MineWorldText.text(_status_said)


# --- choices ---------------------------------------------------------------------------------------

func _on_language(index: int) -> void:
	draft.language = _locales[index]
	_set_text_settings(draft.language, draft.clock)


func _on_clock(index: int) -> void:
	draft.clock = index as MineWorldSettings.ClockFormat
	_set_text_settings(draft.language, draft.clock)


func _on_window_mode(index: int) -> void:
	draft.window_mode = index as MineWorldSettings.WindowMode
	_render()


func _on_window_size(index: int) -> void:
	draft.window_size = _sizes[index]


func _on_render_scale(index: int) -> void:
	draft.render_scale = MineWorldSettings.RENDER_SCALES[index]


func _on_vsync(index: int) -> void:
	draft.vsync = index as MineWorldSettings.VSync


func _on_max_fps(index: int) -> void:
	draft.max_fps = MineWorldSettings.FPS_CAPS[index]


# --- building --------------------------------------------------------------------------------------

func _build(theme: Theme) -> void:
	_root = Control.new()
	_root.name = "SettingsMenu"
	_root.theme = theme
	_root.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
	_root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_root.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(_root)
	var dim := ColorRect.new()
	dim.color = Color(0, 0, 0, 0.35)
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_root.add_child(dim)
	var centre := CenterContainer.new()
	centre.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	centre.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_root.add_child(centre)
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(560, 0)
	centre.add_child(panel)
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 12)
	panel.add_child(column)
	column.add_child(_label("ui.settings.title", 22))
	_tabs = TabContainer.new()
	column.add_child(_tabs)
	_tabs.add_child(_general())
	_tabs.add_child(_display())
	_tab_keys.assign(["ui.settings.tab.general", "ui.settings.tab.display"])
	_status = Label.new()
	column.add_child(_status)
	var row := HBoxContainer.new()
	row.alignment = BoxContainer.ALIGNMENT_END
	column.add_child(row)
	row.add_child(_button("ui.settings.apply", apply))
	row.add_child(_button("ui.settings.revert", revert))
	row.add_child(_button("ui.settings.close", close))
	column.add_child(HSeparator.new())
	column.add_child(_button("ui.settings.quit", func() -> void: quit_requested.emit()))
	_confirm = _confirmation()
	centre.add_child(_confirm)


func _general() -> Control:
	var grid := _grid("General")
	_language = _choice(grid, "ui.settings.language", _on_language)
	_clock = _choice(grid, "ui.settings.clock", _on_clock)
	return grid


func _display() -> Control:
	var grid := _grid("Display")
	_window_mode = _choice(grid, "ui.settings.window_mode", _on_window_mode)
	_window_size = _choice(grid, "ui.settings.window_size", _on_window_size)
	_size_row.assign([grid.get_child(grid.get_child_count() - 2), _window_size])
	_render_scale = _choice(grid, "ui.settings.render_scale", _on_render_scale)
	_scale_row.assign([grid.get_child(grid.get_child_count() - 2), _render_scale])
	_vsync = _choice(grid, "ui.settings.vsync", _on_vsync)
	_max_fps = _choice(grid, "ui.settings.max_fps", _on_max_fps)
	grid.add_child(_label("ui.settings.refresh"))
	_refresh_value = Label.new()
	grid.add_child(_refresh_value)
	grid.add_child(_label("ui.settings.measured"))
	_fps_value = Label.new()
	grid.add_child(_fps_value)
	return grid


func _confirmation() -> PanelContainer:
	var box := PanelContainer.new()
	box.visible = false
	var column := VBoxContainer.new()
	box.add_child(column)
	_confirm_text = Label.new()
	column.add_child(_confirm_text)
	var row := HBoxContainer.new()
	row.alignment = BoxContainer.ALIGNMENT_END
	column.add_child(row)
	row.add_child(_button("ui.settings.keep", keep_display))
	row.add_child(_button("ui.settings.undo", _revert_display))
	return box


func _grid(node_name: String) -> GridContainer:
	var grid := GridContainer.new()
	grid.name = node_name
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 18)
	grid.add_theme_constant_override("v_separation", 8)
	return grid


func _choice(grid: GridContainer, key: String, on_pick: Callable) -> OptionButton:
	grid.add_child(_label(key))
	var option := OptionButton.new()
	option.custom_minimum_size = Vector2(260, 0)
	option.item_selected.connect(on_pick)
	grid.add_child(option)
	return option


func _label(key: String, size: int = 0) -> Label:
	var label := Label.new()
	if size > 0:
		label.add_theme_font_size_override("font_size", size)
	_labels[key] = label
	return label


func _button(key: String, on_press: Callable) -> Button:
	var button := Button.new()
	button.pressed.connect(on_press)
	_buttons[key] = button
	return button


# --- rendering: every text from a key, again on every language change ---------------------------------

func _render() -> void:
	for key in _labels:
		(_labels[key] as Label).text = MineWorldText.text(key)
	for key in _buttons:
		(_buttons[key] as Button).text = MineWorldText.text(key)
	_render_tab_titles()
	_render_choices()
	var rate := MineWorldDisplay.refresh_rate(get_window()) if is_inside_tree() else -1.0
	_refresh_value.text = MineWorldText.text("ui.settings.hz_value", {"rate": int(round(rate))}) if rate > 0.0 \
		else MineWorldText.text("ui.settings.unknown")
	_fps_value.text = MineWorldText.text("ui.settings.fps_value", {"fps": int(round(Engine.get_frames_per_second()))})
	_status.text = MineWorldText.text(_status_said if _status_said != "" else "ui.settings.hint")
	_render_confirm()


func _render_tab_titles() -> void:
	for i in mini(_tab_keys.size(), _tabs.get_tab_count()):
		_tabs.set_tab_title(i, MineWorldText.text(_tab_keys[i]))


func _render_choices() -> void:
	_locales.clear()
	_fill(_language, [], -1)
	for language in MineWorldText.languages():
		_locales.append(language.locale)
		_language.add_item(language.self_name)
	_language.select(_locales.find(draft.language))
	_fill(_clock, ["ui.settings.clock.auto", "ui.settings.clock.12h", "ui.settings.clock.24h"], draft.clock)
	_fill(_window_mode, ["ui.settings.window.windowed", "ui.settings.window.borderless",
		"ui.settings.window.fullscreen"], draft.window_mode)
	_window_mode.set_item_tooltip(2, MineWorldText.text("ui.settings.display.wayland_same")
		if MineWorldDisplay.fullscreen_same_as_borderless() else "")
	_sizes = MineWorldDisplay.size_presets(get_window()) if is_inside_tree() else [draft.window_size]
	if not _sizes.has(draft.window_size):
		_sizes.append(draft.window_size)
	_window_size.clear()
	for size in _sizes:
		_window_size.add_item(MineWorldText.text("ui.settings.size_value", {"width": size.x, "height": size.y}))
	_window_size.select(_sizes.find(draft.window_size))
	_render_scale.clear()
	for percent in MineWorldSettings.RENDER_SCALES:
		_render_scale.add_item(MineWorldText.text("ui.settings.percent_value", {"percent": percent}))
	_render_scale.select(MineWorldSettings.RENDER_SCALES.find(draft.render_scale))
	_fill(_vsync, ["ui.settings.vsync.off", "ui.settings.vsync.on", "ui.settings.vsync.adaptive"], draft.vsync)
	_max_fps.clear()
	for cap in MineWorldSettings.FPS_CAPS:
		_max_fps.add_item(MineWorldText.text("ui.settings.max_fps.none") if cap == 0
			else MineWorldText.text("ui.settings.max_fps.match") if cap < 0
			else MineWorldText.text("ui.settings.fps_value", {"fps": cap}))
	_max_fps.select(MineWorldSettings.FPS_CAPS.find(draft.max_fps))
	var windowed := draft.window_mode == MineWorldSettings.WindowMode.WINDOWED
	for node in _size_row:
		node.visible = windowed
	for node in _scale_row:
		node.visible = not windowed and capabilities.has(MineWorldDisplay.CAP_RENDER_SCALE)


func _fill(option: OptionButton, keys: Array, selected: int) -> void:
	option.clear()
	for key: String in keys:
		option.add_item(MineWorldText.text(key))
	if selected >= 0 and selected < option.item_count:
		option.select(selected)


func _render_confirm() -> void:
	_confirm.visible = confirm_left >= 0.0
	if _confirm.visible:
		_confirm_text.text = MineWorldText.text("ui.settings.confirm", {"seconds": int(ceil(confirm_left))})
