//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`;
//! `configure/needs.yaml`; `pr-s35-nb-needs.md` SD-Nb-4).
//!
//! ```yaml
//! parameters:
//!   - { thresholds: [300, 500, 800], restore: { tiredness: [] } }
//!   - { actor: night, hunger_per_hour: 90, rest_per_hour: 200 }
//! ```
//!
//! ```text
//! actions      none: a need is a condition, not an action; a `rules:` entry is refused as undeclared
//! parameters   hunger_per_hour          10 … 500,   120   rise awake
//!              hunger_asleep_per_hour    0 … 500,    30   rise asleep
//!              tiredness_per_hour       10 … 500,    60   rise awake (and asleep without `sleeping`)
//!              rest_per_hour            10 … 1 000, 125   fall asleep at level 300; in proportion to the level (Q-Nc3-5)
//!              thresholds   three integers in 1 … 1 000, strictly increasing; [400, 600, 900]
//!              restore      { hunger: [eating], tiredness: [sleeping] }, each restorer at most once
//!              The rates may be scoped by actor (the person); thresholds and restore are base only.
//!              Looked up with no place: a person's needs move wherever they are.
//! facts        needs-assigned   participants; not biographical, not configurable
//! requires     eating in the base restore → consumption; sleeping → sleep — checked for a written
//!              file and, with nothing written, for the defaults (SD-Nb-5)
//! ```

use mineworld_contracts::{ComponentTypeId, EntityId, Event, EventTypeId, SystemId};
use mineworld_kernel::WorldRead;
use mineworld_sdk::interactions::{
    self, Audience, FactDecl, InteractionSection, ParameterBound, Position, Resolved, Role, Roles,
    Section,
};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::event::NeedsAssigned;
use crate::level::{MAX, Rates};
use crate::section::EATING;
use crate::system::NeedsSystem;

/// The system `sleeping` requires.
pub const SLEEPING: SystemId = SystemId::from_static("sleep");

/// The three thresholds between a need's four words: `ok` below the first, then `peckish` / `sleepy`,
/// `hungry` / `tired`, and `famished` / `exhausted` from the third.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Thresholds(pub [u16; 3]);

impl core::fmt::Display for Thresholds {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let [a, b, c] = self.0;
        write!(f, "[{a}, {b}, {c}]")
    }
}

/// The bound of [`Thresholds`]: each in `1 ..= 1 000`, strictly increasing.
pub struct StrictlyIncreasing;

impl ParameterBound<Thresholds> for StrictlyIncreasing {
    fn admits(&self, value: &Thresholds) -> bool {
        let [a, b, c] = value.0;
        a >= 1 && a < b && b < c && c <= MAX
    }

    fn describe(&self) -> String {
        "three integers in 1 … 1 000, strictly increasing".to_owned()
    }
}

/// What restores a need: a closed set (step-35 Q-S35-4).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Restorer {
    /// Eating or drinking (consumption's `eat` / `drink`, as inventory's `items-consumed`): restores
    /// hunger.
    Eating,
    /// Sleeping (sleep's `fell-asleep` … `woke`): restores tiredness.
    Sleeping,
}

impl core::fmt::Display for Restorer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Eating => "eating",
            Self::Sleeping => "sleeping",
        })
    }
}

/// Which restorers a world uses for each need. A need left out keeps its default; an empty list says
/// that need is never restored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Restore {
    /// What lowers hunger: `[eating]` or `[]`.
    #[serde(default = "eating")]
    pub hunger: Vec<Restorer>,
    /// What lowers tiredness: `[sleeping]` or `[]`.
    #[serde(default = "sleeping")]
    pub tiredness: Vec<Restorer>,
}

fn eating() -> Vec<Restorer> {
    vec![Restorer::Eating]
}

fn sleeping() -> Vec<Restorer> {
    vec![Restorer::Sleeping]
}

impl Default for Restore {
    fn default() -> Self {
        Self {
            hunger: eating(),
            tiredness: sleeping(),
        }
    }
}

impl Restore {
    /// Whether eating lowers hunger.
    pub fn eating(&self) -> bool {
        self.hunger.contains(&Restorer::Eating)
    }

    /// Whether sleeping lowers tiredness.
    pub fn sleeping(&self) -> bool {
        self.tiredness.contains(&Restorer::Sleeping)
    }
}

impl core::fmt::Display for Restore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let list = |restorers: &[Restorer]| {
            restorers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        };
        write!(
            f,
            "{{ hunger: [{}], tiredness: [{}] }}",
            list(&self.hunger),
            list(&self.tiredness)
        )
    }
}

/// The bound of [`Restore`]: hunger from `[eating]`, tiredness from `[sleeping]`, each at most once.
pub struct ClosedRestorers;

impl ParameterBound<Restore> for ClosedRestorers {
    fn admits(&self, value: &Restore) -> bool {
        let only = |restorers: &[Restorer], allowed: Restorer| {
            restorers.len() <= 1 && restorers.iter().all(|restorer| *restorer == allowed)
        };
        only(&value.hunger, Restorer::Eating) && only(&value.tiredness, Restorer::Sleeping)
    }

    fn describe(&self) -> String {
        "a list from [eating] for hunger and from [sleeping] for tiredness, each at most once"
            .to_owned()
    }
}

mineworld_sdk::parameters! {
    /// What a world may choose about hunger and tiredness.
    pub struct NeedsParameters, partial NeedsPartial {
        /// How much hunger rises per hour awake: from sated to hungry (600) in 5 hours by default,
        /// the spacing of the American Time Use Survey's meal peaks.
        hunger_per_hour: u16 = 120, 10 ..= 500;
        /// How much hunger rises per hour asleep: suppressed in sleep, so breakfast follows a night
        /// rather than a famine (authored).
        hunger_asleep_per_hour: u16 = 30, 0 ..= 500;
        /// How much tiredness rises per hour awake: 1 000 over the ~16.7 waking hours of a day with an
        /// 8-hour night.
        tiredness_per_hour: u16 = 60, 10 ..= 500;
        /// How much tiredness falls per hour asleep at level 300: the fall is in proportion to the level
        /// (the two-process model's recovery, Q-Nc3-5; `mineworld_sleep::recovery`), so an 8-hour night
        /// from 1 000 ends near 35, about 121 per hour on average.
        rest_per_hour: u16 = 125, 10 ..= 1_000;
        /// The thresholds between a need's words: three integers in 1 … 1 000, strictly increasing.
        thresholds: Thresholds = Thresholds([400, 600, 900]), StrictlyIncreasing;
        /// What restores each need: hunger from [eating], tiredness from [sleeping].
        restore: Restore = Restore::default(), ClosedRestorers;
    }
}

/// The rates, scoped by the person alone.
const PERSON: &[Role] = &[Role::Actor];

/// Base only: never under a role, never in a region.
const BASE: &[Role] = &[];

/// Where a role is found in `needs-assigned`'s envelope: the person is its one participant.
const FACT_ROLES: &[(Role, Position)] = &[(Role::Actor, Position::Participant(0))];

impl InteractionSection for NeedsSystem {
    type Parameters = NeedsParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("needs-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("needs-interactions");
    const FACTS: &'static [FactDecl] = &[FactDecl {
        fact: NeedsAssigned::EVENT_TYPE,
        roles: FACT_ROLES,
        default_audience: Audience::Participants,
        narrowest: Audience::Participants,
        biography_configurable: false,
    }];
    const PARAMETER_ROLES: &'static [Role] = PERSON;
    const FIELD_ROLES: &'static [(&'static str, &'static [Role])] = &[
        ("hunger_per_hour", PERSON),
        ("hunger_asleep_per_hour", PERSON),
        ("tiredness_per_hour", PERSON),
        ("rest_per_hour", PERSON),
        ("thresholds", BASE),
        ("restore", BASE),
    ];

    /// `eating` in the base restore requires `consumption`; `sleeping`, `sleep`. With nothing written
    /// this is the defaults' answer: both.
    fn requires(section: &Section<Self>) -> Vec<SystemId> {
        let restore = base_restore(section);
        let mut systems = Vec::new();
        if restore.eating() {
            systems.push(EATING);
        }
        if restore.sleeping() {
            systems.push(SLEEPING);
        }
        systems
    }

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        codec::encode(resolved)
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

/// The restore the section's base says: the last base entry setting it, else the default.
fn base_restore(section: &Section<NeedsSystem>) -> Restore {
    section
        .entries
        .parameters
        .iter()
        .rev()
        .find_map(|entry| entry.fields.restore.clone())
        .unwrap_or_default()
}

/// The parameters in force for `person`: the world's, looked up with the person alone and no place.
pub fn parameters(read: &WorldRead<'_>, person: EntityId) -> NeedsParameters {
    let roles = Roles::new().with(Role::Actor, person);
    interactions::parameters_unplaced::<NeedsSystem>(read, &roles)
}

/// The rates in `parameters`, ready for the level arithmetic.
pub fn rates(parameters: &NeedsParameters) -> Rates {
    Rates {
        hunger_per_hour: parameters.hunger_per_hour,
        hunger_asleep_per_hour: parameters.hunger_asleep_per_hour,
        tiredness_per_hour: parameters.tiredness_per_hour,
        rest_per_hour: parameters.rest_per_hour,
        sleep_rests: parameters.restore.sleeping(),
    }
}

#[cfg(test)]
mod tests {
    use mineworld_kernel::System;

    use super::*;

    /// The section's defaults are fixed before any run (`ARC-23`, I-Nb-8) and pinned to the version.
    #[test]
    fn the_default_section_is_pinned_to_the_version() {
        assert_eq!(NeedsSystem::VERSION.get(), 1);
        let defaults = NeedsParameters::default();
        assert_eq!(
            (
                defaults.hunger_per_hour,
                defaults.hunger_asleep_per_hour,
                defaults.tiredness_per_hour,
                defaults.rest_per_hour,
            ),
            (120, 30, 60, 125)
        );
        assert_eq!(defaults.thresholds, Thresholds([400, 600, 900]));
        assert!(defaults.restore.eating() && defaults.restore.sleeping());
    }
}
