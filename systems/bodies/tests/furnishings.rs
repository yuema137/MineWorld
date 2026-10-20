//! A place's furnishings (`ARC-307`): the named pieces a body can meet, each named by its solids.
//!
//! ```text
//! orphan        a furnishing no solid names                 refused
//! unknown       a solid naming a key the place does not list refused
//! outside       `at` outside its own solids                 refused
//! duplicate     two furnishings with one key                refused
//! too many      a 65th furnishing                           refused
//! facing        360 000 millidegrees                        refused
//! positive      a table of two crossed boxes loads, and an observer is told its furnishings inside
//!               `place-shape`
//! ```

mod support;

use mineworld_bodies::{Body, PlaceShape};
use support::{Plan, Yard};

/// The `body:` section from YAML, exactly as the World Pack loader decodes it.
fn decode(yaml: &str) -> Result<Body, String> {
    serde_saphyr::from_str::<Body>(yaml).map_err(|error| error.to_string())
}

fn refused(yaml: &str, words: &[&str]) {
    let error = match decode(yaml) {
        Ok(body) => panic!("must be refused, but loaded {body:?}"),
        Err(error) => error,
    };
    println!("refused: {error}");
    for word in words {
        assert!(error.contains(word), "names {word}: {error}");
    }
}

/// A 6 m square room with one table (two crossed boxes) and its `furnishings` list as given.
fn room(furnishings: &str, solids_extra: &str) -> String {
    format!(
        "floor: {{ min: {{ x: 0, y: 0 }}, max: {{ x: 6000, y: 6000 }} }}\n\
         furnishings:\n{furnishings}\
         solids:\n\
         \x20 - {{ min: {{ x: 2000, y: 2900 }}, max: {{ x: 4000, y: 3100 }}, height: 720, furnishing: table.1 }}\n\
         \x20 - {{ min: {{ x: 2900, y: 2000 }}, max: {{ x: 3100, y: 4000 }}, height: 720, furnishing: table.1 }}\n\
         {solids_extra}"
    )
}

const TABLE: &str =
    "  - { key: table.1, kind: table.terrace, at: { x: 3000, y: 3000 }, facing: 0 }\n";

#[test]
fn a_furnishing_no_solid_names_is_refused() {
    let extra = "  - { key: chair.1, kind: chair, at: { x: 1000, y: 1000 }, facing: 90000 }\n";
    refused(
        &room(&format!("{TABLE}{extra}"), ""),
        &["chair.1", "named by no solid"],
    );
}

#[test]
fn a_solid_naming_a_furnishing_the_place_does_not_list_is_refused() {
    let extra = "  - { min: { x: 500, y: 500 }, max: { x: 900, y: 900 }, height: 900, furnishing: chair.9 }\n";
    refused(
        &room(TABLE, extra),
        &["solid 2", "chair.9", "does not list"],
    );
}

#[test]
fn a_furnishing_standing_outside_its_own_solids_is_refused() {
    let far = "  - { key: table.1, kind: table.terrace, at: { x: 4500, y: 3000 }, facing: 0 }\n";
    refused(
        &room(far, ""),
        &["table.1", "(4500, 3000)", "outside its own solids"],
    );
}

#[test]
fn two_furnishings_with_one_key_are_refused() {
    refused(
        &room(&format!("{TABLE}{TABLE}"), ""),
        &["table.1", "listed twice"],
    );
}

#[test]
fn a_sixty_fifth_furnishing_is_refused() {
    let mut list = String::new();
    let mut solids = String::new();
    for i in 0..65 {
        let x = 100 + (i % 8) * 700;
        let y = 100 + (i / 8) * 600;
        list.push_str(&format!(
            "  - {{ key: stool.{i}, kind: stool, at: {{ x: {}, y: {} }}, facing: 0 }}\n",
            x + 50,
            y + 50
        ));
        solids.push_str(&format!(
            "  - {{ min: {{ x: {x}, y: {y} }}, max: {{ x: {}, y: {} }}, height: 700, furnishing: stool.{i} }}\n",
            x + 100,
            y + 100
        ));
    }
    let yaml = format!(
        "floor: {{ min: {{ x: 0, y: 0 }}, max: {{ x: 6000, y: 6000 }} }}\nfurnishings:\n{list}solids:\n{solids}"
    );
    // 65 solids also break the 64-solid cap; either refusal names the count.
    refused(&yaml, &["at most 64", "65"]);
}

#[test]
fn a_facing_of_a_full_turn_is_refused() {
    let turned =
        "  - { key: table.1, kind: table.terrace, at: { x: 3000, y: 3000 }, facing: 360000 }\n";
    refused(&room(turned, ""), &["table.1", "360000", "0 to 359999"]);
}

#[test]
fn a_bad_kind_slug_is_refused() {
    let bad = "  - { key: table.1, kind: Table, at: { x: 3000, y: 3000 }, facing: 0 }\n";
    refused(&room(bad, ""), &["furnishing kind", "Table"]);
}

/// A table of two crossed boxes loads, and whoever perceives the room is told the furnishing and
/// which solids name it, inside `place-shape`.
#[test]
fn an_observer_is_told_the_furnishings_of_the_place_it_stands_in() {
    let Body::Place(shape) = decode(&room(TABLE, "")).expect("the room loads") else {
        panic!("a place's body");
    };
    let named: Vec<_> = shape
        .solids()
        .iter()
        .map(|solid| solid.furnishing().map(|key| key.as_str().to_owned()))
        .collect();
    assert_eq!(
        named,
        [Some("table.1".to_owned()), Some("table.1".to_owned())]
    );
    let yard = Yard::new(&Plan::room(shape.clone(), &[("alice", (1_000, 1_000))]));
    let told: Vec<serde_json::Value> = yard
        .observation("alice")
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .filter(|record| record.component_type().as_str() == "place-shape")
        .map(|record| record.payload().clone())
        .collect();
    println!("place-shape told to alice: {told:?}");
    assert_eq!(told.len(), 1, "the room's shape, once");
    assert_eq!(
        told[0]["furnishings"],
        serde_json::json!([
            { "key": "table.1", "kind": "table.terrace", "at": { "x": 3000, "y": 3000 }, "facing": 0 }
        ])
    );
    assert_eq!(told[0]["solids"][1]["furnishing"], "table.1");
    let back: PlaceShape = serde_json::from_value(told[0].clone()).expect("decodes back");
    assert_eq!(back, shape, "the disclosed value is the stated shape");
}
