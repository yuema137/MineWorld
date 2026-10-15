//! The rhythm acceptance of `motivated` (`pr-dl-b-motivated.md` §8, AC-DLb-3 … AC-DLb-6 and AC-DLb-9; C6).
//!
//! Ignored by default: it runs two 30-day social-cafe worlds (every seat on `rule`, then every seat on
//! `motivated`) and needs the release build. Run it explicitly:
//!
//! ```text
//! cargo test --release -p mineworld-cli --test rhythm -- --ignored --nocapture
//! ```
//!
//! ```text
//! S(h)   share of spoken lines (`spoke` facts) at hour h, per mille
//! A(h)   the ATUS socializing row (A-3, 2025) normalised to sum 1 000; read from the TSV, not the model
//! TV     ½ Σ_h |S(h) − A(h)|, per mille
//!
//! AC-DLb-3  Σ_{h ∈ 0..5} S_m(h) ≤ 50 ‰
//! AC-DLb-4  TV(S_m, A) ≤ 250 ‰ and TV(S_r, A) − TV(S_m, A) ≥ 100 ‰
//! AC-DLb-5  seat-consults at 01:00–04:00 (15-minute lattice) standing in a `residential` place ≥ 850 ‰
//!           for motivated, and not lower than rule's by more than 50 ‰
//! AC-DLb-6  consults in `work` agenda slots at the agenda place: motivated ≥ rule − 50 ‰
//! AC-DLb-9  wall(motivated) ≤ 2 × wall(rule), both on this machine and build
//! ```
//!
//! Every measure is read from the saves' stored facts and the authored routines; each verdict is printed as
//! PASS or FAIL before any assertion fires, so a failing threshold is visible with its numbers.

mod bodies;
mod headless;

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use headless::{PACK, Tables, fresh, mineworld, seats, stderr};
use mineworld_contracts::{EntityId, EventEnvelope};
use mineworld_conversation::Spoke;
use mineworld_persistence::format;
use mineworld_presence::PersonEnteredPlace;

const SEED: &str = "7";
const DAYS: u64 = 30;
const DAY: i64 = 86_400;
/// The consult cadence of a headless run: every seat is consulted every fifteen minutes (`people/alice.yaml`).
const LATTICE: i64 = 900;
const ALL_MOTIVATED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../worlds/social-cafe/decisions-motivated.yaml"
);
/// The ATUS table the oracle reads; the test does not depend on the model crate for it.
const ATUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../cognition/motivated/data/atus-a3-2025.tsv"
));

/// Runs the pack for `DAYS` days with every seat on `decisions` (`None`: the pack's default, `rule`), and
/// reads its save back. The scratch directory is removed once the tables are read.
fn run_all(decisions: Option<&str>, name: &str) -> (Tables, Duration) {
    let save = fresh(name);
    let days = DAYS.to_string();
    let save_text = save.to_str().expect("a printable path");
    let mut arguments = vec![
        "run",
        PACK,
        "--headless",
        "--seed",
        SEED,
        "--days",
        &days,
        "--save",
        save_text,
    ];
    if let Some(file) = decisions {
        arguments.extend(["--decisions", file]);
    }
    let started = Instant::now();
    let output = mineworld(&arguments);
    let wall = started.elapsed();
    assert!(output.status.success(), "{name}: {}", stderr(&output));
    (Tables::read(&save), wall)
}

/// What a save says about where people talk and where they stand.
struct History {
    /// `spoke` facts by hour of day.
    spoke: [u64; 24],
    /// Every arrival by person key: (instant in seconds, place key), in fact order.
    arrivals: BTreeMap<String, Vec<(i64, String)>>,
}

fn key_of(keys: &BTreeMap<EntityId, String>, entity: EntityId) -> String {
    keys.get(&entity).cloned().unwrap_or_default()
}

fn read_history(tables: &Tables, keys: &BTreeMap<EntityId, String>) -> History {
    let mut history = History {
        spoke: [0; 24],
        arrivals: BTreeMap::new(),
    };
    for (_, bytes) in &tables.facts {
        let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
        let record = fact.payload();
        if record.payload_for::<Spoke>().is_ok() {
            let hour =
                usize::try_from(fact.at().seconds().rem_euclid(DAY) / 3_600).expect("an hour");
            history.spoke[hour] += 1;
        } else if let Ok(payload) = record.payload_for::<PersonEnteredPlace>() {
            let entered: PersonEnteredPlace =
                serde_json::from_slice(payload).expect("presence's encoding");
            history
                .arrivals
                .entry(key_of(keys, entered.person().entity_id()))
                .or_default()
                .push((
                    fact.at().seconds(),
                    key_of(keys, entered.place().entity_id()),
                ));
        }
    }
    history
}

/// The seconds of a `HH:MM` clock time.
fn clock(text: &str) -> Option<i64> {
    let (hours, minutes) = text.split_once(':')?;
    Some(hours.parse::<i64>().ok()? * 3_600 + minutes.parse::<i64>().ok()? * 60)
}

/// One routine item: `- { from: "05:30", place: cafe, label: work }`.
fn routine_item(line: &str) -> Option<(i64, String, String)> {
    let body = line.trim().strip_prefix("- {")?.strip_suffix('}')?;
    let (mut from, mut place, mut label) = (None, None, None);
    for field in body.split(',') {
        let (name, value) = field.split_once(':')?;
        let value = value.trim().trim_matches('"');
        match name.trim() {
            "from" => from = Some(clock(value)?),
            "place" => place = Some(value.to_owned()),
            "label" => label = Some(value.to_owned()),
            _ => return None,
        }
    }
    Some((from?, place?, label?))
}

fn person_file(seat: &str) -> String {
    let path = Path::new(PACK).join("people").join(format!("{seat}.yaml"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// A seat's authored day, in file order.
fn routine(seat: &str) -> Vec<(i64, String, String)> {
    let items: Vec<_> = person_file(seat).lines().filter_map(routine_item).collect();
    assert!(!items.is_empty(), "{seat} has a routine");
    items
}

/// Where a seat stands at the start of the save, from its `location:` section.
fn start_place(seat: &str) -> String {
    let text = person_file(seat);
    let mut after_location = text
        .lines()
        .skip_while(|line| !line.starts_with("location:"));
    after_location
        .find_map(|line| line.trim().strip_prefix("place:"))
        .map(|place| place.trim().to_owned())
        .unwrap_or_else(|| panic!("{seat} declares a location"))
}

/// The places tagged `residential`: the home role of `rhythm`'s night measure (`Roles::home`).
fn home_places() -> Vec<String> {
    let mut homes = Vec::new();
    for entry in std::fs::read_dir(Path::new(PACK).join("places")).expect("the places directory") {
        let path = entry.expect("a directory entry").path();
        let text = std::fs::read_to_string(&path).expect("a place file");
        if text.lines().any(|line| line.trim() == "- residential") {
            homes.push(
                path.file_stem()
                    .expect("a stem")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    assert!(!homes.is_empty(), "some place is residential");
    homes
}

/// The place a seat stands in at `at`: its start place until its first arrival, then the latest arrival.
fn place_at<'a>(start: &'a str, arrivals: &'a [(i64, String)], at: i64) -> &'a str {
    arrivals
        .iter()
        .take_while(|(when, _)| *when <= at)
        .last()
        .map_or(start, |(_, place)| place.as_str())
}

/// The seat's routine item in force at `at` (wrapping to the previous day's last item before the first).
fn slot_at(items: &[(i64, String, String)], at: i64) -> &(i64, String, String) {
    let clock_of_day = at.rem_euclid(DAY);
    items
        .iter()
        .take_while(|(from, _, _)| *from <= clock_of_day)
        .last()
        .unwrap_or_else(|| items.last().expect("a routine has items"))
}

/// Integer per-mille share of `part` in `whole`.
fn permille(part: u64, whole: u64) -> u64 {
    (part * 1_000).checked_div(whole).unwrap_or(0)
}

/// The socializing row of the ATUS table, normalised to sum 1 000 per hour (`A(h)`).
fn atus_socializing() -> [u64; 24] {
    let row = ATUS
        .lines()
        .find(|line| line.starts_with("socializing-and-communicating\t"))
        .expect("the socializing row");
    let values: Vec<u64> = row
        .split('\t')
        .skip(1)
        .map(|value| value.parse().expect("a per-mille value"))
        .collect();
    assert_eq!(values.len(), 24, "one value per hour");
    let total: u64 = values.iter().sum();
    let mut normalised = [0; 24];
    for (slot, value) in normalised.iter_mut().zip(values) {
        *slot = permille(value, total);
    }
    normalised
}

/// `TV(S, A)` in per mille: half the sum over hours of the absolute differences.
fn total_variation(shares: &[u64; 24], atus: &[u64; 24]) -> u64 {
    shares
        .iter()
        .zip(atus)
        .map(|(share, reference)| share.abs_diff(*reference))
        .sum::<u64>()
        / 2
}

/// Seat-consults at each 15-minute instant of the run, as (seat key, instant, place key).
fn consults<'a>(
    history: &'a History,
    seat_starts: &'a BTreeMap<String, String>,
) -> impl Iterator<Item = (&'a str, i64, &'a str)> + 'a {
    let arrivals = &history.arrivals;
    seat_starts.iter().flat_map(move |(seat, start)| {
        let moves: &[(i64, String)] = arrivals.get(seat).map_or(&[], Vec::as_slice);
        (0..(DAYS as i64) * DAY)
            .step_by(LATTICE as usize)
            .map(move |at| (seat.as_str(), at, place_at(start, moves, at)))
    })
}

/// The four measures of one save.
struct Measures {
    night_speech: u64,
    total_variation: u64,
    spoke_by_hour: [u64; 24],
    night_home: u64,
    agenda_kept: u64,
}

fn measure(
    history: &History,
    seat_starts: &BTreeMap<String, String>,
    routines: &BTreeMap<String, Vec<(i64, String, String)>>,
    homes: &[String],
) -> Measures {
    let total: u64 = history.spoke.iter().sum();
    let mut shares = [0; 24];
    for (share, count) in shares.iter_mut().zip(history.spoke) {
        *share = permille(count, total);
    }
    let night_total: u64 = history.spoke[0..5].iter().sum();
    let night_speech = permille(night_total, total);

    let (mut home, mut home_den) = (0_u64, 0_u64);
    let (mut kept, mut kept_den) = (0_u64, 0_u64);
    for (seat, at, place) in consults(history, seat_starts) {
        let hour = at.rem_euclid(DAY) / 3_600;
        if (1..4).contains(&hour) {
            home_den += 1;
            home += u64::from(homes.iter().any(|home| home == place));
        }
        let (_, agenda_place, label) = slot_at(&routines[seat], at);
        if label == "work" {
            kept_den += 1;
            kept += u64::from(place == agenda_place);
        }
    }
    Measures {
        night_speech,
        total_variation: total_variation(&shares, &atus_socializing()),
        spoke_by_hour: history.spoke,
        night_home: permille(home, home_den),
        agenda_kept: permille(kept, kept_den),
    }
}

#[test]
#[ignore = "the rhythm acceptance: two 30-day runs of the release build (pr-dl-b-motivated.md C6)"]
fn motivated_keeps_the_atus_rhythm_and_the_agenda() {
    let keys = bodies::keys_of(Path::new(PACK));
    let seat_names = seats();
    let seat_starts: BTreeMap<String, String> = seat_names
        .iter()
        .map(|seat| (seat.clone(), start_place(seat)))
        .collect();
    let routines: BTreeMap<String, Vec<(i64, String, String)>> = seat_names
        .iter()
        .map(|seat| (seat.clone(), routine(seat)))
        .collect();
    let homes = home_places();

    let (rule_tables, rule_wall) = run_all(None, "rhythm-rule");
    let (motivated_tables, motivated_wall) = run_all(Some(ALL_MOTIVATED), "rhythm-motivated");
    let rule_history = read_history(&rule_tables, &keys);
    let motivated_history = read_history(&motivated_tables, &keys);
    let rule = measure(&rule_history, &seat_starts, &routines, &homes);
    let motivated = measure(&motivated_history, &seat_starts, &routines, &homes);
    let atus = atus_socializing();

    println!("hour  S_m  S_r   A");
    for (hour, reference) in atus.iter().enumerate() {
        println!(
            "{hour:>4}  {:>3}  {:>3}  {:>3}",
            permille(
                motivated.spoke_by_hour[hour],
                motivated.spoke_by_hour.iter().sum()
            ),
            permille(rule.spoke_by_hour[hour], rule.spoke_by_hour.iter().sum()),
            reference
        );
    }
    println!(
        "wall: rule {:.1} s, motivated {:.1} s",
        rule_wall.as_secs_f64(),
        motivated_wall.as_secs_f64()
    );

    let mut verdicts = Vec::new();
    let mut verdict = |id: &str, pass: bool, detail: String| {
        println!("{id} {} {detail}", if pass { "PASS" } else { "FAIL" });
        verdicts.push((id.to_owned(), pass));
    };
    verdict(
        "AC-DLb-3",
        motivated.night_speech <= 50,
        format!(
            "night speech motivated {} per mille, rule {} (limit 50)",
            motivated.night_speech, rule.night_speech
        ),
    );
    verdict(
        "AC-DLb-4",
        motivated.total_variation <= 250
            && rule
                .total_variation
                .saturating_sub(motivated.total_variation)
                >= 100,
        format!(
            "TV motivated {} rule {} per mille (limits: <= 250, gap >= 100)",
            motivated.total_variation, rule.total_variation
        ),
    );
    verdict(
        "AC-DLb-5",
        motivated.night_home >= 850 && motivated.night_home + 50 >= rule.night_home,
        format!(
            "night at home motivated {} rule {} per mille (limits: >= 850, >= rule - 50)",
            motivated.night_home, rule.night_home
        ),
    );
    verdict(
        "AC-DLb-6",
        motivated.agenda_kept + 50 >= rule.agenda_kept,
        format!(
            "agenda kept motivated {} rule {} per mille (limit: >= rule - 50)",
            motivated.agenda_kept, rule.agenda_kept
        ),
    );
    verdict(
        "AC-DLb-9",
        motivated_wall <= rule_wall * 2,
        format!(
            "wall motivated {:.1} s rule {:.1} s (limit: <= 2x)",
            motivated_wall.as_secs_f64(),
            rule_wall.as_secs_f64()
        ),
    );

    let failed: Vec<&str> = verdicts
        .iter()
        .filter(|(_, pass)| !pass)
        .map(|(id, _)| id.as_str())
        .collect();
    assert!(failed.is_empty(), "failed acceptance criteria: {failed:?}");
}
