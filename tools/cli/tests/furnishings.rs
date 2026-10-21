//! Furnishings (`ARC-307`; `pr-vf-furnishings-from-data.md` C3, C5), through the real binary and the
//! repository's own files.
//!
//! ```text
//! bound      every furnishing kind of every world under worlds/ (listed, not named) resolves to a
//!            binding in each default Presentation Pack (2D roles, 3D kinds) through the `.`-prefix
//!            rule, and every 3D binding's model (and its dressing) stands where contact.json says:
//!            a piece on the floor, its dressing on a surface
//! unbound    a kind no pack binds resolves to nothing (the clients then draw plain boxes, I-FD-5)
//! no names   the scripts that draw furnishings hold no place key, world id or kind literal (I-FD-2)
//! recorded   the standalone 3D slice's recorded observation is exactly what the real server
//!            discloses to an observer in social-cafe's café; drift fails (Q-FD-5). Re-record with
//!            MINEWORLD_WRITE_FIXTURES=1 cargo test -p mineworld-cli --test furnishings
//! ```
//!
//! Without the 3D client (ARC-347's removal check), `bound` skips where the 3D models stand and
//! `no names` skips the 3D script, each saying so; the rest runs unchanged.

mod bodies;
mod headless;
mod market;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use market::Town;
use mineworld_server::WireObservation;
use serde_json::{Value, json};
use support::{CLIENT_3D, Client, Server, client_3d_present};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every world under `worlds/`, and for each the kinds its places' `body.furnishings` name, read
/// from the YAML as data (not through bodies' types).
fn kinds_by_world() -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();
    let worlds = repo().join("worlds");
    for world in std::fs::read_dir(&worlds).expect("worlds/ lists") {
        let world = world.expect("an entry").path();
        let places = world.join("places");
        if !places.is_dir() {
            continue;
        }
        let name = world
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a name")
            .to_owned();
        let mut kinds = BTreeSet::new();
        for file in std::fs::read_dir(&places).expect("places/ lists") {
            let file = file.expect("an entry").path();
            if file.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            let text = std::fs::read_to_string(&file).expect("reads");
            let doc: Value =
                serde_saphyr::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
            for f in doc["body"]["furnishings"].as_array().into_iter().flatten() {
                kinds.insert(f["kind"].as_str().expect("a kind").to_owned());
            }
        }
        out.insert(name, kinds);
    }
    out
}

/// The binding `kind` resolves to under `bound`: itself, else each shorter `.`-prefix.
fn resolve(kind: &str, bound: impl Fn(&str) -> bool) -> Option<String> {
    let mut k = kind;
    loop {
        if bound(k) {
            return Some(k.to_owned());
        }
        k = &k[..k.rfind('.')?];
    }
}

/// The 2D pack's default set's roles, with its `extends` chain.
fn roles_2d() -> BTreeSet<String> {
    let bindings =
        read_json(&repo().join("presentation/mineworld-default/2D/assets/asset_bindings.yaml"));
    let mut roles = BTreeSet::new();
    let mut set = bindings["default_set"].as_str().map(str::to_owned);
    while let Some(name) = set {
        let own = &bindings["sets"][&name];
        roles.extend(
            own["roles"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(r, _)| r.clone()),
        );
        set = own["extends"].as_str().map(str::to_owned);
    }
    roles
}

fn kinds_3d() -> Value {
    let binding =
        read_json(&repo().join("presentation/mineworld-default/3D/furniture/furnishings.yaml"));
    assert_eq!(binding["format"], 1, "a format-1 binding");
    binding["kinds"].clone()
}

#[test]
fn every_kind_every_world_names_is_bound_in_each_default_pack() {
    let worlds = kinds_by_world();
    let used: BTreeSet<&String> = worlds.values().flatten().collect();
    println!("kinds by world: {worlds:?}");
    assert!(!used.is_empty(), "some world furnishes a place");
    let roles = roles_2d();
    let kinds = kinds_3d();
    // Where each 3D model stands is the 3D client's (contact.json); without the client, the 3D bindings
    // are still resolved, and only their placement is not checked.
    let with_3d = client_3d_present(
        &repo(),
        "where each 3D binding's model stands (contact.json)",
    );
    let mounts = if with_3d {
        read_json(&repo().join(CLIENT_3D).join("assets/models/contact.json"))
    } else {
        Value::Null
    };
    let mount = |model: &str| mounts["items"][model]["mount"].as_str().map(str::to_owned);
    let mut missing = Vec::new();
    for kind in &used {
        if resolve(kind, |k| roles.contains(&format!("furnishing.{k}"))).is_none() {
            missing.push(format!("2D: {kind}"));
        }
        match resolve(kind, |k| kinds.get(k).is_some()) {
            None => missing.push(format!("3D: {kind}")),
            Some(_) if !with_3d => {}
            Some(bound) => {
                let b = &kinds[&bound];
                let model = b["model"].as_str().expect("a model");
                if mount(model).as_deref() != Some("floor") {
                    missing.push(format!(
                        "3D: {kind} -> {model} is mounted {:?}, not floor",
                        mount(model)
                    ));
                }
                let sets = b["dressing"].as_array().into_iter().flatten();
                for prop in sets.flat_map(|set| set.as_array().into_iter().flatten()) {
                    let m = prop["model"].as_str().expect("a model");
                    if mount(m).as_deref() != Some("surface") {
                        missing.push(format!(
                            "3D: {kind}'s dressing {m} is mounted {:?}, not surface",
                            mount(m)
                        ));
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "unbound or misplaced: {missing:?}");
}

#[test]
fn a_kind_no_pack_binds_resolves_to_nothing_and_a_specific_kind_falls_back() {
    let roles = roles_2d();
    let kinds = kinds_3d();
    assert_eq!(resolve("spaceship.hull", |k| kinds.get(k).is_some()), None);
    assert_eq!(
        resolve("spaceship.hull", |k| roles
            .contains(&format!("furnishing.{k}"))),
        None
    );
    assert_eq!(
        resolve("chair.carved.oak", |k| kinds.get(k).is_some()).as_deref(),
        Some("chair"),
        "a specific kind falls back one `.`-segment at a time"
    );
}

/// The string literals of a GDScript file, comments removed.
fn literals(path: &Path) -> BTreeSet<String> {
    let text = std::fs::read_to_string(path).expect("reads");
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let mut in_str = false;
        let mut current = String::new();
        for c in line.chars() {
            match (in_str, c) {
                (false, '#') => break,
                (false, '"') => in_str = true,
                (true, '"') => {
                    out.insert(std::mem::take(&mut current));
                    in_str = false;
                }
                (true, c) => current.push(c),
                _ => {}
            }
        }
    }
    out
}

#[test]
fn the_scripts_that_draw_furnishings_name_no_place_world_or_kind() {
    let worlds = kinds_by_world();
    let mut names: BTreeSet<String> = worlds.keys().cloned().collect();
    for (world, kinds) in &worlds {
        names.extend(kinds.iter().cloned());
        for file in
            std::fs::read_dir(repo().join("worlds").join(world).join("places")).expect("lists")
        {
            let path = file.expect("an entry").path();
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.insert(stem.to_owned());
            }
        }
    }
    let mut scripts = vec![repo().join("clients/2d/scripts/scene/furnishings.gd")];
    if client_3d_present(&repo(), "the 3D client's furnishings script (I-FD-2)") {
        scripts.push(repo().join(CLIENT_3D).join("scripts/slice/furnishings.gd"));
    }
    for script in scripts {
        let found: Vec<String> = literals(&script)
            .into_iter()
            .filter(|lit| names.contains(lit))
            .collect();
        assert!(found.is_empty(), "{} names {found:?}", script.display());
    }
}

/// The café's `place-shape` as the real server discloses it to the visitor in social-cafe.
async fn disclosed_cafe_shape() -> Value {
    let pack = repo().join("worlds").join("social-cafe");
    let cafe = Town::read(&pack).id("cafe");
    let server = Server::start(&["server", pack.to_str().expect("a path")]).await;
    let mut client = Client::connect(server.address).await;
    client.join("visitor").await;
    let observation: WireObservation = client.observation().await;
    observation
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .find(|record| {
            record.entity().raw() == cafe.raw() && record.component_type().as_str() == "place-shape"
        })
        .map(|record| record.payload().clone())
        .expect("the café's shape is disclosed to the visitor standing in it")
}

#[tokio::test]
async fn the_standalone_slices_recorded_observation_is_what_the_server_discloses() {
    let shape = disclosed_cafe_shape().await;
    assert_eq!(
        shape["furnishings"].as_array().map(Vec::len),
        Some(16),
        "the café's sixteen furnishings are disclosed"
    );
    let recorded = json!({
        "format": 1,
        "about": "The place-shape the real server discloses to an observer in social-cafe's café: what the standalone 3D slice draws its furniture from (ARC-307, Q-FD-5). Written by tools/cli/tests/furnishings.rs; do not edit by hand.",
        "world": "social-cafe",
        "place": "cafe",
        "place-shape": shape,
    });
    let path = repo().join("tests/fixtures/observations/furnishings/social-cafe-cafe.json");
    let text = serde_json::to_string_pretty(&recorded).expect("serializes") + "\n";
    if std::env::var_os("MINEWORLD_WRITE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("creates");
        std::fs::write(&path, &text).expect("writes");
        println!("recorded {}", path.display());
        return;
    }
    // Compared as values: key order is the serializer's business, the content is the contract.
    assert!(
        read_json(&path) == recorded,
        "{} drifted from the world's data; re-record with MINEWORLD_WRITE_FIXTURES=1",
        path.display()
    );
}

/// AC-FD-4: a generated Lakeside variant — `bodies` enabled, its bakery given S33-b §5.3's floor and
/// solids, the free-standing pieces as furnishings (`pr-vf-furnishings-from-data.md` §3.9) — validates
/// through the real binary, and every kind it names resolves in both default packs. A second world, so
/// the mechanism is not the café's.
#[test]
fn a_lakeside_variants_bakery_carries_furnishings_and_validates() {
    let copy = headless::fresh("furnishings-lakeside").within("lakeside");
    bodies::copy_dir(&repo().join("worlds").join("lakeside"), &copy);
    let world = copy.join("world.yaml");
    let text = std::fs::read_to_string(&world).expect("reads");
    assert_eq!(
        text.matches("  - appearance\n").count(),
        1,
        "one appearance line"
    );
    std::fs::write(
        &world,
        text.replace("  - appearance\n", "  - appearance\n  - bodies\n"),
    )
    .expect("writes");
    let bakery = copy.join("places").join("bakery.yaml");
    let text = std::fs::read_to_string(&bakery).expect("reads");
    let text = text.replace(
        "      x: 2000\n      y: 200\n",
        "      x: 2000\n      y: 400\n",
    );
    let body = "\nbody:\n\
        \x20 floor: { min: { x: -1000, y: 0 }, max: { x: 5000, y: 8000 } }\n\
        \x20 furnishings:\n\
        \x20   - { key: long-table, kind: table.long, at: { x: 50, y: 3000 }, facing: 90000 }\n\
        \x20   - { key: bread-racks, kind: shelves.bread, at: { x: 4700, y: 6700 }, facing: 270000 }\n\
        \x20 solids:\n\
        \x20   - { min: { x: -1000, y: 7000 }, max: { x: 2500, y: 8000 }, height: 2200 }\n\
        \x20   - { min: { x: 2500, y: 4600 }, max: { x: 4600, y: 5200 }, height: 950 }\n\
        \x20   - { min: { x: 4400, y: 5600 }, max: { x: 5000, y: 7800 }, height: 1900, furnishing: bread-racks }\n\
        \x20   - { min: { x: -700, y: 1400 }, max: { x: 800, y: 4600 }, height: 750, furnishing: long-table }\n";
    std::fs::write(&bakery, format!("{text}{body}")).expect("writes");
    let entities = repo().join("entities");
    let presentation = repo().join("presentation").join("mineworld-default");
    let output = headless::mineworld(&[
        "validate",
        copy.to_str().expect("a path"),
        "--packs",
        entities.to_str().expect("a path"),
        "--packs",
        presentation.to_str().expect("a path"),
    ]);
    let printed = format!("{}{}", headless::stdout(&output), headless::stderr(&output));
    println!("{printed}");
    assert!(
        output.status.success(),
        "the Lakeside variant validates: {printed}"
    );
    let roles = roles_2d();
    let kinds = kinds_3d();
    for kind in ["table.long", "shelves.bread"] {
        assert!(
            resolve(kind, |k| roles.contains(&format!("furnishing.{k}"))).is_some(),
            "2D {kind}"
        );
        assert!(
            resolve(kind, |k| kinds.get(k).is_some()).is_some(),
            "3D {kind}"
        );
    }
}
