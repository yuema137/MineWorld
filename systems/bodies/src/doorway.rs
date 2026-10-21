//! Doorways stay passable (`DECISIONS.md` `ARC-330`; S35 N-f).
//!
//! Two behaviour models from games, adopted as data-free rules that read only a place's own geometry:
//!
//! ```text
//! R1  arrival spot   a walk to a place as a whole ends at the first free point of the 50 mm lattice,
//!                    in entry's E3 order from where the walker entered, that is THRESHOLD_CLEAR from
//!                    every doorway point of the place, free by the route planner's own test, and two
//!                    radii and a gap from everyone else standing there; none → the walk ends on
//!                    entering (movement's Wayfinder::arrival_spot, answered in route.rs)
//! R2  give way       a walker's stride blocked by people who rank lower — no walk in progress, or
//!                    (`ARC-333` R1) a walk that started later, ties to the higher id — moves each of
//!                    them, nearest the walker first, to the nearest free lattice point out of the
//!                    walker's corridor, within GIVE_WAY_MAX, reached without crossing a solid —
//!                    THRESHOLD_CLEAR from every doorway point when such a point exists; the
//!                    result is verified (V1–V4) like every candidate (stride.rs)
//! ```
//!
//! Neither rule names a place, a person or a world. Integers only.

use mineworld_contracts::{Millimetres, PlaceId};
use mineworld_kernel::WorldRead;
use mineworld_movement::Passages;

use crate::entry::nearest_free;
use crate::footprint::Footprint;
use crate::geometry::{
    GAP, LATTICE, NUDGED_MAX, PERSON_RADIUS, Point, Room, TOLERANCE, distance2, segment_clear_of,
};

/// How far, centre to centre, an arrival spot and a give-way point keep from every doorway point:
/// one person standing (R 250) plus a passing lane of one person's width (0.6 m) plus the gap. The
/// minimum clear door width of 815 mm (ADA 2010 §404.2.3) plus a body's radius each side; Fruin (1971)
/// level-of-service A gives a standing pedestrian about 1.2 m².
pub const THRESHOLD_CLEAR: Millimetres = Millimetres::new(1_200);

/// The farthest a person who is not walking steps aside for one who is: one comfortable step aside
/// and back, the same 1.2 m as [`THRESHOLD_CLEAR`].
pub const GIVE_WAY_MAX: Millimetres = Millimetres::new(1_200);

/// Every doorway point of `place` in its own frame: the near side of each of movement's passages out
/// of it. A side the world does not model has no point.
pub(crate) fn doors(world: &WorldRead<'_>, place: PlaceId) -> Vec<Point> {
    world
        .component::<Passages>(place.entity_id())
        .into_iter()
        .flat_map(Passages::iter)
        .filter_map(|passage| passage.here())
        .map(|at| Point::new(at.x().value(), at.y().value()))
        .collect()
}

/// Whether `p` keeps [`THRESHOLD_CLEAR`] from every point of `doors`.
pub(crate) fn off_the_threshold(p: Point, doors: &[Point]) -> bool {
    let clear = i64::from(THRESHOLD_CLEAR.value());
    doors
        .iter()
        .all(|door| distance2(p, *door) >= clear * clear)
}

/// The most people one stride's give-way may move: twice a nudge's bound, since a cluster round a
/// doorway is what this rule exists for.
const GIVE_WAY_PEOPLE: usize = 2 * NUDGED_MAX;

/// R2 (`ARC-330`), widened by `ARC-333` R1: the people of `others` (indexes, with their new points)
/// who give way so that a walker at `start` can stride to `target` along a clear corridor — or
/// [`None`] when one of the people in the way does not yield to this walker (`yields[index]` false:
/// a walker whose walk ranks higher), has no point to go to, or the corridor still is not clear
/// after [`GIVE_WAY_PEOPLE`] of them moved.
///
/// The people in the way are taken nearest the walker first (by their projection on the stride, then
/// by index). Each goes to the nearest point (entry's E3 order from where they stand) within
/// [`GIVE_WAY_MAX`] that is free (a radius inside the floor and from every solid, two radii and a gap
/// from everyone, a radius from every object), out of the walker's corridor and reached by a segment
/// that meets no solid — preferring a point [`THRESHOLD_CLEAR`] from every doorway, and taking the
/// nearest one off the corridor only when none is. Like a nudge, the step aside sweeps against the
/// fixed geometry only; the result is verified against everyone by the caller.
pub(crate) fn give_way(
    room: &Room,
    others: &[Point],
    yields: &[bool],
    doors: &[Point],
    objects: &[Footprint],
    start: Point,
    target: Point,
) -> Option<Vec<(usize, Point)>> {
    let apart = 2 * PERSON_RADIUS.value() + GAP.value();
    let reach = i64::from(GIVE_WAY_MAX.value());
    let mut now = others.to_vec();
    let mut moved: Vec<(usize, Point)> = Vec::new();
    loop {
        let in_the_way = (0..now.len())
            .filter(|index| !segment_clear_of(start, target, now[*index], apart))
            .min_by_key(|index| (along(start, target, now[*index]), *index));
        let Some(index) = in_the_way else {
            return (!moved.is_empty() && room.corridor_clear(start, target, &now))
                .then_some(moved);
        };
        if moved.len() >= GIVE_WAY_PEOPLE
            || !yields[index]
            || moved.iter().any(|(done, _)| *done == index)
        {
            return None;
        }
        let from = now[index];
        let mut everyone: Vec<Point> = now
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .map(|(_, at)| *at)
            .collect();
        everyone.push(start);
        let aside = |p: Point| {
            distance2(p, from) <= reach * reach
                && room.free_at(p, &everyone)
                && segment_clear_of(start, target, p, apart)
                && objects
                    .iter()
                    .all(|footprint| !footprint.within(p, PERSON_RADIUS.value()))
                && clear_of_solids(room, from, p)
        };
        let to = nearest_free(room, from, &|p| aside(p) && off_the_threshold(p, doors))
            .or_else(|| nearest_free(room, from, &aside))?;
        now[index] = to;
        moved.push((index, to));
    }
}

/// How far along the stride from `start` to `target` the point `p` lies, unscaled.
fn along(start: Point, target: Point, p: Point) -> i64 {
    let (dx, dy) = (
        i64::from(target.x) - i64::from(start.x),
        i64::from(target.y) - i64::from(start.y),
    );
    dx * (i64::from(p.x) - i64::from(start.x)) + dy * (i64::from(p.y) - i64::from(start.y))
}

/// Whether a person stepping straight from `from` to `to` keeps clear of every solid: every point of
/// the segment, sampled at half the lattice step and at its end, a radius less the tolerance from
/// every solid. The floor is a rectangle, so a segment between two points on it stays on it.
fn clear_of_solids(room: &Room, from: Point, to: Point) -> bool {
    let length = to.minus(from).length();
    let step = i64::from(LATTICE.value() / 2);
    let steps = (length / step).max(1);
    let margin = PERSON_RADIUS.value() - TOLERANCE.value();
    (0..=steps).all(|k| {
        let at = Point::new(
            from.x + i32::try_from((i64::from(to.x - from.x) * k) / steps).expect("on the floor"),
            from.y + i32::try_from((i64::from(to.y - from.y) * k) / steps).expect("on the floor"),
        );
        room.clear_of_solids(at, margin)
    })
}
