//! Food energy through the real binary and the real loader (`ARC-370`; PR FE-a E1, E3, E4, E5).
//!
//! Scratch copies of `worlds/market-town`, switched to the food-energy model by `fixture/needs.rs`
//! (synthetic kcal and servings, never retail or nutrition claims); the reference world is not edited.
//!
//! ```text
//! E3  US-dollar copy, 2 000 kcal/day; "Shell" exponent-0 copy, 2 400 kcal/day   `run --days 2`: people
//!     buy (every sale atomic) and eat; no needs fact after genesis
//! E4  the same copy run 1 day, then resumed to day 2                            the same history as 2 days
//!                                                                                straight
//! E1  configure/needs.yaml contradictions                                        refused by the loader,
//!                                                                                naming file, code and entry
//! E5  the unitless reference world                                               still loads, unitless
//! ```

mod headless;
mod market;
#[path = "fixture/needs.rs"]
mod needs_fixture;

use std::path::Path;

use headless::{Tables, fresh, mineworld, stderr, stdout};
use market::{MARKET_TOWN, run_pack, sales_are_atomic};
use mineworld_contracts::EventEnvelope;
use mineworld_packages::PackRoots;
use mineworld_worldpack::WorldPack;
use needs_fixture::{FoodEnergyCopy, food_energy};

const DOLLARS: FoodEnergyCopy = FoodEnergyCopy {
    daily_kcal: 2_000,
    initial_kcal: 1_500,
    whole_currency: false,
};

const SHELLS: FoodEnergyCopy = FoodEnergyCopy {
    daily_kcal: 2_400,
    initial_kcal: 2_400,
    whole_currency: true,
};

fn facts(save: &Path) -> Vec<EventEnvelope> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| mineworld_persistence::format::decode(bytes, "fact").expect("a fact"))
        .collect()
}

fn count(facts: &[EventEnvelope], slug: &str) -> usize {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == slug)
        .count()
}

/// The copy read as `mineworld` reads it, with the repository's pack roots (`ARC-77`).
fn read(pack: &Path) -> Result<WorldPack, mineworld_worldpack::PackError> {
    let repository = Path::new(MARKET_TOWN).join("../..");
    let roots = PackRoots::new(
        vec![
            repository.join("entities"),
            repository.join("presentation/mineworld-default"),
        ],
        None,
    )
    .expect("the roots exist");
    WorldPack::read_with(pack.to_path_buf(), &roots)
}

/// E3: both food-energy copies — two currencies, two daily uses — run two days through the real
/// controllers: people buy and eat, every sale is atomic (price, item and payment move together), the
/// pack states only its genesis facts, and the reserve is never read back as money.
#[test]
fn two_food_energy_worlds_buy_and_eat_through_the_real_owners() {
    for (name, copy) in [
        ("food-energy-dollars", DOLLARS),
        ("food-energy-shells", SHELLS),
    ] {
        let pack = food_energy(name, Path::new(MARKET_TOWN), copy);
        let saves = fresh(&format!("{name}-save"));
        let save = saves.join("world");
        run_pack(&pack, 5, 2, Some(&save));
        let history = facts(&save);
        let sales = sales_are_atomic(&history);
        assert!(sales > 0, "{name}: somebody bought something");
        assert!(
            count(&history, "items-consumed") > 0,
            "{name}: somebody ate"
        );
        let needs_facts: Vec<_> = history
            .iter()
            .filter(|fact| {
                matches!(
                    fact.event_type().as_str(),
                    "needs-assigned" | "nourishment-assigned" | "needs-interactions-configured"
                )
            })
            .collect();
        assert!(!needs_facts.is_empty(), "{name}: needs is configured");
        assert!(
            needs_facts.iter().all(|fact| matches!(
                fact.caused_by(),
                mineworld_contracts::Causation::WorldGenesis
            )),
            "{name}: needs states facts at genesis only"
        );
    }
}

/// E4: a food-energy world run one day and resumed to two is the world run two days straight — the
/// same facts byte for byte; no regrant, no duplicate meal, no wall-time drain.
#[test]
fn a_food_energy_world_resumed_has_the_history_of_one_never_stopped() {
    let pack = food_energy("food-energy-resume", Path::new(MARKET_TOWN), DOLLARS);
    let saves = fresh("food-energy-resume-saves");
    let straight = saves.join("straight");
    run_pack(&pack, 9, 2, Some(&straight));
    let split = saves.join("split");
    run_pack(&pack, 9, 1, Some(&split));
    let resumed = run_pack(&pack, 9, 2, Some(&split));
    assert!(resumed.contains("resumed"), "{resumed}");
    Tables::read(&split).assert_same_history(&Tables::read(&straight), "food-energy resume");
}

/// E1 (configuration half): the loader refuses each model contradiction naming the file, the owner's
/// code and the entry.
#[test]
fn model_contradictions_in_configure_are_refused_by_name() {
    for (case, section, code, entry) in [
        (
            "unitless-energy",
            "parameters:\n  - { energy_kcal_per_day: 2000 }\n",
            "needs-model-parameter-conflict",
            "parameters[0]",
        ),
        (
            "energy-rate",
            "parameters:\n  - { hunger_model: food-energy, energy_kcal_per_day: 2000 }\n  - { hunger_per_hour: 100 }\n",
            "needs-model-parameter-conflict",
            "parameters[1]",
        ),
        (
            "energy-without-use",
            "parameters:\n  - { hunger_model: food-energy }\n",
            "needs-energy-daily-use-required",
            "the file",
        ),
        (
            "energy-only-scoped",
            "parameters:\n  - { hunger_model: food-energy }\n  - { actor: person, energy_kcal_per_day: 2000 }\n",
            "needs-energy-daily-use-required",
            "the file",
        ),
        (
            "energy-shadowed-asleep-rate",
            "parameters:\n  - { hunger_asleep_per_hour: 20 }\n  - { hunger_model: food-energy, energy_kcal_per_day: 2000 }\n",
            "needs-model-parameter-conflict",
            "parameters[0]",
        ),
        (
            "scoped-model",
            "parameters:\n  - { actor: person, hunger_model: food-energy }\n",
            "hunger_model",
            "needs.yaml",
        ),
    ] {
        let pack = food_energy(
            &format!("food-energy-{case}"),
            Path::new(MARKET_TOWN),
            DOLLARS,
        );
        std::fs::write(pack.join("configure/needs.yaml"), section).expect("writes");
        let refusal = read(&pack)
            .err()
            .map(|error| error.to_string())
            .unwrap_or_else(|| panic!("{case}: expected a refusal"));
        for part in ["configure/needs.yaml", code, entry, "'needs'"] {
            assert!(
                refusal.contains(part),
                "{case}: {part:?} missing from: {refusal}"
            );
        }
    }
}

/// E1 (genesis half), through the real binary: a food-energy world whose person is written in the
/// unitless form, or above their daily use, stops `run --save` with the owner, code, entity and field,
/// and no save.
#[test]
fn contradictory_authored_genesis_stops_the_binary_with_owner_code_subject_and_field() {
    for (case, line, code, field) in [
        (
            "unitless-person",
            "needs: {}",
            "needs-model-mismatch",
            "hunger",
        ),
        (
            "over-capacity",
            "needs: { food_energy: { initial_kcal: 2001 } }",
            "needs-initial-energy-out-of-range",
            "food_energy.initial_kcal",
        ),
    ] {
        let pack = food_energy(
            &format!("food-energy-{case}"),
            Path::new(MARKET_TOWN),
            DOLLARS,
        );
        let alice = pack.join("people/alice.yaml");
        let text = std::fs::read_to_string(&alice)
            .expect("reads")
            .replace("\r\n", "\n");
        let rewritten: Vec<&str> = text
            .lines()
            .map(|written| {
                if written.starts_with("needs:") {
                    line
                } else {
                    written
                }
            })
            .collect();
        std::fs::write(&alice, rewritten.join("\n") + "\n").expect("writes");
        let saves = fresh(&format!("food-energy-{case}-save"));
        let save = saves.join("world");
        let output = mineworld(&[
            "run",
            pack.to_str().expect("a path"),
            "--headless",
            "--seed",
            "1",
            "--days",
            "1",
            "--save",
            save.to_str().expect("a path"),
        ]);
        let said = format!("{}{}", stderr(&output), stdout(&output));
        assert!(!output.status.success(), "{case}: {said}");
        for part in ["'needs'", code, "alice", field] {
            assert!(said.contains(part), "{case}: {part:?} missing from: {said}");
        }
        assert!(!save.exists(), "{case}: no save for a refused genesis");
    }
}

/// E5: the reference Market Town (unitless) still reads and loads, its people unitless.
#[test]
fn the_unitless_reference_world_is_unchanged() {
    let pack = read(Path::new(MARKET_TOWN)).expect("reads");
    let loaded = pack
        .load(mineworld_contracts::WorldTime::EPOCH)
        .expect("loads");
    let world = loaded.world();
    let unitless = world
        .read()
        .components::<mineworld_needs::Needs>()
        .all(|(_, needs)| matches!(needs.hunger(), mineworld_needs::Hunger::Unitless(_)));
    assert!(unitless, "every Market Town person is unitless");
}

/// The needs configured fact as an earlier build stated it: schema 1, with neither food-energy
/// parameter in any resolution (they did not exist).
fn as_an_earlier_build_said(fact: &EventEnvelope) -> EventEnvelope {
    let mut value = serde_json::to_value(fact).expect("a fact encodes");
    let record = &mut value["payload"];
    let bytes: Vec<u8> =
        serde_json::from_value(record["payload"].clone()).expect("the payload's bytes");
    let mut resolved: serde_json::Value = serde_json::from_slice(&bytes).expect("needs' JSON");
    let strip = |parameters: &mut serde_json::Value| {
        if let Some(map) = parameters.as_object_mut() {
            map.remove("hunger_model");
            map.remove("energy_kcal_per_day");
        }
    };
    strip(&mut resolved["base"]["parameters"]);
    if let Some(regions) = resolved["regions"].as_array_mut() {
        for region in regions {
            strip(&mut region[1]["parameters"]);
        }
    }
    record["payload"] =
        serde_json::to_value(serde_json::to_vec(&resolved).expect("encodes")).expect("bytes");
    record["schema_version"] = serde_json::json!(1);
    serde_json::from_value(value).expect("an envelope")
}

/// The save's manifest rewritten as an earlier build wrote it: `needs` at system version 1.
fn age_manifest(connection: &rusqlite::Connection) {
    let body: Vec<u8> = connection
        .query_row("SELECT body FROM manifest", [], |row| row.get(0))
        .expect("a manifest");
    let mut manifest: serde_json::Value = serde_json::from_slice(&body).expect("JSON");
    let mut aged = 0;
    for record in manifest["composition"].as_array_mut().expect("a list") {
        if record["declaration"]["system"] == "needs" {
            record["declaration"]["version"] = serde_json::json!(1);
            aged += 1;
        }
    }
    assert_eq!(aged, 1, "the save's composition names needs once");
    let body = serde_json::to_vec(&manifest).expect("encodes");
    connection
        .execute("UPDATE manifest SET body = ?1", [body])
        .expect("updates");
}

fn copy_family(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for suffix in ["", "-wal", "-shm"] {
        let name = format!("world.sqlite{suffix}");
        if from.join(&name).exists() {
            std::fs::copy(from.join(&name), to.join(&name)).expect("copied");
        }
    }
}

/// C-FE-2 / §3.7: a save of a needs-enabled world made by an earlier build is refused, never migrated.
/// Its configured fact differs in shape, so the drift check names `needs` first; a save whose
/// configuration matches but whose `needs` is version 1 is refused as outdated. A closed save and a
/// crash-tail WAL family (scratch copies) keep every committed fact, journal row, snapshot, sealed
/// block and manifest entry through the refusal.
#[test]
fn an_earlier_builds_needs_save_is_refused_and_left_as_it_was() {
    let town = read(Path::new(MARKET_TOWN)).expect("reads");
    let loaded = town
        .load(mineworld_contracts::WorldTime::EPOCH)
        .expect("loads");
    let saved: Vec<EventEnvelope> = loaded
        .genesis()
        .iter()
        .map(|fact| {
            if fact.event_type().as_str() == "needs-interactions-configured" {
                as_an_earlier_build_said(fact)
            } else {
                fact.clone()
            }
        })
        .collect();
    match town.check_configuration(&saved) {
        Err(mineworld_worldpack::PackError::ConfigurationDrift { system, .. }) => {
            assert_eq!(system.as_str(), "needs");
        }
        other => panic!("expected the drift on needs first, got {other:?}"),
    }

    let saves = fresh("food-energy-old-saves");
    let made = saves.join("made");
    run_pack(Path::new(MARKET_TOWN), 4, 1, Some(&made));
    let closed = saves.join("closed");
    let crashed = saves.join("crashed");
    copy_family(&made, &closed);
    copy_family(&made, &crashed);
    {
        let connection = rusqlite::Connection::open(closed.join("world.sqlite")).expect("opens");
        age_manifest(&connection);
    }
    {
        let connection = rusqlite::Connection::open(crashed.join("world.sqlite")).expect("opens");
        connection
            .pragma_update(None, "wal_autocheckpoint", 0)
            .expect("no checkpoint");
        age_manifest(&connection);
        let tail = saves.join("crash-tail");
        copy_family(&crashed, &tail);
        assert!(
            tail.join("world.sqlite-wal").exists(),
            "a WAL tail was copied"
        );
        drop(connection);
        std::fs::remove_dir_all(&crashed).expect("removable");
        std::fs::rename(&tail, &crashed).expect("renamed");
    }
    for family in [&closed, &crashed] {
        let before = Tables::read(family);
        let output = mineworld(&[
            "run",
            MARKET_TOWN,
            "--headless",
            "--seed",
            "4",
            "--days",
            "2",
            "--save",
            family.to_str().expect("a path"),
        ]);
        let said = format!("{}{}", stderr(&output), stdout(&output));
        assert!(!output.status.success(), "refused: {said}");
        assert!(said.contains("needs"), "names needs: {said}");
        let after = Tables::read(family);
        after.assert_same_history(&before, "a refused earlier-build save");
        assert_eq!(after.manifest.instance, before.manifest.instance);
    }
}
