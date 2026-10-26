## A wall split at its openings: the one function every wall built from disclosed geometry is cut
## with (16f-b, `docs/decisions/ARC-52.md`). The disclosed colliders (`SliceClientBodies`), the plain
## shells (`SliceShells`) and the stub boxes (`SliceBoxDepiction`) all call it, so a doorway's gap is
## computed once, the same way, wherever a wall is drawn or built.
##
## A wall is a span `[from, to]` along one axis. An opening is the position, along that axis, of a
## doorway that opens in the wall. Nothing here knows a place, a world or a tag; every number is the
## caller's.
class_name SliceSpans
extends RefCounted

## A piece no longer than this is dropped: a sliver left between two gaps is not a wall. The same
## unit as the span (metres or millimetres); 0.001 of either is below anything drawn or collided.
const MIN_PIECE := 0.001


## The pieces of the span `[from, to]` left standing when a gap `width` wide is cut, centred, at each
## of `centres`. Gaps may overlap each other or reach past either end. Returns the pieces in order,
## non-overlapping, each `[start, end]`.
static func split(from: float, to: float, centres: Array, width: float) -> Array:
	var cuts := []
	for c in centres:
		cuts.append([float(c) - width * 0.5, float(c) + width * 0.5])
	cuts.sort_custom(func(p, q): return p[0] < q[0])
	var pieces := []
	var at := from
	for cut in cuts:
		if cut[0] > at:
			pieces.append([at, minf(cut[0], to)])
		at = maxf(at, cut[1])
	if at < to:
		pieces.append([at, to])
	return pieces.filter(func(p): return p[1] - p[0] > MIN_PIECE)


## The positions, along a wall, of the doorways that open in it: every `doors` point (Vector2, the
## same frame and unit as the wall) whose distance across the wall's line is at most `tolerance`, and,
## when `within` is given, whose position along the wall lies inside it (`within.x` .. `within.y`).
## `along_x`: the wall runs along x (a doorway's position is its x, its distance across is in y).
static func openings(doors: Array, along_x: bool, line: float, tolerance: float,
		within := Vector2(-INF, INF)) -> Array:
	var out := []
	for d in doors:
		var p := d as Vector2
		var across := p.y if along_x else p.x
		var at := p.x if along_x else p.y
		if absf(across - line) <= tolerance and at >= within.x and at <= within.y:
			out.append(at)
	return out
