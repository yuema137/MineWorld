extends RefCounted

## The client's wording: every user-visible string is a message key, looked up in the Presentation
## Pack's gettext file through the engine's `TranslationServer` (`docs/DECISIONS.md` `ARC-70`,
## `clients/2d/PRESENTATION.md` §7).
##
## The one script that reads the wording. Keys are built from data — `"action." + type`,
## `"reason." + code` — so no script holds an action type or a reason code to word it. A key the
## wording does not hold is shown as a readable form of its last part, so a client with no pack still
## says something true; a code nobody worded is shown as the code itself.

## The language 13b ships and selects; the language switch is the client-settings lane's.
const DEFAULT_LOCALE := "en"


## Adds the pack's wording for `locale` to the translation server and selects that language.
## Returns what went wrong, for a warning; nothing here stops the client.
static func load_pack(pack_dir: String, locale: String = DEFAULT_LOCALE) -> PackedStringArray:
	var problems := PackedStringArray()
	var path := pack_dir.path_join("i18n").path_join(locale + ".po")
	if not FileAccess.file_exists(path):
		problems.append("no wording at %s; keys fall back to their readable form" % path)
		return problems
	var loaded: Resource = ResourceLoader.load(path)
	if not loaded is Translation:
		problems.append("%s is not a gettext translation" % path)
		return problems
	TranslationServer.add_translation(loaded)
	TranslationServer.set_locale((loaded as Translation).locale)
	return problems


## Whether the wording holds `key`.
static func has(key: String) -> bool:
	return String(TranslationServer.translate(key)) != key


## The text of `key` with its `{name}` arguments filled in, or the readable fallback.
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


## A menu entry or a result for an action of `type`: `action.<type>` (or `action.<type>.done`), or,
## unworded, the type made readable followed by the names it concerns.
static func action(type: String, args: Dictionary, done: bool = false) -> String:
	var key := "action." + type + (".done" if done else "")
	if has(key):
		return text(key, args)
	var parts := PackedStringArray([readable(type)])
	for value in args.values():
		if str(value) != "":
			parts.append(str(value))
	return " ".join(parts)


## The code a rejection carries: a kernel reason as a string; a system's as the contract's
## `{"system": {"code": …}}`; any other dictionary by its first key (§15.12 E-1).
static func code(reason: Variant) -> String:
	if typeof(reason) == TYPE_DICTIONARY and not (reason as Dictionary).is_empty():
		var system: Variant = reason.get("system")
		if typeof(system) == TYPE_DICTIONARY and typeof(system.get("code")) == TYPE_STRING:
			return String(system["code"])
		return str((reason as Dictionary).keys()[0])
	return "" if reason == null else str(reason)


## Why not, in words: `reason.<code>`, or the code made readable.
static func reason(rejection: Variant) -> String:
	var said := code(rejection)
	return text("reason." + said) if said != "" else ""


## An amount of the wallet's minor units, in the pack's money format.
static func money(minor: Variant) -> String:
	var units := int(minor)
	var minus := "-" if units < 0 else ""
	units = absi(units)
	return text("format.money", {"amount": "%s%d.%02d" % [minus, units / 100, units % 100]})


## World seconds as the pack's clock.
static func clock(seconds: int) -> String:
	var in_day := posmod(seconds, 86400)
	return text("format.clock", {"day": seconds / 86400 + 1, "hh": "%02d" % (in_day / 3600),
		"mm": "%02d" % ((in_day % 3600) / 60)})


## A key's last part, made readable: `reason.too_far_away` → "too far away".
static func readable(key: String) -> String:
	var parts := key.split(".")
	var last := parts[parts.size() - 1]
	if last == "done" and parts.size() > 1:
		last = parts[parts.size() - 2]
	return last.replace("_", " ").replace("-", " ")
