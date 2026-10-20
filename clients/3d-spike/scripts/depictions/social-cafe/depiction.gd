## Social Café's street, behind the depiction interface (`DECISIONS.md` ARC-144; S33-c §5.4): the slice
## as it has always been drawn, and since S29-d the town's station closing its west end.
##
## It wraps `SliceWorld.build` and states what 16c's binding asserted in code: the café room depicts the
## place tagged `cafe` (the anchor, through its one door) and the place that door opens onto is drawn as
## the street. Every other place binds by doorway coincidence with the drawn doors (The Flower Room
## becomes the world's store, SC-3), as before. Its viewpoint tables stay with the street they belong to
## (`SliceWorld`, `SlicePerf`) and are handed out here unchanged.
##
## THE STATION (S29-d, ARC-192). The depot is the Presentation Pack's station kit `depot`, placed at
## STATION_SITE; its places are entries whose tags are the kit's (data, never a literal here), so the
## hall, the ticket bay and the platform bind to whatever places the world discloses with those tags at
## those doorways. The site is where the world's own passage already puts the arch: the street's doorway
## to the station is (-62 050, -4 900) mm in the street's frame, whose (0, 0) lands at scene (3.45, -4.90)
## (the café door's binding, 12d), so the doorway is at scene (-58.60, 0.00) and the façade, 0.40 m
## behind it, at x = -59.00 on the axis z = 0. If a world moves its station, the binding misses by name.
extends SliceDepiction

## Where the player starts offline: on the pavement, west of the cafe, facing east along the street so
## the first frame is the approach (moved from `SliceWorld`, values unchanged).
const SPAWN := Vector3(-11.5, 0.45, -5.55)
const SPAWN_YAW := -78.0
## The station's site (S29-d §5.5): the kit, the façade line, the street's axis, the arch's facing.
const STATION_SITE := {"kit": "depot", "facade_x": -59.0, "axis_z": 0.0, "facing": "east"}

var _root: Node3D = null
var _station := {}


func build(root: Node3D, far_field: MineWorldFarField, connected: bool) -> Node3D:
	var pack := ProjectSettings.globalize_path("res://").path_join(SliceMain.PACK_DIR).simplify_path()
	_station = StationKit.open(STATION_SITE, pack)
	if _station.has("refusal"):
		refusal = _station["refusal"]
		_station = {}
	_root = SliceWorld.build(root, far_field, connected, _station)
	return _root


func station() -> Dictionary:
	return _station


func place_volumes() -> Array[Area3D]:
	var out: Array[Area3D] = []
	if _root == null:
		return out
	for c in _root.get_children():
		if c is Area3D and c.has_meta("place_id"):
			out.append(c as Area3D)
	return out


func depictions() -> Array[Dictionary]:
	var cafe_door := -1
	for i in SliceTerrace.doors.size():
		var d: Array = SliceTerrace.doors[i]
		if d.size() > 3 and d[3] == SliceWorld.CAFE_PLACE:
			cafe_door = i
			break
	var out: Array[Dictionary] = [
		SliceDepiction.entry(PackedStringArray([SliceDepiction.tag("cafe")]), SliceWorld.CAFE_PLACE,
			SliceLayout.ROOM, cafe_door),
		SliceDepiction.entry(PackedStringArray([SliceDepiction.tag("street")]), SliceWorld.STREET_PLACE,
			SliceLayout.STREET),
	]
	out.append_array(StationViews.entries(_station))
	return out


func spawn() -> Dictionary:
	return {"position": SPAWN, "yaw": SPAWN_YAW}


func viewpoints() -> Dictionary:
	var v := {
		"farfield": SliceWorld.FAR_FIELD_VIEWPOINTS,
		"farfield_shots": SliceWorld.FAR_FIELD_SHOTS,
		"perf": SlicePerf.VIEWS,
		"perf_breakdown": SlicePerf.BREAKDOWN_VIEWS,
		# the café's own walk (`SliceProbeWorld._link_check`): out of the door, a jog, back in, the barista
		"probe_route": [],
	}
	if not _station.is_empty():
		# st_1 from the spawn, st_2 from the street's east end, both facing west (S29-d §5.8)
		var west := [["st_1_spawn_west", Vector3(SPAWN.x, 1.8, SPAWN.z), 90.0, 4.0],
			["st_2_east_end_west", Vector3(30.0, 1.8, 0.0), 90.0, 2.0]]
		v["perf"] = SlicePerf.VIEWS + StationViews.perf(_station)
		v["landmark"] = StationViews.landmark(_station, west,
			SliceWorld.FAR_FIELD_SHOTS.filter(func(s: Array) -> bool: return s[0] == "ff_5_west_aerial"))
		v["station_shots"] = StationViews.shots(_station, west)
		# from the café door out to the street's axis, then west along it to the arch: the carriageway
		# is clear of the pavement's furniture
		v["station_walk"] = StationViews.walk(_station, [Vector3(3.45, 0.45, 0.0)])
	return v
