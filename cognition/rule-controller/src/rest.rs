//! The rest band: a person who stays where they are sits down where the world lets them
//! (`pr-s34d-npcs-agents-sit.md` SD-d-3, S34-d1; `DECISIONS.md` `ARC-147`).
//!
//! The world discloses facts and posture gives the verdicts; this band reads both and computes neither:
//!
//! ```text
//! my place's sit-points     disclosed by posture to whoever perceives the place: each point's key and at
//! a sit { point } offer     one complete offer per sit point, available or refused with posture's verdict
//! ```
//!
//! It acts only when I am not walking and my day keeps me here (no agenda, or at my agenda's place):
//!
//! ```text
//! 1  an available sit offer                    attempt the first, in observation order
//! 2  else the disclosed seats, nearest first   walk to the first whose offer is refused only
//!    (ties by key)                             too_far_away, or that is not offered at all (posture offers
//!                                              sit only within its offer range, Q-S34c3-3)
//!    a seat refused for any other reason       never chosen: taken, crowded, forbidden, busy
//! both on draw REST_DRAW below SITS_BELOW
//! ```
//!
//! Seated and staying, it holds the consult on the same draw (Q-S34d1-1, ruled 2026-11-27), so a spell lasts
//! about two consults. It never decides `stand`: a seated person gets up by setting off somewhere through
//! another band, which posture turns into `stood-up { moved }` (Q-S34-4). `sit` and `stand` are decided by name
//! (`paced.rs` `DECIDED_BY_NAME`), so the offered band never draws them (`ARC-151`) — that alone ends the
//! chance `stand` people bobbed on (F-d-2).
//!
//! **Inert without posture (I-S34-6).** With no `sit` offer and no disclosed sit point this band takes no
//! draw and decides nothing, so a world without posture decides exactly as before.
//!
//! `posture` is read by its slugs and JSON, never through the pack's crate: that crate links bodies, which
//! links a market pack (`ARC-35` check 2), as `needs` is read (`needs.rs`).

use mineworld_contracts::{
    ActionRequest, Affordance, LocalPosition, Millimetres, Observation, Rejection,
};
use mineworld_decision::Draw;
use mineworld_movement::Destination;
use serde::Deserialize;
use serde_json::Value;

use crate::act::walk_to;
use crate::read::{distance, walks};

/// The action type posture provides to sit, by its slug.
pub const SIT: &str = "sit";

/// The action type posture provides to stand, by its slug.
pub const STAND: &str = "stand";

/// The component type posture discloses a place's sit points on, by its slug.
pub const SIT_POINTS: &str = "sit-points";

/// The component type posture discloses a seated person's row on, by its slug.
pub const POSTURE_STATE: &str = "posture-state";

/// Out of 100, at a consult where this band could act: how often the person sits, or sets off for a free
/// seat. An even chance, so a person staying near a free seat sits within two consults on average. A
/// literal, set once and never derived from what it produces (`ARC-23` rule 2), like `FOLLOWS_AGENDA`.
pub(crate) const SITS_BELOW: u64 = 50;

/// The draw index this band uses: new, independent of every other band's (0-6, 8-15).
pub(crate) const REST_DRAW: u64 = 16;

/// A sit point of my place, as posture discloses it: its key, and where a seated person is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    /// The point's key, the name a `sit` payload carries.
    pub key: String,
    /// The seated person's position on the place's floor.
    pub at: LocalPosition,
}

#[derive(Deserialize)]
struct DisclosedPoint {
    key: String,
    at: DisclosedSpot,
}

#[derive(Deserialize)]
struct DisclosedSpot {
    x: i32,
    y: i32,
}

/// The sit points my place discloses, in the order disclosed; none in a world without posture.
pub fn seats(observation: &Observation<Value>) -> Vec<Seat> {
    let Some(here) = observation.self_location() else {
        return Vec::new();
    };
    let Some(record) = observation
        .entity(here.place().entity_id())
        .and_then(|place| {
            place
                .components()
                .iter()
                .find(|record| record.component_type().as_str() == SIT_POINTS)
        })
    else {
        return Vec::new();
    };
    let points: Vec<DisclosedPoint> =
        serde_json::from_value(record.payload().clone()).unwrap_or_default();
    points
        .into_iter()
        .map(|point| Seat {
            key: point.key,
            at: LocalPosition::on_ground(
                Millimetres::new(point.at.x),
                Millimetres::new(point.at.y),
            ),
        })
        .collect()
}

/// The sit point a complete `sit` offer names, by its payload's `point`; none for any other offer.
pub fn sit_point(affordance: &Affordance<Value>) -> Option<&str> {
    if affordance.action_type().as_str() != SIT {
        return None;
    }
    affordance.payload()?.get("point")?.as_str()
}

/// What the rest rule chose, before any draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rest {
    /// Attempt the available complete `sit` offer at this index of the observation's affordances.
    Sit(usize),
    /// Walk to this free seat, out of reach or out of offer range.
    SeekSeat(Seat),
}

/// The first available complete `sit` offer, as its index in the observation's affordances.
pub fn available_sit(observation: &Observation<Value>) -> Option<usize> {
    observation
        .affordances()
        .iter()
        .position(|affordance| affordance.is_available() && sit_point(affordance).is_some())
}

/// The nearest disclosed seat worth walking to: its offer refused only `too_far_away`, or not offered.
/// Ties by key. A seat refused for any other reason is never chosen.
pub fn seat_to_seek(observation: &Observation<Value>) -> Option<Seat> {
    let from = observation.self_location()?.local()?;
    let mut seats: Vec<Seat> = seats(observation)
        .into_iter()
        .filter(|seat| {
            let offer = observation
                .affordances()
                .iter()
                .find(|affordance| sit_point(affordance) == Some(seat.key.as_str()));
            match offer {
                None => true,
                Some(offer) => offer.unavailable_reason() == Some(&Rejection::TooFarAway),
            }
        })
        .collect();
    seats.sort_by(|a, b| {
        distance(from, a.at)
            .cmp(&distance(from, b.at))
            .then_with(|| a.key.cmp(&b.key))
    });
    seats.into_iter().next()
}

/// The rest rule's choice at this observation, before any draw: an available sit, else a free seat to walk
/// to; nothing when I am walking or the world offers no seat.
pub fn choose(observation: &Observation<Value>) -> Option<Rest> {
    // Seated, posture offers no `sit`: every disclosed seat would read "not offered", so a seated person
    // seeks no seat (Q-S34d1-1 ruling; see `seated`).
    if walks(observation) || seated(observation) {
        return None;
    }
    available_sit(observation)
        .map(Rest::Sit)
        .or_else(|| seat_to_seek(observation).map(Rest::SeekSeat))
}

/// Whether posture discloses me seated: my own `posture-state` record, read by slug.
pub fn seated(observation: &Observation<Value>) -> bool {
    observation
        .entity(observation.observer())
        .is_some_and(|me| {
            me.components()
                .iter()
                .any(|record| record.component_type().as_str() == POSTURE_STATE)
        })
}

/// The request a rest choice submits: the offer's own request, or a `walk-to` the seat's position.
/// `Some(None)` when my walk already goes there.
pub fn request(observation: &Observation<Value>, rest: &Rest) -> Option<Option<ActionRequest>> {
    match rest {
        Rest::Sit(index) => {
            let offer = observation.affordances().get(*index)?;
            Some(offer.request(observation.observer(), encoded))
        }
        Rest::SeekSeat(seat) => {
            let here = *observation.self_location()?;
            Some(walk_to(observation, Destination::Place(here.with_local(seat.at)))?.request())
        }
    }
}

/// The rest band of the paced rule (SD-d-3): `Some` when it decides this consult.
pub(crate) fn band(
    observation: &Observation<Value>,
    draw: &Draw,
    staying: bool,
) -> Option<Option<ActionRequest>> {
    if !staying {
        return None;
    }
    // Seated and staying: stay seated on the same draw, before the walking roll (operator ruling
    // 2026-11-27 on Q-S34d1-1, "坐姿按推荐"). No new constant, no new draw index.
    if seated(observation) {
        return (draw.below(100, REST_DRAW) < SITS_BELOW).then_some(None);
    }
    let rest = choose(observation)?;
    if draw.below(100, REST_DRAW) >= SITS_BELOW {
        return None;
    }
    // `Some(None)`: already on my way there, so nothing to ask and nothing else this consult.
    request(observation, &rest)
}

/// This controller's payload encoding for an offered affordance, as `offered.rs`'s.
fn encoded(payload: &Value) -> Vec<u8> {
    serde_json::to_vec(payload).expect("a JSON value is JSON-representable")
}
