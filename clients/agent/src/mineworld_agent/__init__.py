"""Play a seat of a MineWorld world from a program, as a client (`docs/DECISIONS.md` `ARC-96`).

`Player` holds one seat per handle, each over its own `mineworld_sdk.ResumingSeat`, and renders that seat's
own observation compactly: short references, numbered offers, pages and a hard size cap. It adds no world
rule, evaluates no requirement, and carries world-authored text only as typed `WorldText`. No MCP here.
"""
