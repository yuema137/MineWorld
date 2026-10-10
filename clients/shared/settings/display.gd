## Applies display settings to a window (`clients/shared/SETTINGS.md` §5). The only code that does.
##
## It changes only what differs from the window's current state, so applying the defaults to a fresh
## window changes nothing, and under the headless display server every call is a no-op. Per-platform
## behaviour is Godot's; the one platform question asked here is `fullscreen_same_as_borderless`, for
## the menu's Wayland tooltip (AC-SET-16).
class_name MineWorldDisplay
extends RefCounted

## A client that renders 3D offers a render scale in the full-screen modes.
const CAP_RENDER_SCALE := "render_scale"

const PRESETS: Array[Vector2i] = [Vector2i(1280, 720), Vector2i(1600, 900), Vector2i(1920, 1080),
	Vector2i(2560, 1440)]

static var _platform_printed := false


static func apply(settings: MineWorldSettings, window: Window, capabilities: PackedStringArray) -> void:
	var mode := window_mode(settings.window_mode)
	if window.mode != mode:
		window.mode = mode
	if settings.window_mode == MineWorldSettings.WindowMode.WINDOWED and not size_pinned() \
			and window.size != settings.window_size:
		window.size = settings.window_size
		_centre(window)
	if capabilities.has(CAP_RENDER_SCALE):
		var scale := settings.render_scale / 100.0
		if not is_equal_approx(window.scaling_3d_scale, scale):
			window.scaling_3d_scale = scale
	var vsync := vsync_mode(settings.vsync)
	if DisplayServer.window_get_vsync_mode(window.get_window_id()) != vsync:
		DisplayServer.window_set_vsync_mode(vsync, window.get_window_id())
	var cap := effective_fps(settings.max_fps, window)
	if Engine.max_fps != cap:
		Engine.max_fps = cap


static func window_mode(mode: MineWorldSettings.WindowMode) -> Window.Mode:
	match mode:
		MineWorldSettings.WindowMode.BORDERLESS:
			return Window.MODE_FULLSCREEN
		MineWorldSettings.WindowMode.FULLSCREEN:
			return Window.MODE_EXCLUSIVE_FULLSCREEN
	return Window.MODE_WINDOWED


static func vsync_mode(vsync: MineWorldSettings.VSync) -> DisplayServer.VSyncMode:
	match vsync:
		MineWorldSettings.VSync.OFF:
			return DisplayServer.VSYNC_DISABLED
		MineWorldSettings.VSync.ADAPTIVE:
			return DisplayServer.VSYNC_ADAPTIVE
	return DisplayServer.VSYNC_ENABLED


## The frame cap in effect: `-1` is the monitor's own rate, or no cap where the rate is unknown.
static func effective_fps(max_fps: int, window: Window) -> int:
	if max_fps >= 0:
		return max_fps
	var rate := refresh_rate(window)
	return int(round(rate)) if rate > 0.0 else 0


## The monitor's refresh rate, read-only (Godot cannot change it); `-1` where unknown or headless.
static func refresh_rate(window: Window) -> float:
	return DisplayServer.screen_get_refresh_rate(window.current_screen)


## An engine `--resolution` (a launcher's `--res=`, a capture) wins over the saved size for the run.
static func size_pinned() -> bool:
	return OS.get_cmdline_args().has("--resolution")


## The window sizes offered: the presets and the screen's own size, each only if it fits the screen's
## usable rect, in physical pixels; always at least the current size.
static func size_presets(window: Window) -> Array[Vector2i]:
	var usable := DisplayServer.screen_get_usable_rect(window.current_screen).size
	var out: Array[Vector2i] = []
	for size in PRESETS + [DisplayServer.screen_get_size(window.current_screen)]:
		if size.x >= MineWorldSettings.MIN_WINDOW.x and size.y >= MineWorldSettings.MIN_WINDOW.y \
				and (usable == Vector2i.ZERO or (size.x <= usable.x and size.y <= usable.y)) \
				and not out.has(size):
			out.append(size)
	if not out.has(window.size) and window.size.x >= MineWorldSettings.MIN_WINDOW.x:
		out.append(window.size)
	out.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return a.x * a.y < b.x * b.y)
	return out


## The one declared platform case (AC-SET-16): on Wayland exclusive full screen is the same as the
## borderless one (Godot's `DisplayServer` reference), and the menu says so in a tooltip.
static func fullscreen_same_as_borderless() -> bool:
	return DisplayServer.get_name() == "Wayland"


## One line naming the platform, printed once, so every hand check records where it ran.
static func print_platform_once() -> void:
	if _platform_printed:
		return
	_platform_printed = true
	print(platform_line())


static func platform_line() -> String:
	return "[settings] platform %s, driver %s, display %s" % [OS.get_name(),
		RenderingServer.get_current_rendering_driver_name(), DisplayServer.get_name()]


static func _centre(window: Window) -> void:
	var rect := DisplayServer.screen_get_usable_rect(window.current_screen)
	if rect.size == Vector2i.ZERO:
		return
	window.position = rect.position + (rect.size - window.size) / 2
