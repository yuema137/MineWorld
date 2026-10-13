//! The installable system: what it declares, and the four things a world asks of it.

use mineworld_contracts::{
    Action, ActionIntent, ComponentRecord, EntityId, EntityType, Event, EventEnvelope,
    LifecycleState, PersonId, PlaceId, Rejection, RejectionCode, SimDuration, SpatialRequirement,
    SystemId, WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions::{self, Role, Roles};
use serde_json::Value;

use crate::action::{Talk, range, talk_requirement, talk_requirement_within};
use crate::codec;
use crate::component::{ConversationHistory, Heard, REMEMBERED_AT_MOST};
use crate::event::{ConversationStarted, Spoke};
use crate::interactions::owner_default;
use crate::utterance::Utterance;

/// How long a silence has to be before the next exchange starts a new conversation.
///
/// Five minutes of simulated time. Without a rule of this kind
/// [`ConversationStarted`] would either fire on every `talk` — saying nothing — or fire once and never
/// again, which is worse, because two people who spoke yesterday and meet again today have plainly
/// started talking. The rule lives in the system that owns the concept, and it is decided from the
/// world's clock rather than from a wall clock, so a replay makes the same decision.
///
/// This is the compiled default. Since S17's PR IL-b a world may choose another gap — for everyone, for
/// a class of speaker or listener, or in one place — through this pack's section of the World's
/// Interaction List (`gap`, [`crate::interactions`], `ARC-63`).
pub const CONVERSATION_GAP: SimDuration = SimDuration::from_seconds(300);

/// Speaking, and remembering having been spoken to.
///
/// A unit struct: a system holds no fields, because its mutable state is the components it owns and
/// those live in the world behind a gated view (`INV-7`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationSystem;

impl SystemIdentity for ConversationSystem {
    const ID: SystemId = SystemId::from_static("conversation");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// section of the World's Interaction List (`ARC-63`), which is its configuration. It declares no
/// biographical fact (`ARC-29`) and owns no authored section.
impl SystemPack for ConversationSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::interactions!();
}

/// This pack's own reason for refusing a request it cannot read.
///
/// One of the kernel's five closed reasons would be a worse answer: a malformed payload is not a fact
/// about the world, and a client shown `PreconditionFailed` would look for a precondition. A client
/// that does not recognize this code shows the request as refused, which is correct, and a developer
/// building a client by hand gets the detail that says what was wrong with the frame.
const MALFORMED_PAYLOAD: RejectionCode = RejectionCode::from_static("malformed-payload");

impl System for ConversationSystem {
    /// 2 since S17's PR IL-b: the declaration gained the section's component and configured fact.
    /// 3 since PR IL-e: the section gained the `talk` rule, `range`, `remembered`, its two facts'
    /// consequences and the `remember` knob, so a configured fact's payload changed shape (`ARC-25`).
    const VERSION: SystemVersion = SystemVersion::new(3);

    fn declaration(&self) -> SystemDeclaration {
        interactions::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .depending_on([PresenceSystem::ID])
                .owning::<ConversationHistory>()
                .providing::<Talk>()
                .emitting::<ConversationStarted>()
                .emitting::<Spoke>()
                .subscribing_to::<Spoke>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        interactions::install::<Self>(tables)?;
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
    ///
    /// The world's Interaction List is asked once the request names two people and the speaker has a
    /// place to ask it at, and before space is judged (`ARC-63` item 8): a forbidden pair is refused
    /// `PermissionDenied` however near or far they stand, as its offer is. The range space is judged
    /// with is the one the list gives this pair here.
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
        let (permitted, requirement) = talk_terms(world, here.place(), speaker, listener);
        permitted?;
        let there = located(world, listener);
        requirement.evaluate(
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
        let roles = pair(exchange.speaker.entity_id(), exchange.listener.entity_id());
        let gap = interactions::parameters::<Self>(&world.read(), exchange.place, &roles).gap;
        let starts = !continues_a_conversation(
            &world.read(),
            exchange.speaker,
            exchange.listener,
            world.at(),
            SimDuration::from_seconds(i64::from(gap)),
        );

        // Each fact's audience is its owner default unless the world's list narrows it here, for
        // this pair (`ARC-65`); the default is written once, in the fact's declaration.
        let audience = |fact| {
            interactions::consequence::<Self>(
                &world.read(),
                Some(exchange.place),
                fact,
                &roles,
                owner_default(fact, exchange.place),
            )
            .visibility
        };
        let (started_audience, spoke_audience) = (
            audience(&ConversationStarted::EVENT_TYPE),
            audience(&Spoke::EVENT_TYPE),
        );

        let mut emissions = Vec::new();
        if starts {
            emissions.push(
                Emission::new::<ConversationStarted>(
                    codec::encode(&ConversationStarted::new(
                        exchange.speaker,
                        exchange.listener,
                    )),
                    started_audience,
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
                spoke_audience,
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
    ///
    /// The world's list may say this listener keeps nothing of this line (`remember: off`), or keeps
    /// fewer or more lines (`remembered`), both looked up with the fact's speaker, listener and place.
    /// Neither touches the fact itself: it is recorded, perceived and reduced by other packs as before.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if interactions::reduce(world, event)? || *event.event_type() != Spoke::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let spoke: Spoke = codec::event_payload(event.payload())?;
        let listener = spoke.listener().entity_id();
        let roles = pair(spoke.speaker().entity_id(), listener);
        let read = world.read();
        let knobs = interactions::consequence::<Self>(
            &read,
            event.place(),
            &Spoke::EVENT_TYPE,
            &roles,
            event.visibility().clone(),
        )
        .knobs;
        if knobs.remember == Some(false) {
            return Ok(Vec::new());
        }
        let at_most = event.place().map_or(REMEMBERED_AT_MOST, |place| {
            usize::from(interactions::parameters::<Self>(&read, place, &roles).remembered)
        });
        let heard = Heard::new(spoke.speaker(), event.at(), spoke.into_utterance());

        // Read, change, write back, because a history that does not exist yet has to be created and a
        // history that does must not be replaced by a fresh one. Cloning 32 small entries is not the
        // cost worth optimizing in a world of hundreds of people.
        let mut history = world
            .read()
            .component::<ConversationHistory>(listener)
            .cloned()
            .unwrap_or_default();
        history.remember_within(heard, at_most);
        world.insert(listener, history)?;
        Ok(Vec::new())
    }
}

impl PerceptionProvider for ConversationSystem {
    /// Offers `talk` against every person the observer is not.
    ///
    /// This is the answer perception cannot produce: which of this pack's actions apply to this pair,
    /// what `talk` requires of space, and whether the listener is available — a domain question, which
    /// is why the flag is set here and not in the pack that measures distances.
    ///
    /// It deliberately does not check distance, place or whether the action is provided in this world.
    /// Those are perception's two jobs, and duplicating either here would create a second answer that
    /// could disagree with dispatch.
    ///
    /// What it does answer is the world's Interaction List, through the same call `validate` makes: a
    /// forbidden pair's `talk` is offered refused `PermissionDenied`, with its requirement still shown,
    /// and the requirement carries the range the list gives this pair at the observer's place. An
    /// observer with no place is answered with the compiled default, as its dispatch is refused
    /// `PreconditionFailed` before the list is asked.
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
        let available = listener.lifecycle() == LifecycleState::Active;
        let Some(here) = located(world, observer) else {
            return vec![Offer::new::<Talk>(talk_requirement()).with_target_available(available)];
        };
        let (permitted, requirement) = talk_terms(world, here.place(), observer, target);
        let offer = Offer::new::<Talk>(requirement).with_target_available(available);
        vec![match permitted {
            Ok(()) => offer,
            Err(reason) => offer.refused(reason),
        }]
    }

    /// Discloses a person's [`ConversationHistory`] **in that person's own observation, and nowhere
    /// else**.
    ///
    /// This is how `docs/MVP.md` §9.2's third arrow is drawn — *history → her controller's context*.
    /// The first two arrows already existed: the log is the log, and `react` reduces `spoke` into the
    /// component. What was missing was a way for whoever decides Alice's actions to read it, because
    /// a controller is handed an [`Observation`](mineworld_contracts::Observation) and never the world
    /// (`INV-13`), and an observation carried no component records at all.
    ///
    /// One rule, and the narrowest one that answers the question:
    ///
    /// ```text
    /// subject == observer    disclosed: what I have been told is mine to know
    /// otherwise              nothing: what somebody else was told is not mine to read
    /// ```
    ///
    /// The asymmetry is the point. Alice's controller learns that a player spoke to her three minutes
    /// ago; a player learns what Alice said *to them*, from their own history, through the same
    /// mechanism rather than a second one — because `react` writes the entry on the **listener**. And
    /// a client cannot read a stranger's memory by asking, because there is no asking: an observation
    /// is a list of what was exposed.
    ///
    /// Two different things hold `INV-13` up here, and it is worth being exact about which is which.
    /// *What* may be known is this pack's judgement, made by construction: one component is named,
    /// and one observer. *Whom a record may be about* is not left to this pack's good behaviour —
    /// perception drops any record that is not about the subject it asked about
    /// ([`PerceptionProvider::discloses`]), so a pack that answered about a third party would
    /// disclose nothing. The rule above is therefore honest about its own scope, and a client cannot
    /// read a stranger's memory whichever half fails.
    ///
    /// A person who has been told nothing has no component, and nothing is disclosed — absence of
    /// knowledge rather than an empty record, which is the same answer the component store gives.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if subject != observer {
            return Vec::new();
        }
        world
            .component::<ConversationHistory>(observer)
            .map(|history| {
                vec![ComponentRecord::new::<ConversationHistory>(
                    observer,
                    codec::to_value(history),
                )]
            })
            .unwrap_or_default()
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

/// The roles of an exchange: the speaker acts, the listener is its target.
fn pair(speaker: EntityId, listener: EntityId) -> Roles {
    Roles::new()
        .with(Role::Actor, speaker)
        .with(Role::Target, listener)
}

/// What the world's Interaction List says about `speaker` talking to `listener` at `place`: whether it
/// is permitted, and what it requires of space there. One function for `validate` and `offers`, so a
/// dispatch and an affordance cannot ask different questions (`ARC-63` item 8). With nothing
/// configured: permitted, and [`talk_requirement`].
fn talk_terms(
    world: &WorldRead<'_>,
    place: PlaceId,
    speaker: EntityId,
    listener: EntityId,
) -> (Result<(), Rejection>, SpatialRequirement) {
    let roles = pair(speaker, listener);
    let permitted =
        interactions::permits::<ConversationSystem>(world, place, &Talk::ACTION_TYPE, &roles);
    let reach = interactions::parameters::<ConversationSystem>(world, place, &roles).range;
    (permitted, talk_requirement_within(range(reach)))
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
/// second conversation. Checked against the world's clock and the gap this world's section gives the
/// pair here ([`CONVERSATION_GAP`] when it configures none).
fn continues_a_conversation(
    world: &WorldRead<'_>,
    speaker: PersonId,
    listener: PersonId,
    now: WorldTime,
    gap: SimDuration,
) -> bool {
    [(listener, speaker), (speaker, listener)]
        .into_iter()
        .any(|(holder, other)| {
            world
                .component::<ConversationHistory>(holder.entity_id())
                .and_then(|history| history.last_heard_from(other))
                .is_some_and(|heard| within_the_gap(heard.at(), now, gap))
        })
}

/// Whether `then` is recent enough for a conversation to still be going on at `now`.
fn within_the_gap(then: WorldTime, now: WorldTime, gap: SimDuration) -> bool {
    now.duration_since(then)
        .is_some_and(|elapsed| elapsed <= gap)
}
