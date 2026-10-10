//! IB-4 — a real pack's section, scoped by region and by class (step-18-interaction-list §12.5;
//! `docs/DECISIONS.md` `ARC-63`, `ARC-64`).
//!
//! Scratch copies of `worlds/social-cafe`, made at test time, configure `conversation`'s `gap` and are
//! read and loaded by the real loader; scripted `talk` requests go through `World::dispatch`. Two talks
//! 400 s apart are one conversation when the gap that applies is 3 600 s, and two when it is the
//! compiled 300 s. Alice and Bob stand in the café, Dev and Erin in the park; Bob carries the tag
//! `regular`.
//!
//! ```text
//! region   regions: { cafe: { parameters: [ { gap: 3600 } ] } } — the café's pair talks on, the
//!          park's does not                                  M-IB4a (lookups ignore the region) fails it
//! class    classes { regular: person tagged regular }, parameters [ { actor: regular, gap: 3600 } ] —
//!          only Bob's second talk continues; and a place tagged `regular` is not the person class
//!          `regular`: [ { place: regular, gap: 3600 } ] leaves the park's pair at the compiled gap
//!                                                           M-IB4b (classes ignore `of`) fails it
//! ```

use std::path::Path;

use mineworld_contracts::Event;
use mineworld_contracts::{ActionId, ActionIntent, ActionRecord, ActionResult, WorldTime};
use mineworld_conversation::{ConversationStarted, Talk, Utterance};
use mineworld_worldpack::WorldPack;
use mineworld_worldpack::load::LoadedWorld;

const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../worlds/social-cafe");

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copied");
        }
    }
}

/// A scratch copy of social-cafe configured with `configure:` (the keys) and these files, loaded.
fn loaded(
    name: &str,
    keys: &[&str],
    files: &[(&str, &str)],
) -> (mineworld_test_support::Scratch, LoadedWorld) {
    let scratch = mineworld_test_support::scratch!(name).within("social-cafe");
    copy(Path::new(PACK), scratch.path());
    let manifest = std::fs::read_to_string(scratch.join("world.yaml")).expect("readable");
    let configure: String = keys.iter().map(|key| format!("  - {key}\n")).collect();
    std::fs::write(
        scratch.join("world.yaml"),
        format!("{manifest}\nconfigure:\n{configure}"),
    )
    .expect("writable");
    std::fs::create_dir_all(scratch.join("configure")).expect("writable");
    for (file, text) in files {
        std::fs::write(scratch.join(file), text).expect("writable");
    }
    let world = WorldPack::read(scratch.path())
        .expect("the configured copy reads")
        .load(WorldTime::EPOCH)
        .expect("and loads");
    (scratch, world)
}

/// `speaker` talks to `listener` twice, 400 s apart: how many conversations that started.
fn conversations(world: &mut LoadedWorld, speaker: &str, listener: &str, first: u64) -> usize {
    let id = |key: &str| {
        world
            .id(&mineworld_contracts::EntityKey::new(key).expect("a key"))
            .expect("declared")
    };
    let (speaker, listener) = (id(speaker), id(listener));
    let mut started = 0;
    // Each pair speaks in its own window, so the world's clock only moves forward.
    let start = i64::try_from(first).expect("small") * 1_000 + 10;
    for (n, at) in [(first, start), (first + 1, start + 400)] {
        let at = WorldTime::from_seconds(at);
        let talk = Talk::new(Utterance::new("hello").expect("an utterance"));
        let intent = ActionIntent::new(
            ActionId::from_raw(n),
            speaker,
            ActionRecord::new::<Talk>(serde_json::to_vec(&talk).expect("encodes")),
            at,
        )
        .with_target(listener);
        let world = world.world_mut();
        let _ = world.advance_to(at).expect("advances");
        let dispatched = world.dispatch(&intent, at).expect("answered");
        assert!(
            matches!(dispatched.result(), ActionResult::Accepted { .. }),
            "{:?}",
            dispatched.result()
        );
        started += dispatched
            .events()
            .iter()
            .filter(|fact| *fact.event_type() == ConversationStarted::EVENT_TYPE)
            .count();
    }
    started
}

#[test]
fn a_region_scopes_the_gap_to_its_place() {
    let (_scratch, mut world) = loaded(
        "interaction-sections-region",
        &["conversation"],
        &[(
            "configure/conversation.yaml",
            "regions:\n  cafe:\n    parameters: [ { gap: 3600 } ]\n",
        )],
    );
    assert_eq!(
        conversations(&mut world, "alice", "bob", 1),
        1,
        "in the café the gap is 3 600 s"
    );
    assert_eq!(
        conversations(&mut world, "dev", "erin", 3),
        2,
        "in the park it is the compiled 300 s"
    );
}

#[test]
fn a_class_scopes_the_gap_to_its_people_and_never_to_a_place_with_the_same_tag() {
    let classes = "- { class: regular, of: person, tag: regular }\n";
    let (_scratch, mut world) = loaded(
        "interaction-sections-class",
        &["classes", "conversation"],
        &[
            ("configure/classes.yaml", classes),
            (
                "configure/conversation.yaml",
                "parameters:\n  - { actor: regular, gap: 3600 }\n",
            ),
        ],
    );
    assert_eq!(
        conversations(&mut world, "bob", "alice", 1),
        1,
        "Bob is a regular: his gap is 3 600 s"
    );
    assert_eq!(
        conversations(&mut world, "dev", "erin", 3),
        2,
        "Dev is nobody's class: the compiled 300 s"
    );

    // The park carries the tag too; the class is of people, so the park is not in it.
    let (scratch, _) = loaded("interaction-sections-place-tag", &[], &[]);
    // Line endings normalized: a Windows checkout gives CRLF, and the edit below searches for "\n"
    // (step-16 §16.12 PD-p5; CRLF YAML reads the same, PD-q1).
    let park = std::fs::read_to_string(scratch.join("places/park.yaml"))
        .expect("readable")
        .replace("\r\n", "\n");
    drop(scratch);
    let tagged = park.replacen("tags:\n", "tags:\n  - regular\n", 1);
    assert_ne!(tagged, park, "the park's tags were found");
    let (_scratch, mut world) = loaded(
        "interaction-sections-place",
        &["classes", "conversation"],
        &[
            ("configure/classes.yaml", classes),
            (
                "configure/conversation.yaml",
                "parameters:\n  - { place: regular, gap: 3600 }\n",
            ),
            ("places/park.yaml", &tagged),
        ],
    );
    assert_eq!(
        conversations(&mut world, "dev", "erin", 1),
        2,
        "a place tagged `regular` is not in the person class `regular`"
    );
}
