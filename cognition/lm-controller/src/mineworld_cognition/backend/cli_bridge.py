"""A subscription bridge: the user's own installed and logged-in model CLI as a backend (ARC-60).

`CliBridgeBackend` asks one question and reads one answer by running the CLI as the user, headlessly.
What the CLI is — its executable, its arguments, how a request becomes its prompt, how its output
becomes a `Completion` — is a `BridgePreset`'s; this module only runs it, under ARC-60's invariants:

- I-B1: nothing here reads, writes, copies or forwards a CLI's credentials; the CLI authenticates itself;
- I-B2: the child's environment is an allowlist (`ENVIRONMENT_ALLOWLIST` plus the preset's home
  variable); no `*_API_KEY`, no `*_TOKEN` and no key file's values (which never enter `os.environ`,
  D-P5-9) can reach it;
- I-B3: argv is the command plus the preset's literals, the generated schema path and a validated model
  name; the prompt goes through stdin and the schema through a file (D-B2: a Windows `.cmd` shim is run
  through `cmd.exe`, whose argument parsing is an injection surface);
- I-B4: the call is bounded, and on a timeout or a cancellation (the gateway's) the whole process tree
  is killed: POSIX, a new session and `killpg`; Windows, a new process group and `taskkill /T /F`.

The child runs in a fresh, empty temporary directory, removed afterwards, so no project file can be
picked up. A failure is typed (`BackendFailure`); stderr is used to classify it and is never carried.

Starting a subprocess needs an event loop that supports it: on Windows the default proactor loop, not
a selector loop (ledger F-P5b-1).
"""

from __future__ import annotations

import asyncio
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Protocol

from mineworld_sdk.wire.ids import JsonValue

from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.errors import ConfigError

ENVIRONMENT_ALLOWLIST = (
    "PATH",
    "PATHEXT",
    "SYSTEMROOT",
    "COMSPEC",
    "WINDIR",
    "HOME",
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
    "TEMP",
    "TMP",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "XDG_CACHE_HOME",
)
"""What a bridged CLI may see of the environment (D-B3), each only if set."""

_NEVER_PASSED = re.compile(r"(_API_KEY|_TOKEN)$|^(OPENAI|ANTHROPIC)_")
"""Names refused even when a preset's home variable would allow them (defence in depth for I-B2)."""

MODEL_NAME = re.compile(r"^[A-Za-z0-9._:/-]{1,128}$")
"""The only configuration text that reaches argv besides literals and the schema path (D-B2)."""

SCHEMA_FILE = "schema.json"


class BridgePreset(Protocol):
    """One CLI's command line and output format. Ruled-in presets live in `backend/presets/`."""

    @property
    def executable(self) -> str:
        """The name looked up on `PATH` (with `PATHEXT` on Windows) when no `command` is configured."""
        ...

    @property
    def home_variable(self) -> str | None:
        """The CLI's own configuration-directory variable, passed through when set."""
        ...

    def arguments(self, *, model: str | None, schema_path: Path | None) -> list[str]:
        """argv after the command: literals, `schema_path` and `model` only (I-B3)."""
        ...

    def schema_document(self, schema: JsonValue) -> JsonValue:
        """What is written to the schema file (a preset may lower it, `strict_schema`)."""
        ...

    def render_prompt(self, request: CompletionRequest) -> str:
        """The text written to stdin. The cassette key is computed on the request, before this."""
        ...

    def parse(self, stdout: bytes, request: CompletionRequest) -> Completion | BackendFailure:
        """The completion in the CLI's output; usage from its report, else estimated."""
        ...

    def classify(self, stderr_first_line: str, exit_code: int) -> BackendFailure:
        """The typed failure of a non-zero exit."""
        ...


def child_environment(source: Mapping[str, str], home_variable: str | None) -> dict[str, str]:
    """The allowlisted subset of `source` (I-B2)."""
    names = (*ENVIRONMENT_ALLOWLIST, *((home_variable,) if home_variable else ()))
    return {
        name: source[name] for name in names if name in source and not _NEVER_PASSED.search(name)
    }


async def kill_tree(process: asyncio.subprocess.Process) -> None:
    """Kills `process` and every descendant, then reaps it (I-B4; D-B4). `Process.kill()` alone would
    leave a `.cmd` shim's or a launcher's child running."""
    if process.returncode is None:
        if sys.platform == "win32":
            killer = await asyncio.create_subprocess_exec(
                shutil.which("taskkill") or "taskkill",
                "/T",
                "/F",
                "/PID",
                str(process.pid),
                stdin=asyncio.subprocess.DEVNULL,
                stdout=asyncio.subprocess.DEVNULL,
                stderr=asyncio.subprocess.DEVNULL,
            )
            await killer.wait()
            if process.returncode is None:
                try:
                    process.kill()
                except OSError:  # already gone, and not yet reaped
                    pass
        else:
            import signal

            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
    await process.wait()


class CliBridgeBackend:
    def __init__(
        self,
        preset: BridgePreset,
        *,
        command: Sequence[str] | None = None,
        model: str | None = None,
        request_timeout_s: int | None = None,
    ) -> None:
        """`command` replaces the `PATH` lookup (D-B1). A missing executable is a `ConfigError` naming
        what was looked for; there is no search of install directories."""
        if model is not None and not MODEL_NAME.fullmatch(model):
            raise ConfigError(f"model {model!r}: only letters, digits and . _ : / - (at most 128)")
        if command is not None:
            if not command:
                raise ConfigError(
                    "command: give the executable and its fixed arguments, or omit it"
                )
            self._command = list(command)
        else:
            found = shutil.which(preset.executable)
            if found is None:
                raise ConfigError(
                    f"{preset.executable!r} was not found on PATH; install it, or set "
                    f'command = ["/path/to/{preset.executable}"] in the backend table'
                )
            self._command = [found]
        self._preset = preset
        self._model = model
        self._timeout_s = request_timeout_s

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        preset = self._preset
        workdir = Path(tempfile.mkdtemp(prefix="mineworld-bridge-"))
        try:
            schema_path: Path | None = None
            if request.output_schema is not None:
                schema_path = workdir / SCHEMA_FILE
                document = preset.schema_document(request.output_schema)
                schema_path.write_text(json.dumps(document), encoding="utf-8", newline="\n")
            argv = [*self._command, *preset.arguments(model=self._model, schema_path=schema_path)]
            prompt = preset.render_prompt(request).encode("utf-8")
            return await self._run(argv, workdir, prompt, request)
        finally:
            shutil.rmtree(workdir, ignore_errors=True)

    async def _run(
        self, argv: list[str], workdir: Path, prompt: bytes, request: CompletionRequest
    ) -> Completion | BackendFailure:
        if sys.platform == "win32":
            creationflags, new_session = subprocess.CREATE_NEW_PROCESS_GROUP, False
        else:
            creationflags, new_session = 0, True
        try:
            process = await asyncio.create_subprocess_exec(
                *argv,
                cwd=workdir,
                env=child_environment(os.environ, self._preset.home_variable),
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                creationflags=creationflags,
                start_new_session=new_session,
            )
        except OSError:
            return BackendFailure(reason="unreachable")
        try:
            async with asyncio.timeout(self._timeout_s):
                stdout, stderr = await process.communicate(prompt)
        except TimeoutError:
            return BackendFailure(reason="timeout")
        finally:
            # A timeout here, the gateway's cancellation, or any error: nothing outlives the call.
            await kill_tree(process)
        if process.returncode != 0:
            first = stderr.decode("utf-8", "replace").strip().splitlines()[:1]
            return self._preset.classify(first[0] if first else "", process.returncode or 0)
        return self._preset.parse(stdout, request)

    async def aclose(self) -> None:
        return None

    def __repr__(self) -> str:
        return f"CliBridgeBackend({self._preset.executable!r}, model={self._model!r})"
