shader_type canvas_item;

// An ink contour on every prop.
//
// Put the demo's café terrace beside references/02_cafe_street.png at 1:1 and
// the single loudest difference is not colour, shading or texture: it is that
// every object in the plate is drawn with a dark line around it and nothing in
// the spike was. Chairs, planters, railings, benches and people in the
// references all carry a contour; without one, flat shapes sit on the ground
// instead of on it.
//
// The line is drawn *inside* the silhouette, by darkening pixels whose
// neighbourhood is about to run out of alpha. Drawing it outside would need
// padding in every sprite that may not be there. Inside also matches how the
// plates read: the contour eats into the shape rather than haloing it.
//
// Width is given in texels and set per prop from its draw scale, so the line
// lands at a roughly constant thickness on screen whether it is on a lamppost
// drawn at 0.27 or a shopfront drawn at 0.5.

uniform vec4 ink : source_color = vec4(0.12, 0.10, 0.08, 1.0);
uniform float width = 4.0;       // texels
uniform float strength = 0.46;
// How much the line takes its hue from what it outlines. A pure black contour
// on everything reads as clip art; the plates' lines are dark versions of the
// local colour.
uniform float tint = 0.60;

void fragment() {
	vec4 mod_c = COLOR;
	vec4 c = texture(TEXTURE, UV);
	vec2 t = TEXTURE_PIXEL_SIZE * width;

	// Minimum alpha over a ring of taps. Where the shape is solid this is 1
	// and nothing happens; near an edge it falls away and the line appears.
	float mn = c.a;
	mn = min(mn, texture(TEXTURE, UV + vec2( t.x,  0.0)).a);
	mn = min(mn, texture(TEXTURE, UV + vec2(-t.x,  0.0)).a);
	mn = min(mn, texture(TEXTURE, UV + vec2( 0.0,  t.y)).a);
	mn = min(mn, texture(TEXTURE, UV + vec2( 0.0, -t.y)).a);
	float d = 0.7071;
	mn = min(mn, texture(TEXTURE, UV + vec2( t.x,  t.y) * d).a);
	mn = min(mn, texture(TEXTURE, UV + vec2(-t.x,  t.y) * d).a);
	mn = min(mn, texture(TEXTURE, UV + vec2( t.x, -t.y) * d).a);
	mn = min(mn, texture(TEXTURE, UV + vec2(-t.x, -t.y) * d).a);

	// Soften, and require a real boundary. Taking the raw difference inks a
	// thin feature — a character's leg, a chair rail — along its whole width,
	// because every pixel of it is near an edge.
	float edge = smoothstep(0.30, 0.92, clamp(c.a - mn, 0.0, 1.0)) * step(0.45, c.a);

	// the contour, darkened from the local colour rather than flat black
	vec3 line = mix(ink.rgb, c.rgb * 0.46, tint);
	vec3 col = mix(c.rgb, line, edge * strength);

	COLOR = vec4(col, c.a) * mod_c;
}
