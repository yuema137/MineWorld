//! People have bodies: places have walls and furniture, and nobody passes through them or through
//! each other.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-39`, `DEP-13`), and the build's first registered
//! `ArrivalResolver`. Before presence records an arrival into a place that has a shape, this pack
//! answers what the arrival actually achieves: the walker stops at walls and furniture, the people in
//! the way are nudged aside within fixed bounds, and the integer result is verified and degraded
//! when a pair would overlap. So the log holds only arrivals in which no two people overlap and
//! nobody stands in a wall. A place without a shape — every place of a world that does not enable
//! this pack — is untouched, byte for byte.
//!
//! ```text
//! section     body           a place file's floor and solids (ARC-31)
//! emits       place-shaped   at genesis, from the section
//! owns        PlaceShape     `place-shape`, reduced from `place-shaped` by this pack alone, after
//!                            checking the people authored into the place
//! discloses   PlaceShape     to whoever perceives the place, so a client builds the server's walls
//! resolves    arrivals       into a shaped place (presence's ArrivalResolver)
//! depends on  presence
//! ```
//!
//! Rapier sweeps the walker and the nudged people, behind one module (`rapier.rs`), integers in and
//! integers out. Everything else — what is clear, who is nudged and how far, whether the result is
//! acceptable — is integer geometry (`geometry.rs`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod resolve;
mod section;

pub mod component;
pub mod event;
pub mod geometry;
// The adapter is reached by the resolver (S15 PR 12b, PB-C4); until then only its tests use it.
#[allow(dead_code)]
mod rapier;
pub mod system;

pub use component::{Corner, Floor, PlaceShape, SOLID_HEIGHT_MAX, SOLIDS_MAX, Solid};
pub use event::{PlaceShaped, place_shaped};
pub use geometry::{
    CAPACITY_GRID, CHAIN_MAX, CLEARANCE, COORDINATE_BOUND, GAP, HALVINGS, LATTICE, NUDGE_MAX,
    NUDGED_MAX, PERSON_HEIGHT, PERSON_RADIUS, SNAP, TOLERANCE,
};
pub use system::BodiesSystem;
