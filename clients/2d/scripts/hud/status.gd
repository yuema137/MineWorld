extends CanvasLayer

## The status line: who you are, where, the world's time, which world and revision, and whether the
## connection holds (step-13 §4.3 point 8). Everything it shows was told to this client; it states
## nothing of its own. Its text is the pack's wording (`ARC-70`).

const Words := preload("res://scripts/hud/words.gd")

var _who: Label
var _when: Label
var _hint: Label


func _ready() -> void:
	layer = 100
	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.99, 0.97, 0.92, 0.84)
	style.set_corner_radius_all(10)
	style.content_margin_left = 14
	style.content_margin_right = 14
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", style)
	panel.position = Vector2(20, 18)
	var lines := VBoxContainer.new()
	panel.add_child(lines)
	_who = _line(lines, 17)
	# Who and where: a name and a place the world disclosed, joined by punctuation — world content.
	MineWorldText.mark_world_text(_who)
	_when = _line(lines, 13)
	_hint = _line(lines, 12)
	_hint.text = Words.text("ui.hint")
	_money = _line(lines, 13)
	_money.visible = false
	_badge = _line(lines, 12)
	_badge.visible = false
	add_child(panel)


var _money: Label
## The money line as last shown ("" when hidden), for the harness's HUD report.
var money_text := ""


## The money line (S31-e §5.7): my balance in the world's money, my worth beside it when it is disclosed, and
## how many things I hold that have no reference value. Absent without a wallet.
func _show_money(observation: MineWorldObservation) -> void:
	var held := MineWorldTrade.money_of(observation)
	if held.is_empty():
		money_text = ""
	else:
		var worth := MineWorldTrade.wealth_of(observation)
		if worth.is_empty():
			money_text = Words.text("hud.money", {"money": Words.money(held["balance"])})
		else:
			money_text = Words.text("hud.money.wealth", {"money": Words.money(held["balance"]),
				"wealth": Words.money(worth["amount"])})
			if worth["unvalued"] != 0:
				money_text = Words.text("hud.money.unvalued", {"line": money_text, "count": worth["unvalued"]})
	_money.text = money_text
	_money.visible = money_text != ""
	# The board's badge (S31-e §5.9): new notices since the board was last opened, as the board counts them.
	var fresh: int = board.badge() if board != null else 0
	badge_text = Words.text("board.badge", {"count": fresh}) if fresh != 0 else ""
	_badge.text = badge_text
	_badge.visible = badge_text != ""


## The notice board, whose badge this line shows; set by the composition root.
var board
var badge_text := ""
var _badge: Label


func _line(parent: Node, size: int) -> Label:
	var label := Label.new()
	# Composed from a template and data: rendered again by `show_state` on a language change.
	label.auto_translate_mode = Node.AUTO_TRANSLATE_MODE_DISABLED
	label.add_theme_color_override("font_color", Color("4b3826"))
	label.add_theme_font_size_override("font_size", size)
	parent.add_child(label)
	return label


## The clock line the composition root hands in every frame (`MineWorldClockFormat.date_time` of the
## world-time module's displayed instant, TW-e), and whether the host paused the clock. "" until the
## first observation: the line then falls back to the observation's own instant.
var clock_line := ""
var paused := false
var _last := []


## Takes this frame's clock; renders again only when it changed.
func set_clock(line: String, is_paused: bool) -> void:
	if line == clock_line and is_paused == paused:
		return
	clock_line = line
	paused = is_paused
	if not _last.is_empty():
		show_state(_last[0], _last[1], _last[2])


## Refreshes the lines from the newest observation and the connection's state.
func show_state(observation: MineWorldObservation, link: Node, revision: Variant) -> void:
	_last = [observation, link, revision]
	_hint.text = Words.text("ui.hint")
	var state := Words.text("ui.state." + String(link.state))
	if observation != null:
		var me := observation.observer()
		var name := observation.display_name(me)
		var place := observation.entity(observation.place())
		var tags: Array = place.get("tags", [])
		_who.text = Words.text("ui.status.who", {"name": name if name != "" else me,
			"place": ", ".join(PackedStringArray(tags)) if not tags.is_empty() else observation.place()})
		var clock := clock_line if clock_line != "" else Words.day_time(observation.at())
		if paused:
			clock = clock + "  " + Words.text("hud.paused")
		_when.text = Words.text("ui.status.when", {"clock": clock,
			"revision": "—" if revision == null else str(int(revision)),
			"instance": link.instance.right(8), "state": state})
		_show_money(observation)
	else:
		_when.text = state
