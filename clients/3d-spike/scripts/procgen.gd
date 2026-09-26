## Procedurally generated cut-out textures (leaves, flowers, grass, ducks).
##
## No CC0 pack ships a tree light enough to place thirty of: Poly Haven's
## cheapest scanned tree is a 95 MB mesh. So canopies here are built from a
## handful of alpha-cut leaf cards, and the cards themselves are drawn at
## startup. That keeps vegetation self-authored, tiny, and tunable.
class_name ProcGen
extends RefCounted

static var _cache: Dictionary = {}


static func _blank(n: int) -> Image:
	var img := Image.create(n, n, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	return img


static func _leaf(img: Image, cx: float, cy: float, rx: float, ry: float, rot: float, c: Color) -> void:
	var n := img.get_width()
	var ca := cos(-rot)
	var sa := sin(-rot)
	var r := maxf(rx, ry) + 1.0
	var x0 := int(maxf(0.0, cx - r))
	var x1 := int(minf(float(n), cx + r))
	var y0 := int(maxf(0.0, cy - r))
	var y1 := int(minf(float(n), cy + r))
	for y in range(y0, y1):
		for x in range(x0, x1):
			var dx := float(x) + 0.5 - cx
			var dy := float(y) + 0.5 - cy
			var u := (dx * ca - dy * sa) / rx
			var v := (dx * sa + dy * ca) / ry
			# teardrop-ish leaf: an ellipse pinched at one end
			var w := 1.0 - 0.45 * v
			var d := (u * u) / maxf(w * w, 0.05) + v * v
			if d <= 1.0:
				var shade := 1.0 - 0.35 * clampf(v * 0.5 + 0.5, 0.0, 1.0)
				var cc := Color(c.r * shade, c.g * shade, c.b * shade, 1.0)
				img.set_pixel(x, y, cc)
	# midrib, so the card is not a field of flat blobs at close range
	for t in range(0, 24):
		var f := float(t) / 23.0
		var lx := -(f * 2.0 - 1.0) * ry * 0.9
		var px := int(cx + (-sa) * lx)
		var py := int(cy + (ca) * lx)
		if px >= 0 and px < n and py >= 0 and py < n:
			var e := img.get_pixel(px, py)
			if e.a > 0.5:
				img.set_pixel(px, py, e.lightened(0.16))


## One dense cluster of leaves on a transparent card.
static func leaf_card(seed_v: int, base: Color = Color(0.30, 0.46, 0.17)) -> Texture2D:
	var key := "leaf%d%s" % [seed_v, base]
	if _cache.has(key):
		return _cache[key]
	var n := 256
	var img := _blank(n)
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	for i in range(140):
		# cluster toward the middle, thin out at the rim -> a soft silhouette
		var a := rng.randf() * TAU
		var rr := pow(rng.randf(), 0.62) * 0.46
		var cx := n * (0.5 + cos(a) * rr)
		var cy := n * (0.5 + sin(a) * rr * 0.86)
		var s := rng.randf_range(11.0, 20.0)
		var tone := rng.randf_range(-0.09, 0.13)
		var c := Color(
			clampf(base.r + tone * 0.9 + rng.randf_range(-0.03, 0.05), 0.0, 1.0),
			clampf(base.g + tone + rng.randf_range(-0.04, 0.06), 0.0, 1.0),
			clampf(base.b + tone * 0.5, 0.0, 1.0))
		_leaf(img, cx, cy, s * rng.randf_range(0.42, 0.58), s, rng.randf() * TAU, c)
	img.generate_mipmaps()
	var t := ImageTexture.create_from_image(img)
	_cache[key] = t
	return t


## A conifer branch spray: narrow, dark, layered.
static func needle_card(seed_v: int) -> Texture2D:
	var key := "needle%d" % seed_v
	if _cache.has(key):
		return _cache[key]
	var n := 256
	var img := _blank(n)
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	for i in range(220):
		var fy := rng.randf()
		var halfw := (1.0 - fy) * 0.46 + 0.03
		var cx := n * (0.5 + rng.randf_range(-halfw, halfw))
		var cy := n * (0.06 + fy * 0.9)
		var g := rng.randf_range(-0.05, 0.07)
		var c := Color(0.11 + g * 0.5, 0.25 + g, 0.13 + g * 0.6)
		_leaf(img, cx, cy, rng.randf_range(2.0, 3.4), rng.randf_range(10.0, 17.0),
			rng.randf_range(-0.9, 0.9), c)
	img.generate_mipmaps()
	var t := ImageTexture.create_from_image(img)
	_cache[key] = t
	return t


## Flowering ground cover for planters and verges: green base, saturated heads.
## The references use saturated colour *only* in flowers and signage (sec.6).
static func flower_card(seed_v: int) -> Texture2D:
	var key := "flower%d" % seed_v
	if _cache.has(key):
		return _cache[key]
	var n := 256
	var img := _blank(n)
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	for i in range(150):
		var cx := rng.randf() * n
		var cy := n * (0.30 + rng.randf() * 0.70)
		var c := Color(0.22 + rng.randf_range(-0.05, 0.08), 0.40 + rng.randf_range(-0.07, 0.12), 0.16)
		_leaf(img, cx, cy, rng.randf_range(4.0, 8.0), rng.randf_range(9.0, 18.0),
			rng.randf_range(-1.2, 1.2), c)
	var palette := [
		Color(0.92, 0.42, 0.55), Color(0.72, 0.45, 0.85), Color(0.96, 0.88, 0.92),
		Color(0.97, 0.64, 0.24), Color(0.95, 0.93, 0.55), Color(0.62, 0.42, 0.82),
	]
	for i in range(55):
		var cx := rng.randf() * n
		var cy := n * (0.12 + rng.randf() * 0.62)
		var c: Color = palette[rng.randi() % palette.size()]
		var petals := 5
		var pr := rng.randf_range(4.5, 7.5)
		for p in range(petals):
			var a := TAU * float(p) / petals
			_leaf(img, cx + cos(a) * pr * 0.7, cy + sin(a) * pr * 0.7, pr * 0.5, pr * 0.6, a, c)
		_leaf(img, cx, cy, pr * 0.32, pr * 0.32, 0.0, Color(0.98, 0.86, 0.35))
	img.generate_mipmaps()
	var t := ImageTexture.create_from_image(img)
	_cache[key] = t
	return t


## Grass tuft card for verges and the lake trail.
static func grass_card(seed_v: int) -> Texture2D:
	var key := "grass%d" % seed_v
	if _cache.has(key):
		return _cache[key]
	var n := 256
	var img := _blank(n)
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	for i in range(120):
		var bx := rng.randf() * n
		var h := rng.randf_range(70.0, 190.0)
		var lean := rng.randf_range(-0.5, 0.5)
		var g := rng.randf_range(-0.06, 0.10)
		var c := Color(0.26 + g * 0.7, 0.42 + g, 0.16 + g * 0.5)
		var steps := int(h / 3.0)
		for s in range(steps):
			var f := float(s) / float(steps)
			var y := n - 4.0 - f * h
			var x := bx + lean * f * f * 40.0
			var w := (1.0 - f) * rng.randf_range(1.4, 2.6) + 0.5
			for dx in range(-int(w), int(w) + 1):
				var px := int(x) + dx
				var py := int(y)
				if px >= 0 and px < n and py >= 0 and py < n:
					img.set_pixel(px, py, c.lightened(f * 0.22))
	img.generate_mipmaps()
	var t := ImageTexture.create_from_image(img)
	_cache[key] = t
	return t


## A distant conifer ridge, drawn as a single silhouette strip. Used far out on
## the far shore where geometry would be waste.
static func treeline_card(seed_v: int) -> Texture2D:
	var key := "tline%d" % seed_v
	if _cache.has(key):
		return _cache[key]
	var w := 1024
	var h := 256
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	var x := 0.0
	while x < w:
		var th := rng.randf_range(90.0, 210.0)
		var tw := th * rng.randf_range(0.20, 0.34)
		var g := rng.randf_range(-0.03, 0.05)
		var c := Color(0.13 + g, 0.24 + g * 1.4, 0.15 + g)
		for yy in range(int(h - th), h):
			var f := float(h - yy) / th
			var half := tw * (1.0 - f) * 0.5
			# slight raggedness so the silhouette is not a row of clean triangles
			half *= 1.0 + 0.16 * sin(float(yy) * 0.9 + x)
			for xx in range(int(x - half), int(x + half) + 1):
				if xx >= 0 and xx < w:
					img.set_pixel(xx, yy, c)
		x += tw * rng.randf_range(0.42, 0.72)
	img.generate_mipmaps()
	var t := ImageTexture.create_from_image(img)
	_cache[key] = t
	return t
