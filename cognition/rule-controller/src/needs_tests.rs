//! `rule`'s needs policy and bands, decided from observations built by hand (`pr-s35-nc-controllers.md`
//! SD-Nc-1 … SD-Nc-6, AC-Nc-2). The `needs` record is written as the world discloses it — slug and JSON —
//! since this controller may not name the pack's crate (F-Nc-3).

use mineworld_contracts::{
    Action, ActionRequest, ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType,
    LocalPosition, Location, Millimetres, Observation, PerceivedEntity, PlaceId, ProcessId,
    SimDuration, SpatialRequirement, Tag, Tags, WorldTime,
};
use mineworld_conversation::{Talk, talk_requirement};
use mineworld_decision::ModelParameters;
use mineworld_movement::{WalkTo, Way, Ways, move_offer_requirement};
use mineworld_schedule::{Agenda, AgendaLabel, DisclosedAgenda, NextPart};
use mineworld_sleep::{Asleep, Sleep, SleepTerms, Wake};
use serde_json::{Value, json};

use crate::commute::Commute;
use crate::needs::{ClockTime, NeedsPolicy, Night, SleepUntil, sleep_minutes};
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
    // Q-S35-OP1 (a): outside the night only the fully exhausted go to bed — 900 no longer does.
    assert!(bed(View::at(AFTERNOON).feeling(0, 900)).is_none());
    assert!(bed(View::at(AFTERNOON).feeling(0, 999)).is_none());
    let exhausted = bed(View::at(AFTERNOON).feeling(0, 1_000)).expect("exhausted");
    // Begun before the night, a sleep still runs to its end (06:00, 960 minutes), cut by the commute's
    // leave-by (05:16, 916) and the world's longest (840) — not at rested (480), which would wake me at
    // 22:00.
    assert_eq!(slept_for(&exhausted), 840);
    let rested = NeedsPolicy {
        sleep_until: SleepUntil::Rested,
        ..policy.clone()
    };
    let early = rested
        .bed(&View::at(AFTERNOON).feeling(0, 1_000).build(), 7, commute())
        .expect("exhausted");
    assert_eq!(slept_for(&early), 480, "1 000 ÷ 125 per hour");
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
        // Q-S35-OP1 (a): a sleep begun in the night runs to its end (06:00, 390 minutes from 23:30),
        // cut by the leave-by for work at 05:16 (346) — no longer at rested (336).
        assert_eq!(slept_for(&decided), 346, "seed {seed}");
    }
}

#[test]
fn a_night_sleep_runs_to_the_nights_end_unless_the_policy_says_rested() {
    let policy = NeedsPolicy::default();
    let mut free = View::at(LATE_EVENING).feeling(0, 700);
    free.punctual = false;
    let to_end = policy.bed(&free.build(), 7, commute()).expect("bed");
    assert_eq!(slept_for(&to_end), 390, "23:30 → 06:00");
    let rested = NeedsPolicy {
        sleep_until: SleepUntil::Rested,
        ..policy.clone()
    };
    assert_eq!(
        slept_for(&rested.bed(&free.build(), 7, commute()).expect("bed")),
        336,
        "700 ÷ 125 per hour"
    );
    // Past midnight the night's end is the same morning: 02:00 → 06:00.
    let mut late = View::at(86_400 + 2 * 3_600).feeling(0, 700);
    late.punctual = false;
    assert_eq!(
        slept_for(&policy.bed(&late.build(), 7, commute()).expect("bed")),
        240
    );
    // The world's longest still bounds it.
    free.terms = (15, 300);
    assert_eq!(
        slept_for(&policy.bed(&free.build(), 7, commute()).expect("bed")),
        300
    );
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
        ..policy.clone()
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
    // Woken a pace before the commute falls due (D-Nc2-5): now + 2 × pace > FROM − lead, so the next
    // consult, awake, sets out.
    let due = asleep(FROM - LEAD - 2 * PACE + 1).build();
    let woken = policy.asleep(&due, commute(), pace).expect("asleep");
    assert_eq!(woken.as_ref().map(kind), Some(Wake::ACTION_TYPE.as_str()));
    let early = asleep(FROM - LEAD - 2 * PACE).build();
    assert_eq!(policy.asleep(&early, commute(), pace), Some(None));
    let lazy = NeedsPolicy {
        wake_for_commute: false,
        ..policy.clone()
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
        (600, 600, 1_000, 22 * 60, 6 * 60, 60, true)
    );
    assert_eq!(decoded.sleep_until, SleepUntil::NightEnd);
    assert!(decoded.fetch_food);
    assert_eq!(decoded.fetch_from, 600);
    assert_eq!(decoded.shop_tags, vec![Tag::new("shop").expect("a tag")]);
    let written = policy(json!({
        "sleep_until": "rested", "fetch_food": false, "fetch_from": 800,
        "shop_tags": ["bakery", "market"], "tired_from": 600, "exhausted_from": 900,
    }))
    .expect("the amended keys");
    assert_eq!(
        (
            written.sleep_until,
            written.fetch_food,
            written.fetch_from,
            written.exhausted_from
        ),
        (SleepUntil::Rested, false, 800, 900)
    );
    assert_eq!(written.shop_tags.len(), 2);
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
            json!({ "fetch_from": 0 }),
            "'fetch_from' is 1 … 1000, not 0",
        ),
        (json!({ "fetch_from": 1_001 }), "'fetch_from' is 1 … 1000"),
        (
            json!({ "shop_tags": ["a", "b", "c", "d", "e", "f", "g", "h", "i"] }),
            "'shop_tags' lists at most 8 tags, not 9",
        ),
        (json!({ "shop_tags": ["Not A Tag"] }), "contains 'N'"),
        (json!({ "sleep_until": "noon" }), "unknown variant `noon`"),
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

// ---- The fetch band (Q-S35-OP1 (b)) -------------------------------------------------------------

const STREET: u64 = 30;
const STORE: u64 = 31;
const OFFICE: u64 = 32;

fn tag(name: &str) -> Tag {
    Tag::new(name).expect("a tag")
}

/// Where I stand, hungry, and what the world discloses there: the place's own tags, its doorways (each
/// with the destination's tags), the `buy` offers (kind, available) and what I hold (kind, nourishes).
struct Errand {
    at: u64,
    hunger: u16,
    place_tags: Vec<&'static str>,
    doors: Vec<(u64, Vec<&'static str>)>,
    buys: Vec<(u64, bool)>,
    held: Vec<(u64, u16)>,
    walking_to: Option<u64>,
    needs: bool,
}

impl Errand {
    fn at(at: u64) -> Self {
        Self {
            at,
            hunger: 700,
            place_tags: Vec::new(),
            doors: Vec::new(),
            buys: Vec::new(),
            held: Vec::new(),
            walking_to: None,
            needs: true,
        }
    }

    fn build(&self) -> Observation<Value> {
        let here = Location::in_place(place(self.at));
        let leads_to: Vec<Value> = self
            .doors
            .iter()
            .map(|(to, tags)| {
                let mut door =
                    serde_json::to_value(mineworld_movement::Passage::new(place(*to), None, None))
                        .expect("serializes");
                door["to_tags"] = json!(tags);
                door
            })
            .collect();
        let mut site = PerceivedEntity::new(id(self.at), EntityType::Place)
            .with_tags(Tags::new(self.place_tags.iter().map(|name| tag(name))));
        if !self.doors.is_empty() {
            site =
                site.with_components(vec![ComponentRecord::new::<mineworld_movement::Passages>(
                    id(self.at),
                    json!({ "leads_to": leads_to }),
                )]);
        }
        let mut mine = Vec::new();
        if self.needs {
            let nourishment: Vec<Value> = self
                .held
                .iter()
                .filter(|(_, hunger)| *hunger > 0)
                .map(|(raw, hunger)| json!({ "item": item(*raw), "hunger": hunger }))
                .collect();
            mine.push(record(
                ME,
                "needs",
                json!({ "hunger": self.hunger, "tiredness": 0, "nourishment": nourishment }),
            ));
        }
        let held: Vec<Value> = self
            .held
            .iter()
            .map(|(raw, _)| json!({ "item": item(*raw), "count": 1 }))
            .collect();
        mine.push(record(ME, "holdings", json!({ "held": held })));
        if let Some(to) = self.walking_to {
            mine.push(ComponentRecord::new::<mineworld_movement::Walking>(
                id(ME),
                json!({
                    "destination": serde_json::to_value(mineworld_movement::Destination::Place(
                        Location::in_place(place(to)),
                    ))
                    .expect("serializes"),
                    "next": [],
                }),
            ));
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(here)
            .with_components(mine);
        let mut affordances = vec![Affordance::available(
            WalkTo::ACTION_TYPE,
            None,
            move_offer_requirement(),
        )];
        for (raw, available) in &self.buys {
            let buy = ActionTypeId::new("buy").expect("a slug");
            let offer = if *available {
                Affordance::available(buy, None, SpatialRequirement::NONE)
            } else {
                Affordance::unavailable(
                    buy,
                    None,
                    SpatialRequirement::NONE,
                    mineworld_contracts::Rejection::Unavailable,
                )
            };
            affordances.push(offer.with_payload(json!({ "item": item(*raw) })));
        }
        Observation::new(id(ME), WorldTime::from_seconds(AFTERNOON))
            .at_location(here)
            .perceiving(vec![site, me])
            .offering(affordances)
    }
}

/// The place a `walk-to` request names, or the item a `buy` names.
fn walked_to(request: &ActionRequest) -> Value {
    let payload: Value = serde_json::from_slice(request.payload().payload()).expect("JSON");
    payload["to"].clone()
}

fn destination(raw: u64) -> Value {
    serde_json::to_value(mineworld_movement::Destination::Place(Location::in_place(
        place(raw),
    )))
    .expect("serializes")
}

#[test]
fn hungry_with_no_food_a_person_buys_what_is_offered_here_and_not_already_held() {
    let policy = NeedsPolicy::default();
    let mut shop = Errand::at(STORE);
    shop.place_tags = vec!["shop"];
    shop.doors = vec![(STREET, vec!["street"])];
    // 9 is held and does not nourish me (a mug, or a coffee): known not to help. 12 is out of stock.
    shop.held = vec![(9, 0)];
    shop.buys = vec![(12, false), (11, true), (9, true), (10, true)];
    let bought = policy
        .fetch(&shop.build(), false)
        .expect("applies")
        .expect("asks");
    assert_eq!((kind(&bought), eaten(&bought)), ("buy", json!("10")));
    for seed in 0..SEEDS {
        let decided = controller(seed).decide(&shop.build()).expect("buys");
        assert_eq!(eaten(&decided), json!("10"), "seed {seed}");
    }
    // Nothing left that could help: the shop is where food is, so I do not walk out.
    shop.buys = vec![(12, false), (9, true)];
    assert_eq!(policy.fetch(&shop.build(), false), None);
    // Below fetch_from, or holding something that nourishes (the eat band's), the band is silent.
    shop.buys = vec![(10, true)];
    shop.hunger = 599;
    assert_eq!(policy.fetch(&shop.build(), false), None);
    shop.hunger = 700;
    shop.held = vec![(9, 0), (13, 200)];
    assert_eq!(policy.fetch(&shop.build(), false), None);
}

#[test]
fn waiting_early_at_a_punctual_place_a_hungry_person_still_buys_there_but_never_walks_off() {
    // F-Nd-8: arrived early at a shop that is also the next punctual part's place, `holds` keeps me
    // there; a purchase here moves nobody, so it is still made — a walk away is not (D-Nc2-5).
    let policy = NeedsPolicy::default();
    let mut shop = Errand::at(STORE);
    shop.place_tags = vec!["shop"];
    shop.doors = vec![(STREET, vec!["street"])];
    shop.buys = vec![(10, true)];
    let bought = policy
        .fetch(&shop.build(), true)
        .expect("applies")
        .expect("asks");
    assert_eq!((kind(&bought), eaten(&bought)), ("buy", json!("10")));
    // Nothing sold here: waiting, I do not walk out to look elsewhere.
    shop.buys = Vec::new();
    shop.place_tags = Vec::new();
    assert_ne!(
        policy.fetch(&shop.build(), false),
        None,
        "not waiting: out the one door"
    );
    assert_eq!(policy.fetch(&shop.build(), true), None);
    let mut street = Errand::at(STREET);
    street.doors = vec![(HOME, vec!["residential"]), (STORE, vec!["shop"])];
    assert_eq!(policy.fetch(&street.build(), true), None);
}

#[test]
fn hungry_with_no_food_a_person_walks_to_a_shop_tagged_door_or_out_of_a_dead_end() {
    let policy = NeedsPolicy::default();
    let mut street = Errand::at(STREET);
    street.doors = vec![
        (HOME, vec!["residential"]),
        (STORE, vec!["store", "shop"]),
        (OFFICE, vec!["office"]),
    ];
    let walk = policy
        .fetch(&street.build(), false)
        .expect("applies")
        .expect("asks");
    assert_eq!(kind(&walk), WalkTo::ACTION_TYPE.as_str());
    assert_eq!(walked_to(&walk), destination(STORE));
    // Already on my way there: nothing is asked, and nothing redirects the walk.
    street.walking_to = Some(STORE);
    assert_eq!(policy.fetch(&street.build(), false), Some(None));
    street.walking_to = None;
    // The world tags its shops otherwise: the policy names the tag, the code names no place.
    let bakers = NeedsPolicy {
        shop_tags: vec![tag("bakery")],
        ..policy.clone()
    };
    let mut lane = Errand::at(STREET);
    lane.doors = vec![(HOME, vec!["residential"]), (OFFICE, vec!["bakery"])];
    let walk = bakers
        .fetch(&lane.build(), false)
        .expect("applies")
        .expect("asks");
    assert_eq!(walked_to(&walk), destination(OFFICE));
    // A street with several doors and no shop among them: the band has nowhere to send me.
    assert_eq!(policy.fetch(&lane.build(), false), None);
    // At home, one door: out to the street, where a shop may be seen.
    let mut home = Errand::at(HOME);
    home.place_tags = vec!["residential"];
    home.doors = vec![(STREET, vec!["street"])];
    let walk = policy
        .fetch(&home.build(), false)
        .expect("applies")
        .expect("asks");
    assert_eq!(walked_to(&walk), destination(STREET));
    for seed in 0..SEEDS {
        let decided = controller(seed).decide(&home.build()).expect("goes out");
        assert_eq!(walked_to(&decided), destination(STREET), "seed {seed}");
    }
    // Switched off, or with no needs disclosed, the band is inert.
    let off = NeedsPolicy {
        fetch_food: false,
        ..policy.clone()
    };
    assert_eq!(off.fetch(&home.build(), false), None);
    home.needs = false;
    assert_eq!(policy.fetch(&home.build(), false), None);
    for seed in 0..SEEDS {
        let base = PacedRuleController::new(seed, seconds(PACE))
            .commuting(Commute::default(), seconds(STRIDE));
        assert_eq!(
            controller(seed).decide(&home.build()),
            base.decide(&home.build()),
            "seed {seed}"
        );
    }
}

// ---- A bed spot clear of the doors (D-Nc2-4) -----------------------------------------------------

/// Me, tired at 23:30 at home at `(x, y)`, its floor 8 × 6 m and one doorway at (2 000, 400).
fn bedroom(x: i32, y: i32, floor: bool) -> Observation<Value> {
    let spot = |x, y| {
        LocalPosition::new(
            Millimetres::new(x),
            Millimetres::new(y),
            Millimetres::new(0),
        )
    };
    let here = Location::in_place(place(HOME)).with_local(spot(x, y));
    let door = serde_json::to_value(mineworld_movement::Passage::new(
        place(STREET),
        Some(spot(2_000, 400)),
        None,
    ))
    .expect("serializes");
    let mut records = vec![ComponentRecord::new::<mineworld_movement::Passages>(
        id(HOME),
        json!({ "leads_to": [door] }),
    )];
    if floor {
        records.push(record(
            HOME,
            "place-shape",
            json!({ "floor": { "min": { "x": 0, "y": 0 }, "max": { "x": 8_000, "y": 6_000 } },
                    "solids": [] }),
        ));
    }
    let home = PerceivedEntity::new(id(HOME), EntityType::Place).with_components(records);
    let me = PerceivedEntity::new(id(ME), EntityType::Person)
        .at(here)
        .with_components(vec![
            record(ME, "needs", json!({ "hunger": 0, "tiredness": 700 })),
            ComponentRecord::new::<SleepTerms>(
                id(ME),
                json!({ "shortest_minutes": 15, "longest_minutes": 840 }),
            ),
        ]);
    Observation::new(id(ME), WorldTime::from_seconds(LATE_EVENING))
        .at_location(here)
        .perceiving(vec![home, me])
        .offering(vec![
            Affordance::available(WalkTo::ACTION_TYPE, None, move_offer_requirement()),
            Affordance::available(Sleep::ACTION_TYPE, None, SpatialRequirement::NONE),
        ])
}

#[test]
fn at_bedtime_by_a_door_a_person_first_walks_to_their_own_spot_clear_of_it() {
    let policy = NeedsPolicy::default();
    let by_the_door = bedroom(2_000, 700, true);
    let mut spots = Vec::new();
    for seed in 0..SEEDS {
        let walk = policy
            .to_bed(&by_the_door, seed)
            .expect("applies")
            .expect("asks");
        assert_eq!(kind(&walk), WalkTo::ACTION_TYPE.as_str());
        let to = walked_to(&walk)["place"]["local"].clone();
        let (x, y) = (to["x"].as_i64().expect("x"), to["y"].as_i64().expect("y"));
        let away = ((x - 2_000).pow(2) + (y - 400).pow(2)).isqrt();
        assert!(
            away >= 1_500,
            "seed {seed}: ({x}, {y}) is {away} mm from the door"
        );
        assert!(
            (400..7_600).contains(&x) && (400..5_600).contains(&y),
            "on the floor"
        );
        spots.push((x, y));
        let decided = controller(seed).decide(&by_the_door).expect("walks");
        assert_eq!(kind(&decided), WalkTo::ACTION_TYPE.as_str(), "seed {seed}");
    }
    spots.sort_unstable();
    spots.dedup();
    assert!(
        spots.len() > SEEDS as usize / 2,
        "spots spread: {}",
        spots.len()
    );
    // Clear of the door already, or no floor disclosed, or the policy says 0: sleep where I stand.
    let clear = bedroom(6_000, 4_000, true);
    assert_eq!(policy.to_bed(&clear, 7), None);
    assert_eq!(kind(&controller(7).decide(&clear).expect("bed")), "sleep");
    assert_eq!(policy.to_bed(&bedroom(2_000, 700, false), 7), None);
    let anywhere = NeedsPolicy {
        bed_clear_of_doors_mm: 0,
        ..policy
    };
    assert_eq!(anywhere.to_bed(&by_the_door, 7), None);
}
