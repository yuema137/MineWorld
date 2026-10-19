## The slice's key light follows the sun the world discloses (TW-e; `docs/DECISIONS.md` `ARC-270`;
## `clients/shared/WORLD_TIME.md` §6).
##
## `SliceMain` keeps ARC-13's golden-hour constants as the anchor; this node only moves the lights it is
## given. Direction: the interpolated disclosed sun, rotated by the slice's map north, chosen so that San
## Diego's afternoon sun at 19.3 deg on the epoch day stands exactly where ARC-13's rig does. Energy: the
## clear-sky direct beam by elevation (Kasten–Young air mass, Meinel's beam model), normalised to ARC-13's
## 4.4 at 19.3 deg. Colour: ARC-13's warm sun up to 19.3 deg, neutral by 40. Below the horizon the key
## light is off and the sky fill, the ambient, the panorama and the unshaded far field are dimmed by a
## phase table — presentation values, not illuminance: a real lux range cannot be shown at a fixed
## exposure. No astronomy here: the only trigonometry is on the disclosed elevation.
##
## FIXED MODE (`INV-WC-2`): until a `calendar-day` record is disclosed, in every scripted or capture mode
## and with `--sun=fixed`, nothing is touched and the rig is ARC-13's, bit for bit.
class_name SliceSun
extends Node

## Where the slice's map north points, in Godot yaw: `yaw = SLICE_NORTH_YAW_OFFSET - azimuth`. San
## Diego's sun on 2026-10-08 is at 19.3 deg of elevation at 15:45 local standard time, azimuth 248.86 deg
## (NOAA GML's solar algorithm at 32.7157, -117.1611; the pack's refraction moves it under 0.05 deg), so
## -48 = 200.86 - 248.86 (ARC-270 item 1). Presentation content.
const SLICE_NORTH_YAW_OFFSET := 200.86
const GOLDEN_ELEVATION := 19.3
const NEUTRAL_ELEVATION := 40.0
const NEUTRAL := Color(1.0, 0.96, 0.90)
## The sunrise horizon with the pack's standard refraction.
const HORIZON := -0.833
## The share of today's sky light in each phase (ARC-270 item 3).
const PHASE_SKY := {"day": 1.0, "civil-twilight": 0.35, "astronomical-twilight": 0.08, "night": 0.03}
## How fast a change of light is followed: a jump becomes a fade over about a second.
const FOLLOW_PER_SECOND := 3.0

var sun: DirectionalLight3D
var fill: DirectionalLight3D
var environment: Environment
var far_field: Node3D = null
## True while the rig must stay ARC-13's (scripted modes, `--sun=fixed`).
var fixed := false

var _fill_energy := 0.0
var _ambient := 0.0
var _sky_energy := 0.0
var _sky_factor := 1.0
var _far_factor := 1.0
var _far_materials: Array[StandardMaterial3D] = []
var _moving := false


## Takes the rig as built; nothing changes until `follow` is given a valid sun outside fixed mode.
func setup(a_sun: DirectionalLight3D, a_fill: DirectionalLight3D, a_environment: Environment,
		a_fixed: bool) -> void:
	sun = a_sun
	fill = a_fill
	environment = a_environment
	fixed = a_fixed
	_fill_energy = fill.light_energy
	_ambient = environment.ambient_light_energy
	_sky_energy = _sky_material_energy()


## `--sun=fixed` on the command line.
static func fixed_requested() -> bool:
	return OS.get_cmdline_user_args().has("--sun=fixed")


## The clear-sky direct beam at `elevation` (deg), relative to ARC-13's 19.3 deg and scaled to its 4.4.
static func energy_at(elevation: float) -> float:
	if elevation <= HORIZON:
		return 0.0
	return SliceMain.SUN_ENERGY * _beam(elevation) / _beam(GOLDEN_ELEVATION)


## Meinel's clear-sky beam, kW m^-2, over the Kasten–Young air mass.
static func _beam(elevation: float) -> float:
	var air_mass := 1.0 / (sin(deg_to_rad(elevation)) + 0.50572 * pow(elevation + 6.07995, -1.6364))
	return 1.353 * pow(0.7, pow(air_mass, 0.678))


static func colour_at(elevation: float) -> Color:
	var f := clampf((elevation - GOLDEN_ELEVATION) / (NEUTRAL_ELEVATION - GOLDEN_ELEVATION), 0.0, 1.0)
	return SlicePalette.SUN_WARM.lerp(NEUTRAL, f)


## This frame's sun (`MineWorldWorldTime.sun_at`) and phase (`phase_at`), with the frame's `delta`.
func follow(disclosed: Dictionary, phase: String, has_calendar: bool, delta: float) -> void:
	if fixed or not has_calendar or sun == null:
		return
	var elevation := float(disclosed.get("elevation_deg", GOLDEN_ELEVATION))
	var azimuth := float(disclosed.get("azimuth_deg", 0.0))
	if not _moving:
		_moving = true
		_collect_far_field()
	sun.rotation_degrees = Vector3(-elevation, SLICE_NORTH_YAW_OFFSET - azimuth, 0.0)
	sun.light_energy = energy_at(elevation)
	sun.light_color = colour_at(elevation)
	var want: float = PHASE_SKY.get(phase, 1.0)
	var k := clampf(delta * FOLLOW_PER_SECOND, 0.0, 1.0)
	_sky_factor = lerpf(_sky_factor, want, k)
	fill.light_energy = _fill_energy * _sky_factor
	environment.ambient_light_energy = _ambient * _sky_factor
	_set_sky_material_energy(_sky_energy * _sky_factor)
	# The unshaded far field was baked under ARC-13's light: scaled by the light now, against that.
	var golden := SliceMain.SUN_ENERGY * sin(deg_to_rad(GOLDEN_ELEVATION)) + SliceMain.SKY_AMBIENT * 0.8
	var now := sun.light_energy * maxf(sin(deg_to_rad(elevation)), 0.0) + _ambient * _sky_factor * 0.8
	var factor := clampf(now / golden, 0.02, 1.6)
	if absf(factor - _far_factor) > 0.02 * _far_factor:
		_far_factor = factor
		for mat in _far_materials:
			mat.albedo_color = Color(factor, factor, factor)


func _collect_far_field() -> void:
	_far_materials.clear()
	if far_field == null:
		return
	for node in far_field.find_children("*", "MeshInstance3D", true, false):
		var mi := node as MeshInstance3D
		var mats: Array = [mi.material_override]
		if mi.mesh != null:
			for i in mi.mesh.get_surface_count():
				mats.append(mi.mesh.surface_get_material(i))
		for m in mats:
			var std := m as StandardMaterial3D
			if std != null and std.shading_mode == BaseMaterial3D.SHADING_MODE_UNSHADED \
					and not _far_materials.has(std):
				_far_materials.append(std)


func _sky_material_energy() -> float:
	var material: Material = environment.sky.sky_material if environment.sky != null else null
	if material is PanoramaSkyMaterial:
		return (material as PanoramaSkyMaterial).energy_multiplier
	if material is ProceduralSkyMaterial:
		return (material as ProceduralSkyMaterial).energy_multiplier
	return 1.0


func _set_sky_material_energy(value: float) -> void:
	var material: Material = environment.sky.sky_material if environment.sky != null else null
	if material is PanoramaSkyMaterial:
		(material as PanoramaSkyMaterial).energy_multiplier = value
	elif material is ProceduralSkyMaterial:
		(material as ProceduralSkyMaterial).energy_multiplier = value
