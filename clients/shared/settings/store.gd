## Where the settings live, how they are read and how they are saved (`clients/shared/SETTINGS.md` §4).
##
## The only code that reads or writes the settings file. It holds settings only (INV-SET-2): it writes
## the keys of `MineWorldSettings` and nothing from the command line. A harness run never reads or writes
## the user's file (INV-SET-5).
class_name MineWorldSettingsStore
extends RefCounted

const DEFAULT_PATH := "user://settings.cfg"
const VERSION := 1
const NONE := "none"

## The value in effect.
var settings := MineWorldSettings.new()
## The file, as given (`user://…` or an absolute path); "" when settings are not persisted.
var path := ""
## `--extra-locale=` folders, for the checks' marker catalog.
var extra_locale_dirs := PackedStringArray()
## What loading found wrong, one line each, already printed as warnings.
var problems := PackedStringArray()

## The value as last loaded or saved: what a save compares against to know which sections changed.
var _saved := MineWorldSettings.new()
## Set by a load problem: the file is left as it is, and copied to `.bak` before it is next replaced.
var _keep_old := false


## Resolves the file from the command line. A harness flag (`--<flag>` or `--<flag>=…` for a flag in
## `harness_flags`) means `--settings=none` unless `--settings=<path>` is also given.
static func open(args: PackedStringArray, harness_flags: PackedStringArray) -> MineWorldSettingsStore:
	var store := MineWorldSettingsStore.new()
	var chosen := ""
	var harness := false
	for arg in args:
		if arg.begins_with("--settings="):
			chosen = arg.substr(11)
		elif arg.begins_with("--extra-locale="):
			store.extra_locale_dirs.append(arg.substr(15))
		elif arg.begins_with("--"):
			var flag := arg.substr(2).split("=", true, 1)[0]
			if harness_flags.has(flag):
				harness = true
	if chosen == "":
		chosen = NONE if harness else DEFAULT_PATH
	if chosen != NONE:
		store.path = _resolve(chosen)
		store._load()
	return store


## `user://…` and absolute paths as given; a relative path against the working directory the process
## was started in (`PWD`), or the project folder where the shell sets none.
static func _resolve(given: String) -> String:
	if given.begins_with("user://") or given.is_absolute_path():
		return given
	var cwd := OS.get_environment("PWD")
	return (cwd if cwd != "" else ProjectSettings.globalize_path("res://")).path_join(given)


func is_persistent() -> bool:
	return path != ""


## The file's absolute path, for messages and checks.
func absolute_path() -> String:
	return ProjectSettings.globalize_path(path) if path != "" else ""


func _load() -> void:
	var file := absolute_path()
	var config := ConfigFile.new()
	if not FileAccess.file_exists(file):
		# A save interrupted between removing the old file and renaming the new one (Windows, §4).
		if FileAccess.file_exists(file + ".tmp") and config.load(file + ".tmp") == OK:
			_warn("%s is missing; recovered the interrupted save %s.tmp" % [file, file])
		else:
			return
	else:
		var err := config.load(file)
		if err != OK:
			_problem("%s: unreadable (%s); using the defaults; the file is kept until you apply a change" %
				[file, error_string(err)])
			return
	var version: Variant = config.get_value("meta", "version", VERSION)
	if typeof(version) != TYPE_INT:
		_problem("%s: meta/version = %s: not an integer; using the defaults" % [file, var_to_str(version)])
		return
	if int(version) > VERSION:
		_problem("%s: meta/version = %d is newer than this client (%d); using the defaults; the file is kept until you apply a change" %
			[file, int(version), VERSION])
		return
	var found := PackedStringArray()
	settings = MineWorldSettings.read(config, found)
	for line in found:
		_problem("%s: %s" % [file, line])
	_saved = settings.copy()


## A `language` no catalog layer offers is a load problem like any other: one warning naming the file
## and the key, the default, and the file left as it is.
func settle_language(offered: PackedStringArray) -> void:
	if offered.is_empty() or offered.has(settings.language):
		return
	var where := absolute_path() if path != "" else "--settings=none"
	_problem("%s: general/language = \"%s\": no catalog offers it; the default is used" % [where,
		settings.language])
	settings.language = MineWorldSettings.DEFAULT_LANGUAGE
	_saved.language = MineWorldSettings.DEFAULT_LANGUAGE


## Saves `value`: re-reads the file, writes the sections `value` changed since the last load or save,
## keeps every other section as the file has it, and replaces the file atomically. Without a file it
## only takes the value in memory.
func save(value: MineWorldSettings) -> Error:
	if path == "":
		settings = value.copy()
		_saved = value.copy()
		return OK
	var file := absolute_path()
	var dir := file.get_base_dir()
	if not DirAccess.dir_exists_absolute(dir):
		var made := DirAccess.make_dir_recursive_absolute(dir)
		if made != OK:
			_warn("%s: cannot create the folder (%s); the settings stay in memory for this run" %
				[dir, error_string(made)])
			settings = value.copy()
			return made
	# Merge into the file as it is now, if it is one this client can read; an unreadable or newer file
	# is replaced whole (it was kept as .bak above).
	var merged := ConfigFile.new()
	var on_disk := ConfigFile.new()
	if FileAccess.file_exists(file) and on_disk.load(file) == OK \
			and typeof(on_disk.get_value("meta", "version", VERSION)) == TYPE_INT \
			and int(on_disk.get_value("meta", "version", VERSION)) <= VERSION:
		merged = on_disk
	var changed := PackedStringArray()
	for section in MineWorldSettings.SECTIONS:
		# After a load problem every owned section is rewritten, so nothing invalid stays in the file.
		if _keep_old or value.differs_in(_saved, section) or not merged.has_section(section):
			changed.append(section)
	value.write(merged, changed)
	merged.set_value("meta", "version", VERSION)
	if _keep_old and FileAccess.file_exists(file):
		var kept := DirAccess.copy_absolute(file, file + ".bak")
		if kept != OK:
			_warn("%s: cannot keep the old file as .bak (%s); not saved" % [file, error_string(kept)])
			return kept
	var written := merged.save(file + ".tmp")
	if written == OK:
		written = DirAccess.rename_absolute(file + ".tmp", file)
	if written != OK:
		_warn("%s: cannot save (%s); the settings stay in memory for this run" % [file, error_string(written)])
		settings = value.copy()
		return written
	_keep_old = false
	# The baseline of the next save is this value: a section this client has not changed since is not
	# written again, so it never undoes what another client saved there meanwhile.
	settings = value.copy()
	_saved = value.copy()
	return OK


func _problem(line: String) -> void:
	problems.append(line)
	_keep_old = true
	_warn(line)


static func _warn(line: String) -> void:
	push_warning("[settings] " + line)
