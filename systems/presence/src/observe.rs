//! Perception: turning a world into one observer's view of it.
//!
//! This is the `INV-13` boundary in executable form. A controller is never handed the world; it is
//! handed an [`Observation`], which is a value listing what was exposed — and this function is what
//! decides the list. Everything it does is therefore a judgement about *what one person may know*,
//! and the judgements are written down here rather than left implicit:
//!
//! ```text
//! entities     the place you are in, and everybody this pack knows to be in it
//! self         where you are, so a controller can decide to move before deciding to act
//! relations    the present-in edges this pack wrote, for the entities you perceive
//! affordances  what you may attempt, each with the server's verdict (ENGINEERING_RULES §8)
//! events       nothing — see below
//! ```
//!
//! # Why this is a function and not a method on the system
//!
//! Perception is a read. A [`System`](mineworld_kernel::System) is asked four things — install,
//! validate, resolve, react — and none of them is "produce an observation", because an observation
//! changes nothing and belongs to no instant of dispatch. It takes the whole
//! [`World`] rather than a [`WorldRead`] for one reason, and it is the load-bearing one: the
//! affordance answer depends on whether the world still *provides* an action, which is the system
//! registry's answer, not the state's.
//!
//! # What is deliberately not here
//!
//! **Events.** [`Observation::with_events`] exists and this function leaves it empty. Deciding which
//! recorded facts a person learned of means keeping a per-observer position in the log and honouring
//! [`Visibility`](mineworld_contracts::Visibility) over time; that is a perception system's job
//! (S10) and the transport's (S11), and a first cut here would be a second implementation to delete.
//! The events a world emits are visible to whoever dispatched, and the slice this pack belongs to
//! needs no more.
//!
//! **Components.** A perceived entity carries its identity, type, tags and location, and no
//! component records. Not an omission: no system in this world owns a display name yet, and exposing
//! *some* component because it happens to exist is how an observation quietly becomes a window onto
//! everything. When a pack owns state a stranger may read, the world's perception configuration says
//! so — it is not inferred here.

use mineworld_contracts::{
    Affordance, EntityId, Location, Observation, PerceivedEntity, Rejection, Relation,
    SpatialRequirement, WorldTime,
};
use mineworld_kernel::{World, WorldRead};

use crate::component::Presence;
use crate::interaction::{InteractionProvider, Offer};
use crate::system::present_in;

/// What `observer` perceives of `world` at `at`, and what it may attempt.
///
/// Total: an observer this world never allocated, or one this pack knows no location for, perceives
/// nothing and is told so — an empty observation is a legitimate view of a world, and it is the safe
/// answer, because everything exposed has to be added deliberately.
///
/// `providers` are the packs whose actions may appear as affordances, in the order a world composed
/// them. An offer from a pack whose action is not currently provided by an *enabled* system is
/// dropped, and that check is the kernel's route map rather than a list kept here: disabling a pack
/// removes its affordances from every observation in the world, with no edit to this file (`AC-2`).
pub fn observe(
    world: &World,
    observer: EntityId,
    at: WorldTime,
    providers: &[&dyn InteractionProvider],
) -> Observation {
    let read = world.read();
    let here = read.component::<Presence>(observer).map(Presence::location);
    let present = present_with(&read, here);

    let observation = Observation::new(observer, at);
    let observation = match here {
        Some(location) => observation.at_location(location),
        None => observation,
    };
    observation
        .perceiving(perceived(&read, here, &present))
        .relating(edges(&read, &present))
        .offering(affordances(
            world, &read, observer, here, &present, providers,
        ))
}

/// Everybody this pack knows to be in the same place as the observer, in [`EntityId`] order —
/// including the observer, because "who is here" is a question with one answer and a special case
/// for oneself is a special case every reader has to remember.
///
/// Ordered by identity rather than by when anybody arrived: iteration order of the component store
/// is by key, so two runs of the same world produce the same list (`AC-12`).
fn present_with(read: &WorldRead<'_>, here: Option<Location>) -> Vec<EntityId> {
    let Some(here) = here else {
        return Vec::new();
    };
    read.components::<Presence>()
        .filter(|(_, presence)| presence.location().place() == here.place())
        .map(|(entity, _)| entity)
        .collect()
}

/// The entities the observation lists: the place itself, then everybody in it.
///
/// The place is included so that an observation is self-contained — a client that was told only
/// "you are in place 7" would have to ask somewhere else what place 7 is, and there is nowhere else
/// to ask. It carries no location of its own, because a place inside itself is nonsense
/// (`contracts/src/observation.rs`).
fn perceived(
    read: &WorldRead<'_>,
    here: Option<Location>,
    present: &[EntityId],
) -> Vec<PerceivedEntity> {
    let mut entities = Vec::new();
    if let Some(here) = here
        && let Some(place) = read.entity(here.place().entity_id())
    {
        entities.push(
            PerceivedEntity::new(place.id(), place.entity_type()).with_tags(place.tags().clone()),
        );
    }
    for &entity in present {
        let Some(record) = read.entity(entity) else {
            continue;
        };
        let perceived = PerceivedEntity::new(record.id(), record.entity_type())
            .with_tags(record.tags().clone());
        entities.push(match read.component::<Presence>(entity) {
            Some(presence) => perceived.at(presence.location()),
            None => perceived,
        });
    }
    entities
}

/// The edges this pack wrote about the entities the observer perceives.
///
/// Only this pack's own relation type, and only for perceived entities. Exposing every edge touching
/// them would hand out whoever employs them and whoever they are related to, which is a perception
/// judgement this pack has no standing to make — and `INV-13` is defeated by a perception system
/// that exposes every edge exactly as surely as by one that exposes every entity.
///
/// Already in edge order: the relation store is a `BTreeSet`, so this adds no sort of its own.
fn edges(read: &WorldRead<'_>, present: &[EntityId]) -> Vec<Relation> {
    let relation_type = present_in();
    read.relations_of_type(&relation_type)
        .filter(|relation| present.contains(&relation.from()))
        .cloned()
        .collect()
}

/// Every offer every provider makes, priced by the server.
///
/// Two filters and one evaluation, and none of the three knows what any action is:
///
/// 1. an action no enabled system provides is not offered at all — the route map decides, which is
///    the same `INV-10` answer dispatch would give the request;
/// 2. what remains is evaluated against the declared requirement by
///    [`SpatialRequirement::evaluate`], the one implementation every system shares, so that an
///    affordance and a dispatch of the same request cannot disagree;
/// 3. the verdict travels with its reason, because a client told only *no* can only grey something
///    out, while one told `TooFarAway` can say so.
fn affordances(
    world: &World,
    read: &WorldRead<'_>,
    observer: EntityId,
    here: Option<Location>,
    present: &[EntityId],
    providers: &[&dyn InteractionProvider],
) -> Vec<Affordance> {
    let targets = core::iter::once(None).chain(present.iter().copied().map(Some));
    let mut affordances = Vec::new();
    for target in targets {
        for provider in providers {
            for offer in provider.offers(read, observer, target) {
                if world.systems().provider(offer.action_type()).is_none() {
                    continue;
                }
                affordances.push(verdict(read, here, target, &offer));
            }
        }
    }
    affordances
}

/// The server's answer for one offer: available, or unavailable with the reason.
///
/// An observer this pack knows no location for can still be offered an action that requires nothing
/// of space — a message, a telephone call. Anything else is refused as
/// [`Rejection::PreconditionFailed`] rather than as `TooFarAway`, because no distance was ever
/// established: the world does not know where this person is.
fn verdict(
    read: &WorldRead<'_>,
    here: Option<Location>,
    target: Option<EntityId>,
    offer: &Offer,
) -> Affordance {
    let requirement = offer.requirement();
    let there = target
        .and_then(|target| read.component::<Presence>(target))
        .map(Presence::location);
    let evaluated = match here {
        Some(here) => requirement.evaluate(&here, there.as_ref(), offer.target_available()),
        None if requirement == SpatialRequirement::NONE => Ok(()),
        None => Err(Rejection::PreconditionFailed),
    };
    match evaluated {
        Ok(()) => Affordance::available(offer.action_type().clone(), target, requirement),
        Err(reason) => {
            Affordance::unavailable(offer.action_type().clone(), target, requirement, reason)
        }
    }
}
