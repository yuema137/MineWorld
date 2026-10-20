//! `mineworld reference` — the configuration reference generated from the build (`docs/DECISIONS.md`
//! `ARC-90`; `pr-s25a-reference.md` §4, AC-25a-1, -4, -6).
//!
//! Every test runs the real binary. The examples a page prints are read back out of the Markdown and
//! decoded by the installed set's own decoders from a YAML stream, as the loader decodes a World Pack's
//! files (SD-25a-8); the World Pack page's examples are composed into a world and validated.

mod headless;

use std::collections::BTreeMap;
use std::path::Path;

use headless::{fresh, mineworld, stderr, stdout};
use mineworld_worldpack::catalog::AVAILABLE;
use mineworld_worldpack::{Capability, EntityClasses};
use serde::Deserializer as _;
use serde::de::{MapAccess, Visitor};

/// The checked-in reference.
const CHECKED_IN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/reference");

/// AC-25a-1, INV-S25-2: the checked-in reference is exactly what this build generates — on every
/// platform the `core` layer runs on.
#[test]
fn the_checked_in_reference_is_what_this_build_generates() {
    let checked = mineworld(&["reference", "--check", CHECKED_IN]);
    assert!(
        checked.status.success(),
        "{}\nRegenerate it: cargo run -p mineworld-cli -- reference --out docs/reference",
        stderr(&checked)
    );
}

/// What a page predicts for a key a serde-derived file does not list.
const UNKNOWN: &str = "unknown field";

/// The reference this build generates, written into a fresh directory.
fn generated(name: &str) -> headless::Scratch {
    let dir = fresh(name);
    let output = mineworld(&["reference", "--out", text(&dir)]);
    assert!(output.status.success(), "{}", stderr(&output));
    dir
}

fn text(path: &Path) -> &str {
    path.to_str().expect("a printable path")
}

/// Every file under `dir`, by `/`-separated path.
fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(&next).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let name = path
                    .strip_prefix(dir)
                    .expect("inside")
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                found.insert(name, std::fs::read(&path).expect("readable"));
            }
        }
    }
    found
}

/// One documented entry of a page: its heading, its printed example, and the message the page
/// predicts for a key it does not list, when it predicts one.
struct Documented {
    heading: String,
    example: String,
    refused_with: Option<String>,
}

/// The line a page predicts a refusal on: "Any other key is refused with `message`, …".
const PREDICTS: &str = "Any other key is refused with `";

/// The `## ` entries of a page that print an example.
fn documented(page: &str) -> Vec<Documented> {
    let mut entries = Vec::new();
    let mut current: Option<Documented> = None;
    let mut in_example = false;
    for line in page.lines() {
        if in_example {
            if line == "```" {
                in_example = false;
            } else if let Some(entry) = &mut current {
                entry.example.push_str(line);
                entry.example.push('\n');
            }
            continue;
        }
        if let Some(heading) = line.strip_prefix("## ") {
            entries.extend(current.take());
            current = Some(Documented {
                heading: heading.trim_matches('`').to_owned(),
                example: String::new(),
                refused_with: None,
            });
        } else if line == "```yaml" {
            in_example = current
                .as_ref()
                .is_some_and(|entry| entry.example.is_empty());
        } else if let (Some(rest), Some(entry)) = (line.strip_prefix(PREDICTS), &mut current) {
            entry.refused_with = rest.split('`').next().map(str::to_owned);
        }
    }
    entries.extend(current);
    entries
        .into_iter()
        .filter(|entry| !entry.example.is_empty())
        .collect()
}

fn page(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).expect("the page exists")
}

/// The example with one key no file lists added to the mapping its first line opens: a top-level
/// key, or the first key's value one level down.
fn with_unknown_key(example: &str, nested: bool) -> String {
    let indent = if nested { "  " } else { "" };
    format!("{example}{indent}not_a_field: 1\n")
}

/// Decodes a content file's sections, each by the capability that owns its key, as the loader does.
struct Sections;

impl<'de> Visitor<'de> for Sections {
    type Value = usize;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a mapping of sections")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<usize, A::Error> {
        let mut decoded = 0;
        while let Some(key) = map.next_key::<String>()? {
            let (owner, _) = Capability::owning_section(&key)
                .ok_or_else(|| serde::de::Error::custom(format!("no pack owns `{key}`")))?;
            owner.decode_section(&mut map)?;
            decoded += 1;
        }
        Ok(decoded)
    }
}

fn decode_sections(yaml: &str) -> Result<usize, String> {
    serde_saphyr::with_deserializer_from_str(yaml, |file| file.deserialize_map(Sections))
        .map_err(|error| error.to_string())
}

/// What a pack that decodes its section only in its owner's context answers a decode without one
/// (`owns_section_in_context!`).
const IN_CONTEXT_ONLY: &str = "decoded only in its owner's context";

/// The repository's Entity Packs.
const ENTITIES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../entities");

/// `example` (a person file's one section) decoded in its owner's context: in a world that enables
/// the owner and requires every repository Entity Pack holding one of the owner's data directories,
/// with one person carrying it, validated by the real binary. Nothing here names a pack.
fn decode_in_world(example: &str, n: usize) -> Result<(), String> {
    let key = example
        .lines()
        .next()
        .and_then(|line| line.strip_suffix(':'))
        .ok_or("the example opens its section's key")?;
    let (owner, _) = Capability::owning_section(key).ok_or("a pack owns the section")?;
    let directories = owner.data().map_or(&[][..], |data| data.directories);
    let mut packs: Vec<_> = std::fs::read_dir(ENTITIES)
        .expect("the Entity Packs")
        .map(|entry| entry.expect("an entry").path())
        .filter(|pack| {
            directories
                .iter()
                .any(|dir| pack.join(dir.as_str()).is_dir())
        })
        .collect();
    packs.sort();
    let mut requires = String::new();
    for pack in &packs {
        let manifest = std::fs::read_to_string(pack.join("pack.yaml")).expect("a pack.yaml");
        let field = |name: &str| {
            manifest
                .lines()
                .find_map(|line| line.strip_prefix(name))
                .map(str::trim)
                .unwrap_or_else(|| panic!("{} states {name}", pack.display()))
                .to_owned()
        };
        let version = field("version: ");
        let mut parts = version.split('.');
        let (major, minor) = (parts.next().unwrap_or("0"), parts.next().unwrap_or("0"));
        let range = if major == "0" {
            format!("0.{minor}")
        } else {
            major.to_owned()
        };
        requires.push_str(&format!("  {}: \"^{range}\"\n", field("id: ")));
    }
    let root = fresh(&format!("reference-in-context-{n}"));
    let world = root.join("reference-in-context");
    let requires = if requires.is_empty() {
        String::new()
    } else {
        format!("requires:\n{requires}")
    };
    for (file, text) in [
        (
            "world.yaml",
            format!(
                "world:\n  id: reference-in-context\n  name: Reference\n  version: 0.1.0\n  \
                 license: MIT\nmineworld: \"^0.1\"\n{requires}systems:\n  - presence\n  - {}\n\
                 places:\n  - home\npopulation: [ana]\nsettlement: {{ kind: none }}\n",
                owner.id().as_str()
            ),
        ),
        ("places/home.yaml", "tags: [home]\n".to_owned()),
        (
            "people/ana.yaml",
            format!("location: {{ place: home }}\n{example}"),
        ),
    ] {
        let path = world.join(file);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(path, text).expect("written");
    }
    let validated = mineworld(&["validate", text(&world), "--packs", ENTITIES]);
    if validated.status.success() {
        Ok(())
    } else {
        Err(stderr(&validated))
    }
}

/// The capability whose id is `id`.
fn capability(id: &str) -> Capability {
    AVAILABLE
        .into_iter()
        .find(|capability| capability.id().as_str() == id)
        .unwrap_or_else(|| panic!("no installed pack is `{id}`"))
}

fn decode_configuration(heading: &str, yaml: &str) -> Result<(), String> {
    let id = heading
        .strip_prefix("configure/")
        .and_then(|file| file.strip_suffix(".yaml"))
        .unwrap_or_else(|| panic!("not a configure file: {heading}"));
    match id {
        "classes" => serde_saphyr::from_str::<EntityClasses>(yaml)
            .map(drop)
            .map_err(|error| error.to_string()),
        // Its decoder is the loader's own; the World Pack test below loads the example in a world.
        "packages" => Ok(()),
        _ => {
            let owner = capability(id);
            serde_saphyr::with_deserializer_from_str(yaml, |file| owner.decode_configuration(file))
                .map(drop)
                .map_err(|error| error.to_string())
        }
    }
}

/// The reference is a function of the build: two generations are byte-equal, use `\n` only, and a
/// fresh generation passes `--check`.
#[test]
fn the_reference_is_the_same_bytes_every_time_and_checks_clean() {
    let first = generated("reference-first");
    let second = generated("reference-second");
    let (a, b) = (files(&first), files(&second));
    assert_eq!(a, b);
    assert!(a.keys().any(|path| path.starts_with("schema/")), "{a:?}");
    for (path, bytes) in &a {
        assert!(!bytes.contains(&b'\r'), "{path} has a carriage return");
    }
    let checked = mineworld(&["reference", "--check", text(&first)]);
    assert!(checked.status.success(), "{}", stderr(&checked));
}

/// `--check` exits 1 and names every file that differs, is missing or is extra.
#[test]
fn check_names_every_file_that_differs_is_missing_or_is_extra() {
    let dir = generated("reference-drift");
    let sections = dir.join("sections.md");
    let mut edited = std::fs::read_to_string(&sections).expect("readable");
    edited.push_str("A hand edit.\n");
    std::fs::write(&sections, edited).expect("written");
    std::fs::remove_file(dir.join("cli.md")).expect("removed");
    std::fs::write(dir.join("notes.md"), "mine\n").expect("written");

    let checked = mineworld(&["reference", "--check", text(&dir)]);
    assert_eq!(checked.status.code(), Some(1), "{}", stdout(&checked));
    let complaint = stderr(&checked);
    for line in [
        "differs  sections.md",
        "missing  cli.md",
        "extra    notes.md",
    ] {
        assert!(complaint.contains(line), "{line}: {complaint}");
    }
    assert_eq!(complaint.matches("\n  ").count(), 3, "{complaint}");
}

/// `--out` writes nothing into a directory holding a file it did not generate, and removes the
/// generated files it no longer writes.
#[test]
fn out_owns_only_what_it_generates() {
    let dir = fresh("reference-foreign");
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("notes.txt"), "mine").expect("written");
    let refused = mineworld(&["reference", "--out", text(&dir)]);
    assert!(!refused.status.success());
    assert!(
        stderr(&refused).contains("notes.txt"),
        "{}",
        stderr(&refused)
    );
    assert_eq!(files(&dir).len(), 1, "nothing was written");
    assert_eq!(
        std::fs::read_to_string(dir.join("notes.txt")).expect("kept"),
        "mine"
    );

    let dir = generated("reference-stale");
    let banner = page(&dir, "README.md");
    std::fs::write(dir.join("old-page.md"), banner).expect("written");
    let schema = std::fs::read(dir.join("schema/world.schema.json")).expect("readable");
    std::fs::write(dir.join("schema/old.schema.json"), schema).expect("written");
    let rewritten = mineworld(&["reference", "--out", text(&dir)]);
    assert!(rewritten.status.success(), "{}", stderr(&rewritten));
    assert!(!dir.join("old-page.md").exists());
    assert!(!dir.join("schema/old.schema.json").exists());
    let checked = mineworld(&["reference", "--check", text(&dir)]);
    assert!(checked.status.success(), "{}", stderr(&checked));
}

/// The cognition half's files (PR 25b) share the directory: `--out` keeps them as they are, `--check`
/// does not call them extra, and the index links them.
#[test]
fn the_cognition_halfs_files_are_left_to_it() {
    let dir = generated("reference-cognition");
    std::fs::write(dir.join("cognition-toml.md"), "the cognition page\n").expect("written");
    std::fs::write(dir.join("schema/cognition-toml.schema.json"), "{}\n").expect("written");
    let checked = mineworld(&["reference", "--check", text(&dir)]);
    assert!(checked.status.success(), "{}", stderr(&checked));
    let rewritten = mineworld(&["reference", "--out", text(&dir)]);
    assert!(rewritten.status.success(), "{}", stderr(&rewritten));
    assert_eq!(page(&dir, "cognition-toml.md"), "the cognition page\n");
    assert_eq!(page(&dir, "schema/cognition-toml.schema.json"), "{}\n");
    let index = page(&dir, "README.md");
    for link in [
        "(cognition-toml.md)",
        "(schema/cognition-toml.schema.json)",
        "(memory-scopes.md)",
        "(schema/memory-scopes.schema.json)",
    ] {
        assert!(index.contains(link), "{link}: {index}");
    }
}

/// AC-25a-4: every section, configuration and Interaction List section of every pack in the installed
/// set has its entry, on the page for its kind, and no entry names anything else.
#[test]
fn every_section_and_configuration_of_the_installed_set_appears() {
    let dir = generated("reference-packs");
    let headings = |name: &str| -> Vec<String> {
        page(&dir, name)
            .lines()
            .filter_map(|line| line.strip_prefix("## "))
            .map(|heading| heading.trim_matches('`').to_owned())
            .collect()
    };
    let mut sections = Vec::new();
    let mut configured = Vec::new();
    let mut interactions = Vec::new();
    for capability in AVAILABLE {
        if let Some(section) = capability.section() {
            sections.push(section.name.as_str().to_owned());
        }
        if capability.configuration().is_some() {
            let file = format!("configure/{}.yaml", capability.id());
            if capability.interaction_section().is_some() {
                interactions.push(file);
            } else {
                configured.push(file);
            }
        }
    }
    assert!(
        sections.contains(&"fishing".to_owned()),
        "the third-party pack's section"
    );
    assert_eq!(headings("sections.md"), sections);
    let mut expected = vec![
        "configure/classes.yaml".to_owned(),
        "configure/packages.yaml".to_owned(),
    ];
    expected.extend(configured);
    assert_eq!(headings("configure.md"), expected);
    let mut listed = headings("interaction-list.md");
    assert_eq!(listed.remove(0), "The shape every section shares");
    assert_eq!(listed, interactions);
}

/// AC-25a-6: every section's and every configuration's printed example decodes through the installed
/// set's own decoder, and the same example with one unlisted key is refused with the page's predicted
/// message — at least one such refusal per page.
#[test]
fn every_printed_example_decodes_and_an_unlisted_key_is_refused_as_predicted() {
    let dir = generated("reference-examples");
    let mut refused = 0;
    let mut in_context = 0;
    for entry in documented(&page(&dir, "sections.md")) {
        let decoded = match decode_sections(&entry.example) {
            // A section its owner decodes only in its context (`ARC-121` item 2) is decoded where
            // that context exists: in a world.
            Err(error) if error.contains(IN_CONTEXT_ONLY) => {
                decode_in_world(&entry.example, in_context).unwrap_or_else(|error| {
                    panic!("{}: {error}\n{}", entry.heading, entry.example)
                });
                in_context += 1;
                continue;
            }
            decoded => decoded
                .unwrap_or_else(|error| panic!("{}: {error}\n{}", entry.heading, entry.example)),
        };
        assert_eq!(decoded, 1, "{}", entry.heading);
        if let Some(predicted) = &entry.refused_with {
            let error =
                decode_sections(&with_unknown_key(&entry.example, true)).expect_err(&entry.heading);
            assert!(
                error.contains(predicted.as_str()),
                "{}: {error}",
                entry.heading
            );
            refused += 1;
        }
    }
    assert!(refused > 0, "sections.md predicts a refusal");

    for name in ["configure.md", "interaction-list.md"] {
        let mut refused = 0;
        for entry in documented(&page(&dir, name)) {
            decode_configuration(&entry.heading, &entry.example)
                .unwrap_or_else(|error| panic!("{}: {error}\n{}", entry.heading, entry.example));
            let Some(predicted) = &entry.refused_with else {
                continue;
            };
            if entry.heading != "configure/packages.yaml" {
                let error =
                    decode_configuration(&entry.heading, &with_unknown_key(&entry.example, false))
                        .expect_err(&entry.heading);
                assert!(
                    error.contains(predicted.as_str()),
                    "{}: {error}",
                    entry.heading
                );
                refused += 1;
            }
        }
        assert!(refused > 0, "{name} predicts a refusal");
    }
}

/// AC-25a-1, -4, -6 for `decisions.yaml` (step-25 C6): the page has the one entry; its printed example
/// is read and checked by the decision list's own reader, with this build's installed models, against a
/// world whose seat is the example's; the same example with one unlisted key is refused as predicted.
#[test]
fn the_decision_lists_example_is_read_and_an_unlisted_key_is_refused_as_predicted() {
    use mineworld_contracts::EntityKey;
    use mineworld_decision::{ClassRef, DecisionList, Roster, SeatEntry};

    let dir = generated("reference-decisions");
    let entries = documented(&page(&dir, "decisions.md"));
    let headings: Vec<&str> = entries.iter().map(|entry| entry.heading.as_str()).collect();
    assert_eq!(headings, ["decisions.yaml"]);
    let entry = &entries[0];

    let alice = EntityKey::new("alice").expect("a key");
    let class = |name: &str| ClassRef::new(name).expect("a class");
    let roster = Roster {
        seats: vec![SeatEntry {
            seat: alice.clone(),
            class: class("person"),
        }],
        classes: std::collections::BTreeSet::new(),
        implicit: ["person", "place", "item", "organization"]
            .into_iter()
            .map(class)
            .collect(),
        people: std::collections::BTreeSet::from([alice]),
    };
    let read = |yaml: &str| {
        DecisionList::<mineworld_installed_models::Model>::parse(
            yaml,
            Path::new("decisions.yaml"),
            "decisions.yaml",
            &roster,
        )
        .map(drop)
        .map_err(|error| error.to_string())
    };
    read(&entry.example).unwrap_or_else(|error| panic!("{error}\n{}", entry.example));
    let predicted = entry
        .refused_with
        .as_ref()
        .expect("the page predicts a refusal");
    let error = read(&with_unknown_key(&entry.example, false)).expect_err("refused");
    assert!(error.contains(predicted.as_str()), "{error}");
}

/// AC-25a-6 for the World Pack page and the data pack page: the printed examples, composed, are a
/// world `mineworld validate` accepts (`configure/classes.yaml` and `configure/packages.yaml` from
/// configure.md included) and a data pack `mineworld packs validate` accepts; `world.yaml` and
/// `pack.yaml` with one unlisted key are refused as predicted.
#[test]
fn the_world_pack_and_data_pack_examples_load() {
    let dir = generated("reference-world");
    let documented: BTreeMap<String, Documented> =
        ["world-pack.md", "configure.md", "pack-manifest.md"]
            .into_iter()
            .flat_map(|name| documented(&page(&dir, name)))
            .map(|entry| (entry.heading.clone(), entry))
            .collect();
    let example = |heading: &str| {
        documented
            .get(heading)
            .unwrap_or_else(|| panic!("no example for {heading}: {:?}", documented.keys()))
            .example
            .clone()
    };
    let predicted = |heading: &str| {
        documented[heading]
            .refused_with
            .clone()
            .unwrap_or_else(|| panic!("{heading} predicts a refusal"))
    };
    assert_eq!(predicted("world.yaml"), UNKNOWN);

    let root = fresh("reference-world-pack");
    let world_id = example("world.yaml")
        .lines()
        .find_map(|line| line.strip_prefix("  id: "))
        .expect("the example states its id")
        .to_owned();
    let world = root.join(world_id);
    let manifest = format!(
        "{}configure:\n  - classes\n  - packages\n",
        example("world.yaml")
    );
    for (file, text) in [
        ("world.yaml", manifest.clone()),
        ("places/home.yaml", String::new()),
        ("places/shop.yaml", example("places/<key>.yaml")),
        ("people/ana.yaml", example("people/<key>.yaml")),
        ("people/ben.yaml", example("people/<key>.yaml")),
        ("configure/classes.yaml", example("configure/classes.yaml")),
        (
            "configure/packages.yaml",
            example("configure/packages.yaml"),
        ),
    ] {
        let path = world.join(file);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(path, text).expect("written");
    }
    let validated = mineworld(&["validate", text(&world)]);
    assert!(
        validated.status.success(),
        "{}\n{manifest}",
        stderr(&validated)
    );
    std::fs::write(world.join("world.yaml"), with_unknown_key(&manifest, false)).expect("written");
    let refused = mineworld(&["validate", text(&world)]);
    assert!(!refused.status.success());
    assert!(
        stderr(&refused).contains(&predicted("world.yaml")),
        "{}",
        stderr(&refused)
    );

    let packs = fresh("reference-data-pack");
    let pack_file = example("pack.yaml");
    let id = pack_file
        .lines()
        .find_map(|line| line.strip_prefix("id: "))
        .expect("the example states its id");
    let pack = packs.join(id);
    std::fs::create_dir_all(pack.join("items")).expect("a directory");
    std::fs::write(pack.join("pack.yaml"), &pack_file).expect("written");
    std::fs::write(pack.join("items/bread.yaml"), example("items/<key>.yaml")).expect("written");
    let validated = mineworld(&["packs", "validate", text(&pack)]);
    assert!(validated.status.success(), "{}", stderr(&validated));
    std::fs::write(pack.join("pack.yaml"), with_unknown_key(&pack_file, false)).expect("written");
    let refused = mineworld(&["packs", "validate", text(&pack)]);
    assert!(!refused.status.success());
    assert!(
        stderr(&refused).contains(&predicted("pack.yaml")),
        "{}",
        stderr(&refused)
    );
}
