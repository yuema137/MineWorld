//! This pack's answer to presence's question: what an arrival into a shaped place actually achieves
//! (`DECISIONS.md` `ARC-39` and its notes; step-11 §4.5, SD-B6 … SD-B9, SD-B15, SD-O9).
//!
//! ```text
//! inert       `to` has no position, or its place has no shape          → so_far, untouched
//! earlier     another resolver already changed the arrival              → so_far, untouched
//! guard       the place as it stands breaks the invariant               → panic, naming what it found
//! stride      within the place, from a position                         `stride.rs`
//! entry       from another place, from nowhere, or from no position     `entry.rs`
//! ```
//!
//! Every position is whole millimetres; the sweeps, the nudges and the pushes' casts are Rapier's
//! (`rapier.rs`), everything else is integer geometry. The resolver keeps nothing: no static, no
//! cache, no clock — a replayed world asks it again and is told the same thing (`ARC-25`).

use mineworld_contracts::SystemId;
use mineworld_contracts::{
    EntityId, EntityType, ItemId, LocalPosition, Location, Millimetres, PersonId, PlaceId,
};
use mineworld_kernel::{SystemIdentity, WorldRead};
use mineworld_presence::{ArrivalResolver, Arriving, Presence, Resolution};

use crate::component::PlaceShape;
use crate::entry::entry;
use crate::footprint::{Footprint, Placed, flaw};
use crate::geometry::{CLEARANCE, PERSON_RADIUS, Point, Room, TOLERANCE, closest_pair};
use crate::objects::lying_in;
use crate::stride::stride;
use crate::system::{BodiesSystem, name, standing_in};

/// Which steps a resolution runs. Production runs every step; a unit test may turn one off to show
/// what it is for (step-11 PB-8).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Policy {
    /// The head-on bias (step-11 QB-16, SD-B10).
    pub(crate) bias: bool,
    /// Verify, then degrade (step-11 DC-8, I-12).
    pub(crate) verify: bool,
    /// A stride away from a person within the controller's offset is not stopped by them (step-11
    /// SD-Z5, FU-12c-1).
    pub(crate) away_free: bool,
}

/// What every world runs.
pub(crate) const PRODUCTION: Policy = Policy {
    bias: true,
    verify: true,
    away_free: true,
};

/// The way an arrival into a shaped place was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// A stride whose corridor was clear: no scene was built, and the arrival is exactly as asked.
    Clear,
    /// A stride that was swept.
    Swept,
    /// An entry onto a free point.
    Entered,
    /// An entry onto a point people stood on, who were nudged aside.
    EnteredNudging,
    /// An entry placed at the nearest free lattice point instead.
    Placed,
}

/// Whether, and how far, verification degraded a stride's result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Degraded {
    /// The result stood as resolved.
    No,
    /// Re-resolved as blocked at contact, nobody moved.
    Blocked,
    /// The blocked advance halved this many times.
    Halved(u32),
    /// The walker stayed where they were.
    Stayed,
}

/// How a stride treated the loose objects of its place (step-11 SD-O9 step 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Objects {
    /// Nothing was pushed: there were none in the way, or verification degraded the stride.
    Untouched,
    /// The prediction was accepted: this many objects will be pushed by the reactions.
    Pushed(usize),
    /// The pushes could not all be made, so the stride was resolved with the objects solid.
    Solid,
}

/// How an arrival into a shaped place was resolved — reported for tests and tools; the facts presence
/// records are what the world holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    /// The way it went.
    pub route: Route,
    /// Generations of nudges in the result that stood (0 when nobody was moved).
    pub generations: usize,
    /// People moved by the result that stood.
    pub nudged: usize,
    /// Whether the nudge pass failed, so that the walker was blocked at contact.
    pub nudge_failed: bool,
    /// What verification did.
    pub degraded: Degraded,
    /// Whether the head-on bias turned the stride.
    pub biased: bool,
    /// How the objects were treated.
    pub objects: Objects,
}

impl Outcome {
    /// A route that moved nobody, pushed nothing and degraded nothing.
    pub(crate) const fn plain(route: Route) -> Self {
        Self {
            route,
            generations: 0,
            nudged: 0,
            nudge_failed: false,
            degraded: Degraded::No,
            biased: false,
            objects: Objects::Untouched,
        }
    }
}

/// A resolution in this pack's terms: points, not locations.
pub(crate) struct Answer {
    pub(crate) reached: Point,
    pub(crate) displaced: Vec<(EntityId, Point)>,
    pub(crate) stopped_by: Option<EntityId>,
    pub(crate) outcome: Outcome,
}

impl Answer {
    /// Exactly `reached`, nobody moved, nothing pushed.
    pub(crate) const fn plain(reached: Point, route: Route) -> Self {
        Self {
            reached,
            displaced: Vec::new(),
            stopped_by: None,
            outcome: Outcome::plain(route),
        }
    }
}

/// One place as a resolution reads it: its room, the people standing in it (`EntityId` order) and
/// the loose objects lying in it (`ItemId` order).
pub(crate) struct Here {
    pub(crate) room: Room,
    pub(crate) standing: Vec<(EntityId, Point)>,
    pub(crate) objects: Vec<(ItemId, Placed)>,
}

impl ArrivalResolver for BodiesSystem {
    fn resolver_of(&self) -> SystemId {
        Self::ID
    }

    fn resolve(
        &self,
        world: &WorldRead<'_>,
        arriving: &Arriving,
        so_far: Resolution,
    ) -> Resolution {
        if so_far != Resolution::unchanged(arriving) {
            // Another resolver already changed this arrival (step-11 SD-B15): composing two geometric
            // resolvers is out of MVP-0, so this one passes it through, and the next resolution's
            // guard catches any overlap that results.
            return so_far;
        }
        let Some(answer) = answer(
            world,
            arriving.person(),
            arriving.from(),
            arriving.to(),
            PRODUCTION,
        ) else {
            return so_far;
        };
        let to = arriving.to();
        let reached = relocated(to, answer.reached);
        let mut resolution = so_far;
        if reached != to {
            resolution = resolution.stopped_at(reached, answer.stopped_by);
        }
        for (other, at) in answer.displaced {
            let now = world
                .component::<Presence>(other)
                .map(Presence::location)
                .expect("a displaced person stands in the place");
            let person = PersonId::new(other, EntityType::Person).expect("a displaced person");
            resolution = resolution.displacing(person, relocated(now, at));
        }
        resolution
    }
}

/// What this pack would answer for `person` arriving at `to` in the world as it stands, and how —
/// [`None`] where the arrival is not this pack's to resolve (`to` has no position, or its place no
/// shape). For tests and tools: the same computation the resolver runs, with the person's current
/// position read from presence as presence reads it.
pub fn explain(world: &WorldRead<'_>, person: PersonId, to: Location) -> Option<Outcome> {
    let from = world
        .component::<Presence>(person.entity_id())
        .map(Presence::location);
    answer(world, person, from, to, PRODUCTION).map(|answer| answer.outcome)
}

/// The resolution of one arrival, or [`None`] when it is inert (step-11 SD-B2).
fn answer(
    world: &WorldRead<'_>,
    person: PersonId,
    from: Option<Location>,
    to: Location,
    policy: Policy,
) -> Option<Answer> {
    let local = to.local()?;
    let place = to.place();
    let shape = world.component::<PlaceShape>(place.entity_id())?;
    let here = Here {
        room: shape.room(),
        standing: standing_in(world, place),
        objects: lying_in(world, place),
    };
    guard(world, place, &here);
    let target = Point::new(local.x().value(), local.y().value());
    let start = from
        .filter(|from| from.place() == place)
        .and_then(|from| from.local())
        .map(|local| Point::new(local.x().value(), local.y().value()));
    Some(match start {
        Some(start) => stride(&here, person.entity_id(), start, target, policy),
        None => entry(&here, target),
    })
}

/// The guard on the place's current state (step-11 SD-B9, SD-O9 step 1; `ARC-39` notes): every
/// resolved arrival keeps the invariant and genesis checked it, so a place that breaks it holds an
/// arrival that escaped resolution. A resolver has no error path; like `require_registered`, it
/// panics, naming what it found.
fn guard(world: &WorldRead<'_>, place: PlaceId, here: &Here) {
    let escaped = "an arrival escaped resolution: a stating system bypassed presence's \
                   arrivals(), or a host never registered the build's resolvers (DECISIONS.md ARC-39)";
    let (room, standing) = (&here.room, &here.standing);
    let points: Vec<Point> = standing.iter().map(|(_, at)| *at).collect();
    let clearance = i64::from(CLEARANCE.value());
    if let Some((a, b, distance2)) = closest_pair(&points)
        && distance2 < clearance * clearance
    {
        panic!(
            "bodies: {} and {} stand {} mm apart in {}, closer than {} mm, before an arrival into it \
             was resolved — {escaped}",
            name(world, standing[a].0),
            name(world, standing[b].0),
            distance2.isqrt(),
            name(world, place.entity_id()),
            CLEARANCE.value(),
        );
    }
    let margin = PERSON_RADIUS.value() - TOLERANCE.value();
    if let Some((person, at)) = standing.iter().find(|(_, at)| !room.admits(*at, margin)) {
        panic!(
            "bodies: {} stands at ({}, {}) in {}, outside its floor or inside a solid, before an \
             arrival into it was resolved — {escaped}",
            name(world, *person),
            at.x,
            at.y,
            name(world, place.entity_id()),
        );
    }
    let footprints: Vec<Footprint> = here
        .objects
        .iter()
        .map(|(_, placed)| placed.footprint())
        .collect();
    for (index, (object, placed)) in here.objects.iter().enumerate() {
        let others: Vec<Footprint> = footprints
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .map(|(_, footprint)| *footprint)
            .collect();
        if let Some(found) = flaw(room, placed, &others, &points, TOLERANCE.value()) {
            panic!(
                "bodies: {} at ({}, {}) in {} breaks the invariant of a lying object ({found:?}) \
                 before an arrival into it was resolved — {escaped}",
                name(world, object.entity_id()),
                placed.centre.x,
                placed.centre.y,
                name(world, place.entity_id()),
            );
        }
    }
}

/// `location` with its ground position moved to `at`, keeping its place, its height and its facing.
fn relocated(location: Location, at: Point) -> Location {
    let z = location.local().map_or(Millimetres::ZERO, LocalPosition::z);
    let moved = Location::in_place(location.place()).with_local(LocalPosition::new(
        Millimetres::new(at.x),
        Millimetres::new(at.y),
        z,
    ));
    match location.facing() {
        Some(facing) => moved.with_facing(facing),
        None => moved,
    }
}

#[cfg(test)]
mod tests {
    //! PB-8: verify-then-degrade reproduces the prototype's F-P6 and closes it (step-11 §9.8, DC-8).
    //!
    //! The setup is mode R's request 551: person 8 at (2 536, 782) asks for (+1 412, −1 412), toward
    //! the south wall; person 2 stands at (2 970, 314), against that wall. In the prototype the
    //! character controller slid the walker along the wall onto person 2, leaving them 33 mm apart.

    use mineworld_contracts::{
        EntityKey, LocalPosition, Location, Millimetres, PersonId, WorldTime,
    };
    use mineworld_kernel::World;
    use mineworld_presence::{ArrivalResolver, PresenceSystem, arrival, register_resolvers};

    use super::*;
    use crate::event::place_shaped;
    use crate::geometry::distance2;

    const NOW: WorldTime = WorldTime::from_seconds(3_600);

    fn at(place: PlaceId, x: i32, y: i32) -> Location {
        Location::in_place(place).with_local(LocalPosition::on_ground(
            Millimetres::new(x),
            Millimetres::new(y),
        ))
    }

    /// The café with the walker and the person against the wall; the walker, the person and the place.
    fn request_551() -> (World, PersonId, PersonId, PlaceId) {
        register_resolvers(vec![Box::new(BodiesSystem) as Box<dyn ArrivalResolver>]);
        mineworld_movement::register_wayfinders(vec![
            Box::new(BodiesSystem) as Box<dyn mineworld_movement::Wayfinder>
        ]);
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence installs");
        world.install(BodiesSystem).expect("bodies installs");
        let place = world
            .create_entity(EntityKey::new("cafe").expect("a key"), EntityType::Place)
            .expect("created");
        let place = PlaceId::new(place, EntityType::Place).expect("a place");
        let person = |world: &mut World, key: &str| {
            let id = world
                .create_entity(EntityKey::new(key).expect("a key"), EntityType::Person)
                .expect("created");
            PersonId::new(id, EntityType::Person).expect("a person")
        };
        let walker = person(&mut world, "walker");
        let against = person(&mut world, "against");
        let shape: PlaceShape = serde_json::from_value(serde_json::json!({
            "floor": { "min": { "x": 0, "y": 0 }, "max": { "x": 8320, "y": 10320 } },
            "solids": [
                { "min": { "x": 3860, "y": 6570 }, "max": { "x": 8320, "y": 7170 }, "height": 1100 }
            ]
        }))
        .expect("the café");
        let facts = vec![
            arrival(&world.read(), walker, at(place, 2_536, 782)).expect("placed"),
            arrival(&world.read(), against, at(place, 2_970, 314)).expect("placed"),
            place_shaped(place, shape),
        ];
        world.genesis(NOW, facts).expect("the café begins");
        (world, walker, against, place)
    }

    /// The walker's end and the other person's, under `policy`, and how far apart they are.
    fn resolved(policy: Policy) -> (Point, Point, i64, Outcome) {
        let (world, walker, against, place) = request_551();
        let read = world.read();
        let from = read
            .component::<Presence>(walker.entity_id())
            .map(Presence::location);
        let answer = answer(
            &read,
            walker,
            from,
            at(place, 2_536 + 1_412, 782 - 1_412),
            policy,
        )
        .expect("not inert");
        let other = answer
            .displaced
            .iter()
            .find(|(person, _)| *person == against.entity_id())
            .map_or(Point::new(2_970, 314), |(_, at)| *at);
        let apart = distance2(answer.reached, other);
        (answer.reached, other, apart, answer.outcome)
    }

    #[test]
    fn without_verification_the_controller_leaves_the_pair_overlapping() {
        let (walker, other, apart, outcome) = resolved(Policy {
            bias: false,
            verify: false,
            ..PRODUCTION
        });
        println!(
            "verification off: walker {walker:?}, against {other:?}, {} mm apart; {outcome:?}",
            apart.isqrt()
        );
        assert!(
            apart < 595 * 595,
            "the instrument sees F-P6: the walker and the person against the wall overlap, {} mm",
            apart.isqrt()
        );
    }

    /// The production verification, with the bias off so that the geometry is the prototype's.
    #[test]
    fn verify_then_degrade_keeps_the_pair_apart() {
        let (walker, other, apart, outcome) = resolved(Policy {
            bias: false,
            ..PRODUCTION
        });
        println!(
            "verification on: walker {walker:?}, against {other:?}, {} mm apart; {outcome:?}",
            apart.isqrt()
        );
        assert!(
            apart >= 595 * 595,
            "walker {walker:?} and the person against the wall {other:?} are {} mm apart",
            apart.isqrt()
        );
        assert_ne!(outcome.degraded, Degraded::No, "verification degraded it");
    }

    /// And the production policy, bias on, from the same start.
    #[test]
    fn the_production_policy_keeps_the_pair_apart_from_the_same_start() {
        let (walker, other, apart, outcome) = resolved(PRODUCTION);
        println!(
            "production: walker {walker:?}, against {other:?}, {} mm apart; {outcome:?}",
            apart.isqrt()
        );
        assert!(apart >= 595 * 595, "{} mm apart", apart.isqrt());
    }
}
