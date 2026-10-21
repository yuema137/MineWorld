//! Every item an observation names has a name a client can show, in every shipped world.
//!
//! Items are never perceived as entities (`ARC-37`): a shop's listing, a holder's `holdings`, a job's
//! `produces`, a `buy` offer and a `kick` name an item by id only. Two disclosures on the observer's
//! place name them: the `item` pack's `item-catalogue` — `{kinds: [{item, category, name}]}` for every
//! declared kind — and, for a loose object (which is not a kind, `ARC-36`), the tags its entry in bodies'
//! `loose-objects` listing carries. Those are the tables a client resolves an item's name from. Before
//! this test the 2D panels showed `item 26` and the agent's view had no name and answered `unknown_ref`
//! for a good on sale.
//!
//! ```text
//! crawl       every world under worlds/ and the commute-town test world, through the real binary:
//!             each seat's first observation, then the first seat walked into every shop (the places
//!             the world's organizations run a shop at, read from their files)
//! server      every item a crawled observation names is named there: by the item-catalogue, or by
//!             the tags of its loose-objects entry
//! fixtures    MINEWORLD_WRITE_FIXTURES=1 writes, per world, the crawled observations
//!             that each add an item or a shop to tests/fixtures/observations/items/<world>.json; the
//!             agent's view test (clients/agent/tests/test_item_names.py) and the 2D test below read them
//! 2d          (ignored, needs Godot) the 2D client's readers name every item of every fixture by that
//!             name, and no panel row shows an item by its id
//! ```
//!
//! ```text
//! cargo test -p mineworld-cli --test item_names
//! MINEWORLD_WRITE_FIXTURES=1 cargo test -p mineworld-cli --test item_names -- --test-threads=1
//! cargo test -p mineworld-cli --test item_names -- --ignored
//! ```

mod godot2d;
mod headless;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use mineworld_contracts::{ActionResult, EntityId, EntityType, Rejection, WorldTime};
use mineworld_worldpack::{PackRoots, WorldPack};
use serde_json::Value;
use support::{Client, STEP_PACE, Server, walk_step, walk_to};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
/// How long the crawl waits for an arrival after a walk ends.
const ARRIVAL: Duration = Duration::from_secs(5);
/// The most strides one walk is paced for before the crawl gives the place up.
const STRIDES: usize = 400;

/// Every shipped world: each directory under `worlds/` with a `world.yaml`, and the commute-town
/// test world (the needs fixtures' world).
fn worlds() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(Path::new(ROOT).join("worlds"))
        .expect("worlds/ lists")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("world.yaml").is_file())
        .collect();
    found.sort();
    found.push(Path::new(ROOT).join("tools/cli/tests/worlds/commute-town"));
    found
}

fn name_of(world: &Path) -> String {
    world
        .file_name()
        .expect("a directory name")
        .to_string_lossy()
        .into_owned()
}

/// The pack roots the reference worlds are validated with (Lakeside requires them).
fn roots() -> PackRoots {
    PackRoots::new(
        vec![
            Path::new(ROOT).join("entities"),
            Path::new(ROOT).join("presentation/mineworld-default"),
        ],
        None,
    )
    .expect("the repository's pack roots")
}

/// The place keys the world's own organizations run a shop at (economy's `shop: { at }`, ARC-38).
fn shop_places(world: &Path) -> BTreeSet<String> {
    let Ok(files) = std::fs::read_dir(world.join("organizations")) else {
        return BTreeSet::new();
    };
    files
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).expect("an organization file reads");
            let value: Value = serde_saphyr::from_str(&text).expect("an organization file parses");
            value["economy"]["shop"]["at"].as_str().map(str::to_owned)
        })
        .collect()
}

/// The world's seats, in order, and the ids of the places its shops are at, read from the pack itself.
fn seats_and_shops(world: &Path) -> (Vec<String>, Vec<EntityId>) {
    let read = WorldPack::read_with(world, &roots()).expect("the World Pack reads");
    let seats = read
        .seats()
        .iter()
        .map(|seat| seat.as_str().to_owned())
        .collect();
    let loaded = read.load(WorldTime::EPOCH).expect("it loads");
    let view = loaded.world().read();
    let shops = shop_places(world);
    let places = loaded
        .ids()
        .iter()
        .filter(|(key, id)| {
            shops.contains(key.as_str())
                && view
                    .entity(**id)
                    .is_some_and(|entity| entity.entity_type() == EntityType::Place)
        })
        .map(|(_, id)| *id)
        .collect();
    (seats, places)
}

/// One crawled observation: who saw it, where, and the observation as the wire carried it.
struct Seen {
    label: String,
    observation: Value,
}

/// The place the observation's observer stands in.
fn place_of(observation: &Value) -> Option<String> {
    observation["self_location"]["place"]["entity"]
        .as_str()
        .map(str::to_owned)
}

/// The payload of `component` on the entity `id`, if disclosed.
fn component<'a>(observation: &'a Value, id: &str, component: &str) -> Option<&'a Value> {
    observation["entities"]
        .as_array()?
        .iter()
        .find(|entity| entity["id"].as_str() == Some(id))?["components"]
        .as_array()?
        .iter()
        .find(|record| record["component_type"].as_str() == Some(component))
        .map(|record| &record["payload"])
}

/// Item id → name, from the item-catalogue of the observer's place.
fn catalogue(observation: &Value) -> BTreeMap<String, String> {
    let Some(place) = place_of(observation) else {
        return BTreeMap::new();
    };
    component(observation, &place, "item-catalogue")
        .and_then(|payload| payload["kinds"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|kind| {
            Some((
                kind["item"]["entity"].as_str()?.to_owned(),
                kind["name"].as_str()?.to_owned(),
            ))
        })
        .collect()
}

/// Loose object id → its tags joined with ", ", from the observer's place's `loose-objects` listing
/// (items that are not kinds, `ARC-36`); an object without tags is left out.
fn loose_objects(observation: &Value) -> BTreeMap<String, String> {
    let Some(place) = place_of(observation) else {
        return BTreeMap::new();
    };
    component(observation, &place, "loose-objects")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|lying| {
            let tags: Vec<&str> = lying["tags"]
                .as_array()?
                .iter()
                .filter_map(Value::as_str)
                .collect();
            (!tags.is_empty()).then(|| {
                (
                    lying["object"]["entity"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                    tags.join(", "),
                )
            })
        })
        .collect()
}

/// What the observation calls each item it names: the catalogue's name, else the loose object's tags.
fn names(observation: &Value) -> BTreeMap<String, String> {
    let mut found = loose_objects(observation);
    found.extend(catalogue(observation));
    found
}

/// Every item a value names by a typed reference (`{entity, entity_type: "item"}`), at any depth.
fn items_named(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if map.get("entity_type").and_then(Value::as_str) == Some("item")
                && let Some(entity) = map.get("entity").and_then(Value::as_str)
            {
                found.insert(entity.to_owned());
                return;
            }
            map.values().for_each(|inner| items_named(inner, found));
        }
        Value::Array(values) => values.iter().for_each(|inner| items_named(inner, found)),
        _ => {}
    }
}

/// Every item the observation names.
fn items_of(observation: &Value) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    items_named(observation, &mut found);
    found
}

fn has_shop(observation: &Value) -> bool {
    place_of(observation).is_some_and(|place| component(observation, &place, "shop").is_some())
}

/// Walks `actor` into `place`; the first observation in which it stands there, or `None` when the
/// world refuses the walk or it does not arrive.
async fn walk_into(client: &mut Client, actor: EntityId, place: EntityId) -> Option<Value> {
    match client.submit(walk_to(actor, place, None)).await {
        (_, ActionResult::Accepted { .. }) => {}
        _ => return None,
    }
    for _ in 0..STRIDES {
        tokio::time::sleep(STEP_PACE).await;
        match client.submit(walk_step(actor)).await {
            (_, ActionResult::Accepted { .. }) => {}
            (_, ActionResult::Rejected(Rejection::PreconditionFailed)) => break,
            _ => return None,
        }
    }
    let deadline = Instant::now() + ARRIVAL;
    let wanted = place.to_string();
    while Instant::now() < deadline {
        let observation =
            serde_json::to_value(client.observation().await).expect("an observation encodes");
        if place_of(&observation).as_deref() == Some(wanted.as_str()) {
            return Some(observation);
        }
    }
    None
}

/// Every seat's first observation, then the first seat walked into each shop in turn.
async fn crawl(world: &Path) -> Vec<Seen> {
    let (seats, places) = seats_and_shops(world);
    let entities = format!("{ROOT}/entities");
    let presentation = format!("{ROOT}/presentation/mineworld-default");
    let path = world.to_str().expect("a printable path");
    let server = Server::start(&[
        "server",
        path,
        "--packs",
        &entities,
        "--packs",
        &presentation,
    ])
    .await;
    let mut seen = Vec::new();
    // The first seat's client is kept to walk with; every other one leaves after its observation.
    let mut walker: Option<(Client, EntityId, String)> = None;
    for seat in &seats {
        let mut client = Client::connect(server.address).await;
        let (actor, _) = client.join(seat).await;
        let observation =
            serde_json::to_value(client.observation().await).expect("an observation encodes");
        seen.push(Seen {
            label: format!("seat {seat}"),
            observation,
        });
        if walker.is_none() {
            walker = Some((client, actor, seat.clone()));
        } else {
            client.disconnect().await;
        }
    }
    let Some((mut client, actor, walker)) = walker else {
        return seen;
    };
    for place in places {
        if let Some(observation) = walk_into(&mut client, actor, place).await {
            seen.push(Seen {
                label: format!("seat {walker} in place {place}"),
                observation,
            });
        }
    }
    seen
}

fn fixtures() -> PathBuf {
    Path::new(ROOT).join("tests/fixtures/observations/items")
}

/// The crawled observations worth keeping: each that adds an item not yet kept, or the first one in
/// each shop.
fn kept(seen: &[Seen]) -> Vec<&Seen> {
    let mut items = BTreeSet::new();
    let mut shops = BTreeSet::new();
    seen.iter()
        .filter(|one| {
            let named = items_of(&one.observation);
            let new_item = named.iter().any(|item| !items.contains(item));
            let new_shop = has_shop(&one.observation)
                && place_of(&one.observation).is_some_and(|place| !shops.contains(&place));
            items.extend(named);
            if has_shop(&one.observation) {
                shops.extend(place_of(&one.observation));
            }
            new_item || new_shop
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn every_item_an_observation_names_is_named_by_its_place() {
    let write = std::env::var_os("MINEWORLD_WRITE_FIXTURES").is_some();
    let mut failures = Vec::new();
    for world in worlds() {
        let name = name_of(&world);
        let seen = crawl(&world).await;
        let mut items = 0;
        for one in &seen {
            let named = names(&one.observation);
            for item in items_of(&one.observation) {
                items += 1;
                if named.get(&item).is_none_or(String::is_empty) {
                    failures.push(format!(
                        "{name}, {}: item {item} is named by neither the item-catalogue nor loose-objects",
                        one.label
                    ));
                }
            }
        }
        println!("{name}: {} observations, {items} item mentions", seen.len());
        let keep = kept(&seen);
        if write && !keep.is_empty() {
            std::fs::create_dir_all(fixtures()).expect("the fixture directory");
            let observations: Vec<&Value> = keep.iter().map(|one| &one.observation).collect();
            let text = serde_json::to_string(&observations).expect("observations encode");
            std::fs::write(fixtures().join(format!("{name}.json")), text + "\n")
                .expect("the fixture writes");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The fixtures, by world: each file's observations, parsed.
fn recorded() -> Vec<(String, Vec<Value>)> {
    let mut found: Vec<(String, Vec<Value>)> = std::fs::read_dir(fixtures())
        .expect("the item fixtures are committed")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("a fixture reads");
            let observations: Vec<Value> = serde_json::from_str(&text).expect("a fixture parses");
            (
                path.file_stem()
                    .expect("a stem")
                    .to_string_lossy()
                    .into_owned(),
                observations,
            )
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// The 2D client's readers, over every fixture: each item is named by its catalogue name, no panel
/// row shows an item by the unnamed form `item <id>`, and no panel row is raw JSON. The harness prints `THING <world> <n> <id> <name>`
/// and `ROW <world> <n> <row>`.
#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test item_names -- --ignored"]
fn the_2d_panels_name_every_item_of_every_fixture() {
    // Godot from PATH or `GODOT`, with the 2D project's class cache built (the harness names its
    // global classes); without Godot the test fails rather than passes.
    let godot = godot2d::imported();
    let client = Path::new(ROOT).join("clients/2d");
    let output = Command::new(&godot)
        .args(["--headless", "--path"])
        .arg(&client)
        .args(["--script", "res://scripts/harness/item_names.gd", "--"])
        .arg(format!("--fixtures={}", fixtures().display()))
        .output()
        .expect("Godot runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the item-names harness failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut shown: BTreeMap<(String, usize, String), String> = BTreeMap::new();
    let mut rows: Vec<(String, usize, String)> = Vec::new();
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("THING ") {
            let mut parts = rest.splitn(4, ' ');
            let (world, n, id, name) = (
                parts.next().expect("a world").to_owned(),
                parts.next().expect("an index").parse().expect("a number"),
                parts.next().expect("an id").to_owned(),
                parts.next().unwrap_or("").to_owned(),
            );
            shown.insert((world, n, id), name);
        } else if let Some(rest) = line.strip_prefix("ROW ") {
            let mut parts = rest.splitn(3, ' ');
            rows.push((
                parts.next().expect("a world").to_owned(),
                parts.next().expect("an index").parse().expect("a number"),
                parts.next().unwrap_or("").to_owned(),
            ));
        }
    }
    let mut failures = Vec::new();
    let mut checked = 0;
    for (world, observations) in recorded() {
        for (n, observation) in observations.iter().enumerate() {
            let names = names(observation);
            for item in items_of(observation) {
                checked += 1;
                let got = shown.get(&(world.clone(), n, item.clone()));
                if got != names.get(&item) {
                    failures.push(format!(
                        "{world} #{n}: item {item} shown as {got:?}, disclosed as {:?}",
                        names.get(&item)
                    ));
                }
                let unnamed = format!("item {item}");
                for (_, _, row) in rows.iter().filter(|(w, i, _)| *w == world && *i == n) {
                    let shows_id = row.match_indices(&unnamed).any(|(at, _)| {
                        !row[at + unnamed.len()..].starts_with(|c: char| c.is_ascii_digit())
                    });
                    if shows_id {
                        failures.push(format!("{world} #{n}: a panel row shows {unnamed}: {row}"));
                    }
                }
            }
        }
    }
    // Every panel reads its component through the readers, a component no reader knows by its structure:
    // no row is raw JSON (promo v2, the I panel showed `ways`, `currency`, `wealth` as JSON).
    for (world, n, row) in &rows {
        if row.contains("{\"") || row.contains("entity_type") || row.starts_with('[') {
            failures.push(format!("{world} #{n}: a panel row is raw JSON: {row}"));
        }
    }
    assert!(checked > 0, "the fixtures name no item");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
