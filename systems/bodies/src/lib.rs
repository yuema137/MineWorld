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
//! Loose objects (step-11 §18): an item file's `body:` section is one physical object (`ARC-36` note).
//! Walking into one pushes it aside — predicted by the resolver, made by this pack's reaction to each
//! `arrived` — and a person may `kick` or `throw` it, or `shove` another person.
//!
//! ```text
//! section     body           a place file's floor and solids, or an item file's object (ARC-31)
//! emits       place-shaped   at genesis, from a place's section
//!             body-formed    at genesis, from an item's section; object-placed in the next generation
//!             object-moved   pushed (a reaction to arrived), kicked or thrown (an action)
//!             person-shoved  an action; and presence's arrived, stopped-short through arrivals()
//! owns        PlaceShape     `place-shape`, after checking the people authored into the place
//!             BodyShape      `body-shape`, on the object's Item
//!             LooseObjects   `loose-objects`, on the Place, after SD-O5's checks
//! provides    kick, throw    target-less, the object in the payload; shove, a person
//! discloses   PlaceShape     and a listing of the place's objects, to whoever perceives the place
//! resolves    arrivals       into a shaped place (presence's ArrivalResolver)
//! plans       routes         through a shaped place (movement's Wayfinder; route.rs, DEP-34)
//! depends on  presence       (the crate reads the item pack's is_declared, nothing else: ARC-39 note 2;
//!                            and implements movement's Wayfinder: ARC-39 note 5)
//! ```
//!
//! Rapier sweeps people, casts pushed objects and flies kicked and thrown ones, behind one module
//! (`rapier.rs`), integers in and integers out. Everything else — what is clear, who is nudged and how
//! far, whether a result keeps the invariants — is integer geometry (`geometry.rs`, `footprint.rs`).

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
mod route;
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
pub use route::{PLAN_MARGIN, WAYPOINTS_MAX, route_in};
pub use section::{Body, Lies};
pub use system::BodiesSystem;
