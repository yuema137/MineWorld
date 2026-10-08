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
use crate::geometry::{GAP, LATTICE, PERSON_RADIUS, Point, Room, distance2, lattice};
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

/// The free point of the [`LATTICE`] nearest `target`, ties broken by `y` then `x`.
fn nearest_free(room: &Room, target: Point, free: &impl Fn(Point) -> bool) -> Option<Point> {
    lattice(room.floor, PERSON_RADIUS.value(), LATTICE.value())
        .filter(|p| free(*p))
        .min_by_key(|p| (distance2(*p, target), p.y, p.x))
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
