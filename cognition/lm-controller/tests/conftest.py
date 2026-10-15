"""Session-scoped fixtures shared by test modules.

`social_cafe_100`: the 100-day social-cafe history of `AC-10` (`History.build`: `mineworld run`, `inspect`,
`perceived` for Alice and Bob), built once per session and shared by `test_ac10.py` and
`test_retention_real.py`, so the world runs once (pr-fg-b §8.4). Built lazily: a session that selects no
`real_binary` test never runs the binary.
"""

from __future__ import annotations

import pytest
from ac10_harness import History


@pytest.fixture(scope="session")
def social_cafe_100(tmp_path_factory: pytest.TempPathFactory) -> History:
    return History.build(tmp_path_factory.mktemp("ac10"))
