//! `motivated` with needs and sleep (`pr-s35-nc-controllers.md` SD-Nc-7): the asleep rung, the `sleep`
//! choice, the four need considerations, their parameters, and that nothing changes without `needs`.
//! The `needs` record is written as the world discloses it, by its slug (F-Nc-3).

use mineworld_contracts::{
    Action, ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType, LocalPosition,
    Location, Millimetres, Observation, PerceivedEntity, PersonId, PlaceId, SimDuration,
    SpatialRequirement, Tag, Tags, WorldTime,
};
use mineworld_conversation::{Talk, talk_requirement};
use mineworld_decision::ModelParameters;
use mineworld_sleep::{Asleep, Sleep, SleepTerms, Wake};
use serde_json::{Value, json};

use crate::considerations::Consideration;
use crate::model::decide;
use crate::options::Choice;
use crate::parameters::MotivatedParameters;
use crate::score::Scored;
use crate::settings::Settings;

const ME: u64 = 2;
const BOB: u64 = 4;
const HOME: u64 = 20;
const PACE: SimDuration = SimDuration::from_seconds(900);
const STRIDE: SimDuration = SimDuration::from_seconds(30);
const SEEDS: u64 = 64;
/// 23:00 on the first day.
const NIGHT: i64 = 23 * 3_600;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn here() -> Location {
    let place = PlaceId::new(id(HOME), EntityType::Place).expect("a place");
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(1_200),
        Millimetres::new(2_400),
    ))
}

/// Me at home (a `residential` place), Bob beside me, at `at`: my needs if any, whether I and Bob
/// sleep, `sleep` offered (or `wake`, asleep), and a complete `eat` of item 7.
struct View {
    at: i64,
    needs: Option<(u16, u16)>,
    asleep: bool,
    bob_asleep: bool,
}

impl View {
    fn build(&self) -> Observation<Value> {
        let mut mine = vec![ComponentRecord::new::<SleepTerms>(
            id(ME),
            json!({ "shortest_minutes": 15, "longest_minutes": 840 }),
        )];
        if let Some((hunger, tiredness)) = self.needs {
            mine.push(
                serde_json::from_value(json!({
                    "entity": ME.to_string(), "component_type": "needs", "schema_version": 1,
                    "payload": { "hunger": hunger, "tiredness": tiredness,
                                 "rates": { "rest_per_hour": 125 },
                                 "nourishment": [{ "item": { "entity": "7", "entity_type": "item" },
                                                   "hunger": 400 }] },
                }))
                .expect("a record"),
            );
        }
        let asleep = |who: u64| {
            ComponentRecord::new::<Asleep>(id(who), json!({ "asleep": true, "since": 0 }))
        };
        if self.asleep {
            mine.push(asleep(ME));
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(here())
            .with_components(mine);
        let mut bob = PerceivedEntity::new(id(BOB), EntityType::Person).at(here());
        if self.bob_asleep {
            bob = bob.with_components(vec![asleep(BOB)]);
        }
        let home = PerceivedEntity::new(id(HOME), EntityType::Place)
            .with_tags(Tags::new([Tag::new("residential").expect("a tag")]));
        let mut offers = vec![
            Affordance::available(Talk::ACTION_TYPE, Some(id(BOB)), talk_requirement()),
            Affordance::available(
                ActionTypeId::from_static("eat"),
                None,
                SpatialRequirement::NONE,
            )
            .with_payload(json!({ "item": { "entity": "7", "entity_type": "item" } })),
        ];
        if self.asleep {
            offers.push(
                Affordance::available(Wake::ACTION_TYPE, None, SpatialRequirement::NONE)
                    .with_payload(json!({})),
            );
        } else {
            offers.push(Affordance::available(
                Sleep::ACTION_TYPE,
                None,
                SpatialRequirement::NONE,
            ));
        }
        Observation::new(id(ME), WorldTime::from_seconds(self.at))
            .at_location(here())
            .perceiving(vec![me, bob, home])
            .offering(offers)
    }
}

fn view(at: i64, needs: Option<(u16, u16)>) -> View {
    View {
        at,
        needs,
        asleep: false,
        bob_asleep: false,
    }
}

fn scored(observation: &Observation<Value>) -> Vec<Scored> {
    crate::explain(&Settings::defaults(), observation).expect("the observer is in it")
}

fn score_of(entries: &[Scored], wanted: impl Fn(&Choice) -> bool) -> u32 {
    entries
        .iter()
        .find(|entry| wanted(&entry.choice))
        .map_or(0, |entry| entry.score)
}

fn carries(entries: &[Scored], consideration: Consideration) -> bool {
    entries.iter().any(|entry| {
        entry
            .considerations
            .iter()
            .any(|(carried, _)| *carried == consideration)
    })
}

#[test]
fn without_needs_no_choice_or_consideration_is_added() {
    let entries = scored(&view(NIGHT, None).build());
    assert!(
        !entries
            .iter()
            .any(|entry| matches!(entry.choice, Choice::Sleep(_))),
        "sleep is offered, but nobody here has needs"
    );
    for consideration in [
        Consideration::Hungry,
        Consideration::Sated,
        Consideration::Tired,
        Consideration::Wakeful,
    ] {
        assert!(!carries(&entries, consideration), "{consideration:?}");
    }
}

#[test]
fn a_tired_person_at_night_scores_sleep_above_staying_and_mostly_sleeps() {
    let tired = view(NIGHT, Some((0, 900))).build();
    let entries = scored(&tired);
    let sleep = score_of(&entries, |choice| matches!(choice, Choice::Sleep(_)));
    let stay = score_of(&entries, |choice| *choice == Choice::Stay);
    assert!(sleep > stay, "sleep {sleep} vs stay {stay}: {entries:?}");
    // 900 ÷ 125 per hour → 432 minutes, with no punctual part to wake for.
    assert!(
        entries
            .iter()
            .any(|entry| entry.choice == Choice::Sleep(432))
    );
    let slept = (0..SEEDS)
        .filter_map(|seed| decide(&Settings::defaults(), seed, PACE, STRIDE, &tired))
        .filter(|request| *request.action_type() == Sleep::ACTION_TYPE)
        .count();
    assert!(slept * 2 > SEEDS as usize, "{slept} of {SEEDS}");
    // Rested, sleep scores nothing.
    let rested = scored(&view(NIGHT, Some((0, 0))).build());
    assert_eq!(
        score_of(&rested, |choice| matches!(choice, Choice::Sleep(_))),
        0
    );
}

#[test]
fn hunger_raises_eating_and_tiredness_lowers_small_talk() {
    let fed = scored(&view(12 * 3_600, Some((0, 0))).build());
    let hungry = scored(&view(12 * 3_600, Some((900, 0))).build());
    let eat = |entries: &[Scored]| score_of(entries, |choice| matches!(choice, Choice::Offered(1)));
    assert!(
        eat(&hungry) > eat(&fed),
        "{} vs {}",
        eat(&hungry),
        eat(&fed)
    );
    let bob = PersonId::new(id(BOB), EntityType::Person).expect("a person");
    let greet = |entries: &[Scored]| score_of(entries, |choice| *choice == Choice::Greet(bob));
    let tired = scored(&view(12 * 3_600, Some((0, 900))).build());
    assert!(greet(&tired) < greet(&fed));
    assert_eq!(greet(&tired), 0, "wakeful is 0 at 900");
}

#[test]
fn a_sleeper_with_no_commute_due_scores_nothing_and_submits_nothing() {
    // The wake itself is `rule`'s shared rung (`NeedsPolicy::asleep`), tested there.
    let mut asleep = view(NIGHT, Some((900, 100)));
    asleep.asleep = true;
    let observation = asleep.build();
    for seed in 0..SEEDS {
        assert_eq!(
            decide(&Settings::defaults(), seed, PACE, STRIDE, &observation),
            None,
            "seed {seed}"
        );
    }
}

#[test]
fn nobody_greets_or_approaches_a_sleeper() {
    let mut view = view(12 * 3_600, Some((0, 0)));
    view.bob_asleep = true;
    let entries = scored(&view.build());
    assert!(
        !entries
            .iter()
            .any(|entry| matches!(entry.choice, Choice::Greet(_) | Choice::Approach(_))),
        "{entries:?}"
    );
}

#[test]
fn the_need_curves_and_the_sleep_base_are_parameters() {
    let decode = |text: &str| {
        let mut deserializer = serde_json::Deserializer::from_str(text);
        MotivatedParameters::decode(&mut deserializer)
    };
    let parameters = decode(
        r#"{"base": {"sleep": 400}, "curve": {"tired": [[0, 0], [1000, 1000]], "hungry": [[0, 0], [1000, 500]]}}"#,
    )
    .expect("accepted");
    let settings = parameters.settings();
    assert_eq!(settings.base.sleep, 400);
    assert_eq!(settings.curves.tired.at(500), 500);
    assert_eq!(settings.curves.hungry.at(1_000), 500);
    let refused = decode(r#"{"curve": {"wakeful": [[0, 0], [1001, 0]]}}"#)
        .expect_err("outside the input")
        .to_string();
    assert!(refused.contains("outside the input 0 … 1000"), "{refused}");
    assert!(decode(r#"{"curve": {"sated": [[0, 1000], [1000, 0]]}}"#).is_ok());
    let defaults = Settings::defaults();
    assert_eq!(
        (
            defaults.curves.hungry.at(600),
            defaults.curves.tired.at(600),
            defaults.curves.wakeful.at(600),
            defaults.curves.sated.at(900),
            defaults.base.sleep,
        ),
        (800, 700, 300, 0, 1_000)
    );
}

/// Me on a street (place 30) whose one doorway leads to a `shop` (place 31), hungry at 950, holding
/// `held` nourishment.
fn street_with_a_shop(held: u16) -> Observation<Value> {
    let street = PlaceId::new(id(30), EntityType::Place).expect("a place");
    let shop = PlaceId::new(id(31), EntityType::Place).expect("a place");
    let at = Location::in_place(street).with_local(LocalPosition::on_ground(
        Millimetres::new(1_000),
        Millimetres::new(1_000),
    ));
    let mut door = serde_json::to_value(mineworld_movement::Passage::new(shop, None, None))
        .expect("serializes");
    door["to_tags"] = json!(["shop"]);
    let site = PerceivedEntity::new(id(30), EntityType::Place).with_components(vec![
        ComponentRecord::new::<mineworld_movement::Passages>(id(30), json!({ "leads_to": [door] })),
    ]);
    let nourishment: Vec<Value> = if held > 0 {
        vec![json!({ "item": { "entity": "7", "entity_type": "item" }, "hunger": held })]
    } else {
        Vec::new()
    };
    let needs: ComponentRecord<Value> = serde_json::from_value(json!({
        "entity": ME.to_string(), "component_type": "needs", "schema_version": 1,
        "payload": { "hunger": 950, "tiredness": 100, "rates": { "rest_per_hour": 125 },
                     "nourishment": nourishment },
    }))
    .expect("a record");
    let me = PerceivedEntity::new(id(ME), EntityType::Person)
        .at(at)
        .with_components(vec![needs]);
    Observation::new(id(ME), WorldTime::from_seconds(12 * 3_600))
        .at_location(at)
        .perceiving(vec![me, site])
}

#[test]
fn a_hungry_person_holding_no_food_is_drawn_to_a_shop_door() {
    let shop = PlaceId::new(id(31), EntityType::Place).expect("a place");
    let door = |entries: &[Scored]| {
        entries
            .iter()
            .find(|entry| entry.choice == Choice::Door(shop))
            .cloned()
            .expect("the shop door is a choice")
    };
    let empty_handed = door(&scored(&street_with_a_shop(0)));
    assert!(
        empty_handed
            .considerations
            .iter()
            .any(|(carried, _)| *carried == Consideration::Hungry),
        "{empty_handed:?}"
    );
    let holding = door(&scored(&street_with_a_shop(400)));
    assert!(
        !holding
            .considerations
            .iter()
            .any(|(carried, _)| *carried == Consideration::Hungry),
        "{holding:?}"
    );
    assert!(empty_handed.score > holding.score);
}
