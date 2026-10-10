## The far skyline: a PREVIEW of San Diego's real mountains, by angle (RL-b SC-8,
## `.structured-coding/plans/mvp1/pr-rl-b-3d-budget.md` §5; operator rulings
## QRL-2 and QRL-11, 2026-10-09).
##
## WHY IT REPLACES THE RIDGES. The old backdrop was three sine-sum strips 320-760 m
## north of the street, up to 15.7 degrees high, in flat colour. Real mountains
## seen from a town read by their ANGLE, and San Diego's are low: about 1-2 degrees
## above the horizon. A ridge drawn close and tall reads as a stage flat
## (step-22 §2.2, causes 1, 4 and 5). So this draws named summits at their real
## bearings and real elevation angles, layered by real distance, hazed by
## Koschmieder's law.
##
## WHAT IS REAL, AND WHAT IS A PLACEHOLDER. Real: each summit's bearing, distance
## and elevation angle, computed below from its published coordinates and height
## (sources in SUMMITS), and the haze from its real distance. A PLACEHOLDER: the
## shape between summits -- a smooth interpolation with low, slow relief, falling
## to the horizon at each sector's edge. RL-e replaces it with a profile computed
## from USGS 3DEP elevation data. This preview fixes size, direction and
## atmosphere, and nothing else.
##
## GEOMETRY. Each band is a ribbon at a proxy distance inside the camera's far
## plane (near 900 m, mid 1 100 m, far 1 300 m; `CameraRig.FAR` is 1 600 m), its
## crest at `D * tan(angle)` above the eye. The root follows the active camera's
## position every frame and never rotates, so the bands show no parallax, as
## mountains 8-72 km away show none at walking scale. Unshaded, no fog (the depth
## fog is tuned for 70-1 500 m and would count the haze twice), no shadows.
##
## FRAME: +x east, -z north (`street.gd`). Bearing b (clockwise from north) is the
## direction (sin b, 0, -cos b).
class_name SliceSkyline
extends Node3D

## The observer: Market Town's configured location
## (`worlds/market-town/configure/calendar.yaml`), eye about 20 m above sea level.
const ORIGIN_LAT := 32.7157
const ORIGIN_LON := -117.1611
const EYE_M := 20.0
## Mean Earth radius (IUGG), metres.
const EARTH_R := 6371000.0
## Terrestrial refraction coefficient: Gauss's average k ~ 0.13, "a frequently
## used standard value" (Hirt et al. 2010, JGR 115 D21102). Re-read 2026-10-09.
const REFRACTION_K := 0.13
## Meteorological optical range for the preview's haze: a clear coastal
## afternoon. A presentation default (Q-RLb-5); TW-e takes it from the weather.
const VISIBILITY_M := 80000.0
## Koschmieder at the WMO 5 % contrast threshold: contrast C = exp(-3.0 d / V),
## since ln(1/0.05) = 2.996 (WMO-No. 8, CIMO Guide, Part I ch. 9). Re-read 2026-10-09.
const KOSCHMIEDER := 2.996
## Dry chaparral and coastal sage, sRGB: a design default inside DEP-8's albedo
## band 0.2-0.7 (step-22 §5.1).
const TERRAIN_SRGB := Color(0.38, 0.36, 0.27)
## Crest-to-base luminance falloff that suggests slope shading.
const BASE_DARKEN := 0.08
## Relief between summits: at most this share of the local angle, at least
## RELIEF_MIN_DEG of azimuth per undulation. Placeholder shape (header).
const RELIEF := 0.12
const RELIEF_MIN_DEG := 8.0
## Each ribbon reaches this far below the eye, so the far edge of the ground
## plane never shows sky under a band.
const BASE_DEG := -1.0

## Summits, re-read at source 2026-10-09 (RL-b ledger E-RLb-7):
##   [name, latitude, longitude, height m, band, source]
const SUMMITS := [
	["Point Loma", 32.67199, -117.24097, 126.0, "near",
		"GeoNames (USGS GNIS), Old Point Loma Lighthouse, the peninsula's high point"],
	["Mount Soledad", 32.839866731, -117.252491236, 251.0, "near",
		"NGS datasheet 'Soledad', via Wikipedia"],
	["Cowles Mountain", 32.8125497, -117.0311403, 486.0, "mid",
		"GNIS coordinates, Peakbagger height, via Wikipedia"],
	["Mount Helix", 32.7669958, -116.9833603, 416.0, "mid",
		"GeoNames (USGS GNIS); other sources 406-419 m"],
	["San Miguel Mountain", 32.696413711, -116.936399531, 783.0, "mid",
		"NGS datasheet 'San Miguel Reset', via Wikipedia"],
	["Otay Mountain", 32.594567222, -116.844671506, 1088.0, "far",
		"NGS datasheet 'Otay' (PID DC2046), via Wikipedia"],
	["Cuyamaca Peak", 32.946743453, -116.606723761, 1985.0, "far",
		"NGS datasheet 'Cuyamaca reset', via Wikipedia"],
	["Monument Peak", 32.8925502, -116.4202938, 1911.0, "far",
		"hundredpeaks.org 6 271 ft; GeoNames coordinates"],
]

## [band, proxy distance m, sectors as [from, to] bearings in degrees]. Each
## sector holds that band's summits and falls to the horizon at its edges. The
## west, 255-310 degrees, is open sea: no band (A-8).
const BANDS := [
	["far", 1300.0, [[48.0, 128.0]]],
	["mid", 1100.0, [[30.0, 112.0]]],
	["near", 900.0, [[226.0, 255.0], [310.0, 352.0]]],
]
const SEA_FROM := 255.0
const SEA_TO := 310.0

## Columns per degree of azimuth.
const STEPS_PER_DEG := 2

## Set by a probe mutation only (X-5): rotates the drawn bearings.
static var bearing_offset_deg := 0.0


## Build the skyline under `parent`, after the HDRI is loaded.
static func build(parent: Node3D) -> SliceSkyline:
	var s := SliceSkyline.new()
	s.name = "Skyline"
	s.set_meta("mw_category", "backdrop")
	parent.add_child(s)
	var horizon := _horizon_sampler(parent.get_tree().root)
	for b in BANDS:
		s.add_child(_band(b, horizon))
	return s


func _process(_d: float) -> void:
	var cam := get_viewport().get_camera_3d()
	if cam != null:
		global_position = cam.global_position


# --- the numbers -------------------------------------------------------------------

## Bearing (degrees, clockwise from north) and great-circle distance (m) from the
## town origin: the initial bearing and the haversine distance on a sphere.
static func bearing_distance(lat: float, lon: float) -> Vector2:
	var p1 := deg_to_rad(ORIGIN_LAT)
	var p2 := deg_to_rad(lat)
	var dl := deg_to_rad(lon - ORIGIN_LON)
	var y := sin(dl) * cos(p2)
	var x := cos(p1) * sin(p2) - sin(p1) * cos(p2) * cos(dl)
	var brg := fposmod(rad_to_deg(atan2(y, x)), 360.0)
	var a := sin((p2 - p1) * 0.5) ** 2 + cos(p1) * cos(p2) * sin(dl * 0.5) ** 2
	return Vector2(brg, 2.0 * EARTH_R * asin(sqrt(a)))


## Apparent elevation angle (degrees) of a point `h` m high at distance `d` m,
## with Earth curvature and refraction: (h - eye - d^2 (1 - k) / 2R) / d.
static func elevation_deg(h: float, d: float) -> float:
	return rad_to_deg((h - EYE_M - d * d * (1.0 - REFRACTION_K) / (2.0 * EARTH_R)) / d)


## Every summit as [name, band, bearing, distance m, elevation deg].
static func table() -> Array:
	var out: Array = []
	for s in SUMMITS:
		var bd := bearing_distance(s[1], s[2])
		out.append([s[0], s[4], bd.x, bd.y, elevation_deg(s[3], bd.y)])
	return out


## The band's drawn angle at bearing `az` (degrees above the eye; BASE_DEG
## outside its sectors), and the distance its colour is computed from. The
## shape between summits is the placeholder the header describes; at a summit's
## bearing it is that summit's angle exactly.
static func profile(band: String, az: float) -> Vector2:
	az = fposmod(az, 360.0)
	var spec: Array = BANDS.filter(func(b: Array) -> bool: return b[0] == band)[0]
	for sec in spec[2]:
		var a0: float = sec[0]
		var a1: float = sec[1]
		if az < a0 or az > a1:
			continue
		# anchors: the sector's edges at the horizon, its summits between
		var anchors: Array = [[a0, 0.0, 0.0]]
		var ts := table().filter(func(t: Array) -> bool:
			return t[1] == band and t[2] >= a0 and t[2] <= a1)
		ts.sort_custom(func(p: Array, q: Array) -> bool: return p[2] < q[2])
		for t in ts:
			anchors.append([t[2], t[4], t[3]])
		anchors.append([a1, 0.0, 0.0])
		for i in range(anchors.size() - 1):
			var p: Array = anchors[i]
			var q: Array = anchors[i + 1]
			if az < p[0] or az > q[0]:
				continue
			var span: float = q[0] - p[0]
			var t: float = 0.0 if span <= 0.0 else (az - float(p[0])) / span
			var e := smoothstep(0.0, 1.0, t)
			var ang: float = lerpf(p[1], q[1], e)
			# relief: whole half-waves between two anchors, so it vanishes at each
			var halves := maxi(1, int(floor(span / (RELIEF_MIN_DEG * 0.5))))
			ang *= 1.0 + RELIEF * sin(PI * halves * t)
			# distance for the haze: the summits' own, the nearer one at an edge
			var dp: float = p[2] if p[2] > 0.0 else q[2]
			var dq: float = q[2] if q[2] > 0.0 else p[2]
			return Vector2(ang, lerpf(dp, dq, e))
	return Vector2(BASE_DEG, 0.0)


# --- the drawing -------------------------------------------------------------------

static func _band(b: Array, horizon: Callable) -> MeshInstance3D:
	var band: String = b[0]
	var dp: float = b[1]
	var terrain := _terrain_radiance()
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for sec in b[2]:
		var a0: float = sec[0]
		var a1: float = sec[1]
		var n := int(ceil((a1 - a0) * STEPS_PER_DEG))
		for i in range(n):
			var az0 := a0 + (a1 - a0) * float(i) / n
			var az1 := a0 + (a1 - a0) * float(i + 1) / n
			_column(st, band, dp, az0, az1, horizon, terrain)
	var mi := MeshInstance3D.new()
	mi.name = "Band_" + band
	mi.mesh = st.commit()
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.vertex_color_use_as_albedo = true
	m.disable_fog = true
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	mi.material_override = m
	return mi


static func _column(st: SurfaceTool, band: String, dp: float, az0: float, az1: float,
		horizon: Callable, terrain: Color) -> void:
	var p0 := profile(band, az0)
	var p1 := profile(band, az1)
	var top0 := _point(az0, p0.x, dp)
	var top1 := _point(az1, p1.x, dp)
	var bot0 := _point(az0, BASE_DEG, dp)
	var bot1 := _point(az1, BASE_DEG, dp)
	var c0 := _colour(p0.y, horizon.call(az0), terrain)
	var c1 := _colour(p1.y, horizon.call(az1), terrain)
	var b0 := c0.darkened(BASE_DARKEN)
	var b1 := c1.darkened(BASE_DARKEN)
	for v in [[bot0, b0], [top0, c0], [top1, c1], [bot0, b0], [top1, c1], [bot1, b1]]:
		st.set_color(v[1])
		st.add_vertex(v[0])


## The point at bearing `az`, `elev` degrees above the eye, at proxy distance `dp`.
static func _point(az: float, elev: float, dp: float) -> Vector3:
	var a := deg_to_rad(az + bearing_offset_deg)
	return Vector3(sin(a) * dp, dp * tan(deg_to_rad(elev)), -cos(a) * dp)


## Koschmieder: what is left of the land's own radiance at distance `d`, the
## rest the horizon sky's (linear colour, as vertex colours are).
static func _colour(d: float, horizon: Color, terrain: Color) -> Color:
	var c := exp(-KOSCHMIEDER * d / VISIBILITY_M) if d > 0.0 else 0.0
	return horizon.lerp(terrain, c)


## The land's radiance in the slice's light: albedo times what falls on a slope
## facing the viewer -- the sun's share at its elevation plus the sky's. An
## estimate for a preview, stated as one.
static func _terrain_radiance() -> Color:
	var albedo := TERRAIN_SRGB.srgb_to_linear()
	var light := SliceMain.SUN_ENERGY * sin(deg_to_rad(-SliceMain.SUN_ELEVATION)) \
		+ SliceMain.SKY_AMBIENT * 0.8
	return Color(albedo.r * light, albedo.g * light, albedo.b * light)


## The HDRI's colour at the horizon for a bearing: the mean of the panorama rows
## within one degree of the horizon at that azimuth, times the sky's energy, so
## the band meets the sky without a seam. Falls back to the fog colour.
static func _horizon_sampler(root: Node) -> Callable:
	var img: Image = null
	var energy := 1.0
	var envs := root.find_children("*", "WorldEnvironment", true, false)
	if not envs.is_empty():
		var sky := (envs[0] as WorldEnvironment).environment.sky
		if sky != null and sky.sky_material is PanoramaSkyMaterial:
			var pm := sky.sky_material as PanoramaSkyMaterial
			energy = pm.energy_multiplier
			if pm.panorama != null:
				img = pm.panorama.get_image()
				if img != null and img.is_compressed():
					img.decompress()
	if img == null:
		var fog := Color(0.76, 0.80, 0.87).srgb_to_linear()
		return func(_az: float) -> Color: return fog
	var w := img.get_width()
	var h := img.get_height()
	var rows := maxi(1, int(round(h / 180.0)))   # one degree of rows
	var cache := {}
	return func(az: float) -> Color:
		var key := int(round(az))
		if cache.has(key):
			return cache[key]
		# Godot's panorama mapping: u = atan2(x, z) / 2pi (wrapped), v = acos(y) / pi
		var a := deg_to_rad(float(key))
		var u := fposmod(atan2(sin(a), -cos(a)) / TAU, 1.0)
		var x := clampi(int(u * w), 0, w - 1)
		var sum := Color(0, 0, 0)
		var n := 0
		for dy in range(-rows, rows + 1):
			var y := clampi(h / 2 + dy, 0, h - 1)
			sum += img.get_pixel(x, y)
			n += 1
		var c := Color(sum.r / n * energy, sum.g / n * energy, sum.b / n * energy)
		cache[key] = c
		return c


# --- the check (`--skyline-check`, A-8, X-5) ---------------------------------------

## Cast rays at the drawn bands from the eye at each whole degree of azimuth,
## find the crest by bisection on elevation, and compare it with the angle the
## summit table gives (`profile`). PASS iff every degree is within 0.1 degrees,
## no crest exceeds 2.3 degrees, and nothing is drawn over the sea sector.
## The expectation comes from the published coordinates; the measurement from
## the mesh as built, so a mesh drawn at the wrong bearing fails (X-5).
static func check(sky: SliceSkyline) -> bool:
	var tris := {}   # whole degree -> Array of [a, b, c] in the root's frame
	for mi in sky.get_children():
		var mesh := (mi as MeshInstance3D).mesh
		var v: PackedVector3Array = mesh.surface_get_arrays(0)[Mesh.ARRAY_VERTEX]
		for i in range(0, v.size(), 3):
			var t := [v[i], v[i + 1], v[i + 2]]
			var azs: Array[float] = []
			for p in t:
				azs.append(fposmod(rad_to_deg(atan2(p.x, -p.z)), 360.0))
			azs.sort()
			var lo := azs[0]
			var hi := azs[2]
			if hi - lo > 180.0:   # straddles north
				lo = azs[2] - 360.0
				hi = azs[0]
			for dd in range(int(floor(lo)) - 1, int(ceil(hi)) + 2):
				var k := posmod(dd, 360)
				if not tris.has(k):
					tris[k] = []
				tris[k].append(t)
	var worst := 0.0
	var worst_at := -1
	var max_crest := -90.0
	var sea_hits := 0
	var checked := 0
	print("== skyline check (A-8): drawn crest vs the summit table, 1 deg steps ==")
	for t in table():
		print("summit %-20s band %-4s bearing %6.1f deg  distance %5.1f km  elevation %.2f deg"
			% [t[0], t[1], t[2], t[3] / 1000.0, t[4]])
	for az in range(360):
		var expect := -90.0
		for b in BANDS:
			expect = maxf(expect, profile(b[0], float(az)).x)
		# Whole degrees fall on column seams, where a ray can slip between two
		# triangles; the crest is continuous there, so take the higher of two rays
		# a thousandth of a degree either side.
		var near: Array = tris.get(az, [])
		var drawn := maxf(_crest(near, az - 0.001), _crest(near, az + 0.001))
		if float(az) > SEA_FROM and float(az) < SEA_TO and drawn > BASE_DEG + 0.05:
			sea_hits += 1
		if expect <= BASE_DEG and drawn <= BASE_DEG + 0.05:
			continue
		checked += 1
		max_crest = maxf(max_crest, drawn)
		var err := absf(drawn - expect)
		if err > 0.1:
			print("  %3d deg: drawn %.3f, table %.3f" % [az, drawn, expect])
		if err > worst:
			worst = err
			worst_at = az
	var ok := worst <= 0.1 and max_crest <= 2.3 and sea_hits == 0
	print("checked %d degrees; worst |drawn - table| %.3f deg at %d deg; highest crest %.2f deg;"
		% [checked, worst, worst_at, max_crest] + " %d sea-sector degrees drawn" % sea_hits)
	print("skyline check %s (bound 0.1 deg; crest <= 2.3 deg; sea 255-310 open)"
		% ("PASS" if ok else "FAIL"))
	return ok


## The highest elevation (degrees) at which a ray from the eye at bearing `az`
## still meets one of `tris`, by bisection; BASE_DEG if none.
static func _crest(tris: Array, az: float) -> float:
	if tris.is_empty():
		return BASE_DEG
	var a := deg_to_rad(az)
	var lo := BASE_DEG + 0.02
	if not _hits(tris, a, lo):
		return BASE_DEG
	var hi := 6.0
	for i in range(24):
		var mid := (lo + hi) * 0.5
		if _hits(tris, a, mid):
			lo = mid
		else:
			hi = mid
	return lo


static func _hits(tris: Array, a: float, elev_deg: float) -> bool:
	var e := deg_to_rad(elev_deg)
	var dir := Vector3(sin(a) * cos(e), sin(e), -cos(a) * cos(e))
	for t in tris:
		if Geometry3D.ray_intersects_triangle(Vector3.ZERO, dir, t[0], t[1], t[2]) != null:
			return true
	return false
