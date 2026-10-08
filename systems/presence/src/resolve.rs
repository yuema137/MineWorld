//! The seam that lets another pack say what an arrival actually achieves, before this pack records it
//! (`DECISIONS.md` `ARC-39`).
//!
//! A stating system decides that a person goes somewhere; this pack decides what its state may hold.
//! Between the two sits one more question neither can answer alone: does the person actually get
//! there, and does anybody else have to make room? A pack that knows — because it holds state about
//! the people and the place this pack does not — answers it through [`ArrivalResolver`]. This pack
//! asks every registered resolver while it builds an arrival, checks each answer as the owner, and
//! records only the result:
//!
//! ```text
//! Arriving      one arrival as this pack knows it before recording: who, from where, to where
//! Resolution    what it achieves: where the person ends, who else is moved, what stopped them
//! the catalog   the build's resolvers, registered once per process, asked in SystemId order
//! ```
//!
//! # Why a catalog, and why it is not world state
//!
//! An arrival is built inside a dispatch, where the only things in reach are the stating system and a
//! [`WorldRead`] — no composition, no registry — so the resolvers cannot be handed to the constructor
//! per call. They are registered once per process instead, from the build's installed set, by the one
//! assembly path every host uses (`worldpack::compose`). The catalog is the build's compiled-in list
//! of resolver code, identical for every world in the process, set before any world runs and never
//! changed; it is the one process-wide value this pack holds, named here rather than hidden
//! (`ENGINEERING_RULES.md` §15). Exactly three functions touch it: [`register_resolvers`],
//! [`registered_resolvers`] and [`require_registered`].
//!
//! A world that does not enable a resolver's pack is unaffected by the resolver, because a resolver
//! must answer "unchanged" where its own state is absent — and in such a world it is absent.
//!
//! # Never registered means no resolver
//!
//! A process that composes worlds by hand and never registers runs with no resolver, and records
//! exactly what it recorded before this seam existed. A resolver's pack refuses to be installed in
//! such a process ([`require_registered`]), so a resolver is never silently skipped.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use mineworld_contracts::{
    EntityId, EntityType, LifecycleState, Location, PersonId, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::WorldRead;

use crate::component::Presence;
use crate::event::admit;

/// One arrival as presence knows it before recording anything: who, from where, and to where the
/// stating system decided they go.
///
/// Built only by this pack, from its own [`Presence`]: a resolver is told where the person is, never
/// asked to trust a caller about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arriving {
    person: PersonId,
    from: Option<Location>,
    to: Location,
}

impl Arriving {
    pub(crate) const fn new(person: PersonId, from: Option<Location>, to: Location) -> Self {
        Self { person, from, to }
    }

    /// Who is arriving.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where they are now, as this pack records it: [`None`] at a first placement.
    pub const fn from(&self) -> Option<Location> {
        self.from
    }

    /// Where the stating system decided they go.
    pub const fn to(&self) -> Location {
        self.to
    }
}

/// What an arrival achieves: where the person ends, who else is moved, and what stopped them short.
///
/// The fields are private. A resolver starts from the resolution it is handed and changes it only
/// through [`Resolution::stopped_at`] and [`Resolution::displacing`]; this pack checks every answer
/// before it builds a fact (`ARC-39` item 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    reached: Location,
    displaced: Vec<(PersonId, Location)>,
    stopped_by: Option<EntityId>,
}

impl Resolution {
    /// The arrival exactly as decided: what presence records when no resolver changes it.
    pub fn unchanged(arriving: &Arriving) -> Self {
        Self {
            reached: arriving.to,
            displaced: Vec::new(),
            stopped_by: None,
        }
    }

    /// The person ends at `reached` instead, stopped by `by` when the resolver can name what stopped
    /// them.
    #[must_use]
    pub fn stopped_at(mut self, reached: Location, by: Option<EntityId>) -> Self {
        self.reached = reached;
        self.stopped_by = by;
        self
    }

    /// Another person is moved to `to` by this arrival, recorded after the ones already listed.
    #[must_use]
    pub fn displacing(mut self, person: PersonId, to: Location) -> Self {
        self.displaced.push((person, to));
        self
    }

    /// Where the person ends.
    pub const fn reached(&self) -> Location {
        self.reached
    }

    /// Who else is moved, and where to, in the order they are recorded.
    pub fn displaced(&self) -> &[(PersonId, Location)] {
        &self.displaced
    }

    /// What the person stopped at, when a resolver could name it.
    pub const fn stopped_by(&self) -> Option<EntityId> {
        self.stopped_by
    }
}

/// A pack's answer to "what does this arrival actually achieve" (`DECISIONS.md` `ARC-39`).
///
/// Asked by presence while an arrival is being built, before anything is recorded. Handed a
/// [`WorldRead`] and nothing else, as [`PerceptionProvider`](crate::PerceptionProvider) and
/// [`System::validate`](mineworld_kernel::System::validate) are: it can consult any state and write
/// none. The contract an implementation keeps:
///
/// - **pure:** its answer is a function of its arguments alone — no state kept between calls, no
///   cache, no clock, no random source, no process-wide value. A replayed world asks it again and must
///   be told the same thing (`ARC-25`);
/// - **inert where its state is absent:** it returns `so_far` unchanged when the world holds none of
///   its own pack's state about the people and the place involved, which is what keeps a world that
///   does not enable its pack byte-identical;
/// - **it narrows, and presence checks:** it may end the arrival short or bend it within the place and
///   move other people within the place; presence refuses an answer that lengthens the arrival,
///   changes its place, turns the person, invents or drops a position, or moves anybody it may not,
///   naming the resolver ([`Rejection::System`], code `resolution-refused`);
/// - **its pack refuses to join an unregistered world:** the pack's `System::install` calls
///   [`require_registered`] with its own id before anything else.
///
/// A resolver cannot emit a fact. Its pack's own consequences of an arrival happen in that pack's
/// reactions to the facts presence records.
pub trait ArrivalResolver: Send + Sync {
    /// The system this resolver belongs to. Resolvers are asked in ascending order of this id.
    fn resolver_of(&self) -> SystemId;

    /// What the arrival achieves, given what the resolvers asked before this one decided.
    fn resolve(&self, world: &WorldRead<'_>, arriving: &Arriving, so_far: Resolution)
    -> Resolution;
}

/// The build's resolvers, sorted by id: written once per process, by [`register_resolvers`].
static CATALOG: OnceLock<Vec<Box<dyn ArrivalResolver>>> = OnceLock::new();

/// The ids of a list of resolvers, in its order.
fn ids(resolvers: &[Box<dyn ArrivalResolver>]) -> Vec<SystemId> {
    resolvers
        .iter()
        .map(|resolver| resolver.resolver_of())
        .collect()
}

/// Registers the build's resolvers: once per process, before any world is assembled.
///
/// The list is sorted by [`ArrivalResolver::resolver_of`], so the order resolvers are asked in is a
/// function of their names and not of the order a list was written in. Registering the same ids
/// again does nothing, which is what lets every composition register.
///
/// # Panics
///
/// When the list names one id twice, naming it; and when a different list was registered before,
/// naming both. One build has one catalog: a second, different one is a defect of the host, found
/// before any world runs, and not a condition to recover from (`ARC-39` item 5).
pub fn register_resolvers(mut resolvers: Vec<Box<dyn ArrivalResolver>>) {
    resolvers.sort_by_key(|resolver| resolver.resolver_of());
    let offered = ids(&resolvers);
    if let Some(pair) = offered.windows(2).find(|pair| pair[0] == pair[1]) {
        panic!(
            "register_resolvers was given two resolvers for '{}': a build has one resolver per \
             system (DECISIONS.md ARC-39)",
            pair[0]
        );
    }
    let mut pending = Some(resolvers);
    let registered = CATALOG.get_or_init(|| pending.take().unwrap_or_default());
    if pending.is_none() {
        return;
    }
    let current = ids(registered);
    assert!(
        current == offered,
        "register_resolvers was called with {offered:?}, but this process already registered \
         {current:?}: one build has one catalog of resolvers (DECISIONS.md ARC-39)"
    );
}

/// The ids of the registered resolvers, in the order they are asked; [`None`] until something
/// registers.
pub fn registered_resolvers() -> Option<Vec<SystemId>> {
    CATALOG.get().map(|resolvers| ids(resolvers))
}

/// Panics unless `resolver` is a registered resolver: what a resolver's pack calls first in its
/// `System::install`, so that it refuses to join a world whose host never registered it.
///
/// # Panics
///
/// When nothing was registered, or the registered list does not name `resolver`. The message names
/// the pack, [`register_resolvers`] and `ARC-39`.
pub fn require_registered(resolver: &SystemId) {
    match CATALOG.get() {
        None => panic!(
            "the '{resolver}' system resolves arrivals, but this process never registered the \
             build's resolvers: a host calls mineworld_presence::register_resolvers before it \
             installs a resolver's pack, as worldpack::compose does (DECISIONS.md ARC-39)"
        ),
        Some(resolvers) => assert!(
            resolvers
                .iter()
                .any(|registered| registered.resolver_of() == *resolver),
            "the '{resolver}' system resolves arrivals, but the resolvers registered with \
             register_resolvers are {:?}: list it on the installed set's resolution: line \
             (DECISIONS.md ARC-39)",
            ids(resolvers)
        ),
    }
}

/// The refusal code for a resolution presence will not record.
pub(crate) const RESOLUTION_REFUSED: RejectionCode =
    RejectionCode::from_static("resolution-refused");

/// Presence refusing a resolver's answer, naming the resolver and the rule.
pub(crate) fn refused(resolver: &SystemId, rule: &str) -> Rejection {
    Rejection::System {
        code: RESOLUTION_REFUSED,
        detail: Some(format!("{resolver}: {rule}")),
    }
}

/// An arrival after every registered resolver has answered and presence has checked each answer:
/// the [`Arriving`], the [`Resolution`], and the first resolver whose answer changed it, if any.
pub(crate) struct Resolved {
    pub(crate) arriving: Arriving,
    pub(crate) resolution: Resolution,
    pub(crate) changed_by: Option<SystemId>,
}

/// Asks [`admit`], builds the [`Arriving`] from presence's own record, and folds the registered
/// resolvers over it in ascending id, checking the answer after each one (`ARC-39` items 2–3).
pub(crate) fn resolve(
    world: &WorldRead<'_>,
    person: PersonId,
    to: Location,
) -> Result<Resolved, Rejection> {
    admit(world, person, to)?;
    let from = world
        .component::<Presence>(person.entity_id())
        .map(Presence::location);
    let arriving = Arriving::new(person, from, to);
    let unchanged = Resolution::unchanged(&arriving);
    let mut resolution = unchanged.clone();
    let mut changed_by = None;
    for resolver in CATALOG.get().map_or(&[][..], Vec::as_slice) {
        let id = resolver.resolver_of();
        resolution = resolver.resolve(world, &arriving, resolution);
        check(world, &arriving, &resolution).map_err(|rule| refused(&id, rule))?;
        if changed_by.is_none() && resolution != unchanged {
            changed_by = Some(id);
        }
    }
    Ok(Resolved {
        arriving,
        resolution,
        changed_by,
    })
}

/// What presence holds a resolution to before building any fact from it (`ARC-39` item 3): the rule
/// it breaks, or nothing. Integers only.
fn check(
    world: &WorldRead<'_>,
    arriving: &Arriving,
    resolution: &Resolution,
) -> Result<(), &'static str> {
    let to = arriving.to();
    let reached = resolution.reached();
    if reached.place() != to.place() {
        return Err("ended the arrival in another place");
    }
    if reached.facing() != to.facing() {
        return Err("turned the person arriving");
    }
    if reached.local().is_some() != to.local().is_some() {
        return Err("invented or dropped a local position");
    }
    if let Some(from) = arriving.from().filter(|from| from.place() == to.place())
        && let (Some(start), Some(asked), Some(ended)) = (from.local(), to.local(), reached.local())
        && squared(start, ended) > squared(start, asked)
    {
        return Err("lengthened the arrival");
    }
    if admit(world, arriving.person(), reached).is_err() {
        return Err("ended the arrival where presence refuses it");
    }
    check_displaced(world, arriving, resolution)?;
    match resolution.stopped_by() {
        Some(_) if reached == to => Err("named what stopped a person who was not stopped"),
        Some(by) if world.entity(by).is_none() => Err("named a stopper that is not in this world"),
        _ => Ok(()),
    }
}

/// Every displaced entry: a living person, not the one arriving, listed once, now in the place of the
/// arrival, moved within it without turning and without a position invented or dropped.
fn check_displaced(
    world: &WorldRead<'_>,
    arriving: &Arriving,
    resolution: &Resolution,
) -> Result<(), &'static str> {
    let place = arriving.to().place();
    let mut seen = BTreeSet::new();
    for (person, location) in resolution.displaced() {
        if *person == arriving.person() {
            return Err("displaced the person arriving");
        }
        if !seen.insert(person.entity_id()) {
            return Err("displaced one person twice");
        }
        let living_person = world.entity(person.entity_id()).is_some_and(|entity| {
            entity.entity_type() == EntityType::Person
                && entity.lifecycle() != LifecycleState::Destroyed
        });
        if !living_person {
            return Err("displaced something that is not a living person");
        }
        let Some(now) = world
            .component::<Presence>(person.entity_id())
            .map(Presence::location)
            .filter(|now| now.place() == place)
        else {
            return Err("displaced somebody who is not in the place");
        };
        if location.place() != place {
            return Err("displaced somebody into another place");
        }
        if location.facing() != now.facing() {
            return Err("turned a displaced person");
        }
        if location.local().is_some() != now.local().is_some() {
            return Err("invented or dropped a displaced person's local position");
        }
        if admit(world, *person, *location).is_err() {
            return Err("displaced somebody where presence refuses them");
        }
    }
    Ok(())
}

/// The squared distance between two positions, in square millimetres, without overflow.
fn squared(a: mineworld_contracts::LocalPosition, b: mineworld_contracts::LocalPosition) -> i128 {
    let axis = |p: i32, q: i32| {
        let d = i128::from(p) - i128::from(q);
        d * d
    };
    axis(a.x().value(), b.x().value())
        + axis(a.y().value(), b.y().value())
        + axis(a.z().value(), b.z().value())
}
