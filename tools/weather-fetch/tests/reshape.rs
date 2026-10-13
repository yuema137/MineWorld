//! `reshape` on a hand-made `.dly` file (step-19 §18.4 C3 (a), (b) and its failure cases): byte-
//! reproducible (criterion 1), the CSV decodes with the pack's own decoder, quality-flagged and −9999
//! values become empty cells, absent weather types are 0, and malformed input is refused by line.
//!
//! The fixture is station `USW00099999`, 2016: TMAX, TMIN, PRCP, AWND and WDF2 for January to March;
//! WT01 lines for January and February, a WT03 line for February; March has no weather-type line. Every
//! other month of 2016 has no line at all.

use mineworld_weather::CalendarDate;
use mineworld_weather::record::decode;
use mineworld_weather_fetch::reshape_bytes;

const STATION: &str = "USW00099999";

/// One `.dly` line: 31 values, each `(value, qflag)`, by readme §III's columns.
fn line(station: &str, year: i32, month: u8, element: &str, values: &[(i32, char)]) -> String {
    let mut text = format!("{station:<11}{year:04}{month:02}{element:<4}");
    for day in 0..31 {
        let (value, qflag) = values.get(day).copied().unwrap_or((-9_999, ' '));
        let source = if value == -9_999 { ' ' } else { 'W' };
        text.push_str(&format!("{value:>5} {qflag}{source}"));
    }
    assert_eq!(text.len(), 269);
    text
}

/// `n` days of `value`, then −9999 for the rest of the month.
fn days(n: usize, value: impl Fn(usize) -> i32) -> Vec<(i32, char)> {
    (0..n).map(|day| (value(day), ' ')).collect()
}

fn fixture() -> String {
    let mut lines = Vec::new();
    for (month, length) in [(1u8, 31usize), (2, 29), (3, 31)] {
        let m = i32::from(month);
        let mut tmax = days(length, |d| 180 + m * 10 + i32::try_from(d).expect("small"));
        if month == 1 {
            tmax[4] = (-9_999, ' '); // 2016-01-05: missing
            tmax[9] = (205, 'X'); // 2016-01-10: failed a check
        }
        lines.push(line(STATION, 2016, month, "TMAX", &tmax));
        lines.push(line(
            STATION,
            2016,
            month,
            "TMIN",
            &days(length, |d| {
                90 + m * 10 + i32::try_from(d % 7).expect("small")
            }),
        ));
        lines.push(line(
            STATION,
            2016,
            month,
            "PRCP",
            &days(length, |d| if d % 5 == 0 { 25 } else { 0 }),
        ));
        lines.push(line(STATION, 2016, month, "AWND", &days(length, |_| 31)));
        lines.push(line(
            STATION,
            2016,
            month,
            "WDF2",
            &days(length, |d| if d % 2 == 0 { 270 } else { 360 }),
        ));
    }
    // WT lines: a value on the days the weather happened, −9999 on the others.
    let only = |on: &[usize]| -> Vec<(i32, char)> {
        (0..31)
            .map(|day| {
                if on.contains(&day) {
                    (1, ' ')
                } else {
                    (-9_999, ' ')
                }
            })
            .collect()
    };
    lines.push(line(STATION, 2016, 1, "WT01", &only(&[2, 3])));
    lines.push(line(STATION, 2016, 2, "WT01", &only(&[0])));
    lines.push(line(STATION, 2016, 2, "WT03", &only(&[14])));
    format!("{}\n", lines.join("\n"))
}

fn reshaped(dly: &str) -> mineworld_weather_fetch::Written {
    reshape_bytes(
        dly.as_bytes(),
        STATION,
        (2016, 2016),
        "2026-10-09",
        "test-2016",
        "weather-fetch reshape --input test.dly",
    )
    .expect("reshapes")
}

/// (a) Criterion 1: two runs give the same CSV and NOTICE byte for byte — the second a second later,
/// so a wall-clock time leaking into either would show (M-TWd-1) — and the CSV is the pack's format.
#[test]
fn reshape_is_byte_reproducible_and_writes_the_packs_format() {
    let dly = fixture();
    let first = reshaped(&dly);
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let second = reshaped(&dly);
    assert_eq!(first.csv, second.csv, "the CSV");
    assert_eq!(first.notice, second.notice, "the NOTICE");
    assert_eq!(first.report, second.report, "the report");
    let file = decode(first.csv.as_bytes()).expect("the pack decodes the CSV");
    assert_eq!(file.rows().len(), 366);
    assert!(!first.csv.contains('\r'), "LF only");
    println!("{}", first.report);
    for needle in [
        "https://www.ncei.noaa.gov/pub/data/ghcn/daily/all/USW00099999.dly",
        "retrieved 2026-10-09 (UTC)",
        "MODIFIED DATA",
        "not endorsed by NOAA",
        "doi:10.1175/JTECH-D-11-00103.1",
        "doi:10.7289/V5D21VHZ",
        "CC0-1.0",
        "command    weather-fetch reshape --input test.dly",
    ] {
        assert!(first.notice.contains(needle), "the NOTICE says {needle}");
    }
    let counts = format!(
        "output     test-2016.csv, {} bytes, 367 lines (LF)",
        first.csv.len()
    );
    assert!(first.notice.contains(&counts), "{counts}");
    assert!(
        first
            .notice
            .contains(&format!("input      USW00099999.dly, {} bytes", dly.len()))
    );
}

/// (b) −9999 and a quality flag become empty cells; a missing weather type is 0; WT01 is fog and WT03
/// thunder; a month with no line at all is all empty.
#[test]
fn missing_and_flagged_values_are_empty_and_absent_types_are_zero() {
    let written = reshaped(&fixture());
    let file = decode(written.csv.as_bytes()).expect("decodes");
    let row = |month: u8, day: u8| {
        let on = CalendarDate::new(2016, month, day).expect("a date");
        *file
            .rows()
            .iter()
            .find(|row| row.date == on)
            .expect("a row")
    };
    assert_eq!(row(1, 5).tmax_dc, None, "−9999");
    assert_eq!(row(1, 10).tmax_dc, None, "quality-flagged");
    assert_eq!(row(1, 11).tmax_dc, Some(200));
    assert_eq!(
        (row(1, 1).prcp_tenth_mm, row(1, 2).prcp_tenth_mm),
        (Some(25), Some(0))
    );
    assert_eq!(
        (row(1, 1).wdf2_deg, row(1, 2).wdf2_deg, row(1, 2).awnd_dms),
        (Some(270), Some(360), Some(31))
    );
    assert!(
        row(1, 3).fog && row(1, 4).fog && !row(1, 5).fog,
        "WT01 on 3 and 4 January"
    );
    assert!(row(2, 1).fog && row(2, 15).thunder && !row(2, 16).thunder);
    assert!(
        file.rows()
            .iter()
            .filter(|row| row.date.month() == 3)
            .all(|row| !row.fog && !row.thunder),
        "March: no WT line"
    );
    assert!(
        file.rows()
            .iter()
            .filter(|row| row.date.month() >= 4)
            .all(|row| !row.complete()),
        "no line: empty"
    );
    assert!(
        written.report.contains("TMAX 1"),
        "the report counts the flagged value: {}",
        written.report
    );
    assert!(
        written.report.contains("2016-04-01 … 2016-12-31  275 d"),
        "{}",
        written.report
    );
}

/// The failure cases: a malformed line by number, another station, `--from` after `--to`, years the file
/// lacks.
#[test]
fn malformed_input_and_wrong_arguments_are_refused() {
    let dly = fixture();
    let short = dly.replacen("USW00099999201601TMAX", "USW00099999201601TMA", 1);
    let refused = |text: &str, station: &str, years: (i32, i32)| {
        reshape_bytes(text.as_bytes(), station, years, "2026-10-09", "x", "c").expect_err("refused")
    };
    assert!(refused(&short, STATION, (2016, 2016)).starts_with("line 1: a .dly line is 269"));
    let mixed = format!("{dly}{}\n", line("USW00011111", 2016, 4, "TMAX", &[]));
    assert!(refused(&mixed, STATION, (2016, 2016)).starts_with("line 19: station USW00011111"));
    let bad_month = dly.replacen("USW00099999201603TMAX", "USW00099999201613TMAX", 1);
    assert!(refused(&bad_month, STATION, (2016, 2016)).contains("MONTH '13'"));
    assert!(refused(&dly, "USW00023188", (2016, 2016)).contains("the file is station USW00099999"));
    assert!(refused(&dly, STATION, (2016, 2015)).contains("--from 2016 is after --to 2015"));
    assert!(refused(&dly, STATION, (2015, 2016)).contains("the file covers 2016 … 2016"));
}
