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

uniform vec2 world_off = vec2(0.0);   // camera position, for world-anchored grain
uniform float grain_coarse = 0.115;
uniform float grain_fine = 0.055;
uniform float contrast = 0.22;
uniform float vignette = 0.30;
uniform float warmth = 0.055;
uniform float saturation = 0.96;

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
	// coarse: anchored to the world, so it is texture on the scene
	float coarse = fbm((px + world_off) * 0.021) - 0.5;
	// fine: anchored to the screen, so it is tooth in the paper
	float fine = vnoise(px * 0.85) - 0.5;
	float tooth = vnoise(px * 2.7) - 0.5;

	// Grain bites hardest in the midtones and lets highlights stay clean,
	// which is how paper actually behaves.
	float luma = dot(c, vec3(0.2126, 0.7152, 0.0722));
	float bite = 1.0 - abs(luma - 0.48) * 1.25;
	bite = clamp(bite, 0.22, 1.0);

	c += coarse * grain_coarse * bite;
	c += (fine * 0.65 + tooth * 0.35) * grain_fine * bite;

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

	// --- vignette ----------------------------------------------------------
	vec2 q = SCREEN_UV - 0.5;
	float v = 1.0 - vignette * dot(q, q) * 1.9;
	c *= v;

	COLOR = vec4(clamp(c, vec3(0.0), vec3(1.0)), 1.0);
}
