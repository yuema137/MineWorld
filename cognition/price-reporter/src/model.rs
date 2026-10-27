//! One consult of the price clerk (`pr-s31d-reporter-policies.md` §5.2): publish at a slot, else survey
//! when offered, else `rule`'s daily life with its default parameters.
//!
//! Bands 1 and 2 take no draw, and band 3 is `rule` itself, so a seat with no book and no publish offer
//! decides exactly as `rule` (AC-d-4). A refused publish is journaled by the world and not retried within
//! its slot: the next slot tries again (SD-S31d-3).

use mineworld_contracts::{Action, ActionRequest, Observation};
use mineworld_decision::{Consult, DecisionModel};
use mineworld_notices::{NoticeBody, PublishNotice};
use mineworld_price_survey::{PriceBook, Survey};
use mineworld_rule_controller::price::slots::publish_due;
use mineworld_rule_controller::{Rule, RuleParameters, act, read};
use serde_json::Value;

use crate::ReporterParameters;
use crate::suggest::suggest;

/// The price clerk, `price-reporter` in `decisions.yaml`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PriceReporter;

/// Band 1: the bulletin, when a slot is due, the server offers `publish-notice` and the book has lines.
fn publish(parameters: &ReporterParameters, consult: &Consult<'_>) -> Option<ActionRequest> {
    let observation = consult.observation;
    let due = publish_due(
        observation.at().seconds(),
        consult.pace.seconds(),
        parameters.first_minute,
        parameters.last_minute,
        parameters.every_minutes,
    );
    if !due || !read::available::<PublishNotice>(observation, None) {
        return None;
    }
    let me = observation.observer();
    let book: PriceBook = read::disclosed(observation.entity(me)?)?;
    let lines = suggest(&book, parameters);
    if lines.is_empty() {
        return None;
    }
    let notice = PublishNotice::new(
        parameters.topic.clone(),
        NoticeBody::Prices(lines),
        parameters.valid_minutes.saturating_mul(60),
    );
    Some(act::request(me, &notice))
}

/// Band 2: the `survey` the world offers complete and available, as its own request.
fn survey(observation: &Observation<Value>) -> Option<ActionRequest> {
    observation
        .affordances()
        .iter()
        .find(|affordance| {
            *affordance.action_type() == Survey::ACTION_TYPE
                && affordance.target().is_none()
                && affordance.is_available()
                && affordance.payload().is_some()
        })?
        .request(observation.observer(), encoded)
}

fn encoded(payload: &Value) -> Vec<u8> {
    serde_json::to_vec(payload).expect("a JSON value is JSON-representable")
}

impl DecisionModel for PriceReporter {
    const ID: &'static str = "price-reporter";
    const PACKAGE: mineworld_packages::Package = crate::PACKAGE;
    type Parameters = ReporterParameters;

    fn decide(
        &self,
        parameters: &ReporterParameters,
        consult: &Consult<'_>,
    ) -> Option<ActionRequest> {
        publish(parameters, consult)
            .or_else(|| survey(consult.observation))
            .or_else(|| Rule.decide(&RuleParameters::default(), consult))
    }

    fn step(
        &self,
        _parameters: &ReporterParameters,
        consult: &Consult<'_>,
    ) -> Option<ActionRequest> {
        Rule.step(&RuleParameters::default(), consult)
    }
}
