//! What a person is offered and told about sleep (SD-Na-6).

use mineworld_contracts::{ComponentRecord, EntityId, Rejection, SpatialRequirement, WorldTime};
use mineworld_kernel::WorldRead;
use mineworld_presence::Offer;
use serde::Serialize;
use serde_json::Value;

use crate::action::{Sleep, SleepUntil, Wake};
use crate::codec;
use crate::component::{Asleep, SleepTerms};
use crate::interactions::{SleepParameters, terms};
use crate::place::allowed;
use crate::system::{Asked, alarm_within, living_person, located};

/// The `asleep` record as disclosed: whether, and since when; the alarm only to the sleeper. The
/// sleep's Process is never disclosed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct AsleepRecord {
    asleep: bool,
    since: mineworld_contracts::WorldTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    alarm: Option<mineworld_contracts::WorldTime>,
}

/// What `observer` may know about `subject`'s sleep: `asleep` to anyone perception hands the subject
/// to, with the alarm only to the sleeper; `sleep-terms` to oneself, a living Person who is located.
pub(crate) fn disclosed(
    world: &WorldRead<'_>,
    observer: EntityId,
    subject: EntityId,
) -> Vec<ComponentRecord<Value>> {
    let mut records = Vec::new();
    let own = observer == subject;
    if let Some(asleep) = world.component::<Asleep>(subject) {
        let record = AsleepRecord {
            asleep: true,
            since: asleep.since(),
            alarm: own.then(|| asleep.alarm()),
        };
        records.push(ComponentRecord::new::<Asleep>(
            subject,
            codec::to_value(&record),
        ));
    }
    if own
        && living_person(world, subject)
        && let Some(here) = located(world, subject)
    {
        let bounds = terms(world, subject, here);
        records.push(ComponentRecord::new::<SleepTerms>(
            subject,
            codec::to_value(&SleepTerms {
                shortest_minutes: bounds.shortest_minutes,
                longest_minutes: bounds.longest_minutes,
            }),
        ));
    }
    records
}

/// Seconds in a local day. Every multiple of it from instant 0 is a local midnight — `schedule`'s
/// convention, restated as this pack's own constant as `calendar` restates it (`INV-TW-4`), so the pack
/// gains no dependency for one number.
const DAY: i64 = 86_400;

/// What `observer` may attempt, with no target: awake, `sleep` — parametric, `for` is the
/// requester's — and `sleep-until` — complete, its `at` the next morning ([`morning_after`]) — each
/// refused as `validate` would refuse it here; asleep, a complete `wake`, and `sleep` and `sleep-until`
/// refused `Busy`. With a target, or for anything but a living Person, nothing.
pub(crate) fn offered(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if target.is_some() || !living_person(world, observer) {
        return Vec::new();
    }
    let sleep = Offer::new::<Sleep>(SpatialRequirement::NONE);
    let until = Offer::new::<SleepUntil>(SpatialRequirement::NONE);
    if world.has_component::<Asleep>(observer) {
        return vec![
            Offer::complete(&Wake {}, SpatialRequirement::NONE).expect("wake encodes"),
            sleep.refused(Rejection::Busy),
            until.refused(Rejection::Busy),
        ];
    }
    let Some(here) = located(world, observer) else {
        return vec![
            sleep.refused(Rejection::PreconditionFailed),
            until.refused(Rejection::PreconditionFailed),
        ];
    };
    let permitted = allowed(world, observer, here);
    let bounds = terms(world, observer, here);
    let at = morning_after(world.at(), &bounds);
    // validate's order: the bounds (5) before the place (6).
    let until = match alarm_within(world.at(), Asked::Until(at), &bounds) {
        Err(refusal) => until.refused(refusal),
        Ok(_) if !permitted => until.refused(Rejection::PermissionDenied),
        Ok(_) => Offer::complete(&SleepUntil::new(at), SpatialRequirement::NONE)
            .expect("sleep-until encodes"),
    };
    let sleep = if permitted {
        sleep
    } else {
        sleep.refused(Rejection::PermissionDenied)
    };
    vec![sleep, until]
}

/// The instant `sleep-until` is offered for at `now`: the first instant at the morning's time of day
/// (`morning_minute`) strictly more than `shortest_minutes` after `now` (N2-I2). Whether it is also
/// within `longest_minutes` is `validate`'s question, asked of the offer as of a request. Saturates
/// rather than overflows, so an instant at the end of time is refused as out of bound, never wrapped.
pub(crate) fn morning_after(now: WorldTime, bounds: &SleepParameters) -> WorldTime {
    let earliest = now
        .seconds()
        .saturating_add(i64::from(bounds.shortest_minutes) * 60);
    let morning = i64::from(bounds.morning_minute) * 60;
    let candidate = earliest
        .div_euclid(DAY)
        .saturating_mul(DAY)
        .saturating_add(morning);
    let at = if candidate > earliest {
        candidate
    } else {
        candidate.saturating_add(DAY)
    };
    WorldTime::from_seconds(at)
}
