//! The two seat options, from observations built by hand (`pr-s34d-npcs-agents-sit.md` SD-d-4, C3).
//!
//! Posture is never linked: the place discloses a `sit-points` record and posture's offers are `sit` and
//! `stand` affordances carrying their payloads, exactly what an observation carries. The keys are
//! arbitrary; the model names none.

use mineworld_contracts::{
    ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType, LocalPosition, Location,
    Millimetres, Observation, PerceivedEntity, PlaceId, Rejection, SimDuration, SpatialRequirement,
    WorldTime,
};
use mineworld_decision::ModelParameters;
use mineworld_movement::{Destination, WalkTo};
use serde_json::{Value, json};

use crate::model::decide;
use crate::options::Choice;
use crate::parameters::MotivatedParameters;
use crate::settings::Settings;

const CAFE: u64 = 1;
const ME: u64 = 2;
const NOON: i64 = 12 * 3_600;
const PACE: SimDuration = SimDuration::from_seconds(900);
const STRIDE: SimDuration = SimDuration::from_seconds(30);
const SEEDS: u64 = 256;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn cafe() -> PlaceId {
    PlaceId::new(id(CAFE), EntityType::Place).expect("a place")
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

fn offer(slug: &'static str, payload: Value, refused: Option<Rejection>) -> Affordance<Value> {
    let kind = ActionTypeId::from_static(slug);
    let requirement = SpatialRequirement::at_place(cafe());
    match refused {
        None => Affordance::available(kind, None, requirement),
        Some(reason) => Affordance::unavailable(kind, None, requirement, reason),
    }
    .with_payload(payload)
}

/// Me alone in the cafe at (0, 0), `walk-to` offered, the cafe disclosing `seats` and `offers` offered.
fn cafe_with(seats: &[(&str, i32, i32)], offers: Vec<Affordance<Value>>) -> Observation<Value> {
    let here = Location::in_place(cafe()).with_local(at(0, 0));
    let disclosed: Vec<Value> = seats
        .iter()
        .map(|(key, x, y)| json!({ "key": key, "at": { "x": x, "y": y }, "facing": 0 }))
        .collect();
    let mut place = PerceivedEntity::new(id(CAFE), EntityType::Place);
    if !seats.is_empty() {
        place = place.with_components(vec![record(CAFE, "sit-points", json!(disclosed))]);
    }
    let me = PerceivedEntity::new(id(ME), EntityType::Person).at(here);
    let mut affordances = vec![Affordance::available(
        ActionTypeId::from_static("walk-to"),
        None,
        SpatialRequirement::NONE,
    )];
    affordances.extend(offers);
    Observation::new(id(ME), WorldTime::from_seconds(NOON))
        .at_location(here)
        .perceiving(vec![place, me])
        .offering(affordances)
}

fn choices(settings: &Settings, observation: &Observation<Value>) -> Vec<Choice> {
    crate::explain(settings, observation)
        .expect("the observer is in the observation")
        .into_iter()
        .map(|entry| entry.choice)
        .collect()
}

fn submitted(settings: &Settings, observation: &Observation<Value>) -> Vec<String> {
    (0..SEEDS)
        .filter_map(|seed| decide(settings, seed, PACE, STRIDE, observation))
        .map(|request| request.payload().action_type().as_str().to_owned())
        .collect()
}

#[test]
fn a_seat_in_reach_is_one_sit_option_and_its_offer_is_never_a_generic_one() {
    let observation = cafe_with(
        &[("p.near", 300, 0)],
        vec![offer("sit", json!({ "point": "p.near" }), None)],
    );
    let listed = choices(&Settings::defaults(), &observation);
    assert!(listed.contains(&Choice::Sit(1)), "{listed:?}");
    assert!(
        !listed
            .iter()
            .any(|choice| matches!(choice, Choice::Offered(_))),
        "{listed:?}"
    );
    assert!(
        submitted(&Settings::defaults(), &observation).contains(&"sit".to_owned()),
        "some seed sits"
    );
}

#[test]
fn a_seat_out_of_reach_or_not_offered_is_one_seek_seat_option_nearest_first() {
    let observation = cafe_with(
        &[
            ("p.far", 6_000, 0),
            ("p.mid", 2_000, 0),
            ("p.unoffered", 1_500, 0),
        ],
        vec![
            offer(
                "sit",
                json!({ "point": "p.far" }),
                Some(Rejection::TooFarAway),
            ),
            offer(
                "sit",
                json!({ "point": "p.mid" }),
                Some(Rejection::TooFarAway),
            ),
        ],
    );
    let seeking: Vec<String> = choices(&Settings::defaults(), &observation)
        .into_iter()
        .filter_map(|choice| match choice {
            Choice::SeekSeat(seat) => Some(seat.key),
            _ => None,
        })
        .collect();
    assert_eq!(seeking, vec!["p.unoffered".to_owned()]);
}

#[test]
fn a_taken_seat_and_a_seated_persons_stand_are_never_chosen() {
    let taken = cafe_with(
        &[("p.taken", 300, 0)],
        vec![offer(
            "sit",
            json!({ "point": "p.taken" }),
            Some(Rejection::TargetUnavailable),
        )],
    );
    let listed = choices(&Settings::defaults(), &taken);
    assert!(
        !listed
            .iter()
            .any(|choice| matches!(choice, Choice::Sit(_) | Choice::SeekSeat(_))),
        "{listed:?}"
    );
    // Seated, posture offers one complete `stand`: no option is built from it, so nobody bobs (F-d-7).
    let seated = cafe_with(&[], vec![offer("stand", json!({}), None)]);
    assert!(!submitted(&Settings::defaults(), &seated).contains(&"stand".to_owned()));
}

#[test]
fn a_decision_list_that_weighs_both_options_zero_makes_nobody_sit() {
    let mut deserializer =
        serde_json::Deserializer::from_str(r#"{"base": {"sit": 0, "seek-seat": 0}}"#);
    let parameters = MotivatedParameters::decode(&mut deserializer).expect("accepted");
    let settings = parameters.settings();
    assert_eq!((settings.base.sit, settings.base.seek_seat), (0, 0));
    let near = cafe_with(
        &[("p.near", 300, 0)],
        vec![offer("sit", json!({ "point": "p.near" }), None)],
    );
    assert!(!submitted(settings, &near).contains(&"sit".to_owned()));
    let far = cafe_with(
        &[("p.far", 6_000, 0)],
        vec![offer(
            "sit",
            json!({ "point": "p.far" }),
            Some(Rejection::TooFarAway),
        )],
    );
    let walked_to_seat = (0..SEEDS).any(|seed| {
        decide(settings, seed, PACE, STRIDE, &far).is_some_and(|request| {
            serde_json::from_slice::<WalkTo>(request.payload().payload()).is_ok_and(|walk| {
                matches!(walk.to(), Destination::Place(to) if to.local() == Some(at(6_000, 0)))
            })
        })
    });
    assert!(!walked_to_seat);
}

#[test]
fn without_posture_no_seat_option_appears() {
    let listed = choices(&Settings::defaults(), &cafe_with(&[], Vec::new()));
    assert!(
        !listed
            .iter()
            .any(|choice| matches!(choice, Choice::Sit(_) | Choice::SeekSeat(_))),
        "{listed:?}"
    );
}

#[test]
fn a_seated_person_where_the_day_keeps_them_may_stay_seated_and_seeks_no_seat() {
    // Q-S34d1-2, ruled 2026-11-27: `stay-seated`, weighted like `sit`.
    let observation = cafe_with(&[("p.mine", 0, 0), ("p.other", 3_000, 0)], Vec::new());
    let mut entities = observation.entities().to_vec();
    let me = entities.pop().expect("me").with_components(vec![record(
        ME,
        "posture-state",
        json!({ "seated": { "place": CAFE.to_string(), "point": "p.mine", "since": 0 } }),
    )]);
    entities.push(me);
    let seated = Observation::new(id(ME), WorldTime::from_seconds(NOON))
        .at_location(*observation.self_location().expect("here"))
        .perceiving(entities)
        .offering(observation.affordances().to_vec());
    let listed = choices(&Settings::defaults(), &seated);
    assert!(listed.contains(&Choice::StaySeated), "{listed:?}");
    assert!(
        !listed
            .iter()
            .any(|choice| matches!(choice, Choice::SeekSeat(_) | Choice::Sit(_))),
        "{listed:?}"
    );
}
