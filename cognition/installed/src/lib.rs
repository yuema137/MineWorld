//! The decision models this build provides — the build's **installed models**, and its only list of them
//! (`docs/MODULE_SPEC.md` §5.1; `docs/DECISIONS.md` `ARC-33` note of 2026-10-13, `ARC-85`).
//!
//! Installing a model is one line in this crate's `Cargo.toml` and one line below, then a rebuild. Nothing
//! else learns the model: `decisions.yaml` model ids resolve through the [`Model`] catalog this expands to,
//! the hosts drive each seat through it, and `mineworld packs list` lists its packages. An in-process line
//! names a `DecisionModel` type; an `external "<id>"` line names a model that runs out of process and joins
//! a seat over the protocol (none in this build; step-17 P6 adds `lm`).
//!
//! A test (`tests/installed.rs`) holds this list and the manifest's dependencies equal, and refuses two
//! models with one id, an id outside the identifier rule, and the reserved `none`. The template test in
//! `tools/cli/tests/decision_models.rs` runs every in-process model listed here twice and across a
//! kill-and-resume, so a new model is held to determinism with no test edited.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use mineworld_decision::Catalog;

mineworld_decision::models! {
    Rule => mineworld_rule_controller::Rule,
    Motivated => mineworld_motivated::Motivated,
}
