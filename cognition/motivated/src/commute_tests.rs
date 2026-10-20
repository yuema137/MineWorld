//! `motivated`'s commute: a scored option, not a hard rung (`pr-le-leave-early.md` §0.4, C5).

use mineworld_contracts::{
    Action, Affordance, ComponentRecord, EntityId, EntityType, Location, Millimetres, Observation,
    PerceivedEntity, PersonId, PlaceId, ProcessId, SimDuration, WorldTime,
};
use mineworld_conversation::{ConversationHistory, Heard, Talk, Utterance, talk_requirement};
use mineworld_decision::ModelParameters;
use mineworld_movement::{Destination, WalkTo, Way, Ways, move_offer_requirement};
use mineworld_rule_controller::commute::Mode;
use mineworld_schedule::{Agenda, AgendaLabel, DisclosedAgenda, NextPart};
use serde_json::Value;

use crate::model::decide;
use crate::parameters::MotivatedParameters;
use crate::settings::Settings;

const ME: u64 = 2;
const BOB: u64 = 4;
const CAFE: u64 = 1;
const HOME: u64 = 20;
const PACE: SimDuration = SimDuration::from_seconds(900);
const STRIDE: SimDuration = SimDuration::from_seconds(30);
/// 05:30 on day 2; the way is 12 strides, so the lead is 840 s at the default policy.
const FROM: i64 = 86_400 + 5 * 3_600 + 1_800;
const LEAVE_BY: i64 = FROM - 840;
const SEEDS: u64 = 64;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn place(raw: u64) -> PlaceId {
    PlaceId::new(id(raw), EntityType::Place).expect("a place")
}

/// At home, Bob beside me having just spoken; the next part at the café, `punctual` or not, or absent.
fn view(at: i64, next: Option<bool>) -> Observation<Value> {
    let here = Location::in_place(place(HOME));
    let agenda = Agenda::new(
        place(HOME),
        AgendaLabel::new("home").expect("a label"),
        WorldTime::from_seconds(FROM - 12 * 3_600),
        WorldTime::from_seconds(FROM),
        ProcessId::from_raw(1),
    );
    let disclosed = DisclosedAgenda {
        agenda,
        next: next.map(|punctual| NextPart {
            place: place(CAFE),
            label: AgendaLabel::new("work").expect("a label"),
            from: WorldTime::from_seconds(FROM),
            punctual,
        }),
    };
    let mut history = ConversationHistory::default();
    history.remember(Heard::new(
        PersonId::new(id(BOB), EntityType::Person).expect("a person"),
        WorldTime::from_seconds(at - 10),
        Utterance::new("a coffee before work?").expect("an utterance"),
    ));
    let ways = Ways(vec![Way {
        place: place(CAFE),
        doors: 1,
        length: Millimetres::new(13_400),
    }]);
    let me = PerceivedEntity::new(id(ME), EntityType::Person)
        .at(here)
        .with_components(vec![
            ComponentRecord::new::<Agenda>(id(ME), serde_json::to_value(&disclosed).expect("json")),
            ComponentRecord::new::<Ways>(id(ME), serde_json::to_value(&ways).expect("json")),
            ComponentRecord::new::<ConversationHistory>(
                id(ME),
                serde_json::to_value(&history).expect("json"),
            ),
        ]);
    let bob = PerceivedEntity::new(id(BOB), EntityType::Person).at(here);
    Observation::new(id(ME), WorldTime::from_seconds(at))
        .at_location(here)
        .perceiving(vec![me, bob])
        .offering(vec![
            Affordance::available(Talk::ACTION_TYPE, Some(id(BOB)), talk_requirement()),
            Affordance::available(WalkTo::ACTION_TYPE, None, move_offer_requirement()),
        ])
}

fn walks_to_cafe(request: &mineworld_contracts::ActionRequest) -> bool {
    request
        .payload()
        .payload_for::<WalkTo>()
        .ok()
        .and_then(|payload| serde_json::from_slice::<WalkTo>(payload).ok())
        .is_some_and(|walk| walk.to() == Destination::Place(Location::in_place(place(CAFE))))
}

fn commuters(settings: &Settings, at: i64) -> usize {
    (0..SEEDS)
        .filter_map(|seed| decide(settings, seed, PACE, STRIDE, &view(at, Some(true))))
        .filter(walks_to_cafe)
        .count()
}

#[test]
fn once_it_is_time_to_leave_every_seed_sets_out_and_none_stops_to_chat() {
    let settings = Settings::defaults();
    assert_eq!(
        commuters(&settings, LEAVE_BY),
        SEEDS as usize,
        "urgency 1 000"
    );
    assert_eq!(commuters(&settings, FROM - 1), SEEDS as usize);
    let replies = (0..SEEDS)
        .filter_map(|seed| decide(&settings, seed, PACE, STRIDE, &view(LEAVE_BY, Some(true))))
        .filter(|request| *request.action_type() == Talk::ACTION_TYPE)
        .count();
    assert_eq!(replies, 0, "unhurried at 0: no small talk");
}

#[test]
fn well_before_the_lead_nobody_sets_out_and_at_the_boundary_mode_nobody_plans() {
    let settings = Settings::defaults();
    assert_eq!(
        commuters(&settings, LEAVE_BY - 2 * 900),
        0,
        "urgency 0: the commute scores nothing"
    );
    let mut at_boundary = Settings::defaults();
    at_boundary.commute.mode = Mode::AtBoundary;
    assert_eq!(commuters(&at_boundary, LEAVE_BY), 0);
}

#[test]
fn with_no_punctual_part_every_decision_is_what_it_was() {
    // I-LE-4: a part that is not punctual decides exactly as no next part at all.
    let settings = Settings::defaults();
    for at in (FROM - 4 * 3_600..FROM + 3_600).step_by(300) {
        for seed in 0..SEEDS {
            assert_eq!(
                decide(&settings, seed, PACE, STRIDE, &view(at, Some(false))),
                decide(&settings, seed, PACE, STRIDE, &view(at, None)),
                "seed {seed} at {at}"
            );
        }
    }
}

fn decoded(text: &str) -> Result<MotivatedParameters, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    MotivatedParameters::decode(&mut deserializer)
}

#[test]
fn the_commute_block_base_and_curves_decode_and_refuse_by_name_and_position() {
    let parameters = decoded(
        r#"{"commute": {"slack": 0, "margin_minutes": 10}, "base": {"commute": 800},
            "curve": {"departure": [[0, 100], [1000, 1000]]}}"#,
    )
    .expect("accepted");
    let settings = parameters.settings();
    assert_eq!(
        (settings.commute.slack, settings.commute.margin_minutes),
        (0, 10)
    );
    assert_eq!(settings.base.commute, 800);
    assert_eq!(settings.curves.departure.at(0), 100);

    let error = decoded("{\n  \"commute\": {\n    \"slack\": 2001\n  }\n}").expect_err("refused");
    assert!(
        error.to_string().contains("'slack' is 0 … 2000, not 2001"),
        "{error}"
    );
    assert!(
        (3..=4).contains(&error.line()),
        "positioned inside the commute block, where the decoder stood: {error}"
    );
    let error = decoded(r#"{"commute": {"cap_minutes": 0}}"#).expect_err("refused");
    assert!(
        error
            .to_string()
            .contains("'cap_minutes' is 15 … 480, not 0"),
        "{error}"
    );
    let error =
        decoded(r#"{"curve": {"unhurried": [[0, 1000], [1001, 0]]}}"#).expect_err("refused");
    assert!(
        error.to_string().contains("outside the input 0 … 1000"),
        "{error}"
    );
}
