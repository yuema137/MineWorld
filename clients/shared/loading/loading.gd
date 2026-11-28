extends CanvasLayer
## Presentation-only startup UI. Kept under the root so changing scenes cannot erase it.
## Uses no imported assets: it must draw before a client pack is opened.

const NODE_NAME := "MineWorldLoading"
## Process frames without a draw after which the window counts as not drawing (`drawn`).
const UNDRAWN_FRAMES := 3
var _stage: Label
var _elapsed: Label
var _progress: ProgressBar
var _quit: Button
var _first_draw := true
var _undrawn_said := false
var _started := Time.get_ticks_msec()


static func open(tree: SceneTree) -> CanvasLayer:
	var existing := tree.root.get_node_or_null(NodePath(NODE_NAME))
	if existing != null:
		return existing as CanvasLayer
	var screen := (load("res://mineworld_loading/loading.gd") as GDScript).new() as CanvasLayer
	screen.name = NODE_NAME
	tree.root.add_child.call_deferred(screen)
	await screen.ready
	return screen


func _ready() -> void:
	layer = 100
	var background := ColorRect.new()
	background.color = Color(0.035, 0.045, 0.06)
	background.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(background)
	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	background.add_child(center)
	var column := VBoxContainer.new()
	column.custom_minimum_size.x = 480
	column.add_theme_constant_override("separation", 20)
	center.add_child(column)
	var title := Label.new()
	title.text = "MineWorld"
	title.add_theme_font_size_override("font_size", 40)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	column.add_child(title)
	_stage = Label.new()
	_stage.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_stage.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_stage.custom_minimum_size.y = 64
	column.add_child(_stage)
	_progress = ProgressBar.new()
	_progress.custom_minimum_size.y = 12
	_progress.show_percentage = false
	_progress.visible = false
	column.add_child(_progress)
	_elapsed = Label.new()
	_elapsed.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	column.add_child(_elapsed)
	_quit = Button.new()
	_quit.text = "Quit (Esc)"
	_quit.pressed.connect(func() -> void: get_tree().quit())
	column.add_child(_quit)


func _input(event: InputEvent) -> void:
	if not visible or not (event is InputEventKey or event is InputEventJoypadButton):
		return
	if event.is_action_pressed("ui_cancel"):
		get_tree().quit()
	# Gameplay menus are siblings, so a visual overlay alone does not hold their
	# keyboard shortcuts. Mouse events still reach the layer's Quit button.
	get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	_elapsed.text = "%.1f s" % ((Time.get_ticks_msec() - _started) / 1000.0)


func show_stage(words: String) -> void:
	_stage.text = words
	_progress.hide()
	print("[startup] %d ms: %s" % [Time.get_ticks_msec(), words])


func resource_progress(fraction: float) -> void:
	_progress.show()
	_progress.value = clampf(fraction, 0.0, 1.0) * 100.0


func quit_text(words: String) -> void:
	_quit.text = words


## process_frame precedes rendering. Wait for a completed draw before blocking work,
## but never wait for a renderer in headless tests. Returns whether a frame was drawn.
##
## Godot draws nothing while no window can draw: on macOS a window wholly covered by others
## (godot_window_delegate.mm: occlusionState) or minimised. frame_post_draw then never fires,
## and waiting for it stalled startup for as long as the window stayed covered (measured
## 5-227 s, 2026-11-27). The engine still runs process frames, so the wait is bounded in them:
## a draw happens in the same iteration as the process frame before it, unless the iteration
## skips drawing. A draw that takes long (a cold pipeline compile) delays the next process frame
## too, so it is still waited for in full. Undrawn, loading goes on; the next frame the window
## draws shows the current stage, which is the one the work is in.
func drawn() -> bool:
	await get_tree().process_frame
	if DisplayServer.get_name() == "headless":
		return _note_drawn(true)
	var state := {"drawn": false}
	var mark := func() -> void: state["drawn"] = true
	RenderingServer.frame_post_draw.connect(mark, CONNECT_ONE_SHOT)
	for _i in UNDRAWN_FRAMES:
		await get_tree().process_frame
		if state["drawn"]:
			break
	if not state["drawn"]:
		RenderingServer.frame_post_draw.disconnect(mark)
	return _note_drawn(state["drawn"])


func _note_drawn(was_drawn: bool) -> bool:
	if not was_drawn and not _undrawn_said:
		_undrawn_said = true
		print("[startup] %d ms: window not drawing (covered or minimised); loading continues" % Time.get_ticks_msec())
	if was_drawn and _first_draw:
		_first_draw = false
		print("[startup] %d ms: loading UI drawn" % Time.get_ticks_msec())
	return was_drawn


func finish() -> void:
	hide()
	if await drawn():
		print("[startup] %d ms: client scene visible" % Time.get_ticks_msec())
	else:
		print("[startup] %d ms: client scene ready (window not drawing)" % Time.get_ticks_msec())
	queue_free()
