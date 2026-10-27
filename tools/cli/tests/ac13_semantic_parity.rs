//! `AC-13` — a click and a walk-up-look-at-press ask the world for the same thing.
//!
//! ```text
//! 2D:  click a person                              ─┐
//!                                                   ├─►  the same actor, action type, target and
//! 3D:  walk up, look at them, press a key          ─┘    payload, resolved by the same system to
//!                                                        the same result
//! ```
//!
//! # Where these requests come from, and why that matters
//!
//! Not from this file. A Rust test that wrote both frames itself would prove that the comparison
//! works and nothing about any client. The two frames below were **submitted by the real Godot
//! client protocol module**, driven by the real engine against the real server, in its two flavours:
//! one that reports no position because it models none, and one that reports the position it walked
//! to. They are committed as they went out, in `clients/protocol/evidence/`, along with the
//! transcripts of the runs that produced them (`clients/protocol/README.md` says how to reproduce
//! them).
//!
//! Frozen real evidence, in the sense `test-ci-gate-rules.md` §25 requires: the expected values do
//! not come from the implementation under test.
//!
//! # And the comparison comes from the server
//!
//! `mineworld_server::semantic_core` and `differing_fields`, which is `MVP.md` §9's correction as a
//! mechanism: *"the comparison is defined once, in the server, so the S12/S14 test cannot quietly
//! compare a different set of fields."* This test names the one difference it permits, rather than
//! comparing four fields it chose and ignoring what it forgot.

mod support;

use std::collections::BTreeMap;
use std::path::Path;

use mineworld_contracts::{ActionRequest, ActionResult, EntityId, Rejection};
use mineworld_server::{
    RequestField, WireObservation, WirePayload, differing_fields, semantic_core,
};
use serde_json::Value;
use support::{Client, Server};

/// Where the real client's submitted frames are kept.
const EVIDENCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../clients/protocol/evidence"
);

/// What the two clients submitted, in the order they submitted it.
///
/// Read through the contract's own deserialization, so a frame that would not be a legal
/// `ActionRequest` fails here rather than being compared as JSON: the agreement check between the
/// envelope's `action_type` and the payload record's is applied by the contract, which is the check a
/// hand-built client frame can otherwise fail (`spike/FINDINGS.md` F8.1).
fn submitted_by(flavour: &str) -> Vec<Submitted> {
    let path = Path::new(EVIDENCE).join(format!("request-{flavour}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    let recorded: Vec<Value> = serde_json::from_str(&text).expect("a list of submitted requests");
    recorded
        .into_iter()
        .map(|entry| {
            let action_type = entry["request"]["action_type"]
                .as_str()
                .expect("a request states its action type")
                .to_owned();
            let frame = entry["request"].clone();
            let request: ActionRequest<WirePayload> = serde_json::from_value(frame.clone())
                .expect("what the client sent is a legal ActionRequest");
            Submitted {
                action_type,
                frame,
                request,
            }
        })
        .collect()
}

/// One request a client sent: the frame as it went out, and the contract's reading of it.
///
/// Both, deliberately. The comparison wants the typed value; the replay wants the **original bytes**,
/// because re-serializing is not what the client did — `WirePayload` writes itself the way the kernel
/// takes a payload, which is not the way a client writes one.
struct Submitted {
    action_type: String,
    frame: Value,
    request: ActionRequest<WirePayload>,
}

/// What each entity id in the recorded frames names in the world a replay meets.
///
/// Ids are allocated per world instance, so a pack that gains a place renumbers the people in it and
/// a frame recorded before that names somebody else. The client chose each id by a role, never by a
/// literal — itself (the seat it joined), whoever the world tags `barista`, and the place it stood
/// in — so each is translated by that same role, read from this world's own first observation.
/// Positions are not translated: they are the café's, and a change to the café is a change to what
/// the client would send, which needs a new recording.
fn translation(
    recorded: &[Submitted],
    observer: EntityId,
    seen: &WireObservation,
) -> BTreeMap<String, String> {
    let named = |action: &str, read: fn(&Value) -> &Value| -> String {
        let frame = &recorded
            .iter()
            .find(|submitted| submitted.action_type == action)
            .unwrap_or_else(|| panic!("the run submitted a {action}"))
            .frame;
        read(frame)
            .as_str()
            .unwrap_or_else(|| panic!("the {action} names an entity: {frame}"))
            .to_owned()
    };
    let place = seen
        .entity(observer)
        .and_then(|me| me.location())
        .map(|at| {
            serde_json::to_value(at).expect("a location serializes")["place"]["entity"].clone()
        })
        .and_then(|entity| entity.as_str().map(str::to_owned))
        .expect("the seat is placed in this world");
    let barista = support::tagged(seen, "barista").expect("this world has a barista");
    BTreeMap::from([
        (named("talk", |frame| &frame["actor"]), observer.to_string()),
        (named("talk", |frame| &frame["target"]), barista.to_string()),
        (
            named("walk-to", |frame| {
                &frame["payload"]["payload"]["to"]["place"]["place"]["entity"]
            }),
            place,
        ),
    ])
}

/// A recorded frame with every entity it names — `actor`, `target`, and each `entity` of a
/// reference — replaced by what [`translation`] says it names in this world. An id the translation
/// does not know fails the test rather than being sent as recorded.
fn translated(frame: Value, into: &BTreeMap<String, String>) -> Value {
    match frame {
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .map(|(field, value)| {
                    let value = match (&value, field.as_str()) {
                        (Value::String(id), "actor" | "target" | "entity") => {
                            Value::String(into.get(id).cloned().unwrap_or_else(|| {
                                panic!("the recorded {field} {id} has no role in this replay")
                            }))
                        }
                        _ => translated(value, into),
                    };
                    (field, value)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| translated(item, into))
                .collect(),
        ),
        other => other,
    }
}

/// The `talk` request out of one client's run.
fn talk_from(flavour: &str) -> ActionRequest<WirePayload> {
    submitted_by(flavour)
        .into_iter()
        .find(|submitted| submitted.action_type == "talk")
        .map(|submitted| submitted.request)
        .unwrap_or_else(|| panic!("the {flavour} run submitted a talk"))
}

/// The whole of `AC-13`'s first half, read off what two real clients actually sent.
#[test]
fn a_click_and_a_walk_up_submit_the_same_semantic_core() {
    let clicked = talk_from("2d");
    let walked = talk_from("3d");

    assert_eq!(
        semantic_core(&clicked),
        semantic_core(&walked),
        "actor, action type, target and payload — what the world is being asked for"
    );
    assert_eq!(
        differing_fields(&clicked, &walked),
        vec![RequestField::ActorLocation],
        "and the ONE field the contract intends to differ: a 3D client reports the position it \
         walked to, a 2D client that models no position sends none"
    );

    // Named explicitly, so that a reader of this test sees which difference was permitted and why.
    assert_eq!(
        clicked.actor_location(),
        None,
        "the 2D client reported none"
    );
    assert!(
        walked.actor_location().is_some(),
        "the 3D client reported one"
    );
    assert!(
        !RequestField::ActorLocation.is_semantic_core(),
        "and it is not part of the core"
    );
}

/// `AC-13`'s second half: **resolved by the same system to the same result.**
///
/// Both frames are replayed against a server of their own — the real binary, the real pack — so that
/// each meets an identical world and the two answers are comparable. The walk that precedes each
/// `talk` is the client's own, replayed as it was sent — its entity ids translated by role into the
/// replay's world ([`translation`]), so a pack that gains a place does not stale the evidence: one
/// `walk-to` naming the point beside Alice,
/// then the `walk-step`s the client paced (`ADOPTION.md` §4.2, `ARC-75`). Without it the server
/// refuses for distance, which is the server deciding and not this test. The server plans the route
/// round the café's furniture; each request the client sent must be one it accepts, except that the
/// walk's last `walk-step` may be answered `precondition_failed` — the walk being over — and nothing
/// of the walk may follow that.
#[tokio::test]
async fn both_are_resolved_by_the_same_system_to_the_same_result() {
    let mut answers = Vec::new();
    for flavour in ["2d", "3d"] {
        let server = Server::start(&["server", support::PACK]).await;
        let mut client = Client::connect(server.address).await;
        let (observer, _) = client.join("visitor").await;
        let seen = client.observation().await;
        let recorded = submitted_by(flavour);
        let into_this_world = translation(&recorded, observer, &seen);

        let mut talk = None;
        let mut walks = 0;
        let mut steps = 0;
        let mut walk_over = false;
        for submitted in recorded {
            let kind = submitted.action_type.as_str();
            if kind == "walk-step" {
                // Paced as the client paced it, inside the host's request budget (`ARC-97`).
                tokio::time::sleep(support::STEP_PACE).await;
            }
            let (_, result) = client
                .submit(translated(submitted.frame, &into_this_world))
                .await;
            match kind {
                "walk-to" => {
                    walks += 1;
                    assert!(
                        matches!(result, ActionResult::Accepted { .. }),
                        "{flavour}: the walk the client asked for is one the server plans: {result:?}"
                    );
                }
                "walk-step" => {
                    assert!(!walk_over, "{flavour}: no step follows the walk's end");
                    match result {
                        ActionResult::Accepted { .. } => steps += 1,
                        ActionResult::Rejected(Rejection::PreconditionFailed) => walk_over = true,
                        other => panic!(
                            "{flavour}: every step the client sent is one the server accepts, or \
                             the walk is over: {other:?}"
                        ),
                    }
                }
                "talk" => talk = Some(result),
                other => panic!("{flavour}: the client walked and spoke, and sent no {other}"),
            }
        }
        println!(
            "{flavour}: the client walked in {steps} step(s) of {walks} walk(s) before it spoke"
        );
        assert_eq!(
            walks, 1,
            "{flavour}: one walk-to, to the point beside Alice"
        );
        assert!(
            steps >= 2,
            "{flavour}: the walk to Alice is more than one step"
        );
        answers.push((flavour, talk.expect("the run submitted a talk")));
    }

    let (first_flavour, first) = &answers[0];
    let (second_flavour, second) = &answers[1];
    assert!(
        matches!(first, ActionResult::Accepted { .. }),
        "{first_flavour}: {first:?}"
    );
    assert_eq!(
        first, second,
        "{first_flavour} and {second_flavour} were answered identically — the same system \
         resolved both to the same result, which is what AC-13 asks",
    );
}
