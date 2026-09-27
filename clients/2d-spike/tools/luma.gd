extends SceneTree

## Luminance calibration against the reference plates.
##
##   MWLUMA="a.png,b.png,..." godot --headless --path clients/2d-spike \
##       --script tools/luma.gd
##
## The earlier k-means found the plates' colour clusters, which is the right
## instrument for hue and for the depth of the shadows, and the wrong one for
## overall key: it reports where colour mass sits, not how bright the picture
## is. The plates are bright pictures with dark accents, and sampling only the
## extremes lost that. This reports the distribution.

func stats(path: String) -> void:
	var img := Image.load_from_file(path)
	if img == null:
		print("missing ", path)
		return
	img.convert(Image.FORMAT_RGBA8)
	var w := img.get_width()
	var h := img.get_height()
	var lum := PackedFloat32Array()
	var sat_sum := 0.0
	var n := 0
	for y in range(0, h, 2):
		for x in range(0, w, 2):
			var c := img.get_pixel(x, y)
			if c.a < 0.5:
				continue
			lum.append(c.r * 0.2126 + c.g * 0.7152 + c.b * 0.0722)
			var mx: float = maxf(c.r, maxf(c.g, c.b))
			var mn: float = minf(c.r, minf(c.g, c.b))
			sat_sum += 0.0 if mx <= 0.0 else (mx - mn) / mx
			n += 1
	if n == 0:
		return
	var arr := Array(lum)
	arr.sort()
	var mean := 0.0
	for v in arr:
		mean += v
	mean /= arr.size()

	var bright := 0
	var dark := 0
	for v in arr:
		if v > 0.70:
			bright += 1
		if v < 0.20:
			dark += 1

	print("%-34s mean %.3f  p10 %.3f  p25 %.3f  p50 %.3f  p75 %.3f  p90 %.3f  >0.7 %4.1f%%  <0.2 %4.1f%%  sat %.3f"
		% [path.get_file(), mean,
		arr[int(arr.size() * 0.10)], arr[int(arr.size() * 0.25)],
		arr[int(arr.size() * 0.50)], arr[int(arr.size() * 0.75)],
		arr[int(arr.size() * 0.90)],
		100.0 * bright / arr.size(), 100.0 * dark / arr.size(),
		sat_sum / n])


func _init():
	for p in OS.get_environment("MWLUMA").split(",", false):
		stats(p.strip_edges())
	quit(0)
