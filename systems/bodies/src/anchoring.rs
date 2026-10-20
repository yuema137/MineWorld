//! Who is held where they stand: the build's catalog of [`Anchoring`]s (`ARC-62` item 5, `ARC-147`
//! note of 2026-10-19; `pr-s34b-bodies-anchoring.md`).
//!
//! ```text
//! nudge      an anchored person is never moved; a pusher that would touch one fails the pass, so
//!            the walker is blocked at first contact (stride.rs), and an entry falls through to the
//!            nearest free point (entry.rs E3)
//! shove      an anchored target is TargetUnavailable, in validate and in the offer (shove.rs)
//! genesis    every fixed point an implementer names lies where a person fits, or the world does
//!            not load: bodies-fixed-point-in-solid (fixed_points_fit)
//! ```
//!
//! Bodies names no implementer (I-b-4). A pack that pins a person — posture's seated people today, a
//! bed or a vehicle later — implements the trait, and its host lists it on the `extension` line of the
//! installed set. A pack that starts or changes anchoring bumps its own version (SD-b-7).

use std::sync::OnceLock;

use mineworld_contracts::{
    EntityId, EntityType, LocalPosition, PersonId, PlaceId, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::WorldRead;

use crate::component::PlaceShape;
use crate::geometry::{GAP, PERSON_RADIUS, Point, Room};
use crate::system::name;

/// A pack's answer to "is this person held where they stand" (`ARC-62` item 5).
///
/// Asked by bodies, in ascending [`Anchoring::anchoring_of`] order. Handed a [`WorldRead`] and nothing
/// else: it can consult any state and write none. The contract an implementation keeps:
///
/// - **pure:** its answers are a function of its arguments alone — no cache, no clock, no float, no
///   hash map; [`Anchoring::fixed_points`] in key order (I-b-3);
/// - **inert where its state is absent:** `false` and an empty list where its pack holds nothing;
/// - **its pack refuses to join an unregistered world:** the pack's `System::install` calls
///   [`require_anchoring`] with its own id first.
pub trait Anchoring: Send + Sync {
    /// The system this implementer belongs to.
    fn anchoring_of(&self) -> SystemId;

    /// Whether `person` is held where they stand: never nudged, never shoved.
    fn anchored(&self, world: &WorldRead<'_>, person: PersonId) -> bool;

    /// The points of `place` a person must be able to occupy, each with a name an author recognizes
    /// (posture: the sit point's key), in key order.
    fn fixed_points(&self, world: &WorldRead<'_>, place: PlaceId) -> Vec<(String, LocalPosition)>;
}

/// The build's implementers, sorted by id: written once per process, by [`register_anchorings`].
static CATALOG: OnceLock<Vec<Box<dyn Anchoring>>> = OnceLock::new();

fn ids(anchorings: &[Box<dyn Anchoring>]) -> Vec<SystemId> {
    anchorings
        .iter()
        .map(|anchoring| anchoring.anchoring_of())
        .collect()
}

/// Registers the build's implementers: once per process, before any world is assembled. Sorted by
/// id; registering the same ids again does nothing.
///
/// # Panics
///
/// When the list names one id twice, and when a different list was registered before (`ARC-62`).
pub fn register_anchorings(mut anchorings: Vec<Box<dyn Anchoring>>) {
    anchorings.sort_by_key(|anchoring| anchoring.anchoring_of());
    let offered = ids(&anchorings);
    if let Some(pair) = offered.windows(2).find(|pair| pair[0] == pair[1]) {
        panic!(
            "register_anchorings was given two anchorings for '{}': a build has one per system \
             (DECISIONS.md ARC-62)",
            pair[0]
        );
    }
    let mut pending = Some(anchorings);
    let registered = CATALOG.get_or_init(|| pending.take().unwrap_or_default());
    if pending.is_none() {
        return;
    }
    let current = ids(registered);
    assert!(
        current == offered,
        "register_anchorings was called with {offered:?}, but this process already registered \
         {current:?}: one build has one catalog of anchorings (DECISIONS.md ARC-62)"
    );
}

/// The ids of the registered implementers, in the order they are asked; [`None`] until something
/// registers.
pub fn registered_anchorings() -> Option<Vec<SystemId>> {
    CATALOG.get().map(|anchorings| ids(anchorings))
}

/// Panics unless `anchoring` is registered: what an implementer's pack calls first in its
/// `System::install`, so a host that forgot the catalog line fails loudly instead of letting walkers
/// push the people it holds.
///
/// # Panics
///
/// When nothing was registered, or the registered list does not name `anchoring`. The message names
/// the pack, [`register_anchorings`] and `ARC-62`.
pub fn require_anchoring(anchoring: &SystemId) {
    match CATALOG.get() {
        None => panic!(
            "the '{anchoring}' system holds people in place, but this process never registered the \
             build's anchorings: a host calls mineworld_bodies::register_anchorings before it \
             installs an anchoring pack, as worldpack::compose does (DECISIONS.md ARC-62)"
        ),
        Some(anchorings) => assert!(
            anchorings
                .iter()
                .any(|registered| registered.anchoring_of() == *anchoring),
            "the '{anchoring}' system holds people in place, but the build's anchorings {:?} do not \
             name it: list it on the register_anchorings line of the installed set (DECISIONS.md \
             ARC-62)",
            ids(anchorings)
        ),
    }
}

fn catalog() -> &'static [Box<dyn Anchoring>] {
    CATALOG.get().map_or(&[], Vec::as_slice)
}

/// Whether any registered implementer holds `person` where they stand.
pub fn is_anchored(world: &WorldRead<'_>, person: PersonId) -> bool {
    catalog()
        .iter()
        .any(|anchoring| anchoring.anchored(world, person))
}

/// Whether the entity `person` is a person some implementer holds.
pub(crate) fn holds(world: &WorldRead<'_>, person: EntityId) -> bool {
    !catalog().is_empty()
        && PersonId::new(person, EntityType::Person).is_ok_and(|person| is_anchored(world, person))
}

/// One flag per person of `standing`, parallel to it; empty (nobody anchored) when no implementer is
/// registered, so the inert path allocates nothing per person (I-b-1).
pub(crate) fn anchored_in(world: &WorldRead<'_>, standing: &[(EntityId, Point)]) -> Vec<bool> {
    if catalog().is_empty() {
        return Vec::new();
    }
    standing
        .iter()
        .map(|(person, _)| holds(world, *person))
        .collect()
}

/// Every implementer's fixed points of `place`, in implementer order then key order.
pub(crate) fn fixed_points_of(
    world: &WorldRead<'_>,
    place: PlaceId,
) -> Vec<(String, LocalPosition)> {
    catalog()
        .iter()
        .flat_map(|anchoring| anchoring.fixed_points(world, place))
        .collect()
}

/// A fixed point of a place where a person does not fit (SD-b-5).
const FIXED_POINT_IN_SOLID: RejectionCode =
    RejectionCode::from_static("bodies-fixed-point-in-solid");

/// SD-b-5: every point of `points` lies where a person fits in `place` — inside the floor by
/// [`PERSON_RADIUS`], and clear of every solid by `PERSON_RADIUS + GAP` — or the place is refused,
/// naming the place, the point's name, the point and what it meets. Skipped (`Ok`) when the place has
/// no shape: without bodies there is no floor to check against.
///
/// Called by whichever of the two reductions comes second: bodies' `place-shaped` (with the
/// implementers' points) and an implementer's own reduction of its points (with the place's shape).
///
/// # Errors
///
/// `Rejection::System { code: "bodies-fixed-point-in-solid", .. }` for the first point that does not
/// fit.
pub fn fixed_points_fit(
    world: &WorldRead<'_>,
    place: PlaceId,
    points: &[(String, LocalPosition)],
) -> Result<(), Rejection> {
    match world.component::<PlaceShape>(place.entity_id()) {
        Some(shape) => points_fit(world, place, &shape.room(), points),
        None => Ok(()),
    }
}

/// [`fixed_points_fit`] against a room not yet written.
pub(crate) fn points_fit(
    world: &WorldRead<'_>,
    place: PlaceId,
    room: &Room,
    points: &[(String, LocalPosition)],
) -> Result<(), Rejection> {
    let (radius, keep) = (PERSON_RADIUS.value(), PERSON_RADIUS.value() + GAP.value());
    for (key, at) in points {
        let p = Point::new(at.x().value(), at.y().value());
        let refused = |what: String| Rejection::System {
            code: FIXED_POINT_IN_SOLID,
            detail: Some(format!(
                "the fixed point '{key}' of {} lies at ({}, {}), {what}; a person held there keeps \
                 {radius} mm inside the floor and {keep} mm from every solid",
                name(world, place.entity_id()),
                p.x,
                p.y,
            )),
        };
        if !room.floor.holds(p, radius) {
            return Err(refused(format!(
                "outside the floor from ({}, {}) to ({}, {}) shrunk by {radius} mm",
                room.floor.min.x, room.floor.min.y, room.floor.max.x, room.floor.max.y
            )));
        }
        let reach = i64::from(keep) * i64::from(keep);
        if let Some((index, (area, _))) = room
            .solids
            .iter()
            .enumerate()
            .find(|(_, (area, _))| area.distance2(p) < reach)
        {
            return Err(refused(format!(
                "{} mm from solid {index} (from ({}, {}) to ({}, {}))",
                area.distance2(p).isqrt(),
                area.min.x,
                area.min.y,
                area.max.x,
                area.max.y
            )));
        }
    }
    Ok(())
}
