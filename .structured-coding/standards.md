# MineWorld project standards

This file is the machine-readable part of MineWorld's engineering policy for the
`structured-coding` skill. It is **tracked in Git**, which is what makes it the project
standard rather than one developer's local preference.

The authoritative policies are [`docs/ENGINEERING_STANDARDS.md`](../docs/ENGINEERING_STANDARDS.md)
(the complete engineering policy) and [`docs/ENGINEERING_RULES.md`](../docs/ENGINEERING_RULES.md)
(the mandatory pre-implementation rules, including the playable-world and 2D/3D requirements),
alongside [`docs/REUSE_POLICY.md`](../docs/REUSE_POLICY.md) for dependency decisions. Nothing
here replaces them. The `conventions` list below is the subset that a reviewer or an
LLM can judge on a diff; the `checks` list is the subset a command can decide.

Only the single fenced `json` block is read. Everything outside it is prose for humans.

## Current state of the checks

No Rust crate and no Python package exists yet (the repository is specification-only at the
time of writing). The `cargo` checks below are declared now because
`docs/ENGINEERING_STANDARDS.md` §15 mandates them, and they become meaningful the moment
`kernel/` gains a `Cargo.toml`. Until then they report `INCONCLUSIVE`, never `PASS`.

`ruff` and `pyright` are declared but disabled. Enable them in the same change that
introduces the first Python module under `cognition/`.

All `cargo` entries need one deliberate approval before anything runs them:

```sh
python3 .claude/skills/structured-coding/scripts/standards.py approve --project .
python3 .claude/skills/structured-coding/scripts/standards.py inspect --project .
python3 .claude/skills/structured-coding/scripts/standards.py run --project . --base main
```

Nothing in this mechanism blocks a commit or a merge. It reports. The gates that must
actually block are CI's job.

## Declaration

```json
{
  "schema": 2,
  "review": {
    "trigger": "commit",
    "conventions": [
      "Every mutable piece of authoritative state has exactly one owning system, and no system writes a component another system owns",
      "A system that needs another system's state to change emits an event and lets the owner decide, rather than reaching into it",
      "Kernel code contains no domain semantics: no knowledge of jobs, money, romance, hunger, sleep, or any specific activity",
      "Renderer, client, and presentation concepts never appear in simulation contracts or kernel code",
      "LM-provider-specific concepts never appear in cognition contracts; provider details stay behind the backend interface",
      "Controllers and clients may only submit ActionIntents; they never mutate world state directly",
      "Adding a capability adds a module; it does not require invasive edits across unrelated existing modules",
      "Dependencies point one way: World Pack depends on Systems, Systems depend on kernel contracts, and nothing depends back",
      "No God object: no type knows about rendering, networking, cognition, and domain systems at once",
      "Domain concepts crossing a module boundary are explicit named types, not dicts, maps of string to any, JSON blobs, or untyped metadata",
      "Distinct semantic identifiers are distinct types, so they cannot be swapped by accident",
      "Types are chosen so that invalid domain states are difficult or impossible to represent",
      "Abstractions are introduced after a second real implementation exists, not in anticipation of one",
      "Functions perform one conceptual operation; a function past roughly 50 lines is reviewed for mixed responsibilities and past roughly 100 lines is treated as an architectural warning",
      "A file represents one coherent responsibility; a file past roughly 500 lines is reviewed and past roughly 800 lines is a strong warning",
      "Tests assert observable behavior, events, and contracts, not internal helper calls",
      "Integration, contract, and scenario tests carry the validation weight; unit tests exist where they carry real information, not to raise a count",
      "No test exists whose only assertion is that a getter returns a field, a constructor assigns its arguments, an enum lists its variants, or a third-party library works as documented",
      "Core tests do not require a live external LM API; they use deterministic controllers or recorded cognition results",
      "Randomness used in scenarios is seedable, and a fixed seed with fixed inputs and system versions reproduces the run",
      "The authoritative simulation runs and is testable headless, with no renderer present",
      "Every meaningful bug fix lands with a test that failed before the fix, at integration level when the defect spanned components",
      "Single-player and multiplayer use the same server-authoritative path; there is no separate single-player world logic",
      "Comments explain why something exists rather than restating the next line",
      "Before a public stable contract exists, a wrong early interface is changed cleanly instead of wrapped in a compatibility adapter",
      "Human-facing README.md files stay short and readable; every other Markdown file is a specification for a coding agent and must be complete and precise",
      "A design or plan that affects architecture is written into a document before implementation, so it can be reviewed",
      "A renderer or client never evaluates a world rule: it reports interaction intent and the server resolves it",
      "An action that needs spatial conditions declares them in its System contract (same Place, interaction radius, line of access, target available), and an action that needs no proximity declares that too",
      "Semantic world position and render-space position stay separate; no mesh, animation, physics, skeleton, camera, scene tree, or navmesh concept enters a generic contract",
      "Movement keeps MoveIntent, the travel Process, authoritative spatial state, and rendered movement as four distinct things, collapsing into neither renderer-local physics nor teleportation between place names",
      "A spatial or interaction contract is reviewed against whether it can support an embodied 3D client without redesigning the kernel, and whether 2D and 3D can share it without duplicating game logic",
      "The same semantic ActionIntent is produced whichever reference client initiated it; business rules are never implemented per client",
      "Rendering-related work is validated by actually running the relevant reference client, not by inferring it from server tests",
      "Substantial infrastructure is checked against mature existing solutions before being written from scratch",
      "A substantial dependency, and equally a decision to build our own instead of adopting one, carries a short record in docs/DECISIONS.md",
      "An external dependency sits behind a thin MineWorld-owned interface rather than leaking its types across the codebase",
      "No substantial code is copied from another project into MineWorld without its license, attribution and recorded origin",
      "World state, system ownership, network authority, plugin contracts and persistence semantics stay under MineWorld's control rather than a framework's",
      "A presentation pack decides how an interaction looks, never whether it is valid",
      "Visual changes to the default style are judged against the reference images in the reference images of the relevant presentation pack, not against prose alone",
      "A style manifest field must be inferable from reference images, chosen by the creator, or fixed by project policy; anything else does not belong in the schema"
    ]
  },
  "checks": {
    "trigger": "pr",
    "tools": [
      {
        "name": "cargo-fmt",
        "command": [
          "cargo",
          "fmt",
          "--all",
          "--check"
        ],
        "scope": "repository"
      },
      {
        "name": "cargo-check",
        "command": [
          "cargo",
          "check",
          "--workspace",
          "--all-targets"
        ],
        "scope": "repository"
      },
      {
        "name": "cargo-clippy",
        "command": [
          "cargo",
          "clippy",
          "--workspace",
          "--all-targets",
          "--all-features",
          "--",
          "-D",
          "warnings"
        ],
        "scope": "repository"
      },
      {
        "name": "cargo-test",
        "command": [
          "cargo",
          "test",
          "--workspace"
        ],
        "scope": "repository"
      },
      {
        "name": "ruff",
        "enabled": false,
        "scope": "changed"
      },
      {
        "name": "pyright",
        "enabled": false,
        "scope": "changed"
      }
    ]
  }
}
```

## Personal additions

`.structured-coding/standards.local.md` is untracked and personal. It may only add or
tighten: add a tool, enable a disabled one, widen a scope, add a convention, raise a
trigger. An attempt to relax anything declared here is refused by name. A genuine need to
skip a project check belongs in the PR document as a recorded deviation, where a reviewer
sees it.
