//! AC-Na-4 (AC-S35-6, asleep half) and SD-Na-6's offers, on both test worlds.
//!
//! ana sleeps at home; ben is in the same room, cy elsewhere. Over the sleep, every observation of
//! ben's carries ana's `asleep` record without the alarm, ana's own carries the alarm, cy's carries
//! nothing about her; `sleep-terms` is only ever in one's own record; `alarm-set` reaches ana alone.

mod support;

use mineworld_contracts::{ActionResult, Observation, Rejection};
use serde_json::Value;
use support::{Kind, Town, accepted};

/// Every (entity, record) of `component` in `observation`.
fn records<'o>(
    observation: &'o Observation<Value>,
    component: &str,
) -> Vec<(mineworld_contracts::EntityId, &'o Value)> {
    observation
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|record| record.component_type().as_str() == component)
        .map(|record| (record.entity(), record.payload()))
        .collect()
}

fn keys(value: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = value
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

fn affordance<'o>(
    observation: &'o Observation<Value>,
    action: &str,
) -> Option<&'o mineworld_contracts::Affordance<Value>> {
    observation
        .affordances()
        .iter()
        .find(|affordance| affordance.action_type().as_str() == action && affordance.target().is_none())
}

fn audiences(kind: Kind, minutes: u16, longest: u16) {
    let mut town = Town::new(kind);
    let ana = town.person("ana");
    let (result, _) = town.sleep("ana", minutes);
    assert!(accepted(&result), "{result:?}");

    for _ in 0..(minutes / 60) {
        let (own, roommate, away) = (
            town.observation("ana"),
            town.observation("ben"),
            town.observation("cy"),
        );
        let mine = records(&own, "asleep");
        assert_eq!(mine.len(), 1, "{kind:?}");
        assert_eq!(mine[0].0, ana);
        assert_eq!(keys(mine[0].1), ["alarm", "asleep", "since"]);
        let seen = records(&roommate, "asleep");
        assert_eq!(seen.len(), 1, "{kind:?}: the room sees her asleep");
        assert_eq!(seen[0].0, ana);
        assert_eq!(keys(seen[0].1), ["asleep", "since"], "no alarm for others");
        assert_eq!(seen[0].1["asleep"], Value::Bool(true));
        assert!(records(&away, "asleep").is_empty(), "{kind:?}: not elsewhere");

        for (who, observation) in [("ana", &own), ("ben", &roommate), ("cy", &away)] {
            let terms = records(observation, "sleep-terms");
            assert_eq!(terms.len(), 1, "{kind:?}: {who}'s own terms only");
            assert_eq!(terms[0].0, town.person(who));
            assert_eq!(terms[0].1["shortest_minutes"], Value::from(15));
            assert_eq!(terms[0].1["longest_minutes"], Value::from(longest));
        }
        town.pass(3_600);
    }

    let alarm_set = |who: &str| town.perceived(who).iter().filter(|t| *t == "alarm-set").count();
    assert_eq!(alarm_set("ana"), 1);
    assert_eq!(alarm_set("ben"), 0);
    assert_eq!(alarm_set("cy"), 0);
    let fell = |who: &str| town.perceived(who).iter().filter(|t| *t == "fell-asleep").count();
    assert_eq!((fell("ana"), fell("ben"), fell("cy")), (1, 1, 0));
}

#[test]
fn asleep_is_seen_by_the_room_the_alarm_only_by_the_sleeper_flat_and_park() {
    audiences(Kind::FlatAndPark, 840, 840);
}

#[test]
fn asleep_is_seen_by_the_room_the_alarm_only_by_the_sleeper_hostel() {
    audiences(Kind::Hostel, 600, 600);
}

/// The offer answers as dispatch does: awake at home, `sleep` available; elsewhere refused
/// `PermissionDenied`, as the request is; asleep, a complete `wake` and `sleep` refused `Busy`.
#[test]
fn offers_answer_as_dispatch_answers() {
    for kind in [Kind::FlatAndPark, Kind::Hostel] {
        let mut town = Town::new(kind);
        let home = town.observation("ana");
        let sleep = affordance(&home, "sleep").expect("offered");
        assert!(sleep.is_available(), "{kind:?}");
        assert!(sleep.payload().is_none(), "parametric: for is the requester's");
        assert!(affordance(&home, "wake").is_none());

        let away = town.observation("cy");
        let sleep = affordance(&away, "sleep").expect("offered");
        assert_eq!(
            sleep.unavailable_reason(),
            Some(&Rejection::PermissionDenied)
        );
        let (result, _) = town.sleep("cy", 60);
        assert_eq!(result, ActionResult::Rejected(Rejection::PermissionDenied));

        let (result, _) = town.sleep("ana", 60);
        assert!(accepted(&result));
        let asleep = town.observation("ana");
        assert_eq!(
            affordance(&asleep, "sleep").expect("offered").unavailable_reason(),
            Some(&Rejection::Busy)
        );
        let wake = affordance(&asleep, "wake").expect("offered");
        assert!(wake.is_available());
        assert_eq!(wake.payload(), Some(&serde_json::json!({})));
        let (result, _) = town.sleep("ana", 60);
        assert_eq!(result, ActionResult::Rejected(Rejection::Busy));
    }
}
