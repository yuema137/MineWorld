// Hair cards: the strand atlas's alpha is coverage, its RGB strand shading.
//
// The atlas is `tools/hair_atlas.py`'s: eight columns of fine strands with
// transparent gaps, root at the bottom of the image. Blender writes the cards
// with v = 0 at the root and glTF flips v, so here the ROOT IS AT UV.y = 1
// and the tip at UV.y = 0. (The previous atlas and this shader's previous
// version had it the other way; both changed together.)
//
// Alpha-scissor rather than alpha-blend is deliberate: blended hair cards sort
// wrongly against the face in Forward+. A depth prepass gives clean edges.
shader_type spatial;
render_mode cull_disabled, depth_prepass_alpha, diffuse_burley;

uniform sampler2D tex_diffuse : source_color, filter_linear_mipmap_anisotropic;
uniform sampler2D tex_opacity : hint_default_white, filter_linear_mipmap_anisotropic;
uniform vec3 tint : source_color = vec3(0.42, 0.30, 0.20);
// A lighter, warmer tone the brightest strands lean toward: the reference's
// hair is mid-brown with caramel and auburn through the top and the loose
// strands, "not flat, not single-tone" (CHARACTER_IDENTITY.md sec.4).
uniform vec3 highlight : source_color = vec3(0.62, 0.42, 0.26);
uniform float cutoff : hint_range(0.0, 1.0) = 0.35;
uniform float roughness_v : hint_range(0.0, 1.0) = 0.55;
uniform float diffuse_gain : hint_range(0.0, 8.0) = 1.0;
// Roots sit darker than tips, which stops a groom reading as one flat colour
// and gives the parting and the gather their depth.
uniform float root_shade : hint_range(0.0, 1.0) = 0.6;
// Mip levels average strands with their gaps and alpha falls with distance,
// so a scissored card thins away. Coverage is rescaled by the mip level
// (Castaño's alpha-test mip fix, applied at sample time).
uniform float mip_scale : hint_range(0.0, 1.0) = 0.20;
uniform bool flip_back = true;

float mip_level(vec2 uv, vec2 size) {
	vec2 dx = dFdx(uv * size);
	vec2 dy = dFdy(uv * size);
	return max(0.0, 0.5 * log2(max(dot(dx, dx), dot(dy, dy))));
}

void fragment() {
	vec4 op = texture(tex_opacity, UV);
	float a = op.a * (1.0 + mip_level(UV, vec2(textureSize(tex_opacity, 0))) * mip_scale);
	if (a < cutoff) {
		discard;
	}
	// A card is seen from both sides; lit with its front normal from behind,
	// every card turned away from the camera read as a dark stiff strand.
	// Off for the brows: their cards lie on the skin and are only ever seen
	// from the side the bake set up, and flipped they caught the sky.
	if (!FRONT_FACING && flip_back) {
		NORMAL = -NORMAL;
	}
	float strand = clamp(texture(tex_diffuse, UV).r * diffuse_gain, 0.0, 1.0);
	float root = clamp(1.0 - UV.y, 0.0, 1.0);    // 0 at the root, 1 at the tip
	vec3 base = mix(tint, highlight, smoothstep(0.62, 1.0, strand));
	ALBEDO = base * (0.80 + 0.40 * strand) * mix(root_shade, 1.0, root);
	ALPHA = a;
	ALPHA_SCISSOR_THRESHOLD = cutoff;
	ROUGHNESS = roughness_v;
	SPECULAR = 0.35;
	METALLIC = 0.0;
}
