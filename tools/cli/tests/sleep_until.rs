//! `sleep-until` in Market Town through the real binary (pr-nightlife N2-C1).
//!
//! ```text
//! residents_decide_exactly_as_before_sleep_until   N2-I1b (F1): both hosted controllers leave the complete
//!                                                  `sleep-until` offer alone, so a two-day run prints the
//!                                                  base build's summary line for line
//! a_market_town_save_from_before_n2_is_refused…    N2-I14 (F6): the morning changed the sleep section's
//!                                                  configuration fact, so a pre-N2 save is refused by name
//!                                                  as drift and left as it was
//! ```
//!
//! Both read `fixtures/pre-n2-market-town/` (its SOURCE.md): written by the build before N2, never by this one.

mod headless;
mod market;

use std::path::Path;

use headless::{Tables, fresh, mineworld, stderr, stdout};
use market::MARKET_TOWN;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pre-n2-market-town"
);

/// The summary lines a comparison reads: all but the save path, the fact fingerprint and the wall time.
fn compared(summary: &str) -> Vec<&str> {
    summary
        .lines()
        .filter(|line| {
            !line.starts_with("[mineworld] run ")
                && !line.starts_with("history ")
                && !line.starts_with("wall ")
        })
        .collect()
}

/// N2-I1b: with `sleep-until` offered complete every night, the residents of Market Town decide exactly
/// as the build before it did — the same requests, facts, consults and per-person activity over two days
/// (seed 7): the same asleep share at every hour, the same sleep intervals. The only fact that differs is
/// the sleep section's configuration, which the fingerprint line covers and this comparison leaves out.
#[test]
fn residents_decide_exactly_as_before_sleep_until() {
    let saves = fresh("n2-rhythm");
    std::fs::create_dir_all(&saves).expect("writable");
    let save = saves.join("save");
    let output = mineworld(&[
        "run",
        MARKET_TOWN,
        "--headless",
        "--seed",
        "7",
        "--days",
        "2",
        "--save",
        save.to_str().expect("a path"),
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let now = stdout(&output);
    let before = std::fs::read_to_string(Path::new(FIXTURE).join("run-seed-7-days-2.txt"))
        .expect("the fixture reads")
        .replace("\r\n", "\n");
    assert!(
        !now.contains("requests   sleep-until"),
        "no resident asked to sleep until morning: {now}"
    );
    assert_eq!(compared(&now), compared(&before));
}

/// N2-I14 on a **real** earlier-build save: the binary refuses it by the configuration drift on `sleep`
/// — `morning_minute` is new in its configuration fact — and every committed row and the manifest stay
/// as they were (on a scratch copy; the fixture is never opened).
#[test]
fn a_market_town_save_from_before_n2_is_refused_by_drift_on_sleep_and_left_as_it_was() {
    let saves = fresh("sleep-until-pre-n2-save");
    let save = saves.join("save");
    std::fs::create_dir_all(&save).expect("writable");
    std::fs::copy(
        Path::new(FIXTURE).join("world.sqlite"),
        save.join("world.sqlite"),
    )
    .expect("copied");
    let before = Tables::read(&save);
    let output = mineworld(&[
        "run",
        MARKET_TOWN,
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
        "--save",
        save.to_str().expect("a path"),
    ]);
    let said = format!("{}{}", stderr(&output), stdout(&output));
    assert!(!output.status.success(), "refused: {said}");
    for part in [
        "the world's configuration differs from the save's",
        "system 'sleep'",
        "sleep-interactions-configured",
    ] {
        assert!(said.contains(part), "{part:?} missing from: {said}");
    }
    let after = Tables::read(&save);
    after.assert_same_history(&before, "a refused pre-N2 save");
    assert_eq!(after.manifest.instance, before.manifest.instance);
}
