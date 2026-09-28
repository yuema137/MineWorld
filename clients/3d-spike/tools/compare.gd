extends SceneTree
## Compose a side-by-side of a reference plate and one of our renders, so the
## likeness can be judged in one image instead of by flipping between two.
##   godot --headless --path clients/3d-spike --script res://tools/compare.gd -- <left> <right> <out> [crop_l] 
func _init() -> void:
	var a := OS.get_cmdline_user_args()
	if a.size() < 3:
		print("need <left> <right> <out>")
		quit(1)
		return
	var l := Image.load_from_file(ProjectSettings.globalize_path(a[0]))
	var r := Image.load_from_file(ProjectSettings.globalize_path(a[1]))
	if l == null or r == null:
		print("load failed")
		quit(1)
		return
	# match heights, keeping aspect
	var h := 1100
	l.resize(int(l.get_width() * float(h) / l.get_height()), h, Image.INTERPOLATE_LANCZOS)
	r.resize(int(r.get_width() * float(h) / r.get_height()), h, Image.INTERPOLATE_LANCZOS)
	var gap := 16
	var out := Image.create(l.get_width() + gap + r.get_width(), h, false, l.get_format())
	out.fill(Color(0.09, 0.09, 0.10))
	out.blit_rect(l, Rect2i(0, 0, l.get_width(), h), Vector2i(0, 0))
	r.convert(l.get_format())
	out.blit_rect(r, Rect2i(0, 0, r.get_width(), h), Vector2i(l.get_width() + gap, 0))
	out.save_png(ProjectSettings.globalize_path(a[2]))
	print("wrote ", a[2], " ", out.get_width(), "x", out.get_height())
	quit()
