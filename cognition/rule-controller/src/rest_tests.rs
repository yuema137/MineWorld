//! The rest band, decided from observations built by hand (`pr-s34d-npcs-agents-sit.md` SD-d-3, C2).
//!
//! Posture is never linked: its sit points are disclosed as a `sit-points` record and its offers as `sit`
//! affordances carrying `{ point }`, exactly what an observation carries. The keys are arbitrary strings;
//! the band names none. Claims about chance are checked across seeds.

use mineworld_contracts::{
    Action, Affordance, ComponentRecord, EntityId, EntityType, LocalPosition, Location,
    Millimetres, Observation, PerceivedEntity, PlaceId, Rejection, SimDuration, SpatialRequirement,
    WorldTime,
};
use mineworld_decision::Draw;
use mineworld_movement::{Destination, Move, WalkTo, Walking, move_offer_requirement};
use serde_json::{Value, json};

use crate::PacedRuleController;
use crate::rest::{self, Rest};

const HALL: u64 = 1;
const ME: u64 = 3;
const PACE: i64 = 900;
const NOW: i64 = 36_000;
const SEEDS: u64 = 64;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn hall() -> PlaceId {
    PlaceId::new(id(HALL), EntityType::Place).expect("a place")
}

fn at(x: i32, y: i32) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

fn record(entity: u64, component_type: &str, payload: Value) -> ComponentRecord<Value> {
    serde_json::from_value(json!({
        "entity": entity.to_string(),
        "component_type": component_type,
        "schema_version": 1,
        "payload": payload,
    }))
    .expect("a record")
}

/// A seat: its key and position, disclosed in the place's `sit-points`.
struct Point(&'static str, i32, i32);

/// A `sit { point }` offer, available or refused with `refused`.
fn sit(key: &str, refused: Option<Rejection>) -> Affordance<Value> {
    let requirement = SpatialRequirement::at_place(hall());
    let offer = match refused {
        None => Affordance::available(rest_type(rest::SIT), None, requirement),
        Some(reason) => Affordance::unavailable(rest_type(rest::SIT), None, requirement, reason),
    };
    offer.with_payload(json!({ "point": key }))
}

fn rest_type(slug: &'static str) -> mineworld_contracts::ActionTypeId {
    mineworld_contracts::ActionTypeId::from_static(slug)
}

/// Me at the origin of the hall, `walk-to` offered, the hall disclosing `points`, and `offers`.
fn view(points: &[Point], offers: Vec<Affordance<Value>>, walking: bool) -> Observation<Value> {
    view_at(NOW, points, offers, walking)
}

fn view_at(
    now: i64,
    points: &[Point],
    offers: Vec<Affordance<Value>>,
    walking: bool,
) -> Observation<Value> {
    let here = Location::in_place(hall()).with_local(at(0, 0));
    let disclosed: Vec<Value> = points
        .iter()
        .map(|Point(key, x, y)| json!({ "key": key, "at": { "x": x, "y": y }, "facing": 0 }))
        .collect();
    let mut place = PerceivedEntity::new(id(HALL), EntityType::Place);
    if !points.is_empty() {
        place = place.with_components(vec![record(HALL, rest::SIT_POINTS, json!(disclosed))]);
    }
    let mut me = PerceivedEntity::new(id(ME), EntityType::Person).at(here);
    if walking {
        me = me.with_components(vec![ComponentRecord::new::<Walking>(id(ME), json!({}))]);
    }
    let mut affordances = vec![
        Affordance::available(Move::ACTION_TYPE, None, move_offer_requirement()),
        Affordance::available(WalkTo::ACTION_TYPE, None, move_offer_requirement()),
    ];
    affordances.extend(offers);
    Observation::new(id(ME), WorldTime::from_seconds(now))
        .at_location(here)
        .perceiving(vec![place, me])
        .offering(affordances)
}

/// Every seed's band decision on `observation`, staying.
fn decisions(
    observation: &Observation<Value>,
) -> Vec<Option<Option<mineworld_contracts::ActionRequest>>> {
    (0..SEEDS)
        .map(|seed| rest::band(observation, &Draw::new(seed, observation), true))
        .collect()
}

fn walk_goal(request: &mineworld_contracts::ActionRequest) -> Option<LocalPosition> {
    let walk: WalkTo = serde_json::from_slice(request.payload().payload()).ok()?;
    match walk.to() {
        Destination::Place(location) => location.local(),
        _ => None,
    }
}

#[test]
fn an_available_seat_is_sat_on_about_half_the_time_by_its_own_offer() {
    let observation = view(
        &[Point("x.far", 4_000, 0), Point("x.near", 500, 0)],
        vec![
            sit("x.far", Some(Rejection::TooFarAway)),
            sit("x.near", None),
        ],
        false,
    );
    assert_eq!(rest::choose(&observation), Some(Rest::Sit(3)));
    let decided = decisions(&observation);
    let sat = decided.iter().filter(|decision| decision.is_some()).count();
    assert!(
        (16..=48).contains(&sat),
        "an even chance over {SEEDS} seeds, {sat} sat"
    );
    for request in decided.into_iter().flatten().flatten() {
        assert_eq!(request.payload().action_type().as_str(), rest::SIT);
        let payload: Value = serde_json::from_slice(request.payload().payload()).expect("JSON");
        assert_eq!(payload, json!({ "point": "x.near" }));
    }
}

#[test]
fn out_of_reach_the_nearest_free_seat_is_walked_to_ties_by_key() {
    let observation = view(
        &[
            Point("c", 3_000, 0),
            Point("b", 0, 3_000),
            Point("a", 5_000, 0),
        ],
        vec![
            sit("a", Some(Rejection::TooFarAway)),
            sit("b", Some(Rejection::TooFarAway)),
            sit("c", Some(Rejection::TooFarAway)),
        ],
        false,
    );
    // b and c are both 3 m away: b wins by key.
    let Some(Rest::SeekSeat(seat)) = rest::choose(&observation) else {
        panic!("a seat to seek");
    };
    assert_eq!(seat.key, "b");
    let walks: Vec<_> = decisions(&observation)
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    assert!(!walks.is_empty(), "some seed sets off");
    for request in walks {
        assert_eq!(walk_goal(&request), Some(at(0, 3_000)));
    }
}

#[test]
fn a_seat_not_offered_is_out_of_offer_range_and_is_walked_to() {
    // Q-S34c3-3's "not offered" row: disclosed, no offer at all.
    let observation = view(&[Point("far.1", 9_000, 0)], Vec::new(), false);
    let Some(Rest::SeekSeat(seat)) = rest::choose(&observation) else {
        panic!("a seat to seek");
    };
    assert_eq!((seat.key.as_str(), seat.at), ("far.1", at(9_000, 0)));
}

#[test]
fn a_taken_crowded_forbidden_or_busy_seat_is_never_chosen() {
    for reason in [
        Rejection::TargetUnavailable,
        Rejection::PermissionDenied,
        Rejection::Busy,
        Rejection::NoSupportedInteraction,
    ] {
        let observation = view(
            &[Point("taken", 400, 0), Point("other", 6_000, 0)],
            vec![
                sit("taken", Some(reason.clone())),
                sit("other", Some(reason.clone())),
            ],
            false,
        );
        assert_eq!(rest::choose(&observation), None, "{reason:?}");
        assert!(
            decisions(&observation).iter().all(Option::is_none),
            "{reason:?}: the band takes no part"
        );
    }
}

#[test]
fn walking_or_away_from_my_day_rests_nowhere() {
    let points = [Point("a", 400, 0)];
    let walking = view(&points, vec![sit("a", None)], true);
    assert_eq!(rest::choose(&walking), None);
    let standing = view(&points, vec![sit("a", None)], false);
    for seed in 0..SEEDS {
        assert_eq!(
            rest::band(&standing, &Draw::new(seed, &standing), false),
            None
        );
    }
}

#[test]
fn without_posture_the_band_takes_no_part() {
    let observation = view(&[], Vec::new(), false);
    assert_eq!(rest::choose(&observation), None);
    assert!(decisions(&observation).iter().all(Option::is_none));
}

#[test]
fn the_offered_band_never_draws_sit_or_stand() {
    // Seated: posture offers one complete `stand`. Standing beside a free seat: a complete `sit`. Over
    // seeds and instants the paced rule submits `stand` never, and `sit` only through the rest band —
    // which this observation, away from its seat's draw, still decides by name.
    let stand = Affordance::available(rest_type(rest::STAND), None, SpatialRequirement::NONE)
        .with_payload(json!({}));
    for seed in 0..SEEDS {
        for instant in 0..8 {
            let seated = view_at(NOW + instant * PACE, &[], vec![stand.clone()], false);
            let decided =
                PacedRuleController::new(seed, SimDuration::from_seconds(PACE)).decide(&seated);
            assert!(
                decided
                    .is_none_or(|request| request.payload().action_type().as_str() != rest::STAND),
                "seed {seed} stood up by chance"
            );
        }
    }
}
