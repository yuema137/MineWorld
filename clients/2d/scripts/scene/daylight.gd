extends Node

## Day and night in the 2D town (TW-e; `clients/shared/WORLD_TIME.md` §6): one `CanvasModulate` over the
## world's canvas, coloured from the Presentation Pack's `daylight.json` by the disclosed sun's elevation.
## The HUD is on its own `CanvasLayer`s and is not tinted. Without a `calendar` record, or without the
## table (`--presentation=none`), no `CanvasModulate` exists: today's look (`INV-TW-6`). Art content only;
## no rule.

var _elevations: Array = []
var _tints: Array = []
var _modulate: CanvasModulate = null


## Reads `<pack>/daylight.json`, `{ "elevation_deg": [...], "tint": ["#rrggbb", ...] }`. A pack without
## one tints nothing.
func setup(pack_dir: String) -> void:
	if pack_dir == "":
		return
	var path := pack_dir.path_join("daylight.json")
	if not FileAccess.file_exists(path):
		return
	var table: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	if typeof(table) != TYPE_DICTIONARY:
		push_warning("[mineworld-2d] %s: not a daylight table" % path)
		return
	_elevations = table.get("elevation_deg", [])
	_tints = []
	for tint in table.get("tint", []):
		_tints.append(Color.html(String(tint)))
	if _elevations.size() != _tints.size() or _elevations.is_empty():
		push_warning("[mineworld-2d] %s: elevations and tints differ in length" % path)
		_elevations = []
		_tints = []


## The tint for this frame's sun, or none while nothing is disclosed.
func apply_sun(sun: Dictionary, has_calendar: bool) -> void:
	if not has_calendar or _elevations.is_empty():
		if _modulate != null:
			_modulate.queue_free()
			_modulate = null
		return
	if _modulate == null:
		_modulate = CanvasModulate.new()
		_modulate.name = "Daylight"
		add_child(_modulate)
	_modulate.color = tint_at(float(sun.get("elevation_deg", -90.0)))


## The table's colour at `elevation`, linear between rows and clamped at both ends.
func tint_at(elevation: float) -> Color:
	if elevation <= float(_elevations[0]):
		return _tints[0]
	for i in range(1, _elevations.size()):
		var hi := float(_elevations[i])
		if elevation <= hi:
			var lo := float(_elevations[i - 1])
			return (_tints[i - 1] as Color).lerp(_tints[i], (elevation - lo) / maxf(hi - lo, 0.0001))
	return _tints[_tints.size() - 1]
