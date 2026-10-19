//! `rule`'s needs policy and bands, decided from observations built by hand (`pr-s35-nc-controllers.md`
//! SD-Nc-1 … SD-Nc-6, AC-Nc-2). The `needs` record is written as the world discloses it — slug and JSON —
//! since this controller may not name the pack's crate (F-Nc-3).

use mineworld_contracts::{
    Action, ActionRequest, ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType,
    Location, Millimetres, Observation, PerceivedEntity, PlaceId, ProcessId, SimDuration,
    SpatialRequirement, WorldTime,
};
use mineworld_conversation::{Talk, talk_requirement};
use mineworld_decision::ModelParameters;
use mineworld_movement::{WalkTo, Way, Ways, move_offer_requirement};
use mineworld_schedule::{Agenda, AgendaLabel, DisclosedAgenda, NextPart};
use mineworld_sleep::{Asleep, Sleep, SleepTerms, Wake};
use serde_json::{Value, json};

use crate::commute::Commute;
use crate::needs::{ClockTime, NeedsPolicy, Night, sleep_minutes};
use crate::{PacedRuleController, RuleParameters};

const ME: u64 = 3;
const BOB: u64 = 4;
const CAFE: u64 = 1;
const HOME: u64 = 20;
const PACE: i64 = 900;
const STRIDE: i64 = 30;
const SEEDS: u64 = 64;
/// When the punctual part begins: 05:30 on day 2.
const FROM: i64 = 86_400 + 5 * 3_600 + 1_800;
/// 13.4 m and one doorway at a 30 s stride, default policy: a lead of 840 s (`commute_tests.rs`).
const LEAD: i64 = 840;
/// 23:30 on day 1: inside everybody's night (22:00 plus at most 60 minutes).
const LATE_EVENING: i64 = 23 * 3_600 + 1_800;
/// 14:00 on day 1: outside the night.
const AFTERNOON: i64 = 14 * 3_600;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn place(raw: u64) -> PlaceId {
    PlaceId::new(id(raw), EntityType::Place).expect("a place")
}

fn seconds(value: i64) -> SimDuration {
    SimDuration::from_seconds(value)
}

fn item(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "item" })
}

/// A record as the world discloses it, by its component type's slug.
fn record(entity: u64, component_type: &str, payload: Value) -> ComponentRecord<Value> {
    serde_json::from_value(json!({
        "entity": entity.to_string(),
        "component_type": component_type,
        "schema_version": 1,
        "payload": payload,
    }))
    .expect("a record")
}

/// What I am told and offered: needs (hunger, tiredness, rest per hour, nourishment by item), whether
/// I and Bob sleep, my next part (punctual or not), the bounds of sleep here, and the offers.
#[derive(Clone)]
struct View {
    at: i64,
    needs: Option<(u16, u16)>,
    rest_per_hour: u16,
    nourishment: Vec<(u64, u16)>,
    asleep: bool,
    bob_asleep: bool,
    punctual: bool,
    terms: (u16, u16),
    sleep_offered: bool,
    meals: Vec<(&'static str, u64)>,
}

impl View {
    fn at(at: i64) -> Self {
        Self {
            at,
            needs: Some((0, 0)),
            rest_per_hour: 125,
            nourishment: Vec::new(),
            asleep: false,
            bob_asleep: false,
            punctual: true,
            terms: (15, 840),
            sleep_offered: true,
            meals: Vec::new(),
        }
    }

    fn feeling(mut self, hunger: u16, tiredness: u16) -> Self {
        self.needs = Some((hunger, tiredness));
        self
    }

    fn without_needs(mut self) -> Self {
        self.needs = None;
        self
    }

    fn build(&self) -> Observation<Value> {
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
            next: Some(NextPart {
                place: place(CAFE),
                label: AgendaLabel::new("work").expect("a label"),
                from: WorldTime::from_seconds(FROM),
                punctual: self.punctual,
            }),
        };
        let ways = Ways(vec![Way {
            place: place(CAFE),
            doors: 1,
            length: Millimetres::new(13_400),
        }]);
        let mut mine = vec![
            ComponentRecord::new::<Agenda>(id(ME), serde_json::to_value(&disclosed).expect("ok")),
            ComponentRecord::new::<Ways>(id(ME), serde_json::to_value(&ways).expect("ok")),
            ComponentRecord::new::<SleepTerms>(
                id(ME),
                json!({ "shortest_minutes": self.terms.0, "longest_minutes": self.terms.1 }),
            ),
        ];
        if let Some((hunger, tiredness)) = self.needs {
            let nourishment: Vec<Value> = self
                .nourishment
                .iter()
                .map(|(raw, hunger)| json!({ "item": item(*raw), "hunger": hunger }))
                .collect();
            mine.push(record(
                ME,
                "needs",
                json!({
                    "hunger": hunger, "tiredness": tiredness,
                    "words": { "hunger": "fine", "tiredness": "fine" }, "asleep": self.asleep,
                    "rates": { "hunger_per_hour": 120, "hunger_asleep_per_hour": 40,
                               "tiredness_per_hour": 60, "rest_per_hour": self.rest_per_hour },
                    "restore": { "hunger": ["eating"], "tiredness": ["sleeping"] },
                    "nourishment": nourishment,
                }),
            ));
        }
        let asleep =
            |who| ComponentRecord::new::<Asleep>(id(who), json!({ "asleep": true, "since": 0 }));
        if self.asleep {
            mine.push(asleep(ME));
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(here)
            .with_components(mine);
        let mut bob = PerceivedEntity::new(id(BOB), EntityType::Person).at(here);
        if self.bob_asleep {
            bob = bob.with_components(vec![asleep(BOB)]);
        }
        let mut affordances = vec![
            Affordance::available(Talk::ACTION_TYPE, Some(id(BOB)), talk_requirement()),
            Affordance::available(WalkTo::ACTION_TYPE, None, move_offer_requirement()),
        ];
        if self.asleep {
            affordances.push(
                Affordance::available(Wake::ACTION_TYPE, None, SpatialRequirement::NONE)
                    .with_payload(json!({})),
            );
        } else if self.sleep_offered {
            affordances.push(Affordance::available(
                Sleep::ACTION_TYPE,
                None,
                SpatialRequirement::NONE,
            ));
        }
        for (kind, raw) in &self.meals {
            affordances.push(
                Affordance::available(
                    ActionTypeId::new(*kind).expect("a slug"),
                    None,
                    SpatialRequirement::NONE,
                )
                .with_payload(json!({ "item": item(*raw) })),
            );
        }
        Observation::new(id(ME), WorldTime::from_seconds(self.at))
            .at_location(here)
            .perceiving(vec![me, bob])
            .offering(affordances)
    }
}

fn commute() -> Option<(Commute, SimDuration)> {
    Some((Commute::default(), seconds(STRIDE)))
}

fn controller(seed: u64) -> PacedRuleController {
    PacedRuleController::new(seed, seconds(PACE))
        .commuting(Commute::default(), seconds(STRIDE))
        .needing(NeedsPolicy::default())
}

fn kind(request: &ActionRequest) -> &str {
    request.action_type().as_str()
}

fn slept_for(request: &ActionRequest) -> u16 {
    let sleep: Sleep = serde_json::from_slice(request.payload().payload()).expect("a sleep");
    sleep.minutes()
}

fn eaten(request: &ActionRequest) -> Value {
    let payload: Value = serde_json::from_slice(request.payload().payload()).expect("JSON");
    payload["item"]["entity"].clone()
}

#[test]
fn the_eat_band_fires_from_eat_from_and_eats_what_nourishes_most() {
    let policy = NeedsPolicy::default();
    let mut view = View::at(AFTERNOON).feeling(599, 0);
    view.nourishment = vec![(7, 300), (8, 500), (9, 500), (10, 0)];
    view.meals = vec![
        ("eat", 7),
        ("drink", 9),
        ("eat", 8),
        ("eat", 10),
        ("eat", 11),
    ];
    assert!(policy.eat(&view.build()).is_none(), "599 is not hungry");
    let hungry = view.clone().feeling(600, 0).build();
    let meal = policy.eat(&hungry).expect("600 eats");
    // 8 and 9 tie at 500: the lowest ItemId wins; a drink counts; 10 and 11 nourish nothing.
    assert_eq!((kind(&meal), eaten(&meal)), ("eat", json!("8")));
    for seed in 0..SEEDS {
        let decided = controller(seed).decide(&hungry).expect("eats");
        assert_eq!(eaten(&decided), json!("8"), "seed {seed}");
    }
    let mut nothing = view.feeling(900, 0);
    nothing.meals = vec![("eat", 10), ("eat", 11)];
    assert!(
        policy.eat(&nothing.build()).is_none(),
        "nothing nourishing held"
    );
}

#[test]
fn the_bed_band_fires_in_the_night_from_tired_from_and_at_any_hour_from_exhausted_from() {
    let policy = NeedsPolicy::default();
    let bed = |view: View| policy.bed(&view.build(), 7, commute());
    assert!(bed(View::at(LATE_EVENING).feeling(0, 599)).is_none());
    let tired = bed(View::at(LATE_EVENING).feeling(0, 600)).expect("tired at night");
    assert_eq!(kind(&tired), Sleep::ACTION_TYPE.as_str());
    assert!(
        bed(View::at(AFTERNOON).feeling(0, 600)).is_none(),
        "daytime"
    );
    assert!(bed(View::at(AFTERNOON).feeling(0, 899)).is_none());
    assert!(
        bed(View::at(AFTERNOON).feeling(0, 900)).is_some(),
        "exhausted"
    );
    let mut nowhere = View::at(LATE_EVENING).feeling(0, 1_000);
    nowhere.sleep_offered = false;
    assert!(
        bed(nowhere).is_none(),
        "the world does not offer sleep here"
    );
    for seed in 0..SEEDS {
        let decided = controller(seed)
            .decide(&View::at(LATE_EVENING).feeling(0, 700).build())
            .expect("goes to bed");
        assert_eq!(kind(&decided), Sleep::ACTION_TYPE.as_str(), "seed {seed}");
        // Rested after 700 ÷ 125 per hour = 336 minutes, before leaving for work at 05:16 (346).
        assert_eq!(slept_for(&decided), 336, "seed {seed}");
    }
}

#[test]
fn the_bedtime_offset_spreads_people_over_the_first_hour_and_wraps_midnight() {
    let policy = NeedsPolicy::default();
    let minute = |hours: i64, minutes: i64| hours * 3_600 + minutes * 60;
    let mut first: Vec<i64> = Vec::new();
    for observer in 0..SEEDS {
        let person = id(observer);
        assert!(!policy.in_night(7, person, minute(21, 59)));
        assert!(policy.in_night(7, person, minute(23, 0)), "{observer}");
        assert!(policy.in_night(7, person, minute(0, 30)), "past midnight");
        assert!(policy.in_night(7, person, minute(5, 59)));
        assert!(!policy.in_night(7, person, minute(6, 0)));
        let from = (0..=60)
            .find(|offset| policy.in_night(7, person, minute(22, *offset)))
            .expect("in by 23:00");
        assert!(from == 0 || !policy.in_night(7, person, minute(22, from - 1)));
        first.push(from);
    }
    first.sort_unstable();
    first.dedup();
    assert!(first.len() > 10, "bedtimes spread: {first:?}");
    let together = NeedsPolicy {
        bedtime_spread_minutes: 0,
        ..policy
    };
    assert!((0..SEEDS).all(|observer| together.in_night(7, id(observer), minute(22, 0))));
    let owls = NeedsPolicy {
        night: Night {
            from: ClockTime::new(1, 0).expect("a time"),
            until: ClockTime::new(9, 0).expect("a time"),
        },
        bedtime_spread_minutes: 0,
        ..policy
    };
    assert!(!owls.in_night(7, id(1), minute(23, 30)));
    assert!(owls.in_night(7, id(1), minute(8, 59)));
}

#[test]
fn the_sleep_length_comes_from_each_of_its_limits() {
    let felt = |view: &View| crate::needs::own(&view.build()).expect("needs");
    let minutes = |view: View, wake: bool| {
        sleep_minutes(&view.build(), &felt(&view), wake, commute()).expect("bounds")
    };
    // Rested: 600 ÷ 125 per hour = 288 minutes, with no punctual part.
    let mut view = View::at(FROM - 12 * 3_600).feeling(0, 600);
    view.punctual = false;
    assert_eq!(minutes(view.clone(), true), 288);
    // The world's longest.
    view.terms = (15, 60);
    assert_eq!(minutes(view.clone(), true), 60);
    // The commute: three hours before the part, leave by FROM − 840 s → 166 minutes.
    let commuting = View::at(FROM - 3 * 3_600).feeling(0, 600);
    assert_eq!(
        minutes(commuting.clone(), true),
        (3 * 3_600 - LEAD) as u16 / 60
    );
    assert_eq!(minutes(commuting, false), 288, "not waking for work");
    // The world's shortest, when the commute leaves almost no time.
    assert_eq!(minutes(View::at(FROM - 900).feeling(0, 600), true), 15);
    // A slower rest per hour sleeps longer.
    let mut slow = View::at(FROM - 12 * 3_600).feeling(0, 600);
    slow.punctual = false;
    slow.rest_per_hour = 100;
    assert_eq!(minutes(slow, true), 360);
}

#[test]
fn a_sleeper_wakes_only_when_the_commute_is_due_and_does_nothing_else() {
    let policy = NeedsPolicy::default();
    let asleep = |at| {
        let mut view = View::at(at).feeling(800, 100);
        view.asleep = true;
        view.meals = vec![("eat", 7)];
        view.nourishment = vec![(7, 500)];
        view
    };
    let pace = seconds(PACE);
    // Due: now + pace > FROM − lead.
    let due = asleep(FROM - LEAD - PACE + 1).build();
    let woken = policy.asleep(&due, commute(), pace).expect("asleep");
    assert_eq!(woken.as_ref().map(kind), Some(Wake::ACTION_TYPE.as_str()));
    let early = asleep(FROM - LEAD - PACE).build();
    assert_eq!(policy.asleep(&early, commute(), pace), Some(None));
    let lazy = NeedsPolicy {
        wake_for_commute: false,
        ..policy
    };
    assert_eq!(lazy.asleep(&due, commute(), pace), Some(None));
    for seed in 0..SEEDS {
        assert_eq!(
            controller(seed).decide(&early),
            None,
            "seed {seed}: hungry, yet asleep"
        );
        assert_eq!(
            controller(seed).decide(&due).as_ref().map(kind),
            Some(Wake::ACTION_TYPE.as_str())
        );
    }
}

#[test]
fn without_needs_or_asleep_every_band_is_inert_and_every_decision_is_the_base_one() {
    let policy = NeedsPolicy::default();
    for at in [LATE_EVENING, AFTERNOON, FROM - 3 * 3_600, FROM - 600] {
        let mut view = View::at(at).without_needs();
        view.meals = vec![("eat", 7)];
        let observation = view.build();
        assert_eq!(policy.eat(&observation), None);
        assert_eq!(policy.bed(&observation, 7, commute()), None);
        assert_eq!(policy.asleep(&observation, commute(), seconds(PACE)), None);
        for seed in 0..SEEDS {
            let base = PacedRuleController::new(seed, seconds(PACE))
                .commuting(Commute::default(), seconds(STRIDE));
            assert_eq!(
                controller(seed).decide(&observation),
                base.decide(&observation),
                "seed {seed} at {at}"
            );
        }
    }
}

#[test]
fn nobody_greets_approaches_or_invites_a_sleeper() {
    let mut view = View::at(AFTERNOON).feeling(0, 0);
    view.bob_asleep = true;
    let observation = view.build();
    for seed in 0..SEEDS {
        if let Some(request) = controller(seed).decide(&observation) {
            assert_ne!(request.target(), Some(id(BOB)), "seed {seed}: {request:?}");
        }
    }
}

fn policy(value: serde_json::Value) -> Result<NeedsPolicy, serde_json::Error> {
    serde_json::from_value(value)
}

#[test]
fn the_needs_block_decodes_with_the_step_defaults() {
    let decoded = policy(json!({})).expect("defaults");
    assert_eq!(decoded, NeedsPolicy::default());
    assert_eq!(
        (
            decoded.eat_from,
            decoded.tired_from,
            decoded.exhausted_from,
            decoded.night.from.minutes(),
            decoded.night.until.minutes(),
            decoded.bedtime_spread_minutes,
            decoded.wake_for_commute,
        ),
        (600, 600, 900, 22 * 60, 6 * 60, 60, true)
    );
    let owls = policy(json!({
        "night": { "from": "01:00", "until": "09:00" },
        "eat_from": 1, "tired_from": 1_000, "exhausted_from": 1_000,
        "bedtime_spread_minutes": 240, "wake_for_commute": false,
    }))
    .expect("a town of night owls");
    assert_eq!(
        owls.night,
        Night {
            from: ClockTime::new(1, 0).expect("a time"),
            until: ClockTime::new(9, 0).expect("a time"),
        }
    );
    assert!(!owls.wake_for_commute);
    let parameters = RuleParameters::decode(json!({ "needs": { "eat_from": 700 } }))
        .expect("the rule block carries needs");
    assert_eq!(parameters.needs.eat_from, 700);
    assert_eq!(parameters.commute, crate::commute::Commute::default());
}

#[test]
fn the_needs_block_refuses_each_bound_by_name() {
    for (wrong, names) in [
        (json!({ "eat_from": 0 }), "'eat_from' is 1 … 1000, not 0"),
        (json!({ "eat_from": 1_001 }), "'eat_from' is 1 … 1000"),
        (json!({ "tired_from": 0 }), "'tired_from' is 1 … 1000"),
        (
            json!({ "exhausted_from": 1_001 }),
            "'exhausted_from' is 1 … 1000",
        ),
        (
            json!({ "bedtime_spread_minutes": 241 }),
            "'bedtime_spread_minutes' is 0 … 240",
        ),
        (
            json!({ "night": { "from": "22:00", "until": "22:00" } }),
            "the night is empty",
        ),
        (json!({ "night": { "from": "24:00" } }), "not a time of day"),
        (json!({ "night": { "from": "7:00" } }), "not a time of day"),
        (json!({ "night": { "at": "22:00" } }), "unknown field `at`"),
        (json!({ "sleepy_from": 3 }), "unknown field `sleepy_from`"),
        (
            json!({ "tired_from": 700, "exhausted_from": 650 }),
            "'exhausted_from' (650) is below 'tired_from' (700)",
        ),
        (
            json!({ "exhausted_from": 650, "tired_from": 700 }),
            "'exhausted_from' (650) is below 'tired_from' (700)",
        ),
        (
            json!({ "exhausted_from": 500 }),
            "'exhausted_from' (500) is below 'tired_from' (600)",
        ),
    ] {
        let error = policy(wrong.clone()).expect_err("refused").to_string();
        assert!(error.contains(names), "{wrong}: {error}");
    }
}
