//! CP-TW-b (step-19 §17.6), read from saves the binary wrote:
//!
//! ```text
//! mineworld run worlds/market-town --headless --seed 1 --days 120 --save A     # and again into B
//! mineworld run worlds/market-town --headless --seed 1 --days 60  --save R     # then resumed:
//! mineworld run worlds/market-town --headless --seed 1 --days 120 --save R
//! TWB_CP_A=A TWB_CP_B=B TWB_CP_RESUMED=R \
//!   cargo test -p mineworld-weather --test checkpoint -- --ignored --nocapture
//! ```
//!
//! It checks that the three saves hold the same facts byte for byte (two runs, and a run resumed at day
//! 60), that the 121 `weather-day` are dated 2026-10-08 … 2027-02-05, that there is at least one wet
//! day and one dry run of at least seven days, and that every `weather-changed` is what its day's hours
//! imply. Opt-in because the saves are about 1 GB each (F-TWbd-1).

use std::path::PathBuf;

use mineworld_contracts::{Event, EventEnvelope};
use mineworld_persistence::{Durability, PersistenceBackend, SqliteBackend};
use mineworld_weather::{Condition, WeatherChanged, WeatherDay};

const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;

/// Every fact row of the save in `variable`, oldest first, as stored.
fn rows(variable: &str) -> Vec<(u64, Vec<u8>)> {
    let directory = PathBuf::from(std::env::var(variable).unwrap_or_else(|_| panic!("{variable}")));
    SqliteBackend::open(&directory, Durability::ProcessCrash)
        .expect("opens")
        .last_facts(usize::MAX >> 2)
        .expect("reads")
        .into_iter()
        .map(|row| (row.id, row.bytes))
        .collect()
}

fn of<E: Event + for<'de> serde::Deserialize<'de>>(facts: &[EventEnvelope]) -> Vec<(i64, E)> {
    facts
        .iter()
        .filter(|fact| *fact.event_type() == E::EVENT_TYPE)
        .map(|fact| {
            let bytes = fact.payload().payload_for::<E>().expect("its type");
            (
                fact.at().seconds(),
                serde_json::from_slice(bytes).expect("decodes"),
            )
        })
        .collect()
}

#[test]
#[ignore = "opt-in: needs three 120-day saves (TWB_CP_A, TWB_CP_B, TWB_CP_RESUMED)"]
fn cp_tw_b() {
    let a = rows("TWB_CP_A");
    assert_eq!(
        a,
        rows("TWB_CP_B"),
        "two runs: the same facts, byte for byte"
    );
    assert_eq!(
        a,
        rows("TWB_CP_RESUMED"),
        "resumed at day 60: the same facts, byte for byte"
    );
    let facts: Vec<EventEnvelope> = a
        .iter()
        .map(|(_, bytes)| serde_json::from_slice(bytes).expect("a fact"))
        .collect();

    let days = of::<WeatherDay>(&facts);
    assert_eq!(days.len(), 121);
    let date = |day: &WeatherDay| {
        let date = day.date();
        (date.year(), date.month(), date.day())
    };
    assert_eq!(date(&days[0].1), (2026, 10, 8));
    assert_eq!(date(&days[120].1), (2027, 2, 5));
    for (index, (at, day)) in days.iter().enumerate() {
        assert_eq!(*at, i64::try_from(index).expect("small") * DAY);
        assert_eq!(day.day_start().seconds(), *at);
    }

    let wet: Vec<bool> = days.iter().map(|(_, day)| day.summary().wet()).collect();
    let wet_days = wet.iter().filter(|w| **w).count();
    let longest_dry = wet.split(|w| *w).map(<[bool]>::len).max().unwrap_or(0);
    let total_mm: u32 = days
        .iter()
        .map(|(_, day)| u32::from(day.summary().prcp_tenth_mm))
        .sum();
    println!(
        "121 days 2026-10-08 … 2027-02-05: {wet_days} wet, longest dry run {longest_dry} days, \
         {}.{} mm in all",
        total_mm / 10,
        total_mm % 10
    );
    assert!(wet_days >= 1, "at least one wet day");
    assert!(longest_dry >= 7, "at least one dry run of seven days");

    // Every change is what the days' hours imply, across midnights, through the last instant.
    let end = 120 * DAY;
    let mut previous: Option<Condition> = None;
    let mut implied = Vec::new();
    for (_, day) in &days {
        for (hour, weather) in day.hours().iter().enumerate() {
            let at = day.day_start().seconds() + i64::try_from(hour).expect("small") * HOUR;
            if at > end {
                break;
            }
            if previous != Some(weather.condition) {
                implied.push((at, u8::try_from(hour).expect("small"), weather.condition));
            }
            previous = Some(weather.condition);
        }
    }
    let recorded: Vec<(i64, u8, Condition)> = of::<WeatherChanged>(&facts)
        .into_iter()
        .map(|(at, changed)| (at, changed.hour(), changed.condition()))
        .collect();
    println!(
        "{} weather-changed, each as the hours imply",
        recorded.len()
    );
    assert_eq!(recorded, implied);
}
