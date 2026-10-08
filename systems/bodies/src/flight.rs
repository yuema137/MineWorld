//! Kick and throw, on integers (step-11 SD-O11, SD-O13, SD-O14): the launch velocities, an unaimed
//! throw's point, and the landing ladder that verifies what the flight in `rapier.rs` returns.
//!
//! ```text
//! kick    horizontal, KICK_SPEED along the kicker's centre → the object's centre ((1, 0) when they
//!         coincide)
//! throw   to reach its point after THROW_FLIGHT sub-steps: in the plane (point − start) / T, in z
//!         (z_rest − z_start) / T + g·T / 2, with T = THROW_FLIGHT / 60 s — + − × ÷ only (DC-3)
//! land    (1) the simulated end, if it keeps the stored-state invariant (V-O);
//!         (2) else the nearest point of the 50 mm lattice on the floor to the end's ground point
//!             that does, ordered by (distance², y, x);
//!         (3) else the object stays where it was
//! ```

use crate::footprint::{Footprint, Placed, flaw, object_lattice};
use crate::geometry::{
    GAP, GRAVITY, KICK_SPEED, Point, Room, STEPS_PER_SECOND, THROW_DEFAULT, THROW_FLIGHT,
    TOLERANCE, at_least, distance2, scaled_down,
};
use crate::rapier::At;

/// The direction from `from` to `to`, or east when they coincide.
fn direction(from: Point, to: Point) -> Point {
    match to.minus(from) {
        Point { x: 0, y: 0 } => Point::new(1, 0),
        away => away,
    }
}

/// A kicked object's launch velocity, in millimetres per second: horizontal, at least `KICK_SPEED`
/// long, along the kicker's centre → the object's centre (step-11 SD-O11).
pub(crate) fn kick_velocity(kicker: Point, object: Point) -> (i32, i32, i32) {
    let v = at_least(direction(kicker, object), i64::from(KICK_SPEED));
    (v.x, v.y, 0)
}

/// Where an unaimed throw goes (step-11 SD-O13): along the thrower's centre → the object's centre,
/// `THROW_DEFAULT` beyond the object, pulled back along that line until the object's footprint there
/// keeps `GAP` from the floor's edge. The farthest whole millimetre that does, found by bisection.
pub(crate) fn default_aim(room: &Room, object: &Placed, thrower: Point) -> Point {
    let line = direction(thrower, object.centre);
    let norm = line.length().max(1);
    let (hx, hy) = object.shape.half_footprint();
    let along = |t: i64| object.centre.plus(scaled_down(line, t, norm));
    let fits = |p: Point| {
        p.x - hx - GAP.value() >= room.floor.min.x
            && p.x + hx + GAP.value() <= room.floor.max.x
            && p.y - hy - GAP.value() >= room.floor.min.y
            && p.y + hy + GAP.value() <= room.floor.max.y
    };
    let farthest = i64::from(THROW_DEFAULT.value());
    if fits(along(farthest)) {
        return along(farthest);
    }
    let (mut low, mut high) = (0, farthest);
    while high - low > 1 {
        let middle = (low + high) / 2;
        if fits(along(middle)) {
            low = middle;
        } else {
            high = middle;
        }
    }
    along(low)
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
        .min_by_key(|p| (distance2(*p, point), p.y, p.x))
        .map_or((flown.centre, flown.z), |p| (p, half))
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
        // A kick along +x: (5 000, 0, 0). Along (3, 4): at_least((3, 4), 5 000) = (3 000, 4 000).
        assert_eq!(
            kick_velocity(Point::new(2_000, 5_000), Point::new(2_700, 5_000)),
            (5_000, 0, 0)
        );
        assert_eq!(
            kick_velocity(Point::new(0, 0), Point::new(3, 4)),
            (3_000, 4_000, 0)
        );
        // Coincident centres kick east.
        assert_eq!(
            kick_velocity(Point::new(7, 7), Point::new(7, 7)),
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

    #[test]
    fn an_unaimed_throw_goes_three_metres_on_unless_the_wall_is_nearer() {
        let room = cafe();
        // PO-5 a: thrower (2 000, 5 000), ball at (2 600, 5 000): (5 600, 5 000).
        let thrown = lying(ball(110), 2_600, 5_000);
        assert_eq!(
            default_aim(&room, &thrown, Point::new(2_000, 5_000)),
            Point::new(5_600, 5_000)
        );
        // Toward the east wall from (7 000, 5 000): pulled back to 8 320 − 110 − 10 = 8 200.
        let near_wall = lying(ball(110), 7_000, 5_000);
        assert_eq!(
            default_aim(&room, &near_wall, Point::new(6_400, 5_000)),
            Point::new(8_200, 5_000)
        );
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
