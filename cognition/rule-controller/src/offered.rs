//! The paced rule attempts what it is offered (`DECISIONS.md` `ARC-34`, step-10 SD-10).
//!
//! A System Pack installed after this crate was written appears in an observation as affordances,
//! and the controller cannot build its requests: it was never compiled against the pack's types. A
//! **complete affordance** carries the exact request the offering pack would accept, so the controller
//! can attempt it without knowing what it is — and that is the whole of this band:
//!
//! ```text
//! the available complete affordances, in observation order,
//!   less the action types the caller decides by name             none → this band takes no part
//! draw OFFER_DRAW below ATTEMPTS_OFFERED (out of 100)           otherwise → this band takes no part
//! the one chosen by draw OFFERED_CHOICE_DRAW                    submitted through Affordance::request
//! ```
//!
//! Read only from the observation, like everything here (`ARC-27`): whether an affordance is
//! available is the server's verdict, read and never computed, and the request is the affordance's
//! own — this module names no action type and imports no pack's type.
//!
//! **What the rule decides by name is not drawn again here** (`DECISIONS.md` `ARC-151`). Since S27-b,
//! group-activity offers leave, join, accept and decline complete, and the paced rule already decides each
//! of them in its social band. Drawing them here too gave a person who had just decided not to leave a
//! second, independent chance to leave, and activities lasted half as long (F-S27b-5). The caller passes
//! the types it decides by name and they are left out of the pool, so an observation is decided exactly as
//! if those offers carried no payload — the controller's input before S27-b — and every other complete
//! offer keeps its share.
//!
//! The constants are frozen for S9 (step-10 I-9; `ARC-35`, point 6). They were fixed against a synthetic
//! pack offering a complete affordance to every person everywhere (step-10 C-C6, §9.3) and are never
//! retuned for a world that behaves badly: the remedy for that is in what its packs offer.

use mineworld_contracts::{ActionRequest, ActionTypeId, Affordance, Observation};
use mineworld_decision::Draw;
use serde_json::Value;

/// Out of 100: how often a consult that is offered something complete attempts one of the offers.
pub(crate) const ATTEMPTS_OFFERED: u64 = 20;

/// The draw index deciding whether to attempt: new, independent of every other band's (0–6, 8–13).
pub(crate) const OFFER_DRAW: u64 = 14;

/// The draw index choosing which of the available complete affordances is attempted.
pub(crate) const OFFERED_CHOICE_DRAW: u64 = 15;

/// One of the available complete affordances in `observation` whose type is not in `decided_by_name`, as
/// the request it names — sometimes.
///
/// `None` when no such affordance is available, which is every observation of a world whose packs offer
/// nothing complete beyond what the caller decides by name: then no draw decides anything and the paced
/// rule decides exactly as before.
pub(crate) fn attempt(
    observation: &Observation<Value>,
    draw: &Draw,
    decided_by_name: &[ActionTypeId],
    excluded: &dyn Fn(&Affordance<Value>) -> bool,
) -> Option<ActionRequest> {
    let offered: Vec<&Affordance<Value>> = observation
        .affordances()
        .iter()
        // A `penalize` is the boss policy's (`boss.rs`), never a band's chance pick (`ARC-287`).
        .filter(|affordance| {
            affordance.is_available()
                && affordance.payload().is_some()
                && !crate::boss::is_penalty(affordance)
        })
        .filter(|affordance| !decided_by_name.contains(affordance.action_type()))
        // A purchase a policy says is beyond what I spend by chance (`fetch.rs`, S35 N-h round 3).
        .filter(|affordance| !excluded(affordance))
        .collect();
    if offered.is_empty() || draw.below(100, OFFER_DRAW) >= ATTEMPTS_OFFERED {
        return None;
    }
    let chosen = draw.below(offered.len() as u64, OFFERED_CHOICE_DRAW);
    offered
        .get(usize::try_from(chosen).ok()?)?
        .request(observation.observer(), encoded)
}

/// This controller's payload encoding, as for every request it builds: JSON bytes.
fn encoded(payload: &Value) -> Vec<u8> {
    serde_json::to_vec(payload).expect("a JSON value is JSON-representable")
}
