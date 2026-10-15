## The far field: every direction from the street closes on sky, terrain, water and the town's
## distant massing (`docs/MODULE_SPEC.md` sec.6.1.1, `docs/DECISIONS.md` ARC-102; RL-e0 sec.5.4).
##
## WHAT IT REPLACES. Three ridges north of the street (`street.gd` backdrop, removed) that covered
## about +-44 degrees of azimuth and were wound so Godot culled the faces the player saw, over a
## 900 m grass apron whose edge showed against the sky to the east, south and west (E-FF-3 .. 5).
##
## WHAT IT DRAWS, all from the Presentation Pack's `far_field/social-cafe.yaml`:
##
##   silhouette   terrain bands at their real bearings and angles, as ribbons that follow the
##                camera (no parallax at walking scale, as 2-72 km of distance gives none); their
##                height follows the camera's height by the share each layer's real distance gives
##   foreground   the ground from the boundary outward, falling `drop_m` over `run_m`, world-anchored
##   water        the bay's surface, world-anchored, and a camera-anchored ribbon of sea out to the
##                true dip of the horizon
##   skyline      low-rise massing, deterministic from the file's seed, world-anchored, the reserved
##                station slot kept clear
##
## Plus the near ground under the street: a disc of the boundary's radius (category "ground").
##
## WINDING (SD-FF-4). Triangles are emitted in Godot's front-face order -- clockwise as seen by the
## camera that is meant to see them -- and `SurfaceTool.generate_normals()` derives the normals. The
## materials keep back-face culling. Nothing here asserts a winding: the rendered check
## (`slice_far_field_check.gd`) proves the faces are drawn, and reversing them fails it (X-1).
##
## FRAME: +x east, -z north (`street.gd`); bearing b is the direction (sin b, 0, -cos b).
class_name SliceFarFieldView
extends Node3D

## The street's eye at standing height above the pavement (SliceWorld.SPAWN at eye height).
const EYE0 := SliceStreet.WALK_Y + CameraRig.EYE_HEIGHT
## Proxy distances of the camera-anchored layers. They lie beyond the world-anchored foreground and
## water (OUTER_M) and inside the camera's far plane (`CameraRig.FAR` 1 600 m), so everything
## world-anchored is drawn in front of them (D-FF-7: the design's 900/1 100/1 300 m hid the bay).
const PROXY_M := {"near": 1480.0, "mid": 1520.0, "far": 1560.0}
## How far the world-anchored foreground and water reach.
const OUTER_M := 1450.0
## The camera-anchored sea ribbon's distance.
const SEA_RIBBON_M := 1400.0
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
## Massing: footprints and the share of boxes with a pitched cap (sec.5.4).
const BOX_W := Vector2(8.0, 20.0)
const BOX_D := Vector2(8.0, 16.0)
const CAP_RISE_M := 2.2
## The angular width of one massing mesh, so frustum culling drops what is behind the camera.
const MASSING_CHUNK_DEG := 45.0
const WALLS := [Color(0.84, 0.78, 0.67), Color(0.76, 0.66, 0.52), Color(0.56, 0.36, 0.28)]
const ROOF := Color(0.47, 0.30, 0.24)

## Set by the rendered check only: the camera the camera-anchored parts follow. Otherwise the
## viewport's active camera.
var anchor_camera: Camera3D = null

var _field: MineWorldFarField
var _datum := 0.0
var _visibility := 80000.0
var _silhouettes: Node3D
var _layers: Array = []          # [MeshInstance3D, proxy m, median distance m]
var _sea: Node3D = null
var _sea_y := 0.0
var _horizon: Callable


## Builds the far field under `parent`. The panorama sky must already be in the tree.
static func build(parent: Node3D, field: MineWorldFarField) -> SliceFarFieldView:
	var v := SliceFarFieldView.new()
	v.name = "FarField"
	parent.add_child(v)
	v._field = field
	v._datum = field.datum_m()
	v._visibility = float(field.sky()["visibility_m"])
	v._horizon = _horizon_sampler(parent.get_tree().root)
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
	_silhouettes.global_position = Vector3(p.x, rise, p.z)
	for layer in _layers:
		(layer[0] as Node3D).position.y = -rise * float(layer[1]) / float(layer[2])
	if _sea != null:
		var h := maxf(p.y - _sea_y, 0.01)
		var crest := p.y - SEA_RIBBON_M * tan(sqrt(2.0 * h / EARTH_R))
		var base := _sea_y - 50.0
		_sea.global_position = Vector3(p.x, base, p.z)
		_sea.scale = Vector3(1.0, maxf(crest - base, 0.01), 1.0)


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
	var reach: float = _field.boundary()["reach_m"]
	var fg := _field.foreground()
	return GROUND_Y - float(fg["drop_m"]) * smoothstep(reach, reach + float(fg["run_m"]), r)


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
	var reach: float = _field.boundary()["reach_m"]
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var c := Vector3(0.0, GROUND_Y, 0.0)
	for i in FG_SEGMENTS:
		var b0 := 360.0 * i / FG_SEGMENTS
		var b1 := 360.0 * (i + 1) / FG_SEGMENTS
		# centre, then outward and clockwise: clockwise from above, the front face up
		for v in [c, _dir(b0) * reach + c, _dir(b1) * reach + c]:
			st.set_uv(Vector2(v.x, v.z))
			st.add_vertex(v)
	# the old apron's grass, unchanged: accepted near views must not change (V-1)
	var mi := _mesh(st, "ground", Mats.pbr("leafy_grass", 2.2, NEAR_TINT, 0.97), "NearGround")
	add_child(mi)
	var body := StaticBody3D.new()
	body.collision_layer = Build.LAYER_WORLD
	var shape := CollisionShape3D.new()
	var box := BoxShape3D.new()
	box.size = Vector3(2.0 * reach, 0.4, 2.0 * reach)
	shape.shape = box
	body.add_child(shape)
	body.position = Vector3(0.0, GROUND_Y - 0.2, 0.0)
	add_child(body)


# --- silhouettes -------------------------------------------------------------------------------

func _build_silhouettes() -> void:
	_silhouettes = Node3D.new()
	_silhouettes.name = "Silhouettes"
	_silhouettes.set_meta("mw_category", "farfield.silhouette")
	add_child(_silhouettes)
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.vertex_color_use_as_albedo = true
	mat.disable_fog = true   # the haze below is the fog; scene fog would count it twice
	var base_y := -(float(_field.foreground()["drop_m"]) + BASE_BELOW_M)
	for layer in ["far", "mid", "near"]:
		var bands := _field.silhouettes().filter(func(b: Dictionary) -> bool: return b["layer"] == layer)
		if bands.is_empty():
			continue
		var dp: float = PROXY_M[layer]
		var distances: Array[float] = []
		var st := SurfaceTool.new()
		st.begin(Mesh.PRIMITIVE_TRIANGLES)
		for band in bands:
			for p in band["profile"]:
				distances.append(float(p["distance_m"]))
			var bearings := _steps(band["sector"], COLUMNS_PER_DEG)
			for i in range(bearings.size() - 1):
				_ribbon_column(st, band, bearings[i], bearings[i + 1], dp, base_y)
		distances.sort()
		var mi := _mesh(st, "farfield.silhouette", mat, "Band_" + layer)
		_silhouettes.add_child(mi)
		_layers.append([mi, dp, distances[distances.size() / 2]])


## One column of a ribbon, wound clockwise as seen from inside the ring (Godot's front face).
func _ribbon_column(st: SurfaceTool, band: Dictionary, az0: float, az1: float, dp: float,
		base_y: float) -> void:
	var p0 := _crest(band, az0)
	var p1 := _crest(band, az1)
	var albedo := Color.html(band["albedo"])
	var c0 := _haze(p0.y, _horizon.call(az0), albedo)
	var c1 := _haze(p1.y, _horizon.call(az1), albedo)
	var top0 := _dir(az0) * dp + Vector3(0.0, EYE0 + dp * tan(p0.x), 0.0)
	var top1 := _dir(az1) * dp + Vector3(0.0, EYE0 + dp * tan(p1.x), 0.0)
	var bot0 := _dir(az0) * dp + Vector3(0.0, base_y, 0.0)
	var bot1 := _dir(az1) * dp + Vector3(0.0, base_y, 0.0)
	var b0 := c0.darkened(BASE_DARKEN)
	var b1 := c1.darkened(BASE_DARKEN)
	for v in [[bot0, b0], [top0, c0], [top1, c1], [bot0, b0], [top1, c1], [bot1, b1]]:
		st.set_color(v[1])
		st.add_vertex(v[0])


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
## it), times the sky's energy, so a band meets the sky without a seam (RL-b C8). Falls back to the
## fog colour without a panorama.
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
			sum += img.get_pixel(x, clampi(h / 2 + dy, 0, h - 1))
			n += 1
		var c := Color(sum.r / n * energy, sum.g / n * energy, sum.b / n * energy)
		cache[key] = c
		return c


# --- foreground --------------------------------------------------------------------------------

## The ground from the boundary to OUTER_M (or to the shore, in a water sector), falling to the
## datum's drop. Angular edges include every water sector's edges, so ground and water meet exactly.
func _build_foreground() -> void:
	var reach: float = _field.boundary()["reach_m"]
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
		edges.append(float(body["sector"]["from_deg"]))
		edges.append(float(body["sector"]["to_deg"]))
	edges.sort()
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for i in edges.size():
		var b0 := edges[i]
		var b1 := edges[(i + 1) % edges.size()] + (360.0 if i == edges.size() - 1 else 0.0)
		if b1 - b0 < 1e-6:
			continue
		var body := _water_at(fposmod((b0 + b1) * 0.5, 360.0))
		var outer: float = OUTER_M if body.is_empty() else float(body["shore_m"])
		var y_shore := 0.0 if body.is_empty() else float(body["level_m"]) - _datum
		var radii := PackedFloat64Array()
		for k in range(FG_RINGS + 1):
			radii.append(reach * pow(outer / reach, float(k) / FG_RINGS))
		for k in FG_RINGS:
			var y0 := _fg_y(radii[k], outer, body, y_shore)
			var y1 := _fg_y(radii[k + 1], outer, body, y_shore)
			var a0 := _dir(b0) * radii[k] + Vector3(0, y0, 0)
			var a1 := _dir(b1) * radii[k] + Vector3(0, y0, 0)
			var c0 := _dir(b0) * radii[k + 1] + Vector3(0, y1, 0)
			var c1 := _dir(b1) * radii[k + 1] + Vector3(0, y1, 0)
			# same order as _ring_quad: clockwise from above, front face up
			for v in [a0, c0, c1, a0, c1, a1]:
				st.set_color(near.lerp(far, smoothstep(reach, reach + run, Vector2(v.x, v.z).length())))
				st.set_uv(Vector2(v.x, v.z))
				st.add_vertex(v)
	var mat := (Mats.pbr("leafy_grass", 2.2, base, 0.97) as BaseMaterial3D).duplicate() \
		as BaseMaterial3D
	mat.vertex_color_use_as_albedo = true
	add_child(_mesh(st, "farfield.foreground", mat, "Foreground"))


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


# --- water -------------------------------------------------------------------------------------

## The water's surface from the shore to OUTER_M, and the sea ribbon beyond it. A visual plane only
## (SD-FF-5): no reflection of the town, no flow, Fresnel capped at 0.6.
func _build_water() -> void:
	var bodies := _field.water()
	if bodies.is_empty():
		return
	var shader := Shader.new()
	shader.code = "#define FAR_WATER\n" + (load("res://shaders/water.gdshader") as Shader).code
	var ribbon_st := SurfaceTool.new()
	ribbon_st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var ribbon_mat := StandardMaterial3D.new()
	ribbon_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	ribbon_mat.vertex_color_use_as_albedo = true
	ribbon_mat.disable_fog = true
	for body in bodies:
		var y := float(body["level_m"]) - _datum
		_sea_y = y
		var deep := Color.html(body["colour"]["deep"])
		var shallow := Color.html(body["colour"]["shallow"])
		var bearings := _steps(body["sector"], 1)
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
		var shore: float = body["shore_m"]
		for i in range(bearings.size() - 1):
			var d0 := _dir(bearings[i])
			var d1 := _dir(bearings[i + 1])
			_ring_quad(st, d0 * shore + Vector3(0, y, 0), d1 * shore + Vector3(0, y, 0),
				d0 * OUTER_M + Vector3(0, y, 0), d1 * OUTER_M + Vector3(0, y, 0))
			# the sea ribbon: unit height, stretched each frame to the true dip (follow())
			var horizon := sky.lerp(deep.srgb_to_linear(), 0.4)
			var r0 := d0 * SEA_RIBBON_M
			var r1 := d1 * SEA_RIBBON_M
			for v in [r0, r0 + Vector3.UP, r1 + Vector3.UP, r0, r1 + Vector3.UP, r1]:
				ribbon_st.set_color(horizon)
				ribbon_st.add_vertex(v)
		add_child(_mesh(st, "farfield.water", mat, "Water_" + str(body["name"])))
	_sea = Node3D.new()
	_sea.name = "SeaRibbon"
	_sea.set_meta("mw_category", "farfield.water")
	_sea.add_child(_mesh(ribbon_st, "farfield.water", ribbon_mat, "SeaRibbonMesh"))
	add_child(_sea)


# --- skyline -----------------------------------------------------------------------------------

## Low-rise massing, deterministic from the seed: boxes placed by rejection sampling until each
## sector's declared density of plan area is covered, none overlapping another or a reserved slot,
## each facing the street, standing on the ground at its radius. One mesh per 45 degrees.
func _build_skyline() -> void:
	var skyline := _field.skyline()
	var rng := RandomNumberGenerator.new()
	rng.seed = int(skyline["seed"])
	var clear := []          # [centre, radius] kept free of massing
	for name in ["station", "landmark"]:
		var slot: Dictionary = skyline["slots"][name]
		if slot["state"] == "reserved":
			clear.append([_dir(slot["bearing_deg"]) * float(slot["distance_m"]),
				float(slot["footprint_m"]) * 0.5])
	var placed := {}         # grid cell -> Array of [centre, radius]
	var chunks := {}         # chunk index -> SurfaceTool
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.roughness = 0.9
	var boxes := 0
	for sector in skyline["sectors"]:
		var from: float = sector["sector"]["from_deg"]
		var span := _cw(from, sector["sector"]["to_deg"])
		var r0: float = sector["from_m"]
		var r1: float = sector["to_m"]
		var target := deg_to_rad(span) * 0.5 * (r1 * r1 - r0 * r0) * float(sector["density"])
		var covered := 0.0
		var tries := 0
		var limit := int(target / (BOX_W.x * BOX_D.x)) * 8 + 100
		while covered < target and tries < limit:
			tries += 1
			var bearing := fposmod(from + span * rng.randf(), 360.0)
			var r := sqrt(lerpf(r0 * r0, r1 * r1, rng.randf()))
			var w := rng.randf_range(BOX_W.x, BOX_W.y)
			var dpt := rng.randf_range(BOX_D.x, BOX_D.y)
			var h := rng.randf_range(sector["height_m"]["min"], sector["height_m"]["max"])
			var capped := rng.randf() < 0.5
			var wall: Color = WALLS[rng.randi_range(0, WALLS.size() - 1)]
			var centre := _dir(bearing) * r
			var radius := 0.5 * Vector2(w, dpt).length()
			if not _free(centre, radius, placed, clear):
				continue
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
			_box(chunks[chunk], centre + Vector3(0, _ground_y(r), 0), bearing, w, dpt, h, capped, wall)
			boxes += 1
	for chunk in chunks:
		add_child(_mesh(chunks[chunk], "farfield.skyline", mat, "Massing_%d" % chunk))
	print("farfield %d massing boxes, %d meshes" % [boxes, chunks.size()])


static func _free(centre: Vector3, radius: float, placed: Dictionary, clear: Array) -> bool:
	for c in clear:
		if centre.distance_to(c[0]) < radius + float(c[1]):
			return false
	var cell := Vector2i(floori(centre.x / 40.0), floori(centre.z / 40.0))
	for dx in range(-1, 2):
		for dz in range(-1, 2):
			for other in placed.get(cell + Vector2i(dx, dz), []):
				if centre.distance_to(other[0]) < radius + float(other[1]):
					return false
	return true


## A building facing the origin: its front, two sides and its roof (flat, or a low pitch with a
## front gable). The back faces away from every reachable eye and is not built.
static func _box(st: SurfaceTool, base: Vector3, bearing: float, w: float, d: float, h: float,
		capped: bool, wall: Color) -> void:
	var out := _dir(bearing)                    # away from the street
	var side := Vector3(-out.z, 0.0, out.x)     # along the frontage
	var f := base - out * (d * 0.5)             # front face centre at ground
	var b := base + out * (d * 0.5)
	var hw := side * (w * 0.5)
	var up := Vector3(0, h, 0)
	var fl := f - hw
	var fr := f + hw
	var bl := b - hw
	var br := b + hw
	_quad(st, fl, fr, fr + up, fl + up, -out, wall)                  # front
	_quad(st, bl, fl, fl + up, bl + up, -side, wall.darkened(0.12))  # left
	_quad(st, fr, br, br + up, fr + up, side, wall.darkened(0.12))   # right
	if not capped:
		_quad(st, fl + up, fr + up, br + up, bl + up, Vector3.UP, ROOF)
		return
	var ridge := Vector3(0, h + CAP_RISE_M, 0)
	var mf := f + ridge
	var mb := b + ridge
	_tri(st, fl + up, fr + up, mf, -out, wall)                       # front gable
	_quad(st, fl + up, mf, mb, bl + up, -side + Vector3.UP, ROOF)
	_quad(st, mf, fr + up, br + up, mb, side + Vector3.UP, ROOF)


static func _quad(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3, d: Vector3, facing: Vector3,
		colour: Color) -> void:
	_tri(st, a, b, c, facing, colour)
	_tri(st, a, c, d, facing, colour)


## One triangle whose front face looks along `facing`: in Godot's convention the front is clockwise
## as seen from that side, i.e. (b - a) x (c - a) points away from it.
static func _tri(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3, facing: Vector3,
		colour: Color) -> void:
	if (b - a).cross(c - a).dot(facing) > 0.0:
		var t := b
		b = c
		c = t
	for v in [a, b, c]:
		st.set_color(colour.srgb_to_linear())   # vertex colours are linear
		st.add_vertex(v)
