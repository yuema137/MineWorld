//! An arrival from another place, from nowhere, or from a position-less presence in this place
//! (step-11 SD-B8, QP-7; with objects, SD-O9 and QO-20). It always ends in this place — a resolver can
//! neither refuse an arrival nor end it elsewhere (rule (a)) — and it never pushes an object: it lands
//! on a free point, and the capacity checked at genesis, objects included, guarantees one.
//!
//! ```text
//! E1  `to`, if a person fits there: inside the floor, out of the solids, two radii and a gap
//!     from everyone, and a radius clear of every object's footprint
//! E2  else, if only people are in the way, `to` with them nudged aside under the same bounds,
//!     when the result verifies (V1–V4)
//! E3  else the nearest such free point of the 50 mm lattice; nobody moves
//! ```

use mineworld_contracts::EntityId;

use crate::footprint::{Footprint, Placed};
use crate::geometry::{GAP, LATTICE, PERSON_RADIUS, Point, Room, distance2};
use crate::rapier::{Against, Scene};
use crate::resolve::{Answer, Here, Outcome, Route};
use crate::stride::{Candidate, nudge, verifies};

/// Whether a person's disc at `p` keeps a radius from every footprint.
fn clear_of_objects(p: Point, objects: &[Footprint]) -> bool {
    objects
        .iter()
        .all(|footprint| !footprint.within(p, PERSON_RADIUS.value()))
}

pub(crate) fn entry(here: &Here, target: Point) -> Answer {
    let (room, standing) = (&here.room, &here.standing);
    let points: Vec<Point> = standing.iter().map(|(_, at)| *at).collect();
    let objects: Vec<Footprint> = here
        .objects
        .iter()
        .map(|(_, placed)| placed.footprint())
        .collect();
    let outcome = |route, generations, nudged| Outcome {
        generations,
        nudged,
        ..Outcome::plain(route)
    };
    let free = |p: Point| room.free_at(p, &points) && clear_of_objects(p, &objects);
    // E1: a person fits at `to`.
    if free(target) {
        return Answer::plain(target, Route::Entered);
    }
    // E2: only people are in the way; nudge them from `to`, under the same bounds.
    if room.admits(target, PERSON_RADIUS.value()) && clear_of_objects(target, &objects) {
        let placed: Vec<Placed> = here.objects.iter().map(|(_, placed)| *placed).collect();
        let scene = Scene::build(room, &points, &placed);
        let in_scene: Vec<usize> = (0..standing.len()).collect();
        if let Ok(nudged) = nudge(
            &scene,
            standing,
            &in_scene,
            target,
            Point::new(1, 0),
            Against::Walls,
        ) {
            let candidate = Candidate {
                walker: target,
                displaced: nudged.moved,
            };
            if verifies(room, standing, &candidate, &objects) {
                return Answer {
                    reached: target,
                    displaced: candidate
                        .displaced
                        .iter()
                        .map(|(index, at)| (standing[*index].0, *at))
                        .collect(),
                    stopped_by: None,
                    outcome: outcome(
                        Route::EnteredNudging,
                        nudged.generations,
                        candidate.displaced.len(),
                    ),
                };
            }
        }
    }
    // E3: the nearest free point of the lattice; nobody moves.
    let reached = nearest_free(room, target, &free).unwrap_or_else(|| {
        panic!(
            "bodies: no free point for an arrival at ({}, {}): the place's capacity, checked at \
             genesis, is exhausted — which only a world grown after genesis can do (DECISIONS.md \
             ARC-39 note, point 3)",
            target.x, target.y
        )
    });
    Answer {
        reached,
        displaced: Vec::new(),
        stopped_by: in_the_way(here, &objects, target),
        outcome: outcome(Route::Placed, 0, 0),
    }
}

/// The free point of the [`LATTICE`] nearest `target`, ties broken by `y` then `x` — searched outward
/// from `target` (step-11 SD-Z6).
///
/// The lattice is `lattice(floor, PERSON_RADIUS, LATTICE)`: points `(x0 + i·s, y0 + j·s)`, `0 ≤ i <
/// columns`, `0 ≤ j < rows`. The search visits it in square rings of index distance `k = 0, 1, 2 …`
/// about `(ci, cj)`, the lattice indexes nearest `target` clamped into the lattice, and keeps the
/// minimum of `(distance², y, x)` over the free points it visits.
///
/// Why it is the whole lattice's minimum: a point of ring `k` differs from `(ci, cj)` by `k` on some
/// axis, so on that axis it lies at least `k·s − m` from `target`, where `m` is the larger of
/// `target`'s two axis offsets from the point `(ci, cj)`; its distance² is at least `(k·s − m)²` when
/// `k·s > m`. Rings are visited in order and that bound grows with `k`, so once it exceeds the best
/// distance² found every unvisited point has a strictly larger key. Points are distinct, so the key
/// has no ties and the minimum is one point — the full scan's. Integers only; nothing is kept.
pub(crate) fn nearest_free(
    room: &Room,
    target: Point,
    free: &impl Fn(Point) -> bool,
) -> Option<Point> {
    let (margin, step) = (PERSON_RADIUS.value(), i64::from(LATTICE.value()));
    let floor = room.floor;
    let (x0, y0) = (
        i64::from(floor.min.x + margin),
        i64::from(floor.min.y + margin),
    );
    let (x1, y1) = (
        i64::from(floor.max.x - margin),
        i64::from(floor.max.y - margin),
    );
    if x1 < x0 || y1 < y0 {
        return None;
    }
    let (columns, rows) = ((x1 - x0) / step + 1, (y1 - y0) / step + 1);
    let (tx, ty) = (i64::from(target.x), i64::from(target.y));
    let nearest = |t: i64, origin: i64, count: i64| {
        ((t - origin + step / 2).div_euclid(step)).clamp(0, count - 1)
    };
    let (ci, cj) = (nearest(tx, x0, columns), nearest(ty, y0, rows));
    let m = (x0 + ci * step - tx).abs().max((y0 + cj * step - ty).abs());
    let last_ring = ci.max(columns - 1 - ci).max(cj).max(rows - 1 - cj);
    let point = |i: i64, j: i64| {
        Point::new(
            i32::try_from(x0 + i * step).expect("a lattice point lies on the floor"),
            i32::try_from(y0 + j * step).expect("a lattice point lies on the floor"),
        )
    };
    let mut best: Option<(i64, i32, i32)> = None;
    let consider = |best: &mut Option<(i64, i32, i32)>, i: i64, j: i64| {
        if !(0..columns).contains(&i) || !(0..rows).contains(&j) {
            return;
        }
        let p = point(i, j);
        let key = (distance2(p, target), p.y, p.x);
        if best.is_none_or(|kept| key < kept) && free(p) {
            *best = Some(key);
        }
    };
    for k in 0..=last_ring {
        if let Some((kept, _, _)) = best {
            let reach = (k * step - m).max(0);
            if reach * reach > kept {
                break;
            }
        }
        if k == 0 {
            consider(&mut best, ci, cj);
            continue;
        }
        for i in ci - k..=ci + k {
            consider(&mut best, i, cj - k);
            consider(&mut best, i, cj + k);
        }
        for j in cj - k + 1..cj + k {
            consider(&mut best, ci - k, j);
            consider(&mut best, ci + k, j);
        }
    }
    best.map(|(_, y, x)| Point::new(x, y))
}

/// What made `target` unfree: the lowest-`EntityId` person whose disc covers it, else the first
/// object (in `ItemId` order) a person there would overlap; [`None`] for a solid or the floor's edge.
fn in_the_way(here: &Here, objects: &[Footprint], target: Point) -> Option<EntityId> {
    let spacing = i64::from(2 * PERSON_RADIUS.value() + GAP.value());
    here.standing
        .iter()
        .find(|(_, at)| distance2(*at, target) < spacing * spacing)
        .map(|(person, _)| *person)
        .or_else(|| {
            objects
                .iter()
                .position(|footprint| footprint.within(target, PERSON_RADIUS.value()))
                .map(|index| here.objects[index].0.entity_id())
        })
}

#[cfg(test)]
mod tests {
    //! SD-Z6: the outward search is the full scan, point for point.

    use super::*;
    use crate::geometry::{Area, lattice};

    /// The scan SD-Z6 replaced, kept as the reference.
    fn scanned(room: &Room, target: Point, free: &impl Fn(Point) -> bool) -> Option<Point> {
        lattice(room.floor, PERSON_RADIUS.value(), LATTICE.value())
            .filter(|p| free(*p))
            .min_by_key(|p| (distance2(*p, target), p.y, p.x))
    }

    fn room(min: (i32, i32), max: (i32, i32)) -> Room {
        Room {
            floor: Area {
                min: Point::new(min.0, min.1),
                max: Point::new(max.0, max.1),
            },
            solids: Vec::new(),
        }
    }

    /// A deterministic generator (a 64-bit LCG), so that the cases are the same on every run.
    struct Cases(u64);

    impl Cases {
        fn next(&mut self, below: i32) -> i32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            i32::try_from((self.0 >> 33) % u64::from(below.unsigned_abs())).expect("small")
        }
    }

    /// 3 000 generated cases: floors from 600 mm to 12 m on a side, offset anywhere in ±20 m; targets
    /// inside, on the edge of, and up to 5 m outside the floor, on and off the lattice; occupancy from
    /// almost empty to almost full, by a hash of the point; and the empty case (nothing free).
    #[test]
    fn the_outward_search_finds_the_scans_point() {
        let mut cases = Cases(7);
        for case in 0..3_000 {
            let (x, y) = (cases.next(40_000) - 20_000, cases.next(40_000) - 20_000);
            let (w, h) = (600 + cases.next(11_400), 600 + cases.next(11_400));
            let floor = room((x, y), (x + w, y + h));
            let target = Point::new(
                x - 5_000 + cases.next(w + 10_000),
                y - 5_000 + cases.next(h + 10_000),
            );
            let (salt, density) = (cases.next(1_000_000), cases.next(1_001));
            let free = |p: Point| {
                let hash =
                    (i64::from(p.x) * 73_856_093) ^ (i64::from(p.y) * 19_349_663) ^ i64::from(salt);
                hash.rem_euclid(1_000) < i64::from(density)
            };
            assert_eq!(
                nearest_free(&floor, target, &free),
                scanned(&floor, target, &free),
                "case {case}: floor {floor:?}, target {target:?}, density {density}/1000"
            );
        }
    }

    /// The bound is strict (M-Z6). `to` on the lattice point c; free only c + (150, 200) — ring 4,
    /// distance 250 — and c + (250, 0) — ring 5, distance 250 too, and south of the first. The ring-5
    /// bound (5 × 50 − 0)² equals the best distance² found in ring 4, so the search must visit ring 5,
    /// where the tie on distance² goes to the smaller y.
    #[test]
    fn an_equal_distance_in_a_later_ring_wins_on_y() {
        let floor = room((0, 0), (10_000, 10_000));
        let c = Point::new(5_000, 5_000);
        let (north, east) = (Point::new(5_150, 5_200), Point::new(5_250, 5_000));
        let free = |p: Point| p == north || p == east;
        assert_eq!(scanned(&floor, c, &free), Some(east));
        assert_eq!(
            nearest_free(&floor, c, &free),
            Some(east),
            "ring 5 not visited"
        );
    }
}
