## DEP-35 — Client settings: Godot's `ConfigFile`, `DisplayServer` and `TranslationServer`; settings addons declined

**Date** 2026-10-09 · **Status** selected; no dependency added · **Approved by** the primary session at
S20 PR SET-a's design freeze (`DEP-SET-a` of step-20 §11.1; number allocated by the primary session, 2026-10-09) ·
**Relates to** `ARC-76`, `ARC-70`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design** step-20 §4.1–§4.3,
§4.5

**Options compared** (step-20 §4 holds the full tables and sources):

```text
persistence  Godot ConfigFile in user://                 adopt: typed values round-trip, readable,
                                                         per user; our own atomic write and validation
             ProjectSettings override.cfg                reject: per checkout, read once at start
             JSON through FileAccess                     reject: numbers come back as floats
             a Resource saved as .tres                   reject: a user-writable .tres can carry scripts
             OS-native preferences                       reject: native code, three back ends
display      DisplayServer + Engine.max_fps +            adopt: everything except changing the
             Viewport.scaling_3d_scale                   monitor's mode, which Godot does not expose
             project settings (display/window/*)         reject: read at start only
             native code to change the monitor's mode    reject (QSET-3): risky, per OS
menus        Maaack's Godot Menus / Game Template (MIT)  decline adoption, borrow the pattern
                                                         (PlayerConfig: a ConfigFile plus static apply
                                                         helpers): its scene and plugin flow does not
                                                         fit two code-built clients, and it has no
                                                         borderless/exclusive split, frame cap, render
                                                         scale or locale setting
             GGS — Godot Game Settings (MIT)             decline: editor-centric, saves resources, no
                                                         localisation
             our own small module                        build: about six short scripts over three
                                                         engine APIs, typed, with no editor step
text         ARC-70's TranslationServer + gettext .po    confirmed for both clients (CSV needs the
                                                         editor's importer; P-3)
```

**Credit.** The `ConfigFile` + static-apply pattern is Maaack's (`Maaack/Godot-Menus-Template`, MIT).

**Isolating interface.** `MineWorldSettingsStore` (`store.gd`) is the only code that reads or writes
the settings file; `MineWorldDisplay` (`display.gd`) is the only code that applies display settings.
