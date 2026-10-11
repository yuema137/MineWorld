//! The paced rule follows its day: where the observer's own Agenda says to be
//! (`step-09-social.md` §4.3.2, SD-15 as refined by QC-6; `DECISIONS.md` `ARC-32`).
//!
//! The schedule moves nobody; this is the controller choosing to walk where its day says, by asking
//! movement's `walk-to` for the agenda's place, then its strides one at a time — which is all a
//! schedule may ever cause. Movement plans the way, through as many doorways as it takes (`ARC-75`).
//! Read only from the observation (`ARC-27`):
//!
//! ```text
//! my own Agenda    disclosed to me by schedule: the place my day says, and for what
//! my location      where perception says I stand
//! my own walk      disclosed to me by movement: if it already goes there, nothing is asked again
//! ```
//!
//! An observation with no Agenda decides exactly what it decided before agendas existed: no draw is
//! taken and nothing is suppressed.

use mineworld_contracts::{Component, Observation};
use mineworld_schedule::Agenda;
use serde_json::Value;

use crate::paced::Draw;

/// Out of 100, at a consult away from the agenda's place: how often the person heads there. Below
/// 100 so that someone on their way still sometimes greets, invites or wanders — a literal, never
/// derived from what it produces (`ARC-23` rule 2).
pub(crate) const FOLLOWS_AGENDA: u64 = 90;

/// The draw index this module uses: new, independent of the walking scheme's 0–6 and the social
/// initiative's 8–12.
const AGENDA_DRAW: u64 = 13;

/// The observer's own Agenda, decoded with schedule's type, if the observation discloses one.
pub(crate) fn own(observation: &Observation<Value>) -> Option<Agenda> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| *record.component_type() == Agenda::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<Agenda>().ok()?.clone()).ok()
}

/// Whether perception places the observer where the agenda says.
pub(crate) fn is_there(observation: &Observation<Value>, agenda: &Agenda) -> bool {
    observation
        .self_location()
        .is_some_and(|here| here.place() == agenda.place())
}

/// Whether, at this consult, the person heads for the agenda's place.
pub(crate) fn follows(draw: &Draw) -> bool {
    draw.below(100, AGENDA_DRAW) < FOLLOWS_AGENDA
}
