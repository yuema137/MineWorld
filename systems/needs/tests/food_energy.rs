//! Food energy through the real owners (`ARC-370`; FE-a E1–E5 on the synthetic `FoodEnergy` world).
//!
//! ```text
//! pantry ─ shop     gus (big: 2 400 kcal/day)  food_energy 2 400   loaf 2, water 1
//!                   hal (2 000 kcal/day)       food_energy 1 500   loaf 3, water 2
//!                   ivy (no needs)                                 loaf 1
//!                   loaf  500 kcal, "1 loaf" 400 g;  water 0 kcal, "1 glass" 250 ml;  pebble (no needs)
//! ```
//!
//! Every expected number is a literal worked by hand from the layout (NE-2), never the pack's output.

mod support;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_kernel::{Emission, KernelError};
use mineworld_needs::{
    AuthoredNeeds, Hunger, INITIAL_ENERGY_OUT_OF_RANGE, MODEL_MISMATCH, NeedsSystem,
};
use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend};
use support::{Assembled, GENESIS, Kind, Town, accepted, assemble, compose, hour};

/// `who`'s disclosed `food_energy` block.
fn energy(town: &Town, who: &str) -> serde_json::Value {
    town.disclosed(who, who).expect("disclosed to oneself")["food_energy"].clone()
}

fn available(town: &Town, who: &str) -> u64 {
    energy(town, who)["available_kcal"]
        .as_u64()
        .expect("a number")
}

/// NE-2 through the real owners: D = C = 2 000, initial 1 500 → 6 h 1 000 → a 500 kcal loaf 1 500 →
/// 12:00 1 000; empty a day later; the 2 400 class starts full and drains at its own rate.
#[test]
fn the_reserve_drains_at_daily_use_and_a_meal_refills_it() {
    let mut town = Town::new(Kind::FoodEnergy);
    assert_eq!(available(&town, "hal"), 1_500);
    assert_eq!(available(&town, "gus"), 2_400);
    let start = energy(&town, "gus");
    assert_eq!(start["capacity_kcal"], 2_400);
    assert_eq!(start["use_kcal_per_world_day"], 2_400);
    assert_eq!(
        start["bar"],
        serde_json::json!({ "value": 1000, "maximum": 1000 })
    );
    assert_eq!(start["unit"], "kcal");
    assert_eq!(start["replaces_need"], "hunger");
    let hal = town.disclosed("hal", "hal").expect("disclosed");
    assert_eq!(hal["hunger_model"], "food-energy");
    assert_eq!(
        (hal["hunger"].as_u64(), hal["words"]["hunger"].as_str()),
        (Some(250), Some("ok"))
    );
    assert_eq!(hal["food_energy"]["bar"]["value"], 750);
    assert!(
        hal["rates"].get("hunger_per_hour").is_none()
            && hal["rates"].get("tiredness_per_hour").is_some(),
        "only tiredness rates in food energy: {}",
        hal["rates"]
    );

    town.pass_to(hour(6));
    assert_eq!(available(&town, "hal"), 1_000);
    assert_eq!(available(&town, "gus"), 1_800);
    assert!(accepted(&town.eat("hal", "loaf")));
    assert_eq!(available(&town, "hal"), 1_500);
    town.pass_to(hour(12));
    assert_eq!(available(&town, "hal"), 1_000);
    town.pass_to(hour(24 + 12));
    assert_eq!(available(&town, "hal"), 0, "empty after a day");
    let empty = town.disclosed("hal", "hal").expect("disclosed");
    assert_eq!(
        (empty["hunger"].as_u64(), empty["words"]["hunger"].as_str()),
        (Some(1_000), Some("famished"))
    );
    town.pass_to(hour(24 * 5));
    assert_eq!(
        available(&town, "hal"),
        0,
        "zero stays zero, no consequence"
    );
}

/// NE-3/NE-4: a meal at a full reserve is legal and capped; two at one instant both count; water is
/// consumed and changes no stored reserve; ivy (no needs) eats and nothing of `needs` changes.
#[test]
fn caps_same_instant_meals_and_zero_kcal_drinks() {
    let mut town = Town::new(Kind::FoodEnergy);
    let full = town.needs("gus").expect("needs");
    assert!(
        accepted(&town.eat("gus", "loaf")),
        "eating at full is legal"
    );
    assert_eq!(available(&town, "gus"), 2_400, "capped");
    let _ = full;
    town.pass_to(hour(12));
    assert_eq!(available(&town, "hal"), 500);
    assert!(accepted(&town.eat("hal", "loaf")));
    assert!(accepted(&town.eat("hal", "loaf")));
    assert_eq!(
        available(&town, "hal"),
        1_500,
        "both meals at one instant count"
    );
    let before = town.needs("hal");
    assert!(accepted(&town.drink_water("hal")));
    assert_eq!(
        town.needs("hal"),
        before,
        "a zero-kcal drink writes no energy fold"
    );
    assert!(accepted(&town.eat("ivy", "loaf")));
    assert!(town.needs("ivy").is_none());
    let observed = town.disclosed("hal", "hal").expect("disclosed");
    let water = observed["nourishment"]
        .as_array()
        .expect("a list")
        .iter()
        .find(|entry| entry["item"] == serde_json::json!(town.item("water")))
        .expect("water is still held")
        .clone();
    assert_eq!(water["energy_kcal"], 0);
    assert_eq!(water["hunger"], 0, "0 kcal motivates nobody");
    assert_eq!(
        water["serving"]["quantity"],
        serde_json::json!({ "unit": "ml", "amount": 250 })
    );
    let loaf = observed["kinds"]
        .as_array()
        .expect("a list")
        .iter()
        .find(|entry| entry["item"] == serde_json::json!(town.item("loaf")))
        .expect("every kind")
        .clone();
    assert_eq!(loaf["hunger"], 250, "⌈1 000·500/2 000⌉");
    let gus_loaf = town.disclosed("gus", "gus").expect("disclosed")["kinds"]
        .as_array()
        .expect("a list")
        .iter()
        .find(|entry| entry["item"] == serde_json::json!(town.item("loaf")))
        .expect("every kind")["hunger"]
        .clone();
    assert_eq!(gus_loaf, 209, "⌈1 000·500/2 400⌉ = ⌈208.3⌉");
}

/// NE-6: the same rate asleep and awake, no midnight refill, and observing changes nothing.
#[test]
fn sleep_and_observation_change_nothing_about_the_drain() {
    let mut asleep = Town::new(Kind::FoodEnergy);
    let mut awake = Town::new(Kind::FoodEnergy);
    asleep.pass_to(hour(1));
    awake.pass_to(hour(1));
    assert!(accepted(&asleep.sleep("hal", 360)));
    for h in 2..=7 {
        awake.pass_to(hour(h));
        let _ = awake.disclosed("hal", "hal");
    }
    asleep.pass_to(hour(7));
    assert_eq!(available(&asleep, "hal"), available(&awake, "hal"));
    // 1 500 − 2 000 × 7/24 = 916.67 → 916.
    assert_eq!(available(&asleep, "hal"), 916);
    let mut midnight = Town::new(Kind::FoodEnergy);
    midnight.pass_to(hour(22));
    assert!(accepted(&midnight.eat("gus", "loaf")));
    midnight.pass_to(hour(23));
    assert_eq!(
        available(&midnight, "gus"),
        600,
        "200 at 22:00, a loaf, an hour of 2 400/day"
    );
    midnight.pass_to(hour(25));
    assert_eq!(available(&midnight, "gus"), 400, "no refill at 00:00");
}

/// Holder-only disclosure: hal is told nothing about gus's reserve.
#[test]
fn another_person_cannot_see_the_reserve() {
    let town = Town::new(Kind::FoodEnergy);
    assert!(town.disclosed("hal", "gus").is_none());
    assert!(town.disclosed("gus", "hal").is_none());
}

/// The genesis facts of a world, with one more person or item section seeded for a subject that has
/// none in the layout.
fn with_extra(kind: Kind, subject: &str, yaml: &str) -> (Assembled, Vec<Emission>) {
    let assembled = assemble(kind, true);
    let mut facts = assembled.facts.clone();
    let keys = assembled
        .keys
        .iter()
        .map(|(key, id)| (EntityKey::new(key.as_str()).expect("a key"), *id))
        .collect();
    {
        let read = assembled.world.read();
        let seeding = Seeding::new(&read, &keys);
        let authored: AuthoredNeeds = serde_saphyr::from_str(yaml).expect("decodes");
        facts.extend(
            NeedsSystem::seed(&seeding, assembled.keys[subject], &authored).expect("seeded"),
        );
    }
    (assembled, facts)
}

/// E1 at genesis: the other model's form, or a reserve above the person's daily use, stops genesis as
/// the author's error, naming owner, entity key, code and field. gus's 2 400 (his class's use) is
/// accepted in the same world where ivy's 2 001 is refused.
#[test]
fn contradictory_genesis_is_refused_naming_owner_subject_code_and_field() {
    for (kind, subject, yaml, code, field) in [
        (Kind::FoodEnergy, "ivy", "{}", MODEL_MISMATCH, "hunger"),
        (
            Kind::FoodEnergy,
            "ivy",
            "food_energy: { initial_kcal: 2001 }",
            INITIAL_ENERGY_OUT_OF_RANGE,
            "food_energy.initial_kcal",
        ),
        (
            Kind::FoodEnergy,
            "pebble",
            "nourishment: { hunger: 5 }",
            MODEL_MISMATCH,
            "nourishment.hunger",
        ),
        (
            Kind::FlatAndCafe,
            "cy",
            "food_energy: { initial_kcal: 10 }",
            MODEL_MISMATCH,
            "food_energy.initial_kcal",
        ),
        (
            Kind::FlatAndCafe,
            "coffee",
            "nourishment: { energy_kcal: 5, serving: { label: cup, quantity: { unit: ml, amount: 200 } } }",
            MODEL_MISMATCH,
            "nourishment.energy_kcal",
        ),
    ] {
        let (mut assembled, facts) = with_extra(kind, subject, yaml);
        match assembled.world.genesis(GENESIS, facts) {
            Err(error @ KernelError::InvalidGenesisContent { .. }) => {
                let KernelError::InvalidGenesisContent {
                    system,
                    subject: key,
                    code: refused,
                    field: named,
                    ..
                } = &error
                else {
                    unreachable!()
                };
                assert_eq!(system.as_str(), "needs");
                assert_eq!(key.as_str(), subject);
                assert_eq!(*refused, code, "{yaml}");
                assert_eq!(&**named, field, "{yaml}");
                assert!(error.to_string().contains(subject), "{error}");
            }
            other => panic!("{yaml}: expected InvalidGenesisContent, got {other:?}"),
        }
    }
    let town = Town::new(Kind::FoodEnergy);
    assert!(matches!(
        town.needs("gus").map(|needs| needs.hunger()),
        Some(Hunger::FoodEnergy(_))
    ));
}

/// E4/NE-5: a file-backed save closed after a meal and mid-depletion, and reopened, continues exactly
/// as the world that never stopped; nothing is regranted and no wall time drains it.
#[test]
fn a_food_energy_world_resumed_is_the_world_that_never_stopped() {
    fn create(scratch: &mineworld_test_support::Scratch) -> (PersistentWorld, support::Keys) {
        let Assembled {
            world, keys, facts, ..
        } = assemble(Kind::FoodEnergy, true);
        let backend = SqliteBackend::create(scratch, Durability::ProcessCrash).expect("creates");
        let (persisted, _) = PersistentWorld::create(
            Box::new(backend),
            world,
            Creation {
                instance: 0x5eed_0000_0000_0000_0000_0000_00fe_000a,
                pack: "food-energy-test".to_owned(),
                at: GENESIS,
                facts,
            },
        )
        .expect("a save is created");
        (persisted, keys)
    }
    fn before(persisted: &mut PersistentWorld, keys: &support::Keys) {
        let _ = persisted.advance_to(hour(6)).expect("advances");
        support::request_eat(persisted, keys, "hal", "loaf", 1);
        support::request_sleep(persisted, keys, "gus", 240, 2);
        let _ = persisted
            .advance_to(WorldTime::from_seconds(7 * 3_600 + 1_234))
            .expect("advances");
    }
    fn after(persisted: &mut PersistentWorld, keys: &support::Keys) -> Vec<u64> {
        let mut seen = Vec::new();
        for h in [8, 13, 30] {
            let _ = persisted.advance_to(hour(h)).expect("advances");
            for who in ["gus", "hal"] {
                seen.push(support::available_in(persisted.world(), keys[who], hour(h)));
            }
            if h == 13 {
                support::request_eat(persisted, keys, "gus", "loaf", 3);
            }
        }
        seen
    }

    let straight = mineworld_test_support::scratch!(empty "food-energy-straight");
    let (mut uninterrupted, keys) = create(&straight);
    before(&mut uninterrupted, &keys);
    let expected = after(&mut uninterrupted, &keys);

    let stopped = mineworld_test_support::scratch!(empty "food-energy-stopped");
    let (mut first, keys) = create(&stopped);
    before(&mut first, &keys);
    drop(first);
    let backend = SqliteBackend::open(&stopped, Durability::ProcessCrash).expect("opens");
    let (mut resumed, _) =
        PersistentWorld::resume(Box::new(backend), compose(true).0).expect("resumes");
    let got = after(&mut resumed, &keys);
    // By hand: hal 1 500 − 500 (6 h) + 500 = 1 500 at 06:00; 08:00 1 333; 13:00 916; 30:00 0.
    // gus 2 400 − 600 = 1 800 at 06:00 (asleep 06:00–10:00, same rate); 08:00 1 600; 13:00 1 100,
    // a loaf → 1 600; 30:00 (17 h later) 1 600 − 1 700 → 0.
    assert_eq!(got, vec![1_600, 1_333, 1_100, 916, 0, 0]);
    assert_eq!(got, expected, "the uninterrupted world's reserves");
}

/// E5: `restore: { hunger: [] }` in the food-energy model is legal and means what it says — a meal is
/// consumed and the reserve only drains.
#[test]
fn a_world_that_restores_no_hunger_only_drains_the_reserve() {
    let Assembled {
        mut world,
        keys,
        mut facts,
        ..
    } = assemble(Kind::FoodEnergy, true);
    let configured = mineworld_contracts::EventTypeId::from_static("needs-interactions-configured");
    // The configured fact keeps its place, before the sections' facts, as the loader seeds it.
    let at = facts
        .iter()
        .position(|fact| *fact.event_type() == configured)
        .expect("configured");
    facts.remove(at);
    facts.insert(
        at,
        support::configured(
            r#"{ "parameters": [ { "hunger_model": "food-energy", "energy_kcal_per_day": 2000,
              "restore": { "hunger": [], "tiredness": ["sleeping"] } },
              { "actor": "big", "energy_kcal_per_day": 2400 } ] }"#,
            support::BIG_CLASSES,
        ),
    );
    world.genesis(GENESIS, facts).expect("begins");
    let hal = keys["hal"];
    let _ = world.advance_to(hour(6)).expect("advances");
    let before = support::available_in(&world, hal, hour(6));
    let loaf =
        mineworld_contracts::ItemId::new(keys["loaf"], mineworld_contracts::EntityType::Item)
            .expect("an item");
    let intent = mineworld_contracts::ActionIntent::new(
        mineworld_contracts::ActionId::from_raw(1),
        hal,
        support::record(&mineworld_consumption::Eat::new(loaf)),
        hour(6),
    );
    let done = world.dispatch(&intent, hour(6)).expect("answers");
    assert!(accepted(done.result()), "the meal is still eaten");
    assert_eq!(before, 1_000);
    assert_eq!(
        support::available_in(&world, hal, hour(6)),
        1_000,
        "no refill"
    );
}
