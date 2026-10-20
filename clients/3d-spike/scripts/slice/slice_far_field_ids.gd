## The far-field check's ID render and its six criteria (RL-e0b sec.6, `docs/decisions/ARC-290.md`).
##
## WHY AN ID RENDER. FF-BELOW and FF-ABOVE (`slice_far_field_check.gd`) judge whether something is
## drawn below and above the horizon. They cannot see *where* it is drawn, *which* part is in front,
## whether an edge is a vertical cut, whether a band has surface, or whether the ground is built on
## (RL-e0b A-7). So every far-field part is also drawn in a flat identifying colour (I: the slice's
## own nodes hidden, pitch 0; G: the slice shown, looking down at the ring), and six criteria read
## those frames:
##
##   FF-ORDER   land nearer than the sea horizon is drawn in front of the water        (D-1)
##   FF-CREST   the topmost far-field part is a band, within 0.10 deg of its true angle (D-2)
##   FF-SEAM    no two adjacent columns differ in part over a vertical run of 6 rows   (D-3)
##   FF-RELIEF  a near band's pixels vary in luma within a row (texture)               (D-4)
##   FF-RING    from above, the ground between the boundary and 400 m is built on      (D-5)
##   FF-BAY     from each protected eye that is judged, the water still shows          (SD-5's guard)
##
## EVERY EXPECTED VALUE comes from the far-field file the slice loaded, through the formulas of
## RL-e0b sec.4 written here again -- never by calling the view's code (I-4), so a drawing defect
## cannot pass by agreeing with itself. Nothing names a world, a bearing or a band: the viewpoints
## come from the scene (`SliceWorld.FAR_FIELD_VIEWPOINTS`), the rest from the file. The numbers are
## the operator's (Q-3), fixed before any run (ARC-23).
class_name SliceFarFieldIds
extends RefCounted

# --- sec.6.3, fixed --------------------------------------------------------------------------------
const ORDER_NEED := 0.90
const CREST_NEED := 0.95
const CREST_TOL_DEG := 0.10
const CREST_CLEAR_ROWS := 3
const SEAM_ROWS := 6
const SEAM_FROM_DEG := -1.0       # rows judged from e_h - 1.0 deg up
const RELIEF_MIN_PX := 2000
const RELIEF_BELOW_ROWS := 3
const RELIEF_WINDOW := 29
const RELIEF_NEED := 2.5          # / 255
const RING_BIN_M := 20.0
const RING_FROM_M := 20.0         # beyond reach_m
const RING_TO_M := 400.0
const RING_MIN_PX := 200
const RING_WATER_MAX := 0.5
const RING_NEED := 0.15
const BAY_NEED := 0.90
const MIN_JUDGED := 20
const UNKNOWN_MAX := 0.01
const TOL := 24                   # /255 per channel, an ID's colour
## RL-e0b sec.4: the drawing rules' own numbers, restated (the view's are not read).
const T_BAND := 3.0
const T_WATER := 4.0
const EARTH_R := 6371000.0
const REFRACTION_K := 0.13
const OUTER_M := 1450.0

## The G render (sec.6.1).
const G_SIZE := Vector2i(960, 540)
const G_HFOV := 70.0
const G_PITCH := -27.0

## ID colours, sRGB (sec.6.2). The sky is drawn as a flat background colour in the ID frames, so a
## sky pixel is read exactly instead of being compared with E (whose tonemapping differs).
const SKY := Color8(0x00, 0xff, 0x00)
const FIXED := {
	"ground": Color8(0x20, 0x20, 0x20), "foreground": Color8(0x40, 0x40, 0x40),
	"water": Color8(0x00, 0x40, 0xff), "sea-ribbon": Color8(0x00, 0xa0, 0xff),
	"massing": Color8(0xff, 0x80, 0x00),
}
## Decoded ids: 0 unknown, 1 sky, then FIXED in order, then the bands in file order.
const UNKNOWN := 0
const SKY_ID := 1
const NAMES := ["ground", "foreground", "water", "sea-ribbon", "massing"]

var ff: MineWorldFarField
var _palette: Array = []          # [id, Color]
var _names: PackedStringArray = []
var _bands: Array = []            # the file's bands, in order
var _cache := {}
var _datum := 0.0


func _init(field: MineWorldFarField) -> void:
	ff = field
	_datum = field.datum_m()
	_bands = field.silhouettes()
	_names = PackedStringArray(["unknown", "sky"])
	_palette.append([SKY_ID, SKY])
	for n in NAMES:
		_names.append(n)
		_palette.append([_names.size() - 1, FIXED[n]])
	for k in _bands.size():
		_names.append("band:" + str(_bands[k]["name"]))
		_palette.append([_names.size() - 1, band_colour(k)])


static func band_colour(k: int) -> Color:
	return Color8(0xff, 0x00, mini(0x10 + 0x14 * k, 0xff))


## The ID colour for a name from `SliceFarFieldView.ids()`.
func colour_of(name: String) -> Color:
	if FIXED.has(name):
		return FIXED[name]
	for k in _bands.size():
		if name == "band:" + str(_bands[k]["name"]):
			return band_colour(k)
	return Color(1, 0, 1)


func id_of(name: String) -> int:
	return _names.find(name)


func name_of(id: int) -> String:
	return _names[id]


func is_band(id: int) -> bool:
	return id >= 2 + NAMES.size()


func band_index(id: int) -> int:
	return id - 2 - NAMES.size()


## One frame as ids, a byte per pixel: the nearest ID colour within TOL on every channel.
func decode(img: Image) -> PackedByteArray:
	var data := img.get_data()
	var n := img.get_width() * img.get_height()
	var out := PackedByteArray()
	out.resize(n)
	for i in n:
		var j := i * 3
		var key := (data[j] << 16) | (data[j + 1] << 8) | data[j + 2]
		var id: int = _cache.get(key, -1)
		if id < 0:
			id = _nearest(data[j], data[j + 1], data[j + 2])
			_cache[key] = id
		out[i] = id
	return out


func _nearest(r: int, g: int, b: int) -> int:
	var best := UNKNOWN
	var best_d := 1 << 30
	for entry in _palette:
		var c: Color = entry[1]
		var dr := absi(r - c.r8)
		var dg := absi(g - c.g8)
		var db := absi(b - c.b8)
		if dr > TOL or dg > TOL or db > TOL:
			continue
		if dr + dg + db < best_d:
			best_d = dr + dg + db
			best = entry[0]
	return best


# --- the file's geometry, restated (RL-e0b sec.4) -----------------------------------------------

## A band's crest at `bearing`: [height m, distance m], interpolated with a smoothstep between the
## profile's points (RL-e0 sec.5.4).
static func crest_hd(band: Dictionary, bearing: float) -> Vector2:
	var from: float = band["sector"]["from_deg"]
	var o := MineWorldFarField.clockwise(from, bearing)
	var pts: Array = band["profile"]
	for i in range(pts.size() - 1):
		var op := 0.0 if i == 0 else MineWorldFarField.clockwise(from, pts[i]["bearing_deg"])
		var oq := MineWorldFarField.clockwise(from, pts[i + 1]["bearing_deg"])
		if o >= op and o <= oq:
			var e := smoothstep(0.0, 1.0, (o - op) / maxf(oq - op, 1e-9))
			return Vector2(lerpf(pts[i]["height_m"], pts[i + 1]["height_m"], e),
				lerpf(pts[i]["distance_m"], pts[i + 1]["distance_m"], e))
	var last: Dictionary = pts[pts.size() - 1]
	return Vector2(last["height_m"], last["distance_m"])


## The angle (radians) at which a point `h` m above sea level, `d` m away, is seen from scene
## height `eye_y`: with Earth's curvature and refraction.
func angle(h: float, d: float, eye_y: float) -> float:
	return (h - _datum - eye_y - d * d * (1.0 - REFRACTION_K) / (2.0 * EARTH_R)) / d


func water_at(bearing: float) -> Dictionary:
	for body in ff.water():
		if MineWorldFarField.sector_contains(body["sector"], bearing):
			return body
	return {}


## 0 at a sector end, 1 from `taper` degrees inside (or a quarter of the sector).
static func end_factor(sector: Dictionary, bearing: float, taper: float) -> float:
	var span := MineWorldFarField.clockwise(sector["from_deg"], sector["to_deg"])
	var o := MineWorldFarField.clockwise(sector["from_deg"], bearing)
	if o > span:
		return 0.0
	return smoothstep(0.0, maxf(minf(taper, span * 0.25), 1e-6), minf(o, span - o))


## A band's expected crest angle (radians) from scene height `eye_y`, with the end taper: its crest
## falls to its foot (the water's level in a water sector, the foreground's far height elsewhere).
## [angle, distance].
func band_angle(band: Dictionary, bearing: float, eye_y: float) -> Vector2:
	var hd := crest_hd(band, bearing)
	var body := water_at(bearing)
	var foot_h := _datum + ground_y(OUTER_M)
	if not body.is_empty():
		foot_h = float(body["level_m"])
	var foot := angle(foot_h, hd.y, eye_y)
	var s := end_factor(band["sector"], bearing, T_BAND)
	return Vector2(foot + (angle(hd.x, hd.y, eye_y) - foot) * s, hd.y)


## The water's near edge (SD-3.2).
static func shore(body: Dictionary, bearing: float) -> float:
	var s := end_factor(body["sector"], bearing, T_WATER)
	return float(body["shore_m"]) + (OUTER_M - float(body["shore_m"])) * (1.0 - s)


## The sea horizon's dip (degrees, negative) and distance (m) from scene height `eye_y`.
func sea_horizon_deg(eye_y: float) -> float:
	return -rad_to_deg(sqrt(2.0 * maxf(_datum + eye_y, 0.01) / EARTH_R))


func sea_horizon_m(eye_y: float, level_m: float) -> float:
	return sqrt(2.0 * EARTH_R * maxf(eye_y + _datum - level_m, 0.01))


## The ground's scene height at radius `r` (the foreground's fall, RL-e0 sec.5.4).
func ground_y(r: float) -> float:
	var reach: float = ff.boundary()["reach_m"]
	var fg := ff.foreground()
	return SliceFarFieldView.GROUND_Y - float(fg["drop_m"]) \
		* smoothstep(reach, reach + float(fg["run_m"]), r)


# --- the pitch-0 frames' pixels (W x H, HFOV horizontal; the same camera as FF-BELOW) ----------

var _w := SliceFarFieldCheck.W
var _h := SliceFarFieldCheck.H
var _tan_h := tan(deg_to_rad(SliceFarFieldCheck.HFOV * 0.5))
var _tan_v := _tan_h * float(SliceFarFieldCheck.H) / float(SliceFarFieldCheck.W)


func _xh(x: int) -> float:
	return (x + 0.5 - _w * 0.5) / (_w * 0.5) * _tan_h


func _bearing(az: float, x: int) -> float:
	return fposmod(az + rad_to_deg(atan(_xh(x))), 360.0)


## A pixel centre's true elevation (degrees): exact for a pinhole camera at pitch 0, off-axis too.
func _elev(x: int, row: int) -> float:
	var yv := (_h * 0.5 - 0.5 - row) / (_h * 0.5) * _tan_v
	return rad_to_deg(atan(yv / sqrt(1.0 + _xh(x) * _xh(x))))


## The fractional row at which column `x` sees elevation `deg` (row centres are whole numbers).
func _row(x: int, deg: float) -> float:
	var yv := tan(deg_to_rad(deg)) * sqrt(1.0 + _xh(x) * _xh(x))
	return _h * 0.5 - 0.5 - yv / _tan_v * (_h * 0.5)


func _columns() -> PackedInt32Array:
	var cols := PackedInt32Array()
	for x in _w:
		if absf(rad_to_deg(atan(_xh(x)))) <= SliceFarFieldCheck.SECTOR_HALF_DEG:
			cols.append(x)
	return cols


func _covered(hd: PackedByteArray, ed: PackedByteArray, x: int, row: int) -> bool:
	return SliceFarFieldCheck._differs(hd, ed, (row * _w + x) * 3)


static func _pct(a: int, b: int) -> float:
	return 100.0 * float(a) / float(maxi(b, 1))


# --- the I frame's criteria --------------------------------------------------------------------

## Every criterion on one I frame from `pos` facing `az`; `sd`, `hd`, `ed` are the same pose's S
## and H frames and the sky frame. Returns the number of failures (each one printed).
func judge_frame(where: String, pos: Vector3, az: float, ids: PackedByteArray, sd: PackedByteArray,
		hd: PackedByteArray, ed: PackedByteArray, bay: bool) -> int:
	var unknown := ids.count(UNKNOWN)
	if unknown > UNKNOWN_MAX * ids.size():
		print("FAIL farfield-id %s: unreadable (%d unknown px)" % [where, unknown])
		return 1
	var cols := _columns()
	var fails := _order(where, pos, az, ids, hd, ed, cols)
	fails += _crest(where, pos, az, ids, hd, ed, cols)
	fails += _seam(where, pos, az, ids, hd, ed, cols)
	fails += _relief(where, ids, sd, hd, ed, cols)
	if bay:
		fails += _bay(where, pos, az, ids, hd, ed, cols)
	return fails


## FF-ORDER (D-1): where a band nearer than the sea horizon stands in a water sector, the pixel two
## rows below its expected crest is that band or something nearer -- never water or farther land.
func _order(where: String, pos: Vector3, az: float, ids: PackedByteArray, hd: PackedByteArray,
		ed: PackedByteArray, cols: PackedInt32Array) -> int:
	var judged := {}
	var good := {}
	var near_ids := [id_of("massing"), id_of("foreground"), id_of("ground")]
	for x in cols:
		var b := _bearing(az, x)
		var body := water_at(b)
		if body.is_empty():
			continue
		var dh := sea_horizon_m(pos.y, float(body["level_m"]))
		for k in _bands.size():
			if not MineWorldFarField.sector_contains(_bands[k]["sector"], b):
				continue
			var ad := band_angle(_bands[k], b, pos.y)
			if ad.y >= dh:
				continue
			var top := int(ceil(_row(x, rad_to_deg(ad.x))))
			var r := top + 1
			if top < 0 or r >= _h or _covered(hd, ed, x, top) or _covered(hd, ed, x, r):
				continue
			# the probe two rows down must still lie on the band: where its end taper has brought
			# its crest within that of its foot on the water, the probe tests nothing about order
			var foot := rad_to_deg(angle(float(body["level_m"]), ad.y, pos.y))
			if _row(x, foot) < r + 1:
				continue
			judged[k] = judged.get(k, 0) + 1
			var id := ids[r * _w + x]
			var ok := id == id_of("band:" + str(_bands[k]["name"])) or id in near_ids
			if not ok and is_band(id):
				ok = crest_hd(_bands[band_index(id)], b).y < ad.y
			if ok:
				good[k] = good.get(k, 0) + 1
	var fails := 0
	for k in judged:
		var n: int = judged[k]
		var g: int = good.get(k, 0)
		var name := str(_bands[k]["name"])
		if n < MIN_JUDGED:
			print("[farfield-order]  %s  %s  none to judge (%d columns)" % [where, name, n])
		elif g >= ORDER_NEED * n:
			print("[farfield-order]  %s  %s  %d/%d columns land over water (%.1f%%) >= %d%%   ok"
				% [where, name, g, n, _pct(g, n), int(ORDER_NEED * 100)])
		else:
			print("FAIL farfield-order %s %s: %d/%d columns land over water (%.1f%%) < %d%%"
				% [where, name, g, n, _pct(g, n), int(ORDER_NEED * 100)])
			fails += 1
	return fails


## FF-CREST (D-2): the topmost far-field part in a column is a band, within CREST_TOL_DEG of the
## highest expected crest there (with this eye's parallax and the end taper).
func _crest(where: String, pos: Vector3, az: float, ids: PackedByteArray, hd: PackedByteArray,
		ed: PackedByteArray, cols: PackedInt32Array) -> int:
	var judged := 0
	var good := 0
	var err := 0.0
	var eh := sea_horizon_deg(pos.y)
	for x in cols:
		var b := _bearing(az, x)
		var best := -INF
		for band in _bands:
			if MineWorldFarField.sector_contains(band["sector"], b):
				best = maxf(best, band_angle(band, b, pos.y).x)
		if best == -INF:
			continue
		var expect := rad_to_deg(best)
		if not water_at(b).is_empty() and eh >= expect - CREST_TOL_DEG:
			continue   # the sea's horizon, not a band, is the top here
		var rc := _row(x, expect)
		if rc < CREST_CLEAR_ROWS or rc > _h - 1 - CREST_CLEAR_ROWS:
			continue
		var c := int(round(rc))
		var clear := true
		for row in range(c - CREST_CLEAR_ROWS, c + CREST_CLEAR_ROWS + 1):
			if _covered(hd, ed, x, row):
				clear = false
				break
		if not clear:
			continue
		judged += 1
		for row in _h:
			var id := ids[row * _w + x]
			if id == SKY_ID or id == UNKNOWN:
				continue
			var off := _elev(x, row) - expect
			if is_band(id) and absf(off) <= CREST_TOL_DEG:
				good += 1
			else:
				err += off
			break
	if judged < MIN_JUDGED:
		print("[farfield-crest]  %s  none to judge (%d columns)" % [where, judged])
		return 0
	if good >= CREST_NEED * judged:
		print("[farfield-crest]  %s  %d/%d columns within %.2f deg (%.1f%%) >= %d%%   ok"
			% [where, good, judged, CREST_TOL_DEG, _pct(good, judged), int(CREST_NEED * 100)])
		return 0
	print("FAIL farfield-crest %s: %d/%d within %.2f deg (%.1f%%) < %d%% (the others off by %+.2f deg on average)"
		% [where, good, judged, CREST_TOL_DEG, _pct(good, judged), int(CREST_NEED * 100),
			err / maxf(judged - good, 1)])
	return 1


## FF-SEAM (D-3): no two adjacent columns differ in part over SEAM_ROWS rows or more, where one of
## them is a band, the sea ribbon or water (massing and covered pixels excepted).
func _seam(where: String, pos: Vector3, az: float, ids: PackedByteArray, hd: PackedByteArray,
		ed: PackedByteArray, cols: PackedInt32Array) -> int:
	var from := sea_horizon_deg(pos.y) + SEAM_FROM_DEG
	var massing := id_of("massing")
	var wet := [id_of("water"), id_of("sea-ribbon")]
	var seams := []
	for ci in range(cols.size() - 1):
		var x := cols[ci]
		if cols[ci + 1] != x + 1:
			continue
		var run := 0
		var longest := 0
		var pair := []
		for row in _h:
			if _elev(x, row) < from:
				break
			var a := ids[row * _w + x]
			var c := ids[row * _w + x + 1]
			var differ := a != c and a != massing and c != massing and a != UNKNOWN and c != UNKNOWN \
				and (is_band(a) or is_band(c) or a in wet or c in wet) \
				and not _covered(hd, ed, x, row) and not _covered(hd, ed, x + 1, row)
			if differ:
				run += 1
				if run > longest:
					longest = run
					pair = [a, c]
			else:
				run = 0
		if longest >= SEAM_ROWS:
			seams.append([_bearing(az, x), pair, longest])
	if seams.is_empty():
		print("[farfield-seam]   %s  0 seams   ok" % where)
		return 0
	print("FAIL farfield-seam %s: %d seams" % [where, seams.size()])
	for s in seams.slice(0, 8):
		print("  seam at bearing %.2f between %s and %s (%d rows)"
			% [s[0], name_of(s[1][0]), name_of(s[1][1]), s[2]])
	return 1


## FF-RELIEF (D-4): each near band's uncovered pixels, RELIEF_BELOW_ROWS or more under its measured
## crest, vary in luma within a row: the mean of the luma's deviation over 1-degree windows.
func _relief(where: String, ids: PackedByteArray, sd: PackedByteArray, hd: PackedByteArray,
		ed: PackedByteArray, cols: PackedInt32Array) -> int:
	var fails := 0
	for k in _bands.size():
		if _bands[k]["layer"] != "near":
			continue
		var bid := id_of("band:" + str(_bands[k]["name"]))
		var top := {}
		for x in cols:
			for row in _h:
				if ids[row * _w + x] == bid:
					top[x] = row
					break
		var ok := PackedByteArray()
		ok.resize(_w * _h)
		var count := 0
		for x in top:
			for row in range(int(top[x]) + RELIEF_BELOW_ROWS, _h):
				if ids[row * _w + x] == bid and not _covered(hd, ed, x, row):
					ok[row * _w + x] = 1
					count += 1
		if count < RELIEF_MIN_PX:
			continue
		var sum_sd := 0.0
		var windows := 0
		for row in _h:
			var start := 0
			while start + RELIEF_WINDOW <= cols.size():
				var lum := PackedFloat32Array()
				for ci in range(start, start + RELIEF_WINDOW):
					var i := row * _w + cols[ci]
					if ok[i] == 0:
						break
					lum.append(0.2126 * sd[i * 3] + 0.7152 * sd[i * 3 + 1] + 0.0722 * sd[i * 3 + 2])
				start += RELIEF_WINDOW
				if lum.size() < RELIEF_WINDOW:
					continue
				var mean := 0.0
				for l in lum:
					mean += l
				mean /= lum.size()
				var v := 0.0
				for l in lum:
					v += (l - mean) * (l - mean)
				sum_sd += sqrt(v / lum.size())
				windows += 1
		var name := str(_bands[k]["name"])
		if windows == 0:
			print("[farfield-relief] %s  %s  none to judge (no full 1-degree window)" % [where, name])
			continue
		var mean_sd := sum_sd / windows
		if mean_sd >= RELIEF_NEED:
			print("[farfield-relief] %s  %s  luma sd %.1f/255 >= %.1f   ok" % [where, name, mean_sd, RELIEF_NEED])
		else:
			print("FAIL farfield-relief %s %s: luma sd %.1f/255 < %.1f (%d windows)"
				% [where, name, mean_sd, RELIEF_NEED, windows])
			fails += 1
	return fails


## FF-BAY (SD-5's guard): from a protected eye, a column of a water sector that is uncovered from
## the water's expected near edge up to the sea horizon shows the water in at least one row.
func _bay(where: String, pos: Vector3, az: float, ids: PackedByteArray, hd: PackedByteArray,
		ed: PackedByteArray, cols: PackedInt32Array) -> int:
	var judged := 0
	var good := 0
	var wet := [id_of("water"), id_of("sea-ribbon")]
	var eh := sea_horizon_deg(pos.y)
	var o := Vector2(pos.x, pos.z)
	for x in cols:
		var b := _bearing(az, x)
		var body := water_at(b)
		if body.is_empty():
			continue
		# within T_WATER of the water's ends SD-3.2 and SD-3.3 narrow the water to nothing on
		# purpose; the guard judges the bearings where the full bay is drawn
		if end_factor(body["sector"], b, T_WATER) < 1.0:
			continue
		var dv := Vector2(sin(deg_to_rad(b)), -cos(deg_to_rad(b)))
		var s := shore(body, b)
		var p := o.dot(dv)
		var t := -p + sqrt(maxf(p * p - (o.length_squared() - s * s), 0.0))
		var es := rad_to_deg(atan((float(body["level_m"]) - _datum - pos.y) / maxf(t, 1.0)))
		var r0 := int(ceil(_row(x, eh)))
		var r1 := int(floor(_row(x, es)))
		if r0 < 0 or r1 >= _h or r1 < r0:
			continue
		var clear := true
		for row in range(r0, r1 + 1):
			if _covered(hd, ed, x, row):
				clear = false
				break
		if not clear:
			continue
		judged += 1
		for row in range(r0, r1 + 1):
			if ids[row * _w + x] in wet:
				good += 1
				break
	if judged < MIN_JUDGED:
		return 0
	if good >= BAY_NEED * judged:
		print("[farfield-bay]    %s  %d/%d columns show water (%.1f%%) >= %d%%   ok"
			% [where, good, judged, _pct(good, judged), int(BAY_NEED * 100)])
		return 0
	print("FAIL farfield-bay %s: %d/%d columns show water (%.1f%%) < %d%%"
		% [where, good, judged, _pct(good, judged), int(BAY_NEED * 100)])
	return 1


# --- the G frame -------------------------------------------------------------------------------

## FF-RING (D-5): looking down from `pos` at bearing `az` (G_PITCH, G_SIZE, G_HFOV), each ground
## pixel's ground radius, by ray-ground intersection; in 20 m bins from reach_m + 20 to 400 m within
## the sector, a bin of RING_MIN_PX or more that is less than half water is RING_NEED massing.
func judge_ring(where: String, pos: Vector3, az: float, ids: PackedByteArray) -> int:
	var reach: float = ff.boundary()["reach_m"]
	var lo := reach + RING_FROM_M
	var n := maxi(int(ceil((RING_TO_M - lo) / RING_BIN_M)), 0)
	var total := PackedInt32Array()
	var mass := PackedInt32Array()
	var wet := PackedInt32Array()
	total.resize(n)
	mass.resize(n)
	wet.resize(n)
	var land := [id_of("ground"), id_of("foreground"), id_of("massing")]
	var water := [id_of("water"), id_of("sea-ribbon")]
	var massing := id_of("massing")
	var th := tan(deg_to_rad(G_HFOV * 0.5))
	var tv := th * float(G_SIZE.y) / float(G_SIZE.x)
	var cp := cos(deg_to_rad(G_PITCH))
	var sp := sin(deg_to_rad(G_PITCH))
	var cy := cos(deg_to_rad(-az))
	var sy := sin(deg_to_rad(-az))
	for y in G_SIZE.y:
		var yv := (G_SIZE.y * 0.5 - 0.5 - y) / (G_SIZE.y * 0.5) * tv
		var dy := yv * cp + sp
		if dy >= 0.0:
			continue
		var z1 := yv * sp - cp
		for x in G_SIZE.x:
			var id := ids[y * G_SIZE.x + x]
			var is_land := id in land
			if not is_land and not id in water:
				continue
			var xh := (x + 0.5 - G_SIZE.x * 0.5) / (G_SIZE.x * 0.5) * th
			var dx := cy * xh + sy * z1
			var dz := -sy * xh + cy * z1
			var t := (pos.y - SliceFarFieldView.GROUND_Y) / -dy
			for i in 4:
				var px := pos.x + t * dx
				var pz := pos.z + t * dz
				t = (pos.y - ground_y(sqrt(px * px + pz * pz))) / -dy
			var hx := pos.x + t * dx
			var hz := pos.z + t * dz
			var r := sqrt(hx * hx + hz * hz)
			if r < lo or r >= lo + n * RING_BIN_M:
				continue
			var bearing := fposmod(rad_to_deg(atan2(hx, -hz)), 360.0)
			if absf(fposmod(bearing - az + 180.0, 360.0) - 180.0) > SliceFarFieldCheck.SECTOR_HALF_DEG:
				continue
			var k := int((r - lo) / RING_BIN_M)
			total[k] += 1
			if id == massing:
				mass[k] += 1
			elif not is_land:
				wet[k] += 1
	var fails := 0
	for k in n:
		if total[k] < RING_MIN_PX or wet[k] >= RING_WATER_MAX * total[k]:
			continue
		var a := lo + k * RING_BIN_M
		var share := float(mass[k]) / float(total[k])
		if share >= RING_NEED:
			print("[farfield-ring]   %s  %d-%d m  massing %.1f%% >= %d%%   ok"
				% [where, int(a), int(a + RING_BIN_M), 100.0 * share, int(RING_NEED * 100)])
		else:
			print("FAIL farfield-ring %s %d-%d m: massing %.1f%% < %d%% (%d px)"
				% [where, int(a), int(a + RING_BIN_M), 100.0 * share, int(RING_NEED * 100), total[k]])
			fails += 1
	return fails
