extends Node2D

## The composition root of the 2D reference client: arguments in, parts wired, nothing decided.
##
## ```text
## link        the connection (the only one, ARC-47 R5)   town        the layout learned (ARC-45)
## intents     the only script that submits (R1)          walker      clicks and keys → requests
## places      façades, rooms, ground, dressing           people      who the observation lists
## presentation the pack, read at runtime (ARC-46)        projection  plan metres ↔ pixels
## menu        the offers about one subject (13b)         readers     own components, names
## panels      what was disclosed to me (I, H)            toasts      the world's answers
## talk_line   a typed line for a request                 words       the pack's wording (ARC-70)
## ```
##
## Arguments (after `--`): `--server=host:port`, `--seat=key`, `--invite=TOKEN` (the server's, from
## its join line), `--nickname=NAME`, `--presentation=<dir>|none`,
## `--variant=<set>`, `--drive[=scenario]`, `--capture`, `--shots=<dir>`. The launcher
## `./mineworld-2d` passes them; see `clients/2d/README.md`.

const Link := preload("res://scripts/link.gd")
const Intents := preload("res://scripts/intents.gd")
const Walker := preload("res://scripts/walker.gd")
const Town := preload("res://scripts/town.gd")
const ScreenProjection := preload("res://scripts/projection.gd")
const Presentation := preload("res://scripts/presentation.gd")
const Places := preload("res://scripts/scene/places.gd")
const People := preload("res://scripts/scene/people.gd")
const Grade := preload("res://scripts/scene/grade.gd")
const Status := preload("res://scripts/hud/status.gd")
const Words := preload("res://scripts/hud/words.gd")
const Readers := preload("res://scripts/hud/readers.gd")
const Panels := preload("res://scripts/hud/panels.gd")
const Toasts := preload("res://scripts/hud/toasts.gd")
const TalkLine := preload("res://scripts/hud/talk_line.gd")
const Menu := preload("res://scripts/menu.gd")
const Drive := preload("res://scripts/harness/drive.gd")

const DEFAULT_PACK := "presentation/mineworld-default/2D"

var options: Dictionary = {}
var presentation
var projection
var town
var link: Node
var intents: Node
var walker: Node
var places: Node
var people: Node
var status: CanvasLayer
var grade: CanvasLayer
var readers
var panels: CanvasLayer
var toasts: CanvasLayer
var talk_line: CanvasLayer
var menu: CanvasLayer
var world: Node2D
var camera: Camera2D
var latest: MineWorldObservation = null


func _ready() -> void:
	options = parse(OS.get_cmdline_user_args())
	_inputs()
	presentation = Presentation.new()
	var pack := String(options.get("presentation", DEFAULT_PACK))
	if pack != "none":
		presentation.load_pack(repo_path(pack), String(options.get("variant", "")))
		# The pack's wording, unless a run asks to play without it (the language-independence check).
		if not options.has("no-wording"):
			for problem in Words.load_pack(repo_path(pack)):
				push_warning("[mineworld-2d] %s" % problem)
	for problem in presentation.errors:
		push_warning("[mineworld-2d] %s" % problem)
	if presentation.is_plain():
		projection = ScreenProjection.new(ScreenProjection.PLAN, 24.0)
	else:
		projection = ScreenProjection.new(String(presentation.setting(["projection", "kind"], "isometric")),
			float(presentation.setting(["projection", "px_per_metre"], 32.0)))
		RenderingServer.set_default_clear_color(Color(String(presentation.setting(["clear_color"], "#e0ecee"))))
	town = Town.new()
	world = Node2D.new()
	world.name = "World"
	world.y_sort_enabled = true
	places = Places.new()
	add_child(places)
	places.setup(projection, presentation, town, world, self)
	add_child(world)
	people = People.new()
	add_child(people)
	people.setup(projection, presentation, town, world)
	link = Link.new()
	add_child(link)
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
	_hud()
	link.welcomed.connect(_on_welcomed)
	link.observed.connect(_on_observed)
	link.resolved.connect(_on_resolved)
	link.refused.connect(_on_refused)
	link.state_changed.connect(_on_state_changed)
	if options.has("drive"):
		var drive: Node = Drive.new()
		drive.app = self
		add_child(drive)
	link.open(String(options.get("server", "127.0.0.1:7878")), String(options.get("seat", "carol")),
		String(options.get("invite", "")), String(options.get("nickname", "2d-player")))


## The interaction layer (13b): readers and panels, toasts, the typed line, the menu.
func _hud() -> void:
	readers = Readers.new()
	readers.town = town
	panels = Panels.new()
	add_child(panels)
	toasts = Toasts.new()
	add_child(toasts)
	talk_line = TalkLine.new()
	add_child(talk_line)
	menu = Menu.new()
	menu.intents = intents
	menu.walker = walker
	menu.readers = readers
	menu.talk_line = talk_line
	add_child(menu)


func _process(_delta: float) -> void:
	if walker.placed:
		people.set_player(walker.body_plan)
		camera.position = projection.to_screen(walker.body_plan)
		grade.set_world_offset(camera.position)


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_ESCAPE:
				if menu.is_open():
					menu.close()
				else:
					get_tree().quit(0)
			KEY_E:
				open_menu_at(get_local_mouse_position(), false)
			KEY_Q:
				if latest != null:
					open_menu(latest.observer())
			KEY_I:
				panels.toggle("things")
			KEY_H:
				panels.toggle("conversations")
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		# Picking reads the event's own position, moved into this node's (the world's) frame, so a
		# scripted click and a real one take the same path.
		var at: InputEventMouseButton = make_input_local(event)
		if menu.is_open():
			menu.close()
		open_menu_at(at.position, true)


## A click (or E) at a point of the world's frame: a person's figure opens their menu (one's own figure
## opens one's own); otherwise a click walks — through a doorway near it, or to the floor (D-b-1).
func open_menu_at(at: Vector2, walk_otherwise: bool) -> void:
	var picked: String = people.pick(at)
	if picked != "":
		open_menu(picked)
	elif walk_otherwise:
		walker.walk_to_plan(projection.to_plan(at))


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
		world.modulate = Color(0.75, 0.75, 0.75)
	elif state == "seated":
		world.modulate = Color.WHITE
	status.show_state(latest, link, link.client.revision)


func _on_welcomed(observer: String, world_summary: Dictionary) -> void:
	var instance := String(world_summary.get("instance", ""))
	if instance != town.instance:
		town.reset(instance)
	readers.reset(instance)
	people.observer = observer


func _on_observed(observation: MineWorldObservation) -> void:
	latest = observation
	town.learn(observation)
	readers.learn(observation)
	places.reconcile(observation)
	walker.observe(observation)
	people.reconcile(observation)
	menu.refresh(observation)
	panels.show_panels(readers.panels(observation))
	# The module's `revision` (16b): the persisted revision `latest` was computed from.
	status.show_state(observation, link, link.client.revision)


## The world answered: a stride goes to the walker (13a); anything else the player chose is a toast
## worded from the result (D-b-6). After a `too_far_away` on a person, the toast offers the menu's
## own "walk to" entry when the world offers walking (D-b-3) — a choice, not a retry.
func _on_resolved(token: String, _action_id: String, result: Dictionary) -> void:
	walker.resolved(token, result)
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
	var keys := {"move_left": [KEY_A, KEY_LEFT], "move_right": [KEY_D, KEY_RIGHT],
		"move_up": [KEY_W, KEY_UP], "move_down": [KEY_S, KEY_DOWN]}
	for action in keys:
		if InputMap.has_action(action):
			continue
		InputMap.add_action(action)
		for code in keys[action]:
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
