//! T1 and T12 (c): at assembly, through the real World Pack loader with this pack in the installed set,
//! a bad `configure/happenings.yaml` is refused by name — naming the file, the line and column, and the
//! key where one value is wrong, or the entry where two disagree — and happenings without calendar is
//! refused naming calendar (design §5.1, AC-WE-a-8).

use mineworld_test_support::Scratch;
use mineworld_worldpack::WorldPack;

/// A valid configuration: §5.1's example, Market Town's shape.
const VALID: &str = "seed: 7
kinds:
  - id: market-day
    title: Saturday market
    places: [street]
    hours: \"08:00-14:00\"
    days: 1
    announce_days: 0
    audience: public
  - { id: festival, title: Harvest festival, places: [park], hours: \"16:00-22:00\", announce_days: 7 }
  - { id: street-musician, places: [street, park], hours: \"11:00-13:00\", audience: place }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
  - { kind: festival, place: park, date: \"10-31\" }
  - { kind: market-day, place: street, monthday: 1 }
random:
  - { kind: street-musician, place: street, per_mille_per_day: 150 }
  - { kind: street-musician, place: park, per_mille_per_day: 60, weekdays: [saturday, sunday], months: [5, 6, 7, 8, 9] }
limits:
  per_day: 3
  per_place_running: 1
generators: []
";

const CALENDAR: &str =
    "epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 32.7157\nlongitude: -117.1611\n";

/// A scratch World Pack — a street and a park, Ada on the street — enabling `systems` and configuring
/// `configure`, with `configure/happenings.yaml` as given; removed when the test ends (DEP-29).
fn pack(id: &str, systems: &[&str], configure: &[&str], happenings: &str) -> Scratch {
    let scratch = mineworld_test_support::scratch!(empty id);
    let root = scratch.path();
    for directory in ["people", "places", "configure"] {
        std::fs::create_dir_all(root.join(directory)).expect("writable");
    }
    let write = |file: &str, text: &str| std::fs::write(root.join(file), text).expect("writable");
    let list =
        |items: &[&str]| -> String { items.iter().map(|item| format!("  - {item}\n")).collect() };
    write(
        "world.yaml",
        &format!(
            "world:\n  id: {id}\n  name: A Town With Happenings\nsystems:\n{}configure:\n{}places:\n  \
             - street\n  - park\npopulation:\n  - ada\n",
            list(systems),
            list(configure)
        ),
    );
    write("places/street.yaml", "tags: [street]\n");
    write("places/park.yaml", "tags: [park]\n");
    write("people/ada.yaml", "location:\n  place: street\n");
    if configure.contains(&"calendar") {
        write("configure/calendar.yaml", CALENDAR);
    }
    write("configure/happenings.yaml", happenings);
    scratch
}

fn enabled(id: &str, happenings: &str) -> Scratch {
    pack(
        id,
        &["presence", "calendar", "happenings"],
        &["calendar", "happenings"],
        happenings,
    )
}

fn refusal(scratch: &Scratch) -> String {
    WorldPack::read(scratch.path())
        .and_then(|pack| pack.assemble().map(|_| ()))
        .expect_err("refused")
        .to_string()
}

fn file() -> String {
    std::path::Path::new("configure")
        .join("happenings.yaml")
        .to_string_lossy()
        .into_owned()
}

/// The valid configuration assembles and seeds one `happenings-configured`; `until: 24:00` is legal.
#[test]
fn a_valid_configuration_assembles_and_seeds_its_fact() {
    for (id, text) in [
        ("happening-town", VALID.to_owned()),
        (
            "midnight-town",
            VALID.replace("\"11:00-13:00\"", "\"20:00-24:00\""),
        ),
    ] {
        let good = enabled(id, &text);
        let assembled = WorldPack::read(good.path())
            .expect("reads")
            .assemble()
            .expect("assembles");
        let configured: Vec<_> = assembled
            .facts
            .iter()
            .filter(|fact| fact.event_type().as_str() == "happenings-configured")
            .collect();
        assert_eq!(configured.len(), 1, "{id}");
    }
}

/// T12 (c): happenings enabled without calendar is refused at assembly, naming both.
#[test]
fn happenings_without_calendar_is_refused_naming_calendar() {
    let lonely = pack(
        "dateless-town",
        &["presence", "happenings"],
        &["happenings"],
        VALID,
    );
    let refused = refusal(&lonely);
    eprintln!("REFUSAL {refused}");
    assert!(
        refused.contains("happenings"),
        "names happenings: {refused}"
    );
    assert!(refused.contains("calendar"), "names calendar: {refused}");
}

/// One wrong value is refused naming the file, its line and column, and the key.
#[test]
fn a_bad_value_is_refused_naming_file_position_and_key() {
    for (id, text, says) in [
        (
            "unknown-key-town",
            format!("colour: red\n{VALID}"),
            "colour",
        ),
        (
            "keen-musician-town",
            VALID.replace("per_mille_per_day: 150", "per_mille_per_day: 1001"),
            "per_mille_per_day",
        ),
        (
            "backwards-town",
            VALID.replace("\"08:00-14:00\"", "\"14:00-08:00\""),
            "hours",
        ),
        (
            "endless-town",
            VALID.replace("\"08:00-14:00\"", "\"24:00-24:00\""),
            "hours",
        ),
        (
            "smirch-town",
            VALID.replace("\"10-31\"", "\"13-01\""),
            "date",
        ),
        (
            "early-town",
            VALID.replace("announce_days: 7", "announce_days: 31"),
            "announce_days",
        ),
        (
            "busy-town",
            VALID.replace("per_day: 3", "per_day: 17"),
            "per_day",
        ),
        (
            "shouting-town",
            VALID.replace("id: market-day", "id: Market-Day"),
            "kind id",
        ),
        (
            "long-town",
            VALID.replace("    days: 1\n", "    days: 15\n"),
            "days",
        ),
        (
            "both-town",
            VALID.replace(
                "weekdays: [saturday] }",
                "weekdays: [saturday], monthday: 2 }",
            ),
            "exactly one of weekdays, date and monthday",
        ),
        (
            "late-town",
            VALID.replace("monthday: 1", "monthday: 31"),
            "monthday is 1 to 28",
        ),
        (
            "funday-town",
            VALID.replace("weekdays: [saturday]", "weekdays: [funday]"),
            "weekdays",
        ),
        (
            "thirteenth-town",
            VALID.replace("months: [5, 6, 7, 8, 9]", "months: [5, 13]"),
            "months",
        ),
    ] {
        let bad = enabled(id, &text);
        let refused = refusal(&bad);
        eprintln!("REFUSAL {refused}");
        assert!(refused.contains(&file()), "names the file: {refused}");
        assert!(
            refused.contains("line") && refused.contains("column"),
            "{id}: {refused}"
        );
        assert!(refused.contains(says), "{id} names {says}: {refused}");
    }
}

/// Entries that disagree with the catalog or each other are refused naming the file and the entry.
#[test]
fn a_disagreement_between_entries_is_refused_naming_the_entry() {
    for (id, text, says) in [
        (
            "twin-town",
            VALID.replace("id: festival", "id: market-day"),
            "the kind 'market-day' is declared twice",
        ),
        (
            "undeclared-town",
            VALID.replace(
                "{ kind: festival, place: park, date",
                "{ kind: fair, place: park, date",
            ),
            "calendar[1] names the kind 'fair'",
        ),
        (
            "wandering-town",
            VALID.replace(
                "{ kind: market-day, place: street, weekdays",
                "{ kind: market-day, place: park, weekdays",
            ),
            "calendar[0] puts 'market-day' at 'park', which is not one of its kind's places",
        ),
        (
            "echo-town",
            VALID.replace(
                "place: park, per_mille_per_day: 60",
                "place: street, per_mille_per_day: 60",
            ),
            "random[0] and random[1] both draw 'street-musician' at 'street'",
        ),
        (
            "repeated-generator-town",
            VALID.replace("generators: []", "generators: [fair-maker, fair-maker]"),
            "generators lists 'fair-maker' twice",
        ),
        (
            "empty-town",
            "seed: 7\nkinds: []\n".to_owned(),
            "kinds lists 1 to 64 kinds, not 0",
        ),
    ] {
        let bad = enabled(id, &text);
        let refused = refusal(&bad);
        eprintln!("REFUSAL {refused}");
        assert!(refused.contains(&file()), "names the file: {refused}");
        assert!(refused.contains(says), "{id}: {says}: {refused}");
    }
}

/// A place key that is no Place of the world is refused by the loader, naming the key and the file.
#[test]
fn a_kind_at_an_unknown_place_is_refused_naming_it() {
    let bad = enabled(
        "placeless-town",
        &VALID
            .replace("places: [park]", "places: [pier]")
            .replace("place: park, date", "place: pier, date"),
    );
    let refused = refusal(&bad);
    eprintln!("REFUSAL {refused}");
    assert!(refused.contains("pier"), "names the key: {refused}");
    assert!(refused.contains("happenings"), "names the file: {refused}");
}
