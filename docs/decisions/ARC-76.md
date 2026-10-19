## ARC-76 — Client settings are a shared, presentation-only client module

**Date** 2026-10-09 · **Approved by** the primary session at S20 PR SET-a's design freeze (2026-10-08;
`ARC-SET-a` of step-20 §11.1; number allocated by the primary session, 2026-10-09) · **Relates to** `ARC-46`, `ARC-47`, `ARC-70`, `DEP-35`, `DEP-36`,
[`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§2–3, 10–12 · **Design**
`.structured-coding/plans/mvp0/step-20-client-settings.md` §3, §12; specification
[`clients/shared/SETTINGS.md`](../clients/shared/SETTINGS.md)

**Problem.** The operator requires an in-game settings menu in both reference clients: language (`en`
by default, `zh-Hans`), display (window mode, resolution, VSync, frame-rate cap) and a 12-hour or
24-hour clock, persisted per user, never reaching the server ("Framework, not demo" item 4). Two clients
that each built their own would drift; settings placed in the protocol module or on the server would
mix presentation with the world.

**Decided.**

1. Client settings are **one Godot module**, `clients/shared/settings/`, in a sibling project
   `clients/shared/` (the protocol module's pattern, `ADOPTION.md` §1). Each client takes it by symlink
   as `res://mineworld_settings`. No autoload, no required project setting, no import step.
2. The module is **presentation-only and has no network code**. A setting never becomes an
   `ActionIntent`, a frame or a request, and a client's frames are the same whatever its settings.
3. **One per-user `ConfigFile`**, `user://settings.cfg`, shared by both clients through the custom user
   directory `MineWorld`. It holds settings only: never an invite, an admin token, a seat, an address, a
   nickname or a world. Every scripted and capture mode runs on defaults (`--settings=none`).
4. **Host commands are not settings.** Pause and day length (S19 `/admin/clock`) live in a separate
   module, `clients/shared/host/` (SET-c), which contributes a tab to the menu and never reads or writes
   the settings file; the admin token is held in memory only.
5. The module has **no platform branch** beyond one declared case (a Wayland tooltip); per-platform
   behaviour is Godot's.

**Rejected.** Settings in the protocol module (it is the wire, owned by S11); settings as a server- or
world-side concept (they are one person's presentation); an autoload (a module a client adopts must not
change the project's global composition); a copy per client (two copies drift).
