//! Market Town's shop hours through the real binary (`pr-nightlife.md` N1-C3; N1-I7, N1-I8; `ARC-364`).
//!
//! ```text
//! night    `run --headless worlds/market-town --seed 7 --days 2`: the three shops open with the ruled
//!          hours (café 05:30–22:00, store 06:00–23:00, newsstand 06:00–18:00); no `sold` at a shop
//!          outside its hours; a sale before the evening close and a breakfast after the morning
//!          opening; N1-I7 — at most one refused `buy` `shop-closed` per shop per closed interval, and
//!          nobody enters a closed shop more than once per closed interval; at least one resident sleeps, wakes and
//!          starts a shift present
//! resume   a Market Town save created from the pack without hours (what a binary before N1 saved)
//!          resumes under the pack with hours: hours are content, not configuration (`ARC-61` item 7),
//!          so it is not drift and keeps its genesis terms — its shops open with no hours
//! ```
//!
//! Every oracle is read from the stored facts; each shop's hours are parsed here from its own
//! `shop-opened` payload, independently of economy's code.

mod headless;
mod market;
mod social;

use std::collections::BTreeMap;
use std::path::Path;

use mineworld_contracts::EventEnvelope;
use serde_json::Value;

use headless::{Tables, fresh, mineworld};
use market::MarketCopy;

const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;

fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).unwrap_or(Value::Null)
}

fn id_of(value: &Value) -> u64 {
    let entity = value.get("entity").unwrap_or(value);
    entity
        .as_u64()
        .or_else(|| entity.as_str()?.parse().ok())
        .expect("an entity id")
}

fn of<'a>(facts: &'a [EventEnvelope], kind: &'a str) -> impl Iterator<Item = &'a EventEnvelope> {
    facts
        .iter()
        .filter(move |fact| fact.event_type().as_str() == kind)
}

/// "HH:MM" as seconds since midnight.
fn seconds(text: &str) -> i64 {
    let (hours, minutes) = text.split_once(':').expect("HH:MM");
    hours.parse::<i64>().expect("hours") * HOUR + minutes.parse::<i64>().expect("minutes") * 60
}

/// A shop's daily intervals `[from, until)` in seconds, as its `shop-opened` states them.
type Hours = Vec<(i64, i64)>;

fn open(hours: &Hours, at: i64) -> bool {
    let now = at.rem_euclid(DAY);
    hours.iter().any(|&(from, until)| {
        if from < until {
            (from..until).contains(&now)
        } else {
            now >= from || now < until
        }
    })
}

/// Every shop's place and hours (`None`: no hours), from the genesis `shop-opened` facts.
fn shops(facts: &[EventEnvelope]) -> BTreeMap<u64, Option<Hours>> {
    of(facts, "shop-opened")
        .map(|fact| {
            let opened = payload(fact);
            let hours = opened.get("hours").map(|hours| {
                hours
                    .as_array()
                    .expect("a list")
                    .iter()
                    .map(|opening| {
                        (
                            seconds(opening["from"].as_str().expect("from")),
                            seconds(opening["until"].as_str().expect("until")),
                        )
                    })
                    .collect()
            });
            (id_of(&opened["place"]), hours)
        })
        .collect()
}

/// `run --headless` of `pack` with seed 7 to day `days`, into `save`; its stdout.
fn run(pack: &Path, days: u64, save: &Path) -> String {
    run_with(pack, days, save, None)
}

/// [`run`] with an optional decision list (`--decisions`).
fn run_with(pack: &Path, days: u64, save: &Path, decisions: Option<&str>) -> String {
    let days = days.to_string();
    let mut arguments = vec![
        "run",
        pack.to_str().expect("a printable path"),
        "--headless",
        "--seed",
        "7",
        "--days",
        &days,
        "--save",
        save.to_str().expect("a printable path"),
    ];
    if let Some(decisions) = decisions {
        arguments.extend(["--decisions", decisions]);
    }
    let output = mineworld(&arguments);
    assert!(output.status.success(), "{}", headless::stderr(&output));
    headless::stdout(&output)
}

/// The instant the shop last closed before `at`, a closed instant (minute steps; at most a day back).
fn closed_since(hours: &Hours, at: i64) -> i64 {
    let mut since = at;
    while since > at - DAY && !open(hours, since - 60) {
        since -= 60;
    }
    since
}

/// No `sold` at a shop outside its hours, and N1-I7: at most one refused `buy` `shop-closed` per shop
/// per closed interval, nobody entering a closed shop more than once per closed interval. Returns the shop sales as
/// (instant, place).
fn keeps_hours(
    what: &str,
    facts: &[EventEnvelope],
    printed: &str,
    shops: &BTreeMap<u64, Option<Hours>>,
    nights: u64,
) -> Vec<(i64, u64)> {
    // A shop without hours is always open.
    let sells_at = |place: u64, at: i64| shops[&place].as_ref().is_none_or(|hours| open(hours, at));
    let sales: Vec<(i64, u64)> = of(facts, "sold")
        .map(|fact| (fact.at().seconds(), id_of(&payload(fact)["place"])))
        .filter(|(_, place)| shops.contains_key(place))
        .collect();
    for &(at, place) in &sales {
        assert!(
            sells_at(place, at),
            "{what}: sold at shop {place} at {at} s, outside its hours"
        );
    }
    // The run report aggregates refusals by code (not by shop), so the bound is the aggregate of
    // N1-I7: three shops, each closed at most once a night and once more at the run's edges.
    // Every refused-buy line must end in its count, so a changed report format fails loudly here.
    let rejected: Vec<(&str, u64)> = headless::lines(printed, "requests   buy rejected")
        .into_iter()
        .map(|line| {
            let count = line.split_whitespace().last().expect("a count");
            (line, count.parse::<u64>().expect("the count ends the line"))
        })
        .collect();
    let refused: u64 = rejected
        .iter()
        .filter(|(line, _)| line.contains("\"shop-closed\""))
        .map(|(_, count)| count)
        .sum();
    let mut closed_entries: BTreeMap<(u64, u64, i64), u64> = BTreeMap::new();
    for fact in of(facts, "person-entered-place") {
        let entered = payload(fact);
        let place = id_of(&entered["place"]);
        let at = fact.at().seconds();
        if let Some(Some(hours)) = shops.get(&place)
            && !open(hours, at)
        {
            // Keyed by the closed interval itself — the instant the shop last closed — so a lunch closure
            // and the night after it are two intervals (N1-I7: once per closed interval).
            let closed_at = closed_since(hours, at);
            *closed_entries
                .entry((id_of(&entered["person"]), place, closed_at))
                .or_default() += 1;
        }
    }
    println!(
        "{what}: {} shop sales; shop-closed refusals {refused}; entries into closed shops \
         (person, place, closed since) {closed_entries:?}",
        sales.len()
    );
    assert!(
        refused <= 3 * (nights + 1),
        "{what}: at most one per shop per closed interval: {refused}"
    );
    assert!(
        closed_entries.values().all(|count| *count <= 1),
        "{what}: nobody walks into a closed shop more than once per closed interval: {closed_entries:?}"
    );
    sales
}

/// N1-C3 scenario: a night and a morning in Market Town under the ruled hours, every seat on `rule`.
#[test]
fn market_town_shops_keep_their_hours_through_a_night_and_residents_still_eat() {
    let save = fresh("shop-hours-night");
    let printed = run(Path::new(MARKET), 2, &save);
    let facts = social::facts_of(&Tables::read(&save));
    let shops = shops(&facts);

    // The ruled hours, as content states them (Q-L1).
    let mut ruled: Vec<Hours> = shops
        .values()
        .map(|hours| hours.clone().expect("hours"))
        .collect();
    ruled.sort();
    let expected: Vec<Hours> = vec![
        vec![(seconds("05:30"), seconds("22:00"))],
        vec![(seconds("06:00"), seconds("18:00"))],
        vec![(seconds("06:00"), seconds("23:00"))],
    ];
    assert_eq!(ruled, expected);

    // A sale in the evening before a close; a breakfast after the morning opening.
    let sales = keeps_hours("rule", &facts, &printed, &shops, 2);
    let evening = sales
        .iter()
        .filter(|(at, _)| (16 * HOUR..22 * HOUR).contains(&at.rem_euclid(DAY)))
        .count();
    let breakfast = sales
        .iter()
        .filter(|(at, _)| {
            *at >= DAY && (5 * HOUR + 30 * 60..10 * HOUR).contains(&at.rem_euclid(DAY))
        })
        .count();
    println!("evening sales {evening}, day-1 breakfast sales {breakfast}");
    assert!(
        evening > 0 && breakfast > 0,
        "evening {evening}, breakfast {breakfast}"
    );

    // The town begins at 07:00 after a night's sleep (ARC-371; people/*.yaml `needs:`): nobody is so
    // tired that they go back to bed before the evening (with the midnight levels, 700 tiredness at 07:00, the
    // first resident fell asleep at 16:30 of day 0; with the morning levels, at 22:15).
    let genesis = facts.first().expect("a genesis").at().seconds();
    let morning_naps: Vec<i64> = of(&facts, "fell-asleep")
        .map(|fact| fact.at().seconds())
        .filter(|at| (genesis..genesis - genesis.rem_euclid(DAY) + 20 * HOUR).contains(at))
        .collect();
    assert!(
        morning_naps.is_empty(),
        "asleep before 20:00 of the first day: {morning_naps:?}"
    );

    // Home → sleep → wake → work: someone wakes on day 1 and starts a shift present after it.
    let woke: BTreeMap<u64, i64> = of(&facts, "woke")
        .filter(|fact| fact.at().seconds() >= DAY)
        .map(|fact| (id_of(&payload(fact)["person"]), fact.at().seconds()))
        .collect();
    let commuted = of(&facts, "shift-started").any(|fact| {
        let started = payload(fact);
        started["present"].as_bool() == Some(true)
            && woke
                .get(&id_of(&started["employee"]))
                .is_some_and(|at| *at < fact.at().seconds())
    });
    assert!(
        commuted,
        "a resident woke on day 1 and then started a shift present"
    );
}

/// N1-I7 audit of the `motivated` controller (`pr-nightlife.md` §2.2): it reads no `shop-stock` and
/// walks to places by role, so the same night with every seat `motivated` must keep the same bounds.
#[test]
fn motivated_residents_do_not_haunt_closed_shops() {
    let all_motivated = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../cognition/motivated/examples/social-cafe-all-motivated.yaml"
    );
    let save = fresh("shop-hours-motivated");
    let printed = run_with(Path::new(MARKET), 2, &save, Some(all_motivated));
    let facts = social::facts_of(&Tables::read(&save));
    let _ = keeps_hours("motivated", &facts, &printed, &shops(&facts), 2);
}
/// N1-I7 where residents actually meet a closure: a copy of Market Town whose café closes for lunch
/// (12:00–14:00), when Carol, Ivan, Erin, Felix and the wanderer's routines take them there. Under `rule`,
/// people do walk into the closed café in that window — so the closure is met, not merely absent — and
/// still: no sale at the café inside it, at most one refused `buy` per shop per closed interval, nobody
/// in more than once a day, and the café sells again from 14:00. (Under `motivated` no seat buys anything
/// in Market Town at all, base content included, so this encounter can only be driven by `rule`.)
#[test]
fn residents_meet_a_lunchtime_closure_and_neither_buy_there_nor_haunt_it() {
    let pack = MarketCopy::with_hours(
        "shop-hours-lunch-pack",
        "cafe-company",
        "[{ from: \"05:30\", until: \"12:00\" }, { from: \"14:00\", until: \"22:00\" }]",
    );
    let save = fresh("shop-hours-lunch");
    let printed = run(Path::new(pack.path()), 2, &save);
    let facts = social::facts_of(&Tables::read(&save));
    let shops = shops(&facts);
    let (cafe, _) = shops
        .iter()
        .find(|(_, hours)| hours.as_ref().is_some_and(|hours| hours.len() == 2))
        .expect("the café, with its split hours");
    let lunch = |at: i64| (12 * HOUR..14 * HOUR).contains(&at.rem_euclid(DAY));
    let met = of(&facts, "person-entered-place")
        .filter(|fact| id_of(&payload(fact)["place"]) == *cafe && lunch(fact.at().seconds()))
        .count();
    let sales = keeps_hours("rule, lunch closure", &facts, &printed, &shops, 2);
    let afternoon = sales
        .iter()
        .filter(|(at, place)| place == cafe && (14 * HOUR..22 * HOUR).contains(&at.rem_euclid(DAY)))
        .count();
    println!("entries into the closed café at lunch {met}; café sales 14:00–22:00 {afternoon}");
    assert!(
        met > 0,
        "residents walk into the café while it is closed for lunch"
    );
    assert!(afternoon > 0, "the café sells again after lunch");
}

/// N1-I8: a Market Town save from before hours resumes under the pack with hours — the discriminating
/// part: were hours a configuration fact, `run` would refuse the resume as drift (`ARC-61` item 7) — and
/// its genesis still opens every shop without hours. That the resumed shops then *sell* after the pack's
/// new closing time is asserted at the owner, on a real save, by economy's
/// `hours.rs::a_save_from_before_hours_resumes_with_its_shops_always_open`.
#[test]
fn a_market_town_save_from_before_hours_resumes_without_drift() {
    let before = MarketCopy::always_open("shop-hours-pack-before");
    let save = fresh("shop-hours-before-save");
    let _ = run(Path::new(before.path()), 1, &save);
    let resumed = run(Path::new(MARKET), 2, &save);
    assert!(resumed.contains("resumed"), "{resumed}");
    let facts = social::facts_of(&Tables::read(&save));
    let shops = shops(&facts);
    assert_eq!(shops.len(), 3);
    assert!(
        shops.values().all(Option::is_none),
        "the save keeps its genesis terms: always open"
    );
}
