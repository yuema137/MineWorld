//! The installable system: what it declares, how it decides a sleep, the facts it reduces, and the
//! alarm.

use mineworld_contracts::{
    Action, ActionIntent, Causation, ComponentRecord, EntityId, EntityType, Event, EventEnvelope, LifecycleState,
    PersonId, PlaceId, Rejection, RejectionCode, SystemId, Visibility, WorldTime,
};
use mineworld_conversation::Spoke;
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{Arrived, Offer, PerceptionProvider, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions;
use serde_json::Value;

use crate::action::{Sleep, Wake};
use crate::codec;
use crate::component::{Asleep, SleepTerms};
use crate::event::{AlarmSet, FellAsleep, Woke, WokeReason};
use crate::interactions::{audience, terms};
use crate::offer::{disclosed, offered};
use crate::place::allowed;
use crate::process::{SleepProcess, SleepState};

/// Sleeping, and getting up.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SleepSystem;

impl SystemIdentity for SleepSystem {
    const ID: SystemId = SystemId::from_static("sleep");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// section of the World's Interaction List (`ARC-63`), which is its configuration. No fact is
/// biographical by default (step-35 Q-S35-13); a world's `consequences` may switch either on.
impl SystemPack for SleepSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::interactions!();
}

/// The refusal of a `for` outside the bounds in force; its detail names them.
pub const OUT_OF_BOUND: RejectionCode = RejectionCode::from_static("sleep-for-out-of-bound");

impl System for SleepSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on presence, whose positions it reads. Hears presence's `arrived` and conversation's
    /// `spoke` through their published types, with no dependency on conversation (`ARC-28`).
    fn declaration(&self) -> SystemDeclaration {
        interactions::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .depending_on([PresenceSystem::ID])
                .owning::<Asleep>()
                .owning::<SleepTerms>()
                .providing::<Sleep>()
                .providing::<Wake>()
                .emitting::<FellAsleep>()
                .emitting::<AlarmSet>()
                .emitting::<Woke>()
                .subscribing_to::<AlarmSet>()
                .subscribing_to::<Woke>()
                .subscribing_to::<Arrived>()
                .subscribing_to::<Spoke>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        interactions::install::<Self>(tables)?;
        tables.component::<Asleep>()?;
        tables.component::<SleepTerms>()
    }

    /// Whether this request may happen, in order (step-35 §4.2; SD-Na-2).
    ///
    /// `sleep`: 1. the payload decodes — `PreconditionFailed` otherwise; 2. the actor is a living
    /// Person and there is no target — `NoSupportedInteraction`; 3. not already asleep — `Busy`;
    /// 4. located — `PreconditionFailed`; 5. `for` within the bounds in force for the actor there —
    /// [`OUT_OF_BOUND`], naming them; 6. sleep is allowed there (`src/place.rs`) — `PermissionDenied`.
    ///
    /// `wake`: the payload decodes, a living Person with no target, and asleep —
    /// `NoSupportedInteraction` otherwise.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let kind = intent.payload().action_type();
        if *kind == Sleep::ACTION_TYPE {
            let sleep: Sleep = codec::action_payload(intent.payload())
                .map_err(|_| Rejection::PreconditionFailed)?;
            may_sleep(world, intent.actor(), intent.target(), sleep.minutes()).map(|_| ())
        } else if *kind == Wake::ACTION_TYPE {
            let _: Wake = codec::action_payload(intent.payload())
                .map_err(|_| Rejection::PreconditionFailed)?;
            may_wake(world, intent.actor(), intent.target()).map(|_| ())
        } else {
            Err(Rejection::PreconditionFailed)
        }
    }

    /// `sleep` states `fell-asleep` (to the place, or as the list routes it) and `alarm-set` (to the
    /// sleeper alone); `wake` states `woke` with `reason: asked`. Writes nothing: the reducer does.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let read = world.read();
        let actor = intent.actor();
        let unresolvable = || KernelError::ActionNotResolvedBySystem {
            system: Self::ID,
            action_type: intent.action_type().clone(),
        };
        if *intent.payload().action_type() == Sleep::ACTION_TYPE {
            let sleep: Sleep = codec::action_payload(intent.payload())?;
            let here = may_sleep(&read, actor, intent.target(), sleep.minutes())
                .map_err(|_| unresolvable())?;
            let person = PersonId::new(actor, EntityType::Person)?;
            let alarm = WorldTime::from_seconds(
                read.at().seconds() + i64::from(sleep.minutes()) * 60,
            );
            return Ok(vec![
                Emission::new::<FellAsleep>(
                    codec::encode(&FellAsleep::new(person, here)),
                    audience(&read, &FellAsleep::EVENT_TYPE, actor, here),
                )
                .about(vec![actor])
                .with_participants(vec![actor])
                .at_place(here),
                Emission::new::<AlarmSet>(
                    codec::encode(&AlarmSet::new(person, alarm)),
                    Visibility::Entities([actor].into_iter().collect()),
                )
                .about(vec![actor])
                .with_participants(vec![actor])
                .at_place(here),
            ]);
        }
        let asleep = may_wake(&read, actor, intent.target()).map_err(|_| unresolvable())?;
        let person = PersonId::new(actor, EntityType::Person)?;
        Ok(vec![woke(&read, person, &asleep, WokeReason::Asked)])
    }

    /// The only writes of [`Asleep`]: `alarm-set` starts the sleep's Process and inserts it; `woke`
    /// removes it and ends the Process when it is still running (an early wake). The world's section
    /// is reduced first.
    ///
    /// Two other packs' facts end a sleep, each by stating `woke` — never by touching `Asleep`:
    /// conversation's `spoke` wakes an asleep speaker (`acted`) and, when the listener's
    /// `woken_by_talk` holds where they are, an asleep listener (`addressed`), speaker first;
    /// presence's `arrived` — after genesis — wakes an asleep walker (`acted`), however they were
    /// moved (F-Na-4).
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if interactions::reduce(world, event)? {
            return Ok(Vec::new());
        }
        let kind = event.event_type();
        if *kind == AlarmSet::EVENT_TYPE {
            let set: AlarmSet = codec::event_payload(event.payload())?;
            let person = set.person().entity_id();
            let mut start = ProcessStart::<SleepProcess>::new(codec::encode(&SleepState::new(
                set.person(),
            )))
            .with_participants(vec![person])
            .ending_at(set.alarm())
            .uninterruptible();
            if let Some(place) = event.place() {
                start = start.at_place(place);
            }
            let process = world.start_process(start)?;
            world.insert(person, Asleep::new(event.at(), set.alarm(), process))?;
        } else if *kind == Woke::EVENT_TYPE {
            let woke: Woke = codec::event_payload(event.payload())?;
            let person = woke.person().entity_id();
            if let Some(asleep) = world.remove::<Asleep>(person)?
                && world.read().process(asleep.process()).is_some()
            {
                world.end_process::<SleepProcess>(asleep.process())?;
            }
        } else if *kind == Spoke::EVENT_TYPE {
            let spoke: Spoke = codec::event_payload(event.payload())?;
            let read = world.read();
            let listener = spoke.listener();
            let woken_by_talk = located(&read, listener.entity_id()).is_none_or(|place| {
                terms(&read, listener.entity_id(), place).woken_by_talk
            });
            let mut wakes = Wakes::default();
            wakes.add(&read, spoke.speaker(), WokeReason::Acted);
            if woken_by_talk {
                wakes.add(&read, listener, WokeReason::Addressed);
            }
            return Ok(wakes.0);
        } else if *kind == Arrived::EVENT_TYPE && *event.caused_by() != Causation::WorldGenesis {
            let arrived: Arrived = codec::event_payload(event.payload())?;
            let mut wakes = Wakes::default();
            wakes.add(&world.read(), arrived.person(), WokeReason::Acted);
            return Ok(wakes.0);
        }
        Ok(Vec::new())
    }

    /// The alarm: the sleep's Process ends, and the sleeper wakes with `reason: alarm`. A sleeper who
    /// is no longer a living Person, or whose sleep this no longer is, wakes to nothing.
    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let state: SleepState = codec::state(process.state_for::<SleepProcess>()?).ok_or(
            KernelError::ProcessNotRunning {
                process: process.id(),
            },
        )?;
        world.end_process::<SleepProcess>(process.id())?;
        let read = world.read();
        let sleeper = state.sleeper();
        let asleep = read
            .component::<Asleep>(sleeper.entity_id())
            .filter(|asleep| asleep.process() == process.id());
        match asleep {
            Some(asleep) if living_person(&read, sleeper.entity_id()) => {
                Ok(vec![woke(&read, sleeper, asleep, WokeReason::Alarm)])
            }
            _ => Ok(Vec::new()),
        }
    }
}

impl PerceptionProvider for SleepSystem {
    /// With no target, for a living Person: awake, `sleep` (parametric), refused as dispatch would
    /// refuse it here; asleep, a complete `wake` and `sleep` refused `Busy` (`src/offer.rs`).
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        offered(world, observer, target)
    }

    /// `asleep { asleep, since }` to whoever perceives the sleeper, with `alarm` only to the sleeper;
    /// `sleep-terms` to oneself (`src/offer.rs`).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        disclosed(world, observer, subject)
    }
}

/// Whether `entity` is a living Person: the only one who sleeps.
pub(crate) fn living_person(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world.entity(entity).is_some_and(|record| {
        record.entity_type() == EntityType::Person
            && record.lifecycle() != LifecycleState::Destroyed
    })
}

/// Where `entity` is, as presence has been told; [`None`] is unknown.
pub(crate) fn located(world: &WorldRead<'_>, entity: EntityId) -> Option<PlaceId> {
    world
        .component::<Presence>(entity)
        .map(|presence| presence.location().place())
}

/// SD-Na-2's `sleep` checks after decoding, in order; the place to sleep at when they all pass.
pub(crate) fn may_sleep(
    world: &WorldRead<'_>,
    actor: EntityId,
    target: Option<EntityId>,
    minutes: u16,
) -> Result<PlaceId, Rejection> {
    if !living_person(world, actor) || target.is_some() {
        return Err(Rejection::NoSupportedInteraction);
    }
    if world.has_component::<Asleep>(actor) {
        return Err(Rejection::Busy);
    }
    let here = located(world, actor).ok_or(Rejection::PreconditionFailed)?;
    let bounds = terms(world, actor, here);
    if minutes < bounds.shortest_minutes || minutes > bounds.longest_minutes {
        return Err(Rejection::System {
            code: OUT_OF_BOUND,
            detail: Some(format!(
                "for must be {} … {} minutes, not {minutes}",
                bounds.shortest_minutes, bounds.longest_minutes
            )),
        });
    }
    if !allowed(world, actor, here) {
        return Err(Rejection::PermissionDenied);
    }
    Ok(here)
}

/// SD-Na-2's `wake` checks after decoding; the sleep it ends.
pub(crate) fn may_wake(
    world: &WorldRead<'_>,
    actor: EntityId,
    target: Option<EntityId>,
) -> Result<Asleep, Rejection> {
    if !living_person(world, actor) || target.is_some() {
        return Err(Rejection::NoSupportedInteraction);
    }
    world
        .component::<Asleep>(actor)
        .copied()
        .ok_or(Rejection::NoSupportedInteraction)
}

/// The `woke` facts one reaction states: at most one per person, and only for a person asleep, so two
/// triggers in one reaction never wake one sleeper twice (SD-Na-4).
#[derive(Default)]
struct Wakes(Vec<Emission>, Vec<PersonId>);

impl Wakes {
    fn add(&mut self, read: &WorldRead<'_>, person: PersonId, reason: WokeReason) {
        if self.1.contains(&person) {
            return;
        }
        if let Some(asleep) = read.component::<Asleep>(person.entity_id()) {
            self.0.push(woke(read, person, asleep, reason));
            self.1.push(person);
        }
    }
}

/// `woke` for `person`, ending `asleep` now, stated to the place they are in at this instant (as the
/// list routes it), or to them alone when they are nowhere known.
pub(crate) fn woke(
    read: &WorldRead<'_>,
    person: PersonId,
    asleep: &Asleep,
    reason: WokeReason,
) -> Emission {
    let who = person.entity_id();
    let slept = u64::try_from(read.at().seconds() - asleep.since().seconds()).unwrap_or(0);
    let payload = codec::encode(&Woke::new(person, slept, reason));
    let emission = |visibility| {
        Emission::new::<Woke>(payload.clone(), visibility)
            .about(vec![who])
            .with_participants(vec![who])
    };
    match located(read, who) {
        Some(place) => {
            emission(audience(read, &Woke::EVENT_TYPE, who, place)).at_place(place)
        }
        None => emission(Visibility::Entities([who].into_iter().collect())),
    }
}
