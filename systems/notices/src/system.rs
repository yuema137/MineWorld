//! The installable system: what it declares, how it judges and resolves its three actions, the facts
//! it reduces, and what it offers and discloses.

use std::collections::BTreeSet;

use mineworld_attributes_model::{
    AttributeOwner, AttributeValue, AttributesComposed, AttributesHanded, QualifiedId,
    RunningCatalogue, ValueSlug, disclose_owned,
};
use mineworld_contracts::{
    ActionIntent, ComponentRecord, EntityId, EntityType, Event, EventEnvelope, EventTypeId,
    LifecycleState, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, ProcessStart, System, SystemDeclaration, SystemIdentity,
    SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::action::{PublishNotice, REQUIREMENT, Subscribe, Unsubscribe};
use crate::board::{BoardState, Notice, NoticesProcess, board, board_process};
use crate::codec;
use crate::component::{DisclosedSubscriptions, NoticeBoardRecord, Subscriptions};
use crate::event::{
    NoticePublished, NoticesConfigured, Origin, Subscribed, SubscriptionsSeeded, Unsubscribed,
};
use crate::judge::{
    Change, code, judge_actor, judge_publish, judge_subscription, publishes_any, validity,
};

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): none. Reading the
/// paper is not an event in a life.
pub const BIOGRAPHICAL: &[EventTypeId] = &[];

/// Public, time-limited notices on declared topics, delivered to subscribers.
#[derive(Default)]
pub struct NoticesSystem;

impl SystemIdentity for NoticesSystem {
    const ID: SystemId = SystemId::from_static("notices");
}

/// What the build needs to know about this pack beyond [`System`] (`ARC-33`): it takes
/// `configure/notices.yaml` (`ARC-61`), it owns the `notices:` section of a person's file, and none of
/// its facts is biographical.
impl SystemPack for NoticesSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::configures!();
    mineworld_sdk::owns_section!();
    mineworld_sdk::ships_data!("attributes/subscriptions.yaml");
}

/// The attribute this pack owns (step-30 §9.6 N-2).
pub const SUBSCRIPTIONS_ATTRIBUTE: &str = "notices.subscriptions";

/// The refusal of a second source of initial subscriptions in a world whose subscriptions are the
/// `notices.subscriptions` attribute (S31-b2 §5.4, Q-S31b2-2).
fn subscriptions_are_attributes(event_type: EventTypeId, detail: &str) -> KernelError {
    refused(
        event_type,
        Rejection::System {
            code: RejectionCode::from_static("subscriptions-are-attributes"),
            detail: Some(detail.to_owned()),
        },
    )
}

/// `notices.subscriptions` (S31-b2 §5.3): projected from the person's [`Subscriptions`], sorted;
/// absent when the person has none. Exists only where the world enables `attributes`.
impl AttributeOwner for NoticesSystem {
    fn project(
        world: &WorldRead<'_>,
        person: EntityId,
        attribute: &QualifiedId,
    ) -> Option<AttributeValue> {
        if attribute.as_str() != SUBSCRIPTIONS_ATTRIBUTE {
            return None;
        }
        let held = world.component::<Subscriptions>(person)?;
        held.topics()
            .iter()
            .map(|topic| ValueSlug::new(topic.as_str()).ok())
            .collect::<Option<BTreeSet<ValueSlug>>>()
            .map(AttributeValue::Set)
    }
}

/// Whether the running world's catalogue declares `notices.subscriptions`.
fn subscriptions_are_declared(read: &WorldRead<'_>) -> bool {
    RunningCatalogue::in_world(read).is_some_and(|running| {
        running
            .catalogue()
            .attributes()
            .any(|declaration| declaration.id.as_str() == SUBSCRIPTIONS_ATTRIBUTE)
    })
}

/// A fact this pack may not take: a defect in whoever stated it past the rule (`ARC-26`).
fn refused(event_type: EventTypeId, reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: NoticesSystem::ID,
        event_type,
        reason,
    }
}

/// The board's state as a reduction needs it, or the refusal naming the fact.
fn board_for(read: &WorldRead<'_>, kind: &EventTypeId) -> Result<BoardState, KernelError> {
    board(read).ok_or_else(|| refused(kind.clone(), code("notices-unconfigured")))
}

/// Every living Person subscribed to `topic` now, in id order.
fn subscribers(read: &WorldRead<'_>, topic: &crate::TopicId) -> BTreeSet<EntityId> {
    read.components::<Subscriptions>()
        .filter(|(_, subscriptions)| subscriptions.contains(topic))
        .filter(|(entity, _)| {
            read.entity(*entity).is_some_and(|record| {
                record.entity_type() == EntityType::Person
                    && record.lifecycle() != LifecycleState::Destroyed
            })
        })
        .map(|(entity, _)| entity)
        .collect()
}

impl System for NoticesSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency: `notices` reads no other System's state. The audience of a notice is honoured by
    /// presence's audience rule (`ARC-43`), which every perceived stream already runs.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Subscriptions>()
            .owning::<NoticeBoardRecord>()
            .providing::<PublishNotice>()
            .providing::<Subscribe>()
            .providing::<Unsubscribe>()
            .emitting::<NoticesConfigured>()
            .emitting::<SubscriptionsSeeded>()
            .emitting::<Subscribed>()
            .emitting::<Unsubscribed>()
            .emitting::<NoticePublished>()
            .subscribing_to::<NoticesConfigured>()
            .subscribing_to::<SubscriptionsSeeded>()
            .subscribing_to::<Subscribed>()
            .subscribing_to::<Unsubscribed>()
            .subscribing_to::<NoticePublished>()
            // The owner interface (S31-b2 §§5.3-5.4, ARC-124): subscribing needs no dependency
            // (ARC-28 item 2); in a world without `attributes` neither fact is ever stated.
            .subscribing_to::<AttributesComposed>()
            .subscribing_to::<AttributesHanded>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Subscriptions>()?;
        tables.component::<NoticeBoardRecord>()
    }

    /// The payload decodes (`PreconditionFailed` otherwise), there is no target
    /// (`NoSupportedInteraction`), then the action's own judge (`src/judge.rs`).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let actor = intent.actor();
        let kind = intent.action_type();
        if *kind == <PublishNotice as mineworld_contracts::Action>::ACTION_TYPE {
            let request: PublishNotice = codec::read_action(intent)?;
            no_target(intent)?;
            judge_publish(world, actor, &request).map(|_| ())
        } else if *kind == <Subscribe as mineworld_contracts::Action>::ACTION_TYPE {
            let request: Subscribe = codec::read_action(intent)?;
            no_target(intent)?;
            judge_subscription(world, actor, Change::Subscribe, request.topic()).map(|_| ())
        } else {
            let request: Unsubscribe = codec::read_action(intent)?;
            no_target(intent)?;
            judge_subscription(world, actor, Change::Unsubscribe, request.topic()).map(|_| ())
        }
    }

    /// States the one fact the action causes. A notice's audience is read from the `Subscriptions`
    /// components **now**, never at reduction (I-b-3).
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let read = world.read();
        let actor = intent.actor();
        let kind = intent.action_type();
        if *kind == <PublishNotice as mineworld_contracts::Action>::ACTION_TYPE {
            let event = NoticePublished::EVENT_TYPE;
            let request: PublishNotice =
                codec::read_action(intent).map_err(|reason| refused(event.clone(), reason))?;
            let (publisher, state) = judge_publish(&read, actor, &request)
                .map_err(|reason| refused(event.clone(), reason))?;
            let (issued_at, valid_until) = validity(read.at(), request.valid_for());
            let notice = Notice::new(
                state.next_id(),
                request.topic().clone(),
                publisher,
                request.body().clone(),
                issued_at,
                valid_until,
            );
            let audience = subscribers(&read, request.topic());
            return Ok(vec![NoticePublished::new(notice).emission(audience)]);
        }
        if *kind == <Subscribe as mineworld_contracts::Action>::ACTION_TYPE {
            let event = Subscribed::EVENT_TYPE;
            let request: Subscribe =
                codec::read_action(intent).map_err(|reason| refused(event.clone(), reason))?;
            let person = judge_subscription(&read, actor, Change::Subscribe, request.topic())
                .map_err(|reason| refused(event.clone(), reason))?;
            return Ok(vec![
                Subscribed::new(person, request.topic().clone()).emission(),
            ]);
        }
        let event = Unsubscribed::EVENT_TYPE;
        let request: Unsubscribe =
            codec::read_action(intent).map_err(|reason| refused(event.clone(), reason))?;
        let person = judge_subscription(&read, actor, Change::Unsubscribe, request.topic())
            .map_err(|reason| refused(event.clone(), reason))?;
        Ok(vec![
            Unsubscribed::new(person, request.topic().clone()).emission(),
        ])
    }

    /// The only writes of [`Subscriptions`] and of the board's state (`I-S31-2`):
    ///
    /// - `notices-configured` starts the board, once (`notices-configured-twice` otherwise);
    /// - `subscriptions-seeded` sets a person's first subscriptions: `default` only when it has none,
    ///   `authored` replacing them; every topic must be declared (`unknown-topic`), and the world must
    ///   be configured (`notices-unconfigured`);
    /// - `subscribed` / `unsubscribed` add or drop one declared topic, which must change the set;
    /// - `notice-published` becomes the newest notice of its (topic, publisher), with the next number.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type().clone();
        if kind == NoticesConfigured::EVENT_TYPE {
            let configured: NoticesConfigured = codec::event_payload(event.payload())?;
            if board_process(&world.read()).is_some() {
                return Err(refused(kind, code("notices-configured-twice")));
            }
            world.start_process(
                ProcessStart::<NoticesProcess>::new(codec::encode(&BoardState::configured(
                    configured.topics().to_vec(),
                )))
                .uninterruptible(),
            )?;
        } else if kind == AttributesComposed::EVENT_TYPE {
            // Genesis reduces configuration facts before context facts (A-10), so a person holding
            // subscriptions now got them from `defaults.by_tag`: a second source (§5.4). The payload
            // is read, not the running catalogue, which `attributes` may start after this reaction.
            let composed: AttributesComposed = codec::event_payload(event.payload())?;
            let declared = composed
                .catalogue
                .attributes
                .iter()
                .any(|declaration| declaration.id.as_str() == SUBSCRIPTIONS_ATTRIBUTE);
            if declared && world.read().components::<Subscriptions>().next().is_some() {
                return Err(subscriptions_are_attributes(
                    kind,
                    "use roles in a population profile instead of defaults.by_tag",
                ));
            }
        } else if kind == AttributesHanded::EVENT_TYPE {
            return handed(event);
        } else if kind == SubscriptionsSeeded::EVENT_TYPE {
            let seeded: SubscriptionsSeeded = codec::event_payload(event.payload())?;
            let read = world.read();
            if seeded.origin() == Origin::Authored && subscriptions_are_declared(&read) {
                return Err(subscriptions_are_attributes(
                    kind,
                    "write attributes: { notices.subscriptions: [...] }",
                ));
            }
            let state = board_for(&read, &kind)?;
            if seeded
                .topics()
                .iter()
                .any(|topic| state.topic(topic).is_none())
            {
                return Err(refused(kind, code("unknown-topic")));
            }
            let person = seeded.person().entity_id();
            let has = read.component::<Subscriptions>(person).is_some();
            if seeded.origin() == Origin::Authored || !has {
                world.insert(person, Subscriptions::of(seeded.topics().iter().cloned()))?;
            }
        } else if kind == Subscribed::EVENT_TYPE || kind == Unsubscribed::EVENT_TYPE {
            let read = world.read();
            let state = board_for(&read, &kind)?;
            let (person, topic, adding) = if kind == Subscribed::EVENT_TYPE {
                let fact: Subscribed = codec::event_payload(event.payload())?;
                (fact.person().entity_id(), fact.topic().clone(), true)
            } else {
                let fact: Unsubscribed = codec::event_payload(event.payload())?;
                (fact.person().entity_id(), fact.topic().clone(), false)
            };
            if state.topic(&topic).is_none() {
                return Err(refused(kind, code("unknown-topic")));
            }
            let current = read
                .component::<Subscriptions>(person)
                .cloned()
                .unwrap_or_default();
            let next = match (adding, current.contains(&topic)) {
                (true, false) => current.with(topic),
                (false, true) => current.without(&topic),
                (true, true) => return Err(refused(kind, code("already-subscribed"))),
                (false, false) => return Err(refused(kind, code("not-subscribed"))),
            };
            world.insert(person, next)?;
        } else if kind == NoticePublished::EVENT_TYPE {
            let published: NoticePublished = codec::event_payload(event.payload())?;
            let (id, state) = {
                let read = world.read();
                let Some(process) = board_process(&read) else {
                    return Err(refused(kind, code("notices-unconfigured")));
                };
                (process.id(), board_for(&read, &kind)?)
            };
            if published.notice().id() != state.next_id() {
                return Err(refused(kind, Rejection::PreconditionFailed));
            }
            let next = state.posted(published.notice().clone());
            world.set_process_state::<NoticesProcess>(id, codec::encode(&next))?;
        }
        Ok(Vec::new())
    }
}

/// A person's resolved `notices.subscriptions`, handed by `attributes` at genesis (S31-b2 §5.3): stated
/// as the same `subscriptions-seeded` fact a configuration default is, `origin: default` (SD-b2-2), so
/// the existing reduction writes [`Subscriptions`] and checks every topic. Another owner's values are
/// not this pack's; a value that is not a set is refused `PreconditionFailed`.
fn handed(event: &EventEnvelope) -> Result<Vec<Emission>, KernelError> {
    let kind = event.event_type().clone();
    let handed: AttributesHanded = codec::event_payload(event.payload())?;
    if handed.owner != NoticesSystem::ID {
        return Ok(Vec::new());
    }
    let Some((_, value)) = handed
        .values
        .iter()
        .find(|(id, _)| id.as_str() == SUBSCRIPTIONS_ATTRIBUTE)
    else {
        return Ok(Vec::new());
    };
    let AttributeValue::Set(slugs) = value else {
        return Err(refused(kind, Rejection::PreconditionFailed));
    };
    let topics = slugs
        .iter()
        .map(|slug| crate::TopicId::new(slug.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| refused(kind, code("unknown-topic")))?;
    Ok(vec![
        SubscriptionsSeeded::new(handed.person, topics, Origin::Default).emission(),
    ])
}

/// None of this pack's actions has a target.
fn no_target(intent: &ActionIntent) -> Result<(), Rejection> {
    if intent.target().is_some() {
        Err(Rejection::NoSupportedInteraction)
    } else {
        Ok(())
    }
}

impl PerceptionProvider for NoticesSystem {
    /// With no target, to a living Person in a configured world, each offer **incomplete** and once
    /// (Q-S31a-3), so a generic controller never posts or subscribes at random, in this order:
    /// `publish-notice` to a person who publishes some topic; `subscribe` while some declared topic is
    /// not in its set; `unsubscribe` while its set is not empty.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        if target.is_some() {
            return Vec::new();
        }
        let Ok((person, state)) = judge_actor(world, observer) else {
            return Vec::new();
        };
        let held = world.component::<Subscriptions>(observer);
        let mut offers = Vec::new();
        if publishes_any(&state, person) {
            offers.push(Offer::new::<PublishNotice>(REQUIREMENT));
        }
        if state
            .topics()
            .iter()
            .any(|topic| !held.is_some_and(|held| held.contains(topic.topic())))
        {
            offers.push(Offer::new::<Subscribe>(REQUIREMENT));
        }
        if held.is_some_and(|held| !held.is_empty()) {
            offers.push(Offer::new::<Unsubscribe>(REQUIREMENT));
        }
        offers
    }

    /// To the person themself only (I-S31-9): its `subscriptions`, and its `notice-board` — the
    /// notices current at this read's instant on the topics it subscribes to or publishes, absent
    /// when there are none. Nothing to any other observer.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if observer != subject {
            return Vec::new();
        }
        let mut disclosed = Vec::new();
        let held = world.component::<Subscriptions>(subject);
        if let Some(held) = held {
            let record = DisclosedSubscriptions {
                topics: held.topics().to_vec(),
                attributes: disclose_owned::<Self>(world, observer, subject),
            };
            disclosed.push(ComponentRecord::new::<Subscriptions>(
                subject,
                codec::to_value(&record),
            ));
        }
        let Ok((person, state)) = judge_actor(world, subject) else {
            return disclosed;
        };
        let notices: Vec<Notice> = state
            .current(world.at())
            .filter(|notice| {
                held.is_some_and(|held| held.contains(notice.topic()))
                    || state
                        .topic(notice.topic())
                        .is_some_and(|topic| topic.publishers().contains(&person))
            })
            .cloned()
            .collect();
        if !notices.is_empty() {
            disclosed.push(ComponentRecord::new::<NoticeBoardRecord>(
                subject,
                codec::to_value(&NoticeBoardRecord { notices }),
            ));
        }
        disclosed
    }
}
