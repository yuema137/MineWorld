#!/usr/bin/env python3
"""Run one CI layer: the only place that says which commands a layer is.

`ENGINEERING_STANDARDS.md` §16 names CI's layers and `docs/DECISIONS.md` ARC-48 gives them triggers.
The workflow (`.github/workflows/ci.yml`) names a layer and calls this script inside the toolchain
container; it never names a command (step-14 I-S13-9). Keeping the table here means the commands a
layer runs are reviewed as policy in one list, the same entry point runs on a laptop
(`python3 scripts/ci_layer.py fast`), and moving to another CI service does not move the policy.

Before the first command it prints the toolchain and how the checkout was made (shallow or not,
partial-clone filter), because a green layer is only evidence about the toolchain and history it ran
with. Every command is printed before it runs and timed after; the first failure stops the layer with
that command's exit status. Nothing is retried and nothing is allowed to fail (ARC-48).

    python3 scripts/ci_layer.py fast | core | parity
                                                   run a layer (on Linux in the toolchain container;
                                                   parity natively on macOS and Windows runners)
    python3 scripts/ci_layer.py docs              a documentation-only change: the doc checks only, on
                                                   the bare runner (step-14 §16)
    python3 scripts/ci_layer.py platforms         S16's packages natively on macOS and Windows
                                                   (`.github/actions/native`, not the container)
    python3 scripts/ci_layer.py python            the Python workspace: static checks, the binary, pytest
    python3 scripts/ci_layer.py python-smoke      the same without the binary or the real_server tests
    python3 scripts/ci_layer.py core --shard K/N  shard K of N of `core` (ARC-110; scripts/ci_shard.py
                                                   says which tests): the shards of `test`, `test-macos`, `test-windows`
    python3 scripts/ci_layer.py <layer> <layer>...  several layers in order; the first failure stops them
                                                   (macOS's two jobs, ARC-110)
    python3 scripts/ci_layer.py --list <layer> [--shard K/N]
                                                   print a layer's commands without running them
    python3 scripts/ci_layer.py --offline-check    vendor outside the checkout, then check the CLI
                                                   offline with an empty CARGO_HOME (EC-3 (b))
    python3 scripts/ci_layer.py --prune-cache     before CI saves target/: drop the workspace's own
                                                   artifacts and the tests' scratch saves, keep the
                                                   compiled dependencies
"""

from __future__ import annotations

import json
import os
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import ci_shard  # noqa: E402  (beside this script; the shards of `core`, ARC-110)

# The declared checks of `.structured-coding/standards.md`, split by layer (ARC-48). Order matters
# only for speed of the verdict: the cheapest checks first.
LAYERS: dict[str, list[list[str]]] = {
    "fast": [
        ["cargo", "fmt", "--all", "--check"],
        ["python3", "scripts/check_doc_headings.py"],
        ["python3", "scripts/check_decision_ids.py"],
        ["python3", "scripts/check_ci_pins.py"],
        ["python3", "scripts/check_scratch.py", "scan"],
        # AC-S26-9 (step-26 §5.5): no committed key, no secrets path in a runtime directory (S26-b1).
        ["python3", "scripts/check_secrets.py", "--self-test"],
        ["python3", "scripts/check_secrets.py", "scan"],
        ["python3", "scripts/ci_parity.py", "--self-test"],
        # The cast binding's structure (`ARC-92`): the committed file, then the checker's own cases.
        ["python3", "scripts/check_cast.py"],
        ["python3", "scripts/check_cast.py", "--self-test"],
        # Every pack's place art within its 10 MB budget (operator ruling on F-d1-5, S14 16f-d2).
        ["python3", "scripts/check_pack_size.py"],
        ["python3", "scripts/check_pack_size.py", "--self-test"],
        ["python3", "scripts/ci_changes.py", "--self-test"],
        # The packaging script's A-R3 exclusions, A-R4 budget and archive determinism (S23 R-a).
        ["python3", "scripts/package.py", "--self-test"],
        # The release workflow's tag, checksum and notes rules (S23 R-b; R-d's publishing gates share the file).
        ["python3", "scripts/ci_release.py", "--self-test"],
        # The nightly's classifiers (ARC-83), each under a second: a verdict rule that fails open would
        # otherwise be found only by a red night that never comes.
        ["python3", "scripts/ci_nightly.py", "--self-test"],
        ["python3", "scripts/ci_godot.py", "--self-test"],
        ["python3", "scripts/ci_repeat.py", "--self-test"],
        ["python3", "scripts/ci_stability.py", "--self-test"],
        # The partition of `core` into the test jobs' shards (ARC-110): complete and deterministic.
        ["python3", "scripts/ci_shard.py", "--self-test"],
        ["cargo", "check", "--workspace", "--all-targets"],
        ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
        # The code graph's licences, sources and bans (DEP-22, deny.toml); never advisories (ARC-48).
        ["cargo", "deny", "check", "licenses", "sources", "bans"],
    ],
    # Building first and running second runs the same tests; it only makes the log say how long the
    # build took and how long the tests did, which is what CI's budget is judged by. After a passing
    # suite, nothing may be left behind (ENGINEERING_STANDARDS.md §22, "Test scratch"; QTH-4).
    "core": [
        ["cargo", "test", "--workspace", "--no-run"],
        ["cargo", "test", "--workspace"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # A documentation-only change (step-14 §16, ARC-48's note of 2026-10-09 for 13x): the only CI
    # commands that read the audited docs set, plus the classifier's own self-test. Pure Python and the
    # standard library, run on the bare runner in `fast`'s place; no toolchain, no container.
    "docs": [
        ["python3", "scripts/check_doc_headings.py"],
        ["python3", "scripts/check_decision_ids.py"],
        ["python3", "scripts/ci_changes.py", "--self-test"],
    ],
    # AC-8's record on a native runner (macOS, Windows): the release binary, then every world recorded
    # (docs/DECISIONS.md ARC-49). The Linux legs record from the runtime image instead. On Windows the
    # script finds target/release/mineworld.exe.
    "parity": [
        ["cargo", "build", "--release", "--locked", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_parity.py", "record", "--binary", "target/release/mineworld"],
    ],
    # S16's packages on every platform (step-16 §16.12 PD-p1, §17.12 PD-q4): run natively on macOS and
    # Windows by the `platforms` job, outside the container. The subset of the suite that S16's crates and
    # commands own and that is portable today; the whole workspace on Windows is S13's (RE-p1). Each PR
    # of S16 that lands a portable CLI target adds it here (E-c: `third_party`, and PD-p3's offline check).
    "platforms": [
        # E-c: everything the lock names, first (EC-3 (a)'s fetch). The lock guard's `cargo metadata
        # --offline` reads every workspace crate's dependencies, dev-dependencies of crates this layer
        # never builds included (`trybuild`'s `glob`), so nothing after this may depend on what a build
        # happened to download.
        ["cargo", "fetch", "--locked"],
        ["cargo", "build", "--locked", "-p", "mineworld-cli"],
        # --no-fail-fast: on a platform, one red test binary must not hide another's result.
        [
            "cargo", "test", "--locked", "--no-fail-fast",
            "-p", "mineworld-packages", "-p", "mineworld-worldpack", "-p", "mineworld-installed-systems",
        ],
        [
            "cargo", "test", "--locked", "--no-fail-fast", "-p", "mineworld-cli",
            "--test", "packs", "--test", "requirements", "--test", "entity_packs",
            "--test", "third_party",
        ],
        # E-c: the lock guard and the graph check over this platform's checkout (EC-13), then the build
        # offline from a vendor directory with an empty CARGO_HOME (PD-p3).
        [
            "cargo", "test", "--locked", "--no-fail-fast",
            "-p", "mineworld-acceptance", "--test", "package_sources",
        ],
        ["python3", "scripts/ci_layer.py", "--offline-check"],
    ],
    # macOS's share of `platforms` (ARC-110): what that layer checks and `core` on the same runner does not.
    # The lock first, before any cargo command that could rewrite Cargo.lock; then EC-3 (b)'s offline
    # vendor check. Its test commands are a subset of `core`, which macOS runs in the same job pair.
    "platforms-lock": [
        ["cargo", "fetch", "--locked"],
        ["python3", "scripts/ci_layer.py", "--offline-check"],
    ],
}

# Layer 4, nightly (docs/DECISIONS.md ARC-83; .github/workflows/nightly.yml). None of these runs on a
# pull request or in `core`; each writes its result to artifacts/nightly/ for the night's verdict.
NIGHTLY_OUT = "artifacts/nightly"
LAYERS.update({
    # AC-8 at the long horizon on a native runner (macOS, Windows); the Linux legs record from the
    # runtime image instead (ARC-49's legs, the long profile).
    "parity-long": [
        ["cargo", "build", "--release", "--locked", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_parity.py", "record", "--binary", "target/release/mineworld", "--profile", "long",
         "--timings", f"{NIGHTLY_OUT}/timings-parity-long.json"],
    ],
    # The server killed and restarted under a seated SDK client, every world's 300-day save replayed, and
    # CA-13 on market-town's save (ENGINEERING_STANDARDS.md §16 layer 4). The SDK runs through the lock.
    "stability": [
        ["cargo", "build", "--release", "--locked", "-p", "mineworld-cli"],
        ["uv", "run", "--locked", "python", "scripts/ci_stability.py", "restarts", "--binary", "target/release/mineworld"],
        ["python3", "scripts/ci_stability.py", "replay", "--binary", "target/release/mineworld"],
        # The cognition package's `slow` cases, which no PR runs (pr-s28a D-a-15; primary ruling
        # 2026-10-16): the busy-store retention tests, the transparency harness's frames of 1 and 4 096, and
        # the full 50-kill test. None needs the binary. `-m slow` replaces the member's `addopts` expression.
        ["uv", "run", "--locked", "pytest", "cognition/lm-controller", "-m", "slow"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # The memory scale gates (pr-s28g §5.5): 12 people and the tiled 100-person café, each for 300 days, through
    # the release binary (the gates set MINEWORLD_BIN to it) and the shipped memory code. `-m scale` replaces the
    # member's `addopts` expression; the timings land in artifacts/nightly/timings-memory-scale.json.
    "memory-scale": [
        ["cargo", "build", "--release", "--locked", "-p", "mineworld-cli"],
        ["uv", "run", "--locked", "pytest", "cognition/lm-controller", "-m", "scale"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # The unchanged default suite sampled twice more on this commit (flakes); every sample runs.
    "core-repeat": [
        ["cargo", "test", "--workspace", "--no-run"],
        ["python3", "scripts/ci_repeat.py", "--times", "2", "--summary", f"{NIGHTLY_OUT}/repeat.txt", "--",
         "cargo", "test", "--workspace", "--no-fail-fast"],
    ],
    # The 25 #[ignore]d Godot tests, one at a time (each starts Godot and a server), on every OS (DEP-45):
    # `cargo test -p mineworld-cli` over client_2d, client_2d_interact, client_2d_interact_stub and
    # client_settings with `--ignored --test-threads=1`, minus this OS's named skips (QC-5), which
    # ci_godot.py holds with their reasons; it records the failing tests' names for the night's verdict.
    "clients": [
        ["cargo", "build", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_godot.py", "tests"],
        ["python3", "scripts/ci_godot.py", "coverage"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # PR 16g1 (AC-3DS-1 … -4, -6): typed speech in both Godot clients, and Alice on her `lm` seat with
    # Milestone D's scripted model recalling a line told in the other view. It needs Godot and the Python
    # workspace at once, so it is its own nightly job (`speech-linux`), never part of `clients`.
    "speech": [
        ["uv", "sync", "--locked"],
        ["cargo", "build", "-p", "mineworld-cli"],
        ["cargo", "test", "-p", "mineworld-cli", "--test", "speech_both_ways", "--", "--ignored",
         "--test-threads=1"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # RL-e0 (Q-FF-5, operator 2026-10-15): the far-field checks per PR on Linux, when ci_changes.py says
    # a client, a Presentation Pack or a launcher changed (the `farfield` job of ci.yml) -- the hosted Linux
    # run of the 3D check; the nightly `farfield-3d` layer runs the 3D check on Windows, and the `clients`
    # layer the 2D check and the validators on all three OSes (ci_godot.py SKIPS and DEDICATED, R-FF-1).
    "farfield": [
        ["cargo", "build", "-p", "mineworld-cli"],
        # S33-d: Lakeside's 3D check runs nightly on Windows (farfield-3d), not in this 50-min job
        ["cargo", "test", "-p", "mineworld-cli", "--test", "client_far_field", "--", "--ignored",
         "--skip", "three_d_lakeside_far_field_closes_every_open_sector", "--test-threads=1"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # RL-e0 R-FF-1 on Windows (F-FF-CI-3, 2026-10-18): the 3D rendered far-field check alone, in the nightly
    # `farfield-windows` job (120 min), which `clients-windows` skips by name (ci_godot.py DEDICATED).
    "farfield-3d": [
        ["cargo", "build", "-p", "mineworld-cli"],
        ["cargo", "test", "-p", "mineworld-cli", "--test", "client_far_field", "--", "--ignored", "--exact",
         "three_d_far_field_closes_every_open_sector", "--test-threads=1"],
        # S33-d AC-d-1: Lakeside drawn from its depiction file
        ["cargo", "test", "-p", "mineworld-cli", "--test", "client_far_field", "--", "--ignored", "--exact",
         "three_d_lakeside_far_field_closes_every_open_sector", "--test-threads=1"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # The bash launchers' probes, Linux and macOS (W-9): the 3D slice's verdicts parsed (they exit 0 on
    # failure), the protocol module's live checks, and AC-13's evidence judged by its owning test.
    "clients-probes": [
        ["cargo", "build", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_godot.py", "slice", "--drive"],
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--link"],
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--target"],
        # S33-c (ARC-144): Lakeside on the same server path, drawn from its depiction file (S33-d); its
        # scene walls against the server's along the whole probe route, now that S33-e draws the pier and
        # S33-f the bakery's and the boat shed's rooms (S33-d sec.15, C11; D-S33d-16)
        ["python3", "scripts/ci_godot.py", "slice", "--world=lakeside", "--link"],
        ["python3", "scripts/ci_godot.py", "slice", "--world=lakeside", "--geometry"],
        # S33-f (AC-f-2, AC-S33-3): every Lakeside sit point holds every body this checkout can build
        # within 20 mm of the drawn seat; bodies a checkout without LFS lacks are reported SKIP
        ["python3", "scripts/ci_godot.py", "slice", "--world=lakeside", "--seat"],
        # S33-i's agent + 3D observer pair is its own layer, `agent-observer`, so that Windows runs it too.
        # S15 12e: bodies in the 3D client (step-11 §22.4; the QE-9 requirement on 13c's clients job)
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--geometry"],
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--bodies"],
        # S14 16f-b (ARC-52): a world with no depiction, drawn plainly from the server's walls and walked
        # through its doorways; the art held to the disclosure with the greybox pack too
        ["python3", "scripts/ci_godot.py", "slice", "--world=bodies-yard", "--link"],
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--geometry",
         "--presentation=presentation/mineworld-greybox/3D"],
        ["bash", "clients/protocol/run.sh", "evidence"],
        ["bash", "clients/protocol/run.sh", "affordances"],
        ["bash", "clients/protocol/run.sh", "reconnect"],
        ["bash", "clients/protocol/run.sh", "perceived"],
        ["bash", "clients/protocol/run.sh", "deltas"],
        ["bash", "clients/protocol/run.sh", "admin"],
        ["cargo", "test", "-p", "mineworld-cli", "--test", "ac13_semantic_parity"],
        # S36-P4: runtime foot grounding (pr-s36-p4-foot-grounding.md §4 commit 4). Its own checks gate: the per-side
        # lift, flat on = off, the step straddle, the cost. Every body the client draws over the seed-1 course and
        # across the slice street's kerb is measured and recorded, not yet judged: the course's residual frames stay
        # record-only (Q-S36-P4-3); on the street, since S36-P4b's drawn-step colliders (Q-S36-P4-4) every body's
        # straddles pass and only the walker's touchdown frames remain (pr-s36-p4b-colliders-jog.md §7).
        ["python3", "tools/character_pipeline/cast.py", "feet", "--self-test", "artifacts/nightly/feet-selftest"],
        ["python3", "tools/character_pipeline/cast.py", "feet", "--run", "artifacts/nightly/feet-seed1",
         "--seed", "1", "--record-only"],
        ["python3", "tools/character_pipeline/cast.py", "feet", "--run", "artifacts/nightly/feet-street",
         "--scene", "street", "--record-only"],
        # S36-P5: every drawn prop of both towns' café, street and store, dumped by the slice, then judged
        # against the disclosed solids and its drawn support (`cast.py contact-scene`, PIPELINE.md §18).
        # Last, so a scene failure never hides the probes above; the dumps are uploaded with the night.
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--props",
         "--props-dump=artifacts/nightly/props/social-cafe"],
        ["python3", "scripts/ci_godot.py", "slice", "--world=market-town", "--props",
         "--props-dump=artifacts/nightly/props/market-town"],
        ["uv", "run", "--locked", "python", "tools/character_pipeline/cast.py", "contact-scene",
         "artifacts/nightly/props"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # S33-i (AC-i-4, ARC-175): the scripted MCP agent walks Lakeside while a real 3D client watches from the
    # street. It needs Godot and the Python workspace and no bash launcher, so unlike `clients-probes` it runs
    # in all three `clients-<os>` jobs, after their `clients` layer, each of which sets up uv.
    "agent-observer": [
        ["uv", "sync", "--locked"],
        ["cargo", "build", "-p", "mineworld-cli"],
        ["cargo", "test", "-p", "mineworld-cli", "--test", "agent_with_3d_observer", "--", "--ignored",
         "--test-threads=1"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
})
# The nightly's layers record how they ended (artifacts/nightly/layer-<layer>.txt): `started`, then
# `passed` or `failed at: <command>`. The night's verdict reads it, so that a layer that never started is
# INCONCLUSIVE and one that ran and failed is FAIL (I-13c-3). Per-PR layers write nothing.
NIGHTLY_LAYERS = {
    "parity-long", "stability", "core-repeat", "clients", "clients-probes", "agent-observer", "farfield-3d",
    "memory-scale", "speech",
}

# The Python workspace (sdk/python; DECISIONS.md DEP-26, ARC-48's note of 2026-10-08). Its tests never
# join `core`, whose job is a required check that must not get slower (pr-s10-p3 QP3-3). Every
# command runs through the lock (`uv run --locked`). A repository script is run by `sys.executable`
# rather than the literal `python3`, which Windows does not have; these layers also run outside the
# container, on Windows and macOS.
# The workspace's members (root pyproject.toml): the SDK (S10 P3), the cognition package (S10 P5a) and the
# agent player library (S27-c).
PYTHON_MEMBERS = ["sdk/python", "cognition/lm-controller", "clients/agent"]
# The onboarding harness (S25 PR 25f, tools/onboarding/PROTOCOL.md §9): standard-library scripts, not a workspace
# member (no dependency, nothing in uv.lock), held to the same static checks; its tests run with the workspace's
# pytest. Its own tests never start an agent or call a model (INV-S25-6).
# The world tiler (pr-s28g §5.2) joins them: one script over PyYAML (the root dev group) and its tests.
PYTHON_TOOLS = ["tools/onboarding", "tools/worldgen"]
PYTHON_STATIC: list[list[str]] = [
    ["uv", "sync", "--locked"],
    ["uv", "run", "--locked", "ruff", "check", *PYTHON_MEMBERS, *PYTHON_TOOLS],
    ["uv", "run", "--locked", "ruff", "format", "--check", *PYTHON_MEMBERS, *PYTHON_TOOLS],
    ["uv", "run", "--locked", "pyright", *PYTHON_MEMBERS, *PYTHON_TOOLS],
]
# The character pipeline (tools/character_pipeline, S26-b1) is not a workspace member, so it is checked by path with
# the same tools; its contact checks' libraries (S36-P1) come from the root dev group. Its pyright configuration is in pyproject.toml
# (`executionEnvironments`), so that `import mwcast` resolves in the tests.
PIPELINE = "tools/character_pipeline"
PYTHON_STATIC += [
    ["uv", "run", "--locked", "ruff", "check", PIPELINE],
    ["uv", "run", "--locked", "ruff", "format", "--check", PIPELINE],
    ["uv", "run", "--locked", "pyright", f"{PIPELINE}/mwcast", f"{PIPELINE}/tests", f"{PIPELINE}/cast.py"],
]
# The asset-generation tooling (tools/asset_generation) is linted by path with its own ruff.toml; its offline
# provenance test needs `yaml`, from the root's dev group (S26 cast director).
ASSET_GENERATION = "tools/asset_generation"
PYTHON_STATIC += [
    ["uv", "run", "--locked", "ruff", "check", ASSET_GENERATION],
    ["uv", "run", "--locked", "ruff", "format", "--check", ASSET_GENERATION],
]
# The casting director's 64 generated files are pinned: --check re-derives them, refuses a stray or missing
# brief directory, and holds the cast to the operator's picks (step-26 §§7.1-7.4a).
CAST_DIRECTOR_CHECK = [
    sys.executable, f"{PIPELINE}/casting/cast_director.py",
    "--config", "presentation/mineworld-default/3D/characters/casting.yaml", "--check",
]
# pytest runs once per member, never over both at once: given two paths, pytest takes its rootdir and
# ini file from their common ancestor, the repository root, whose pyproject.toml has no pytest section,
# and both members' `addopts` (the network guard among them) would be dropped silently
# (pr-s10-p5-backends.md D-P5-11). The cognition member's `addopts` also deselect `live_model`; no CI
# command selects that marker (D-P5-12).
# The static checks are also part of `fast`: they add about 6 s to it, measured on PR #98's first run
# (uv sync 3.0 s, ruff 0.1 s, pyright 2.5 s), well under the 60 s the ruling allows (QP3-3), so a Python
# lint or type error blocks a merge like a Rust one. The `python` layers keep them too, so that they are
# also judged on Windows and macOS, where `fast` does not run (D-P3-11, AP-12).
LAYERS["fast"] += PYTHON_STATIC
LAYERS["python"] = [
    *PYTHON_STATIC,
    # The real_server tests start the real binary; they fail, never skip, without it (D-P3-10).
    ["cargo", "build", "-p", "mineworld-cli"],
    ["uv", "run", "--locked", "pytest", "sdk/python"],
    ["uv", "run", "--locked", "pytest", "cognition/lm-controller"],
    ["uv", "run", "--locked", "pytest", "clients/agent"],
    ["uv", "run", "--locked", "pytest", "tools/onboarding"],
    ["uv", "run", "--locked", "pytest", "tools/worldgen"],
    ["uv", "run", "--locked", "pytest", f"{PIPELINE}/tests"],
    [sys.executable, f"{PIPELINE}/cast.py", "check-characters"],
    # S36-P1: C-3 and C-4 over every prop of the model library (needs numpy and the mesh libraries, so it runs
    # from the uv environment); a prop without a mount entry fails.
    ["uv", "run", "--locked", "python", f"{PIPELINE}/cast.py", "contact-items"],
    CAST_DIRECTOR_CHECK,
    ["uv", "run", "--locked", "pytest", f"{ASSET_GENERATION}/tests"],
    # Milestone D (S10 P7, D-P7-4): the real server and the real cognition process, started with the
    # workspace's interpreter, so it runs here, where both toolchains are, and is ignored in `core`. Its
    # unplugged half is an ordinary test and runs in `core`.
    [
        "cargo", "test", "-p", "mineworld-cli", "--test", "milestone_d", "--",
        "--ignored", "--exact", "she_remembers_across_the_two_views",
    ],
    # S31-d AC-d-11: the price clerk on a scripted LM (the real cognition process) and on `price-reporter`
    # post the same kind of notice, in two worlds; ignored in `core` for the same reason as Milestone D.
    ["cargo", "test", "-p", "mineworld-cli", "--test", "price_reporter_lm", "--", "--ignored"],
    [sys.executable, "scripts/check_scratch.py", "left", "--target-dir", "target"],
]
# IC-7 (S10 P7): a copy of the checkout without `cognition/lm-controller/` and its workspace line still
# syncs, builds and passes — the SDK, the agent library, and the whole Rust workspace, Milestone D's
# unplugged run among it (`scripts/ci_removability.py`, `pr-s10-p7-milestone-d.md` D-P7-3). And ARC-347
# (16e D-e-3): a copy without the 3D client formats, lints and passes every Rust test. The 3D check runs
# first, so its verdict is in the log whatever IC-7's is; without Godot in the container, its 2D drive is
# left out by name (`--no-godot`).
LAYERS["removability"] = [
    [sys.executable, "scripts/ci_removability.py", "--target", "3d", "--no-godot"],
    [sys.executable, "scripts/ci_removability.py"],
]
# No Rust build: the tests that need the binary — the SDK's real_server tests and the cognition
# package's real_binary AC-10 scenario (pr-s10-p4 §5.10) — are deselected by name, visibly, here and
# nowhere else. A `-m` on the command line replaces the member's `addopts` `-m "not live_model"` rather
# than adding to it, so the cognition expression restates that deselection.
LAYERS["python-smoke"] = [
    *PYTHON_STATIC,
    ["uv", "run", "--locked", "pytest", "sdk/python", "-m", "not real_server"],
    [
        "uv", "run", "--locked", "pytest", "cognition/lm-controller",
        "-m", "not live_model and not real_binary and not slow and not scale",
    ],
    ["uv", "run", "--locked", "pytest", "clients/agent", "-m", "not real_server"],
    ["uv", "run", "--locked", "pytest", "tools/onboarding", "-m", "not real_binary"],
    ["uv", "run", "--locked", "pytest", "tools/worldgen", "-m", "not real_binary"],
    ["uv", "run", "--locked", "pytest", f"{PIPELINE}/tests"],
    [sys.executable, f"{PIPELINE}/cast.py", "check-characters"],
    CAST_DIRECTOR_CHECK,
    ["uv", "run", "--locked", "pytest", f"{ASSET_GENERATION}/tests"],
]

# The pyright wrapper otherwise prefers whatever `node` is on PATH over the locked Node wheel, and asks
# PyPI for its newest version on every run (DEP-26).
COMMAND_ENVIRONMENT = {"PYRIGHT_PYTHON_GLOBAL_NODE": "0", "PYRIGHT_PYTHON_IGNORE_WARNINGS": "1"}

# Layers whose disk use is worth recording (step-14 A13-3: free disk and the size of target/).
MEASURES_DISK = {"core", "parity-long", "stability", "core-repeat", "memory-scale"}

ENVIRONMENT: list[list[str]] = [
    ["rustc", "-V"],
    ["cargo", "-V"],
    ["cargo", "fmt", "--version"],
    ["cargo", "clippy", "--version"],
    ["cargo", "deny", "--version"],
    ["git", "--version"],
    ["uv", "--version"],
    ["git", "rev-parse", "HEAD"],
    ["git", "rev-parse", "--is-shallow-repository"],
    ["git", "config", "--get", "remote.origin.partialclonefilter"],
]
# The `docs` layer runs on the bare runner: probing `cargo` or `rustc` there would make rustup install
# the toolchain `rust-toolchain.toml` pins, which is the minute the layer exists to save.
ENVIRONMENT_OF: dict[str, list[list[str]]] = {
    "docs": [
        ["python3", "--version"],
        ["git", "--version"],
        ["git", "rev-parse", "HEAD"],
        ["git", "rev-parse", "--is-shallow-repository"],
    ],
}


def report(command: list[str]) -> None:
    """Prints a command's output for the record; its failure is information, not a verdict."""
    try:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    except FileNotFoundError:
        print(f"[ci] {shlex.join(command)}: (not found on PATH)", flush=True)
        return
    output = (result.stdout or result.stderr).strip() or f"(none; exit {result.returncode})"
    print(f"[ci] {shlex.join(command)}: {output}", flush=True)


def size(path: Path) -> int:
    """Bytes under a directory, without following links (`du`, which Windows lacks)."""
    total = 0
    for directory, _, files in os.walk(path):
        for name in files:
            try:
                total += os.lstat(os.path.join(directory, name)).st_size
            except OSError:
                pass  # a file removed while walking: it no longer takes space
    return total


def gigabytes(count: int) -> str:
    return f"{count / 1024**3:.1f} G"


def disk(moment: str) -> None:
    """Prints free disk and target/'s size for the record, with no `df` or `du`, which Windows lacks;
    the record is information, never a verdict."""
    usage = shutil.disk_usage(ROOT)
    print(f"[ci] disk {moment}: {gigabytes(usage.free)} free of {gigabytes(usage.total)}", flush=True)
    for path in ("target", "target/tmp"):
        if (ROOT / path).exists():
            print(f"[ci]   {path}: {gigabytes(size(ROOT / path))}", flush=True)


def resolved(command: list[str]) -> list[str]:
    """`python3` is this interpreter: Windows runners have no `python3` on PATH. The listed command
    (`--list`) stays as written."""
    return [sys.executable, *command[1:]] if command[0] == "python3" else command


def offline_check() -> int:
    """EC-3 (b) on this runner (step-16 §16.12 PD-p3): vendor every dependency to a directory outside
    the checkout, then check the CLI with `--offline --frozen` against it and an empty CARGO_HOME, so
    nothing can come from the network or from a cache.

    The vendor directory and the empty home are siblings of the checkout and are removed afterwards.
    The configuration `cargo vendor` prints is written to a file with its directory as a `/` path, so
    it needs no escaping in TOML on Windows."""
    outside = ROOT.parent
    vendor = outside / "mineworld-vendor"
    home = outside / "mineworld-empty-cargo-home"
    config = outside / "mineworld-vendor.toml"
    for path in (vendor, home):
        shutil.rmtree(path, ignore_errors=True)
    home.mkdir()
    try:
        vendored = subprocess.run(
            ["cargo", "vendor", "--locked", "--versioned-dirs", str(vendor)],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        if vendored.returncode != 0:
            print(vendored.stderr, file=sys.stderr, flush=True)
            return vendored.returncode
        lines = [
            f'directory = "{vendor.as_posix()}"' if line.startswith("directory = ") else line
            for line in vendored.stdout.splitlines()
        ]
        config.write_text("\n".join(lines) + "\n", encoding="utf-8")
        crates = sum(1 for entry in vendor.iterdir() if entry.is_dir())
        print(f"[ci] vendored {crates} crates to {vendor}; CARGO_HOME={home} (empty)", flush=True)
        environment = {**os.environ, "CARGO_HOME": str(home)}
        command = [
            "cargo", "check", "--offline", "--frozen", "--config", str(config), "-p", "mineworld-cli",
        ]
        print(f"[ci] $ {shlex.join(command)}", flush=True)
        return subprocess.run(command, cwd=ROOT, env=environment).returncode
    finally:
        for path in (vendor, home):
            shutil.rmtree(path, ignore_errors=True)
        config.unlink(missing_ok=True)


def result(layer: str, line: str) -> None:
    """Appends to a nightly layer's result file; per-PR layers have none."""
    if layer in NIGHTLY_LAYERS:
        path = ROOT / NIGHTLY_OUT / f"layer-{layer}.txt"
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "a", encoding="utf-8", newline="\n") as out:
            out.write(line + "\n")


# Layers that may run as shards (ARC-110): `core`'s cargo commands become shard K's, and the scratch check
# still closes the layer, so each shard leaves nothing behind. `core-repeat` shards the same partition
# (ARC-149): shard K builds its targets, then samples its tests twice through ci_repeat.py.
# `clients` shards its Godot test targets by ci_godot.py's GODOT_WEIGHTS (the nightly's Linux jobs, which
# outran 75 minutes unsharded beside the probes); its build, coverage and scratch checks run in every shard.
SHARDABLE = {"core", "core-repeat", "clients"}
GODOT_TESTS_COMMAND = ["python3", "scripts/ci_godot.py", "tests"]


def repeat_commands(cargo: list[list[str]]) -> list[list[str]]:
    """A shard's cargo commands as `core-repeat` runs them: the build as it is, and each test command
    sampled twice (`--no-fail-fast`, as the unsharded layer). The doctests, when the shard holds them,
    write their own summary beside repeat.txt; the report merges a job's summaries by sample."""
    out = []
    for command in cargo:
        if "--no-run" in command:
            out.append(command)
            continue
        summary = f"{NIGHTLY_OUT}/repeat{'-doc' if '--doc' in command else ''}.txt"
        out.append(["python3", "scripts/ci_repeat.py", "--times", "2", "--summary", summary, "--",
                    *command[:2], "--no-fail-fast", *command[2:]])
    return out


def commands_of(layer: str, shard: str | None) -> list[list[str]]:
    """A layer's commands, or shard K/N's of it: ci_shard.py's cargo commands, then the layer's
    non-cargo-test commands in their order."""
    if shard is None:
        return LAYERS[layer]
    if layer == "clients":
        return [[*command, "--shard", shard] if command == GODOT_TESTS_COMMAND else command
                for command in LAYERS[layer]]
    cargo, account = ci_shard.shard_commands(shard)
    print(f"[ci] shard {shard} of {layer}; {account}", flush=True)
    if layer == "core-repeat":
        return repeat_commands(cargo)
    return [*cargo, *(command for command in LAYERS[layer] if command[:2] != ["cargo", "test"])]


def run(layer: str, shard: str | None = None) -> int:
    print(f"[ci] layer {layer}{f' shard {shard}' if shard else ''} in {ROOT}", flush=True)
    for command in ENVIRONMENT_OF.get(layer, ENVIRONMENT):
        report(resolved(command))
    measured = layer in MEASURES_DISK
    if measured:
        disk("before")
    result(layer, "started")
    started = time.monotonic()
    commands = commands_of(layer, shard)
    for command in commands:
        print(f"[ci] $ {shlex.join(command)}", flush=True)
        began = time.monotonic()
        try:
            status = subprocess.run(
                resolved(command), cwd=ROOT, env={**os.environ, **COMMAND_ENVIRONMENT}
            ).returncode
        except FileNotFoundError:
            print(f"[ci] {command[0]}: not found on PATH", flush=True)
            status = 127
        print(f"[ci] exit {status} after {time.monotonic() - began:.1f} s: {shlex.join(command)}", flush=True)
        if status != 0:
            if measured:
                disk("after (failed)")
            print(f"[ci] layer {layer} FAILED at: {shlex.join(command)}", flush=True)
            result(layer, f"failed at: {shlex.join(command)} (exit {status})")
            return status
    if measured:
        disk("after")
    print(f"[ci] layer {layer} passed: {len(commands)} command(s) in {time.monotonic() - started:.1f} s")
    result(layer, f"passed in {time.monotonic() - started:.0f} s")
    return 0


def prune_cache() -> int:
    """Leaves target/ holding compiled dependencies only, so the cache CI saves stays small."""
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if metadata.returncode != 0:
        print(metadata.stderr, file=sys.stderr)
        return metadata.returncode
    packages = sorted(package["name"] for package in json.loads(metadata.stdout)["packages"])
    shutil.rmtree(ROOT / "target" / "tmp", ignore_errors=True)
    command = ["cargo", "clean"] + [argument for name in packages for argument in ("-p", name)]
    print(f"[ci] $ cargo clean -p <{len(packages)} workspace packages>", flush=True)
    status = subprocess.run(command, cwd=ROOT).returncode
    disk("after pruning")
    return status


def main(arguments: list[str]) -> int:
    known = ", ".join(LAYERS)
    if arguments == ["--prune-cache"]:
        return prune_cache()
    if arguments == ["--offline-check"]:
        return offline_check()
    shard = None
    if len(arguments) >= 2 and arguments[-2] == "--shard":
        shard, arguments = arguments[-1], arguments[:-2]
        if not any(argument in SHARDABLE for argument in arguments):
            print(f"ci_layer.py: only {', '.join(sorted(SHARDABLE))} runs as shards", file=sys.stderr)
            return 2
        try:
            ci_shard.parse_shard(shard)
        except ValueError as error:
            print(f"ci_layer.py: {error}", file=sys.stderr)
            return 2
    if len(arguments) == 2 and arguments[0] == "--list" and arguments[1] in LAYERS:
        for command in commands_of(arguments[1], shard):
            print(shlex.join(command))
        return 0
    # Several layers run in order and the first failure stops the rest: macOS packs its checks into two
    # jobs, because its runners are the scarce ones (ARC-110). A shard applies to the shardable layers.
    if arguments and all(argument in LAYERS for argument in arguments):
        for layer in arguments:
            status = run(layer, shard if layer in SHARDABLE else None)
            if status != 0:
                return status
        return 0
    print(
        f"usage: ci_layer.py <layer>... [--shard K/N|K,L/N] | --list <layer> [--shard K/N] | --prune-cache"
        f"   (layers: {known})",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
