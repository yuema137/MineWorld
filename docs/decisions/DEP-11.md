## DEP-11 — The CLI's argument parsing: `clap`

**Date** 2026-10-06 · **Status** selected, integrated in PR 09 · **Design**
`.structured-coding/plans/mvp0/step-08-headless.md` §10.1 Q12

**Problem.** `mineworld` parses its own command line. Until S7 it had three subcommands and four
options, parsed by hand, and `tools/cli/src/main.rs` recorded when that would stop being right:
*"`clap` is the right answer the day `create` and `inspect` arrive with real option surfaces."*

**The trigger that was met.** S7 adds `run` (`--headless`, `--seed`, `--days`, `--save`), `inspect`
(`--last`) and `create`: six subcommands and ten options, with typed values (`u64`, a path, a socket
address), required and repeatable options, and help text per command. Hand parsing at that size is
re-implementing commodity infrastructure (`REUSE_POLICY.md`), and keeping it would have meant moving
the recorded trigger after reaching it.

**Options considered.** Hand parsing (status quo: grows with every option, and every refusal text
is ours to keep consistent); `pico-args` and `lexopt` (small, but still leave usage, help and
validation to us); `argh` (derive-based and small, but its conventions are Fuchsia's and its
ecosystem thin); **`clap` 4 with `derive`** (the ecosystem's standard, maintained, typed values,
generated help and errors that name the offending argument).

**Choice: `clap` 4 with the `derive` feature.** Dual MIT / Apache-2.0.

**Isolating interface.** `tools/cli/src/main.rs` alone: one derived `Cli` type and its subcommand
enum, turned into the command's own plain values before anything runs. No `clap` type appears in
any other module or crate, so replacing the parser is a change to one file.

**Accepted limitations.** Compile time and binary size grow by the parser's; the command's help
and error wording become `clap`'s format rather than hand-written prose.
