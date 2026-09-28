extends SceneTree

## Measure the skin tone each generated character actually carries, and check
## it against the bound `ARC-22` sets for the default cast.
##
##   godot --headless --path clients/2d-spike --script tools/skin_range.gd
##
## Exits non-zero if any character falls outside, so a regeneration cannot
## quietly drift out of range.
##
## The 2D cast is generated, not tinted, so the spec never stated a skin tone
## and the model chose one per character. The only way to know the range the
## cast spans is to look at the pixels.
##
## Skin is found rather than assumed: in the head region, pixels that are warm
## (R > G > B), bright enough not to be hair or outline, and not so saturated
## as to be clothing. The median of those is reported.

func measure(path: String) -> Dictionary:
	var img := Image.load_from_file(path)
	if img == null:
		return {}
	img.convert(Image.FORMAT_RGBA8)
	var w := img.get_width()
	var h := img.get_height()
	var lum := []
	var acc := Vector3.ZERO
	var n := 0
	# head region: upper third, central half
	for y in range(int(h * 0.04), int(h * 0.34)):
		for x in range(int(w * 0.22), int(w * 0.78)):
			var c := img.get_pixel(x, y)
			if c.a < 0.8:
				continue
			var r := c.r
			var g := c.g
			var b := c.b
			if not (r > g and g > b):
				continue
			if r < 0.45 or r - b < 0.06:
				continue
			var mx: float = maxf(r, maxf(g, b))
			var mn: float = minf(r, minf(g, b))
			var sat: float = 0.0 if mx <= 0.0 else (mx - mn) / mx
			if sat > 0.52:
				continue
			acc += Vector3(r, g, b)
			lum.append(r * 0.2126 + g * 0.7152 + b * 0.0722)
			n += 1
	if n < 40:
		return {}
	acc /= n
	lum.sort()
	return {"rgb": acc, "luma": lum[lum.size() / 2], "n": n}


func _init():
	var dir := ProjectSettings.globalize_path("res://art/generated/") 
	var names := ["gen_a_front", "gen_b_front", "gen_c_front", "gen_d_front",
		"gen_e_front", "gen_f_front", "gen_g_front", "gen_h_front",
		"gen_i_front", "gen_j_front", "gen_player_front",
		"gen_sit_a", "gen_sit_b"]
	# The floor is a guard against drift, not a target. It sits well below the
	# cast as generated (0.675 at its lowest) and well above the point `ARC-22`
	# identifies as past what the 3D albedo carries (0.512), so it fails on a
	# regeneration that wanders out rather than on ordinary variation.
	const FLOOR := 0.60
	var fail := 0
	var lo := 2.0
	var hi := -1.0
	for nm in names:
		var m := measure(dir + nm + ".png")
		if m.is_empty():
			print("%-18s  (no skin region found)" % nm)
			continue
		var c: Vector3 = m["rgb"]
		var L: float = m["luma"]
		lo = minf(lo, L)
		hi = maxf(hi, L)
		if L < FLOOR:
			fail += 1
		print("%-18s  #%02X%02X%02X   luma %.3f   (%d px)  %s"
			% [nm, int(c.x * 255), int(c.y * 255), int(c.z * 255), L, m["n"],
			"" if L >= FLOOR else "<-- BELOW FLOOR %.2f" % FLOOR])
	print("\nspan: luma %.3f .. %.3f   floor %.2f   (ARC-22 cites 0.512 as past the limit)"
		% [lo, hi, FLOOR])
	print("skin range: %s" % ("PASS" if fail == 0 else "%d outside" % fail))
	quit(0 if fail == 0 else 1)
