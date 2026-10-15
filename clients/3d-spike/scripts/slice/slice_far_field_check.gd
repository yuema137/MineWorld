## The rendered far-field check (`--slice-farfield`) and the far-field regression views
## (`--slice-farfield-shots`): RL-e0 sec.6 and sec.7, `docs/DECISIONS.md` ARC-103.
##
## WHY RENDERED. The defect this exists for was render-time culling: the ridges were in the scene,
## a winding assertion passed, and the GPU drew nothing. No schema or ray-cast check sees that, so
## this one judges pixels -- and only against frames of the same run (never a stored image, DEP-45's
## note): at fixed viewpoints and eight azimuths each, the far field shown (S), hidden (H), and the
## sky alone (E).
##
## THE TWO CRITERIA (sec.6.3; constants fixed before any run, Q-FF-1, never tuned after one):
##   FF-BELOW  of the open pixels just below the horizon, >= 90 % differ between S and H: below the
##             horizon a far field replaces all the sky.
##   FF-ABOVE  in land sectors, of the open pixels just above the horizon, >= 25 % differ: terrain
##             rises above the horizon, and a deleted or culled band leaves 0 %.
## A pixel is OPEN when H and E do not differ there (nothing near is drawn); two pixels DIFFER when
## any 8-bit channel differs by more than 8 (`tools/frame_diff.gd`'s threshold). A sector whose open
## pixels are under 2 % of its band is `covered`.
class_name SliceFarFieldCheck
extends RefCounted

# --- sec.6, fixed --------------------------------------------------------------------------------
const W := 1440
const H := 240
const HFOV := 50.0
const SECTOR_HALF_DEG := 22.5
const DIFF := 8
const N_BELOW := 0.90
const N_ABOVE := 0.25
const COVERED := 0.02
const BELOW_FROM := -1.0     # degrees, relative to e_h
const BELOW_TO := -0.1
const ABOVE_TO := 0.25
const EARTH_R := 6371000.0
const AZIMUTHS := [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0]
const SECTOR_NAMES := ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]
## sec.6.1: [id, scene position]
const VIEWPOINTS := [
	["VP-spawn", Vector3(-11.5, 1.80, -5.55)],
	["VP-street", Vector3(0.0, 1.66, 0.0)],
	["VP-cafe", Vector3(3.45, 1.80, -5.60)],
	["VP-florist", Vector3(11.84, 1.80, -5.60)],
	["VP-west", Vector3(-33.5, 1.80, 0.0)],
	["VP-east", Vector3(33.5, 1.66, 0.0)],
	["VP-high", Vector3(0.0, 40.0, 0.0)],
]

# --- sec.7, the regression views ------------------------------------------------------------------
const SHOTS_DIR := "res://shots/social-cafe/farfield"
## [name, position, yaw (0 = north, -90 = east), pitch]
const SHOTS := [
	["ff_1_north_street", Vector3(0.0, 1.8, -1.0), 0.0, 8.0],
	["ff_2_north_aerial", Vector3(0.0, 40.0, 25.0), 0.0, -4.0],
	["ff_3_south_aerial", Vector3(0.0, 40.0, -25.0), 180.0, -4.0],
	["ff_4_east_aerial", Vector3(-30.0, 40.0, 0.0), -90.0, -4.0],
	["ff_5_west_aerial", Vector3(30.0, 40.0, 0.0), 90.0, -4.0],
	["ff_6_west_street_end", Vector3(-24.0, 1.8, 0.0), 90.0, 2.0],
	["ff_7_east_street_end", Vector3(24.0, 1.8, 0.0), -90.0, 2.0],
]
const SHOT_SIZE := Vector2i(1600, 900)
const SHOT_SAVE := Vector2i(960, 540)
const SHOT_QUALITY := 0.85

var probe: SliceProbe
var _view: SliceFarFieldView
var _sv: SubViewport
var _cam: Camera3D
var _far_nodes: Array[Node3D] = []


func _init(p: SliceProbe) -> void:
	probe = p


## The check: 7 viewpoints x 8 azimuths x (S, H) + 8 x E = 120 renders.
func run() -> void:
	_setup(Vector2i(W, H))
	_cam.keep_aspect = Camera3D.KEEP_WIDTH
	_cam.fov = HFOV
	var datum := probe.slice.far_field.datum_m()
	var open_water := _open_water_sectors()
	var sky := []
	for az in AZIMUTHS:
		sky.append(await _render_at(Vector3(0.0, 1.8, 0.0), az, "E"))
	var fails := 0
	for vp in VIEWPOINTS:
		for i in AZIMUTHS.size():
			var az: float = AZIMUTHS[i]
			var shown := await _render_at(vp[1], az, "S")
			var hidden := await _render_at(vp[1], az, "H")
			var e_h := -rad_to_deg(sqrt(2.0 * (datum + float(vp[1].y)) / EARTH_R))
			var where := "%s %-2s" % [vp[0], SECTOR_NAMES[i]]
			var below := _band(shown, hidden, sky[i], e_h + BELOW_FROM, e_h + BELOW_TO)
			fails += _judge(where, "below", below, N_BELOW)
			if _in_any(az, open_water):
				print("[farfield] %s above  open water to the horizon: not held (sec.6.3)" % where)
			else:
				var above := _band(shown, hidden, sky[i], e_h, e_h + ABOVE_TO)
				fails += _judge(where, "above", above, N_ABOVE)
	_teardown()
	print("\n%s" % ("all farfield checks pass" if fails == 0 else "%d FARFIELD CHECKS FAILED" % fails))


## The regression views of sec.7, JPEG q85 at 960 x 540.
func shots() -> void:
	_setup(SHOT_SIZE)
	_cam.keep_aspect = Camera3D.KEEP_HEIGHT
	_cam.fov = CameraRig.FOV
	DirAccess.make_dir_recursive_absolute(SHOTS_DIR)
	for s in SHOTS:
		var img := await _render(s[1], float(s[2]), float(s[3]), "S")
		img.resize(SHOT_SAVE.x, SHOT_SAVE.y, Image.INTERPOLATE_LANCZOS)
		var path := "%s/%s.jpg" % [SHOTS_DIR, s[0]]
		img.save_jpg(path, SHOT_QUALITY)
		print("farfield shot %-22s at %s yaw %.0f pitch %.0f" % [s[0], s[1], s[2], s[3]])
	_teardown()


# --- rendering -----------------------------------------------------------------------------------

## One offscreen viewport sharing the slice's World3D; the main view stops drawing 3D meanwhile.
func _setup(size: Vector2i) -> void:
	var slice := probe.slice
	_view = slice.world.find_child("FarField", true, false) as SliceFarFieldView
	_far_nodes.clear()
	_collect(slice.world)
	probe.player.visible = false
	probe.get_viewport().disable_3d = true
	_sv = SubViewport.new()
	_sv.size = size
	_sv.world_3d = probe.get_viewport().find_world_3d()
	_sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	_cam = Camera3D.new()
	_cam.near = CameraRig.NEAR
	_cam.far = CameraRig.FAR
	_sv.add_child(_cam)
	probe.add_child(_sv)
	_cam.current = true
	_view.anchor_camera = _cam


func _teardown() -> void:
	_view.anchor_camera = null
	_set_far(true)
	probe.slice.world.visible = true
	probe.player.visible = true
	probe.get_viewport().disable_3d = false
	_sv.queue_free()


func _collect(n: Node) -> void:
	if n is Node3D and str(n.get_meta("mw_category", "")).begins_with("farfield."):
		_far_nodes.append(n)
		return
	for c in n.get_children():
		_collect(c)


func _set_far(shown: bool) -> void:
	for n in _far_nodes:
		n.visible = shown


## A frame from `pos`, facing `yaw_deg` (rotation about +y; 0 faces north), in mode S, H or E.
func _render(pos: Vector3, yaw_deg: float, pitch_deg: float, mode: String) -> Image:
	_set_far(mode == "S")
	probe.slice.world.visible = mode != "E"
	_cam.global_position = pos
	_cam.rotation = Vector3(deg_to_rad(pitch_deg), deg_to_rad(yaw_deg), 0.0)
	_view.follow()
	for i in 3:
		await probe.get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img := _sv.get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	return img


# --- the arithmetic ------------------------------------------------------------------------------

## A frame facing true bearing `bearing` (clockwise from north): the camera's yaw is its negation.
func _render_at(pos: Vector3, bearing: float, mode: String) -> Image:
	return await _render(pos, -bearing, 0.0, mode)


## Counts over the sector columns and the rows whose elevation lies in [lo, hi] degrees:
## [band pixels, open pixels, open pixels where S and H differ].
static func _band(s: Image, h: Image, e: Image, lo: float, hi: float) -> Array:
	var sd := s.get_data()
	var hd := h.get_data()
	var ed := e.get_data()
	var tan_v := tan(deg_to_rad(HFOV * 0.5)) * float(H) / float(W)
	var tan_h := tan(deg_to_rad(HFOV * 0.5))
	var cols := PackedInt32Array()
	for x in W:
		var off := rad_to_deg(atan((x + 0.5 - W * 0.5) / (W * 0.5) * tan_h))
		if absf(off) <= SECTOR_HALF_DEG:
			cols.append(x)
	var total := 0
	var open := 0
	var differ := 0
	for row in H:
		var elev := rad_to_deg(atan((H * 0.5 - 0.5 - row) / (H * 0.5) * tan_v))
		if elev < lo or elev > hi:
			continue
		for x in cols:
			var i := (row * W + x) * 3
			total += 1
			if _differs(hd, ed, i):
				continue
			open += 1
			if _differs(sd, hd, i):
				differ += 1
	return [total, open, differ]


static func _differs(a: PackedByteArray, b: PackedByteArray, i: int) -> bool:
	return absi(a[i] - b[i]) > DIFF or absi(a[i + 1] - b[i + 1]) > DIFF \
		or absi(a[i + 2] - b[i + 2]) > DIFF


## One verdict line (sec.6.6); returns 1 on a failure.
static func _judge(where: String, band: String, c: Array, need: float) -> int:
	var total: int = c[0]
	var open: int = c[1]
	var differ: int = c[2]
	if total == 0 or open < COVERED * total:
		print("[farfield] %s %s  covered (open %.1f%% < 2%%)"
			% [where, band, 100.0 * open / maxf(total, 1)])
		return 0
	var share := float(differ) / float(open)
	if share >= need:
		print("[farfield] %s %s %4d/%-4d open, %4d differ (%5.1f%%) >= %d%%   ok"
			% [where, band, open, total, differ, 100.0 * share, int(need * 100)])
		return 0
	print("FAIL farfield %s %s: %d/%d differ (%.1f%%) < %d%%"
		% [where.replace("  ", " ").strip_edges(), band, differ, open, 100.0 * share, int(need * 100)])
	return 1


func _open_water_sectors() -> Array:
	var out := []
	for body in probe.slice.far_field.water():
		if body["horizon"] == "open":
			out.append(body["sector"])
	return out


static func _in_any(bearing: float, sectors: Array) -> bool:
	for s in sectors:
		if MineWorldFarField.sector_contains(s, bearing):
			return true
	return false
