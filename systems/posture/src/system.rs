//! The installable system: what it declares, how it decides, and how it reduces.

use mineworld_contracts::{
    Action, ActionIntent, EntityId, Event, EventEnvelope, EventTypeId, LocalPosition, Location,
    Orientation, PersonId, PlaceId, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_movement::{MovementSystem, WalkStarted, is_walking};
use mineworld_presence::{Arrived, Presence, PresenceSystem, arrival};
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions;

use crate::action::{Sit, Stand};
use crate::codec;
use crate::component::Posture;
use crate::event::{PlaceSitPoints, SatDown, StandCause, StoodUp};
use crate::interactions::{audience, permits_sit, reach};
use crate::point::{SIT_POINT_SPACING, SitPoint, SitPoints};

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): none. Sitting down is
/// not a life event, and the Interaction List cannot make it one (`ARC-147`).
pub const BIOGRAPHICAL: &[EventTypeId] = &[];

/// This pack's own reason for refusing a request it cannot read, the same code as the other packs'.
const MALFORMED_PAYLOAD: RejectionCode = RejectionCode::from_static("malformed-payload");

/// Where people may sit, and who is seated.
///
/// A unit struct: its state is the components it owns, held in the world (`INV-7`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PostureSystem;

impl SystemIdentity for PostureSystem {
    const ID: SystemId = SystemId::from_static("posture");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts (none), the `sit-points:` section of a place's file, which its `AuthoredSection`
/// impl (`src/section.rs`) describes, and its section of the World's Interaction List
/// (`src/interactions.rs`).
impl SystemPack for PostureSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
    mineworld_sdk::interactions!();
}

impl System for PostureSystem {
    /// Version 1: the first release of the pack (`ARC-25`).
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        interactions::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .depending_on([PresenceSystem::ID, MovementSystem::ID])
                .owning::<SitPoints>()
                .owning::<Posture>()
                .providing::<Sit>()
                .providing::<Stand>()
                .emitting::<PlaceSitPoints>()
                .emitting::<SatDown>()
                .emitting::<StoodUp>()
                .emitting::<Arrived>()
                .subscribing_to::<PlaceSitPoints>()
                .subscribing_to::<SatDown>()
                .subscribing_to::<StoodUp>()
                .subscribing_to::<Arrived>()
                .subscribing_to::<WalkStarted>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        interactions::install::<Self>(tables)?;
        tables.component::<SitPoints>()?;
        tables.component::<Posture>()
    }

    /// Decides whether a request is admissible, reading state and writing nothing, in the step's order
    /// (`step-34-posture.md` §6.1; SD-a-3, SD-a-4). The offer asks the same function, so the offer and
    /// the dispatch answer the same reason first (I-a-1).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let actor = person(world, intent.actor()).ok_or(Rejection::PreconditionFailed)?;
        let here = located(world, intent.actor()).ok_or(Rejection::PreconditionFailed)?;
        let action = intent.action_type();
        if *action == Stand::ACTION_TYPE {
            readable::<Stand>(intent)?;
            return seated(world, actor)
                .map(|_| ())
                .ok_or(Rejection::PreconditionFailed);
        }
        if *action == Sit::ACTION_TYPE {
            let sit = readable::<Sit>(intent)?;
            return sit_verdict(world, actor, &here, sit.point()).map(|_| ());
        }
        // Routing sends this system only the actions it declared; anything else is not one of them.
        Err(Rejection::NoSupportedInteraction)
    }

    /// States what happened. Writes nothing itself: the person moves through presence's own `arrived`,
    /// and the row is written while reducing `sat-down` and `stood-up` (I-S34-3).
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let unresolvable = || KernelError::ActionNotResolvedBySystem {
            system: Self::ID,
            action_type: intent.action_type().clone(),
        };
        let read = world.read();
        let actor = person(&read, intent.actor()).ok_or_else(unresolvable)?;
        let here = located(&read, intent.actor()).ok_or_else(unresolvable)?;
        let action = intent.action_type();
        if *action == Stand::ACTION_TYPE {
            let row = seated(&read, actor).ok_or_else(unresolvable)?;
            return Ok(vec![stood_up(&read, actor, row, StandCause::Requested)]);
        }
        if *action != Sit::ACTION_TYPE {
            return Err(unresolvable());
        }
        let sit: Sit = codec::action_payload(intent.payload())?;
        let point = sit_verdict(&read, actor, &here, sit.point()).map_err(|reason| {
            KernelError::FactRefusedByOwner {
                system: Self::ID,
                event_type: SatDown::EVENT_TYPE,
                reason,
            }
        })?;
        let place = here.place();
        let target = seat_location(place, &point);
        let mut facts = Vec::new();
        if !already_there(&here, &point) {
            facts.push(arrival(&read, actor, target).map_err(|reason| {
                KernelError::FactRefusedByOwner {
                    system: Self::ID,
                    event_type: Arrived::EVENT_TYPE,
                    reason,
                }
            })?);
        }
        let heard_by = audience(&read, place, &SatDown::EVENT_TYPE, actor.entity_id());
        facts.push(
            SatDown::new(
                actor,
                place,
                point.key().clone(),
                point.at(),
                Orientation::facing(point.facing()),
            )
            .emission(heard_by),
        );
        Ok(facts)
    }

    /// Reduces this pack's own facts into the components it owns, and stands a seated person up when
    /// presence says they are somewhere else, or movement says they set off (Q-S34-4, SD-a-6).
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if interactions::reduce(world, event)? {
            return Ok(Vec::new());
        }
        let kind = event.event_type();
        if *kind == PlaceSitPoints::EVENT_TYPE {
            // The list was checked when it was decoded, so there is nothing to refuse here.
            let stated: PlaceSitPoints = codec::event_payload(event.payload())?;
            world.insert(stated.place().entity_id(), stated.points().clone())?;
        } else if *kind == SatDown::EVENT_TYPE {
            let sat: SatDown = codec::event_payload(event.payload())?;
            world.insert(
                sat.person().entity_id(),
                Posture::seated(sat.place(), sat.point().clone(), event.at()),
            )?;
        } else if *kind == StoodUp::EVENT_TYPE {
            let stood: StoodUp = codec::event_payload(event.payload())?;
            if seated(&world.read(), stood.person()).is_some() {
                world.remove::<Posture>(stood.person().entity_id())?;
            }
        } else if *kind == Arrived::EVENT_TYPE {
            // Read from the fact's payload, not from `Presence`, so the answer does not depend on
            // whether presence reduced the arrival first.
            let arrived: Arrived = codec::event_payload(event.payload())?;
            let read = world.read();
            if let Some(row) = seated(&read, arrived.person())
                && !on_point(&read, row, &arrived.location())
            {
                return Ok(vec![stood_up(
                    &read,
                    arrived.person(),
                    row,
                    StandCause::Moved,
                )]);
            }
        } else if *kind == WalkStarted::EVENT_TYPE {
            let started: WalkStarted = codec::event_payload(event.payload())?;
            let read = world.read();
            if let Some(row) = seated(&read, started.person()) {
                return Ok(vec![stood_up(
                    &read,
                    started.person(),
                    row,
                    StandCause::Moved,
                )]);
            }
        }
        Ok(Vec::new())
    }
}

/// The verdict on `actor` sitting on `key` from `here`, in the step's order (SD-a-3), and the point
/// when it is admissible. One function for `validate`, `resolve` and the offer (I-a-1).
///
/// ```text
/// 1  the list forbids it                          PermissionDenied
/// 2  no such point in the actor's place           NoSupportedInteraction
/// 3  already seated, or walking                   Busy
/// 4  no position, or farther than reach from at   TooFarAway
/// 5  somebody seated on it                        TargetUnavailable
/// 6  a standing person within the spacing of at   TargetUnavailable
/// 7  presence would refuse the arrival            TargetUnavailable   (BC-3)
/// ```
pub(crate) fn sit_verdict(
    world: &WorldRead<'_>,
    actor: PersonId,
    here: &Location,
    key: &crate::SitPointKey,
) -> Result<SitPoint, Rejection> {
    let place = here.place();
    permits_sit(world, place, actor.entity_id())?;
    let point = world
        .component::<SitPoints>(place.entity_id())
        .and_then(|points| points.get(key))
        .cloned()
        .ok_or(Rejection::NoSupportedInteraction)?;
    if seated(world, actor).is_some() || is_walking(world, actor) {
        return Err(Rejection::Busy);
    }
    let local = here.local().ok_or(Rejection::TooFarAway)?;
    if !within(local, point.at(), reach(world, place, actor.entity_id())) {
        return Err(Rejection::TooFarAway);
    }
    let occupied = world.components::<Posture>().any(|(other, row)| {
        other != actor.entity_id() && row.place() == place && row.point() == point.key()
    });
    if occupied {
        return Err(Rejection::TargetUnavailable);
    }
    let crowded = world.components::<Presence>().any(|(other, presence)| {
        let there = presence.location();
        other != actor.entity_id()
            && there.place() == place
            && world.component::<Posture>(other).is_none()
            && there.local().is_some_and(|spot| {
                distance_squared(spot, point.at()) < squared(SIT_POINT_SPACING.value())
            })
    });
    if crowded {
        return Err(Rejection::TargetUnavailable);
    }
    if !already_there(here, &point) {
        arrival(world, actor, seat_location(place, &point))
            .map_err(|_| Rejection::TargetUnavailable)?;
    }
    Ok(point)
}

/// Where a person seated on `point` of `place` is: its position, facing its facing.
fn seat_location(place: PlaceId, point: &SitPoint) -> Location {
    Location::in_place(place)
        .with_local(point.at())
        .with_facing(Orientation::facing(point.facing()))
}

/// Whether `here` is already exactly the seat: the same position and the same facing, so sitting needs
/// no arrival (SD-a-5).
fn already_there(here: &Location, point: &SitPoint) -> bool {
    here.local() == Some(point.at()) && here.facing() == Some(Orientation::facing(point.facing()))
}

/// Whether `location` is still on the seated row's point: the same place and the same position. Facing
/// may change while seated (BC-2).
fn on_point(world: &WorldRead<'_>, row: &Posture, location: &Location) -> bool {
    location.place() == row.place()
        && world
            .component::<SitPoints>(row.place().entity_id())
            .and_then(|points| points.get(row.point()))
            .is_some_and(|point| location.local() == Some(point.at()))
}

/// `stood-up` for the seated `row`, heard as the list says at the row's place.
fn stood_up(world: &WorldRead<'_>, person: PersonId, row: &Posture, cause: StandCause) -> Emission {
    let heard_by = audience(world, row.place(), &StoodUp::EVENT_TYPE, person.entity_id());
    StoodUp::new(person, row.place(), row.point().clone(), cause).emission(heard_by)
}

fn squared(millimetres: i32) -> i64 {
    i64::from(millimetres) * i64::from(millimetres)
}

fn distance_squared(a: LocalPosition, b: LocalPosition) -> i64 {
    let dx = i64::from(a.x().value()) - i64::from(b.x().value());
    let dy = i64::from(a.y().value()) - i64::from(b.y().value());
    dx * dx + dy * dy
}

/// Whether `a` is within `range` of `b` on the floor, inclusive.
fn within(a: LocalPosition, b: LocalPosition, range: mineworld_contracts::Millimetres) -> bool {
    distance_squared(a, b) <= squared(range.value())
}

/// Reads an action payload as `A`, or refuses the request as malformed.
fn readable<A: Action>(intent: &ActionIntent) -> Result<A, Rejection> {
    codec::action_payload::<A>(intent.payload()).map_err(|error| Rejection::System {
        code: MALFORMED_PAYLOAD,
        detail: Some(error.to_string()),
    })
}

/// That entity as a person, if this world holds a person by that identity.
pub(crate) fn person(world: &WorldRead<'_>, entity: EntityId) -> Option<PersonId> {
    let record = world.entity(entity)?;
    PersonId::new(record.id(), record.entity_type()).ok()
}

/// Where presence says that entity is; [`None`] is *unknown*, not *nowhere*.
pub(crate) fn located(world: &WorldRead<'_>, entity: EntityId) -> Option<Location> {
    world.component::<Presence>(entity).map(Presence::location)
}

/// The person's seated row, if they are seated.
pub(crate) fn seated<'a>(world: &WorldRead<'a>, person: PersonId) -> Option<&'a Posture> {
    world.component::<Posture>(person.entity_id())
}
