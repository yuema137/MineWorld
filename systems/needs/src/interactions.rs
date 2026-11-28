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
//! parameters   hunger_model             unitless | food-energy, unitless; base only (ARC-370)
//!              energy_kcal_per_day      1 … 100 000, 2 000   food-energy use per world day and capacity;
//!                                       only in food-energy, where a base entry must set it
//!              hunger_per_hour          10 … 500,   120   rise awake (unitless only)
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

use mineworld_authoring::ConfigurationRefusal;
use mineworld_contracts::{
    ComponentSchemaVersion, ComponentTypeId, EntityId, Event, EventSchemaVersion, EventTypeId,
    RejectionCode, SystemId,
};
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

/// A parameter written beside the hunger model it does not belong to (`ARC-370`).
pub const MODEL_PARAMETER_CONFLICT: RejectionCode =
    RejectionCode::from_static("needs-model-parameter-conflict");

/// The food-energy model without a base `energy_kcal_per_day` (`ARC-370`).
pub const DAILY_USE_REQUIRED: RejectionCode =
    RejectionCode::from_static("needs-energy-daily-use-required");

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

/// Which model hunger is in (`ARC-370`): a unitless deficit, or a kcal reserve.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum HungerModel {
    /// A deficit 0 … 1 000 rising at `hunger_per_hour` (`ARC-284`): today's behaviour.
    Unitless,
    /// A kcal reserve, full at `energy_kcal_per_day`, draining at that rate asleep and awake.
    FoodEnergy,
}

impl core::fmt::Display for HungerModel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Unitless => "unitless",
            Self::FoodEnergy => "food-energy",
        })
    }
}

/// The bound of [`HungerModel`]: either; the type already says so.
pub struct EitherModel;

impl ParameterBound<HungerModel> for EitherModel {
    fn admits(&self, _: &HungerModel) -> bool {
        true
    }

    fn describe(&self) -> String {
        "unitless or food-energy".to_owned()
    }
}

mineworld_sdk::parameters! {
    /// What a world may choose about hunger and tiredness.
    pub struct NeedsParameters, partial NeedsPartial {
        /// Which model hunger is in: `unitless` (a deficit, the rates below) or `food-energy` (a kcal
        /// reserve, `energy_kcal_per_day`).
        hunger_model: HungerModel = HungerModel::Unitless, EitherModel;
        /// Food energy used per world day (86 400 world seconds), kcal, and the reserve's capacity: 2 000
        /// by default, the FDA Daily Value reference intake on US Nutrition Facts labels (21 CFR
        /// 101.9(c)(9)). Only in `food-energy`, where a base entry must set it.
        energy_kcal_per_day: u32 = 2_000, 1 ..= 100_000;
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
    const CONFIGURED_SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(2);
    const COMPONENT_SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(2);
    const FIELD_ROLES: &'static [(&'static str, &'static [Role])] = &[
        ("hunger_model", BASE),
        ("energy_kcal_per_day", PERSON),
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

    /// The two models' parameters are exclusive (`ARC-370`): in `unitless`, `energy_kcal_per_day` is
    /// refused wherever written; in `food-energy`, `hunger_per_hour` and `hunger_asleep_per_hour` are,
    /// even shadowed, and a base entry must set `energy_kcal_per_day`.
    fn validate_authored(
        section: &Section<Self>,
        resolved: &Resolved<Self>,
    ) -> Result<(), ConfigurationRefusal> {
        let model = resolved.base.parameters.hunger_model;
        let conflict = |field: &str| {
            section
                .entries_setting(field)
                .into_iter()
                .next()
                .map(|entry| ConfigurationRefusal::Owner {
                    entry: Some(entry),
                    code: MODEL_PARAMETER_CONFLICT,
                    detail: format!("'{field}' is not a parameter of the {model} hunger model"),
                })
        };
        match model {
            HungerModel::Unitless => conflict("energy_kcal_per_day").map_or(Ok(()), Err),
            HungerModel::FoodEnergy => {
                for field in ["hunger_per_hour", "hunger_asleep_per_hour"] {
                    if let Some(refusal) = conflict(field) {
                        return Err(refusal);
                    }
                }
                let stated = section.entries.parameters.iter().any(|entry| {
                    entry.selectors.specificity() == 0 && entry.fields.energy_kcal_per_day.is_some()
                });
                if stated {
                    Ok(())
                } else {
                    Err(ConfigurationRefusal::Owner {
                        entry: None,
                        code: DAILY_USE_REQUIRED,
                        detail: "the food-energy hunger model needs a base entry setting \
                                 'energy_kcal_per_day'"
                            .to_owned(),
                    })
                }
            }
        }
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

/// The parameters that apply to every person and item: the base, no role filled. What decides the
/// hunger model, which no role may scope.
pub fn base_parameters(read: &WorldRead<'_>) -> NeedsParameters {
    interactions::parameters_unplaced::<NeedsSystem>(read, &Roles::new())
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
        assert_eq!(NeedsSystem::VERSION.get(), 2);
        let defaults = NeedsParameters::default();
        assert_eq!(
            (defaults.hunger_model, defaults.energy_kcal_per_day),
            (HungerModel::Unitless, 2_000)
        );
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
