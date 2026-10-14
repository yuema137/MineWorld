//! FA-7 — what a forgetting section cannot say is refused by the real loader, at its line and column
//! (`pr-fg-a-relationship-forgetting.md` §4, §5.6 S1–S2; `docs/MODULE_SPEC.md` §4.2).
//!
//! Scratch copies of `worlds/social-cafe`, each with one section and the classes `resident` and
//! `commuter`. The refusals named here are the new ones; the IL-e refusals keep their exact words in
//! `social_sections.rs`, `interactions.rs` and the acceptance tests, unedited.
//!
//! ```text
//! a curve name no curve has                         S1, the catalog bound
//! a curve name of the wrong form                    CurveName's own decoding
//! familiarity_decay_days set under place:           S2, decay is about a pair      M-FA7
//! known_at_most set under target:, or in a region   S2, the cap is about the holder
//! forget_after_days set under actor:                S2, conversation's, the listener's
//! known_at_most 4 or 1 501; familiarity_decay_days 0   S1, ranges, today's words
//! ```

mod headless;

use std::path::Path;

use headless::{PACK, Scratch, fresh};
use mineworld_contracts::WorldTime;
use mineworld_worldpack::WorldPack;

const CLASSES: &str = "- { class: resident, of: person, tag: resident }\n\
                       - { class: commuter, of: person, tag: commuter }\n";

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

/// A scratch social-cafe configuring `classes` and the section `key` as `text`.
fn copy(name: &str, key: &str, text: &str) -> Scratch {
    let scratch = fresh(name).within("social-cafe");
    copy_dir(Path::new(PACK), &scratch);
    let manifest = scratch.join("world.yaml");
    let original = std::fs::read_to_string(&manifest).expect("reads");
    std::fs::write(
        &manifest,
        format!("{original}\nconfigure:\n  - classes\n  - {key}\n"),
    )
    .expect("writes");
    std::fs::create_dir_all(scratch.join("configure")).expect("writable");
    std::fs::write(scratch.join("configure/classes.yaml"), CLASSES).expect("writes");
    std::fs::write(scratch.join(format!("configure/{key}.yaml")), text).expect("writes");
    scratch
}

/// What the loader says about the copy: its refusal, or `None` when it reads and loads.
fn refusal(name: &str, key: &str, text: &str) -> Option<String> {
    let scratch = copy(name, key, text);
    match WorldPack::read(&scratch) {
        Err(error) => Some(error.to_string()),
        Ok(pack) => pack
            .load(WorldTime::EPOCH)
            .err()
            .map(|error| error.to_string()),
    }
}

#[test]
fn what_a_forgetting_section_cannot_say_is_refused_at_its_line_and_column() {
    for (name, key, text, expected) in [
        (
            "fa7-unknown-curve",
            "relationships",
            "parameters:\n  - { familiarity_decay: fast }\n",
            "'familiarity_decay' is one of none, linear, half-life, power, not fast",
        ),
        (
            "fa7-malformed-curve",
            "relationships",
            "parameters:\n  - { regard_decay: Half Life }\n",
            "'Half Life' is not a curve name",
        ),
        (
            "fa7-place-decay",
            "relationships",
            // `place` is the implicit class of every place, so only the role is wrong here.
            "parameters:\n  - { place: place, familiarity_decay_days: 30 }\n",
            "'familiarity_decay_days' of 'relationships' may not be scoped by the role 'place' \
             (its roles: actor, target)",
        ),
        (
            "fa7-region-cap",
            "relationships",
            "regions:\n  cafe:\n    parameters: [ { known_at_most: 10 } ]\n",
            "a region may not set 'known_at_most': 'relationships' does not let a place scope it \
             (its roles: actor)",
        ),
        (
            "fa7-target-cap",
            "relationships",
            "parameters:\n  - { target: resident, known_at_most: 10 }\n",
            "'known_at_most' of 'relationships' may not be scoped by the role 'target' (its roles: \
             actor)",
        ),
        (
            "fa7-actor-forget",
            "conversation",
            "parameters:\n  - { actor: resident, forget_after_days: 2 }\n",
            "'forget_after_days' of 'conversation' may not be scoped by the role 'actor' (its \
             roles: target)",
        ),
        (
            "fa7-cap-low",
            "relationships",
            "parameters:\n  - { known_at_most: 4 }\n",
            "'known_at_most' is 5 … 1500, not 4",
        ),
        (
            "fa7-cap-high",
            "relationships",
            "parameters:\n  - { known_at_most: 1501 }\n",
            "'known_at_most' is 5 … 1500, not 1501",
        ),
        (
            "fa7-days-zero",
            "relationships",
            "parameters:\n  - { familiarity_decay_days: 0 }\n",
            "'familiarity_decay_days' is 1 … 3650, not 0",
        ),
    ] {
        let refused = refusal(name, key, text).unwrap_or_else(|| panic!("{name}: accepted"));
        assert!(refused.contains(expected), "{name}: {refused}");
        // Every offending entry is on the file's last line: the second, or the third for a region.
        let line = if text.starts_with("regions") { 3 } else { 2 };
        assert!(
            refused.contains(&format!("error: line {line} column ")),
            "{name}: at its line and column: {refused}"
        );
    }
}

/// The same fields, scoped as their owners allow, read and load: the refusals above are about the
/// role, not the field.
#[test]
fn the_same_fields_scoped_as_their_owners_allow_are_accepted() {
    for (name, key, text) in [
        (
            "fa7-ok-relationships",
            "relationships",
            "parameters:\n  - { familiarity_decay: half-life, familiarity_decay_days: 30, \
             regard_decay: power, known_at_most: 40 }\n  - { target: resident, \
             familiarity_decay_days: 90 }\n  - { actor: commuter, known_at_most: 15 }\n\
             regions:\n  cafe:\n    parameters: [ { spoke_familiarity: 20 } ]\n",
        ),
        (
            "fa7-ok-conversation",
            "conversation",
            "parameters:\n  - { target: resident, forget_after_days: 7 }\n",
        ),
    ] {
        assert_eq!(refusal(name, key, text), None, "{name}");
    }
}
