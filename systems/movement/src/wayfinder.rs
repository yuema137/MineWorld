//! The catalog through which whoever owns a place's geometry plans the route a walk follows through
//! it (`DECISIONS.md` `ARC-75`, `ARC-62`; step-11 SD-N3).
//!
//! A walk is this pack's state, and each of its strides is checked by this pack's own rule. What this
//! pack does not know is what stands in a place — walls, furniture, objects — because that is another
//! pack's state, and this pack never names another pack. So it asks:
//!
//! ```text
//! RouteAsk      one leg as this pack plans it: who, in which place, from where, to where, and the
//!               person who just stopped them, if any
//! RouteAnswer   the waypoints to follow — non-empty, integer, ending at the (possibly adjusted)
//!               goal — or `unreachable`
//! the catalog   the build's wayfinders, registered once per process, asked in SystemId order;
//!               the first answer is the route, and with none the leg is the straight segment
//! ```
//!
//! # Why a catalog, and why it is not world state
//!
//! For the reason presence's arrival resolvers are one (`ARC-39` item 5): a route is planned inside a
//! dispatch, where only a [`WorldRead`] is in reach, so the wayfinders are registered once per process
//! from the build's installed set, by the one assembly path every host uses (`worldpack::compose`,
//! through `Capability::register_extensions`). It is the build's compiled-in list of planner code,
//! identical for every world in the process and set before any world runs. Exactly three functions
//! touch it: [`register_wayfinders`], [`registered_wayfinders`] and [`require_wayfinder`].
//!
//! # Never registered means straight lines
//!
//! A process that composes worlds by hand and never registers plans every leg as the straight segment
//! from where the walker is to where they are going — which is what a world without a geometry pack
//! walks anyway. A wayfinder's pack refuses to join a world in such a process ([`require_wayfinder`]),
//! so a wayfinder is never silently skipped.

use std::sync::OnceLock;

use mineworld_contracts::{LocalPosition, PersonId, PlaceId, SystemId};
use mineworld_kernel::WorldRead;

/// One leg of a walk as this pack plans it: who walks, in which place, from where, to where, and whom
/// to plan round.
///
/// Built only by this pack, from its own state and presence's: a wayfinder is told where the walker is,
/// never asked to trust a caller about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteAsk {
    person: PersonId,
    place: PlaceId,
    from: LocalPosition,
    to: LocalPosition,
    avoid: Option<PersonId>,
}

impl RouteAsk {
    pub(crate) const fn new(
        person: PersonId,
        place: PlaceId,
        from: LocalPosition,
        to: LocalPosition,
        avoid: Option<PersonId>,
    ) -> Self {
        Self {
            person,
            place,
            from,
            to,
            avoid,
        }
    }

    /// Who is walking.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The place the leg lies in.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Where the walker is, in the place's frame.
    pub const fn from(&self) -> LocalPosition {
        self.from
    }

    /// Where the leg should end, in the place's frame.
    pub const fn to(&self) -> LocalPosition {
        self.to
    }

    /// The person who stopped the walker's last stride, to be planned round for this leg only.
    pub const fn avoid(&self) -> Option<PersonId> {
        self.avoid
    }
}

/// The positions a leg passes through, in order: never empty, and ending at the leg's goal — which a
/// wayfinder may have moved to the nearest point a person can stand on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waypoints(Vec<LocalPosition>);

impl Waypoints {
    /// The waypoints, or [`None`] for an empty list: a route goes somewhere.
    pub fn new(points: Vec<LocalPosition>) -> Option<Self> {
        (!points.is_empty()).then_some(Self(points))
    }

    /// The waypoints, in order.
    pub fn points(&self) -> &[LocalPosition] {
        &self.0
    }

    /// Where the leg ends.
    pub fn goal(&self) -> LocalPosition {
        *self.0.last().expect("a route is never empty")
    }

    pub(crate) fn into_points(self) -> Vec<LocalPosition> {
        self.0
    }
}

/// A wayfinder's answer for a place it plans in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteAnswer {
    /// Follow these.
    Waypoints(Waypoints),
    /// No way from `from` to anywhere near `to` in this place: the walk is refused or ends `no-route`.
    Unreachable,
}

/// A pack's answer to "how does this person get from here to there in this place" (`ARC-75`).
///
/// Asked by this pack while it plans a leg, before anything is recorded. Handed a [`WorldRead`] and
/// nothing else, like a perception provider: it can consult any state and write none. The
/// contract an implementation keeps (`ARC-62` item 4):
///
/// - **pure:** its answer is a function of its arguments alone — no state kept between calls, no
///   cache, no clock, no random source, no process-wide value. A replayed world asks again and must be
///   told the same thing (`ARC-25`);
/// - **inert where its state is absent:** it answers [`None`] for a place about which its pack holds
///   nothing, which is what keeps a world that does not enable its pack walking straight;
/// - **a plan, never a permission:** every stride of the walk is still a request this pack checks and
///   presence's resolvers resolve;
/// - **its pack refuses to join an unregistered world:** the pack's `System::install` calls
///   [`require_wayfinder`] with its own id.
pub trait Wayfinder: Send + Sync {
    /// The system this wayfinder belongs to. Wayfinders are asked in ascending order of this id.
    fn wayfinder_of(&self) -> SystemId;

    /// The route for `ask`, or [`None`] when this wayfinder knows nothing about its place.
    fn route(&self, world: &WorldRead<'_>, ask: &RouteAsk) -> Option<RouteAnswer>;
}

/// The build's wayfinders, sorted by id: written once per process, by [`register_wayfinders`].
static CATALOG: OnceLock<Vec<Box<dyn Wayfinder>>> = OnceLock::new();

/// The ids of a list of wayfinders, in its order.
fn ids(wayfinders: &[Box<dyn Wayfinder>]) -> Vec<SystemId> {
    wayfinders
        .iter()
        .map(|wayfinder| wayfinder.wayfinder_of())
        .collect()
}

/// Registers the build's wayfinders: once per process, before any world is assembled.
///
/// The list is sorted by [`Wayfinder::wayfinder_of`], so the order wayfinders are asked in is a
/// function of their names. Registering the same ids again does nothing, which is what lets every
/// composition register.
///
/// # Panics
///
/// When the list names one id twice, naming it; and when a different list was registered before,
/// naming both (`ARC-62` item 4).
pub fn register_wayfinders(mut wayfinders: Vec<Box<dyn Wayfinder>>) {
    wayfinders.sort_by_key(|wayfinder| wayfinder.wayfinder_of());
    let offered = ids(&wayfinders);
    if let Some(pair) = offered.windows(2).find(|pair| pair[0] == pair[1]) {
        panic!(
            "register_wayfinders was given two wayfinders for '{}': a build has one wayfinder per \
             system (DECISIONS.md ARC-75)",
            pair[0]
        );
    }
    let mut pending = Some(wayfinders);
    let registered = CATALOG.get_or_init(|| pending.take().unwrap_or_default());
    if pending.is_none() {
        return;
    }
    let current = ids(registered);
    assert!(
        current == offered,
        "register_wayfinders was called with {offered:?}, but this process already registered \
         {current:?}: one build has one catalog of wayfinders (DECISIONS.md ARC-75)"
    );
}

/// The ids of the registered wayfinders, in the order they are asked; [`None`] until something
/// registers.
pub fn registered_wayfinders() -> Option<Vec<SystemId>> {
    CATALOG.get().map(|wayfinders| ids(wayfinders))
}

/// Panics unless `wayfinder` is a registered wayfinder: what a wayfinder's pack calls in its
/// `System::install`, so that it refuses to join a world whose host never registered it.
///
/// # Panics
///
/// When nothing was registered, or the registered list does not name `wayfinder`. The message names
/// the pack, [`register_wayfinders`] and `ARC-75`.
pub fn require_wayfinder(wayfinder: &SystemId) {
    match CATALOG.get() {
        None => panic!(
            "the '{wayfinder}' system plans routes, but this process never registered the build's \
             wayfinders: a host calls mineworld_movement::register_wayfinders before it installs a \
             wayfinder's pack, as worldpack::compose does (DECISIONS.md ARC-75)"
        ),
        Some(wayfinders) => assert!(
            wayfinders
                .iter()
                .any(|registered| registered.wayfinder_of() == *wayfinder),
            "the '{wayfinder}' system plans routes, but the wayfinders registered with \
             register_wayfinders are {:?}: list it on movement's extension line in the installed \
             set (DECISIONS.md ARC-75)",
            ids(wayfinders)
        ),
    }
}

/// The first registered wayfinder's answer for `ask`, or [`None`] when none answers — the leg is
/// then the straight segment.
pub(crate) fn route(world: &WorldRead<'_>, ask: &RouteAsk) -> Option<RouteAnswer> {
    CATALOG
        .get()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .find_map(|wayfinder| wayfinder.route(world, ask))
}
