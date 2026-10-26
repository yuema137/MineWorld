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
	print("[landmark] prefix %s: %d nodes; %d viewpoints" % [_prefix, _hidden.size(), rows.size()])
	var fails := 0
	if rows.is_empty() or _hidden.is_empty():
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
