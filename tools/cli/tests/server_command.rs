//! `mineworld server worlds/social-cafe` — the acceptance this PR is judged on, run for real.
//!
//! Nothing here is mocked and nothing is in-process. The test starts the **actual binary** cargo
//! built, pointed at the **actual World Pack** in the repository, on an ephemeral port; then it
//! connects a real WebSocket client, occupies a seat the pack offers, and reads what the server
//! sends. That is the whole of PR 05c's A4:
//!
//! ```text
//! mineworld server worlds/social-cafe starts
//! a client connects to it
//! and what it perceives is the world the YAML describes
//! ```
//!
//! The last line is what makes this more than a smoke test: the observation has to name Alice, at the
//! position `people/alice.yaml` authored, with `talk` offered against her and refused for distance —
//! all decided by the server, from a file on disk, through the whole stack.
//!
//! The harness moved to `support/mod.rs` in PR 05d, where the `AC-15` test needs the same real binary
//! and real sockets. Nothing about what is asserted here changed with it.

mod support;

use mineworld_server::ServerFrame;
use serde_json::json;
use support::{Client, Server, may_talk_to, tagged};

#[tokio::test]
async fn the_server_starts_from_the_pack_and_says_what_it_is_hosting() {
    let server = Server::start(&["server", support::PACK]).await;

    let status = server.status().await;

    assert_eq!(
        status["entities"], 6,
        "the world the pack describes: two places and four people — {status}",
    );
    let systems: Vec<&str> = status["systems"]
        .as_array()
        .expect("a list of systems")
        .iter()
        .map(|system| system["system"].as_str().expect("a name"))
        .collect();
    assert_eq!(
        systems,
        ["presence", "movement", "conversation"],
        "in the order world.yaml states, which is the order they reduce in",
    );
    assert_eq!(
        status["seats"],
        json!(["alice", "visitor", "wanderer"]),
        "the seats the pack offers: two for players, and Alice for the agent that drives her \
         through the same roster — {status}",
    );
    assert!(
        status["instance"].as_str().is_some_and(|id| id.len() == 32),
        "and which running world this is, which AC-15's evidence names: {status}",
    );
}

#[tokio::test]
async fn a_client_connects_to_the_hosted_pack_and_perceives_the_world_the_yaml_describes() {
    let server = Server::start(&["server", support::PACK]).await;
    let mut client = Client::connect(server.address).await;

    // The seat the pack offers, by the authoring key `world.yaml` names — a client never names an
    // entity, and this is the whole of why the roster is the world's.
    let (observer, world) = client.join("visitor").await;
    assert_eq!(
        observer.raw(),
        5,
        "the visitor is the fifth entity the pack allocates (two places, then alice and bob) — \
         deterministically, every time",
    );
    assert_eq!(world.entities, 6);

    // And then the world itself, as this observer perceives it.
    let observation = client.observation().await;
    let perceived: Vec<u64> = observation
        .entities()
        .iter()
        .map(|entity| entity.id().raw())
        .collect();
    assert_eq!(
        perceived,
        [1, 3, 4, 5, 6],
        "the café and everybody in it, by the ids the pack resolved its keys to — and not the \
         street (2), which is another place",
    );

    let alice = tagged(&observation, "barista").expect("alice is perceived, by her tag");
    assert_eq!(alice.raw(), 3);
    let position = observation
        .entity(alice)
        .and_then(|alice| alice.location())
        .and_then(|location| location.local())
        .expect("the position people/alice.yaml authored");
    assert_eq!(
        (position.x().value(), position.y().value()),
        (1200, 2400),
        "millimetres, exactly as the file wrote them, through the whole stack",
    );

    let talk: Vec<(bool, Option<u64>)> = observation
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == "talk")
        .map(|affordance| {
            (
                affordance.is_available(),
                affordance.target().map(|target| target.raw()),
            )
        })
        .collect();
    assert_eq!(
        talk.len(),
        3,
        "talk offered against the three other people: {talk:?}",
    );
    assert!(
        talk.iter().all(|(available, _)| !available),
        "and refused for all of them, because the pack puts the visitor at the door — the server \
         decided that, not the client: {talk:?}",
    );
    assert!(
        !may_talk_to(&observation, alice),
        "which is the same answer read the way a client reads it",
    );

    // `arrive` is retired (`DECISIONS.md` `ARC-26`): no system in this world provides it, so a
    // client that still sends it is answered `unavailable`, and nobody is moved. Moving is `move`.
    let (_, answer) = client
        .submit(json!({
            "actor": observer,
            "action_type": "arrive",
            "target": null,
            "payload": { "action_type": "arrive", "payload": { "location": {
                "place": { "entity": "1", "entity_type": "place" },
                "local": { "x": 1200, "y": 1000, "z": 0 },
                "facing": null,
            } } },
            "actor_location": null,
        }))
        .await;
    assert_eq!(answer, mineworld_contracts::ActionResult::Unavailable);
}

#[tokio::test]
async fn a_seat_the_pack_does_not_offer_is_refused() {
    let server = Server::start(&["server", support::PACK]).await;
    let mut client = Client::connect(server.address).await;

    // `bob` exists in this world and is not a seat: `world.yaml` offers `visitor`, `wanderer` and
    // `alice`, so nothing can connect *as* him. The roster is the world's, not the client's.
    client.send(json!({ "t": "join", "seat": "bob" })).await;
    let frame = client.frame().await;
    assert!(
        matches!(frame, ServerFrame::Refused { .. }),
        "got: {frame:?}",
    );
}
