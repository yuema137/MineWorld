extends SceneTree
## How different two captured frames are (step-15 §18.3 J-4).
##
##   godot --headless --path clients/3d-spike --script res://tools/frame_diff.gd -- <a> <b> [threshold]
##
## <a> and <b> are two PNGs, or two directories, in which case every PNG in <a>
## is compared with the file of the same name in <b>. Per pair it prints the
## mean absolute difference (0-255, averaged over R, G and B), the share of
## pixels whose largest channel difference exceeds `threshold` (default 8), and
## the bounding box of those pixels -- so a change can be found and looked at,
## not only counted. Exits 1 if a pair cannot be compared.
func _init() -> void:
	var a := OS.get_cmdline_user_args()
	if a.size() < 2:
		print("need <a> <b> [threshold]")
		quit(1)
		return
	var threshold := int(a[2]) if a.size() > 2 else 8
	var left := ProjectSettings.globalize_path(a[0])
	var right := ProjectSettings.globalize_path(a[1])
	var ok := true
	if DirAccess.dir_exists_absolute(left):
		var names := Array(DirAccess.get_files_at(left)).filter(
			func(n: String) -> bool: return n.ends_with(".png"))
		names.sort()
		for n in names:
			ok = _pair(left.path_join(n), right.path_join(n), n, threshold) and ok
	else:
		ok = _pair(left, right, left.get_file(), threshold)
	quit(0 if ok else 1)


func _pair(pa: String, pb: String, label: String, threshold: int) -> bool:
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
	var da := ia.get_data()
	var db := ib.get_data()
	var w := ia.get_width()
	var total := 0
	var over := 0
	var lo := Vector2i(w, ia.get_height())
	var hi := Vector2i(-1, -1)
	for i in range(0, da.size(), 3):
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
	var n := da.size() / 3
	print("%-40s mean |d| %6.3f   over %d/255: %6.3f %%   box %s" % [label,
		float(total) / (n * 3), threshold, 100.0 * over / n,
		"-" if over == 0 else "(%d,%d)-(%d,%d)" % [lo.x, lo.y, hi.x, hi.y]])
	return true
