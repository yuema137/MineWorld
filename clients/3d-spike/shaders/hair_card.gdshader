// Hair cards with the opacity mask in its own texture.
//
// The CC0 hair groom ships diffuse and opacity as separate maps, and
// StandardMaterial3D can only take alpha from the albedo texture's own alpha
// channel. Rather than composite the two offline -- which would mean decoding
// and re-encoding PNGs with no image library on the machine -- the two are
// sampled separately here.
//
// Alpha-scissor rather than alpha-blend is deliberate: blended hair cards sort
// wrongly against the face in Forward+, and a hard edge with alpha-to-coverage
// reads better than cards visibly drawing in front of a nose.
shader_type spatial;
render_mode cull_disabled, depth_prepass_alpha, alpha_to_coverage, diffuse_burley;

uniform sampler2D tex_diffuse : source_color, filter_linear_mipmap_anisotropic;
// The strand atlas carries coverage in its ALPHA channel and a soft AO/shading
// term in RGB -- measured, not assumed: RGB averages 0.69 across the map while
// alpha runs the full 0..1 range. Sampling .r here instead of .a leaves every
// card fully opaque, which reads as black shards rather than hair.
uniform sampler2D tex_opacity : hint_default_white, filter_linear_mipmap_anisotropic;
uniform vec3 tint : source_color = vec3(0.42, 0.30, 0.20);
uniform float cutoff : hint_range(0.0, 1.0) = 0.35;
uniform float roughness_v : hint_range(0.0, 1.0) = 0.55;
// The CC0 diffuse map is a near-black brown (mean luma 0.08), so it is used as
// strand-level variation over `tint` rather than as the colour itself.
uniform float diffuse_gain : hint_range(0.0, 8.0) = 4.0;
// Roots sit darker than tips on real hair; the cheapest way to stop a groom
// reading as one flat colour.
uniform float root_shade : hint_range(0.0, 1.0) = 0.6;

void fragment() {
	vec4 op = texture(tex_opacity, UV);
	float a = op.a;
	if (a < cutoff) {
		discard;
	}
	float strand = clamp(texture(tex_diffuse, UV).r * diffuse_gain, 0.0, 1.5);
	ALBEDO = tint * (0.55 + strand) * mix(root_shade, 1.0, clamp(UV.y, 0.0, 1.0))
		* mix(0.75, 1.0, op.r);
	ALPHA = a;
	ALPHA_SCISSOR_THRESHOLD = cutoff;
	ROUGHNESS = roughness_v;
	SPECULAR = 0.35;
	METALLIC = 0.0;
}
