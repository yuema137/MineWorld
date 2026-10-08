//! This pack's answer to presence's question: what an arrival into a shaped place actually achieves
//! (`DECISIONS.md` `ARC-39` and its note; step-11 §4.5, SD-B6 … SD-B9, SD-B15).
//!
//! ```text
//! inert       `to` has no position, or its place has no shape          → so_far, untouched
//! earlier     another resolver already changed the arrival              → so_far, untouched
//! guard       the place as it stands breaks the invariant               → panic, naming the pair
//! stride      within the place, from a position:
//!               clear corridor (integers)                               → exactly `to`
//!               walls-only reach W, contact reach B (two sweeps)
//!               candidate: W, or W cut NUDGE_MAX beyond contact
//!               nudge pass from the candidate; on failure, blocked at B
//!               verify on integers; degrade: blocked → halved ×8 → stay
//! entry       from another place, from nowhere, or from no position:
//!               `to` if a person fits there
//!               else nudge from `to`, if only people are in the way and the result verifies
//!               else the nearest free point of the 50 mm lattice
//! ```
//!
//! Every position is whole millimetres; the two sweeps and the nudges are Rapier's (`rapier.rs`),
//! everything else is integer geometry. The resolver keeps nothing: no static, no cache, no clock —
//! a replayed world asks it again and is told the same thing (`ARC-25`).

use mineworld_contracts::SystemId;
use mineworld_contracts::{
    EntityId, EntityType, LocalPosition, Location, Millimetres, PersonId, PlaceId,
};
use mineworld_kernel::{SystemIdentity, WorldRead};
use mineworld_presence::{ArrivalResolver, Arriving, Presence, Resolution};

use crate::component::PlaceShape;
use crate::geometry::{
    CHAIN_MAX, CLEARANCE, GAP, HALVINGS, NUDGE_MAX, NUDGED_MAX, PERSON_RADIUS, Point, Room, SNAP,
    TOLERANCE, at_least, closest_pair, distance2, first_met, head_on, no_longer_than, scaled_down,
    turned_right,
};
use crate::rapier::{Against, Scene};
use crate::system::{BodiesSystem, name, standing_in};

/// Which steps a resolution runs. Production runs every step; a unit test may turn one off to show
/// what it is for (step-11 PB-8).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Policy {
    /// The head-on bias (step-11 QB-16, SD-B10).
    pub(crate) bias: bool,
    /// Verify, then degrade (step-11 DC-8, I-12).
    pub(crate) verify: bool,
}

/// What every world runs.
pub(crate) const PRODUCTION: Policy = Policy {
    bias: true,
    verify: true,
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
}

/// A resolution in this pack's terms: points, not locations.
struct Answer {
    reached: Point,
    displaced: Vec<(EntityId, Point)>,
    stopped_by: Option<EntityId>,
    outcome: Outcome,
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
    let room = shape.room();
    let standing = standing_in(world, place);
    guard(world, place, &room, &standing);
    let target = Point::new(local.x().value(), local.y().value());
    let start = from
        .filter(|from| from.place() == place)
        .and_then(|from| from.local())
        .map(|local| Point::new(local.x().value(), local.y().value()));
    Some(match start {
        Some(start) => stride(&room, &standing, person.entity_id(), start, target, policy),
        None => entry(&room, &standing, target),
    })
}

/// The guard on the place's current state (step-11 SD-B9; `ARC-39` note, point 2): every resolved
/// arrival keeps the invariant and genesis checked it, so a place that breaks it holds an arrival that
/// escaped resolution. A resolver has no error path; like `require_registered`, it panics, naming
/// what it found.
fn guard(world: &WorldRead<'_>, place: PlaceId, room: &Room, standing: &[(EntityId, Point)]) {
    let points: Vec<Point> = standing.iter().map(|(_, at)| *at).collect();
    let clearance = i64::from(CLEARANCE.value());
    if let Some((a, b, distance2)) = closest_pair(&points)
        && distance2 < clearance * clearance
    {
        panic!(
            "bodies: {} and {} stand {} mm apart in {}, closer than {} mm, before an arrival into it \
             was resolved — an arrival escaped resolution: a stating system bypassed presence's \
             arrivals(), or a host never registered the build's resolvers (DECISIONS.md ARC-39)",
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
             arrival into it was resolved — an arrival escaped resolution: a stating system \
             bypassed presence's arrivals(), or a host never registered the build's resolvers \
             (DECISIONS.md ARC-39)",
            name(world, *person),
            at.x,
            at.y,
            name(world, place.entity_id()),
        );
    }
}

/// Everybody's position in one candidate result: the walker's and each displaced person's.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    walker: Point,
    /// Indexes into the others, with their new positions, in the order they were moved.
    displaced: Vec<(usize, Point)>,
}

/// A stride within the place (step-11 SD-B6; the prototype's mode R′, §9.7).
fn stride(
    room: &Room,
    standing: &[(EntityId, Point)],
    walker: EntityId,
    start: Point,
    target: Point,
    policy: Policy,
) -> Answer {
    let me = standing
        .iter()
        .position(|(person, _)| *person == walker)
        .expect("a walker with a position in the place stands in it");
    let others: Vec<(EntityId, Point)> = standing
        .iter()
        .copied()
        .filter(|(person, _)| *person != walker)
        .collect();
    let other_points: Vec<Point> = others.iter().map(|(_, at)| *at).collect();
    if room.corridor_clear(start, target, &other_points) {
        return Answer {
            reached: target,
            displaced: Vec::new(),
            stopped_by: None,
            outcome: Outcome::plain(Route::Clear),
        };
    }

    // The head-on bias (step-11 SD-B10): a walker whose first person met stands within the band of
    // their line aims to their right instead — "keep right" — so that two walkers meeting head-on
    // pass rather than pushing each other straight back. Only the walker's own aim turns; nudges stay
    // straight away from their pusher.
    let asked = target.minus(start);
    let met = first_met(start, asked, &other_points)
        .filter(|(_, cross2)| policy.bias && head_on(asked, *cross2))
        .map(|(index, _)| others[index].0);
    let aim = match met {
        Some(_) => start.plus(turned_right(asked)),
        None => target,
    };

    let points: Vec<Point> = standing.iter().map(|(_, at)| *at).collect();
    let scene = Scene::build(room, &points);
    let reach = reach(&scene, me, start, aim);

    // The scene index of each other person: they keep their place in `standing`.
    let in_scene: Vec<usize> = (0..standing.len()).filter(|index| *index != me).collect();
    let desired = aim.minus(start);
    let fallback = if desired == Point::new(0, 0) {
        Point::new(1, 0)
    } else {
        desired
    };
    let pass = nudge(&scene, &others, &in_scene, reach.candidate, fallback);
    let nudge_failed = pass.is_err();
    let (primary, generations) = match pass {
        Ok(nudged) => (
            Candidate {
                walker: reach.candidate,
                displaced: nudged.moved,
            },
            nudged.generations,
        ),
        Err(()) => (
            Candidate {
                walker: reach.blocked,
                displaced: Vec::new(),
            },
            0,
        ),
    };

    let (result, degraded) = if !policy.verify || verifies(room, &others, &primary) {
        (primary, Degraded::No)
    } else {
        degrade(room, &others, start, reach.blocked)
    };
    let generations = if result.displaced.is_empty() {
        0
    } else {
        generations
    };
    let stopped_by = if result.walker == target {
        None
    } else if result.walker == aim {
        // The bias turned a walker who then reached the turned aim: what turned them is the person
        // they met head-on.
        met
    } else if degraded == Degraded::No && !nudge_failed && reach.walls_stopped {
        None
    } else {
        reach.touched.map(|index| standing[index].0)
    };
    Answer {
        reached: result.walker,
        displaced: result
            .displaced
            .iter()
            .map(|(index, at)| (others[*index].0, *at))
            .collect(),
        stopped_by,
        outcome: Outcome {
            route: Route::Swept,
            generations,
            nudged: result.displaced.len(),
            nudge_failed,
            degraded,
            biased: met.is_some(),
        },
    }
}

impl Outcome {
    /// A route that moved nobody and degraded nothing.
    const fn plain(route: Route) -> Self {
        Self {
            route,
            generations: 0,
            nudged: 0,
            nudge_failed: false,
            degraded: Degraded::No,
            biased: false,
        }
    }
}

/// What the two sweeps of a stride say: where the walker would end before anybody is nudged, where it
/// stops if blocked, and who it touched.
struct Reach {
    /// R′ step 3's candidate: the walls-only reach W, or W cut `NUDGE_MAX` beyond contact.
    candidate: Point,
    /// Where a blocked walker stands: the contact reach B's first touch of a person.
    blocked: Point,
    /// Whether the candidate is W itself — the walls, not a person, ended the stride.
    walls_stopped: bool,
    /// The first person the contact sweep touched, by scene index.
    touched: Option<usize>,
}

/// R′ steps 1–3 (step-11 §4.5.1): the walls-only reach W and the contact reach B, each quantized,
/// snapped to `target` within [`SNAP`], and never longer than asked; then the candidate.
///
/// How far the walker gets with people solid is the contact sweep's end, sliding included — the
/// prototype's B, which the candidate rule measures. Where a blocked walker stops is where that sweep
/// first touched a person, not where the controller's slide along their curve carried it on to: a
/// blocked walker ends at contact (step-11 §17.11, DB-4).
fn reach(scene: &Scene, me: usize, start: Point, target: Point) -> Reach {
    let desired = target.minus(start);
    let settle = |end: Point| {
        let snapped = if distance2(end, target) <= i64::from(SNAP.value()).pow(2) {
            target
        } else {
            end
        };
        no_longer_than(start, snapped, target)
    };
    let walls = settle(scene.sweep(Some(me), start, desired, Against::Fixed).end);
    let contact = scene.sweep(Some(me), start, desired, Against::FixedAndPeople);
    let swept = settle(contact.end);
    let (reach_walls, reach_contact) = (walls.minus(start).length(), swept.minus(start).length());
    let beyond = reach_contact + i64::from(NUDGE_MAX.value());
    let walls_stopped = reach_walls <= beyond;
    Reach {
        candidate: if walls_stopped {
            walls
        } else {
            start.plus(scaled_down(walls.minus(start), beyond, reach_walls))
        },
        blocked: contact.contact.map_or(swept, settle),
        walls_stopped,
        touched: contact.touched,
    }
}

/// Verify-then-degrade's fallbacks, after the resolved result failed verification (step-11 DC-8):
/// blocked at contact; then the advance along the blocked path halved, up to [`HALVINGS`] times;
/// then stay. Staying is valid by induction: the place as it stood passed the guard.
fn degrade(
    room: &Room,
    others: &[(EntityId, Point)],
    start: Point,
    blocked: Point,
) -> (Candidate, Degraded) {
    let only = |walker: Point| Candidate {
        walker,
        displaced: Vec::new(),
    };
    if verifies(room, others, &only(blocked)) {
        return (only(blocked), Degraded::Blocked);
    }
    let advance = blocked.minus(start);
    for k in 1..=HALVINGS {
        let walker = start.plus(Point::new(advance.x / (1 << k), advance.y / (1 << k)));
        if verifies(room, others, &only(walker)) {
            return (only(walker), Degraded::Halved(k));
        }
    }
    (only(start), Degraded::Stayed)
}

/// V1–V3 on integers (step-11 SD-B6 step 6): every pair in the place at least [`CLEARANCE`] apart,
/// and every moved centre inside the floor and out of every solid, to within [`TOLERANCE`].
fn verifies(room: &Room, others: &[(EntityId, Point)], candidate: &Candidate) -> bool {
    let mut points: Vec<Point> = others.iter().map(|(_, at)| *at).collect();
    for (index, at) in &candidate.displaced {
        points[*index] = *at;
    }
    points.push(candidate.walker);
    let clearance = i64::from(CLEARANCE.value());
    let apart =
        closest_pair(&points).is_none_or(|(_, _, distance2)| distance2 >= clearance * clearance);
    let margin = PERSON_RADIUS.value() - TOLERANCE.value();
    apart
        && room.admits(candidate.walker, margin)
        && candidate
            .displaced
            .iter()
            .all(|(_, at)| room.admits(*at, margin))
}

/// The people a nudge pass moved, in the order it moved them, and how many generations it took.
struct Nudged {
    moved: Vec<(usize, Point)>,
    generations: usize,
}

/// The nudge pass (step-11 SD-B7; QB-10, I-11). Generation 1's pusher is the arriving person at
/// `pusher`; generation g's pushers are the people moved in generation g − 1. In each generation, for
/// each pusher, every person not yet moved, in `EntityId` order, who is closer than two radii to it is
/// swept straight away from its centre — against the fixed geometry only — just far enough to stand
/// two radii and a gap from it. When two centres coincide, `fallback` gives the direction.
///
/// The pass fails if a nudge would exceed `NUDGE_MAX + GAP`, if the walls cut one more than [`SNAP`]
/// short, if more than [`NUDGED_MAX`] people would move, or if any pair is still closer than
/// [`CLEARANCE`] after [`CHAIN_MAX`] generations.
fn nudge(
    scene: &Scene,
    others: &[(EntityId, Point)],
    in_scene: &[usize],
    pusher: Point,
    fallback: Point,
) -> Result<Nudged, ()> {
    let touching = i64::from(2 * PERSON_RADIUS.value());
    let spacing = i64::from(2 * PERSON_RADIUS.value() + GAP.value());
    let longest = i64::from(NUDGE_MAX.value() + GAP.value());
    let mut now: Vec<Point> = others.iter().map(|(_, at)| *at).collect();
    let mut moved: Vec<(usize, Point)> = Vec::new();
    let mut frontier = vec![pusher];
    let mut generations = 0;
    for _ in 0..CHAIN_MAX {
        let mut next = Vec::new();
        for pushing in &frontier {
            for index in 0..others.len() {
                if moved.iter().any(|(done, _)| *done == index) {
                    continue;
                }
                let at = now[index];
                let between = distance2(at, *pushing);
                if between >= touching * touching {
                    continue;
                }
                let needed = spacing - between.isqrt();
                if needed > longest {
                    return Err(());
                }
                let away = if between == 0 {
                    fallback
                } else {
                    at.minus(*pushing)
                };
                let by = at_least(away, needed);
                if distance2(by, Point::new(0, 0)) > longest * longest {
                    return Err(());
                }
                let swept = scene
                    .sweep(Some(in_scene[index]), at, by, Against::Fixed)
                    .end;
                let end = if distance2(swept, at.plus(by)) <= i64::from(SNAP.value()).pow(2) {
                    at.plus(by)
                } else {
                    swept
                };
                let travelled = end.minus(at).length();
                if travelled + i64::from(SNAP.value()) < by.length()
                    || distance2(end, at) > longest * longest
                {
                    return Err(());
                }
                now[index] = end;
                moved.push((index, end));
                next.push(end);
                if moved.len() > NUDGED_MAX {
                    return Err(());
                }
            }
        }
        if next.is_empty() {
            break;
        }
        generations += 1;
        frontier = next;
    }
    let mut everyone = now;
    everyone.push(pusher);
    let clearance = i64::from(CLEARANCE.value());
    if closest_pair(&everyone).is_some_and(|(_, _, distance2)| distance2 < clearance * clearance) {
        return Err(());
    }
    Ok(Nudged { moved, generations })
}

/// An arrival from another place, from nowhere, or from a position-less presence in this place
/// (step-11 SD-B8, QP-7). It always ends in this place: a resolver can neither refuse an arrival nor
/// end it elsewhere (rule (a)), and the capacity checked at genesis guarantees a free point.
fn entry(room: &Room, standing: &[(EntityId, Point)], target: Point) -> Answer {
    let points: Vec<Point> = standing.iter().map(|(_, at)| *at).collect();
    let outcome = |route, generations, nudged| Outcome {
        generations,
        nudged,
        ..Outcome::plain(route)
    };
    // E1: a person fits at `to`.
    if room.free_at(target, &points) {
        return Answer {
            reached: target,
            displaced: Vec::new(),
            stopped_by: None,
            outcome: outcome(Route::Entered, 0, 0),
        };
    }
    // E2: only people are in the way; nudge them from `to`, under the same bounds.
    if room.admits(target, PERSON_RADIUS.value()) {
        let scene = Scene::build(room, &points);
        let in_scene: Vec<usize> = (0..standing.len()).collect();
        if let Ok(nudged) = nudge(&scene, standing, &in_scene, target, Point::new(1, 0)) {
            let candidate = Candidate {
                walker: target,
                displaced: nudged.moved,
            };
            if verifies(room, standing, &candidate) {
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
    let reached = room.nearest_free(target, &points).unwrap_or_else(|| {
        panic!(
            "bodies: no free point for an arrival at ({}, {}): the place's capacity, checked at \
             genesis, is exhausted — which only a world grown after genesis can do (DECISIONS.md \
             ARC-39 note, point 3)",
            target.x, target.y
        )
    });
    let spacing = i64::from(2 * PERSON_RADIUS.value() + GAP.value());
    let stopped_by = standing
        .iter()
        .find(|(_, at)| distance2(*at, target) < spacing * spacing)
        .map(|(person, _)| *person);
    Answer {
        reached,
        displaced: Vec::new(),
        stopped_by,
        outcome: outcome(Route::Placed, 0, 0),
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
