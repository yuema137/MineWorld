//! MineWorld's acceptance and conformance tests for its frozen criteria (`docs/ARCHITECTURE.md` §14).
//!
//! This crate holds no code. Its tests, under `tests/`, check properties that belong to no one crate:
//! what a set of pull requests changed and did not, or what several crates do together that none of
//! them can show alone:
//!
//! ```text
//! precursor_vocabulary   I-2: the framework PRs that precede the market name no market concept
//!                        (docs/DECISIONS.md ARC-35, point 7) — reads the repository and its git
//!                        history, and fails, naming what is missing, without it
//! complete_affordances   CP-3: a headless person attempts an action of a pack the controller was
//!                        never compiled against (docs/DECISIONS.md ARC-34) — a synthetic System
//!                        Pack defined in the test, driven by the real perception, controller and
//!                        kernel on the `mineworld run` schedule
//! ac1_composability      AC-1: Market Town is Social Café plus installed packs and configuration
//!                        (docs/DECISIONS.md ARC-35 and its notes) — the two transformation merges'
//!                        change set from git history (fails, naming it, without the full history),
//!                        the dependency structure from `cargo metadata`, and the world delta
//! arrival_resolvers      SC-2 … SC-5, SC-8: presence resolves an arrival before it records it
//!                        (docs/DECISIONS.md ARC-39) — synthetic resolvers (tests/resolvers/) driven
//!                        through the real kernel and movement pack, in a process that registers them
//! arrival_resolvers_unregistered
//!                        a process that never registers: a resolver's pack refuses to install, and
//!                        a world without one runs unchanged
//! arrival_resolvers_resume
//!                        SC-6: SIGKILL and resume with a resolver installed, byte for byte (a
//!                        program, harness = false)
//! seam_vocabulary        SC-7: the seam's sources name no physics, and movement names no resolver
//! client_rules           I-S14-1 and I-S14-6: no reference client names an action outside the
//!                        files that build requests, copies a rule as a constant, or reads a pack
//!                        (step-15 §19, PR 16a) — reads every client script
//! ```
//!
//! A test-only System Pack defined here is compiled into no library, which is what lets such a test
//! say that a crate it drives has never seen the pack.

#![forbid(unsafe_code)]
