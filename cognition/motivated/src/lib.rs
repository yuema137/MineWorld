//! `motivated`: a no-LLM decision model whose daily rhythm is calibrated from the American Time Use
//! Survey, Table A-3 (`docs/DECISIONS.md` `DEP-47`, `DEP-48`; `pr-dl-b-motivated.md`).
//!
//! It scores every option an observation offers by multiplying integer considerations (input → response
//! curve → 0 … 1 000), picks by a seeded weighted draw among the best, and takes its daily rhythm from the
//! ATUS curves. It holds no state, uses no float, and reads only what observations already disclose.
//!
//! ```text
//! data/atus-a3-2025.tsv   the six rows, per-mille, checksummed (compiled in with include_str!)
//! curve                   a piecewise-linear integer response curve, clamped at both ends
//! curves                  Table::parse, from_a3, and the curve set the model names
//! settings                the calibrated and authored curves, base weights and role tags
//! options, considerations the choices a consult offers, and the values that score them
//! score                   compensation (Mark's factor, in integers) and the top-band draw
//! model                   one consult: invitation, reply, then the choice
//! explain                 every choice with its considerations and score (the test oracle)
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::float_arithmetic)]

mod considerations;
mod curve;
mod curves;
mod explain;
mod model;
mod options;
mod parameters;
mod score;
mod settings;

/// This Controller Pack's package identity, from its `Cargo.toml` (`docs/DECISIONS.md` `ARC-53`).
pub const PACKAGE: mineworld_packages::Package = mineworld_packages::package!();

pub use curve::{Curve, CurveError, POINTS_AT_MOST, SCALE};
pub use curves::{CURVE_NAMES, CurveSet, EDITION, TABLE, Table, TableError, from_a3};
pub use explain::explain;
pub use model::{MOTIVATED_SALT, Motivated, decide, step};
pub use options::Choice;
pub use parameters::MotivatedParameters;
pub use score::{Scored, combine};
pub use settings::{Base, Curves, Roles, Settings};

#[cfg(test)]
mod parameters_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod commute_tests;
