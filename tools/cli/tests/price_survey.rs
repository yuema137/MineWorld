//! S31-c C5 — a surveyor's book holds exactly the sales it perceived, in a real town
//! (`pr-s31c-price-survey.md` §8, AC-c-6 … AC-c-9; `docs/decisions/ARC-116.md`).
//!
//! ```text
//! reporter_knows_only_what_it_perceived   AC-c-6  W2: Market Town + price-survey, Carol a surveyor
//!                                                 of food and drink; 30 days, seed 7. The causes of
//!                                                 her `price-noted { sale }` equal the tracked `sold`
//!                                                 she perceived by presence's fold over the real log
//! moving_the_routine_removes_store_sales  AC-c-7  W3: Carol's 07:00 store segment moved to the park
//! book_is_persisted_and_replayed          AC-c-8  W2's save replays; its last snapshot's book equals a
//!                                                 mirror fold of the save's own facts
//! optional_and_removable                  AC-c-9  Social Café + price-survey without economy fails
//!                                                 validate naming economy; W2 without Carol's
//!                                                 section states no price-survey fact
//! ```
//!
//! Oracles: presence's `perceived_by` over the save's facts, the copy's own `items/*.yaml` text for
//! which kinds are tracked, and `mineworld perceived`; never a value `price-survey` reports about
//! itself. The pack's facts are read by slug and JSON field, with no dependency on its types.
//! Long (four 30-day runs): milestone and CI only (R-LOAD-1).

mod headless;
mod market;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use headless::{Scratch, Tables, fresh, mineworld, stderr, stdout};
use market::{MARKET_TOWN, Town, run_pack};
use mineworld_contracts::{Causation, EntityId, EventEnvelope, EventId, ItemId, PlaceId};
use mineworld_persistence::{WorldRevision, format};
use serde_json::Value;

const DAYS: u64 = 30;
const SEED: u64 = 7;
const SECTION: &str = "price-survey: { kinds: { categories: [food, drink] } }\n";
const STORE_LINE: &str = "  - { from: \"07:00\", place: store, label: shopping }\n";
const PARK_LINE: &str = "  - { from: \"07:00\", place: park, label: shopping }\n";

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the pack lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// Every file of a pack, relative to it, with its text.
fn files(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut found = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("lists") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let text = std::fs::read_to_string(&path).expect("reads");
                found.insert(path.strip_prefix(root).expect("inside").to_path_buf(), text);
            }
        }
    }
    found
}

/// A Market Town copy with `price-survey` enabled after `weather`; with Carol's section when
/// `section`; with her 07:00 segment in the park when `to_park`. Asserted to differ from Market Town
/// by exactly those edits.
struct Copy {
    scratch: Scratch,
}

impl Copy {
    fn new(name: &str, section: bool, to_park: bool) -> Self {
        let scratch = fresh(name);
        let root = scratch.join("market-town");
        copy_dir(Path::new(MARKET_TOWN), &root);
        let manifest = root.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("reads");
        let enabled = text.replacen("  - weather\n", "  - weather\n  - price-survey\n", 1);
        std::fs::write(&manifest, &enabled).expect("writes");
        let carol = root.join("people/carol.yaml");
        let mut person = std::fs::read_to_string(&carol).expect("reads");
        if to_park {
            assert_eq!(
                person.matches(STORE_LINE).count(),
                1,
                "carol shops at 07:00"
            );
            person = person.replace(STORE_LINE, PARK_LINE);
        }
        if section {
            person.push_str(SECTION);
        }
        std::fs::write(&carol, &person).expect("writes");
        let (ours, theirs) = (files(&root), files(Path::new(MARKET_TOWN)));
        let differing: Vec<_> = ours
            .iter()
            .filter(|(path, text)| theirs.get(*path) != Some(text))
            .map(|(path, _)| path.clone())
            .collect();
        let mut expected = vec![PathBuf::from("world.yaml")];
        if section || to_park {
            expected.insert(0, PathBuf::from("people/carol.yaml"));
        }
        assert_eq!(differing, expected, "only the edits differ");
        assert_eq!(ours.len(), theirs.len(), "no file added or removed");
        Self { scratch }
    }

    fn world(&self) -> PathBuf {
        self.scratch.join("market-town")
    }

    fn save(&self) -> PathBuf {
        self.scratch.join("save")
    }

    fn run(&self) {
        run_pack(&self.world(), SEED, DAYS, Some(&self.save()));
    }

    fn facts(&self) -> Vec<EventEnvelope> {
        Tables::read(&self.save())
            .facts
            .iter()
            .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
            .collect()
    }
}

fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).expect("a JSON payload")
}

/// The item kinds whose file says `category: food` or `category: drink`, read from the copy's text.
fn tracked_items(world: &Path, town: &Town) -> BTreeSet<ItemId> {
    files(&world.join("items"))
        .into_iter()
        .filter(|(_, text)| text.contains("category: food") || text.contains("category: drink"))
        .map(|(path, _)| {
            let key = path.file_stem().expect("a stem").to_str().expect("UTF-8");
            ItemId::new(town.id(key), mineworld_contracts::EntityType::Item).expect("an item")
        })
        .collect()
}

struct Sale {
    item: ItemId,
    seller: EntityId,
    price: u64,
    place: PlaceId,
}

fn sale(fact: &EventEnvelope) -> Sale {
    let value = payload(fact);
    let field = |key: &str| value[key].clone();
    Sale {
        item: serde_json::from_value(field("item")).expect("item"),
        seller: serde_json::from_value(field("seller")).expect("seller"),
        price: field("price").as_u64().expect("price"),
        place: serde_json::from_value(field("place")).expect("place"),
    }
}

/// AC-c-6's equality on one copy's save: the tracked `sold` Carol perceived, by event id.
fn assert_book_equals_perception(copy: &Copy) -> (BTreeSet<EventId>, Vec<EventEnvelope>) {
    let town = Town::read(&copy.world());
    let carol = town.id("carol");
    let facts = copy.facts();
    let tracked = tracked_items(&copy.world(), &town);
    let perceived: BTreeSet<EventId> =
        mineworld_presence::audience::perceived_by(facts.iter(), carol, None)
            .filter(|fact| fact.event_type().as_str() == "sold")
            .filter(|fact| tracked.contains(&sale(fact).item))
            .map(EventEnvelope::id)
            .collect();
    let by_id: BTreeMap<EventId, &EventEnvelope> = facts.iter().map(|f| (f.id(), f)).collect();
    let mut noted = BTreeSet::new();
    for fact in facts
        .iter()
        .filter(|f| f.event_type().as_str() == "price-noted")
    {
        let value = payload(fact);
        if value["source"] != "sale" {
            continue;
        }
        let surveyor: EntityId = serde_json::from_value(value["surveyor"].clone()).expect("an id");
        assert_eq!(surveyor, carol, "the only surveyor");
        let Causation::Event(cause) = fact.caused_by() else {
            panic!("#{} is not caused by a fact", fact.id().raw());
        };
        let sold = sale(by_id[cause]);
        let note = &value["notes"][0];
        let noted_item: ItemId = serde_json::from_value(note["item"].clone()).expect("an item");
        let noted_seller: EntityId =
            serde_json::from_value(note["seller"].clone()).expect("a seller");
        let noted_place: PlaceId = serde_json::from_value(value["place"].clone()).expect("a place");
        assert_eq!(
            (
                noted_item,
                noted_seller,
                note["price"].as_u64(),
                noted_place
            ),
            (sold.item, sold.seller, Some(sold.price), sold.place),
            "#{} agrees with its sale",
            fact.id().raw()
        );
        assert_eq!(fact.at(), by_id[cause].at(), "same instant as its sale");
        noted.insert(*cause);
    }
    assert_eq!(noted, perceived, "noted sales = perceived tracked sales");
    (perceived, facts)
}

#[test]
fn reporter_knows_only_what_it_perceived() {
    let copy = Copy::new("price-survey-w2", true, false);
    copy.run();
    let (perceived, facts) = assert_book_equals_perception(&copy);
    let town = Town::read(&copy.world());
    let (store, cafe) = (town.id("store"), town.id("cafe"));
    let by_id: BTreeMap<EventId, &EventEnvelope> = facts.iter().map(|f| (f.id(), f)).collect();
    let places: BTreeSet<EntityId> = perceived
        .iter()
        .map(|id| sale(by_id[id]).place.entity_id())
        .collect();
    assert!(
        places.contains(&store),
        "non-vacuous: a store sale perceived"
    );
    assert!(places.contains(&cafe), "non-vacuous: a café sale perceived");
    let tracked = tracked_items(&copy.world(), &town);
    assert!(
        facts.iter().any(|fact| fact.event_type().as_str() == "sold"
            && tracked.contains(&sale(fact).item)
            && !perceived.contains(&fact.id())),
        "non-vacuous: a tracked sale Carol did not perceive"
    );
    // The binary's export agrees with the library's fold.
    let world = copy.world();
    let save = copy.save();
    let output = mineworld(&[
        "perceived",
        world.to_str().expect("UTF-8"),
        "--save",
        save.to_str().expect("UTF-8"),
        "--person",
        "carol",
        "--json",
    ]);
    assert!(output.status.success(), "perceived: {}", stderr(&output));
    let exported: BTreeSet<EventId> = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("one fact per line"))
        .filter(|fact| fact["event_type"] == "sold")
        .map(|fact| {
            let raw: u64 = fact["id"]
                .as_str()
                .and_then(|id| id.parse().ok())
                .or_else(|| fact["id"].as_u64())
                .expect("an id");
            EventId::from_raw(raw)
        })
        .filter(|id| tracked.contains(&sale(by_id[id]).item))
        .collect();
    assert_eq!(exported, perceived, "mineworld perceived agrees");
}

#[test]
fn moving_the_routine_removes_store_sales() {
    let copy = Copy::new("price-survey-w3", true, true);
    copy.run();
    let (perceived, facts) = assert_book_equals_perception(&copy);
    let town = Town::read(&copy.world());
    let store = town.id("store");
    let by_id: BTreeMap<EventId, &EventEnvelope> = facts.iter().map(|f| (f.id(), f)).collect();
    assert!(
        perceived
            .iter()
            .all(|id| sale(by_id[id]).place.entity_id() != store),
        "no store sale in Carol's book"
    );
    assert!(
        facts
            .iter()
            .any(|fact| fact.event_type().as_str() == "sold"
                && sale(fact).place.entity_id() == store),
        "the store still sells"
    );
}

/// §5.3's rules written out: per kind a ring of `depth`, pruned by `window` at each note; by item.
fn mirror(facts: &[EventEnvelope], carol: EntityId, before: u64) -> BTreeMap<String, Vec<Value>> {
    let mut depth = 0usize;
    let mut window = 0i64;
    let mut kinds: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for fact in facts.iter().filter(|fact| fact.id().raw() < before) {
        let value = payload(fact);
        match fact.event_type().as_str() {
            "survey-book-opened" => {
                depth = usize::try_from(value["depth"].as_u64().expect("depth")).expect("small");
                window = value["window_s"].as_i64().expect("window");
            }
            "price-noted" => {
                let surveyor: EntityId =
                    serde_json::from_value(value["surveyor"].clone()).expect("an id");
                assert_eq!(surveyor, carol);
                let at = fact.at().seconds();
                for note in value["notes"].as_array().expect("notes") {
                    let ring = kinds.entry(note["item"].to_string()).or_default();
                    ring.push(serde_json::json!({
                        "at": at, "place": value["place"], "seller": note["seller"],
                        "price": note["price"], "source": value["source"],
                        "in_stock": note["in_stock"],
                    }));
                    let excess = ring.len().saturating_sub(depth);
                    ring.drain(..excess);
                }
                for ring in kinds.values_mut() {
                    ring.retain(|note| note["at"].as_i64().expect("at") + window > at);
                }
                kinds.retain(|_, ring| !ring.is_empty());
            }
            _ => {}
        }
    }
    kinds
}

#[test]
fn book_is_persisted_and_replayed() {
    let copy = Copy::new("price-survey-replay", true, false);
    copy.run();
    let world = copy.world();
    let save = copy.save();
    let output = mineworld(&[
        "replay",
        world.to_str().expect("UTF-8"),
        "--save",
        save.to_str().expect("UTF-8"),
    ]);
    assert!(
        output.status.success(),
        "replay: {}{}",
        stdout(&output),
        stderr(&output)
    );
    let tables = Tables::read(&save);
    let (revision, stored) = tables.snapshots.last().expect("a snapshot");
    let snapshot = format::decode_snapshot(stored, WorldRevision::from_raw(*revision))
        .expect("the snapshot decodes");
    let carol = Town::read(&world).id("carol");
    let record = snapshot
        .components
        .iter()
        .find(|record| {
            record.component_type().as_str() == "survey-book" && record.entity() == carol
        })
        .expect("carol's book is in the snapshot");
    let book: Value = serde_json::from_slice(record.payload()).expect("decodes");
    let held: BTreeMap<String, Vec<Value>> = book["kinds"]
        .as_array()
        .expect("kinds")
        .iter()
        .map(|kind| {
            (
                kind["item"].to_string(),
                kind["notes"].as_array().expect("notes").clone(),
            )
        })
        .collect();
    let facts = copy.facts();
    assert_eq!(held, mirror(&facts, carol, snapshot.time.next_event));
}

#[test]
fn optional_and_removable() {
    // W4: Social Café with price-survey and no economy is refused by name.
    let scratch = fresh("price-survey-w4");
    let root = scratch.join("social-cafe");
    copy_dir(Path::new(headless::PACK), &root);
    let manifest = root.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("reads");
    std::fs::write(
        &manifest,
        text.replacen("  - movement\n", "  - movement\n  - price-survey\n", 1),
    )
    .expect("writes");
    let output = mineworld(&["validate", root.to_str().expect("UTF-8")]);
    let printed = format!("{}{}", stdout(&output), stderr(&output));
    assert!(!output.status.success(), "validate refuses W4: {printed}");
    assert!(printed.contains("economy"), "names economy: {printed}");

    // W2 without Carol's section: enabled, and silent.
    let copy = Copy::new("price-survey-silent", false, false);
    copy.run();
    let facts = copy.facts();
    assert!(
        !facts.iter().any(|fact| matches!(
            fact.event_type().as_str(),
            "survey-book-opened" | "price-noted"
        )),
        "no price-survey fact without a surveyor"
    );
}
