//! `AC-13` with the **real** clients (S14 PR 16e §4.1): talk, shove, kick, throw (its complete form),
//! buy and eat, each done the 2D way (click the figure or the object, choose the entry) and the 3D way
//! (walk up, look, press), send the same request by the server's own definition (`differing_fields`,
//! the allowed difference being `ActorLocation`), get the same kind of answer, and cause the same fact
//! types in the save.
//!
//! One step list, two players: the list is 13b's selector format, played by the 2D client's harness
//! (`clients/2d/scripts/harness/interact.gd`, `bodies.gd`) and by the 3D client's market probe
//! (`clients/3d-spike/scripts/slice/slice_probe_market.gd`), each on a fresh hosted world. Neither
//! player names an action type, a person or an object: this file does, and it takes every one of them
//! from the world's genesis observation of the seat (the nearest person offered a talk, the nearest
//! loose object of its place lying on the floor for a kick), never from a world's literals.
//!
//! The oracles are the server's: the requests are parsed through the contract, the answers are the
//! server's `RESULT` frames, and the caused facts are read from each save after its server stops.
//! `ac13_semantic_parity.rs` (the protocol module's demo flavours) stays as it is (Q16e-1).
//!
//! Every test starts Godot, so every test is `#[ignore]`d and runs at the milestone:
//!
//! ```text
//! cargo test -p mineworld-cli --test ac13_clients -- --ignored --test-threads=1
//! ```

mod godot2d;
mod godot3d;
mod headless;
mod support;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;

use godot2d::worlds::id;
use godot2d::{MARKET_TOWN, World};
use godot3d::{LOCOMOTION, depiction, interactions, result_kind, world_key};
use headless::Tables;
use mineworld_contracts::{ActionResult, Causation, EventEnvelope};
use mineworld_persistence::{JournalEntry, Outcome, WorldInput, format};
use mineworld_server::{RequestField, differing_fields};
use serde_json::{Value, json};
use support::{Client, SaveDir};

const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
const LAKESIDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/lakeside");

/// What both players type for the talk: one test constant (§4.1).
const WORDS: &str = "Good morning, this is one request from two clients";
/// The adversarial case's words (AC-e-3): the same talk with another utterance.
const OTHER_WORDS: &str = "Good evening, these words differ";
/// How long a kicked or thrown object is given to come to rest before the next step.
const SETTLE_S: u64 = 3;

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// The first seat a World Pack declares (`world.yaml`'s `seats:`): the seat both clients take.
fn first_seat(pack: &str) -> String {
    let text = std::fs::read_to_string(Path::new(pack).join("world.yaml")).expect("world.yaml");
    let mut lines = text.lines().skip_while(|l| l.trim_end() != "seats:");
    lines.next();
    lines
        .find_map(|l| l.trim().strip_prefix("- "))
        .unwrap_or_else(|| panic!("{pack} declares no seat"))
        .trim()
        .to_owned()
}

/// A point `{x, y}` in millimetres, wherever a payload keeps it (`local`, or the value itself).
fn point(value: &Value) -> Option<(f64, f64)> {
    let at = value.get("local").unwrap_or(value);
    Some((at.get("x")?.as_f64()?, at.get("y")?.as_f64()?))
}

fn apart(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// Whether a listed loose object lies on the floor: its centre's height is its own half-height (a
/// box's `z` half-extent, a ball's radius), as the bodies pack lists it (`Lying::at`). One on a solid
/// stands higher by the solid's height.
fn on_floor(listed: &Value) -> bool {
    let half = listed["shape"]["box"]["z"]
        .as_f64()
        .or_else(|| listed["shape"]["ball"].as_f64());
    let z = listed["at"].get("z").and_then(Value::as_f64);
    matches!((half, z), (Some(half), Some(z)) if (z - half).abs() <= 1.0)
}

/// `mineworld server <pack> --save <save>`, with the repository's pack roots given as an operator gives
/// them (`--packs`): a world that requires a pack outside the build finds it there (Lakeside's
/// Presentation Packs, ARC-54), and a world that requires none is unaffected. No `--agent`: the step
/// list needs nobody under an agent, and a seat named here would not exist in every world (Lakeside
/// has no `alice`).
fn served(pack: &str, save: &SaveDir) -> Vec<String> {
    let mut arguments: Vec<String> = ["server", pack, "--save", save.path()]
        .map(str::to_owned)
        .to_vec();
    for root in ["entities", "presentation/mineworld-default"] {
        arguments.push("--packs".to_owned());
        arguments.push(format!("{}/../../{root}", env!("CARGO_MANIFEST_DIR")));
    }
    arguments
}

/// The first entity a payload refers to by a typed reference, or "" (13b's `about`).
fn about(offer: &Value) -> String {
    offer["payload"]
        .as_object()
        .and_then(|fields| {
            fields
                .values()
                .find(|v| v.get("entity").and_then(Value::as_str).is_some())
        })
        .map_or_else(String::new, id)
}

/// What the seat's first observation offers and shows: everything a step list is built from.
struct Genesis {
    observer: String,
    frame: Value,
}

impl Genesis {
    /// Joins `seat` on a throwaway copy of the world and keeps its first observation. Two loads of one
    /// pack give every entity the same id (`AC-12`), so the ids hold for the worlds the clients join.
    async fn of(pack: &str, seat: &str) -> Self {
        let save = SaveDir::new("ac13-genesis");
        let mut world = World::start(&owned(&served(pack, &save)), None).await;
        let mut client = Client::connect(world.address).await;
        let (observer, _) = client.join(seat).await;
        let observation = client.observation().await;
        drop(client);
        world.kill();
        Self {
            observer: observer.raw().to_string(),
            frame: serde_json::to_value(&observation).expect("an observation serializes"),
        }
    }

    fn place(&self) -> String {
        id(&self.frame["self_location"]["place"])
    }

    fn here(&self) -> (f64, f64) {
        point(&self.frame["self_location"]).unwrap_or((0.0, 0.0))
    }

    fn offers(&self) -> Vec<Value> {
        self.frame["affordances"]
            .as_array()
            .cloned()
            .unwrap_or_default()
    }

    /// The nearest person in the observer's place offered `action_type`, by the frame's positions.
    fn nearest_person(&self, action_type: &str) -> Option<String> {
        let offered: Vec<String> = self
            .offers()
            .iter()
            .filter(|o| o["action_type"] == action_type && !o["target"].is_null())
            .map(|o| id(&o["target"]))
            .collect();
        let entities = self.frame["entities"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        entities
            .iter()
            .filter(|e| e["entity_type"] == "person")
            .filter(|e| id(&e["id"]) != self.observer && offered.contains(&id(&e["id"])))
            .filter(|e| id(&e["location"]["place"]) == self.place())
            .map(|e| {
                let d = point(&e["location"]).map_or(f64::MAX, |p| apart(p, self.here()));
                (d, id(&e["id"]))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, person)| person)
    }

    /// The nearest loose object of the observer's place that `action_type` can be done to, other than
    /// `except`: for a kick, one lying on the floor; for a throw, any. Taken from the place's disclosed
    /// listing, not from the menu: the bodies pack offers a kick or a throw only within 800 mm of the
    /// object (step-11 QO-7), so the genesis menu, seen from the seat, names none (nightly 40412844680).
    fn nearest_object(&self, action_type: &str, except: Option<&str>) -> Option<String> {
        let kick = action_type == "kick";
        let entities = self.frame["entities"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let place = entities
            .iter()
            .find(|e| id(&e["id"]) == self.place())
            .cloned()
            .unwrap_or_default();
        let listing = place["components"]
            .as_array()
            .and_then(|cs| cs.iter().find(|c| c["component_type"] == "loose-objects"))
            .map(|c| c["payload"].clone())
            .unwrap_or_default();
        let lying: Vec<Value> = listing
            .as_array()
            .cloned()
            .or_else(|| listing["objects"].as_array().cloned())
            .unwrap_or_default();
        lying
            .iter()
            .filter(|l| !kick || on_floor(l))
            .map(|l| (id(&l["object"]), point(&l["at"])))
            .filter(|(object, _)| Some(object.as_str()) != except)
            .map(|(object, at)| (at.map_or(f64::MAX, |p| apart(p, self.here())), object))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, object)| object)
    }

    /// The first item an offer of `action_type` on the observer's own menu is about.
    fn own_item(&self, action_type: &str) -> Option<String> {
        self.offers()
            .iter()
            .find(|o| o["action_type"] == action_type && o["target"].is_null())
            .map(about)
            .filter(|item| !item.is_empty())
    }

    /// The step list for `actions`, in this order: the own-menu actions first (they need no walk),
    /// then talk, shove, kick, throw. An action the world offers nothing for fails by name: a content
    /// gap, never a silent skip.
    fn steps(&self, actions: &[&str], words: &str) -> Value {
        let gap = |what: &str| -> ! {
            panic!(
                "{what}: the seat's genesis observation offers none (a content gap): {}",
                self.frame["affordances"]
            )
        };
        let mut steps = Vec::new();
        for action in ["buy", "eat"] {
            if actions.contains(&action) {
                let item = self.own_item(action).unwrap_or_else(|| gap(action));
                steps.extend([
                    json!({ "open": "self" }),
                    json!({ "choose": { "action_type": action, "about": item } }),
                ]);
            }
        }
        if actions.contains(&"talk") {
            let person = self.nearest_person("talk").unwrap_or_else(|| gap("talk"));
            steps.extend([
                json!({ "open": person }),
                json!({ "choose": { "walk": true } }),
                json!({ "open": person }),
                json!({ "choose": { "action_type": "talk", "target": person }, "input": words }),
            ]);
        }
        if actions.contains(&"shove") {
            let person = self.nearest_person("shove").unwrap_or_else(|| gap("shove"));
            steps.extend([
                json!({ "walk_near": person }),
                json!({ "open": person }),
                json!({ "choose": { "action_type": "shove", "target": person } }),
                json!({ "sleep": SETTLE_S }),
            ]);
        }
        let mut kicked = None;
        if actions.contains(&"kick") {
            let object = self
                .nearest_object("kick", None)
                .unwrap_or_else(|| gap("kick"));
            steps.extend(object_steps(&object, "kick"));
            kicked = Some(object);
        }
        if actions.contains(&"throw") {
            let object = self
                .nearest_object("throw", kicked.as_deref())
                .or_else(|| self.nearest_object("throw", None))
                .unwrap_or_else(|| gap("throw"));
            steps.extend(object_steps(&object, "throw"));
        }
        Value::Array(steps)
    }
}

/// Walk up to `object`, open it (2D: click it; 3D: look at it, press B) and choose its complete
/// `action_type` offer; then let it come to rest.
fn object_steps(object: &str, action_type: &str) -> Vec<Value> {
    vec![
        json!({ "walk_near": object }),
        json!({ "open_object": object }),
        json!({ "choose": { "action_type": action_type, "about": object } }),
        json!({ "sleep": SETTLE_S }),
    ]
}

/// One client's run: its lines and its save.
struct Run {
    lines: Vec<String>,
    save: SaveDir,
}

/// Plays `steps` with the 2D client on a fresh hosted `pack`.
async fn play_2d(pack: &str, seat: &str, name: &str, steps: &Value) -> Run {
    // The save has a scratch of its own name: the client run takes `name` for its steps file.
    let save = SaveDir::new(&format!("{name}-save"));
    let mut world = World::start(&owned(&served(pack, &save)), None).await;
    let key = world_key(pack);
    let (status, lines) =
        godot2d::worlds::play(world.address, seat, name, steps, &[key.as_str()]).await;
    world.kill();
    assert!(
        status.success() && godot2d::passed(&lines),
        "{name}: the 2D drive passes:\n{}",
        lines.join("\n")
    );
    Run { lines, save }
}

/// Plays `steps` with the 3D client on a fresh hosted `pack`.
async fn play_3d(pack: &str, seat: &str, name: &str, steps: &Value) -> Run {
    // The save has a scratch of its own name: the client run takes `name` for its steps file.
    let save = SaveDir::new(&format!("{name}-save"));
    let mut world = World::start(&owned(&served(pack, &save)), None).await;
    let run = play_3d_on(world.address, pack, seat, name, steps);
    world.kill();
    Run { lines: run, save }
}

fn play_3d_on(
    address: SocketAddr,
    pack: &str,
    seat: &str,
    name: &str,
    steps: &Value,
) -> Vec<String> {
    let (status, lines) = godot3d::play_in(&depiction(pack), address, seat, name, steps, &[]);
    assert!(
        status.success() && godot3d::passed(&lines),
        "{name}: the 3D probe passes:\n{}",
        lines.join("\n")
    );
    lines
}

/// The seat's interactions in a save, in the order the world received them: each request's action
/// type, the kind of its answer, and the fact types it caused (by `caused_by`), sorted.
fn caused(save: &SaveDir, actor: &str) -> Vec<(String, String, Vec<String>)> {
    let tables = Tables::read(Path::new(save.path()));
    let mut by_action: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for (_, bytes) in &tables.facts {
        let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
        if let Causation::Action(action) = fact.caused_by() {
            by_action
                .entry(action.raw())
                .or_default()
                .push(fact.event_type().as_str().to_owned());
        }
    }
    let mut out = Vec::new();
    for (_, bytes) in &tables.journal {
        let entry: JournalEntry = format::decode(bytes, "journal entry").expect("an entry");
        let WorldInput::Dispatch { intent, .. } = &entry.input else {
            continue;
        };
        if intent.actor().raw().to_string() != actor
            || LOCOMOTION.contains(&intent.action_type().as_str())
        {
            continue;
        }
        let kind = match &entry.outcome {
            Outcome::Dispatched(ActionResult::Accepted { .. }) => "accepted",
            Outcome::Dispatched(ActionResult::Rejected(_)) => "rejected",
            _ => "other",
        };
        let mut types = by_action
            .remove(&intent.action_id().raw())
            .unwrap_or_default();
        types.sort();
        out.push((
            intent.action_type().as_str().to_owned(),
            kind.to_owned(),
            types,
        ));
    }
    out
}

/// `AC-13` between two runs' requests, pair by pair in order: the same action types, every pair
/// differing in nothing but `ActorLocation`, the same kind of answer. `Err` names what differs.
fn same_requests(two: &[String], three: &[String]) -> Result<Vec<String>, String> {
    let (a, b) = (interactions(two), interactions(three));
    let types = |run: &[(Value, _)]| -> Vec<String> {
        run.iter()
            .map(|(r, _)| {
                r["request"]["action_type"]
                    .as_str()
                    .unwrap_or("")
                    .to_owned()
            })
            .collect()
    };
    if types(&a) != types(&b) {
        return Err(format!(
            "the action types differ: 2D {:?}, 3D {:?}",
            types(&a),
            types(&b)
        ));
    }
    let mut verdicts = Vec::new();
    for ((line_2d, request_2d), (line_3d, request_3d)) in a.iter().zip(&b) {
        let action = line_2d["request"]["action_type"].as_str().unwrap_or("");
        let differing = differing_fields(request_2d, request_3d);
        if differing.iter().any(|f| *f != RequestField::ActorLocation) {
            return Err(format!(
                "{action}: the requests differ in {differing:?}:\n  2D {}\n  3D {}",
                line_2d["request"], line_3d["request"]
            ));
        }
        let (kind_2d, kind_3d) = (
            result_kind(two, &line_2d["token"]),
            result_kind(three, &line_3d["token"]),
        );
        if kind_2d != kind_3d {
            return Err(format!(
                "{action}: answered {kind_2d} to 2D, {kind_3d} to 3D"
            ));
        }
        verdicts.push(format!("{action}: differing {differing:?}, both {kind_2d}"));
    }
    Ok(verdicts)
}

/// AC-e-1, AC-e-2 on one world: the step list for `actions`, played by both clients, compared.
async fn the_same_requests(pack: &str, tag: &str, actions: &[&str]) -> (Run, Run) {
    let seat = first_seat(pack);
    let genesis = Genesis::of(pack, &seat).await;
    let steps = genesis.steps(actions, WORDS);
    println!("{tag}: steps from the genesis observation of {seat}: {steps}");
    let two = play_2d(pack, &seat, &format!("ac13-{tag}-2d"), &steps).await;
    let three = play_3d(pack, &seat, &format!("ac13-{tag}-3d"), &steps).await;
    let verdicts =
        same_requests(&two.lines, &three.lines).unwrap_or_else(|why| panic!("{tag}: {why}"));
    let sent: Vec<String> = interactions(&two.lines)
        .iter()
        .map(|(r, _)| {
            r["request"]["action_type"]
                .as_str()
                .unwrap_or("")
                .to_owned()
        })
        .collect();
    assert_eq!(
        sent,
        actions_in_order(actions),
        "{tag}: every step sent one request"
    );
    let (caused_2d, caused_3d) = (
        caused(&two.save, &genesis.observer),
        caused(&three.save, &genesis.observer),
    );
    assert_eq!(
        caused_2d, caused_3d,
        "{tag}: the same answers and the same caused fact types, request by request"
    );
    println!("AC-13 with the real clients, {tag}:");
    for (verdict, (_, kind, facts)) in verdicts.iter().zip(&caused_2d) {
        println!("  {verdict}; the save: {kind}, caused {facts:?}");
    }
    (two, three)
}

/// The order `Genesis::steps` plays `actions` in.
fn actions_in_order(actions: &[&str]) -> Vec<String> {
    ["buy", "eat", "talk", "shove", "kick", "throw"]
        .into_iter()
        .filter(|a| actions.contains(a))
        .map(str::to_owned)
        .collect()
}

/// AC-e-1 and AC-e-3 on Social Café: talk, shove, kick, throw; then the 2D talk with other words is
/// told apart, by `payload`.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test ac13_clients -- --ignored"]
async fn social_cafe_talk_shove_kick_throw() {
    let (_, three) = the_same_requests(
        SOCIAL_CAFE,
        "social-cafe",
        &["talk", "shove", "kick", "throw"],
    )
    .await;

    // AC-e-3: the 2D client says something else; the comparison fails and names the payload.
    let seat = first_seat(SOCIAL_CAFE);
    let genesis = Genesis::of(SOCIAL_CAFE, &seat).await;
    let other = play_2d(
        SOCIAL_CAFE,
        &seat,
        "ac13-social-cafe-2d-other",
        &genesis.steps(&["talk", "shove", "kick", "throw"], OTHER_WORDS),
    )
    .await;
    let why = same_requests(&other.lines, &three.lines)
        .expect_err("another utterance is another request");
    assert!(
        why.starts_with("talk") && why.contains("Payload"),
        "AC-e-3: the failure names the talk's payload: {why}"
    );
    println!("AC-e-3 counterfactual: {why}");
}

/// AC-e-1 and AC-e-2 on Market Town: buy, talk, shove, kick, throw.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test ac13_clients -- --ignored"]
async fn market_town_buy_talk_shove_kick_throw() {
    the_same_requests(
        MARKET_TOWN,
        "market-town",
        &["buy", "talk", "shove", "kick", "throw"],
    )
    .await;
}

/// AC-e-2's second world (Q16e-2): Lakeside, where the seat eats what it carries.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test ac13_clients -- --ignored"]
async fn lakeside_eat() {
    the_same_requests(LAKESIDE, "lakeside", &["eat"]).await;
}
