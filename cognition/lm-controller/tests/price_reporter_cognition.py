"""The cognition process the price clerk's swap test starts: `python -m mineworld_cognition`, with a scripted model.

`tools/cli/tests/price_reporter_lm.rs` (S31-d C7, AC-d-11) runs this file as a process, with the workspace's
interpreter and `--config FILE`, exactly as an operator runs `python -m mineworld_cognition`, as
`milestone_d_cognition.py` is for Milestone D. Everything but the model is the shipped controller: the
assignment, the seat runner, the triggers (the `due` trigger on `day-began`), the fillers, the consult engine and
the decision log. The configuration says `recording.mode = "scripted"`, so no adapter is built and no network
client exists (`build_router`).

The model is `scripted_reporter.reporter`, the pure function the committed cassette was recorded from: for each
kind of `MY PRICE BOOK` with a sale shown, the median of the sales shown, resting on every note shown.
"""

from __future__ import annotations

import sys

from scripted_reporter import reporter

from mineworld_cognition.__main__ import main
from mineworld_cognition.backend.scripted import ScriptedBackend

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:], scripted=ScriptedBackend(reporter)))
