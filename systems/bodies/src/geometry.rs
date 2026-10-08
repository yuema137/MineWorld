//! Integer geometry: everything this pack decides about bodies, except one sweep.
//!
//! Every position here is whole millimetres in a place's frame, every product is `i64` or `i128`, and
//! every root is `isqrt`. No `sqrt`, `sin` or `cos` from `std` appears anywhere in this pack: a value
//! that reaches Rapier is either a literal or converted from these integers by one function, and a
//! value that comes back is quantized by one function (`rapier.rs`; `DEP-13`, step-11 DC-3, DC-6).
//! So what this file computes is exactly reproducible on any machine, and only the sweep is float.

use mineworld_contracts::Millimetres;

/// A person's radius: the 3D client's capsule (`player.gd`). Every person in a place with a shape is
/// this capsule in MVP-0 (step-11 QP-6).
pub const PERSON_RADIUS: Millimetres = Millimetres::new(300);

/// A person's height, feet to crown: the 3D client's capsule.
pub const PERSON_HEIGHT: Millimetres = Millimetres::new(1_720);

/// The character controller's offset: a walker stopped by something ends this far from touching it,
/// so `2 × PERSON_RADIUS + GAP` = 610 mm from a person who stopped them.
pub const GAP: Millimetres = Millimetres::new(10);

/// The invariant this pack keeps on integers: no two people in one place closer than this
/// (`2 × PERSON_RADIUS − 5 mm`). Also the bound authored people are refused under at genesis, so a
/// world that loads is a world the resolver accepts.
pub const CLEARANCE: Millimetres = Millimetres::new(595);

/// How close a centre may come to the floor's edge shrunk by the radius, or to a solid grown by it.
pub const TOLERANCE: Millimetres = Millimetres::new(5);

/// The most one arrival moves anybody else, on top of the overlap it creates: a nudge is at most
/// `NUDGE_MAX + GAP` = 310 mm (`ARC-39` note; step-11 QB-10, I-11).
pub const NUDGE_MAX: Millimetres = Millimetres::new(300);

/// Generations of nudges per arrival: a nudged person may nudge one further generation, no more.
pub const CHAIN_MAX: usize = 2;

/// People moved by one arrival, at most.
pub const NUDGED_MAX: usize = 4;

/// Verify, then degrade: how often the blocked advance is halved before the walker stays.
pub const HALVINGS: u32 = 8;

/// A sweep ending within this of its target reached the target exactly (step-11 F-B9).
pub const SNAP: Millimetres = Millimetres::new(1);

/// The search lattice for placing a person who arrives where nobody fits (step-11 SD-B8).
pub const LATTICE: Millimetres = Millimetres::new(50);

/// The sub-lattice the capacity check counts: `13 × LATTICE`. A disc of radius 610 mm covers at most
/// four of its points, so `4 × (people − 1) + 1` points guarantee one is free (step-11 SD-B4).
pub const CAPACITY_GRID: Millimetres = Millimetres::new(650);

/// No authored body coordinate lies beyond ±100 m: single precision's step there is under 0.01 mm, so a
/// round trip through Rapier never moves a whole millimetre by itself.
pub const COORDINATE_BOUND: Millimetres = Millimetres::new(100_000);

/// A point on a place's floor, in whole millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Point {
    pub(crate) const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// An axis-aligned rectangle on the floor: `min` is its south-west corner, `max` its north-east.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Area {
    pub(crate) min: Point,
    pub(crate) max: Point,
}

/// One place's fixed geometry, as the resolver and the adapter read it: the walkable floor, whose
/// edge is the place's walls, and the solid boxes standing on it, in authored order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Room {
    pub(crate) floor: Area,
    /// Each solid's footprint and its height above the floor.
    pub(crate) solids: Vec<(Area, i32)>,
}
