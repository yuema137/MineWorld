//! AC-Nb-1 — the `needs` pack through the real loader (`pr-s35-nb-needs.md` §5; AC-S35-3).
//!
//! Scratch copies of `worlds/social-cafe` and `worlds/lakeside`, made at test time; the reference worlds
//! are not edited. Social Café has no item system, so a case needing `consumption` appends `item`,
//! `inventory` and `consumption` to its copy; Lakeside has them, and a case that must lack
//! `consumption` removes it from its copy.
//!
//! ```text
//! a needs: section without needs enabled                  refused, naming needs
//! needs: { nourishment } without consumption               refused, SectionRequiresSystem
//! needs enabled, no sleep, no configure/needs.yaml         refused, DefaultConfigurationRequiresSystem
//! restore: { tiredness: [sleeping] } written without sleep refused, ConfigurationRequiresSystem
//! thresholds [600, 400, 900]; rest_per_hour 5;             refused at their line and column
//!   thresholds under actor:
//! nourishment: { hunger: 1001 }; hunger with nourishment   refused at their line
//! restore: { tiredness: [] } and no sleep                  loads, and runs a day
//! ```

mod headless;

use std::path::Path;

use headless::{Scratch, fresh};
use mineworld_contracts::WorldTime;
use mineworld_needs::Needs;
use mineworld_packages::PackRoots;
use mineworld_worldpack::WorldPack;

const WORLDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds");

/// Each world's first person and an item kind of its own.
const SOCIAL: (&str, &str, &str) = ("social-cafe", "alice", "cafe-box");
const LAKESIDE: (&str, &str, &str) = ("lakeside", "alba", "fish");

const CLASSES: &str = "- { class: night, of: person, tag: night }\n";

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copied");
        }
    }
}

/// One case: which systems to append (after `appearance`) or remove, what to append to the first
/// person's and the item's files, and `configure/needs.yaml` if any.
#[derive(Default)]
struct Case<'a> {
    append: &'a [&'a str],
    remove: &'a [&'a str],
    person: Option<&'a str>,
    item: Option<&'a str>,
    section: Option<&'a str>,
}

fn read_lf(path: &Path) -> String {
    std::fs::read_to_string(path)
        .expect("reads")
        .replace("\r\n", "\n")
}

fn append(path: &Path, text: &str) {
    let mut body = read_lf(path);
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(text);
    body.push('\n');
    std::fs::write(path, body).expect("writes");
}

fn copy(name: &str, (world, person, item): (&str, &str, &str), case: &Case<'_>) -> Scratch {
    let scratch = fresh(name).within(world);
    copy_dir(&Path::new(WORLDS).join(world), &scratch);
    let manifest = scratch.join("world.yaml");
    let mut text = read_lf(&manifest);
    assert!(
        text.contains("  - appearance\n"),
        "{world} lists appearance"
    );
    let appended: String = case
        .append
        .iter()
        .map(|system| format!("  - {system}\n"))
        .collect();
    text = text.replacen(
        "  - appearance\n",
        &format!("  - appearance\n{appended}"),
        1,
    );
    for system in case.remove {
        let line = format!("  - {system}\n");
        assert!(text.contains(&line), "{world} lists {system}");
        text = text.replacen(&line, "", 1);
    }
    if let Some(section) = case.section {
        text = mineworld_test_support::manifest::configure_also(&text, &["classes", "needs"]);
        std::fs::create_dir_all(scratch.join("configure")).expect("writable");
        std::fs::write(scratch.join("configure/classes.yaml"), CLASSES).expect("writes");
        std::fs::write(scratch.join("configure/needs.yaml"), section).expect("writes");
    }
    std::fs::write(&manifest, text).expect("writes");
    if let Some(section) = case.person {
        append(&scratch.join(format!("people/{person}.yaml")), section);
    }
    if let Some(section) = case.item {
        append(&scratch.join(format!("items/{item}.yaml")), section);
    }
    scratch
}

/// The copy read as `mineworld` reads it, with the repository's pack roots (`ARC-77`).
fn read(scratch: &Scratch) -> Result<WorldPack, mineworld_worldpack::PackError> {
    let repository = Path::new(WORLDS).join("..");
    let roots = PackRoots::new(
        vec![
            repository.join("entities"),
            repository.join("presentation/mineworld-default"),
        ],
        None,
    )
    .expect("the roots exist");
    WorldPack::read_with(scratch.to_path_buf(), &roots)
}

/// What the loader says: its refusal, or `None` when the copy reads and loads.
fn refusal(scratch: &Scratch) -> Option<String> {
    match read(scratch) {
        Err(error) => Some(error.to_string()),
        Ok(pack) => pack
            .load(WorldTime::EPOCH)
            .err()
            .map(|error| error.to_string()),
    }
}

/// The systems a world needs appended for `consumption`, and those for `needs` with it.
fn eating(world: &str) -> &'static [&'static str] {
    if world == "social-cafe" {
        &["item", "inventory", "consumption"]
    } else {
        &[]
    }
}

fn with_needs(world: &str, sleep: bool) -> Vec<&'static str> {
    let mut systems = eating(world).to_vec();
    if sleep {
        systems.push("sleep");
    }
    systems.push("needs");
    systems
}

fn refused(name: &str, world: (&str, &str, &str), case: &Case<'_>, expected: &[&str]) {
    let scratch = copy(&format!("{name}-{}", world.0), world, case);
    let refused = refusal(&scratch).unwrap_or_else(|| panic!("{name} {}: accepted", world.0));
    for needle in expected {
        assert!(refused.contains(needle), "{name} {}: {refused}", world.0);
    }
}

#[test]
fn a_needs_section_without_needs_is_refused_by_name() {
    for world in [SOCIAL, LAKESIDE] {
        let case = Case {
            person: Some("needs: {}"),
            ..Case::default()
        };
        refused(
            "needs-not-enabled",
            world,
            &case,
            &["'needs'", "does not enable"],
        );
    }
}

#[test]
fn nourishment_without_consumption_is_refused_by_name() {
    for world in [SOCIAL, LAKESIDE] {
        let remove: &[&str] = if world.0 == "lakeside" {
            &["consumption"]
        } else {
            &[]
        };
        let case = Case {
            append: &["sleep", "needs"],
            remove,
            item: Some("needs: { nourishment: { hunger: 100 } }"),
            section: Some("parameters:\n  - { restore: { hunger: [] } }\n"),
            ..Case::default()
        };
        refused(
            "needs-nourishment-alone",
            world,
            &case,
            &[&format!("gives '{}'", world.2), "needs 'consumption'"],
        );
    }
}

#[test]
fn the_default_restorer_without_sleep_is_refused_by_name() {
    for world in [SOCIAL, LAKESIDE] {
        let append = with_needs(world.0, false);
        let case = Case {
            append: &append,
            ..Case::default()
        };
        refused(
            "needs-default-no-sleep",
            world,
            &case,
            &[
                "world.yaml enables 'needs', whose default configuration needs 'sleep'",
                "configure/needs.yaml",
            ],
        );
    }
}

#[test]
fn a_written_sleeping_restorer_without_sleep_is_refused_by_name() {
    for world in [SOCIAL, LAKESIDE] {
        let append = with_needs(world.0, false);
        let case = Case {
            append: &append,
            section: Some("parameters:\n  - { restore: { tiredness: [sleeping] } }\n"),
            ..Case::default()
        };
        refused(
            "needs-written-no-sleep",
            world,
            &case,
            &["the 'needs' system's configuration needs 'sleep'"],
        );
    }
}

#[test]
fn bad_values_are_refused_at_their_line_and_column() {
    for world in [SOCIAL, LAKESIDE] {
        let append = with_needs(world.0, true);
        for (name, section, person, item, expected) in [
            (
                "needs-thresholds-order",
                Some("parameters:\n  - { thresholds: [600, 400, 900] }\n"),
                None,
                None,
                "strictly increasing",
            ),
            (
                "needs-rest-low",
                Some("parameters:\n  - { rest_per_hour: 5 }\n"),
                None,
                None,
                "'rest_per_hour' is 10 … 1000, not 5",
            ),
            (
                "needs-thresholds-actor",
                Some("parameters:\n  - { actor: night, thresholds: [300, 500, 800] }\n"),
                None,
                None,
                "thresholds",
            ),
            (
                "needs-nourishment-high",
                None,
                None,
                Some("needs:\n  nourishment: { hunger: 1001 }"),
                "0 … 1 000, not 1001",
            ),
            (
                "needs-both-forms",
                None,
                Some("needs: { hunger: 1, nourishment: { hunger: 1 } }"),
                None,
                "never both",
            ),
        ] {
            let case = Case {
                append: &append,
                section,
                person,
                item,
                ..Case::default()
            };
            refused(name, world, &case, &[expected, "line "]);
        }
    }
}

/// With `restore: { tiredness: [] }` stated, a world without `sleep` loads: tiredness only rises, and
/// the world says so. It runs a day.
#[test]
fn an_explicit_tiredness_without_sleep_loads_and_runs_a_day() {
    for world in [SOCIAL, LAKESIDE] {
        let append = with_needs(world.0, false);
        let case = Case {
            append: &append,
            person: Some("needs: {}"),
            item: Some("needs: { nourishment: { hunger: 300 } }"),
            section: Some("parameters:\n  - { restore: { tiredness: [] } }\n"),
            ..Case::default()
        };
        let scratch = copy(&format!("needs-accepted-{}", world.0), world, &case);
        let mut loaded = read(&scratch)
            .expect("reads")
            .load(WorldTime::EPOCH)
            .unwrap_or_else(|error| panic!("{}: {error}", world.0));
        let world_mut = loaded.world_mut();
        let _ = world_mut
            .advance_to(WorldTime::from_seconds(86_400))
            .expect("a day passes");
        assert_eq!(
            world_mut.read().components::<Needs>().count(),
            1,
            "{}: one person with needs",
            world.0
        );
    }
}
