//! A bad pack is refused **by name**, saying what is wrong and where.
//!
//! This is the acceptance criterion this crate exists for as much as loading is. A World Pack is
//! hand-written, so being wrong is its normal state during authoring, and the difference between a
//! usable format and an unusable one is entirely in what happens then: a panic, a silently ignored
//! field or *invalid world* all fail equally.
//!
//! Each test writes a real pack directory to disk, reads it, and asserts the variant and the values
//! the refusal carries — not the message text, which is prose and may be improved. What the message
//! *must* contain is checked once, in [`every_refusal_names_the_file_it_is_about`], because "and
//! where" is half the requirement.

use mineworld_test_support::Scratch;

use mineworld_contracts::{EntityKey, SystemId};
use mineworld_kernel::KernelError;
use mineworld_worldpack::{ContentKind, Declared, PackError, WorldPack};

/// A pack directory written for one test, under cargo's own temporary directory for this target.
///
/// Named after the test, and named after the pack's own id: a pack's id must be its directory's name,
/// so a fixture that got that wrong would be refused for the wrong reason.
struct Fixture {
    root: Scratch,
}

impl Fixture {
    /// An empty directory called `id`, with `people/` and `places/` in it.
    fn new(id: &str) -> Self {
        let root = mineworld_test_support::scratch!(id);
        std::fs::create_dir_all(root.join("people")).expect("a writable temporary directory");
        std::fs::create_dir_all(root.join("places")).expect("a writable temporary directory");
        Self { root }
    }

    /// A pack with a legal manifest naming one place and one person, both written. The starting point
    /// for every test that breaks exactly one thing.
    fn sound(id: &str) -> Self {
        let fixture = Self::new(id);
        fixture.manifest(
            "
systems:
  - presence
places:
  - cafe
population:
  - alice
",
        );
        fixture.write("places/cafe.yaml", "tags: [cafe]\n");
        fixture.write(
            "people/alice.yaml",
            "tags: [barista]\nlocation:\n  place: cafe\n",
        );
        fixture
    }

    /// Writes `world.yaml`: the identity block, which every pack needs, plus `body`.
    fn manifest(&self, body: &str) {
        let id = self.id();
        self.write(
            "world.yaml",
            &format!("world:\n  id: {id}\n  name: A Test World\n{body}"),
        );
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        std::fs::write(&path, contents).unwrap_or_else(|error| {
            panic!("the fixture must be writable: {} — {error}", path.display())
        });
    }

    fn remove(&self, relative: &str) {
        std::fs::remove_file(self.root.join(relative)).expect("the fixture file exists");
    }

    fn id(&self) -> String {
        self.root
            .file_name()
            .expect("a named directory")
            .to_string_lossy()
            .into_owned()
    }

    fn read(&self) -> Result<WorldPack, PackError> {
        WorldPack::read(&self.root)
    }

    /// The refusal this pack produces. Panics if it loads, because a test that silently passed on a
    /// pack that turned out to be valid would assert nothing.
    fn refusal(&self) -> PackError {
        match self.read() {
            Ok(_) => panic!("this pack must be refused, and it loaded"),
            Err(error) => error,
        }
    }
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring key")
}

#[test]
fn a_path_that_is_not_a_directory_is_refused_by_name() {
    let refusal = WorldPack::read("worlds/social-cafe/world.yaml")
        .expect_err("a World Pack is a directory, not a file");

    assert!(
        matches!(refusal, PackError::NotAPackDirectory { .. }),
        "got: {refusal}",
    );
}

#[test]
fn a_pack_with_no_manifest_is_refused_by_name() {
    let fixture = Fixture::sound("no-manifest");
    fixture.remove("world.yaml");

    let refusal = fixture.refusal();

    assert!(
        matches!(refusal, PackError::FileMissing { ref path } if path.ends_with("world.yaml")),
        "got: {refusal}",
    );
}

#[test]
fn a_declared_person_with_no_file_is_refused_by_name() {
    let fixture = Fixture::sound("missing-person-file");
    fixture.remove("people/alice.yaml");

    let refusal = fixture.refusal();

    assert!(
        matches!(
            refusal,
            PackError::ContentFileMissing { ref key, kind: ContentKind::Person, ref path }
                if *key == self::key("alice") && path.ends_with("people/alice.yaml")
        ),
        "got: {refusal}",
    );
}

#[test]
fn a_person_file_the_manifest_does_not_list_is_refused_by_name() {
    let fixture = Fixture::sound("undeclared-person-file");
    fixture.write("people/carol.yaml", "tags: [regular]\n");

    let refusal = fixture.refusal();

    // The commonest authoring mistake there is: the file is written, the list is not updated, and a
    // loader that ignored the file would leave the author with a missing person and no reason why.
    assert!(
        matches!(
            refusal,
            PackError::ContentFileNotDeclared {
                ref key,
                kind: ContentKind::Person,
                list: Declared::Population,
                ..
            } if key == "carol"
        ),
        "got: {refusal}",
    );
}

#[test]
fn an_unknown_system_is_refused_by_name_and_lists_the_ones_that_exist() {
    let fixture = Fixture::new("unknown-system");
    // A name no pack will ever take, so installing a real pack can never break this test (F-10).
    fixture.manifest("systems:\n  - presence\n  - no-such-system\n");

    let refusal = fixture.refusal();

    let PackError::UnknownSystem { system, available } = &refusal else {
        panic!("got: {refusal}");
    };
    assert_eq!(
        *system,
        SystemId::new("no-such-system").expect("a legal name")
    );
    assert!(
        available.contains("presence") && available.contains("conversation"),
        "an author who typed a system that does not exist is shown the ones that do: {available}",
    );
}

#[test]
fn a_system_enabled_twice_is_refused_by_name() {
    let fixture = Fixture::new("system-twice");
    fixture.manifest("systems:\n  - presence\n  - presence\n");

    let refusal = fixture.refusal();

    assert!(
        matches!(refusal, PackError::SystemDeclaredTwice { ref system }
            if *system == SystemId::new("presence").expect("a legal name")),
        "got: {refusal}",
    );
}

#[test]
fn a_key_declared_twice_is_refused_by_name_and_says_where_both_were() {
    let fixture = Fixture::new("duplicate-key");
    fixture.manifest(
        "
systems:
  - presence
places:
  - cafe
population:
  - cafe
",
    );

    let refusal = fixture.refusal();

    assert!(
        matches!(
            refusal,
            PackError::KeyDeclaredTwice {
                ref key,
                first: Declared::Places,
                second: Declared::Population,
            } if *key == self::key("cafe")
        ),
        "got: {refusal}",
    );
}

#[test]
fn a_person_in_a_place_that_does_not_exist_is_refused_by_name() {
    let fixture = Fixture::sound("person-in-no-such-place");
    fixture.write(
        "people/alice.yaml",
        "tags: [barista]\nlocation:\n  place: park\n",
    );

    let refusal = fixture.refusal();

    let PackError::PersonInUnknownPlace {
        person,
        place,
        known,
        path,
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(*person, key("alice"));
    assert_eq!(*place, key("park"));
    assert!(known.contains("cafe"), "the places it does have: {known}");
    assert!(path.ends_with("people/alice.yaml"));
}

#[test]
fn a_bad_field_is_refused_by_name_and_located_in_its_file() {
    let fixture = Fixture::sound("bad-field");
    fixture.write(
        "people/alice.yaml",
        "tags: [barista]\nlocatoin:\n  place: cafe\n",
    );

    let refusal = fixture.refusal();

    let PackError::Malformed { path, detail, .. } = &refusal else {
        panic!("got: {refusal}");
    };
    assert!(path.ends_with("people/alice.yaml"));
    assert!(
        detail.contains("locatoin"),
        "a misspelled field is named, not ignored: {detail}",
    );
    assert!(
        detail.contains('2'),
        "and located — line 2 is where it was written: {detail}",
    );
}

#[test]
fn a_field_of_the_wrong_shape_is_refused_by_name() {
    let fixture = Fixture::sound("wrong-shape");
    // A position is millimetres as an integer. There are no floats anywhere in MineWorld, because a
    // float in the event log would make a replay platform-dependent (`AC-12`, `DD-5`).
    fixture.write(
        "people/alice.yaml",
        "location:\n  place: cafe\n  position:\n    x: 1.5\n    y: 0\n",
    );

    let refusal = fixture.refusal();

    assert!(
        matches!(refusal, PackError::Malformed { ref path, .. }
            if path.ends_with("people/alice.yaml")),
        "got: {refusal}",
    );
}

#[test]
fn a_repeated_yaml_key_is_refused_rather_than_resolved() {
    let fixture = Fixture::new("duplicate-mapping-key");
    fixture.write(
        "world.yaml",
        "world:\n  id: duplicate-mapping-key\n  name: T\nsystems: [presence]\nsystems: [conversation]\n",
    );

    let refusal = fixture.refusal();

    // The other reading of "a duplicate key": one YAML mapping with the same key twice. A parser that
    // silently took the last wins would give the author a world composed of a system they deleted.
    let PackError::Malformed { detail, .. } = &refusal else {
        panic!("got: {refusal}");
    };
    assert!(
        detail.contains("duplicate") && detail.contains("systems"),
        "the repeated key is named and located: {detail}",
    );
}

#[test]
fn an_illegal_name_is_refused_in_the_contracts_own_words() {
    let fixture = Fixture::sound("illegal-key");
    fixture.manifest(
        "
systems:
  - presence
places:
  - cafe
population:
  - Alice
",
    );

    let refusal = fixture.refusal();

    // The identifier rule lives in `contracts/src/ids.rs` and is applied by the contract's own
    // deserializer, so the complaint is the contract's rather than a second implementation's.
    let PackError::Malformed { path, detail, .. } = &refusal else {
        panic!("got: {refusal}");
    };
    assert!(path.ends_with("world.yaml"));
    assert!(
        detail.to_lowercase().contains("lowercase") || detail.contains("Alice"),
        "the rule that was broken, or at least the value that broke it: {detail}",
    );
}

#[test]
fn a_pack_whose_id_is_not_its_directory_is_refused_by_name() {
    let fixture = Fixture::new("not-lakewood");
    fixture.write(
        "world.yaml",
        "world:\n  id: lakewood\n  name: Lakewood\nsystems: []\n",
    );

    let refusal = fixture.refusal();

    assert!(
        matches!(refusal, PackError::PackIdIsNotItsDirectory { ref declared, ref directory }
            if declared == "lakewood" && directory == "not-lakewood"),
        "got: {refusal}",
    );
}

#[test]
fn a_seat_that_is_not_one_of_the_people_is_refused_by_name() {
    let fixture = Fixture::sound("seat-is-nobody");
    fixture.manifest(
        "
systems:
  - presence
places:
  - cafe
population:
  - alice
seats:
  - cafe
",
    );

    let refusal = fixture.refusal();

    assert!(
        matches!(refusal, PackError::SeatIsNotOneOfThePeople { ref seat }
            if *seat == key("cafe")),
        "got: {refusal}",
    );
}

#[test]
fn content_that_needs_a_system_the_pack_did_not_enable_is_refused_by_name() {
    let fixture = Fixture::sound("location-without-presence");
    fixture.manifest(
        "
systems: []
places:
  - cafe
population:
  - alice
",
    );

    let refusal = fixture.refusal();

    // The pack places somebody without enabling the system that owns placement. Refused as the
    // author's missing line rather than accepted as a world where everybody is quietly nowhere.
    assert!(
        matches!(
            refusal,
            PackError::ContentNeedsASystem { ref subject, content: "location", ref system, .. }
                if *subject == key("alice") && *system == SystemId::new("presence").expect("legal")
        ),
        "got: {refusal}",
    );
}

#[test]
fn a_composition_the_kernel_refuses_is_reported_in_the_kernels_own_words() {
    let fixture = Fixture::sound("conversation-without-presence");
    fixture.manifest(
        "
systems:
  - conversation
places:
  - cafe
population:
  - alice
",
    );
    // No location, so the pack-level capability check passes and the refusal comes from the kernel:
    // `conversation` declares a dependency on `presence`, and a world is composed or it is not.
    fixture.write("people/alice.yaml", "tags: [barista]\n");

    let pack = fixture.read().expect("the pack itself is consistent");
    let refusal = match pack.load(mineworld_contracts::WorldTime::EPOCH) {
        Ok(_) => panic!("a world with conversation and no presence cannot be composed"),
        Err(error) => error,
    };

    assert!(
        matches!(
            refusal,
            PackError::Composition {
                source: KernelError::SystemDependencyMissing { .. }
            }
        ),
        "got: {refusal}",
    );
}

#[test]
fn every_refusal_names_the_file_it_is_about() {
    // "By name" is half the requirement; "and where" is the other half. Every refusal that concerns a
    // file must mention that file in its own message, because the author is reading the message and
    // not matching on the variant.
    let cases: Vec<(&str, PackError)> = vec![
        ("world.yaml", {
            let fixture = Fixture::sound("named-missing-manifest");
            fixture.remove("world.yaml");
            fixture.refusal()
        }),
        ("people/alice.yaml", {
            let fixture = Fixture::sound("named-missing-person");
            fixture.remove("people/alice.yaml");
            fixture.refusal()
        }),
        ("people/alice.yaml", {
            let fixture = Fixture::sound("named-bad-field");
            fixture.write("people/alice.yaml", "nmae: Alice\n");
            fixture.refusal()
        }),
        ("people/alice.yaml", {
            let fixture = Fixture::sound("named-unknown-place");
            fixture.write("people/alice.yaml", "location:\n  place: park\n");
            fixture.refusal()
        }),
        ("items/lantern.yaml", {
            with_kinds("named-missing-item", false, "items:\n  - lantern\n").refusal()
        }),
        ("organizations/chess-club.yaml", {
            let fixture = with_kinds("named-undeclared-organization", false, "");
            fixture.write("organizations/chess-club.yaml", "tags: [club]\n");
            fixture.refusal()
        }),
    ];

    for (expected, refusal) in cases {
        let message = refusal.to_string();
        assert!(
            message.contains(expected),
            "a refusal about {expected} must say so: {message}",
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Passages (`MODULE_SPEC.md` §4.1): a doorway between two places, owned by `movement`.
// ---------------------------------------------------------------------------------------------

/// A sound pack with two places, `cafe` and `street`, `movement` enabled, and `cafe`'s file stating
/// `passages` as given.
fn with_passages(id: &str, cafe: &str) -> Fixture {
    let fixture = Fixture::sound(id);
    fixture.manifest(
        "
systems:
  - presence
  - movement
places:
  - cafe
  - street
population:
  - alice
",
    );
    fixture.write("places/cafe.yaml", cafe);
    fixture.write("places/street.yaml", "tags: [street]\n");
    fixture
}

const DOORWAY: &str = "
passages:
  - to: street
    here: { x: 4600, y: 2000 }
    there: { x: 0, y: 2000 }
";

#[test]
fn a_doorway_between_two_places_loads_into_both_places_passages() {
    use mineworld_contracts::{LocalPosition, Millimetres, PlaceId};
    use mineworld_movement::{PassageOpened, Passages};

    let fixture = with_passages("doorway", DOORWAY);
    let loaded = fixture
        .read()
        .expect("a doorway between two declared places reads")
        .load(mineworld_contracts::WorldTime::EPOCH)
        .expect("and loads");

    let place = |name: &str| {
        PlaceId::new(
            loaded.id(&key(name)).expect("declared"),
            mineworld_contracts::EntityType::Place,
        )
        .expect("a place")
    };
    let (cafe, street) = (place("cafe"), place("street"));
    let at = |x, y| LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y));
    let passages = |of: PlaceId| {
        loaded
            .world()
            .components()
            .get::<Passages>(of.entity_id())
            .cloned()
            .expect("movement reduced the passage into this place")
    };

    let opened: Vec<_> = loaded
        .genesis()
        .iter()
        .filter(|fact| {
            *fact.event_type() == <PassageOpened as mineworld_contracts::Event>::EVENT_TYPE
        })
        .collect();
    println!(
        "genesis: {} fact(s), {} passage-opened",
        loaded.genesis().len(),
        opened.len()
    );
    assert_eq!(opened.len(), 1, "one passage, stated once");
    assert_eq!(
        *loaded.genesis()[0].event_type(),
        <PassageOpened as mineworld_contracts::Event>::EVENT_TYPE,
        "passages are stated before anybody is placed"
    );

    let out = passages(cafe);
    let way_out = out.to(street).expect("the café opens onto the street");
    assert_eq!(
        (way_out.here(), way_out.there()),
        (Some(at(4_600, 2_000)), Some(at(0, 2_000)))
    );
    let back = passages(street);
    let way_back = back.to(cafe).expect("and the street onto the café");
    assert_eq!(
        (way_back.here(), way_back.there()),
        (Some(at(0, 2_000)), Some(at(4_600, 2_000)))
    );
}

#[test]
fn a_passage_to_an_undeclared_place_is_refused_by_name() {
    let fixture = with_passages("passage-unknown", "passages:\n  - to: park\n");
    let refusal = fixture.refusal();
    assert!(
        matches!(
            refusal,
            PackError::PassageToUnknownPlace { ref place, ref to, .. }
                if *place == key("cafe") && *to == key("park")
        ),
        "got: {refusal}",
    );
}

#[test]
fn a_passage_from_a_place_to_itself_is_refused_by_name() {
    let fixture = with_passages("passage-itself", "passages:\n  - to: cafe\n");
    let refusal = fixture.refusal();
    assert!(
        matches!(refusal, PackError::PassageToItself { ref place, .. } if *place == key("cafe")),
        "got: {refusal}",
    );
}

#[test]
fn a_passage_stated_twice_is_refused_by_name_whichever_files_state_it() {
    // Once in each of the two files: a passage holds both ways, so this is the same doorway twice.
    let both_files = with_passages("passage-both-files", "passages:\n  - to: street\n");
    both_files.write("places/street.yaml", "passages:\n  - to: cafe\n");
    // Twice in one file.
    let one_file = with_passages(
        "passage-one-file",
        "passages:\n  - to: street\n  - to: street\n",
    );

    for fixture in [both_files, one_file] {
        let refusal = fixture.refusal();
        assert!(
            matches!(
                refusal,
                PackError::PassageStatedTwice { ref first, ref second, .. }
                    if *first == key("cafe") && *second == key("street")
            ),
            "{}: got: {refusal}",
            fixture.id()
        );
    }
}

#[test]
fn a_passage_in_a_pack_without_movement_is_refused_by_name() {
    let fixture = with_passages("passage-without-movement", DOORWAY);
    fixture.manifest(
        "
systems:
  - presence
places:
  - cafe
  - street
population:
  - alice
",
    );
    let refusal = fixture.refusal();
    assert!(
        matches!(
            refusal,
            PackError::ContentNeedsASystem { ref subject, content: "passage", ref system, .. }
                if *subject == key("cafe") && *system == SystemId::new("movement").expect("legal")
        ),
        "got: {refusal}",
    );
}

#[test]
fn every_passage_refusal_names_the_file_that_states_it() {
    let cases = [
        (
            "places/cafe.yaml",
            with_passages("named-passage-unknown", "passages:\n  - to: park\n"),
        ),
        (
            "places/cafe.yaml",
            with_passages("named-passage-itself", "passages:\n  - to: cafe\n"),
        ),
        ("places/street.yaml", {
            let fixture = with_passages("named-passage-twice", "passages:\n  - to: street\n");
            fixture.write("places/street.yaml", "passages:\n  - to: cafe\n");
            fixture
        }),
        (
            "places/cafe.yaml",
            with_passages(
                "named-passage-field",
                "passages:\n  - to: street\n    door: 1\n",
            ),
        ),
    ];
    for (expected, fixture) in cases {
        let message = fixture.refusal().to_string();
        println!("{}: {message}", fixture.id());
        assert!(
            message.contains(expected),
            "a refusal about {expected} must say so: {message}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Sections (`MODULE_SPEC.md` §4.1 rule 6, `DECISIONS.md` `ARC-31`): a top-level key of a content
// file that a System Pack owns, decoded by that pack's own type.
// ---------------------------------------------------------------------------------------------

/// A sound pack with `naming` enabled (or not), and alice's file as given.
fn with_naming(id: &str, enabled: bool, alice: &str) -> Fixture {
    let fixture = Fixture::sound(id);
    let naming = if enabled { "  - naming\n" } else { "" };
    fixture.manifest(&format!(
        "
systems:
  - presence
{naming}places:
  - cafe
population:
  - alice
"
    ));
    fixture.write("people/alice.yaml", alice);
    fixture
}

#[test]
fn a_section_is_read_by_its_owner_when_the_owner_is_enabled() {
    let fixture = with_naming(
        "section-sound",
        true,
        "tags: [barista]\nname: Alice Moreau\nlocation:\n  place: cafe\n",
    );
    let pack = fixture
        .read()
        .expect("a named person in a world that enables naming");
    let loaded = pack
        .load(mineworld_contracts::WorldTime::EPOCH)
        .expect("loads");
    let named: Vec<&str> = loaded
        .genesis()
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect();
    assert_eq!(
        named,
        ["arrived", "named"],
        "the arrival, then the section's own fact"
    );
}

#[test]
fn a_section_whose_owner_is_not_enabled_is_refused_naming_the_owner() {
    let fixture = with_naming(
        "section-owner-off",
        false,
        "name: Alice Moreau\nlocation:\n  place: cafe\n",
    );

    let refusal = fixture.refusal();

    let PackError::ContentNeedsASystem {
        subject,
        content,
        system,
        path,
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(*subject, key("alice"));
    assert_eq!(*content, "name");
    assert_eq!(*system, SystemId::new("naming").expect("an id"));
    assert!(path.ends_with("people/alice.yaml"));
}

#[test]
fn a_misspelled_section_is_an_unknown_field_listing_the_sections_too() {
    let fixture = with_naming(
        "section-misspelled",
        true,
        "tags: [barista]\nnmae: Alice Moreau\n",
    );

    let refusal = fixture.refusal();

    let PackError::Malformed { path, detail, .. } = &refusal else {
        panic!("got: {refusal}");
    };
    println!("{detail}");
    assert!(path.ends_with("people/alice.yaml"));
    assert!(detail.contains("nmae"), "the key is named: {detail}");
    assert!(
        detail.contains("`name`"),
        "and the legal keys include the sections this build's packs own: {detail}"
    );
    assert!(detail.contains("line 2 column 1"), "located: {detail}");
}

#[test]
fn an_invalid_section_is_refused_by_its_owners_type_at_its_line() {
    let fixture = with_naming(
        "section-invalid",
        true,
        "tags: [barista]\nlocation:\n  place: cafe\nname: \"\"\n",
    );

    let refusal = fixture.refusal();

    let PackError::Malformed { path, detail, .. } = &refusal else {
        panic!("got: {refusal}");
    };
    println!("{detail}");
    assert!(path.ends_with("people/alice.yaml"));
    assert!(
        detail.contains("a name must be"),
        "the owner's own words, not the loader's: {detail}"
    );
    assert!(
        detail.contains("line 4 column 7"),
        "located at the value, straight from the stream (DEP-10): {detail}"
    );
}

#[test]
fn a_section_in_a_kind_of_file_its_owner_does_not_allow_is_refused() {
    let fixture = with_naming("section-in-a-place", true, "location:\n  place: cafe\n");
    fixture.write(
        "places/cafe.yaml",
        "tags: [cafe]\nname: The Copper Kettle\n",
    );

    let refusal = fixture.refusal();

    let PackError::SectionNotCarriedHere {
        subject,
        section,
        kind,
        carried_by,
        path,
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(*subject, key("cafe"));
    assert_eq!(section.as_str(), "name");
    assert_eq!(*kind, ContentKind::Place);
    assert_eq!(carried_by, "person");
    assert!(path.ends_with("places/cafe.yaml"));
    assert!(refusal.to_string().contains("places/cafe.yaml"));
}

/// A sound pack with `schedule` enabled (or not), places `cafe` and `park`, and alice's file as given.
fn with_schedule(id: &str, enabled: bool, alice: &str) -> Fixture {
    let fixture = Fixture::sound(id);
    let schedule = if enabled { "  - schedule\n" } else { "" };
    fixture.manifest(&format!(
        "
systems:
  - presence
{schedule}places:
  - cafe
  - park
population:
  - alice
  - bob
"
    ));
    fixture.write("places/park.yaml", "tags: [park]\n");
    fixture.write("people/bob.yaml", "location:\n  place: cafe\n");
    fixture.write("people/alice.yaml", alice);
    fixture
}

const ALICE_WITH: &str = "location:\n  place: cafe\nroutine:\n";

#[test]
fn a_routine_naming_an_undeclared_place_is_refused_by_name() {
    let fixture = with_schedule(
        "routine-unknown-place",
        true,
        &format!(
            "{ALICE_WITH}  - {{ from: \"06:00\", place: cafe, label: work }}\n  - {{ from: \"18:00\", place: beach, label: swim }}\n"
        ),
    );

    let refusal = fixture.refusal();

    let PackError::SectionNamesUnknownEntity {
        subject,
        section,
        key: named,
        expected,
        path,
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(*subject, key("alice"));
    assert_eq!(section.as_str(), "routine");
    assert_eq!(*named, key("beach"));
    assert_eq!(*expected, mineworld_contracts::EntityType::Place);
    assert!(path.ends_with("people/alice.yaml"));
    assert!(refusal.to_string().contains("people/alice.yaml"));
}

#[test]
fn a_routine_naming_a_person_where_a_place_belongs_is_refused_by_name() {
    let fixture = with_schedule(
        "routine-person-as-place",
        true,
        &format!(
            "{ALICE_WITH}  - {{ from: \"06:00\", place: cafe, label: work }}\n  - {{ from: \"18:00\", place: bob, label: visit }}\n"
        ),
    );

    let refusal = fixture.refusal();

    assert!(
        matches!(
            &refusal,
            PackError::SectionNamesUnknownEntity { key: named, expected, .. }
                if *named == key("bob") && *expected == mineworld_contracts::EntityType::Place
        ),
        "got: {refusal}"
    );
}

#[test]
fn a_routine_in_a_world_without_schedule_is_refused_naming_schedule() {
    let fixture = with_schedule(
        "routine-owner-off",
        false,
        &format!(
            "{ALICE_WITH}  - {{ from: \"06:00\", place: cafe, label: work }}\n  - {{ from: \"18:00\", place: park, label: walk }}\n"
        ),
    );

    let refusal = fixture.refusal();

    assert!(
        matches!(
            &refusal,
            PackError::ContentNeedsASystem { content: "routine", system, .. }
                if *system == SystemId::new("schedule").expect("an id")
        ),
        "got: {refusal}"
    );
}

#[test]
fn an_overlapping_routine_is_refused_by_schedules_own_rule_at_its_line() {
    let fixture = with_schedule(
        "routine-overlapping",
        true,
        &format!(
            "{ALICE_WITH}  - {{ from: \"18:00\", place: cafe, label: work }}\n  - {{ from: \"06:00\", place: park, label: walk }}\n"
        ),
    );

    let refusal = fixture.refusal();

    let PackError::Malformed { path, detail, .. } = &refusal else {
        panic!("got: {refusal}");
    };
    println!("{detail}");
    assert!(path.ends_with("people/alice.yaml"));
    assert!(
        detail.contains("strictly increasing times, but 06:00 follows 18:00"),
        "schedule's own words: {detail}"
    );
    assert!(
        detail.contains("line 4"),
        "located at the section: {detail}"
    );
}

// ---------------------------------------------------------------------------------------------
// Items and organizations (`MODULE_SPEC.md` §4.1, `DECISIONS.md` `ARC-36`): two more content kinds,
// refused by the same standard. Each fixture creates its own `items/` and `organizations/` (F-23).
// ---------------------------------------------------------------------------------------------

/// A sound pack whose manifest adds `extra` (with `naming` enabled when asked), with empty `items/`
/// and `organizations/` directories.
fn with_kinds(id: &str, naming: bool, extra: &str) -> Fixture {
    let fixture = Fixture::sound(id);
    let naming = if naming { "  - naming\n" } else { "" };
    fixture.manifest(&format!(
        "
systems:
  - presence
{naming}places:
  - cafe
population:
  - alice
{extra}"
    ));
    for directory in ["items", "organizations"] {
        std::fs::create_dir_all(fixture.root.join(directory)).expect("a writable directory");
    }
    fixture
}

#[test]
fn a_declared_item_with_no_file_is_refused_by_name() {
    let fixture = with_kinds("missing-item-file", false, "items:\n  - lantern\n");

    let refusal = fixture.refusal();

    assert!(
        matches!(
            refusal,
            PackError::ContentFileMissing { ref key, kind: ContentKind::Item, ref path }
                if *key == self::key("lantern") && path.ends_with("items/lantern.yaml")
        ),
        "got: {refusal}",
    );
}

#[test]
fn an_organization_file_no_list_declares_is_refused_by_name() {
    let fixture = with_kinds("undeclared-organization-file", false, "");
    fixture.write("organizations/chess-club.yaml", "tags: [club]\n");

    let refusal = fixture.refusal();

    assert!(
        matches!(
            refusal,
            PackError::ContentFileNotDeclared {
                ref key,
                kind: ContentKind::Organization,
                list: Declared::Organizations,
                ref path,
            } if key == "chess-club" && path.ends_with("organizations/chess-club.yaml")
        ),
        "got: {refusal}",
    );
}

#[test]
fn a_key_declared_as_a_place_and_an_item_is_refused_naming_both_lists() {
    let fixture = with_kinds("place-and-item", false, "items:\n  - cafe\n");

    let refusal = fixture.refusal();

    assert!(
        matches!(
            refusal,
            PackError::KeyDeclaredTwice {
                ref key,
                first: Declared::Places,
                second: Declared::Items,
            } if *key == self::key("cafe")
        ),
        "keys are one namespace across the four lists: {refusal}",
    );
}

#[test]
fn a_key_declared_as_a_person_and_an_organization_is_refused_naming_both_lists() {
    let fixture = with_kinds(
        "person-and-organization",
        false,
        "organizations:\n  - alice\n",
    );

    let refusal = fixture.refusal();

    assert!(
        matches!(
            refusal,
            PackError::KeyDeclaredTwice {
                ref key,
                first: Declared::Population,
                second: Declared::Organizations,
            } if *key == self::key("alice")
        ),
        "got: {refusal}",
    );
}

#[test]
fn a_field_an_item_does_not_have_is_refused_at_its_line() {
    let fixture = with_kinds("item-with-a-location", false, "items:\n  - lantern\n");
    fixture.write(
        "items/lantern.yaml",
        "tags: [light]\nlocation:\n  place: cafe\n",
    );

    let refusal = fixture.refusal();

    let PackError::Malformed { path, detail, .. } = &refusal else {
        panic!("got: {refusal}");
    };
    println!("{detail}");
    assert!(path.ends_with("items/lantern.yaml"));
    assert!(detail.contains("location"), "the key is named: {detail}");
    assert!(
        detail.contains("`tags`") && detail.contains("`note`"),
        "and the fields an item file does have are listed: {detail}"
    );
    assert!(detail.contains("line 2 column 1"), "located: {detail}");
}

#[test]
fn a_section_an_item_may_not_carry_is_refused_naming_the_kinds_that_may() {
    let fixture = with_kinds("item-with-a-name", true, "items:\n  - lantern\n");
    fixture.write("items/lantern.yaml", "tags: [light]\nname: The Lantern\n");

    let refusal = fixture.refusal();

    let PackError::SectionNotCarriedHere {
        subject,
        section,
        kind,
        carried_by,
        path,
    } = &refusal
    else {
        panic!("got: {refusal}");
    };
    assert_eq!(*subject, key("lantern"));
    assert_eq!(section.as_str(), "name");
    assert_eq!(*kind, ContentKind::Item);
    assert_eq!(carried_by, "person");
    assert!(path.ends_with("items/lantern.yaml"));
}
