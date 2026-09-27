extends RefCounted

## 2:1 isometric projection shared by everything in the spike.
##
## One world unit is 2 metres. A world unit measures TILE_W across the screen
## and TILE_H down it; one unit of height is TILE_W / 2 pixels up the screen.
## Fixing this here, once, is what keeps trees, shopfronts and people agreeing
## about how big a person is.

const TILE_W := 128.0
const TILE_H := 64.0
const METRES_PER_UNIT := 2.0

## Pixels per metre, so art scales can be derived rather than guessed.
const PX_PER_M_X := TILE_W / 2.0 / METRES_PER_UNIT   # 32
const PX_PER_M_Z := TILE_W / 2.0 / METRES_PER_UNIT   # 32, vertical

static func to_screen(w: Vector2) -> Vector2:
	return Vector2((w.x - w.y) * TILE_W * 0.5, (w.x + w.y) * TILE_H * 0.5)

static func to_world(s: Vector2) -> Vector2:
	var a := s.x / (TILE_W * 0.5)
	var b := s.y / (TILE_H * 0.5)
	return Vector2((b + a) * 0.5, (b - a) * 0.5)

## Screen-space input turned into a world-space direction, so "up" on the
## keyboard is "up" on the screen whatever the projection is doing.
static func screen_dir_to_world(d: Vector2) -> Vector2:
	if d == Vector2.ZERO:
		return Vector2.ZERO
	return to_world(d).normalized()
