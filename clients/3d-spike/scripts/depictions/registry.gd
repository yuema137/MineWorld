## Which world this client presents, and the depiction that draws it (`DECISIONS.md` ARC-144, ARC-146,
## ARC-177; PR 16f-c).
##
##   key()      --world-key=<key> when given; else, offline (no --server=), the slice's own street,
##              social-cafe, as before; else "" -- and the far-field loader refuses a join with no key
##              (FAR_FIELD_WORLD_UNKNOWN, RL-e0), unchanged
##   open(key)  the first of, in order:
##              0. `--slice-depiction-file=<path>` given: that file, drawn by `SliceDataDepiction`
##                 (a variant, or another world's file, on this world's server; no GDScript change);
##              1. the pack's `depictions/<key>.yaml` (`SlicePack`): drawn by `SliceDataDepiction`, the
##                 generic builders of S33-d revision 2 (`ARC-177`). The pack decides which worlds it
##                 draws from data: the client keeps no list;
##              2. a bespoke depiction script shipped with the client, res://scripts/depictions/<key>/
##                 depiction.gd. When it is absent, the world's 3D presentation file 3d-<key>.pck beside
##                 the executable is loaded if it exists (Godot's ProjectSettings.load_resource_pack), and
##                 the script looked for again (ARC-146);
##              3. still absent: the world is drawn plainly from the server's disclosure
##                 (`SlicePlainDepiction`; 16f-b, `ARC-52`, the operator's Q-16f-1 (A), amending ARC-146):
##                 its far field from the pack (a pack without one is refused FAR_FIELD_MISSING by the
##                 far-field loader, before anything is drawn), plain ground, and every place from its
##                 own `place-shape`.
##              Only a key that is not a world key (empty, or a path) gives null, and the caller refuses
##              PRESENTATION_WORLD_MISSING by name. A depiction file the reader refuses refuses the run by
##              its own code (`SliceMain._open_depiction`); there is no fallback past it.
##
## A path convention, not a table: a world is drawn from data iff its pack has the file, and by a bespoke
## script iff the script is present, so this client keeps no list of world keys (and no list of any
## world's place keys, SC-4).
class_name SliceWorlds
extends RefCounted

const DEPICTION := "res://scripts/depictions/%s/depiction.gd"
## A world's depiction file, relative to the pack's root (`docs/MODULE_SPEC.md` sec.6.1.2).
const DATA_FILE := "depictions/%s.yaml"
## The slice offline: the street it has always drawn.
const OFFLINE_KEY := "social-cafe"
## The refusal's code (step-33 §8.2, PK-2).
const MISSING := "PRESENTATION_WORLD_MISSING"

## The depiction this run draws, once opened (read by the layout, the probes and the checks).
static var current: SliceDepiction = null
## Where `current` came from, as the start line prints it: its file, its script, or "the world's
## disclosure".
static var source := ""


static func key() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--world-key="):
			return a.trim_prefix("--world-key=")
	return OFFLINE_KEY if SliceLink.address_from_args() == "" else ""


## The depiction for `world`, or null when `world` is no world key (see above).
static func open(world: String) -> SliceDepiction:
	if world == "" or world.contains("/") or world.contains(".."):
		return null
	var file := _data_file(world)
	if file != "":
		var data := SliceDataDepiction.new()
		data.file_path = file
		return _opened(data, world, file)
	var path := DEPICTION % world
	if not ResourceLoader.exists(path):
		var pck := OS.get_executable_path().get_base_dir().path_join("3d-%s.pck" % world)
		if FileAccess.file_exists(pck):
			print("[slice] loading the 3D presentation file %s" % pck)
			ProjectSettings.load_resource_pack(pck)
	if not ResourceLoader.exists(path):
		return _opened(SlicePlainDepiction.new(), world,
			"the world's disclosure (no depiction of it in this pack or client; drawn plainly)")
	var script := load(path) as GDScript
	if script == null:
		return null
	var depiction := script.new() as SliceDepiction
	if depiction == null:
		return null
	return _opened(depiction, world, path)


## The depiction file for `world`: the one named on the command line, else the pack's, else "".
static func _data_file(world: String) -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with(SliceDataDepiction.FILE_ARG):
			return a.trim_prefix(SliceDataDepiction.FILE_ARG)
	var pack := SlicePack.select()
	if not pack.ok():
		return ""
	var file := pack.path(DATA_FILE % world)
	return file if FileAccess.file_exists(file) else ""


static func _opened(depiction: SliceDepiction, world: String, from: String) -> SliceDepiction:
	depiction.key = world
	current = depiction
	source = from
	return depiction


## The refusal, as printed and shown (RL-e0's line format, `MineWorldFarField.refusal_line`).
static func refusal_line(world: String) -> String:
	return "[mineworld-slice] %s: { world: %s }: this client has no depiction for the world; install its 3D presentation file 3d-%s.pck" \
		% [MISSING, world, world]
