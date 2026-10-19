## DEP-26 — The Python toolchain: uv, ruff, pyright, pytest and pytest-socket

**Date** 2026-10-08 · **Status** adopted by S10 PR P3 · **Approved by** the primary session at P3's
freeze (QP3-3, QP3-4, QP3-6) and the operator's platform requirement of 2026-10-08 · **Relates to**
`DEP-17`, `DEP-18`, `ARC-48` · **Design** step-17 §4.7; P3 D-P3-4, D-P3-10, D-P3-11, R-P3-1, R-P3-2,
R-P3-9, R-P3-10 · **Placeholder** `DEP-S10-e`

**Problem.** Python enters a Rust repository. Its environments must be reproducible from one lock on
Linux, macOS and Windows; its code must be linted and type-checked strictly from the first module; its
suite must run with no network but localhost (`I-11`); and CI must run all of it without slowing the
required Rust checks.

**Choices, each with its alternatives:**

| Piece | Adopted (licence; version locked at P3) | Declined, and why |
| --- | --- | --- |
| Environments, locking, running | **uv** (MIT OR Apache-2.0, verified on PyPI 2026-10-08; uv 0.12) with a **virtual workspace root** (`pyproject.toml` with only `[tool.uv.workspace]`) and one universal `uv.lock`; its build backend `uv_build` for the SDK package | pip + venv + pip-tools: two tools and no workspace; Poetry: heavier, slower, no universal lock across members. A lock per package: two resolutions of shared dependencies once P6 adds a member (QP3-4) |
| Lint and format | **ruff** (MIT; 0.16) | flake8 + isort + black: three tools doing what one does |
| Types | **pyright** (MIT; 1.1.414) from PyPI with its **`nodejs` extra**, which brings Node.js as a locked wheel (`nodejs-wheel-binaries`, MIT) for every platform | mypy: weaker inference on Pydantic and asyncio code, and the step's design names pyright; basedpyright: equivalent, kept as R-P3-1's fallback; Node.js in the CI image: a second toolchain to pin |
| Tests | **pytest** (MIT; 9.1) | unittest: no fixtures or markers to run the real server once per test and deselect it by name |
| Network guard | **pytest-socket** (MIT; 0.8.1), `--allow-hosts=127.0.0.1,::1` on every platform | our own socket patching in `conftest.py`: re-implementing a small, purpose-built library (QP3-6) |
| CI, non-Linux legs | **`astral-sh/setup-uv`**, pinned by commit SHA in `ci.yml` (recorded with the job) | installing uv by a shell script fetched at run time: not pinned |

**Verified at P3 (C1), not assumed:**
- uv accepts a virtual root, and `uv sync --locked` builds the workspace from it.
- `uv.lock` holds `win_amd64`, `macosx_*_arm64` and `manylinux*_x86_64` wheels for every compiled
  dependency (`pydantic-core`, `websockets`, `nodejs-wheel-binaries`).
- The pyright wrapper prefers a `node` already on `PATH` and, unless told otherwise, asks PyPI whether it
  is the newest release on every run. CI therefore runs it with `PYRIGHT_PYTHON_GLOBAL_NODE=0` (the locked
  Node wheel, not whatever Node the runner has) and `PYRIGHT_PYTHON_IGNORE_WARNINGS=1` (no version query).
- pytest-socket blocks `connect` to any host but the allowed ones; with `--allow-hosts` given it ignores
  `--disable-socket`, so asyncio's self-pipe works on every platform with the same flags.

**Isolating interface.** The commands live in `.structured-coding/standards.md` and in
`scripts/ci_layer.py`; every one runs `uv run --locked`, so the lock is the whole resolution.

**Revisit** if uv's licence changes, if pyright's wrapper stops shipping its JavaScript, or when a
second workspace member needs a different Python floor.
