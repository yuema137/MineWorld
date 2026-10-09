shader_type canvas_item;

// Paper tooth and a colour grade, run over the finished frame.
//
// Flat vector art fails the reference comparison for two measurable reasons,
// and neither is an asset problem:
//
//   1. no detail below the shape scale — every surface is a dead-flat fill,
//      so nothing reads as painted on anything;
//   2. no dark mass — a k-means over the reference plates puts ~11% of their
//      pixels below #363328, where the spike had almost nothing that dark.
//
// This pass addresses both. The grain is split: a coarse mottle anchored to
// the world, so it reads as texture lying on the ground and does not swim when
// the camera moves, and a fine tooth fixed to the screen, so it reads as the
// paper the whole scene is printed on. The grade then bends the curve, warms
// the light, cools the shade and closes down the corners.

uniform sampler2D screen_tex : hint_screen_texture, filter_linear;

// Real cold-press paper tooth, derived from PuzzleAndy's CC0 watercolour scans
// (see ASSETS.md). Scanned paper is irregular in a way value noise is not:
// noise is uniformly random at every point, whereas paper has fibre, direction
// and clumping. Using both — this for the tooth, noise for the broad mottle —
// costs one 512px texture and is the difference between "grainy" and "printed".
uniform sampler2D paper : filter_linear, repeat_enable;
uniform vec2 paper_size = vec2(512.0);

uniform vec2 world_off = vec2(0.0);   // camera position, for world-anchored grain
uniform float grain_coarse = 0.042;
uniform float grain_fine = 0.020;
uniform float contrast = 0.05;
uniform float vignette = 0.08;
uniform float warmth = 0.040;

// The plates are bright pictures with dark accents, not dark pictures. Measured
// over their whole frame they sit at mean luminance 0.52 with only ~4% of
// pixels below 0.20; an earlier build of this scene measured 0.41-0.48 with
// 11-28% below 0.20. Exposure and a black floor are what close that gap: the
// plates' shadows stop at a value and stay there rather than rolling off.
uniform float exposure = 0.99;
uniform float black_lift = 0.045;
uniform float saturation = 0.94;

float hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

// Value noise with smooth interpolation.
float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	vec2 u = f * f * (3.0 - 2.0 * f);
	return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
	           mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

float fbm(vec2 p) {
	float v = 0.0;
	float a = 0.5;
	for (int i = 0; i < 4; i++) {
		v += a * vnoise(p);
		p *= 2.03;
		a *= 0.5;
	}
	return v;
}

void fragment() {
	vec2 px = FRAGCOORD.xy;
	vec3 c = texture(screen_tex, SCREEN_UV).rgb;

	// --- grain -------------------------------------------------------------
	// coarse: anchored to the world, so it is texture lying on the scene
	float coarse = fbm((px + world_off) * 0.021) - 0.5;
	// Sampled mirrored, so a 512px tile covers any viewport without a seam.
	vec2 puv = px / paper_size;
	puv = abs(fract(puv * 0.5) * 2.0 - 1.0);
	float fine = texture(paper, puv).r - 0.5;
	// a second, larger tap anchored to the world, so the ground carries
	// paper texture that stays put as the camera moves
	vec2 wuv = (px + world_off) / (paper_size * 2.6);
	wuv = abs(fract(wuv * 0.5) * 2.0 - 1.0);
	float tooth = texture(paper, wuv).r - 0.5;

	// Grain bites hardest in the midtones and lets highlights stay clean,
	// which is how paper actually behaves.
	float luma = dot(c, vec3(0.2126, 0.7152, 0.0722));
	float bite = 1.0 - abs(luma - 0.48) * 1.25;
	bite = clamp(bite, 0.22, 1.0);

	c += coarse * grain_coarse * bite;
	c += (fine * 1.30 + tooth * 0.80) * grain_fine * bite;

	// --- grade -------------------------------------------------------------
	// S-curve: the references carry real dark mass; flat vector does not.
	luma = dot(c, vec3(0.2126, 0.7152, 0.0722));
	float s = luma + contrast * (luma - 0.5) * (1.0 - abs(luma - 0.5) * 1.1);
	c *= (luma > 0.001) ? (s / luma) : 1.0;

	// warm the light, cool and deepen the shade — never a neutral grey shadow
	float lift = smoothstep(0.32, 1.0, luma);
	c += vec3(warmth, warmth * 0.62, -warmth * 0.30) * lift;
	c -= vec3(-0.012, 0.004, 0.022) * (1.0 - lift) * 0.85;

	// saturation, pulled very slightly down: the plates are calmer than raw
	// vector fills are
	float g = dot(c, vec3(0.2126, 0.7152, 0.0722));
	c = mix(vec3(g), c, saturation);

	// --- key ---------------------------------------------------------------
	c *= exposure;
	c = vec3(black_lift) + c * (1.0 - black_lift);

	// --- vignette ----------------------------------------------------------
	vec2 q = SCREEN_UV - 0.5;
	float v = 1.0 - vignette * dot(q, q) * 1.9;
	c *= v;

	COLOR = vec4(clamp(c, vec3(0.0), vec3(1.0)), 1.0);
}
