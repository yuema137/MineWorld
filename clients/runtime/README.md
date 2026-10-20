# Shared runtime

The one Godot engine a release bundle carries for both clients. It holds no game code: started with
`-- --client=2d` or `-- --client=3d --root=<runtime folder>`, it loads that client's pack and starts it.
`scripts/package.py` exports it; nobody runs it from a checkout.

The design and the rules (which settings live here, which are applied at run time) are in
[`step-23 PR S23-RT`](../../.structured-coding/plans/mvp0/pr-s23-rt-shared-runtime.md) and `ARC-80`.
