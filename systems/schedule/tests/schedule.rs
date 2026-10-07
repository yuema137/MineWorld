//! A person's day, kept by a Process its owner never ends, over a hand-built town with the real packs.
//!
//! ```text
//! genesis     routine-assigned → Routine, one `routine` process, the first agenda-changed (caused by
//!             the assignment) — at midnight, the segment that wraps past it
//! wakes       nothing a second early; at each boundary exactly one agenda-changed, caused by the
//!             process; the process rescheduled, never ended, never replaced
//! moves none  three agenda changes later the person stands exactly where presence put them
//! section     every routine rule refused by schedule's own types, as the loader decodes it
//! disclosure  the agenda to its holder, and to nobody else
//! ```

mod support;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{Causation, Component, EntityType, ProcessId};
use mineworld_presence::Presence;
use mineworld_schedule::{
    Agenda, AgendaChanged, AuthoredRoutine, DAY, Routine, RoutineProcess, ScheduleSystem,
};
use support::{ALICE_DAY, Town, key, routine, t, types};

/// Six in the morning and six in the evening: Alice's two boundaries, in seconds after midnight.
const SIX_AM: i64 = 6 * 3_600;
const SIX_PM: i64 = 18 * 3_600;

fn agenda(town: &Town) -> Agenda {
    town.world
        .read()
        .component::<Agenda>(town.alice)
        .cloned()
        .expect("alice has an agenda")
}

fn decoded(fact: &mineworld_contracts::EventEnvelope) -> AgendaChanged {
    serde_json::from_slice(
        fact.payload()
            .payload_for::<AgendaChanged>()
            .expect("schedule's fact"),
    )
    .expect("decodes")
}

#[test]
fn a_routine_assigned_at_midnight_starts_the_day_with_the_segment_that_wraps_past_it() {
    let (town, genesis) = Town::begun();
    assert_eq!(
        types(&genesis),
        ["arrived", "arrived", "routine-assigned", "agenda-changed"],
        "the assignment, then the first agenda it implies"
    );
    let assigned = &genesis[2];
    let first = &genesis[3];
    assert_eq!(*assigned.caused_by(), Causation::WorldGenesis);
    assert_eq!(
        *first.caused_by(),
        Causation::Event(assigned.id()),
        "the first agenda is caused by the assignment, not by a wake"
    );

    let now = agenda(&town);
    assert_eq!(
        now.place(),
        town.park,
        "at midnight, yesterday evening's walk is in force"
    );
    assert_eq!(now.label().as_str(), "walk");
    assert_eq!(now.since(), t(0));
    assert_eq!(now.until(), t(SIX_AM));

    let read = town.world.read();
    let process = read
        .process(now.routine())
        .expect("the routine process runs");
    assert!(
        process.state_for::<RoutineProcess>().is_ok(),
        "schedule's own kind"
    );
    assert_eq!(
        process.expected_end(),
        Some(t(SIX_AM)),
        "woken at the next boundary"
    );
    assert_eq!(process.participants(), [town.alice]);
    assert!(read.component::<Routine>(town.alice).is_some());
    assert!(
        read.component::<Agenda>(town.bob).is_none(),
        "Bob has no `routine:` section, so no day is kept for him"
    );
}

#[test]
fn every_boundary_fires_once_in_order_caused_by_the_one_routine_process() {
    let (mut town, _) = Town::begun();
    let routine: ProcessId = agenda(&town).routine();

    let early = town.world.advance_to(t(SIX_AM - 1)).expect("advances");
    assert!(
        early.events().is_empty(),
        "nothing a second before the boundary"
    );

    // Three days: six boundaries, each exactly once, alternating café and park.
    let mut seen = Vec::new();
    let mut at = SIX_AM - 1;
    for day in 0..3 {
        for boundary in [SIX_AM, SIX_PM] {
            let instant = day * DAY + boundary;
            let quiet = town.world.advance_to(t(instant - 1)).expect("advances");
            assert!(
                quiet.events().is_empty() || at == instant - 1,
                "nothing between boundaries"
            );
            let due = town.world.advance_to(t(instant)).expect("advances");
            assert_eq!(types(due.events()), ["agenda-changed"], "at t{instant}");
            let fact = &due.events()[0];
            assert_eq!(
                *fact.caused_by(),
                Causation::Process(routine),
                "caused by the routine process"
            );
            let changed = decoded(fact);
            assert_eq!(
                changed.routine(),
                routine,
                "the same process, never replaced"
            );
            seen.push((instant, changed.label().as_str().to_owned()));
            at = instant;
        }
    }
    assert_eq!(
        seen,
        [
            (SIX_AM, "work".to_owned()),
            (SIX_PM, "walk".to_owned()),
            (DAY + SIX_AM, "work".to_owned()),
            (DAY + SIX_PM, "walk".to_owned()),
            (2 * DAY + SIX_AM, "work".to_owned()),
            (2 * DAY + SIX_PM, "walk".to_owned()),
        ]
    );
    let now = agenda(&town);
    assert_eq!(now.place(), town.park);
    assert_eq!(now.since(), t(2 * DAY + SIX_PM));
    assert_eq!(
        now.until(),
        t(3 * DAY + SIX_AM),
        "the next boundary is tomorrow morning"
    );
    assert_eq!(
        town.world
            .read()
            .process(routine)
            .and_then(mineworld_kernel::Process::expected_end),
        Some(t(3 * DAY + SIX_AM)),
        "rescheduled, still running"
    );
}

/// Structural, before the behaviour: schedule provides no action, depends on nothing and emits only its
/// own two facts — stating presence's `arrived` would need a dependency the registry checks (`ARC-26`),
/// so it cannot move anybody, by construction rather than by restraint.
#[test]
fn schedule_states_only_its_own_vocabulary() {
    use mineworld_kernel::System;
    let declared = ScheduleSystem.declaration();
    assert!(declared.provides().is_empty(), "no action");
    assert!(declared.depends_on().is_empty(), "no dependency");
    assert!(
        declared.emits_owned_by_others().is_empty(),
        "no other pack's facts"
    );
    let emits: Vec<&str> = declared.emits().iter().map(|kind| kind.as_str()).collect();
    assert_eq!(emits, ["routine-assigned", "agenda-changed"]);
}

#[test]
fn an_agenda_moves_nobody() {
    let (mut town, _) = Town::begun();
    let where_she_was = town.world.read().component::<Presence>(town.alice).cloned();
    let advanced = town.world.advance_to(t(DAY + SIX_PM)).expect("advances");
    assert_eq!(
        types(advanced.events()),
        ["agenda-changed"; 4],
        "four agenda changes, and nothing else — no arrival, no entry"
    );
    assert_eq!(
        town.world.read().component::<Presence>(town.alice).cloned(),
        where_she_was,
        "she stands exactly where presence put her: an agenda is not a move (ARC-32)"
    );
    assert_ne!(
        agenda(&town).place(),
        where_she_was.expect("placed").location().place(),
        "although her agenda now says somewhere else"
    );
}

#[test]
fn every_routine_rule_is_refused_by_schedules_own_types_as_the_loader_decodes_it() {
    let refused = |yaml: &str, expected: &str| {
        let error = serde_saphyr::from_str::<<ScheduleSystem as AuthoredSection>::Authored>(yaml)
            .expect_err(yaml);
        assert!(
            error.to_string().contains(expected),
            "{yaml:?} is refused with {expected:?}: {error}"
        );
    };
    refused("[]", "a routine has 2 to 24 segments, but this one has 0");
    refused(
        "- { from: \"06:00\", place: cafe, label: work }",
        "but this one has 1",
    );
    refused(
        "- { from: \"18:00\", place: cafe, label: work }\n- { from: \"06:00\", place: park, label: walk }",
        "strictly increasing times, but 06:00 follows 18:00",
    );
    refused(
        "- { from: \"06:00\", place: cafe, label: work }\n- { from: \"06:00\", place: park, label: walk }",
        "strictly increasing",
    );
    refused(
        "- { from: \"24:00\", place: cafe, label: work }\n- { from: \"06:00\", place: park, label: walk }",
        "a time of day is \"HH:MM\"",
    );
    refused(
        "- { from: \"6:00\", place: cafe, label: work }\n- { from: \"18:00\", place: park, label: walk }",
        "a time of day is \"HH:MM\"",
    );
    refused(
        "- { from: \"06:00\", place: cafe, label: Work }\n- { from: \"18:00\", place: park, label: walk }",
        "an agenda label must be",
    );
    refused(
        "- { from: \"06:00\", place: cafe, label: work }\n- { from: \"12:00\", place: cafe, label: work }\n- { from: \"18:00\", place: park, label: walk }",
        "the segment from 12:00 repeats the one before it",
    );
    // The day wraps: the last segment is the first one's neighbour.
    refused(
        "- { from: \"06:00\", place: cafe, label: work }\n- { from: \"12:00\", place: park, label: walk }\n- { from: \"18:00\", place: cafe, label: work }",
        "the segment from 06:00 repeats the one before it",
    );
    refused(
        "- { from: \"06:00\", place: cafe, label: work, note: early }\n- { from: \"18:00\", place: park, label: walk }",
        "unknown field `note`",
    );

    let day: AuthoredRoutine = routine(ALICE_DAY);
    let named: Vec<String> = ScheduleSystem::references(&day)
        .iter()
        .map(|reference| {
            assert_eq!(reference.entity_type, EntityType::Place);
            reference.key.as_str().to_owned()
        })
        .collect();
    assert_eq!(
        named,
        ["cafe", "park"],
        "every segment's place, to be checked as a place"
    );
}

#[test]
fn a_routine_naming_a_person_where_a_place_belongs_is_refused_by_its_owner() {
    let (town, _) = Town::begun();
    let read = town.world.read();
    let seeding = Seeding::new(&read, &town.keys);
    let wrong = routine(
        "- { from: \"06:00\", place: bob, label: work }\n- { from: \"18:00\", place: park, label: walk }",
    );
    assert_eq!(
        ScheduleSystem::seed(&seeding, town.alice, &wrong).expect_err("bob is not a place"),
        mineworld_contracts::Rejection::PreconditionFailed
    );
    assert!(
        ScheduleSystem::seed(&seeding, town.keys[&key("cafe")], &routine(ALICE_DAY)).is_err(),
        "and a routine for a place is refused: a day is a person's"
    );
}

#[test]
fn an_agenda_is_disclosed_to_its_holder_and_to_nobody_else() {
    let (town, _) = Town::begun();
    let disclosed = |observer, subject| {
        town.observation(observer, t(0))
            .entity(subject)
            .map(|entity| {
                entity
                    .components()
                    .iter()
                    .any(|record| *record.component_type() == Agenda::COMPONENT_TYPE)
            })
            .unwrap_or(false)
    };
    assert!(
        disclosed(town.alice, town.alice),
        "Alice is told her own agenda"
    );
    assert!(
        town.observation(town.bob, t(0))
            .entity(town.alice)
            .is_some(),
        "Bob perceives Alice"
    );
    assert!(!disclosed(town.bob, town.alice), "and is not told hers");
}
