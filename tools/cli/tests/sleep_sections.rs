//! AC-Na-8 — the `sleep` pack through the real loader (`pr-s35-na-sleep.md` §5; `docs/MODULE_SPEC.md`
//! §4.2).
//!
//! Scratch copies of `worlds/social-cafe` and `worlds/lakeside`, made at test time; the reference worlds
//! are not edited.
//!
//! ```text
//! configure/sleep.yaml without sleep in systems:     refused, naming sleep
//! longest_minutes: 900                               refused at its line and column, naming 60 … 840
//! a rule naming wake                                 refused: wake is not a section action
//! a class the world does not define                  refused, naming the class
//! social-cafe and lakeside with sleep appended       load; every person asks to sleep an hour where
//!                                                    they stand, and is accepted or refused
//!                                                    PermissionDenied; a day later nobody is asleep
//! ```

mod headless;

use std::path::Path;

use headless::{Scratch, fresh};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityType, Rejection, WorldTime,
};
use mineworld_sleep::{Asleep, Sleep};
use mineworld_worldpack::WorldPack;

const WORLDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds");

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

/// A scratch copy of `world`, with `sleep` appended to `systems:` when `enabled`, and with
/// `configure/sleep.yaml` (and `configure/classes.yaml`) when given.
fn copy(name: &str, world: &str, enabled: bool, section: Option<&str>, classes: &str) -> Scratch {
    let scratch = fresh(name).within(world);
    copy_dir(&Path::new(WORLDS).join(world), &scratch);
    let manifest = scratch.join("world.yaml");
    let mut text = std::fs::read_to_string(&manifest).expect("reads");
    if enabled {
        // `appearance` is the last system of both worlds; `sleep` follows it.
        assert!(
            text.contains("  - appearance\n"),
            "{world} lists appearance"
        );
        text = text.replacen("  - appearance\n", "  - appearance\n  - sleep\n", 1);
    }
    if let Some(section) = section {
        text.push_str("\nconfigure:\n  - classes\n  - sleep\n");
        std::fs::create_dir_all(scratch.join("configure")).expect("writable");
        std::fs::write(scratch.join("configure/classes.yaml"), classes).expect("writes");
        std::fs::write(scratch.join("configure/sleep.yaml"), section).expect("writes");
    }
    std::fs::write(&manifest, text).expect("writes");
    scratch
}

/// What the loader says: its refusal, or `None` when the copy reads and loads.
fn refusal(scratch: &Scratch) -> Option<String> {
    match WorldPack::read(scratch) {
        Err(error) => Some(error.to_string()),
        Ok(pack) => pack
            .load(WorldTime::EPOCH)
            .err()
            .map(|error| error.to_string()),
    }
}

const CLASSES: &str = "- { class: bench, of: place, tag: park }\n";

#[test]
fn what_a_sleep_section_cannot_say_is_refused_by_name_and_position() {
    let off = copy(
        "sleep-section-not-enabled",
        "social-cafe",
        false,
        Some("parameters:\n  - { woken_by_talk: false }\n"),
        CLASSES,
    );
    let refused = refusal(&off).expect("refused");
    assert!(refused.contains("'sleep'"), "{refused}");
    assert!(refused.contains("does not enable"), "{refused}");

    for (name, text, expected) in [
        (
            "sleep-longest-high",
            "parameters:\n  - { longest_minutes: 900 }\n",
            "'longest_minutes' is 60 … 840, not 900",
        ),
        (
            "sleep-rule-wake",
            "rules:\n  - { action: wake, effect: forbid }\n",
            "wake",
        ),
    ] {
        let scratch = copy(name, "social-cafe", true, Some(text), CLASSES);
        let refused = refusal(&scratch).unwrap_or_else(|| panic!("{name}: accepted"));
        assert!(refused.contains(expected), "{name}: {refused}");
        assert!(
            refused.contains("error: line 2 column "),
            "{name}: at its line and column: {refused}"
        );
    }

    let undefined = copy(
        "sleep-class-undefined",
        "social-cafe",
        true,
        Some("rules:\n  - { action: sleep, place: hammock, effect: permit }\n"),
        CLASSES,
    );
    let refused = refusal(&undefined).expect("refused");
    assert!(refused.contains("hammock"), "{refused}");

    let sound = copy(
        "sleep-section-sound",
        "social-cafe",
        true,
        Some("rules:\n  - { action: sleep, place: bench, effect: permit }\n"),
        CLASSES,
    );
    assert_eq!(refusal(&sound), None);
}

/// Both reference worlds load with `sleep` appended and run a day: every person may ask, the answer is
/// the pack's, and every sleep ends at its alarm.
#[test]
fn social_cafe_and_lakeside_load_and_run_a_day_with_sleep() {
    for world in ["social-cafe", "lakeside"] {
        let scratch = copy(&format!("sleep-runs-{world}"), world, true, None, "");
        let mut loaded = WorldPack::read(&scratch)
            .expect("reads")
            .load(WorldTime::EPOCH)
            .unwrap_or_else(|error| panic!("{world}: {error}"));
        let people: Vec<_> = loaded
            .world()
            .read()
            .entities()
            .filter(|entity| entity.entity_type() == EntityType::Person)
            .map(|entity| entity.id())
            .collect();
        assert!(!people.is_empty(), "{world} has people");
        let at = WorldTime::from_seconds(60);
        let world_mut = loaded.world_mut();
        let _ = world_mut.advance_to(at).expect("advances");
        let mut slept = 0;
        for (n, person) in people.iter().enumerate() {
            let intent = ActionIntent::new(
                ActionId::from_raw(n as u64 + 1),
                *person,
                ActionRecord::new::<Sleep>(serde_json::to_vec(&Sleep::new(60)).expect("encodes")),
                at,
            );
            match world_mut.dispatch(&intent, at).expect("answered").result() {
                ActionResult::Accepted { .. } => slept += 1,
                ActionResult::Rejected(Rejection::PermissionDenied) => {}
                other => panic!("{world}: {other:?}"),
            }
        }
        assert_eq!(
            world_mut.read().components::<Asleep>().count(),
            slept,
            "{world}"
        );
        let _ = world_mut
            .advance_to(WorldTime::from_seconds(86_400))
            .expect("a day passes");
        assert_eq!(
            world_mut.read().components::<Asleep>().count(),
            0,
            "{world}: every alarm rang"
        );
    }
}
