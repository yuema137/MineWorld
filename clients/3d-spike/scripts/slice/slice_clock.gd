## The world clock in the 3D slice (TW-e, SET-c): the world-time estimate fed from the link, the HUD's
## date and time and `Paused`, the sun rig following the disclosed sun, the host module and the World
## tab, and closing a world this client's launcher hosts (`clients/shared/WORLD_TIME.md` §6,
## `clients/shared/HOST.md` §§4–5). Kept out of `SliceMain`, which only calls it.
class_name SliceClock
extends Node

var world_time := MineWorldWorldTime.new()
var host_clock: MineWorldHostClock
var sun_rig: SliceSun
## Started by the local launcher (`--hosted-here`): closing stops the host unless kept running.
var hosted_here := false

var _settings: MineWorldSettingsStore
var _client: MineWorldClient = null
var _seen: MineWorldObservation = null
var _line: Label = null
var _closing_left := -1.0


## Before the menu exists: the host module (with its token, if the launcher gave one) and the sun rig,
## found under `slice` by the names the probes also read: `Sun`, `SkyFill`, and the far field's
## `FarField`.
func setup(settings: MineWorldSettingsStore, slice: Node3D, environment: Environment, fixed: bool) -> void:
	_settings = settings
	hosted_here = OS.get_cmdline_user_args().has("--hosted-here")
	get_tree().auto_accept_quit = not hosted_here
	host_clock = MineWorldHostClock.new()
	add_child(host_clock)
	host_clock.setup("http://" + SliceLink.address_from_args())
	sun_rig = SliceSun.new()
	add_child(sun_rig)
	sun_rig.setup(slice.get_node("Sun") as DirectionalLight3D, slice.get_node("SkyFill") as DirectionalLight3D,
		environment, fixed)
	sun_rig.far_field = slice.find_child("FarField", true, false) as Node3D


## The World tab, and Quit through `close_game`.
func attach_menu(menu: MineWorldSettingsMenu) -> void:
	menu.add_tab("ui.settings.tab.world", MineWorldWorldTab.new(menu, host_clock))
	menu.quit_requested.connect(close_game)


## Once connected: the link's client feeds the estimate and the host module.
func attach_client(client: MineWorldClient, hud_line: Label) -> void:
	_client = client
	_line = hud_line
	client.welcomed.connect(func(_seat: String, _observer: String, world: Dictionary) -> void:
		world_time.on_welcome(world, Time.get_ticks_usec())
		host_clock.on_welcome(world))
	client.clock_changed.connect(func(at: int, time_scale: int, paused: bool) -> void:
		world_time.on_clock(at, time_scale, paused, Time.get_ticks_usec())
		host_clock.on_clock(at, time_scale, paused))


func _process(delta: float) -> void:
	if _closing_left >= 0.0:
		_closing_left -= delta
		if _closing_left < 0.0 or _client == null or not _client.is_seated():
			get_tree().quit(0)
	if _client == null:
		return
	if _client.latest != null and _client.latest != _seen:
		_seen = _client.latest
		world_time.on_observation(_seen, Time.get_ticks_usec())
	if not world_time.anchored():
		return
	var shown := world_time.display_seconds(Time.get_ticks_usec())
	if _line != null:
		var line := MineWorldClockFormat.date_time(shown, world_time.date_at(shown))
		_line.text = line + "  " + MineWorldText.text("hud.paused") if world_time.paused() else line
	sun_rig.follow(world_time.sun_at(shown), world_time.phase_at(shown), world_time.has_calendar(), delta)


## Window close or the menu's Quit (HOST.md §5).
func close_game() -> void:
	if not hosted_here or not host_clock.has_token():
		get_tree().quit(0)
	elif _settings.settings.keep_running:
		get_tree().quit(10)
	elif _closing_left < 0.0:
		host_clock.stop_world()
		_closing_left = MineWorldHostClock.WAIT_SECONDS


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST and hosted_here:
		close_game()
