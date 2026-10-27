## `--slice-station-kit`'s piece rows (S29-e AC-e-2): the station pieces built twice, off to one side of
## the scene and unbatched, and measured from their meshes -- once at the depot's sizes (the kit's
## `fitout` and `track` groups and the world-scale benches), once at village scale (S33-e's halt: an
## 8 x 5 m waiting room, a 4.6 x 4.6 m bay, a 4 x 40 m platform, one track ending at a buffer stop, a
## timber style). The same rows must hold at both scales: nothing in a piece is the depot's.
##
## Then the depot's drawn fixtures, as `StationFixtures` shows them offline (the defaults).
## Every row: `PIECE <scale> <row> <value> [lo .. hi] ok|OUT OF RANGE`.
class_name StationPiecesProbe
extends RefCounted

## VISUAL_SLICE §3.4's ranges and the cited values' tolerance.
const SEAT := Vector2(0.42, 0.47)
const COUNTER := Vector2(1.00, 1.10)
const TOL := 0.01
## Where the pieces are built for measuring: far below the scene, out of every view.
const AWAY := Vector3(0, -500, 0)


static func run(kit: StationKit) -> int:
	print("\n== S29-e -- the station pieces, built and measured ==\n")
	var t := kit.group("track")
	var depot := {"bench": Vector3(6.0, 0.6, 0.45), "kiosk": Vector3(2.0, 2.0, 2.4),
		"bay": Vector2(6.0, kit.m("fitout", "counter_depth_mm")), "counter": kit.m("fitout", "counter_mm"),
		"screen": kit.m("fitout", "screen_mm"), "grilles": kit.n("fitout", "grilles"),
		"board": kit.pair("fitout", "board_mm"), "clock": kit.m("fitout", "clock_mm"),
		"platform": Vector2(11.0, 60.0), "tracks": kit.n("track", "tracks"), "end": "through",
		"gauge": kit.m("track", "gauge_mm"), "pitch": kit.m("track", "tie_pitch_mm"),
		"tie": kit.pair("track", "tie_mm"), "rail": kit.pair("track", "rail_mm"),
		"strip": kit.m("track", "strip_mm"), "edge": kit.m("track", "edge_mm"), "style": "oak"}
	var village := depot.duplicate()
	village.merge({"bench": Vector3(3.0, 0.5, 0.45), "kiosk": Vector3.ZERO, "bay": Vector2(4.6, 0.6),
		"grilles": 1, "board": Vector2(1.6, 0.9), "clock": 0.5, "platform": Vector2(4.0, 40.0), "tracks": 1,
		"end": "buffer", "style": "timber"}, true)
	var fails := 0
	var root := Node3D.new()
	root.name = "PiecesProbe"
	(Engine.get_main_loop() as SceneTree).root.add_child(root)
	for scale: String in ["depot", "village"]:
		fails += _scale(root, scale, depot if scale == "depot" else village)
	root.queue_free()
	fails += _fixtures()
	print("\n%s" % ("all station piece checks pass" if fails == 0
		else "%d STATION PIECE CHECKS OUT OF RANGE" % fails))
	return fails


static func _row(scale: String, nm: String, v: float, lo: float, hi: float) -> int:
	var ok := v >= lo - 1e-6 and v <= hi + 1e-6
	print("PIECE %-8s %-36s %8.3f   [%.3f .. %.3f]  %s" % [scale, nm, v, lo, hi, "ok" if ok else "OUT OF RANGE"])
	return 0 if ok else 1


static func _text(scale: String, nm: String, n: Node, want: String) -> int:
	var keys := PackedStringArray()
	for l in n.find_children("*", "Label3D", true, false):
		if l.has_meta("text_key"):
			keys.append(String(l.get_meta("text_key")))
	var ok := want in keys
	print("PIECE %-8s %-36s %s  %s" % [scale, nm, ",".join(keys), "ok" if ok else "OUT OF RANGE"])
	return 0 if ok else 1


## The AABB of every mesh under `n`, in `n`'s parent frame.
static func _box(n: Node3D) -> AABB:
	if n is MeshInstance3D:
		return n.transform * (n as MeshInstance3D).get_aabb()
	var out := AABB()
	var first := true
	for mi in n.find_children("*", "MeshInstance3D", true, false):
		var m := mi as MeshInstance3D
		var b := (n.transform * (n.global_transform.affine_inverse() * m.global_transform)) * m.get_aabb()
		out = b if first else out.merge(b)
		first = false
	return out


static func _scale(root: Node3D, scale: String, p: Dictionary) -> int:
	var style := StationPieces.style_of(p["style"])
	var fails := 0
	var g := Node3D.new()
	g.position = AWAY
	root.add_child(g)
	var bench := StationPieces.bench_row(g, p["bench"].x, p["bench"].y, p["bench"].z, style)
	var slat := bench.get_child(0) as MeshInstance3D
	fails += _row(scale, "bench seat top", slat.position.y + slat.get_aabb().end.y, SEAT.x, SEAT.y)
	var bb := _box(bench)
	fails += _row(scale, "bench length within its solid", bb.size.x, p["bench"].x - 0.10, p["bench"].x + TOL)
	fails += _row(scale, "bench depth within its solid", bb.size.z, p["bench"].y - 0.20, p["bench"].y + TOL)
	if p["kiosk"].x > 0.0:
		var k := StationPieces.kiosk(g, Vector2(p["kiosk"].x, p["kiosk"].y), p["kiosk"].z, style)
		var kb := _box(k)
		fails += _row(scale, "kiosk height", kb.end.y, p["kiosk"].z - TOL, p["kiosk"].z + TOL)
	var bay := StationPieces.window_bay(g, p["bay"].x, p["counter"], p["bay"].y, p["screen"], p["grilles"], style)
	var counter := bay.get_child(0) as MeshInstance3D
	fails += _row(scale, "counter height", counter.position.y + counter.get_aabb().end.y, COUNTER.x, COUNTER.y)
	fails += _row(scale, "window bay width", _box(bay).size.x, p["bay"].x - TOL, p["bay"].x + 0.04 + TOL)
	fails += _text(scale, "window plate key", bay, "sign.window_closed")
	var board := StationPieces.board(g, p["board"].x, p["board"].y, style)
	fails += _row(scale, "board width", _box(board).size.x, p["board"].x, p["board"].x + 0.16 + TOL)
	fails += _text(scale, "board heading key", board, "sign.departures")
	var clock := StationPieces.clock(g, p["clock"], style)
	fails += _row(scale, "clock diameter", _box(clock).size.x, p["clock"], p["clock"] + 0.08 + TOL)
	var ts := StationTrack.style_of()
	var plat := StationTrack.platform(g, Rect2(0, 0, p["platform"].x, p["platform"].y), Vector3.LEFT, 0.0,
		p["edge"] + 0.5, p["strip"], 0.3, ts)
	fails += _row(scale, "warning strip width", (plat.get_child(3) as MeshInstance3D).get_aabb().size.x,
		p["strip"] - TOL, p["strip"] + TOL)
	for i in int(p["tracks"]):
		var x := -1.65 - 4.5 * i
		var tr := StationTrack.track(g, Vector3(x, -p["edge"], 0), Vector3(x, -p["edge"], -p["platform"].y),
			p["gauge"], p["tie"], p["pitch"], p["rail"], p["end"], ts)
		var a := _box(tr.get_child(0) as Node3D)
		var b := _box(tr.get_child(1) as Node3D)
		var gap := maxf(a.position.x, b.position.x) - minf(a.end.x, b.end.x)
		fails += _row(scale, "track %d gauge (inside faces)" % i, gap, p["gauge"] - 0.002, p["gauge"] + 0.002)
		fails += _row(scale, "track %d edge above rail head" % i, 0.0 - (a.end.y), p["edge"] - TOL, p["edge"] + TOL)
		var ties: PackedVector3Array = tr.get_meta("tie_points")
		var pitch := ties[0].distance_to(ties[1]) if ties.size() > 1 else 0.0
		fails += _row(scale, "track %d tie pitch" % i, pitch, p["pitch"] - TOL, p["pitch"] + TOL)
		var buffers := tr.get_child_count() > 4
		print("PIECE %-8s track %d end %s, buffer stop %s" % [scale, i, p["end"], "drawn" if buffers else "none"])
		if buffers != (p["end"] == "buffer"):
			fails += 1
	return fails


## The depot's drawn fixtures as they show offline: the defaults (ARC-108).
static func _fixtures() -> int:
	var fails := 0
	var want := {"board": MineWorldText.text("sign.no_service"), "window": "shut", "kiosk": "shut",
		"clock": "no hands"}
	for r in StationFixtures.rows():
		var ok: bool = r["shows"] == want.get(r["fixture"], "")
		print("PIECE fixture  %-8s in %-18s shows %-16s %s" % [r["fixture"], r["volume"] if r["place"] == ""
			else r["place"], r["shows"], "ok" if ok else "OUT OF RANGE"])
		if not ok:
			fails += 1
	return fails
