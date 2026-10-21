// Glazed and quarry tile, computed per pixel: no texture, no screen read, no compute (S29-d §5.4;
// DECISIONS.md ARC-192). One shader, one material per pattern (StationTile.material):
//
//   0 zigzag   the depot's domes and lanterns: stepped chevron bands of blue, white, green and yellow
//              glazed tile on a dark grout line (NRHP 72000248 §7: "brightly colored tiles with zig-zag
//              patterns"); UV in tile units, from the mesh's own UV times the instance's `tiles`
//   1 emblem   the railway's cross in a circle on a blue square, on a quad's 0..1 UV
//   2 quarry   the floor's clay tile in two sizes and two colours (NRHP §7), in world metres
//   3 frieze   the wainscot's geometric field under a stepped band, in world metres (S29-e)
//
// Every tile's value varies by +/- 4 % from a hash of its cell, so no two tiles are identical. It writes
// colour and roughness only.
shader_type spatial;
render_mode blend_mix, depth_draw_opaque, cull_back, diffuse_burley, specular_schlick_ggx;

uniform int pattern = 0;
uniform float tile_m = 0.15;
uniform vec3 glaze_blue : source_color = vec3(0.16, 0.30, 0.50);
uniform vec3 glaze_green : source_color = vec3(0.20, 0.40, 0.32);
uniform vec3 glaze_yellow : source_color = vec3(0.78, 0.62, 0.26);
uniform vec3 glaze_white : source_color = vec3(0.86, 0.83, 0.76);
uniform vec3 grout : source_color = vec3(0.30, 0.28, 0.25);
uniform vec3 quarry_red : source_color = vec3(0.50, 0.27, 0.17);
uniform vec3 quarry_buff : source_color = vec3(0.62, 0.46, 0.30);
uniform float rough_glaze = 0.25;
uniform float rough_quarry = 0.8;
uniform float grout_w = 0.06;
// tiles around and down a dome or a lantern (the mesh's UV is 0..1 both ways)
instance uniform vec2 tiles = vec2(48.0, 12.0);

varying vec3 world_pos;
varying vec3 world_nrm;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_nrm = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

float hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// 1 inside a tile, 0 on its grout line; `t` in tile units
float in_tile(vec2 t, float w) {
	vec2 f = fract(t);
	vec2 d = min(f, 1.0 - f);
	return step(w * 0.5, min(d.x, d.y));
}

vec3 vary(vec3 c, vec2 cell) {
	return c * (0.96 + 0.08 * hash(cell));
}

vec3 band_colour(float k) {
	float i = mod(k, 5.0);
	if (i < 1.0) return glaze_blue;
	if (i < 2.0) return glaze_white;
	if (i < 3.0) return glaze_green;
	if (i < 4.0) return glaze_white;
	return glaze_yellow;
}

void fragment() {
	vec3 col;
	float rough = rough_glaze;
	float inside;
	if (pattern == 0) {
		vec2 t = UV * tiles;
		vec2 cell = floor(t);
		float zig = abs(mod(cell.x, 4.0) - 2.0);
		col = vary(band_colour(cell.y + zig), cell);
		inside = in_tile(t, grout_w);
	} else if (pattern == 1) {
		vec2 p = UV - 0.5;
		float r = length(p);
		bool bar = abs(p.x) < 0.07 || abs(p.y) < 0.07;
		col = (r < 0.40 && !(bar && r < 0.32)) ? glaze_white : glaze_blue;
		col = (r > 0.40 && r < 0.43) ? glaze_yellow : col;
		vec2 t = UV * 6.0;
		col = vary(col, floor(t));
		inside = in_tile(t, grout_w * 0.5);
	} else if (pattern == 2) {
		vec2 t = world_pos.xz / (tile_m * 2.0);
		vec2 cell = floor(t);
		bool small_cell = mod(cell.x + cell.y, 2.0) > 0.5;
		vec2 tt = small_cell ? t * 2.0 : t;
		vec2 c2 = floor(tt);
		col = vary(small_cell ? quarry_buff : quarry_red, c2);
		inside = in_tile(tt, grout_w * 1.5);
		rough = rough_quarry;
	} else {
		vec3 n = abs(world_nrm);
		float h = n.x > n.z ? world_pos.z : world_pos.x;
		vec2 t = vec2(h, world_pos.y) / tile_m;
		vec2 cell = floor(t);
		float diamond = abs(mod(cell.x, 6.0) - 3.0) + abs(mod(cell.y, 6.0) - 3.0);
		col = vary(diamond < 2.0 ? glaze_blue : (diamond < 3.0 ? glaze_yellow : glaze_green), cell);
		inside = in_tile(t, grout_w);
	}
	ALBEDO = mix(grout, col, inside);
	ROUGHNESS = mix(0.9, rough, inside);
	SPECULAR = 0.5;
}
