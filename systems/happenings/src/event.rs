//! The facts this pack records (design §5.4). None has a public constructor: only this pack states
//! them.
//!
//! `happenings-configured` is the world's configuration (`ARC-61`), SystemInternal with no subjects.

use mineworld_contracts::{EntityKey, Event, EventSchemaVersion, EventTypeId, PlaceId, SystemId};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::configuration::{HappeningsConfiguration, Limits};
use crate::draw;
use crate::entry::{CalendarEntry, RandomEntry};
use crate::kind::KindCatalog;
use crate::system::HappeningsSystem;

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
