## The slice's material authority.
##
## `DEP-8`'s coherence procedure, step 2: take material authority away from the
## asset authors. Every albedo in the slice is named here, every texture tile
## size is stated in metres here, and nothing downstream invents a colour. That
## is what stops thirteen Poly Haven textures and thirty-eight Poly Haven props
## from reading as thirteen textures and thirty-eight props.
##
## Colours are read off the reference plates as pixels, at magnification, and
## are then corrected against captured runtime frames -- an albedo judged
## outside the lighting rig is judged wrong (`DEP-8` step 3).
##
## Texel density target ~512 px/m for a walkable street (`DEP-8` step 4). Every
## `Mats.pbr` call below states the metres one 1k tile covers, so the density is
## 1024 / metres and is checkable by reading this file: 2.0 m gives 512 px/m.
class_name SlicePalette
extends RefCounted

# --- the reference palette ----------------------------------------------------
# Read from presentation/mineworld-default/3D/references/, named for what the
# plate shows rather than for the hex, so a later correction stays legible.

## 03: the cafe's shopfront joinery. A muted sage-eucalyptus green, mid value,
## visibly *painted timber* and not a saturated enamel.
const CAFE_GREEN := Color(0.243, 0.322, 0.278)
const CAFE_GREEN_DARK := Color(0.150, 0.203, 0.176)
const CAFE_GREEN_LIGHT := Color(0.336, 0.412, 0.360)
## 03: the fascia lettering and the window script. Warm off-white, not white.
const SIGN_CREAM := Color(0.914, 0.870, 0.760)
## 05: Riverstone Books, a near-black charcoal green.
const BOOKS_GREEN := Color(0.116, 0.140, 0.126)
## 05: Sunrise Bakery's cream fascia.
const BAKERY_CREAM := Color(0.828, 0.774, 0.664)
## 05: the awning canvas over the corner cafe.
const AWNING_GREEN := Color(0.196, 0.250, 0.208)
## 05: the bakery's blue-and-white striped awning.
const AWNING_BLUE := Color(0.352, 0.438, 0.530)
## 03/05: cast iron -- lamp columns, brackets, bistro frames, railings.
const IRONWORK := Color(0.078, 0.082, 0.080)
## 03: the hanging sign board and the A-board frames.
const SIGN_TIMBER := Color(0.236, 0.163, 0.112)
## 03: slate, for chalkboards.
const SLATE := Color(0.128, 0.140, 0.136)
## 03: the brass door furniture.
const BRASS := Color(0.706, 0.556, 0.278)
## 03: the interior's warm pendant light.
const LAMP_WARM := Color(1.0, 0.760, 0.470)
## 05: the low sun.
const SUN_WARM := Color(1.0, 0.80, 0.56)

# Timber inside the cafe: the ceiling boards, the shelving, the counter front.
# Darkened 2026-10-06: through 03's glass the shelving and the ceiling are dark
# stained timber, and the pale versions made the room read as lit by daylight.
const CEIL_TIMBER := Color(0.330, 0.218, 0.136)
const SHELF_TIMBER := Color(0.262, 0.176, 0.112)
const COUNTER_TIMBER := Color(0.300, 0.208, 0.142)


# --- architectural surfaces ---------------------------------------------------
# One tile size per surface class, stated in metres of world space.

## Grey-buff granite setts with dark soil joints, as both 03 and 05 lay them:
## roughly square heads of 0.15-0.22 m, in loose courses, each stone a slightly
## different grey. `cobblestone_floor_08` carries about nine courses per 1k tile,
## so 1.6 m puts a course near 0.18 m. Was `rectangular_paving` until 2026-10-06:
## warm beige oblong slabs whose joints did not read at 03's framing (preview
## miss #2).
static func setts() -> Material:
	return Mats.pbr("cobblestone_floor_08", 1.60, Color(1.10, 1.06, 0.99), 0.90)

## The same stone, darker, for the gutter course so the road edge is not one
## flat plane.
static func setts_grey() -> Material:
	return Mats.pbr("cobblestone_floor_08", 1.60, Color(0.78, 0.78, 0.78), 0.88)

## 05's carriageway: the pavements' setts, a course larger and a shade darker,
## worn smoother by wheels.
static func road_setts() -> Material:
	return Mats.pbr("cobblestone_floor_08", 2.10, Color(0.80, 0.79, 0.77), 0.82)

static func asphalt() -> Material:
	return Mats.pbr("worn_asphalt", 4.20, Color(0.78, 0.78, 0.82), 0.96)

## Kerbs, entrance steps, plinths, bollard bases, planter walls. Tiled large so
## the source's form joints read as one stone rather than as panelling.
static func kerbstone() -> Material:
	return Mats.pbr("concrete", 3.40, Color(0.88, 0.88, 0.90), 0.84)

## 03: the cafe's coursed rubble limestone above the shopfront.
static func limestone() -> Material:
	return Mats.pbr("sandstone_blocks_05", 2.60, Color(1.08, 1.03, 0.94), 0.94)

## 05: Riverstone Books and the corner block.
static func brick_red() -> Material:
	return Mats.pbr("red_bricks_04", 1.80, Color(1.34, 1.10, 0.96), 0.95)

## 05: the apartment block's buff stock brick, and the ghost-sign gable.
static func brick_buff() -> Material:
	return Mats.pbr("yellow_bricks", 1.80, Color(1.10, 1.06, 0.98), 0.95)

static func render_cream() -> Material:
	return Mats.pbr("beige_wall_001", 2.40, Color(1.06, 1.00, 0.90), 0.93)

## 02: terracotta pantiles.
static func pantile() -> Material:
	return Mats.pbr("clay_roof_tiles_02", 1.40, Color(0.90, 0.80, 0.74), 0.90)

static func slate_roof() -> Material:
	return Mats.pbr("roof_slates_03", 1.30, Color(0.80, 0.80, 0.84), 0.88)


# --- interior surfaces --------------------------------------------------------

static func floor_boards() -> Material:
	# stained, not raw: the raw tint filled the lower half of every interior
	# frame with bright orange board
	return Mats.pbr("wood_floor_worn", 2.20, Color(0.74, 0.62, 0.52), 0.62)

## 03: the ceiling is exposed warm boards running front to back, and it is the
## single most visible interior surface after the shelving wall.
static func ceiling_boards() -> Material:
	return Mats.pbr("brown_planks_09", 1.60, Color(0.58, 0.44, 0.32), 0.82)

static func joinery_timber() -> Material:
	return Mats.pbr("wood_table_worn", 1.30, Color(1.06, 0.94, 0.82), 0.48)

static func counter_top() -> Material:
	return Mats.pbr("wood_table_worn", 1.05, Color(0.78, 0.66, 0.56), 0.28)

## The metro tiling behind the counter.
static func bar_tile() -> Material:
	return Mats.pbr("long_white_tiles", 1.00, Color(1.02, 0.98, 0.92), 0.22)

static func plaster_interior() -> Material:
	# a warm limewash rather than near-white: through the glass, pale plaster was
	# most of what the room read as (preview miss #1). 0.86/0.72/0.56 rendered
	# orange under the pendants; this is a browner, darker limewash.
	return Mats.pbr("painted_plaster_wall", 2.60, Color(0.66, 0.56, 0.45), 0.92)


static var _rug: Material = null

## A flat-woven rug: a field of stepped diamonds in madder red and indigo on a
## wool-cream ground, inside two border bands, with a per-thread weave noise.
## Authored here as pixels rather than sourced: no CC0 library carries a rug
## texture, and a generated one carries no licence question (`DEP-8`). Preview
## miss #7: the rug was one flat colour.
static func rug() -> Material:
	if _rug != null:
		return _rug
	var w := 256
	var h := 512
	var img := Image.create(w, h, false, Image.FORMAT_RGB8)
	var ground := Color(0.78, 0.68, 0.54)
	var red := Color(0.56, 0.20, 0.15)
	var indigo := Color(0.20, 0.24, 0.36)
	var ochre := Color(0.74, 0.54, 0.26)
	var rng := RandomNumberGenerator.new()
	rng.seed = 2207
	for y in range(h):
		for x in range(w):
			var bx := mini(x, w - 1 - x)
			var by := mini(y, h - 1 - y)
			var edge := mini(bx, by)
			var c := ground
			if edge < 14:
				c = red
			elif edge < 20:
				c = ground
			elif edge < 30:
				c = indigo if ((x + y) / 8) % 2 == 0 else ochre
			elif edge < 36:
				c = ground
			else:
				# the field: stepped diamonds on a 64 px lattice, quantised to 8 px
				# so the edges step as a flat weave's do
				var cx := float(((x - 36) % 64) / 8 * 8) - 28.0
				var cy := float(((y - 36) % 64) / 8 * 8) - 28.0
				var d := absf(cx) + absf(cy)
				if d < 12.0:
					c = indigo
				elif d < 24.0:
					c = red
				elif d < 30.0:
					c = ochre
			# weave: alternate warp rows a shade apart, and per-thread noise
			var k := 0.92 + 0.06 * float(y % 2) + rng.randf_range(-0.04, 0.04)
			img.set_pixel(x, y, Color(c.r * k, c.g * k, c.b * k))
	img.generate_mipmaps()
	var m := StandardMaterial3D.new()
	m.albedo_texture = ImageTexture.create_from_image(img)
	m.roughness = 0.97
	m.texture_filter = BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS_ANISOTROPIC
	_rug = m
	return m


# --- painted joinery ----------------------------------------------------------
# Painted timber, not enamel: roughness stays high enough that the sun gives a
# broad soft sheen rather than a hotspot.

static func painted(c: Color, rough := 0.52) -> Material:
	return Mats.paint(c, rough)

static func iron(rough := 0.42) -> Material:
	return Mats.paint(IRONWORK, rough, 0.35)

static func brass(rough := 0.26) -> Material:
	return Mats.paint(BRASS, rough, 0.90)

static func slate_board() -> Material:
	return Mats.paint(SLATE, 0.88)

## Awning and banner canvas: double sided, because an awning is a single-sided
## sheet seen from below as often as from above, and matt, because a low sun on
## a specular sheet reads as plastic.
static func canvas(c: Color) -> Material:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.roughness = 0.88
	m.metallic = 0.0
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	# canvas is thin: a little light comes through it, which is what keeps the
	# underside of an awning from going flat black at 17 degrees of sun.
	m.backlight_enabled = true
	m.backlight = c * 0.30
	return m


# --- glass --------------------------------------------------------------------

## Shopfront glazing on the enterable unit. Clear enough that the room behind it
## is the subject -- looking in is most of what makes 03 work -- and smooth
## enough that the street reflects in it, which is what stops a window reading
## as a hole. `metallic_specular` carries the reflection because a transparent
## material in Forward+ takes its environment from the specular term.
static func shop_glass() -> Material:
	var m := StandardMaterial3D.new()
	m.albedo_color = Color(0.74, 0.79, 0.80, 0.085)
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.roughness = 0.025
	m.metallic = 0.0
	m.metallic_specular = 0.58
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	# Refraction off on purpose: the pane is a single quad, so a refracted
	# screen sample would smear the street across the room behind it.
	return m

## Upper-storey and secondary-facade glazing, where there is no room behind.
## Darker and more reflective, so it reads as a window from the street without
## needing an interior.
static func dead_glass() -> Material:
	var m := StandardMaterial3D.new()
	m.albedo_color = Color(0.055, 0.070, 0.082, 0.80)
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.roughness = 0.035
	m.metallic = 0.0
	m.metallic_specular = 0.72
	return m

## A pane in a door leaf: clear, thin, and double sided.
static func door_glass() -> Material:
	var m := shop_glass().duplicate() as StandardMaterial3D
	m.albedo_color = Color(0.78, 0.82, 0.84, 0.11)
	return m


# --- signage ------------------------------------------------------------------

static var _serif: Font = null

## A serif face for shop signage. The reference fascias are serif capitals and a
## grotesque reads immediately as the wrong town. Resolved from the host's own
## fonts rather than committed, so no font licence enters the repository; the
## list ends in the generic `serif` family, which every platform resolves.
static func serif_font() -> Font:
	if _serif == null:
		var f := SystemFont.new()
		f.font_names = PackedStringArray([
			"Georgia", "Times New Roman", "Liberation Serif", "DejaVu Serif", "serif"])
		f.antialiasing = TextServer.FONT_ANTIALIASING_GRAY
		_serif = f
	return _serif

static var _script_font: Font = null

## The handwritten chalk and window-script face. Same sourcing rule.
static func script_font() -> Font:
	if _script_font == null:
		var f := SystemFont.new()
		f.font_names = PackedStringArray([
			"Snell Roundhand", "Bradley Hand", "Segoe Script", "Comic Sans MS",
			"Georgia", "serif"])
		f.antialiasing = TextServer.FONT_ANTIALIASING_GRAY
		_script_font = f
	return _script_font
