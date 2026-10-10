//! The record format's own failure classes (step-19 §18.4 C2 (a), (b)): what decodes identically on
//! every platform, and every refusal with its line. The world-level claims are `tests/record.rs`'s.

use super::*;

/// One complete row of `date`, plain values.
fn row(date: CalendarDate, index: i64) -> RecordRow {
    RecordRow {
        date,
        tmax_dc: Some(i16::try_from(180 + index % 40).expect("small")),
        tmin_dc: Some(i16::try_from(100 + index % 30).expect("small")),
        prcp_tenth_mm: Some(if index % 5 == 0 { 12 } else { 0 }),
        awnd_dms: Some(25),
        wdf2_deg: Some(270),
        fog: index % 9 == 0,
        thunder: false,
    }
}

/// Every day of `from ..= to`, complete.
fn rows(from: i32, to: i32) -> Vec<RecordRow> {
    let mut date = CalendarDate::new(from, 1, 1).expect("exists");
    let end = CalendarDate::new(to, 12, 31).expect("exists");
    let mut rows = Vec::new();
    let mut index = 0;
    loop {
        rows.push(row(date, index));
        if date == end {
            return rows;
        }
        date = date.next().expect("a next day");
        index += 1;
    }
}

/// The file's text with line `line` (1-based) replaced.
fn with_line(text: &str, line: usize, replacement: &str) -> String {
    text.lines()
        .enumerate()
        .map(|(index, original)| {
            let chosen = if index + 1 == line {
                replacement
            } else {
                original
            };
            format!("{chosen}\n")
        })
        .collect()
}

fn refusal(text: &str) -> DecodeError {
    decode(text.as_bytes()).expect_err("refused")
}

/// (a) LF, CRLF, CRLF with a BOM, and no final newline decode to the same rows, and the writer's own
/// output decodes back to its rows.
#[test]
fn lf_crlf_bom_and_no_final_newline_decode_identically() {
    let rows = rows(2016, 2016);
    let lf = encode(&rows);
    assert!(!lf.contains('\r'), "the writer writes LF only");
    let crlf = lf.replace('\n', "\r\n");
    let bom = [BOM, crlf.as_bytes()].concat();
    let unterminated = lf.trim_end_matches('\n');
    let decoded = decode(lf.as_bytes()).expect("LF decodes");
    assert_eq!(decoded.rows(), rows.as_slice());
    assert_eq!((decoded.first_year(), decoded.years()), (2016, 1));
    for (form, bytes) in [
        ("CRLF", crlf.as_bytes()),
        ("CRLF + BOM", bom.as_slice()),
        ("no final newline", unterminated.as_bytes()),
    ] {
        assert_eq!(decode(bytes).expect(form), decoded, "{form}");
    }
}

/// (b) Each refusal names its line and what is wrong.
#[test]
fn every_refusal_names_its_line() {
    let text = encode(&rows(2015, 2015));
    let line_of = |date: &str| {
        text.lines()
            .position(|line| line.starts_with(date))
            .expect("present")
            + 1
    };
    let march_2 = line_of("2015-03-02");
    let march_3 = text.lines().nth(march_2).expect("the next line").to_owned();
    let cases: Vec<(String, usize, &str)> = vec![
        (
            with_line(&text, 1, "date,tmax,tmin,prcp,awnd,wdf2,fog,thunder"),
            1,
            "the header is",
        ),
        // Unsorted: 2015-03-02's line replaced by 2015-02-28's, before the line it follows.
        (
            with_line(
                &text,
                march_2,
                text.lines().nth(march_2 - 3).expect("a line"),
            ),
            march_2,
            "is before the line before",
        ),
        // A duplicate: 2015-03-03's line replaced by 2015-03-02's.
        (
            with_line(
                &text,
                march_2 + 1,
                text.lines().nth(march_2 - 1).expect("a line"),
            ),
            march_2 + 1,
            "repeats the line before",
        ),
        (
            with_line(&text, march_2, &march_3),
            march_2,
            "skips from 2015-03-01",
        ),
        (
            text.lines()
                .take(1)
                .chain(text.lines().skip(2))
                .map(|l| format!("{l}\n"))
                .collect(),
            2,
            "begins on 2015-01-02",
        ),
        (
            text.lines().take(365).map(|l| format!("{l}\n")).collect(),
            365,
            "ends on 2015-12-30",
        ),
        (
            with_line(&text, march_2, "2015-03-02,19.5,100,0,25,270,0,0"),
            march_2,
            "tmax_dc is 0.1 °C from -900 to 600 or empty, not '19.5'",
        ),
        (
            with_line(&text, march_2, "2015-03-02,195,100,0,300,270,0,0"),
            march_2,
            "awnd_dms is 0.1 m/s from 0 to 255",
        ),
        (
            with_line(&text, march_2, "2015-03-02,195,100,0,25,270,2,0"),
            march_2,
            "fog is 0 or 1, not '2'",
        ),
        (
            with_line(&text, march_2, "2015-03-02,195,100,0,25,270,0"),
            march_2,
            "8 cells",
        ),
        (
            with_line(&text, march_2, "2015-03-02,195,\t100,0,25,270,0,0"),
            march_2,
            "a tab",
        ),
        (
            with_line(&text, march_2, "2015-03-02,\"195\",100,0,25,270,0,0"),
            march_2,
            "a quoted cell",
        ),
        (
            with_line(&text, march_2, "2015-03-02,195,100,0,25,270,0,0\r "),
            march_2,
            "a carriage return inside the line",
        ),
        (
            with_line(&text, march_2, "2015-03-02,195,100,0,25,270,0,0 °"),
            march_2,
            "a non-ASCII byte",
        ),
        (
            with_line(&text, march_2, "2015-02-30,195,100,0,25,270,0,0"),
            march_2,
            "a day that exists",
        ),
    ];
    for (text, line, why) in cases {
        let refused = refusal(&text);
        assert_eq!(refused.line, line, "{why}: {refused}");
        assert!(refused.to_string().contains(why), "{why}: {refused}");
        assert!(refused.to_string().starts_with(&format!("line {line}: ")));
    }
    assert!(refusal("").to_string().contains("the header is"));
    assert!(refusal(&format!("{HEADER}\n")).message.contains("no rows"));
}

/// Gaps: a short run copies the last complete day; a long run is the rules' or refused naming both
/// dates; a run at the start is a long run; a missing wind alone keeps the day.
#[test]
fn gaps_are_filled_drawn_or_refused() {
    let mut rows = rows(2015, 2015);
    for index in [40, 41, 42] {
        rows[index].tmax_dc = None; // 2015-02-10 … 12: three days
    }
    for row in &mut rows[100..104] {
        row.prcp_tenth_mm = None; // 2015-04-11 … 14: four days
    }
    rows[0].tmin_dc = None; // 2015-01-01: a run at the start
    rows[200].awnd_dms = None;
    rows[200].wdf2_deg = None;
    let file = decode(encode(&rows).as_bytes()).expect("decodes");
    let packed = pack(&file, Fill::Rules).expect("packs");
    let days = packed.days();
    assert_eq!(days[0].origin(), DayOrigin::Rule);
    for index in [40, 41, 42] {
        assert_eq!(days[index].origin(), DayOrigin::Filled);
        assert_eq!(days[index].tmax_dc(), days[39].tmax_dc());
        assert_eq!(days[index].prcp_tenth_mm(), days[39].prcp_tenth_mm());
        assert_eq!(days[index].fog(), days[39].fog());
    }
    assert!((100..104).all(|index| days[index].origin() == DayOrigin::Rule));
    assert_eq!(days[200].origin(), DayOrigin::Record);
    assert_eq!(
        (days[200].awnd_dms(), days[200].wind_from_deg()),
        (None, None)
    );
    assert_eq!(
        (days[199].awnd_dms(), days[199].wind_from_deg()),
        (Some(25), Some(270))
    );

    let refused = pack(&file, Fill::None).expect_err("a run at the start is a long gap");
    assert_eq!(
        (refused.line, show(refused.first), show(refused.last)),
        (2, "2015-01-01".into(), "2015-01-01".into())
    );
    rows[0].tmin_dc = Some(100);
    let file = decode(encode(&rows).as_bytes()).expect("decodes");
    let refused = pack(&file, Fill::None).expect_err("four days is a long gap");
    assert_eq!(
        refused.to_string(),
        "line 102: 2015-04-11 … 2015-04-14 are missing, more than 3 days, and `fill` is `none`"
    );
}

/// The packed series round-trips through its hex form, and a damaged form is refused.
#[test]
fn the_packed_series_round_trips_and_refuses_damage() {
    let file = decode(encode(&rows(2016, 2016)).as_bytes()).expect("decodes");
    let packed = pack(&file, Fill::Rules).expect("packs");
    let text = String::from(packed.clone());
    assert_eq!(text.len(), 366 * 18);
    assert_eq!(PackedDays::try_from(text.clone()), Ok(packed));
    assert!(PackedDays::try_from(text[1..].to_owned()).is_err());
    assert!(PackedDays::try_from(text.to_uppercase()).is_err());
    let mut origin_three = text.clone();
    origin_three.replace_range(16..18, "0c");
    assert!(PackedDays::try_from(origin_three).is_err());
}

/// SD-TW-d-4's arithmetic on its own, including a `first_year` that is not the file's first.
#[test]
fn the_record_date_keeps_month_and_day_and_loops() {
    let file = decode(encode(&rows(2015, 2016)).as_bytes()).expect("decodes");
    let date = |y, m, d| CalendarDate::new(y, m, d).expect("exists");
    let series = |first| {
        RecordSeries::new(
            Station::try_from("TEST".to_owned()).expect("a station"),
            &file,
            first,
            Fill::Rules,
        )
        .expect("a series")
    };
    let from_2015 = series(2015);
    assert_eq!(
        from_2015.record_date(date(2026, 10, 8), 2026),
        date(2015, 10, 8)
    );
    assert_eq!(
        from_2015.record_date(date(2027, 2, 28), 2026),
        date(2016, 2, 28)
    );
    assert_eq!(
        from_2015.record_date(date(2028, 2, 29), 2026),
        date(2015, 2, 28)
    );
    assert_eq!(
        from_2015.record_date(date(2025, 6, 1), 2026),
        date(2016, 6, 1)
    );
    let from_2016 = series(2016);
    assert_eq!(
        from_2016.record_date(date(2026, 10, 8), 2026),
        date(2016, 10, 8)
    );
    assert_eq!(
        from_2016.record_date(date(2028, 2, 29), 2026),
        date(2016, 2, 29)
    );
    let (record, day) = from_2016.day(date(2028, 2, 29), 2026);
    assert_eq!(record, date(2016, 2, 29));
    assert_eq!(
        day.tmax_dc(),
        file.rows()[365 + 59].tmax_dc.expect("present")
    );
    assert!(
        RecordSeries::new(
            Station::try_from("T".to_owned()).expect("ok"),
            &file,
            2017,
            Fill::Rules
        )
        .expect_err("outside")
        .contains("first_year 2017 is outside the record's years 2015 … 2016")
    );
    assert!(Station::try_from("A STATION".to_owned()).is_err());
    assert!(Station::try_from("X".repeat(33)).is_err());
}
