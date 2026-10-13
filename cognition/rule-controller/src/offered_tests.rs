//! The paced rule's offer band, decided from observations built by hand (step-10 C-C4, `ARC-34`).
//!
//! The action offered here is `ring`, which no crate this controller depends on defines: its type is
//! a bare name and its payload a JSON value, exactly what an observation carries for a pack the
//! controller was never compiled against. Claims are checked across seeds and instants, because the
//! band is a seeded draw.

use mineworld_contracts::{
    Action, ActionRequest, ActionTypeId, Affordance, EntityId, EntityType, LocalPosition, Location,
    Millimetres, Observation, PerceivedEntity, PlaceId, Rejection, SimDuration, SpatialRequirement,
    WorldTime,
};
use mineworld_conversation::{Talk, talk_requirement};
use mineworld_movement::{Move, WalkTo, move_offer_requirement};
use serde_json::{Value, json};

use crate::PacedRuleController;
use crate::offered;
use crate::paced::Draw;

const HALL: u64 = 1;
const ME: u64 = 3;
const BOB: u64 = 4;
const PACE: i64 = 900;
const NOW: i64 = 36_000;
const SEEDS: u64 = 64;
/// Consults per seed: with SEEDS, enough that a 20 % band and a 20 % greeting both certainly occur.
const INSTANTS: i64 = 16;

const RING: ActionTypeId = ActionTypeId::from_static("ring");

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn in_hall(x: i32, y: i32) -> Location {
    Location::in_place(PlaceId::new(id(HALL), EntityType::Place).expect("a place")).with_local(
        LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y)),
    )
}

fn controller(seed: u64) -> PacedRuleController {
    PacedRuleController::new(seed, SimDuration::from_seconds(PACE))
}

fn ring(bell: &str) -> Value {
    json!({ "bell": bell })
}

fn complete(bell: &str) -> Affordance<Value> {
    Affordance::available(RING, None, SpatialRequirement::NONE).with_payload(ring(bell))
}

fn refused(bell: &str) -> Affordance<Value> {
    Affordance::unavailable(
        RING,
        None,
        SpatialRequirement::NONE,
        Rejection::PreconditionFailed,
    )
    .with_payload(ring(bell))
}

/// Me in the hall with Bob, whom I may talk to, `move` and `walk-to` offered (incomplete, as movement
/// offers them), and `extra` offered as well.
fn hall(at: i64, extra: Vec<Affordance<Value>>) -> Observation<Value> {
    let mut affordances = vec![
        Affordance::available(Move::ACTION_TYPE, None, move_offer_requirement()),
        Affordance::available(WalkTo::ACTION_TYPE, None, move_offer_requirement()),
        Affordance::available(Talk::ACTION_TYPE, Some(id(BOB)), talk_requirement()),
    ];
    affordances.extend(extra);
    Observation::new(id(ME), WorldTime::from_seconds(at))
        .at_location(in_hall(0, 0))
        .perceiving(vec![
            PerceivedEntity::new(id(HALL), EntityType::Place),
            PerceivedEntity::new(id(ME), EntityType::Person).at(in_hall(0, 0)),
            PerceivedEntity::new(id(BOB), EntityType::Person).at(in_hall(1_200, 0)),
        ])
        .offering(affordances)
}

/// Every decision over SEEDS × INSTANTS for observations offering `extra`.
fn decisions(extra: &[Affordance<Value>]) -> Vec<(ActionRequest, Observation<Value>)> {
    let mut decided = Vec::new();
    for seed in 0..SEEDS {
        for k in 0..INSTANTS {
            let observation = hall(NOW + k * PACE, extra.to_vec());
            if let Some(request) = controller(seed).decide(&observation) {
                decided.push((request, observation));
            }
        }
    }
    decided
}

fn rang(request: &ActionRequest) -> Option<Value> {
    (*request.action_type() == RING)
        .then(|| serde_json::from_slice(request.payload().payload()).expect("JSON bytes"))
}

/// Only an affordance the server says is available is attempted: an unavailable complete affordance
/// is never submitted, whatever the seed, while an available one beside it is.
#[test]
fn only_available_complete_affordances_are_attempted() {
    let decided = decisions(&[refused("low"), complete("high"), refused("cracked")]);
    let rung: Vec<Value> = decided
        .iter()
        .filter_map(|(request, _)| rang(request))
        .collect();
    assert!(!rung.is_empty(), "the available offer is attempted");
    assert!(
        rung.iter().all(|payload| *payload == ring("high")),
        "{rung:?}"
    );

    let decided = decisions(&[refused("low"), refused("high")]);
    assert!(
        decided.iter().all(|(request, _)| rang(request).is_none()),
        "nothing available, nothing attempted"
    );
}

/// What is submitted is exactly the request the chosen affordance names — its type, its target, its
/// payload — and every available choice is chosen by some seed.
#[test]
fn the_request_is_exactly_the_offer() {
    let targeted = Affordance::available(RING, Some(id(BOB)), SpatialRequirement::NONE)
        .with_payload(ring("hand"));
    let offers = vec![complete("low"), complete("high"), targeted];
    let mut seen = Vec::new();
    for (request, observation) in decisions(&offers) {
        let Some(payload) = rang(&request) else {
            continue;
        };
        let named: Vec<ActionRequest> = observation
            .affordances()
            .iter()
            .filter_map(|affordance| {
                affordance.request(id(ME), |value| serde_json::to_vec(value).unwrap())
            })
            .collect();
        assert!(
            named.contains(&request),
            "{request:?} is not a request any offer names"
        );
        assert_eq!(request.actor(), id(ME));
        if !seen.contains(&payload) {
            seen.push(payload);
        }
    }
    for choice in [ring("low"), ring("high"), ring("hand")] {
        assert!(seen.contains(&choice), "{choice} is never chosen: {seen:?}");
    }
}

/// Where complete affordances are offered, people still greet: the band draws its own indices, so it
/// takes a share of consults rather than the greeting band's share.
#[test]
fn greetings_still_happen_where_offers_are_made() {
    let decided = decisions(&[complete("low"), complete("high")]);
    let rings = decided
        .iter()
        .filter(|(request, _)| rang(request).is_some())
        .count();
    let greetings = decided
        .iter()
        .filter(|(request, _)| {
            *request.action_type() == Talk::ACTION_TYPE && request.target() == Some(id(BOB))
        })
        .count();
    assert!(rings > 0, "offers are attempted");
    assert!(greetings > 0, "and greetings still happen beside them");
}

/// An affordance without a payload is an action this controller would have to know to attempt; one it
/// does not know is never attempted.
#[test]
fn incomplete_affordances_of_unknown_actions_are_never_attempted() {
    let incomplete = Affordance::available(RING, None, SpatialRequirement::NONE);
    let decided = decisions(&[incomplete]);
    assert!(!decided.is_empty(), "the person still does things");
    assert!(
        decided
            .iter()
            .all(|(request, _)| *request.action_type() != RING),
        "an action offered without its request is never attempted"
    );
}

/// With nothing complete available the band decides nothing: it returns nothing for any seed, so the
/// rest of the paced rule draws and decides exactly as before it existed.
#[test]
fn no_complete_affordance_no_new_draw_decides_anything() {
    let nothing_complete = [
        vec![],
        vec![Affordance::available(RING, None, SpatialRequirement::NONE)],
        vec![refused("low")],
    ];
    for extra in nothing_complete {
        for seed in 0..SEEDS {
            for k in 0..INSTANTS {
                let observation = hall(NOW + k * PACE, extra.clone());
                assert_eq!(
                    offered::attempt(&observation, &Draw::new(seed, &observation)),
                    None
                );
            }
        }
    }
}
