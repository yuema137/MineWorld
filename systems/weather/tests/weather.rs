//! The weather in a running world (step-19 §17.5 C3): genesis order, a month of days and changes that
//! match the days' hours, a restart mid-day that continues exactly, an unconfigured pack that does
//! nothing, disclosure only on the place an observer is in, calendar's facts untouched by weather
//! (INV-TW-10), and the climate state's snapshot budget.

mod support;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Component, EntityId, EntityType,
    EventEnvelope, PlaceId,
};
use mineworld_kernel::{ProcessKind, World};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend,
};
use mineworld_presence::observe;
use mineworld_weather::{
    ClimateProcess, ClimateState, Condition, WeatherDay, WeatherNow, WeatherToday,
};
use support::{
    DAY, HOUR, PlaceMe, assemble, begun, changes, compose, days, key, providers, run, t, weather,
};

/// The climate's state, read back.
fn climate(world: &World) -> Option<ClimateState> {
    let read = world.read();
    read.processes()
        .find(|process| *process.process_type() == ClimateProcess::PROCESS_TYPE)
        .map(|process| {
            serde_json::from_slice(process.state_for::<ClimateProcess>().expect("ours"))
                .expect("decodes")
        })
}

/// (a) Genesis: both configurations, then day 0 begins, then its weather, then the condition at 00:00 —
/// all at instant 0, in that order (step-19 §17.2: generation 0, 1, 2, 3).
#[test]
fn genesis_configures_then_begins_the_day_then_its_weather() {
    let (_, _, genesis) = begun(Some(&weather(19)));
    let ours: Vec<(i64, &str)> = genesis
        .iter()
        .filter(|fact| {
            let kind = fact.event_type().as_str();
            kind.starts_with("weather") || kind.starts_with("calendar") || kind == "day-began"
        })
        .map(|fact| (fact.at().seconds(), fact.event_type().as_str()))
        .collect();
    assert_eq!(
        ours,
        [
            (0, "calendar-configured"),
            (0, "weather-configured"),
            (0, "day-began"),
            (0, "weather-day"),
            (0, "weather-changed"),
        ]
    );
    let first = changes(&genesis);
    assert_eq!(first[0].1.hour(), 0, "the first change is hour 0");
}

/// The changes the days' own hours imply, through instant `end`: every hour whose condition differs
/// from the hour before it (across midnight too), with the condition, cloud and amount of that hour.
fn implied(
    days: &[(mineworld_contracts::WorldTime, WeatherDay)],
    end: i64,
) -> Vec<(i64, u8, Condition, u8, u16)> {
    let mut previous: Option<Condition> = None;
    let mut expected = Vec::new();
    for (_, day) in days {
        for (hour, weather) in day.hours().iter().enumerate() {
            let at = day.day_start().seconds() + i64::try_from(hour).expect("small") * HOUR;
            if at > end {
                break;
            }
            if previous != Some(weather.condition) {
                expected.push((
                    at,
                    u8::try_from(hour).expect("small"),
                    weather.condition,
                    weather.cloud_oktas,
                    weather.precipitation_tenth_mm,
                ));
            }
            previous = Some(weather.condition);
        }
    }
    expected
}

/// (b) Thirty days: 31 `weather-day` (the run's last instant is a midnight, TWa-D5), one per local
/// midnight, dated by the calendar; and the `weather-changed` facts are exactly the changes the days'
/// hours imply, each at its hour.
#[test]
fn a_month_of_weather_changes_exactly_when_its_hours_say() {
    let end = 30 * DAY;
    let (world, _, facts) = run(Some(&weather(19)), end);
    let recorded_days = days(&facts);
    assert_eq!(recorded_days.len(), 31);
    for (index, (at, day)) in recorded_days.iter().enumerate() {
        let index = i64::try_from(index).expect("small");
        assert_eq!(at.seconds(), index * DAY);
        assert_eq!(day.day_start(), *at);
    }
    let first = recorded_days[0].1.date();
    assert_eq!((first.year(), first.month(), first.day()), (2026, 10, 8));
    let last = recorded_days[30].1.date();
    assert_eq!((last.year(), last.month(), last.day()), (2026, 11, 7));

    let recorded: Vec<(i64, u8, Condition, u8, u16)> = changes(&facts)
        .iter()
        .map(|(at, changed)| {
            (
                at.seconds(),
                changed.hour(),
                changed.condition(),
                changed.cloud_oktas(),
                changed.precipitation_tenth_mm(),
            )
        })
        .collect();
    let expected = implied(&recorded_days, end);
    assert_eq!(recorded, expected);
    for pair in recorded.windows(2) {
        assert_ne!(pair[0].2, pair[1].2, "each change changes the condition");
    }
    let kinds: std::collections::BTreeSet<Condition> =
        recorded.iter().map(|change| change.2).collect();
    let wet_days = recorded_days
        .iter()
        .filter(|(_, d)| d.summary().wet())
        .count();
    println!(
        "30 days: {} weather-changed, {wet_days} wet days, conditions {kinds:?}",
        recorded.len()
    );
    assert!(
        kinds.len() >= 4,
        "a changeable month shows several conditions: {kinds:?}"
    );

    // The state is the fold: the last day, and the hour of the last change.
    let state = climate(&world).expect("a climate");
    assert_eq!(state.today(), Some(&recorded_days[30].1));
    // `now` is the hour of today the condition holds from: the last change's, if it was today, or 0
    // when yesterday's condition carried over midnight (SD-TW-b-10 (c)).
    let last_change = recorded.last().expect("a change");
    let today_start = recorded_days[30].1.day_start().seconds();
    let expected_now = if last_change.0 >= today_start {
        last_change.1
    } else {
        0
    };
    assert_eq!(state.now(), expected_now);
    assert_eq!(state.chain(), recorded_days[30].1.chain());
}

/// `weather-changed` is Public; `weather-day` and the configuration are SystemInternal; none has
/// subjects.
#[test]
fn the_weathers_facts_have_their_visibility_and_no_subjects() {
    let (_, _, facts) = run(Some(&weather(19)), 2 * DAY);
    let ours: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| fact.event_type().as_str().starts_with("weather"))
        .collect();
    assert!(ours.len() >= 5);
    for fact in ours {
        let expected = if fact.event_type().as_str() == "weather-changed" {
            mineworld_contracts::Visibility::Public
        } else {
            mineworld_contracts::Visibility::SystemInternal
        };
        assert_eq!(*fact.visibility(), expected, "{}", fact.event_type());
        assert!(fact.subjects().is_empty(), "{}", fact.event_type());
    }
}

/// A fact's projection without its id (and causation, which names ids): when, what, to whom, about
/// whom, and the payload's bytes. `with_ids` adds the id — the control of M-TWb-1.
fn projection(facts: &[EventEnvelope], with_ids: bool) -> Vec<String> {
    facts
        .iter()
        .filter(|fact| {
            matches!(
                fact.event_type().as_str(),
                "calendar-configured" | "day-began" | "daylight-changed"
            )
        })
        .map(|fact| {
            let id = if with_ids {
                format!("{:?} ", fact.id())
            } else {
                String::new()
            };
            format!(
                "{id}{} {} {:?} {:?} {}",
                fact.at().seconds(),
                fact.event_type(),
                fact.visibility(),
                fact.subjects(),
                String::from_utf8_lossy(fact.payload().payload())
            )
        })
        .collect()
}

/// Calendar's facts through instant `end` in a world without weather installed.
fn without_weather(end: i64) -> Vec<EventEnvelope> {
    let (mut world, _, facts) = assemble(compose(false), true, None);
    let mut all = world.genesis(t(0), facts).expect("genesis");
    all.extend(world.advance_to(t(end)).expect("advances").into_events());
    all
}

/// (g) INV-TW-10, criterion 1b: over 30 days calendar's facts are the same with and without weather —
/// instants, types, visibility, subjects and payload bytes. Their ids are not: they are one world
/// counter, and the weather's facts interleave (M-TWb-1, kept here as a control).
#[test]
fn calendars_facts_are_the_same_with_and_without_weather() {
    let end = 30 * DAY;
    let (_, _, with) = run(Some(&weather(19)), end);
    let without = without_weather(end);
    let (a, b) = (projection(&with, false), projection(&without, false));
    assert_eq!(
        a.len(),
        1 + 31 + 30 * 6,
        "configured, 31 days, six light changes a day"
    );
    assert_eq!(a, b, "calendar is untouched by weather");
    assert_ne!(
        projection(&with, true),
        projection(&without, true),
        "M-TWb-1: with ids the comparison sees the weather's facts interleaved"
    );
}

/// (e) Weather enabled without `configure/weather.yaml` states nothing, starts no climate and discloses
/// nothing; calendar's facts are what they are without weather installed at all.
#[test]
fn an_unconfigured_weather_states_and_discloses_nothing() {
    let end = 3 * DAY;
    let (world, keys, facts) = run(None, end);
    assert!(
        facts
            .iter()
            .all(|fact| !fact.event_type().as_str().starts_with("weather"))
    );
    assert!(climate(&world).is_none());
    assert_eq!(
        projection(&facts, false),
        projection(&without_weather(end), false)
    );
    let seen = observe(&world, keys[&key("ada")], t(end), &providers());
    assert!(
        seen.entities()
            .iter()
            .flat_map(|entity| entity.components())
            .all(|record| !record.component_type().as_str().starts_with("weather"))
    );
}

/// The weather records an observation carries, by the entity they are attached to.
fn weather_records(world: &World, observer: EntityId, at: i64) -> Vec<(EntityId, String)> {
    observe(world, observer, t(at), &providers())
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|record| record.component_type().as_str().starts_with("weather"))
        .map(|record| (record.entity(), record.component_type().to_string()))
        .collect()
}

/// (f) An observer in no place gets no weather records; placed, they get both, on the place and
/// nowhere else, and the records say today's weather and the condition now.
#[test]
fn only_an_observer_in_a_place_is_told_the_weather_on_that_place() {
    let noon = 12 * HOUR;
    let (mut world, keys, facts) = run(Some(&weather(19)), noon);
    let (ada, bo, square) = (keys[&key("ada")], keys[&key("bo")], keys[&key("square")]);
    assert_eq!(weather_records(&world, bo, noon), [], "bo is in no place");
    let both = vec![
        (square, WeatherToday::COMPONENT_TYPE.to_string()),
        (square, WeatherNow::COMPONENT_TYPE.to_string()),
    ];
    assert_eq!(
        weather_records(&world, ada, noon),
        both,
        "on the square only"
    );

    let seen = observe(&world, ada, t(noon), &providers());
    let place = seen
        .entities()
        .iter()
        .find(|entity| entity.id() == square)
        .expect("the square");
    let record = |kind: &str| {
        place
            .components()
            .iter()
            .find(|record| record.component_type().as_str() == kind)
            .expect("disclosed")
            .payload()
            .clone()
    };
    let today: WeatherToday = serde_json::from_value(record("weather-today")).expect("decodes");
    assert_eq!(today.0, days(&facts)[0].1);
    let now: WeatherNow = serde_json::from_value(record("weather-now")).expect("decodes");
    let last = changes(&facts).last().expect("a change").1;
    assert_eq!((now.hour, now.condition), (last.hour(), last.condition()));
    assert_eq!(
        now.condition,
        today.0.hours()[12].condition,
        "the condition at noon"
    );
    println!(
        "disclosure size: weather-today {} B, weather-now {} B",
        serde_json::to_vec(&record("weather-today"))
            .expect("encodes")
            .len(),
        serde_json::to_vec(&record("weather-now"))
            .expect("encodes")
            .len()
    );

    let square_place = PlaceId::new(square, EntityType::Place).expect("a place");
    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        bo,
        ActionRecord::new::<PlaceMe>(
            serde_json::to_vec(&PlaceMe {
                place: square_place,
            })
            .expect("encodes"),
        ),
        t(noon),
    );
    let done = world.dispatch(&intent, t(noon)).expect("dispatches");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    assert_eq!(
        weather_records(&world, bo, noon),
        both,
        "placed, bo is told too"
    );
}

/// (h) The climate state rides every snapshot, so its encoding has a budget: 8 KB (SD-TW-b-6). Measured
/// at every midnight of a month and at its largest.
#[test]
fn the_climate_state_stays_within_its_snapshot_budget() {
    let (mut world, _, _) = begun(Some(&weather(19)));
    let mut largest = 0;
    for day in 1..=30 {
        let _ = world
            .advance_to(t(day * DAY + 13 * HOUR))
            .expect("advances");
        let state = climate(&world).expect("a climate");
        largest = largest.max(serde_json::to_vec(&state).expect("encodes").len());
    }
    println!("largest encoded climate state: {largest} bytes");
    assert!(largest <= 8 * 1_024, "{largest} bytes > 8 KB");
}

/// A save of this test's own, removed when the test ends (DEP-29).
struct Scratch(mineworld_test_support::Scratch);

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(&scratch.0, Durability::ProcessCrash).expect("opens"))
}

fn create(scratch: &Scratch) -> PersistentWorld {
    let (world, _, facts) = assemble(compose(true), true, Some(&weather(19)));
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: 0x5eed_0000_0000_0000_0000_0000_0019_0b01,
            pack: "weather-test".to_owned(),
            at: t(0),
            facts,
        },
    )
    .expect("a save is created")
    .0
}

/// (c) A world stopped mid-day (day 2, 14:30) and resumed continues exactly as the uninterrupted world:
/// the same facts, byte for byte, through day 8 — the chain, today and the next change ride the save —
/// and the resumed climate state is the fold of the facts.
#[test]
fn a_world_stopped_mid_day_continues_as_the_uninterrupted_world_does() {
    let stop = 2 * DAY + 14 * HOUR + 1_800;
    let end = 8 * DAY;

    let straight = Scratch(mineworld_test_support::scratch!(empty "straight"));
    let mut uninterrupted = create(&straight);
    let _ = uninterrupted.advance_to(t(stop)).expect("advances");
    let straight_after = uninterrupted
        .advance_to(t(end))
        .expect("advances")
        .into_events();

    let stopped = Scratch(mineworld_test_support::scratch!(empty "stopped"));
    let mut first = create(&stopped);
    let _ = first.advance_to(t(stop)).expect("advances");
    drop(first);
    let (mut resumed, _) = PersistentWorld::resume(open(&stopped), compose(true)).expect("resumes");
    let resumed_after = resumed.advance_to(t(end)).expect("advances").into_events();

    assert_eq!(
        days(&resumed_after).len(),
        6,
        "days 3 … 8 begin after the stop"
    );
    assert_eq!(
        serde_json::to_vec(&resumed_after).expect("encodes"),
        serde_json::to_vec(&straight_after).expect("encodes"),
        "the resumed world's facts after the stop are the uninterrupted world's, byte for byte"
    );

    let state = climate(resumed.world()).expect("a climate");
    let last_day = days(&resumed_after).last().expect("a day").1.clone();
    let last_change = changes(&resumed_after).last().expect("a change").1;
    assert_eq!(state.today(), Some(&last_day));
    assert_eq!(state.chain(), last_day.chain());
    let hour_of_last_change =
        if changes(&resumed_after).last().expect("one").0 >= last_day.day_start() {
            last_change.hour()
        } else {
            0
        };
    assert_eq!(state.now(), hour_of_last_change);
}
