//! `sleep-until` (pr-nightlife N2-C1, N2-I1, N2-I2): an exact, server-computed sleep target.
//!
//! Every instant below is a literal of the test world's clock (genesis 01:00 on day 0, instant 0 local
//! midnight), computed by hand, never by the pack: 07:00 on day 1 is 86 400 + 25 200 = 111 600.

mod support;

use mineworld_contracts::{
    ActionRecord, ActionResult, Affordance, Rejection, Visibility, WorldTime,
};
use mineworld_sleep::{AlarmSet, FellAsleep, OUT_OF_BOUND, Sleep, SleepUntil, Woke, WokeReason};
use serde_json::Value;
use support::{Kind, Town, accepted, facts_of, record};

const DAY: i64 = 86_400;
/// 07:00 on day 1.
const MORNING_1: i64 = DAY + 7 * 3_600;

fn hms(day: i64, hours: i64, minutes: i64, seconds: i64) -> WorldTime {
    WorldTime::from_seconds(day * DAY + hours * 3_600 + minutes * 60 + seconds)
}

/// `who`'s `sleep-until` affordance at the town's current instant.
fn offer(town: &Town, who: &str) -> Affordance<Value> {
    town.observation(who)
        .affordances()
        .iter()
        .find(|affordance| affordance.action_type().as_str() == "sleep-until")
        .cloned()
        .expect("sleep-until is offered to a living person")
}

/// The `at` a complete offer carries.
fn offered_at(affordance: &Affordance<Value>) -> WorldTime {
    assert!(affordance.is_available(), "{affordance:?}");
    let at = affordance
        .payload()
        .and_then(|payload| payload.get("at"))
        .and_then(Value::as_i64)
        .expect("a complete offer names its at");
    WorldTime::from_seconds(at)
}

fn until(at: i64) -> ActionRecord {
    record(&SleepUntil::new(WorldTime::from_seconds(at)))
}

fn refusal(result: &ActionResult) -> Rejection {
    match result {
        ActionResult::Rejected(reason) => reason.clone(),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn out_of_bound(reason: &Rejection) -> bool {
    matches!(reason, Rejection::System { code, .. } if *code == OUT_OF_BOUND)
}

/// At 23:00:12 the offer is 07:00:00 the next morning exactly, and taking it sets exactly that alarm:
/// `alarm-set` to the sleeper alone, `fell-asleep` with no alarm, and the alarm wakes her there.
#[test]
fn at_23_00_12_the_offer_is_seven_the_next_morning_and_the_alarm_is_exact() {
    let mut town = Town::new(Kind::FlatAndPark);
    town.pass_to(hms(0, 23, 0, 12));
    let affordance = offer(&town, "ana");
    assert_eq!(offered_at(&affordance), WorldTime::from_seconds(MORNING_1));

    // The offer's own request, as a client or controller copies it — no arithmetic.
    let ana = town.person("ana");
    let request = affordance
        .request(ana, |payload| {
            serde_json::to_vec(payload).expect("a payload encodes")
        })
        .expect("complete");
    let (result, facts) = town.submit_now(ana, None, request.payload().clone());
    assert!(accepted(&result), "{result:?}");
    let set = facts_of::<AlarmSet>(&facts);
    assert_eq!(set.len(), 1);
    assert_eq!(set[0].1.alarm(), WorldTime::from_seconds(MORNING_1));
    let alarm_fact = facts
        .iter()
        .find(|fact| fact.event_type().as_str() == "alarm-set")
        .expect("alarm-set");
    assert_eq!(
        *alarm_fact.visibility(),
        Visibility::Entities([ana].into_iter().collect()),
        "the alarm is the holder's alone"
    );
    assert_eq!(facts_of::<FellAsleep>(&facts).len(), 1);
    let asleep = town.asleep("ana").expect("asleep");
    assert_eq!(asleep.alarm(), WorldTime::from_seconds(MORNING_1));

    let before = town.pass_to(WorldTime::from_seconds(MORNING_1 - 1));
    assert!(facts_of::<Woke>(&before).is_empty(), "not before 07:00");
    let woke = facts_of::<Woke>(&town.pass(1));
    assert_eq!(woke.len(), 1);
    assert_eq!(woke[0].0, WorldTime::from_seconds(MORNING_1));
    assert_eq!(woke[0].1.reason(), WokeReason::Alarm);
    assert_eq!(
        woke[0].1.slept(),
        (MORNING_1 - hms(0, 23, 0, 12).seconds()) as u64
    );
}

/// The sleeper's `asleep` record carries the alarm to her alone; another person perceives only that
/// she sleeps (SD-Na-6, unchanged by `sleep-until`).
#[test]
fn the_alarm_is_disclosed_to_the_sleeper_alone() {
    let mut town = Town::new(Kind::FlatAndPark);
    town.pass_to(hms(0, 23, 0, 0));
    let ana = town.person("ana");
    let (result, _) = town.submit_now(ana, None, until(MORNING_1));
    assert!(accepted(&result), "{result:?}");
    let alarm_in = |who: &str| -> Option<Value> {
        let observation = town.observation(who);
        observation
            .entity(ana)
            .and_then(|entity| {
                entity
                    .components()
                    .iter()
                    .find(|record| record.component_type().as_str() == "asleep")
            })
            .map(|record| record.payload().clone())
            .and_then(|payload| payload.get("alarm").cloned())
    };
    assert_eq!(alarm_in("ana"), Some(Value::from(MORNING_1)));
    assert_eq!(alarm_in("ben"), None, "ben sees ana asleep, not her alarm");
}

/// 06:59 and 15:00: the next morning is out of bound (24 h 1 min and 16 h, beyond 840 minutes), so
/// the offer is refused as validate refuses it — and validate refuses the same `at`.
#[test]
fn when_the_morning_is_too_far_the_offer_is_refused_as_validate_refuses_it() {
    for now in [hms(1, 6, 59, 0), hms(0, 15, 0, 0)] {
        let mut town = Town::new(Kind::FlatAndPark);
        town.pass_to(now);
        let affordance = offer(&town, "ana");
        assert!(!affordance.is_available(), "{now:?}: {affordance:?}");
        assert!(
            out_of_bound(affordance.unavailable_reason().expect("a reason")),
            "{now:?}: {affordance:?}"
        );
        let next_morning = if now.seconds() < MORNING_1 {
            MORNING_1
        } else {
            MORNING_1 + DAY
        };
        let ana = town.person("ana");
        let (result, facts) = town.submit_now(ana, None, until(next_morning));
        assert!(out_of_bound(&refusal(&result)), "{now:?}: {result:?}");
        assert!(facts.is_empty(), "a refusal records nothing");
    }
}

/// The bounds are `shortest_minutes … longest_minutes` ahead, inclusive, in seconds; a past, present,
/// too near, too far or overflowing `at` is out of bound and records nothing.
#[test]
fn at_must_lie_within_the_bounds_in_seconds() {
    let mut town = Town::new(Kind::FlatAndPark);
    let now = hms(0, 22, 0, 0).seconds();
    town.pass_to(WorldTime::from_seconds(now));
    let ana = town.person("ana");
    for at in [
        now - 1,
        now,
        now + 15 * 60 - 1,
        now + 840 * 60 + 1,
        i64::MAX,
        i64::MIN,
    ] {
        let (result, facts) = town.submit_now(ana, None, until(at));
        let reason = refusal(&result);
        assert!(out_of_bound(&reason), "at {at}: {reason:?}");
        assert!(facts.is_empty(), "at {at}: a refusal records nothing");
    }
    let (shortest, _) = town.submit_now(ana, None, until(now + 15 * 60));
    assert!(accepted(&shortest), "15 minutes ahead is admitted");
    let mut far = Town::new(Kind::FlatAndPark);
    far.pass_to(WorldTime::from_seconds(now));
    let ana = far.person("ana");
    let (longest, _) = far.submit_now(ana, None, until(now + 840 * 60));
    assert!(accepted(&longest), "840 minutes ahead is admitted");
}

/// `sleep-until` is admitted where `sleep` is and refused where it is, with `sleep`'s refusals:
/// not where sleep is allowed (`PermissionDenied`, offer and dispatch agree), asleep (`Busy`).
#[test]
fn it_is_refused_where_sleep_is_and_while_asleep() {
    let mut town = Town::new(Kind::FlatAndPark);
    town.pass_to(hms(0, 23, 0, 0));
    // cy stands in the street: not residential.
    let street = offer(&town, "cy");
    assert_eq!(
        street.unavailable_reason(),
        Some(&Rejection::PermissionDenied)
    );
    let cy = town.person("cy");
    let (result, _) = town.submit_now(cy, None, until(MORNING_1));
    assert_eq!(refusal(&result), Rejection::PermissionDenied);

    let ana = town.person("ana");
    let (result, _) = town.submit_now(ana, None, until(MORNING_1));
    assert!(accepted(&result));
    assert_eq!(
        offer(&town, "ana").unavailable_reason(),
        Some(&Rejection::Busy)
    );
    let (again, _) = town.submit_now(ana, None, until(MORNING_1));
    assert_eq!(refusal(&again), Rejection::Busy);
    let (sleep, _) = town.submit_now(ana, None, record(&Sleep::new(60)));
    assert_eq!(refusal(&sleep), Rejection::Busy, "and sleep is busy too");
}

/// F8: the world's `sleep` rules govern `sleep-until`, place and actor forms alike, with no rule
/// written for it — in the hostel a traveller is forbidden the garden and everyone else permitted it;
/// in a configured flat a place rule forbids the flat and permits the street.
#[test]
fn the_worlds_sleep_rules_govern_sleep_until() {
    // Actor form: the hostel's garden region forbids travellers (ben) and permits everyone else (ana).
    let mut hostel = Town::new(Kind::Hostel);
    hostel.pass_to(hms(0, 21, 0, 0));
    for who in ["ana", "ben"] {
        let (walked, _) = hostel.walk(who, "garden");
        assert!(accepted(&walked), "{who} walks to the garden: {walked:?}");
    }
    let at = MORNING_1;
    let (ben_result, _) = hostel.submit_now(hostel.person("ben"), None, until(at));
    assert_eq!(
        refusal(&ben_result),
        Rejection::PermissionDenied,
        "forbid sleep, actor form"
    );
    assert_eq!(
        offer(&hostel, "ben").unavailable_reason(),
        Some(&Rejection::PermissionDenied)
    );
    assert_eq!(
        offered_at(&offer(&hostel, "ana")),
        WorldTime::from_seconds(at)
    );
    let (ana_result, _) = hostel.submit_now(hostel.person("ana"), None, until(at));
    assert!(accepted(&ana_result), "permit sleep: {ana_result:?}");

    // Place form: forbid the residential flat, permit the street.
    let section = r#"{ "rules": [
        { "action": "sleep", "place": "home", "effect": "forbid" },
        { "action": "sleep", "place": "kerb", "effect": "permit" } ] }"#;
    let classes = r#"[ { "class": "home", "of": "place", "tag": "residential" },
                        { "class": "kerb", "of": "place", "tag": "street" } ]"#;
    let mut town = Town::configured(section, classes);
    town.pass_to(hms(0, 23, 0, 0));
    let ana = town.person("ana");
    let (flat, _) = town.submit_now(ana, None, until(MORNING_1));
    assert_eq!(
        refusal(&flat),
        Rejection::PermissionDenied,
        "forbid sleep, place form"
    );
    assert_eq!(
        offered_at(&offer(&town, "cy")),
        WorldTime::from_seconds(MORNING_1)
    );
    let cy = town.person("cy");
    let (street, _) = town.submit_now(cy, None, until(MORNING_1));
    assert!(accepted(&street), "permit sleep, place form: {street:?}");
}

/// The morning is a regional parameter: the base says 06:30, the park's region 09:00 — each person is
/// offered their own place's morning.
#[test]
fn the_morning_is_the_worlds_and_regional() {
    let section = r#"{
      "parameters": [ { "morning_minute": 390 } ],
      "rules": [ { "action": "sleep", "effect": "permit" } ],
      "regions": { "park": { "parameters": [ { "morning_minute": 540 } ] } }
    }"#;
    let mut town = Town::configured(section, "[]");
    town.pass_to(hms(0, 22, 0, 0));
    let (walked, _) = town.walk("cy", "park");
    assert!(accepted(&walked), "{walked:?}");
    let now = town.now();
    assert!(now < hms(0, 23, 0, 0), "the walk ends before 23:00");
    assert_eq!(
        offered_at(&offer(&town, "ana")),
        hms(1, 6, 30, 0),
        "the flat takes the base morning"
    );
    assert_eq!(
        offered_at(&offer(&town, "cy")),
        hms(1, 9, 0, 0),
        "the park takes its region's"
    );
}

/// Each of the four ways a sleep ends ends a `sleep-until`: the alarm (above), `wake`, being spoken to,
/// and walking — each with exactly one `woke`, `Asleep` gone and no sleep Process left.
#[test]
fn every_way_a_sleep_ends_ends_a_sleep_until() {
    let start = |town: &mut Town| {
        town.pass_to(hms(0, 23, 0, 0));
        let ana = town.person("ana");
        let (result, _) = town.submit_now(ana, None, until(MORNING_1));
        assert!(accepted(&result));
        town.pass(600);
    };
    let ended = |town: &Town, facts: &[mineworld_contracts::EventEnvelope], why: WokeReason| {
        let woke = facts_of::<Woke>(facts);
        assert_eq!(woke.len(), 1, "{why:?}");
        assert_eq!(woke[0].1.reason(), why);
        assert!(town.asleep("ana").is_none());
        assert_eq!(town.sleeping_processes(), 0);
    };

    let mut asked = Town::new(Kind::FlatAndPark);
    start(&mut asked);
    let (_, facts) = asked.wake("ana");
    ended(&asked, &facts, WokeReason::Asked);

    let mut addressed = Town::new(Kind::FlatAndPark);
    start(&mut addressed);
    let (_, facts) = addressed.talk("ben", "ana");
    ended(&addressed, &facts, WokeReason::Addressed);

    let mut acted = Town::new(Kind::FlatAndPark);
    start(&mut acted);
    let (_, facts) = acted.walk("ana", "street");
    ended(&acted, &facts, WokeReason::Acted);

    for town in [&asked, &addressed, &acted] {
        assert!(facts_of::<Woke>(&town.log).len() == 1, "one woke each");
    }
}

/// A world without the sleep pack offers no `sleep-until` and refuses one as unprovided.
#[test]
fn a_world_without_sleep_offers_nothing() {
    let town = Town::without_sleep(Kind::FlatAndPark);
    assert!(
        town.observation("ana")
            .affordances()
            .iter()
            .all(|affordance| !affordance.action_type().as_str().starts_with("sleep")),
        "nothing about sleep"
    );
}

/// A world saved mid-`sleep-until` and reopened wakes at exactly the same alarm as the uninterrupted
/// world: the alarm is in the Process the save holds, not in anything the request left behind.
#[test]
fn a_world_saved_mid_sleep_until_wakes_at_its_alarm_after_reopening() {
    use mineworld_contracts::{ActionId, ActionIntent};
    use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend};
    use support::{Assembled, GENESIS, assemble, compose};

    let create = |scratch: &mineworld_test_support::Scratch| {
        let Assembled {
            world,
            people,
            facts,
            ..
        } = assemble(Kind::FlatAndPark, true, None);
        let backend = SqliteBackend::create(scratch, Durability::ProcessCrash).expect("creates");
        let (mut persisted, _) = PersistentWorld::create(
            Box::new(backend),
            world,
            Creation {
                instance: 0x5eed_0000_0000_0000_0000_0000_0035_000b,
                pack: "sleep-until-test".to_owned(),
                at: GENESIS,
                facts,
            },
        )
        .expect("a save is created");
        let ana = people["ana"];
        let night = hms(0, 23, 0, 12);
        let _ = persisted.advance_to(night).expect("advances");
        let intent = ActionIntent::new(ActionId::from_raw(1), ana, until(MORNING_1), night);
        let done = persisted.dispatch(&intent, night).expect("dispatches");
        assert!(accepted(done.result()), "{:?}", done.result());
        let _ = persisted.advance_to(hms(1, 2, 0, 0)).expect("advances");
        persisted
    };
    let after = |persisted: &mut PersistentWorld| {
        facts_of::<Woke>(
            &persisted
                .advance_to(hms(1, 9, 0, 0))
                .expect("advances")
                .into_events(),
        )
    };

    let straight = mineworld_test_support::scratch!(empty "until-straight");
    let mut uninterrupted = create(&straight);
    let expected = after(&mut uninterrupted);
    assert_eq!(expected.len(), 1);
    assert_eq!(expected[0].0, WorldTime::from_seconds(MORNING_1));
    assert_eq!(expected[0].1.reason(), WokeReason::Alarm);

    let stopped = mineworld_test_support::scratch!(empty "until-stopped");
    drop(create(&stopped));
    let backend = SqliteBackend::open(&stopped, Durability::ProcessCrash).expect("opens");
    let (mut resumed, _) =
        PersistentWorld::resume(Box::new(backend), compose(true).0).expect("resumes");
    assert_eq!(
        after(&mut resumed),
        expected,
        "the same woke, instant and payload"
    );
}

/// The advance stop (`ARC-374`, N2-I4): a sleep is identified by its Process, its horizon is the alarm, and it
/// holds until it ends — however it ends — and no later sleep is mistaken for it.
#[test]
fn the_advance_stop_identifies_a_sleep_its_alarm_and_its_end() {
    use mineworld_sdk::advance::AdvanceStop;
    use mineworld_sleep::SleepSystem;

    let stop = SleepSystem;
    assert_eq!(stop.name(), "sleep");
    let mut town = Town::new(Kind::FlatAndPark);
    town.pass_to(hms(0, 23, 0, 0));
    let ana = town.person("ana");
    assert_eq!(stop.identify(&town.world.read(), ana), None, "awake");
    let (result, _) = town.submit_now(ana, None, until(MORNING_1));
    assert!(accepted(&result));
    let occurrence = stop
        .identify(&town.world.read(), ana)
        .expect("asleep, so identified");
    assert_eq!(
        stop.horizon(&town.world.read(), ana, occurrence),
        Some(WorldTime::from_seconds(MORNING_1)),
        "the horizon is the alarm"
    );
    assert!(stop.holds(&town.world.read(), ana, occurrence));
    town.pass(600);
    let _ = town.wake("ana");
    assert!(!stop.holds(&town.world.read(), ana, occurrence), "woken");
    assert_eq!(stop.horizon(&town.world.read(), ana, occurrence), None);
    let (result, _) = town.sleep("ana", 60);
    assert!(accepted(&result));
    let again = stop
        .identify(&town.world.read(), ana)
        .expect("asleep again");
    assert_ne!(again, occurrence, "a new sleep is a new occurrence");
    assert!(!stop.holds(&town.world.read(), ana, occurrence));
}
