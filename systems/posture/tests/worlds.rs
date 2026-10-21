//! Every shipped world's sit points (S34-c SD-c-4; AC-c-1, AC-c-2, AC-c-3, I-c-1, I-c-2, I-c-3).
//!
//! The worlds are discovered by listing `worlds/`, so no world, place or coordinate is named here. For
//! each World Pack, and for a seeded generated one:
//!
//! ```text
//! 1  it loads and begins: S34-b's load check refuses a sit point inside furniture or off the floor
//! 2  every sit point of a shaped place stands within reach (750 mm) of a solid: next to furniture
//! 3  removable: the same pack with `posture` and every `sit-points:` section deleted begins from the
//!    same genesis facts, posture's own left out
//! ```
//!
//! What each check would pass without proving (`ARC-23`): a world list that is empty (at least two
//! worlds must carry sit points); a removal that removes nothing (the stripped pack must not enable
//! posture, and the full one must state `place-sit-points`).

mod support;

use std::path::{Path, PathBuf};

use mineworld_bodies::PlaceShape;
use mineworld_kernel::{Emission, SystemIdentity};
use mineworld_packages::PackRoots;
use mineworld_posture::{PostureSystem, REACH, SitPoints};
use mineworld_test_support::Scratch;
use mineworld_worldpack::{AssembledWorld, WorldPack};
use support::GENESIS;

fn worlds() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../worlds");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("worlds/ lists")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("world.yaml").is_file())
        .collect();
    found.sort();
    assert!(found.len() >= 3, "the shipped worlds are found: {found:?}");
    found
}

/// The repository's pack roots, as `mineworld` is given them (`ARC-77`).
fn roots() -> PackRoots {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    PackRoots::new(
        vec![
            repository.join("entities"),
            repository.join("presentation/mineworld-default"),
        ],
        None,
    )
    .expect("the roots exist")
}

fn assemble(path: &Path) -> AssembledWorld {
    WorldPack::read_with(path.to_path_buf(), &roots())
        .and_then(|pack| pack.assemble())
        .unwrap_or_else(|error| panic!("{} assembles: {error}", path.display()))
}

/// Check 2 on a begun world: the number of sit points it holds, each next to a solid.
fn next_to_furniture(label: &str, assembled: AssembledWorld) -> usize {
    let AssembledWorld {
        mut world, facts, ..
    } = assembled;
    world
        .genesis(GENESIS, facts)
        .unwrap_or_else(|error| panic!("{label} begins (S34-b's load check): {error:?}"));
    let read = world.read();
    let reach = i64::from(REACH.value());
    let mut count = 0;
    for (place, points) in read.components::<SitPoints>() {
        let Some(shape) = read.component::<PlaceShape>(place) else {
            continue;
        };
        for point in points.iter() {
            count += 1;
            let (x, y) = (
                i64::from(point.at().x().value()),
                i64::from(point.at().y().value()),
            );
            let near = shape.solids().iter().any(|solid| {
                let gap = |v: i64, low: i64, high: i64| (low - v).max(v - high).max(0);
                let dx = gap(
                    x,
                    solid.min().x().value().into(),
                    solid.max().x().value().into(),
                );
                let dy = gap(
                    y,
                    solid.min().y().value().into(),
                    solid.max().y().value().into(),
                );
                dx * dx + dy * dy <= reach * reach
            });
            assert!(
                near,
                "{label}: sit point {} at ({x}, {y}) is farther than {reach} mm from every solid",
                point.key()
            );
        }
    }
    count
}

/// Copies `from` into `to`, file by file.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copies");
        }
    }
}

/// `text` without its top-level `sit-points:` block.
fn without_sit_points(text: &str) -> String {
    let mut kept = String::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("sit-points:") {
            inside = true;
            continue;
        }
        if inside && (line.is_empty() || line.starts_with(' ') || line.starts_with('-')) {
            continue;
        }
        inside = false;
        kept.push_str(line);
        kept.push('\n');
    }
    kept
}

/// A scratch copy of the pack at `path` with `posture` and its sections removed.
fn stripped(path: &Path, name: &str) -> Scratch {
    let scratch = mineworld_test_support::scratch!(empty name);
    let root = scratch
        .path()
        .join(path.file_name().expect("a directory name"));
    copy_tree(path, &root);
    let world = std::fs::read_to_string(root.join("world.yaml")).expect("reads");
    let world: String = world
        .lines()
        .filter(|line| line.trim() != "- posture")
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(root.join("world.yaml"), world).expect("writes");
    for entry in std::fs::read_dir(root.join("places")).expect("places/ lists") {
        let file = entry.expect("an entry").path();
        let text = std::fs::read_to_string(&file).expect("reads");
        std::fs::write(&file, without_sit_points(&text)).expect("writes");
    }
    scratch
}

/// Genesis facts that are not posture's.
fn others(facts: &[Emission]) -> Vec<Emission> {
    facts
        .iter()
        .filter(|fact| *fact.owner() != PostureSystem::ID)
        .cloned()
        .collect()
}

fn enables_posture(path: &Path) -> bool {
    std::fs::read_to_string(path.join("world.yaml"))
        .expect("reads")
        .lines()
        .any(|line| line.trim() == "- posture")
}

/// Checks 1–3 on one pack; the number of sit points it carries.
fn check(path: &Path, label: &str) -> usize {
    let count = next_to_furniture(label, assemble(path));
    if enables_posture(path) {
        let full = assemble(path);
        assert!(
            full.facts
                .iter()
                .any(|fact| *fact.owner() == PostureSystem::ID),
            "{label} states its sit points"
        );
        let scratch = stripped(path, &format!("stripped-{label}"));
        let copy = scratch.path().join(path.file_name().expect("a name"));
        assert!(!enables_posture(&copy), "{label}: posture is removed");
        let bare = assemble(&copy);
        assert_eq!(
            others(&full.facts),
            bare.facts,
            "{label}: removing posture and its sections leaves genesis as it was (I-c-2)"
        );
    }
    count
}

#[test]
fn every_shipped_world_loads_its_sit_points_beside_furniture_and_without_posture_is_unchanged() {
    let mut seated_worlds = 0;
    for path in worlds() {
        let label = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a name")
            .to_owned();
        let count = check(&path, &label);
        println!("{label}: {count} sit points");
        if count > 0 {
            seated_worlds += 1;
        }
    }
    assert!(
        seated_worlds >= 2,
        "at least two shipped worlds carry sit points"
    );
}

/// A seeded generator: xorshift64*.
struct Seeded(u64);

impl Seeded {
    fn below(&mut self, n: i32) -> i32 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let wide = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % u64::try_from(n).expect("positive");
        i32::try_from(wide).expect("small")
    }
}

/// SD-c-1's rule on a generated place: benches (1 900 × 700 boxes) dropped at random on a 20 × 15 m
/// floor, kept only where they and their front points clear everything placed before; three points per
/// bench, 600 mm apart, 260 mm off the open long face.
fn generated(seed: u64, root: &Path) {
    let mut random = Seeded(seed);
    let mut solids: Vec<(i32, i32, i32, i32)> = Vec::new();
    let mut points: Vec<(i32, i32, i32)> = Vec::new();
    let clear = |solids: &[(i32, i32, i32, i32)], (x, y): (i32, i32), keep: i64| {
        solids.iter().all(|&(x0, y0, x1, y1)| {
            let dx = i64::from((x0 - x).max(x - x1).max(0));
            let dy = i64::from((y0 - y).max(y - y1).max(0));
            dx * dx + dy * dy >= keep * keep
        })
    };
    for _ in 0..40 {
        if solids.len() == 6 {
            break;
        }
        let (x0, y0) = (1_000 + random.below(16_000), 1_000 + random.below(12_000));
        let bench = (x0, y0, x0 + 1_900, y0 + 700);
        let south = random.below(2) == 0;
        let y = if south { y0 - 260 } else { y0 + 700 + 260 };
        let facing = if south { 180_000 } else { 0 };
        let mine: Vec<(i32, i32, i32)> = [x0 + 350, x0 + 950, x0 + 1_550]
            .into_iter()
            .map(|x| (x, y, facing))
            .collect();
        let corners = [
            (bench.0, bench.1),
            (bench.2, bench.3),
            (bench.0, bench.3),
            (bench.2, bench.1),
        ];
        let apart = corners.iter().all(|&corner| clear(&solids, corner, 1_200))
            && mine.iter().all(|&(x, y, _)| clear(&solids, (x, y), 260))
            && points.iter().all(|&(px, py, _)| {
                mine.iter()
                    .all(|&(x, y, _)| (px - x).abs() + (py - y).abs() > 1_200)
            });
        if apart {
            solids.push(bench);
            points.extend(mine);
        }
    }
    let mut text = String::from(
        "tags: [square]\nbody:\n  floor: { min: { x: 0, y: 0 }, max: { x: 20000, y: 15000 } }\n  solids:\n",
    );
    for (x0, y0, x1, y1) in &solids {
        text.push_str(&format!(
            "    - {{ min: {{ x: {x0}, y: {y0} }}, max: {{ x: {x1}, y: {y1} }}, height: 840 }}\n"
        ));
    }
    text.push_str("sit-points:\n");
    for (index, (x, y, facing)) in points.iter().enumerate() {
        text.push_str(&format!(
            "  - {{ key: seat.{index}, at: {{ x: {x}, y: {y} }}, facing: {facing} }}\n"
        ));
    }
    for directory in ["places", "people"] {
        std::fs::create_dir_all(root.join(directory)).expect("writable");
    }
    std::fs::write(root.join("places/square.yaml"), text).expect("writes");
    std::fs::write(
        root.join("people/ann.yaml"),
        "location:\n  place: square\n  position: { x: 500, y: 500 }\n",
    )
    .expect("writes");
    let id = root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a name");
    std::fs::write(
        root.join("world.yaml"),
        format!(
            "world:\n  id: {id}\n  name: A Square\nsystems:\n  - presence\n  - movement\n  - bodies\n  \
             - posture\nplaces:\n  - square\npopulation:\n  - ann\n"
        ),
    )
    .expect("writes");
}

#[test]
fn a_generated_square_holds_the_same_three_checks() {
    const SEED: u64 = 0x5334_C0DE_0000_0001;
    let scratch = mineworld_test_support::scratch!(empty "generated-square");
    let root = scratch.path().join("generated-square");
    generated(SEED, &root);
    let count = check(&root, "generated-square");
    println!("seed {SEED:#x}: {count} sit points");
    assert!(count >= 6, "seed {SEED:#x}: the generator placed benches");
}
