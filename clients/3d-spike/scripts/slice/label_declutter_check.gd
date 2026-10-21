extends SceneTree

## The overhead names never cover one another (`SliceLabelDeclutter`). Prints `PASS <case>` or
## `FAIL <case>: <why>` per case and exits 1 on any failure.
##
##   godot --headless --path clients/3d-spike --script res://scripts/slice/label_declutter_check.gd
##
## Cases: the greedy pass on rectangles (a pile of five equal captions, a crowd past the lift bound),
## then real figures -- two and three people one behind another along the view, their names built by
## `SliceFigures.overhead_label` as the client builds them, settled through a real camera, and their
## drawn rectangles measured back on screen.

var _failed := false


func _init() -> void:
	_pile()
	_crowd()
	await process_frame
	_figures("two along the view", [Vector3(0, 0, -3), Vector3(0.15, 0, -5)])
	_figures("three in a row, Zz on the middle one", [Vector3(0, 0, -3), Vector3(0.2, 0, -4.5),
		Vector3(-0.1, 0, -6)], 1)
	_figures("two apart stay put", [Vector3(-1.5, 0, -4), Vector3(1.5, 0, -4)], -1, true)
	quit(1 if _failed else 0)


func _check(ok: bool, name: String, why: String) -> void:
	if ok:
		print("PASS ", name)
	else:
		_failed = true
		print("FAIL %s: %s" % [name, why])


## Five captions on one spot: the nearest stays, the next ones stack above it, the rest fade.
func _pile() -> void:
	var items: Array = []
	for i in 5:
		items.append({"rect": Rect2(100, 300, 120, 20), "priority": float(5 - i)})
	var placed := SliceLabelDeclutter.place(items)
	var rects := _placed_rects(items, placed)
	_check(placed[4]["lift"] == 0.0 and placed[4]["shown"], "the nearest caption stays", str(placed[4]))
	_check(_disjoint(rects), "a pile is stacked without overlap", str(rects))
	var shown := placed.filter(func(p: Dictionary) -> bool: return p["shown"]).size()
	_check(shown == 3, "a pile shows the lift bound's worth (3 of 5)", "%d shown" % shown)


## Captions side by side that do not touch are never moved.
func _crowd() -> void:
	var items: Array = []
	for i in 6:
		items.append({"rect": Rect2(10 + i * 130, 300, 120, 20), "priority": float(i)})
	var placed := SliceLabelDeclutter.place(items)
	_check(placed.all(func(p: Dictionary) -> bool: return p["shown"] and p["lift"] == 0.0),
		"captions that do not touch stay where they belong", str(placed))


static func _placed_rects(items: Array, placed: Array) -> Array:
	var out: Array = []
	for i in items.size():
		if placed[i]["shown"]:
			var r: Rect2 = items[i]["rect"]
			out.append(Rect2(r.position - Vector2(0, placed[i]["lift"]), r.size))
	return out


static func _disjoint(rects: Array) -> bool:
	for i in rects.size():
		for j in range(i + 1, rects.size()):
			if (rects[i] as Rect2).intersects(rects[j]):
				return false
	return true


## People standing at `at`, a camera at eye height looking down -Z; after settling, every shown
## caption's drawn rectangle is apart from every other's. `asleep` gives that person a Zz.
func _figures(name: String, at: Array, asleep := -1, unmoved := false) -> void:
	var holder := Node3D.new()
	root.add_child(holder)
	var camera := Camera3D.new()
	camera.fov = CameraRig.FOV
	holder.add_child(camera)
	camera.position = Vector3(0, 1.7, 0)
	camera.make_current()
	var figures: Array = []
	for i in at.size():
		var figure := Node3D.new()
		holder.add_child(figure)
		figure.position = at[i]
		var label := SliceFigures.overhead_label("Person Number %d" % i, 2.05, 48)
		figure.add_child(label)
		if i == asleep:
			var zz := SliceFigures.overhead_label("Zz", 2.05, 56)
			zz.offset = Vector2(0, 56 * 1.2)
			figure.add_child(zz)
		figures.append(figure)
	for _frame in 60:
		SliceLabelDeclutter.settle(figures, camera, 0.5)
	var rects: Array = []
	var moved := false
	for figure in figures:
		for child in figure.get_children():
			var l := child as Label3D
			if l.transparency > 0.5:
				continue
			rects.append(_drawn(l, camera))
			moved = moved or absf(l.offset.y - (l.get_meta("base_offset") as Vector2).y) > 0.01
	var per_figure := figures.map(func(f: Node3D) -> int: return f.get_child_count())
	_check(_disjoint(_merge_by_figure(figures, camera)), name, "rects %s (labels per figure %s)" % [rects, per_figure])
	if unmoved:
		_check(not moved, name + ": nothing moved", "a caption was lifted")
	holder.queue_free()


## Each figure's shown captions as one rectangle, as drawn now.
static func _merge_by_figure(figures: Array, camera: Camera3D) -> Array:
	var out: Array = []
	for figure in figures:
		var merged := Rect2()
		var any := false
		for child in (figure as Node3D).get_children():
			var l := child as Label3D
			if l.transparency > 0.5:
				continue
			var r := _drawn(l, camera)
			merged = r if not any else merged.merge(r)
			any = true
		if any:
			out.append(merged)
	return out


## A caption's rectangle on screen as drawn: its anchor projected, moved by its current offset.
static func _drawn(l: Label3D, camera: Camera3D) -> Rect2:
	var viewport_h := camera.get_viewport().get_visible_rect().size.y
	var px := l.pixel_size * viewport_h / (2.0 * tan(deg_to_rad(camera.fov) * 0.5))
	var font := l.font if l.font != null else ThemeDB.fallback_font
	var size := (font.get_multiline_string_size(l.text, HORIZONTAL_ALIGNMENT_CENTER, -1, l.font_size)
		+ Vector2.ONE * 2.0 * l.outline_size) * px
	var centre := camera.unproject_position(l.global_position) - Vector2(0, l.offset.y * px)
	return Rect2(centre - size * 0.5, size)
