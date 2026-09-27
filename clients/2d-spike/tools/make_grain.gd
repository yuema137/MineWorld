extends SceneTree

## Derives a neutral paper-tooth texture from scanned watercolour washes.
##
##   godot --headless --path clients/2d-spike --script tools/make_grain.gd
##
## Source: PuzzleAndy, "CC0 Watercolor Textures", OpenGameArt, CC0 1.0.
## The scans are strongly coloured washes and are not usable as-is. What is
## wanted from them is the substrate: the irregular tooth of real cold-press
## paper, which is organically uneven in a way value noise is not. So this
## keeps only the high-frequency luminance — desaturate, subtract a blurred
## copy of itself, normalise around mid grey — and throws the colour and the
## composition away.
##
## Set MWGRAIN_SRC to the directory holding the downloaded scans. The scans
## themselves are not committed; this derived 512px tile is.

const OUT := "res://art/paper_grain.png"
const SIZE := 512


func _init():
	var dir := OS.get_environment("MWGRAIN_SRC")
	if dir == "":
		print("set MWGRAIN_SRC to the directory holding the source scans")
		quit(1)
		return
	var srcs := ["wc_6.jpg", "wc_8.jpg", "wc_3.jpg", "wc_11.jpg"]
	var acc := PackedFloat32Array()
	acc.resize(SIZE * SIZE)
	acc.fill(0.0)
	var used := 0

	for name in srcs:
		var img := Image.load_from_file(dir.path_join(name))
		if img == null:
			print("skip ", name)
			continue
		img.convert(Image.FORMAT_RGBA8)
		# square crop from the middle, then down to the working size
		var n: int = min(img.get_width(), img.get_height())
		var c := Image.create(n, n, false, Image.FORMAT_RGBA8)
		c.blit_rect(img, Rect2i((img.get_width() - n) / 2,
			(img.get_height() - n) / 2, n, n), Vector2i.ZERO)
		c.resize(SIZE, SIZE, Image.INTERPOLATE_LANCZOS)

		# A blurred copy, by round-tripping through a smaller size. The radius
		# is deliberately tiny: a wide blur leaves the wash edges and the
		# artist's brush strokes in the residue, and those read as smears of
		# dirt over the scene rather than as tooth in the paper.
		var blur := c.duplicate() as Image
		blur.resize(SIZE / 2, SIZE / 2, Image.INTERPOLATE_LANCZOS)
		blur.resize(SIZE, SIZE, Image.INTERPOLATE_CUBIC)

		for y in SIZE:
			for x in SIZE:
				var a := c.get_pixel(x, y)
				var b := blur.get_pixel(x, y)
				var la := a.r * 0.2126 + a.g * 0.7152 + a.b * 0.0722
				var lb := b.r * 0.2126 + b.g * 0.7152 + b.b * 0.0722
				# clamp before accumulating: ink splatter in the scans is
				# high-frequency too, and would survive as black specks
				acc[y * SIZE + x] += clampf(la - lb, -0.055, 0.055)
		used += 1

	if used == 0:
		print("no sources found in ", dir)
		quit(1)
		return

	# normalise: centre on 0.5 and spread to a usable amplitude
	var lo := 1e9
	var hi := -1e9
	for v in acc:
		lo = minf(lo, v / used)
		hi = maxf(hi, v / used)
	var span: float = maxf(hi - lo, 0.0001)

	var out := Image.create(SIZE, SIZE, false, Image.FORMAT_RGBA8)
	for y in SIZE:
		for x in SIZE:
			var v: float = (acc[y * SIZE + x] / used - lo) / span
			# a gentle S so the tooth keeps its bite without clipping
			v = clampf(0.5 + (v - 0.5) * 1.20, 0.0, 1.0)
			out.set_pixel(x, y, Color(v, v, v, 1.0))

	out.save_png(OUT)
	print("grain written from %d source(s): %s  %dx%d" % [used, OUT, SIZE, SIZE])
	quit(0)
