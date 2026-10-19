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
##
## THE ID RENDER (RL-e0b sec.6, `docs/decisions/ARC-290.md`): from the "judged" viewpoints the far
## field is also drawn in flat identifying colours (I), and from the "high" one looking down at the
## ring (G); `slice_far_field_ids.gd` judges position, order, seams, texture, the ring and the bay
## from those frames. The viewpoints are the street's (`SliceWorld.FAR_FIELD_VIEWPOINTS`); every
## expected value comes from the far-field file the slice loaded.
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

# --- sec.7, the regression views (SliceWorld.FAR_FIELD_SHOTS), under shots/<world>/farfield -------
const SHOT_SIZE := Vector2i(1600, 900)
const SHOT_SAVE := Vector2i(960, 540)
const SHOT_QUALITY := 0.85

var probe: SliceProbe
var _view: SliceFarFieldView
var _sv: SubViewport
var _cam: Camera3D
var _far_nodes: Array[Node3D] = []
var _slice_nodes: Array[Node3D] = []


func _init(p: SliceProbe) -> void:
	probe = p


## The check: 7 viewpoints x 8 azimuths x (S, H) + 8 x E = 120 renders, then the ID renders of
## RL-e0b sec.6.1: 3 judged viewpoints x 8 azimuths (I) + 8 looking down (G) = 32.
func run() -> void:
	_setup(Vector2i(W, H))
	var restore := _cost_effects_off()
	_cam.keep_aspect = Camera3D.KEEP_WIDTH
	_cam.fov = HFOV
	var datum := probe.slice.far_field.datum_m()
	var open_water := _open_water_sectors()
	var sky := []
	for az in AZIMUTHS:
		sky.append(await _render_at(Vector3(0.0, 1.8, 0.0), az, "E"))
	var fails := 0
	var judged := SliceWorld.far_field_viewpoints("judged").map(func(v: Array) -> String: return v[0])
	var kept := {}           # judged viewpoint id -> [[S, H] per azimuth]
	for vp in SliceWorld.far_field_viewpoints(""):
		# a progress line per viewpoint, so a slow renderer is told from a hung one (ARC-291)
		print("[farfield] progress: viewpoint %s at %d s" % [vp[0], Time.get_ticks_msec() / 1000])
		for i in AZIMUTHS.size():
			var az: float = AZIMUTHS[i]
			var shown := await _render_at(vp[1], az, "S")
			var hidden := await _render_at(vp[1], az, "H")
			if vp[0] in judged:
				if not kept.has(vp[0]):
					kept[vp[0]] = []
				kept[vp[0]].append([shown.get_data(), hidden.get_data()])
			var e_h := -rad_to_deg(sqrt(2.0 * (datum + float(vp[1].y)) / EARTH_R))
			var where := "%s %-2s" % [vp[0], SECTOR_NAMES[i]]
			var below := _band(shown, hidden, sky[i], e_h + BELOW_FROM, e_h + BELOW_TO)
			fails += _judge(where, "below", below, N_BELOW)
			if _in_any(az, open_water):
				print("[farfield] %s above  open water to the horizon: not held (sec.6.3)" % where)
			else:
				var above := _band(shown, hidden, sky[i], e_h, e_h + ABOVE_TO)
				fails += _judge(where, "above", above, N_ABOVE)
	fails += await _id_criteria(kept, sky)
	restore.call()
	_teardown()
	print("\n%s" % ("all farfield checks pass" if fails == 0 else "%d FARFIELD CHECKS FAILED" % fails))


## The regression views of sec.7, JPEG q85 at 960 x 540.
func shots() -> void:
	_setup(SHOT_SIZE)
	_cam.keep_aspect = Camera3D.KEEP_HEIGHT
	_cam.fov = CameraRig.FOV
	var dir := "res://shots/%s/farfield" % probe.slice.far_field.path.get_file().get_basename()
	DirAccess.make_dir_recursive_absolute(dir)
	for s in SliceWorld.FAR_FIELD_SHOTS:
		var img := await _render(s[1], float(s[2]), float(s[3]), "S")
		img.resize(SHOT_SAVE.x, SHOT_SAVE.y, Image.INTERPOLATE_LANCZOS)
		var path := "%s/%s.jpg" % [dir, s[0]]
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
	_slice_nodes.clear()
	for c in slice.world.get_children():
		if c is Node3D and c != _view:
			_slice_nodes.append(c)
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


## R-FF-1, ruled 2026-10-15 (operator answered directly in the primary session; option (b)): for the
## check frames only, the slice's cost-only effects are off -- directional shadows, SSAO and glow. None
## decides where the far field is drawn (it casts no shadow, and its ribbons are unshaded and fog-free),
## and S and H are rendered alike, so the criteria are unchanged; on a software renderer they made each
## render take about a minute. The regression views (`shots`) keep the full look. Returns the restorer.
func _cost_effects_off() -> Callable:
	var env: Environment = probe.slice._env
	var lights: Array = probe.slice.find_children("*", "DirectionalLight3D", true, false).filter(
		func(l: DirectionalLight3D) -> bool: return l.shadow_enabled)
	var was := [env.ssao_enabled, env.glow_enabled]
	env.ssao_enabled = false
	env.glow_enabled = false
	for l in lights:
		(l as DirectionalLight3D).shadow_enabled = false
	print("[farfield] check frames: shadows (%d lights), SSAO and glow off (R-FF-1 (b))" % lights.size())
	return func() -> void:
		env.ssao_enabled = was[0]
		env.glow_enabled = was[1]
		for l in lights:
			(l as DirectionalLight3D).shadow_enabled = true


func _teardown() -> void:
	_view.anchor_camera = null
	_set_far(true)
	for n in _slice_nodes:
		n.visible = true
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


## A frame from `pos`, facing `yaw_deg` (rotation about +y; 0 faces north), in mode S, H or E, or
## I (the far field alone, in ID colours) or G (everything, the far field in ID colours).
func _render(pos: Vector3, yaw_deg: float, pitch_deg: float, mode: String) -> Image:
	_set_far(mode in ["S", "I", "G"])
	probe.slice.world.visible = mode != "E"
	for n in _slice_nodes:
		n.visible = mode != "I"
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


# --- RL-e0b sec.6: the ID renders -----------------------------------------------------------------

## The I and G renders and their criteria; returns the number of failures.
func _id_criteria(kept: Dictionary, sky: Array) -> int:
	var ids := SliceFarFieldIds.new(probe.slice.far_field)
	var undo := _id_state_on(ids)
	var fails := 0
	var protected := SliceWorld.far_field_viewpoints("protected").map(
		func(v: Array) -> String: return v[0])
	for vp in SliceWorld.far_field_viewpoints("judged"):
		for i in AZIMUTHS.size():
			var az: float = AZIMUTHS[i]
			var img := await _render_at(vp[1], az, "I")
			var where := "%s %-2s" % [vp[0], SECTOR_NAMES[i]]
			var pair: Array = kept[vp[0]][i]
			fails += ids.judge_frame(where, vp[1], az, ids.decode(img), pair[0], pair[1],
				(sky[i] as Image).get_data(), vp[0] in protected)
	_sv.size = SliceFarFieldIds.G_SIZE
	_cam.fov = SliceFarFieldIds.G_HFOV
	for vp in SliceWorld.far_field_viewpoints("high"):
		for i in AZIMUTHS.size():
			var az: float = AZIMUTHS[i]
			var img := await _render(vp[1], -az, SliceFarFieldIds.G_PITCH, "G")
			fails += ids.judge_ring("%s %-2s" % [vp[0], SECTOR_NAMES[i]], vp[1], az, ids.decode(img))
	_sv.size = Vector2i(W, H)
	_cam.fov = HFOV
	undo.call()
	return fails


## sec.6.2: every far-field mesh in its flat ID colour (a band through its shader's id_mode, so its
## parallax stays), the sky a flat background colour, and nothing that changes a colour after it is
## drawn: no fog, no colour adjustment, a linear tonemapper at exposure 1, no anti-aliasing or
## debanding. Returns the restorer.
func _id_state_on(ids: SliceFarFieldIds) -> Callable:
	var swapped := []
	for entry in _view.ids():
		var mi := entry[0] as MeshInstance3D
		var colour := ids.colour_of(entry[1])
		var mat := mi.material_override
		if mat is ShaderMaterial and (mat as ShaderMaterial).shader.resource_path.ends_with(
				"far_silhouette.gdshader"):
			(mat as ShaderMaterial).set_shader_parameter("id_mode", true)
			(mat as ShaderMaterial).set_shader_parameter("id_colour", colour)
		else:
			var flat := StandardMaterial3D.new()
			flat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			flat.albedo_color = colour
			flat.disable_fog = true
			mi.material_override = flat
		swapped.append([mi, mat])
	var env: Environment = probe.slice._env
	var was := [env.background_mode, env.background_color, env.fog_enabled,
		env.volumetric_fog_enabled, env.adjustment_enabled, env.tonemap_mode, env.tonemap_exposure,
		env.tonemap_white, _sv.screen_space_aa, _sv.msaa_3d, _sv.use_debanding, _sv.use_taa]
	env.background_mode = Environment.BG_COLOR
	env.background_color = SliceFarFieldIds.SKY
	env.fog_enabled = false
	env.volumetric_fog_enabled = false
	env.adjustment_enabled = false
	env.tonemap_mode = Environment.TONE_MAPPER_LINEAR
	env.tonemap_exposure = 1.0
	env.tonemap_white = 1.0
	_sv.screen_space_aa = Viewport.SCREEN_SPACE_AA_DISABLED
	_sv.msaa_3d = Viewport.MSAA_DISABLED
	_sv.use_debanding = false
	_sv.use_taa = false
	return func() -> void:
		for s in swapped:
			var mi := s[0] as MeshInstance3D
			if s[1] is ShaderMaterial and mi.material_override == s[1]:
				(s[1] as ShaderMaterial).set_shader_parameter("id_mode", false)
			mi.material_override = s[1]
		env.background_mode = was[0]
		env.background_color = was[1]
		env.fog_enabled = was[2]
		env.volumetric_fog_enabled = was[3]
		env.adjustment_enabled = was[4]
		env.tonemap_mode = was[5]
		env.tonemap_exposure = was[6]
		env.tonemap_white = was[7]
		_sv.screen_space_aa = was[8]
		_sv.msaa_3d = was[9]
		_sv.use_debanding = was[10]
		_sv.use_taa = was[11]
