//! What the model decides, from observations built by hand (`pr-dl-b-motivated.md` §9, unit tier).
//!
//! The same discipline as `rule-controller`'s tests: the observation is assembled here, so the model is
//! tested against exactly the view it is given. Whether a running world produces such a view is the
//! integration tests' claim (`tools/cli/tests/motivated.rs`).

use mineworld_contracts::{
    Action, Affordance, EntityId, EntityType, LocalPosition, Location, Millimetres, Observation,
    PerceivedEntity, PersonId, PlaceId, Rejection, SimDuration, WorldTime,
};
use mineworld_conversation::{Talk, talk_requirement};
use mineworld_decision::Draw;
use serde_json::Value;

use crate::model::{MOTIVATED_SALT, decide};
use crate::options::Choice;
use crate::score::{Scored, combine, select};
use crate::settings::Settings;

const ME: u64 = 2;
const BOB: u64 = 4;
const CAFE: u64 = 1;

/// A pace of fifteen minutes, the social-cafe pace.
const PACE: SimDuration = SimDuration::from_seconds(900);

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

/// Noon, on the first day.
const NOON: i64 = 12 * 3_600;

/// Where the observer stands: the cafe, on the ground.
fn here() -> Location {
    let place = PlaceId::new(id(CAFE), EntityType::Place).expect("a place");
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(1_200),
        Millimetres::new(2_400),
    ))
}

/// An observer alone in the cafe: presence and nothing else disclosed, and no affordances.
fn presence_only(at: i64) -> Observation<Value> {
    let me = PerceivedEntity::new(id(ME), EntityType::Person).at(here());
    Observation::new(id(ME), WorldTime::from_seconds(at))
        .at_location(here())
        .perceiving(vec![me])
        .offering(Vec::new())
}

/// The observer and Bob in the cafe, with the server's verdict on whether Bob may be addressed.
fn with_bob(at: i64, may_talk: bool) -> Observation<Value> {
    let me = PerceivedEntity::new(id(ME), EntityType::Person).at(here());
    let bob = PerceivedEntity::new(id(BOB), EntityType::Person).at(here());
    let verdict = if may_talk {
        Affordance::available(Talk::ACTION_TYPE, Some(id(BOB)), talk_requirement())
    } else {
        Affordance::unavailable(
            Talk::ACTION_TYPE,
            Some(id(BOB)),
            talk_requirement(),
            Rejection::TooFarAway,
        )
    };
    Observation::new(id(ME), WorldTime::from_seconds(at))
        .at_location(here())
        .perceiving(vec![me, bob])
        .offering(vec![verdict])
}

fn bob() -> PersonId {
    PersonId::new(id(BOB), EntityType::Person).expect("a person")
}

fn choices_of(observation: &Observation<Value>) -> Vec<Choice> {
    crate::explain(&Settings::defaults(), observation)
        .expect("the observer is in the observation")
        .into_iter()
        .map(|entry| entry.choice)
        .collect()
}

#[test]
fn a_presence_only_observation_decides_and_the_only_choice_is_to_stay() {
    // I-DLb-4: every input is optional. With nothing disclosed but presence, the one choice is to stay,
    // and staying submits nothing.
    let observation = presence_only(NOON);
    assert_eq!(choices_of(&observation), vec![Choice::Stay]);
    let decision = decide(&Settings::defaults(), 7, PACE, &observation);
    assert_eq!(decision, None);
}

#[test]
fn a_person_the_server_will_not_let_me_address_is_never_greeted() {
    // M-DLb-5: an offer of `greet` without the server's `may_talk_to` verdict would greet Bob here.
    let refused = choices_of(&with_bob(NOON, false));
    assert!(
        !refused
            .iter()
            .any(|choice| matches!(choice, Choice::Greet(_))),
        "{refused:?}"
    );
    let allowed = choices_of(&with_bob(NOON, true));
    assert!(allowed.contains(&Choice::Greet(bob())), "{allowed:?}");
}

#[test]
fn compensation_matches_three_hand_computed_cases() {
    // k = 1: the modification is 0, so the value passes through: 1 000 × 600 / 1 000.
    assert_eq!(combine(1_000, &[600]), 600);
    // k = 2: the modification is 500. A value of 500 adjusts to 500 + 250 × 500 / 1 000 = 625, and the
    // score is 1 000 × 625 / 1 000 = 625, then 625 × 625 / 1 000 = 390 (floored from 390.6).
    assert_eq!(combine(1_000, &[500, 500]), 390);
    // k = 5: the modification is 800. A value of 500 adjusts to 500 + 400 × 500 / 1 000 = 700, and
    // 1 000 → 700 → 490 → 343 → 240 → 168 over five multiplications.
    assert_eq!(combine(1_000, &[500, 500, 500, 500, 500]), 168);
}

#[test]
fn a_zero_consideration_takes_the_whole_score_to_zero() {
    assert_eq!(combine(900, &[0, 1_000]), 0);
    assert_eq!(combine(300, &[]), 300, "no considerations score the base");
}

#[test]
fn the_top_band_draw_chooses_only_within_the_band_and_a_low_door_is_never_chosen() {
    // Scores 500 (stay) and 480 (greet) are within 100 per mille of the best; a door at 100 is not.
    // A draw over instants covers the band and never the door.
    let scored = vec![
        Scored {
            choice: Choice::Stay,
            considerations: Vec::new(),
            score: 500,
        },
        Scored {
            choice: Choice::Greet(bob()),
            considerations: Vec::new(),
            score: 480,
        },
        Scored {
            choice: Choice::Door(PlaceId::new(id(CAFE), EntityType::Place).expect("a place")),
            considerations: Vec::new(),
            score: 100,
        },
    ];
    let mut seen_stay = false;
    let mut seen_greet = false;
    for step in 0..200 {
        let observation = presence_only(NOON + step * 60);
        let draw = Draw::salted(MOTIVATED_SALT, 7, &observation);
        let chosen = select(&scored, 100, &draw, 0).expect("the band is not empty");
        match chosen {
            Choice::Stay => seen_stay = true,
            Choice::Greet(_) => seen_greet = true,
            other => panic!("{other:?} is outside the top band"),
        }
    }
    assert!(
        seen_stay && seen_greet,
        "the band is drawn over: both choices appear"
    );
}

#[test]
fn nothing_scoring_above_zero_chooses_nothing() {
    let scored = vec![Scored {
        choice: Choice::Stay,
        considerations: Vec::new(),
        score: 0,
    }];
    let observation = presence_only(NOON);
    let draw = Draw::salted(MOTIVATED_SALT, 7, &observation);
    assert_eq!(select(&scored, 100, &draw, 0), None);
}

#[test]
fn the_same_observation_and_seed_decide_the_same() {
    // ARC-27: a model is a function of its settings, seed, pace and observation.
    let observation = with_bob(NOON, true);
    let settings = Settings::defaults();
    assert_eq!(
        decide(&settings, 7, PACE, &observation),
        decide(&settings, 7, PACE, &observation)
    );
}
