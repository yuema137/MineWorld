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

impl Area {
    /// Whether `p` lies inside this rectangle shrunk by `margin` on every side, edges included.
    pub(crate) fn holds(&self, p: Point, margin: i32) -> bool {
        p.x >= self.min.x + margin
            && p.x <= self.max.x - margin
            && p.y >= self.min.y + margin
            && p.y <= self.max.y - margin
    }

    /// The squared distance from `p` to the nearest point of this rectangle: zero inside it.
    pub(crate) fn distance2(&self, p: Point) -> i64 {
        let gap = |v: i32, low: i32, high: i32| -> i64 {
            if v < low {
                i64::from(low) - i64::from(v)
            } else if v > high {
                i64::from(v) - i64::from(high)
            } else {
                0
            }
        };
        let (dx, dy) = (
            gap(p.x, self.min.x, self.max.x),
            gap(p.y, self.min.y, self.max.y),
        );
        dx * dx + dy * dy
    }
}

/// One place's fixed geometry, as the resolver and the adapter read it: the walkable floor, whose
/// edge is the place's walls, and the solid boxes standing on it, in authored order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Room {
    pub(crate) floor: Area,
    /// Each solid's footprint and its height above the floor.
    pub(crate) solids: Vec<(Area, i32)>,
}

impl Room {
    /// Whether a centre at `p` keeps `margin` from the floor's edge and from every solid: the one test
    /// behind "inside the floor" and "out of the solids", at whatever margin the caller is held to.
    pub(crate) fn admits(&self, p: Point, margin: i32) -> bool {
        self.floor.holds(p, margin) && self.clear_of_solids(p, margin)
    }

    /// Whether a centre at `p` is at least `radius` from every solid.
    pub(crate) fn clear_of_solids(&self, p: Point, radius: i32) -> bool {
        let reach = i64::from(radius) * i64::from(radius);
        self.solids
            .iter()
            .all(|(area, _)| area.distance2(p) >= reach)
    }

    /// The nearest solid's distance from `p`, rounded down, when one is closer than `radius`.
    pub(crate) fn solid_within(&self, p: Point, radius: i32) -> Option<i64> {
        let reach = i64::from(radius) * i64::from(radius);
        self.solids
            .iter()
            .map(|(area, _)| area.distance2(p))
            .filter(|distance2| *distance2 < reach)
            .min()
            .map(i64::isqrt)
    }

    /// The points of the [`CAPACITY_GRID`], anchored at the floor's south-west corner plus a radius
    /// on each axis, where a person fits: inside the floor shrunk by a radius and at least a radius
    /// from every solid (step-11 SD-B4).
    pub(crate) fn capacity(&self) -> usize {
        let r = PERSON_RADIUS.value();
        lattice(self.floor, r, CAPACITY_GRID.value())
            .filter(|p| self.admits(*p, r))
            .count()
    }
}

/// The points of a lattice of spacing `step` anchored at `floor.min + (margin, margin)` that lie
/// inside `floor` shrunk by `margin`, south to north and, within a row, west to east.
pub(crate) fn lattice(floor: Area, margin: i32, step: i32) -> impl Iterator<Item = Point> {
    let (x0, y0) = (floor.min.x + margin, floor.min.y + margin);
    let (x1, y1) = (floor.max.x - margin, floor.max.y - margin);
    let columns = if x1 < x0 { 0 } else { (x1 - x0) / step + 1 };
    let rows = if y1 < y0 { 0 } else { (y1 - y0) / step + 1 };
    (0..rows).flat_map(move |row| {
        (0..columns).map(move |column| Point::new(x0 + column * step, y0 + row * step))
    })
}

/// The squared distance between two points, exactly.
pub(crate) fn distance2(a: Point, b: Point) -> i64 {
    let dx = i64::from(a.x) - i64::from(b.x);
    let dy = i64::from(a.y) - i64::from(b.y);
    dx * dx + dy * dy
}

/// The closest pair among `people`, by index, with its squared distance; [`None`] for fewer than two.
pub(crate) fn closest_pair(people: &[Point]) -> Option<(usize, usize, i64)> {
    let mut best: Option<(usize, usize, i64)> = None;
    for a in 0..people.len() {
        for b in (a + 1)..people.len() {
            let d = distance2(people[a], people[b]);
            if best.is_none_or(|(_, _, closest)| d < closest) {
                best = Some((a, b, d));
            }
        }
    }
    best
}
