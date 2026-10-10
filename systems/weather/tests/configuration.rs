//! At assembly, through the real World Pack loader with this pack in the installed set: weather without
//! calendar is refused (criterion 4), and a bad `configure/weather.yaml` is refused naming the file, its
//! line and column, and the key (C3 (d), (i)).

use mineworld_test_support::Scratch;
use mineworld_worldpack::WorldPack;

/// A valid weather configuration: two seasons' worth of months, all the same.
fn valid() -> String {
    let month = "    - { p_wet_after_dry: 147, p_wet_after_wet: 447, rain_tenth_mm: [8, 28, 54, 93, 178], \
                 tmax_dc: 191, tmin_dc: 102, t_noise_dc: 30, t_ar_permille: 600, \
                 wet_tmax_shift_dc: -20, fog_permille: 80, thunder_permille: 40, \
                 overcast_morning_permille: 150, wind_dms: 25, wind_from_deg: 300 }\n";
    format!(
        "source: rules\nseed: 19\nrules:\n  months:\n{}",
        month.repeat(12)
    )
}

const CALENDAR: &str =
    "epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 32.7157\nlongitude: -117.1611\n";

/// A scratch World Pack — a square, Ada in it — enabling `systems` and configuring `configure`, with
/// `configure/weather.yaml` as given; named as the pack and removed when the test ends (DEP-29).
fn pack(id: &str, systems: &[&str], configure: &[&str], weather: &str) -> Scratch {
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
            "world:\n  id: {id}\n  name: A Weathered Square\nsystems:\n{}configure:\n{}places:\n  - \
             square\npopulation:\n  - ada\n",
            list(systems),
            list(configure)
        ),
    );
    write("places/square.yaml", "tags: [square]\n");
    write("people/ada.yaml", "location:\n  place: square\n");
    if configure.contains(&"calendar") {
        write("configure/calendar.yaml", CALENDAR);
    }
    write("configure/weather.yaml", weather);
    scratch
}

fn refusal(scratch: &Scratch) -> String {
    WorldPack::read(scratch.path())
        .and_then(|pack| pack.assemble().map(|_| ()))
        .expect_err("refused")
        .to_string()
}

/// (d) Criterion 4: weather enabled without calendar is refused at assembly, naming both.
#[test]
fn weather_without_calendar_is_refused_at_assembly() {
    let good = pack(
        "weathered-square",
        &["presence", "calendar", "weather"],
        &["calendar", "weather"],
        &valid(),
    );
    let assembled = WorldPack::read(good.path())
        .expect("reads")
        .assemble()
        .expect("assembles");
    assert!(
        assembled
            .facts
            .iter()
            .any(|fact| fact.event_type().as_str() == "weather-configured")
    );

    let lonely = pack(
        "dateless-square",
        &["presence", "weather"],
        &["weather"],
        &valid(),
    );
    let refused = refusal(&lonely);
    eprintln!("REFUSAL {refused}");
    assert!(refused.contains("weather"), "names weather: {refused}");
    assert!(refused.contains("calendar"), "names calendar: {refused}");
}

/// (i) A bad configuration is refused at assembly, naming the file (by the platform's separator), the
/// line and column, and the key.
#[test]
fn a_bad_weather_configuration_is_refused_naming_file_position_and_key() {
    let file = std::path::Path::new("configure").join("weather.yaml");
    let file = file.to_string_lossy().into_owned();
    for (id, text, key) in [
        (
            "recorded-square",
            valid().replace("source: rules", "source: record"),
            "record",
        ),
        (
            "wet-square",
            valid().replacen("p_wet_after_wet: 447", "p_wet_after_wet: 1447", 1),
            "p_wet_after_wet",
        ),
        (
            "cold-square",
            valid().replacen("tmin_dc: 102", "tmin_dc: 300", 1),
            "tmin_dc",
        ),
        (
            "short-square",
            valid().replacen(
                "rain_tenth_mm: [8, 28, 54, 93, 178]",
                "rain_tenth_mm: [8, 28]",
                1,
            ),
            "rain_tenth_mm",
        ),
    ] {
        let bad = pack(
            id,
            &["presence", "calendar", "weather"],
            &["calendar", "weather"],
            &text,
        );
        let refused = refusal(&bad);
        eprintln!("REFUSAL {refused}");
        assert!(refused.contains(&file), "names the file: {refused}");
        assert!(
            refused.contains("line") && refused.contains("column"),
            "{refused}"
        );
        assert!(refused.contains(key), "names {key}: {refused}");
    }
}
