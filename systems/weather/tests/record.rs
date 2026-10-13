//! `source: record` in a running world, through the real World Pack loader (step-19 §18.4 C2): the same
//! facts from LF, CRLF and BOM files (a); the leap-day mapping (c, criterion 2); short and long gaps (d);
//! a changed value is configuration drift and a CRLF checkout is not (e, criterion 3); the climate
//! state's budget and the record Process's size (f); a resume equals the uninterrupted world (g). The
//! decoder's refusals, each with its line, are `src/record/tests.rs`'s (b).
//!
//! The record is a hand-made two-year file, 2015–2016, written into a scratch World Pack (DEP-29): its
//! values follow a simple rule of the day's index so a test can name any day's, and it holds a two-day
//! gap (2015-03-10 … 11, no maximum), a five-day gap (2015-07-01 … 05, no precipitation), 29 February
//! 2016, and empty wind speed and direction cells on 2016-10-08 (quality-flagged values dropped).

use mineworld_calendar::DayBegan;
use mineworld_contracts::{Event, EventEnvelope, WorldTime};
use mineworld_kernel::{ProcessKind, World};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend,
};
use mineworld_test_support::Scratch;
use mineworld_weather::generate::weather_day;
use mineworld_weather::record::{encode, show};
use mineworld_weather::{
    CalendarDate, ClimateProcess, ClimateState, Origin, RecordProcess, RecordRow,
    WeatherConfigured, WeatherDay,
};
use mineworld_worldpack::{PackError, WorldPack};

const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;
const DATA: &str = "data/weather/test-2015-2016.csv";

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day).expect("a date")
}

/// The fixture's day `index` (0 = 2015-01-01), complete.
fn fixture_row(index: i64, on: CalendarDate) -> RecordRow {
    let small = |value: i64| i16::try_from(value).expect("small");
    let tmax = 140 + (index * 7) % 120;
    let prcp = match index % 6 {
        0 => 3 + (index % 40) * 5,
        1 => 1,
        _ => 0,
    };
    RecordRow {
        date: on,
        tmax_dc: Some(small(tmax)),
        tmin_dc: Some(small(tmax - 50 - index % 30)),
        prcp_tenth_mm: Some(u16::try_from(prcp).expect("small")),
        awnd_dms: Some(u8::try_from(10 + index % 40).expect("small")),
        wdf2_deg: Some(u16::try_from(10 * (1 + index % 36)).expect("small")),
        fog: index % 13 == 0,
        thunder: index % 12 == 0,
    }
}

/// The fixture's 731 rows, with its gaps and its dropped cell.
fn fixture() -> Vec<RecordRow> {
    let mut rows = Vec::new();
    let mut on = date(2015, 1, 1);
    let mut index = 0;
    while on.year() <= 2016 {
        let mut row = fixture_row(index, on);
        if on == date(2015, 3, 10) || on == date(2015, 3, 11) {
            row.tmax_dc = None;
        }
        if (date(2015, 7, 1)..=date(2015, 7, 5)).contains(&on) {
            row.prcp_tenth_mm = None;
        }
        if on == date(2016, 10, 8) {
            row.awnd_dms = None;
            row.wdf2_deg = None;
        }
        rows.push(row);
        on = on.next().expect("a next day");
        index += 1;
    }
    assert_eq!(rows.len(), 731);
    rows
}

fn row_of(rows: &[RecordRow], on: CalendarDate) -> RecordRow {
    *rows
        .iter()
        .find(|row| row.date == on)
        .expect("in the fixture")
}

/// The configuration: San Diego's January climate every month, the fixture as the record.
fn configuration(first_year: i32, fill: &str) -> String {
    let month = "    - { p_wet_after_dry: 147, p_wet_after_wet: 447, rain_tenth_mm: [8, 28, 54, 93, 178], \
                 tmax_dc: 191, tmin_dc: 102, t_noise_dc: 30, t_ar_permille: 600, \
                 wet_tmax_shift_dc: -20, fog_permille: 80, thunder_permille: 40, \
                 overcast_morning_permille: 150, wind_dms: 25, wind_from_deg: 300 }\n";
    format!(
        "source: record\nseed: 19\nrecord:\n  station: TEST0000001\n  data: {DATA}\n  first_year: \
         {first_year}\nfill: {fill}\nrules:\n  months:\n{}",
        month.repeat(12)
    )
}

/// A scratch World Pack — a square, Ada in it, San Diego's calendar from 2026-10-08 — replaying the
/// record `csv` (bytes as given) under `weather`.
fn pack(id: &str, weather: &str, csv: &[u8]) -> Scratch {
    let scratch = mineworld_test_support::scratch!(empty id);
    let root = scratch.path();
    for directory in ["people", "places", "configure", "data/weather"] {
        std::fs::create_dir_all(root.join(directory)).expect("writable");
    }
    let write =
        |file: &str, bytes: &[u8]| std::fs::write(root.join(file), bytes).expect("writable");
    write(
        "world.yaml",
        format!(
            "world:\n  id: {id}\n  name: A Recorded Square\nsystems:\n  - presence\n  - calendar\n  - \
             weather\nconfigure:\n  - calendar\n  - weather\nplaces:\n  - square\npopulation:\n  - ada\n"
        )
        .as_bytes(),
    );
    write("places/square.yaml", b"tags: [square]\n");
    write("people/ada.yaml", b"location:\n  place: square\n");
    write(
        "configure/calendar.yaml",
        b"epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 32.7157\nlongitude: -117.1611\n",
    );
    write("configure/weather.yaml", weather.as_bytes());
    write(DATA, csv);
    scratch
}

/// Genesis of the pack at `root`, through the loader.
fn genesis(root: &std::path::Path) -> (World, Vec<EventEnvelope>) {
    let assembled = WorldPack::read(root)
        .expect("reads")
        .assemble()
        .expect("assembles");
    let mut world = assembled.world;
    let facts = world
        .genesis(WorldTime::from_seconds(0), assembled.facts)
        .expect("genesis");
    (world, facts)
}

fn configured_payload(facts: &[EventEnvelope]) -> Vec<u8> {
    facts
        .iter()
        .find(|fact| *fact.event_type() == WeatherConfigured::EVENT_TYPE)
        .expect("weather is configured")
        .payload()
        .payload()
        .clone()
}

fn of<E: Event + for<'de> serde::Deserialize<'de>>(facts: &[EventEnvelope]) -> Vec<E> {
    facts
        .iter()
        .filter(|fact| *fact.event_type() == E::EVENT_TYPE)
        .map(|fact| {
            serde_json::from_slice(fact.payload().payload_for::<E>().expect("its type"))
                .expect("decodes")
        })
        .collect()
}

/// Every fact through world date `last` (inclusive of its midnight).
fn run_to(root: &std::path::Path, last: CalendarDate) -> Vec<EventEnvelope> {
    let (mut world, mut facts) = genesis(root);
    let end = (last.days() - date(2026, 10, 8).days()) * DAY;
    facts.extend(
        world
            .advance_to(WorldTime::from_seconds(end))
            .expect("advances")
            .into_events(),
    );
    facts
}

fn day_on(days: &[WeatherDay], on: CalendarDate) -> &WeatherDay {
    days.iter()
        .find(|day| day.date() == on)
        .expect("a weather day on that date")
}

/// (a) Criterion 6: one record in LF, CRLF, CRLF with a BOM, and without a final newline gives the same
/// `weather-configured`, byte for byte.
#[test]
fn lf_crlf_bom_and_no_final_newline_configure_identically() {
    let lf = encode(&fixture());
    let crlf = lf.replace('\n', "\r\n");
    let forms: [(&str, Vec<u8>); 4] = [
        ("lf", lf.clone().into_bytes()),
        ("crlf", crlf.clone().into_bytes()),
        (
            "bom",
            [b"\xEF\xBB\xBF".as_slice(), crlf.as_bytes()].concat(),
        ),
        ("open", lf.trim_end_matches('\n').as_bytes().to_vec()),
    ];
    let payloads: Vec<(&str, Vec<u8>)> = forms
        .iter()
        .map(|(name, bytes)| {
            let scratch = pack(
                &format!("form-{name}"),
                &configuration(2015, "rules"),
                bytes,
            );
            (*name, configured_payload(&genesis(scratch.path()).1))
        })
        .collect();
    println!(
        "weather-configured: {} bytes in each of {:?}",
        payloads[0].1.len(),
        payloads.iter().map(|(name, _)| *name).collect::<Vec<_>>()
    );
    for (name, payload) in &payloads[1..] {
        assert!(*payload == payloads[0].1, "{name} differs from lf");
    }
}

/// (c) Criterion 2, with the epoch 2026-10-08: record dates as SD-TW-d-4 says, the expected values
/// written before the run (step-19 §18.4 C2 (c)); and (d) the gaps.
#[test]
fn world_dates_replay_record_dates_and_gaps_are_filled_or_drawn() {
    let rows = fixture();
    let csv = encode(&rows);

    // first_year 2015: the world's 2026 replays 2015, 2027 replays 2016, 2028 replays 2015 again.
    let scratch = pack("from-2015", &configuration(2015, "rules"), csv.as_bytes());
    let facts = run_to(scratch.path(), date(2028, 7, 6));
    let days: Vec<WeatherDay> = of(&facts);
    for (world, record) in [
        (date(2026, 10, 8), date(2015, 10, 8)),
        (date(2027, 2, 28), date(2016, 2, 28)), // the record's 2016-02-29 is skipped: 2027 has none
        (date(2027, 3, 1), date(2016, 3, 1)),
        (date(2028, 2, 29), date(2015, 2, 28)), // 2015 is not a leap year
        (date(2028, 3, 1), date(2015, 3, 1)),
        (date(2027, 10, 8), date(2016, 10, 8)), // the wind cell is empty: the month's wind
    ] {
        let day = day_on(&days, world);
        assert_eq!(
            day.origin(),
            Origin::Record { date: record },
            "{}",
            show(world)
        );
        let row = row_of(&rows, record);
        let summary = day.summary();
        assert_eq!(
            (
                Some(summary.tmax_dc),
                Some(summary.tmin_dc),
                Some(summary.prcp_tenth_mm)
            ),
            (row.tmax_dc, row.tmin_dc, row.prcp_tenth_mm),
            "{}: the record's own values",
            show(world)
        );
        assert_eq!(summary.awnd_dms, row.awnd_dms.map_or(25, u16::from));
        assert_eq!((summary.fog, summary.thunder), (row.fog, row.thunder));
    }
    assert!(
        days.iter().all(|day| day.origin()
            != Origin::Record {
                date: date(2016, 2, 29)
            }),
        "a record 29 February is used only by a world 29 February"
    );

    // (d) The two-day gap is the day before's values, marked filled.
    let before = row_of(&rows, date(2015, 3, 9));
    for (world, record) in [
        (date(2028, 3, 10), date(2015, 3, 10)),
        (date(2028, 3, 11), date(2015, 3, 11)),
    ] {
        let day = day_on(&days, world);
        assert_eq!(day.origin(), Origin::Filled { date: record });
        let summary = day.summary();
        assert_eq!(
            (
                Some(summary.tmax_dc),
                Some(summary.tmin_dc),
                Some(summary.prcp_tenth_mm),
                summary.fog
            ),
            (
                before.tmax_dc,
                before.tmin_dc,
                before.prcp_tenth_mm,
                before.fog
            )
        );
    }
    // The five-day gap is five rule days, the first drawn from the last record day's carry.
    let calendar: Vec<(i64, DayBegan)> = facts
        .iter()
        .filter(|fact| *fact.event_type() == DayBegan::EVENT_TYPE)
        .map(|fact| {
            (
                fact.at().seconds(),
                serde_json::from_slice(fact.payload().payload_for::<DayBegan>().expect("ours"))
                    .expect("decodes"),
            )
        })
        .collect();
    let configured: WeatherConfigured =
        serde_json::from_slice(&configured_payload(&facts)).expect("decodes");
    let mut previous = day_on(&days, date(2028, 6, 30)).clone();
    assert_eq!(
        previous.origin(),
        Origin::Record {
            date: date(2015, 6, 30)
        }
    );
    for offset in 1..=5 {
        let world = date(2028, 7, offset);
        let day = day_on(&days, world);
        assert_eq!(day.origin(), Origin::Rule, "{}", show(world));
        let began = &calendar
            .iter()
            .find(|(at, _)| *at == day.day_start().seconds())
            .expect("its day-began")
            .1;
        let drawn = weather_day(
            configured.rules(),
            configured.seed(),
            began.day(),
            previous.chain(),
        );
        assert_eq!(
            *day,
            drawn,
            "{}: the rules, from the day before's carry",
            show(world)
        );
        previous = day.clone();
    }
    assert_eq!(
        day_on(&days, date(2028, 7, 6)).origin(),
        Origin::Record {
            date: date(2015, 7, 6)
        }
    );

    // first_year 2016: the world's 2026 replays 2016, and its 2028 does too (leap to leap).
    let scratch = pack("from-2016", &configuration(2016, "rules"), csv.as_bytes());
    let days: Vec<WeatherDay> = of(&run_to(scratch.path(), date(2028, 3, 1)));
    assert_eq!(
        day_on(&days, date(2026, 10, 8)).origin(),
        Origin::Record {
            date: date(2016, 10, 8)
        }
    );
    assert_eq!(
        day_on(&days, date(2028, 2, 29)).origin(),
        Origin::Record {
            date: date(2016, 2, 29)
        }
    );
}

/// TWd-R1: every record day of a year replays its row's fog, thunder and wind, and the year holds days
/// with fog, with thunder, and with the wind missing (the month's wind then), so a summary that drops
/// any of them fails here, in the default suite.
#[test]
fn record_days_replay_their_rows_fog_thunder_and_wind() {
    let rows = fixture();
    let scratch = pack(
        "flags",
        &configuration(2015, "rules"),
        encode(&rows).as_bytes(),
    );
    // World 2026-10-08 … 2027-10-08 replays record 2015-10-08 … 2016-10-08.
    let days: Vec<WeatherDay> = of(&run_to(scratch.path(), date(2027, 10, 8)));
    let (mut fog, mut thunder, mut windless) = (0, 0, 0);
    for day in &days {
        let Origin::Record { date: record } = day.origin() else {
            continue;
        };
        let row = row_of(&rows, record);
        let summary = day.summary();
        assert_eq!(summary.fog, row.fog, "{}: fog", show(record));
        assert_eq!(summary.thunder, row.thunder, "{}: thunder", show(record));
        // The month's wind is the configuration's: 25 (0.1 m/s) from 300 degrees, every month.
        assert_eq!(
            summary.awnd_dms,
            row.awnd_dms.map_or(25, u16::from),
            "{}: wind speed",
            show(record)
        );
        assert_eq!(
            summary.wind_from_deg,
            row.wdf2_deg.map_or(300, |degrees| degrees % 360),
            "{}: wind direction",
            show(record)
        );
        fog += usize::from(row.fog);
        thunder += usize::from(row.thunder);
        windless += usize::from(row.awnd_dms.is_none() && row.wdf2_deg.is_none());
    }
    println!("record days with fog {fog}, thunder {thunder}, no wind speed {windless}");
    assert!(
        fog > 0 && thunder > 0 && windless > 0,
        "the year exercises every flag"
    );
}

/// (b, at assembly) `fill: none` with the five-day gap, and a `first_year` outside the file, are refused
/// as the pack's own rejection, naming the attachment, the line and the dates.
#[test]
fn a_long_gap_under_fill_none_and_a_year_outside_are_refused() {
    let csv = encode(&fixture());
    for (id, weather, says) in [
        (
            "no-fill",
            configuration(2015, "none"),
            "line 183: 2015-07-01 … 2015-07-05 are missing",
        ),
        (
            "too-late",
            configuration(2017, "rules"),
            "first_year 2017 is outside the record's years 2015 … 2016",
        ),
    ] {
        let scratch = pack(id, &weather, csv.as_bytes());
        let refused = WorldPack::read(scratch.path())
            .expect("reads")
            .assemble()
            .map(|_| ())
            .expect_err("refused");
        let text = format!("{refused} {refused:?}");
        println!("REFUSAL {text}");
        assert!(text.contains("weather-record-invalid"), "{text}");
        assert!(text.contains(DATA), "names the attachment: {text}");
        assert!(text.contains(says), "{says}: {text}");
    }
}

/// (e) Criterion 3: a changed value is drift through the real check; the same file with CRLF is not.
#[test]
fn a_changed_value_is_drift_and_a_crlf_checkout_is_not() {
    let csv = encode(&fixture());
    let scratch = pack("drift", &configuration(2015, "rules"), csv.as_bytes());
    let (_, saved) = genesis(scratch.path());
    let file = scratch
        .path()
        .join("data")
        .join("weather")
        .join("test-2015-2016.csv");
    let check = || {
        WorldPack::read(scratch.path())
            .expect("reads")
            .check_configuration(&saved)
    };

    std::fs::write(&file, csv.replace("\n", "\r\n")).expect("writable");
    check().expect("CRLF is the same record");

    let first = csv.lines().nth(1).expect("2015-01-01");
    let changed = first.replacen("2015-01-01,140,", "2015-01-01,141,", 1);
    assert_ne!(first, changed, "the edit applies");
    std::fs::write(&file, csv.replacen(first, &changed, 1)).expect("writable");
    match check() {
        Err(PackError::ConfigurationDrift { system, .. }) => assert_eq!(system.as_str(), "weather"),
        other => panic!("expected weather's drift, got {other:?}"),
    }
}

fn process_state<P: ProcessKind>(world: &World) -> Vec<u8> {
    let read = world.read();
    read.processes()
        .find(|process| *process.process_type() == P::PROCESS_TYPE)
        .map(|process| process.state_for::<P>().expect("ours").to_vec())
        .expect("the process")
}

/// (f) The climate state stays within TW-b's 8 KB in a record world; the record Process holds the
/// series, and its size is printed for the ledger.
#[test]
fn the_climate_stays_small_and_the_record_rides_its_own_process() {
    let scratch = pack(
        "budget",
        &configuration(2015, "rules"),
        encode(&fixture()).as_bytes(),
    );
    let (mut world, _) = genesis(scratch.path());
    let record = process_state::<RecordProcess>(&world);
    let mut largest = 0;
    for day in 1..=40 {
        let _ = world
            .advance_to(WorldTime::from_seconds(day * DAY + 13 * HOUR))
            .expect("advances");
        largest = largest.max(process_state::<ClimateProcess>(&world).len());
    }
    println!(
        "record process state: {} bytes for 731 days; largest climate state: {largest} bytes",
        record.len()
    );
    assert!(largest <= 8 * 1_024, "{largest} bytes > 8 KB");
    assert_eq!(
        process_state::<RecordProcess>(&world),
        record,
        "the record Process is never rewritten"
    );
    let state: ClimateState =
        serde_json::from_slice(&process_state::<ClimateProcess>(&world)).expect("decodes");
    assert_eq!(state.epoch_year(), Some(2026));
    assert!(
        state.configured().record().is_none(),
        "the series is not in the climate state"
    );
}

/// (g) A record world stopped mid-year and resumed equals the uninterrupted world across a new year.
#[test]
fn a_resumed_record_world_continues_as_the_uninterrupted_one() {
    let scratch = pack(
        "resume",
        &configuration(2016, "rules"),
        encode(&fixture()).as_bytes(),
    );
    let pack = WorldPack::read(scratch.path()).expect("reads");
    let stop = 50 * DAY + 14 * HOUR + 1_800;
    let end = 120 * DAY;
    let create = |save: &Scratch| {
        let assembled = pack.assemble().expect("assembles");
        let backend = SqliteBackend::create(save, Durability::ProcessCrash).expect("creates");
        PersistentWorld::create(
            Box::new(backend),
            assembled.world,
            Creation {
                instance: 0x5eed_0000_0000_0000_0000_0000_0019_0d01,
                pack: "record-test".to_owned(),
                at: WorldTime::from_seconds(0),
                facts: assembled.facts,
            },
        )
        .expect("created")
        .0
    };
    let straight = mineworld_test_support::scratch!(empty "straight");
    let mut uninterrupted = create(&straight);
    let _ = uninterrupted
        .advance_to(WorldTime::from_seconds(stop))
        .expect("advances");
    let expected = uninterrupted
        .advance_to(WorldTime::from_seconds(end))
        .expect("advances")
        .into_events();

    let stopped = mineworld_test_support::scratch!(empty "stopped");
    let mut first = create(&stopped);
    let _ = first
        .advance_to(WorldTime::from_seconds(stop))
        .expect("advances");
    drop(first);
    let backend: Box<dyn PersistenceBackend> =
        Box::new(SqliteBackend::open(&stopped, Durability::ProcessCrash).expect("opens"));
    let (mut resumed, _) =
        PersistentWorld::resume(backend, pack.compose().expect("composes").world).expect("resumes");
    let after = resumed
        .advance_to(WorldTime::from_seconds(end))
        .expect("advances")
        .into_events();
    let days: Vec<WeatherDay> = of(&after);
    assert_eq!(days.len(), 70);
    assert_eq!(
        days.last().expect("a day").origin(),
        Origin::Record {
            date: date(2015, 2, 5)
        },
        "world 2027 replays 2015 (first_year 2016, two years, looped), from the saved epoch"
    );
    assert_eq!(
        serde_json::to_vec(&after).expect("encodes"),
        serde_json::to_vec(&expected).expect("encodes"),
        "the resumed world's facts are the uninterrupted world's, byte for byte"
    );
}
