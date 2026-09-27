extends Node2D

## The warm ring under the player.
##
## The procedural player sprite draws this into its own SVG. Generated sprites
## are pictures of a person and nothing else, so for those the ring is drawn
## here. It is not decoration: with eleven people on the square, it is the only
## thing that says which one is you.

const COL := Color(1.0, 0.79, 0.39)


func _ready() -> void:
	z_index = -1


func _draw() -> void:
	for spec in [[26.0, 10.0, 0.16], [21.0, 8.0, 0.22]]:
		var pts := PackedVector2Array()
		for i in range(33):
			var a := TAU * i / 32.0
			pts.append(Vector2(cos(a) * spec[0], sin(a) * spec[1]))
		draw_colored_polygon(pts, Color(COL, spec[2]))
	var outline := PackedVector2Array()
	for i in range(33):
		var a := TAU * i / 32.0
		outline.append(Vector2(cos(a) * 26.0, sin(a) * 10.0))
	draw_polyline(outline, Color(COL, 0.55), 2.4, true)
