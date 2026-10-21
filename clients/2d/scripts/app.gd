extends Node2D

## The composition root of the 2D reference client: arguments in, parts wired, nothing decided.
##
## ```text
## link        the connection (the only one, ARC-47 R5)   town        the layout learned (ARC-45)
## intents     the only script that submits (R1)          walker      clicks and keys → requests
## places      façades, rooms, ground, dressing           people      who the observation lists
## furnishings the disclosed furniture (ARC-307)          objects     the loose objects, moved (ARC-342)
## presentation the pack, read at runtime (ARC-46)        projection  plan metres ↔ pixels
## menu        the offers about one subject (13b)         readers     own components, names
## panels      what was disclosed to me (I, H)            toasts      the world's answers
## talk_line   a typed line for a request                 words       the pack's wording (ARC-70)
## settings_menu  the shared settings menu (S20, `res://mineworld_settings`, `clients/shared/SETTINGS.md`)
## world_time  the world's time, date and sun (TW-e, `res://mineworld_world_time`, `WORLD_TIME.md`)
## host_clock  host commands and the World tab (SET-c, `res://mineworld_host`, `HOST.md`)
## ```
##
## Arguments (after `--`): `--server=host:port`, `--seat=key`, `--invite=TOKEN` (the server's, from
## its join line), `--nickname=NAME`, `--presentation=<dir>|none`, `--no-wording`,
## `--variant=<set>`, `--drive[=scenario]`, `--capture`, `--shots=<dir>`, `--settings=<path>|none`,
## `--world-key=<key>` (the World Pack the far field is read for, RL-e0), `--hosted-here` (the local
## launcher hosts this world: closing stops it unless the player keeps it running, `HOST.md` §5)
## (`--drive` and `--capture` mean `none` unless a path is given: a harness never reads the player's
## settings). The launcher `./mineworld-2d` passes them; see `clients/2d/README.md`.

const Link := preload("res://scripts/link.gd")
const Intents := preload("res://scripts/intents.gd")
const Walker := preload("res://scripts/walker.gd")
const Town := preload("res://scripts/town.gd")
const ScreenProjection := preload("res://scripts/projection.gd")
const Presentation := preload("res://scripts/presentation.gd")
const Places := preload("res://scripts/scene/places.gd")
const People := preload("res://scripts/scene/people.gd")
const Furnishings := preload("res://scripts/scene/furnishings.gd")
const Objects := preload("res://scripts/scene/objects.gd")
const Grade := preload("res://scripts/scene/grade.gd")
const Status := preload("res://scripts/hud/status.gd")
const Words := preload("res://scripts/hud/words.gd")
const Readers := preload("res://scripts/hud/readers.gd")
const Panels := preload("res://scripts/hud/panels.gd")
const Toasts := preload("res://scripts/hud/toasts.gd")
const Menu := preload("res://scripts/menu.gd")
const Drive := preload("res://scripts/harness/drive.gd")
const Daylight := preload("res://scripts/scene/daylight.gd")
const JoinScreen := preload("res://scripts/hud/join_screen.gd")
const Speech := preload("res://scripts/scene/speech.gd")
## The wording of a welcome that took a Person over (PR 13e, ARC-344), by `welcome.took_over`; `held`
## (one's own seat coming back) and `none` say nothing.
const TOOK_OVER_WORDING := {"hosted": "ui.seat.hosted", "connection": "ui.seat.connection"}

const DEFAULT_PACK := "presentation/mineworld-default/2D"
## The scripted and capture flags: each runs on default settings (INV-SET-5).
const HARNESS_FLAGS: PackedStringArray = ["drive", "capture"]

var options: Dictionary = {}
var settings_store: MineWorldSettingsStore
var settings_menu: MineWorldSettingsMenu
var presentation
var projection
var town
var link: Node
var intents: Node
var walker: Node
var places: Node
var people: Node
## The world's furnishings of every place stood in (ARC-307).
var furnishings: Node
## The loose objects of the observer's place, and their motion (ARC-342).
var objects: Node
var status: CanvasLayer
var grade: CanvasLayer
var readers
var panels: CanvasLayer
var toasts: CanvasLayer
var talk_line: MineWorldLineInput
var menu: CanvasLayer
var world: Node2D
var camera: Camera2D
var latest: MineWorldObservation = null
## Whether the connection is down (the view is grey), and the lateness events already told (by id).
var _down := false
var _noticed := {}
## The join screen (PR 13e): shown without `--seat` and whenever the connection ends for good.
var join_screen: CanvasLayer
## The latest welcome's `took_over`, told once the first observation names the Person.
var _took_over_untold := ""
## Speech bubbles and activity notices from the facts observed (PR 13e, ARC-343).
var speech: Node
## The Presentation Pack's far field for this world (RL-e0); null in plain drawing or when refused.
var far_field: MineWorldFarField = null
## The world's time between frames (TW-e), fed from the link; the HUD and the tint read it.
var world_time := MineWorldWorldTime.new()
var host_clock: MineWorldHostClock
var daylight: Node
var hosted_here := false
## Wall seconds left to wait for the host to stop before quitting anyway; < 0 when not closing.
var _closing_left := -1.0
## The offered affordance a throw is aimed with, waiting for a floor click ({} when not aiming), the
## object it is about, and the place it was chosen in (ARC-342).
var aiming: Dictionary = {}
var _aim_object := ""
var _aim_place := ""


func _ready() -> void:
	options = parse(OS.get_cmdline_user_args())
	hosted_here = options.has("hosted-here")
	# Closing a world this client's launcher hosts stops it gracefully first (HOST.md §5).
	get_tree().auto_accept_quit = not hosted_here
	_inputs()
	presentation = Presentation.new()
	var pack := String(options.get("presentation", DEFAULT_PACK))
	if pack != "none":
		presentation.load_pack(repo_path(pack), String(options.get("variant", "")))
	_settings(repo_path(pack) if pack != "none" else "")
	for problem in presentation.errors:
		push_warning("[mineworld-2d] %s" % problem)
	if presentation.is_plain():
		projection = ScreenProjection.new(ScreenProjection.PLAN, 24.0)
	else:
		projection = ScreenProjection.new(String(presentation.setting(["projection", "kind"], "isometric")),
			float(presentation.setting(["projection", "px_per_metre"], 32.0)))
		RenderingServer.set_default_clear_color(Color(String(presentation.setting(["clear_color"], "#e0ecee"))))
	if not _load_far_field(pack):
		return
	town = Town.new()
	world = Node2D.new()
	world.name = "World"
	world.y_sort_enabled = true
	places = Places.new()
	places.far_field = far_field
	add_child(places)
	places.setup(projection, presentation, town, world, self)
	add_child(world)
	furnishings = Furnishings.new()
	add_child(furnishings)
	furnishings.setup(projection, presentation, town, world)
	objects = Objects.new()
	add_child(objects)
	objects.setup(projection, presentation, town, world)
	people = People.new()
	add_child(people)
	people.setup(projection, presentation, town, world)
	link = Link.new()
	add_child(link)
	link.client.clock_changed.connect(_on_clock)
	intents = Intents.new()
	intents.link = link
	add_child(intents)
	walker = Walker.new()
	walker.town = town
	walker.intents = intents
	walker.projection = projection
	add_child(walker)
	camera = Camera2D.new()
	camera.zoom = Vector2.ONE * float(presentation.setting(["camera", "zoom"], 1.0))
	camera.position_smoothing_enabled = true
	camera.position_smoothing_speed = 4.0
	add_child(camera)
	camera.make_current()
	grade = Grade.new()
	add_child(grade)
	grade.setup(presentation)
	status = Status.new()
	add_child(status)
	daylight = Daylight.new()
	add_child(daylight)
	daylight.setup(repo_path(pack) if pack != "none" else "")
	_hud()
	join_screen = JoinScreen.new()
	add_child(join_screen)
	join_screen.join_requested.connect(_on_join_requested)
	speech = Speech.new()
	add_child(speech)
	speech.setup(people, toasts, presentation, world)
	link.ended.connect(_on_link_ended)
	link.welcomed.connect(_on_welcomed)
	link.observed.connect(_on_observed)
	link.resolved.connect(_on_resolved)
	link.refused.connect(_on_refused)
	link.state_changed.connect(_on_state_changed)
	if options.has("drive"):
		var drive: Node = Drive.new()
		drive.app = self
		add_child(drive)
	# One join path for one player and many (I-13e-3): with `--seat` straight in, as every scripted run
	# does; without, the join screen, prefilled from the arguments (Q-13e-1).
	var server_at := String(options.get("server", "127.0.0.1:7878"))
	var credential := String(options.get("invite", ""))
	var nickname := String(options.get("nickname", "2d-player"))
	join_screen.prefill(server_at, credential, nickname, String(options.get("seat", "")))
	if options.has("seat"):
		link.open(server_at, String(options["seat"]), credential, nickname)
	else:
		join_screen.open("")


## The player pressed Join or Take over on the join screen: the one call path (R5) opens the link.
func _on_join_requested(address: String, seat: String, credential: String, nickname: String,
		take_over: bool) -> void:
	join_screen.close()
	link.open(address, seat, credential, nickname, take_over)


## The connection ended for good (ARC-344): a player who was seated is told why, and the join screen
## shows the answer, offering "Take over" when the seat is occupied. Nothing rejoins by itself.
func _on_link_ended(code: String) -> void:
	if latest != null:
		toasts.toast({"kind": "seat", "action_type": "", "target": null, "reason": code},
			Words.text("ui.seat.ended", {"reason": Words.reason(code)}))
	join_screen.open(code)


## The far field of the world this client presents (RL-e0, `clients/2d/PRESENTATION.md` §8), read
## from the pack before anything is drawn. The world is named by `--world-key=<key>` (the launcher
## passes its `--world` directory's name). A missing or faulty far field is refused by name: printed,
## shown, and in a harness run the process exits 3; the world is not entered. Plain drawing has no
## Presentation Pack and so no far field, and says so.
func _load_far_field(pack: String) -> bool:
	if presentation.is_plain():
		print("[mineworld-2d] plain drawing: no Presentation Pack, no far field (diagnostic mode)")
		return true
	far_field = MineWorldFarField.load_for(repo_path(pack), String(options.get("world-key", "")), "2d")
	if far_field.ok():
		return true
	var line := far_field.refusal_line("2d")
	printerr(line)
	print(line)
	var layer := CanvasLayer.new()
	var label := Label.new()
	label.text = line
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.set_anchors_preset(Control.PRESET_FULL_RECT)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	label.add_theme_color_override("font_color", Color("3b2c1e"))
	layer.add_child(label)
	add_child(layer)
	if options.has("drive") or options.has("capture"):
		get_tree().quit(3)
	return false


## The player's settings, before anything is drawn (S20): the file (or defaults for a harness), the
## wording's layers — the shared module's, the pack's (unless `--presentation=none`), the user's —
## unless a run plays without wording (`--no-wording`, the language-independence check), the CJK font,
## the clock, the language and the display.
func _settings(pack_dir: String) -> void:
	settings_store = MineWorldSettingsStore.open(OS.get_cmdline_user_args(), HARNESS_FLAGS)
	for problem in MineWorldText.load_layers(pack_dir, settings_store.extra_locale_dirs,
			not options.has("no-wording")):
		push_warning("[mineworld-2d] %s" % problem)
	settings_store.settle_language(MineWorldText.locales())
	MineWorldText.install_font_fallback()
	MineWorldText.set_clock(settings_store.settings.clock)
	MineWorldText.set_language(settings_store.settings.language)
	MineWorldDisplay.print_platform_once()
	MineWorldDisplay.apply(settings_store.settings, get_window(), PackedStringArray())


## The interaction layer (13b): readers and panels, toasts, the typed line, the menu; and the settings
## menu (S20), over everything, which stops the walker while it is open.
func _hud() -> void:
	readers = Readers.new()
	readers.town = town
	panels = Panels.new()
	add_child(panels)
	panels.wake_pressed.connect(_on_wake_pressed)
	toasts = Toasts.new()
	add_child(toasts)
	talk_line = MineWorldLineInput.new()
	add_child(talk_line)
	menu = Menu.new()
	menu.intents = intents
	menu.walker = walker
	menu.readers = readers
	menu.talk_line = talk_line
	add_child(menu)
	menu.aim_chosen.connect(_on_aim_chosen)
	settings_menu = MineWorldSettingsMenu.new()
	add_child(settings_menu)
	settings_menu.setup(settings_store, PackedStringArray([MineWorldSettingsMenu.CAP_HOSTED_HERE])
		if hosted_here else PackedStringArray(), _menu_theme())
	# No look capability here: 2D has no mouse look, so the tab shows the key map alone (SET-b).
	settings_menu.add_tab("ui.settings.tab.controls", MineWorldControlsTab.new(settings_menu,
		PackedStringArray(), Controls2D.key_rows()))
	host_clock = MineWorldHostClock.new()
	add_child(host_clock)
	host_clock.setup("http://" + String(options.get("server", "127.0.0.1:7878")))
	settings_menu.add_tab("ui.settings.tab.world", MineWorldWorldTab.new(settings_menu, host_clock))
	settings_menu.quit_requested.connect(close_game)
	settings_menu.opened.connect(func() -> void: walker.process_mode = Node.PROCESS_MODE_DISABLED)
	settings_menu.closed.connect(func() -> void: walker.process_mode = Node.PROCESS_MODE_INHERIT)


## The settings menu in this client's warm panel colours.
static func _menu_theme() -> Theme:
	var theme := Theme.new()
	var panel := StyleBoxFlat.new()
	panel.bg_color = Color(0.99, 0.97, 0.92, 0.97)
	panel.set_corner_radius_all(12)
	panel.set_content_margin_all(22)
	theme.set_stylebox("panel", "PanelContainer", panel)
	theme.set_color("font_color", "Label", Color("3b2c1e"))
	theme.set_font_size("font_size", "Label", 15)
	return theme


## Every composed text again, in the language now selected: the status line, door labels, markers,
## the open menu and panels. Toasts already shown are cleared rather than left in the old language.
func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST and hosted_here:
		close_game()
		return
	if what != NOTIFICATION_TRANSLATION_CHANGED or status == null:
		return
	if latest != null:
		places.reconcile(latest)
		people.reconcile(latest)
		menu.refresh(latest)
		panels.show_panels(readers.panels(latest))
	toasts.clear()
	status.show_state(latest, link, link.client.revision if link.client != null else null)


func _process(delta: float) -> void:
	if _closing_left >= 0.0:
		_closing_left -= delta
		if _closing_left < 0.0 or link.state != "seated":
			get_tree().quit(0)
	if walker == null:
		return   # the far field was refused: the world was not entered (_load_far_field)
	if world_time.anchored():
		var shown := world_time.display_seconds(Time.get_ticks_usec())
		status.set_clock(MineWorldClockFormat.date_time(shown, world_time.date_at(shown)), world_time.paused())
		daylight.apply_sun(world_time.sun_at(shown), world_time.has_calendar())
	if walker.placed:
		people.set_player(walker.body_plan)
		camera.position = projection.to_screen(walker.body_plan)
		grade.set_world_offset(camera.position)


func _unhandled_input(event: InputEvent) -> void:
	if walker == null or join_screen.visible:
		return
	if event is InputEventKey and event.pressed and not event.echo:
		var action := Controls2D.action_for(event.keycode)
		if action == "menu":
			# Esc drops an aim, else closes what is open, else opens the settings menu, which holds Quit
			# (QSET-2).
			if not aiming.is_empty():
				drop_aim()
			elif menu.is_open():
				menu.close()
			else:
				settings_menu.toggle()
			return
		if settings_menu.is_open():
			return
		match action:
			"spot_menu":
				open_menu_at(get_local_mouse_position(), false)
			"person_menu":
				if latest != null:
					open_menu(latest.observer())
			"things":
				panels.toggle("things")
			"conversations":
				panels.toggle("conversations")
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT \
			and not settings_menu.is_open():
		# Picking reads the event's own position, moved into this node's (the world's) frame, so a
		# scripted click and a real one take the same path.
		var at: InputEventMouseButton = make_input_local(event)
		if menu.is_open():
			menu.close()
		if not aiming.is_empty():
			# An aim takes the click, wherever it lands — on a person too (the hint says so).
			_throw_at(at.position)
			return
		open_menu_at(at.position, true)
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_RIGHT \
			and not aiming.is_empty():
		drop_aim()


## A click (or E) at a point of the world's frame: a person's figure opens their menu (one's own figure
## opens one's own); else a drawn object opens its menu when the world offers me anything about it,
## and a click on one it offers nothing about walks to where it lies; otherwise a click walks —
## through a doorway near it, or to the floor (D-b-1).
func open_menu_at(at: Vector2, walk_otherwise: bool) -> void:
	var picked: String = people.pick(at)
	if picked != "":
		open_menu(picked)
		return
	var object: String = objects.pick(at)
	if object != "" and latest != null and not Menu.about_object(latest, object).is_empty():
		if not talk_line.is_open():
			menu.open(object, latest, get_viewport().get_canvas_transform() * objects.drawn_screen(object), true)
	elif object != "" and walk_otherwise:
		var lies: Vector3i = objects.disclosed_at(object)
		walker.walk_to(latest.place(), town.to_plan(latest.place(), {"x": lies.x, "y": lies.y}))
	elif walk_otherwise:
		walker.walk_to_plan(projection.to_plan(at))


## "Throw … at…" was chosen: the next floor click aims it (ARC-342). The status line says so.
func _on_aim_chosen(affordance: Dictionary) -> void:
	if latest == null:
		return
	aiming = affordance
	_aim_object = Menu._about(affordance)
	_aim_place = latest.place()
	status.set_aim(Words.text("ui.aim.hint", {"item": readers.thing(_aim_object)}))


## The aim's floor click: the point, in integer millimetres of my place's frame, sent on the offered
## affordance as clicked. Whether it can be thrown there is the server's answer, never checked here.
func _throw_at(at: Vector2) -> void:
	var offered := aiming
	var place := _aim_place
	drop_aim()
	intents.aim_throw(offered, town.local_in(place, projection.to_plan(at)))


## Drops an aim: nothing is sent.
func drop_aim() -> void:
	aiming = {}
	_aim_object = ""
	_aim_place = ""
	status.set_aim("")


## "Wake up…" in the asleep panel: the player's own menu opens, and the offered `wake` is chosen there, the
## path every offer takes (S35 N-e §4.3). No script but intents.gd names the action.
func _on_wake_pressed() -> void:
	if latest != null:
		open_menu(latest.observer())


## Opens the menu about a perceived person, or about oneself when `id` is the observer.
func open_menu(id: String) -> void:
	if latest == null or talk_line.is_open():
		return
	var figure: Variant = people.figure_at(id)
	var screen: Vector2 = get_viewport().get_canvas_transform() * (figure if figure != null else Vector2.ZERO)
	menu.open(id, latest, screen)


## While the connection is down the last observation stays drawn, dimmed: it is what was true, not
## what is (`ADOPTION.md` §6.1). A walk in progress is abandoned and nothing is replayed; a request
## whose answer never came is reported as unknown.
func _on_state_changed(state: String, _reason: String) -> void:
	if state == "reconnecting" or state == "closed":
		var unanswered: String = walker.abandon()
		if unanswered != "":
			toasts.toast({"kind": "unknown", "action_type": "", "target": null}, Words.text("ui.walk-unknown"))
		for asked in intents.drop_pending():
			toasts.toast(_record("unknown", asked), Words.text("ui.result.unknown", {"action": _what(asked, false)}))
		menu.close()
		_down = true
		_shade()
	elif state == "seated":
		_down = false
		_shade()
	status.show_state(latest, link, link.client.revision)


func _on_welcomed(observer: String, world_summary: Dictionary) -> void:
	var instance := String(world_summary.get("instance", ""))
	if instance != town.instance:
		town.reset(instance)
		_noticed.clear()
	readers.reset(instance)
	people.observer = observer
	_took_over_untold = link.took_over()
	speech.reset(instance)
	world_time.on_welcome(world_summary, Time.get_ticks_usec())
	host_clock.on_welcome(world_summary)


func _on_observed(observation: MineWorldObservation) -> void:
	latest = observation
	world_time.on_observation(observation, Time.get_ticks_usec())
	town.learn(observation)
	readers.learn(observation)
	places.reconcile(observation)
	furnishings.reconcile(observation)
	objects.reconcile(observation)
	# An aim lasts while its object is drawn in the place it was chosen in.
	if not aiming.is_empty() and (observation.place() != _aim_place or not objects.drawn_ids().has(_aim_object)):
		drop_aim()
	walker.observe(observation)
	people.reconcile(observation)
	menu.refresh(observation)
	panels.show_panels(readers.panels(observation))
	for late in MineWorldNeeds.lateness(observation):
		_notice_lateness(late)
	_tell_took_over(observation)
	# After people.reconcile, so a bubble finds its speaker drawn (PRESENTATION.md §4.1).
	speech.present(observation)
	_shade()
	# The module's `revision` (16b): the persisted revision `latest` was computed from.
	status.show_state(observation, link, link.client.revision)


## A lateness the employer told me of, once per event (S35 N-e §4.3): a notice, or a missed-shift notice
## when I never came. The words are the pack's; the minutes are a unit conversion of the disclosed seconds.
func _notice_lateness(late: Dictionary) -> void:
	var id: String = late["id"]
	if id == "" or _noticed.has(id):
		return
	_noticed[id] = true
	var args := {"employer": readers.thing(late["employer"]), "level": Words.word(late["level"]),
		"count": late["count"]}
	if late["absent"]:
		toasts.toast({"kind": "notice", "action_type": "", "target": null},
			Words.text("ui.notice.missed-shift", args))
		return
	args["minutes"] = MineWorldClockFormat.whole_minutes(late["late_by"])
	toasts.toast({"kind": "notice", "action_type": "", "target": null},
		Words.text("ui.notice.late-for-shift", args))


## A welcome that took a Person over, told once, by the name the first observation gives them
## (R-S11-3 shown; ARC-344): from the town (`hosted`) or from another player (`connection`).
func _tell_took_over(observation: MineWorldObservation) -> void:
	var key: String = TOOK_OVER_WORDING.get(_took_over_untold, "")
	_took_over_untold = ""
	if key != "":
		toasts.toast({"kind": "seat", "action_type": "", "target": null, "took_over": link.took_over()},
			Words.text(key, {"who": observation.display_name(observation.observer())}))


## The view's tint (S35 N-e §4.3): grey while the connection is down, a night blue while I am asleep as the
## observation says, white otherwise. One function reads both facts; no script computes either.
func _shade() -> void:
	if _down:
		world.modulate = Color(0.75, 0.75, 0.75)
	elif latest != null and not MineWorldNeeds.asleep_of(latest, latest.observer()).is_empty():
		world.modulate = Color(0.62, 0.66, 0.82)
	else:
		world.modulate = Color.WHITE


## A `clock` frame: the world-time estimate and the host module re-anchor (PROTOCOL.md §5.9).
func _on_clock(at: int, time_scale: int, paused: bool) -> void:
	world_time.on_clock(at, time_scale, paused, Time.get_ticks_usec())
	host_clock.on_clock(at, time_scale, paused)


## Window close or the menu's Quit (HOST.md §5): a world this client's launcher hosts is stopped
## gracefully (exit 0), or left running when the player chose so (exit 10); otherwise quit as before.
func close_game() -> void:
	if not hosted_here or host_clock == null or not host_clock.has_token():
		get_tree().quit(0)
	elif settings_store.settings.keep_running:
		get_tree().quit(10)
	elif _closing_left < 0.0:
		host_clock.stop_world()
		_closing_left = MineWorldHostClock.WAIT_SECONDS


## The world answered: a stride goes to the walker (13a); anything else the player chose is a toast
## worded from the result (D-b-6). After a `too_far_away` on a person, the toast offers the menu's
## own "walk to" entry when the world offers walking (D-b-3) — a choice, not a retry.
func _on_resolved(token: String, action_id: String, result: Dictionary) -> void:
	walker.resolved(token, result, action_id)
	var asked: Dictionary = intents.take(token)
	if asked.is_empty():
		if result.has("rejected") or result.has("unavailable"):
			toasts.toast({"kind": "rejected", "action_type": "", "target": null},
				Words.text("ui.walk-rejected", {"reason": Words.reason(result.get("rejected"))}))
		return
	if result.has("accepted"):
		toasts.toast(_record("accepted", asked), Words.text("ui.result.accepted", {"done": _what(asked, true)}))
	elif result.has("rejected"):
		var button := ""
		var press := Callable()
		var target: Variant = asked.get("target")
		if Words.code(result["rejected"]) == "too_far_away" and target != null and latest != null \
				and not intents.offered_walk(latest).is_empty():
			button = Words.text("ui.walk-to", {"target": readers.person(String(target))})
			press = walker.approach.bind(String(target))
		var record := _record("rejected", asked)
		record["reason"] = result["rejected"]
		toasts.toast(record, Words.text("ui.result.rejected", {"action": _what(asked, false),
			"reason": Words.reason(result["rejected"])}), button, press)
	else:
		toasts.toast(_record("unavailable", asked), Words.text("ui.result.unavailable", {"action": _what(asked, false)}))


func _on_refused(code: String, token: String, _detail: String) -> void:
	walker.refused(token)
	if token == "" and not link.is_seated():
		return   # a refused join: the join screen shows its code (PR 13e)
	var asked: Dictionary = intents.take(token)
	var record := _record("refused", asked)
	record["reason"] = code
	toasts.toast(record, Words.text("ui.result.refused", {"action": _what(asked, false), "code": code}))


## A toast's data: what kind of answer, to which request.
static func _record(kind: String, asked: Dictionary) -> Dictionary:
	return {"kind": kind, "action_type": asked.get("action_type", ""), "target": asked.get("target"),
		"about": asked.get("about", "")}


## What a request was, in words: its action's entry wording, or its "done" wording.
func _what(asked: Dictionary, done: bool) -> String:
	var target: Variant = asked.get("target")
	var about := String(asked.get("about", ""))
	return Words.action(String(asked.get("action_type", "")), {
		"target": "" if target == null else readers.person(String(target)),
		"item": "" if about == "" else readers.thing(about)}, done)


func _inputs() -> void:
	for action in Controls2D.WALK:
		if InputMap.has_action(action):
			continue
		InputMap.add_action(action)
		for code in Controls2D.WALK[action]:
			var event := InputEventKey.new()
			event.physical_keycode = code
			InputMap.action_add_event(action, event)


## `--key=value` and `--flag` arguments as a dictionary.
static func parse(args: PackedStringArray) -> Dictionary:
	var out := {}
	for arg in args:
		if not arg.begins_with("--"):
			continue
		var pair := arg.substr(2).split("=", true, 1)
		out[pair[0]] = pair[1] if pair.size() > 1 else ""
	return out


## A path relative to the repository root (two levels above this project), or an absolute one.
static func repo_path(path: String) -> String:
	if path.is_absolute_path():
		return path
	return ProjectSettings.globalize_path("res://").path_join("../..").path_join(path).simplify_path()
