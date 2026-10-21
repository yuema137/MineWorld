## The near water of a depiction file's `water` section (`DECISIONS.md` ARC-177, DEP-78; S33-d sec.5.3):
## the file's polygon, triangulated, at `level_m`, drawn with the slice's `water.gdshader` under
## `#define NEAR_LAKE` (depth fade, shore band, lily pads), in the colours of the world's far-field water
## the file names, shaded as the far field's water is (unshaded, a capped Fresnel to the sky's horizon
## colour, both copied from the far field's water material), so the near water continues the far field's
## at the same level and colours with no seam. No flow, no foam simulation.
##
## Lily pads are a decal: a single-channel mask generated at load from each `lilies` entry's polygon,
## cover and seed (zero texture bytes). Nothing here names a world, a place or a tag.
class_name DepictionWater
extends RefCounted

## The lily mask's side, pixels (S33-d sec.5.3: a 256 x 256 single-channel image).
const MASK_SIZE := 256
## A floating leaf's diameter range, metres: water-lily (Nymphaea) leaf blades are roughly 10 to 30 cm
## across; approximate, shared by every world's lilies.
const PAD_M := Vector2(0.10, 0.30)


## Draws the water `w` under `parent`; `origin` is the scene position of plan (0, 0).
static func build(parent: Node3D, w: Dictionary, far_field: MineWorldFarField, origin: Vector3) -> MeshInstance3D:
	var outline := _scene_polygon(w["polygon_mm"], origin)
	var triangles := Geometry2D.triangulate_polygon(outline)
	var y := origin.y + float(w["level_m"])
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	st.set_normal(Vector3.UP)
	for p in outline:
		st.add_vertex(Vector3(p.x, y, p.y))
	for k in triangles:
		st.add_index(k)
	var mat := ShaderMaterial.new()
	mat.shader = Shader.new()
	mat.shader.code = "#define NEAR_LAKE\n" + (load("res://shaders/water.gdshader") as Shader).code
	var named := false
	var bodies: Array = far_field.water() if far_field != null else []
	for body in bodies:
		if body["name"] == w["far_field_water"]:
			mat.set_shader_parameter("deep", Color.html(body["colour"]["deep"]))
			mat.set_shader_parameter("shallow", Color.html(body["colour"]["shallow"]))
			named = true
	if not named:
		print("[depiction] water: the far field has no water named %s; the shader's colours stand"
			% w["far_field_water"])
	# the far field's own water material, when drawn: its colours, horizon and Fresnel cap exactly, so
	# the two waters meet with no seam (the shader shades both alike)
	var drawn := parent.find_child("Water_%s" % w["far_field_water"], true, false) as MeshInstance3D
	if drawn != null and drawn.material_override is ShaderMaterial:
		var far := drawn.material_override as ShaderMaterial
		for uniform in ["deep", "shallow", "sky_horizon", "fresnel_cap"]:
			mat.set_shader_parameter(uniform, far.get_shader_parameter(uniform))
	var shader: Dictionary = w.get("shader", {}) if w.get("shader") != null else {}
	for uniform in ["depth_fade_m", "shore_foam"]:
		if shader.has(uniform):
			mat.set_shader_parameter(uniform, float(shader[uniform]))
	var lilies: Array = w.get("lilies", []) if w.get("lilies") != null else []
	if not lilies.is_empty():
		_lilies(mat, lilies, origin)
	var mi := MeshInstance3D.new()
	mi.name = "NearWater"
	mi.mesh = st.commit()
	mi.material_override = mat
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	parent.add_child(mi)
	print("[depiction] water: %d triangles at y %.2f" % [triangles.size() / 3, y])
	return mi


## The lily mask over the bounds of every `lilies` polygon, and its rectangle in the scene.
static func _lilies(mat: ShaderMaterial, lilies: Array, origin: Vector3) -> void:
	var polygons := []
	for entry in lilies:
		polygons.append(_scene_polygon(entry["polygon_mm"], origin))
	var bounds := _box(polygons[0])
	for polygon in polygons:
		bounds = bounds.merge(_box(polygon))
	if bounds.size.x <= 0.0 or bounds.size.y <= 0.0:
		return
	var img := Image.create_empty(MASK_SIZE, MASK_SIZE, false, Image.FORMAT_L8)
	var px_per_m := float(MASK_SIZE) / maxf(bounds.size.x, bounds.size.y)
	for k in lilies.size():
		var entry: Dictionary = lilies[k]
		var polygon: PackedVector2Array = polygons[k]
		var rng := RandomNumberGenerator.new()
		rng.seed = int(entry["seed"])
		var box := _box(polygon)
		var mean := (PAD_M.x + PAD_M.y) * 0.5
		var pads := int(float(entry["cover"]) * _area(polygon) / (PI * mean * mean * 0.25))
		var tries := pads * 4
		while pads > 0 and tries > 0:
			tries -= 1
			var at := Vector2(rng.randf_range(box.position.x, box.end.x), rng.randf_range(box.position.y, box.end.y))
			if not Geometry2D.is_point_in_polygon(at, polygon):
				continue
			_disc(img, (at - bounds.position) * px_per_m, rng.randf_range(PAD_M.x, PAD_M.y) * 0.5 * px_per_m)
			pads -= 1
	var side := maxf(bounds.size.x, bounds.size.y)
	mat.set_shader_parameter("lily_mask", ImageTexture.create_from_image(img))
	mat.set_shader_parameter("lily_rect", Vector4(bounds.position.x, bounds.position.y, side, side))


static func _disc(img: Image, centre: Vector2, radius: float) -> void:
	var r := maxf(radius, 0.75)
	for y in range(int(centre.y - r), int(centre.y + r) + 1):
		for x in range(int(centre.x - r), int(centre.x + r) + 1):
			if x < 0 or y < 0 or x >= MASK_SIZE or y >= MASK_SIZE:
				continue
			if Vector2(x + 0.5, y + 0.5).distance_to(centre) <= r:
				img.set_pixel(x, y, Color.WHITE)


## A plan polygon (mm) as scene (x, z) points.
static func _scene_polygon(points: Array, origin: Vector3) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in points:
		out.append(Vector2(origin.x + float(p[0]) / 1000.0, origin.z - float(p[1]) / 1000.0))
	return out


static func _box(polygon: PackedVector2Array) -> Rect2:
	var r := Rect2(polygon[0], Vector2.ZERO)
	for p in polygon:
		r = r.expand(p)
	return r


static func _area(polygon: PackedVector2Array) -> float:
	var twice := 0.0
	for k in polygon.size():
		var a := polygon[k]
		var b := polygon[(k + 1) % polygon.size()]
		twice += a.x * b.y - b.x * a.y
	return absf(twice) * 0.5
