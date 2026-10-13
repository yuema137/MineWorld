//! `reshape`: a station's `.dly` file → the weather pack's record rows for whole years, and a report of
//! what the record lacks (step-19 SD-TW-d-1, -2, -8b).
//!
//! A value of −9999, or one with a quality flag, becomes an empty cell. A weather type records presence
//! only, so a day without it is `0`. `fog = WT01 ∨ WT02 ∨ WT21`; `thunder = WT03`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use mineworld_weather::CalendarDate;
use mineworld_weather::record::{FILL_RUN_MAX, RecordRow, show};

use crate::dly::Dly;

/// The weather types the report counts, by year.
pub const WEATHER_TYPES: [&str; 8] = [
    "WT01", "WT02", "WT03", "WT08", "WT13", "WT14", "WT16", "WT21",
];

/// The elements a row is made of.
const ELEMENTS: [&str; 5] = ["TMAX", "TMIN", "PRCP", "AWND", "WDF2"];

/// The rows and the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reshaped {
    /// One row per day, 1 January of `from` to 31 December of `to`.
    pub rows: Vec<RecordRow>,
    /// The report, as printed.
    pub report: String,
}

/// Reshapes `dly`'s years `from ..= to`, for `station`.
///
/// # Errors
///
/// A station other than the file's, `from` after `to`, a year the file does not cover, or a value
/// outside the record format's range (named with its date and element).
pub fn reshape(dly: &Dly, station: &str, from: i32, to: i32) -> Result<Reshaped, String> {
    if dly.station != station {
        return Err(format!(
            "the file is station {}, not {station}",
            dly.station
        ));
    }
    if from > to {
        return Err(format!("--from {from} is after --to {to}"));
    }
    let (first, last) = dly.years().ok_or("the file has no lines")?;
    if from < first || to > last {
        return Err(format!(
            "the file covers {first} … {last}; {from} … {to} is not inside it"
        ));
    }
    let mut flagged: BTreeMap<&str, usize> = BTreeMap::new();
    let mut population: BTreeMap<(&str, i32), usize> = BTreeMap::new();
    let mut rows = Vec::new();
    let mut on = CalendarDate::new(from, 1, 1).ok_or("a year outside the calendar")?;
    while on.year() <= to {
        let (year, month, day) = (on.year(), on.month(), on.day());
        let usable = |element: &str| {
            dly.value(year, month, element, day)
                .and_then(|v| v.usable())
        };
        for element in ELEMENTS.iter().chain(WEATHER_TYPES.iter()) {
            if dly
                .value(year, month, element, day)
                .is_some_and(|v| v.flagged())
            {
                *flagged.entry(element).or_insert(0) += 1;
            }
        }
        for kind in WEATHER_TYPES {
            if usable(kind).is_some() {
                *population.entry((kind, year)).or_insert(0) += 1;
            }
        }
        let bounded = |element: &str, low: i32, high: i32| -> Result<Option<i32>, String> {
            match usable(element) {
                Some(value) if !(low..=high).contains(&value) => Err(format!(
                    "{} {element} is {value}, outside the record format's {low} … {high}",
                    show(on)
                )),
                other => Ok(other),
            }
        };
        let narrow = |value: Option<i32>| value.map(|v| i16::try_from(v).expect("bounded"));
        rows.push(RecordRow {
            date: on,
            tmax_dc: narrow(bounded("TMAX", -900, 600)?),
            tmin_dc: narrow(bounded("TMIN", -900, 600)?),
            prcp_tenth_mm: bounded("PRCP", 0, 65_535)?.map(|v| u16::try_from(v).expect("bounded")),
            awnd_dms: bounded("AWND", 0, 255)?.map(|v| u8::try_from(v).expect("bounded")),
            wdf2_deg: bounded("WDF2", 0, 360)?.map(|v| u16::try_from(v).expect("bounded")),
            fog: ["WT01", "WT02", "WT21"]
                .iter()
                .any(|kind| usable(kind).is_some()),
            thunder: usable("WT03").is_some(),
        });
        on = on.next().ok_or("a day outside the calendar")?;
    }
    let report = report(&rows, &flagged, &population, from, to);
    Ok(Reshaped { rows, report })
}

fn report(
    rows: &[RecordRow],
    flagged: &BTreeMap<&str, usize>,
    population: &BTreeMap<(&str, i32), usize>,
    from: i32,
    to: i32,
) -> String {
    let mut text = String::new();
    let _ = writeln!(text, "rows {} ({from}-01-01 … {to}-12-31)", rows.len());
    let _ = writeln!(
        text,
        "missing by year (TMAX TMIN PRCP AWND WDF2; days missing any of the first three)"
    );
    for year in from..=to {
        let of_year: Vec<&RecordRow> = rows.iter().filter(|row| row.date.year() == year).collect();
        let count = |missing: &dyn Fn(&RecordRow) -> bool| {
            of_year.iter().filter(|row| missing(row)).count()
        };
        let _ = writeln!(
            text,
            "  {year}  {:>3} {:>3} {:>3} {:>3} {:>3}  {:>3}",
            count(&|row| row.tmax_dc.is_none()),
            count(&|row| row.tmin_dc.is_none()),
            count(&|row| row.prcp_tenth_mm.is_none()),
            count(&|row| row.awnd_dms.is_none()),
            count(&|row| row.wdf2_deg.is_none()),
            count(&|row| !row.complete()),
        );
    }
    let mut runs = Vec::new();
    let mut index = 0;
    while index < rows.len() {
        if rows[index].complete() {
            index += 1;
            continue;
        }
        let end = (index..rows.len())
            .find(|at| rows[*at].complete())
            .unwrap_or(rows.len());
        runs.push((rows[index].date, rows[end - 1].date, end - index));
        index = end;
    }
    let long = runs.iter().filter(|run| run.2 > FILL_RUN_MAX).count();
    let _ = writeln!(
        text,
        "gap runs {} (longer than {FILL_RUN_MAX} days: {long})",
        runs.len()
    );
    for (first, last, length) in &runs {
        let _ = writeln!(text, "  {} … {}  {length} d", show(*first), show(*last));
    }
    let _ = writeln!(text, "weather types: days present by year");
    let _ = writeln!(text, "  year  {}", WEATHER_TYPES.join(" "));
    for year in from..=to {
        let cells: Vec<String> = WEATHER_TYPES
            .iter()
            .map(|kind| {
                format!(
                    "{:>4}",
                    population.get(&(*kind, year)).copied().unwrap_or(0)
                )
            })
            .collect();
        let _ = writeln!(text, "  {year}  {}", cells.join(" "));
    }
    let fog_days = rows.iter().filter(|row| row.fog).count();
    let thunder_days = rows.iter().filter(|row| row.thunder).count();
    let _ = writeln!(text, "fog days {fog_days}, thunder days {thunder_days}");
    let flagged: Vec<String> = flagged
        .iter()
        .map(|(element, count)| format!("{element} {count}"))
        .collect();
    let _ = writeln!(
        text,
        "quality-flagged values dropped: {}",
        if flagged.is_empty() {
            "none".to_owned()
        } else {
            flagged.join(", ")
        }
    );
    text
}
