//! NV-2, NV-3 and NV-4 — the walk, for real (step-11 §21.8; `DECISIONS.md` `ARC-73`, `DEP-34`).
//!
//! The worlds are test-time copies of `worlds/bodies-yard`'s shape: its court, verbatim, and a hall of
//! the same size and door whose furniture is drawn for each claim — a wall with a 1 100 mm slot, a
//! U-shaped pocket, a table, an enclosed ring — plus an attic no passage reaches. A scripted session
//! drives the real `mineworld server`: `walk-to`, then `walk-step` until the server refuses a step
//! because the walk is over. What happened is read back from the save, facts decoded with this file's
//! own mirrors of the payloads (the `bodies/mod.rs` practice, DO-12).
//!
//! ```text
//! hall   12 000 × 9 000, door (11 600, 4 500) ↔ court (400, 5 000)
//!        slot wall   (3 000, 0)–(3 300, 3 900) and (3 000, 5 000)–(3 300, 9 000): a 1 100 mm slot
//!        pocket      west (6 700, 700)–(7 000, 3 600), south (7 000, 700)–(9 000, 1 000),
//!                    north (7 000, 3 000)–(9 000, 3 300): open east
//!        table       (7 000, 6 000)–(9 000, 7 000)
//!        ring        (9 700, 6 000)–(11 500, 8 400), walls 200 thick: a closed pocket inside
//! court  10 000 square, pillar (4 700, 4 700)–(5 300, 5 300)
//! attic  no body, no passage
//! ```
//!
//! Every waypoint below is hand-computed from those numbers with obstacles grown by 360 mm
//! (R 300 + GAP 10 + PLAN_MARGIN 50), never read back from the code under test (`ARC-23` rule 2).

mod headless;
mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use headless::{Scratch, Tables, fresh, mineworld, stderr, stdout};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EventEnvelope, Rejection,
    WorldTime,
};
use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend, format};
use mineworld_worldpack::WorldPack;
use serde_json::{Value, json};
use support::{Client, SaveDir, Server};

const COURT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../worlds/bodies-yard/places/court.yaml"
);

/// The hall's furniture.
const SLOT: [(i32, i32, i32, i32); 2] = [(3_000, 0, 3_300, 3_900), (3_000, 5_000, 3_300, 9_000)];
const POCKET: [(i32, i32, i32, i32); 3] = [
    (6_700, 700, 7_000, 3_600),
    (7_000, 700, 9_000, 1_000),
    (7_000, 3_000, 9_000, 3_300),
];
const TABLE: (i32, i32, i32, i32) = (7_000, 6_000, 9_000, 7_000);
const RING: [(i32, i32, i32, i32); 4] = [
    (9_700, 6_000, 11_500, 6_200),
    (9_700, 8_200, 11_500, 8_400),
    (9_700, 6_200, 9_900, 8_200),
    (11_300, 6_200, 11_500, 8_200),
];

/// One person of a test world: key, place, position.
type Placed = (&'static str, &'static str, i32, i32);

/// One loose object: key, place, position, half-extent of a box.
type Object = (&'static str, &'static str, i32, i32, i32);

/// Writes a walking world into `directory`: the hall (with its furniture when `bodies`), the court,
/// the attic, `people` and `objects`. Without `bodies` the world has no `body:` section and does not
/// enable the pack.
fn world(directory: &Path, people: &[Placed], objects: &[Object], bodies: bool) -> PathBuf {
    let pack = directory.join("walking-yard");
    for sub in ["places", "people", "items"] {
        std::fs::create_dir_all(pack.join(sub)).expect("a directory");
    }
    let keys = |list: Vec<&str>| {
        list.iter()
            .map(|key| format!("  - {key}\n"))
            .collect::<String>()
    };
    let population = keys(people.iter().map(|(key, ..)| *key).collect());
    let items = keys(objects.iter().map(|(key, ..)| *key).collect());
    let mut manifest = format!(
        "world:\n  id: walking-yard\n  name: Walking Yard\n  version: 0.1.0\n  license: MIT\n\
         mineworld: \"^0.1\"\nsystems:\n  - presence\n  - movement\n{}places:\n  - attic\n  - court\n  \
         - hall\npopulation:\n{population}seats:\n{population}",
        if bodies { "  - bodies\n" } else { "" }
    );
    if !objects.is_empty() {
        manifest.push_str(&format!("items:\n{items}"));
    }
    std::fs::write(pack.join("world.yaml"), manifest).expect("world.yaml");
    let solid = |(x0, y0, x1, y1): (i32, i32, i32, i32)| {
        format!(
            "    - {{ min: {{ x: {x0}, y: {y0} }}, max: {{ x: {x1}, y: {y1} }}, height: 1000 }}\n"
        )
    };
    let mut hall = String::from(
        "tags:\n  - hall\npassages:\n  - to: court\n    here: { x: 11600, y: 4500 }\n    \
         there: { x: 400, y: 5000 }\n",
    );
    if bodies {
        hall.push_str(
            "body:\n  floor:\n    min: { x: 0, y: 0 }\n    max: { x: 12000, y: 9000 }\n  solids:\n",
        );
        for furniture in SLOT.iter().chain(&POCKET).chain([&TABLE]).chain(&RING) {
            hall.push_str(&solid(*furniture));
        }
    }
    std::fs::write(pack.join("places/hall.yaml"), hall).expect("hall.yaml");
    let court = std::fs::read_to_string(COURT).expect("the yard's court");
    let court = if bodies {
        court
    } else {
        "tags:\n  - court\n".to_owned()
    };
    std::fs::write(pack.join("places/court.yaml"), court).expect("court.yaml");
    std::fs::write(pack.join("places/attic.yaml"), "tags:\n  - attic\n").expect("attic.yaml");
    for (key, place, x, y) in people {
        std::fs::write(
            pack.join(format!("people/{key}.yaml")),
            format!("tags:\n  - {key}\nlocation:\n  place: {place}\n  position:\n    x: {x}\n    y: {y}\n"),
        )
        .expect("a person");
    }
    for (key, place, x, y, half) in objects {
        std::fs::write(
            pack.join(format!("items/{key}.yaml")),
            format!(
                "tags:\n  - {key}\nbody:\n  shape: {{ box: {{ x: {half}, y: {half}, z: {half} }} }}\n  \
                 at: {{ place: {place}, x: {x}, y: {y} }}\n"
            ),
        )
        .expect("an object");
    }
    let validated = mineworld(&["validate", pack.to_str().expect("a path")]);
    assert!(
        validated.status.success(),
        "the test world validates: {}{}",
        stdout(&validated),
        stderr(&validated)
    );
    pack
}

/// What every key of `pack` resolved to.
fn ids(pack: &Path) -> BTreeMap<String, EntityId> {
    WorldPack::read(pack)
        .expect("reads")
        .assemble()
        .expect("assembles")
        .ids
        .into_iter()
        .map(|(key, id)| (key.as_str().to_owned(), id))
        .collect()
}

fn reference(entity: EntityId, kind: &str) -> Value {
    json!({ "entity": entity.to_string(), "entity_type": kind })
}

fn request(actor: EntityId, action: &str, target: Option<EntityId>, payload: Value) -> Value {
    json!({
        "actor": actor,
        "action_type": action,
        "target": target,
        "payload": { "action_type": action, "payload": payload },
        "actor_location": null,
    })
}

fn walk_to_point(actor: EntityId, place: EntityId, x: i32, y: i32) -> Value {
    request(
        actor,
        "walk-to",
        None,
        json!({ "to": { "place": {
            "place": reference(place, "place"),
            "local": { "x": x, "y": y, "z": 0 },
            "facing": null,
        } } }),
    )
}

fn walk_to_place(actor: EntityId, place: EntityId) -> Value {
    request(
        actor,
        "walk-to",
        None,
        json!({ "to": { "place": { "place": reference(place, "place"), "local": null, "facing": null } } }),
    )
}

fn walk_to_person(actor: EntityId, person: EntityId) -> Value {
    request(
        actor,
        "walk-to",
        None,
        json!({ "to": { "person": reference(person, "person") } }),
    )
}

fn walk_step(actor: EntityId) -> Value {
    request(actor, "walk-step", None, json!({}))
}

fn stride(actor: EntityId, place: EntityId, x: i32, y: i32) -> Value {
    support::stride(actor, place, x, y)
}

/// The refusal's code, when the answer is `Rejected(System { code })`.
fn code(result: &ActionResult) -> Option<String> {
    match result {
        ActionResult::Rejected(Rejection::System { code, .. }) => Some(code.as_str().to_owned()),
        _ => None,
    }
}

/// Steps until the server refuses a step because there is no walk; returns the steps accepted.
async fn step_out(client: &mut Client, me: EntityId) -> usize {
    for steps in 0..80 {
        let (_, result) = client.submit(walk_step(me)).await;
        match result {
            ActionResult::Accepted { .. } => {}
            ActionResult::Rejected(Rejection::PreconditionFailed) => return steps,
            other => panic!("a step is accepted, or refused once the walk is over: {other:?}"),
        }
    }
    panic!("the walk did not end in 80 steps");
}

/// The walking record the observer's own observation discloses: its next waypoints, as (x, y).
async fn next_waypoints(client: &mut Client, me: EntityId) -> Vec<(i64, i64)> {
    let observation = client
        .observation_where("the walker's own walking record", |observation| {
            observation.entity(me).is_some_and(|entity| {
                entity
                    .components()
                    .iter()
                    .any(|record| record.component_type().as_str() == "walking")
            })
        })
        .await;
    let record = observation
        .entity(me)
        .and_then(|entity| {
            entity
                .components()
                .iter()
                .find(|record| record.component_type().as_str() == "walking")
        })
        .expect("disclosed")
        .payload()
        .clone();
    record["next"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|point| {
            (
                point["x"].as_i64().expect("x"),
                point["y"].as_i64().expect("y"),
            )
        })
        .collect()
}

/// One fact, as this test reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Fact {
    Arrived {
        person: EntityId,
        place: EntityId,
        at: Option<(i64, i64)>,
    },
    StoppedShort {
        person: EntityId,
    },
    Entered {
        person: EntityId,
        place: EntityId,
    },
    WalkStarted {
        person: EntityId,
    },
    WalkEnded {
        person: EntityId,
        outcome: String,
    },
    ObjectMoved,
    Shoved,
    Other(String),
}

fn entity(value: &Value) -> EntityId {
    serde_json::from_value(value["entity"].clone()).expect("an entity reference")
}

/// The facts of a save after genesis, in order.
fn facts(save: &Path) -> Vec<Fact> {
    let tables = Tables::read(save);
    tables
        .facts
        .iter()
        .map(|(_, bytes)| {
            let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
            let record = fact.payload();
            let payload: Value = serde_json::from_slice(record.payload()).expect("JSON");
            match record.event_type().as_str() {
                "arrived" => Fact::Arrived {
                    person: entity(&payload["person"]),
                    place: entity(&payload["location"]["place"]),
                    at: payload["location"]["local"].as_object().map(|local| {
                        (
                            local["x"].as_i64().expect("x"),
                            local["y"].as_i64().expect("y"),
                        )
                    }),
                },
                "stopped-short" => Fact::StoppedShort {
                    person: entity(&payload["person"]),
                },
                "person-entered-place" => Fact::Entered {
                    person: entity(&payload["person"]),
                    place: entity(&payload["place"]),
                },
                "walk-started" => Fact::WalkStarted {
                    person: entity(&payload["person"]),
                },
                "walk-ended" => Fact::WalkEnded {
                    person: entity(&payload["person"]),
                    outcome: payload["outcome"].as_str().expect("an outcome").to_owned(),
                },
                "object-moved" => Fact::ObjectMoved,
                "person-shoved" => Fact::Shoved,
                other => Fact::Other(other.to_owned()),
            }
        })
        .collect()
}

/// The walker's arrivals after the first `walk-started`: (place, position).
fn trail(facts: &[Fact], walker: EntityId) -> Vec<(EntityId, (i64, i64))> {
    let start = facts
        .iter()
        .position(|fact| *fact == Fact::WalkStarted { person: walker })
        .expect("a walk started");
    facts[start..]
        .iter()
        .filter_map(|fact| match fact {
            Fact::Arrived {
                person,
                place,
                at: Some(at),
            } if *person == walker => Some((*place, *at)),
            _ => None,
        })
        .collect()
}

fn outcomes(facts: &[Fact], walker: EntityId) -> Vec<String> {
    facts
        .iter()
        .filter_map(|fact| match fact {
            Fact::WalkEnded { person, outcome } if *person == walker => Some(outcome.clone()),
            _ => None,
        })
        .collect()
}

/// Every stride within one place is at most 1 340 mm.
fn assert_strides(trail: &[(EntityId, (i64, i64))], from: (EntityId, (i64, i64))) {
    let mut at = from;
    for next in trail {
        if next.0 == at.0 {
            let d2 = (next.1.0 - at.1.0).pow(2) + (next.1.1 - at.1.1).pow(2);
            assert!(
                d2 <= 1_340 * 1_340,
                "a stride of {} mm: {at:?} → {next:?}",
                d2.isqrt()
            );
        }
        at = *next;
    }
}

/// A server on `pack`, saving into `save`, with one session joined to `seat`.
async fn session(
    pack: &Path,
    save: &SaveDir,
    seat: &str,
    extra: &[&str],
) -> (Server, Client, EntityId) {
    let mut arguments = vec![
        "server",
        pack.to_str().expect("a path"),
        "--save",
        save.path(),
    ];
    arguments.extend_from_slice(extra);
    let server = Server::start(&arguments).await;
    let mut client = Client::connect(server.address).await;
    let (me, _) = client.join(seat).await;
    (server, client, me)
}

/// NV-2 (a) and (b): through the 1 100 mm slot, and out of the U-shaped pocket round its closed side.
#[tokio::test]
async fn a_walk_goes_through_the_slot_and_out_of_the_pocket() {
    let directory = fresh("walking-slot");
    let pack = world(
        &directory,
        &[("ada", "hall", 1_500, 1_500), ("pia", "hall", 8_000, 2_000)],
        &[],
        true,
    );
    let id = ids(&pack);
    let (hall, ada, pia) = (id["hall"], id["ada"], id["pia"]);
    let save = SaveDir::new("walking-slot-save");
    let (server, mut client, me) = session(&pack, &save, "ada", &[]).await;
    assert_eq!(me, ada);
    let (_, result) = client.submit(walk_to_point(ada, hall, 4_500, 1_500)).await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    // (a) Grown slot wall: (2 640, −360)–(3 660, 4 260) and (2 640, 4 640)–(3 660, 9 360). The straight
    // line meets the first; the route bends at its north-west and north-east corners — through the slot,
    // whose grown width is 380 mm — then straight to the goal. Start → (3 660, 4 260) directly would
    // cross the wall (at x 2 640 it is at y 2 956), so the first corner is (2 640, 4 260).
    assert_eq!(
        next_waypoints(&mut client, ada).await,
        [(2_640, 4_260), (3_660, 4_260), (4_500, 1_500)]
    );
    let steps = step_out(&mut client, ada).await;
    let mut pia_client = Client::connect(server.address).await;
    pia_client.join("pia").await;
    // (b) From inside the pocket (8 000, 2 000) to (5 500, 2 000), behind its closed west side. Grown:
    // west (6 340, 340)–(7 360, 3 960), south (6 640, 340)–(9 360, 1 360), north (6 640, 2 640)–
    // (9 360, 3 660). South of the pocket no corner lies inside the shrunk floor (y > 360), so the way
    // is out of the mouth, round the north wall's east end and over the west wall's top:
    // (9 360, 2 640) → (9 360, 3 660) → (7 360, 3 960) → (6 340, 3 960) → (5 500, 2 000).
    let (_, result) = pia_client
        .submit(walk_to_point(pia, hall, 5_500, 2_000))
        .await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(
        next_waypoints(&mut pia_client, pia).await,
        [
            (9_360, 2_640),
            (9_360, 3_660),
            (7_360, 3_960),
            (6_340, 3_960)
        ]
    );
    let pia_steps = step_out(&mut pia_client, pia).await;
    drop(server);

    let facts = facts(Path::new(save.path()));
    let ada_trail = trail(&facts, ada);
    println!("(a) {steps} steps: {ada_trail:?}");
    assert_strides(&ada_trail, (hall, (1_500, 1_500)));
    for waypoint in [(2_640, 4_260), (3_660, 4_260), (4_500, 1_500)] {
        assert!(
            ada_trail.iter().any(|(_, at)| *at == waypoint),
            "(a) a stride ends at {waypoint:?}"
        );
    }
    assert_eq!(
        ada_trail.last(),
        Some(&(hall, (4_500, 1_500))),
        "(a) at the requested point"
    );
    assert_eq!(outcomes(&facts, ada), ["arrived"]);
    let pia_trail = trail(&facts, pia);
    println!("(b) {pia_steps} steps: {pia_trail:?}");
    assert_strides(&pia_trail, (hall, (8_000, 2_000)));
    for waypoint in [
        (9_360, 2_640),
        (9_360, 3_660),
        (7_360, 3_960),
        (6_340, 3_960),
    ] {
        assert!(
            pia_trail.iter().any(|(_, at)| *at == waypoint),
            "(b) the path leaves the pocket's mouth through {waypoint:?}"
        );
    }
    assert_eq!(pia_trail.last(), Some(&(hall, (5_500, 2_000))));
    assert_eq!(outcomes(&facts, pia), ["arrived"]);
    assert!(
        !facts
            .iter()
            .any(|fact| matches!(fact, Fact::StoppedShort { .. })),
        "(a), (b): no stride stopped short"
    );
}

/// NV-2 (c): to a person behind the table, then again while that person moves.
#[tokio::test]
async fn a_walk_to_a_person_goes_round_the_table_and_follows_them() {
    let directory = fresh("walking-person");
    let pack = world(
        &directory,
        &[("ada", "hall", 4_000, 3_500), ("bea", "hall", 8_000, 7_600)],
        &[],
        true,
    );
    let id = ids(&pack);
    let (hall, ada, bea) = (id["hall"], id["ada"], id["bea"]);
    let save = SaveDir::new("walking-person-save");
    let (server, mut client, _) = session(&pack, &save, "ada", &[]).await;
    let (_, result) = client.submit(walk_to_person(ada, bea)).await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    // The table grown: (6 640, 5 640)–(9 360, 7 360). The straight line meets it (at x 6 640, y 6 206);
    // the route bends at its north-west corner and ends at Bea.
    assert_eq!(
        next_waypoints(&mut client, ada).await,
        [(6_640, 7_360), (8_000, 7_600)]
    );
    step_out(&mut client, ada).await;
    drop(server);
    let first = facts(Path::new(save.path()));

    // Again, in a fresh world, and Bea steps 1 530 mm west after Ada's second stride: the walk re-plans
    // toward where she went.
    let save = SaveDir::new("walking-person-save-2");
    let (server, mut client, _) = session(&pack, &save, "ada", &[]).await;
    let mut bea_client = Client::connect(server.address).await;
    bea_client.join("bea").await;
    client.submit(walk_to_person(ada, bea)).await;
    for _ in 0..2 {
        client.submit(walk_step(ada)).await;
    }
    let (_, moved) = bea_client.submit(stride(bea, hall, 6_500, 7_900)).await;
    assert!(matches!(moved, ActionResult::Accepted { .. }), "{moved:?}");
    step_out(&mut client, ada).await;
    drop(server);
    let second = facts(Path::new(save.path()));

    assert_eq!(outcomes(&first, ada), ["arrived"]);
    assert_eq!(outcomes(&second, ada), ["arrived"]);
    let ends: Vec<(i64, i64)> = [&first, &second]
        .iter()
        .map(|facts| trail(facts, ada).last().expect("Ada walked").1)
        .collect();
    println!("(c) ends {ends:?}");
    for (end, bea_at) in ends.iter().zip([(8_000, 7_600), (6_500, 7_900)]) {
        let d2 = (end.0 - bea_at.0).pow(2) + (end.1 - bea_at.1).pow(2);
        assert!(
            (610 * 610..=1_200 * 1_200).contains(&d2),
            "(c) within 1 200 mm of Bea at {bea_at:?}, and not on her: {end:?}, {} mm",
            d2.isqrt()
        );
    }
}

/// NV-2 (d): into the court through the doorway.
#[tokio::test]
async fn a_walk_into_the_court_crosses_at_the_passage() {
    let directory = fresh("walking-court");
    let pack = world(&directory, &[("ada", "hall", 10_000, 4_500)], &[], true);
    let id = ids(&pack);
    let (hall, court, ada) = (id["hall"], id["court"], id["ada"]);
    let save = SaveDir::new("walking-court-save");
    let (server, mut client, _) = session(&pack, &save, "ada", &[]).await;
    let (_, result) = client.submit(walk_to_point(ada, court, 2_000, 5_000)).await;
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    step_out(&mut client, ada).await;
    drop(server);
    let facts = facts(Path::new(save.path()));
    // 1 600 mm to the door: 1 340, 260; the crossing to (400, 5 000); 1 600 mm: 1 340, 260.
    assert_eq!(
        trail(&facts, ada),
        [
            (hall, (11_340, 4_500)),
            (hall, (11_600, 4_500)),
            (court, (400, 5_000)),
            (court, (1_740, 5_000)),
            (court, (2_000, 5_000))
        ]
    );
    assert!(facts.contains(&Fact::Entered {
        person: ada,
        place: court
    }));
    assert_eq!(outcomes(&facts, ada), ["arrived"]);
}

/// NV-2 (e): every refusal, by its code.
#[tokio::test]
async fn walks_nobody_can_make_are_refused_by_name() {
    let directory = fresh("walking-refused");
    let pack = world(
        &directory,
        &[
            ("ada", "hall", 1_500, 1_500),
            ("cal", "court", 8_000, 8_000),
        ],
        &[],
        true,
    );
    let id = ids(&pack);
    let (hall, attic, ada, cal) = (id["hall"], id["attic"], id["ada"], id["cal"]);
    let save = SaveDir::new("walking-refused-save");
    let (server, mut client, _) = session(&pack, &save, "ada", &[]).await;
    // Inside the ring: grown, its free hollow is (10 260, 6 560)–(10 940, 7 840), enclosed.
    let (_, pocket) = client.submit(walk_to_point(ada, hall, 10_600, 7_200)).await;
    assert_eq!(code(&pocket).as_deref(), Some("no-route"), "{pocket:?}");
    let (_, elsewhere) = client.submit(walk_to_person(ada, cal)).await;
    assert_eq!(elsewhere, ActionResult::Rejected(Rejection::TooFarAway));
    let (_, isolated) = client.submit(walk_to_place(ada, attic)).await;
    assert_eq!(code(&isolated).as_deref(), Some("no-route"), "{isolated:?}");
    let (_, malformed) = client
        .submit(request(
            ada,
            "walk-to",
            None,
            json!({ "to": { "somewhere": 1 } }),
        ))
        .await;
    assert_eq!(
        code(&malformed).as_deref(),
        Some("malformed-payload"),
        "{malformed:?}"
    );
    let (_, idle) = client.submit(walk_step(ada)).await;
    assert_eq!(idle, ActionResult::Rejected(Rejection::PreconditionFailed));
    drop(server);
    assert!(
        !facts(Path::new(save.path()))
            .iter()
            .any(|fact| matches!(fact, Fact::WalkStarted { .. })),
        "nothing was recorded"
    );
}

/// NV-2 (f): a second walk-to replaces the walk, a move stops it, a shove does not.
#[tokio::test]
async fn a_walk_is_replaced_stopped_and_survives_a_shove() {
    let directory = fresh("walking-superseded");
    let pack = world(
        &directory,
        &[("ada", "hall", 1_500, 7_500), ("dan", "hall", 2_200, 6_000)],
        &[],
        true,
    );
    let id = ids(&pack);
    let (hall, ada, dan) = (id["hall"], id["ada"], id["dan"]);
    let save = SaveDir::new("walking-superseded-save");
    let (server, mut client, _) = session(&pack, &save, "ada", &[]).await;
    client.submit(walk_to_point(ada, hall, 1_500, 4_500)).await;
    // 1 414 mm away: one step does not get there, so the move below stops a walk in progress.
    client.submit(walk_to_point(ada, hall, 2_500, 8_500)).await;
    client.submit(walk_step(ada)).await;
    let (_, moved) = client.submit(stride(ada, hall, 1_500, 7_500)).await;
    assert!(matches!(moved, ActionResult::Accepted { .. }), "{moved:?}");
    // Then south to (1 500, 4 500); one stride to (1 500, 6 160), 718 mm from Dan, who shoves Ada.
    client.submit(walk_to_point(ada, hall, 1_500, 4_500)).await;
    client.submit(walk_step(ada)).await;
    let mut dan_client = Client::connect(server.address).await;
    dan_client.join("dan").await;
    let (_, shoved) = dan_client
        .submit(request(dan, "shove", Some(ada), json!({})))
        .await;
    assert!(
        matches!(shoved, ActionResult::Accepted { .. }),
        "{shoved:?}"
    );
    step_out(&mut client, ada).await;
    drop(server);
    let facts = facts(Path::new(save.path()));
    assert_eq!(outcomes(&facts, ada), ["replaced", "stopped", "arrived"]);
    assert!(facts.contains(&Fact::Shoved), "the shove happened");
    let last = trail(&facts, ada).last().copied();
    assert_eq!(
        last,
        Some((hall, (1_500, 4_500))),
        "and the walk arrived after it"
    );
}

/// NV-2 (g) on the server: eight hours of the world pass in a wall second (`--time-scale 28800`), more
/// than a day in the wait below, and nobody who is not stepped moves.
#[tokio::test]
async fn a_walk_nobody_steps_takes_no_calendar_time() {
    let directory = fresh("walking-idle");
    let pack = world(&directory, &[("ada", "hall", 1_500, 1_500)], &[], true);
    let id = ids(&pack);
    let (hall, ada) = (id["hall"], id["ada"]);
    let save = SaveDir::new("walking-idle-save");
    let (server, mut client, _) = session(&pack, &save, "ada", &["--time-scale", "28800"]).await;
    client.submit(walk_to_point(ada, hall, 4_500, 1_500)).await;
    let before = next_waypoints(&mut client, ada).await;
    let started = server.status().await["at"]
        .as_i64()
        .expect("the world's instant");
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    let later = server.status().await["at"]
        .as_i64()
        .expect("the world's instant");
    println!("world time {started} → {later}");
    assert!(
        later - started >= 86_400,
        "a day passed: {started} → {later}"
    );
    assert_eq!(
        next_waypoints(&mut client, ada).await,
        before,
        "the walk waits"
    );
    drop(server);
    let facts = facts(Path::new(save.path()));
    assert!(trail(&facts, ada).is_empty(), "no stride without a step");
    assert!(outcomes(&facts, ada).is_empty(), "and nothing ended it");
}

/// M-N3's scenario: a crate across the direct line is planned round, never pushed.
#[tokio::test]
async fn a_walk_goes_round_a_loose_object() {
    let directory = fresh("walking-object");
    let pack = world(
        &directory,
        &[("ada", "hall", 1_500, 1_000)],
        &[("crate", "hall", 1_500, 2_500, 300)],
        true,
    );
    let id = ids(&pack);
    let (hall, ada) = (id["hall"], id["ada"]);
    let save = SaveDir::new("walking-object-save");
    let (server, mut client, _) = session(&pack, &save, "ada", &[]).await;
    client.submit(walk_to_point(ada, hall, 1_500, 4_000)).await;
    // The crate (1 200, 2 200)–(1 800, 2 800) grown: (840, 1 840)–(2 160, 3 160). The line x = 1 500 is
    // 660 mm from either side, so west and east are equally short; the tie goes to the corner listed
    // first (SW before SE), and the route bends at the west corners.
    let route = next_waypoints(&mut client, ada).await;
    println!("round the crate: {route:?}");
    assert_eq!(route, [(840, 1_840), (840, 3_160), (1_500, 4_000)]);
    step_out(&mut client, ada).await;
    drop(server);
    let facts = facts(Path::new(save.path()));
    assert!(
        !facts.contains(&Fact::ObjectMoved),
        "the crate was walked round, not pushed"
    );
    assert_eq!(trail(&facts, ada).last(), Some(&(hall, (1_500, 4_000))));
    assert_eq!(outcomes(&facts, ada), ["arrived"]);
}

/// NV-3: without bodies, a walk is straight — one waypoint per leg — and every stride lands where it
/// was asked, across the café and into the street of `worlds/social-cafe`.
#[tokio::test]
async fn without_bodies_a_walk_is_straight() {
    let pack = Path::new(support::PACK);
    let id = ids(pack);
    let (cafe, street, visitor) = (id["cafe"], id["street"], id["visitor"]);
    let save = SaveDir::new("walking-cafe-save");
    let (server, mut client, _) = session(pack, &save, "visitor", &[]).await;
    client
        .submit(walk_to_point(visitor, street, 0, 4_000))
        .await;
    // The café's door at (1 610, 200), 400 mm from the visitor at (1 610, 600): one waypoint.
    assert_eq!(next_waypoints(&mut client, visitor).await, [(1_610, 200)]);
    step_out(&mut client, visitor).await;
    drop(server);
    let facts = facts(Path::new(save.path()));
    assert_eq!(
        trail(&facts, visitor),
        [
            (cafe, (1_610, 200)),
            (street, (0, 3_000)),
            (street, (0, 4_000))
        ]
    );
    assert!(
        !facts
            .iter()
            .any(|fact| matches!(fact, Fact::StoppedShort { .. }))
    );
    assert_eq!(outcomes(&facts, visitor), ["arrived"]);
}

// ---------------------------------------------------------------------------------------------
// NV-4 — determinism: two processes, a SIGKILL between two walk-steps, and replay
// ---------------------------------------------------------------------------------------------

/// Set in the child process, which runs the walk's first part into the save named by its value and
/// waits to be killed.
const CHILD: &str = "WALKING_NV4_CHILD";
const INSTANCE: u128 = 0x0012_0001_0000_0000_0000_0000_0000_0004;

/// The NV-4 walk: from (10 000, 4 500) in the hall, round a crate standing on the line to the door, to
/// (2 000, 5 000) in the court — `walk-to` and six steps: (10 140, 3 840) and (11 460, 3 840), the
/// crate's grown corners (south and north are equally short; the tie is the search's to break), the
/// door, the crossing, two strides. The child is killed after the crossing. A route with a tie is
/// what makes the search's order observable (M-N4).
fn nv4_pack(directory: &Scratch) -> PathBuf {
    world(
        directory,
        &[("ada", "hall", 10_000, 4_500)],
        &[("crate", "hall", 10_800, 4_500, 300)],
        true,
    )
}

fn nv4_requests(pack: &Path) -> Vec<ActionRecord> {
    let id = ids(pack);
    let walk_to = walk_to_point(id["ada"], id["court"], 2_000, 5_000);
    let step = walk_step(id["ada"]);
    // A record's payload is the encoded bytes, as the server encodes a submitted payload.
    std::iter::once(walk_to)
        .chain(std::iter::repeat_n(step, 6))
        .map(|request| {
            let bytes = serde_json::to_vec(&request["payload"]["payload"]).expect("encodes");
            serde_json::from_value(json!({
                "action_type": request["action_type"],
                "payload": bytes,
            }))
            .expect("a record")
        })
        .collect()
}

/// Creates the save at `save` and dispatches `requests[range]` at their fixed instants.
fn nv4_run(pack: &Path, save: &Path, range: std::ops::Range<usize>, create: bool) {
    let world_pack = WorldPack::read(pack).expect("reads");
    let ada = ids(pack)["ada"];
    let mut persistent = if create {
        let backend = SqliteBackend::create(save, Durability::ProcessCrash).expect("creates");
        let assembled = world_pack.assemble().expect("assembles");
        PersistentWorld::create(
            Box::new(backend),
            assembled.world,
            Creation {
                instance: INSTANCE,
                pack: world_pack.id().to_owned(),
                at: WorldTime::EPOCH,
                facts: assembled.facts,
            },
        )
        .expect("a save")
        .0
    } else {
        let backend = SqliteBackend::open(save, Durability::ProcessCrash).expect("opens");
        let composed = world_pack.compose().expect("composes");
        PersistentWorld::resume(Box::new(backend), composed.world)
            .expect("resumes")
            .0
    };
    let requests = nv4_requests(pack);
    for index in range {
        let at = WorldTime::from_seconds(10 + i64::try_from(index).expect("small"));
        let _ = persistent.advance_to(at).expect("advances");
        let intent = ActionIntent::new(
            ActionId::from_raw(u64::try_from(index).expect("small") + 1),
            ada,
            requests[index].clone(),
            at,
        );
        let dispatched = persistent.dispatch(&intent, at).expect("dispatches");
        assert!(
            matches!(dispatched.result(), ActionResult::Accepted { .. }),
            "request {index}: {:?}",
            dispatched.result()
        );
    }
}

#[test]
fn a_walk_killed_between_two_steps_resumes_to_the_same_bytes_and_replays() {
    if let Some(target) = std::env::var_os(CHILD) {
        let target = PathBuf::from(target);
        let pack = target.join("walking-yard");
        nv4_run(&pack, &target.join("save"), 0..5, true);
        println!("NV4-READY");
        std::thread::sleep(std::time::Duration::from_secs(600));
        return;
    }
    let directory = fresh("walking-nv4");
    let pack = nv4_pack(&directory);
    // The uninterrupted run.
    let whole = directory.join("whole");
    nv4_run(&pack, &whole, 0..7, true);
    // The killed run: a child process creates the save, walks to the third step, and is killed.
    let mut child = std::process::Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--exact",
            "a_walk_killed_between_two_steps_resumes_to_the_same_bytes_and_replays",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env(CHILD, &*directory)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("the child runs");
    let reader = std::io::BufReader::new(child.stdout.take().expect("piped"));
    let ready = std::io::BufRead::lines(reader)
        .map_while(Result::ok)
        .any(|line| line.contains("NV4-READY"));
    assert!(ready, "the child walked its part");
    child.kill().expect("killed");
    let status = child.wait().expect("reaped");
    assert!(
        !status.success(),
        "the child was killed, not finished: {status:?}"
    );
    let killed = directory.join("save");
    let walking_at_kill = Tables::read(&killed).facts.len();
    nv4_run(&pack, &killed, 5..7, false);
    let (a, b) = (Tables::read(&whole), Tables::read(&killed));
    println!(
        "NV-4: {} facts uninterrupted; the killed save held {walking_at_kill} and resumed to {}",
        a.facts.len(),
        b.facts.len()
    );
    a.assert_same_history(&b, "NV-4");
    let facts = facts(&whole);
    assert_eq!(
        outcomes(&facts, ids(&pack)["ada"]),
        ["arrived"],
        "the walk arrived"
    );
    let replayed = mineworld(&[
        "replay",
        pack.to_str().expect("a path"),
        "--save",
        killed.to_str().expect("a path"),
    ]);
    assert!(
        replayed.status.success(),
        "replay regenerates every fact: {}{}",
        stdout(&replayed),
        stderr(&replayed)
    );
}
