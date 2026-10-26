// Lake surface. Two scrolling noise normals and a depth-faded shoreline.
// Reflection comes from the sky cubemap for free because roughness is low.
//
// FAR_WATER (RL-e0 SD-FF-5; defined by `SliceFarFieldView`, which prepends `#define FAR_WATER`):
// the slice's distant bay. Unshaded, so nothing of the town is reflected; the sky's horizon colour
// is mixed in by a Schlick Fresnel capped at `fresnel_cap` (0.6): a wind-roughened sea near the
// horizon is darker than the sky behind it, which is why the real sea horizon is a visible line.
// The promenade's lake (`town.gd`) does not define it and is unchanged.
//
// NEAR_LAKE (S33-d, DEP-78; defined by `DepictionWater`, which prepends `#define NEAR_LAKE`): a depiction
// file's near water. It reads the depth texture, so it draws in the transparent pass: `depth_fade_m`
// blends shallow into deep over that depth of water (0 = off), `shore_foam` lightens a thin band where
// the water is shallower than 5 cm (S33-d sec.5.3; 0 = off), and `lily_mask` over `lily_rect` (world x, z,
// width, depth) draws lily pads as a masked decal. Every uniform's default draws today's water. Neither the
// far field nor the promenade defines it, so their water is unchanged by construction (I-d-2).
shader_type spatial;
#if defined(FAR_WATER)
render_mode unshaded, cull_back;
#elif defined(NEAR_LAKE)
render_mode cull_disabled, specular_schlick_ggx, depth_draw_always;
#else
render_mode cull_disabled, specular_schlick_ggx;
#endif

uniform vec3 shallow : source_color = vec3(0.19, 0.34, 0.36);
uniform vec3 deep : source_color = vec3(0.06, 0.13, 0.20);
uniform float wave_scale = 22.0;
uniform float wave_strength = 0.13;
// FAR_WATER only: the sky's colour at the horizon (linear) and the Fresnel ceiling.
uniform vec3 sky_horizon = vec3(0.6, 0.65, 0.72);
uniform float fresnel_cap = 0.6;
#ifdef NEAR_LAKE
uniform sampler2D depth_tex : hint_depth_texture, filter_nearest;
uniform float depth_fade_m = 0.0;
uniform float shore_foam = 0.0;
uniform sampler2D lily_mask : hint_default_black, filter_linear;
uniform vec4 lily_rect = vec4(0.0);
uniform vec3 lily_colour : source_color = vec3(0.17, 0.31, 0.11);
varying vec3 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}
#endif

float hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float vnoise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(hash(i), hash(i + vec2(1, 0)), f.x),
	           mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), f.x), f.y);
}

void fragment() {
	vec2 w = (NODE_POSITION_WORLD.xz + (INV_VIEW_MATRIX * vec4(VERTEX, 1.0)).xz) / wave_scale;
	float t = TIME * 0.05;
	float n1 = vnoise(w * 1.0 + vec2(t, t * 0.6));
	float n2 = vnoise(w * 2.7 - vec2(t * 0.8, t * 1.3));
	float n3 = vnoise(w * 6.1 + vec2(t * 1.7, -t));
	float e = 0.06;
	float dx = vnoise(w + vec2(e, 0.0)) - n1 + (vnoise(w * 2.7 + vec2(e, 0.0)) - n2) * 0.5;
	float dz = vnoise(w + vec2(0.0, e)) - n1 + (vnoise(w * 2.7 + vec2(0.0, e)) - n2) * 0.5;

#ifdef FAR_WATER
	// The noise slope as a world normal, then the view's grazing angle on it.
	vec3 n_world = normalize(vec3(-dx * wave_strength * 8.0, 1.0, -dz * wave_strength * 8.0));
	vec3 n_view = normalize((VIEW_MATRIX * vec4(n_world, 0.0)).xyz);
	float cos_v = clamp(dot(n_view, VIEW), 0.0, 1.0);
	float fresnel = min(0.02 + 0.98 * pow(1.0 - cos_v, 5.0), fresnel_cap);
	float far = clamp(length(VERTEX) / 400.0, 0.0, 1.0);
	vec3 water = mix(shallow, deep, far * 0.8 + n3 * 0.1);
	ALBEDO = mix(water, sky_horizon, fresnel);
#else
	NORMAL_MAP = normalize(vec3(-dx, 1.0 - wave_strength, -dz) * 0.5 + 0.5);
	NORMAL_MAP_DEPTH = wave_strength * 8.0;

	float far = clamp(length(VERTEX) / 400.0, 0.0, 1.0);
	ALBEDO = mix(shallow, deep, far * 0.8 + n3 * 0.1);
	ROUGHNESS = mix(0.035, 0.085, far);
	METALLIC = 0.10;
	SPECULAR = 0.85;
#ifdef NEAR_LAKE
	// The water's depth along the view ray: the scene behind it, from the depth texture, less the surface.
	float raw = texture(depth_tex, SCREEN_UV).r;
	vec4 behind = INV_PROJECTION_MATRIX * vec4(SCREEN_UV * 2.0 - 1.0, raw, 1.0);
	float water_depth = max(-behind.z / behind.w + VERTEX.z, 0.0);
	float fade = depth_fade_m > 0.0 ? clamp(water_depth / depth_fade_m, 0.0, 1.0) : 1.0;
	ALBEDO = mix(shallow, ALBEDO, fade);
	ALPHA = depth_fade_m > 0.0 ? mix(0.45, 1.0, fade) : 1.0;
	if (shore_foam > 0.0) {
		float band = 1.0 - clamp(water_depth / 0.05, 0.0, 1.0);
		ALBEDO = mix(ALBEDO, vec3(0.85, 0.88, 0.86), band * shore_foam);
	}
	if (lily_rect.z > 0.0 && lily_rect.w > 0.0) {
		vec2 uv = (world_pos.xz - lily_rect.xy) / lily_rect.zw;
		if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0) {
			float pad = texture(lily_mask, uv).r;
			ALBEDO = mix(ALBEDO, lily_colour, pad);
			ROUGHNESS = mix(ROUGHNESS, 0.6, pad);
			ALPHA = max(ALPHA, pad);
		}
	}
#endif
#endif
}
