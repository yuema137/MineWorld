## The open place's platform and the line beside it (S29-e §5.5), from the kit's `track` group: the slab
## over the place's floor with its coping, warning strip and edge on the far side from the hall, the
## tracks along that edge, the trackbed, a cutting at each end that the line leaves through (Q-S29e-1),
## a fence with a closed gate across each platform end, and the platform's departures board on the
## hall's wall beside the door. No train (step-29 §5.2).
class_name StationDepotPlatform
extends RefCounted


## `door`: the kit door from the main place into `place`; its wall's outward direction is the side the
## platform's edge and the tracks lie on.
static func build(g: Node3D, kit: StationKit, place: String, door: Dictionary, style: Dictionary) -> Node3D:
	if door.is_empty():
		return null
	var o: Vector3 = StationDepot.report["origins"][place]
	var r := kit.floor_rect(place)
	var out := StationDepot.outward(String(door["wall"]))
	var along := Vector3(absf(out.z), 0, absf(out.x))
	var ts := StationTrack.style_of()
	var top := o.y
	var lo := StationDepot.at(o, r.position)
	var hi := StationDepot.at(o, r.end)
	var rect := Rect2(minf(lo.x, hi.x), minf(lo.z, hi.z), absf(hi.x - lo.x), absf(hi.z - lo.z))
	var edge_mm := kit.m("track", "edge_mm")
	var plat := StationTrack.platform(g, rect, out, top, edge_mm + 0.5, kit.m("track", "strip_mm"),
		kit.m("track", "coping_mm"), ts)
	var line: Vector3 = plat.get_meta("edge_line")
	var half := (rect.size.y if absf(out.x) > 0.5 else rect.size.x) * 0.5
	var cut := kit.m("track", "cutting_mm")
	var near := kit.m("track", "near_mm")
	var centres := kit.m("track", "centres_mm")
	var count := kit.n("track", "tracks")
	var rail_y := top - edge_mm
	var tie := kit.pair("track", "tie_mm")
	var rail := kit.pair("track", "rail_mm")
	var rows := []
	for i in count:
		var c := Vector3(line.x, rail_y, line.z) + out * (near + centres * i)
		var t := StationTrack.track(g, c - along * (half + cut), c + along * (half + cut), kit.m("track", "gauge_mm"),
			tie, kit.m("track", "tie_pitch_mm"), rail, "through", ts)
		rows.append({"centre_from_edge": near + centres * i, "ties": t.get_meta("count")})
	# the trackbed between the platform and beyond the far track, and the bank back up to the ground
	var width := near * 2.0 + centres * (count - 1)
	var bed_y := rail_y - rail.y - 0.12
	var bed_mid := Vector3(line.x, bed_y, line.z) + out * width * 0.5
	var bed_size := Vector3(width, 0.4, half * 2.0) if absf(out.x) > 0.5 else Vector3(half * 2.0, 0.4, width)
	Build.box(g, bed_mid - Vector3(0, 0.2, 0), bed_size, ts["ballast"], 0.0, true)
	var bank_h := top - bed_y + 0.3
	var bank_size := Vector3(6.0, bank_h, half * 2.0) if absf(out.x) > 0.5 else Vector3(half * 2.0, bank_h, 6.0)
	Build.box(g, bed_mid + out * (width * 0.5 + 3.0) + Vector3(0, bank_h * 0.5 - 0.3, 0), bank_size, ts["bank"],
		0.0, true)
	for s: float in [-1.0, 1.0]:
		var mid := Vector3(line.x, bed_y, line.z) + out * width * 0.5
		StationTrack.cutting(g, mid + along * s * half, mid + along * s * (half + cut), width,
			kit.m("track", "bank_mm"), ts)
		var a := Vector3(line.x, top, line.z) + along * s * (half - 0.05)
		var wall_side := a - out * (rect.size.x if absf(out.x) > 0.5 else rect.size.y)
		StationTrack.fence(g, a - out * 0.05, wall_side + out * 0.05, kit.m("track", "fence_mm"), 0.5, ts)
	# where the near ground must be cut so the sunken trackbed shows: from under the platform (its slab
	# covers the cut) across the tracks to the far bank, and along the line to past the overbridges
	var terrace := kit.m("track", "bank_mm") * 1.2
	var c0 := line - out * terrace - along * (half + cut + 9.0)
	var c1 := line + out * (width + 6.0) + along * (half + cut + 9.0)
	StationDepotInterior.report["trench"] = [Rect2(minf(c0.x, c1.x), minf(c0.z, c1.z), absf(c1.x - c0.x),
		absf(c1.z - c0.z))]
	var bsize := kit.pair("fitout", "platform_board_mm")
	var there := Vector2(door["there"][0], door["there"][1]) / 1000.0
	var wall_face := StationDepot.at(o, there) - out * (SliceLayout.DOOR_INSET - 0.06)
	var board := StationDepotInterior._put(StationPieces.board(g, bsize.x, bsize.y, style),
		wall_face + along * 3.2 + Vector3(0, 2.4, 0), out)
	StationFixtures.register(board, String(StationDepot.report["volumes"].get(place, "")))
	StationDepotInterior.report["fixtures"]["platform_board"] = board
	StationDepotInterior.report["platform"] = {"top_y": top, "rail_y": rail_y, "tracks": rows,
		"strip_m": kit.m("track", "strip_mm"), "gauge_m": kit.m("track", "gauge_mm"),
		"pitch_m": kit.m("track", "tie_pitch_mm"), "edge": line}
	return plat
