//! The decision-model extension point: how each seat decides, as a selectable, typed choice
//! (`docs/MODULE_SPEC.md` §5.1; `docs/DECISIONS.md` `ARC-84`, `ARC-85`).
//!
//! ```text
//! DecisionModel, SeatModel, Configured   the trait every in-process model implements, and what a
//!                                        host holds per seat (model.rs)
//! Seed, Consult                          what a consult is given besides the parameters
//! Draw, mix                              the seeded, counter-based draws (draw.rs)
//! ModelParameters, bounded               typed, bounded parameters (parameters.rs)
//! ```
//!
//! A model is a Controller Pack's model, never a new pack type. It reads an `Observation` and returns at
//! most one `ActionRequest`; what it may attempt is whatever an installed System offered, revalidated at
//! dispatch, so no model can grant itself an action (`INV-10`). The rule controller is the first model
//! (`rule`), and with no decision list every seat is `rule` and every byte of every world as before.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod draw;
mod model;
mod parameters;

pub use draw::{Draw, mix};
pub use model::{Configured, Consult, DecisionModel, ModelId, SeatModel, Seed};
pub use parameters::{ModelParameters, bounded};

#[cfg(test)]
mod draw_tests;

#[cfg(test)]
mod parameters_tests;
