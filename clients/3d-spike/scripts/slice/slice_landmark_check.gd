## The rendered landmark check (`--slice-landmark`) and the station shots (`--slice-station-shots`):
## S29-d §5.9 and §5.8, `DECISIONS.md` ARC-192. RL-e0's harness (`SliceFarFieldCheck`): offscreen frames of
## the same run, never a stored image, with RL-e0's pixel rule (two pixels differ when any 8-bit channel
## differs by more than 8).
##
## THREE RENDERS per viewpoint, constants fixed before any run (Q-FF-1's rule):
##   S  everything
##   H  every node whose `mw_category` begins with the check's prefix hidden (`station.` here; S33-e's
##      pier passes `landmark.` with `--slice-landmark-prefix=`)
##   F  every near node hidden: only the sky and the far field
## A pixel is OPEN in H when H and F do not differ there: with the landmark gone, sky or far field shows.
##
##   LM-1  (each `landmark` row with a bearing) in the bearing's window of +/- 10 degrees, between the
##         horizon and the band's top (the lower west-end eaves, `StationViews.band_top`): of the open
##         pixels, >= 60 % differ between S and H -- the landmark closes the view.
##   LM-2  (each `footprint` row) of the pixels whose ray meets the ground inside the landmark's plan
##         (`StationDepot.report.footprint`), >= 60 % differ between S and H.
##
## A WORLD DRAWN FROM DATA (S33-e sec.5.3; `DECISIONS.md` ARC-178; Q-S33e-2 and Q-S33e-4 (A)) names its
## landmark's views by two roles of its far-field viewpoints, and its landmark's parts carry their own
## `mw_category` (`landmark.<name>` from a deck), so nothing here knows the world:
##   LM-1  (each `landmark_near` viewpoint) the window is the landmark's own drawn extent -- the bearings of
##         its parts that rise above the horizon (the far-field check's dip), from the horizon to their top
##         -- because a landmark narrower than the depot's 20 degrees cannot fill a fixed window; open
##         pixels are where H matches O (every near geometry instance hidden, the far field shown).
##   LM-2  (each `landmark_far` viewpoint, facing the landmark's centre) the footprint is the landmark's own
##         rendered silhouette: the pixels where X (the landmark alone) differs from E (the sky alone).
## Both pass at the same 60 %.
##
## Verdicts: `PASS landmark <view> LM-<n> <fraction>` or `FAIL ...`, then `all landmark checks pass` or
## `<n> LANDMARK CHECKS FAILED`.
class_name SliceLandmarkCheck
extends SliceFarFieldCheck

const LM_SIZE := Vector2i(960, 540)
const NEED := 0.60
## Every second pixel each way: the fractions are of a 480 x 270 lattice, fixed before any run.
const STEP := 2
const DEFAULT_PREFIX := "station."

var _prefix := DEFAULT_PREFIX
var _hidden: Array[Node3D] = []


func run() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--slice-landmark-prefix="):
			_prefix = a.trim_prefix("--slice-landmark-prefix=")
	var rows: Array = SliceWorlds.current.viewpoints().get("landmark", [])
	_setup(LM_SIZE)
	var restore := _cost_effects_off()
	_cam.keep_aspect = Camera3D.KEEP_HEIGHT
	_cam.fov = CameraRig.FOV
	_hidden.clear()
	_collect_prefix(probe.slice.world)
	var nears := SliceWorlds.current.far_field_viewpoints("landmark_near")
	var fars := SliceWorlds.current.far_field_viewpoints("landmark_far")
	print("[landmark] prefix %s: %d nodes; %d viewpoints; %d landmark_near, %d landmark_far"
		% [_prefix, _hidden.size(), rows.size(), nears.size(), fars.size()])
	var fails := 0
	if (rows.is_empty() and nears.is_empty() and fars.is_empty()) or _hidden.is_empty():
		fails += 1
		print("FAIL landmark none: this depiction states no landmark viewpoint or draws nothing under %s" % _prefix)
	for row in rows:
		var shown := await _shot(row[1], row[2], row[3], "S")
		var hidden := await _shot(row[1], row[2], row[3], "H")
		var far := await _shot(row[1], row[2], row[3], "F")
		if typeof(row[4]) == TYPE_STRING:
			fails += _judge_lm(row[0], "LM-2", _footprint(shown, hidden))
		else:
			print("[landmark] %s: bearing %.0f deg +/- %.0f, band 0 .. %.2f deg" % [row[0], row[4], StationViews.WINDOW_DEG,
				row[5]])
			fails += _judge_lm(row[0], "LM-1", _window(shown, hidden, far, float(row[4]), float(row[5])))
	if not (nears.is_empty() and fars.is_empty()) and not _hidden.is_empty():
		_near.clear()
		_gather_near(probe.slice.world)
		var aabbs := _aabbs()
		_cam.keep_aspect = Camera3D.KEEP_WIDTH
		_sv.size = EXTENT_SIZE
		for vp in nears:
			fails += await _extent_lm1(vp[0], vp[1], aabbs)
		_sv.size = SILHOUETTE_SIZE
		for vp in fars:
			fails += await _silhouette_lm2(vp[0], vp[1], aabbs)
		_show("S")
	for n in _hidden:
		n.visible = true
	restore.call()
	_teardown()
	print("\n%s" % ("all landmark checks pass" if fails == 0 else "%d LANDMARK CHECKS FAILED" % fails))


## The station shots (S29-d §5.8): JPEG q85 at 960 x 540 under shots/<world>/station, RL-e0's settings.
func station_shots() -> void:
	_setup(SHOT_SIZE)
	_cam.keep_aspect = Camera3D.KEEP_HEIGHT
	_cam.fov = CameraRig.FOV
	var dir := "res://shots/%s/station" % probe.slice.far_field.path.get_file().get_basename()
	DirAccess.make_dir_recursive_absolute(dir)
	for s in SliceWorlds.current.viewpoints().get("station_shots", []):
		var img := await _render(s[1], float(s[2]), float(s[3]), "S")
		img.resize(SHOT_SAVE.x, SHOT_SAVE.y, Image.INTERPOLATE_LANCZOS)
		img.save_jpg("%s/%s.jpg" % [dir, s[0]], SHOT_QUALITY)
		print("station shot %-26s at %s yaw %.0f pitch %.0f" % [s[0], s[1], s[2], s[3]])
	_teardown()


func _collect_prefix(n: Node) -> void:
	if n is Node3D and str(n.get_meta("mw_category", "")).begins_with(_prefix):
		_hidden.append(n)
		return
	for c in n.get_children():
		_collect_prefix(c)


## A frame in mode S, H or F (above).
func _shot(pos: Vector3, yaw_deg: float, pitch_deg: float, mode: String) -> Image:
	_set_far(true)
	probe.slice.world.visible = true
	for n in _slice_nodes:
		n.visible = mode != "F"
	for n in _hidden:
		n.visible = mode == "S"
	_cam.global_position = pos
	_cam.rotation = Vector3(deg_to_rad(pitch_deg), deg_to_rad(yaw_deg), 0.0)
	_view.follow()
	for i in 3:
		await probe.get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img := _sv.get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	return img


## LM-1's counts: [open pixels in the window's band, of them differing between S and H].
func _window(s: Image, h: Image, f: Image, bearing: float, top_deg: float) -> Array:
	var sd := s.get_data()
	var hd := h.get_data()
	var fd := f.get_data()
	var open := 0
	var differ := 0
	for y in range(0, LM_SIZE.y, STEP):
		for x in range(0, LM_SIZE.x, STEP):
			var ray := _cam.project_ray_normal(Vector2(x + 0.5, y + 0.5))
			var elev := rad_to_deg(asin(clampf(ray.y, -1.0, 1.0)))
			if elev < 0.0 or elev > top_deg:
				continue
			var bear := fposmod(rad_to_deg(atan2(ray.x, -ray.z)), 360.0)
			if absf(angle_difference(deg_to_rad(bear), deg_to_rad(bearing))) > deg_to_rad(StationViews.WINDOW_DEG):
				continue
			var i := (y * LM_SIZE.x + x) * 3
			if _differs(hd, fd, i):
				continue
			open += 1
			if _differs(sd, hd, i):
				differ += 1
	return [open, differ]


## LM-2's counts: [pixels whose ray meets the ground inside the plan, of them differing between S and H].
func _footprint(s: Image, h: Image) -> Array:
	var sd := s.get_data()
	var hd := h.get_data()
	var plan: Array = StationDepot.report.get("footprint", [])
	var ground := float(StationDepot.report.get("floor_y", 0.0))
	var inside := 0
	var differ := 0
	for y in range(0, LM_SIZE.y, STEP):
		for x in range(0, LM_SIZE.x, STEP):
			var ray := _cam.project_ray_normal(Vector2(x + 0.5, y + 0.5))
			if ray.y > -0.001:
				continue
			var p := _cam.global_position + ray * ((ground - _cam.global_position.y) / ray.y)
			var hit := false
			for r in plan:
				hit = hit or (r as Rect2).has_point(Vector2(p.x, p.z))
			if not hit:
				continue
			inside += 1
			if _differs(sd, hd, (y * LM_SIZE.x + x) * 3):
				differ += 1
	return [inside, differ]


static func _judge_lm(view: String, criterion: String, c: Array) -> int:
	var n: int = c[0]
	var share := float(c[1]) / float(maxi(n, 1))
	var ok := n > 0 and share >= NEED
	print("%s landmark %s %s %.3f   (%d of %d pixels differ; need %.2f)" % ["PASS" if ok else "FAIL", view,
		criterion, share, c[1], n, NEED])
	return 0 if ok else 1


# --- a world drawn from data: the roles `landmark_near` and `landmark_far` (S33-e sec.5.3) -------------

## LM-1's frame: wide and tall enough for a landmark rising a few degrees above the horizon; LM-2's frame.
const EXTENT_SIZE := Vector2i(1440, 480)
const EXTENT_HFOV := 50.0
const SILHOUETTE_SIZE := Vector2i(960, 720)
const SILHOUETTE_HFOV := 60.0

var _near: Array[GeometryInstance3D] = []


## LM-1 in the landmark's own drawn extent, from `eye`.
func _extent_lm1(id: String, eye: Vector3, aabbs: Array) -> int:
	var datum := probe.slice.far_field.datum_m()
	var e_h := -rad_to_deg(sqrt(2.0 * maxf(datum + eye.y, 0.0) / EARTH_R))
	var bearings: Array[float] = []
	var top := -INF
	for box in aabbs:
		for k in 8:
			var c := (box as AABB).get_endpoint(k)
			var el := _elevation(eye, c)
			if el > e_h:
				bearings.append(_bearing(eye, c))
				top = maxf(top, el)
	if bearings.is_empty():
		print("FAIL landmark %s LM-1: nothing under %s rises above the horizon from here" % [id, _prefix])
		return 1
	var centre := _mid_bearing(bearings)
	var lo := INF
	var hi := -INF
	for b in bearings:
		var off := wrapf(b - centre, -180.0, 180.0)
		lo = minf(lo, off)
		hi = maxf(hi, off)
	_cam.fov = EXTENT_HFOV
	var sd := (await _frame(eye, centre, 0.0, "S")).get_data()
	var hd := (await _frame(eye, centre, 0.0, "H")).get_data()
	var od := (await _frame(eye, centre, 0.0, "O")).get_data()
	var w := EXTENT_SIZE.x
	var hh := EXTENT_SIZE.y
	var tan_h := tan(deg_to_rad(EXTENT_HFOV * 0.5))
	var tan_v := tan_h * float(hh) / float(w)
	var open := 0
	var differ := 0
	for row in hh:
		var el := rad_to_deg(atan((hh * 0.5 - 0.5 - row) / (hh * 0.5) * tan_v))
		if el < e_h or el > top:
			continue
		for x in w:
			var az := rad_to_deg(atan((x + 0.5 - w * 0.5) / (w * 0.5) * tan_h))
			if az < lo or az > hi:
				continue
			var i := (row * w + x) * 3
			if _differs(hd, od, i):
				continue
			open += 1
			if _differs(sd, hd, i):
				differ += 1
	print("[landmark] %s: the landmark's extent, bearings %.1f .. %.1f deg, band %.2f .. %.2f deg"
		% [id, fposmod(centre + lo, 360.0), fposmod(centre + hi, 360.0), e_h, top])
	return _judge_lm(id, "LM-1", [open, differ])


## LM-2 over the landmark's rendered silhouette, from `eye`, facing its centre.
func _silhouette_lm2(id: String, eye: Vector3, aabbs: Array) -> int:
	if aabbs.is_empty():
		print("FAIL landmark %s LM-2: nothing under %s is drawn" % [id, _prefix])
		return 1
	var all := aabbs[0] as AABB
	for box in aabbs:
		all = all.merge(box)
	var c := all.get_center()
	var bearing := _bearing(eye, c)
	var pitch := _elevation(eye, c)
	_cam.fov = SILHOUETTE_HFOV
	var s := (await _frame(eye, bearing, pitch, "S")).get_data()
	var h := (await _frame(eye, bearing, pitch, "H")).get_data()
	var x := (await _frame(eye, bearing, pitch, "X")).get_data()
	var e := (await _frame(eye, bearing, pitch, "E")).get_data()
	var footprint := 0
	var differ := 0
	for i in range(0, s.size(), 3):
		if not _differs(x, e, i):
			continue
		footprint += 1
		if _differs(s, h, i):
			differ += 1
	print("[landmark] %s: the landmark's silhouette, facing %.1f deg, pitch %.1f deg" % [id, bearing, pitch])
	return _judge_lm(id, "LM-2", [footprint, differ])


## Every geometry instance that is neither the landmark's nor the far field's.
func _gather_near(n: Node) -> void:
	if n == _view or n == probe.player or (n is Node3D and _hidden.has(n as Node3D)) \
			or (n is Node3D and str(n.get_meta("mw_category", "")).begins_with("farfield.")):
		return
	if n is GeometryInstance3D:
		_near.append(n)
	for c in n.get_children():
		_gather_near(c)


## The world AABB of every visual part of the landmark.
func _aabbs() -> Array:
	var out := []
	for root in _hidden:
		for v in root.find_children("*", "VisualInstance3D", true, false):
			var vi := v as VisualInstance3D
			if vi.get_aabb().size != Vector3.ZERO:
				out.append(vi.global_transform * vi.get_aabb())
	return out


## The frame's visibility: S everything, H the landmark hidden, O the far field and sky only, X the
## landmark alone, E the sky alone.
func _show(mode: String) -> void:
	probe.slice.world.visible = true
	for n in _slice_nodes:
		n.visible = true
	for n in _hidden:
		n.visible = mode in ["S", "X"]
	for g in _near:
		g.visible = mode in ["S", "H"]
	_set_far(mode in ["S", "H", "O"])


## A frame from `eye`, facing true bearing `bearing` (clockwise from north), pitched `pitch` degrees.
func _frame(eye: Vector3, bearing: float, pitch: float, mode: String) -> Image:
	_show(mode)
	_cam.global_position = eye
	_cam.rotation = Vector3(deg_to_rad(pitch), deg_to_rad(-bearing), 0.0)
	_view.follow()
	for i in 3:
		await probe.get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img := _sv.get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	return img


## True bearing of `p` from `eye`, degrees clockwise from north (-z).
static func _bearing(eye: Vector3, p: Vector3) -> float:
	return fposmod(rad_to_deg(atan2(p.x - eye.x, -(p.z - eye.z))), 360.0)


static func _elevation(eye: Vector3, p: Vector3) -> float:
	var flat := Vector2(p.x - eye.x, p.z - eye.z).length()
	return rad_to_deg(atan2(p.y - eye.y, flat))


## The bearing in the middle of a set of bearings.
static func _mid_bearing(bearings: Array[float]) -> float:
	var v := Vector2.ZERO
	for b in bearings:
		v += Vector2(sin(deg_to_rad(b)), cos(deg_to_rad(b)))
	var mean := fposmod(rad_to_deg(atan2(v.x, v.y)), 360.0)
	var lo := INF
	var hi := -INF
	for b in bearings:
		var off := wrapf(b - mean, -180.0, 180.0)
		lo = minf(lo, off)
		hi = maxf(hi, off)
	return fposmod(mean + (lo + hi) * 0.5, 360.0)
