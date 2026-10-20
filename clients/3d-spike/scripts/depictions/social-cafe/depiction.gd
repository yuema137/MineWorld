## Social Café's street, behind the depiction interface (`DECISIONS.md` ARC-144; S33-c §5.4): the slice
## as it has always been drawn, with no visual or behavioural change.
##
## It wraps `SliceWorld.build` and states what 16c's binding asserted in code: the café room depicts the
## place tagged `cafe` (the anchor, through its one door) and the place that door opens onto is drawn as
## the street. Every other place binds by doorway coincidence with the drawn doors (The Flower Room
## becomes the world's store, SC-3), as before. Its viewpoint tables stay with the street they belong to
## (`SliceWorld`, `SlicePerf`) and are handed out here unchanged.
extends SliceDepiction

## Where the player starts offline: on the pavement, west of the cafe, facing east along the street so
## the first frame is the approach (moved from `SliceWorld`, values unchanged).
const SPAWN := Vector3(-11.5, 0.45, -5.55)
const SPAWN_YAW := -78.0

var _root: Node3D = null


func build(root: Node3D, far_field: MineWorldFarField, connected: bool) -> Node3D:
	_root = SliceWorld.build(root, far_field, connected)
	return _root


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
	return [
		SliceDepiction.entry(PackedStringArray([SliceDepiction.tag("cafe")]), SliceWorld.CAFE_PLACE,
			SliceLayout.ROOM, cafe_door),
		SliceDepiction.entry(PackedStringArray([SliceDepiction.tag("street")]), SliceWorld.STREET_PLACE,
			SliceLayout.STREET),
	]


func spawn() -> Dictionary:
	return {"position": SPAWN, "yaw": SPAWN_YAW}


func viewpoints() -> Dictionary:
	return {
		"farfield": SliceWorld.FAR_FIELD_VIEWPOINTS,
		"farfield_shots": SliceWorld.FAR_FIELD_SHOTS,
		"perf": SlicePerf.VIEWS,
		"perf_breakdown": SlicePerf.BREAKDOWN_VIEWS,
		# the café's own walk (`SliceProbeWorld._link_check`): out of the door, a jog, back in, the barista
		"probe_route": [],
	}
