extends SceneTree

## Make the shared ground textures tile without a seam.
##
##   MWSEAM_IN=<dir> godot --headless --path clients/2d-spike \
##       --script tools/make_seamless.gd
##
## ground_plaza_cobble arrives with a real vertical seam — its wrap discontinuity
## measures far above the texture's own adjacent-pixel baseline, so tiling it
## shows a hard line down the plaza. Grass and pavement are close to seamless
## already; stone path is mildly off.
##
## The fix is a wrap-band cross-blend: the strip along one edge is blended into
## the strip along the opposite edge with a linear ramp, so whatever the wrap
## discontinuity was, it is spread over a band instead of landing on one column.
## It costs a little detail in two narrow strips and it is reliable, which
## matters more here than cleverness.

const BAND := 0.10        # fraction of the image blended at each wrap


func seam_score(img: Image, vertical: bool) -> float:
	## Mean absolute difference across the wrap, against the mean absolute
	## difference between ordinary neighbours. A ratio near 1 is seamless.
	var w := img.get_width()
	var h := img.get_height()
	var wrap := 0.0
	var base := 0.0
	var count: int = h if vertical else w
	for i in count:
		var a: Color
		var b: Color
		var c: Color
		var d: Color
		if vertical:
			a = img.get_pixel(w - 1, i); b = img.get_pixel(0, i)
			c = img.get_pixel(w / 2, i); d = img.get_pixel(w / 2 + 1, i)
		else:
			a = img.get_pixel(i, h - 1); b = img.get_pixel(i, 0)
			c = img.get_pixel(i, h / 2); d = img.get_pixel(i, h / 2 + 1)
		wrap += absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b)
		base += absf(c.r - d.r) + absf(c.g - d.g) + absf(c.b - d.b)
	wrap /= count
	base = maxf(base / count, 0.0001)
	return wrap / base


## Only treat an axis that is actually bad. Blending a wrap that was already
## clean introduces a discontinuity where the band ends, which measurably made
## grass and the stone path worse on the first pass.
const BAD := 2.5


func blend_wrap(img: Image, do_x: bool, do_y: bool) -> Image:
	var w := img.get_width()
	var h := img.get_height()
	var bw := maxi(2, int(w * BAND))
	var bh := maxi(2, int(h * BAND))
	var out := img.duplicate() as Image

	# horizontal wrap: fold the right band into the left band
	for y in (h if do_x else 0):
		for x in bw:
			var t := float(x) / float(bw)          # 0 at the edge, 1 inside
			var a := img.get_pixel(x, y)
			var b := img.get_pixel(w - bw + x, y)
			var k := 0.5 * (1.0 - t)
			out.set_pixel(x, y, a.lerp(b, k))
	# vertical wrap
	var tmp := out.duplicate() as Image
	for x in (w if do_y else 0):
		for y in bh:
			var t := float(y) / float(bh)
			var a := tmp.get_pixel(x, y)
			var b := tmp.get_pixel(x, h - bh + y)
			var k := 0.5 * (1.0 - t)
			out.set_pixel(x, y, a.lerp(b, k))
	return out


func _init():
	var dir := OS.get_environment("MWSEAM_IN")
	if dir == "":
		print("set MWSEAM_IN")
		quit(1)
		return
	var out_dir := ProjectSettings.globalize_path("res://art/ground")
	DirAccess.make_dir_recursive_absolute(out_dir)
	for n in ["ground_grass", "ground_pavement_slabs", "ground_plaza_cobble",
			"ground_stone_path"]:
		var img := Image.load_from_file("%s/%s.png" % [dir, n])
		if img == null:
			print("missing ", n)
			continue
		img.convert(Image.FORMAT_RGBA8)
		var bx := seam_score(img, true)
		var by := seam_score(img, false)
		var fixed := blend_wrap(img, bx > BAD, by > BAD)
		var ax := seam_score(fixed, true)
		var ay := seam_score(fixed, false)
		fixed.save_png("%s/%s.png" % [out_dir, n])
		print("%-24s seam x %5.1f -> %4.1f   y %5.1f -> %4.1f%s"
			% [n, bx, ax, by, ay,
			"   (treated)" if (bx > BAD or by > BAD) else "   (already clean)"])
	quit(0)
