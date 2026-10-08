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
mod entry;
mod flight;
mod footprint;
mod genesis;
mod launch;
mod objects;
mod offer;
mod push;
mod rapier;
mod section;
mod shove;
mod stride;

pub mod action;
pub mod component;
pub mod event;
pub mod geometry;
pub mod resolve;
pub mod system;

pub use action::{
    Kick, Shove, Throw, Toward, kick_requirement, shove_requirement, throw_requirement,
};
pub use component::{
    BodyShape, Corner, Floor, HalfExtents, LooseObjects, Lying, PlaceShape, SOLID_HEIGHT_MAX,
    SOLIDS_MAX, Solid,
};
pub use event::{
    BodyFormed, How, ObjectMoved, ObjectPlaced, PersonShoved, PlaceShaped, body_formed,
    place_shaped,
};
pub use geometry::{
    CAPACITY_GRID, CHAIN_MAX, CLEARANCE, COORDINATE_BOUND, GAP, HALVINGS, KICK_REACH, KICK_SPEED,
    KICK_STEPS, LATTICE, NUDGE_MAX, NUDGED_MAX, OBJECT_HALF_HEIGHT_MAX, OBJECT_HALF_MAX,
    OBJECT_HALF_MIN, OBJECTS_MAX, PATH_EVERY, PATH_MAX, PERSON_HEIGHT, PERSON_RADIUS, REST_SPEED,
    REST_STEPS, SHOVE_DISTANCE, SHOVE_REACH, SNAP, THROW_DEFAULT, THROW_FLIGHT, THROW_RANGE_MAX,
    THROW_REACH, THROW_STEPS, TOLERANCE,
};
pub use resolve::{Degraded, Objects, Outcome, Route, explain};
pub use section::{Body, Lies};
pub use system::BodiesSystem;
