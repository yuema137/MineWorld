extends RefCounted

## Plan metres to screen pixels and back: the one projection the client draws through.
##
## "Plan" is Godot's 2D frame in metres as `MineWorldSpace.to_2d` gives it — `+x` east, `+y` south,
## because Godot's 2D `+y` is down the screen. Every world position reaches this file as plan metres
## and leaves it as pixels; nothing here is sent to the world, so a projection can change what the
## player sees and never what the client asks for (step-13 I-5).
##
## `isometric` is the 2:1 projection of `ARC-14`'s `town` style (the spike's `Iso.gd`, stated in
## metres): east runs down-right, north runs up-right, and a metre of height is `ppm` pixels up.
## `plan` is north-up at `ppm` pixels per metre, the projection of plain drawing.

const ISOMETRIC := "isometric"
const PLAN := "plan"

var kind := ISOMETRIC
## Pixels per metre: of height in `isometric`, of ground in `plan`.
var ppm := 32.0


func _init(projection_kind: String = ISOMETRIC, pixels_per_metre: float = 32.0) -> void:
	kind = projection_kind if projection_kind == PLAN else ISOMETRIC
	ppm = pixels_per_metre


## A ground point, plan metres → screen pixels.
func to_screen(plan: Vector2) -> Vector2:
	if kind == PLAN:
		return plan * ppm
	return Vector2((plan.x - plan.y) * ppm, (plan.x + plan.y) * ppm * 0.5)


## A screen pixel on the ground → plan metres. The exact inverse of [method to_screen].
func to_plan(screen: Vector2) -> Vector2:
	if kind == PLAN:
		return screen / ppm
	var a := screen.x / ppm
	var b := screen.y / (ppm * 0.5)
	return Vector2((a + b) * 0.5, (b - a) * 0.5)


## How many pixels up the screen a height of `metres` is drawn.
func height_px(metres: float) -> float:
	return metres * ppm


## A direction on the screen (from the keyboard) as a unit plan direction, so "up" is up the screen
## whatever the projection does.
func screen_dir_to_plan(direction: Vector2) -> Vector2:
	if direction == Vector2.ZERO:
		return Vector2.ZERO
	return to_plan(direction).normalized()


## A plan direction as the screen direction it is drawn in.
func plan_dir_to_screen(direction: Vector2) -> Vector2:
	return to_screen(direction)
