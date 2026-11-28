//! A shop's sales hours as authored, seeded, saved and resumed (`pr-nightlife.md` N1-C1; N1-I2, N1-I4,
//! N1-I8), through the real World Pack loader and a real SQLite save. Oracles are hand-computed
//! instants of the authored intervals, written here.
//!
//! The scratch town has five shops on one street, each run by its own organization:
//!
//! ```text
//! always    no hours                                   open at every instant
//! shut      hours: []                                  closed at every instant
//! day       06:00–23:00                                open 06:00:00 … 22:59:59
//! night     22:00–06:00                                open 22:00:00 … 05:59:59, past midnight
//! split     08:00–12:00 and 14:00–18:00                a world's own content, no code change (F14)
//! ```

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityKey, EntityType, PlaceId, WorldTime,
};
use mineworld_economy::{Buy, Shop};
use mineworld_kernel::World;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision, format,
};
use mineworld_test_support::Scratch;
use mineworld_worldpack::{AssembledWorld, WorldPack};

const DAY: i64 = 86_400;
const HOUR: i64 = 3_600;
const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_00e1_0001;

/// The shops: (place and organization key stem, `hours:` line or none).
const SHOPS: [(&str, Option<&str>); 5] = [
    ("always", None),
    ("shut", Some("[]")),
    ("day", Some("[{ from: \"06:00\", until: \"23:00\" }]")),
    ("night", Some("[{ from: \"22:00\", until: \"06:00\" }]")),
    (
        "split",
        Some("[{ from: \"08:00\", until: \"12:00\" }, { from: \"14:00\", until: \"18:00\" }]"),
    ),
];

/// A scratch World Pack with [`SHOPS`]; `hours(stem)` overrides a shop's `hours:` line (`None`: none).
fn pack(id: &str, hours: impl Fn(&str) -> Option<String>) -> Scratch {
    let scratch = mineworld_test_support::scratch!(empty id);
    let root = scratch.path();
    for directory in ["people", "places", "configure", "items", "organizations"] {
        std::fs::create_dir_all(root.join(directory)).expect("writable");
    }
    let write = |file: &str, text: &str| std::fs::write(root.join(file), text).expect("writable");
    let places: String = SHOPS
        .iter()
        .map(|(stem, _)| format!("  - {stem}\n"))
        .collect();
    let organizations: String = SHOPS
        .iter()
        .map(|(stem, _)| format!("  - {stem}-co\n"))
        .collect();
    write(
        "world.yaml",
        &format!(
            "world:\n  id: {id}\n  name: Shop Hours Street\nsystems:\n  - presence\n  - movement\n  \
             - item\n  - inventory\n  - economy\nconfigure:\n  - economy\nplaces:\n  - street\n\
             {places}population:\n  - alice\nitems:\n  - bread\norganizations:\n{organizations}"
        ),
    );
    write(
        "configure/economy.yaml",
        "currency: { name: coin, symbol: c, exponent: 0, symbol_at: after }\n",
    );
    write(
        "items/bread.yaml",
        "item: { category: food, name: Bread }\n",
    );
    let doors: String = SHOPS
        .iter()
        .map(|(stem, _)| format!("  - to: {stem}\n"))
        .collect();
    write(
        "places/street.yaml",
        &format!("tags: [street]\npassages:\n{doors}"),
    );
    for (stem, _) in SHOPS {
        write(&format!("places/{stem}.yaml"), "tags: [shop]\n");
        let line = hours(stem).map_or_else(String::new, |hours| format!("\n    hours: {hours}"));
        write(
            &format!("organizations/{stem}-co.yaml"),
            &format!(
                "holdings: {{ bread: 50 }}\neconomy:\n  wallet: 1000\n  shop:\n    at: {stem}\n    \
                 prices: {{ bread: 100 }}{line}\n"
            ),
        );
    }
    write(
        "people/alice.yaml",
        "location:\n  place: day\neconomy: { wallet: 100000 }\n",
    );
    scratch
}

/// The pack as [`SHOPS`] authors it.
fn authored(stem: &str) -> Option<String> {
    SHOPS
        .iter()
        .find(|(name, _)| *name == stem)
        .and_then(|(_, hours)| hours.map(str::to_owned))
}

fn key(text: &str) -> EntityKey {
    EntityKey::new(text).expect("a key")
}

/// The shop in place `stem`.
fn shop(
    world: &World,
    ids: &std::collections::BTreeMap<EntityKey, mineworld_contracts::EntityId>,
    stem: &str,
) -> Shop {
    world
        .read()
        .component::<Shop>(ids[&key(stem)])
        .cloned()
        .expect("a shop")
}

fn at(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// The day's instants every shop is judged at, and whether each shop is open then: hand-computed
/// from the table in the module's header. Day 1 (instant 86 400 …) is used for the instants past
/// midnight so that wrapping is judged on a later day too.
const JUDGED: [(i64, [bool; 5]); 9] = [
    // always, shut, day, night, split
    (5 * HOUR + 59 * 60 + 59, [true, false, false, true, false]),
    (6 * HOUR, [true, false, true, false, false]),
    (8 * HOUR, [true, false, true, false, true]),
    (12 * HOUR, [true, false, true, false, false]),
    (14 * HOUR, [true, false, true, false, true]),
    (18 * HOUR, [true, false, true, false, false]),
    (22 * HOUR, [true, false, true, true, false]),
    (23 * HOUR, [true, false, false, true, false]),
    (DAY, [true, false, false, true, false]),
];

/// N1-C1, F14: authored hours reach each shop through `shop-opened`, and each is judged at the
/// hand-computed instants; the split shop is a world's own content, loaded with no code change.
#[test]
fn authored_hours_reach_every_shop_and_judge_its_day() {
    let scratch = pack("hours-judged", authored);
    let AssembledWorld {
        mut world,
        ids,
        facts,
        ..
    } = WorldPack::read(scratch.path())
        .expect("reads")
        .assemble()
        .expect("assembles");
    world.genesis(at(0), facts).expect("genesis");
    for (seconds, expected) in JUDGED {
        for ((stem, _), open) in SHOPS.iter().zip(expected) {
            assert_eq!(
                shop(&world, &ids, stem).open_at(at(seconds)),
                open,
                "{stem} at {seconds} s"
            );
            assert_eq!(
                shop(&world, &ids, stem).open_at(at(seconds + 7 * DAY)),
                open,
                "{stem} a week later"
            );
        }
    }
    assert!(shop(&world, &ids, "always").hours().is_none());
}

/// N1-C1: every malformed form stops the load, naming the shop's file and, for a broken rule, the
/// interval's index; never a shop that is silently always open.
#[test]
fn malformed_hours_stop_the_load_naming_the_file_and_the_interval() {
    let nine = format!(
        "[{}]",
        (0..9)
            .map(|hour| format!("{{ from: \"{hour:02}:00\", until: \"{hour:02}:30\" }}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let cases: [(&str, String, &str); 5] = [
        (
            "equal",
            "[{ from: \"08:00\", until: \"12:00\" }, { from: \"09:00\", until: \"09:00\" }]"
                .to_owned(),
            "interval 1 opens and closes at 09:00",
        ),
        (
            "overlap",
            "[{ from: \"22:00\", until: \"06:00\" }, { from: \"05:00\", until: \"07:00\" }]"
                .to_owned(),
            "intervals 0 and 1 overlap",
        ),
        ("ninth", nine, "at most 8 intervals, but these have 9"),
        (
            "malformed",
            "[{ from: \"7:00\", until: \"12:00\" }]".to_owned(),
            "\"7:00\"",
        ),
        (
            "unknown",
            "[{ from: \"07:00\", until: \"12:00\", staffed: true }]".to_owned(),
            "staffed",
        ),
    ];
    for (name, hours, says) in cases {
        let scratch = pack(&format!("hours-{name}"), |stem| {
            (stem == "day")
                .then(|| hours.clone())
                .or_else(|| authored(stem))
        });
        let refusal = WorldPack::read(scratch.path())
            .and_then(|pack| pack.assemble().map(|_| ()))
            .expect_err("refused")
            .to_string();
        assert!(
            refusal.contains("day-co.yaml") && refusal.contains(says),
            "{name}: {refusal}"
        );
    }
}

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(scratch.path(), Durability::ProcessCrash).expect("opens"))
}

/// A save of `pack` created at instant 0 in `save`, dropped; returns the saved shops.
fn create(pack: &Scratch, save: &Scratch) -> Vec<Shop> {
    let AssembledWorld {
        world, ids, facts, ..
    } = WorldPack::read(pack.path())
        .expect("reads")
        .assemble()
        .expect("assembles");
    let backend = SqliteBackend::create(save.path(), Durability::ProcessCrash).expect("creates");
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "economy-hours-test".to_owned(),
            at: at(0),
            facts,
        },
    )
    .expect("a save is created");
    SHOPS
        .iter()
        .map(|(stem, _)| shop(persisted.world(), &ids, stem))
        .collect()
}

/// The save in `save` resumed into `pack`'s composition, after the drift check every host runs
/// (`ARC-61` item 7): the resumed world and its ids.
fn resume(
    pack: &Scratch,
    save: &Scratch,
) -> (
    PersistentWorld,
    std::collections::BTreeMap<EntityKey, mineworld_contracts::EntityId>,
) {
    let read = WorldPack::read(pack.path()).expect("reads");
    let backend = SqliteBackend::open(save.path(), Durability::ProcessCrash).expect("opens");
    let genesis: Vec<_> = backend
        .facts_of(WorldRevision::GENESIS)
        .expect("genesis rows")
        .iter()
        .map(|row| format::decode(&row.bytes, "fact").expect("a fact"))
        .collect();
    read.check_configuration(&genesis)
        .expect("hours are content, not configuration: no drift");
    let ids = read.assemble().expect("assembles").ids;
    let composed = read.compose().expect("composes").world;
    let (resumed, _) = PersistentWorld::resume(open(save), composed).expect("resumes");
    (resumed, ids)
}

/// N1-C1: save → reopen gives every shop exactly its saved terms, hours included.
#[test]
fn hours_are_the_same_after_save_and_reopen() {
    let town = pack("hours-saved", authored);
    let save = mineworld_test_support::scratch!(empty "hours-saved-save");
    let saved = create(&town, &save);
    let (mut resumed, ids) = resume(&town, &save);
    let reopened: Vec<Shop> = SHOPS
        .iter()
        .map(|(stem, _)| shop(resumed.world(), &ids, stem))
        .collect();
    assert_eq!(reopened, saved);
    assert!(reopened[2].hours().is_some() && reopened[0].hours().is_none());
    // The reopened save judges its hours: the `day` shop, closed at 23:00, refuses at 23:30.
    assert_eq!(
        buy_at_half_past_eleven(&mut resumed, &ids),
        ActionResult::Rejected(mineworld_contracts::Rejection::System {
            code: mineworld_economy::code::SHOP_CLOSED,
            detail: None
        })
    );
}

/// N1-I8 (`ARC-61` item 7 applied): hours are authored content, never a configuration fact, so a save
/// created before its pack wrote hours is **not** drift. It resumes and keeps exactly its genesis
/// terms: every shop always open, and a buy at 23:30 in the shop the pack now closes at 23:00 sells.
/// A save written before N1 encodes `shop-opened` and `shop` with no `hours` key, as this one does.
#[test]
fn a_save_from_before_hours_resumes_with_its_shops_always_open() {
    let before = pack("hours-before", |_| None);
    let save = mineworld_test_support::scratch!(empty "hours-before-save");
    let saved = create(&before, &save);
    assert!(saved.iter().all(|shop| shop.hours().is_none()));

    let after = pack("hours-after", authored);
    let (mut resumed, ids) = resume(&after, &save);
    let shops: Vec<Shop> = SHOPS
        .iter()
        .map(|(stem, _)| shop(resumed.world(), &ids, stem))
        .collect();
    assert_eq!(shops, saved, "the save keeps its genesis terms");

    // Alice stands in the `day` shop, which the pack now closes at 23:00: the save's shop still sells.
    let day = PlaceId::new(ids[&key("day")], EntityType::Place).expect("a place");
    assert!(
        resumed
            .world()
            .read()
            .component::<Shop>(day.entity_id())
            .is_some_and(|shop| shop.open_at(at(23 * HOUR + 30 * 60))),
        "the pre-hours save's day shop is still open at 23:30"
    );
    let sold = buy_at_half_past_eleven(&mut resumed, &ids);
    assert!(matches!(sold, ActionResult::Accepted { .. }), "{sold:?}");
}

/// Alice, standing in the `day` shop, buys one bread at 23:30 of day 0; the result.
fn buy_at_half_past_eleven(
    world: &mut PersistentWorld,
    ids: &std::collections::BTreeMap<EntityKey, mineworld_contracts::EntityId>,
) -> ActionResult {
    let late = at(23 * HOUR + 30 * 60);
    let bread =
        mineworld_contracts::ItemId::new(ids[&key("bread")], EntityType::Item).expect("an item");
    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        ids[&key("alice")],
        ActionRecord::new::<Buy>(serde_json::to_vec(&Buy::new(bread, 100)).expect("encodes")),
        late,
    );
    world
        .dispatch(&intent, late)
        .expect("dispatches")
        .result()
        .clone()
}
