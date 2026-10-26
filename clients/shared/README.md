# Shared client modules

Godot modules that both reference clients (`clients/2d`, `clients/3d-spike`) take by symlink, beside the
protocol module (`clients/protocol`). This folder is a small Godot project so the modules can be run and
checked on their own.

- `settings/` — the in-game settings: language (English, 简体中文), display, clock, and the menu. A client
  takes it as `res://mineworld_settings`. Settings stay on your machine and never reach the server.
- `environment/` — the far field: reads a Presentation Pack's `far_field/<world-key>.yaml` and refuses
  a missing or faulty one by name. A client takes it as `res://mineworld_environment`. The rules are in
  [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §6.1.1.
- `world_time/` — the world's date, time and sun between frames, as the server disclosed them. A client
  takes it as `res://mineworld_world_time`. See [`WORLD_TIME.md`](WORLD_TIME.md).
- `host/` — pause, resume and day length for the host, and the settings menu's World tab. A client takes
  it as `res://mineworld_host`. See [`HOST.md`](HOST.md).
- `trade/` — money in the world's own currency, price tags, the sale panel, the stall and the notice
  board, read the same way for both clients. A client takes it as `res://mineworld_trade`. See
  [`TRADE.md`](TRADE.md).
- `station/` — what a station's departures board, ticket window and kiosk show, from what the world
  disclosed and nothing else (`docs/decisions/ARC-108.md`). A client takes it as `res://mineworld_station`.
- `hud/` — the one-line text box both clients open when the player types something to say: talk in 2D
  and 3D, invite in 2D. Chinese is typed through your system's input method. A client takes it as
  `res://mineworld_hud`.
- `checks/` — headless checks of the modules.

Run a check:

```sh
godot --headless --path clients/shared --import
godot --headless --path clients/shared --script res://checks/store_check.gd
```

On Windows, clone with `git config core.symlinks true` (Developer Mode), or the modules arrive as text
files.

The specification is [`SETTINGS.md`](SETTINGS.md).
