extends SceneTree

## The shared span function against hand-computed cases (16f-b C1; `pr-16f-presentation-pack-3d.md` §7):
## no doorway, one, two overlapping, one at a corner, one outside the span; and which doorways open in
## a wall. Prints one row per case and `all span checks pass`, or `<n> SPAN CHECKS FAILED` and exits 1.
##
##   godot --headless --path clients/3d-spike --script res://scripts/places/span_check.gd

const Spans := preload("res://scripts/places/spans.gd")

var _fails := 0


func _init() -> void:
	# a 10 m wall, gaps 2 m wide
	_expect("no doorway", Spans.split(0.0, 10.0, [], 2.0), [[0.0, 10.0]])
	_expect("one doorway", Spans.split(0.0, 10.0, [5.0], 2.0), [[0.0, 4.0], [6.0, 10.0]])
	_expect("two overlapping", Spans.split(0.0, 10.0, [5.0, 4.0], 2.0), [[0.0, 3.0], [6.0, 10.0]])
	_expect("one at a corner", Spans.split(0.0, 10.0, [0.5], 2.0), [[1.5, 10.0]])
	_expect("one outside the span", Spans.split(0.0, 10.0, [12.0], 2.0), [[0.0, 10.0]])
	_expect("two apart", Spans.split(0.0, 10.0, [2.0, 8.0], 2.0), [[0.0, 1.0], [3.0, 7.0], [9.0, 10.0]])
	_expect("one filling the span", Spans.split(0.0, 1.0, [0.5], 2.0), [])
	# which doorways open in a wall along x at y = 0, within 0.5 of its line
	var doors := [Vector2(5.0, 0.3), Vector2(5.0, 2.0), Vector2(7.0, -0.5), Vector2(12.0, 0.0)]
	_expect("openings", Spans.openings(doors, true, 0.0, 0.5), [5.0, 7.0, 12.0])
	_expect("openings within the span", Spans.openings(doors, true, 0.0, 0.5, Vector2(0.0, 10.0)), [5.0, 7.0])
	# a wall along y at x = 3: a doorway's position is its y
	_expect("openings along y", Spans.openings([Vector2(3.2, 4.0), Vector2(1.0, 4.0)], false, 3.0, 0.5), [4.0])
	print("all span checks pass" if _fails == 0 else "%d SPAN CHECKS FAILED" % _fails)
	quit(0 if _fails == 0 else 1)


func _expect(name: String, got: Array, want: Array) -> void:
	var ok := got.size() == want.size()
	if ok:
		for i in got.size():
			ok = ok and _same(got[i], want[i])
	print("%s %s: got %s, expected %s" % ["ok  " if ok else "FAIL", name, got, want])
	if not ok:
		_fails += 1


static func _same(a: Variant, b: Variant) -> bool:
	if typeof(a) == TYPE_ARRAY and typeof(b) == TYPE_ARRAY:
		if a.size() != b.size():
			return false
		for i in a.size():
			if not _same(a[i], b[i]):
				return false
		return true
	return absf(float(a) - float(b)) < 0.000001
