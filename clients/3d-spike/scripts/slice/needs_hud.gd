class_name SliceNeedsHud
extends Node

## What the packs disclose about me, in the slice's HUD (S35 N-e, `pr-s35-ne-presentation.md` §4.4): how I feel,
## with a bar per need; that I am asleep; my next part of the day; and each lateness notice once.
##
## Words and numbers only. The rows are the shared reader's facts worded by the shared catalog, the same keys
## and arguments as the 2D panels (`clients/2d/scripts/hud/readers.gd`), so the two clients agree for the same
## observation (AC-S35-10). No threshold, no level arithmetic, no decision about sleep (I-Ne-1): a bar takes
## the disclosed level as its value and nothing else.

const BAR_X := 300.0
const BAR_Y_OFFSET := 7.0
const BAR_SIZE := Vector2(292, 8)
## The brightness of the night view while I am asleep (the view dims; no time skip, step §6.5).
const ASLEEP_BRIGHTNESS := 0.6

var _hud: ControlsHud = null
var _env: Environment = null
var _link: SliceLink = null
var _latest: MineWorldObservation = null
var _lines: Array[Label] = []
var _bars := {}   ## line index -> ProgressBar, for a row that carries a level
var _wake_hint: Label = null
var _told := {}   ## lateness event ids already toasted: each fact is told once


## `env` is the slice's environment: the view dims through its brightness while I sleep.
func attach(hud: ControlsHud, env: Environment) -> void:
	_hud = hud
	_env = env


## Renders every observation the link receives from now on.
func bind_link(link: SliceLink) -> void:
	_link = link
	link.observed.connect(refresh)


func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSLATION_CHANGED and _latest != null:
		_render(_latest)


## Shows the observation's rows, and toasts each lateness fact not told before.
func refresh(obs: MineWorldObservation) -> void:
	_latest = obs
	_render(obs)
	for late in MineWorldNeeds.lateness(obs):
		var id: String = late["id"]
		if id == "" or _told.has(id):
			continue
		_told[id] = true
		if _hud != null:
			_hud.toast(_notice(late))


## The rows as the HUD shows them: `[{text, level}]`, `level` is -1 for a row without a bar. Static, so the
## parity probe builds the same rows without a HUD.
static func entries(obs: MineWorldObservation, place_of: Callable) -> Array:
	var out: Array = []
	var me := obs.observer()
	var records: Variant = obs.entity(me).get("components", [])
	if typeof(records) != TYPE_ARRAY:
		return out
	for record in records:
		if typeof(record) != TYPE_DICTIONARY or typeof(record.get("payload")) != TYPE_DICTIONARY:
			continue
		match String(record.get("component_type", "")):
			"needs":
				for need in MineWorldNeeds.own_needs(obs):
					out.append({"text": MineWorldText.text("ui.row.need", {
						"need": MineWorldText.code("word", need["need"]),
						"word": MineWorldText.code("word", need["word"])}), "level": need["level"]})
			"asleep":
				var asleep := MineWorldNeeds.asleep_of(obs, me)
				if asleep.is_empty():
					continue
				out.append({"text": _asleep_text(asleep), "level": -1})
			"agenda":
				var next := MineWorldNeeds.next_part(obs)
				if next.is_empty():
					continue
				var key := "ui.row.agenda-next.punctual" if next["punctual"] else "ui.row.agenda-next"
				out.append({"text": MineWorldText.text(key, {"label": next["label"],
					"place": place_of.call(obs, next["place"]),
					"from": MineWorldClockFormat.clock(next["from"])}), "level": -1})
	return out


## The rows' text only: `entries` without the levels.
static func rows(obs: MineWorldObservation, place_of: Callable) -> PackedStringArray:
	var out := PackedStringArray()
	for entry in entries(obs, place_of):
		out.append(entry["text"])
	return out


## A place as the 2D client words it: its tags, else its id (`readers.gd` `place`, from this observation).
static func place_tags(obs: MineWorldObservation, id: String) -> String:
	var tags: Variant = obs.entity(id).get("tags", [])
	if typeof(tags) == TYPE_ARRAY and not (tags as Array).is_empty():
		return ", ".join(PackedStringArray(tags))
	return id


static func _asleep_text(asleep: Dictionary) -> String:
	var since := MineWorldClockFormat.clock(asleep["since"])
	if asleep["alarm"] == null:
		return MineWorldText.text("ui.row.asleep.no-alarm", {"since": since})
	return MineWorldText.text("ui.row.asleep", {"since": since, "alarm": MineWorldClockFormat.clock(asleep["alarm"])})


func _render(obs: MineWorldObservation) -> void:
	if _hud == null:
		return
	var rows := entries(obs, Callable(SliceNeedsHud, "place_tags"))
	while _lines.size() < rows.size():
		_lines.append(_hud.add_line(""))
	for i in _lines.size():
		_lines[i].visible = i < rows.size()
		if i < rows.size():
			_lines[i].text = rows[i]["text"]
			_show_bar(i, rows[i]["level"], _lines[i])
		elif _bars.has(i):
			(_bars[i] as ProgressBar).visible = false
	_show_asleep(obs)


## While I sleep: the view dims, and the way to wake up is a line of its own. Wake-up is a menu offer once the
## B self menu (PR #250) is in; until then the line says that moving wakes me (F-Ne-7; the server's `acted`).
func _show_asleep(obs: MineWorldObservation) -> void:
	var asleep := not MineWorldNeeds.asleep_of(obs, obs.observer()).is_empty()
	if _env != null:
		_env.adjustment_brightness = ASLEEP_BRIGHTNESS if asleep else 1.0
	if asleep and _wake_hint == null:
		_wake_hint = _hud.add_line("")
	if _wake_hint != null:
		_wake_hint.visible = asleep
		if asleep:
			_wake_hint.text = MineWorldText.text("hud.asleep.move-to-wake")


## A bar beside a need's row: the disclosed level on the pack's own scale (`MineWorldNeeds.LEVEL_MAX`).
func _show_bar(index: int, level: int, line: Label) -> void:
	if level < 0:
		if _bars.has(index):
			(_bars[index] as ProgressBar).visible = false
		return
	if not _bars.has(index):
		var bar := ProgressBar.new()
		bar.min_value = 0
		bar.max_value = MineWorldNeeds.LEVEL_MAX
		bar.step = 200
		bar.show_percentage = false
		bar.custom_minimum_size = BAR_SIZE
		_hud.add_child(bar)
		_bars[index] = bar
	var shown := _bars[index] as ProgressBar
	shown.position = Vector2(BAR_X, line.position.y + BAR_Y_OFFSET)
	shown.value = level
	shown.visible = true


func _notice(late: Dictionary) -> String:
	var employer := _link.shown_text(late["employer"]) if _link != null else String(late["employer"])
	var level := MineWorldText.code("word", late["level"])
	if late["absent"]:
		return MineWorldText.text("ui.notice.missed-shift", {"employer": employer, "level": level,
			"count": late["count"]})
	return MineWorldText.text("ui.notice.late-for-shift", {"employer": employer,
		"minutes": MineWorldClockFormat.whole_minutes(late["late_by"]), "level": level, "count": late["count"]})
