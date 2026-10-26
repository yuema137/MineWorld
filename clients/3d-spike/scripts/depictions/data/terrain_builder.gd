## The ground of a depiction file's `terrain` section (`DECISIONS.md` ARC-177, DEP-77; S33-d sec.5.0.3):
## one height grid rasterised at load from the file's regions, height profiles and the near water's bed,
## drawn as one `ArrayMesh` with a splat of at most four layers, and collided as one `HeightMapShape3D`
## on `Build.LAYER_WORLD`, where people stand by the ray of step-33 sec.4.6. Height is presentation:
## the server never sees it.
##
## The rules, all from the file:
##   - outside every region, a point follows the two profiles whose bearings bracket it, each sampled at
##     its distance from the scene origin (the far field's centre), blended by angle; inside the near
##     water's polygon the ground is its bed, falling `bed.depth_m` below the water over `bed.run_m`
##     from the polygon's edge;
##   - a region sets its own height (flat, crowned, or a ramp between two lines) and blends into its
##     surroundings over `blend_m`; later regions win;
##   - the splat layer is the region's, a path's within its half-width, else `default_layer`.
## Deterministic: no noise; the same file gives the same grid on every OS, and its hash is printed.
## Nothing here names a world, a place, a tag or a value that differs between worlds.
class_name DepictionTerrain
extends RefCounted

## The splat's layer count (step-33 sec.4.1; the validator's cap).
const MAX_LAYERS := 4

## The scene position of grid vertex (0, 0), the cell, and the vertex counts along x and z.
var x0 := 0.0
var z0 := 0.0
var cell := 1.0
var nx := 0
var nz := 0
## Heights, row-major (`j * nx + i`; i along +x, j along +z), and each vertex's layer index.
var heights := PackedFloat32Array()
var layers := PackedByteArray()
## The grid's SHA-256, hex.
var grid_hash := ""

var _origin := Vector3.ZERO


## Rasterises `t` (the file's `terrain`) and draws it under `parent`. `water` is the file's `water`
## section or null; `origin` is the scene position of plan (0, 0).
static func build(parent: Node3D, t: Dictionary, water: Variant, paths: Array, origin: Vector3) -> DepictionTerrain:
	var started := Time.get_ticks_msec()
	var terrain := DepictionTerrain.new()
	terrain._origin = origin
	terrain._rasterise(t, water, paths)
	terrain._draw(parent, t)
	terrain._collide(parent)
	print("[depiction] terrain %dx%d at %.2f m, hash %s, %d ms" % [terrain.nx, terrain.nz, terrain.cell,
		terrain.grid_hash, Time.get_ticks_msec() - started])
	return terrain


## The drawn ground's height at a scene point (bilinear in the grid; the edge's outside it).
func height_at(x: float, z: float) -> float:
	var fx := clampf((x - x0) / cell, 0.0, float(nx - 1))
	var fz := clampf((z - z0) / cell, 0.0, float(nz - 1))
	var i := mini(int(fx), nx - 2)
	var j := mini(int(fz), nz - 2)
	var u := fx - float(i)
	var v := fz - float(j)
	var a := lerpf(heights[j * nx + i], heights[j * nx + i + 1], u)
	var b := lerpf(heights[(j + 1) * nx + i], heights[(j + 1) * nx + i + 1], u)
	return lerpf(a, b, v)


## A plan point (mm) in the scene, at the drawn ground's height.
func on_ground(mm: Vector2) -> Vector3:
	var p := _origin + Vector3(mm.x / 1000.0, 0.0, -mm.y / 1000.0)
	p.y = height_at(p.x, p.z)
	return p


# --- rasterising -----------------------------------------------------------------------------------

func _plan(i: int, j: int) -> Vector2:
	var x := x0 + float(i) * cell
	var z := z0 + float(j) * cell
	return Vector2((x - _origin.x) * 1000.0, (_origin.z - z) * 1000.0)


func _rasterise(t: Dictionary, water: Variant, paths: Array) -> void:
	cell = float(t["cell_m"])
	var e: Array = t["extent_mm"]
	x0 = _origin.x + float(e[0]) / 1000.0
	z0 = _origin.z - (float(e[1]) + float(e[3])) / 1000.0
	nx = int(round(float(e[2]) / 1000.0 / cell)) + 1
	nz = int(round(float(e[3]) / 1000.0 / cell)) + 1
	heights.resize(nx * nz)
	layers.resize(nx * nz)
	var names := []
	for layer in t["layers"]:
		names.append(layer["name"])
	var default_layer := names.find(t["default_layer"])
	var profiles: Array = t.get("profiles", [])
	profiles = profiles.duplicate()
	profiles.sort_custom(func(a, b): return float(a["bearing_deg"]) < float(b["bearing_deg"]))
	var lake := PackedVector2Array()
	var level := 0.0
	if water != null:
		lake = _polygon(water["polygon_mm"])
		level = float(water["level_m"])
	var bed: Variant = t.get("bed")
	for j in nz:
		for i in nx:
			var p := _plan(i, j)
			var h := _profile_height(profiles, x0 + float(i) * cell, z0 + float(j) * cell)
			if not lake.is_empty() and Geometry2D.is_point_in_polygon(p, lake):
				h = level
				if bed != null:
					var d := _distance_to_polygon(p, lake) / 1000.0
					h = level - float(bed["depth_m"]) * clampf(d / float(bed["run_m"]), 0.0, 1.0)
			heights[j * nx + i] = h
			layers[j * nx + i] = default_layer
	for region in t["regions"]:
		_region(region, names.find(region["layer"]), float(t["blend_m"]))
	for path in paths:
		_path(path, names.find(path["layer"]))
	var ctx := HashingContext.new()
	ctx.start(HashingContext.HASH_SHA256)
	ctx.update(heights.to_byte_array())
	ctx.update(layers)
	grid_hash = ctx.finish().hex_encode()


## The profiles' height at scene (x, z): the two whose bearings bracket the point's bearing from the
## scene origin, each sampled at its distance, blended by angle. No profile: the street's surface.
static func _profile_height(profiles: Array, x: float, z: float) -> float:
	if profiles.is_empty():
		return 0.0
	var distance := Vector2(x, z).length()
	var bearing := fposmod(rad_to_deg(atan2(x, -z)), 360.0)
	if profiles.size() == 1:
		return _sample(profiles[0], distance)
	var after := 0
	while after < profiles.size() and float(profiles[after]["bearing_deg"]) < bearing:
		after += 1
	var a: Dictionary = profiles[(after - 1 + profiles.size()) % profiles.size()]
	var b: Dictionary = profiles[after % profiles.size()]
	var span := fposmod(float(b["bearing_deg"]) - float(a["bearing_deg"]), 360.0)
	var into := fposmod(bearing - float(a["bearing_deg"]), 360.0)
	var w := 0.0 if span <= 0.0 else into / span
	return lerpf(_sample(a, distance), _sample(b, distance), w)


## A profile at `distance` metres: linear between samples, the end samples' heights beyond them.
static func _sample(profile: Dictionary, distance: float) -> float:
	var samples: Array = profile["samples"]
	if samples.is_empty():
		return 0.0
	if distance <= float(samples[0]["distance_m"]):
		return float(samples[0]["height_m"])
	for k in range(1, samples.size()):
		var lo: Dictionary = samples[k - 1]
		var hi: Dictionary = samples[k]
		if distance <= float(hi["distance_m"]):
			var run := float(hi["distance_m"]) - float(lo["distance_m"])
			var w := 0.0 if run <= 0.0 else (distance - float(lo["distance_m"])) / run
			return lerpf(float(lo["height_m"]), float(hi["height_m"]), w)
	return float(samples[-1]["height_m"])


## A region: its height inside, blended into what lies round it over `blend` metres.
func _region(region: Dictionary, layer: int, blend: float) -> void:
	var polygon := _polygon(region["polygon_mm"])
	var box := _bounds(polygon).grow(blend * 1000.0)
	var span := _span(box)
	for j in range(span.position.y, span.end.y):
		for i in range(span.position.x, span.end.x):
			var p := _plan(i, j)
			var own := _region_height(region["height"], polygon, p)
			if Geometry2D.is_point_in_polygon(p, polygon):
				heights[j * nx + i] = own
				layers[j * nx + i] = layer
			elif blend > 0.0:
				var d := _distance_to_polygon(p, polygon) / 1000.0
				if d < blend:
					var w := smoothstep(1.0, 0.0, d / blend)
					heights[j * nx + i] = lerpf(heights[j * nx + i], own, w)


## A region's own height at plan point `p` (mm).
static func _region_height(height: Dictionary, polygon: PackedVector2Array, p: Vector2) -> float:
	if height.has("ramp"):
		var r: Dictionary = height["ramp"]
		var a0 := _point(r["from_mm"][0])
		var a1 := _point(r["from_mm"][1])
		var b0 := _point(r["to_mm"][0])
		var normal := (a1 - a0).orthogonal().normalized()
		var across := (b0 - a0).dot(normal)
		var t := 0.0 if absf(across) < 1.0 else clampf((p - a0).dot(normal) / across, 0.0, 1.0)
		return lerpf(float(r["from_m"]), float(r["to_m"]), t)
	var h := float(height["flat_m"])
	if height.has("crown_m"):
		var box := _bounds(polygon)
		var along_x: bool = height["crown_along"] == "x"
		var lo: float = box.position.y if along_x else box.position.x
		var size: float = box.size.y if along_x else box.size.x
		var at: float = p.y if along_x else p.x
		var half := size * 0.5
		if half > 0.0:
			var off := (at - (lo + half)) / half
			h += float(height["crown_m"]) * maxf(0.0, 1.0 - off * off)
	return h


## A path paints its layer within its half-width of its centre line.
func _path(path: Dictionary, layer: int) -> void:
	var line := _polygon(path["line_mm"])
	var half := float(path["width_m"]) * 500.0
	var span := _span(_bounds(line).grow(half))
	for j in range(span.position.y, span.end.y):
		for i in range(span.position.x, span.end.x):
			if _distance_to_line(_plan(i, j), line) <= half:
				layers[j * nx + i] = layer


## The grid vertices (i, j) a plan box (mm) covers, clipped to the grid; end exclusive.
func _span(box: Rect2) -> Rect2i:
	var xa := _origin.x + box.position.x / 1000.0
	var xb := _origin.x + box.end.x / 1000.0
	var za := _origin.z - box.end.y / 1000.0
	var zb := _origin.z - box.position.y / 1000.0
	var i0 := clampi(int(floor((xa - x0) / cell)), 0, nx)
	var i1 := clampi(int(ceil((xb - x0) / cell)) + 1, 0, nx)
	var j0 := clampi(int(floor((za - z0) / cell)), 0, nz)
	var j1 := clampi(int(ceil((zb - z0) / cell)) + 1, 0, nz)
	return Rect2i(i0, j0, maxi(i1 - i0, 0), maxi(j1 - j0, 0))


# --- drawing -----------------------------------------------------------------------------------------

const SPLAT := """
shader_type spatial;
render_mode cull_back;
uniform sampler2D tex0 : source_color, filter_linear_mipmap, repeat_enable;
uniform sampler2D tex1 : source_color, filter_linear_mipmap, repeat_enable;
uniform sampler2D tex2 : source_color, filter_linear_mipmap, repeat_enable;
uniform sampler2D tex3 : source_color, filter_linear_mipmap, repeat_enable;
uniform vec4 tint0 : source_color = vec4(1.0);
uniform vec4 tint1 : source_color = vec4(1.0);
uniform vec4 tint2 : source_color = vec4(1.0);
uniform vec4 tint3 : source_color = vec4(1.0);
uniform vec4 repeat_m = vec4(1.0);
varying vec3 world;
void vertex() { world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz; }
void fragment() {
	vec4 w = COLOR / max(COLOR.r + COLOR.g + COLOR.b + COLOR.a, 0.0001);
	vec2 p = world.xz;
	vec3 c = texture(tex0, p / repeat_m.x).rgb * tint0.rgb * w.r
		+ texture(tex1, p / repeat_m.y).rgb * tint1.rgb * w.g
		+ texture(tex2, p / repeat_m.z).rgb * tint2.rgb * w.b
		+ texture(tex3, p / repeat_m.w).rgb * tint3.rgb * w.a;
	ALBEDO = c;
	ROUGHNESS = 0.95;
}
"""


func _draw(parent: Node3D, t: Dictionary) -> void:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var weights := [Color(1, 0, 0, 0), Color(0, 1, 0, 0), Color(0, 0, 1, 0), Color(0, 0, 0, 1)]
	for j in nz:
		for i in nx:
			st.set_color(weights[clampi(layers[j * nx + i], 0, MAX_LAYERS - 1)])
			st.set_uv(Vector2(i, j))
			st.add_vertex(Vector3(x0 + float(i) * cell, heights[j * nx + i], z0 + float(j) * cell))
	for j in nz - 1:
		for i in nx - 1:
			var a := j * nx + i
			var b := a + 1
			var c := a + nx
			var d := c + 1
			st.add_index(a)
			st.add_index(b)
			st.add_index(c)
			st.add_index(b)
			st.add_index(d)
			st.add_index(c)
	st.generate_normals()
	var mat := ShaderMaterial.new()
	mat.shader = Shader.new()
	mat.shader.code = SPLAT
	var repeat := Vector4.ONE
	var given: Array = t["layers"]
	for k in MAX_LAYERS:
		var layer: Dictionary = given[mini(k, given.size() - 1)]
		mat.set_shader_parameter("tex%d" % k, Mats.tex(layer["material"], "diff"))
		mat.set_shader_parameter("tint%d" % k, Color.html(layer["tint"]) if layer.has("tint") else Color.WHITE)
		repeat[k] = float(layer["repeat_m"])
	mat.set_shader_parameter("repeat_m", repeat)
	var mi := MeshInstance3D.new()
	mi.name = "Terrain"
	mi.mesh = st.commit()
	mi.material_override = mat
	parent.add_child(mi)


func _collide(parent: Node3D) -> void:
	var shape := HeightMapShape3D.new()
	shape.map_width = nx
	shape.map_depth = nz
	shape.map_data = heights
	var cs := CollisionShape3D.new()
	cs.shape = shape
	cs.scale = Vector3(cell, 1.0, cell)
	var body := StaticBody3D.new()
	body.name = "TerrainBody"
	body.collision_layer = Build.LAYER_WORLD
	body.position = Vector3(x0 + float(nx - 1) * cell * 0.5, 0.0, z0 + float(nz - 1) * cell * 0.5)
	body.add_child(cs)
	parent.add_child(body)


# --- plan geometry -----------------------------------------------------------------------------------

static func _point(p: Array) -> Vector2:
	return Vector2(float(p[0]), float(p[1]))


static func _polygon(points: Array) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in points:
		out.append(_point(p))
	return out


static func _bounds(points: PackedVector2Array) -> Rect2:
	var r := Rect2(points[0], Vector2.ZERO)
	for p in points:
		r = r.expand(p)
	return r


## The distance (mm) from `p` to a closed polygon's boundary.
static func _distance_to_polygon(p: Vector2, polygon: PackedVector2Array) -> float:
	var best := INF
	for k in polygon.size():
		var a := polygon[k]
		var b := polygon[(k + 1) % polygon.size()]
		best = minf(best, p.distance_to(Geometry2D.get_closest_point_to_segment(p, a, b)))
	return best


## The distance (mm) from `p` to an open polyline.
static func _distance_to_line(p: Vector2, line: PackedVector2Array) -> float:
	var best := INF
	for k in line.size() - 1:
		best = minf(best, p.distance_to(Geometry2D.get_closest_point_to_segment(p, line[k], line[k + 1])))
	return best
