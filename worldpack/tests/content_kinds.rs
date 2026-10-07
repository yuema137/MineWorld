//! Items and organizations are content kinds of a World Pack (`DECISIONS.md` `ARC-36`): read from
//! `items/` and `organizations/`, created after every person, and changing nothing a world without
//! them would have had.
//!
//! The fixture is written at runtime and is neutral content (a lantern, a pebble, a chess club), so the
//! test proves the format's capability without any pack that would use it.

use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityKey, EntityType, Tag, WorldTime};
use mineworld_worldpack::{LoadedWorld, WorldPack};

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring key")
}

/// Writes a pack called `id`: two places, two people placed in them, and — when `with_kinds` — two
/// item kinds (listed out of key order on purpose) and one organization, each a tags-only file.
fn write_pack(id: &str, with_kinds: bool) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(id);
    let _ = std::fs::remove_dir_all(&root);
    let write = |relative: &str, contents: &str| {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a writable directory");
        std::fs::write(&path, contents).expect("a writable file");
    };
    let kinds = if with_kinds {
        "items:\n  - pebble\n  - lantern\norganizations:\n  - chess-club\n"
    } else {
        ""
    };
    write(
        "world.yaml",
        &format!(
            "world:\n  id: {id}\n  name: Things\nsystems:\n  - presence\nplaces:\n  - park\n  - cafe\npopulation:\n  - bob\n  - alice\n{kinds}"
        ),
    );
    write("places/cafe.yaml", "tags: [cafe]\n");
    write("places/park.yaml", "tags: [park]\n");
    write(
        "people/alice.yaml",
        "tags: [regular]\nlocation:\n  place: cafe\n",
    );
    write(
        "people/bob.yaml",
        "tags: [regular]\nlocation:\n  place: park\n",
    );
    if with_kinds {
        write(
            "items/lantern.yaml",
            "tags: [light]\nnote: Hangs by the door.\n",
        );
        write("items/pebble.yaml", "tags: [stone]\n");
        write("organizations/chess-club.yaml", "tags: [club]\n");
    }
    root
}

fn load(root: &Path) -> (WorldPack, LoadedWorld) {
    let pack = WorldPack::read(root).expect("the fixture reads");
    let loaded = pack.load(WorldTime::EPOCH).expect("the fixture loads");
    (pack, loaded)
}

#[test]
fn items_and_organizations_are_allocated_after_people_in_key_order() {
    let (pack, loaded) = load(&write_pack("content-kinds", true));
    let (_, bare) = load(&write_pack("content-kinds-bare", false));

    // Read: the two kinds, in key order, with what their files said.
    let items: Vec<&str> = pack.items().keys().map(EntityKey::as_str).collect();
    assert_eq!(items, ["lantern", "pebble"]);
    let organizations: Vec<&str> = pack.organizations().keys().map(EntityKey::as_str).collect();
    assert_eq!(organizations, ["chess-club"]);

    // Loaded: places, then people, then items, then organizations, each in key order.
    let ids: Vec<(&str, u64)> = loaded
        .ids()
        .iter()
        .map(|(key, id)| (key.as_str(), id.raw()))
        .collect();
    let mut by_id = ids.clone();
    by_id.sort_by_key(|(_, id)| *id);
    assert_eq!(
        by_id,
        [
            ("cafe", 1),
            ("park", 2),
            ("alice", 3),
            ("bob", 4),
            ("lantern", 5),
            ("pebble", 6),
            ("chess-club", 7),
        ],
        "every id a world without items or organizations has stays put"
    );

    let read = loaded.world().read();
    for (name, entity_type, tag, path) in [
        ("lantern", EntityType::Item, "light", "items/lantern.yaml"),
        ("pebble", EntityType::Item, "stone", "items/pebble.yaml"),
        (
            "chess-club",
            EntityType::Organization,
            "club",
            "organizations/chess-club.yaml",
        ),
    ] {
        let entity = read
            .entity(loaded.id(&key(name)).expect("the key resolves"))
            .expect("the entity exists");
        assert_eq!(entity.entity_type(), entity_type, "{name}");
        assert!(
            entity.tags().contains(&Tag::new(tag).expect("a tag")),
            "{name} carries its authored tags: {:?}",
            entity.tags()
        );
        let provenance = entity
            .metadata()
            .expect("an authored entity has provenance");
        assert_eq!(provenance.source_pack, "content-kinds");
        assert_eq!(provenance.source_path, path);
    }
    let lantern = read
        .entity(loaded.id(&key("lantern")).expect("resolves"))
        .expect("exists");
    assert_eq!(
        lantern
            .metadata()
            .and_then(|provenance| provenance.authoring_note.as_deref()),
        Some("Hangs by the door.")
    );

    // Tags-only files add no genesis fact: the genesis is the bare pack's, id for id, byte for byte.
    let genesis = |world: &LoadedWorld| {
        world
            .genesis()
            .iter()
            .map(|fact| {
                (
                    fact.id(),
                    fact.event_type().clone(),
                    fact.payload().payload().clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert!(
        !genesis(&bare).is_empty(),
        "the comparison compares something"
    );
    assert_eq!(genesis(&loaded), genesis(&bare));
}
