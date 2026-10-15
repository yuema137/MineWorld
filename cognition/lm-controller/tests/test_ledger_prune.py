"""AFGb-10: the budget ledger prunes itself by wall time and no window changes (pr-fg-b §5.12; G9).

The oracle is `MemoryLedger`, which never forgets: fed the same charges, every window `precheck` reads
(an hour, a day) at every charged moment must agree with the pruned `SqliteLedger`.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.budget import DAY_S, HOUR_S, MemoryLedger, SqliteLedger

ALICE = EntityKey("alice")
BOB = EntityKey("bob")
CHARGES = (0, HOUR_S, DAY_S, 2 * DAY_S + 1, 3 * DAY_S)


def test_old_rows_go_and_every_window_stays_what_it_was(tmp_path: Path) -> None:
    # Its mutation prunes at `latest - DAY_S / 2`: the day window at 3 d loses the charge at 2 d + 1.
    path = tmp_path / "ledger.sqlite"
    pruned, oracle = SqliteLedger(path), MemoryLedger()
    try:
        for index, at in enumerate(CHARGES):
            for seat in (ALICE, BOB):
                tokens = 100 * (index + 1) + (7 if seat == BOB else 0)
                pruned.charge(seat, at, calls=1, tokens=tokens, estimated=False)
                oracle.charge(seat, at, calls=1, tokens=tokens, estimated=False)
            for seat in (ALICE, BOB):
                for seconds in (HOUR_S, DAY_S):
                    assert pruned.window(seat, at, seconds) == oracle.window(seat, at, seconds), (
                        seat,
                        at,
                        seconds,
                    )
    finally:
        pruned.close()
    connection = sqlite3.connect(path)
    try:
        kept = connection.execute("SELECT at FROM budget_ledger ORDER BY at, seat").fetchall()
    finally:
        connection.close()
    # After the charge at 3 d, rows before 3 d - 2 d = 1 d are gone: those at 0 and 1 h, for both seats.
    assert [at for (at,) in kept] == [
        DAY_S,
        DAY_S,
        2 * DAY_S + 1,
        2 * DAY_S + 1,
        3 * DAY_S,
        3 * DAY_S,
    ]
