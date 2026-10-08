//! Integer wall strides (step-11 SD-Z4, DB-10 option 3): a stride that only the floor's edge can stop
//! is answered without Rapier.
//!
//! The centre's free region against the floor alone is `C`, the floor shrunk by a radius and the
//! controller's offset (R + GAP): an axis-aligned rectangle. When `start` lies in `C` and nothing but
//! the floor's edge is near the bounding box of `start` and `target` — no solid's box grown by R + GAP
//! meets it, every other person's centre keeps 2R + GAP from it, every object's footprint keeps
//! R + GAP from it — the walker ends at `clamp(target, C)`: each coordinate clamped into `C`. That is
//! the controller's stop at the wall (PB-5 a's 8 010) and a slide along it that keeps all the
//! tangential motion (a deliberate change for this one case, QZ-1).
//!
//! The end never lies farther from `start` than `target` does: clamping into a convex set is the
//! projection onto it, which is non-expansive, and `start` is its own projection — so
//! |clamp(target) − start| = |clamp(target) − clamp(start)| ≤ |target − start|. It lies in the box
//! (clamping moves each coordinate toward `start`'s, which is inside `C`), so nothing the conditions
//! keep away is touched. Integers only.

use crate::footprint::Footprint;
use crate::geometry::{Area, GAP, PERSON_RADIUS, Point, Room};

/// Where a stride from `start` toward `target` ends when only the floor's edge can stop it, or
/// [`None`] when something else is near enough to matter (or `start` is not in `C`).
pub(crate) fn walled(
    room: &Room,
    others: &[Point],
    objects: &[Footprint],
    start: Point,
    target: Point,
) -> Option<Point> {
    let offset = PERSON_RADIUS.value() + GAP.value();
    let inner = Area {
        min: Point::new(room.floor.min.x + offset, room.floor.min.y + offset),
        max: Point::new(room.floor.max.x - offset, room.floor.max.y - offset),
    };
    if !room.floor.holds(start, offset) {
        return None;
    }
    let path = Area::spanning(start, target);
    let apart = i64::from(2 * PERSON_RADIUS.value() + GAP.value());
    let alone = room
        .solids
        .iter()
        .all(|(solid, _)| !solid.grown(offset).meets(&path))
        && others
            .iter()
            .all(|other| path.distance2(*other) >= apart * apart)
        && objects
            .iter()
            .all(|footprint| footprint.keeps(path, offset));
    alone.then(|| inner.clamp(target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::distance2;

    /// The café room of §17.4: a 8 320 × 10 320 floor from the origin, no solids.
    fn cafe() -> Room {
        Room {
            floor: Area {
                min: Point::new(0, 0),
                max: Point::new(8_320, 10_320),
            },
            solids: Vec::new(),
        }
    }

    /// The clamp never lengthens a stride: 20 000 generated strides from points of `C`, toward targets
    /// anywhere within 6 m, inside and outside the floor (ZC-4's review item).
    #[test]
    fn a_walled_stride_is_never_longer_than_asked() {
        let room = cafe();
        let mut state: u64 = 12_345;
        let mut next = |below: i32| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            i32::try_from((state >> 33) % u64::from(below.unsigned_abs())).expect("small")
        };
        for case in 0..20_000 {
            let start = Point::new(310 + next(7_701), 310 + next(9_701));
            let target = Point::new(
                start.x - 6_000 + next(12_001),
                start.y - 6_000 + next(12_001),
            );
            let end = walled(&room, &[], &[], start, target).expect("nothing but walls");
            assert!(
                distance2(start, end) <= distance2(start, target),
                "case {case}: {start:?} → {target:?} ended at {end:?}"
            );
            assert!(room.floor.holds(end, 310), "case {case}: {end:?} outside C");
        }
    }

    /// TZ-5 e's condition by literals: a start 305 mm from the wall is outside `C`.
    #[test]
    fn a_start_outside_c_is_not_walled() {
        let room = cafe();
        assert_eq!(
            walled(
                &room,
                &[],
                &[],
                Point::new(8_015, 3_000),
                Point::new(8_400, 3_000)
            ),
            None
        );
        assert_eq!(
            walled(
                &room,
                &[],
                &[],
                Point::new(8_010, 3_000),
                Point::new(8_400, 3_000)
            ),
            Some(Point::new(8_010, 3_000))
        );
    }
}
