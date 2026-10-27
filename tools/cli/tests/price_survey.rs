//! S31-c C5 — a surveyor's book holds exactly the sales it perceived, in a real town
//! (`pr-s31c-price-survey.md` §8, AC-c-6 … AC-c-9; `docs/decisions/ARC-116.md`).
//!
//! ```text
//! reporter_knows_only_what_it_perceived   AC-c-6  W2: Market Town + price-survey, Carol a surveyor
//!                                                 of food and drink; 30 days, seed 7. The causes of
//!                                                 her `price-noted { sale }` equal the tracked `sold`
//!                                                 she perceived by presence's fold over the real log
//! moving_the_routine_removes_store_sales  AC-c-7  W3: Carol's 07:00 store segment moved to the park;
//!                                                 against W2, no store sale while her agenda names
//!                                                 the store, and fewer store sales in her book
//! book_is_persisted_and_replayed          AC-c-8  W2's save replays; its last snapshot's book equals a
//!                                                 mirror fold of the save's own facts
//! optional_and_removable                  AC-c-9  Social Café + price-survey without economy fails
//!                                                 validate naming economy; W2 without Carol's
//!                                                 section states no price-survey fact
//! generated_world_notes_what_it_perceived_and_surveys_what_is_shown
//!                                         AC-c-2 (a)(b)(d), AC-c-3 (a), AC-c-6 on W5: a World Pack
//!                                                 the test writes, sharing no key, name or amount
//!                                                 with Market Town or W1
//! the_pack_names_no_market_town_key       §8      `git grep` over the pack's source is empty
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
use mineworld_contracts::{Causation, EntityId, EventEnvelope, EventId, ItemId, PersonId, PlaceId};
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

/// A `price-noted` or `survey-book-opened` payload's surveyor: a typed `PersonId` on the wire.
fn surveyor_of(value: &Value) -> EntityId {
    serde_json::from_value::<PersonId>(value["surveyor"].clone())
        .expect("a person id")
        .entity_id()
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
    let facts = copy.facts();
    let perceived = assert_noted_equals_perceived(&copy.world(), &facts, "carol");
    (perceived, facts)
}

/// AC-c-6's equality on any world's facts, for the one surveyor `key`: the causes of its
/// `price-noted { sale }` equal the tracked `sold` it perceived by presence's fold, each note agrees
/// with its sale and shares its instant. Returns the perceived tracked sales.
fn assert_noted_equals_perceived(
    world: &Path,
    facts: &[EventEnvelope],
    key: &str,
) -> BTreeSet<EventId> {
    let town = Town::read(world);
    let carol = town.id(key);
    let tracked = tracked_items(world, &town);
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
        assert_eq!(surveyor_of(&value), carol, "the only surveyor");
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
    perceived
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
    // W2, the routine at the store: the baseline the move is measured against.
    let w2 = Copy::new("price-survey-w3-baseline", true, false);
    w2.run();
    let (w2_perceived, w2_facts) = assert_book_equals_perception(&w2);
    let w2_book = store_sales_by_agenda(&w2.world(), &w2_perceived, &w2_facts);
    // W3, the 07:00 segment moved to the park.
    let copy = Copy::new("price-survey-w3", true, true);
    copy.run();
    let (perceived, facts) = assert_book_equals_perception(&copy);
    let book = store_sales_by_agenda(&copy.world(), &perceived, &facts);
    let town = Town::read(&copy.world());
    let store = town.id("store");
    // The routine brought Carol to the store in W2, so her book there holds store sales her agenda
    // put her in front of; moved, it holds none. She may still walk in on an errand of her own (a
    // hungry person goes where food is sold, rule's fetch band), and a sale she sees then is in her
    // book as AC-c-6 requires: what the move removes is the routine's share, which is most of it.
    assert!(
        w2_book.contains(&Some(store)),
        "non-vacuous: in W2 Carol notes store sales while her routine has her at the store"
    );
    assert!(
        book.iter().all(|agenda| *agenda != Some(store)),
        "no store sale in Carol's W3 book while her agenda names the store"
    );
    assert!(
        book.len() < w2_book.len(),
        "moving the routine removes store sales from Carol's book: W3 {} vs W2 {}",
        book.len(),
        w2_book.len()
    );
    assert!(
        facts
            .iter()
            .any(|fact| fact.event_type().as_str() == "sold"
                && sale(fact).place.entity_id() == store),
        "the store still sells"
    );
}

/// For each store sale in Carol's book (`perceived`), the place her agenda named when it was sold, read
/// from schedule's `agenda-changed` facts in log order (`None` before her first one).
fn store_sales_by_agenda(
    world: &Path,
    perceived: &BTreeSet<EventId>,
    facts: &[EventEnvelope],
) -> Vec<Option<EntityId>> {
    let town = Town::read(world);
    let (carol, store) = (town.id("carol"), town.id("store"));
    let mut agenda: Option<EntityId> = None;
    let mut out = Vec::new();
    for fact in facts {
        match fact.event_type().as_str() {
            "agenda-changed" => {
                let value = payload(fact);
                let person: PersonId =
                    serde_json::from_value(value["person"].clone()).expect("person");
                if person.entity_id() == carol {
                    let place: Option<PlaceId> =
                        serde_json::from_value(value["place"].clone()).expect("place");
                    agenda = place.map(|place| place.entity_id());
                }
            }
            "sold" if perceived.contains(&fact.id()) && sale(fact).place.entity_id() == store => {
                out.push(agenda);
            }
            _ => {}
        }
    }
    out
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
                assert_eq!(surveyor_of(&value), carol);
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

// ── W5: a generated second world (§8 revision 2) ─────────────────────────────────────────────────

/// W5's run: short, because the world is small and every arm needs only its first sales.
const W5_DAYS: u64 = 3;
const W5_SEED: u64 = 11;

/// W5's item kinds: key, category, display name. `tea` and `soap` of §8's sketch are Market Town
/// keys, so W5 uses `mate` and `twine` (D-c-6).
const W5_ITEMS: [(&str, &str, &str); 3] = [
    ("rye", "food", "Rye Loaf"),
    ("mate", "drink", "Yerba Mate"),
    ("twine", "goods", "Hemp Twine"),
];

/// W5's shops: operator, place, and per kind `(item, holding, price)`. Every amount is W5's own.
type Shop = (
    &'static str,
    &'static str,
    &'static [(&'static str, u32, u64)],
);
const W5_SHOPS: [Shop; 2] = [
    (
        "loft-bakers",
        "bakery",
        &[("rye", 41, 37), ("mate", 17, 53), ("twine", 13, 71)],
    ),
    (
        "square-kiosk",
        "kiosk",
        &[("rye", 23, 43), ("mate", 29, 47), ("twine", 19, 67)],
    ),
];

/// W5's people: key, starting place, wallet. `ines` is the surveyor. No `name:`: `naming` is not one
/// of the six packs §8 enables, so a person's display name is not W5's content.
const W5_PEOPLE: [(&str, &str, u64); 3] = [
    ("ines", "bakery", 7919),
    ("tomas", "bakery", 9973),
    ("wen", "kiosk", 8761),
];

/// Ines's section: food and drink, with settings none of W1's or W2's.
const W5_SECTION: &str = "price-survey: { kinds: { categories: [food, drink] }, depth: 9, \
                          window: \"3d\", resurvey_after: \"2h\" }\n";

/// W1's keys (the pack fixture, `systems/price-survey/tests/support/mod.rs`).
const W1_KEYS: [&str; 13] = [
    "store",
    "cafe",
    "street",
    "apple",
    "bread",
    "juice",
    "pen",
    "corner-store",
    "grace",
    "pia",
    "alice",
    "bob",
    "nat",
];

/// W5: a World Pack written from the tables above. Two shops in two places joined by a square, only
/// the six packs §8 names, and no key, name or amount of Market Town or W1.
fn generated_square(scratch: &Path) -> PathBuf {
    let world = scratch.join("generated-square");
    for directory in ["places", "people", "items", "organizations", "configure"] {
        std::fs::create_dir_all(world.join(directory)).expect("writable");
    }
    let list = |keys: Vec<&str>| keys.join(", ");
    let people = list(W5_PEOPLE.iter().map(|person| person.0).collect());
    let manifest = format!(
        "world:\n  id: generated-square\n  name: Generated Square\n  version: 0.1.0\n  license: MIT\n\
         mineworld: \"^0.1\"\n\
         systems: [presence, movement, item, inventory, economy, price-survey]\n\
         places: [bakery, kiosk, square]\npopulation: [{people}]\nitems: [{}]\n\
         organizations: [{}]\nseats: [{people}]\nconfigure: [economy]\n\
         settlement: {{ kind: none }}\n",
        list(W5_ITEMS.iter().map(|item| item.0).collect()),
        list(W5_SHOPS.iter().map(|shop| shop.0).collect()),
    );
    let mut files = vec![
        ("world.yaml".to_owned(), manifest),
        (
            "configure/economy.yaml".to_owned(),
            "currency: { name: square bit, symbol: b, exponent: 0, symbol_at: after }\n".to_owned(),
        ),
        (
            "places/square.yaml".to_owned(),
            "tags: [square, public, outdoor]\n".to_owned(),
        ),
    ];
    for (_, place, _) in W5_SHOPS {
        files.push((
            format!("places/{place}.yaml"),
            "tags: [shop, public]\npassages:\n  - to: square\n".to_owned(),
        ));
    }
    for (key, category, name) in W5_ITEMS {
        files.push((
            format!("items/{key}.yaml"),
            format!("tags: [{category}]\nitem: {{ category: {category}, name: {name} }}\n"),
        ));
    }
    for (operator, place, stock) in W5_SHOPS {
        let holdings = stock
            .iter()
            .map(|(item, count, _)| format!("{item}: {count}"))
            .collect::<Vec<_>>()
            .join(", ");
        let prices = stock
            .iter()
            .map(|(item, _, price)| format!("{item}: {price}"))
            .collect::<Vec<_>>()
            .join(", ");
        files.push((
            format!("organizations/{operator}.yaml"),
            format!(
                "tags: [business]\nholdings: {{ {holdings} }}\n\
                 economy:\n  wallet: 3001\n  shop:\n    at: {place}\n    prices: {{ {prices} }}\n"
            ),
        ));
    }
    for (key, place, wallet) in W5_PEOPLE {
        let section = if key == "ines" { W5_SECTION } else { "" };
        files.push((
            format!("people/{key}.yaml"),
            format!(
                "tags: [resident]\nlocation:\n  place: {place}\n\
                 economy: {{ wallet: {wallet} }}\n{section}"
            ),
        ));
    }
    for (file, text) in files {
        std::fs::write(world.join(file), text).expect("writes");
    }
    world
}

/// W5 shares no key with Market Town or W1, and no item's name appears in any Market Town file.
fn assert_nothing_shared(world: &Path) {
    let ours = Town::read(world);
    let market = Town::read(Path::new(MARKET_TOWN));
    let shared: Vec<&String> = ours
        .ids
        .keys()
        .filter(|key| market.ids.contains_key(*key) || W1_KEYS.contains(&key.as_str()))
        .collect();
    assert!(shared.is_empty(), "W5 shares keys {shared:?}");
    let texts = files(Path::new(MARKET_TOWN));
    for (_, _, name) in W5_ITEMS {
        assert!(
            !texts.values().any(|text| text.contains(name)),
            "{name} appears in Market Town"
        );
    }
}

/// The shop's authored stock and price of each tracked kind at `place`, by `(item, seller)` id.
fn authored_asks(town: &Town, place: EntityId) -> BTreeMap<(ItemId, EntityId), (u64, u32)> {
    let tracked: BTreeSet<&str> = W5_ITEMS
        .iter()
        .filter(|(_, category, _)| matches!(*category, "food" | "drink"))
        .map(|item| item.0)
        .collect();
    W5_SHOPS
        .iter()
        .filter(|(_, at, _)| town.id(at) == place)
        .flat_map(|(operator, _, stock)| {
            stock
                .iter()
                .filter(|(item, _, _)| tracked.contains(item))
                .map(|(item, count, price)| {
                    let item = ItemId::new(town.id(item), mineworld_contracts::EntityType::Item)
                        .expect("an item");
                    ((item, town.id(operator)), (*price, *count))
                })
        })
        .collect()
}

/// W5 through the real binary: AC-c-6's equality, AC-c-2 (a), (b), (d) and AC-c-3 (a), with
/// non-vacuity floors fixed before the first run (`ARC-23`).
#[test]
fn generated_world_notes_what_it_perceived_and_surveys_what_is_shown() {
    let scratch = fresh("price-survey-w5");
    let world = generated_square(&scratch);
    assert_nothing_shared(&world);
    let save = scratch.join("save");
    run_pack(&world, W5_SEED, W5_DAYS, Some(&save));
    let facts: Vec<EventEnvelope> = Tables::read(&save)
        .facts
        .iter()
        .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
        .collect();
    let town = Town::read(&world);
    let ines = town.id("ines");

    // AC-c-6's equality on W5; with it, (b) and (d): a sale Ines did not perceive and a perceived
    // `twine` sale are noted by nothing.
    let perceived = assert_noted_equals_perceived(&world, &facts, "ines");
    let sales: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "sold")
        .collect();
    let all_perceived: BTreeSet<EventId> =
        mineworld_presence::audience::perceived_by(facts.iter(), ines, None)
            .map(EventEnvelope::id)
            .collect();
    let twine = town.id("twine");
    let tracked = tracked_items(&world, &town);
    eprintln!(
        "W5 {W5_DAYS} days: {} sales, {} perceived tracked",
        sales.len(),
        perceived.len()
    );
    // (a) a witnessed sale is noted.
    assert!(!perceived.is_empty(), "AC-c-2 (a): Ines notes a sale");
    // (b) a tracked sale where Ines was not is not noted.
    assert!(
        sales
            .iter()
            .any(|fact| tracked.contains(&sale(fact).item) && !all_perceived.contains(&fact.id())),
        "AC-c-2 (b): a tracked sale Ines did not perceive"
    );
    // (d) a `twine` sale Ines perceived is not noted.
    assert!(
        sales.iter().any(|fact| sale(fact).item.entity_id() == twine
            && all_perceived.contains(&fact.id())
            && !perceived.contains(&fact.id())),
        "AC-c-2 (d): a perceived goods sale, unnoted"
    );

    // AC-c-3 (a): each survey copies exactly the tracked asks shown where Ines stands, in
    // `(item, seller)` order, each with the shop's stock: authored less what was sold before it.
    let surveys: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| {
            fact.event_type().as_str() == "price-noted" && payload(fact)["source"] == "ask"
        })
        .collect();
    assert!(!surveys.is_empty(), "AC-c-3 (a): Ines surveys");
    for survey in surveys {
        let value = payload(survey);
        assert_eq!(surveyor_of(&value), ines);
        let place: PlaceId = serde_json::from_value(value["place"].clone()).expect("a place");
        let notes: Vec<(ItemId, EntityId, u64, Option<u32>)> = value["notes"]
            .as_array()
            .expect("notes")
            .iter()
            .map(|note| {
                (
                    serde_json::from_value(note["item"].clone()).expect("an item"),
                    serde_json::from_value(note["seller"].clone()).expect("a seller"),
                    note["price"].as_u64().expect("a price"),
                    note["in_stock"]
                        .as_u64()
                        .map(|n| u32::try_from(n).expect("small")),
                )
            })
            .collect();
        let expected: Vec<(ItemId, EntityId, u64, Option<u32>)> =
            authored_asks(&town, place.entity_id())
                .into_iter()
                .map(|((item, seller), (price, holding))| {
                    let sold: u32 = sales
                        .iter()
                        .filter(|fact| fact.id().raw() < survey.id().raw())
                        .map(|fact| (sale(fact), payload(fact)["count"].as_u64()))
                        .filter(|(sold, _)| sold.item == item && sold.seller == seller)
                        .map(|(_, count)| u32::try_from(count.expect("a count")).expect("small"))
                        .sum();
                    (item, seller, price, Some(holding - sold))
                })
                .collect();
        assert_eq!(
            notes,
            expected,
            "#{} copies the shown asks",
            survey.id().raw()
        );
    }
}

/// §8: the pack's source names no Market Town key, so a rule that holds only for the default town
/// cannot hide in it. The pack's own W1 fixture and README use these words by design; the source
/// does not (D-c-7).
#[test]
fn the_pack_names_no_market_town_key() {
    let output = std::process::Command::new("git")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .args([
            "grep",
            "-n",
            "-i",
            "-w",
            "-E",
            "carol|grace|store|cafe|bread",
            "--",
            "systems/price-survey/src",
            "systems/price-survey/Cargo.toml",
        ])
        .output()
        .expect("git runs");
    assert_eq!(
        output.status.code(),
        Some(1),
        "git grep finds Market Town keys: {}{}",
        stdout(&output),
        stderr(&output)
    );
}
