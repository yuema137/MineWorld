//! `price-reporter`: the price clerk's deterministic decision model (`ARC-116` note; `ARC-117`;
//! `pr-s31d-reporter-policies.md` §§5.2, 5.3).
//!
//! A clerk is a Person in a seat; this model is one backend for it, and `lm` is another (`ARC-183`). One
//! consult:
//!
//! ```text
//! 1  publish   a publish slot lies in (t − pace, t], publish-notice is offered available, my own
//!              survey-book decodes and yields lines        → publish-notice { topic, prices, valid_for }
//! 2  survey    survey is offered complete and available     → that affordance's own request
//! 3  daily     `rule` with its default parameters, unchanged (pricing hold): no new draw
//! ```
//!
//! ```text
//! parameters.rs   the block: topic, the slots, validity, the statistic's knobs
//! suggest.rs      one line per kind: the sale median, else the deduped ask median; the supply adjustment
//! model.rs        the three bands
//! ```
//!
//! It holds no state, uses no float, and reads only its own observation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::float_arithmetic)]

mod model;
mod parameters;
mod suggest;

pub use model::PriceReporter;
pub use parameters::ReporterParameters;
pub use suggest::suggest;

/// This Controller Pack's package identity, from its `Cargo.toml` (`docs/DECISIONS.md` `ARC-53`).
pub const PACKAGE: mineworld_packages::Package = mineworld_packages::package!();

#[cfg(test)]
mod tests;
