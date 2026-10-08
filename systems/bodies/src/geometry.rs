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

    /// Whether a stride from `from` to `to` meets nothing, by integers alone — the fast path, on which
    /// no scene is built (step-11 SD-B6 step 2). Clear means: the centre's whole segment keeps a radius
    /// and a gap from the floor's edge (both ends do, and the shrunk floor is convex); the segment's
    /// box, grown by a radius and a gap, meets no solid's footprint; and every other person keeps two
    /// radii and a gap from the segment. This is the definition of the clear case, not an
    /// approximation of the sweep: a stride that is not clear is swept.
    pub(crate) fn corridor_clear(&self, from: Point, to: Point, others: &[Point]) -> bool {
        let margin = PERSON_RADIUS.value() + GAP.value();
        if !self.floor.holds(from, margin) || !self.floor.holds(to, margin) {
            return false;
        }
        let grown = Area {
            min: Point::new(from.x.min(to.x) - margin, from.y.min(to.y) - margin),
            max: Point::new(from.x.max(to.x) + margin, from.y.max(to.y) + margin),
        };
        let meets = |area: &Area| {
            area.min.x <= grown.max.x
                && area.max.x >= grown.min.x
                && area.min.y <= grown.max.y
                && area.max.y >= grown.min.y
        };
        if self.solids.iter().any(|(area, _)| meets(area)) {
            return false;
        }
        let apart = 2 * PERSON_RADIUS.value() + GAP.value();
        others
            .iter()
            .all(|other| segment_clear_of(from, to, *other, apart))
    }

    /// Whether a person fits at `p` with room to spare: inside the floor shrunk by a radius, a radius
    /// from every solid, and two radii and a gap from each of `others` (step-11 SD-B8 E1).
    pub(crate) fn free_at(&self, p: Point, others: &[Point]) -> bool {
        let apart = i64::from(2 * PERSON_RADIUS.value() + GAP.value());
        self.admits(p, PERSON_RADIUS.value())
            && others
                .iter()
                .all(|other| distance2(p, *other) >= apart * apart)
    }

    /// The free point (as [`Room::free_at`]) of the [`LATTICE`] nearest `target`, ties broken by `y`
    /// then `x`; [`None`] only when the floor has none.
    pub(crate) fn nearest_free(&self, target: Point, others: &[Point]) -> Option<Point> {
        lattice(self.floor, PERSON_RADIUS.value(), LATTICE.value())
            .filter(|p| self.free_at(*p, others))
            .min_by_key(|p| (distance2(*p, target), p.y, p.x))
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

impl Point {
    /// `self − other`, as an offset.
    pub(crate) const fn minus(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    /// `self + offset`.
    pub(crate) const fn plus(self, offset: Self) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y)
    }

    /// The length of this offset, rounded down.
    pub(crate) fn length(self) -> i64 {
        distance2(self, Self::new(0, 0)).isqrt()
    }
}

/// `offset` scaled by `numerator / denominator`, each component truncated toward zero, so the result
/// is never longer than the exact scaling. `denominator` must be positive.
pub(crate) fn scaled_down(offset: Point, numerator: i64, denominator: i64) -> Point {
    let scale = |v: i32| {
        i32::try_from(i64::from(v) * numerator / denominator).expect("a scaled-down offset fits")
    };
    Point::new(scale(offset.x), scale(offset.y))
}

/// The offset along `direction` that is at least `length` long: `direction × length / |direction|`,
/// each component rounded away from zero and the length divided by `|direction|` rounded down, so
/// what it lacks in exactness it makes up in length. `direction` must not be zero.
pub(crate) fn at_least(direction: Point, length: i64) -> Point {
    let norm = direction.length().max(1);
    let scale = |v: i32| {
        let product = i64::from(v) * length;
        let quotient = product / norm;
        let rounded = if product % norm == 0 {
            quotient
        } else {
            quotient + product.signum()
        };
        i32::try_from(rounded).expect("a nudge fits")
    };
    Point::new(scale(direction.x), scale(direction.y))
}

/// `p`, moved toward `from` if needed so that it is no farther from `from` than `target` is: a
/// resolver may shorten or bend an arrival, never lengthen it (`ARC-39` rule (c)). A quantized sweep
/// that slid along a wall can come back a fraction of a millimetre long; this takes it back.
pub(crate) fn no_longer_than(from: Point, p: Point, target: Point) -> Point {
    let (asked, got) = (distance2(from, target), distance2(from, p));
    if got <= asked {
        return p;
    }
    let ceil_root = |v: i64| {
        let root = v.isqrt();
        if root * root == v { root } else { root + 1 }
    };
    from.plus(scaled_down(p.minus(from), asked.isqrt(), ceil_root(got)))
}

/// Whether every point of the segment `a`–`b` is at least `radius` from `p`, exactly, in `i128`.
pub(crate) fn segment_clear_of(a: Point, b: Point, p: Point, radius: i32) -> bool {
    let wide = |v: i32| i128::from(v);
    let (abx, aby) = (wide(b.x) - wide(a.x), wide(b.y) - wide(a.y));
    let (apx, apy) = (wide(p.x) - wide(a.x), wide(p.y) - wide(a.y));
    let reach = wide(radius) * wide(radius);
    let along = apx * abx + apy * aby;
    let span = abx * abx + aby * aby;
    if span == 0 || along <= 0 {
        return apx * apx + apy * apy >= reach;
    }
    if along >= span {
        let (bpx, bpy) = (wide(p.x) - wide(b.x), wide(p.y) - wide(b.y));
        return bpx * bpx + bpy * bpy >= reach;
    }
    let cross = abx * apy - aby * apx;
    cross * cross >= reach * span
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
