//! S30-e (`pr-s30e-builtin-catalogue.md` §8): the built-in catalogue, its check, the San Diego County v3
//! profile, and backends and masks — through the real binary.

mod headless;

use std::path::Path;

use headless::{fresh, mineworld, stderr, stdout};
use serde_json::Value;

const ENTITIES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../entities");
const BUILTIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-builtin"
);
const BACKENDS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-backends"
);

/// The sensitive `core` attributes, private by Q-S30-8 (I-e-2).
const SENSITIVE: [&str; 7] = [
    "core.natal-sex",
    "core.birth",
    "core.age-years",
    "core.ancestry",
    "core.religion",
    "core.mass",
    "core.honesty",
];

fn json(arguments: &[&str]) -> Value {
    let output = mineworld(arguments);
    assert!(output.status.success(), "{}", stderr(&output));
    serde_json::from_str(&stdout(&output)).expect("one JSON document")
}

fn attributes(document: &Value) -> Vec<&Value> {
    document["attributes"]
        .as_array()
        .expect("attributes")
        .iter()
        .collect()
}

fn ids(document: &Value) -> Vec<&str> {
    attributes(document)
        .iter()
        .filter_map(|declaration| declaration["id"].as_str())
        .collect()
}

/// A copy of `from` at `to`, every file.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copied");
        }
    }
}

/// AC-e-3 (AC-S30-2), AC-e-7: 93 `core` declarations in the six groups; every sensitive one private and
/// every appearance one but `mass` perceivable.
#[test]
fn catalogue_holds_93_core_attributes_in_six_groups() {
    let document = json(&[
        "attributes",
        "catalogue",
        BUILTIN,
        "--json",
        "--packs",
        ENTITIES,
    ]);
    assert_eq!(document["namespaces"]["core"], 93);
    assert_eq!(document["backend"]["id"], "mineworld");
    assert_eq!(document["mask"]["id"], "core");
    let mut groups = std::collections::BTreeMap::new();
    for declaration in attributes(&document) {
        let id = declaration["id"].as_str().unwrap_or_default();
        if !id.starts_with("core.") {
            continue;
        }
        let group = declaration["group"].as_str().unwrap_or_default().to_owned();
        *groups.entry(group.clone()).or_insert(0) += 1;
        let visibility = declaration["visibility"].as_str().unwrap_or_default();
        if SENSITIVE.contains(&id) {
            assert_eq!(visibility, "private", "{id}");
        } else if group == "appearance" {
            assert_eq!(visibility, "perceivable", "{id}");
        }
    }
    let expected = [
        ("appearance", 21),
        ("background", 15),
        ("behaviour", 14),
        ("capability", 13),
        ("lifestyle", 14),
        ("psychology", 16),
    ];
    assert_eq!(
        groups.into_iter().collect::<Vec<_>>(),
        expected.map(|(group, count)| (group.to_owned(), count))
    );
}

/// AC-e-1, AC-e-3: `check` on the world requiring both packs is clean, prints the count and each reader
/// line, and lists only the profile's `verify` marks.
#[test]
fn catalogue_check_is_clean_with_reader_lines_and_verify_marks() {
    let output = mineworld(&["attributes", "check", BUILTIN, "--packs", ENTITIES]);
    let printed = stdout(&output);
    assert!(output.status.success(), "{}{printed}", stderr(&output));
    assert!(
        printed.contains("core: 93 attribute(s), each with its reader line"),
        "{printed}"
    );
    assert!(
        printed.contains("  core.stature: C, OV: height in both clients; tall"),
        "{printed}"
    );
    let verify: Vec<&str> = printed
        .lines()
        .filter(|line| line.starts_with("verify: "))
        .collect();
    assert_eq!(verify.len(), 1, "{verify:?}");
    assert!(verify[0].starts_with("verify: core.build: "), "{verify:?}");
}

/// AC-e-4 (X-S30-20): one `core` id removed from `masks/core.yaml` fails `check`, naming it.
#[test]
fn catalogue_check_refuses_a_core_attribute_without_its_reader_line() {
    let root = fresh("s30e-reader-line");
    copy_tree(Path::new(ENTITIES), &root.join("entities"));
    let mask = root.join("entities/person-attributes/masks/core.yaml");
    let text = std::fs::read_to_string(&mask).expect("readable");
    // The entry stays (so glasses still exists) but its `why` is no reader line.
    let planted: Vec<&str> = text
        .lines()
        .map(|line| {
            if line.contains("id: core.glasses,") {
                "  - { id: core.glasses, why: \"glasses\" }"
            } else {
                line
            }
        })
        .collect();
    assert_ne!(
        planted.join("\n") + "\n",
        text,
        "one line names core.glasses"
    );
    std::fs::write(&mask, planted.join("\n") + "\n").expect("written");
    let packs = root.join("entities");
    let output = mineworld(&[
        "attributes",
        "check",
        BUILTIN,
        "--packs",
        packs.to_str().expect("utf-8"),
    ]);
    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("core.glasses has no reader line"),
        "{}",
        stderr(&output)
    );
}

/// AC-S30-5 under v3: 1 000 synthetic adults realise every stratified marginal within 0.5 pp.
#[test]
fn profile_v3_realises_its_marginals_for_a_thousand_people() {
    let document = json(&[
        "attributes",
        "resolve",
        BUILTIN,
        "--json",
        "--synthetic",
        "1000",
        "--packs",
        ENTITIES,
    ]);
    assert_eq!(document["profile"]["id"], "san-diego-county");
    assert_eq!(document["profile"]["version"], 3);
    let marginal: f64 = document["worst"]["marginal_pp"]
        .as_str()
        .and_then(|text| text.parse().ok())
        .expect("a marginal");
    assert!(marginal <= 0.5, "{}", document["worst"]);
}

/// AC-e-6: `backend: toy` with `mask: toy-small` composes the toy catalogue as masked, plus `core`'s
/// appearance group and quantities (and the instant they derive from), and nothing else of `core`.
#[test]
fn backend_toy_composes_its_masked_catalogue_with_mineworlds_appearance() {
    let world = format!("{BACKENDS}/attributes-backends");
    let packs = format!("{BACKENDS}/packs");
    let document = json(&[
        "attributes",
        "catalogue",
        &world,
        "--json",
        "--packs",
        ENTITIES,
        "--packs",
        &packs,
    ]);
    let held = ids(&document);
    assert_eq!(document["backend"]["id"], "toy");
    assert!(held.contains(&"toy-backend.mood"), "{held:?}");
    assert!(!held.contains(&"toy-backend.focus"), "masked: {held:?}");
    for kept in [
        "core.stature",
        "core.hair-colour",
        "core.age-years",
        "core.birth",
    ] {
        assert!(held.contains(&kept), "{kept}: {held:?}");
    }
    for dropped in ["core.chronotype", "core.religion", "core.languages-spoken"] {
        assert!(!held.contains(&dropped), "{dropped}: {held:?}");
    }

    let output = mineworld(&[
        "attributes",
        "backends",
        &world,
        "--packs",
        ENTITIES,
        "--packs",
        &packs,
    ]);
    let printed = stdout(&output);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        printed.contains("* toy 0.1.0") && printed.contains("  mineworld 1.0.0"),
        "{printed}"
    );
}

/// AC-e-6: a person file stating an attribute the mask leaves out is `MaskedOut`.
#[test]
fn backend_a_masked_attribute_stated_by_a_person_is_masked_out() {
    let root = fresh("s30e-masked-out");
    copy_tree(Path::new(BACKENDS), &root);
    std::fs::write(
        root.join("attributes-backends/people/bob.yaml"),
        "name: Bob\nlocation: { place: cafe }\nattributes:\n  toy-backend.focus: sharp\n",
    )
    .expect("written");
    let world = root.join("attributes-backends");
    let packs = root.join("packs");
    let output = mineworld(&[
        "attributes",
        "catalogue",
        world.to_str().expect("utf-8"),
        "--packs",
        ENTITIES,
        "--packs",
        packs.to_str().expect("utf-8"),
    ]);
    assert!(!output.status.success(), "{}", stdout(&output));
    let refusal = stderr(&output);
    assert!(
        refusal.contains("MaskedOut") && refusal.contains("toy-backend.focus"),
        "{refusal}"
    );
}

/// S30-f §5.3: every built-in a consumer reads (`observed`, `prompt`, `speech`) carries a prompt rank
/// except the casting vocabulary `hair-style` and `hair-texture` and the textless instant `birth`;
/// no two built-ins share one.
#[test]
fn every_consumer_faceted_builtin_has_its_own_prompt_rank() {
    let document = json(&[
        "attributes",
        "catalogue",
        BUILTIN,
        "--json",
        "--packs",
        ENTITIES,
    ]);
    let unranked = ["core.hair-style", "core.hair-texture", "core.birth"];
    let mut ranks = std::collections::BTreeMap::new();
    for declaration in attributes(&document) {
        let id = declaration["id"].as_str().expect("an id");
        if !id.starts_with("core.") {
            continue;
        }
        let consumed = declaration["facets"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|facet| ["observed", "prompt", "speech"].contains(&facet.as_str().unwrap_or("")));
        match declaration["prompt_rank"].as_u64() {
            Some(rank) => {
                assert!(!unranked.contains(&id), "{id} is ranked");
                if let Some(other) = ranks.insert(rank, id.to_owned()) {
                    panic!("{id} and {other} share rank {rank}");
                }
            }
            None => assert!(!consumed || unranked.contains(&id), "{id} has no rank"),
        }
    }
    assert_eq!(ranks.len(), 60);
}
