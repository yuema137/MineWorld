//! A manager's policy about lateness, as data (`DECISIONS.md` `ARC-287`; `pr-le-leave-early.md` §0.5).
//!
//! Where a world turns wage deductions on and an employer has a manager, the world offers that manager
//! a **complete** `penalize` per late member of staff, its amount the most the world allows, and tells
//! the manager about each lateness (`late_staff` on their own disclosed `employment` record: how many
//! times within the window, and how serious). This module reads both by their slugs — never through the
//! pack's crate, which this controller may not name (`ARC-35` check 2) — and decides as a policy says:
//!
//! ```text
//! penalize_from   notice | warning | final | never      the least serious lateness penalized; warning
//! share           1 … 1 000 ‰ of the offered bound      how hard; 1 000
//! decision        the first offered penalty, in observation order, whose level ≥ penalize_from;
//!                 amount = max(1, bound × share ÷ 1 000)
//! ```
//!
//! No draw: a strict boss and a lenient one differ by data. In a world that offers no `penalize`
//! nothing here applies and nothing is read.

use core::fmt;

use mineworld_contracts::{ActionRequest, Affordance, Observation, PersonId};
use schemars::JsonSchema;
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// The action type a manager is offered: employment's `penalize`, by its slug.
pub const PENALIZE: &str = "penalize";

/// The component type a manager is told about late staff on: employment's `employment`, by its slug.
const EMPLOYMENT: &str = "employment";

/// How serious a lateness is, as the world states it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// Below the warning threshold.
    Notice,
    /// At or above it.
    Warning,
    /// At or above the final threshold.
    Final,
}

/// The least serious lateness a boss penalizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PenalizeFrom {
    /// Every lateness.
    Notice,
    /// A warning or worse.
    #[default]
    Warning,
    /// Only a final one.
    Final,
    /// None: this boss only complains.
    Never,
}

impl PenalizeFrom {
    /// Whether a lateness of `level` is penalized.
    pub fn admits(self, level: Level) -> bool {
        match self {
            Self::Notice => true,
            Self::Warning => level >= Level::Warning,
            Self::Final => level == Level::Final,
            Self::Never => false,
        }
    }
}

/// A boss policy (§0.5). Every field may be omitted and keeps its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Boss {
    /// `notice`, `warning`, `final` or `never`: the least serious lateness penalized.
    pub penalize_from: PenalizeFrom,
    /// Per mille of the offered bound taken, 1 … 1 000.
    #[schemars(range(min = 1, max = 1_000))]
    pub share: u32,
}

impl Default for Boss {
    fn default() -> Self {
        Self {
            penalize_from: PenalizeFrom::Warning,
            share: 1_000,
        }
    }
}

/// The keys of a boss block.
const KEYS: &[&str] = &["penalize_from", "share"];

impl<'de> Deserialize<'de> for Boss {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(BossFields)
    }
}

struct BossFields;

impl<'de> Visitor<'de> for BossFields {
    type Value = Boss;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a boss block ({})", KEYS.join(", "))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Boss, A::Error> {
        let mut boss = Boss::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            match key.as_str() {
                "penalize_from" => boss.penalize_from = map.next_value()?,
                "share" => {
                    let share: i64 = map.next_value()?;
                    if !(1..=1_000).contains(&share) {
                        return Err(A::Error::custom(format!(
                            "'share' is 1 … 1000, not {share}"
                        )));
                    }
                    boss.share = u32::try_from(share).map_err(A::Error::custom)?;
                }
                _ => return Err(A::Error::unknown_field(&key, KEYS)),
            }
            seen.push(key);
        }
        Ok(boss)
    }
}

/// One member of staff whose lateness the world tells a manager about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct LateStaff {
    /// Who.
    pub employee: PersonId,
    /// Latenesses within the window, this one included.
    pub count: u32,
    /// How serious.
    pub level: Level,
    /// The most a penalty may take.
    pub bound: u64,
}

/// The part of the disclosed `employment` record a boss reads.
#[derive(Deserialize)]
struct Staff {
    #[serde(default)]
    late_staff: Vec<LateStaff>,
}

/// An offered penalty: the affordance, and the lateness it is for.
#[derive(Debug, Clone, Copy)]
pub struct Offered<'a> {
    /// The complete `penalize` affordance.
    pub affordance: &'a Affordance<Value>,
    /// Its index in the observation's affordances.
    pub index: usize,
    /// The lateness, as the manager was told it.
    pub late: LateStaff,
}

/// Whether an affordance is a `penalize`.
pub fn is_penalty(affordance: &Affordance<Value>) -> bool {
    affordance.action_type().as_str() == PENALIZE
}

/// Every available complete `penalize` offered to me, with the lateness each is for, in observation
/// order; empty in a world that offers none.
pub fn offered(observation: &Observation<Value>) -> Vec<Offered<'_>> {
    let mut found: Vec<Offered<'_>> = Vec::new();
    let candidates = observation
        .affordances()
        .iter()
        .enumerate()
        .filter(|(_, affordance)| is_penalty(affordance) && affordance.is_available());
    let mut staff: Option<Vec<LateStaff>> = None;
    for (index, affordance) in candidates {
        let Some(employee) = affordance
            .payload()
            .and_then(|payload| payload.get("employee"))
            .and_then(|value| serde_json::from_value::<PersonId>(value.clone()).ok())
        else {
            continue;
        };
        let told = staff.get_or_insert_with(|| late_staff(observation));
        if let Some(late) = told.iter().find(|late| late.employee == employee) {
            found.push(Offered {
                affordance,
                index,
                late: *late,
            });
        }
    }
    found
}

/// The late staff my own disclosed `employment` record names.
fn late_staff(observation: &Observation<Value>) -> Vec<LateStaff> {
    observation
        .entity(observation.observer())
        .and_then(|me| {
            me.components()
                .iter()
                .find(|record| record.component_type().as_str() == EMPLOYMENT)
        })
        .and_then(|record| serde_json::from_value::<Staff>(record.payload().clone()).ok())
        .map(|staff| staff.late_staff)
        .unwrap_or_default()
}

/// The request for `offered` at `amount`: the offered payload, with its amount replaced.
pub fn request(
    observation: &Observation<Value>,
    offered: &Offered<'_>,
    amount: u64,
) -> Option<ActionRequest> {
    offered
        .affordance
        .request(observation.observer(), |payload| {
            let mut payload = payload.clone();
            if let Some(map) = payload.as_object_mut() {
                map.insert("amount".to_owned(), Value::from(amount));
            }
            serde_json::to_vec(&payload).expect("a JSON value is JSON-representable")
        })
}

impl Boss {
    /// The penalty this policy submits at this consult, if any: no draw.
    pub fn decide(&self, observation: &Observation<Value>) -> Option<ActionRequest> {
        if self.penalize_from == PenalizeFrom::Never {
            return None;
        }
        let chosen = offered(observation)
            .into_iter()
            .find(|offered| self.penalize_from.admits(offered.late.level))?;
        let amount = (chosen.late.bound * u64::from(self.share) / 1_000).max(1);
        request(observation, &chosen, amount)
    }
}
