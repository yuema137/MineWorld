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
mod format;
mod json;
mod model;
mod models;
mod parameters;
mod read;
mod resolve;

pub use draw::{Draw, mix};
pub use format::{ClassRef, JsonObject, PolicyId, Position, Roster, SeatEntry};
pub use json::{
    JSON_FORMAT, MemoryPolicyChoiceJson, ModelJson, ResolvedDecisionsJson, SeatJson, SourceSpanJson,
};
pub use mineworld_sdk::Package;
pub use model::{Configured, Consult, DecisionModel, ModelId, SeatModel, Seed};
pub use models::{Catalog, Decoded};
pub use parameters::{ModelParameters, bounded};
pub use read::{CHAIN_LINKS, DecisionList, DecisionsError, Fallback, MemoryChoice};
pub use resolve::{
    MemoryPolicyChoice, Origin, ResolvedDecisions, ResolvedSeat, SeatOverride, Source, SourceSpan,
};

/// What the [`models!`] macro expands to. Not part of the API.
#[doc(hidden)]
pub mod __private {
    pub use serde::de::{Deserializer, Error as DeError};
}

#[cfg(test)]
mod draw_tests;

#[cfg(test)]
mod parameters_tests;

#[cfg(test)]
mod read_tests;
