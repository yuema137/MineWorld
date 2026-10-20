// A far-field terrain band (`scripts/slice/far_field_view.gd`; RL-e0b SD-2, SD-4).
//
// PARALLAX PER COLUMN (SD-2). The band's root follows the camera in x and z, not in y. Each vertex
// carries P(d)/d in UV2.x, where P(d) is its proxy radius and d its real distance (UV2.y). Raising
// the eye by `rise` changes a point's elevation angle by -rise/d; at the proxy that is a drop of
// rise * P/d, so the vertex is lifted by rise * (1 - P/d) to keep the camera-relative angle true.
//
// TEXTURE (SD-4). The vertex colour's luminance is modulated by a two-octave value noise in the
// band's real coordinates (feature size F = 40 m, then 10 m; the first octave is three times finer
// across than up, so it reads as gullies). Its contrast fades with the Koschmieder transmittance
// that already sets the band's colour: a band 2 km away keeps ~93 % of it, one 60 km away almost
// none. The outline never moves: the texture changes colour only.
//
// ID MODE. The rendered check draws every far-field part in a flat identifying colour
// (`slice_far_field_ids.gd`); `id_mode` does that for a band while keeping its parallax.
shader_type spatial;
render_mode unshaded, fog_disabled, cull_back;

uniform float rise = 0.0;
uniform float eye0 = 1.8;
uniform float visibility_m = 80000.0;
uniform float texture_amp = 0.45;
uniform bool id_mode = false;
uniform vec3 id_colour : source_color = vec3(1.0, 0.0, 1.0);

const float KOSCHMIEDER = 2.996;
const float FEATURE_M = 40.0;

varying vec3 v_real;
varying float v_dist;

// A hash without trigonometry, so it stays stable at large coordinates on every renderer.
float hash13(vec3 p) {
	p = fract(p * 0.1031);
	p += dot(p, p.zyx + 31.32);
	return fract((p.x + p.y) * p.z);
}

float vnoise3(vec3 p) {
	vec3 i = floor(p);
	vec3 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float a = mix(hash13(i), hash13(i + vec3(1, 0, 0)), f.x);
	float b = mix(hash13(i + vec3(0, 1, 0)), hash13(i + vec3(1, 1, 0)), f.x);
	float c = mix(hash13(i + vec3(0, 0, 1)), hash13(i + vec3(1, 0, 1)), f.x);
	float d = mix(hash13(i + vec3(0, 1, 1)), hash13(i + vec3(1, 1, 1)), f.x);
	return mix(mix(a, b, f.y), mix(c, d, f.y), f.z);
}

// The vertex colour is linear; the texture's contrast is defined on displayed luminance (RL-e0b
// SD-4: L is the 8-bit luma FF-RELIEF measures), so it is applied in sRGB and converted back.
vec3 to_srgb(vec3 c) {
	c = max(c, vec3(0.0));
	return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), c));
}

vec3 to_linear(vec3 c) {
	c = max(c, vec3(0.0));
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), c));
}

void vertex() {
	float p = max(length(VERTEX.xz), 1.0);
	v_dist = UV2.y;
	// the point's real position relative to the street's eye: the proxy scaled out to d
	v_real = vec3(VERTEX.x, VERTEX.y - eye0, VERTEX.z) * (UV2.y / p);
	VERTEX.y += rise * (1.0 - UV2.x);
}

void fragment() {
	if (id_mode) {
		ALBEDO = id_colour;
	} else {
		float tau = exp(-KOSCHMIEDER * v_dist / visibility_m);
		vec3 q = v_real / FEATURE_M;
		float n = 0.65 * vnoise3(vec3(q.x * 3.0, q.y, q.z * 3.0)) + 0.35 * vnoise3(q * 4.0);
		ALBEDO = to_linear(to_srgb(COLOR.rgb) * (1.0 + texture_amp * tau * (n - 0.5)));
	}
}
