//! The facts this pack records (design §5.4). None has a public constructor: only this pack states
//! them.
//!
//! `happenings-configured` is the world's configuration (`ARC-61`), SystemInternal with no subjects.
//! `happening-announced` is Public — an announcement is news. `happening-began` and `happening-ended`
//! are Public or heard at the happening's place, as its kind's `audience` says. `happening-withheld`
//! is SystemInternal: an author can read why a festival never happened (`INV-15`), and nobody
//! perceives it. The three lifecycle facts are about the happening's place and happen there; none is
//! biographical (a market day is not something a person did).

use mineworld_contracts::{
    EntityKey, Event, EventSchemaVersion, EventTypeId, PlaceId, SystemId, WorldTime,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::configuration::{HappeningsConfiguration, Limits};
use crate::draw;
use crate::entry::{CalendarEntry, RandomEntry};
use crate::generate::{DayIndex, DraftSource, HappeningDraft};
use crate::host::{HappeningRef, OpenHappening, WithheldReason};
use crate::kind::{HappeningKindId, KindCatalog, Title};
use crate::system::HappeningsSystem;

/// A happening was admitted and is announced: what, where, from when until when, what it is called,
/// and which source proposed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningAnnounced {
    happening: HappeningRef,
    until: WorldTime,
    title: Option<Title>,
    source: DraftSource,
}

impl Event for HappeningAnnounced {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("happening-announced");
    const OWNER: SystemId = HappeningsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl HappeningAnnounced {
    pub(crate) fn new(open: OpenHappening, title: Option<Title>, source: DraftSource) -> Self {
        Self {
            until: open.until(),
            happening: open.into_happening(),
            title,
            source,
        }
    }

    /// The happening.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }

    /// The instant it ends.
    pub const fn until(&self) -> WorldTime {
        self.until
    }

    /// Its kind's title, if the kind has one.
    pub const fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }

    /// Who proposed it.
    pub const fn source(&self) -> &DraftSource {
        &self.source
    }

    /// The open happening it adds to the almanac.
    pub(crate) fn open(&self) -> OpenHappening {
        OpenHappening::new(self.happening.clone(), self.until)
    }
}

/// A happening began; it runs until `until`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningBegan {
    happening: HappeningRef,
    until: WorldTime,
}

impl Event for HappeningBegan {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("happening-began");
    const OWNER: SystemId = HappeningsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl HappeningBegan {
    pub(crate) const fn new(happening: HappeningRef, until: WorldTime) -> Self {
        Self { happening, until }
    }

    /// The happening.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }

    /// The instant it ends.
    pub const fn until(&self) -> WorldTime {
        self.until
    }
}

/// A happening ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningEnded {
    happening: HappeningRef,
}

impl Event for HappeningEnded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("happening-ended");
    const OWNER: SystemId = HappeningsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl HappeningEnded {
    pub(crate) const fn new(happening: HappeningRef) -> Self {
        Self { happening }
    }

    /// The happening.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }
}

/// A draft was not admitted: what it proposed, why it was withheld, and who proposed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningWithheld {
    kind: HappeningKindId,
    place: PlaceId,
    day: DayIndex,
    reason: WithheldReason,
    source: DraftSource,
}

impl Event for HappeningWithheld {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("happening-withheld");
    const OWNER: SystemId = HappeningsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl HappeningWithheld {
    pub(crate) fn new(draft: &HappeningDraft, reason: WithheldReason, source: DraftSource) -> Self {
        Self {
            kind: draft.kind().clone(),
            place: draft.place(),
            day: draft.day(),
            reason,
            source,
        }
    }

    /// The kind proposed.
    pub const fn kind(&self) -> &HappeningKindId {
        &self.kind
    }

    /// The place proposed.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The day proposed.
    pub const fn day(&self) -> DayIndex {
        self.day
    }

    /// Why it was withheld.
    pub const fn reason(&self) -> WithheldReason {
        self.reason
    }

    /// Who proposed it.
    pub const fn source(&self) -> &DraftSource {
        &self.source
    }
}

/// A random entry with its draw key: FNV-1a over `"random"`, its kind and its place's key, computed
/// once when the world is seeded, so the key is part of the fact and never depends on the entry's
/// position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyedRandom {
    key: u64,
    entry: RandomEntry<PlaceId>,
}

impl KeyedRandom {
    /// The entry's stable draw key.
    pub const fn key(&self) -> u64 {
        self.key
    }

    /// The entry.
    pub const fn entry(&self) -> &RandomEntry<PlaceId> {
        &self.entry
    }
}

/// The world's happenings, as configured, every place resolved to its id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningsConfigured {
    seed: u64,
    kinds: KindCatalog<PlaceId>,
    calendar: Vec<CalendarEntry<PlaceId>>,
    random: Vec<KeyedRandom>,
    limits: Limits,
    generators: Vec<SystemId>,
}

impl Event for HappeningsConfigured {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("happenings-configured");
    const OWNER: SystemId = HappeningsSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl HappeningsConfigured {
    /// What `configuration` states once `resolve` has named each of its places; the first place
    /// `resolve` refuses refuses the whole.
    pub(crate) fn of<E>(
        configuration: &HappeningsConfiguration,
        resolve: impl Fn(&EntityKey) -> Result<PlaceId, E>,
    ) -> Result<Self, E> {
        let kinds = configuration.kinds().map_places(&resolve)?;
        let calendar = configuration
            .calendar()
            .iter()
            .map(|entry| Ok(entry.at(resolve(entry.place())?)))
            .collect::<Result<Vec<_>, E>>()?;
        let random = configuration
            .random()
            .iter()
            .map(|entry| {
                Ok(KeyedRandom {
                    key: draw::key(&["random", entry.kind().as_str(), entry.place().as_str()]),
                    entry: entry.at(resolve(entry.place())?),
                })
            })
            .collect::<Result<Vec<_>, E>>()?;
        Ok(Self {
            seed: configuration.seed(),
            kinds,
            calendar,
            random,
            limits: configuration.limits(),
            generators: configuration.generators().to_vec(),
        })
    }

    /// The seed of every random draw.
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// The kind catalog.
    pub const fn kinds(&self) -> &KindCatalog<PlaceId> {
        &self.kinds
    }

    /// The calendar entries, in file order.
    pub fn calendar(&self) -> &[CalendarEntry<PlaceId>] {
        &self.calendar
    }

    /// The random entries, in file order, with their keys.
    pub fn random(&self) -> &[KeyedRandom] {
        &self.random
    }

    /// The host's caps.
    pub const fn limits(&self) -> Limits {
        self.limits
    }

    /// The external generators the world selects.
    pub fn generators(&self) -> &[SystemId] {
        &self.generators
    }
}
