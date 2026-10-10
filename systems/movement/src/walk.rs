//! A walk, decided (`DECISIONS.md` `ARC-75`; step-11 SD-N1 … SD-N10): which places it passes through,
//! the route of each leg, and what one `walk-step` does.
//!
//! ```text
//! begin   walk-to:   the place chain (breadth-first over Passages, places in PlaceId order) and the
//!                    first leg's route; refused no-route when either is missing
//! step    walk-step: account the last stride (stopped short or displaced → re-plan; < 50 mm → a
//!                    stall); arrived? ; plan the leg if it is not planned; then one stride toward
//!                    the next waypoint, at most WALK_STRIDE and never past it, or the crossing at a
//!                    doorway — or the end of the walk
//! arrived the walker's own recorded arrival ends the walk when it is the destination
//! ```
//!
//! Pure functions of the world as it stands and the walk as stored: nothing here reads a clock, the
//! time scale or a random source, and nothing writes. Integers only — squares in `i128`, roots by
//! `isqrt`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use mineworld_contracts::{
    EntityId, EntityType, LifecycleState, LocalPosition, Location, Millimetres, PersonId, PlaceId,
    Rejection, RejectionCode,
};
use mineworld_kernel::WorldRead;
use mineworld_presence::Presence;

use crate::action::{
    Destination, PERSON_APPROACH, REPLANS_MAX, STALL_PROGRESS, STALLS_MAX, TARGET_MOVED,
    WALK_STRIDE,
};
use crate::component::{Asked, Passages, Walking};
use crate::event::Ended;
use crate::wayfinder::{self, RouteAnswer, RouteAsk};

/// Why a walk is refused when no way leads to its destination (step-11 note N-1): part of this pack's
/// public vocabulary, never renamed.
pub const NO_ROUTE: RejectionCode = RejectionCode::from_static("no-route");

/// The refusal for a destination no way leads to.
pub(crate) fn no_route(detail: &str) -> Rejection {
    Rejection::System {
        code: NO_ROUTE,
        detail: Some(detail.to_owned()),
    }
}

/// What one `walk-step` does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Step {
    /// One stride — or the crossing at a doorway — to `to`, the walk then stored as `walking`.
    Stride { to: Location, walking: Box<Walking> },
    /// The walk ends.
    End(Ended),
}

/// Where `person` is, as presence records it.
pub(crate) fn presence(world: &WorldRead<'_>, person: EntityId) -> Option<Location> {
    world.component::<Presence>(person).map(Presence::location)
}

/// The places to enter, in order, to get from `from` to `to` through passages — empty when they are
/// one place — or [`None`] when no chain of passages joins them. Breadth-first, neighbours in
/// `PlaceId` order, so the chain is the shortest and the same on every run.
pub(crate) fn places_to(world: &WorldRead<'_>, from: PlaceId, to: PlaceId) -> Option<Vec<PlaceId>> {
    if from == to {
        return Some(Vec::new());
    }
    let mut came_from: BTreeMap<PlaceId, PlaceId> = BTreeMap::new();
    let mut seen = BTreeSet::from([from]);
    let mut queue = VecDeque::from([from]);
    while let Some(place) = queue.pop_front() {
        let Some(passages) = world.component::<Passages>(place.entity_id()) else {
            continue;
        };
        for passage in passages.iter() {
            let next = passage.to();
            if !seen.insert(next) {
                continue;
            }
            came_from.insert(next, place);
            if next == to {
                let mut chain = vec![to];
                let mut at = to;
                while let Some(previous) = came_from.get(&at).copied().filter(|p| *p != from) {
                    chain.push(previous);
                    at = previous;
                }
                chain.reverse();
                return Some(chain);
            }
            queue.push_back(next);
        }
    }
    None
}

/// Where the current leg ends, as the world stands: the doorway into the next place, or the
/// destination in its own place. `Err` ends the walk.
struct Target {
    /// The doorway's position here, or the destination's, when the world models one.
    at: Option<LocalPosition>,
    /// The next place and the doorway's position in it, when the leg ends at a doorway.
    crossing: Option<(PlaceId, Option<LocalPosition>)>,
}

fn target(world: &WorldRead<'_>, walking: &Walking, here: &Location) -> Result<Target, Ended> {
    if let Some(next) = walking.legs.first().copied() {
        let passage = world
            .component::<Passages>(here.place().entity_id())
            .and_then(|passages| passages.to(next))
            .ok_or(Ended::NoRoute)?;
        return Ok(Target {
            at: passage.here(),
            crossing: Some((next, passage.there())),
        });
    }
    match walking.destination {
        Destination::Place(location) => {
            if location.place() != here.place() {
                return Err(Ended::NoRoute);
            }
            Ok(Target {
                at: location.local(),
                crossing: None,
            })
        }
        Destination::Person(person) => {
            let there = person_in(world, person, here.place()).ok_or(Ended::NoRoute)?;
            Ok(Target {
                at: there.local(),
                crossing: None,
            })
        }
    }
}

/// Where a living `person` stands, when it is in `place`.
fn person_in(world: &WorldRead<'_>, person: PersonId, place: PlaceId) -> Option<Location> {
    let living = world.entity(person.entity_id()).is_some_and(|entity| {
        entity.entity_type() == EntityType::Person
            && entity.lifecycle() != LifecycleState::Destroyed
    });
    presence(world, person.entity_id()).filter(|location| living && location.place() == place)
}

/// The waypoints of one leg in `place`, from `from` to `to`, planning round `avoid`: the first
/// registered wayfinder's answer, or the straight segment when none answers. `Err` when the
/// wayfinder answers that no way leads there.
fn plan(
    world: &WorldRead<'_>,
    person: PersonId,
    place: PlaceId,
    from: LocalPosition,
    to: LocalPosition,
    avoid: Option<EntityId>,
) -> Result<Vec<LocalPosition>, ()> {
    let avoid = avoid.and_then(|entity| PersonId::new(entity, EntityType::Person).ok());
    let ask = RouteAsk::new(person, place, from, to, avoid);
    match wayfinder::route(world, &ask) {
        None => Ok(vec![to]),
        Some(RouteAnswer::Waypoints(waypoints)) => Ok(waypoints.into_points()),
        Some(RouteAnswer::Unreachable) => Err(()),
    }
}

/// The walk `walk-to` would record for `person`, standing at `here`, going to `destination` — or why
/// it is refused (`validate` and `resolve` both ask, so they cannot disagree).
///
/// ```text
/// person destination   a living person other than the walker, with a presence, else
///                      PreconditionFailed; in the walker's place, else TooFarAway
/// place destination    presence admits it (asked by the caller)
/// the chain            places_to, else no-route
/// the first leg        planned when both ends have positions; Unreachable → no-route
/// ```
pub(crate) fn begin(
    world: &WorldRead<'_>,
    person: PersonId,
    here: &Location,
    destination: Destination,
) -> Result<Walking, Rejection> {
    let place = match destination {
        Destination::Place(location) => location.place(),
        Destination::Person(other) => {
            if other == person {
                return Err(Rejection::PreconditionFailed);
            }
            let living = world.entity(other.entity_id()).is_some_and(|entity| {
                entity.entity_type() == EntityType::Person
                    && entity.lifecycle() != LifecycleState::Destroyed
            });
            let there = presence(world, other.entity_id())
                .filter(|_| living)
                .ok_or(Rejection::PreconditionFailed)?;
            if there.place() != here.place() {
                return Err(Rejection::TooFarAway);
            }
            there.place()
        }
    };
    let legs = places_to(world, here.place(), place)
        .ok_or_else(|| no_route("no chain of passages leads to the destination's place"))?;
    let mut walking = Walking {
        destination,
        legs,
        waypoints: Vec::new(),
        goal: None,
        progress: None,
        stalls: 0,
        replans: 0,
        stopped_by: None,
    };
    let target = target(world, &walking, here).map_err(|_| no_route("the destination left"))?;
    if let (Some(from), Some(to)) = (here.local(), target.at) {
        let waypoints = plan(world, person, here.place(), from, to, None)
            .map_err(|()| no_route("no way leads there inside the place"))?;
        walking.goal = waypoints.last().copied();
        walking.waypoints = waypoints;
    }
    Ok(walking)
}

/// What one `walk-step` does for `person`, standing at `here`, on `walking` (step-11 SD-N2, SD-N9).
pub(crate) fn step(
    world: &WorldRead<'_>,
    person: PersonId,
    here: &Location,
    walking: &Walking,
) -> Step {
    let mut walking = walking.clone();
    // 1. The last stride: did it end where it was asked, and did it get anywhere?
    let mut replan = false;
    if let Some(last) = walking.progress {
        replan = *here != last.to;
        let progressed = match (last.from.local(), here.local()) {
            (Some(from), Some(now)) if last.from.place() == here.place() => {
                distance2(from, now) >= square(STALL_PROGRESS)
            }
            _ => true,
        };
        walking.stalls = if progressed { 0 } else { walking.stalls + 1 };
        if walking.stalls >= STALLS_MAX {
            return Step::End(Ended::Stalled);
        }
    }
    // 2. Already there?
    if arrived(world, &walking, here) {
        return Step::End(Ended::Arrived);
    }
    let target = match target(world, &walking, here) {
        Ok(target) => target,
        Err(ended) => return Step::End(ended),
    };
    // 3. A person destination that moved far from where the leg was planned to end.
    if let (Destination::Person(_), true, Some(goal), Some(now)) = (
        walking.destination,
        walking.legs.is_empty(),
        walking.goal,
        target.at,
    ) && distance2(goal, now) > square(TARGET_MOVED)
    {
        replan = true;
    }
    let (Some(from), Some(to)) = (here.local(), target.at) else {
        // No positions to plan between: cross, or go straight to the destination.
        return match target.crossing {
            Some((next, there)) => cross(walking, here, next, there),
            None => match (here.local(), target.at) {
                (None, Some(at)) => stride_to(walking, here, at),
                _ => Step::End(Ended::Arrived),
            },
        };
    };
    // 4. The leg's route: planned once per leg, again when a stride went wrong or the target moved.
    if walking.goal.is_none() || replan {
        if walking.goal.is_some() {
            walking.replans += 1;
            if walking.replans > REPLANS_MAX {
                return Step::End(Ended::Stalled);
            }
        }
        let avoid = if replan { walking.stopped_by } else { None };
        match plan(world, person, here.place(), from, to, avoid) {
            Ok(waypoints) => {
                walking.goal = waypoints.last().copied();
                walking.waypoints = waypoints;
            }
            Err(()) => return Step::End(Ended::NoRoute),
        }
    }
    walking.stopped_by = None;
    while walking.waypoints.first() == Some(&from) {
        walking.waypoints.remove(0);
    }
    // 5. One stride, the crossing, or the end.
    match (walking.waypoints.first().copied(), target.crossing) {
        (Some(next), _) => {
            let aim = match walking.destination {
                Destination::Person(_)
                    if walking.waypoints.len() == 1 && walking.legs.is_empty() =>
                {
                    approach(from, next)
                }
                _ => next,
            };
            stride_to(walking, here, toward(from, aim, WALK_STRIDE))
        }
        (None, Some((next, there))) => cross(walking, here, next, there),
        (None, None) => Step::End(Ended::Arrived),
    }
}

/// A stride to `to` in the walker's place, the walk stored with the stride as asked.
fn stride_to(mut walking: Walking, here: &Location, to: LocalPosition) -> Step {
    let mut location = Location::in_place(here.place()).with_local(to);
    if let Some(facing) = here.facing() {
        location = location.with_facing(facing);
    }
    walking.progress = Some(Asked {
        from: *here,
        to: location,
    });
    Step::Stride {
        to: location,
        walking: Box::new(walking),
    }
}

/// The crossing into `next`, at the doorway's position there when the world models it; the next leg is
/// planned at the next step, from wherever the walker is then.
fn cross(
    mut walking: Walking,
    here: &Location,
    next: PlaceId,
    there: Option<LocalPosition>,
) -> Step {
    let mut location = Location::in_place(next);
    if let Some(there) = there {
        location = location.with_local(there);
    }
    if let Some(facing) = here.facing() {
        location = location.with_facing(facing);
    }
    walking.legs.remove(0);
    walking.waypoints.clear();
    walking.goal = None;
    walking.progress = Some(Asked {
        from: *here,
        to: location,
    });
    Step::Stride {
        to: location,
        walking: Box::new(walking),
    }
}

/// Whether a walker at `here` has arrived: in the destination's place with no more places to enter,
/// and at the leg's planned end (a place destination), or within [`PERSON_APPROACH`] of the person.
/// A place destination without a position is reached on entering its place.
pub(crate) fn arrived(world: &WorldRead<'_>, walking: &Walking, here: &Location) -> bool {
    if !walking.legs.is_empty() {
        return false;
    }
    match walking.destination {
        Destination::Place(location) => {
            location.place() == here.place()
                && match location.local() {
                    None => true,
                    Some(_) => walking.goal.is_some() && here.local() == walking.goal,
                }
        }
        Destination::Person(person) => {
            person_in(world, person, here.place()).is_some_and(|there| {
                match (here.local(), there.local()) {
                    (Some(mine), Some(theirs)) => {
                        distance2(mine, theirs) <= square(PERSON_APPROACH)
                    }
                    _ => true,
                }
            })
        }
    }
}

/// Where a stride toward a person's position `at` should aim from `from`: the point on the way that is
/// [`PERSON_APPROACH`] from `at`, or `at` itself when the walker is already that near.
fn approach(from: LocalPosition, at: LocalPosition) -> LocalPosition {
    let d2 = distance2(from, at);
    if d2 <= square(PERSON_APPROACH) {
        return at;
    }
    let length = ceil_root(d2);
    let reach = i128::from(PERSON_APPROACH.value());
    // `at + (from − at) · A / ⌈|from − at|⌉`, each axis truncated toward zero: never farther than A
    // from `at`.
    offset(at, from, reach, length)
}

/// The point at most `most` from `from` along the segment to `to`; `to` itself when it is that near.
pub(crate) fn toward(from: LocalPosition, to: LocalPosition, most: Millimetres) -> LocalPosition {
    let d2 = distance2(from, to);
    if d2 <= square(most) {
        return to;
    }
    offset(from, to, i128::from(most.value()), ceil_root(d2))
}

/// `origin + (towards − origin) · numerator / denominator`, each axis truncated toward zero — so the
/// result is never farther from `origin` than the exact scaling.
fn offset(
    origin: LocalPosition,
    towards: LocalPosition,
    numerator: i128,
    denominator: i128,
) -> LocalPosition {
    let axis = |o: Millimetres, t: Millimetres| {
        let (o, t) = (i128::from(o.value()), i128::from(t.value()));
        let moved = o + (t - o) * numerator / denominator;
        Millimetres::new(i32::try_from(moved).expect("a point between two positions fits"))
    };
    LocalPosition::new(
        axis(origin.x(), towards.x()),
        axis(origin.y(), towards.y()),
        axis(origin.z(), towards.z()),
    )
}

/// The squared distance between two positions, in square millimetres, exactly.
pub(crate) fn distance2(a: LocalPosition, b: LocalPosition) -> i128 {
    let axis = |p: Millimetres, q: Millimetres| {
        let d = i128::from(p.value()) - i128::from(q.value());
        d * d
    };
    axis(a.x(), b.x()) + axis(a.y(), b.y()) + axis(a.z(), b.z())
}

fn square(length: Millimetres) -> i128 {
    let v = i128::from(length.value());
    v * v
}

/// ⌈√v⌉ for `v > 0`.
fn ceil_root(v: i128) -> i128 {
    let root = v.isqrt();
    if root * root == v { root } else { root + 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i32, y: i32) -> LocalPosition {
        LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
    }

    /// A stride is never longer than asked, ends exactly at a waypoint within reach, and keeps the
    /// direction: hand-computed literals.
    #[test]
    fn a_stride_is_at_most_the_walk_stride_and_ends_at_a_near_waypoint() {
        // 3 000 mm east: 1 340 mm east exactly.
        assert_eq!(toward(p(0, 0), p(3_000, 0), WALK_STRIDE), p(1_340, 0));
        // 2 000 mm east, between one and two strides: still one stride, 1 340 mm east.
        assert_eq!(toward(p(0, 0), p(2_000, 0), WALK_STRIDE), p(1_340, 0));
        // 1 000 mm away: the waypoint itself.
        assert_eq!(toward(p(0, 0), p(600, 800), WALK_STRIDE), p(600, 800));
        // (3 000, 4 000), 5 000 mm: 1 340 / 5 000 of it = (804, 1 072), exactly 1 340 mm.
        assert_eq!(toward(p(0, 0), p(3_000, 4_000), WALK_STRIDE), p(804, 1_072));
        // (1 000, 1 000), ⌈1 414.2⌉ = 1 415: 1 000 · 1 340 / 1 415 = 946.9 → 946 on each axis,
        // 1 337.9 mm — never longer than 1 340.
        assert_eq!(toward(p(0, 0), p(1_000, 1_000), WALK_STRIDE), p(946, 946));
    }

    /// The approach point lies 1 200 mm short of the person, on the line from the walker.
    #[test]
    fn a_walk_to_a_person_aims_at_personal_distance() {
        // The person at (5 000, 0), the walker at the origin: aim at (3 800, 0).
        assert_eq!(approach(p(0, 0), p(5_000, 0)), p(3_800, 0));
        // Already within 1 200 mm: the person's own position.
        assert_eq!(approach(p(4_000, 0), p(5_000, 0)), p(5_000, 0));
    }
}
