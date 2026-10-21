## `--slice-station-kit` (S29-d AC-d-2): the station kit as read, then the depot as BUILT, measured by rays
## against its colliders and from its meshes rather than read back from the numbers it was built from
## (`ARC-23`'s rule). `--slice-station-kit-file=<path>` measures another kit at the same site: the
## reduced-hall variant, or a planted refusal, which never gets this far (exit 3 by name).
##
## Every row prints the measured value beside its range and says which it is; the last line is
## `all station kit checks pass` or `<n> STATION KIT CHECKS OUT OF RANGE`.
class_name StationKitProbe
extends RefCounted

## AC-d-2's tolerances, metres.
const HALL_TOL := 0.05
const POINT_TOL := 0.01
const TOWER_TOL := 0.05
## How far a measuring ray may travel.
const RAY := 80.0


static func run(probe: SliceProbe) -> void:
	var st: Dictionary = SliceWorlds.current.station() if SliceWorlds.current != null else {}
	if st.is_empty():
		print("FAIL: this depiction draws no station\n\n1 STATION KIT CHECKS OUT OF RANGE")
		return
	var kit: StationKit = st["kit"]
	print("== S29-d -- the station kit %s, as read ==\n" % kit.path)
	for line in kit.lines():
		print(line)
	var fails := measure(probe)
	fails += StationPiecesProbe.run(kit)
	print("\n%s" % ("all station kit checks pass" if fails == 0
		else "%d STATION KIT CHECKS OUT OF RANGE" % fails))


## The depot as built, measured (also `--slice-measure`'s station rows); returns the rows out of range.
static func measure(probe: SliceProbe) -> int:
	var st: Dictionary = SliceWorlds.current.station() if SliceWorlds.current != null else {}
	if st.is_empty():
		return 0
	var kit: StationKit = st["kit"]
	print("\n== the station as built (rays against its colliders, its meshes) ==\n")
	var space := probe.player.get_world_3d().direct_space_state
	var fails := _hall(kit, space)
	fails += _doors(kit, space)
	fails += _towers(kit, st["site"], space, probe.slice.world)
	return fails


static func _row(nm: String, v: float, lo: float, hi: float) -> int:
	var ok := v >= lo - 1e-6 and v <= hi + 1e-6
	print("KIT %-40s %8.3f   [%.3f .. %.3f]  %s" % [nm, v, lo, hi, "ok" if ok else "OUT OF RANGE"])
	return 0 if ok else 1


## The distance from `from` along `dir` to the first world collider, or INF.
static func _cast(space: PhysicsDirectSpaceState3D, from: Vector3, dir: Vector3) -> float:
	var q := PhysicsRayQueryParameters3D.create(from, from + dir * RAY, Build.LAYER_WORLD)
	var hit := space.intersect_ray(q)
	return INF if hit.is_empty() else from.distance_to(hit["position"])


## The main place's inside, wall face to wall face, across and along, 1.5 m in from its corners (clear of
## every door and of the arch).
static func _hall(kit: StationKit, space: PhysicsDirectSpaceState3D) -> int:
	var main := kit.main_place()
	var o: Vector3 = StationDepot.report["origins"][main]
	var r := kit.floor_rect(main)
	var up := Vector3(0, 1.0, 0)
	var a := StationDepot.at(o, Vector2(r.get_center().x, r.position.y + 1.5)) + up
	var b := StationDepot.at(o, Vector2(r.end.x - 1.5, r.get_center().y)) + up
	var across := _cast(space, a, Vector3.RIGHT) + _cast(space, a, Vector3.LEFT)
	var along := _cast(space, b, Vector3.FORWARD) + _cast(space, b, Vector3.BACK)
	return _row("%s inside, east-west" % main, across, r.size.x - HALL_TOL, r.size.x + HALL_TOL) \
		+ _row("%s inside, north-south" % main, along, r.size.y - HALL_TOL, r.size.y + HALL_TOL)


## Each door: its leaves (the clear opening between the jambs, and the head), and its doorway points
## 0.40 m from the wall's faces on either side.
static func _doors(kit: StationKit, space: PhysicsDirectSpaceState3D) -> int:
	var fails := 0
	for d in kit.doors():
		var entry: Array = SliceTerrace.doors[int(StationDepot.report["doors"][d["name"]])]
		var sill: Vector3 = entry[0]
		var out := Vector3(sin(entry[1]), 0.0, cos(entry[1]))
		var side := Vector3(out.z, 0.0, -out.x)
		var inside := sill - out * 0.15 + Vector3(0, 1.0, 0)
		var clear := _cast(space, inside, side) + _cast(space, inside, -side)
		var leaves := int(d["leaves"])
		var head := _cast(space, sill - out * 0.15 + Vector3(0, 0.1, 0), Vector3.UP) + 0.1
		fails += _row("door %s leaf width" % d["name"], clear / leaves, StationKit.LEAF_W.x, StationKit.LEAF_W.y)
		fails += _row("door %s leaf height" % d["name"], head, StationKit.LEAF_H.x, StationKit.LEAF_H.y)
		var half := (kit.pair("arch", "clear_mm").x * 0.5 + 0.5) if d["from"] == StationKit.OUTSIDE \
			else clear * 0.5 + 0.6
		var op := SliceLayout.out_point(entry) + side * half + Vector3(0, 1.0, 0)
		var ip := SliceLayout.in_point(entry) + side * half + Vector3(0, 1.0, 0)
		fails += _row("door %s out-point to its wall" % d["name"], _cast(space, op, -out),
			SliceLayout.DOOR_INSET - POINT_TOL, SliceLayout.DOOR_INSET + POINT_TOL)
		fails += _row("door %s in-point to its wall" % d["name"], _cast(space, ip, out),
			SliceLayout.DOOR_INSET - POINT_TOL, SliceLayout.DOOR_INSET + POINT_TOL)
	return fails


## The towers' inner faces, from the forecourt's centre line; and their height, measured from what was
## drawn, printed beside its source.
static func _towers(kit: StationKit, site: Dictionary, space: PhysicsDirectSpaceState3D, world: Node3D) -> int:
	var x := float(site["facade_x"]) + kit.m("towers", "forward_mm") * 0.5
	var c := Vector3(x, float(StationDepot.report["floor_y"]) + 1.0, float(site["axis_z"]))
	var want := kit.m("towers", "inner_offset_mm")
	var fails := _row("north tower's inner face from the axis", _cast(space, c, Vector3.FORWARD),
		want - TOWER_TOL, want + TOWER_TOL)
	fails += _row("south tower's inner face from the axis", _cast(space, c, Vector3.BACK),
		want - TOWER_TOL, want + TOWER_TOL)
	var top := -INF
	for mi in world.find_children("*", "MeshInstance3D", true, false):
		if _under(mi, "station.tower"):
			var m := mi as MeshInstance3D
			top = maxf(top, (m.global_transform * m.get_aabb()).end.y)
	var stated := kit.m("towers", "shaft_mm") + kit.pair("towers", "belfry_mm").y + kit.pair("towers", "drum_mm").y \
		+ kit.pair("towers", "dome_mm").y + kit.pair("towers", "lantern_mm").y + kit.m("towers", "finial_mm")
	fails += _row("tower height above the floor, drawn", top - float(StationDepot.report["floor_y"]),
		stated - TOWER_TOL, stated + TOWER_TOL)
	print("KIT tower height's source: %s" % kit.group("towers")["source"])
	return fails


## Whether `n` or an ancestor carries `category`.
static func _under(n: Node, category: String) -> bool:
	while n != null:
		if str(n.get_meta("mw_category", "")) == category:
			return true
		n = n.get_parent()
	return false
