## Overhead names that never cover one another, for both reference clients (promo v2: names
## overlapped when people stood close, in 3D behind the counter and in 2D in the yard).
##
## The standard answer of map renderers to the same problem (MapLibre / Mapbox GL symbol placement,
## OpenLayers' `declutter`): place labels greedily in priority order, each against the screen
## rectangles of the labels already placed. The nearest person's name is placed first and never
## moves; a name that would overlap is lifted up the screen, above the one it hits, by at most
## MAX_LIFT_LINES of its own height; one that still overlaps fades out. Neither Godot's Label3D nor
## its Control labels do collision placement, and the greedy pass is a few dozen lines over Rect2, so
## it is written here once and both clients project their own captions into it.
##
## Presentation only. Nothing here reads or decides world state: it moves the drawn caption, never the
## person, and it is deterministic -- the same frame places the same way. It knows no name, no world
## and no pixel size: the caller supplies screen rectangles and priorities.
class_name MineWorldLabelPlacement
extends RefCounted

## How far a name may be lifted above where it belongs, in its own line heights; past it, it fades.
const MAX_LIFT_LINES := 2.5
## Clear space kept between two placed captions, screen pixels.
const GAP_PX := 3.0
## How fast a caption glides to its placed offset and fades in or out, per second: a lift is shown
## moving, never a jump, so a crowd passing does not flicker.
const SETTLE_RATE := 10.0
## Bounded work per label: every overlap found lifts it past one placed rectangle.
const MAX_TRIES := 12


## The placement of `items`, each `{rect: Rect2 (screen pixels, y down, where it belongs),
## priority: float (lower is placed first)}`. Returns one `{lift: float (pixels up), shown: bool}` per
## item, in the input order. Ties keep the input order, so the answer is a function of the input.
static func place(items: Array) -> Array:
	var order := range(items.size())
	order.sort_custom(func(a: int, b: int) -> bool:
		var pa: float = items[a]["priority"]
		var pb: float = items[b]["priority"]
		return pa < pb or (pa == pb and a < b))
	var out: Array = []
	out.resize(items.size())
	var placed: Array[Rect2] = []
	for i in order:
		var rest: Rect2 = items[i]["rect"]
		var lift := 0.0
		var shown := true
		for _try in MAX_TRIES:
			var at := Rect2(rest.position - Vector2(0, lift), rest.size)
			var hit: Variant = _first_overlap(at, placed)
			if hit == null:
				break
			lift += at.end.y - (hit as Rect2).position.y + GAP_PX
			if lift > MAX_LIFT_LINES * rest.size.y:
				shown = false
				break
		if shown and _first_overlap(Rect2(rest.position - Vector2(0, lift), rest.size), placed) != null:
			shown = false
		if shown:
			placed.append(Rect2(rest.position - Vector2(0, lift), rest.size))
		out[i] = {"lift": lift if shown else 0.0, "shown": shown}
	return out


## The share of the way a caption moves toward its placed state this frame.
static func step(delta: float) -> float:
	return 1.0 - exp(-SETTLE_RATE * maxf(delta, 0.0))


static func _first_overlap(rect: Rect2, placed: Array[Rect2]) -> Variant:
	var grown := rect.grow(GAP_PX * 0.5)
	for other in placed:
		if grown.intersects(other.grow(GAP_PX * 0.5)):
			return other
	return null
