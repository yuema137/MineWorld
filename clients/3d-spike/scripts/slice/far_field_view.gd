## The far field: every direction from the street closes on sky, terrain, water and the town's
## distant massing (`docs/MODULE_SPEC.md` sec.6.1.1, `docs/DECISIONS.md` ARC-102; RL-e0 sec.5.4).
##
## WHAT IT REPLACES. Three ridges north of the street (`street.gd` backdrop, removed) that covered
## about +-44 degrees of azimuth and were wound so Godot culled the faces the player saw, over a
## 900 m grass apron whose edge showed against the sky to the east, south and west (E-FF-3 .. 5).
##
## WHAT IT DRAWS, all from the Presentation Pack's `far_field/social-cafe.yaml`:
##
##   silhouette   terrain bands at their real bearings and angles, one mesh per band, as ribbons
##                that follow the camera in x and z (no parallax at walking scale, as 2-72 km of
##                distance gives none); the eye's height moves each column by -rise/d of its own
##                real distance d (`shaders/far_silhouette.gdshader`, RL-e0b SD-2)
##   foreground   the ground from the boundary outward, falling `drop_m` over `run_m`, world-anchored
##   water        the bay's surface, world-anchored, and a camera-anchored ribbon of sea out to the
##                true dip of the horizon
##   skyline      low-rise massing, deterministic from the file's seed, world-anchored, the reserved
##                station slot kept clear, roofs capped below the protected eyes' sightlines
##
## Plus the near ground under the street: a disc of the boundary's radius (category "ground").
##
## DRAWING RULES FROM ANY HEIGHT (RL-e0b, `docs/decisions/ARC-290.md`):
##   depth     every camera-anchored column sits at a proxy radius P(d) that grows with its real
##             distance, and the sea ribbon at P of the sea horizon's distance from the eye, so the
##             depth test orders land and water as reality does (SD-1)
##   ends      a band's crest falls to its foot (the surface at its distance) over T_BAND at each
##             end; the bay's near edge curves out to OUTER_M and the sea ribbon descends to its
##             base over T_WATER at each end: no vertical cut (SD-3)
##   texture   near bands carry noise whose contrast fades with the haze (SD-4)
##   ring      massing from the file's `from_m`, each roof kept CAP_MARGIN_DEG below what it stands
##             in front of, seen from every protected eye the scene names (SD-5)
##
## WINDING (SD-FF-4). Triangles are emitted in Godot's front-face order -- clockwise as seen by the
## camera that is meant to see them -- and `SurfaceTool.generate_normals()` derives the normals. The
## materials keep back-face culling. Nothing here asserts a winding: the rendered check
## (`slice_far_field_check.gd`) proves the faces are drawn, and reversing them fails it (X-1).
##
## FRAME: +x east, -z north (`street.gd`); bearing b is the direction (sin b, 0, -cos b).
class_name SliceFarFieldView
extends Node3D

## The street's eye at standing height above the pavement (Social Café's spawn at eye height).
const EYE0 := SliceStreet.WALK_Y + CameraRig.EYE_HEIGHT
## Proxy radii (SD-1): a camera-anchored column at real distance d is drawn at
## P(d) = P_MIN + (P_MAX - P_MIN) * clamp(ln(d / D_LO) / ln(D_HI / D_LO), 0, 1). P_MIN lies beyond
## the world-anchored foreground and water (OUTER_M), P_MAX inside the camera's far plane
## (`CameraRig.FAR` 1 600 m, unchanged), so everything world-anchored is in front of every proxy and
## the proxies are ordered as their real distances are.
const P_MIN := 1460.0
const P_MAX := 1590.0
## The real distances the log scale spans: the nearest band and beyond the farthest one.
const D_LO := 1500.0
const D_HI := 120000.0
## How far the world-anchored foreground and water reach.
const OUTER_M := 1450.0
## SD-3: a band's crest falls to its foot over this much of each end (or a quarter of its sector,
## whichever is less), and the bay's edges and the sea ribbon's ends over T_WATER. Wider than the
## steepest cited profile slope, so the descent is a slope and not a cut.
const T_BAND := 3.0
const T_WATER := 4.0
## In a water sector a band stops this far below its foot, so no sky shows between it and the sea.
const FOOT_PAD_RAD := 0.05 * PI / 180.0
## SD-5: a ring roof stays at least this far below what it stands in front of, from every protected
## eye; a box whose cap is under CAP_MIN_M is a wall or shed of at least CAP_FLOOR_M, one whose cap
## is under CAP_FLOOR_M is not built.
const CAP_MARGIN_DEG := 0.25
const CAP_MIN_M := 2.5
const CAP_FLOOR_M := 1.0
## The sky's share in the sea ribbon's colour, the rest the water's deep colour (RL-e0 Q-FF-2 caps the
## Fresnel share at 0.6). The ribbon is drawn fog-free and compared with a sky that the depth fog
## dims, so at 0.6 it matched that sky where the panorama's horizon is bright; 0.4 keeps the far
## water apart from the sky at every bearing (RL-e0b, the rotated and Lakeside runs).
const RIBBON_FRESNEL := 0.4
## SD-4: the band texture's contrast at zero distance.
const TEXTURE_AMP := 0.45
## Mean Earth radius (IUGG), metres.
const EARTH_R := 6371000.0
## Terrestrial refraction coefficient k = 0.13 (Hirt et al. 2010, JGR 115 D21102; RL-b C8).
const REFRACTION_K := 0.13
## Koschmieder at the WMO 5 % contrast threshold: ln(1 / 0.05) (WMO-No. 8, Part I ch. 9; RL-b C8).
const KOSCHMIEDER := 2.996
## Crest-to-base luminance falloff that suggests slope shading (RL-b C8).
const BASE_DARKEN := 0.08
## Each ribbon reaches this far below the foreground's lowest point, so no view sees under it.
const BASE_BELOW_M := 40.0
const COLUMNS_PER_DEG := 2
## The near disc sits where the old apron did: under every walking surface.
const GROUND_Y := -0.14
## Foreground: angular and radial segments (sec.5.4).
const FG_SEGMENTS := 128
const FG_RINGS := 16
## Dry grass and chaparral, sRGB (RL-b C8's albedo, inside DEP-8's 0.2-0.7 band).
const SURFACE_TINT := {
	"scrub": Color(0.38, 0.36, 0.27), "dry_grass": Color(0.38, 0.36, 0.27),
	"grass": Color(0.50, 0.62, 0.38), "rough_ground": Color(0.42, 0.38, 0.32),
}
## The near ground's grass: the 900 m apron's tint, which the accepted views were taken with.
const NEAR_TINT := Color(0.50, 0.62, 0.38)
## The angular width of one massing mesh, so frustum culling drops what is behind the camera.
const MASSING_CHUNK_DEG := 45.0

## Set by the rendered check only: the camera the camera-anchored parts follow. Otherwise the
## viewport's active camera.
var anchor_camera: Camera3D = null

var _field: MineWorldFarField
var _datum := 0.0
var _visibility := 80000.0
var _silhouettes: Node3D
var _bands: Array = []           # [MeshInstance3D, ShaderMaterial, band name], in file order
var _sea: Node3D = null
var _sea_y := 0.0
var _horizon: Callable
var _eyes := PackedVector3Array()
var _ids: Array = []             # [MeshInstance3D, id name]


## Builds the far field under `parent`. The panorama sky must already be in the tree.
## `protected_eyes`: scene positions whose sightlines to the far field the ring's roofs keep clear
## (SD-5); the scene that builds the view names them. `near_ground`: draw the flat ground inside the
## boundary; false when the depiction draws its own ground there (a depiction file's terrain, S33-d).
static func build(parent: Node3D, field: MineWorldFarField,
		protected_eyes: PackedVector3Array = PackedVector3Array(), near_ground := true) -> SliceFarFieldView:
	var v := SliceFarFieldView.new()
	v.name = "FarField"
	parent.add_child(v)
	v._field = field
	v._eyes = protected_eyes
	v._datum = field.datum_m()
	v._visibility = float(field.sky()["visibility_m"])
	v._horizon = _horizon_sampler(parent.get_tree().root)
	if near_ground:
		v._near_ground()
	v._build_silhouettes()
	v._build_foreground()
	v._build_water()
	v._build_skyline()
	v.follow()
	# Diagnostic (RL-e0 sec.7 V-1): any capture mode with the far field hidden, so a later PR can
	# tell far-field pixels from the rest of an accepted view.
	if "--slice-farfield-hidden" in OS.get_cmdline_user_args():
		for c in v.get_children():
			if str(c.get_meta("mw_category", "")).begins_with("farfield."):
				(c as Node3D).visible = false
	return v


func _process(_d: float) -> void:
	follow()


## Moves the camera-anchored parts to the camera that is drawing.
func follow() -> void:
	var cam := anchor_camera if anchor_camera != null else get_viewport().get_camera_3d()
	if cam == null:
		return
	var p := cam.global_position
	var rise := p.y - EYE0
	_silhouettes.global_position = Vector3(p.x, 0.0, p.z)
	for band in _bands:
		(band[1] as ShaderMaterial).set_shader_parameter("rise", rise)
	if _sea != null:
		# the sea ribbon at the proxy of the sea horizon's distance from this eye (SD-1)
		var h := maxf(p.y - _sea_y, 0.01)
		var pr := proxy(sqrt(2.0 * EARTH_R * h))
		var crest := p.y - pr * tan(sqrt(2.0 * h / EARTH_R))
		var base := _sea_y - 50.0
		_sea.global_position = Vector3(p.x, base, p.z)
		_sea.scale = Vector3(pr, maxf(crest - base, 0.01), pr)


## The proxy radius of a column at real distance `d` (SD-1).
static func proxy(d: float) -> float:
	var t := clampf(log(maxf(d, 1.0) / D_LO) / log(D_HI / D_LO), 0.0, 1.0)
	return P_MIN + (P_MAX - P_MIN) * t


## Every far-field mesh and the name the rendered check identifies it by: "ground", "foreground",
## "water", "sea-ribbon", "massing", or "band:<name>" (RL-e0b sec.6.2).
func ids() -> Array:
	return _ids.duplicate()


## The angular distance from `bearing` to the nearer end of `sector`, degrees (0 outside it).
static func end_distance(sector: Dictionary, bearing: float) -> float:
	var span := _cw(sector["from_deg"], sector["to_deg"])
	var o := _cw(sector["from_deg"], bearing)
	if o > span:
		return 0.0
	return minf(o, span - o)


## The share of full height a band (or the sea) keeps at `bearing`: 0 at a sector end, 1 from
## `taper` degrees inside it (or a quarter of the sector, whichever is less).
static func end_factor(sector: Dictionary, bearing: float, taper: float) -> float:
	var t := minf(taper, _cw(sector["from_deg"], sector["to_deg"]) * 0.25)
	return smoothstep(0.0, maxf(t, 1e-6), end_distance(sector, bearing))


# --- geometry helpers --------------------------------------------------------------------------

static func _dir(bearing_deg: float) -> Vector3:
	var a := deg_to_rad(bearing_deg)
	return Vector3(sin(a), 0.0, -cos(a))


static func _cw(a: float, b: float) -> float:
	return fposmod(b - a, 360.0)


## Bearings across a sector at `per_deg` columns per degree, both ends included.
static func _steps(sector: Dictionary, per_deg: int) -> PackedFloat64Array:
	var from: float = sector["from_deg"]
	var span := _cw(from, sector["to_deg"])
	var n := maxi(1, int(ceil(span * per_deg)))
	var out := PackedFloat64Array()
	for i in range(n + 1):
		out.append(from + span * float(i) / float(n))
	return out


## The ground's height (scene y) at radius `r` from the origin, outside the boundary.
func _ground_y(r: float) -> float:
	var edge_m: float = _field.boundary()["reach_m"]
	var fg := _field.foreground()
	return GROUND_Y - float(fg["drop_m"]) * smoothstep(edge_m, edge_m +float(fg["run_m"]), r)


## The water body whose sector holds `bearing`, or {}.
func _water_at(bearing: float) -> Dictionary:
	for body in _field.water():
		var s: Dictionary = body["sector"]
		if _cw(s["from_deg"], bearing) < _cw(s["from_deg"], s["to_deg"]):
			return body
	return {}


static func _mesh(st: SurfaceTool, category: String, mat: Material, name: String) -> MeshInstance3D:
	st.generate_normals()
	var mi := MeshInstance3D.new()
	mi.name = name
	mi.mesh = st.commit()
	mi.material_override = mat
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.set_meta("mw_category", category)
	return mi


## One quad of an annulus seen from above, front face up: inner/outer at two bearings.
static func _ring_quad(st: SurfaceTool, a0: Vector3, a1: Vector3, b0: Vector3, b1: Vector3) -> void:
	for v in [a0, b0, b1, a0, b1, a1]:
		st.set_uv(Vector2(v.x, v.z))
		st.add_vertex(v)


# --- near ground -------------------------------------------------------------------------------

## The ground under the street, out to the boundary: what the 900 m apron was for (no frame shows
## the void under the slice), now ending where the foreground begins. Walkable, as the apron was.
func _near_ground() -> void:
	var edge_m: float = _field.boundary()["reach_m"]
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var c := Vector3(0.0, GROUND_Y, 0.0)
	for i in FG_SEGMENTS:
		var b0 := 360.0 * i / FG_SEGMENTS
		var b1 := 360.0 * (i + 1) / FG_SEGMENTS
		# centre, then outward and clockwise: clockwise from above, the front face up
		for v in [c, _dir(b0) * edge_m + c, _dir(b1) * edge_m + c]:
			st.set_uv(Vector2(v.x, v.z))
			st.add_vertex(v)
	# the old apron's grass, unchanged: accepted near views must not change (V-1)
	var mi := _mesh(st, "ground", Mats.pbr("leafy_grass", 2.2, NEAR_TINT, 0.97), "NearGround")
	add_child(mi)
	_ids.append([mi, "ground"])
	var body := StaticBody3D.new()
	body.collision_layer = Build.LAYER_WORLD
	var shape := CollisionShape3D.new()
	var box := BoxShape3D.new()
	box.size = Vector3(2.0 * edge_m, 0.4, 2.0 * edge_m)
	shape.shape = box
	body.add_child(shape)
	body.position = Vector3(0.0, GROUND_Y - 0.2, 0.0)
	add_child(body)


# --- silhouettes -------------------------------------------------------------------------------

## One mesh per band, each with its own material: the per-height parallax is set per material in
## follow(), and the rendered check can colour each band apart (SD-2, sec.6.2).
func _build_silhouettes() -> void:
	_silhouettes = Node3D.new()
	_silhouettes.name = "Silhouettes"
	_silhouettes.set_meta("mw_category", "farfield.silhouette")
	add_child(_silhouettes)
	var shader := load("res://shaders/far_silhouette.gdshader") as Shader
	var base_y := -(float(_field.foreground()["drop_m"]) + BASE_BELOW_M)
	for band in _field.silhouettes():
		var st := SurfaceTool.new()
		st.begin(Mesh.PRIMITIVE_TRIANGLES)
		var bearings := _steps(band["sector"], COLUMNS_PER_DEG)
		for i in range(bearings.size() - 1):
			_ribbon_column(st, band, bearings[i], bearings[i + 1], base_y)
		# the haze below is the fog; scene fog would count it twice (the shader is fog-free)
		var mat := ShaderMaterial.new()
		mat.shader = shader
		mat.set_shader_parameter("eye0", EYE0)
		mat.set_shader_parameter("visibility_m", _visibility)
		mat.set_shader_parameter("texture_amp", TEXTURE_AMP)
		var mi := _mesh(st, "farfield.silhouette", mat, "Band_" + str(band["name"]))
		_silhouettes.add_child(mi)
		_bands.append([mi, mat, band["name"]])
		_ids.append([mi, "band:" + str(band["name"])])


## One column of a ribbon, wound clockwise as seen from inside the ring (Godot's front face). Each
## edge stands at the proxy of its own real distance (SD-1) and carries UV2 = (P/d, d) for the
## shader's parallax and texture (SD-2, SD-4).
func _ribbon_column(st: SurfaceTool, band: Dictionary, az0: float, az1: float, base_y: float) -> void:
	var e0 := _column_edge(band, az0, base_y)
	var e1 := _column_edge(band, az1, base_y)
	var albedo := Color.html(band["albedo"])
	var c0 := _haze(e0[4], _horizon.call(az0), albedo)
	var c1 := _haze(e1[4], _horizon.call(az1), albedo)
	var b0 := c0.darkened(BASE_DARKEN)
	var b1 := c1.darkened(BASE_DARKEN)
	# [position, colour, UV2]
	var top0 := [e0[0], c0, e0[1]]
	var top1 := [e1[0], c1, e1[1]]
	var bot0 := [e0[2], b0, e0[3]]
	var bot1 := [e1[2], b1, e1[3]]
	for v in [bot0, top0, top1, bot0, top1, bot1]:
		st.set_color(v[1])
		st.set_uv2(v[2])
		st.add_vertex(v[0])


## One edge of a band column at `bearing`: [top, top UV2, bottom, bottom UV2, real distance].
##
## The crest is `_crest_eff` (SD-3.1). The bottom reaches `base_y` (below the foreground, never
## seen) in a land sector; in a water sector it stops at the band's foot, so the sea in front of the
## land shows below it, blending to `base_y` where the sea ribbon tapers. Near a band end the
## bottom rises to the foot with the crest, so the end closes in a slope.
func _column_edge(band: Dictionary, bearing: float, base_y: float) -> Array:
	var c := _crest(band, bearing)
	var d := c.y
	var p := proxy(d)
	var k := p / d
	var foot := _foot(bearing, d)
	var s := end_factor(band["sector"], bearing, T_BAND)
	var alpha := foot + (c.x - foot) * s
	var top := _dir(bearing) * p + Vector3(0.0, EYE0 + p * tan(alpha), 0.0)
	var foot_y := EYE0 + p * tan(foot - FOOT_PAD_RAD)
	var body := _water_at(bearing)
	var wet := 0.0 if body.is_empty() else end_factor(body["sector"], bearing, T_WATER)
	var full_y := lerpf(base_y, foot_y, wet)
	var full_k := lerpf(1.0, k, wet)
	var bottom := _dir(bearing) * p + Vector3(0.0, lerpf(foot_y, full_y, s), 0.0)
	return [top, Vector2(k, d), bottom, Vector2(lerpf(k, full_k, s), d), d]


## The apparent angle (radians, from the street's eye) of the surface at real distance `d` along
## `bearing`: the water's level in a water sector, the foreground's far height elsewhere; with
## Earth's curvature and refraction. A band's crest falls to it at the band's ends (SD-3.1).
func _foot(bearing: float, d: float) -> float:
	var body := _water_at(bearing)
	var y := _ground_y(OUTER_M) if body.is_empty() else float(body["level_m"]) - _datum
	var drop := d * d * (1.0 - REFRACTION_K) / (2.0 * EARTH_R)
	return (y - EYE0 - drop) / d


## The crest's apparent angle with the end taper of SD-3.1 (radians, from the street's eye).
func _crest_eff(band: Dictionary, bearing: float) -> float:
	var c := _crest(band, bearing)
	var foot := _foot(bearing, c.y)
	return foot + (c.x - foot) * end_factor(band["sector"], bearing, T_BAND)


## The crest at `bearing`: x = its apparent angle above the street's eye (radians), with Earth's
## curvature and refraction; y = its distance (m). Height and distance are interpolated between
## the profile's points with a smoothstep: no relief is invented between them.
func _crest(band: Dictionary, bearing: float) -> Vector2:
	var from: float = band["sector"]["from_deg"]
	var o := _cw(from, bearing)
	var pts: Array = band["profile"]
	var h: float = pts[pts.size() - 1]["height_m"]
	var d: float = pts[pts.size() - 1]["distance_m"]
	for i in range(pts.size() - 1):
		var op := _cw(from, pts[i]["bearing_deg"])
		var oq := _cw(from, pts[i + 1]["bearing_deg"])
		if i == 0:
			op = 0.0
		if o >= op and o <= oq:
			var e := smoothstep(0.0, 1.0, (o - op) / maxf(oq - op, 1e-9))
			h = lerpf(pts[i]["height_m"], pts[i + 1]["height_m"], e)
			d = lerpf(pts[i]["distance_m"], pts[i + 1]["distance_m"], e)
			break
	var drop := d * d * (1.0 - REFRACTION_K) / (2.0 * EARTH_R)
	return Vector2((h - _datum - EYE0 - drop) / d, d)


## Koschmieder: what is left of the land's own radiance at distance `d`, the rest the horizon sky's.
func _haze(d: float, horizon: Color, albedo_srgb: Color) -> Color:
	var a := albedo_srgb.srgb_to_linear()
	var light := SliceMain.SUN_ENERGY * sin(deg_to_rad(-SliceMain.SUN_ELEVATION)) \
		+ SliceMain.SKY_AMBIENT * 0.8
	var land := Color(a.r * light, a.g * light, a.b * light)
	return horizon.lerp(land, exp(-KOSCHMIEDER * d / _visibility))


## The panorama's colour at the horizon for a bearing (the mean of the rows within one degree of
## it), times the sky's energy, so a band meets the sky without a seam (RL-b C8). Sampled at whole
## degrees and interpolated between them, so the colour has no 1-degree step (RL-e0b SD-3). Falls
## back to the fog colour without a panorama.
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
	var rows := maxi(1, int(round(h / 180.0)))
	var cache := {}
	var at := func(key: int) -> Color:
		if cache.has(key):
			return cache[key]
		# Godot's panorama mapping: u = atan2(x, z) / 2pi (wrapped), v = acos(y) / pi
		var a := deg_to_rad(float(key))
		var u := fposmod(atan2(sin(a), -cos(a)) / TAU, 1.0)
		var x := clampi(int(u * w), 0, w - 1)
		var sum := Color(0, 0, 0)
		var n := 0
		for dy in range(-rows, rows + 1):
			sum += img.get_pixel(x, clampi(h / 2 + dy, 0, h - 1))
			n += 1
		var c := Color(sum.r / n * energy, sum.g / n * energy, sum.b / n * energy)
		cache[key] = c
		return c
	return func(az: float) -> Color:
		var lo := floori(az)
		return (at.call(lo) as Color).lerp(at.call(lo + 1), az - float(lo))


## The sky the far water reflects at `bearing`: the panorama's horizon, but never brighter than the
## depth fog that the sky is drawn through, so the water does not outshine the sky it meets.
func _reflected_sky(bearing: float, fog: Color) -> Color:
	var c: Color = _horizon.call(bearing)
	return Color(minf(c.r, fog.r), minf(c.g, fog.g), minf(c.b, fog.b))


# --- foreground --------------------------------------------------------------------------------

## The ground from the boundary to OUTER_M (or to the shore, in a water sector), falling to the
## datum's drop. Angular edges include every water sector's edges, and every half degree where the
## shore curves out (SD-3.2), so ground and water meet exactly.
func _build_foreground() -> void:
	var edge_m: float = _field.boundary()["reach_m"]
	var run: float = _field.foreground()["run_m"]
	var surface := _surface_tint(SURFACE_TINT[_field.foreground()["surface"]])
	# The ground starts as the near grass at the boundary and is the declared surface once it has
	# fallen, so it does not change colour in a line. Vertex colours are stored unsigned-normalised
	# (at most 1), so the material's tint is the larger of the two per channel and the vertex colour
	# scales down from it.
	var base := Color(maxf(NEAR_TINT.r, surface.r), maxf(NEAR_TINT.g, surface.g),
		maxf(NEAR_TINT.b, surface.b))
	var near := Color(NEAR_TINT.r / base.r, NEAR_TINT.g / base.g, NEAR_TINT.b / base.b)
	var far := Color(surface.r / base.r, surface.g / base.g, surface.b / base.b)
	var edges := PackedFloat64Array()
	for i in FG_SEGMENTS:
		edges.append(360.0 * i / FG_SEGMENTS)
	for body in _field.water():
		var from := float(body["sector"]["from_deg"])
		var span := _cw(from, body["sector"]["to_deg"])
		var taper := minf(T_WATER, span * 0.25)
		var n := int(ceil(taper * COLUMNS_PER_DEG))
		for k in range(n + 1):
			var o := taper * float(k) / float(maxi(n, 1))
			edges.append(fposmod(from + o, 360.0))
			edges.append(fposmod(from + span - o, 360.0))
	edges.sort()
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for i in edges.size():
		var b0 := edges[i]
		var b1 := edges[(i + 1) % edges.size()] + (360.0 if i == edges.size() - 1 else 0.0)
		if b1 - b0 < 1e-6:
			continue
		var body := _water_at(fposmod((b0 + b1) * 0.5, 360.0))
		var outer0: float = OUTER_M if body.is_empty() else _shore(body, b0)
		var outer1: float = OUTER_M if body.is_empty() else _shore(body, b1)
		var y_shore := 0.0 if body.is_empty() else float(body["level_m"]) - _datum
		for k in FG_RINGS:
			var f0 := float(k) / FG_RINGS
			var f1 := float(k + 1) / FG_RINGS
			var ra0 := edge_m * pow(outer0 / edge_m, f0)
			var ra1 := edge_m * pow(outer1 / edge_m, f0)
			var rc0 := edge_m * pow(outer0 / edge_m, f1)
			var rc1 := edge_m * pow(outer1 / edge_m, f1)
			var a0 := _dir(b0) * ra0 + Vector3(0, _fg_y(ra0, outer0, body, y_shore), 0)
			var a1 := _dir(b1) * ra1 + Vector3(0, _fg_y(ra1, outer1, body, y_shore), 0)
			var c0 := _dir(b0) * rc0 + Vector3(0, _fg_y(rc0, outer0, body, y_shore), 0)
			var c1 := _dir(b1) * rc1 + Vector3(0, _fg_y(rc1, outer1, body, y_shore), 0)
			# same order as _ring_quad: clockwise from above, front face up
			for v in [a0, c0, c1, a0, c1, a1]:
				st.set_color(near.lerp(far, smoothstep(edge_m, edge_m +run, Vector2(v.x, v.z).length())))
				st.set_uv(Vector2(v.x, v.z))
				st.add_vertex(v)
	var mat := (Mats.pbr("leafy_grass", 2.2, base, 0.97) as BaseMaterial3D).duplicate() \
		as BaseMaterial3D
	mat.vertex_color_use_as_albedo = true
	var mi := _mesh(st, "farfield.foreground", mat, "Foreground")
	add_child(mi)
	_ids.append([mi, "foreground"])


## The tint that makes `leafy_grass` average to `albedo` (sRGB): the declared surface colour is an
## albedo, and the texture is green, so the tint is the albedo over the texture's mean.
static func _surface_tint(albedo: Color) -> Color:
	var tex := Mats.tex("leafy_grass", "diff")
	if tex == null:
		return albedo
	var img := tex.get_image()
	if img == null:
		return albedo
	if img.is_compressed():
		img.decompress()
	img = img.duplicate()
	img.resize(1, 1, Image.INTERPOLATE_BILINEAR)
	var mean := img.get_pixel(0, 0)
	return Color(albedo.r / maxf(mean.r, 0.05), albedo.g / maxf(mean.g, 0.05),
		albedo.b / maxf(mean.b, 0.05))


## In a water sector the ground meets the water's level at the shore.
func _fg_y(r: float, outer: float, body: Dictionary, y_shore: float) -> float:
	var y := _ground_y(r)
	if body.is_empty():
		return y
	return lerpf(y, y_shore, smoothstep(outer - 60.0, outer, r))


## The water's near edge at `bearing` (SD-3.2): `shore_m` inside the sector, curving out to
## OUTER_M over T_WATER at each end, so no radial edge remains.
func _shore(body: Dictionary, bearing: float) -> float:
	var s := end_factor(body["sector"], bearing, T_WATER)
	return float(body["shore_m"]) + (OUTER_M - float(body["shore_m"])) * (1.0 - s)


# --- water -------------------------------------------------------------------------------------

## The water's surface from the shore to OUTER_M, and the sea ribbon beyond it. A visual plane only
## (SD-FF-5): no reflection of the town, no flow, Fresnel capped at 0.6. The ribbon is built at unit
## radius and unit height, and follow() scales it to the proxy of the sea horizon's distance and to
## the true dip; over T_WATER at each end its top descends to its base (SD-3.3).
func _build_water() -> void:
	var bodies := _field.water()
	if bodies.is_empty():
		return
	var shader := Shader.new()
	shader.code = "#define FAR_WATER\n" + (load("res://shaders/water.gdshader") as Shader).code
	var ribbon_st := SurfaceTool.new()
	ribbon_st.begin(Mesh.PRIMITIVE_TRIANGLES)
	# The ribbon stands beyond the depth fog's end. `StandardMaterial3D.disable_fog` left it drawn in
	# the fog's colour -- the sky's own -- so it is drawn by a fog-free shader, as the bands are.
	var ribbon_mat := ShaderMaterial.new()
	ribbon_mat.shader = Shader.new()
	ribbon_mat.shader.code = "shader_type spatial;\nrender_mode unshaded, fog_disabled, cull_back;\n" \
		+ "void fragment() { ALBEDO = COLOR.rgb; }\n"
	# the depth fog's colour, which dims the drawn sky near the horizon (none without fog)
	var fog := Color(INF, INF, INF)
	var envs := get_tree().root.find_children("*", "WorldEnvironment", true, false)
	if not envs.is_empty() and (envs[0] as WorldEnvironment).environment.fog_enabled:
		var env := (envs[0] as WorldEnvironment).environment
		fog = env.fog_light_color.srgb_to_linear() * env.fog_light_energy
	for body in bodies:
		var y := float(body["level_m"]) - _datum
		_sea_y = y
		var deep := Color.html(body["colour"]["deep"])
		var shallow := Color.html(body["colour"]["shallow"])
		var bearings := _steps(body["sector"], COLUMNS_PER_DEG)
		var sky := Color(0, 0, 0)
		for b in bearings:
			sky += _horizon.call(b)
		sky /= float(bearings.size())
		var mat := ShaderMaterial.new()
		mat.shader = shader
		mat.set_shader_parameter("deep", deep)
		mat.set_shader_parameter("shallow", shallow)
		mat.set_shader_parameter("sky_horizon", Vector3(sky.r, sky.g, sky.b))
		mat.set_shader_parameter("fresnel_cap", 0.6)
		var st := SurfaceTool.new()
		st.begin(Mesh.PRIMITIVE_TRIANGLES)
		var deep_lin := deep.srgb_to_linear()
		for i in range(bearings.size() - 1):
			var d0 := _dir(bearings[i])
			var d1 := _dir(bearings[i + 1])
			var s0 := _shore(body, bearings[i])
			var s1 := _shore(body, bearings[i + 1])
			_ring_quad(st, d0 * s0 + Vector3(0, y, 0), d1 * s1 + Vector3(0, y, 0),
				d0 * OUTER_M + Vector3(0, y, 0), d1 * OUTER_M + Vector3(0, y, 0))
			var t0 := Vector3.UP * end_factor(body["sector"], bearings[i], T_WATER)
			var t1 := Vector3.UP * end_factor(body["sector"], bearings[i + 1], T_WATER)
			# the far water reflects the sky at its own bearing, not a mean over the whole sector,
			# with a Fresnel share of RIBBON_FRESNEL (inside Q-FF-2's cap of 0.6)
			var h0: Color = _reflected_sky(bearings[i], fog).lerp(deep_lin, 1.0 - RIBBON_FRESNEL)
			var h1: Color = _reflected_sky(bearings[i + 1], fog).lerp(deep_lin, 1.0 - RIBBON_FRESNEL)
			for v in [[d0, h0], [d0 + t0, h0], [d1 + t1, h1], [d0, h0], [d1 + t1, h1], [d1, h1]]:
				ribbon_st.set_color(v[1])
				ribbon_st.add_vertex(v[0])
		var water := _mesh(st, "farfield.water", mat, "Water_" + str(body["name"]))
		add_child(water)
		_ids.append([water, "water"])
	_sea = Node3D.new()
	_sea.name = "SeaRibbon"
	_sea.set_meta("mw_category", "farfield.water")
	var ribbon := _mesh(ribbon_st, "farfield.water", ribbon_mat, "SeaRibbonMesh")
	_sea.add_child(ribbon)
	add_child(_sea)
	_ids.append([ribbon, "sea-ribbon"])


# --- skyline -----------------------------------------------------------------------------------

## Low-rise massing, deterministic from the seed: boxes placed by rejection sampling until each
## sector's declared density of plan area is covered, none overlapping another or a reserved slot,
## each facing the street, standing on the ground at its radius, its roof capped below the protected
## eyes' sightlines (SD-5, `_cap`). One mesh per 45 degrees. The random draws happen in the same
## order whatever the cap decides, so the layout is deterministic from the seed (I-7).
func _build_skyline() -> void:
	var skyline := _field.skyline()
	var rng := RandomNumberGenerator.new()
	rng.seed = int(skyline["seed"])
	var clear := []          # [centre, radius] kept free of massing
	for name in ["station", "landmark"]:
		var slot: Dictionary = skyline["slots"][name]
		if slot["state"] == "reserved" or slot["state"] == "filled":
			clear.append([_dir(slot["bearing_deg"]) * float(slot["distance_m"]),
				float(slot["footprint_m"]) * 0.5])
	var placed := {}         # grid cell -> Array of [centre, radius]
	var chunks := {}         # chunk index -> SurfaceTool
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.roughness = 0.9
	var boxes := 0
	var low := 0
	for sector in skyline["sectors"]:
		var from: float = sector["sector"]["from_deg"]
		var span := _cw(from, sector["sector"]["to_deg"])
		var r0: float = sector["from_m"]
		var r1: float = sector["to_m"]
		var target := deg_to_rad(span) * 0.5 * (r1 * r1 - r0 * r0) * float(sector["density"])
		var covered := 0.0
		var tries := 0
		var limit := int(target / (SliceFarFieldMassing.BOX_W.x * SliceFarFieldMassing.BOX_D.x)) * 8 + 100
		while covered < target and tries < limit:
			tries += 1
			var bearing := fposmod(from + span * rng.randf(), 360.0)
			var r := sqrt(lerpf(r0 * r0, r1 * r1, rng.randf()))
			var w := rng.randf_range(SliceFarFieldMassing.BOX_W.x, SliceFarFieldMassing.BOX_W.y)
			var dpt := rng.randf_range(SliceFarFieldMassing.BOX_D.x, SliceFarFieldMassing.BOX_D.y)
			var h := rng.randf_range(sector["height_m"]["min"], sector["height_m"]["max"])
			var capped := rng.randf() < 0.5
			var walls := SliceFarFieldMassing.WALLS
			var wall: Color = walls[rng.randi_range(0, walls.size() - 1)]
			var centre := _dir(bearing) * r
			var radius := 0.5 * Vector2(w, dpt).length()
			if not SliceFarFieldMassing.free_at(centre, radius, placed, clear):
				continue
			var cap := _cap(centre, radius, _ground_y(r))
			if cap < CAP_FLOOR_M:
				continue
			if cap < CAP_MIN_M:
				h = maxf(cap, CAP_FLOOR_M)
				capped = false
				low += 1
			elif h + (SliceFarFieldMassing.CAP_RISE_M if capped else 0.0) > cap:
				if capped and cap - SliceFarFieldMassing.CAP_RISE_M >= CAP_MIN_M:
					h = cap - SliceFarFieldMassing.CAP_RISE_M
				else:
					capped = false
					h = cap
			var cell := Vector2i(floori(centre.x / 40.0), floori(centre.z / 40.0))
			if not placed.has(cell):
				placed[cell] = []
			placed[cell].append([centre, radius])
			covered += w * dpt
			var chunk := int(bearing / MASSING_CHUNK_DEG)
			if not chunks.has(chunk):
				var st := SurfaceTool.new()
				st.begin(Mesh.PRIMITIVE_TRIANGLES)
				st.set_smooth_group(-1)   # flat faces: a building is not a pebble
				chunks[chunk] = st
			SliceFarFieldMassing.box(chunks[chunk], centre + Vector3(0, _ground_y(r), 0), bearing, w, dpt,
				h, capped, wall)
			boxes += 1
	for chunk in chunks:
		var mi := _mesh(chunks[chunk], "farfield.skyline", mat, "Massing_%d" % chunk)
		add_child(mi)
		_ids.append([mi, "massing"])
	print("farfield %d massing boxes (%d walls or sheds under the sightline cap), %d meshes"
		% [boxes, low, chunks.size()])


## The tallest a box at `centre` (footprint radius `radius`, ground `ground_y`) may stand, so that
## from every protected eye its roof stays CAP_MARGIN_DEG below what it stands in front of: the
## highest band crest at its bearing from that eye (`_crest_eff`, with the eye's parallax), or in a
## water sector the water's far side (`_sightline`). The whole roof stands in front of it, so the limit
## is the lowest across every bearing the box covers from that eye, not only its centre's. INF
## without protected eyes.
func _cap(centre: Vector3, radius: float, ground_y: float) -> float:
	var cap := INF
	for eye in _eyes:
		var to := Vector2(centre.x - eye.x, centre.z - eye.z)
		var dist := maxf(to.length() - radius, 1.0)
		var bearing := fposmod(rad_to_deg(atan2(to.x, -to.y)), 360.0)
		var half := rad_to_deg(atan2(radius, maxf(to.length(), 1.0)))
		var samples := 1 + 2 * maxi(1, int(ceil(half)))
		var limit := INF
		for k in samples:
			var b := bearing + half * (2.0 * float(k) / float(samples - 1) - 1.0)
			limit = minf(limit, _sightline(eye, fposmod(b, 360.0)))
		if is_inf(limit):
			continue
		var top := eye.y + dist * tan(limit - deg_to_rad(CAP_MARGIN_DEG))
		cap = minf(cap, top - ground_y)
	return cap


## The lowest angle (radians) an eye must see clear along `bearing`: in a water sector the water's
## far side, so a strip of water stays in view over the roofs; otherwise the highest band crest
## there. INF when neither exists.
##
## SD-5.2 as frozen named the water's NEAR edge. From the street's ends (eye 1.8 m, the shore 1 km
## away) that kept every roof within ~140 m of the street in the bay's bearings under 1 m, so no box
## was built there and FF-RING failed in every bay bin (R-3's material stop). The far side keeps
## the bay in view (FF-BAY) and lets the ring be built (FF-RING): recorded in the design's ledger.
func _sightline(eye: Vector3, bearing: float) -> float:
	var body := _water_at(bearing)
	if not body.is_empty():
		# The water's far side: where it meets the nearest land in front of the sea horizon (that
		# band's foot), or the sea horizon itself. A roof kept below it leaves a strip of water in
		# view.
		var y := float(body["level_m"]) - _datum
		# where the ray from the eye meets the water's near edge; toward the sector's ends the far
		# water narrows to nothing (SD-3.3), and the reference falls back to this near edge
		var o := Vector2(eye.x, eye.z)
		var dv := Vector2(sin(deg_to_rad(bearing)), -cos(deg_to_rad(bearing)))
		var s := _shore(body, bearing)
		var b := o.dot(dv)
		var t := -b + sqrt(maxf(b * b - (o.length_squared() - s * s), 0.0))
		var near := atan((y - eye.y) / maxf(t, 1.0))
		var h := maxf(eye.y - y, 0.01)
		var far := -sqrt(2.0 * h / EARTH_R)
		var dh := sqrt(2.0 * EARTH_R * h)
		for band in _field.silhouettes():
			if not MineWorldFarField.sector_contains(band["sector"], bearing):
				continue
			var d := _crest(band, bearing).y
			if d < dh:
				far = minf(far, (y - eye.y - d * d * (1.0 - REFRACTION_K) / (2.0 * EARTH_R)) / d)
		return near + (maxf(far, near) - near) * end_factor(body["sector"], bearing, T_WATER)
	var best := -INF
	var rise := eye.y - EYE0
	for band in _field.silhouettes():
		if not MineWorldFarField.sector_contains(band["sector"], bearing):
			continue
		var d := _crest(band, bearing).y
		best = maxf(best, _crest_eff(band, bearing) - rise / d)
	return INF if best == -INF else best
