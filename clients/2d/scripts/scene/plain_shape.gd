extends Node2D

## Plain drawing (`clients/2d/PRESENTATION.md` §6): what a role looks like when no pack binds it.
## Complete enough to play — a person, the player, a building, a prop — and nothing more.

## `person`, `player`, `block` or `prop`.
var kind := "prop"
## In pixels: a block's width and height; a person's or prop's radius in `x`.
var size := Vector2(10, 10)
var colour := Color(0.55, 0.5, 0.45)
## A person's facing, as a screen direction; drawn as a tick.
var facing := Vector2.DOWN:
	set(value):
		facing = value
		queue_redraw()


func _draw() -> void:
	match kind:
		"person", "player":
			var r := maxf(size.x, 7.0)
			if kind == "player":
				draw_arc(Vector2.ZERO, r + 5.0, 0.0, TAU, 32, Color(1.0, 0.79, 0.39), 3.0, true)
			draw_circle(Vector2(0, -r), r, colour)
			draw_arc(Vector2(0, -r), r, 0.0, TAU, 24, Color(0.2, 0.17, 0.14), 1.5, true)
			var dir := facing.normalized() if facing != Vector2.ZERO else Vector2.DOWN
			draw_line(Vector2(0, -r), Vector2(0, -r) + dir * (r + 5.0), Color(0.2, 0.17, 0.14), 2.0, true)
		"ring":
			# The warm ring under the player (the spike's Ring.gd): with a dozen people in a room, the
			# only thing that says which one is you.
			var outline := PackedVector2Array()
			for i in range(33):
				var a := TAU * i / 32.0
				outline.append(Vector2(cos(a) * size.x, sin(a) * size.y))
			draw_colored_polygon(outline, Color(1.0, 0.79, 0.39, 0.2))
			draw_polyline(outline, Color(1.0, 0.79, 0.39, 0.6), 2.4, true)
		"block":
			var box := Rect2(Vector2(-size.x * 0.5, -size.y), size)
			draw_rect(box, Color(colour, 0.85), true)
			draw_rect(box, Color(0.25, 0.22, 0.18), false, 2.0)
			draw_rect(Rect2(Vector2(-6, -18), Vector2(12, 18)), Color(0.3, 0.24, 0.18), true)
		_:
			draw_circle(Vector2.ZERO, maxf(size.x, 4.0), Color(colour, 0.8))
