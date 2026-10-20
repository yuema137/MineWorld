//! A World Pack's `hosting:` block states how a host paces the world it serves (`DECISIONS.md` `ARC-69`
//! item 9): a whole number of world seconds per wall second from 1 to 24, read by `mineworld server` and by
//! nothing else. Anything above 24 is refused by name at load, because a pack cannot make a world
//! fast-forward; an unknown key in the block is refused like any other unknown field.

use mineworld_worldpack::{PackError, WorldPack};

/// A one-place, one-person pack whose `world.yaml` carries `hosting` after its identity, in a scratch named
/// after its id (a pack's id is its directory's name), removed when the test ends (DEP-29).
struct Scratch(mineworld_test_support::Scratch);

impl Scratch {
    fn world(id: &str, hosting: &str) -> Self {
        let root = mineworld_test_support::scratch!(id);
        for directory in ["people", "places"] {
            std::fs::create_dir_all(root.join(directory)).expect("scratch directory");
        }
        let manifest = format!(
            "world:\n  id: {id}\n  name: Hosting\n{hosting}systems: [presence]\n\
             places: [cafe]\npopulation: [alice]\n"
        );
        std::fs::write(root.join("world.yaml"), manifest).expect("world.yaml");
        std::fs::write(root.join("places/cafe.yaml"), "tags: [cafe]\n").expect("place");
        std::fs::write(
            root.join("people/alice.yaml"),
            "tags: [barista]\nlocation:\n  place: cafe\n",
        )
        .expect("person");
        Self(root)
    }

    fn read(&self) -> Result<WorldPack, PackError> {
        WorldPack::read(&self.0)
    }
}

#[test]
fn a_stated_day_length_scale_is_read_and_changes_nothing_a_pack_loads() {
    let scratch = Scratch::world("hosting-stated", "hosting:\n  time_scale: 12\n");
    let pack = scratch.read().expect("reads");
    assert_eq!(
        pack.hosting_time_scale().map(std::num::NonZeroU32::get),
        Some(12)
    );
    pack.load(mineworld_contracts::WorldTime::EPOCH).expect(
        "a pack that states a host pace still loads: the key is host content, not world state",
    );
}

#[test]
fn a_pack_that_states_no_hosting_says_none() {
    let scratch = Scratch::world("hosting-absent", "");
    let pack = scratch.read().expect("reads");
    assert_eq!(pack.hosting_time_scale(), None);
}

#[test]
fn a_scale_above_a_day_in_24_hours_is_refused_by_file_and_key() {
    for value in ["25", "0"] {
        let text = format!("hosting:\n  time_scale: {value}\n");
        let scratch = Scratch::world("hosting-out-of-range", &text);
        let refusal = scratch.read().expect_err("out of range");
        let message = refusal.to_string();
        assert!(
            matches!(refusal, PackError::HostingTimeScale { value: found, .. } if found.to_string() == value),
            "{value}: refused as out of range, not as {message}"
        );
        assert!(
            message.contains("world.yaml") && message.contains("hosting.time_scale"),
            "the refusal names the file and the key: {message}"
        );
    }
}

#[test]
fn an_unknown_key_in_the_hosting_block_is_refused() {
    let scratch = Scratch::world(
        "hosting-unknown",
        "hosting:\n  time_scale: 12\n  fast_forward: true\n",
    );
    assert!(
        scratch.read().is_err(),
        "a block that is not a time scale is not silently accepted"
    );
}
