"""Session-scoped fixtures shared by test modules.

`social_cafe_100`: the 100-day social-cafe history of `AC-10` (`History.build`: `mineworld run`, `inspect`,
`perceived` for Alice and Bob), built once per session and shared by `test_ac10.py` and
`test_retention_real.py`, so the world runs once (pr-fg-b §8.4). `social_cafe_12`: the 2-day run with all
twelve people's exports (pr-s28a §5.9). Both are built lazily: a session that selects no `real_binary` test
never runs the binary.
"""

from __future__ import annotations

import pytest
from ac10_harness import History
from home_streams import Cafe12, cafe12


@pytest.fixture(scope="session")
def social_cafe_100(tmp_path_factory: pytest.TempPathFactory) -> History:
    return History.build(tmp_path_factory.mktemp("ac10"))


@pytest.fixture(scope="session")
def social_cafe_12(tmp_path_factory: pytest.TempPathFactory) -> Cafe12:
    """The twelve people of social-cafe, seed 7, 2 days (pr-s28a §5.9): one run and twelve exports per
    session, shared by the home's real-stream tests."""
    return cafe12(tmp_path_factory.mktemp("cafe12"))
