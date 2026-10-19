//! The `sit_points:` section, decoded as the World Pack loader decodes it, and seeded and reduced at genesis
//! (`ARC-31`, `ARC-147`, AC-a-6).
//!
//! Every refusal names its problem: a list that could hold an invalid state is never read into one.

use std::collections::BTreeMap;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{EntityId, EntityKey, EntityType, Rejection, WorldTime};
use mineworld_kernel::World;
use mineworld_posture::{PostureSystem, SitPointKey, SitPoints};

const GENESIS: WorldTime = WorldTime::from_seconds(0);

fn decode(text: &str) -> Result<SitPoints, String> {
    serde_saphyr::from_str::<SitPoints>(text).map_err(|error| error.to_string())
}

/// One point, as a list item, at `(x, y)`.
fn point(key: &str, x: i32, y: i32) -> String {
    format!("  - {{ key: {key}, at: {{ x: {x}, y: {y} }}, facing: 180000 }}\n")
}

fn list(items: &[String]) -> String {
    items.concat()
}

#[test]
fn a_list_decodes_sorted_by_key_with_its_group_and_facing() {
    let text = "\
- { key: window-bench.2, group: window-bench, at: { x: 3300, y: 1160 }, facing: 180000 }
- { key: window-bench.1, group: window-bench, at: { x: 2700, y: 1160 }, facing: 180000 }
- { key: pier-bench.1, at: { x: 100, y: 200 }, facing: 90000 }
";
    let points = decode(text).expect("a valid list decodes");
    let keys: Vec<&str> = points.iter().map(|point| point.key().as_str()).collect();
    assert_eq!(keys, ["pier-bench.1", "window-bench.1", "window-bench.2"]);
    let first = points
        .get(&SitPointKey::new("window-bench.1").expect("a key"))
        .expect("held");
    assert_eq!(first.at().x().value(), 2_700);
    assert_eq!(first.at().y().value(), 1_160);
    assert_eq!(first.at().z().value(), 0, "free floor: z is zero");
    assert_eq!(first.facing().value(), 180_000);
    assert_eq!(
        first.group().map(|group| group.as_str()),
        Some("window-bench")
    );
    let pier = points
        .get(&SitPointKey::new("pier-bench.1").expect("a key"))
        .expect("held");
    assert!(pier.group().is_none(), "a point without a group names none");
}

#[test]
fn an_empty_list_is_a_place_with_no_sit_points() {
    assert!(decode("[]").expect("valid").is_empty());
}

#[test]
fn a_duplicate_key_is_refused_naming_the_key() {
    let error =
        decode(&list(&[point("bench.1", 0, 0), point("bench.1", 2_000, 0)])).expect_err("refused");
    assert!(
        error.contains("bench.1") && error.contains("listed twice"),
        "{error}"
    );
}

#[test]
fn sixty_five_points_are_refused_and_sixty_four_accepted() {
    let row = |index: i32| point(&format!("bench.{index}"), index * 1_000, 0);
    let sixty_four: Vec<String> = (0..64).map(row).collect();
    assert_eq!(
        decode(&list(&sixty_four)).expect("64 is the bound").len(),
        64
    );
    let sixty_five: Vec<String> = (0..65).map(row).collect();
    let error = decode(&list(&sixty_five)).expect_err("65 is refused");
    assert!(error.contains("at most 64"), "{error}");
}

#[test]
fn two_points_509_mm_apart_are_refused_and_510_mm_accepted() {
    let apart = |distance: i32| {
        decode(&list(&[
            point("bench.1", 0, 0),
            point("bench.2", distance, 0),
        ]))
    };
    let error = apart(509).expect_err("509 mm is closer than the spacing");
    assert!(
        error.contains("bench.1") && error.contains("bench.2") && error.contains("510"),
        "{error}"
    );
    assert_eq!(
        apart(510).expect("510 mm is the spacing, inclusive").len(),
        2
    );
    // Diagonal distance counts: (300, 400) is 500 mm, refused; (360, 480) is 600 mm, accepted.
    let diagonal = |dx, dy| decode(&list(&[point("bench.1", 0, 0), point("bench.2", dx, dy)]));
    assert!(
        diagonal(300, 400).is_err(),
        "500 mm diagonal is closer than 510"
    );
    assert!(diagonal(360, 480).is_ok(), "600 mm diagonal is clear");
}

#[test]
fn a_coordinate_beyond_100_000_mm_is_refused_and_the_bound_accepted() {
    assert!(decode(&list(&[point("bench.1", 100_000, -100_000)])).is_ok());
    let error = decode(&list(&[point("bench.1", 100_001, 0)])).expect_err("refused");
    assert!(
        error.contains("100000") && error.contains("bench.1"),
        "{error}"
    );
}

#[test]
fn an_unknown_field_and_a_malformed_key_are_refused_by_name() {
    let unknown = decode("- { key: bench.1, at: { x: 0, y: 0 }, facing: 0, height: 450 }\n")
        .expect_err("refused");
    assert!(unknown.contains("height"), "{unknown}");
    let long = "a".repeat(49);
    for key in ["Bench.1", "bench_1", "bench 1", "", long.as_str()] {
        let text = format!("- {{ key: \"{key}\", at: {{ x: 0, y: 0 }}, facing: 0 }}\n");
        let error = decode(&text).expect_err("a malformed key is refused");
        assert!(error.contains("sit point key"), "{key:?}: {error}");
    }
    assert!(decode("- { key: \"bench-1.a\", at: { x: 0, y: 0 }, facing: 0 }\n").is_ok());
}

#[test]
fn a_point_without_its_position_or_facing_is_refused() {
    assert!(
        decode("- { key: bench.1, facing: 0 }\n").is_err(),
        "no position"
    );
    assert!(
        decode("- { key: bench.1, at: { x: 0, y: 0 } }\n").is_err(),
        "no facing"
    );
}

/// Seeding is the owner's: the section becomes one `place-sit-points` fact about the place, and genesis
/// reduces it into the place's `SitPoints`, exactly as a decoded list reads.
#[test]
fn a_place_seeds_its_sit_points_at_genesis_and_holds_them_after() {
    let mut world = World::new();
    world.install(PostureSystem).expect("posture installs");
    let place = world
        .create_entity(EntityKey::new("cafe").expect("a key"), EntityType::Place)
        .expect("a place");
    let person = world
        .create_entity(EntityKey::new("alice").expect("a key"), EntityType::Person)
        .expect("a person");
    let keys: BTreeMap<EntityKey, EntityId> = [
        (EntityKey::new("cafe").expect("a key"), place),
        (EntityKey::new("alice").expect("a key"), person),
    ]
    .into_iter()
    .collect();
    let authored = decode(&list(&[point("window-bench.1", 2_700, 1_160)])).expect("valid");
    let facts = {
        let read = world.read();
        PostureSystem::seed(&Seeding::new(&read, &keys), place, &authored).expect("seeded")
    };
    assert_eq!(facts.len(), 1);
    world.genesis(GENESIS, facts).expect("begins");

    let held = world
        .read()
        .component::<SitPoints>(place)
        .cloned()
        .expect("the place holds its sit points");
    assert_eq!(held, authored);

    // The same section on a person's file is refused by its owner, whatever the loader would say.
    let read = world.read();
    let refused = PostureSystem::seed(&Seeding::new(&read, &keys), person, &authored)
        .expect_err("a person is not a place");
    assert_eq!(refused, Rejection::PreconditionFailed);
}
