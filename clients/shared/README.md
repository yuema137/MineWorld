# Shared client modules

Godot modules that both reference clients (`clients/2d`, `clients/3d-spike`) take by symlink, beside the
protocol module (`clients/protocol`). This folder is a small Godot project so the modules can be run and
checked on their own.

- `settings/` — the in-game settings: language (English, 简体中文), display, clock, and the menu. A client
  takes it as `res://mineworld_settings`. Settings stay on your machine and never reach the server.
- `checks/` — headless checks of the modules.

Run a check:

```sh
godot --headless --path clients/shared --import
godot --headless --path clients/shared --script res://checks/store_check.gd
```

On Windows, clone with `git config core.symlinks true` (Developer Mode), or the modules arrive as text
files.

The specification is [`SETTINGS.md`](SETTINGS.md).
