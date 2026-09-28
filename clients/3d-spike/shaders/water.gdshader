// Lake surface. Two scrolling noise normals and a depth-faded shoreline.
// Reflection comes from the sky cubemap for free because roughness is low.
shader_type spatial;
render_mode cull_disabled, specular_schlick_ggx;

uniform vec3 shallow : source_color = vec3(0.19, 0.34, 0.36);
uniform vec3 deep : source_color = vec3(0.06, 0.13, 0.20);
uniform float wave_scale = 22.0;
uniform float wave_strength = 0.13;

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

	NORMAL_MAP = normalize(vec3(-dx, 1.0 - wave_strength, -dz) * 0.5 + 0.5);
	NORMAL_MAP_DEPTH = wave_strength * 8.0;

	float far = clamp(length(VERTEX) / 400.0, 0.0, 1.0);
	ALBEDO = mix(shallow, deep, far * 0.8 + n3 * 0.1);
	ROUGHNESS = mix(0.035, 0.085, far);
	METALLIC = 0.10;
	SPECULAR = 0.85;
}
