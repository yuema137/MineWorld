//! TW-a adversarial criterion 5: an invalid `configure/calendar.yaml` is refused **at assembly**,
//! naming the file and the key — through the real World Pack loader, with this pack installed in the
//! build's installed set — and every out-of-range value the configuration admits is refused by
//! decoding.

use mineworld_calendar::CalendarSystem;
use mineworld_sdk::SystemPack;
use mineworld_worldpack::WorldPack;

const VALID: &str =
    "epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 32.7157\nlongitude: -117.1611\n";

/// A scratch World Pack — a square, Ada in it, presence and calendar enabled, `configure/calendar.yaml`
/// as given — named as the pack and removed when the test ends (DEP-29).
fn pack(id: &str, configuration: &str) -> mineworld_test_support::Scratch {
    let scratch = mineworld_test_support::scratch!(empty id);
    let root = scratch.path();
    for directory in ["people", "places", "configure"] {
        std::fs::create_dir_all(root.join(directory)).expect("writable");
    }
    let write = |file: &str, text: &str| std::fs::write(root.join(file), text).expect("writable");
    write(
        "world.yaml",
        &format!(
            "world:\n  id: {id}\n  name: A Dated Square\nsystems:\n  - presence\n  - calendar\n\
             configure:\n  - calendar\nplaces:\n  - square\npopulation:\n  - ada\n"
        ),
    );
    write("places/square.yaml", "tags: [square]\n");
    write("people/ada.yaml", "location:\n  place: square\n");
    write("configure/calendar.yaml", configuration);
    scratch
}

#[test]
fn a_latitude_of_91_is_refused_at_assembly_naming_the_file_and_the_key() {
    let good = pack("dated-square", VALID);
    let read = WorldPack::read(good.path()).expect("the valid pack reads");
    let assembled = read.assemble().expect("and assembles");
    assert!(
        assembled
            .facts
            .iter()
            .any(|fact| fact.event_type().as_str() == "calendar-configured"),
        "its genesis carries the configuration"
    );

    let bad = pack(
        "misplaced-square",
        "epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 91\nlongitude: -117.1611\n",
    );
    let refusal = WorldPack::read(bad.path())
        .and_then(|pack| pack.assemble().map(|_| ()))
        .expect_err("latitude 91 is refused")
        .to_string();
    eprintln!("REFUSAL {refusal}");
    // The file, by its components: the separator is the platform's (macOS, Linux and Windows).
    let file = std::path::Path::new("configure").join("calendar.yaml");
    assert!(
        refusal.contains(&*file.to_string_lossy()),
        "names the file: {refusal}"
    );
    assert!(refusal.contains("latitude"), "names the key: {refusal}");
    assert!(refusal.contains("91"), "names the value: {refusal}");
}

/// The owner's types refuse every value outside the configuration's range, each naming its key; and
/// the values at each bound are accepted.
#[test]
fn every_out_of_range_value_is_refused_by_decoding_and_names_its_key() {
    let decode = |text: &str| {
        serde_saphyr::with_deserializer_from_str(text, |file| {
            CalendarSystem::decode_configuration(file)
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
    };
    let with = |key: &str, value: &str| {
        VALID
            .lines()
            .map(|line| {
                if line.starts_with(&format!("{key}:")) {
                    format!("{key}: {value}")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(decode(VALID), Ok(()));
    for (key, value) in [
        ("latitude", "-90.5"),
        ("longitude", "180.000001"),
        ("longitude", "-181"),
        ("utc_offset", "-14:30"),
        ("utc_offset", "08:00"),
        ("utc_offset", "+05:60"),
        ("utc_offset", "-8"),
        ("epoch", "1900-12-31"),
        ("epoch", "2100-01-01"),
        ("epoch", "2026-02-29"),
        ("epoch", "2026-10-8"),
    ] {
        let refusal = decode(&with(key, value)).expect_err(&format!("{key}: {value}"));
        assert!(refusal.contains(key), "{key}: {value} → {refusal}");
    }
    for (key, value) in [
        ("latitude", "90"),
        ("latitude", "-90"),
        ("longitude", "180"),
        ("utc_offset", "+14:00"),
        ("utc_offset", "-14:00"),
        ("utc_offset", "+05:30"),
        ("epoch", "1901-01-01"),
        ("epoch", "2099-12-31"),
        ("epoch", "2028-02-29"),
    ] {
        assert_eq!(decode(&with(key, value)), Ok(()), "{key}: {value}");
    }
    let unknown = decode(&format!("{VALID}dst: true\n")).expect_err("an unknown key");
    assert!(unknown.contains("dst"), "{unknown}");
}
