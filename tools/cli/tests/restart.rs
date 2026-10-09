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

mod fixture;
mod support;

use std::os::unix::process::ExitStatusExt;

use mineworld_contracts::ActionResult;
use serde_json::json;
use support::{Client, SaveDir, Server, run_command, tagged, talk, walk};

/// At the café counter, in reach of Alice behind it (`worldpack/tests/social_cafe.rs` `AT_THE_COUNTER`).
const NEXT_TO_ALICE: (i32, i32) = (6_000, 6_200);
/// Where the pack seats the visitor (`worlds/social-cafe/people/visitor.yaml`).
const AT_THE_DOOR: (i32, i32) = (1_610, 600);

/// How much of the day a hosted world here runs through: it begins at genesis, 00:00, and a world
/// second passes per wall second, so the test is over long before five hours of world time.
const HOSTED_FROM_GENESIS: (i64, i64) = (0, 5 * 3_600);

#[tokio::test]
async fn a_killed_server_restarts_as_the_same_world_where_it_stopped() {
    // The revision literals below hold only while no routine boundary falls as the world runs
    // (step-09 §4.3.7): checked first, so a routine edited into this window fails here, by name.
    fixture::assert_quiet(HOSTED_FROM_GENESIS.0, HOSTED_FROM_GENESIS.1, "restart.rs");
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
    assert_eq!(strides.len(), 4, "7 116 mm is four strides of at most 2 m");
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
        4,
        "a first exchange starts a conversation and records what was said, and the two become \
         acquainted (relationships, once per direction)"
    );
    let status = first.status().await;
    let instance = status["instance"].clone();
    let revision = status["revision"].clone();
    assert_eq!(
        revision,
        json!(6),
        "genesis, the four strides and the talk: six revisions, all committed"
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
        Some(6),
        "the first observation after the restart is of revision 6"
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
        "ONE fact: the conversation continued, because Alice's history survived the kill, and so did
         the two people's acquaintance — a world rebuilt from the pack alone would have started a new
         conversation and a new acquaintance, and recorded four"
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
        json!(7),
        "the six before the kill, and one talk after"
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
        printed.contains("7 revision(s) re-executed from genesis")
            && printed.contains("head revision 7"),
        "{printed}"
    );
}

/// A request the world refuses is still a revision: it moved the clock, and its `ActionId` must never
/// be issued again — which a restart would do if the journal skipped it.
#[tokio::test]
async fn a_refused_request_is_persisted_so_its_identity_is_never_reissued() {
    // Its revision literal (2) holds only while no routine boundary falls as the world runs.
    fixture::assert_quiet(HOSTED_FROM_GENESIS.0, HOSTED_FROM_GENESIS.1, "restart.rs");
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

/// step-12 SB-5 (`F-13`, CP-B3). The reactive controller `--agent alice` is bound at the restart's
/// instant, so a line it answered before the kill is not answered again: after the restart a silent
/// client hears nothing from Alice, and nothing is journaled. Before S11-B the controller started
/// with no memory of whom it had answered and replied to the old line once more.
#[tokio::test]
async fn a_restarted_agent_does_not_answer_again_what_it_answered_before_the_kill() {
    fixture::assert_quiet(HOSTED_FROM_GENESIS.0, HOSTED_FROM_GENESIS.1, "restart.rs");
    let save = SaveDir::new("f13");
    let command = [
        "server",
        support::PACK,
        "--agent",
        "alice",
        "--save",
        save.path(),
    ];

    let mut first = Server::start(&command).await;
    let mut window = Client::connect(first.address).await;
    let (visitor, _) = window.join("visitor").await;
    let seen = window.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there");
    let cafe = seen
        .self_location()
        .expect("the visitor knows where it is")
        .place()
        .entity_id();
    window
        .walk_accepted(walk(visitor, cafe, AT_THE_DOOR, NEXT_TO_ALICE))
        .await;
    window
        .observation_where("talk to alice available", |observation| {
            support::may_talk_to(observation, alice)
        })
        .await;
    window
        .submit_accepted(talk(visitor, alice, "one line, answered once"))
        .await;
    let answered = window
        .observation_where("a reply from alice", |observation| {
            support::own_history(observation).is_some_and(|history| !history.is_empty())
        })
        .await;
    let heard_before = support::own_history(&answered)
        .expect("a history")
        .heard()
        .len();
    first.kill();
    drop(window);

    let second = Server::start(&command).await;
    let mut window = Client::connect(second.address).await;
    window.join("visitor").await;
    let at_restart = second.status().await["revision"].clone();
    // Silent for fifteen wall seconds: fifteen consults of a controller that is due every second.
    let silence = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let mut last = window.observation().await;
    while std::time::Instant::now() < silence {
        last = window.observation().await;
    }
    assert_eq!(
        second.status().await["revision"],
        at_restart,
        "nothing was journaled while the client was silent: Alice did not answer the old line again"
    );
    assert_eq!(
        support::own_history(&last).map_or(0, |history| history.heard().len()),
        heard_before,
        "and the visitor heard nothing new from her"
    );
}
