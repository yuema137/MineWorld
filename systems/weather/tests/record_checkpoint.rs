//! CP-TW-d (step-19 §18.6), read from saves the binary wrote:
//!
//! ```text
//! mineworld run worlds/market-town --headless --seed 1 --days 365 --save U     # uninterrupted
//! mineworld run worlds/market-town --headless --seed 1 --days 200 --save R     # then resumed:
//! mineworld run worlds/market-town --headless --seed 1 --days 365 --save R
//! TWD_CP_U=U TWD_CP_R=R cargo test -p mineworld-weather --test record_checkpoint -- --ignored --nocapture
//! ```
//!
//! 1. Seasonality: more wet days (at least 0.3 mm) dated December–February than June–August.
//! 2. Fog: at least one `weather-day` with fog in May–July (world 2027, record 2016; the tool's report
//!    shows WT01/WT02 populated in 2016, E-TWd-4).
//! 3. Exact replay: world 2026-10-08 is `Record { 2015-10-08 }` and world 2027-10-08 — day 365, at the
//!    run's last instant — is `Record { 2016-10-08 }`, each with TMAX, TMIN and PRCP equal to the
//!    committed CSV's row, read from the file here.
//! 4. Restart: the save resumed at day 200 holds the same facts as the uninterrupted one, byte for byte.
//!
//! Opt-in because the saves are several GB each (F-TWbd-1, F-SAVE-1).

use std::path::PathBuf;

use mineworld_contracts::{Event, EventEnvelope};
use mineworld_persistence::{Durability, PersistenceBackend, SqliteBackend};
use mineworld_weather::record::{RecordRow, decode};
use mineworld_weather::{CalendarDate, Origin, WeatherDay};

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

fn record_row(on: CalendarDate) -> RecordRow {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../worlds/market-town/data/weather/san-diego-usw00023188-2015-2024.csv");
    let file = decode(&std::fs::read(path).expect("the committed record")).expect("decodes");
    *file
        .rows()
        .iter()
        .find(|row| row.date == on)
        .expect("in the record")
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day).expect("a date")
}

#[test]
#[ignore = "opt-in: needs two 365-day saves (TWD_CP_U, TWD_CP_R)"]
fn cp_tw_d() {
    let uninterrupted = rows("TWD_CP_U");
    assert!(
        uninterrupted == rows("TWD_CP_R"),
        "4. resumed at day 200: the same facts, byte for byte"
    );
    let days: Vec<(i64, WeatherDay)> = uninterrupted
        .iter()
        .map(|(_, bytes)| serde_json::from_slice::<EventEnvelope>(bytes).expect("a fact"))
        .filter(|fact| *fact.event_type() == WeatherDay::EVENT_TYPE)
        .map(|fact| {
            (
                fact.at().seconds(),
                serde_json::from_slice(fact.payload().payload_for::<WeatherDay>().expect("ours"))
                    .expect("decodes"),
            )
        })
        .collect();
    assert_eq!(days.len(), 366, "days 0 … 365");

    let wet = |months: &[u8]| {
        days.iter()
            .filter(|(_, day)| {
                months.contains(&day.date().month()) && day.summary().prcp_tenth_mm >= 3
            })
            .count()
    };
    let (winter, summer) = (wet(&[12, 1, 2]), wet(&[6, 7, 8]));
    println!("1. wet days: December–February {winter}, June–August {summer}");
    assert!(winter > summer, "1. seasonality");

    let fog: Vec<String> = days
        .iter()
        .filter(|(_, day)| (5..=7).contains(&day.date().month()) && day.summary().fog)
        .map(|(_, day)| format!("{:?}", day.date()))
        .collect();
    println!("2. fog days in May–July: {}", fog.len());
    assert!(!fog.is_empty(), "2. fog");

    for (index, world, record) in [
        (0usize, date(2026, 10, 8), date(2015, 10, 8)),
        (365, date(2027, 10, 8), date(2016, 10, 8)),
    ] {
        let (at, day) = &days[index];
        assert_eq!(*at, i64::try_from(index).expect("small") * 86_400);
        assert_eq!(day.date(), world);
        assert_eq!(
            day.origin(),
            Origin::Record { date: record },
            "3. the record date"
        );
        let row = record_row(record);
        let summary = day.summary();
        println!(
            "3. world {world:?} replays {record:?}: TMAX {} TMIN {} PRCP {} (CSV {:?} {:?} {:?})",
            summary.tmax_dc,
            summary.tmin_dc,
            summary.prcp_tenth_mm,
            row.tmax_dc,
            row.tmin_dc,
            row.prcp_tenth_mm
        );
        assert_eq!(
            (
                Some(summary.tmax_dc),
                Some(summary.tmin_dc),
                Some(summary.prcp_tenth_mm)
            ),
            (row.tmax_dc, row.tmin_dc, row.prcp_tenth_mm),
            "3. the CSV row's cells"
        );
    }
    let origins = days.iter().fold([0usize; 3], |mut counts, (_, day)| {
        counts[match day.origin() {
            Origin::Record { .. } => 0,
            Origin::Filled { .. } => 1,
            Origin::Rule => 2,
        }] += 1;
        counts
    });
    println!(
        "origins: record {}, filled {}, rule {}",
        origins[0], origins[1], origins[2]
    );
}
