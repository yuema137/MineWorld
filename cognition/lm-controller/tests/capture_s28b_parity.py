"""AC-b-2's frozen parity evidence (pr-s28b-retention-on-seat-views.md §8.2, C0), and the shared-home runs it is
taken on.

`shared(policy, parameters)` runs the synthetic 12-seat world (`home_streams.world(7, 30)`) through `absorb` in
**one shared home**, the seats' frames of 256 interleaved by seed 1 (`home_streams.interleave`), every seat under
the same policy, and returns each seat view's dump at the end. `on_pass`, when given, sees every seat right after
each of its retention passes (the pass `absorb` runs is wrapped, in this process only, as `home_harness` does).

`tests/golden/s28b_parity.json` was written by running this module once, at C0, on S28-a's merged code, before any
change of S28-b (`tests/golden/SOURCE.md` records the command and the commit): per (policy, seat), the sha256 of
the seat view's dump at the end of the run. From C2 on, `test_retention_forgotten.py` recomputes the dumps through
`shared`, removes the columns S28-b adds (and maps `meta.schema` back), and compares with the file: attribution
never changes what is forgotten (I-b-2). The file is the oracle; this module only computes.
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
        seat: retention(policy, **parameters) if policies is None else policies(seat)
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


def digests() -> dict[str, dict[str, str]]:
    return {
        policy: {
            seat: hashlib.sha256(dump).hexdigest()
            for seat, dump in shared(policy, parameters).items()
        }
        for policy, parameters in POLICIES.items()
    }


def main() -> None:
    """Writes the golden once (C0). Run from the repository root:
    `uv run --locked python cognition/lm-controller/tests/capture_s28b_parity.py --write`."""
    if sys.argv[1:] != ["--write"]:
        raise SystemExit("usage: capture_s28b_parity.py --write (C0 only; the file is the oracle)")
    found = {"seed": SEED, "days": DAYS, "frame": FRAME, "interleaving": INTERLEAVING}
    found |= {"policies": POLICIES, "dumps": digests()}
    GOLDEN.write_text(json.dumps(found, indent=1, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
