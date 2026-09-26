# Renderer-integration spike

A throwaway proof that MineWorld's merged contracts survive contact with a real renderer, run
before the kernel exists and while changing a contract is still cheap.

Nothing here is production code and nothing here is the kernel. The verdict is in
[`FINDINGS.md`](FINDINGS.md); this file only says how to run it.

```sh
./run.sh 2d      # the 2D client alone
./run.sh 3d      # the 3D client alone
./run.sh both    # both in turn against one server — this is what writes evidence/parity.json
```

Needs Rust on `$PATH` and Godot 4.7.2 as `godot`. The clients open a window: a headless Godot run
prints but renders nothing, so there would be no frame to capture. Screenshots and logs land in
`evidence/`.
