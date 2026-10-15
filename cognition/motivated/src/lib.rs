//! `motivated`: a no-LLM decision model whose daily rhythm is calibrated from the American Time Use
//! Survey, Table A-3 (`docs/DECISIONS.md` `DEP-47`, `DEP-48`; `pr-dl-b-motivated.md`).
//!
//! This crate, so far, holds the calibration: the committed table, its parser, its checksums, and the pure
//! derivation into the shipped curve set `atus-a3-2025`. All values are per-mille integers. No float appears
//! anywhere in the crate (I-DLb-2).
//!
//! ```text
//! data/atus-a3-2025.tsv   the six rows, per-mille, checksummed (compiled in with include_str!)
//! curve                   a piecewise-linear integer response curve, clamped at both ends
//! curves                  Table::parse, from_a3, and the curve set the model will name
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::float_arithmetic)]

mod curve;
mod curves;

pub use curve::{Curve, CurveError, POINTS_AT_MOST, SCALE};
pub use curves::{CURVE_NAMES, CurveSet, EDITION, TABLE, Table, TableError, from_a3};
