"""AC-b-2's frozen parity evidence (pr-s28b-retention-on-seat-views.md §8.2, C0), and the shared-home runs it is
taken on.

`shared(policy, parameters)` runs the synthetic 12-seat world (`home_streams.world(7, 30)`) through `absorb` in
**one shared home**, the seats' frames of 256 interleaved by seed 1 (`home_streams.interleave`), every seat under
the same policy, and returns each seat view's dump at the end. `on_pass`, when given, sees every seat right after
each of its retention passes (the pass `absorb` runs is wrapped, in this process only, as `home_harness` does).

`tests/golden/s28b_parity.json` was written by running this module on S28-a's merged code, before any change of
S28-b to `src/` (`tests/golden/SOURCE.md` records the commands and the commit): per (policy, seat), the sha256 of
the seat view's **parity view** at the end of the run (`parity_view`), and, from C0's first capture, the sha256 of
the whole dump. From C2 on, `test_retention_forgotten.py` recomputes the dumps through `shared` and compares their
parity views with the file: attribution never changes what is forgotten (I-b-2). The file is the oracle; this
module only computes.

The parity view is the dump without what schema 5 necessarily changes (ledger D-b-2): the `meta.schema` line, the
`forgotten` columns after `last_key`, and the pass log's `bytes_before` and `bytes_after` (which measure the dump,
attribution included) and `prose_contradictions`. It is written from the positional grammar of both schemas
(`ARC-101`'s note), by column position, so one function reads a schema-4 and a schema-5 dump alike.
"""

from __future__ import annotations

import hashlib
import json
import sys
from collections.abc import Callable, Generator, Mapping
from contextlib import contextmanager
from pathlib import Path

import pytest
from home_streams import CAST, interleave, seat_streams, world
from memory_fixtures import INSTANCE
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey
from retention_driver import retention
from support import run

from mineworld_cognition.memory.home import MemoryHome, SeatView, Verify
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention import absorb as absorb_module
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.host import PassCause, PassReport, run_pass
from mineworld_cognition.memory.retention.selection import Retention

GOLDEN = Path(__file__).resolve().parent / "golden" / "s28b_parity.json"
SEED = 7
DAYS = 30
FRAME = 256
INTERLEAVING = 1
POLICIES: dict[str, dict[str, int]] = {
    "consolidate": {},
    "decay": {"threshold": 1000},
    "importance-recency-cap": {"k_l0": 0, "k_l1": 0, "k_l2": 0, "k_l3": 0},
}
"""AC-b-1's three policies, as the design fixes them: `decay` at threshold 1 000 (everything decays below it),
`importance-recency-cap` with every k at 0 (nothing is chosen, so every unit merges or drops)."""

type OnPass = Callable[[EntityKey, SeatView, PassReport], None]


def streams() -> dict[EntityKey, list[PerceivedEvent]]:
    return seat_streams(world(SEED, DAYS))


def identity(seat: EntityKey) -> StoreIdentity:
    return StoreIdentity(world_instance=INSTANCE, seat=seat, self_entity=CAST[seat])


@contextmanager
def watched(on_pass: OnPass | None) -> Generator[None]:
    """Wraps the pass `absorb` runs, so `on_pass` sees each seat right after each of its passes."""
    if on_pass is None:
        yield
        return
    hook = on_pass

    async def recorded(
        store: SeatView, kept: Retention, now: int, cause: PassCause = "day"
    ) -> PassReport:
        report = await run_pass(store, kept, now, cause)
        hook(store.identity.seat, store, report)
        return report

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(absorb_module, "run_pass", recorded)
        yield


def shared(
    policy: str,
    parameters: Mapping[str, int],
    *,
    on_pass: OnPass | None = None,
    policies: Callable[[EntityKey], Retention] | None = None,
    verify: Verify = "off",
) -> dict[EntityKey, bytes]:
    """Every seat a view of one shared in-memory home under `policy`; each seat view's dump at the end.
    `policies` replaces the one policy by a per-seat choice."""
    perceived = streams()
    kept = {
        seat: retention(policy, None, **parameters) if policies is None else policies(seat)
        for seat in perceived
    }
    views: dict[EntityKey, SeatView] = {}
    with watched(on_pass), MemoryHome.open(":memory:", INSTANCE, verify=verify) as home:
        for seat, batch in interleave(perceived, FRAME, INTERLEAVING):
            view = views.get(seat)
            if view is None:
                view = views[seat] = home.seat(identity(seat))
            run(absorb(view, batch, batch[-1].id, kept[seat]), timeout_s=600)
        return {seat: views[seat].dump() for seat in sorted(views)}


PASS_LOG_KEPT = (*range(1, 10), *range(12, 15))
"""The pass-log cells a parity view keeps, by position after the table name: `trigger` … `forced` (1 … 9), then
`forgotten_ranges`, `model_decisions`, `fallback_decisions` (12 … 14)."""


def parity_view(dump: bytes) -> bytes:
    """The dump without what schema 5 necessarily changes (module docstring), and without schema 6's
    `access_anchor` lines (pr-s28d C3: each held fact's payload anchors, a table schema 5 did not have)."""
    lines: list[str] = []
    for line in dump.decode("utf-8").splitlines():
        cells: list[object] = json.loads(line)
        if cells[:2] == ["meta", "schema"] or cells[0] == "access_anchor":
            continue
        if cells[0] == "forgotten":
            cells = cells[:3]
        elif cells[0] == "retention_passes":
            cells = [cells[0], *(cells[at] for at in PASS_LOG_KEPT)]
        lines.append(json.dumps(cells, ensure_ascii=False, separators=(",", ":")))
    return ("\n".join(lines) + "\n").encode("utf-8")


def digests() -> dict[str, dict[str, str]]:
    return {
        policy: {
            seat: hashlib.sha256(parity_view(dump)).hexdigest()
            for seat, dump in shared(policy, parameters).items()
        }
        for policy, parameters in POLICIES.items()
    }


def main() -> None:
    """Writes the golden once (C0). Run from the repository root:
    `uv run --locked python cognition/lm-controller/tests/capture_s28b_parity.py --write`."""
    if sys.argv[1:] != ["--write"]:
        raise SystemExit("usage: capture_s28b_parity.py --write (C0 only; the file is the oracle)")
    found: dict[str, object] = json.loads(GOLDEN.read_text(encoding="utf-8"))
    found |= {"seed": SEED, "days": DAYS, "frame": FRAME, "interleaving": INTERLEAVING}
    found |= {"policies": POLICIES, "views": digests()}
    GOLDEN.write_text(json.dumps(found, indent=1, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
