//! IC-2 — `AC-6` through the command an operator types: **restarting the server preserves the world
//! exactly** (step-06 §5).
//!
//! ```text
//! mineworld server worlds/social-cafe --save DIR     a client walks up to Alice and talks to her
//! SIGKILL                                            no shutdown, no checkpoint, no flush
//! mineworld server worlds/social-cafe --save DIR     the same command, a new process
//! ```
//!
//! What the restarted server must show, each read off frames a client receives:
//!
//! ```text
//! the same world        the same instance, and the same persisted revision it was killed at
//! the same people       the barista is the same EntityId; the visitor still stands where it walked
//! the same memory       a second `talk` inside CONVERSATION_GAP records ONE fact, not two: Alice's
//!                       ConversationHistory survived, so the conversation is continuing rather than
//!                       starting — state only the save could have carried across the kill
//! the same identities   the new fact's EventId is the old head's + 1, and the new request's
//!                       ActionId is above every one issued before the kill — the restart defect
//!                       (ActionIds from 1 and a clock from the epoch on every start) is closed
//! ```
//!
//! Then `mineworld replay` re-executes the whole save from genesis and finds it reproduces.
//!
//! No `--agent` here, deliberately: a rule controller's bookkeeping of whom it has answered is the
//! controller's own memory, not world state, and is not persisted (step-06 §9.1 L-3) — with an agent on
//! Alice the restarted controller would answer the old line again and race the test's request.

mod support;

use std::os::unix::process::ExitStatusExt;

use mineworld_contracts::ActionResult;
use serde_json::json;
use support::{Client, SaveDir, Server, run_command, tagged, talk, walk};

const NEXT_TO_ALICE: (i32, i32) = (1_200, 1_000);
/// Where the pack seats the visitor (`worlds/social-cafe/people/visitor.yaml`).
const AT_THE_DOOR: (i32, i32) = (4_600, 200);

#[tokio::test]
async fn a_killed_server_restarts_as_the_same_world_where_it_stopped() {
    let save = SaveDir::new("restart");
    let command = ["server", support::PACK, "--save", save.path()];

    // ── Before. ──────────────────────────────────────────────────────────────────────────────
    let mut first = Server::start(&command).await;
    let mut window = Client::connect(first.address).await;
    let (visitor, welcome) = window.join("visitor").await;
    assert_eq!(
        welcome.revision.map(|revision| revision.raw()),
        Some(1),
        "a new persisted world is at genesis, revision 1"
    );
    let seen = window.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there");
    let cafe = seen
        .self_location()
        .expect("the visitor knows where it is")
        .place()
        .entity_id();
    let strides = window
        .walk_accepted(walk(visitor, cafe, AT_THE_DOOR, NEXT_TO_ALICE))
        .await;
    println!("walked to Alice in {} strides", strides.len());
    assert_eq!(strides.len(), 2, "3 493 mm is two strides of at most 2 m");
    let walked = strides.last().expect("a stride").0;
    let walked_facts: Vec<_> = strides
        .iter()
        .flat_map(|(_, facts)| facts.iter().copied())
        .collect();
    window
        .observation_where("talk to alice available", |observation| {
            support::may_talk_to(observation, alice)
        })
        .await;
    let (talked, talked_facts) = window
        .submit_accepted(talk(visitor, alice, "before the crash"))
        .await;
    assert_eq!(
        talked_facts.len(),
        2,
        "a first exchange starts a conversation and records what was said"
    );
    let status = first.status().await;
    let instance = status["instance"].clone();
    let revision = status["revision"].clone();
    assert_eq!(
        revision,
        json!(4),
        "genesis, the two strides and the talk: four revisions, all committed"
    );
    let last_event = walked_facts
        .iter()
        .chain(&talked_facts)
        .map(|event| event.raw())
        .max()
        .expect("facts were recorded");

    // ── The kill. ────────────────────────────────────────────────────────────────────────────
    let died = first.kill();
    assert_eq!(
        died.signal(),
        Some(9),
        "the server died of SIGKILL: {died:?}"
    );
    drop(window);

    // ── After: the same command, a new process. ──────────────────────────────────────────────
    let second = Server::start(&command).await;
    let status = second.status().await;
    assert_eq!(status["instance"], instance, "the same world, by identity");
    assert_eq!(
        status["revision"], revision,
        "at the revision it was killed at"
    );

    let mut window = Client::connect(second.address).await;
    let (visitor_again, welcome) = window.join("visitor").await;
    assert_eq!(
        visitor_again, visitor,
        "the seat resolves to the same Person"
    );
    assert_eq!(
        welcome.instance.to_string(),
        instance.as_str().expect("a string")
    );
    let (seen_revision, seen) = window.perceived().await;
    assert_eq!(
        seen_revision.map(|revision| revision.raw()),
        Some(4),
        "the first observation after the restart is of revision 4"
    );
    assert_eq!(
        tagged(&seen, "barista"),
        Some(alice),
        "the barista is the same EntityId"
    );
    let standing = seen.self_location().expect("the visitor knows where it is");
    assert_eq!(
        standing
            .local()
            .map(|local| (local.x().value(), local.y().value())),
        Some(NEXT_TO_ALICE),
        "and the visitor stands where it walked before the kill"
    );
    assert!(
        support::may_talk_to(&seen, alice),
        "so Alice is in reach at once"
    );

    let (talked_again, facts) = window
        .submit_accepted(talk(visitor, alice, "after the crash"))
        .await;
    assert_eq!(
        facts.len(),
        1,
        "ONE fact: the conversation continued, because Alice's history survived the kill — a world \
         rebuilt from the pack alone would have started a new conversation and recorded two"
    );
    assert_eq!(
        facts[0].raw(),
        last_event + 1,
        "event identity continues at the old head + 1: nothing reused, nothing skipped"
    );
    assert!(
        talked_again.raw() > talked.raw() && talked_again.raw() > walked.raw(),
        "and the request's identity is above every one issued before the kill ({walked}, {talked} \
         → {talked_again})"
    );
    let after = second.status().await;
    assert_eq!(
        after["revision"],
        json!(5),
        "the four before the kill, and one talk after"
    );
    assert_eq!(after["faults"], json!(0));
    drop(window);
    drop(second);

    // ── The save re-executes from genesis. ───────────────────────────────────────────────────
    let (status, printed) = run_command(&["replay", support::PACK, "--save", save.path()]);
    assert!(
        status.success(),
        "mineworld replay verifies the save: {printed}"
    );
    assert!(
        printed.contains("5 revision(s) re-executed from genesis")
            && printed.contains("head revision 5"),
        "{printed}"
    );
}

/// A request the world refuses is still a revision: it moved the clock, and its `ActionId` must never
/// be issued again — which a restart would do if the journal skipped it.
#[tokio::test]
async fn a_refused_request_is_persisted_so_its_identity_is_never_reissued() {
    let save = SaveDir::new("refused");
    let command = ["server", support::PACK, "--save", save.path()];
    let mut first = Server::start(&command).await;
    let mut window = Client::connect(first.address).await;
    let (visitor, _) = window.join("visitor").await;
    let seen = window.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there");
    // At the door, Alice is out of reach: the server refuses, and that refusal is history too.
    let (refused, result) = window.submit(talk(visitor, alice, "from the door")).await;
    assert!(matches!(result, ActionResult::Rejected(_)), "{result:?}");
    assert_eq!(first.status().await["revision"], json!(2));
    first.kill();
    drop(window);

    let second = Server::start(&command).await;
    let mut window = Client::connect(second.address).await;
    window.join("visitor").await;
    let (next, _) = window
        .submit(talk(visitor, alice, "still at the door"))
        .await;
    assert_eq!(
        next.raw(),
        refused.raw() + 1,
        "the allocator resumed past the refused request's identity"
    );
}
