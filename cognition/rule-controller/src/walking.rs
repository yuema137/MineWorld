//! A walk in progress, as the observer sees their own: whether they are walking, and the next stride
//! (`step-11-bodies.md` §21 SD-N16; `DECISIONS.md` `ARC-75`).
//!
//! Movement holds the walk — its destination and route — as the walker's `walking` component and
//! discloses it to whoever perceives the walker. The walker's sender keeps asking for strides while
//! that record is disclosed, one `walk-step` per stride, at its own cadence. This module is the
//! controller's half of that: it reads the record and asks for the stride. It never plans, never measures
//! a distance and takes no draw.

use mineworld_contracts::{Action, ActionRecord, ActionRequest, Component, Observation};
use mineworld_movement::{Destination, WalkStep, WalkTo, Walking};
use serde::Deserialize;
use serde_json::Value;

/// The part of movement's disclosed `walking` record this controller reads: where the walk goes.
///
/// A private copy of movement's wire shape (`{ destination, next }`), because movement does not
/// export its disclosed type. Finding F-12n2-1 (step-11 §21.15): movement should export a typed
/// disclosed walk, and this copy should go.
#[derive(Deserialize)]
struct DisclosedWalk {
    destination: Destination,
}

/// What a walking band chose: a request, or to carry on with the walk already under way.
pub(crate) enum Walked {
    /// Ask for this walk.
    Ask(ActionRequest),
    /// My own disclosed walk already goes there: ask for nothing (step-11 N-D14).
    Continue,
}

impl Walked {
    /// The request to make, if any.
    pub(crate) fn request(self) -> Option<ActionRequest> {
        match self {
            Self::Ask(request) => Some(request),
            Self::Continue => None,
        }
    }
}

/// Where my own disclosed walk goes, if I am walking and the record reads.
fn heading(observation: &Observation<Value>) -> Option<Destination> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| *record.component_type() == Walking::COMPONENT_TYPE)?;
    let walk: DisclosedWalk =
        serde_json::from_value(record.payload_for::<Walking>().ok()?.clone()).ok()?;
    Some(walk.destination)
}

/// A `walk-to` request for `to`, if the world offers `walk-to` to me; [`Walked::Continue`] when my own
/// walk already goes there. The route is the world's: this names a destination and nothing else.
pub(crate) fn walk_to(observation: &Observation<Value>, to: Destination) -> Option<Walked> {
    if heading(observation) == Some(to) {
        return Some(Walked::Continue);
    }
    let offered = observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == WalkTo::ACTION_TYPE && affordance.is_available()
    });
    offered.then(|| {
        let payload = serde_json::to_vec(&WalkTo::new(to))
            .expect("a request payload is JSON-representable by construction");
        Walked::Ask(ActionRequest::new(
            observation.observer(),
            ActionRecord::new::<WalkTo>(payload),
        ))
    })
}

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
