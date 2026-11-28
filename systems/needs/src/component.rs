//! The state this pack owns (schema 2, `ARC-370`).

use mineworld_contracts::WorldTime;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::food_energy::{Reserve, Serving, motivation};
use crate::level::{Level, Mode, Need, Rates, current, fold};
use crate::system::NeedsSystem;

/// A person's hunger, in the model their world chose: a unitless deficit (`ARC-284`) or a kcal reserve
/// (`ARC-370`). One state per person; never both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Hunger {
    /// A deficit 0 … 1 000 rising at the world's rates.
    Unitless(Level),
    /// A kcal reserve draining at the person's daily use.
    FoodEnergy(Reserve),
}

/// How hungry and how tired a person is, each stored with the instant it was true — read at world
/// time, never ticked (`ARC-284`, `ARC-86`). Tiredness alone carries whether the person sleeps.
///
/// Inserted from the person's `needs-assigned` at genesis; rewritten only when another pack's fact
/// changes a need (a meal, falling asleep, waking), and then only the needs that fact changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Needs {
    hunger: Hunger,
    tiredness: Level,
}

owned_component! {
    component = Needs,
    owner = NeedsSystem,
    component_type = "needs",
    schema_version = 2,
}

impl Needs {
    /// Both needs at `at`, awake: a person at genesis.
    pub(crate) const fn assigned(hunger: Hunger, tiredness: u16, at: WorldTime) -> Self {
        Self {
            hunger,
            tiredness: Level::new(tiredness, at, Mode::Awake),
        }
    }

    /// Hunger as stored.
    pub const fn hunger(&self) -> Hunger {
        self.hunger
    }

    /// Tiredness as stored.
    pub const fn tiredness(&self) -> Level {
        self.tiredness
    }

    /// The mode the needs move in: whether the person was asleep at the last of sleep's facts.
    pub const fn mode(&self) -> Mode {
        self.tiredness.mode()
    }

    /// `(hunger, tiredness)` at `now` under `rates`, hunger normalized 0 … 1 000 in either model;
    /// `daily` is the person's food-energy use. `None` only when `now` is before a stored reserve.
    pub fn current(&self, now: WorldTime, rates: &Rates, daily: u32) -> Option<(u16, u16)> {
        let hunger = match &self.hunger {
            Hunger::Unitless(level) => current(level, Need::Hunger, now, rates),
            Hunger::FoodEnergy(reserve) => reserve.reading(now, daily)?.hunger,
        };
        Some((
            hunger,
            current(&self.tiredness, Need::Tiredness, now, rates),
        ))
    }

    /// Unitless hunger brought to `at` and lowered by `by`, never below zero; tiredness untouched.
    /// `None` for a food-energy person.
    #[must_use]
    pub fn fed(self, by: u32, at: WorldTime, rates: &Rates) -> Option<Self> {
        let Hunger::Unitless(level) = self.hunger else {
            return None;
        };
        let folded = fold(&level, Need::Hunger, at, rates);
        let left = u32::from(folded.stored()).saturating_sub(by);
        let left = u16::try_from(left).unwrap_or(0);
        Some(Self {
            hunger: Hunger::Unitless(Level::new(left, at, folded.mode())),
            ..self
        })
    }

    /// The reserve after `count` servings of `kcal` at `at`, capped at `daily`. `None` for a unitless
    /// person, or when `at` is before the stored reserve.
    #[must_use]
    pub fn energized(self, kcal: u32, count: u32, at: WorldTime, daily: u32) -> Option<Self> {
        let Hunger::FoodEnergy(reserve) = self.hunger else {
            return None;
        };
        Some(Self {
            hunger: Hunger::FoodEnergy(reserve.fed(at, daily, kcal, count)?),
            ..self
        })
    }

    /// Both needs brought to `at`, moving in `mode` from then on. A reserve drains at one rate asleep
    /// and awake, so it is left as stored.
    #[must_use]
    pub fn turned(self, mode: Mode, at: WorldTime, rates: &Rates) -> Self {
        let hunger = match self.hunger {
            Hunger::Unitless(level) => {
                Hunger::Unitless(fold(&level, Need::Hunger, at, rates).in_mode(mode))
            }
            reserve @ Hunger::FoodEnergy(_) => reserve,
        };
        Self {
            hunger,
            tiredness: fold(&self.tiredness, Need::Tiredness, at, rates).in_mode(mode),
        }
    }
}

/// What one of an item kind does to hunger when eaten or drunk: the item form of the `needs:` section
/// (`ARC-285`, `ARC-370`). On the item kind; disclosed only inside its holder's own `needs` record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Nourishment {
    /// Lowers a unitless hunger by `hunger`.
    Unitless {
        /// Hunger one lowers by, 0 … 1 000.
        hunger: u16,
    },
    /// Adds `energy_kcal` to a reserve: one whole serving.
    FoodEnergy {
        /// kcal per serving, 0 … 100 000.
        energy_kcal: u32,
        /// What a serving is.
        serving: Serving,
    },
}

owned_component! {
    component = Nourishment,
    owner = NeedsSystem,
    component_type = "nourishment",
    schema_version = 2,
}

impl Nourishment {
    /// How strongly one motivates a person whose capacity is `capacity` (their daily use), on the
    /// unitless 0 … 1 000 scale controllers rank food by: the hunger it lowers, or `⌈1 000·min(K, C)/C⌉`.
    pub fn motivation(&self, capacity: u32) -> u16 {
        match self {
            Self::Unitless { hunger } => *hunger,
            Self::FoodEnergy { energy_kcal, .. } => motivation(*energy_kcal, capacity),
        }
    }
}
