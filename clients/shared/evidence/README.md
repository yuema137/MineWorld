# Settings module evidence (S20 SET-a)

Stills of the settings menu, captured by the harnesses on macOS (Godot 4.7.2, Metal, a 120 Hz display),
for the operator's review. They are evidence, not reference images: nothing compares against them.

- `2d/` — `./mineworld-2d --drive=settings --capture --shots=<this folder>/2d` on market-town: the menu
  over the HUD in English, then in 简体中文 (General and Display tabs).
- `3d/` — `./mineworld-slice --world --settings --stills=<this folder>/3d` on social-cafe: the same, and
  the HUD alone in 简体中文.

The checks behind them are listed in `clients/shared/SETTINGS.md` §9; the PR's acceptance evidence is in
`.structured-coding/plans/mvp0/step-20-client-settings.md` §12.5.
