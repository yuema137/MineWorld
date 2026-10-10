## Client UI text: the catalog layers, the languages they offer, the live switch, the readable
## fallback and the CJK font fallback (`clients/shared/SETTINGS.md` §6; `docs/DECISIONS.md` ARC-70).
##
## Static: one set of catalogs per process, registered with Godot's `TranslationServer`. Keys built
## from data are confined to `FAMILIES` and built here only (`code`). World content never passes
## through this file (INV-SET-10); logs never do either (INV-SET-6).
class_name MineWorldText
extends RefCounted

## The families whose keys are built from data: `action.<type>`, `reason.<code>`, `state.<state>`.
const FAMILIES: PackedStringArray = ["action", "reason", "state"]
## Nodes that show world content join this group; the evidence walk and translation leave them alone.
const WORLD_TEXT_GROUP := &"mineworld_world_text"
const USER_LAYER := "user://locale"
const FONT_FILE := "fonts/NotoSansSC-Regular.otf"


## A language some layer offers: its locale and how it names itself.
class Language:
	extends RefCounted
	var locale: String
	var self_name: String

	func _init(a_locale: String, a_self_name: String) -> void:
		locale = a_locale
		self_name = a_self_name


static var _registered: Array[Translation] = []
static var _merged := {}                    ## locale -> Translation, the registered ones
static var _clock: MineWorldSettings.ClockFormat = MineWorldSettings.ClockFormat.AUTO
static var _font_installed := false


## The module's own folder, absolute (it is reached through the client's symlink).
static func module_dir() -> String:
	return ProjectSettings.globalize_path((MineWorldText as Script).resource_path.get_base_dir())


## Loads the layers in order — the shared `locale/`, then `<pack_dir>/i18n/` (skipped when `pack_dir`
## is ""), then `user://locale/`, then each of `extra_dirs` — merges them per locale (a later layer's
## entry replaces an earlier one) and registers the result, replacing an earlier call's. `wording`
## false loads no layer at all. Returns what went wrong, for the caller to warn with.
static func load_layers(pack_dir: String, extra_dirs: PackedStringArray = PackedStringArray(),
		wording: bool = true) -> PackedStringArray:
	var problems := PackedStringArray()
	for old in _registered:
		TranslationServer.remove_translation(old)
	_registered.clear()
	_merged.clear()
	if not wording:
		return problems
	var layers := PackedStringArray([module_dir().path_join("locale")])
	if pack_dir != "":
		layers.append(pack_dir.path_join("i18n"))
	layers.append(ProjectSettings.globalize_path(USER_LAYER))
	layers.append_array(extra_dirs)
	for layer in layers:
		_load_layer(layer, problems)
	for locale in _merged:
		var merged: Translation = _merged[locale]
		TranslationServer.add_translation(merged)
		_registered.append(merged)
	return problems


static func _load_layer(dir: String, problems: PackedStringArray) -> void:
	if not DirAccess.dir_exists_absolute(dir):
		return
	var names := Array(DirAccess.get_files_at(dir))
	names.sort()
	for name: String in names:
		if name.get_extension() != "po":
			continue
		var path := dir.path_join(name)
		var loaded: Resource = ResourceLoader.load(path, "", ResourceLoader.CACHE_MODE_IGNORE)
		if not loaded is Translation:
			problems.append("[settings] %s is not a gettext catalog; skipped" % path)
			continue
		var catalog := loaded as Translation
		var locale := TranslationServer.standardize_locale(catalog.locale)
		if not _merged.has(locale):
			var fresh := Translation.new()
			fresh.locale = locale
			_merged[locale] = fresh
		var into: Translation = _merged[locale]
		for key in catalog.get_message_list():
			if key != "":
				into.add_message(key, catalog.get_message(key))


## The locales some layer offers a `language.self_name` for, `en` first.
static func locales() -> PackedStringArray:
	var out := PackedStringArray()
	for language in languages():
		out.append(language.locale)
	return out


static func languages() -> Array[Language]:
	var out: Array[Language] = []
	var keys := _merged.keys()
	keys.sort()
	for locale: String in keys:
		var said := String((_merged[locale] as Translation).get_message("language.self_name"))
		if said != "":
			out.append(Language.new(locale, said))
	out.sort_custom(func(a: Language, b: Language) -> bool:
		return a.locale == "en" or (b.locale != "en" and a.locale < b.locale))
	return out


## Selects a language, live: Godot sends NOTIFICATION_TRANSLATION_CHANGED to every node.
static func set_language(locale: String) -> void:
	TranslationServer.set_locale(TranslationServer.standardize_locale(locale))


static func language() -> String:
	return TranslationServer.get_locale()


## Whether some layer words `key` in the current language or in English.
static func has(key: String) -> bool:
	return String(TranslationServer.translate(key)) != key


## The text of `key` with its `{name}` arguments filled in; a key no layer has is shown as its readable
## form (followed by its arguments), never raw.
static func text(key: String, args: Dictionary = {}) -> String:
	if has(key):
		var said := String(TranslationServer.translate(key))
		return said.format(args) if not args.is_empty() else said
	var fallback := readable(key)
	if args.is_empty():
		return fallback
	var values := PackedStringArray()
	for value in args.values():
		values.append(str(value))
	return "%s: %s" % [fallback, "  ·  ".join(values)]


## A key built from data: `<family>.<code>`, for a family of FAMILIES only.
static func code(family: String, a_code: String, args: Dictionary = {}) -> String:
	if not FAMILIES.has(family):
		push_error("[settings] %s is not a family whose keys are built from data" % family)
		return readable(a_code)
	return text(family + "." + a_code, args)


## A key's last part, made readable: `reason.too_far_away` → "too far away" (13b's rule).
static func readable(key: String) -> String:
	var parts := key.split(".")
	var last := parts[parts.size() - 1]
	if last == "done" and parts.size() > 1:
		last = parts[parts.size() - 2]
	return last.replace("_", " ").replace("-", " ")


## The clock setting; `clock()` answers the form in effect.
static func set_clock(clock: MineWorldSettings.ClockFormat) -> void:
	_clock = clock


static func clock_setting() -> MineWorldSettings.ClockFormat:
	return _clock


## H12 or H24: the setting, or for AUTO the current language's `clock.default`.
static func clock() -> MineWorldSettings.ClockFormat:
	if _clock != MineWorldSettings.ClockFormat.AUTO:
		return _clock
	return MineWorldSettings.ClockFormat.H24 if text("clock.default") == "24h" \
		else MineWorldSettings.ClockFormat.H12


## Appends the bundled Noto Sans SC to the default font's fallbacks, once. Latin keeps its font.
static func install_font_fallback() -> bool:
	if _font_installed:
		return true
	var font := FontFile.new()
	var path := module_dir().path_join(FONT_FILE)
	if font.load_dynamic_font(path) != OK:
		push_warning("[settings] %s: the CJK font did not load; Chinese falls back to the system's fonts" % path)
		return false
	var base: Font = ThemeDB.fallback_font
	var fallbacks := base.fallbacks
	fallbacks.append(font)
	base.fallbacks = fallbacks
	_font_installed = true
	return true


## The bundled font, for the glyph check.
static func bundled_font() -> FontFile:
	var font := FontFile.new()
	font.allow_system_fallback = false
	font.load_dynamic_font(module_dir().path_join(FONT_FILE))
	return font


## Marks a node as showing world content: never translated, never in the evidence walk.
static func mark_world_text(node: Node) -> void:
	node.add_to_group(WORLD_TEXT_GROUP)
	node.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED


## Every visible UI text under `root`, as "<node path>\t<text>", for the AC-SET-1/-2 evidence walk.
## World text (the group, and anything under a member) is left out.
static func visible_ui_texts(root: Node) -> PackedStringArray:
	var out := PackedStringArray()
	_walk(root, out)
	return out


static func _walk(node: Node, out: PackedStringArray) -> void:
	if node.is_in_group(WORLD_TEXT_GROUP):
		return
	if node is CanvasItem and not (node as CanvasItem).is_visible_in_tree():
		return
	if node is CanvasLayer and not (node as CanvasLayer).visible:
		return
	var path := str(node.get_path())
	if node is OptionButton:
		var option := node as OptionButton
		for i in option.item_count:
			_add(out, "%s[%d]" % [path, i], option.get_item_text(i))
	elif node is Button:
		_add(out, path, (node as Button).text)
	elif node is Label:
		_add(out, path, (node as Label).text)
	elif node is RichTextLabel:
		_add(out, path, (node as RichTextLabel).get_parsed_text())
	elif node is TabContainer:
		var tabs := node as TabContainer
		for i in tabs.get_tab_count():
			_add(out, "%s{%d}" % [path, i], tabs.get_tab_title(i))
	for child in node.get_children():
		_walk(child, out)


static func _add(out: PackedStringArray, path: String, said: String) -> void:
	if said.strip_edges() != "":
		out.append("%s\t%s" % [path, said])
