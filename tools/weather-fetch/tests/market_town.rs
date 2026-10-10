//! Market Town's weather (step-19 §18.4 C5): its rules are exactly `fit` of its committed record
//! (SD-TW-d-10, criterion 7), and those rules reproduce the record's climate — each month's wet-day
//! frequency within ±3 points and the pooled mean wet-spell length within ±15 % over 100 world years
//! (criterion 5).

use std::path::PathBuf;

use mineworld_weather::generate::day;
use mineworld_weather::record::{RecordFile, RecordRow, decode};
use mineworld_weather::{CalendarDate, Chain, WeatherConfiguration};
use mineworld_weather_fetch::fit::fit;

fn town(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../worlds/market-town")
        .join(file)
}

/// A text file as committed, read as LF whatever the checkout did to it.
fn read(file: &str) -> String {
    std::fs::read_to_string(town(file))
        .unwrap_or_else(|error| panic!("{file}: {error}"))
        .replace("\r\n", "\n")
}

fn record() -> RecordFile {
    let bytes = std::fs::read(town("data/weather/san-diego-usw00023188-2015-2024.csv"))
        .expect("the record is readable");
    decode(&bytes).expect("the record decodes")
}

/// Criterion 7: the committed configure/weather.yaml is, byte for byte, what `fit` writes for the
/// committed record with it as the base — so the data and its fitted rules cannot drift apart.
#[test]
fn the_committed_rules_are_the_fit_of_the_committed_record() {
    let committed = read("configure/weather.yaml");
    let fitted = fit(&record(), &committed, None).expect("fits");
    assert!(
        fitted == committed,
        "configure/weather.yaml is not `fit` of the record; re-run tools/weather-fetch fit"
    );
}

const WET: u16 = 3;

/// Per month, wet days and days; and every wet spell's length (a spell ends at a dry or missing day).
fn climate(days: impl Iterator<Item = (u8, Option<u16>)>) -> ([(u32, u32); 12], Vec<u32>) {
    let mut months = [(0u32, 0u32); 12];
    let mut spells = Vec::new();
    let mut spell = 0u32;
    for (month, prcp) in days {
        let wet = prcp.is_some_and(|p| p >= WET);
        if prcp.is_some() {
            let entry = &mut months[usize::from(month) - 1];
            entry.1 += 1;
            entry.0 += u32::from(wet);
        }
        if wet {
            spell += 1;
        } else if spell > 0 {
            spells.push(spell);
            spell = 0;
        }
    }
    if spell > 0 {
        spells.push(spell);
    }
    (months, spells)
}

fn rows_climate(rows: &[RecordRow]) -> ([(u32, u32); 12], Vec<u32>) {
    climate(rows.iter().map(|row| {
        let prcp = if row.complete() {
            row.prcp_tenth_mm
        } else {
            None
        };
        (row.date.month(), prcp)
    }))
}

/// Criterion 5 (M-TWd-5 must turn both halves red).
#[test]
fn the_fitted_rules_reproduce_the_records_wet_days_and_spells() {
    let file = record();
    let (recorded, recorded_spells) = rows_climate(file.rows());
    let configuration: WeatherConfiguration =
        serde_saphyr::from_str(&read("configure/weather.yaml")).expect("decodes");
    let rules = configuration.rules();

    let mut on = CalendarDate::new(2001, 1, 1).expect("a date");
    let mut chain = Chain::default();
    let mut simulated_days = Vec::new();
    for index in 0..36_524 {
        let (summary, carry) = day(rules, configuration.seed(), index, on.month(), chain);
        chain = carry;
        simulated_days.push((on.month(), Some(summary.prcp_tenth_mm)));
        on = on.next().expect("a next day");
    }
    let (simulated, simulated_spells) = climate(simulated_days.into_iter());

    let mut failures = Vec::new();
    for month in 0..12 {
        let per_mille = |(wet, days): (u32, u32)| i64::from(wet) * 1_000 / i64::from(days);
        let (record, model) = (per_mille(recorded[month]), per_mille(simulated[month]));
        println!(
            "month {:>2}: wet {record:>3} ‰ in the record ({} of {} days), {model:>3} ‰ in 100 years",
            month + 1,
            recorded[month].0,
            recorded[month].1
        );
        if (record - model).abs() > 30 {
            failures.push(format!(
                "month {}: {model} ‰ against the record's {record} ‰",
                month + 1
            ));
        }
    }
    let mean = |spells: &[u32]| {
        i64::from(spells.iter().sum::<u32>()) * 1_000 / i64::try_from(spells.len()).expect("small")
    };
    let (record_spell, model_spell) = (mean(&recorded_spells), mean(&simulated_spells));
    println!(
        "mean wet spell: {record_spell} ‰ days in the record ({} spells), {model_spell} in 100 years ({} spells)",
        recorded_spells.len(),
        simulated_spells.len()
    );
    if (model_spell - record_spell).abs() * 100 > record_spell * 15 {
        failures.push(format!(
            "mean wet spell {model_spell} against the record's {record_spell} (thousandths of a day)"
        ));
    }
    assert!(failures.is_empty(), "criterion 5:\n{}", failures.join("\n"));
}
