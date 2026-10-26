//! Every point the server puts a person at clears every solid (S36-P5a commit 2; step-36 O-7,
//! `ARC-332`), over every world found in `worlds/`.
//!
//! ```text
//! worlds     every directory of worlds/ with a world.yaml validates (genesis runs every bodies check)
//! points     in every place with a `body:` section, every person's start, both ends of every
//!            passage and every sit point lies inside the floor shrunk by R + GAP (260 mm) and at
//!            least R + GAP from every solid; a place without `body:` is listed as not judged
//! variants   a copy of social-cafe with alice's start 5 mm inside the counter's R + GAP band (it still
//!            passes genesis, which holds starts to R) fails naming alice, the café, solid 0 and the
//!            distance; a copy of bodies-yard with a sit point 200 mm from its solid fails naming the
//!            key; 200 seeded places with points on their free lattice all pass
//! ```
//!
//! The geometry and the points are read from the files' text (`bodies/mod.rs`), never from bodies'
//! code (DO-12); nothing here names a world to decide (I-S36-5).

mod bodies;
mod headless;

use std::path::{Path, PathBuf};

use bodies::{
    GAP, PointKind, R, Rect, Room, YARD, copy_dir, distance_to_rect, points_of, rooms_of,
};
use headless::{PACK, fresh, mineworld, stderr, stdout};

/// How far every point must stay from every solid and from the floor's edge: a person's radius and
/// the character controller's gap.
const CLEAR: i32 = R + GAP;

/// The repository's `worlds/`.
const WORLDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds");
/// The repository's Entity Packs.
const ENTITIES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../entities");
/// The repository's default Presentation Packs.
const PRESENTATION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../presentation/mineworld-default"
);

/// One judged point, as printed.
#[derive(Debug)]
struct Row {
    world: String,
    place: String,
    kind: PointKind,
    key: String,
    /// The nearest solid (index from 0, in the order written) and its distance, if the place has any.
    nearest: Option<(usize, f64)>,
    /// The distance to the floor's edge; negative outside the floor.
    edge: i32,
    /// The smaller of the two, less `CLEAR`: negative is a failure.
    margin: f64,
}

impl Row {
    fn fails(&self) -> bool {
        self.margin < 0.0
    }

    fn line(&self) -> String {
        let nearest = self.nearest.map_or_else(
            || "no solid".to_owned(),
            |(index, distance)| format!("solid {index} at {distance:.0} mm"),
        );
        format!(
            "{} {} {:?} {}: {nearest}, edge {} mm, margin {:.0} mm",
            self.world, self.place, self.kind, self.key, self.edge, self.margin
        )
    }
}

/// The distance from `(x, y)` to the edge of `floor`, inward (negative outside).
fn to_edge((x, y): (i32, i32), (x0, y0, x1, y1): Rect) -> i32 {
    (x - x0).min(x1 - x).min(y - y0).min(y1 - y)
}

/// One point judged in its room.
fn judge(world: &str, place: &str, kind: PointKind, key: &str, at: (i32, i32), room: &Room) -> Row {
    let nearest = room
        .solids
        .iter()
        .enumerate()
        .map(|(index, (rect, _))| (index, distance_to_rect(at, *rect)))
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let edge = to_edge(at, room.floor);
    let closest = nearest.map_or(f64::from(edge), |(_, d)| d.min(f64::from(edge)));
    Row {
        world: world.to_owned(),
        place: place.to_owned(),
        kind,
        key: key.to_owned(),
        nearest,
        edge,
        margin: closest - f64::from(CLEAR),
    }
}

/// Every point of the world at `pack` judged; the places with no `body:` that hold points, listed.
fn judge_world(pack: &Path) -> (Vec<Row>, Vec<String>) {
    let world = pack
        .file_name()
        .expect("a world's directory")
        .to_string_lossy()
        .into_owned();
    let rooms = rooms_of(pack);
    let mut rows = Vec::new();
    let mut unjudged = Vec::new();
    for point in points_of(pack) {
        match rooms.get(&point.place) {
            Some(room) => rows.push(judge(
                &world,
                &point.place,
                point.kind,
                &point.key,
                point.at,
                room,
            )),
            None => {
                let line = format!("{world} {}: no body: not judged", point.place);
                if !unjudged.contains(&line) {
                    unjudged.push(line);
                }
            }
        }
    }
    (rows, unjudged)
}

/// Every world directory of `worlds/`: those with a `world.yaml`, sorted.
fn worlds() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(WORLDS)
        .expect("worlds/ lists")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("world.yaml").is_file())
        .collect();
    found.sort();
    found
}

#[test]
fn every_world_validates_and_every_point_a_person_is_placed_at_clears_every_solid() {
    let found = worlds();
    assert!(found.len() >= 2, "worlds/ holds worlds: {found:?}");
    let mut failures = Vec::new();
    let mut judged = 0;
    for pack in &found {
        // The repository's pack roots, as `packs.rs` gives them: a world may require an Entity or
        // Presentation Pack that is data, not part of the build.
        let output = mineworld(&[
            "validate",
            pack.to_str().expect("a printable path"),
            "--packs",
            ENTITIES,
            "--packs",
            PRESENTATION,
        ]);
        assert!(
            output.status.success(),
            "{pack:?} validates: {}{}",
            stdout(&output),
            stderr(&output)
        );
        let (rows, unjudged) = judge_world(pack);
        for row in &rows {
            println!("{}", row.line());
            if row.fails() {
                failures.push(row.line());
            }
        }
        for line in unjudged {
            println!("{line}");
        }
        judged += rows.len();
    }
    assert!(judged > 0, "some point was judged");
    assert!(
        failures.is_empty(),
        "points within R + GAP ({CLEAR} mm) of a solid or the floor's edge:\n{}",
        failures.join("\n")
    );
}

/// A copy of the world at `from`, in scratch, under its own directory name.
fn copy(from: &str, name: &str) -> (headless::Scratch, PathBuf) {
    let scratch = fresh(name);
    let source = Path::new(from);
    let copy = scratch.join(source.file_name().expect("a name"));
    copy_dir(source, &copy);
    (scratch, copy)
}

fn replace_once(path: &Path, old: &str, new: &str) {
    let text = std::fs::read_to_string(path).expect("reads");
    assert_eq!(text.matches(old).count(), 1, "{path:?} says {old:?} once");
    std::fs::write(path, text.replacen(old, new, 1)).expect("writes");
}

#[test]
fn a_start_inside_a_solids_band_is_named_with_the_solid_and_the_distance() {
    let (_scratch, pack) = copy(PACK, "layout-start");
    let rooms = rooms_of(&pack);
    let counter = rooms["cafe"].solids[0].0;
    // 5 mm inside the band: R + GAP − 5 = 255 mm north of the counter, which genesis (R) still admits.
    let y = counter.3 + CLEAR - 5;
    replace_once(
        &pack.join("people/alice.yaml"),
        "    x: 6000\n    y: 8000\n",
        &format!("    x: 6000\n    y: {y}\n"),
    );
    let (rows, _) = judge_world(&pack);
    let failing: Vec<&Row> = rows.iter().filter(|row| row.fails()).collect();
    assert_eq!(failing.len(), 1, "{failing:?}");
    let line = failing[0].line();
    println!("refused: {line}");
    for word in [
        "cafe",
        "Start",
        "alice",
        "solid 0 at 255 mm",
        "margin -5 mm",
    ] {
        assert!(line.contains(word), "names {word:?}: {line}");
    }
}

#[test]
fn a_sit_point_200_mm_from_a_solid_is_named_by_its_key() {
    let (_scratch, pack) = copy(YARD, "layout-sit");
    let rooms = rooms_of(&pack);
    let (place, room) = rooms
        .iter()
        .find(|(_, room)| !room.solids.is_empty())
        .expect("a place with a solid");
    let (x0, y0, x1, _) = room.solids[0].0;
    let at = ((x0 + x1) / 2, y0 - 200);
    let file = pack.join("places").join(format!("{place}.yaml"));
    let mut text = std::fs::read_to_string(&file).expect("reads");
    text.push_str(&format!(
        "\nsit-points:\n  - {{ key: planted.1, at: {{ x: {}, y: {} }}, facing: 0 }}\n",
        at.0, at.1
    ));
    std::fs::write(&file, text).expect("writes");
    let (rows, _) = judge_world(&pack);
    let failing: Vec<String> = rows.iter().filter(|r| r.fails()).map(Row::line).collect();
    assert_eq!(failing.len(), 1, "{failing:?}");
    println!("refused: {}", failing[0]);
    for word in [place.as_str(), "Sit", "planted.1", "solid 0 at 200 mm"] {
        assert!(failing[0].contains(word), "names {word:?}: {}", failing[0]);
    }
}

/// A seeded generator (splitmix64).
struct Seeded(u64);

impl Seeded {
    fn below(&mut self, bound: i32) -> i32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        i32::try_from(z % u64::from(bound.unsigned_abs())).expect("below an i32")
    }
}

#[test]
fn points_on_the_free_lattice_of_200_seeded_places_all_pass() {
    let mut rng = Seeded(0x5336_0052);
    let mut placed = 0;
    for _ in 0..200 {
        let floor = (0, 0, 4_000 + rng.below(8_001), 4_000 + rng.below(8_001));
        let mut solids: Vec<(Rect, i32)> = Vec::new();
        for _ in 0..rng.below(12) {
            let (w, d) = (200 + rng.below(1_001), 200 + rng.below(1_001));
            let (x, y) = (rng.below(floor.2 - w), rng.below(floor.3 - d));
            solids.push(((x, y, x + w, y + d), 700));
        }
        let room = Room { floor, solids };
        // The free lattice: every 100 mm point the rule admits, from which each kind takes some.
        let kinds = [PointKind::Start, PointKind::Doorway, PointKind::Sit];
        let mut y = floor.1 + CLEAR;
        while y <= floor.3 - CLEAR {
            let mut x = floor.0 + CLEAR;
            while x <= floor.2 - CLEAR {
                let free = room
                    .solids
                    .iter()
                    .all(|(rect, _)| distance_to_rect((x, y), *rect) >= f64::from(CLEAR));
                if free && rng.below(40) == 0 {
                    let kind = kinds[usize::try_from(rng.below(3)).expect("small")];
                    let row = judge("generated", "room", kind, "p", (x, y), &room);
                    assert!(!row.fails(), "{}", row.line());
                    placed += 1;
                }
                x += 100;
            }
            y += 100;
        }
    }
    assert!(placed > 1_000, "the lattice gave points: {placed}");
}
