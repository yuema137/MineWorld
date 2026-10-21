## The glazed-tile materials (`shaders/glazed_tile.gdshader`, S29-d §5.4): one material per pattern,
## built once and shared, so every tiled surface of one pattern draws with one material.
##
## The colours are the shader's defaults: warm and slightly desaturated, inside the pack's palette, as
## the landmark exception requires (ART_DIRECTION.md "Architecture", step-29 Q-S29-5).
class_name StationTile
extends RefCounted

const SHADER := "res://shaders/glazed_tile.gdshader"
## The kit's pattern names, as the shader numbers them.
const PATTERNS := {"zigzag": 0, "emblem": 1, "quarry": 2, "frieze": 3}

static var _cache := {}


## The material for `pattern` (a kit `tile` value), or null for a name the shader does not draw.
static func material(pattern: String) -> ShaderMaterial:
	if not PATTERNS.has(pattern):
		push_error("glazed tile: no pattern %s" % pattern)
		return null
	if _cache.has(pattern):
		return _cache[pattern]
	var m := ShaderMaterial.new()
	m.shader = Mats.shader(SHADER)
	m.set_shader_parameter("pattern", PATTERNS[pattern])
	_cache[pattern] = m
	return m


## Puts `pattern` on a mesh's surface (not as an override), with `tiles` tile units around and down its
## UV. A surface material keeps the mesh out of `SliceBatch` (which merges overrides only), so its own
## UV and instance parameter survive: domes and lanterns are a few draws, and the floor, drawn by
## world position, is merged as usual through `material_override`.
static func apply(mi: MeshInstance3D, pattern: String, tiles := Vector2(48.0, 12.0)) -> void:
	mi.material_override = null
	(mi.mesh as PrimitiveMesh).material = material(pattern)
	mi.set_instance_shader_parameter("tiles", tiles)
