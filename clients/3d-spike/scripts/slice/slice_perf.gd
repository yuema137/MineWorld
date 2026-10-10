## The slice's frame-cost instrument, fixed before any budget lever is pulled
## (RL-b, `.structured-coding/plans/mvp1/pr-rl-b-3d-budget.md` §3, M-1 … M-7).
##
## Why it is its own file and its own protocol. The earlier `--perf` took twenty
## samples per view with no warm-up, so a single pipeline compilation or a
## throttled window could land in the sample: one re-run reported 8.7 s worst
## frames (E-RLb-0). A number taken that way says nothing about a lever. So:
##
##   M-1  eight named views, fixed here and never edited to suit a result;
##   M-2  per view: settle 6 frames, discard 120 warm-up frames, measure 300;
##        each frame records the wall interval and the viewport's measured CPU
##        and GPU render time; draw calls, primitives and objects come from
##        the `Performance` monitors; video, texture and buffer memory at the end;
##   M-3  vsync off (the probe already does this for every scripted mode), and
##        the run records machine, OS, adapter, Godot build and renderer;
##   M-4  the render size is `--res=WxH` (the launcher defaults `--perf` to the
##        Default tier's 1920x1080). The project's stretch mode is `viewport`,
##        which renders at `content_scale_size` whatever the window is -- that
##        is why `--res` used to change nothing (A-2) -- so it is set here;
##   M-5  a run with any measured frame over 250 ms is INCONCLUSIVE. The
##        launcher runs three fresh processes and re-runs an inconclusive one;
##        every run writes JSON, and the statistic per view is the median over
##        the conclusive runs of that view's p95 frame interval;
##   M-7  `--perf-breakdown`: three views, everything on, then each category
##        switched off in turn. Categories are told apart by the `mw_category`
##        meta set where the node is made, never by guessing from a name.
class_name SlicePerf
extends RefCounted

## M-1. [name, position, yaw, pitch]. Each is the pose of an existing shot or
## perf view, except `skyline east`, which looks east along the street above the
## eaves line so the backdrop is in frame.
const VIEWS := [
	["street wide", Vector3(-20.0, 0.45, -5.20), -75.0, -1.0],
	["cafe frontage", Vector3(0.30, 0.45, -5.55), -52.0, 3.0],
	["interior", Vector3(2.75, 0.60, -10.60), -38.0, -3.0],
	["doorway", Vector3(3.45, 0.45, -6.90), 0.0, 0.0],
	["street east", Vector3(22.0, 0.45, -5.30), 96.0, -1.0],
	["south side", Vector3(2.0, 0.45, -4.60), 165.0, 1.0],
	["florist interior", Vector3(11.4, 0.45, -9.10), -38.0, -6.0],
	["skyline east", Vector3(-20.0, 0.45, -5.20), -90.0, 4.0],
]

## M-7's views.
const BREAKDOWN_VIEWS := ["street wide", "cafe frontage", "interior"]

## M-7's categories, each switched off alone against the all-on base.
const CATEGORIES := ["merged", "gltf", "foliage", "characters", "labels", "backdrop",
	"ground", "voxelgi", "ssil", "ssao", "shadows", "occlusion"]

const SETTLE := 6
const WARMUP := 120
const MEASURED := 300
const INCONCLUSIVE_MS := 250.0

## M-6, fixed before the baseline was seen and never changed after it.
const PASS_P95_MS := 16.7
const PASS_DRAW_CALLS := 2000
const PASS_PRIMITIVES := 3000000
const PASS_VIDEO_MB := 2048.0
const PASS_RES := Vector2i(1920, 1080)
const PASS_GI := "voxel"

var probe: SliceProbe
var _vp: Viewport
var _rid: RID


func _init(p: SliceProbe) -> void:
	probe = p


## The `--perf-<key>=value` user argument, or "".
static func arg(key: String) -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--perf-%s=" % key):
			return a.substr(("--perf-%s=" % key).length())
	return ""


static func flag(key: String) -> bool:
	return ("--perf-" + key) in OS.get_cmdline_user_args()


func run() -> void:
	_vp = probe.get_viewport()
	_apply_res()
	await probe._settle(SETTLE)
	_rid = _vp.get_viewport_rid()
	RenderingServer.viewport_set_measure_render_time(_rid, true)
	if flag("breakdown"):
		await _breakdown()
	else:
		await _protocol()


# --- M-4: the render size --------------------------------------------------------

func _apply_res() -> void:
	var r := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--res="):
			r = a.substr(6)
	if r == "":
		return
	var parts := r.split("x")
	if parts.size() != 2:
		push_error("--res=%s is not WxH" % r)
		return
	var size := Vector2i(int(parts[0]), int(parts[1]))
	var root := probe.get_tree().root
	root.content_scale_size = size
	DisplayServer.window_set_size(size)


func _render_size() -> Vector2i:
	return _vp.get_texture().get_size()


# --- M-2: one view ---------------------------------------------------------------

## Place, settle, warm up, then measure. Returns the view's record.
func _measure_view(v: Array) -> Dictionary:
	probe.player.place(v[1], v[2], v[3])
	probe.player.set_camera(SliceProbe.FP)
	await probe._settle(SETTLE)
	for i in range(WARMUP):
		await RenderingServer.frame_post_draw
	var wall: Array[float] = []
	var cpu: Array[float] = []
	var gpu: Array[float] = []
	var draws := 0
	var prims := 0
	var objs := 0
	var over := 0
	var last := Time.get_ticks_usec()
	for i in range(MEASURED):
		await RenderingServer.frame_post_draw
		var now := Time.get_ticks_usec()
		var ms := float(now - last) / 1000.0
		last = now
		wall.append(ms)
		if ms > INCONCLUSIVE_MS:
			over += 1
		cpu.append(RenderingServer.viewport_get_measured_render_time_cpu(_rid))
		gpu.append(RenderingServer.viewport_get_measured_render_time_gpu(_rid))
		draws = maxi(draws, int(Performance.get_monitor(
			Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)))
		prims = maxi(prims, int(Performance.get_monitor(
			Performance.RENDER_TOTAL_PRIMITIVES_IN_FRAME)))
		objs = maxi(objs, int(Performance.get_monitor(
			Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)))
	return {"name": v[0], "wall": _stats(wall), "cpu": _stats(cpu), "gpu": _stats(gpu),
		"draw_calls": draws, "primitives": prims, "objects": objs, "over_250": over}


## Mean, p50, p95 and max. Percentiles are nearest-rank on the sorted samples.
static func _stats(xs: Array[float]) -> Dictionary:
	var s := xs.duplicate()
	s.sort()
	var total := 0.0
	for x in s:
		total += x
	var n := s.size()
	return {"mean": total / n, "p50": s[_rank(n, 0.50)], "p95": s[_rank(n, 0.95)],
		"max": s[n - 1]}


static func _rank(n: int, q: float) -> int:
	return clampi(int(ceil(q * n)) - 1, 0, n - 1)


static func _memory() -> Dictionary:
	return {
		"video_mb": Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED) / 1048576.0,
		"texture_mb": Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED) / 1048576.0,
		"buffer_mb": Performance.get_monitor(Performance.RENDER_BUFFER_MEM_USED) / 1048576.0,
	}


func _machine() -> Dictionary:
	return {
		"cpu": OS.get_processor_name(),
		"os": "%s %s" % [OS.get_name(), OS.get_version()],
		"adapter": RenderingServer.get_video_adapter_name(),
		"adapter_api": RenderingServer.get_video_adapter_api_version(),
		"driver": RenderingServer.get_current_rendering_driver_name(),
		"renderer": str(ProjectSettings.get_setting("rendering/renderer/rendering_method")),
		"godot": Engine.get_version_info()["string"],
	}


# --- M-2 … M-5: one run of the protocol ------------------------------------------

func _protocol() -> void:
	var size := _render_size()
	var m := _machine()
	print("== RL-b frame cost (M-1 … M-5), %dx%d, gi=%s ==" % [size.x, size.y,
		probe.slice.gi_name()])
	print("machine  %s | %s | %s (%s %s) | Godot %s | %s" % [m["cpu"], m["os"],
		m["adapter"], m["driver"], m["adapter_api"], m["godot"], m["renderer"]])
	print("per view: settle %d, warm-up %d discarded, %d measured" % [SETTLE, WARMUP, MEASURED])
	_header()
	var views: Array = []
	var inconclusive := false
	for v in VIEWS:
		var r: Dictionary = await _measure_view(v)
		views.append(r)
		_row(r)
		if int(r["over_250"]) > 0:
			inconclusive = true
	var mem := _memory()
	if views.all(func(r: Dictionary) -> bool: return float(r["gpu"]["max"]) == 0.0):
		# Measured, not assumed (E-RLb-2): Godot 4.7.2's Metal driver reports no
		# GPU timestamps, so the gpu columns read 0. The wall interval with vsync
		# off is then the frame cost, and it is what M-5 and M-6 judge.
		print("\ngpu time     not reported by this driver (%s): the gpu columns read 0"
			% RenderingServer.get_current_rendering_driver_name())
	print("\nobjects in frame (last view)  %d" % int(
		Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)))
	print("video memory %.1f MB   texture %.1f MB   buffer %.1f MB"
		% [mem["video_mb"], mem["texture_mb"], mem["buffer_mb"]])
	print("run verdict  %s" % ("INCONCLUSIVE (a measured frame over %.0f ms)"
		% INCONCLUSIVE_MS if inconclusive else "conclusive"))
	var rec := {"protocol": "RL-b M-1..M-5", "res": [size.x, size.y],
		"gi": probe.slice.gi_name(), "machine": m, "views": views, "memory": mem,
		"inconclusive": inconclusive, "run": arg("run"),
		"time": Time.get_datetime_string_from_system()}
	var out := arg("out")
	if out != "":
		_write(out, rec)
		_aggregate(out.get_base_dir())


func _header() -> void:
	print("%-17s %8s %8s %8s %8s | %7s %7s | %7s %7s | %6s %9s %6s"
		% ["view", "mean", "p50", "p95", "max", "cpu p50", "cpu p95", "gpu p50", "gpu p95",
			"draws", "prims", "objs"])


func _row(r: Dictionary) -> void:
	var w: Dictionary = r["wall"]
	var c: Dictionary = r["cpu"]
	var g: Dictionary = r["gpu"]
	print("%-17s %8.2f %8.2f %8.2f %8.2f | %7.2f %7.2f | %7.2f %7.2f | %6d %9d %6d%s"
		% [r["name"], w["mean"], w["p50"], w["p95"], w["max"], c["p50"], c["p95"],
			g["p50"], g["p95"], r["draw_calls"], r["primitives"], r["objects"],
			"   [%d frames > %.0f ms]" % [r["over_250"], INCONCLUSIVE_MS]
			if int(r["over_250"]) > 0 else ""])


static func _write(path: String, rec: Dictionary) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		push_error("cannot write %s" % path)
		return
	f.store_string(JSON.stringify(rec, "  "))
	f.close()


## M-5 over every conclusive run recorded in this session's directory: the
## median of the runs' p95 per view, and M-6's verdict once three runs exist.
static func _aggregate(dir: String) -> void:
	var runs: Array = []
	var skipped := 0
	var names := Array(DirAccess.get_files_at(dir)).filter(
		func(n: String) -> bool: return n.ends_with(".json"))
	names.sort()
	for n in names:
		var rec: Variant = JSON.parse_string(FileAccess.get_file_as_string(dir.path_join(n)))
		if typeof(rec) != TYPE_DICTIONARY or str(rec.get("protocol", "")) != "RL-b M-1..M-5":
			continue
		if bool(rec["inconclusive"]):
			skipped += 1
			continue
		runs.append(rec)
	if runs.is_empty():
		return
	var res: Array = runs[0]["res"]
	var gi := str(runs[0]["gi"])
	print("\n== M-5 over %d conclusive run(s) (%d inconclusive, not averaged in), %dx%d, gi=%s =="
		% [runs.size(), skipped, int(res[0]), int(res[1]), gi])
	print("%-17s %14s %20s %7s %10s" % ["view", "median p95 ms", "p95 per run", "draws",
		"prims"])
	var all_ok := true
	var verdicts := {"p95": true, "draws": true, "prims": true, "vram": true}
	for i in range(VIEWS.size()):
		var p95s: Array[float] = []
		var draws := 0
		var prims := 0
		for r in runs:
			var v: Dictionary = (r["views"] as Array)[i]
			p95s.append(float((v["wall"] as Dictionary)["p95"]))
			draws = maxi(draws, int(v["draw_calls"]))
			prims = maxi(prims, int(v["primitives"]))
		var per_run := ", ".join(PackedStringArray(
			p95s.map(func(x: float) -> String: return "%.2f" % x)))
		var sorted := p95s.duplicate()
		sorted.sort()
		var med: float = sorted[sorted.size() / 2] if sorted.size() % 2 == 1 \
			else (sorted[sorted.size() / 2 - 1] + sorted[sorted.size() / 2]) * 0.5
		print("%-17s %14.2f %20s %7d %10d" % [VIEWS[i][0], med, per_run, draws, prims])
		verdicts["p95"] = verdicts["p95"] and med <= PASS_P95_MS
		verdicts["draws"] = verdicts["draws"] and draws <= PASS_DRAW_CALLS
		verdicts["prims"] = verdicts["prims"] and prims <= PASS_PRIMITIVES
	var vram := 0.0
	for r in runs:
		vram = maxf(vram, float((r["memory"] as Dictionary)["video_mb"]))
	verdicts["vram"] = vram <= PASS_VIDEO_MB
	print("video memory (max over runs)  %.1f MB" % vram)
	for k in verdicts:
		all_ok = all_ok and bool(verdicts[k])
	var applies := gi == PASS_GI and int(res[0]) == PASS_RES.x and int(res[1]) == PASS_RES.y
	if not applies:
		print("M-6  not applicable to this configuration (it is judged at gi=%s, %dx%d)"
			% [PASS_GI, PASS_RES.x, PASS_RES.y])
	elif runs.size() < 3:
		print("M-6  INCONCLUSIVE: %d of 3 conclusive runs" % runs.size())
	else:
		print("M-6  %s   p95<=%.1f ms %s | draws<=%d %s | prims<=%d %s | vram<=%.0f MB %s"
			% ["PASS" if all_ok else "FAIL", PASS_P95_MS, _ok(verdicts["p95"]),
				PASS_DRAW_CALLS, _ok(verdicts["draws"]), PASS_PRIMITIVES,
				_ok(verdicts["prims"]), PASS_VIDEO_MB, _ok(verdicts["vram"])])


static func _ok(b: bool) -> String:
	return "pass" if b else "FAIL"


# --- M-7: the breakdown ----------------------------------------------------------

func _breakdown() -> void:
	var size := _render_size()
	print("== RL-b breakdown (M-7), %dx%d, gi=%s ==" % [size.x, size.y, probe.slice.gi_name()])
	var found := {}
	for c in CATEGORIES:
		found[c] = _members(c)
		print("category %-11s %s" % [c, _describe(c, found[c])])
	var rec := {"protocol": "RL-b M-7", "res": [size.x, size.y],
		"gi": probe.slice.gi_name(), "machine": _machine(), "views": {}}
	for vn in BREAKDOWN_VIEWS:
		var v: Array = VIEWS.filter(func(x: Array) -> bool: return x[0] == vn)[0]
		print("\n-- %s --" % vn)
		print("%-12s %8s %8s %8s %8s %8s %9s %6s %s" % ["off", "p95", "d p95", "gpu p50",
			"d gpu", "draws", "prims", "objs", "d draws / d prims / d objs"])
		var base: Dictionary = await _measure_view(v)
		_brow("(none)", base, base)
		var rows := {"(none)": base}
		for c in CATEGORIES:
			if (found[c] as Array).is_empty():
				print("%-12s (nothing in this category: not measured)" % c)
				continue
			_switch(c, found[c], false)
			var r: Dictionary = await _measure_view(v)
			_switch(c, found[c], true)
			_brow(c, r, base)
			rows[c] = r
		rec["views"][vn] = rows
	var out := arg("out")
	if out != "":
		_write(out, rec)


func _brow(c: String, r: Dictionary, base: Dictionary) -> void:
	var w: Dictionary = r["wall"]
	var bw: Dictionary = base["wall"]
	var g: Dictionary = r["gpu"]
	var bg: Dictionary = base["gpu"]
	print("%-12s %8.2f %+8.2f %8.2f %+8.2f %8d %9d %6d %+d / %+d / %+d%s" % [c, w["p95"],
		w["p95"] - bw["p95"], g["p50"], g["p50"] - bg["p50"], r["draw_calls"],
		r["primitives"], r["objects"], int(r["draw_calls"]) - int(base["draw_calls"]),
		int(r["primitives"]) - int(base["primitives"]),
		int(r["objects"]) - int(base["objects"]),
		"   [INCONCLUSIVE: %d frames > %.0f ms]" % [r["over_250"], INCONCLUSIVE_MS]
		if int(r["over_250"]) > 0 else ""])


## What a category switches. Node categories are found by the `mw_category`
## meta their maker set; the rest are named engine objects.
func _members(c: String) -> Array:
	var slice: SliceMain = probe.slice
	match c:
		"merged", "gltf", "foliage", "backdrop", "ground":
			var out: Array = []
			_by_meta(slice.world, c, out)
			return out
		"characters":
			var occ: Node3D = probe.player.slot.occupant if probe.player.slot != null else null
			return [occ] if occ != null else []
		"labels":
			return slice.world.find_children("*", "Label3D", true, false)
		"voxelgi":
			return slice.find_children("*", "VoxelGI", true, false)
		"ssil", "ssao":
			return slice.find_children("*", "WorldEnvironment", true, false)
		"shadows":
			return slice.find_children("*", "DirectionalLight3D", true, false).filter(
				func(l: DirectionalLight3D) -> bool: return l.shadow_enabled)
		"occlusion":
			return [_vp] if _vp.use_occlusion_culling else []
	return []


static func _by_meta(n: Node, c: String, out: Array) -> void:
	if n.has_meta("mw_category") and str(n.get_meta("mw_category")) == c:
		out.append(n)
		return
	for ch in n.get_children():
		_by_meta(ch, c, out)


static func _describe(c: String, nodes: Array) -> String:
	if nodes.is_empty():
		return "none found"
	var tris := 0
	for n in nodes:
		if n is MeshInstance3D and (n as MeshInstance3D).mesh != null:
			var m := (n as MeshInstance3D).mesh
			for s in range(m.get_surface_count()):
				tris += m.surface_get_array_index_len(s) / 3
	return "%d node(s)%s" % [nodes.size(), "" if tris == 0 else ", %d triangles" % tris]


func _switch(c: String, nodes: Array, on: bool) -> void:
	match c:
		"ssil":
			for we in nodes:
				(we as WorldEnvironment).environment.ssil_enabled = on
		"ssao":
			for we in nodes:
				(we as WorldEnvironment).environment.ssao_enabled = on
		"shadows":
			for l in nodes:
				(l as DirectionalLight3D).shadow_enabled = on
		"occlusion":
			_vp.use_occlusion_culling = on
		_:
			for n in nodes:
				(n as Node3D).visible = on
