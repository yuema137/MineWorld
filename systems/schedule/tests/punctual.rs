//! A punctual part of the day, and the next part disclosed to its holder (`ARC-280`, SD-LE-1, SD-LE-2).
//!
//! ```text
//! section     punctual: true accepted; absent is false; yes, 1 and "true" refused at their position
//! bytes       a routine with no punctual part is assigned with the same payload bytes as before
//! next        the part that begins at the agenda's until: mid-day, from the last part to the first
//!             (next day), across midnight, and in a routine of two parts
//! audience    the next part travels in the holder's own agenda record, to nobody else
//! ```

mod support;

use mineworld_contracts::Component;
use mineworld_schedule::{
    Agenda, AgendaLabel, AuthoredRoutine, DAY, DisclosedAgenda, RoutineAssigned,
};
use support::{Town, routine, t, types};

const PUNCTUAL_DAY: &str = "
- { from: \"00:30\", place: park, label: night }
- { from: \"05:30\", place: cafe, label: work, punctual: true }
- { from: \"14:00\", place: park, label: walk }
";

fn label(text: &str) -> AgendaLabel {
    serde_json::from_value(serde_json::Value::from(text)).expect("a label")
}

/// The agenda record Alice is told at `at`, decoded as its disclosed shape.
fn told(town: &Town, at: i64) -> DisclosedAgenda {
    let observation = town.observation(town.alice, t(at));
    let record = observation
        .entity(town.alice)
        .and_then(|entity| {
            entity
                .components()
                .iter()
                .find(|record| *record.component_type() == Agenda::COMPONENT_TYPE)
        })
        .expect("Alice is told her agenda");
    let value = record.payload().clone();
    // Every existing decoder of the record reads it as the stored Agenda: it still must.
    let _: Agenda = serde_json::from_value(value.clone()).expect("decodes as Agenda");
    serde_json::from_value(value).expect("decodes as DisclosedAgenda")
}

fn advance(town: &mut Town, at: i64) {
    let _ = town.world.advance_to(t(at)).expect("advances");
}

#[test]
fn punctual_is_a_flag_absent_by_default_and_refused_unless_a_boolean() {
    let day: AuthoredRoutine = routine(PUNCTUAL_DAY);
    let flags: Vec<bool> = day.segments().iter().map(|part| part.punctual()).collect();
    assert_eq!(flags, [false, true, false]);

    // `yes` (and a quoted "true") is a YAML 1.1 boolean the loader accepts for every flag
    // (serde-saphyr's default), so it is `true` here as everywhere else; a number and any other word
    // are refused at their position.
    for wrong in ["1", "sometimes"] {
        let yaml = format!(
            "- {{ from: \"06:00\", place: cafe, label: work }}\n- {{ from: \"18:00\", place: park, label: walk, punctual: {wrong} }}"
        );
        let error = serde_saphyr::from_str::<AuthoredRoutine>(&yaml).expect_err(&yaml);
        let message = error.to_string();
        assert!(
            message.contains("line 2"),
            "{wrong} is refused at its line: {message}"
        );
    }
}

#[test]
fn a_routine_with_no_punctual_part_keeps_its_payload_bytes() {
    let (town, genesis) = Town::begun();
    assert_eq!(types(&genesis)[2], "routine-assigned");
    let payload = genesis[2]
        .payload()
        .payload_for::<RoutineAssigned>()
        .expect("schedule's fact");
    // The literal bytes of the base build: ids are this town's creation order.
    let expected = "{\"person\":{\"entity\":\"3\",\"entity_type\":\"person\"},\"segments\":[{\"from\":\"06:00\",\"place\":{\"entity\":\"1\",\"entity_type\":\"place\"},\"label\":\"work\"},{\"from\":\"18:00\",\"place\":{\"entity\":\"2\",\"entity_type\":\"place\"},\"label\":\"walk\"}]}";
    assert_eq!(
        (town.cafe.entity_id(), town.park.entity_id()),
        (
            town.keys[&support::key("cafe")],
            town.keys[&support::key("park")]
        )
    );
    assert_eq!(String::from_utf8_lossy(payload), expected);

    let (_, genesis) = Town::begun_with(PUNCTUAL_DAY);
    let payload = genesis[2]
        .payload()
        .payload_for::<RoutineAssigned>()
        .expect("schedule's fact");
    assert!(
        String::from_utf8_lossy(payload).contains("\"label\":\"work\",\"punctual\":true}"),
        "a punctual part is written: {}",
        String::from_utf8_lossy(payload)
    );
}

#[test]
fn the_next_part_is_the_one_that_begins_when_the_agenda_ends() {
    let (mut town, _) = Town::begun_with(PUNCTUAL_DAY);

    // Midnight: the walk begun yesterday at 14:00 runs until 00:30; next is the night part.
    let now = told(&town, 0);
    assert_eq!(now.agenda.label(), &label("walk"));
    let next = now.next.expect("a next part");
    assert_eq!(
        (next.place, next.label, next.from, next.punctual),
        (town.park, label("night"), t(1_800), false)
    );

    // 01:00: the night part; next is work at 05:30, punctual.
    advance(&mut town, 3_600);
    let next = told(&town, 3_600).next.expect("a next part");
    assert_eq!(
        (next.place, next.label, next.from, next.punctual),
        (town.cafe, label("work"), t(5 * 3_600 + 1_800), true)
    );

    // 15:00, the last part of the day: next is the first part, tomorrow.
    advance(&mut town, 15 * 3_600);
    let next = told(&town, 15 * 3_600).next.expect("a next part");
    assert_eq!(
        (next.place, next.label, next.from, next.punctual),
        (town.park, label("night"), t(DAY + 1_800), false)
    );

    // 00:45 the next day, across midnight: work at 05:30 that day.
    advance(&mut town, DAY + 2_700);
    let next = told(&town, DAY + 2_700).next.expect("a next part");
    assert_eq!(
        (next.label, next.from, next.punctual),
        (label("work"), t(DAY + 5 * 3_600 + 1_800), true)
    );
}

#[test]
fn in_a_routine_of_two_parts_the_next_part_is_the_other_one() {
    let (mut town, _) = Town::begun();
    let next = told(&town, 0).next.expect("a next part");
    assert_eq!((next.place, next.from), (town.cafe, t(6 * 3_600)));
    advance(&mut town, 7 * 3_600);
    let next = told(&town, 7 * 3_600).next.expect("a next part");
    assert_eq!((next.place, next.from), (town.park, t(18 * 3_600)));
}

#[test]
fn the_next_part_is_told_to_its_holder_only() {
    let (town, _) = Town::begun_with(PUNCTUAL_DAY);
    let observation = town.observation(town.bob, t(0));
    let about_alice = observation.entity(town.alice).expect("Bob perceives Alice");
    assert!(
        about_alice
            .components()
            .iter()
            .all(|record| *record.component_type() != Agenda::COMPONENT_TYPE),
        "neither her agenda nor its next part"
    );
}
