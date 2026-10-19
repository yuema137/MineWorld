//! T16 and T17 (WE-a design §6.2, §9.2): economy's price modifiers, through the real World Pack loader
//! with calendar and happenings installed. While a configured happening runs, the store's listing and
//! `buy` charge the adjusted price; `prices-adjusted` and `prices-restored` are caused by the
//! happening's own facts; an unconfigured economy states nothing new; and a modifier left behind when
//! happenings is removed is refused by name.

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Causation, EntityId, EventEnvelope, ItemId,
    WorldTime,
};
use mineworld_economy::{Buy, Listing, MoneyTransferred, PricesAdjusted, PricesRestored};
use mineworld_happenings::{HappeningBegan, HappeningEnded};
use mineworld_kernel::World;
use mineworld_presence::{PerceptionProvider, observe};
use mineworld_test_support::Scratch;
use mineworld_worldpack::{AssembledWorld, WorldPack};
use serde::Deserialize;

const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;

const CALENDAR: &str =
    "epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 32.7157\nlongitude: -117.1611\n";

/// A Saturday market on the street, 08:00–14:00 (2026-10-10 is day 2).
const HAPPENINGS: &str = "seed: 7
kinds:
  - { id: market-day, places: [street], hours: \"08:00-14:00\" }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
";

/// 10 % off at the store on market days — and a modifier for a kind this world never has, which never
/// matches (R-WE-a-4).
const MODIFIERS: &str = "price_modifiers:
  - { during: market-day, shop: store, permille: 900 }
  - { during: book-fair, shop: store, permille: 500 }
";

/// A scratch World Pack — a store on a street, the corner store selling apples (100) and bread (333),
/// Alice in the store with 10 000 — enabling `systems` and configuring `configure`, each configure file
/// as given; removed when the test ends (DEP-29).
fn pack(id: &str, systems: &[&str], configure: &[(&str, &str)]) -> Scratch {
    let scratch = mineworld_test_support::scratch!(empty id);
    let root = scratch.path();
    for directory in ["people", "places", "configure", "items", "organizations"] {
        std::fs::create_dir_all(root.join(directory)).expect("writable");
    }
    let write = |file: &str, text: &str| std::fs::write(root.join(file), text).expect("writable");
    let list =
        |items: Vec<&str>| -> String { items.iter().map(|item| format!("  - {item}\n")).collect() };
    let configured: Vec<&str> = configure.iter().map(|(key, _)| *key).collect();
    write(
        "world.yaml",
        &format!(
            "world:\n  id: {id}\n  name: A Market Street\nsystems:\n{}configure:\n{}places:\n  - store\n  \
             - street\npopulation:\n  - alice\nitems:\n  - apple\n  - bread\norganizations:\n  - \
             corner-store\n",
            list(systems.to_vec()),
            list(configured)
        ),
    );
    write(
        "places/store.yaml",
        "tags: [store]\npassages:\n  - to: street\n",
    );
    write("places/street.yaml", "tags: [street]\n");
    write(
        "items/apple.yaml",
        "item: { category: food, name: Apple }\n",
    );
    write(
        "items/bread.yaml",
        "item: { category: food, name: Bread }\n",
    );
    write(
        "organizations/corner-store.yaml",
        "holdings: { apple: 20, bread: 10 }\neconomy:\n  wallet: 1000\n  shop: { at: store, prices: \
         { apple: 100, bread: 333 } }\n",
    );
    write(
        "people/alice.yaml",
        "location:\n  place: store\neconomy: { wallet: 10000 }\n",
    );
    for (key, text) in configure {
        write(&format!("configure/{key}.yaml"), text);
    }
    scratch
}

const SYSTEMS: [&str; 7] = [
    "presence",
    "movement",
    "item",
    "inventory",
    "economy",
    "calendar",
    "happenings",
];

/// A world the loader assembled, begun at instant 0.
struct Loaded {
    world: World,
    ids: std::collections::BTreeMap<mineworld_contracts::EntityKey, EntityId>,
    genesis: Vec<EventEnvelope>,
    providers: Vec<Box<dyn PerceptionProvider>>,
}

fn load(scratch: &Scratch) -> Loaded {
    let AssembledWorld {
        mut world,
        ids,
        facts,
        providers,
    } = WorldPack::read(scratch.path())
        .expect("reads")
        .assemble()
        .expect("assembles");
    let genesis = world
        .genesis(WorldTime::from_seconds(0), facts)
        .expect("genesis");
    Loaded {
        world,
        ids,
        genesis,
        providers,
    }
}

fn refusal(scratch: &Scratch) -> String {
    WorldPack::read(scratch.path())
        .and_then(|pack| pack.assemble().map(|_| ()))
        .expect_err("refused")
        .to_string()
}

fn id(loaded: &Loaded, key: &str) -> EntityId {
    loaded.ids[&mineworld_contracts::EntityKey::new(key).expect("a key")]
}

fn item(loaded: &Loaded, key: &str) -> ItemId {
    ItemId::new(id(loaded, key), mineworld_contracts::EntityType::Item).expect("an item")
}

/// The store's listing as Alice perceives it: (item, price) in item order.
fn listed(world: &World, providers: &[&dyn PerceptionProvider], alice: EntityId) -> Vec<u64> {
    let seen = observe(world, alice, WorldTime::from_seconds(0), providers);
    let record = seen
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .find(|record| record.component_type().as_str() == "shop")
        .expect("the store's listing");
    let listing: Listing = serde_json::from_value(record.payload().clone()).expect("a listing");
    listing.listed().iter().map(|line| line.price()).collect()
}

/// Alice buys one `item` at `at`; returns what she paid.
fn buy(world: &mut World, alice: EntityId, item: ItemId, at: i64, n: u64) -> u64 {
    let intent = ActionIntent::new(
        ActionId::from_raw(n),
        alice,
        ActionRecord::new::<Buy>(serde_json::to_vec(&Buy::new(item)).expect("encodes")),
        WorldTime::from_seconds(at),
    );
    let done = world
        .dispatch(&intent, WorldTime::from_seconds(at))
        .expect("dispatches");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    of::<MoneyTransferred>(done.events())[0].1.amount()
}

fn of<E: mineworld_contracts::Event + for<'de> Deserialize<'de>>(
    facts: &[EventEnvelope],
) -> Vec<(&EventEnvelope, E)> {
    facts
        .iter()
        .filter(|fact| *fact.event_type() == E::EVENT_TYPE)
        .map(|fact| {
            let bytes = fact.payload().payload_for::<E>().expect("its type");
            (fact, serde_json::from_slice(bytes).expect("decodes"))
        })
        .collect()
}

/// T16: before, during and after Saturday's market the store charges 100 / 333, then 90 / 300
/// (333 × 0.9 = 299.7, rounded half up), then 100 / 333 again — in its listing and at the till — and
/// the adjustment and restoration are caused by the happening's own facts.
#[test]
fn a_market_day_lowers_the_stores_prices_while_it_runs() {
    let scratch = pack(
        "market-street",
        &SYSTEMS,
        &[
            ("calendar", CALENDAR),
            ("happenings", HAPPENINGS),
            ("economy", MODIFIERS),
        ],
    );
    let mut loaded = load(&scratch);
    let alice = id(&loaded, "alice");
    let (apple, bread) = (item(&loaded, "apple"), item(&loaded, "bread"));
    let providers: Vec<&dyn PerceptionProvider> = loaded
        .providers
        .iter()
        .map(|provider| &**provider)
        .collect();
    let mut facts = loaded.genesis.clone();

    let before = 2 * DAY + 7 * HOUR;
    facts.extend(
        loaded
            .world
            .advance_to(WorldTime::from_seconds(before))
            .expect("advances")
            .into_events(),
    );
    assert_eq!(listed(&loaded.world, &providers, alice), [100, 333]);
    assert_eq!(buy(&mut loaded.world, alice, apple, before, 1), 100);

    // In the instant the market begins, its adjustment is already in force.
    let opening = 2 * DAY + 8 * HOUR;
    facts.extend(
        loaded
            .world
            .advance_to(WorldTime::from_seconds(opening))
            .expect("advances")
            .into_events(),
    );
    assert_eq!(buy(&mut loaded.world, alice, apple, opening, 5), 90);

    let during = 2 * DAY + 9 * HOUR;
    facts.extend(
        loaded
            .world
            .advance_to(WorldTime::from_seconds(during))
            .expect("advances")
            .into_events(),
    );
    assert_eq!(listed(&loaded.world, &providers, alice), [90, 300]);
    assert_eq!(buy(&mut loaded.world, alice, apple, during, 2), 90);
    assert_eq!(buy(&mut loaded.world, alice, bread, during, 3), 300);

    let after = 2 * DAY + 15 * HOUR;
    facts.extend(
        loaded
            .world
            .advance_to(WorldTime::from_seconds(after))
            .expect("advances")
            .into_events(),
    );
    assert_eq!(listed(&loaded.world, &providers, alice), [100, 333]);
    assert_eq!(buy(&mut loaded.world, alice, bread, after, 4), 333);

    let began = of::<HappeningBegan>(&facts);
    let ended = of::<HappeningEnded>(&facts);
    let adjusted = of::<PricesAdjusted>(&facts);
    let restored = of::<PricesRestored>(&facts);
    assert_eq!((began.len(), adjusted.len()), (1, 1));
    assert_eq!((ended.len(), restored.len()), (1, 1));
    assert_eq!(
        *adjusted[0].0.caused_by(),
        Causation::Event(began[0].0.id())
    );
    assert_eq!(
        *restored[0].0.caused_by(),
        Causation::Event(ended[0].0.id())
    );
    assert_eq!(adjusted[0].1.happening(), began[0].1.happening());
    assert_eq!(adjusted[0].1.permille(), 900);
    assert_eq!(adjusted[0].0.at(), began[0].0.at());
    let store = id(&loaded, "store");
    assert_eq!(adjusted[0].1.place().entity_id(), store);
    assert_eq!(
        adjusted[0].0.place().map(|place| place.entity_id()),
        Some(store)
    );
}

/// T16, the unconfigured half: the same world without `configure/economy.yaml` states no economy
/// configuration and no price fact through the market, and charges the authored price during it.
#[test]
fn an_unconfigured_economy_states_nothing_new() {
    let scratch = pack(
        "plain-market-street",
        &SYSTEMS,
        &[("calendar", CALENDAR), ("happenings", HAPPENINGS)],
    );
    let mut loaded = load(&scratch);
    let alice = id(&loaded, "alice");
    let apple = item(&loaded, "apple");
    let during = 2 * DAY + 9 * HOUR;
    let mut facts = loaded.genesis.clone();
    facts.extend(
        loaded
            .world
            .advance_to(WorldTime::from_seconds(during))
            .expect("advances")
            .into_events(),
    );
    assert_eq!(of::<HappeningBegan>(&facts).len(), 1, "the market is on");
    assert_eq!(
        buy(&mut loaded.world, alice, apple, during, 1),
        100,
        "the authored price"
    );
    facts.extend(
        loaded
            .world
            .advance_to(WorldTime::from_seconds(3 * DAY))
            .expect("advances")
            .into_events(),
    );
    for kind in ["economy-configured", "prices-adjusted", "prices-restored"] {
        assert!(
            facts.iter().all(|fact| fact.event_type().as_str() != kind),
            "{kind}"
        );
    }
}

/// T17: modifiers left behind when happenings is removed are refused, naming happenings; a bad modifier
/// is refused naming the file, its line and column, and the key.
#[test]
fn modifiers_without_happenings_are_refused_by_name() {
    let orphan = pack(
        "orphan-market-street",
        &["presence", "movement", "item", "inventory", "economy"],
        &[("economy", MODIFIERS)],
    );
    let refused = refusal(&orphan);
    eprintln!("REFUSAL {refused}");
    assert!(
        refused.contains("happenings"),
        "names happenings: {refused}"
    );
    assert!(refused.contains("economy"), "names economy: {refused}");

    let file = std::path::Path::new("configure")
        .join("economy.yaml")
        .to_string_lossy()
        .into_owned();
    for (id, text, says) in [
        (
            "steep-market-street",
            MODIFIERS.replace("permille: 900", "permille: 20000"),
            "permille",
        ),
        (
            "typo-market-street",
            MODIFIERS.replace("permille:", "per_mille:"),
            "per_mille",
        ),
    ] {
        let bad = pack(
            id,
            &SYSTEMS,
            &[
                ("calendar", CALENDAR),
                ("happenings", HAPPENINGS),
                ("economy", &text),
            ],
        );
        let refused = refusal(&bad);
        eprintln!("REFUSAL {refused}");
        assert!(refused.contains(&file), "names the file: {refused}");
        assert!(
            refused.contains("line") && refused.contains("column"),
            "{refused}"
        );
        assert!(refused.contains(says), "names {says}: {refused}");
    }
    let twice = pack(
        "twice-market-street",
        &SYSTEMS,
        &[
            ("calendar", CALENDAR),
            ("happenings", HAPPENINGS),
            (
                "economy",
                &format!("{MODIFIERS}  - {{ during: market-day, shop: store, permille: 800 }}\n"),
            ),
        ],
    );
    let refused = refusal(&twice);
    assert!(
        refused.contains("price_modifiers[2]: 'market-day' at 'store' is configured twice"),
        "{refused}"
    );
}
