//! The calendar in a running world (step-19 §16.4 C3): seven days of facts, one midnight shared with
//! `schedule`, light changes in order, a restart mid-day that continues exactly, disclosure only on
//! the place an observer is in, and a configuration refused at assembly by file and key.

mod support;

use mineworld_calendar::{
    CalendarDate, CalendarDayRecord, CalendarLight, CalendarProcess, CalendarState, Phase,
};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Component, EntityType, EventEnvelope,
    PlaceId,
};
use mineworld_kernel::ProcessKind;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend,
};
use mineworld_presence::observe;
use mineworld_schedule::TimeOfDay;
use support::{DAY, PlaceMe, SAN_DIEGO, assemble, begun, compose, days, key, light, providers, t};

/// CP-TW-a, at the pack: seven days → seven `day-began`, 2026-10-08 … 14, Thursday … Wednesday, each
/// at a local midnight that is also `schedule`'s (INV-TW-4), and each day's six light changes in
/// order at the instants its record names.
#[test]
fn seven_days_have_seven_dates_and_their_light_in_order() {
    let (mut world, _, genesis) = begun(Some(SAN_DIEGO));
    let mut facts = genesis;
    facts.extend(
        world
            .advance_to(t(7 * DAY - 1))
            .expect("advances")
            .into_events(),
    );

    let began = days(&facts);
    let dates: Vec<(i32, u8, u8, u8)> = began
        .iter()
        .map(|(_, day)| {
            let date = day.day().date();
            (date.year(), date.month(), date.day(), day.day().weekday())
        })
        .collect();
    assert_eq!(
        dates,
        [
            (2026, 10, 8, 3),
            (2026, 10, 9, 4),
            (2026, 10, 10, 5),
            (2026, 10, 11, 6),
            (2026, 10, 12, 0),
            (2026, 10, 13, 1),
            (2026, 10, 14, 2),
        ],
        "Thursday 8 October … Wednesday 14 October"
    );
    for (index, (at, day)) in began.iter().enumerate() {
        let index = i64::try_from(index).expect("small");
        assert_eq!(
            at.seconds(),
            index * DAY,
            "day-began at local midnight {index}"
        );
        assert_eq!(TimeOfDay::of(*at), 0, "INV-TW-4: schedule's midnight too");
        assert_eq!(
            day.day().day_start(),
            *at,
            "the record names its own midnight"
        );
        assert_eq!(day.phase(), Phase::Night, "San Diego is dark at midnight");
        assert_eq!(day.day().track().len(), 97);
    }

    let changes = light(&facts);
    assert_eq!(changes.len(), 7 * 6, "six light changes a day");
    for (index, (_, day)) in began.iter().enumerate() {
        let expected: Vec<(i64, Phase)> = day
            .day()
            .events()
            .light_changes()
            .into_iter()
            .map(|(at, phase)| (day.day().day_start().seconds() + i64::from(at), phase))
            .collect();
        let recorded: Vec<(i64, Phase)> = changes[index * 6..index * 6 + 6]
            .iter()
            .map(|(at, changed)| (at.seconds(), changed.phase()))
            .collect();
        assert_eq!(recorded, expected, "day {index}");
        assert_eq!(
            recorded.iter().map(|(_, phase)| *phase).collect::<Vec<_>>(),
            [
                Phase::AstronomicalTwilight,
                Phase::CivilTwilight,
                Phase::Day,
                Phase::CivilTwilight,
                Phase::AstronomicalTwilight,
                Phase::Night,
            ]
        );
    }

    // The process's state is the fold of the facts: the last day and the last phase.
    let read = world.read();
    let process = read
        .processes()
        .find(|process| *process.process_type() == CalendarProcess::PROCESS_TYPE)
        .expect("one calendar process");
    let state: CalendarState =
        serde_json::from_slice(process.state_for::<CalendarProcess>().expect("ours"))
            .expect("decodes");
    assert_eq!(state.day(), began.last().expect("a day").1.day());
    assert_eq!(state.phase(), changes.last().expect("a change").1.phase());
    assert_eq!(
        process.expected_end(),
        Some(t(7 * DAY)),
        "next due: midnight"
    );
}

/// `day-began` and `daylight-changed` are Public and about nobody; the configuration is
/// SystemInternal and about nobody (`ARC-61` item 8).
#[test]
fn the_calendars_facts_are_public_and_its_configuration_is_internal() {
    let (mut world, _, genesis) = begun(Some(SAN_DIEGO));
    let mut facts = genesis;
    facts.extend(world.advance_to(t(DAY)).expect("advances").into_events());
    let ours: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| {
            fact.event_type().as_str().contains("day")
                || fact.event_type().as_str().starts_with("calendar")
        })
        .collect();
    assert!(ours.len() >= 8);
    for fact in ours {
        let expected = if fact.event_type().as_str() == "calendar-configured" {
            mineworld_contracts::Visibility::SystemInternal
        } else {
            mineworld_contracts::Visibility::Public
        };
        assert_eq!(*fact.visibility(), expected, "{}", fact.event_type());
        assert!(fact.subjects().is_empty(), "{}", fact.event_type());
    }
}

/// A world that enables the calendar without configuring it states nothing and discloses nothing
/// (IL-a: a pack that is not configured is never seeded).
#[test]
fn an_unconfigured_calendar_states_and_discloses_nothing() {
    let (mut world, keys, genesis) = begun(None);
    let mut facts = genesis;
    facts.extend(
        world
            .advance_to(t(3 * DAY))
            .expect("advances")
            .into_events(),
    );
    assert!(days(&facts).is_empty());
    assert!(light(&facts).is_empty());
    let seen = observe(&world, keys[&key("ada")], t(3 * DAY), &providers());
    assert!(
        seen.entities()
            .iter()
            .flat_map(|entity| entity.components())
            .all(|record| !record.component_type().as_str().starts_with("calendar")),
    );
}

/// The calendar records an observation carries, by the entity they are attached to.
fn calendar_records(
    world: &mineworld_kernel::World,
    observer: mineworld_contracts::EntityId,
    at: i64,
) -> Vec<(mineworld_contracts::EntityId, String)> {
    observe(world, observer, t(at), &providers())
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|record| record.component_type().as_str().starts_with("calendar"))
        .map(|record| (record.entity(), record.component_type().to_string()))
        .collect()
}

/// §16.8 (c): an observer in no place gets no calendar records; placed, they get both, on the place
/// and nowhere else; and the records say today's date and the light now.
#[test]
fn only_an_observer_in_a_place_is_told_the_date_and_the_light_on_that_place() {
    let (mut world, keys, _) = begun(Some(SAN_DIEGO));
    let _ = world.advance_to(t(12 * 3_600)).expect("advances to noon");
    let (ada, bo, square) = (keys[&key("ada")], keys[&key("bo")], keys[&key("square")]);

    assert_eq!(
        calendar_records(&world, bo, 12 * 3_600),
        [],
        "bo is in no place"
    );
    let both = vec![
        (square, CalendarDayRecord::COMPONENT_TYPE.to_string()),
        (square, CalendarLight::COMPONENT_TYPE.to_string()),
    ];
    assert_eq!(
        calendar_records(&world, ada, 12 * 3_600),
        both,
        "on the square only"
    );

    // The records' content: today, and daylight, at noon.
    let seen = observe(&world, ada, t(12 * 3_600), &providers());
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
    let today: CalendarDayRecord = serde_json::from_value(record("calendar-day")).expect("decodes");
    assert_eq!(Some(today.0.date()), CalendarDate::new(2026, 10, 8));
    assert_eq!(today.0.weekday(), 3);
    let noon = today.0.track()[48];
    assert_eq!(
        (noon.elevation(), noon.azimuth()),
        (50_759, 189_418),
        "§16.5: the 12:00 sample is the pinned one, within 0.002° of NOAA's model (E-TWa-2)"
    );
    let now: CalendarLight = serde_json::from_value(record("calendar-light")).expect("decodes");
    assert_eq!(now.phase, Phase::Day);

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
        t(12 * 3_600),
    );
    let done = world.dispatch(&intent, t(12 * 3_600)).expect("dispatches");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    assert_eq!(
        calendar_records(&world, bo, 12 * 3_600),
        both,
        "placed, bo is told too"
    );
}

/// A save of this test's own, removed when the test ends (DEP-29).
struct Scratch(mineworld_test_support::Scratch);

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(&scratch.0, Durability::ProcessCrash).expect("opens"))
}

fn create(scratch: &Scratch) -> PersistentWorld {
    let (world, _, facts) = assemble(compose(true), Some(SAN_DIEGO));
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: 0x5eed_0000_0000_0000_0000_0000_0019_0a01,
            pack: "calendar-test".to_owned(),
            at: t(0),
            facts,
        },
    )
    .expect("a save is created")
    .0
}

/// A world stopped mid-day (day 2, 14:30) and resumed continues exactly as the uninterrupted world:
/// the same facts, byte for byte, through day 8 — the Process state and its next wake ride the save.
#[test]
fn a_world_stopped_mid_day_continues_as_the_uninterrupted_world_does() {
    let stop = 2 * DAY + 14 * 3_600 + 1_800;
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
        days(&resumed_after).last().map(|(_, day)| day.day().date()),
        CalendarDate::new(2026, 10, 16),
        "day 8 is 16 October"
    );
    assert_eq!(
        serde_json::to_vec(&resumed_after).expect("encodes"),
        serde_json::to_vec(&straight_after).expect("encodes"),
        "the resumed world's facts after the stop are the uninterrupted world's, byte for byte"
    );
}
