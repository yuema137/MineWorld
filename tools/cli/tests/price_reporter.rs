//! S31-d C5 — **the clerk publishes from its book, and keepers follow current suggestions, in two towns**
//! (`pr-s31d-reporter-policies.md` §8: AC-d-4, AC-d-8, AC-d-9, AC-d-10; step-31 §16.2's checkpoint "fixture
//! world 10 d: notices published, a policy follows").
//!
//! ```text
//! AC-d-4   Social Café and Lakeside, every seat on price-reporter = every seat on rule, byte for byte
//! AC-d-8   W2 (Market Town + notices + price-survey, Carol the clerk), 10 days: ≥ 10 notices by Carol and no
//!          refused publish; every line = the test's own §5.3 over Carol's book folded from her price-noted
//!          facts; ≥ 1 keeper price-set toward the current suggestion, none away, all bounded; no move
//!          without a current suggestion; twice and across a kill at day 5, byte for byte
//! AC-d-9   W3 (euros, market.daily, Hana the clerk, Alice on motivated), 10 days: ≥ 5 notices by Hana; the
//!          same oracle; ≥ 1 café price-set toward, none away, bounded
//! AC-d-10  W4: (a) without notices (also on W3), (b) without price-survey, (c) without a pricing block
//! ```
//!
//! Every fact is read from the save by slug (`ARC-35` item 6): this file names no market or S31 crate. The
//! floors (10, 5, 1) were fixed in the design before any run (`ARC-23`); a miss is a recorded FAIL, never a
//! tuned run. These runs are long; per the speed-first rule they are run at the milestone.

mod headless;
// Only its pack roots and path are used here.
#[allow(dead_code)]
mod lakeside;
mod market;
mod price_world;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use headless::{Tables, fresh, mineworld, stderr, stdout};
use market::Town;
use mineworld_contracts::{ActionResult, EntityId, EventEnvelope};
use mineworld_persistence::{JournalEntry, WorldInput, format};
use price_world::{
    CAROL, Clerk, HANA, Posted, TOWN, Variant, assert_audiences, entity, facts, notices, payload,
    references, world,
};

const SEED: &str = "7";
const DAYS: u64 = 10;
const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

/// §5.3's knobs as the decisions files below leave them: the model's defaults.
const MIN_BASIS: usize = 3;
const SCARCE_BELOW: u64 = 3;
const SCARCE_PERMILLE: u128 = 100;
const PLENTY_ABOVE: u64 = 20;
const PLENTY_PERMILLE: u128 = 50;
/// price-survey's defaults for a section that names neither: the ring and the window.
const DEPTH: usize = 32;
const WINDOW: i64 = 7 * 86_400;
/// The pricing block's default bounds.
const FLOOR_PERMILLE: u64 = 500;
const CEILING_PERMILLE: u64 = 2_000;

const TOWARD: &str = "pricing: { sell: { policy: toward-suggestion } }";

/// `run <pack> --headless` with these flags, which must succeed; its stdout.
fn run(pack: &Path, days: u64, save: &Path, extra: &[&str]) -> String {
    let days = days.to_string();
    let mut arguments = vec![
        "run",
        pack.to_str().expect("UTF-8"),
        "--headless",
        "--seed",
        SEED,
        "--days",
        &days,
        "--save",
        save.to_str().expect("UTF-8"),
    ];
    arguments.extend_from_slice(extra);
    let output = mineworld(&arguments);
    assert!(
        output.status.success(),
        "run failed: {}\n{}",
        stderr(&output),
        stdout(&output)
    );
    stdout(&output)
}

/// A decisions file in `directory`; its path.
fn decisions(directory: &Path, name: &str, text: &str) -> PathBuf {
    std::fs::create_dir_all(directory).expect("a directory");
    let path = directory.join(format!("{name}.decisions.yaml"));
    std::fs::write(&path, text).expect("writes");
    path
}

/// W2's decisions: Carol the clerk, every `rule` seat priced toward suggestions.
fn w2_decisions(directory: &Path, pricing: bool) -> PathBuf {
    let rule = if pricing {
        format!("  rule: {{ parameters: {{ {TOWARD} }} }}\n")
    } else {
        String::new()
    };
    decisions(
        directory,
        if pricing { "w2" } else { "w2-hold" },
        &format!(
            "models:\n  price-reporter: {{ parameters: {{ topic: {}, first_minute: 480, last_minute: 1200, \
             every_minutes: 240, valid_minutes: 360 }} }}\n{rule}assign:\n  - {{ seat: {}, model: price-reporter }}\n",
            CAROL.topic, CAROL.key
        ),
    )
}

/// W3's decisions: Hana the evening clerk; Alice on `motivated` and Felix on `rule`, both priced.
fn w3_decisions(directory: &Path) -> PathBuf {
    decisions(
        directory,
        "w3",
        &format!(
            "models:\n  price-reporter: {{ parameters: {{ topic: {}, first_minute: 960, last_minute: 1200, \
             every_minutes: 120, valid_minutes: 840 }} }}\n  rule: {{ parameters: {{ {TOWARD} }} }}\n  \
             motivated: {{ parameters: {{ {TOWARD} }} }}\nassign:\n  - {{ seat: {}, model: price-reporter }}\n  \
             - {{ seat: alice, model: motivated }}\n",
            HANA.topic, HANA.key
        ),
    )
}

fn arg(path: &Path) -> &str {
    path.to_str().expect("UTF-8")
}

/// The refused requests of `actor` for `action_type`, from the journal.
fn refused(save: &Path, actor: EntityId, action_type: &str) -> usize {
    Tables::read(save)
        .journal
        .iter()
        .filter(|(_, bytes)| {
            let entry: JournalEntry = format::decode(bytes, "journal entry").expect("decodes");
            matches!(
                (&entry.input, &entry.outcome),
                (WorldInput::Dispatch { intent, .. }, mineworld_persistence::Outcome::Dispatched(ActionResult::Rejected(_)))
                    if intent.actor() == actor && intent.action_type().as_str() == action_type
            )
        })
        .count()
}

fn count(facts: &[EventEnvelope], kind: &str) -> usize {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == kind)
        .count()
}

// ---- the oracle: §5.3 written out, over the book folded from the facts ---------------------------

/// One note of the clerk's book, as folded from a `price-noted` fact.
#[derive(Debug, Clone, Copy)]
struct Note {
    at: i64,
    place: EntityId,
    seller: EntityId,
    price: u64,
    sale: bool,
    in_stock: Option<u64>,
}

/// The clerk's book as it stands just before fact `before`: every `price-noted` of the clerk folded in
/// order, each kind's ring kept to [`DEPTH`], then the notes older than [`WINDOW`] at `now` dropped.
fn book(
    facts: &[EventEnvelope],
    clerk: EntityId,
    before: usize,
    now: i64,
) -> BTreeMap<EntityId, Vec<Note>> {
    let mut kinds: BTreeMap<EntityId, Vec<Note>> = BTreeMap::new();
    for fact in &facts[..before] {
        if fact.event_type().as_str() != "price-noted" {
            continue;
        }
        let body = payload(fact);
        if entity(&body["surveyor"]) != clerk {
            continue;
        }
        let at = fact.at().seconds();
        for noted in body["notes"].as_array().expect("notes") {
            let ring = kinds.entry(entity(&noted["item"])).or_default();
            ring.push(Note {
                at,
                place: entity(&body["place"]),
                seller: entity(&noted["seller"]),
                price: noted["price"].as_u64().expect("a price"),
                sale: body["source"] == "sale",
                in_stock: noted["in_stock"].as_u64(),
            });
            let excess = ring.len().saturating_sub(DEPTH);
            ring.drain(..excess);
        }
        for ring in kinds.values_mut() {
            ring.retain(|note| note.at + WINDOW > at);
        }
    }
    for ring in kinds.values_mut() {
        ring.retain(|note| note.at + WINDOW > now);
    }
    kinds.retain(|_, ring| !ring.is_empty());
    kinds
}

fn median(mut values: Vec<u64>) -> Option<u64> {
    values.sort_unstable();
    let middle = values.len() / 2;
    match values.len() {
        0 => None,
        n if n % 2 == 1 => Some(values[middle]),
        _ => Some(((u128::from(values[middle - 1]) + u128::from(values[middle])) / 2) as u64),
    }
}

/// §5.3 over one book: (item, price, basis) per kind, at most 32 by largest basis, by item.
fn expected_lines(book: &BTreeMap<EntityId, Vec<Note>>) -> Vec<(EntityId, u64, u64)> {
    let mut lines = Vec::new();
    for (item, notes) in book {
        let sales: Vec<u64> = notes.iter().filter(|n| n.sale).map(|n| n.price).collect();
        let mut newest: BTreeMap<(EntityId, EntityId), Note> = BTreeMap::new();
        for note in notes.iter().filter(|n| !n.sale) {
            newest.insert((note.seller, note.place), *note);
        }
        let asks: Vec<u64> = newest.values().map(|n| n.price).collect();
        let stock: Vec<u64> = newest.values().filter_map(|n| n.in_stock).collect();
        let (mut base, basis) = if sales.len() >= MIN_BASIS {
            (median(sales.clone()).expect("some"), sales.len())
        } else if asks.len() >= MIN_BASIS {
            (median(asks.clone()).expect("some"), asks.len())
        } else {
            continue;
        };
        if let Some(stock) = median(stock) {
            if stock < SCARCE_BELOW {
                base = (u128::from(base) * (1_000 + SCARCE_PERMILLE) / 1_000) as u64;
            } else if stock > PLENTY_ABOVE {
                base = (u128::from(base) * (1_000 - PLENTY_PERMILLE) / 1_000) as u64;
            }
        }
        lines.push((*item, base.max(1), basis as u64));
    }
    if lines.len() > 32 {
        lines.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
        lines.truncate(32);
        lines.sort_by_key(|line| line.0);
    }
    lines
}

/// Every line of every notice by `clerk` equals the oracle over its book at that instant; how many notices.
fn lines_match_the_book(facts: &[EventEnvelope], posted: &[Posted], clerk: EntityId) -> usize {
    for notice in posted {
        let got: Vec<(EntityId, u64, u64)> = notice.body["prices"]
            .as_array()
            .expect("a prices body")
            .iter()
            .map(|line| {
                (
                    entity(&line["item"]),
                    line["price"].as_u64().expect("a price"),
                    line["basis"].as_u64().expect("a basis"),
                )
            })
            .collect();
        let expected = expected_lines(&book(facts, clerk, notice.index, notice.at));
        assert_eq!(got, expected, "the notice at {} is the book's", notice.at);
    }
    posted.len()
}

// ---- the keepers: moves toward the suggestion current at each instant -----------------------------

/// The suggestion for `item` on `subscriber`'s board at `at`: the current notices on `topics` (issued at or
/// before, valid until after, and not replaced by a newer one of the same publisher and topic); the newest
/// line, ties to the lower publisher.
fn current_suggestion(
    posted: &[Posted],
    topics: &BTreeSet<String>,
    item: EntityId,
    at: i64,
) -> Option<u64> {
    let mut newest: BTreeMap<(String, EntityId), &Posted> = BTreeMap::new();
    for notice in posted
        .iter()
        .filter(|n| n.at <= at && topics.contains(&n.topic))
    {
        newest.insert((notice.topic.clone(), notice.publisher), notice);
    }
    newest
        .values()
        .filter(|notice| at < notice.valid_until)
        .filter_map(|notice| {
            let line = notice.body["prices"]
                .as_array()?
                .iter()
                .find(|line| entity(&line["item"]) == item)?;
            Some((
                notice.at,
                std::cmp::Reverse(notice.publisher),
                line["price"].as_u64()?,
            ))
        })
        .max()
        .map(|(_, _, price)| price)
}

/// The keepers' moves: (toward, away), each `to` checked against the bounds of I-d-5, and each move
/// explained by a current suggestion (AC-d-8 (d)).
fn keeper_moves(
    facts: &[EventEnvelope],
    posted: &[Posted],
    sellers: &BTreeSet<EntityId>,
    topic: &str,
    items: &BTreeMap<EntityId, u64>,
) -> (usize, usize) {
    let topics: BTreeSet<String> = [topic.to_owned()].into();
    let (mut toward, mut away) = (0, 0);
    for fact in facts
        .iter()
        .filter(|f| f.event_type().as_str() == "price-set")
    {
        let body = payload(fact);
        if !sellers.contains(&entity(&body["seller"])) {
            continue;
        }
        let item = entity(&body["item"]);
        let (from, to) = (
            body["from"].as_u64().expect("from"),
            body["to"].as_u64().expect("to"),
        );
        let reference = items[&item];
        let low = (reference * FLOOR_PERMILLE / 1_000).max(1);
        let high = (reference * CEILING_PERMILLE / 1_000).max(low);
        assert!(
            (low..=high).contains(&to),
            "{body} lies within [{low}, {high}]"
        );
        let at = fact.at().seconds();
        let suggestion = current_suggestion(posted, &topics, item, at)
            .unwrap_or_else(|| panic!("the move {body} at {at} has a current suggestion"));
        if to.abs_diff(suggestion) < from.abs_diff(suggestion) || to == low || to == high {
            toward += 1;
        } else {
            eprintln!("away from the suggestion {suggestion} at {at}: {body}");
            away += 1;
        }
    }
    (toward, away)
}

/// The reference values by kind id in the copy.
fn reference_ids(town: &Town) -> BTreeMap<EntityId, u64> {
    references()
        .into_iter()
        .filter_map(|(kind, value)| town.ids.get(&kind).map(|id| (*id, value)))
        .collect()
}

fn clerk_notices(facts: &[EventEnvelope], clerk: EntityId, topic: &str) -> Vec<Posted> {
    notices(facts)
        .into_iter()
        .filter(|notice| notice.publisher == clerk && notice.topic == topic)
        .collect()
}

// ---- AC-d-4 ------------------------------------------------------------------------------------

/// The seats `world.yaml` lists, read from its text (Lakeside's pack needs its pack roots to load).
fn seats(pack: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(pack.join("world.yaml")).expect("world.yaml reads");
    text.lines()
        .skip_while(|line| *line != "seats:")
        .skip(1)
        .take_while(|line| line.starts_with("  - "))
        .map(|line| line.trim_start_matches("  - ").trim().to_owned())
        .collect()
}

/// Every seat of `pack` on `model`, 10 days, with the pack roots `roots`; the save's tables.
fn every_seat_on(pack: &Path, roots: &[String], model: &str, save: &Path) -> Tables {
    let flags: Vec<String> = seats(pack)
        .iter()
        .map(|seat| format!("{seat}={model}"))
        .collect();
    assert!(!flags.is_empty(), "{} lists seats", pack.display());
    let mut extra: Vec<&str> = roots.iter().map(String::as_str).collect();
    for flag in &flags {
        extra.extend(["--seat-model", flag.as_str()]);
    }
    run(pack, DAYS, save, &extra);
    Tables::read(save)
}

#[test]
fn delegation_equals_rule() {
    for (name, pack, roots) in [
        (
            "social-cafe",
            Path::new(SOCIAL_CAFE).to_path_buf(),
            Vec::new(),
        ),
        ("lakeside", lakeside::lakeside(), lakeside::roots()),
    ] {
        let (a, b) = (
            fresh(&format!("price-reporter-{name}-clerk")),
            fresh(&format!("price-reporter-{name}-rule")),
        );
        let clerks = every_seat_on(&pack, &roots, "price-reporter", &a);
        let rule = every_seat_on(&pack, &roots, "rule", &b);
        assert!(rule.facts.len() > 100, "{name}: the run did something");
        rule.assert_same_history(&clerks, &format!("{name}: every seat on price-reporter"));
    }
}

// ---- AC-d-8 ------------------------------------------------------------------------------------

#[test]
fn clerk_publishes_and_keepers_follow() {
    let scratch = fresh("price-reporter-w2");
    let root = scratch.path().to_path_buf();
    let pack = world(&root, CAROL, TOWN);
    let town = Town::read(&pack);
    let carol = town.id(CAROL.key);
    let save = root.join("save");
    run(
        &pack,
        DAYS,
        &save,
        &["--decisions", arg(&w2_decisions(&root, true))],
    );
    let facts = facts(&save);
    let posted = clerk_notices(&facts, carol, CAROL.topic);
    // (a)
    assert!(
        posted.len() >= 10,
        "at least 10 notices by Carol: {}",
        posted.len()
    );
    assert_eq!(
        refused(&save, carol, "publish-notice"),
        0,
        "no publish is refused"
    );
    assert_audiences(&facts, &posted);
    // (b)
    lines_match_the_book(&facts, &posted, carol);
    // (c), (d)
    let shops: BTreeSet<EntityId> = [town.id("corner-store"), town.id("cafe-company")].into();
    let (toward, away) = keeper_moves(&facts, &posted, &shops, CAROL.topic, &reference_ids(&town));
    assert!(toward >= 1, "a keeper moved toward a suggestion");
    assert_eq!(away, 0, "no keeper moved away");
}

#[test]
fn same_history_twice_and_across_a_resume() {
    let scratch = fresh("price-reporter-w2-resume");
    let root = scratch.path().to_path_buf();
    let pack = world(&root, CAROL, TOWN);
    let file = w2_decisions(&root, true);
    let flags = ["--decisions", arg(&file)];
    let (a, b, c) = (root.join("a"), root.join("b"), root.join("c"));
    run(&pack, DAYS, &a, &flags);
    run(&pack, DAYS, &b, &flags);
    run(&pack, 5, &c, &flags);
    run(&pack, DAYS, &c, &flags);
    let control = Tables::read(&a);
    control.assert_same_history(&Tables::read(&b), "W2: a second run");
    control.assert_same_history(&Tables::read(&c), "W2: killed at day 5 and resumed");
}

// ---- AC-d-9 ------------------------------------------------------------------------------------

#[test]
fn second_world() {
    let scratch = fresh("price-reporter-w3");
    let root = scratch.path().to_path_buf();
    let pack = world(&root, HANA, TOWN);
    let town = Town::read(&pack);
    let hana = town.id(HANA.key);
    let save = root.join("save");
    run(
        &pack,
        DAYS,
        &save,
        &["--decisions", arg(&w3_decisions(&root))],
    );
    let facts = facts(&save);
    let posted = clerk_notices(&facts, hana, HANA.topic);
    assert!(
        posted.len() >= 5,
        "at least 5 notices by Hana: {}",
        posted.len()
    );
    assert_audiences(&facts, &posted);
    lines_match_the_book(&facts, &posted, hana);
    let cafe: BTreeSet<EntityId> = [town.id("cafe-company")].into();
    let (toward, away) = keeper_moves(&facts, &posted, &cafe, HANA.topic, &reference_ids(&town));
    assert!(
        toward >= 1,
        "Alice, on motivated, moved toward a suggestion"
    );
    assert_eq!(away, 0, "Alice never moved away");
}

// ---- AC-d-10 -----------------------------------------------------------------------------------

/// A removal variant, 5 days: its facts.
fn removed(
    name: &str,
    clerk: Clerk,
    variant: Variant,
    decisions: impl FnOnce(&Path) -> PathBuf,
) -> Vec<EventEnvelope> {
    let scratch = fresh(&format!("price-reporter-{name}"));
    let root = scratch.path().to_path_buf();
    let pack = world(&root, clerk, variant);
    let save = root.join("save");
    run(&pack, 5, &save, &["--decisions", arg(&decisions(&root))]);
    facts(&save)
}

#[test]
fn removable() {
    let without_notices = Variant {
        notices: false,
        ..TOWN
    };
    for (name, clerk) in [("w4a", CAROL), ("w4a-w3", HANA)] {
        let facts = removed(name, clerk, without_notices, |root| {
            if clerk.key == "hana" {
                w3_decisions(root)
            } else {
                w2_decisions(root, true)
            }
        });
        assert_eq!(count(&facts, "notice-published"), 0, "{name}: no notice");
        assert_eq!(
            count(&facts, "price-set"),
            0,
            "{name}: no move from a suggestion"
        );
        assert!(count(&facts, "sold") >= 1, "{name}: trade goes on");
    }
    let without_survey = Variant {
        survey: false,
        ..TOWN
    };
    let facts = removed("w4b", CAROL, without_survey, |root| {
        w2_decisions(root, true)
    });
    assert_eq!(
        count(&facts, "notice-published"),
        0,
        "w4b: no book, no notice"
    );
    assert!(count(&facts, "sold") >= 1, "w4b: trade goes on");
    let facts = removed("w4c", CAROL, TOWN, |root| w2_decisions(root, false));
    let genesis = facts.first().map_or(0, |fact| fact.at().seconds());
    for kind in ["price-set", "listed", "withdrawn"] {
        let after: usize = facts
            .iter()
            .filter(|fact| fact.event_type().as_str() == kind && fact.at().seconds() > genesis)
            .count();
        assert_eq!(
            after, 0,
            "w4c: no {kind} after genesis without a pricing block"
        );
    }
}
