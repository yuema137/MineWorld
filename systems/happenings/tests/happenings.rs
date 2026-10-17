//! Happenings in a running world (design §9.2): genesis order (T4), a year that matches its calendar
//! (T5), the first pass's catch-up (T6), happenings that do not depend on what people do (T11), and
//! the almanac's snapshot budget (T15).

mod support;

use std::collections::BTreeSet;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EventEnvelope, Visibility,
};
use mineworld_happenings::{
    AlmanacProcess, HappeningAnnounced, HappeningBegan, HappeningEnded, HappeningWithheld,
};
use mineworld_kernel::{ProcessKind, World};
use support::{DAY, HOUR, PlaceMe, begun, key, of, ours, place, run, run_on, t};

/// Saturday markets on the street and a festival in the park on 10-31, announced a week ahead.
const MARKET_AND_FESTIVAL: &str = "seed: 7
kinds:
  - { id: market-day, title: Saturday market, places: [street], hours: \"08:00-14:00\" }
  - { id: festival, title: Harvest festival, places: [park], hours: \"16:00-22:00\", announce_days: 7 }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
  - { kind: festival, place: park, date: \"10-31\" }
";

/// The days, from instant 0, a fact of type `E` falls on, with its time of day in seconds.
fn when<E: mineworld_contracts::Event + for<'de> serde::Deserialize<'de>>(
    facts: &[EventEnvelope],
) -> Vec<(i64, i64)> {
    of::<E>(facts)
        .iter()
        .map(|(at, _)| (at.seconds().div_euclid(DAY), at.seconds().rem_euclid(DAY)))
        .collect()
}

/// T4: genesis states both configurations, then day 0 begins, then the pass announces what begins on
/// day 0 — all at instant 0, in that order.
#[test]
fn genesis_then_day_zero() {
    let thursday = "seed: 7
kinds:
  - { id: opening, places: [square], hours: \"09:00-10:00\" }
calendar:
  - { kind: opening, place: square, weekdays: [thursday] }
";
    let (_, _, genesis) = begun(Some(thursday));
    let order: Vec<(i64, &str)> = genesis
        .iter()
        .filter(|fact| {
            let kind = fact.event_type().as_str();
            ours(fact) || kind == "calendar-configured" || kind == "day-began"
        })
        .map(|fact| (fact.at().seconds(), fact.event_type().as_str()))
        .collect();
    assert_eq!(
        order,
        [
            (0, "calendar-configured"),
            (0, "happenings-configured"),
            (0, "day-began"),
            (0, "happening-announced"),
        ],
        "2026-10-08, the epoch, is a Thursday"
    );
}

/// T5: over 400 days the market begins on exactly the Saturdays — day 2 (2026-10-10) and every seventh
/// day after, listed here from the epoch's weekday, not by the code — at 08:00 and ends at 14:00; the
/// festival is announced exactly 7 days before each 10-31 (2026-10-31 is day 23, 2027-10-31 day 388).
#[test]
fn a_year_of_happenings_matches_its_calendar() {
    let (_, keys, facts) = run(Some(MARKET_AND_FESTIVAL), 400 * DAY);
    let saturdays: Vec<i64> = (0..400).filter(|day| (day - 2) % 7 == 0).collect();
    assert_eq!(saturdays.len(), 57);
    let street = place(&keys, "street");
    let market = |facts: &[EventEnvelope]| -> Vec<EventEnvelope> {
        facts
            .iter()
            .filter(|fact| String::from_utf8_lossy(fact.payload().payload()).contains("market-day"))
            .cloned()
            .collect()
    };
    let market_facts = market(&facts);
    let began = when::<HappeningBegan>(&market_facts);
    let ended = when::<HappeningEnded>(&market_facts);
    let expected_began: Vec<(i64, i64)> = saturdays.iter().map(|day| (*day, 8 * HOUR)).collect();
    let expected_ended: Vec<(i64, i64)> = saturdays
        .iter()
        .filter(|day| **day * DAY + 14 * HOUR <= 400 * DAY)
        .map(|day| (*day, 14 * HOUR))
        .collect();
    assert_eq!(began, expected_began);
    assert_eq!(ended, expected_ended);
    assert!(
        of::<HappeningBegan>(&market_facts)
            .iter()
            .all(|(_, fact)| fact.happening().place() == street)
    );

    let festival: Vec<EventEnvelope> = facts
        .iter()
        .filter(|fact| String::from_utf8_lossy(fact.payload().payload()).contains("festival"))
        .cloned()
        .collect();
    let announced = of::<HappeningAnnounced>(&festival);
    let days: Vec<(i64, i64)> = announced
        .iter()
        .map(|(at, fact)| {
            (
                at.seconds().div_euclid(DAY),
                fact.happening().start().seconds(),
            )
        })
        .collect();
    assert_eq!(
        days,
        [(16, 23 * DAY + 16 * HOUR), (381, 388 * DAY + 16 * HOUR)],
        "announced on day 16 for day 23 and on day 381 for day 388"
    );
    assert_eq!(
        announced[0].1.title().map(|title| title.as_str()),
        Some("Harvest festival")
    );
    assert!(
        of::<HappeningWithheld>(&facts).is_empty(),
        "nothing refused"
    );
    println!(
        "400 days: {} market days, {} festivals",
        began.len(),
        announced.len()
    );
}

/// A happening from 00:00 begins the instant it is announced, at its midnight: a process cannot be due
/// at the instant it starts (`INV-3`), so `happening-began` is stated at once, then it ends on time.
#[test]
fn a_happening_from_midnight_begins_when_it_is_announced() {
    let text = "seed: 7
kinds:
  - { id: night-market, places: [street], hours: \"00:00-02:00\" }
calendar:
  - { kind: night-market, place: street, weekdays: [thursday, friday] }
";
    let (_, _, facts) = run(Some(text), 2 * DAY);
    let ours: Vec<(i64, &str)> = facts
        .iter()
        .filter(|fact| fact.event_type().as_str().starts_with("happening-"))
        .map(|fact| (fact.at().seconds(), fact.event_type().as_str()))
        .collect();
    assert_eq!(
        ours,
        [
            (0, "happening-announced"),
            (0, "happening-began"),
            (2 * HOUR, "happening-ended"),
            (DAY, "happening-announced"),
            (DAY, "happening-began"),
            (DAY + 2 * HOUR, "happening-ended"),
        ],
        "Thursday 2026-10-08 and Friday 2026-10-09"
    );
}

/// T6: a world whose epoch is three days before 10-31 still announces the festival — late, on day 0,
/// by the first pass's catch-up — and announces it once.
#[test]
fn catch_up_announces_late_inside_the_first_horizon() {
    let calendar = support::calendar_from("2026-10-28");
    let (_, _, facts) = run_on(&calendar, Some(MARKET_AND_FESTIVAL), 10 * DAY);
    let festival: Vec<(i64, i64)> = of::<HappeningAnnounced>(&facts)
        .iter()
        .filter(|(_, fact)| fact.happening().kind().as_str() == "festival")
        .map(|(at, fact)| (at.seconds(), fact.happening().start().seconds()))
        .collect();
    assert_eq!(festival, [(0, 3 * DAY + 16 * HOUR)]);
    let began = when::<HappeningBegan>(&facts);
    assert!(began.contains(&(3, 16 * HOUR)), "it happens: {began:?}");
}

/// The visibility and subjects of each fact: announced Public, about and at its place; began and ended
/// as the kind's audience says; the configuration and withheld SystemInternal, about nobody.
#[test]
fn each_fact_has_its_visibility_subjects_and_place() {
    let text = "seed: 7
kinds:
  - { id: market-day, places: [street], hours: \"08:00-14:00\" }
  - { id: busker, places: [street], hours: \"09:00-10:00\", audience: place }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
  - { kind: busker, place: street, weekdays: [saturday] }
";
    let (_, keys, facts) = run(Some(text), 3 * DAY);
    let street = place(&keys, "street");
    let mut seen = BTreeSet::new();
    for fact in facts.iter().filter(|fact| ours(fact)) {
        let kind = fact.event_type().as_str();
        let payload = String::from_utf8_lossy(fact.payload().payload()).into_owned();
        seen.insert(kind.to_owned());
        match kind {
            "happenings-configured" | "happening-withheld" => {
                assert_eq!(*fact.visibility(), Visibility::SystemInternal, "{kind}");
                assert!(fact.subjects().is_empty(), "{kind}");
            }
            _ => {
                let expected = if kind != "happening-announced" && payload.contains("busker") {
                    Visibility::Place(street)
                } else {
                    Visibility::Public
                };
                assert_eq!(*fact.visibility(), expected, "{kind} {payload}");
                assert_eq!(fact.subjects(), [street.entity_id()], "{kind}");
                assert_eq!(fact.place(), Some(street), "{kind}");
            }
        }
    }
    for kind in [
        "happenings-configured",
        "happening-announced",
        "happening-began",
        "happening-ended",
        "happening-withheld",
    ] {
        assert!(seen.contains(kind), "{kind} was stated: {seen:?}");
    }
}

/// The happening facts of a world, as bytes, without ids.
fn happening_facts(facts: &[EventEnvelope]) -> Vec<String> {
    facts
        .iter()
        .filter(|fact| ours(fact))
        .map(|fact| {
            format!(
                "{} {} {}",
                fact.at().seconds(),
                fact.event_type(),
                String::from_utf8_lossy(fact.payload().payload())
            )
        })
        .collect()
}

/// T11: what people do — what a controller's seed changes — does not change the happenings. One world
/// is left alone; in the other, `bo` arrives at the street at noon each day. Their `happening-*` facts
/// are the same, though their other facts are not.
#[test]
fn controller_seed_does_not_change_happenings() {
    let text = "seed: 7
kinds:
  - { id: market-day, places: [street], hours: \"08:00-14:00\" }
  - { id: street-musician, places: [street, park], hours: \"11:00-13:00\" }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
random:
  - { kind: street-musician, place: street, per_mille_per_day: 150 }
  - { kind: street-musician, place: park, per_mille_per_day: 300 }
";
    let end = 60 * DAY;
    let (_, _, quiet) = run(Some(text), end);

    let (mut world, keys, mut busy) = begun(Some(text));
    let street = place(&keys, "street");
    for day in 0..60 {
        let noon = day * DAY + 12 * HOUR;
        busy.extend(world.advance_to(t(noon)).expect("advances").into_events());
        busy.extend(arrive(&mut world, keys[&key("bo")], street, noon, day));
    }
    busy.extend(world.advance_to(t(end)).expect("advances").into_events());

    assert_ne!(
        quiet.len(),
        busy.len(),
        "the worlds differ in what people did"
    );
    let (a, b) = (happening_facts(&quiet), happening_facts(&busy));
    assert!(a.len() > 20, "a busy two months: {}", a.len());
    assert_eq!(a, b);
}

/// `bo` asks to be put at `place` at `at`.
fn arrive(
    world: &mut World,
    bo: mineworld_contracts::EntityId,
    place: mineworld_contracts::PlaceId,
    at: i64,
    n: i64,
) -> Vec<EventEnvelope> {
    let intent = ActionIntent::new(
        ActionId::from_raw(u64::try_from(n).expect("small") + 1),
        bo,
        ActionRecord::new::<PlaceMe>(serde_json::to_vec(&PlaceMe { place }).expect("encodes")),
        t(at),
    );
    let done = world.dispatch(&intent, t(at)).expect("dispatches");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    done.events().to_vec()
}

/// The almanac's encoded state, read back.
fn almanac_bytes(world: &World) -> usize {
    world
        .read()
        .processes()
        .find(|process| *process.process_type() == AlmanacProcess::PROCESS_TYPE)
        .map(|process| process.state_for::<AlmanacProcess>().expect("ours").len())
        .expect("an almanac")
}

/// T15: the almanac rides every snapshot, so its encoding has a budget: 16 KB at the default limits.
/// A town as busy as the defaults allow — three happenings begin every day, each announced 30 days
/// ahead, at three places — measured at every midnight for 120 days, past the point where an almanac
/// that never forgot an ended happening would have outgrown it.
#[test]
fn almanac_within_budget() {
    let text = "seed: 7
kinds:
  - { id: dawn-market-of-the-old-town-sq, places: [square], hours: \"06:00-07:00\", announce_days: 30 }
  - { id: noon-fair-of-the-old-town-park, places: [park], hours: \"12:00-13:00\", announce_days: 30 }
  - { id: dusk-music-of-the-old-town-str, places: [street], hours: \"18:00-19:00\", announce_days: 30 }
calendar:
  - { kind: dawn-market-of-the-old-town-sq, place: square, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: noon-fair-of-the-old-town-park, place: park, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: dusk-music-of-the-old-town-str, place: street, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
";
    let (mut world, _, _) = begun(Some(text));
    let mut largest = 0;
    for day in 1..=120 {
        let _ = world.advance_to(t(day * DAY)).expect("advances");
        largest = largest.max(almanac_bytes(&world));
    }
    println!("largest encoded almanac state: {largest} bytes");
    assert!(largest > 4 * 1_024, "the almanac is full: {largest} bytes");
    assert!(largest <= 16 * 1_024, "{largest} bytes > 16 KB");
}
