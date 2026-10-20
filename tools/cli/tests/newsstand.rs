//! The station newsstand: Market Town's hall sells the paper, its keeper is paid, and a town without
//! economy has no shop there (S29-b §7, AC-b-1 … AC-b-7). Every criterion runs on Market Town and on
//! `kiosk-yard`, a generated town that shares no key with it.
//!
//! The oracles are the pack's own files (the authored kind and price) and the genesis and stored facts,
//! decoded by `market::market_fact`; never a value the code under test reports about itself.

mod headless;
mod market;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use headless::{Tables, fresh, mineworld, stderr};
use market::{MARKET_TOWN, MarketFact, Town, market_fact, market_lives, run_pack};
use mineworld_contracts::{EntityId, EntityType, EventEnvelope, WorldTime};
use mineworld_test_support::Scratch;
use mineworld_worldpack::WorldPack;
use serde_json::Value;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// `kiosk-yard`: a small town with a station whose hall is a newsstand (§7). No key is Market Town's.
fn kiosk_yard(scratch: &Path) -> PathBuf {
    let world = scratch.join("kiosk-yard");
    for directory in ["places", "people", "items", "organizations"] {
        std::fs::create_dir_all(world.join(directory)).expect("writable");
    }
    let files = [
        (
            "world.yaml",
            "world:\n  id: kiosk-yard\n  name: Kiosk Yard\n  version: 0.1.0\n  license: MIT\n\
             mineworld: \"^0.1\"\n\
             systems: [presence, movement, schedule, item, inventory, economy, employment]\n\
             places: [booking-office, concourse, lane, platform-1]\npopulation: [lu, mo]\nitems: [gazette]\n\
             organizations: [paper-kiosk]\nseats: [lu, mo]\n\
             settlement:\n  kind: town\n  station: concourse\n  landmark: concourse\n",
        ),
        ("places/lane.yaml", "tags: [street, public, outdoor]\n"),
        (
            "places/concourse.yaml",
            "tags: [station, public, landmark]\npassages:\n  - to: lane\n",
        ),
        (
            "places/booking-office.yaml",
            "tags: [station, ticket-office, public]\npassages:\n  - to: concourse\n",
        ),
        (
            "places/platform-1.yaml",
            "tags: [station, stop, platform, public, outdoor]\npassages:\n  - to: concourse\n",
        ),
        (
            "items/gazette.yaml",
            "tags: [goods, reading]\nitem: { category: goods, name: Gazette }\n",
        ),
        (
            "organizations/paper-kiosk.yaml",
            "tags: [business, newsstand]\nholdings: { gazette: 6 }\n\
             economy:\n  wallet: 50000\n  shop:\n    at: concourse\n    prices: { gazette: 150 }\n",
        ),
        (
            "people/mo.yaml",
            "tags: [vendor]\nlocation:\n  place: concourse\n\
             routine:\n  - { from: \"06:00\", place: concourse, label: kiosk }\n\
             \x20 - { from: \"18:00\", place: lane, label: home }\n\
             economy: { wallet: 1000 }\n\
             job:\n  employer: paper-kiosk\n  workplace: concourse\n  from: \"06:00\"\n  until: \"11:00\"\n\
             \x20 wage: 100\n  produces: { gazette: 3 }\n",
        ),
        (
            "people/lu.yaml",
            "tags: [traveller]\nlocation:\n  place: concourse\n\
             routine:\n  - { from: \"06:00\", place: concourse, label: wait }\n\
             \x20 - { from: \"18:00\", place: lane, label: home }\n\
             economy: { wallet: 5000 }\n",
        ),
    ];
    for (file, text) in files {
        std::fs::write(world.join(file), text).expect("writes");
    }
    world
}

fn scratch(name: &str) -> Scratch {
    mineworld_test_support::scratch!(empty name)
}

/// The authored shop of `organization` in `pack`: its place and its prices, read from the file.
fn authored_shop(pack: &Path, organization: &str) -> (String, BTreeMap<String, u64>) {
    let text = std::fs::read_to_string(
        pack.join("organizations")
            .join(format!("{organization}.yaml")),
    )
    .expect("the organization file reads");
    let value: Value = serde_saphyr::from_str(&text).expect("the organization file parses");
    let shop = &value["economy"]["shop"];
    let prices = shop["prices"]
        .as_object()
        .expect("prices")
        .iter()
        .map(|(item, price)| (item.clone(), price.as_u64().expect("a price")))
        .collect();
    (shop["at"].as_str().expect("at").to_owned(), prices)
}

/// The keys of a loaded pack by id, and its genesis facts.
fn loaded(pack: &Path) -> (BTreeMap<EntityId, String>, Vec<EventEnvelope>) {
    let read = WorldPack::read(pack).expect("the pack reads");
    let loaded = read.load(WorldTime::EPOCH).expect("it loads");
    let keys = loaded
        .ids()
        .iter()
        .map(|(key, id)| (*id, key.as_str().to_owned()))
        .collect();
    (keys, loaded.genesis().to_vec())
}

fn facts_of(tables: &Tables) -> Vec<EventEnvelope> {
    tables
        .facts
        .iter()
        .map(|(_, bytes)| {
            mineworld_persistence::format::decode(bytes, "fact").expect("a stored fact decodes")
        })
        .collect()
}

/// AC-b-1: the hall's shop opens at genesis with the authored kind at the authored price.
#[test]
fn the_hall_offers_the_paper() {
    let yard = scratch("newsstand-offers");
    for (pack, organization) in [
        (PathBuf::from(MARKET_TOWN), "station-news"),
        (kiosk_yard(&yard), "paper-kiosk"),
    ] {
        let (at, prices) = authored_shop(&pack, organization);
        let (keys, genesis) = loaded(&pack);
        let opened: Vec<BTreeMap<String, u64>> = genesis
            .iter()
            .filter_map(market_fact)
            .filter_map(|fact| match fact {
                MarketFact::ShopOpened { place, prices, .. } if keys[&place.entity_id()] == at => {
                    Some(
                        prices
                            .into_iter()
                            .map(|(item, price)| (keys[&item.entity_id()].clone(), price))
                            .collect(),
                    )
                }
                _ => None,
            })
            .collect();
        assert_eq!(opened, vec![prices], "{}: the shop at {at}", pack.display());
    }
}

/// AC-b-3: a town without economy opens no shop at all, so its hall sells nothing.
#[test]
fn no_shop_without_economy() {
    for world in ["social-cafe", "lakeside"] {
        let pack = Path::new(ROOT).join("worlds").join(world);
        let read = WorldPack::read_with(
            &pack,
            &mineworld_packages::PackRoots::new(
                vec![
                    Path::new(ROOT).join("entities"),
                    Path::new(ROOT).join("presentation/mineworld-default"),
                ],
                None,
            )
            .expect("the repository's roots"),
        )
        .expect("reads");
        let genesis = read
            .load(WorldTime::EPOCH)
            .expect("loads")
            .genesis()
            .to_vec();
        assert!(
            !genesis
                .iter()
                .any(|fact| fact.event_type().as_str() == "shop-opened"),
            "{world} opens no shop"
        );
    }
}

/// AC-b-2, AC-b-4 and AC-b-5 on the variant: a sale at the hall paired with the paper, the keeper paid,
/// and the market's 30-day conditions with the variant's own job holders.
#[test]
fn the_variant_sells_and_pays() {
    let yard = scratch("newsstand-variant");
    let pack = kiosk_yard(&yard);
    let save = fresh("newsstand-variant-save");
    run_pack(&pack, 1, 30, Some(&save));
    let town = Town::read(&pack);
    let facts = facts_of(&Tables::read(&save));
    let kiosk = town.id("paper-kiosk");
    let gazette = town.id("gazette");
    let sales = facts
        .iter()
        .filter_map(market_fact)
        .filter(|fact| {
            matches!(fact, MarketFact::ItemsTransferred { from, item, .. }
                if *from == kiosk && item.entity_id() == gazette)
        })
        .count();
    let wages = facts
        .iter()
        .filter_map(market_fact)
        .filter(|fact| matches!(fact, MarketFact::WageDue { employee, .. } if employee.entity_id() == town.id("mo")))
        .count();
    eprintln!("kiosk-yard 30 days: {sales} gazettes sold, {wages} wages due to mo");
    assert!(wages >= 1, "the keeper is paid");
    assert!(sales >= 1, "a gazette is sold at the concourse");
    market_lives(&town, &facts, 30);
}

/// AC-b-6: economy removed with its sections kept is refused by name, naming a carrier.
#[test]
fn economy_removed_refuses_the_sections() {
    let yard = scratch("newsstand-removed");
    let market = yard.join("market-town");
    copy_dir(Path::new(MARKET_TOWN), &market);
    let variant = kiosk_yard(&yard.join("variant"));
    for pack in [market, variant] {
        let manifest = pack.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("reads");
        let without = if text.contains("  - economy\n") {
            text.replacen("  - economy\n", "", 1)
        } else {
            text.replacen("inventory, economy, employment", "inventory, employment", 1)
        };
        assert_ne!(without, text, "economy was enabled");
        std::fs::write(&manifest, without).expect("writes");
        let output = mineworld(&["validate", pack.to_str().expect("printable")]);
        let said = stderr(&output);
        assert!(!output.status.success(), "{said}");
        assert!(said.contains("economy"), "names the section: {said}");
        assert!(said.contains(".yaml"), "names a carrier file: {said}");
    }
}

/// AC-b-7: no place, person or item id moves; the newsstand's organization is the last id.
#[test]
fn no_id_moves() {
    let yard = scratch("newsstand-ids");
    for (pack, organization) in [
        (PathBuf::from(MARKET_TOWN), "station-news"),
        (kiosk_yard(&yard), "paper-kiosk"),
    ] {
        let read = WorldPack::read(&pack).expect("reads");
        let loaded = read.load(WorldTime::EPOCH).expect("loads");
        let world = loaded.world().read();
        let mut expected = 1;
        for keys in [
            read.places().keys().collect::<Vec<_>>(),
            read.people().keys().collect(),
            read.items().keys().collect(),
        ] {
            for key in keys {
                assert_eq!(loaded.ids()[key].raw(), expected, "{key}");
                expected += 1;
            }
        }
        let highest = loaded
            .ids()
            .values()
            .filter(|id| {
                world
                    .entity(**id)
                    .is_some_and(|entity| entity.entity_type() == EntityType::Organization)
            })
            .max()
            .expect("organizations");
        let key = mineworld_contracts::EntityKey::new(organization).expect("a key");
        assert_eq!(
            loaded.ids()[&key],
            *highest,
            "{organization} is the last id"
        );
    }
}

/// A content check over any world: every job whose employer runs a shop is worked at that shop's place.
#[test]
fn the_newsstand_job_is_at_the_shop() {
    let yard = scratch("newsstand-job");
    for pack in [PathBuf::from(MARKET_TOWN), kiosk_yard(&yard)] {
        let mut checked = 0;
        for entry in std::fs::read_dir(pack.join("people")).expect("people/ lists") {
            let path = entry.expect("an entry").path();
            let value: Value =
                serde_saphyr::from_str(&std::fs::read_to_string(&path).expect("reads"))
                    .expect("parses");
            let Some(job) = value.get("job") else {
                continue;
            };
            let employer = job["employer"].as_str().expect("an employer");
            let file = pack.join("organizations").join(format!("{employer}.yaml"));
            let organization: Value =
                serde_saphyr::from_str(&std::fs::read_to_string(&file).expect("reads"))
                    .expect("parses");
            let Some(at) = organization["economy"]["shop"]["at"].as_str() else {
                continue;
            };
            assert_eq!(
                job["workplace"].as_str(),
                Some(at),
                "{}: works for {employer}, whose shop is at {at}",
                path.display()
            );
            checked += 1;
        }
        assert!(checked >= 1, "{}: a shop keeper is checked", pack.display());
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copied");
        }
    }
}
