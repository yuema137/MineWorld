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

/// The head-on bias (step-11 QB-16, SD-B10): a person first met within this distance of the walker's
/// line counts as head-on.
pub const BIAS_BAND: Millimetres = Millimetres::new(200);

/// The head-on bias's turn, `(along, right)`: a head-on walker's stride is turned right to
/// `along · d + right · right(d)`, scaled back to no longer than `d` — atan(1/4) ≈ 14.04°.
pub const BIAS_TURN: (i64, i64) = (4, 1);

/// `⌈√(along² + right²) × 1000⌉` for [`BIAS_TURN`]: √17 × 1000 = 4 123.1, rounded up so the turned
/// stride is never longer than the one asked for (`ARC-39` rule (c)).
const BIAS_NORM_MILLI: i64 = 4_124;

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

/// A box's smallest half-extent, and a ball's smallest radius (step-11 §18.3.1).
pub const OBJECT_HALF_MIN: Millimetres = Millimetres::new(50);

/// A box's largest half-extent in x and y, and a ball's largest radius: an object touching a person is
/// then within reach (300 + 10 + 400 = 710 mm < 800 mm).
pub const OBJECT_HALF_MAX: Millimetres = Millimetres::new(400);

/// A box's largest half-extent in z.
pub const OBJECT_HALF_HEIGHT_MAX: Millimetres = Millimetres::new(500);

/// Loose objects lying in one place, at most: every one is built into each scene of that place.
pub const OBJECTS_MAX: usize = 32;

/// How far a kicker's centre may be from the object's ground point (step-11 §8.1).
pub const KICK_REACH: Millimetres = Millimetres::new(800);

/// How far a thrower's centre may be from the object's ground point (step-11 §8.1).
pub const THROW_REACH: Millimetres = Millimetres::new(800);

/// How far a shover's centre may be from the target's (step-11 §8.1).
pub const SHOVE_REACH: Millimetres = Millimetres::new(1_000);

/// How far a shove asks to move its target (step-11 §8.1, QB-10): the deliberate, larger displacement.
pub const SHOVE_DISTANCE: Millimetres = Millimetres::new(500);

/// A kicked object's initial speed, in millimetres per second: horizontal, away from the kicker.
pub const KICK_SPEED: i32 = 5_000;

/// Sub-steps of 1/60 s a kick's flight may take, at most: 3 s.
pub const KICK_STEPS: u32 = 180;

/// Sub-steps of 1/60 s a throw's flight may take, at most: 4 s.
pub const THROW_STEPS: u32 = 240;

/// Sub-steps a throw's arc is aimed to take to its point: 0.8 s.
pub const THROW_FLIGHT: i32 = 48;

/// How far beyond the object an unaimed throw is aimed.
pub const THROW_DEFAULT: Millimetres = Millimetres::new(3_000);

/// An aimed throw's point lies at most this far from the object's ground point (step-11 §8.1).
pub const THROW_RANGE_MAX: Millimetres = Millimetres::new(6_000);

/// An object slower than this, in millimetres per second, …
pub const REST_SPEED: i32 = 50;

/// … for this many consecutive sub-steps has come to rest.
pub const REST_STEPS: u32 = 10;

/// A path keyframe every this many sub-steps: 0.1 s (step-11 §4.7).
pub const PATH_EVERY: u32 = 6;

/// Keyframes in a path, at most (step-11 §4.7).
pub const PATH_MAX: usize = 40;

/// The push bisection's upper bound: above `PERSON_RADIUS + GAP + √2 · OBJECT_HALF_MAX` (≈ 876 mm),
/// and a power of two, so the bisection takes exactly ten halvings.
pub const PUSH_SEARCH: i32 = 1_024;

/// Gravity in millimetres per second squared: the value the adapter gives Rapier (9.81 m/s²), here as
/// an integer so that a launch velocity is computed in integers.
pub const GRAVITY: i32 = 9_810;

/// Sub-steps per second (the fixed step is 1/60 s, step-11 DC-4).
pub const STEPS_PER_SECOND: i32 = 60;

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

/// The first person a stride from `from` by `d` meets: among `others`, those ahead of the walker
/// (`d·(p − from) > 0`), within two radii and a gap of the walker's line, and no farther along it
/// than the stride's end plus two radii and a gap; the one least far along, ties to the earlier
/// index (`EntityId` order). Returns its index and its squared distance from the line, times |d|²
/// (`cross²`), exactly (step-11 SD-B10).
pub(crate) fn first_met(from: Point, d: Point, others: &[Point]) -> Option<(usize, i128)> {
    let wide = |v: i32| i128::from(v);
    let length2 = wide(d.x) * wide(d.x) + wide(d.y) * wide(d.y);
    if length2 == 0 {
        return None;
    }
    let spacing = wide(2 * PERSON_RADIUS.value() + GAP.value());
    others
        .iter()
        .enumerate()
        .filter_map(|(index, p)| {
            let (px, py) = (wide(p.x) - wide(from.x), wide(p.y) - wide(from.y));
            let along = wide(d.x) * px + wide(d.y) * py;
            let cross = wide(d.x) * py - wide(d.y) * px;
            let beyond = along - length2;
            let near_enough = beyond <= 0 || beyond * beyond <= spacing * spacing * length2;
            (along > 0 && cross * cross < spacing * spacing * length2 && near_enough).then_some((
                index,
                along,
                cross * cross,
            ))
        })
        .min_by_key(|(index, along, _)| (*along, *index))
        .map(|(index, _, cross2)| (index, cross2))
}

/// Whether a person met at `cross2` (as [`first_met`] returns it) is head-on: within [`BIAS_BAND`]
/// of a stride `d`'s line.
pub(crate) fn head_on(d: Point, cross2: i128) -> bool {
    let band = i128::from(BIAS_BAND.value());
    let length2 = i128::from(d.x) * i128::from(d.x) + i128::from(d.y) * i128::from(d.y);
    cross2 < band * band * length2
}

/// The stride `d` turned right by [`BIAS_TURN`] and scaled back, each component truncated toward
/// zero: `(along · d + right · right(d)) × 1000 / ⌈√(along² + right²) × 1000⌉`, where `right(d)` =
/// `(d.y, −d.x)` is the walker's right with z up. Never longer than `d`.
pub(crate) fn turned_right(d: Point) -> Point {
    let (along, right) = BIAS_TURN;
    let component =
        |v: i32, w: i32| (along * i64::from(v) + right * i64::from(w)) * 1_000 / BIAS_NORM_MILLI;
    let turned = Point::new(
        i32::try_from(component(d.x, d.y)).expect("a turned stride fits"),
        i32::try_from(component(d.y, -d.x)).expect("a turned stride fits"),
    );
    assert!(
        distance2(turned, Point::new(0, 0)) <= distance2(d, Point::new(0, 0)),
        "a turned stride is never longer than the stride asked for"
    );
    turned
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

#[cfg(test)]
mod tests {
    use super::*;

    /// SD-B10's turn, over every stride a `move` may ask for on a 37 mm grid: never longer than the
    /// stride (rule (c)), always to the walker's right, and about 14° off it.
    #[test]
    fn the_head_on_turn_is_right_never_longer_and_about_fourteen_degrees() {
        let mut checked = 0;
        for x in (-2_000..=2_000).step_by(37) {
            for y in (-2_000..=2_000).step_by(37) {
                let d = Point::new(x, y);
                let length2 = distance2(d, Point::new(0, 0));
                if length2 == 0 || length2 > 2_000 * 2_000 {
                    continue;
                }
                let turned = turned_right(d);
                assert!(
                    distance2(turned, Point::new(0, 0)) <= length2,
                    "{d:?} → {turned:?}"
                );
                let cross =
                    i64::from(d.x) * i64::from(turned.y) - i64::from(d.y) * i64::from(turned.x);
                let dot =
                    i64::from(d.x) * i64::from(turned.x) + i64::from(d.y) * i64::from(turned.y);
                assert!(cross < 0, "turned right: {d:?} → {turned:?}");
                // tan θ = −cross / dot, and atan(1/4) ≈ 14.04°: 1/5 < tan θ < 1/3 for any stride
                // longer than a few millimetres.
                if length2 >= 100 * 100 {
                    assert!(
                        -cross * 5 > dot && -cross * 3 < dot,
                        "about 14°: {d:?} → {turned:?}"
                    );
                }
                checked += 1;
            }
        }
        assert!(checked > 9_000, "strides checked: {checked}");
    }
}
