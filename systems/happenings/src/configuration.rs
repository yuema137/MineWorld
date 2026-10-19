//! `configure/happenings.yaml`, decoded by this pack's own types (`ARC-61`; design §5.1): deserializing
//! is validating, so a bad value is refused at assembly with the file, its line and column, and a
//! message naming the key; a refusal between entries names the entry by its list and index.
//!
//! ```yaml
//! seed: 7                     # u64; content, not --seed (F-WE-a-1)
//! kinds:                      # 1 … 64: what may happen, where, at which hours
//!   - { id: market-day, title: Saturday market, places: [street], hours: "08:00-14:00" }
//!   - { id: festival, places: [park], hours: "16:00-22:00", days: 1, announce_days: 7, audience: public }
//! calendar:                   # exactly one of weekdays | date | monthday per entry
//!   - { kind: market-day, place: street, weekdays: [saturday] }
//!   - { kind: festival, place: park, date: "10-31" }
//! random:                     # (kind, place) unique: it is the entry's stable draw key
//!   - { kind: street-musician, place: street, per_mille_per_day: 150 }
//! limits: { per_day: 3, per_place_running: 1 }
//! generators: []              # external generators this world selects, by system id (C3)
//! ```
//!
//! What the configuration becomes is [`HappeningsConfigured`](crate::event::HappeningsConfigured), with
//! every place resolved to its id.

use std::collections::BTreeSet;

use mineworld_contracts::{EntityKey, SystemId};
use serde::{Deserialize, Serialize};

use crate::entry::{CalendarEntry, FileCalendarEntry, RandomEntry};
use crate::kind::{KindCatalog, RawKind, bounded};

/// The most entries one of `calendar:` and `random:` may list: a bound on the work and on the
/// withheld facts of one day, and on the configuration the almanac carries in every snapshot.
pub const ENTRIES_MAX: usize = 64;

bounded!(
    /// The most happenings that may begin on one day, world-wide.
    PerDay: u8 = "u8", 1, 16, 3, "per_day"
);
bounded!(
    /// The most happenings that may overlap in time at one place.
    PerPlaceRunning: u8 = "u8", 1, 4, 1, "per_place_running"
);

/// The host's own caps (design §5.3), enforced on every draft from every source (INV-WE-6).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// The most happenings that may begin on one day, world-wide: 1 to 16, 3 when unsaid.
    #[serde(default)]
    per_day: PerDay,
    /// The most happenings that may overlap at one place: 1 to 4, 1 when unsaid.
    #[serde(default)]
    per_place_running: PerPlaceRunning,
}

impl Limits {
    /// The most happenings that may begin on one day.
    pub const fn per_day(self) -> PerDay {
        self.per_day
    }

    /// The most happenings that may overlap at one place.
    pub const fn per_place_running(self) -> PerPlaceRunning {
        self.per_place_running
    }
}

/// The happenings a world keeps: its seed, its kinds, its calendar and random entries, its limits, and
/// the external generators it selects.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawConfiguration")]
pub struct HappeningsConfiguration {
    seed: u64,
    kinds: KindCatalog<EntityKey>,
    calendar: Vec<CalendarEntry<EntityKey>>,
    random: Vec<RandomEntry<EntityKey>>,
    limits: Limits,
    generators: Vec<SystemId>,
}

/// The schema is the file's shape: deserializing goes through [`RawConfiguration`].
impl schemars::JsonSchema for HappeningsConfiguration {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("HappeningsConfiguration")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <RawConfiguration as schemars::JsonSchema>::json_schema(generator)
    }
}

/// The example the generated reference prints: a Saturday market, a yearly festival announced a week
/// ahead, and a street musician on a seeded chance.
fn example() -> serde_json::Value {
    serde_json::json!({
        "seed": 7,
        "kinds": [
            { "id": "market-day", "title": "Saturday market", "places": ["street"], "hours": "08:00-14:00" },
            { "id": "harvest-festival", "places": ["park"], "hours": "16:00-22:00", "announce_days": 7 },
            { "id": "street-musician", "places": ["street"], "hours": "11:00-13:00", "audience": "place" },
        ],
        "calendar": [
            { "kind": "market-day", "place": "street", "weekdays": ["saturday"] },
            { "kind": "harvest-festival", "place": "park", "date": "10-31" },
        ],
        "random": [{ "kind": "street-musician", "place": "street", "per_mille_per_day": 150 }],
        "limits": { "per_day": 3, "per_place_running": 1 },
    })
}

/// The file as written.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = example())]
struct RawConfiguration {
    /// The seed every random draw comes from: part of the world, not the run's --seed.
    seed: u64,
    /// What may happen, where and at which hours: 1 to 64 kinds, ids unique.
    kinds: Vec<RawKind>,
    /// Happenings on dates: each names exactly one of weekdays, date and monthday. At most 64.
    #[serde(default)]
    calendar: Vec<FileCalendarEntry>,
    /// Happenings on a seeded chance per day; each (kind, place) once. At most 64.
    #[serde(default)]
    random: Vec<RandomEntry<EntityKey>>,
    /// The world's caps on how many happenings begin a day and overlap at a place.
    #[serde(default)]
    limits: Limits,
    /// External generators this world selects, by system id, each once.
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::SystemName>")]
    generators: Vec<SystemId>,
}

impl TryFrom<RawConfiguration> for HappeningsConfiguration {
    type Error = String;

    fn try_from(raw: RawConfiguration) -> Result<Self, Self::Error> {
        let kinds = KindCatalog::from_list(raw.kinds)?;
        let calendar: Vec<CalendarEntry<EntityKey>> =
            raw.calendar.into_iter().map(|entry| entry.0).collect();
        for (list, count) in [("calendar", calendar.len()), ("random", raw.random.len())] {
            if count > ENTRIES_MAX {
                return Err(format!(
                    "{list} lists at most {ENTRIES_MAX} entries, not {count}"
                ));
            }
        }
        for (index, entry) in calendar.iter().enumerate() {
            admitted(&kinds, "calendar", index, entry.kind(), entry.place())?;
        }
        let mut drawn = BTreeSet::new();
        for (index, entry) in raw.random.iter().enumerate() {
            admitted(&kinds, "random", index, entry.kind(), entry.place())?;
            if !drawn.insert((entry.kind(), entry.place())) {
                let first = raw
                    .random
                    .iter()
                    .position(|other| {
                        other.kind() == entry.kind() && other.place() == entry.place()
                    })
                    .expect("the first of the two");
                return Err(format!(
                    "random[{first}] and random[{index}] both draw '{}' at '{}': each (kind, place) \
                     is one entry, because it is the entry's draw key",
                    entry.kind(),
                    entry.place().as_str()
                ));
            }
        }
        let mut selected = BTreeSet::new();
        for generator in &raw.generators {
            if !selected.insert(generator) {
                return Err(format!("generators lists '{generator}' twice"));
            }
        }
        Ok(Self {
            seed: raw.seed,
            kinds,
            calendar,
            random: raw.random,
            limits: raw.limits,
            generators: raw.generators,
        })
    }
}

/// Refuses an entry whose kind the catalog lacks, or whose place is not one of its kind's places.
fn admitted(
    kinds: &KindCatalog<EntityKey>,
    list: &str,
    index: usize,
    kind: &crate::kind::HappeningKindId,
    place: &EntityKey,
) -> Result<(), String> {
    let Some(declared) = kinds.get(kind) else {
        return Err(format!(
            "{list}[{index}] names the kind '{kind}', which kinds does not declare"
        ));
    };
    if !declared.admits(place) {
        return Err(format!(
            "{list}[{index}] puts '{kind}' at '{}', which is not one of its kind's places",
            place.as_str()
        ));
    }
    Ok(())
}

impl HappeningsConfiguration {
    /// The seed of every random draw.
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// The kind catalog, places by key.
    pub const fn kinds(&self) -> &KindCatalog<EntityKey> {
        &self.kinds
    }

    /// The calendar entries, in file order.
    pub fn calendar(&self) -> &[CalendarEntry<EntityKey>] {
        &self.calendar
    }

    /// The random entries, in file order.
    pub fn random(&self) -> &[RandomEntry<EntityKey>] {
        &self.random
    }

    /// The host's caps.
    pub const fn limits(&self) -> Limits {
        self.limits
    }

    /// The external generators the world selects, in file order.
    pub fn generators(&self) -> &[SystemId] {
        &self.generators
    }
}
