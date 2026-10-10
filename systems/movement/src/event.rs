//! The facts this pack owns: two places open onto each other; a walk started; a walk ended.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, LocalPosition, PersonId, PlaceId, SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::action::Destination;
use crate::codec;
use crate::system::MovementSystem;

/// A person set out on a walk (`DECISIONS.md` `ARC-73`): stated when `walk-to` is resolved, before
/// any stride.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkStarted {
    person: PersonId,
    destination: Destination,
}

impl Event for WalkStarted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("walk-started");
    const OWNER: SystemId = MovementSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl WalkStarted {
    /// Who set out.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where to.
    pub const fn destination(&self) -> Destination {
        self.destination
    }
}

/// How a walk ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ended {
    /// The walker got there.
    Arrived,
    /// The walker made no progress for `STALLS_MAX` steps, or re-planned more than `REPLANS_MAX` times.
    Stalled,
    /// A later leg, or a re-plan, found no way; or a person destination left the walker's place.
    NoRoute,
    /// The walker asked for another walk.
    Replaced,
    /// The walker's own `move`: direct control always wins.
    Stopped,
}

/// A walk ended, and how (`DECISIONS.md` `ARC-73`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkEnded {
    person: PersonId,
    destination: Destination,
    outcome: Ended,
}

impl Event for WalkEnded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("walk-ended");
    const OWNER: SystemId = MovementSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl WalkEnded {
    /// Who was walking.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where to.
    pub const fn destination(&self) -> Destination {
        self.destination
    }

    /// How it ended.
    pub const fn outcome(&self) -> Ended {
        self.outcome
    }
}

/// The `walk-started` fact, heard in `place` — where the walker stands — about the walker.
pub(crate) fn walk_started(person: PersonId, destination: Destination, place: PlaceId) -> Emission {
    Emission::new::<WalkStarted>(
        codec::encode(&WalkStarted {
            person,
            destination,
        }),
        Visibility::Place(place),
    )
    .about(vec![person.entity_id()])
    .with_participants(vec![person.entity_id()])
    .at_place(place)
}

/// The `walk-ended` fact, heard in `place` — where the walker stands — about the walker.
pub(crate) fn walk_ended(
    person: PersonId,
    destination: Destination,
    outcome: Ended,
    place: PlaceId,
) -> Emission {
    Emission::new::<WalkEnded>(
        codec::encode(&WalkEnded {
            person,
            destination,
            outcome,
        }),
        Visibility::Place(place),
    )
    .about(vec![person.entity_id()])
    .with_participants(vec![person.entity_id()])
    .at_place(place)
}

/// Two places open onto each other through one doorway, at `a_at` in `a`'s frame and `b_at` in
/// `b`'s.
///
/// One fact for both directions, because a doorway is one thing: [`MovementSystem`] reduces it into
/// the [`Passages`](crate::Passages) of both places, so the way out and the way back cannot be
/// stated differently.
///
/// Stated at genesis only in this pack's first version. No action opens a passage at runtime; a
/// system that builds or locks doors is a later pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassageOpened {
    a: PlaceId,
    a_at: Option<LocalPosition>,
    b: PlaceId,
    b_at: Option<LocalPosition>,
}

impl Event for PassageOpened {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("passage-opened");
    const OWNER: SystemId = MovementSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PassageOpened {
    /// One side of the doorway.
    pub const fn a(&self) -> (PlaceId, Option<LocalPosition>) {
        (self.a, self.a_at)
    }

    /// The other side.
    pub const fn b(&self) -> (PlaceId, Option<LocalPosition>) {
        (self.b, self.b_at)
    }
}

/// This pack's [`PassageOpened`] as a fact ready to be recorded at genesis.
///
/// Public for the reason presence's `arrival` is: a world is assembled as well as run, and the only
/// way to give a place its [`Passages`](crate::Passages) is a fact this pack reduces. The encoding
/// stays this pack's own. Whether the two ends are distinct places of this world is checked when the
/// fact is reduced, by the owner ([`MovementSystem`]), which refuses rather than writes.
pub fn passage(
    a: PlaceId,
    a_at: Option<LocalPosition>,
    b: PlaceId,
    b_at: Option<LocalPosition>,
) -> Emission {
    Emission::new::<PassageOpened>(
        codec::encode(&PassageOpened { a, a_at, b, b_at }),
        Visibility::Public,
    )
    .about(vec![a.entity_id(), b.entity_id()])
    .at_place(a)
}
