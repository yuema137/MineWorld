//! The facts this pack records: the sit points a place was authored with, and somebody sitting down or
//! standing up.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, LocalPosition, Orientation, PersonId, PlaceId,
    SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::key::SitPointKey;
use crate::point::SitPoints;
use crate::system::PostureSystem;

/// `place` has these sit points: stated at genesis, from the place's authored `sit-points:` section, and
/// reduced by this pack alone into the place's [`SitPoints`](crate::SitPoints) (`ARC-31`, `ARC-147`).
///
/// Public, like `place-shaped`: where a place's seats are is what anybody who sees the place sees.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceSitPoints {
    place: PlaceId,
    points: SitPoints,
}

impl Event for PlaceSitPoints {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("place-sit-points");
    const OWNER: SystemId = PostureSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PlaceSitPoints {
    /// States that `place` has `points`.
    pub const fn new(place: PlaceId, points: SitPoints) -> Self {
        Self { place, points }
    }

    /// Which place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Its sit points.
    pub const fn points(&self) -> &SitPoints {
        &self.points
    }

    /// As a fact ready to record: public, about the place, at the place.
    pub fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::Public)
            .about(vec![self.place.entity_id()])
            .at_place(self.place)
    }
}

/// `person` sat down on `point` in `place`, at `at`, facing `facing`.
///
/// Stated by `sit`, after presence's `arrived` has put the person on the point. Reduced into
/// [`Posture::Seated`](crate::Posture) with the instant the fact is recorded as `since`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SatDown {
    person: PersonId,
    place: PlaceId,
    point: SitPointKey,
    at: LocalPosition,
    facing: Orientation,
}

impl Event for SatDown {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("sat-down");
    const OWNER: SystemId = PostureSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl SatDown {
    /// States that `person` sat down on `point`.
    pub const fn new(
        person: PersonId,
        place: PlaceId,
        point: SitPointKey,
        at: LocalPosition,
        facing: Orientation,
    ) -> Self {
        Self {
            person,
            place,
            point,
            at,
            facing,
        }
    }

    /// Who sat down.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// On which sit point.
    pub const fn point(&self) -> &SitPointKey {
        &self.point
    }

    /// The seated position.
    pub const fn at(&self) -> LocalPosition {
        self.at
    }

    /// The facing.
    pub const fn facing(&self) -> Orientation {
        self.facing
    }

    /// As a fact ready to record: heard by `heard_by` — the place, unless an Interaction List narrows it
    /// to the participants (`ARC-63`), which is the one thing a list may do to it — about the person, at
    /// the place.
    pub fn emission(&self, heard_by: Visibility) -> Emission {
        Emission::new::<Self>(codec::encode(self), heard_by)
            .about(vec![self.person.entity_id()])
            .with_participants(vec![self.person.entity_id()])
            .at_place(self.place)
    }
}

/// Why somebody stood up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandCause {
    /// They asked to: `stand`.
    Requested,
    /// They were moved off their point by their own movement (`ARC-147`, SD-a-6).
    Moved,
}

/// `person` stood up from `point` in `place`, for `cause`.
///
/// Reduced into the removal of the person's [`Posture`](crate::Posture) row. The location does not
/// change: a sit point is on free floor, so a person stands where they sat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoodUp {
    person: PersonId,
    place: PlaceId,
    point: SitPointKey,
    cause: StandCause,
}

impl Event for StoodUp {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("stood-up");
    const OWNER: SystemId = PostureSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl StoodUp {
    /// States that `person` stood up from `point`, for `cause`.
    pub const fn new(
        person: PersonId,
        place: PlaceId,
        point: SitPointKey,
        cause: StandCause,
    ) -> Self {
        Self {
            person,
            place,
            point,
            cause,
        }
    }

    /// Who stood up.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// From which sit point.
    pub const fn point(&self) -> &SitPointKey {
        &self.point
    }

    /// Why.
    pub const fn cause(&self) -> StandCause {
        self.cause
    }

    /// As a fact ready to record: heard by `heard_by` (the place, or what a list narrows it to), about
    /// the person, at the place.
    pub fn emission(&self, heard_by: Visibility) -> Emission {
        Emission::new::<Self>(codec::encode(self), heard_by)
            .about(vec![self.person.entity_id()])
            .with_participants(vec![self.person.entity_id()])
            .at_place(self.place)
    }
}
