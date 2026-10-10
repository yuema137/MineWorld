//! The installable system: what it declares, and the four things a world asks of it.

use mineworld_contracts::{
    Action, ActionIntent, ComponentRecord, EntityId, EntityType, Event, EventEnvelope,
    LifecycleState, Location, PersonId, PlaceId, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_sdk::SystemPack;
use serde::Serialize;
use serde_json::Value;

use mineworld_presence::{
    Arrived, Offer, PerceptionProvider, PresenceSystem, StoppedShort, admit, arrivals,
};

use crate::action::{
    Destination, Move, WalkStep, WalkTo, move_offer_requirement, stride_requirement,
};
use crate::codec;
use crate::component::{Passage, Passages, Walking};
use crate::event::{Ended, PassageOpened, WalkEnded, WalkStarted, walk_ended, walk_started};
use crate::walk::{self, Step};

/// Whether a person may walk where they ask — decided here, on the server, against where the world
/// says they are — and the walks people are on.
///
/// It decides and does not record positions: a person's position is presence's state, and this
/// system states presence's own `arrived` fact, built by presence's `arrivals`, for presence to
/// reduce (`DECISIONS.md` `ARC-26`). That is why it depends on presence and presence knows nothing
/// of it, and why disabling it takes away `move` and walking and nothing else (`AC-2`).
#[derive(Default)]
pub struct MovementSystem;

impl SystemIdentity for MovementSystem {
    const ID: SystemId = SystemId::from_static("movement");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): nothing.
/// It declares no biographical fact and owns no authored section; a place's `passages` are a field
/// of the World Pack format, bound to this pack by the loader (`ARC-31`, point 5).
impl SystemPack for MovementSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
}

/// This pack's own reason for refusing a request it cannot read — the same code, for the same
/// reason, as the other packs' (a malformed payload is a fact about the request, not the world).
const MALFORMED_PAYLOAD: RejectionCode = RejectionCode::from_static("malformed-payload");

impl System for MovementSystem {
    /// Version 2: `walk-to`, `walk-step`, the `walking` component and the two walk facts
    /// (`DECISIONS.md` `ARC-75`), so a save written by version 1 is refused by name (`ARC-25`).
    const VERSION: SystemVersion = SystemVersion::new(2);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<Passages>()
            .owning::<Walking>()
            .providing::<Move>()
            .providing::<WalkTo>()
            .providing::<WalkStep>()
            .emitting::<Arrived>()
            .emitting::<StoppedShort>()
            .emitting::<PassageOpened>()
            .emitting::<WalkStarted>()
            .emitting::<WalkEnded>()
            .subscribing_to::<PassageOpened>()
            .subscribing_to::<Arrived>()
            .subscribing_to::<StoppedShort>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Passages>()?;
        tables.component::<Walking>()
    }

    /// Decides whether this person may make this request, reading state and writing nothing.
    ///
    /// ```text
    /// every request  payload readable                    else malformed-payload
    ///                actor exists, is a person           else PreconditionFailed /
    ///                                                         NoSupportedInteraction
    ///                the actor has a Presence            else PreconditionFailed (placement is genesis)
    /// move           presence admits the destination     else PreconditionFailed
    ///                reachable in one move               else TooFarAway (see `reachable`)
    /// walk-to        presence admits a place destination else PreconditionFailed
    ///                walk::begin                         TooFarAway, PreconditionFailed, no-route
    /// walk-step      the actor is walking                else PreconditionFailed
    /// ```
    ///
    /// The position moved *from* is presence's record. The `actor_location` a client may put on
    /// the request is ignored, as every pack ignores it: it is a report, and the server decides
    /// (`ENGINEERING_RULES.md` §8).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let action = intent.action_type();
        if *action == WalkTo::ACTION_TYPE {
            let requested: WalkTo = readable(intent)?;
            let (person, here) = walker(world, intent)?;
            if let Destination::Place(to) = requested.to() {
                admit(world, person, to)?;
            }
            return walk::begin(world, person, &here, requested.to()).map(|_| ());
        }
        if *action == WalkStep::ACTION_TYPE {
            let _: WalkStep = readable(intent)?;
            let (person, _) = walker(world, intent)?;
            return world
                .component::<Walking>(person.entity_id())
                .map(|_| ())
                .ok_or(Rejection::PreconditionFailed);
        }
        let requested: Move = readable(intent)?;
        let actor = world
            .entity(intent.actor())
            .ok_or(Rejection::PreconditionFailed)?;
        let person = PersonId::new(actor.id(), actor.entity_type())
            .map_err(|_| Rejection::NoSupportedInteraction)?;
        let to = requested.to();
        admit(world, person, to)?;
        let from =
            walk::presence(world, person.entity_id()).ok_or(Rejection::PreconditionFailed)?;
        reachable(world, &from, &to)
    }

    /// Resolves an admitted request into the facts it causes.
    ///
    /// ```text
    /// move       walk-ended { stopped } when the actor was walking, then presence's facts for the
    ///            arrival (arrivals)
    /// walk-to    walk-ended { replaced } when the actor was walking; the new Walking; walk-started
    /// walk-step  one stride through `reachable` and arrivals, the walk updated — or walk-ended
    /// ```
    ///
    /// Positions are never written here: presence records what `arrivals` states.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let action = intent.action_type();
        let read = world.read();
        let actor = read.require_entity(intent.actor())?;
        let person = PersonId::new(actor.id(), actor.entity_type())?;
        let here = walk::presence(&read, person.entity_id());
        let walking = read.component::<Walking>(person.entity_id()).cloned();
        let mut facts = Vec::new();

        if *action == WalkTo::ACTION_TYPE {
            let requested: WalkTo = codec::action_payload(intent.payload())?;
            let here = here.ok_or_else(|| refused(WalkStarted::EVENT_TYPE))?;
            let planned = walk::begin(&read, person, &here, requested.to()).map_err(|reason| {
                KernelError::FactRefusedByOwner {
                    system: Self::ID,
                    event_type: WalkStarted::EVENT_TYPE,
                    reason,
                }
            })?;
            if let Some(previous) = walking {
                facts.push(walk_ended(
                    person,
                    previous.destination(),
                    Ended::Replaced,
                    here.place(),
                ));
            }
            facts.push(walk_started(person, requested.to(), here.place()));
            world.insert(person.entity_id(), planned)?;
            return Ok(facts);
        }

        if *action == WalkStep::ACTION_TYPE {
            let (Some(here), Some(walking)) = (here, walking) else {
                return Err(refused(WalkEnded::EVENT_TYPE));
            };
            let step = walk::step(&read, person, &here, &walking);
            let (to, next) = match step {
                Step::End(outcome) => {
                    facts.push(walk_ended(
                        person,
                        walking.destination(),
                        outcome,
                        here.place(),
                    ));
                    world.remove::<Walking>(person.entity_id())?;
                    return Ok(facts);
                }
                Step::Stride { to, walking } => (to, *walking),
            };
            if reachable(&read, &here, &to).is_err() {
                // Only a wayfinder that moved a doorway's end beyond a stride of the doorway can make
                // a planned stride one `move` would refuse: the walk cannot go on this way.
                facts.push(walk_ended(
                    person,
                    walking.destination(),
                    Ended::NoRoute,
                    here.place(),
                ));
                world.remove::<Walking>(person.entity_id())?;
                return Ok(facts);
            }
            facts.extend(arrived(&read, person, to)?);
            world.insert(person.entity_id(), next)?;
            return Ok(facts);
        }

        let requested: Move = codec::action_payload(intent.payload())?;
        let arrival = arrived(&read, person, requested.to())?;
        if let (Some(walking), Some(here)) = (walking, here) {
            facts.push(walk_ended(
                person,
                walking.destination(),
                Ended::Stopped,
                here.place(),
            ));
            world.remove::<Walking>(person.entity_id())?;
        }
        facts.extend(arrival);
        Ok(facts)
    }

    /// Reduces this pack's facts and follows the walkers' own arrivals.
    ///
    /// ```text
    /// passage-opened   → Passages of both places; a passage from a place to itself, or to something
    ///                    that is not a place of this world, is refused and nothing is written
    /// arrived          (presence's) a walker recorded where their walk ends → Walking removed,
    ///                  walk-ended { arrived }, caused by the same request
    /// stopped-short    (presence's) a walker stopped by a person → remembered, to plan round them
    /// ```
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == Arrived::EVENT_TYPE {
            let arrived: Arrived = codec::event_payload(event.payload())?;
            let person = arrived.person();
            let read = world.read();
            let Some(walking) = read.component::<Walking>(person.entity_id()).cloned() else {
                return Ok(Vec::new());
            };
            if !walk::arrived(&read, &walking, &arrived.location()) {
                return Ok(Vec::new());
            }
            world.remove::<Walking>(person.entity_id())?;
            return Ok(vec![walk_ended(
                person,
                walking.destination(),
                Ended::Arrived,
                arrived.location().place(),
            )]);
        }
        if *kind == StoppedShort::EVENT_TYPE {
            let stopped: StoppedShort = codec::event_payload(event.payload())?;
            let person = stopped.person().entity_id();
            let read = world.read();
            let by_person = stopped.by().filter(|by| {
                read.entity(*by)
                    .is_some_and(|entity| entity.entity_type() == EntityType::Person)
            });
            if let Some(mut walking) = read.component::<Walking>(person).cloned() {
                walking.stopped_by = by_person;
                world.insert(person, walking)?;
            }
            return Ok(Vec::new());
        }
        if *kind != PassageOpened::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let opened: PassageOpened = codec::event_payload(event.payload())?;
        let (a, a_at) = opened.a();
        let (b, b_at) = opened.b();
        {
            let read = world.read();
            if a == b || !is_place(&read, a) || !is_place(&read, b) {
                return Err(KernelError::FactRefusedByOwner {
                    system: Self::ID,
                    event_type: PassageOpened::EVENT_TYPE,
                    reason: Rejection::PreconditionFailed,
                });
            }
        }

        for (from, passage) in [
            (a, Passage::new(b, a_at, b_at)),
            (b, Passage::new(a, b_at, a_at)),
        ] {
            let mut passages = world
                .read()
                .component::<Passages>(from.entity_id())
                .cloned()
                .unwrap_or_default();
            passages.open(passage);
            world.insert(from.entity_id(), passages)?;
        }
        Ok(Vec::new())
    }
}

/// The payload as this action type, or `malformed-payload`.
fn readable<A: Action + serde::de::DeserializeOwned>(
    intent: &ActionIntent,
) -> Result<A, Rejection> {
    codec::action_payload(intent.payload()).map_err(|error| Rejection::System {
        code: MALFORMED_PAYLOAD,
        detail: Some(error.to_string()),
    })
}

/// The actor as a person who has a presence: who, and where they are.
fn walker(world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(PersonId, Location), Rejection> {
    let actor = world
        .entity(intent.actor())
        .ok_or(Rejection::PreconditionFailed)?;
    let person = PersonId::new(actor.id(), actor.entity_type())
        .map_err(|_| Rejection::NoSupportedInteraction)?;
    let here = walk::presence(world, person.entity_id()).ok_or(Rejection::PreconditionFailed)?;
    Ok((person, here))
}

/// Presence's facts for `person` arriving at `to`, through presence's constructor — whatever facts
/// presence says the arrival becomes.
fn arrived(
    world: &WorldRead<'_>,
    person: PersonId,
    to: Location,
) -> Result<Vec<Emission>, KernelError> {
    arrivals(world, person, to).map_err(|reason| KernelError::FactRefusedByOwner {
        system: PresenceSystem::ID,
        event_type: Arrived::EVENT_TYPE,
        reason,
    })
}

/// A request that `validate` admitted and the world no longer supports at resolution.
fn refused(event_type: mineworld_contracts::EventTypeId) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: MovementSystem::ID,
        event_type,
        reason: Rejection::PreconditionFailed,
    }
}

/// Whether `person` is on a walk: what a host asks to decide whether to step a seat (step-11 SD-N15).
pub fn is_walking(world: &WorldRead<'_>, person: PersonId) -> bool {
    world.component::<Walking>(person.entity_id()).is_some()
}

/// What a walking person discloses: the destination and the next four waypoints at most.
#[derive(Serialize)]
struct Disclosed<'a> {
    destination: Destination,
    next: &'a [mineworld_contracts::LocalPosition],
}

/// The most waypoints a walking person discloses.
const DISCLOSED_WAYPOINTS: usize = 4;

impl PerceptionProvider for MovementSystem {
    /// Offers `move` and `walk-to` to any living person, and `walk-step` to one who is walking — each
    /// once, against nobody, without a request (step-11 SD-N7, `ARC-34`).
    ///
    /// With [`move_offer_requirement`] — nothing required of a target — because an offer's
    /// requirement is evaluated against its target and none has one. Whether one particular
    /// destination is reachable is answered when it is asked, by [`MovementSystem::validate`].
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        if target.is_some() {
            return Vec::new();
        }
        let is_living_person = world.entity(observer).is_some_and(|entity| {
            entity.entity_type() == EntityType::Person
                && entity.lifecycle() != LifecycleState::Destroyed
        });
        if !is_living_person {
            return Vec::new();
        }
        let mut offers = vec![
            Offer::new::<Move>(move_offer_requirement()),
            Offer::new::<WalkTo>(move_offer_requirement()),
        ];
        if world.component::<Walking>(observer).is_some() {
            offers.push(Offer::new::<WalkStep>(move_offer_requirement()));
        }
        offers
    }

    /// Discloses a place's [`Passages`] — where its doorways are — to whoever perceives the place, and
    /// a walking person's walk — where to, and the next waypoints — to whoever perceives the person.
    ///
    /// Perception asks only about entities the observation already lists, and it lists only the
    /// observer's own place, so this tells a person the ways out of the room they stand in and of no
    /// other. It states *where* a doorway is, never whether one may pass: that stays
    /// [`MovementSystem::validate`]'s judgement, made when a `move` is asked (`ENGINEERING_RULES.md`
    /// §8). A controller that cannot see the door cannot walk out of the room, which is why this
    /// exists (step-08 §10.1 Q4). A walk is disclosed so that a client draws the server's own plan and
    /// a sender knows to keep stepping (step-11 SD-N8). A person who is not walking, and a place with
    /// no passages, disclose nothing — absence rather than an empty record.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        _observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let Some(entity) = world.entity(subject) else {
            return Vec::new();
        };
        match entity.entity_type() {
            EntityType::Place => world
                .component::<Passages>(subject)
                .map(|passages| {
                    vec![ComponentRecord::new::<Passages>(
                        subject,
                        codec::to_value(passages),
                    )]
                })
                .unwrap_or_default(),
            EntityType::Person => world
                .component::<Walking>(subject)
                .map(|walking| {
                    let waypoints = walking.waypoints();
                    let disclosed = Disclosed {
                        destination: walking.destination(),
                        next: &waypoints[..waypoints.len().min(DISCLOSED_WAYPOINTS)],
                    };
                    vec![ComponentRecord::new::<Walking>(
                        subject,
                        codec::to_value(&disclosed),
                    )]
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }
}

/// Whether `to` is one move from `from`. Every distance is decided by the contract layer's one
/// evaluator with the destination or a doorway as the target — there is no distance arithmetic in
/// this function.
///
/// ```text
/// same place      a stride: `to` within MAX_STRIDE of `from`
/// another place   through the passage from `from`'s place to `to`'s:
///                   none                    TooFarAway — not adjoining; that is travel
///                   doorway here known      `from` within MAX_STRIDE of it, else TooFarAway
///                   doorway there known     `to` within MAX_STRIDE of it, else TooFarAway
/// ```
///
/// Where a position is not modelled — on either side of a request or a doorway — the evaluator's
/// own degeneracy applies (`CORE_CONCEPTS.md` §6.3): the range becomes *same place*, so a semantic
/// world moves freely within a place and through its passages. Every stride of a walk is checked
/// here too, so a walk is never a way round the `move` rule.
fn reachable(world: &WorldRead<'_>, from: &Location, to: &Location) -> Result<(), Rejection> {
    let stride = stride_requirement();
    if from.place() == to.place() {
        return stride.evaluate(from, Some(to), true);
    }
    let passage = world
        .component::<Passages>(from.place().entity_id())
        .and_then(|passages| passages.to(to.place()))
        .ok_or(Rejection::TooFarAway)?;
    if let Some(here) = passage.here() {
        let doorway = Location::in_place(from.place()).with_local(here);
        stride.evaluate(from, Some(&doorway), true)?;
    }
    if let Some(there) = passage.there() {
        let doorway = Location::in_place(to.place()).with_local(there);
        stride.evaluate(to, Some(&doorway), true)?;
    }
    Ok(())
}

/// Whether this world holds a living place by that identity.
fn is_place(world: &WorldRead<'_>, place: PlaceId) -> bool {
    world.entity(place.entity_id()).is_some_and(|record| {
        record.entity_type() == EntityType::Place && record.lifecycle() != LifecycleState::Destroyed
    })
}
