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

## The street's frame in this scene: where its (0, 0) stands. The café's door out-point is the street's
## side of the café's passage, (0, 2 800) mm; that out-point is (6 + DOOR_X, NORTH_FACE + DOOR_INSET) =
## (3.45, -7.70), so the frame's origin is 2.80 m south of it. Data, the same rule `SliceLayout` binds by.
const STREET_FRAME_ORIGIN := Vector3(3.45, 0.0, -4.90)
## The halt's door (S29-a): the street side of the station hall's arch, as the world discloses it, mm in
## the street's frame, and the outward normal of the hall's east façade there (toward the street: east).
## The sill stands DOOR_INSET behind the doorway, toward the hall (16c's rule). Drawn as a closed door in a
## short façade at the street's west forecourt: no interior and no furniture (ARC-307).
const HALT_DOORWAY := Vector2(-62050.0, -4900.0)
const HALT_OUTWARD := Vector3(1.0, 0.0, 0.0)
const HALT_DOOR_W := 1.60
const HALT_DOOR_H := 2.80

var _root: Node3D = null
var _station := {}
var _halt_door := -1


func build(root: Node3D, far_field: MineWorldFarField, connected: bool) -> Node3D:
	var pack := SlicePack.select().dir
	_station = StationKit.open(STATION_SITE, pack)
	if _station.has("refusal"):
		refusal = _station["refusal"]
		_station = {}
	_root = SliceWorld.build(root, far_field, connected, _station)
	# Without a station kit the hall's arch is still a disclosed doorway: a closed door stands for it.
	if _station.is_empty():
		_halt_door = _build_halt_door(_root)
	return _root


func station() -> Dictionary:
	return _station


## The halt's door in the street façade, registered in the scene's door registry; returns its index.
func _build_halt_door(g: Node3D) -> int:
	var at := STREET_FRAME_ORIGIN + MineWorldSpace.to_3d({"x": HALT_DOORWAY.x, "y": HALT_DOORWAY.y})
	var sill := Vector3(at.x, SliceStreet.WALK_Y, at.z) - HALT_OUTWARD * SliceLayout.DOOR_INSET
	var yaw := atan2(HALT_OUTWARD.x, HALT_OUTWARD.z)
	var f := Node3D.new()
	f.name = "HaltDoor"
	g.add_child(f)
	f.position = sill
	f.rotation.y = yaw
	# the façade around the opening (local +z is the street), and the door in it
	var stone := SlicePalette.limestone()
	var half := HALT_DOOR_W * 0.5
	var pier := 1.6
	var top := HALT_DOOR_H + 1.2
	for sx in [-1.0, 1.0]:
		Build.box(f, Vector3(sx * (half + pier * 0.5), top * 0.5, -0.20), Vector3(pier, top, 0.40), stone)
	Build.box(f, Vector3(0.0, HALT_DOOR_H + (top - HALT_DOOR_H) * 0.5, -0.20),
		Vector3(HALT_DOOR_W, top - HALT_DOOR_H, 0.40), stone)
	Profile.door(f, 0.0, 0.0, HALT_DOOR_W, HALT_DOOR_H, Color(0.30, 0.20, 0.12), false)
	SliceTerrace.doors.append([sill, yaw, "the halt -- door", "", 0.0, 0.0])
	return SliceTerrace.doors.size() - 1


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
	if _station.is_empty():
		out.append(SliceDepiction.entry(PackedStringArray([SliceDepiction.tag("station")]), "",
			SliceLayout.DOOR, _halt_door))
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
