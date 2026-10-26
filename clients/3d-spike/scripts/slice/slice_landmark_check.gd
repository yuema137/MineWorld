## The rendered landmark check (`--slice-landmark`): step-29 sec.9.2's S/H method, with the hidden category a
## parameter (S33-e sec.5.3; Q-S33e-2, ruled 2026-10-21) -- `landmark.` by default (a world whose landmark is
## drawn from data, `DECISIONS.md` ARC-178), `--slice-landmark-category=station.` for the towns' depots.
##
## WHY RENDERED. A landmark is a landmark because it is seen: it closes a view and it stands out from afar.
## A schema check or a ray sees that it exists, not that the GPU drew it where a player looks. So this judges
## pixels, and only against frames of the same run (RL-e0's rule, `SliceFarFieldCheck`).
##
## THE FRAMES, from each viewpoint, facing the landmark:
##   S  everything shown          H  the landmark's category hidden
##   O  the far field and sky only (what is OPEN: nothing near is drawn)
##   X  the landmark alone        E  the sky alone
##
## THE TWO CRITERIA (constants fixed before any run; never tuned after one):
##   LM-1 (closes the view)  from every viewpoint with the role `landmark_near`: in the landmark's own drawn
##        extent -- the azimuths of its parts that rise above the horizon, from the horizon to their top
##        (Q-S33e-4 (A), computed from the geometry) -- at least 60 % of the pixels open in H (H and O agree)
##        differ between S and H.
##   LM-2 (seen from afar)   from every viewpoint with the role `landmark_far`: of the landmark's projected
##        footprint (the pixels where X differs from E), at least 60 % differ between S and H.
## Two pixels DIFFER when any 8-bit channel differs by more than 8 (`SliceFarFieldCheck.DIFF`).
## Verdicts `all landmark checks pass` / `<n> LANDMARK CHECKS FAILED`; a failure names the viewpoint and the
## criterion (`FAIL landmark VP-L-2 LM-1 ...`).
class_name SliceLandmarkCheck
extends SliceFarFieldCheck

const CATEGORY_ARG := "--slice-landmark-category="
const DEFAULT_CATEGORY := "landmark."
const NEED := 0.60
## LM-1's frame: wide and tall enough for a landmark that rises a few degrees above the horizon.
const LM1_SIZE := Vector2i(1440, 480)
const LM1_HFOV := 50.0
## LM-2's frame.
const LM2_SIZE := Vector2i(960, 720)
const LM2_HFOV := 60.0

var category := DEFAULT_CATEGORY
var _landmark: Array[Node3D] = []
var _near: Array[GeometryInstance3D] = []


func _init(p: SliceProbe) -> void:
	super(p)
	for a in OS.get_cmdline_user_args():
		if a.begins_with(CATEGORY_ARG):
			category = a.trim_prefix(CATEGORY_ARG)


## The check: LM-1 at every `landmark_near` viewpoint and LM-2 at every `landmark_far` one.
func run() -> void:
	_setup(LM1_SIZE)
	var restore := _cost_effects_off()
	_cam.keep_aspect = Camera3D.KEEP_WIDTH
	_landmark.clear()
	_near.clear()
	_gather(probe.slice.world)
	print("[landmark] category %s*: %d drawn parts; %d near geometry instances"
		% [category, _landmark.size(), _near.size()])
	var fails := 0
	var nears := SliceWorlds.current.far_field_viewpoints("landmark_near")
	var fars := SliceWorlds.current.far_field_viewpoints("landmark_far")
	if nears.is_empty() and fars.is_empty():
		print("FAIL landmark: the depiction names no landmark_near or landmark_far viewpoint")
		fails += 1
	var aabbs := _aabbs()
	for vp in nears:
		fails += await _lm1(vp[0], vp[1], aabbs)
	_sv.size = LM2_SIZE
	for vp in fars:
		fails += await _lm2(vp[0], vp[1], aabbs)
	_show("S")
	restore.call()
	_teardown()
	print("\n%s" % ("all landmark checks pass" if fails == 0 else "%d LANDMARK CHECKS FAILED" % fails))


# --- the criteria --------------------------------------------------------------------------------

func _lm1(id: String, eye: Vector3, aabbs: Array) -> int:
	var where := "%s LM-1" % id
	var datum := probe.slice.far_field.datum_m()
	var e_h := -rad_to_deg(sqrt(2.0 * maxf(datum + eye.y, 0.0) / EARTH_R))
	# the parts that rise above the horizon: their bearings and their top
	var bearings: Array[float] = []
	var top := -INF
	for box in aabbs:
		for c in _corners(box):
			var el := _elevation(eye, c)
			if el > e_h:
				bearings.append(_bearing(eye, c))
				top = maxf(top, el)
	if bearings.is_empty():
		print("FAIL landmark %s: nothing tagged %s* rises above the horizon from here" % [where, category])
		return 1
	var centre := _mid_bearing(bearings)
	var lo := INF
	var hi := -INF
	for b in bearings:
		var off := wrapf(b - centre, -180.0, 180.0)
		lo = minf(lo, off)
		hi = maxf(hi, off)
	_cam.fov = LM1_HFOV
	var s := await _frame(eye, centre, 0.0, "S")
	var h := await _frame(eye, centre, 0.0, "H")
	var o := await _frame(eye, centre, 0.0, "O")
	var sd := s.get_data()
	var hd := h.get_data()
	var od := o.get_data()
	var w := LM1_SIZE.x
	var hh := LM1_SIZE.y
	var tan_h := tan(deg_to_rad(LM1_HFOV * 0.5))
	var tan_v := tan_h * float(hh) / float(w)
	var total := 0
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
			total += 1
			if _differs(hd, od, i):
				continue
			open += 1
			if _differs(sd, hd, i):
				differ += 1
	var window := "bearings %.1f-%.1f deg, elevation %.2f-%.2f deg" % [
		fposmod(centre + lo, 360.0), fposmod(centre + hi, 360.0), e_h, top]
	return _judge_lm(where, window, open, differ, total)


func _lm2(id: String, eye: Vector3, aabbs: Array) -> int:
	var where := "%s LM-2" % id
	if aabbs.is_empty():
		print("FAIL landmark %s: nothing tagged %s* is drawn" % [where, category])
		return 1
	var all := aabbs[0] as AABB
	for box in aabbs:
		all = all.merge(box)
	var c := all.get_center()
	var bearing := _bearing(eye, c)
	var pitch := _elevation(eye, c)
	_cam.fov = LM2_HFOV
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
	var window := "facing %.1f deg, pitch %.1f deg" % [bearing, pitch]
	return _judge_lm(where, window, footprint, differ, LM2_SIZE.x * LM2_SIZE.y)


func _judge_lm(where: String, window: String, open: int, differ: int, total: int) -> int:
	if open == 0:
		print("FAIL landmark %s: no open pixel in its window (%s; %d pixels)" % [where, window, total])
		return 1
	var share := float(differ) / float(open)
	if share >= NEED:
		print("[landmark] %s %d/%d differ (%.1f%%) >= %d%%   ok  (%s)"
			% [where, differ, open, 100.0 * share, int(NEED * 100), window])
		return 0
	print("FAIL landmark %s: %d/%d differ (%.1f%%) < %d%% (%s)"
		% [where, differ, open, 100.0 * share, int(NEED * 100), window])
	return 1


# --- the scene ----------------------------------------------------------------------------------

## The landmark's parts (by category) and every other near geometry instance (not the far field's).
func _gather(n: Node) -> void:
	if n == _view or (n is Node3D and str(n.get_meta("mw_category", "")).begins_with("farfield.")):
		return
	if n is Node3D and str(n.get_meta("mw_category", "")).begins_with(category):
		_landmark.append(n)
		return
	if n is GeometryInstance3D and n != probe.player:
		_near.append(n)
	for c in n.get_children():
		_gather(c)


## The world AABB of every visual part of the landmark.
func _aabbs() -> Array:
	var out := []
	for root in _landmark:
		for v in root.find_children("*", "VisualInstance3D", true, false):
			var vi := v as VisualInstance3D
			if vi.get_aabb().size != Vector3.ZERO:
				out.append(vi.global_transform * vi.get_aabb())
	return out


## The frame's visibility: S, H, O, X or E (above).
func _show(mode: String) -> void:
	for n in _landmark:
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


static func _corners(box: AABB) -> Array:
	var out := []
	for i in 8:
		out.append(box.get_endpoint(i))
	return out


## True bearing of `p` from `eye`, degrees clockwise from north (-z).
static func _bearing(eye: Vector3, p: Vector3) -> float:
	return fposmod(rad_to_deg(atan2(p.x - eye.x, -(p.z - eye.z))), 360.0)


static func _elevation(eye: Vector3, p: Vector3) -> float:
	var flat := Vector2(p.x - eye.x, p.z - eye.z).length()
	return rad_to_deg(atan2(p.y - eye.y, flat))


## The bearing in the middle of a set of bearings (by their mean direction).
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
