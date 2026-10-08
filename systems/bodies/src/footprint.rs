//! Integer geometry of loose objects (step-11 SD-O2, SD-O5, SD-O8, SD-O17): an object's footprint on
//! the floor, the invariant every stored object keeps, the push bisection, and what an object costs a
//! place's capacity.
//!
//! Like `geometry.rs`, whole millimetres, `i64`/`i128` products and `isqrt` only: only the push's
//! shape cast and a flight are float, in `rapier.rs`.
//!
//! An object's **footprint** is its outline on the floor: a box's x–y rectangle, a ball's disc. Every
//! check of an object against a person uses the person's disc of `PERSON_RADIUS` and the footprint,
//! with `TOLERANCE` as 12b uses it for people: a person and an object, or two objects, may come
//! together to within 5 mm of touching and no closer.

use crate::component::{BodyShape, Lying};
use crate::geometry::{
    Area, CAPACITY_GRID, GAP, LATTICE, PERSON_RADIUS, PUSH_SEARCH, Point, Room, TOLERANCE,
    at_least, distance2, segment_clear_of,
};

/// A footprint's outline, about its centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outline {
    /// A rectangle of these half-extents.
    Rect { hx: i32, hy: i32 },
    /// A disc of this radius.
    Disc { r: i32 },
}

/// An object's outline on the floor, at a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Footprint {
    pub(crate) centre: Point,
    outline: Outline,
}

impl Footprint {
    /// The footprint of an object of `shape` centred at `centre`.
    pub(crate) const fn new(shape: BodyShape, centre: Point) -> Self {
        let outline = match shape {
            BodyShape::Box(half) => Outline::Rect {
                hx: half.x().value(),
                hy: half.y().value(),
            },
            BodyShape::Ball(radius) => Outline::Disc { r: radius.value() },
        };
        Self { centre, outline }
    }

    /// The same outline, centred at `centre`.
    #[cfg_attr(not(test), expect(dead_code, reason = "the pushes arrive in PO-C3"))]
    pub(crate) const fn at(self, centre: Point) -> Self {
        Self { centre, ..self }
    }

    /// Its bounding rectangle.
    fn bounds(&self) -> Area {
        let (hx, hy) = match self.outline {
            Outline::Rect { hx, hy } => (hx, hy),
            Outline::Disc { r } => (r, r),
        };
        Area {
            min: Point::new(self.centre.x - hx, self.centre.y - hy),
            max: Point::new(self.centre.x + hx, self.centre.y + hy),
        }
    }

    /// Whether some point of this footprint is closer than `reach` to `p`. `reach` is positive.
    pub(crate) fn within(&self, p: Point, reach: i32) -> bool {
        match self.outline {
            Outline::Rect { .. } => self.bounds().distance2(p) < i64::from(reach).pow(2),
            Outline::Disc { r } => distance2(self.centre, p) < i64::from(reach + r).pow(2),
        }
    }

    /// Whether a person standing at `p` overlaps this footprint by more than [`TOLERANCE`].
    pub(crate) fn under(&self, p: Point) -> bool {
        self.within(p, PERSON_RADIUS.value() - TOLERANCE.value())
    }

    /// Whether the footprint lies inside `floor`, edges included.
    pub(crate) fn inside(&self, floor: Area) -> bool {
        let bounds = self.bounds();
        bounds.min.x >= floor.min.x
            && bounds.max.x <= floor.max.x
            && bounds.min.y >= floor.min.y
            && bounds.max.y <= floor.max.y
    }

    /// Whether the footprint meets `area` over more than its edge.
    pub(crate) fn meets(&self, area: Area) -> bool {
        match self.outline {
            Outline::Rect { .. } => {
                let bounds = self.bounds();
                bounds.min.x < area.max.x
                    && bounds.max.x > area.min.x
                    && bounds.min.y < area.max.y
                    && bounds.max.y > area.min.y
            }
            Outline::Disc { r } => area.distance2(self.centre) < i64::from(r).pow(2),
        }
    }

    /// Whether this footprint and `other` overlap by more than `tolerance` millimetres.
    pub(crate) fn overlaps(&self, other: &Self, tolerance: i32) -> bool {
        match (self.outline, other.outline) {
            (Outline::Rect { hx, hy }, Outline::Rect { hx: ox, hy: oy }) => {
                let across = hx + ox - (self.centre.x - other.centre.x).abs();
                let along = hy + oy - (self.centre.y - other.centre.y).abs();
                across > tolerance && along > tolerance
            }
            (Outline::Disc { r }, Outline::Disc { r: or }) => {
                distance2(self.centre, other.centre) < i64::from(r + or - tolerance).pow(2)
            }
            (Outline::Disc { r }, Outline::Rect { .. }) => {
                other.bounds().distance2(self.centre) < i64::from(r - tolerance).pow(2)
            }
            (Outline::Rect { .. }, Outline::Disc { .. }) => other.overlaps(self, tolerance),
        }
    }

    /// Whether a person's centre moving from `from` to `to` keeps `margin` from this footprint all the
    /// way, by integers alone: a ball exactly; a box when the segment's box, grown by `margin`, does
    /// not reach it — the definition of the clear case, as `Room::corridor_clear` defines it for
    /// solids (step-11 SD-O9 step 2).
    #[expect(
        dead_code,
        reason = "the resolver's fast path with objects arrives in PO-C3"
    )]
    pub(crate) fn clear_of_segment(&self, from: Point, to: Point, margin: i32) -> bool {
        match self.outline {
            Outline::Disc { r } => segment_clear_of(from, to, self.centre, margin + r),
            Outline::Rect { .. } => {
                let bounds = self.bounds();
                let (low_x, high_x) = (from.x.min(to.x) - margin, from.x.max(to.x) + margin);
                let (low_y, high_y) = (from.y.min(to.y) - margin, from.y.max(to.y) + margin);
                bounds.max.x < low_x
                    || bounds.min.x > high_x
                    || bounds.max.y < low_y
                    || bounds.min.y > high_y
            }
        }
    }
}

/// Where an object rests, as SD-O2 allows: on the floor, or on the top of one solid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resting {
    /// On the floor.
    Floor,
    /// On the top of the solid at this index, its footprint within that top.
    Solid(usize),
}

/// Why an object's place breaks the invariant of the stored state (step-11 SD-O2), in the order the
/// checks run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flaw {
    /// Its footprint leaves the floor.
    Outside,
    /// It rests neither on the floor nor on a solid's top.
    Floating,
    /// Its footprint meets a solid (other than the one it rests on), at this index.
    InSolid(usize),
    /// It overlaps another object, at this index of the others given.
    Overlap(usize),
    /// A person's disc overlaps it, at this index of the people given.
    OnPerson(usize),
}

/// One object as the invariant reads it: its shape and where its centre is, `(x, y)` and height `z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Placed {
    pub(crate) shape: BodyShape,
    pub(crate) centre: Point,
    pub(crate) z: i32,
}

impl Placed {
    /// A stored row with its shape.
    pub(crate) fn of(lying: &Lying, shape: BodyShape) -> Self {
        let at = lying.at();
        Self {
            shape,
            centre: Point::new(at.x().value(), at.y().value()),
            z: at.z().value(),
        }
    }

    pub(crate) const fn footprint(&self) -> Footprint {
        Footprint::new(self.shape, self.centre)
    }

    /// What it rests on, to within `tolerance` of the exact height, if anything.
    pub(crate) fn resting(&self, room: &Room, tolerance: i32) -> Option<Resting> {
        let half = self.shape.half_height();
        if (self.z - half).abs() <= tolerance {
            return Some(Resting::Floor);
        }
        let footprint = self.footprint();
        room.solids
            .iter()
            .position(|(area, height)| {
                (self.z - (height + half)).abs() <= tolerance && footprint.inside(*area)
            })
            .map(Resting::Solid)
    }
}

/// The invariant of the stored state for one object, against the room, the other objects and the
/// people, in SD-O5's order: within the floor; resting on the floor or a solid's top (heights to
/// within `tolerance`); clear of every other solid; overlapping no other object and no person's disc
/// by more than [`TOLERANCE`].
pub(crate) fn flaw(
    room: &Room,
    object: &Placed,
    others: &[Footprint],
    people: &[Point],
    tolerance: i32,
) -> Option<Flaw> {
    let footprint = object.footprint();
    if !footprint.inside(room.floor) {
        return Some(Flaw::Outside);
    }
    let Some(resting) = object.resting(room, tolerance) else {
        return Some(Flaw::Floating);
    };
    if let Some(index) = room
        .solids
        .iter()
        .enumerate()
        .position(|(index, (area, _))| resting != Resting::Solid(index) && footprint.meets(*area))
    {
        return Some(Flaw::InSolid(index));
    }
    if let Some(index) = others
        .iter()
        .position(|other| footprint.overlaps(other, TOLERANCE.value()))
    {
        return Some(Flaw::Overlap(index));
    }
    people
        .iter()
        .position(|person| footprint.under(*person))
        .map(Flaw::OnPerson)
}

/// The most points of the place's [`CAPACITY_GRID`] an object of `shape` can cover wherever it lies
/// (step-11 SD-O5): `(⌊(2·hx + 2R) / 650⌋ + 1) · (⌊(2·hy + 2R) / 650⌋ + 1)`.
pub(crate) fn blocks(shape: BodyShape) -> usize {
    let (hx, hy) = shape.half_footprint();
    let r = PERSON_RADIUS.value();
    let grid = CAPACITY_GRID.value();
    let across = |half: i32| usize::try_from((2 * half + 2 * r) / grid + 1).expect("positive");
    across(hx) * across(hy)
}

/// The push of an object whose footprint a person at `p` overlaps (step-11 SD-O8): the offset, along
/// the integer direction from `p` to the footprint's centre ((1, 0) when they coincide), of the
/// smallest whole millimetre `t` in `[0, PUSH_SEARCH]` at which the footprint is at least
/// `PERSON_RADIUS + GAP` from `p`, found by bisection in exactly ten halvings. [`None`] when even
/// `PUSH_SEARCH` is not enough.
#[cfg_attr(not(test), expect(dead_code, reason = "the pushes arrive in PO-C3"))]
pub(crate) fn push_offset(footprint: Footprint, p: Point) -> Option<Point> {
    let direction = match footprint.centre.minus(p) {
        Point { x: 0, y: 0 } => Point::new(1, 0),
        away => away,
    };
    let apart = PERSON_RADIUS.value() + GAP.value();
    let clear = |t: i32| {
        let moved = footprint.at(footprint.centre.plus(at_least(direction, i64::from(t))));
        !moved.within(p, apart)
    };
    if !clear(PUSH_SEARCH) {
        return None;
    }
    let (mut low, mut high) = (0, PUSH_SEARCH);
    while high - low > 1 {
        let middle = (low + high) / 2;
        if clear(middle) {
            high = middle;
        } else {
            low = middle;
        }
    }
    Some(at_least(direction, i64::from(high)))
}

/// The points of the [`LATTICE`] on the floor where an object of `shape` could rest, anchored at the
/// floor's south-west corner plus its half-footprint, south to north and west to east.
#[expect(dead_code, reason = "the landing ladder arrives in PO-C4")]
pub(crate) fn object_lattice(room: &Room, shape: BodyShape) -> impl Iterator<Item = Point> {
    let (hx, hy) = shape.half_footprint();
    let step = LATTICE.value();
    let (x0, y0) = (room.floor.min.x + hx, room.floor.min.y + hy);
    let (x1, y1) = (room.floor.max.x - hx, room.floor.max.y - hy);
    let count = |low: i32, high: i32| {
        if high < low {
            0
        } else {
            (high - low) / step + 1
        }
    };
    let (columns, rows) = (count(x0, x1), count(y0, y1));
    (0..rows).flat_map(move |row| {
        (0..columns).map(move |column| Point::new(x0 + column * step, y0 + row * step))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ball(r: i32) -> BodyShape {
        serde_json::from_value(serde_json::json!({ "ball": r })).expect("a ball")
    }

    fn cube(half: i32) -> BodyShape {
        serde_json::from_value(serde_json::json!({ "box": { "x": half, "y": half, "z": half } }))
            .expect("a box")
    }

    /// The bisection against hand-computed literals: the footprint ends exactly 310 mm (R + GAP) from
    /// the pusher, or, along a diagonal, the first whole millimetre past it.
    #[test]
    fn a_push_moves_the_footprint_just_clear_of_the_pusher() {
        // A ball r 110 at (4 000, 5 000), a person at (3 880, 5 000): 10 mm into the disc's reach of
        // 300. Clear at 310 + 110 = 420 from the centre: the ball goes +300 to (4 300, 5 000).
        let pushed = push_offset(
            Footprint::new(ball(110), Point::new(4_000, 5_000)),
            Point::new(3_880, 5_000),
        );
        assert_eq!(pushed, Some(Point::new(300, 0)));
        // A box half 200 at (3 500, 5 000), its face at 3 300; a person at (3 290, 5 000): the face
        // must reach 3 600 — the box goes +300 to (3 800, 5 000).
        let pushed = push_offset(
            Footprint::new(cube(200), Point::new(3_500, 5_000)),
            Point::new(3_290, 5_000),
        );
        assert_eq!(pushed, Some(Point::new(300, 0)));
        // The same box met corner-on, the person at (3 140, 4 640): the corner (3 300, 4 800) is
        // √(160² + 160²) ≈ 226 mm away; along (360, 360) the corner must reach 310 mm, i.e. 84 mm
        // more along the diagonal (226.27 + 84 > 310 > 226.27 + 83): at_least((1, 1)·84) = (60, 60)
        // rounded away from zero → (60, 60) has length 84.85 ≥ 84.
        let pushed = push_offset(
            Footprint::new(cube(200), Point::new(3_500, 5_000)),
            Point::new(3_140, 4_640),
        );
        assert_eq!(pushed, Some(Point::new(60, 60)));
    }

    /// A centre coinciding with the pusher's is pushed +x, by exactly 310 + 110 mm.
    #[test]
    fn a_coincident_centre_goes_east() {
        let pushed = push_offset(
            Footprint::new(ball(110), Point::new(0, 0)),
            Point::new(0, 0),
        );
        assert_eq!(pushed, Some(Point::new(420, 0)));
    }

    /// blocks(o) for the shapes the yard uses: a ball r 110 and a box half 200 cover at most 2 × 2
    /// points of the 650 mm grid; a crate half 300, 2 × 2 as well ((600 + 600) / 650 + 1 = 2); a box
    /// half 400, 3 × 3.
    #[test]
    fn what_an_object_costs_a_places_capacity() {
        assert_eq!(blocks(ball(110)), 4);
        assert_eq!(blocks(cube(200)), 4);
        assert_eq!(blocks(cube(300)), 4);
        assert_eq!(blocks(cube(400)), 9);
    }
}
