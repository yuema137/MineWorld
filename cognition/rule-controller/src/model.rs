//! `rule`: the paced rule controller as a decision model, the first and the default
//! (`pr-dl-a-decision-list.md` SD-DLa-2; `docs/MODULE_SPEC.md` §5.1).
//!
//! Zero-sized. Each consult builds `PacedRuleController::new(seed, pace)` from the host's [`Consult`],
//! with this seat's commute policy and the host's stride cadence, and asks it — the controller is
//! a few words and one short tag list (the needs policy's `shop_tags`), so building it costs little. Its parameters are three blocks, `commute`
//! (`ARC-282`, `pr-le-leave-early.md` §0.4), `boss` (`ARC-287`) and `needs` (`ARC-286`), every field
//! defaulted: with no decision list every seat is this model with the default policies, and in a world
//! with no punctual part, no manager and no `needs`/`sleep` they are inert, so every byte of such a
//! world is what it was before.

use core::fmt;

use mineworld_contracts::ActionRequest;
use mineworld_decision::{Consult, DecisionModel, ModelParameters};
use schemars::JsonSchema;
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserializer, Serialize};

use crate::PacedRuleController;
use crate::boss::Boss;
use crate::commute::Commute;
use crate::needs::NeedsPolicy;

/// The paced rule controller, `rule` in `decisions.yaml`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rule;

/// `rule`'s parameters, as `decisions.yaml` writes them under `models: { rule: … }`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct RuleParameters {
    /// When to set out for a punctual part of the day.
    pub commute: Commute,
    /// When a manager penalizes lateness, and how hard (`ARC-287`).
    pub boss: Boss,
    /// When to eat, go to bed and wake for work, in a world with `needs` and `sleep` (`ARC-286`).
    pub needs: NeedsPolicy,
}

/// The keys of the block.
const KEYS: &[&str] = &["commute", "boss", "needs"];

impl ModelParameters for RuleParameters {
    fn decode<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(Fields)
    }
}

struct Fields;

impl<'de> Visitor<'de> for Fields {
    type Value = RuleParameters;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a mapping of rule parameters ({})",
            KEYS.join(", ")
        )
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<RuleParameters, A::Error> {
        let mut parameters = RuleParameters::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            match key.as_str() {
                "commute" => parameters.commute = map.next_value()?,
                "boss" => parameters.boss = map.next_value()?,
                "needs" => parameters.needs = map.next_value()?,
                _ => return Err(A::Error::unknown_field(&key, KEYS)),
            }
            seen.push(key);
        }
        Ok(parameters)
    }
}

impl Rule {
    fn controller(parameters: &RuleParameters, consult: &Consult<'_>) -> PacedRuleController {
        PacedRuleController::new(consult.seed.raw(), consult.pace)
            .commuting(parameters.commute, consult.stride)
            .bossing(parameters.boss)
            .needing(parameters.needs.clone())
    }
}

impl DecisionModel for Rule {
    const ID: &'static str = "rule";
    const PACKAGE: mineworld_packages::Package = crate::PACKAGE;
    type Parameters = RuleParameters;

    fn decide(&self, parameters: &RuleParameters, consult: &Consult<'_>) -> Option<ActionRequest> {
        Self::controller(parameters, consult).decide(consult.observation)
    }

    fn step(&self, parameters: &RuleParameters, consult: &Consult<'_>) -> Option<ActionRequest> {
        Self::controller(parameters, consult).step(consult.observation)
    }
}
