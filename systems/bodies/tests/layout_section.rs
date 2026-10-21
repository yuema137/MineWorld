//! Solids of different things stand apart (`ARC-332`, S36-P5a commit 1): two solids of one place whose
//! footprints share any area must share a `group` or name one `furnishing`, or the `body:` section is
//! refused when the world loads, with `bodies-solids-overlap`.
//!
//! ```text
//! ungrouped     two overlapping solids, no group           refused, naming both and the overlap
//! grouped       the same pair with one group               loads
//! one piece     the same pair naming one furnishing        loads
//! two groups    the same pair in different groups          refused
//! touching      two solids sharing an edge or a corner     load
//! bad key       a group with an upper-case letter          refused, naming the key
//! generated     200 seeded places of solids apart          all load; each refused by index once
//!               one solid is pushed 1-300 mm into another
//! ```

use mineworld_bodies::Body;

/// The `body:` section from YAML, exactly as the World Pack loader decodes it.
fn decode(yaml: &str) -> Result<Body, String> {
    serde_saphyr::from_str::<Body>(yaml).map_err(|error| error.to_string())
}

fn refused(yaml: &str, words: &[&str]) {
    let error = match decode(yaml) {
        Ok(body) => panic!("must be refused, but loaded {body:?}"),
        Err(error) => error,
    };
    for word in words {
        assert!(error.contains(word), "names {word:?}: {error}");
    }
}

fn loads(yaml: &str) {
    if let Err(error) = decode(yaml) {
        panic!("must load, but: {error}\n{yaml}");
    }
}

/// A footprint `(x0, y0, x1, y1)`, in millimetres.
type Rect = (i32, i32, i32, i32);

/// One solid's line: its footprint and whatever else it carries (`, group: g`).
fn solid((x0, y0, x1, y1): Rect, extra: &str) -> String {
    format!(
        "  - {{ min: {{ x: {x0}, y: {y0} }}, max: {{ x: {x1}, y: {y1} }}, height: 700{extra} }}\n"
    )
}

/// A 10 m square room with these solid lines, and these furnishing lines if any.
fn room(solids: &[String], furnishings: &str) -> String {
    let mut yaml = "floor: { min: { x: 0, y: 0 }, max: { x: 10000, y: 10000 } }\n".to_owned();
    if !furnishings.is_empty() {
        yaml.push_str("furnishings:\n");
        yaml.push_str(furnishings);
    }
    yaml.push_str("solids:\n");
    for line in solids {
        yaml.push_str(line);
    }
    yaml
}

/// A table, and a chair tucked 60 mm under its east edge.
const TABLE: Rect = (4000, 4000, 5000, 5000);
const CHAIR: Rect = (4940, 4300, 5440, 4700);

#[test]
fn two_overlapping_solids_of_no_group_are_refused_naming_both_and_the_overlap() {
    refused(
        &room(
            &[
                solid((0, 0, 400, 400), ""),
                solid(TABLE, ""),
                solid(CHAIR, ""),
            ],
            "",
        ),
        &[
            "bodies-solids-overlap",
            "solids 1 and 2",
            "60 mm x 400 mm",
            "share a `group`",
        ],
    );
}

#[test]
fn the_same_pair_in_one_group_loads() {
    loads(&room(
        &[
            solid(TABLE, ", group: seating.east"),
            solid(CHAIR, ", group: seating.east"),
        ],
        "",
    ));
}

#[test]
fn the_same_pair_naming_one_furnishing_loads() {
    loads(&room(
        &[
            solid(TABLE, ", furnishing: table.1"),
            solid(CHAIR, ", furnishing: table.1"),
        ],
        "  - { key: table.1, kind: table, at: { x: 4500, y: 4500 }, facing: 0 }\n",
    ));
}

#[test]
fn the_same_pair_in_two_groups_or_one_grouped_is_refused() {
    refused(
        &room(
            &[solid(TABLE, ", group: a"), solid(CHAIR, ", group: b")],
            "",
        ),
        &["bodies-solids-overlap", "solids 0 and 1"],
    );
    refused(
        &room(&[solid(TABLE, ", group: a"), solid(CHAIR, "")], ""),
        &["bodies-solids-overlap", "solids 0 and 1"],
    );
}

#[test]
fn solids_touching_along_an_edge_or_at_a_corner_load() {
    loads(&room(
        &[
            solid(TABLE, ""),
            solid((5000, 4000, 5500, 5000), ""),
            solid((5000, 5000, 5500, 5500), ""),
            solid((4000, 3500, 5000, 4000), ""),
        ],
        "",
    ));
}

#[test]
fn a_group_with_an_upper_case_letter_is_refused_naming_it() {
    refused(
        &room(&[solid(TABLE, ", group: Seating")], ""),
        &["solid group", "Seating"],
    );
}

/// A seeded generator (splitmix64): the same places on every platform and every run.
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

fn overlap(a: Rect, b: Rect) -> bool {
    a.0.max(b.0) < a.2.min(b.2) && a.1.max(b.1) < a.3.min(b.3)
}

/// 2 to 12 solids, 200 to 1200 mm a side, apart from each other, on a 10 m floor (rejection
/// sampling).
fn place_apart(rng: &mut Seeded) -> Vec<Rect> {
    let wanted = 2 + rng.below(11);
    let mut solids: Vec<Rect> = Vec::new();
    for _ in 0..400 {
        if solids.len() == usize::try_from(wanted).expect("small") {
            break;
        }
        let (w, d) = (200 + rng.below(1001), 200 + rng.below(1001));
        let (x, y) = (rng.below(10_000 - w), rng.below(10_000 - d));
        let candidate = (x, y, x + w, y + d);
        if solids.iter().all(|other| !overlap(*other, candidate)) {
            solids.push(candidate);
        }
    }
    assert!(solids.len() >= 2, "the generator places at least two");
    solids
}

/// `moving`, its centre lined up with `into`'s on the north-south axis and its west face `depth` mm
/// west of `into`'s east face: the two footprints then share a band `depth` mm wide (at most the
/// narrower width) and at least the shallower depth.
fn pushed(moving: Rect, into: Rect, depth: i32) -> Rect {
    let (w, d) = (moving.2 - moving.0, moving.3 - moving.1);
    let x0 = into.2 - depth;
    let y0 = (into.1 + into.3) / 2 - d / 2;
    (x0, y0, x0 + w, y0 + d)
}

fn lines(solids: &[Rect]) -> Vec<String> {
    solids.iter().map(|rect| solid(*rect, "")).collect()
}

#[test]
fn two_hundred_generated_places_of_solids_apart_load_and_each_one_pushed_in_is_refused_by_index() {
    let mut rng = Seeded(0x5336_0005);
    for case in 0..200 {
        let mut solids = place_apart(&mut rng);
        loads(&room(&lines(&solids), ""));

        let into = usize::try_from(rng.below(i32::try_from(solids.len()).expect("small")))
            .expect("an index");
        let moving = (into + 1) % solids.len();
        let depth = 1 + rng.below(300);
        solids[moving] = pushed(solids[moving], solids[into], depth);
        // The pushed solid may now also meet a third; the first pair in written order is named, and
        // it involves the pushed one.
        let first = (0..solids.len())
            .flat_map(|i| (i + 1..solids.len()).map(move |j| (i, j)))
            .find(|&(i, j)| overlap(solids[i], solids[j]))
            .unwrap_or_else(|| panic!("case {case}: the push made an overlap"));
        assert!(
            first.0 == moving || first.1 == moving,
            "case {case}: the first overlapping pair {first:?} involves solid {moving}"
        );
        refused(
            &room(&lines(&solids), ""),
            &[
                "bodies-solids-overlap",
                &format!("solids {} and {}", first.0, first.1),
            ],
        );
    }
}
