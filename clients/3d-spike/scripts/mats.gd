## Material library for the 3D spike.
##
## Textures are loaded straight off disk with Image.load_from_file rather than
## through Godot's import pipeline, so the project runs from a clean checkout
## with no editor pass and nothing generated has to be committed.
##
## Every textured architectural/ground material uses *world* triplanar mapping.
## That is the single biggest coherence lever in the whole spike: brick is the
## same physical size on every wall, paving is the same size everywhere, and no
## box ever needs hand-authored UVs. ART_DIRECTION.md sec.18 in one setting.
class_name Mats
extends RefCounted

const TEX_DIR := "res://assets/textures"

static var _tex_cache: Dictionary = {}
static var _mat_cache: Dictionary = {}


static func tex(slug: String, map: String) -> Texture2D:
	var key := slug + "/" + map
	if _tex_cache.has(key):
		return _tex_cache[key]
	var path := "%s/%s/%s_%s_1k.jpg" % [TEX_DIR, slug, slug, map]
	var t: Texture2D = null
	if ResourceLoader.exists(path):
		t = load(path) as Texture2D
	if t == null:
		# no import pass has run -- read the file directly
		var img := Image.load_from_file(path)
		if img != null:
			img.generate_mipmaps()
			t = ImageTexture.create_from_image(img)
		else:
			push_warning("missing texture %s" % path)
	_tex_cache[key] = t
	return t


## A PBR material from a Poly Haven diff/nor_gl/arm triple.
## `metres` is how many metres of world space one texture tile covers.
static func pbr(slug: String, metres: float, tint := Color.WHITE, rough := 1.0) -> Material:
	var key := "%s|%f|%s|%f" % [slug, metres, tint, rough]
	if _mat_cache.has(key):
		return _mat_cache[key]
	var m := ORMMaterial3D.new()
	m.albedo_texture = tex(slug, "diff")
	m.albedo_color = tint
	var n := tex(slug, "nor_gl")
	if n != null:
		m.normal_enabled = true
		m.normal_texture = n
		m.normal_scale = 1.0
	var o := tex(slug, "arm")
	if o != null:
		m.orm_texture = o
	m.roughness = rough
	m.uv1_triplanar = true
	m.uv1_world_triplanar = true
	m.uv1_triplanar_sharpness = 2.0
	var s := 1.0 / maxf(metres, 0.01)
	m.uv1_scale = Vector3(s, s, s)
	m.texture_filter = BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS_ANISOTROPIC
	_mat_cache[key] = m
	return m


## Flat painted surface: shopfront joinery, awnings, metalwork, clothing.
static func paint(c: Color, rough := 0.55, metal := 0.0) -> Material:
	var key := "paint|%s|%f|%f" % [c, rough, metal]
	if _mat_cache.has(key):
		return _mat_cache[key]
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.roughness = rough
	m.metallic = metal
	m.metallic_specular = 0.5
	_mat_cache[key] = m
	return m


## Shop glass: dark, reflective, and lit from behind by an emissive interior
## card so windows read warm at dusk the way they do in every reference.
static func glass() -> Material:
	if _mat_cache.has("glass"):
		return _mat_cache["glass"]
	var m := StandardMaterial3D.new()
	m.albedo_color = Color(0.030, 0.038, 0.045, 0.74)
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.roughness = 0.06
	m.metallic = 0.0
	m.metallic_specular = 0.62
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	_mat_cache["glass"] = m
	return m


## Shop glazing for a unit you can walk into. The dark 74%-opaque `glass()` is
## tuned to read as a reflective window from across the street; behind it there
## used to be nothing worth seeing. Where there is now a real room, the glass
## has to be clear enough to show it -- looking into the room is most of what
## makes `03_cafe_frontage` and `05_main_street_golden_hour` work, and it has to
## work from inside looking out as well.
static func clear_glass() -> Material:
	if _mat_cache.has("clear_glass"):
		return _mat_cache["clear_glass"]
	var m := StandardMaterial3D.new()
	m.albedo_color = Color(0.80, 0.84, 0.86, 0.09)
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.roughness = 0.03
	m.metallic = 0.0
	m.metallic_specular = 0.42
	_mat_cache["clear_glass"] = m
	return m


static func emissive(c: Color, energy := 1.0) -> Material:
	var key := "emis|%s|%f" % [c, energy]
	if _mat_cache.has(key):
		return _mat_cache[key]
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.emission_enabled = true
	m.emission = c
	m.emission_energy_multiplier = energy
	m.roughness = 0.9
	_mat_cache[key] = m
	return m


## Alpha-scissored card material for leaves, flowers and grass.
static func card(t: Texture2D, backlit := 0.0) -> Material:
	var m := StandardMaterial3D.new()
	m.albedo_texture = t
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA_SCISSOR
	m.alpha_scissor_threshold = 0.45
	m.alpha_antialiasing_mode = BaseMaterial3D.ALPHA_ANTIALIASING_ALPHA_TO_COVERAGE
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	m.roughness = 0.82
	m.specular_mode = BaseMaterial3D.SPECULAR_DISABLED
	if backlit > 0.0:
		# Cheap stand-in for leaf translucency: without it a canopy backlit by a
		# low sun goes flat black and the golden-hour read collapses.
		m.backlight_enabled = true
		m.backlight = Color(0.30, 0.42, 0.18) * backlit
	m.texture_filter = BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS_ANISOTROPIC
	return m


# --- named materials used across the town -------------------------------------

static func promenade() -> Material:
	return pbr("concrete_pavement", 1.8, Color(1.08, 1.02, 0.93), 0.95)

static func sidewalk() -> Material:
	return pbr("pavement_02", 2.2, Color(1.04, 1.0, 0.95), 0.95)

static func setts() -> Material:
	return pbr("cobblestone_floor_08", 1.05, Color(0.98, 0.96, 0.92))

static func road() -> Material:
	return pbr("asphalt_01", 4.0, Color(0.82, 0.82, 0.84))

static func brick() -> Material:
	return pbr("red_brick_03", 1.0, Color(1.12, 0.98, 0.9))

static func brick_tan() -> Material:
	return pbr("red_brick_03", 1.0, Color(1.55, 1.34, 1.05), 1.05)

static func stucco(c := Color(1.0, 0.97, 0.9)) -> Material:
	return pbr("beige_wall_001", 2.0, c)

static func cutstone() -> Material:
	return pbr("stone_brick_wall_001", 1.5, Color(1.16, 1.1, 0.98))

static func planks() -> Material:
	return pbr("weathered_brown_planks", 1.5, Color(1.0, 0.95, 0.86))

static func roof() -> Material:
	return pbr("roof_slates_03", 1.2, Color(0.86, 0.84, 0.84))

static func lawn() -> Material:
	return pbr("leafy_grass", 1.4, Color(0.62, 0.86, 0.42))
