//! Kick and throw, on integers (step-11 SD-O11, SD-O13, SD-O14): the launch velocities, an unaimed
//! throw's point, and the landing ladder that verifies what the flight in `rapier.rs` returns.
//!
//! ```text
//! kick    horizontal, KICK_SPEED from the object toward the room's free centre (SD-O13's p3 note:
//!         a default that brings objects back to where people are)
//! throw   unaimed: toward the free centre, at most THROW_DEFAULT; aimed: to its point.
//!         To reach its point after THROW_FLIGHT sub-steps: in the plane (point − start) / T, in z
//!         (z_rest − z_start) / T + g·T / 2, with T = THROW_FLIGHT / 60 s — + − × ÷ only (DC-3)
//! land    (1) the simulated end, if it keeps the stored-state invariant (V-O);
//!         (2) else the nearest point of the 50 mm lattice on the floor to the end's ground point
//!             that does, ordered by (distance², y, x);
//!         (3) else the object stays where it was
//! ```

use crate::component::BodyShape;
use crate::footprint::{Footprint, Placed, flaw, object_lattice};
use crate::geometry::{
    GAP, GRAVITY, KICK_SPEED, PERSON_RADIUS, PULL_BACK_STEP, Point, REST_CLEARANCE, Room,
    STEPS_PER_SECOND, THROW_DEFAULT, THROW_FLIGHT, TOLERANCE, at_least, distance2, scaled_down,
};
use crate::rapier::At;

/// The direction from `from` to `to`, or east when they coincide.
fn direction(from: Point, to: Point) -> Point {
    match to.minus(from) {
        Point { x: 0, y: 0 } => Point::new(1, 0),
        away => away,
    }
}

/// The room's **free centre** for an object of `shape` (step-11 SD-O13 as amended, rung p3): the
/// floor rectangle's centre when the object's footprint there meets no solid; otherwise the nearest
/// point of the object's 50 mm lattice whose footprint meets no solid, ordered by (distance², y, x).
/// The floor's own centre otherwise (a floor covered by solids everywhere — unreachable here, since a
/// placed object proves a free point exists).
pub(crate) fn free_centre(room: &Room, shape: BodyShape) -> Point {
    let centre = Point::new(
        (room.floor.min.x + room.floor.max.x) / 2,
        (room.floor.min.y + room.floor.max.y) / 2,
    );
    // p4: the target keeps REST_CLEARANCE from every solid, so what gathers there stays pushable.
    if rests_clear(room, shape, centre) {
        return centre;
    }
    let meets_none = |p: Point| {
        let footprint = Footprint::new(shape, p);
        footprint.inside(room.floor) && room.solids.iter().all(|(area, _)| !footprint.meets(*area))
    };
    object_lattice(room, shape)
        .filter(|p| rests_clear(room, shape, *p))
        .min_by_key(|p| (distance2(*p, centre), p.y, p.x))
        .or_else(|| {
            object_lattice(room, shape)
                .filter(|p| meets_none(*p))
                .min_by_key(|p| (distance2(*p, centre), p.y, p.x))
        })
        .unwrap_or(centre)
}

/// A kicked object's launch velocity, in millimetres per second (SD-O11; SD-O13's p3 note):
/// horizontal, at least `KICK_SPEED` long, from the object toward the room's free centre. A kick has
/// no aim, so it always takes this default. An object already at the free centre goes along the
/// kicker's centre → the object's centre, and east when those coincide too.
///
/// **Never into the kicker** (§18.11 DO-18, a defect fix to p3): when the line from the object toward
/// the free centre runs ahead into the kicker's body — the kicker is ahead along it (positive dot
/// product) and the line passes within `PERSON_RADIUS + half + GAP` of the kicker's centre, `half`
/// being the object's larger half-footprint — the object goes away from the kicker instead: along
/// kicker → object, east if those coincide.
pub(crate) fn kick_velocity(
    kicker: Point,
    object: Point,
    centre: Point,
    half: i32,
) -> (i32, i32, i32) {
    let away = direction(kicker, object);
    let toward = match centre.minus(object) {
        Point { x: 0, y: 0 } => away,
        line if into(line, kicker.minus(object), half) => away,
        line => line,
    };
    let v = at_least(toward, i64::from(KICK_SPEED));
    (v.x, v.y, 0)
}

/// Whether a line from the object along `line` runs into a person whose centre is at `person`
/// (relative to the object), for an object of larger half-footprint `half`.
fn into(line: Point, person: Point, half: i32) -> bool {
    let wide = |v: i32| i128::from(v);
    let along = wide(line.x) * wide(person.x) + wide(line.y) * wide(person.y);
    let cross = wide(line.x) * wide(person.y) - wide(line.y) * wide(person.x);
    let length2 = wide(line.x).pow(2) + wide(line.y).pow(2);
    let reach = wide(PERSON_RADIUS.value() + half + GAP.value());
    along > 0 && cross * cross < reach * reach * length2
}

/// Where an unaimed throw goes (SD-O13 as amended, rung p3): toward the room's free centre — the
/// free centre itself when it is at most `THROW_DEFAULT` away, else `THROW_DEFAULT` along the line
/// to it (each component truncated toward zero, so never farther). An object already at the free
/// centre is thrown straight up and lands where it lay. Every point on that line keeps the object's
/// footprint inside the floor: both ends do, and the floor is convex.
pub(crate) fn default_aim(room: &Room, object: &Placed) -> Point {
    let centre = free_centre(room, object.shape);
    let line = centre.minus(object.centre);
    let squared = distance2(line, Point::new(0, 0));
    let farthest = i64::from(THROW_DEFAULT.value());
    if squared <= farthest * farthest {
        return centre;
    }
    let floor = squared.isqrt();
    let length = if floor * floor == squared {
        floor
    } else {
        floor + 1
    };
    object.centre.plus(scaled_down(line, farthest, length))
}

/// How high `shape`'s centre rests at `point`: on the top of the first solid whose top holds the
/// footprint there, else on the floor.
pub(crate) fn rest_height(room: &Room, object: &Placed, point: Point) -> i32 {
    let footprint = object.footprint().at(point);
    let half = object.shape.half_height();
    room.solids
        .iter()
        .find(|(area, _)| footprint.inside(*area))
        .map_or(half, |(_, height)| height + half)
}

/// A thrown object's launch velocity, in millimetres per second (step-11 SD-O13): aimed to reach
/// `point` at rest height `z_rest` after `THROW_FLIGHT` sub-steps of 1/60 s. With T = 48 / 60 s, the
/// plane's (d / T) is d · 60 / 48 and z's g·T / 2 is 9 810 · 48 / 120 mm/s, each truncated toward zero.
pub(crate) fn throw_velocity(start: &Placed, point: Point, z_rest: i32) -> (i32, i32, i32) {
    let per_second = |d: i32| d * STEPS_PER_SECOND / THROW_FLIGHT;
    let lift = GRAVITY * THROW_FLIGHT / (2 * STEPS_PER_SECOND);
    let d = point.minus(start.centre);
    (
        per_second(d.x),
        per_second(d.y),
        per_second(z_rest - start.z) + lift,
    )
}

/// The landing ladder (step-11 SD-O14, QO-12): where the flown object is recorded to lie.
///
/// `others` are the other objects' footprints and `people` everybody standing in the place; the
/// object's own invariant (V-O) is SD-O2's — within the floor, resting on the floor or a solid's top to
/// within `TOLERANCE`, clear of the other solids, of the other objects and of every person's disc.
pub(crate) fn land(
    room: &Room,
    flown: &Placed,
    end: At,
    others: &[Footprint],
    people: &[Point],
) -> At {
    let end = pulled_back(room, flown, end);
    let (point, z) = end;
    let at_end = Placed {
        centre: point,
        z,
        ..*flown
    };
    if flaw(room, &at_end, others, people, TOLERANCE.value()).is_none() {
        return end;
    }
    let half = flown.shape.half_height();
    object_lattice(room, flown.shape)
        .filter(|p| {
            let there = Placed {
                centre: *p,
                z: half,
                ..*flown
            };
            flaw(room, &there, others, people, 0).is_none()
        })
        .min_by_key(|p| {
            (
                !rests_clear(room, flown.shape, *p),
                distance2(*p, point),
                p.y,
                p.x,
            )
        })
        .map_or((flown.centre, flown.z), |p| (p, half))
}

/// Whether an object of `shape` resting on the floor at `p` lies inside the floor and keeps
/// `REST_CLEARANCE` from every solid (SD-O13's p4 note).
pub(crate) fn rests_clear(room: &Room, shape: BodyShape, p: Point) -> bool {
    let footprint = Footprint::new(shape, p);
    footprint.inside(room.floor)
        && room
            .solids
            .iter()
            .all(|(area, _)| footprint.keeps(*area, REST_CLEARANCE.value()))
}

/// p4 for a flight's end (SD-O13's p4 note): an end resting on the floor within `REST_CLEARANCE` of
/// a solid is pulled back along the line toward where the flight began, in `PULL_BACK_STEP` mm steps,
/// to the first point that keeps the clearance. When no point of that line does — the flight was
/// **blocked** by the solid it lies against — the end stands. An end on a solid's top, or already
/// clear, stands too.
pub(crate) fn pulled_back(room: &Room, flown: &Placed, end: At) -> At {
    let (point, z) = end;
    let half = flown.shape.half_height();
    if (z - half).abs() > TOLERANCE.value() || rests_clear(room, flown.shape, point) {
        return end;
    }
    let back = flown.centre.minus(point);
    let squared = distance2(back, Point::new(0, 0));
    let length = squared.isqrt();
    if length == 0 {
        return end;
    }
    let step = i64::from(PULL_BACK_STEP);
    (1..=length / step)
        .map(|k| point.plus(scaled_down(back, k * step, length)))
        .chain(core::iter::once(flown.centre))
        .find(|p| rests_clear(room, flown.shape, *p))
        .map_or(end, |p| (p, half))
}

#[cfg(test)]
mod tests {
    //! The integer launch velocities against hand-computed literals (step-11 §18.6), and PO-4 f: a
    //! flight cut after two sub-steps leaves the object in the air, and the landing ladder puts it on
    //! the nearest free lattice point at rest height.

    use super::*;
    use crate::component::BodyShape;
    use crate::geometry::Area;
    use crate::rapier::{Launch, fly};

    fn ball(r: i32) -> BodyShape {
        serde_json::from_value(serde_json::json!({ "ball": r })).expect("a ball")
    }

    /// The prototype's café: floor (0, 0)–(8 320, 10 320), the counter at 1 100 mm.
    fn cafe() -> Room {
        Room {
            floor: Area {
                min: Point::new(0, 0),
                max: Point::new(8_320, 10_320),
            },
            solids: vec![(
                Area {
                    min: Point::new(3_860, 6_570),
                    max: Point::new(8_320, 7_170),
                },
                1_100,
            )],
        }
    }

    fn lying(shape: BodyShape, x: i32, y: i32) -> Placed {
        Placed {
            shape,
            centre: Point::new(x, y),
            z: shape.half_height(),
        }
    }

    #[test]
    fn launch_velocities_are_the_hand_computed_integers() {
        // A kick goes toward the free centre (p3). From (2 700, 5 000) toward the café's (4 160,
        // 5 160): along (1 460, 160), |·| rounded down 1 468; 1 460 · 5 000 / 1 468 = 4 972.8 → 4 973,
        // 160 · 5 000 / 1 468 = 544.96 → 545 (rounded away from zero, at_least).
        let centre = Point::new(4_160, 5_160);
        assert_eq!(
            kick_velocity(
                Point::new(2_000, 5_000),
                Point::new(2_700, 5_000),
                centre,
                110
            ),
            (4_973, 545, 0)
        );
        // DO-18: the kicker at (3 400, 5 080), between the ball and the centre — 704 mm ahead along
        // the line and 3 mm off it (cross 4 800 / 1 468), well within 300 + 110 + 10 — so the ball goes
        // away from the kicker, along (−700, −80): |·| rounded down 704; 700 · 5 000 / 704 = 4 971.6 →
        // 4 972, 80 · 5 000 / 704 = 568.2 → 569, both negative.
        assert_eq!(
            kick_velocity(
                Point::new(3_400, 5_080),
                Point::new(2_700, 5_000),
                centre,
                110
            ),
            (-4_972, -569, 0)
        );
        // Along (3, 4): at_least((3, 4), 5 000) = (3 000, 4 000), with the kicker behind.
        assert_eq!(
            kick_velocity(Point::new(-9, -9), Point::new(0, 0), Point::new(3, 4), 110),
            (3_000, 4_000, 0)
        );
        // An object at the free centre goes away from the kicker; coincident with the kicker, east.
        assert_eq!(
            kick_velocity(Point::new(3_460, 5_160), centre, centre, 110),
            (5_000, 0, 0)
        );
        assert_eq!(
            kick_velocity(Point::new(7, 7), Point::new(7, 7), Point::new(7, 7), 110),
            (5_000, 0, 0)
        );
        // A ball r 110 at (2 600, 5 000) thrown to (5 600, 5 000) on the floor: 3 000 · 60 / 48 = 3 750
        // in x; in z, 0 + 9 810 · 48 / 120 = 3 924.
        let ball = lying(ball(110), 2_600, 5_000);
        assert_eq!(
            throw_velocity(&ball, Point::new(5_600, 5_000), 110),
            (3_750, 0, 3_924)
        );
        // To the counter's top (1 100 + 110): z gains 1 100 · 60 / 48 = 1 375.
        assert_eq!(
            throw_velocity(&ball, Point::new(2_600, 6_870), 1_210),
            (0, 2_337, 5_299)
        );
    }

    /// The hall of bodies-yard: 12 m × 9 m, the table (5 000, 4 000)–(7 000, 5 000) over its centre.
    fn hall() -> Room {
        Room {
            floor: Area {
                min: Point::new(0, 0),
                max: Point::new(12_000, 9_000),
            },
            solids: vec![(
                Area {
                    min: Point::new(5_000, 4_000),
                    max: Point::new(7_000, 5_000),
                },
                750,
            )],
        }
    }

    #[test]
    fn the_free_centre_is_the_floors_centre_unless_a_solid_stands_there() {
        // The café's centre (4 160, 5 160) is clear of the counter.
        assert_eq!(free_centre(&cafe(), ball(110)), Point::new(4_160, 5_160));
        // The hall's centre (6 000, 4 500) is under the table. A ball r 110's lattice is
        // (110 + 50 i, 110 + 50 j); p4 wants its centre at least 300 + 110 = 410 mm from the table:
        // north, y ≥ 5 410 — the lattice's 5 410; south, y ≤ 3 590 — 3 560. The nearest is
        // (6 010, 5 410): d² = 10² + 910² = 828 200, against (5 960, 5 410) 829 700 and (6 010,
        // 3 560) 883 700.
        assert_eq!(free_centre(&hall(), ball(110)), Point::new(6_010, 5_410));
    }

    #[test]
    fn an_unaimed_throw_goes_toward_the_free_centre_at_most_three_metres() {
        let room = cafe();
        // PO-5 a's ball at (2 600, 5 000): the centre is 1 568 mm away, so the aim is the centre.
        let near = lying(ball(110), 2_600, 5_000);
        assert_eq!(default_aim(&room, &near), Point::new(4_160, 5_160));
        // From the corner (200, 200): along (3 960, 4 960), |·| rounded up 6 347; 3 000 of it is
        // (3 960 · 3 000 / 6 347, 4 960 · 3 000 / 6 347) = (1 871, 2 344), truncated.
        let corner = lying(ball(110), 200, 200);
        assert_eq!(default_aim(&room, &corner), Point::new(2_071, 2_544));
        // An object at the free centre is thrown straight up.
        let centred = lying(ball(110), 4_160, 5_160);
        assert_eq!(default_aim(&room, &centred), Point::new(4_160, 5_160));
    }

    #[test]
    fn a_flight_cut_short_lands_on_the_nearest_free_lattice_point() {
        let room = cafe();
        let ball = lying(ball(110), 2_600, 5_000);
        let flown = fly(&Launch {
            room: &room,
            people: &[Point::new(2_000, 5_000)],
            others: &[],
            flying: ball,
            velocity: throw_velocity(&ball, Point::new(5_600, 5_000), 110),
            steps: 2,
        });
        let (ground, z) = flown.end;
        println!("after 2 sub-steps: ({}, {}, {z})", ground.x, ground.y);
        assert!(z > 110 + 5, "still in the air: z {z}");
        let people = [Point::new(2_000, 5_000)];
        let landed = land(&room, &ball, flown.end, &[], &people);
        println!("landed at {landed:?}");
        // The 50 mm lattice for r 110 runs x = 110 + 50k, y = 110 + 50k: the end's ground point lies
        // within 25 mm of one, which is free.
        let (point, rest) = landed;
        assert_eq!(rest, 110, "at rest height");
        assert_eq!((point.x - 110) % 50, 0, "on the lattice: {point:?}");
        assert_eq!((point.y - 110) % 50, 0, "on the lattice: {point:?}");
        assert!(
            distance2(point, ground) <= 2 * 25 * 25,
            "nearest: {point:?}"
        );
    }
}
