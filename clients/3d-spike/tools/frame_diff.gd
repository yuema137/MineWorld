extends SceneTree
## How different two captured frames are (step-15 §19.3 J-4).
##
##   godot --headless --path clients/3d-spike --script res://tools/frame_diff.gd -- <a> <b> [threshold]
##         [--mask=<png or dir>] [--heat=<dir>]
##
## <a> and <b> are two PNGs, or two directories, in which case every PNG in <a>
## is compared with the file of the same name in <b>. Per pair it prints the
## mean absolute difference (0-255, averaged over R, G and B), the share of
## pixels whose largest channel difference exceeds `threshold` (default 8), and
## the bounding box of those pixels -- so a change can be found and looked at,
## not only counted. Exits 1 if a pair cannot be compared.
##
## Two optional arguments, added by RL-b (§7 V-1, V-5); without them the output
## is exactly as before:
##   --mask=<png or dir>  pixels that are not black in the mask (the same-named
##                        file, for a directory) are left out of both counts; the
##                        share is then of the unmasked pixels, and the line says
##                        how many were masked
##                        several masks, comma-separated, are united
##   --heat=<dir>         writes <dir>/<name>: the frame <b> darkened to a third,
##                        with every pixel over the threshold drawn in red, so a
##                        change is located by eye rather than by its box
##   --write-mask=<dir>   writes <dir>/<name>: white where a pixel differs by
##                        more than the threshold, black elsewhere -- a mask for
##                        --mask (V-5: a frame with and without the backdrop)
var _write_mask := ""


func _init() -> void:
	var pos: Array[String] = []
	var mask := ""
	var heat := ""
	for x in OS.get_cmdline_user_args():
		if x.begins_with("--mask="):
			mask = x.substr(7)
		elif x.begins_with("--heat="):
			heat = ProjectSettings.globalize_path(x.substr(7))
		elif x.begins_with("--write-mask="):
			_write_mask = ProjectSettings.globalize_path(x.substr(13))
			DirAccess.make_dir_recursive_absolute(_write_mask)
		else:
			pos.append(x)
	if pos.size() < 2:
		print("need <a> <b> [threshold] [--mask=<png or dir>] [--heat=<dir>]")
		quit(1)
		return
	var threshold := int(pos[2]) if pos.size() > 2 else 8
	var left := ProjectSettings.globalize_path(pos[0])
	var right := ProjectSettings.globalize_path(pos[1])
	if heat != "":
		DirAccess.make_dir_recursive_absolute(heat)
	var ok := true
	if DirAccess.dir_exists_absolute(left):
		var names := Array(DirAccess.get_files_at(left)).filter(
			func(n: String) -> bool: return n.ends_with(".png"))
		names.sort()
		for n in names:
			ok = _pair(left.path_join(n), right.path_join(n), n, threshold, _masks(mask, n),
				heat) and ok
	else:
		ok = _pair(left, right, left.get_file(), threshold, _masks(mask, ""), heat)
	quit(0 if ok else 1)


## The mask files for frame `n`: each comma-separated entry is a PNG, or a
## directory holding one of the frame's name.
static func _masks(spec: String, n: String) -> Array[String]:
	var out: Array[String] = []
	if spec == "":
		return out
	for p in spec.split(","):
		var g := ProjectSettings.globalize_path(p)
		out.append(g.path_join(n) if n != "" and DirAccess.dir_exists_absolute(g) else g)
	return out


func _pair(pa: String, pb: String, label: String, threshold: int, mask_paths: Array[String],
		heat: String) -> bool:
	var ia := Image.load_from_file(pa)
	var ib := Image.load_from_file(pb) if FileAccess.file_exists(pb) else null
	if ia == null or ib == null:
		print("%-40s cannot compare (missing or unreadable)" % label)
		return false
	if ia.get_size() != ib.get_size():
		print("%-40s cannot compare: %s vs %s" % [label, ia.get_size(), ib.get_size()])
		return false
	ia.convert(Image.FORMAT_RGB8)
	ib.convert(Image.FORMAT_RGB8)
	var dm := PackedByteArray()
	for mask_path in mask_paths:
		if not FileAccess.file_exists(mask_path):
			print("%-40s cannot compare: no mask %s" % [label, mask_path])
			return false
		var im := Image.load_from_file(mask_path)
		if im == null or im.get_size() != ia.get_size():
			print("%-40s cannot compare: mask unreadable or of another size" % label)
			return false
		im.convert(Image.FORMAT_L8)
		if dm.is_empty():
			dm = im.get_data()
		else:
			var d2 := im.get_data()
			for i in range(dm.size()):
				dm[i] = maxi(dm[i], d2[i])
	var wm := PackedByteArray()
	if _write_mask != "":
		wm.resize(ia.get_width() * ia.get_height())
	var da := ia.get_data()
	var db := ib.get_data()
	var hd := PackedByteArray()
	if heat != "":
		hd = db.duplicate()
		for i in range(hd.size()):
			hd[i] = hd[i] / 3
	var w := ia.get_width()
	var total := 0
	var over := 0
	var masked := 0
	var lo := Vector2i(w, ia.get_height())
	var hi := Vector2i(-1, -1)
	for i in range(0, da.size(), 3):
		if not dm.is_empty() and dm[i / 3] > 0:
			masked += 1
			continue
		var d0 := absi(da[i] - db[i])
		var d1 := absi(da[i + 1] - db[i + 1])
		var d2 := absi(da[i + 2] - db[i + 2])
		total += d0 + d1 + d2
		if maxi(d0, maxi(d1, d2)) > threshold:
			over += 1
			var px := (i / 3) % w
			var py := (i / 3) / w
			lo = Vector2i(mini(lo.x, px), mini(lo.y, py))
			hi = Vector2i(maxi(hi.x, px), maxi(hi.y, py))
			if heat != "":
				hd[i] = 255
				hd[i + 1] = 0
				hd[i + 2] = 0
			if not wm.is_empty():
				wm[i / 3] = 255
	if not wm.is_empty():
		Image.create_from_data(w, ia.get_height(), false, Image.FORMAT_L8, wm).save_png(
			_write_mask.path_join(label if label.ends_with(".png") else label + ".png"))
	if heat != "":
		Image.create_from_data(w, ia.get_height(), false, Image.FORMAT_RGB8, hd).save_png(
			heat.path_join(label if label.ends_with(".png") else label + ".png"))
	var n := da.size() / 3 - masked
	if n <= 0:
		print("%-40s every pixel masked" % label)
		return true
	print("%-40s mean |d| %6.3f   over %d/255: %6.3f %%   box %s%s" % [label,
		float(total) / (n * 3), threshold, 100.0 * over / n,
		"-" if over == 0 else "(%d,%d)-(%d,%d)" % [lo.x, lo.y, hi.x, hi.y],
		"" if masked == 0 else "   masked %.2f %%" % (100.0 * masked / (n + masked))])
	return true
