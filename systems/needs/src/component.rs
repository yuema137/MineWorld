//! The state this pack owns.

use mineworld_contracts::WorldTime;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::level::{Level, Mode, Need, Rates, current, fold};
use crate::system::NeedsSystem;

/// How hungry and how tired a person is, each stored as a level, the instant it was true and the mode
/// it has moved in since — read at world time, never ticked (`ARC-284`, `ARC-86`).
///
/// Inserted from the person's `needs-assigned` at genesis; rewritten only when another pack's fact
/// changes a need (a meal, falling asleep, waking), and then only the needs that fact changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Needs {
    hunger: Level,
    tiredness: Level,
}

owned_component! {
    component = Needs,
    owner = NeedsSystem,
    component_type = "needs",
    schema_version = 1,
}

impl Needs {
    /// Both needs at `at`, awake: a person at genesis.
    pub(crate) const fn assigned(hunger: u16, tiredness: u16, at: WorldTime) -> Self {
        Self {
            hunger: Level::new(hunger, at, Mode::Awake),
            tiredness: Level::new(tiredness, at, Mode::Awake),
        }
    }

    /// Hunger as stored.
    pub const fn hunger(&self) -> Level {
        self.hunger
    }

    /// Tiredness as stored.
    pub const fn tiredness(&self) -> Level {
        self.tiredness
    }

    /// The mode the needs move in: whether the person was asleep at the last of sleep's facts.
    pub const fn mode(&self) -> Mode {
        self.hunger.mode()
    }

    /// `(hunger, tiredness)` at `now` under `rates`.
    pub fn current(&self, now: WorldTime, rates: &Rates) -> (u16, u16) {
        (
            current(&self.hunger, Need::Hunger, now, rates),
            current(&self.tiredness, Need::Tiredness, now, rates),
        )
    }

    /// Hunger brought to `at` and lowered by `by`, never below zero; tiredness untouched.
    #[must_use]
    pub fn fed(self, by: u32, at: WorldTime, rates: &Rates) -> Self {
        let folded = fold(&self.hunger, Need::Hunger, at, rates);
        let left = u32::from(folded.stored()).saturating_sub(by);
        let left = u16::try_from(left).unwrap_or(0);
        Self {
            hunger: Level::new(left, at, folded.mode()),
            ..self
        }
    }

    /// Both needs brought to `at`, moving in `mode` from then on.
    #[must_use]
    pub fn turned(self, mode: Mode, at: WorldTime, rates: &Rates) -> Self {
        Self {
            hunger: fold(&self.hunger, Need::Hunger, at, rates).in_mode(mode),
            tiredness: fold(&self.tiredness, Need::Tiredness, at, rates).in_mode(mode),
        }
    }
}

/// How much one of an item kind lowers hunger when eaten or drunk: the item form of the `needs:`
/// section (`ARC-285`). On the item kind; read only by this pack, never disclosed raw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nourishment {
    hunger: u16,
}

owned_component! {
    component = Nourishment,
    owner = NeedsSystem,
    component_type = "nourishment",
    schema_version = 1,
}

impl Nourishment {
    pub(crate) const fn new(hunger: u16) -> Self {
        Self { hunger }
    }

    /// Hunger one item lowers by.
    pub const fn hunger(&self) -> u16 {
        self.hunger
    }
}
