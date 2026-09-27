extends SceneTree

## Turn generated candidates into sprites that obey the spike's scale rule.
##
##   MWNORM=<spec.json> godot --headless --path clients/2d-spike \
##       --script tools/normalize.gd
##
## A candidate arrives as a 1024x1536 picture of a person somewhere in the
## middle of it. A sprite has to be cut to the figure, anchored where the
## figure meets the ground, and sized so that `Iso.gd`'s rule still holds:
## 1 m is 32 px, art is authored at 2x and drawn down, so a 1.75 m person is
## 112 px tall in the file and 56 px on screen at a draw scale of 0.5.
## Generating large and scaling by eye per sprite is how a set of assets stops
## agreeing about how big a person is.
##
## It also writes a QA sheet compositing each sprite over the actual paving
## colour, because an alpha channel that reads clean in a histogram can still
## halo against a warm mid grey, and the only way to know is to look.

const PX_PER_M := 32.0        # must match Iso.PX_PER_M_Z
const AUTHOR_SCALE := 2.0     # art is authored at 2x and drawn down
const PAVING := Color("bab1a6")   # must match Ground.gd C["stone"]

const IN_DIR := "res://art/candidates"
const OUT_DIR := "res://art/generated"
const QA := "res://shots/_qa_normalize.png"


## Alpha below this is treated as nothing. Generated PNGs carry a wide skirt of
## near-zero alpha that contributes no colour but inflates the bounding box.
const ALPHA_FLOOR := 0.06


func _bbox(img: Image) -> Rect2i:
	var w := img.get_width()
	var h := img.get_height()
	var minx := w
	var maxx := -1
	var miny := h
	var maxy := -1
	for y in h:
		for x in w:
			if img.get_pixel(x, y).a > ALPHA_FLOOR:
				minx = mini(minx, x)
				maxx = maxi(maxx, x)
				miny = mini(miny, y)
				maxy = maxi(maxy, y)
	if maxx < 0:
		return Rect2i()
	return Rect2i(minx, miny, maxx - minx + 1, maxy - miny + 1)


## Drop the near-zero-alpha skirt outright, and un-multiply any colour that has
## been dragged toward the matte on genuinely soft pixels. Whether this is
## needed at all is what the QA sheet answers.
func _clean(img: Image, defringe: bool) -> void:
	var w := img.get_width()
	var h := img.get_height()
	for y in h:
		for x in w:
			var c := img.get_pixel(x, y)
			if c.a <= ALPHA_FLOOR:
				img.set_pixel(x, y, Color(0, 0, 0, 0))
				continue
			if defringe and c.a < 0.98:
				# straight alpha recovered from a premultiplied edge
				var k := 1.0 / c.a
				img.set_pixel(x, y, Color(
					minf(c.r * k, 1.0), minf(c.g * k, 1.0),
					minf(c.b * k, 1.0), c.a))


func _init():
	var spec_path := OS.get_environment("MWNORM")
	if spec_path == "":
		print("set MWNORM to the spec json")
		quit(1)
		return
	var f := FileAccess.open(spec_path, FileAccess.READ)
	if f == null:
		print("cannot read ", spec_path)
		quit(1)
		return
	var doc: Dictionary = JSON.parse_string(f.get_as_text())
	var defringe := OS.get_environment("MWDEFRINGE") == "1"

	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(OUT_DIR))
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path("res://shots"))

	var done: Array = []
	# Merge into whatever is already there. Each spec file covers one family —
	# characters, props — and writing the manifest fresh per run silently drops
	# every family but the last one normalized.
	var manifest := {}
	var prev := FileAccess.open("%s/generated.json" % OUT_DIR, FileAccess.READ)
	if prev != null:
		var old = JSON.parse_string(prev.get_as_text())
		if old is Dictionary:
			manifest = old
		prev.close()

	for s in doc["specs"]:
		var name: String = s["name"]
		var src := "%s/%s.png" % [IN_DIR, name]
		var path := ProjectSettings.globalize_path(src)
		if not FileAccess.file_exists(path):
			continue
		var img := Image.load_from_file(path)
		img.convert(Image.FORMAT_RGBA8)
		_clean(img, defringe)

		var bb := _bbox(img)
		if bb.size.x <= 0:
			print("empty after cleaning: ", name)
			continue
		var cut := Image.create(bb.size.x, bb.size.y, false, Image.FORMAT_RGBA8)
		cut.blit_rect(img, bb, Vector2i.ZERO)

		# the scale rule: real height in metres decides pixel height
		var m: float = float(s.get("height_m", 1.75))
		var target_h := int(round(m * PX_PER_M * AUTHOR_SCALE))
		var k := float(target_h) / float(cut.get_height())
		cut.resize(maxi(1, int(round(cut.get_width() * k))), target_h,
			Image.INTERPOLATE_LANCZOS)

		cut.save_png("%s/%s.png" % [OUT_DIR, name])
		manifest[name] = {
			"w": cut.get_width(), "h": cut.get_height(),
			"ax": cut.get_width() / 2.0, "ay": cut.get_height(),
			"height_m": m, "source": "generated",
		}
		done.append({"name": name, "img": cut, "m": m})
		print("normalized %-22s %dx%d  (%.2f m, draw scale %.3f)"
			% [name, cut.get_width(), cut.get_height(), m, 1.0 / AUTHOR_SCALE])

	# manifest fragment, merged into props.json by the caller
	var mf := FileAccess.open("%s/generated.json" % OUT_DIR, FileAccess.WRITE)
	mf.store_string(JSON.stringify(manifest, "  ", true))
	mf.close()

	if done.is_empty():
		print("nothing normalized")
		quit(1)
		return

	# --- QA sheet: every sprite over paving and over the darkest ground ------
	# Two grounds, because a fringe that is invisible on warm grey is obvious
	# on foliage shade, and vice versa.
	var cols := mini(done.size(), 7)
	var rows := int(ceil(float(done.size()) / cols))
	var cell := 150
	var sheet := Image.create(cell * cols, cell * rows * 2, false, Image.FORMAT_RGBA8)
	for y in sheet.get_height():
		for x in sheet.get_width():
			var half := 0 if y < cell * rows else 1
			sheet.set_pixel(x, y, PAVING if half == 0 else Color("3c4a33"))
	for i in done.size():
		var e: Dictionary = done[i]
		var im: Image = e["img"]
		# drawn at the size it will actually appear on screen
		var use := im.duplicate() as Image
		use.resize(maxi(1, int(im.get_width() / AUTHOR_SCALE)),
			maxi(1, int(im.get_height() / AUTHOR_SCALE)), Image.INTERPOLATE_LANCZOS)
		for half in 2:
			var src := im if half == 0 else use
			var ox := (i % cols) * cell + (cell - src.get_width()) / 2
			var oy := half * cell * rows + (i / cols) * cell + (cell - src.get_height()) / 2
			if src.get_width() < cell and src.get_height() < cell:
				sheet.blend_rect(src, Rect2i(Vector2i.ZERO, src.get_size()),
					Vector2i(ox, oy))
	sheet.save_png(QA)
	print("qa sheet: ", QA, "  (top half: authored size over paving, bottom half: draw size over shade)")
	quit(0)
