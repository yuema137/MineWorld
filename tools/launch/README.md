# mineworld-launch

The double-click entry points of a downloadable MineWorld bundle: **MineWorld 2D**, **MineWorld 3D** and
**MineWorld 2D + 3D**. Each starts a local server on a world, opens the client joined to it, and stops the
server when you close the game. Saves and logs go to your per-user `MineWorld` folder.

From a repository checkout you do not need this: use `./mineworld-2d` or `./mineworld-slice --world`.

The full behaviour — bundle layout, arguments, `launch.toml`, stopping, errors — is specified in
[`LAUNCHER.md`](LAUNCHER.md); the decision is `ARC-78` in [`docs/DECISIONS.md`](../../docs/DECISIONS.md).
