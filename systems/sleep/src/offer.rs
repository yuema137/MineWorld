//! What a person is offered and told about sleep (SD-Na-6).

use mineworld_contracts::{ComponentRecord, EntityId, Rejection, SpatialRequirement};
use mineworld_kernel::WorldRead;
use mineworld_presence::Offer;
use serde::Serialize;
use serde_json::Value;

use crate::action::{Sleep, Wake};
use crate::codec;
use crate::component::{Asleep, SleepTerms};
use crate::interactions::terms;
use crate::place::allowed;
use crate::system::{living_person, located};

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

/// What `observer` may attempt, with no target: awake, `sleep` — parametric, `for` is the
/// requester's — refused as `validate` would refuse it here; asleep, a complete `wake` and `sleep`
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
    if world.has_component::<Asleep>(observer) {
        return vec![
            Offer::complete(&Wake {}, SpatialRequirement::NONE).expect("wake encodes"),
            sleep.refused(Rejection::Busy),
        ];
    }
    vec![match located(world, observer) {
        None => sleep.refused(Rejection::PreconditionFailed),
        Some(here) if !allowed(world, observer, here) => sleep.refused(Rejection::PermissionDenied),
        Some(_) => sleep,
    }]
}
