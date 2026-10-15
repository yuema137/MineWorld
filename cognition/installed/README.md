# mineworld-installed-models

The decision models this build provides, one line each.

To install your own model: add its crate to this crate's `Cargo.toml`, add one line to the `models!` list
in `src/lib.rs`, rebuild, and name its id in a world's `decisions.yaml`. Nothing else changes, and the
determinism tests pick the new model up on their own.

The specification is [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §5.1.
