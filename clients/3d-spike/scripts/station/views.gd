## What a depiction states about the station it draws, computed from the kit, the site and the built
## depot (`StationDepot.report`) rather than typed per world (S29-d §§5.7, 5.8): the depiction entries the
## binding uses, the perf views, the landmark check's viewpoints, the station shots and the walk in.
##
## The entries' tags are the kit's, read as data: this file writes no tag and no place key (I-d-4).
class_name StationViews
extends RefCounted

## The landmark window's half-width about the facing, degrees (S29-d §5.9).
const WINDOW_DEG := 10.0
## A body's height over its floor in the perf views (`SlicePerf.VIEWS`: 0.45 on the 0.14 pavement).
const BODY_UP := 0.31
## The shots' and checks' eye height above the floor.
const EYE := 1.66


## One ROOM entry per kit place a door leads to, on that door, with the kit's tags.
static func entries(station: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if station.is_empty():
		return out
	var kit: StationKit = station["kit"]
	var doors: Dictionary = StationDepot.report.get("doors", {})
	for d in kit.doors():
		var to := String(d["to"])
		out.append(SliceDepiction.entry(kit.tags(to), String(kit.place(to)["volume"]), SliceLayout.ROOM,
			int(doors.get(d["name"], -1))))
	return out


## The station's perf views (S29-d §5.8): the forecourt, the hall's far corner looking across it, and the
## open place's far end looking along it. [name, body position, yaw, pitch].
static func perf(station: Dictionary) -> Array:
	var kit: StationKit = station["kit"]
	var site: Dictionary = station["site"]
	var r := StationDepot.report
	var y := float(r["floor_y"]) + BODY_UP
	var out := [["station forecourt", Vector3(_forecourt_x(kit, site), y, float(site["axis_z"])), 90.0, 8.0]]
	var main := kit.main_place()
	var hr := kit.floor_rect(main)
	var corner := StationDepot.at(r["origins"][main], hr.end - Vector2(1.0, 1.0))
	out.append(["station hall wide", Vector3(corner.x, y, corner.z), 135.0, 2.0])
	for p in kit.place_names():
		if kit.place(p)["shell"] == "open" and r["origins"].has(p):
			var pr := kit.floor_rect(p)
			var end := StationDepot.at(r["origins"][p], Vector2(pr.get_center().x, pr.end.y - 1.0))
			out.append(["station platform", Vector3(end.x, y, end.z), 180.0, 0.0])
	return out


## The landmark check's rows (S29-d §5.9): each of `lm1` ([name, eye, yaw, pitch]) with the compass
## bearing it faces and the band's top -- the elevation, from that eye, of the lower of the two frontage
## eaves at the street's west end, `station.eaves` -- and each of `lm2` as a footprint row.
static func landmark(station: Dictionary, lm1: Array, lm2: Array) -> Array:
	var out := []
	for v in lm1:
		var eye: Vector3 = v[1]
		out.append([v[0], eye, v[2], v[3], fposmod(-float(v[2]), 360.0), band_top(eye, station.get("eaves", []))])
	for s in lm2:
		out.append([s[0], s[1], s[2], s[3], "footprint"])
	return out


## The elevation (degrees) of the lowest of `eaves` seen from `eye`. `eaves` are scene points: of each
## side of the street, the westernmost unit's eaves at its west corner on the façade. Since 16f-d2 the
## terrace is pack art, so its authoring source writes these points into the depiction file
## (`station_kit.eaves`) instead of the client reading the built terrace.
static func band_top(eye: Vector3, eaves: Array) -> float:
	var lowest := INF
	for w: Vector3 in eaves:
		var dist := Vector2(w.x - eye.x, w.z - eye.z).length()
		lowest = minf(lowest, rad_to_deg(atan2(w.y - eye.y, dist)))
	return lowest


## The station's shots (S29-d §5.8, `st_1` ... `st_4`): the depiction's two street views, the forecourt,
## and the arch's threshold looking in and out. [name, camera position, yaw, pitch].
static func shots(station: Dictionary, street_views: Array) -> Array:
	var kit: StationKit = station["kit"]
	var site: Dictionary = station["site"]
	var y := float(StationDepot.report["floor_y"]) + EYE
	var z := float(site["axis_z"])
	var out := street_views.duplicate()
	out.append(["st_3_forecourt", Vector3(_forecourt_x(kit, site), y, z), 90.0, 8.0])
	var sill := float(site["facade_x"]) - 0.45
	out.append(["st_4_arch_threshold_in", Vector3(sill, y, z + 0.6), 90.0, 0.0])
	out.append(["st_4_arch_threshold_out", Vector3(sill, y, z + 0.6), -90.0, 0.0])
	out.append_array(inside_shots(kit))
	return out


## S29-e §5.7's shots inside, from the kit and the built depot: the hall from its south-east corner
## looking up its length; the ticket window 2.2 m from its counter; the board from 5 m; the platform
## from its far end along its edge. [name, eye, yaw, pitch].
static func inside_shots(kit: StationKit) -> Array:
	var r := StationDepot.report
	var y := float(r["floor_y"]) + EYE
	var main := kit.main_place()
	var o: Vector3 = r["origins"][main]
	var hr := kit.floor_rect(main)
	var out := []
	var se := StationDepot.at(o, Vector2(hr.end.x - 1.5, hr.position.y + 1.5))
	out.append(["st_5_hall_wide", Vector3(se.x, y, se.z), 18.0, 4.0])
	for d in kit.doors():
		if d["from"] != main:
			continue
		var p := String(d["to"])
		if not r["origins"].has(p):
			continue
		var pr := kit.floor_rect(p)
		var po: Vector3 = r["origins"][p]
		var dir := StationDepot.outward(String(d["wall"]))
		match String(kit.place(p)["shell"]):
			"annex":
				# the counter stands on the far wall, `dir` from the door
				var far := StationDepot.at(po, pr.get_center()) + dir * ((pr.size.y if absf(dir.z) > 0.5
					else pr.size.x) * 0.5 - kit.m("fitout", "counter_depth_mm"))
				var eye := far - dir * 2.2
				out.append(["st_6_ticket_window", Vector3(eye.x, float(r["floor_y"]) + 1.6, eye.z),
					rad_to_deg(atan2(-dir.x, -dir.z)), -6.0])
			"open":
				var e := StationDepot.at(po, Vector2(pr.get_center().x, pr.end.y - 1.0)) + dir * (pr.size.x * 0.5 - 2.0)
				out.append(["st_8_platform", Vector3(e.x, float(r["floor_y"]) + 1.7, e.z), 180.0, -3.0])
	var bp := StationDepot.at(o, Vector2(hr.get_center().x, hr.end.y - 5.0))
	out.append(["st_7_board", Vector3(bp.x, y, bp.z), 0.0, 14.0])
	return out


## The walk into the station's main place: through `via` (scene points on the street), then in by the
## main place's door, which the probe finds by its tags.
static func walk(station: Dictionary, via: Array) -> Dictionary:
	var kit: StationKit = station["kit"]
	var site: Dictionary = station["site"]
	var approach := Vector3(_forecourt_x(kit, site), via[0].y if not via.is_empty() else 0.45,
		float(site["axis_z"]))
	return {"via": via + [approach], "tags": kit.tags(kit.main_place())}


## 15 m in front of the towers, on the axis: where the forecourt opens to the street.
static func _forecourt_x(kit: StationKit, site: Dictionary) -> float:
	return float(site["facade_x"]) + kit.m("towers", "forward_mm") + 15.0
