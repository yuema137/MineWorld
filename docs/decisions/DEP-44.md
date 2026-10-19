## DEP-44 — The resuming seat's reconnect loop is our own, over the adopted `websockets` transport

**Date** 2026-10-10 · **Status** adopted by S10 PR P3b (no new dependency; `uv.lock` unchanged) ·
**Approved by** the primary session at P3b's freeze (Q-P3b-1, 2026-10-10) · **Relates to** `DEP-25`,
`ARC-56` · **Design** `pr-s10-p3b-perceived.md` §4.3 (D-P3b-5, D-P3b-9, D-P3b-10), §4.5 · **Placeholder**
`DEP-P3b-a`

**Problem.** A Python seat that must not miss a fact rejoins after a dropped socket, a `lagged` refusal
or a server restart (`server/PROTOCOL.md` §§4.2, 5.5, 5.8). Which outcomes are retried, whether a rejoin
carries `resume`, which cursor it presents, and when a changed world instance ends the seat are protocol
decisions; the delays between attempts are the small remainder.

**Options considered** (facts read 2026-10-10 from the installed or cached distributions):

| Candidate | Licence (from its own metadata); version | Verdict |
| --- | --- | --- |
| `websockets`' reconnecting iterator, `async for connection in connect(url)` | BSD-3-Clause (`License-Expression`, 17.2, already `DEP-25`) | **Declined** for the protocol loop. `websockets/asyncio/client.py` `connect.__aiter__`: it backs off only when opening a connection fails (`process_exception` decides which failures); a connection that ends after it was yielded is reopened at once and resets the backoff, and nothing in it knows `join`, `resume`, the hold, the cursor or a world instance — the whole problem. Its default delays (`websockets/client.py` `backoff`) come from module globals read from `WEBSOCKETS_BACKOFF_*` environment variables at import (`D-P3-9`: the SDK reads no environment), the first `random.random() * 5` s from the unseeded global generator, then 3.1 s × 1.618 up to 90 s — past the server's default 30 s hold. A `reconnect_delays` callable can replace that sequence, but the sleep is `asyncio.sleep`, not injectable for a fake clock, and each retry is logged through the library's logger |
| `tenacity` | Apache-2.0 (`License: Apache 2.0`, 9.1.4) | **Declined**: a dependency for a capped exponential sequence of a few lines; the decision of *what* to retry stays ours either way, and it would put its own sleep and clock in the path the scripted tests replace |
| `backoff` | MIT (`License: MIT`, 2.2.1) | **Declined**, for the same reasons |
| **Our loop**: `mineworld_sdk/resuming.py` (the seat) and `mineworld_sdk/reconnect.py` (the table and the delays) | — | **Adopted** |

**Choice.** `ResumingSeat` owns the loop (`pr-s10-p3b-perceived.md` §4.5's table is the whole retry
policy). Its delays: the first attempt after a loss is immediate, then `first_delay × factor^k` capped
at `ceiling`, each scaled by a factor drawn from `random.Random(policy.seed)` (per seat), until
`give_up_after` wall seconds from the loss. The sleep and the clock are injectable (default
`asyncio.sleep` and the running loop's `time()`); no environment variable is read. The transport stays
`websockets` (`DEP-25`) with its keepalive; `session.py` remains the only importer of `websockets` in the
library.

**Revisit** when a second protocol client in Python needs the same loop: it then moves, unchanged, into a
shared module, not into a library.

**Dated note, 2026-10-14 (S10 P6b, C1): retry and breaker libraries declined.** For the LM controller's
layered retries (`ARC-73`) and its per-seat circuit breaker, the following are declined, and our own code
is built instead (`pr-s10-p6-lm-controller.md` §4). Licences were read on 2026-10-14 from each project's
PyPI metadata; none was installed or added, and `uv.lock` is unchanged.

| Candidate | Licence (PyPI metadata, 2026-10-14); latest version | Verdict |
| --- | --- | --- |
| `tenacity` | Apache-2.0; 9.2.1 | **Declined.** It owns the loop and the sleep, while the controller needs per-class budgets under one deadline and attempt cap, seeded jitter, an injected clock, and a record per attempt |
| `backoff` | MIT; 2.2.1 | **Declined**, for the same reasons |
| `stamina` | MIT; 26.1.0 (wraps `tenacity`) | **Declined**, for the same reasons as `tenacity` |
| `pybreaker` | BSD (classifier "BSD License", no licence field); 1.4.1 | **Declined.** It counts HTTP-call failures; the controller's breaker counts consult outcomes, excludes budget refusals, and runs its cooldown in simulated time |
| `aiobreaker` | BSD; 1.2.0 | **Declined**, for the same reason as `pybreaker` |
| `purgatory` | MIT; 3.0.1 | **Declined**, for the same reason as `pybreaker` |

The licence of `pybreaker` and `aiobreaker` is taken from their PyPI metadata as written; their licence
files were not read in this session.
