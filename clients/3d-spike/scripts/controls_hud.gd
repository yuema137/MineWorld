## The on-screen controls, shared by the promenade and the slice so the two
## cannot drift apart again (they did: both said "F5 camera", and on a Mac F5 is
## a media key unless Fn is held, so the operator could not find the camera
## switch at all).
##
## Three things:
##   - the controls line, top left, always visible;
##   - the current camera, under it;
##   - a centred toast naming the camera mode when it changes, which fades after
##     a couple of seconds -- the thing that tells a first-time player that a
##     key press did something.
##
## Appearance only. It reads `Player.camera_mode_changed` and decides nothing
## about the world.
class_name ControlsHud
extends CanvasLayer

const CONTROLS := "W A S D  move     Shift  jog     Space  jump     V  camera     " \
	+ "Esc  release mouse"
const TOAST_HOLD := 1.6
const TOAST_FADE := 0.6

## Display names, keyed by CameraRig.MODE_NAMES. Wording is presentation.
const MODE_TITLES := {
	"first person": "First person",
	"third person rear": "Third person — rear",
	"third person front": "Third person — front",
}

var _mode: Label
var _toast: Label
var _tween: Tween = null
var _lines := 0


static func attach(parent: Node, player: Player) -> ControlsHud:
	var h := ControlsHud.new()
	parent.add_child(h)
	h._build(player)
	return h


func _build(player: Player) -> void:
	add_child(_line(CONTROLS, Vector2(18, 14), 15))
	_mode = _line("camera: %s" % _title(player.rig.mode_name()), Vector2(18, 38), 15)
	add_child(_mode)

	_toast = Label.new()
	_toast.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_toast.set_anchors_and_offsets_preset(Control.PRESET_CENTER_BOTTOM)
	_toast.offset_top = -170
	_toast.offset_bottom = -120
	_toast.offset_left = -400
	_toast.offset_right = 400
	_style(_toast, 30)
	_toast.modulate.a = 0.0
	add_child(_toast)

	player.camera_mode_changed.connect(_on_mode)


## One more status line under the camera line, for a scene's own state (the
## slice's semantic place, its server connection). Lines stack downward.
func add_line(text: String) -> Label:
	_lines += 1
	var l := _line(text, Vector2(18, 38 + 24 * _lines), 15)
	add_child(l)
	return l


## Lines of conversation, as subtitles: bottom centre, wrapped, the newest
## CAPTION_LINES stacked, each held long enough to read -- CAPTION_BASE plus one
## second per CAPTION_CPS characters, within CAPTION_MIN..CAPTION_MAX (so a short
## line stays up beside the reply that answers it). A toast's
## 1.6 s is for "Third person"; a sentence needs longer, and a reply must not
## wipe the line it answers.
const CAPTION_LINES := 3
const CAPTION_BASE := 2.5
const CAPTION_CPS := 14.0
const CAPTION_MIN := 6.0
const CAPTION_MAX := 14.0

var _caption: Label = null
var _captions: Array = []          ## [text, seconds left]


func caption(text: String) -> void:
	if _caption == null:
		_caption = Label.new()
		_caption.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		_caption.vertical_alignment = VERTICAL_ALIGNMENT_BOTTOM
		_caption.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_caption.set_anchors_and_offsets_preset(Control.PRESET_CENTER_BOTTOM)
		_caption.grow_vertical = Control.GROW_DIRECTION_BEGIN
		_caption.offset_left = -560
		_caption.offset_right = 560
		_caption.offset_bottom = -24
		_caption.offset_top = -24
		_style(_caption, 24)
		var bg := StyleBoxFlat.new()
		bg.bg_color = Color(0, 0, 0, 0.45)
		bg.set_content_margin_all(10)
		bg.set_corner_radius_all(6)
		_caption.add_theme_stylebox_override("normal", bg)
		add_child(_caption)
	_captions.append([text, clampf(CAPTION_BASE + text.length() / CAPTION_CPS,
		CAPTION_MIN, CAPTION_MAX)])
	while _captions.size() > CAPTION_LINES:
		_captions.pop_front()
	_show_captions()


func _process(delta: float) -> void:
	if _captions.is_empty():
		return
	var before := _captions.size()
	for c in _captions:
		c[1] -= delta
	_captions = _captions.filter(func(c): return c[1] > 0.0)
	if _captions.size() != before:
		_show_captions()


func _show_captions() -> void:
	_caption.text = "\n".join(_captions.map(func(c): return c[0]))
	_caption.visible = not _captions.is_empty()


## Free-form messages from the scene (a connection state, a server answer),
## shown the same way as the camera toast.
func toast(text: String) -> void:
	_toast.text = text
	if _tween != null:
		_tween.kill()
	_toast.modulate.a = 1.0
	_tween = create_tween()
	_tween.tween_interval(TOAST_HOLD)
	_tween.tween_property(_toast, "modulate:a", 0.0, TOAST_FADE)


func _on_mode(mode_name: String) -> void:
	_mode.text = "camera: %s" % _title(mode_name)
	toast(_title(mode_name))


static func _title(mode_name: String) -> String:
	return MODE_TITLES.get(mode_name, mode_name)


func _line(txt: String, pos: Vector2, size: int) -> Label:
	var l := Label.new()
	l.text = txt
	l.position = pos
	_style(l, size)
	return l


static func _style(l: Label, size: int) -> void:
	l.add_theme_font_size_override("font_size", size)
	l.add_theme_color_override("font_color", Color(1, 1, 1, 0.88))
	l.add_theme_color_override("font_shadow_color", Color(0, 0, 0, 0.75))
	l.add_theme_constant_override("shadow_offset_y", 2)
	l.add_theme_constant_override("shadow_offset_x", 2)
