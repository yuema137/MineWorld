//! A walk in progress, as the observer sees their own: whether they are walking, and the next stride
//! (`step-11-bodies.md` §21 SD-N16; `DECISIONS.md` `ARC-75`).
//!
//! Movement holds the walk — its destination and route — as the walker's `walking` component and
//! discloses it to whoever perceives the walker. The walker's sender keeps asking for strides while
//! that record is disclosed, one `walk-step` per stride, at its own cadence. This module is the
//! controller's half of that: it reads the record and asks for the stride. It never plans, never measures
//! a distance and takes no draw.

use mineworld_contracts::{Action, ActionRecord, ActionRequest, Component, Observation};
use mineworld_movement::{WalkStep, Walking};
use serde_json::Value;

/// Whether the observation discloses the observer's own walk: what a host reads to keep stepping a
/// seat, so that no host decodes a pack's component itself.
pub fn walks(observation: &Observation<Value>) -> bool {
    observation
        .entity(observation.observer())
        .is_some_and(|me| {
            me.components()
                .iter()
                .any(|record| *record.component_type() == Walking::COMPONENT_TYPE)
        })
}

/// The next stride of the observer's own walk: a `walk-step` when their walk is disclosed and the
/// world offers `walk-step`, otherwise nothing.
pub(crate) fn step(observation: &Observation<Value>) -> Option<ActionRequest> {
    let offered = observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == WalkStep::ACTION_TYPE && affordance.is_available()
    });
    (offered && walks(observation)).then(|| {
        let payload = serde_json::to_vec(&WalkStep::default())
            .expect("an empty payload is JSON-representable");
        ActionRequest::new(
            observation.observer(),
            ActionRecord::new::<WalkStep>(payload),
        )
    })
}
