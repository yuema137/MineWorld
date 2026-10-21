//! The explanation of one consult: every offered choice with its considerations and score, the table the
//! unit tests use as their oracle (`pr-dl-b-motivated.md` §5.3, step-24 §6.1).

use mineworld_contracts::Observation;
use serde_json::Value;

use crate::options::Context;
use crate::score::{Scored, score_all};
use crate::settings::Settings;

/// Every choice the observation offers at this instant, scored, or nothing when the observer is not in it.
pub fn explain(settings: &Settings, observation: &Observation<Value>) -> Option<Vec<Scored>> {
    let context = Context::new(observation, settings)?;
    Some(score_all(&context, settings))
}
