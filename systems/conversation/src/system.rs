//! The installable system: what it declares, and the four things a world asks of it.

use mineworld_contracts::{
    ActionIntent, EntityId, EntityType, Event, EventEnvelope, LifecycleState, PersonId, PlaceId,
    Rejection, RejectionCode, SimDuration, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::{InteractionProvider, Offer, Presence, PresenceSystem};

use crate::action::{Talk, talk_requirement};
use crate::codec;
use crate::component::{ConversationHistory, Heard};
use crate::event::{ConversationStarted, Spoke};
use crate::utterance::Utterance;

/// How long a silence has to be before the next exchange starts a new conversation.
///
/// Five minutes of simulated time. Without a rule of this kind
/// [`ConversationStarted`] would either fire on every `talk` — saying nothing — or fire once and never
/// again, which is worse, because two people who spoke yesterday and meet again today have plainly
/// started talking. The gap is this pack's policy and becomes configuration in S7; what matters
/// architecturally is that the rule lives in the system that owns the concept, and that it is decided
/// from the world's clock rather than from a wall clock, so a replay makes the same decision.
pub const CONVERSATION_GAP: SimDuration = SimDuration::from_seconds(300);

/// Speaking, and remembering having been spoken to.
///
/// A unit struct: a system holds no fields, because its mutable state is the components it owns and
/// those live in the world behind a gated view (`INV-7`).
pub struct ConversationSystem;

impl SystemIdentity for ConversationSystem {
    const ID: SystemId = SystemId::from_static("conversation");
}

/// This pack's own reason for refusing a request it cannot read.
///
/// One of the kernel's five closed reasons would be a worse answer: a malformed payload is not a fact
/// about the world, and a client shown `PreconditionFailed` would look for a precondition. A client
/// that does not recognize this code shows the request as refused, which is correct, and a developer
/// building a client by hand gets the detail that says what was wrong with the frame.
const MALFORMED_PAYLOAD: RejectionCode = RejectionCode::from_static("malformed-payload");

impl System for ConversationSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<ConversationHistory>()
            .providing::<Talk>()
            .emitting::<ConversationStarted>()
            .emitting::<Spoke>()
            .subscribing_to::<Spoke>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<ConversationHistory>()
    }

    /// Decides whether this person can speak to that one, reading state and writing nothing.
    ///
    /// The order is deliberate, and it is the order a player experiences: first whether the request
    /// makes sense at all, then whether it is possible here and now. Everything spatial is answered by
    /// one call to [`SpatialRequirement::evaluate`](mineworld_contracts::SpatialRequirement::evaluate)
    /// against the requirement this pack declared — there is no distance arithmetic in this crate, and
    /// that is what makes the answer a client is shown in an affordance and the answer a dispatch
    /// gives the same answer.
    ///
    /// The positions come from [`Presence`], which this pack reads and cannot write. The
    /// `actor_location` a client may report on the request is deliberately **ignored**: it is a claim
    /// by an unprivileged process about where it thinks it is, and `ENGINEERING_RULES.md` §8 puts the
    /// decision on the server. A world that later wants to honour an unapplied client position gains
    /// it by having a movement system apply it, not by this system trusting it.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let _talk: Talk =
            codec::action_payload(intent.payload()).map_err(|error| Rejection::System {
                code: MALFORMED_PAYLOAD,
                detail: Some(error.to_string()),
            })?;

        let speaker = intent.actor();
        let listener = intent.target().ok_or(Rejection::PreconditionFailed)?;
        if listener == speaker {
            return Err(Rejection::NoSupportedInteraction);
        }
        if !is_person(world, speaker) {
            return Err(Rejection::NoSupportedInteraction);
        }
        let listener_record = world.entity(listener).ok_or(Rejection::TargetUnavailable)?;
        if listener_record.entity_type() != EntityType::Person {
            return Err(Rejection::NoSupportedInteraction);
        }

        let here = located(world, speaker).ok_or(Rejection::PreconditionFailed)?;
        let there = located(world, listener);
        talk_requirement().evaluate(
            &here,
            there.as_ref(),
            listener_record.lifecycle() == LifecycleState::Active,
        )
    }

    /// States what happened, and writes nothing.
    ///
    /// One or two facts: the exchange always, and the beginning of a conversation when the two have
    /// been silent for longer than [`CONVERSATION_GAP`]. The component this pack owns is written while
    /// *reducing* the exchange, not here, which is what makes the history a projection of the log.
    ///
    /// The two audiences differ, and both are judgements this pack is the only one able to make: an
    /// exchange can be overheard by anyone in the place, while two people beginning to talk is
    /// something only they are party to.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let exchange = Exchange::of(&world.read(), intent)?;
        let starts = !continues_a_conversation(
            &world.read(),
            exchange.speaker,
            exchange.listener,
            world.at(),
        );

        let mut emissions = Vec::new();
        if starts {
            emissions.push(
                Emission::new::<ConversationStarted>(
                    codec::encode(&ConversationStarted::new(
                        exchange.speaker,
                        exchange.listener,
                    )),
                    Visibility::Participants,
                )
                .about(exchange.both())
                .with_participants(exchange.both())
                .at_place(exchange.place),
            );
        }
        let both = exchange.both();
        emissions.push(
            Emission::new::<Spoke>(
                codec::encode(&Spoke::new(
                    exchange.speaker,
                    exchange.listener,
                    exchange.utterance,
                )),
                Visibility::Place(exchange.place),
            )
            .about(vec![exchange.listener.entity_id()])
            .with_participants(both)
            .at_place(exchange.place),
        );
        Ok(emissions)
    }

    /// Reduces an exchange into the listener's history.
    ///
    /// The whole of Alice remembering. It is one insert into one component this pack owns, from one
    /// fact this pack emitted — which is why `docs/MVP.md` §9.2 can call it a projection rather than a
    /// memory system, and why a replay of the log reproduces it.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != Spoke::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let spoke: Spoke = codec::event_payload(event.payload())?;
        let listener = spoke.listener().entity_id();
        let heard = Heard::new(spoke.speaker(), event.at(), spoke.into_utterance());

        // Read, change, write back, because a history that does not exist yet has to be created and a
        // history that does must not be replaced by a fresh one. Cloning 32 small entries is not the
        // cost worth optimizing in a world of hundreds of people.
        let mut history = world
            .read()
            .component::<ConversationHistory>(listener)
            .cloned()
            .unwrap_or_default();
        history.remember(heard);
        world.insert(listener, history)?;
        Ok(Vec::new())
    }
}

impl InteractionProvider for ConversationSystem {
    /// Offers `talk` against every person the observer is not.
    ///
    /// This is the answer perception cannot produce: which of this pack's actions apply to this pair,
    /// what `talk` requires of space, and whether the listener is available — a domain question, which
    /// is why the flag is set here and not in the pack that measures distances.
    ///
    /// It deliberately does not check distance, place or whether the action is provided in this world.
    /// Those are perception's two jobs, and duplicating either here would create a second answer that
    /// could disagree with dispatch.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        let Some(target) = target else {
            // Speaking is directed at somebody; there is nothing to offer against nobody.
            return Vec::new();
        };
        if target == observer || !is_person(world, observer) {
            return Vec::new();
        }
        let Some(listener) = world.entity(target) else {
            return Vec::new();
        };
        if listener.entity_type() != EntityType::Person {
            return Vec::new();
        }
        vec![
            Offer::new::<Talk>(talk_requirement())
                .with_target_available(listener.lifecycle() == LifecycleState::Active),
        ]
    }
}

/// What `resolve` needs, established once from state `validate` has already checked.
struct Exchange {
    speaker: PersonId,
    listener: PersonId,
    place: PlaceId,
    utterance: Utterance,
}

impl Exchange {
    /// Reads the request and the world into the four things an exchange is.
    ///
    /// Every failure here is a case [`ConversationSystem::validate`] already refuses, and dispatch
    /// always validates before it resolves. They are still errors rather than assumptions, because the
    /// honest report for "my own validation and my own resolution disagree" is that this system failed
    /// to resolve an action it provides — which is what
    /// [`KernelError::ActionNotResolvedBySystem`] says, and what the kernel documents an `Err` out of
    /// `resolve` to mean: a bug in a system, not a refused request. The alternative, a panic, would
    /// take down a running world.
    fn of(world: &WorldRead<'_>, intent: &ActionIntent) -> Result<Self, KernelError> {
        let talk: Talk = codec::action_payload(intent.payload())?;
        let listener = intent.target().ok_or_else(|| unresolvable(intent))?;
        let speaker_record = world.require_entity(intent.actor())?;
        let listener_record = world.require_entity(listener)?;
        let place = located(world, intent.actor())
            .ok_or_else(|| unresolvable(intent))?
            .place();

        Ok(Self {
            speaker: PersonId::new(speaker_record.id(), speaker_record.entity_type())?,
            listener: PersonId::new(listener_record.id(), listener_record.entity_type())?,
            place,
            utterance: talk.into_utterance(),
        })
    }

    /// Both parties, speaker first: the order is part of the fact, so it is stated once here rather
    /// than at each emission.
    fn both(&self) -> Vec<EntityId> {
        vec![self.speaker.entity_id(), self.listener.entity_id()]
    }
}

/// The refusal for a case `validate` has already excluded — see [`Exchange::of`].
fn unresolvable(intent: &ActionIntent) -> KernelError {
    KernelError::ActionNotResolvedBySystem {
        system: ConversationSystem::ID,
        action_type: intent.action_type().clone(),
    }
}

/// Whether this world holds a person by that identity.
fn is_person(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world
        .entity(entity)
        .is_some_and(|record| record.entity_type() == EntityType::Person)
}

/// Where this world says that entity is, as far as [`PresenceSystem`] has been told.
///
/// [`None`] is *unknown*, not *nowhere*: a component whose owning system is not installed reads as
/// absent, which is what lets this pack tolerate a world where presence has been disabled without
/// pretending to know positions.
fn located(world: &WorldRead<'_>, entity: EntityId) -> Option<mineworld_contracts::Location> {
    world.component::<Presence>(entity).map(Presence::location)
}

/// Whether these two are already in a conversation, from what either of them remembers hearing.
///
/// Both directions, because a reply is part of the same conversation: Alice's history records what Bob
/// said to her and Bob's records what she said to him, so asking only one would make every reply start a
/// second conversation. Checked against the world's clock and [`CONVERSATION_GAP`].
fn continues_a_conversation(
    world: &WorldRead<'_>,
    speaker: PersonId,
    listener: PersonId,
    now: WorldTime,
) -> bool {
    [(listener, speaker), (speaker, listener)]
        .into_iter()
        .any(|(holder, other)| {
            world
                .component::<ConversationHistory>(holder.entity_id())
                .and_then(|history| history.last_heard_from(other))
                .is_some_and(|heard| within_the_gap(heard.at(), now))
        })
}

/// Whether `then` is recent enough for a conversation to still be going on at `now`.
fn within_the_gap(then: WorldTime, now: WorldTime) -> bool {
    now.duration_since(then)
        .is_some_and(|elapsed| elapsed <= CONVERSATION_GAP)
}
