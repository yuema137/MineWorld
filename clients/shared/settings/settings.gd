## One person's presentation settings on one machine: a typed value, never a Dictionary
## (`clients/shared/SETTINGS.md` §3). A setting never reaches the server (INV-SET-1).
##
## `read` builds a value from a `ConfigFile` and says what it could not accept; nothing invalid survives
## it. `write` puts the named sections back. The sections this value owns are `general`, `display` and
## `input`; any other section of the file belongs to someone else and is neither read nor judged here.
class_name MineWorldSettings
extends RefCounted

enum ClockFormat { AUTO, H12, H24 }
enum WindowMode { WINDOWED, BORDERLESS, FULLSCREEN }
enum VSync { OFF, ON, ADAPTIVE }

const GENERAL := "general"
const DISPLAY := "display"
const INPUT := "input"
const SECTIONS: PackedStringArray = ["general", "display", "input"]

## The keys of each owned section, as written in the file. `input` holds no key binding, ever (QSET-7).
const KEYS := {
	"general": ["language", "clock"],
	"display": ["window_mode", "window_size", "render_scale", "vsync", "max_fps"],
	"input": ["mouse_sensitivity", "invert_y"],
}

## How each enum is written in the file: readable words, so a person can edit it.
const CLOCK_WORDS: PackedStringArray = ["auto", "12h", "24h"]
const WINDOW_WORDS: PackedStringArray = ["windowed", "borderless", "fullscreen"]
const VSYNC_WORDS: PackedStringArray = ["off", "on", "adaptive"]

const RENDER_SCALES: Array[int] = [50, 67, 75, 85, 100]
## 0 is no cap; -1 is "match the display's refresh rate".
const FPS_CAPS: Array[int] = [0, 30, 60, 120, 144, 165, 240, -1]
const MIN_WINDOW := Vector2i(640, 360)
const MAX_WINDOW := Vector2i(7680, 4320)
## Percent of the accepted look rate (`player.gd` MOUSE_SENS); 100 is today's feel exactly (SD-SETb-2).
const MOUSE_SENSITIVITIES: Array[int] = [50, 75, 100, 125, 150, 200]

const DEFAULT_LANGUAGE := "en"
const DEFAULT_WINDOW := Vector2i(1600, 900)
const DEFAULT_MOUSE_SENSITIVITY := 100

var language := DEFAULT_LANGUAGE
var clock: ClockFormat = ClockFormat.AUTO
var window_mode: WindowMode = WindowMode.WINDOWED
var window_size := DEFAULT_WINDOW
var render_scale := 100
var vsync: VSync = VSync.ON
var max_fps := 0
var mouse_sensitivity := DEFAULT_MOUSE_SENSITIVITY
var invert_y := false


func copy() -> MineWorldSettings:
	var other := MineWorldSettings.new()
	other.language = language
	other.clock = clock
	other.window_mode = window_mode
	other.window_size = window_size
	other.render_scale = render_scale
	other.vsync = vsync
	other.max_fps = max_fps
	other.mouse_sensitivity = mouse_sensitivity
	other.invert_y = invert_y
	return other


## The look rate multiplier the 3D player takes (`player.gd` `set_look`), SD-SETb-4.
func look_scale() -> float:
	return mouse_sensitivity / 100.0


## Whether `other` differs from this value in one owned section.
func differs_in(other: MineWorldSettings, section: String) -> bool:
	if section == GENERAL:
		return language != other.language or clock != other.clock
	if section == DISPLAY:
		return window_mode != other.window_mode or window_size != other.window_size \
			or render_scale != other.render_scale or vsync != other.vsync or max_fps != other.max_fps
	if section == INPUT:
		return mouse_sensitivity != other.mouse_sensitivity or invert_y != other.invert_y
	return false


func equals(other: MineWorldSettings) -> bool:
	for section in SECTIONS:
		if differs_in(other, section):
			return false
	return true


## A value read from `config`. Every missing key takes its default silently; every key of the wrong
## type or out of range takes its default and adds `"<section>/<key> = <raw>: <why>"` to `problems`;
## every unknown key of an owned section adds a problem and is dropped.
static func read(config: ConfigFile, problems: PackedStringArray) -> MineWorldSettings:
	var value := MineWorldSettings.new()
	for section in SECTIONS:
		if not config.has_section(section):
			continue
		for key in config.get_section_keys(section):
			if not (KEYS[section] as Array).has(key):
				problems.append("%s/%s = %s: not a setting this module knows; dropped" % [section, key,
					var_to_str(config.get_value(section, key))])
				continue
			var raw: Variant = config.get_value(section, key)
			var why := value._take(key, raw)
			if why != "":
				problems.append("%s/%s = %s: %s; the default is used" % [section, key, var_to_str(raw), why])
	return value


## Writes the named owned sections of this value into `config`, each replacing the section whole.
func write(config: ConfigFile, sections: PackedStringArray) -> void:
	for section in sections:
		if config.has_section(section):
			config.erase_section(section)
		if section == GENERAL:
			config.set_value(GENERAL, "language", language)
			config.set_value(GENERAL, "clock", CLOCK_WORDS[clock])
		elif section == DISPLAY:
			config.set_value(DISPLAY, "window_mode", WINDOW_WORDS[window_mode])
			config.set_value(DISPLAY, "window_size", window_size)
			config.set_value(DISPLAY, "render_scale", render_scale)
			config.set_value(DISPLAY, "vsync", VSYNC_WORDS[vsync])
			config.set_value(DISPLAY, "max_fps", max_fps)
		elif section == INPUT:
			config.set_value(INPUT, "mouse_sensitivity", mouse_sensitivity)
			config.set_value(INPUT, "invert_y", invert_y)


## Takes one raw value, or says why not (and leaves the default).
func _take(key: String, raw: Variant) -> String:
	match key:
		"language":
			if typeof(raw) != TYPE_STRING or not _is_locale(String(raw)):
				return "not a locale"
			language = TranslationServer.standardize_locale(String(raw))
		"clock":
			var at := _word(raw, CLOCK_WORDS)
			if at < 0:
				return "not one of %s" % ", ".join(CLOCK_WORDS)
			clock = at as ClockFormat
		"window_mode":
			var at := _word(raw, WINDOW_WORDS)
			if at < 0:
				return "not one of %s" % ", ".join(WINDOW_WORDS)
			window_mode = at as WindowMode
		"window_size":
			if typeof(raw) != TYPE_VECTOR2I:
				return "not a size"
			var size: Vector2i = raw
			if size.x < MIN_WINDOW.x or size.y < MIN_WINDOW.y or size.x > MAX_WINDOW.x or size.y > MAX_WINDOW.y:
				return "outside %s … %s" % [MIN_WINDOW, MAX_WINDOW]
			window_size = size
		"render_scale":
			if typeof(raw) != TYPE_INT or not RENDER_SCALES.has(int(raw)):
				return "not one of %s" % str(RENDER_SCALES)
			render_scale = int(raw)
		"vsync":
			var at := _word(raw, VSYNC_WORDS)
			if at < 0:
				return "not one of %s" % ", ".join(VSYNC_WORDS)
			vsync = at as VSync
		"max_fps":
			if typeof(raw) != TYPE_INT or not FPS_CAPS.has(int(raw)):
				return "not one of %s" % str(FPS_CAPS)
			max_fps = int(raw)
		"mouse_sensitivity":
			if typeof(raw) != TYPE_INT or not MOUSE_SENSITIVITIES.has(int(raw)):
				return "not one of %s" % str(MOUSE_SENSITIVITIES)
			mouse_sensitivity = int(raw)
		"invert_y":
			if typeof(raw) != TYPE_BOOL:
				return "not true or false"
			invert_y = raw
	return ""


static func _word(raw: Variant, words: PackedStringArray) -> int:
	return words.find(String(raw)) if typeof(raw) == TYPE_STRING else -1


## A locale's shape: letters, digits and `_`/`-`, starting with a letter. Whether a catalog offers it
## is the store's question (`MineWorldSettingsStore.settle_language`).
static func _is_locale(text: String) -> bool:
	if text.is_empty() or text.length() > 32:
		return false
	var head := text.unicode_at(0)
	if not ((head >= 65 and head <= 90) or (head >= 97 and head <= 122)):
		return false
	for i in text.length():
		var c := text.unicode_at(i)
		var ok := (c >= 48 and c <= 57) or (c >= 65 and c <= 90) or (c >= 97 and c <= 122) or c == 95 or c == 45
		if not ok:
			return false
	return true
