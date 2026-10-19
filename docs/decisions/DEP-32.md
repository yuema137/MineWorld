## DEP-32 — Reading a `.env`-style key file: `python-dotenv`'s `dotenv_values`, never `load_dotenv`

**Date** 2026-10-09 · **Status** adopted by S10 PR P5a (`python-dotenv>=1.2,<2`; locked 1.2.4) ·
**Approved by** the operator's requirement of 2026-10-09 (API keys through a `.env`-style file) and the
primary session at the P5a freeze (QP5-6, QP5-10 … QP5-12) · **Licence** BSD-3-Clause · **Relates to**
`DEP-27`, `ARCHITECTURE.md` §9.2 · **Design** `pr-s10-p5-backends.md` §4.2b, D-P5-9, D-P5-14, AP5-9,
AP5-13

**Problem.** A user's hosted API needs a key. The configuration names only the **variable** (`key_env`);
its value comes from the process environment, or from a `.env`-style file the operator names. That
file's format (quoting, `export` prefixes, comments, multiline values, CRLF) is easy to get subtly wrong.

**Options considered:**

| Candidate | Licence; maturity | Verdict |
| --- | --- | --- |
| **`python-dotenv`** | **BSD-3-Clause; 1.2.4; no runtime dependencies** | **Adopted**, one call: `dotenv_values(path, interpolate=False)`, which returns a mapping and does not touch `os.environ` |
| our own parser | ~40 lines | Rejected: the quoting and escaping corner cases would be ours to find (`REUSE_POLICY.md` §12) |
| `pydantic-settings` | MIT | Rejected: it binds every settings field to environment variables; our file is TOML and only key values come from the environment |
| `environs` | MIT | Rejected: wraps `python-dotenv` and mutates `os.environ` by default |
| process environment only | — | Kept as one of two sources, not the only one: the operator asked for `.env`-style files |

**Choice.** `secrets.py` is the only importer of `dotenv` and the only reader of a key. The process
environment wins over the file (`python-dotenv`'s own `override=False` default). Never `load_dotenv`, so
a file's values never become ambient credentials for a child process or for another library's implicit
read; never interpolation; never a default file location. A key file inside a directory holding a
`world.yaml` is refused (credentials never travel with a world), and on macOS and Linux a file readable
by group or others is refused, as `ssh` refuses a key. A key value lives in a `Secret` whose `repr` is
redacted, and is never logged, recorded, put in an exception, or written to the ledger.

**Revisit** if `python-dotenv` gains runtime dependencies or changes `dotenv_values`' contract, or if
an OS keychain becomes a required source (a separate decision).
