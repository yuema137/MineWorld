//! The standalone 3D slice's recorded station furnishings (S29-e; `ARC-307`, Q-FD-5's rule for the café):
//! the `place-shape` bodies states for every `station`-tagged place of social-cafe, with the place's tags,
//! written to `tests/fixtures/observations/furnishings/social-cafe-station-<place>.json`. The client draws
//! the benches and the kiosk from these when no server is attached, into the kit place the tags bind,
//! exactly as it draws them from a live disclosure.
//!
//! The payload is bodies' own `place-shaped` genesis fact for the place, decoded by bodies' published type:
//! the component bodies discloses as `place-shape` is that same `PlaceShape`. A drift from the world's
//! data fails here; re-record with `MINEWORLD_WRITE_FIXTURES=1`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mineworld_bodies::PlaceShaped;
use mineworld_contracts::{EntityId, Tag, WorldTime};
use mineworld_worldpack::{PackRoots, WorldPack};
use serde_json::{Value, json};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const WORLD: &str = "social-cafe";

fn repo() -> PathBuf {
    PathBuf::from(ROOT)
}

fn roots() -> PackRoots {
    PackRoots::new(
        vec![
            repo().join("entities"),
            repo().join("presentation/mineworld-default"),
        ],
        None,
    )
    .expect("the repository's pack roots")
}

/// place key -> (its tags, its `place-shape` as bodies states it), for every `station`-tagged place.
fn station_shapes(world: &Path) -> BTreeMap<String, (Vec<String>, Value)> {
    let pack = WorldPack::read_with(world, &roots()).expect("the world reads");
    let loaded = pack.load(WorldTime::EPOCH).expect("the world loads");
    let keys: BTreeMap<EntityId, String> = loaded
        .ids()
        .iter()
        .map(|(key, id)| (*id, key.as_str().to_owned()))
        .collect();
    let station = Tag::new("station").expect("a tag");
    let mut out = BTreeMap::new();
    for fact in loaded.genesis() {
        let Ok(payload) = fact.payload().payload_for::<PlaceShaped>() else {
            continue;
        };
        let shaped: PlaceShaped = serde_json::from_slice(payload).expect("bodies' encoding");
        let key = keys[&shaped.place().entity_id()].clone();
        let place = pack
            .places()
            .iter()
            .find(|(k, _)| k.as_str() == key)
            .map(|(_, p)| p)
            .expect("the shaped place is the pack's");
        if !place.tags.contains(&station) {
            continue;
        }
        let tags = place.tags.iter().map(|t| t.as_str().to_owned()).collect();
        out.insert(
            key,
            (
                tags,
                serde_json::to_value(shaped.shape()).expect("serializes"),
            ),
        );
    }
    out
}

#[test]
fn the_standalone_slices_recorded_station_furnishings_are_the_worlds() {
    let shapes = station_shapes(&repo().join("worlds").join(WORLD));
    assert!(
        shapes
            .values()
            .any(|(_, s)| s["furnishings"].as_array().is_some_and(|f| !f.is_empty())),
        "the station is furnished"
    );
    let write = std::env::var_os("MINEWORLD_WRITE_FIXTURES").is_some();
    for (place, (tags, shape)) in shapes {
        if shape["furnishings"].as_array().is_none_or(Vec::is_empty) {
            continue;
        }
        let recorded = json!({
            "format": 1,
            "about": "The place-shape bodies states for a station place of social-cafe, with the place's tags: what the standalone 3D slice draws the station's furnishings from (ARC-307, S29-e). Written by tools/cli/tests/station_furnishings_fixture.rs; do not edit by hand.",
            "world": WORLD,
            "place": place,
            "tags": tags,
            "place-shape": shape,
        });
        let path = repo().join(format!(
            "tests/fixtures/observations/furnishings/{WORLD}-station-{place}.json"
        ));
        if write {
            let text = serde_json::to_string_pretty(&recorded).expect("serializes") + "\n";
            std::fs::write(&path, text).expect("writes");
            println!("recorded {}", path.display());
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} is missing; record it", path.display()));
        let on_disk: Value = serde_json::from_str(&text).expect("JSON");
        assert!(
            on_disk == recorded,
            "{} drifted from the world's data; re-record with MINEWORLD_WRITE_FIXTURES=1",
            path.display()
        );
    }
}
