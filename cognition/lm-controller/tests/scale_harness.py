"""The memory scale harness (pr-s28g-scale-gates.md §5.3): a world run for many days, each person's perceived
facts streamed from `mineworld perceived` into the shipped memory code, and the cost formulas checked (§5.5).

The run is real: `mineworld run`, `mineworld perceived … --json` read through a pipe (never a file of facts), the
stack and retention resolved from `mineworld decisions --json`, `absorb` into `MemoryHome`s opened with
`MemoryConfig()`'s page cache. Each group of people (one home) is absorbed in its own `spawn`ed worker process, so
its peak RSS is its own and never the test process's (F-S28g-8).

`check(...)` holds every S-n of §5.5 in one place and returns each violation as text; the gates assert it is
empty, and the per-PR smoke test shows each message fires on a planted violation. No model is called anywhere.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import math
import os
import re
import subprocess
import sys
import time
from collections import Counter
from collections.abc import Iterator, Sequence
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass, field
from itertools import chain
from multiprocessing import get_context
from pathlib import Path
from typing import Literal, cast

import yaml
from binary import REPOSITORY, binary
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId, WorldInstanceId
from retention_stream import draw

from mineworld_cognition.controller.assignment import ResolvedDecisions
from mineworld_cognition.memory.config import MemoryConfig
from mineworld_cognition.memory.explain import explain
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.bound import StoreBounds
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, resolve
from mineworld_cognition.memory.scopes.anchored import AnchoredParameters
from mineworld_cognition.memory.scopes.host import recall
from mineworld_cognition.memory.scopes.interface import RecallQuery
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryStack, resolve_stack

MIB = 1_048_576
DAY = 86_400
REBUILD_FRAME = 256
CONTINUITY_FROM_DAY = 8
EXPLAINED_IDS = 1_000
SAMPLE_SEED = 7
FILE_SLACK = 8 * MIB
RSS_SLACK = 256 * MIB
RECORD_BYTES = 512
CACHE_RECORDS = 0
"""No record LRU exists (QS28f-1 as ruled), so AS28-17's `cache_records` term is 0 (R-S28g-f-1)."""
STEADY_STATE = 0.70
"""AS28-12's steady state: physical at most 0.70 of logical at 300 days, on the café."""
K_FILE = 3.95
"""S-3's factor (QS28g-3): the largest measured `file_bytes / physical` plus 25 %, rounded up to 0.05. Until the
first nightly measures it (C4, INCONCLUSIVE at implementation), FG-b2's 3.13 (G-9) times 1.25 = 3.9125, so 3.95. The
timings file records the measured ratio each night, so the factor is recalibrated from evidence, never guessed."""
DECISIONS = Path(__file__).resolve().parent / "fixtures" / "scale.decisions.yaml"
RUN_TIMEOUT_S = 6 * 3600

type Homes = Literal["one", "per-block"]


@dataclass(frozen=True)
class ScaleSpec:
    world: Path
    days: int
    seed: int
    frame: int
    homes: Homes
    workers: int
    sample: int
    """Seats rebuilt alone and compared (S-5)."""
    continuity: bool
    """C-7 / C-8 recall points and the steady state (S-7 … S-9): the café only."""


@dataclass(frozen=True)
class SeatResult:
    seat: str
    facts: int
    passes: int
    largest_size: int
    """The largest `view.size()` after any absorb call."""
    largest_after_pass: int
    """The largest `PassReport.bytes_after` of any pass."""
    size: int
    dump_sha256: str | None
    """Sampled seats only."""
    l2_empty_days: tuple[int, ...] = ()
    most_met_in_l3: bool | None = None
    unexplained: int | None = None


@dataclass(frozen=True)
class HomeResult:
    path: str
    seats: tuple[SeatResult, ...]
    physical: int
    logical: int
    file_bytes: int
    peak_rss: int
    anchor_rows_max: int
    anchors_max: int
    wall: dict[str, float] = field(default_factory=dict[str, float])


@dataclass(frozen=True)
class Person:
    key: EntityKey
    name: str


@dataclass(frozen=True)
class ScaleRun:
    spec: ScaleSpec
    homes: tuple[HomeResult, ...]
    rebuilt: dict[str, str]
    """Sampled seat → sha256 of its view rebuilt alone."""
    bounds: StoreBounds
    page_cache_kib: int
    wall: dict[str, float]


# ── The binary ──────────────────────────────────────────────────────────────────────────────────


def _mineworld(*arguments: str | Path) -> str:
    result = subprocess.run(
        [str(binary()), *(str(one) for one in arguments)],
        cwd=REPOSITORY,
        capture_output=True,
        encoding="utf-8",
        timeout=RUN_TIMEOUT_S,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"mineworld {' '.join(map(str, arguments))} exited {result.returncode}: {result.stderr}"
        )
    return result.stdout


TILER = REPOSITORY / "tools" / "worldgen" / "tile.py"


def tiled_world(source: Path, people: int, out: Path) -> Path:
    """`source` tiled to `people` with the committed tiler (pr-s28g §5.2) into `out/<id>`: generated at use,
    never committed (QS28g-4). Names stay unique per block; the single station hall is dropped (QS28g-1)."""
    world = out / f"{source.name}-{people}"
    subprocess.run(
        [
            sys.executable,
            str(TILER),
            "--source",
            str(source),
            "--people",
            str(people),
            "--id",
            world.name,
            "--out",
            str(world),
            "--unique",
            "name",
            "--drop",
            "settlement",
        ],
        cwd=REPOSITORY,
        check=True,
        timeout=600,
    )
    return world


def run_world(spec: ScaleSpec, scratch: Path) -> tuple[Path, WorldInstanceId]:
    save = scratch / "save"
    _mineworld(
        "run",
        spec.world,
        "--headless",
        "--seed",
        str(spec.seed),
        "--days",
        str(spec.days),
        "--save",
        save,
    )
    found = re.search(r"instance ([0-9a-f]{32})", _mineworld("inspect", save))
    assert found is not None, "mineworld inspect printed no world instance"
    return save, WorldInstanceId(found.group(1))


def people(world: Path) -> list[Person]:
    """Every person of the world, in key order, with the name their file gives (the `named` fact's name)."""
    out: list[Person] = []
    for path in sorted((world / "people").glob("*.yaml")):
        data = cast(dict[str, object], yaml.safe_load(path.read_text(encoding="utf-8")))
        out.append(Person(EntityKey(path.stem), str(data.get("name", ""))))
    return out


def memory_choice(world: Path) -> MemoryPolicyChoice:
    """The fixture's `memory.default`, read through `mineworld decisions --json` and the controller's document
    model. Every person gets it, seats or not: memory is the controller's, and every person perceives."""
    document = ResolvedDecisions.model_validate_json(
        _mineworld("decisions", world, "--decisions", DECISIONS, "--json")
    )
    choices = {
        entry.memory.model_dump_json() for entry in document.seats if entry.memory is not None
    }
    assert len(choices) == 1, (
        f"the scale fixture gives every seat one memory choice, not {len(choices)}"
    )
    return MemoryPolicyChoice.model_validate_json(choices.pop())


def frames(
    world: Path, save: Path, person: EntityKey, size: int, split_days: bool
) -> Iterator[list[PerceivedEvent]]:
    """The person's perceived facts in frames of `size`, read from the binary's pipe. With `split_days`, a frame
    also ends at each first fact of a new day (that fact included), where a day's pass runs."""
    command = [
        str(binary()),
        "perceived",
        str(world),
        "--save",
        str(save),
        "--person",
        str(person),
        "--json",
    ]
    with subprocess.Popen(
        command, cwd=REPOSITORY, stdout=subprocess.PIPE, encoding="utf-8"
    ) as process:
        assert process.stdout is not None
        frame: list[PerceivedEvent] = []
        day: int | None = None
        for text in process.stdout:
            if not text.strip():
                continue
            one = PerceivedEvent.model_validate_json(text)
            frame.append(one)
            new_day = day is not None and one.at // DAY > day
            day = one.at // DAY
            if len(frame) >= size or (split_days and new_day):
                yield frame
                frame = []
        if frame:
            yield frame
        if process.wait() != 0:
            raise RuntimeError(f"mineworld perceived --person {person} exited {process.returncode}")


def _named(one: PerceivedEvent) -> tuple[EntityId, str] | None:
    if one.event_type != "named":
        return None
    body = one.payload.payload
    if not isinstance(body, dict):
        return None
    who = body.get("person")
    name = body.get("name")
    if not isinstance(who, dict) or not isinstance(name, str):
        return None
    entity = who.get("entity")
    return (EntityId(entity), name) if isinstance(entity, str) else None


# ── One group, in a worker ──────────────────────────────────────────────────────────────────────


def peak_rss_bytes() -> int:
    """This process's peak resident set, per OS."""
    if sys.platform == "win32":
        import ctypes
        from ctypes import wintypes

        class Counters(ctypes.Structure):
            _fields_ = [
                ("cb", wintypes.DWORD),
                ("PageFaultCount", wintypes.DWORD),
                ("PeakWorkingSetSize", ctypes.c_size_t),
                ("WorkingSetSize", ctypes.c_size_t),
                ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
                ("QuotaPagedPoolUsage", ctypes.c_size_t),
                ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
                ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                ("PagefileUsage", ctypes.c_size_t),
                ("PeakPagefileUsage", ctypes.c_size_t),
            ]

        counters = Counters()
        counters.cb = ctypes.sizeof(Counters)
        psapi = ctypes.WinDLL("psapi")
        kernel = ctypes.WinDLL("kernel32")
        kernel.GetCurrentProcess.restype = wintypes.HANDLE
        psapi.GetProcessMemoryInfo(kernel.GetCurrentProcess(), ctypes.byref(counters), counters.cb)
        return int(counters.PeakWorkingSetSize)
    import resource

    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return peak if sys.platform == "darwin" else peak * 1024


@dataclass(frozen=True)
class GroupJob:
    spec: ScaleSpec
    save: Path
    instance: WorldInstanceId
    group: tuple[Person, ...]
    home_path: Path
    sampled: frozenset[str]
    choice: MemoryPolicyChoice


def _anchor_rows(home: MemoryHome, view: SeatView) -> int:
    rows = 0
    for (table,) in home.connection.execute(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name LIKE '%\\_known' ESCAPE '\\'"
    ).fetchall():
        (count,) = home.connection.execute(
            f"SELECT count(*) FROM {table} WHERE seat = ?", (view.seat_id,)
        ).fetchone()
        rows = max(rows, int(count))
    return rows


def _anchors_max(stack: MemoryStack) -> int:
    caps = [
        bound.parameters.anchors_max
        for bound in stack.scopes
        if isinstance(bound.parameters, AnchoredParameters)
    ]
    return min(caps) if caps else 0


def _file_bytes(path: Path) -> int:
    return sum(
        candidate.stat().st_size
        for candidate in (
            path,
            path.with_name(path.name + "-wal"),
            path.with_name(path.name + "-journal"),
        )
        if candidate.exists()
    )


def _identity(
    instance: WorldInstanceId, person: Person, stream: Iterator[list[PerceivedEvent]]
) -> tuple[StoreIdentity, list[list[PerceivedEvent]]]:
    """The person's entity, from the `named` fact whose name is theirs: frames are read until it appears."""
    held: list[list[PerceivedEvent]] = []
    for frame in stream:
        held.append(frame)
        for one in frame:
            found = _named(one)
            if found is not None and found[1] == person.name:
                return StoreIdentity(
                    world_instance=instance, seat=person.key, self_entity=found[0]
                ), held
    raise AssertionError(f"{person.key}: no `named` fact names {person.name!r}")


def _counterpart(facts: Counter[EntityId], me: EntityId) -> EntityId | None:
    del facts[me]
    return facts.most_common(1)[0][0] if facts else None


def absorb_group(job: GroupJob) -> HomeResult:
    """Every person of the group, seat-major in key order, into one home (runs in a worker)."""
    spec = job.spec
    started = time.monotonic()
    page_cache_kib = MemoryConfig().page_cache_kib
    loop = asyncio.new_event_loop()
    results: list[SeatResult] = []
    wall: dict[str, float] = {}
    job.home_path.parent.mkdir(parents=True, exist_ok=True)
    anchor_rows = 0
    anchors_max = 0
    try:
        with MemoryHome.open(job.home_path, job.instance, page_cache_kib=page_cache_kib) as home:
            for person in sorted(job.group, key=lambda one: one.key):
                retention = resolve(job.choice, PolicyRegistry.installed(), seat=person.key)
                stack = resolve_stack(job.choice, ScopeRegistry.installed(), seat=person.key)
                anchors_max = _anchors_max(stack)
                sampled = str(person.key) in job.sampled
                watched = sampled and spec.continuity
                stream = frames(spec.world, job.save, person.key, spec.frame, split_days=watched)
                identity, first = _identity(job.instance, person, stream)
                view = home.seat(identity)
                home.bind_scopes(view, stack)
                facts = passes = largest = after = 0
                empty_days: list[int] = []
                met: Counter[EntityId] = Counter()
                names: dict[EntityId, str] = {}
                """The C-8 oracle's names: from the `named` facts this seat perceived, never from memory."""
                ids: list[EventId] = []
                for frame in chain(first, stream):
                    began = time.monotonic()
                    report = loop.run_until_complete(absorb(view, frame, frame[-1].id, retention))
                    wall["absorb"] = wall.get("absorb", 0.0) + time.monotonic() - began
                    facts += len(frame)
                    passes += len(report.passes)
                    largest = max(largest, view.size())
                    after = max([after, *(one.bytes_after for one in report.passes)])
                    if watched:
                        for one in frame:
                            met.update(set(one.participants) | set(one.subjects))
                            ids.append(one.id)
                            found = _named(one)
                            if found is not None:
                                names[found[0]] = found[1]
                        last = frame[-1]
                        day = last.at // DAY
                        if (
                            len(frame) > 1
                            and frame[-2].at // DAY < day
                            and day >= CONTINUITY_FROM_DAY
                        ):
                            section = recall(
                                view, stack, RecallQuery(at=last.at), on_defect="raise"
                            ).section
                            if not any(line.level == "L2" for line in section.lines):
                                empty_days.append(day)
                most_met_in_l3: bool | None = None
                unexplained: int | None = None
                if watched:
                    most = _counterpart(met, identity.self_entity)
                    section = recall(
                        view, stack, RecallQuery(at=spec.days * DAY), on_defect="raise"
                    ).section
                    name = names.get(most, "") if most is not None else ""
                    most_met_in_l3 = bool(name) and any(
                        line.level == "L3" and name in line.text for line in section.lines
                    )
                    perceived = frozenset(ids)
                    drawn = (
                        [ids[draw(SAMPLE_SEED, n, 3) % len(ids)] for n in range(EXPLAINED_IDS)]
                        if ids
                        else []
                    )
                    unexplained = sum(
                        explain(view, event, perceived).kind == "never-perceived" for event in drawn
                    )
                dump = hashlib.sha256(view.dump()).hexdigest() if sampled else None
                results.append(
                    SeatResult(
                        seat=str(person.key),
                        facts=facts,
                        passes=passes,
                        largest_size=largest,
                        largest_after_pass=after,
                        size=view.size(),
                        dump_sha256=dump,
                        l2_empty_days=tuple(empty_days),
                        most_met_in_l3=most_met_in_l3,
                        unexplained=unexplained,
                    )
                )
                anchor_rows = max(anchor_rows, _anchor_rows(home, view))
            home.reclaim()
            began = time.monotonic()
            physical = sum(len(text.encode("utf-8")) + 1 for text in home.dump_lines())
            wall["dump"] = time.monotonic() - began
            logical = sum(home.view_of(EntityKey(one.seat)).size() for one in results)
    finally:
        loop.close()
    wall["group"] = time.monotonic() - started
    return HomeResult(
        path=str(job.home_path),
        seats=tuple(results),
        physical=physical,
        logical=logical,
        file_bytes=_file_bytes(job.home_path),
        peak_rss=peak_rss_bytes(),
        anchor_rows_max=anchor_rows,
        anchors_max=anchors_max,
        wall=dict(wall),
    )


@dataclass(frozen=True)
class RebuildJob:
    spec: ScaleSpec
    save: Path
    instance: WorldInstanceId
    person: Person
    path: Path
    choice: MemoryPolicyChoice


def rebuild_alone(job: RebuildJob) -> str:
    """The seat rebuilt alone in a fresh one-seat home from a null cursor, frames of 256: its view's sha256."""
    loop = asyncio.new_event_loop()
    try:
        with MemoryHome.open(
            job.path, job.instance, page_cache_kib=MemoryConfig().page_cache_kib
        ) as home:
            retention = resolve(job.choice, PolicyRegistry.installed(), seat=job.person.key)
            stack = resolve_stack(job.choice, ScopeRegistry.installed(), seat=job.person.key)
            stream = frames(
                job.spec.world, job.save, job.person.key, REBUILD_FRAME, split_days=False
            )
            identity, first = _identity(job.instance, job.person, stream)
            view = home.seat(identity)
            home.bind_scopes(view, stack)
            for frame in (*first, *stream):
                loop.run_until_complete(absorb(view, frame, frame[-1].id, retention))
            return hashlib.sha256(view.dump()).hexdigest()
    finally:
        loop.close()


# ── The whole run ───────────────────────────────────────────────────────────────────────────────


def _block(key: EntityKey) -> str:
    found = re.search(r"-(\d\d)$", str(key))
    return found.group(1) if found else "01"


def groups(spec: ScaleSpec, everyone: Sequence[Person]) -> dict[str, tuple[Person, ...]]:
    if spec.homes == "one":
        return {"memory": tuple(everyone)}
    out: dict[str, list[Person]] = {}
    for person in everyone:
        out.setdefault(f"block-{_block(person.key)}", []).append(person)
    return {name: tuple(members) for name, members in sorted(out.items())}


def sampled_seats(everyone: Sequence[Person], count: int) -> list[str]:
    """`count` keys drawn with SplitMix64 (seed 7) over the sorted keys, without repeats."""
    keys = sorted(str(one.key) for one in everyone)
    chosen: list[str] = []
    n = 0
    while len(chosen) < min(count, len(keys)):
        key = keys[draw(SAMPLE_SEED, n, 5) % len(keys)]
        if key not in chosen:
            chosen.append(key)
        n += 1
    return chosen


def run_scale(spec: ScaleSpec, scratch: Path) -> ScaleRun:
    wall: dict[str, float] = {}
    began = time.monotonic()
    save, instance = run_world(spec, scratch)
    wall["run"] = time.monotonic() - began
    everyone = people(spec.world)
    choice = memory_choice(spec.world)
    bounds = resolve(choice, PolicyRegistry.installed()).bounds
    sampled = sampled_seats(everyone, spec.sample)
    store = scratch / "store" / str(instance)
    jobs = [
        GroupJob(
            spec, save, instance, members, store / f"{name}.sqlite", frozenset(sampled), choice
        )
        for name, members in groups(spec, everyone).items()
    ]
    by_key = {str(one.key): one for one in everyone}
    rebuilds = [
        RebuildJob(
            spec,
            save,
            instance,
            by_key[key],
            scratch / "alone" / str(instance) / f"{key}.sqlite",
            choice,
        )
        for key in sorted(sampled, reverse=True)
    ]
    (scratch / "alone" / str(instance)).mkdir(parents=True, exist_ok=True)
    with ProcessPoolExecutor(
        max_workers=max(1, spec.workers), mp_context=get_context("spawn")
    ) as pool:
        began = time.monotonic()
        homes = tuple(pool.map(absorb_group, jobs))
        wall["absorb"] = time.monotonic() - began
        began = time.monotonic()
        rebuilt = dict(
            zip(
                (job.person.key for job in rebuilds), pool.map(rebuild_alone, rebuilds), strict=True
            )
        )
        wall["rebuild"] = time.monotonic() - began
    return ScaleRun(
        spec,
        homes,
        {str(k): v for k, v in rebuilt.items()},
        bounds,
        MemoryConfig().page_cache_kib,
        wall,
    )


# ── The checks (§5.5) ───────────────────────────────────────────────────────────────────────────


def check(run: ScaleRun) -> list[str]:
    """Every S-n violation of the run, as text; empty when every check holds."""
    bounds = run.bounds
    out: list[str] = []
    for home in run.homes:
        for seat in home.seats:
            if seat.largest_size > bounds.bound:
                out.append(f"S-1 {seat.seat}: view {seat.largest_size} B > bound {bounds.bound} B")
            if seat.largest_after_pass > bounds.target:
                out.append(
                    f"S-1 {seat.seat}: after a pass {seat.largest_after_pass} B > target {bounds.target} B"
                )
            if seat.seat in run.rebuilt and seat.dump_sha256 != run.rebuilt[seat.seat]:
                out.append(
                    f"S-5 {seat.seat}: rebuilt alone {run.rebuilt[seat.seat]} != in the run {seat.dump_sha256}"
                )
            if run.spec.continuity and seat.dump_sha256 is not None:
                if seat.l2_empty_days:
                    out.append(f"S-8 {seat.seat}: L2 empty on days {list(seat.l2_empty_days)}")
                if seat.most_met_in_l3 is False:
                    out.append(f"S-8 {seat.seat}: the most-met counterpart is not in L3")
                if seat.unexplained:
                    out.append(
                        f"S-9 {seat.seat}: {seat.unexplained} of {EXPLAINED_IDS} ids unexplained"
                    )
        seats = len(home.seats)
        if not home.physical <= home.logical <= seats * bounds.bound:
            out.append(
                f"S-2 {home.path}: physical {home.physical} B, logical {home.logical} B, {seats} seats"
            )
        limit = math.floor(home.physical * K_FILE) + FILE_SLACK
        if home.file_bytes > limit:
            out.append(
                f"S-3 {home.path}: file {home.file_bytes} B > {limit} B (physical * {K_FILE} + 8 MiB)"
            )
        rss = run.page_cache_kib * 1024 + CACHE_RECORDS * RECORD_BYTES + RSS_SLACK
        if home.peak_rss > rss:
            out.append(f"S-4 {home.path}: peak RSS {home.peak_rss} B > {rss} B")
        if home.anchor_rows_max > home.anchors_max:
            out.append(
                f"S-6 {home.path}: {home.anchor_rows_max} anchor rows > anchors_max {home.anchors_max}"
            )
        if run.spec.continuity and home.physical > STEADY_STATE * home.logical:
            out.append(
                f"S-7 {home.path}: physical {home.physical} B > {STEADY_STATE} * logical {home.logical} B"
            )
    return out


def timings(run: ScaleRun) -> dict[str, object]:
    """The night's measurements (advisory, never asserted): walls, rates, bytes, ratios, peaks."""
    facts = sum(seat.facts for home in run.homes for seat in home.seats)
    absorb_s = run.wall.get("absorb", 0.0)
    return {
        "world": run.spec.world.name,
        "days": run.spec.days,
        "homes_mode": run.spec.homes,
        "workers": run.spec.workers,
        "platform": sys.platform,
        "cpus": os.cpu_count(),
        "wall_s": run.wall,
        "facts": facts,
        "facts_per_s_absorb": facts / absorb_s if absorb_s else None,
        "k_file": K_FILE,
        "homes": [
            {
                "path": Path(home.path).name,
                "seats": len(home.seats),
                "physical": home.physical,
                "logical": home.logical,
                "physical_over_logical": home.physical / home.logical if home.logical else None,
                "file_bytes": home.file_bytes,
                "file_over_physical": home.file_bytes / home.physical if home.physical else None,
                "peak_rss": home.peak_rss,
                "anchor_rows_max": home.anchor_rows_max,
                "wall_s": home.wall,
            }
            for home in run.homes
        ],
    }


def write_timings(run: ScaleRun, name: str) -> Path:
    out = REPOSITORY / "artifacts" / "nightly" / f"timings-{name}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    existing: dict[str, object] = {}
    if out.is_file():
        existing = cast(dict[str, object], json.loads(out.read_text(encoding="utf-8")))
    existing[run.spec.world.name] = timings(run)
    out.write_text(json.dumps(existing, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out
