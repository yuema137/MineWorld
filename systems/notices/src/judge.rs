//! The one decision `offers` and `validate` both call (I-b-7), so what a person is offered and what a
//! dispatch admits cannot disagree.

use mineworld_contracts::{
    EntityId, EntityType, LifecycleState, PersonId, Rejection, RejectionCode, WorldTime,
};
use mineworld_kernel::WorldRead;

use crate::action::PublishNotice;
use crate::board::{BoardState, board};
use crate::body::judge_body;
use crate::component::Subscriptions;
use crate::event::ConfiguredTopic;
use crate::topic::TopicId;

/// A refusal of this pack's own, by code.
pub(crate) fn code(code: &'static str) -> Rejection {
    Rejection::System {
        code: RejectionCode::from_static(code),
        detail: None,
    }
}

/// The actor as a living Person, in a world whose notices are configured — else
/// `NoSupportedInteraction`: there is nothing of this pack to do.
pub(crate) fn judge_actor(
    world: &WorldRead<'_>,
    actor: EntityId,
) -> Result<(PersonId, BoardState), Rejection> {
    let living_person = world.entity(actor).is_some_and(|record| {
        record.entity_type() == EntityType::Person
            && record.lifecycle() != LifecycleState::Destroyed
    });
    if !living_person {
        return Err(Rejection::NoSupportedInteraction);
    }
    let person =
        PersonId::new(actor, EntityType::Person).map_err(|_| Rejection::NoSupportedInteraction)?;
    let state = board(world).ok_or(Rejection::NoSupportedInteraction)?;
    Ok((person, state))
}

/// The topic, if the board declares it — else `unknown-topic`.
fn declared<'s>(state: &'s BoardState, topic: &TopicId) -> Result<&'s ConfiguredTopic, Rejection> {
    state.topic(topic).ok_or_else(|| code("unknown-topic"))
}

/// Whether `actor` may post `request` now, in the fixed order of `pr-s31b-notices.md` §5.7:
/// NotAPerson, UnknownTopic, NotAPublisher, BodyMismatch, BodyTooLarge, BodyInvalid,
/// ValidityOutOfRange, OverDailyLimit. Two reasons at once always report the first.
///
/// # Errors
///
/// The first rule broken.
pub fn judge_publish(
    world: &WorldRead<'_>,
    actor: EntityId,
    request: &PublishNotice,
) -> Result<(PersonId, BoardState), Rejection> {
    let (person, state) = judge_actor(world, actor)?;
    let topic = declared(&state, request.topic())?;
    if !topic.publishers().contains(&person) {
        return Err(Rejection::PermissionDenied);
    }
    judge_body(world, topic.body(), request.body())?;
    if request.valid_for() < topic.valid_min() || request.valid_for() > topic.valid_max() {
        return Err(code("validity-out-of-range"));
    }
    if state.issued_within(request.topic(), person, world.at()) >= usize::from(topic.per_day()) {
        return Err(code("over-daily-limit"));
    }
    Ok((person, state))
}

/// What a subscription request asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Add the topic.
    Subscribe,
    /// Drop the topic.
    Unsubscribe,
}

/// Whether `actor` may make `change` to `topic` now: a living Person in a configured world
/// (`NoSupportedInteraction`), a declared topic (`unknown-topic`), and a change that changes
/// something (`already-subscribed`, `not-subscribed`).
///
/// # Errors
///
/// The first rule broken.
pub fn judge_subscription(
    world: &WorldRead<'_>,
    actor: EntityId,
    change: Change,
    topic: &TopicId,
) -> Result<PersonId, Rejection> {
    let (person, state) = judge_actor(world, actor)?;
    declared(&state, topic)?;
    let held = world
        .component::<Subscriptions>(actor)
        .is_some_and(|subscriptions| subscriptions.contains(topic));
    match (change, held) {
        (Change::Subscribe, true) => Err(code("already-subscribed")),
        (Change::Unsubscribe, false) => Err(code("not-subscribed")),
        _ => Ok(person),
    }
}

/// Whether `person` publishes at least one topic.
pub(crate) fn publishes_any(state: &BoardState, person: PersonId) -> bool {
    state
        .topics()
        .iter()
        .any(|topic| topic.publishers().contains(&person))
}

/// The issue time and the end of validity of a notice posted at `now` for `valid_for` seconds.
pub(crate) fn validity(now: WorldTime, valid_for: u32) -> (WorldTime, WorldTime) {
    (
        now,
        WorldTime::from_seconds(now.seconds() + i64::from(valid_for)),
    )
}
