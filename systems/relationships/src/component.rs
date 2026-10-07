//! The state this pack owns: whom a person knows, and how well.

use mineworld_contracts::{EventId, PersonId, WorldTime};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::level::{Level, level};
use crate::system::RelationshipsSystem;

/// The bounds of the values, inclusive.
const FAMILIARITY: (i32, i32) = (0, 1_000);
const REGARD: (i32, i32) = (-1_000, 1_000);

/// What one person holds about one other (`step-09-social.md` SD-5). Integers only (I-7), asymmetric:
/// Alice's regard for Bob is not Bob's for Alice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipValues {
    familiarity: i32,
    regard: i32,
    exchanges: u32,
    activities_shared: u32,
    first_met: WorldTime,
    first_met_by: EventId,
    last_contact: WorldTime,
}

impl RelationshipValues {
    /// Nothing yet, met at `at` through the fact `by`.
    pub const fn met(at: WorldTime, by: EventId) -> Self {
        Self {
            familiarity: 0,
            regard: 0,
            exchanges: 0,
            activities_shared: 0,
            first_met: at,
            first_met_by: by,
            last_contact: at,
        }
    }

    /// How well this person knows the other, `0..=1000`.
    pub const fn familiarity(&self) -> i32 {
        self.familiarity
    }

    /// What this person thinks of the other, `-1000..=1000`.
    pub const fn regard(&self) -> i32 {
        self.regard
    }

    /// How many times the other has spoken to this person, or this person to them.
    pub const fn exchanges(&self) -> u32 {
        self.exchanges
    }

    /// How many activities the two have done together.
    pub const fn activities_shared(&self) -> u32 {
        self.activities_shared
    }

    /// When they first met.
    pub const fn first_met(&self) -> WorldTime {
        self.first_met
    }

    /// The fact through which they first met.
    pub const fn first_met_by(&self) -> EventId {
        self.first_met_by
    }

    /// The last time anything passed between them.
    pub const fn last_contact(&self) -> WorldTime {
        self.last_contact
    }

    /// The level these values read as.
    pub const fn level(&self) -> Level {
        level(self.familiarity, self.regard)
    }

    /// Adds to familiarity and regard, saturating at their bounds, and records the contact.
    pub(crate) fn adjust(&mut self, familiarity: i32, regard: i32, at: WorldTime) {
        self.familiarity = self
            .familiarity
            .saturating_add(familiarity)
            .clamp(FAMILIARITY.0, FAMILIARITY.1);
        self.regard = self.regard.saturating_add(regard).clamp(REGARD.0, REGARD.1);
        self.last_contact = at;
    }

    pub(crate) fn exchanged(&mut self) {
        self.exchanges = self.exchanges.saturating_add(1);
    }

    pub(crate) fn shared_an_activity(&mut self) {
        self.activities_shared = self.activities_shared.saturating_add(1);
    }
}

/// One entry: a counterpart and the values held about them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acquaintance {
    counterpart: PersonId,
    values: RelationshipValues,
}

impl Acquaintance {
    /// Whom this entry is about.
    pub const fn counterpart(&self) -> PersonId {
        self.counterpart
    }

    /// What is held about them.
    pub const fn values(&self) -> &RelationshipValues {
        &self.values
    }
}

/// Everybody a person knows, by counterpart (`DECISIONS.md` `ARC-28`).
///
/// A component on the `from` Person keyed by the counterpart: `DD-6`'s "state keyed by the triple"
/// for the one relation type this pack declares, with no kernel change. Every entry has its `knows`
/// edge and every edge its entry — both written in the same reduction by the one owner. A list sorted
/// by counterpart rather than a map, because a person's identity is not a string key and the encoding
/// must be the same on every run (`AC-12`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acquaintances {
    known: Vec<Acquaintance>,
}

impl Acquaintances {
    /// Every entry, by counterpart.
    pub fn known(&self) -> &[Acquaintance] {
        &self.known
    }

    /// What is held about `counterpart`, if they are known.
    pub fn of(&self, counterpart: PersonId) -> Option<&RelationshipValues> {
        self.known
            .iter()
            .find(|entry| entry.counterpart == counterpart)
            .map(Acquaintance::values)
    }

    /// The entry for `counterpart`, created as `met` if there was none; whether it was created.
    pub(crate) fn entry(
        &mut self,
        counterpart: PersonId,
        met: RelationshipValues,
    ) -> (&mut RelationshipValues, bool) {
        let position = match self
            .known
            .binary_search_by(|entry| entry.counterpart.cmp(&counterpart))
        {
            Ok(found) => (found, false),
            Err(at) => {
                self.known.insert(
                    at,
                    Acquaintance {
                        counterpart,
                        values: met,
                    },
                );
                (at, true)
            }
        };
        (&mut self.known[position.0].values, position.1)
    }

    /// How many people are known.
    pub fn len(&self) -> usize {
        self.known.len()
    }

    /// Whether nobody is known.
    pub fn is_empty(&self) -> bool {
        self.known.is_empty()
    }
}

owned_component! {
    component = Acquaintances,
    owner = RelationshipsSystem,
    component_type = "acquaintances",
    schema_version = 1,
}
