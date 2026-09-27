extends SceneTree

## Contact sheet of chosen sprites, for looking at art without launching the
## scene:  godot --headless --path clients/2d-spike --script tools/sheet.gd
## Names come from the MWSHEET env var, comma separated.

func _init():
	var names := OS.get_environment("MWSHEET").split(",", false)
	var bg := Color(0.72, 0.70, 0.66)
	var cell := 300
	var cols := 5
	var rows := int(ceil(float(names.size()) / cols))
	var out := Image.create(cell * cols, cell * rows, false, Image.FORMAT_RGBA8)
	out.fill(bg)
	for i in names.size():
		var t: Texture2D = load("res://art/svg/%s.svg" % names[i].strip_edges())
		if t == null:
			print("missing ", names[i]); continue
		var im := t.get_image()
		im.convert(Image.FORMAT_RGBA8)
		var k: float = float(cell - 20) / float(max(im.get_width(), im.get_height()))
		im.resize(int(im.get_width() * k), int(im.get_height() * k), Image.INTERPOLATE_LANCZOS)
		var ox := (i % cols) * cell + (cell - im.get_width()) / 2
		var oy := (i / cols) * cell + (cell - im.get_height()) - 10
		out.blend_rect(im, Rect2i(Vector2i.ZERO, im.get_size()), Vector2i(ox, oy))
	out.save_png("res://shots/_sheet.png")
	print("sheet: ", names.size(), " sprites")
	quit(0)
