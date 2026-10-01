//! `AC-15` — **there is only one Alice.**
//!
//! ```text
//! Terminal 1:  mineworld server worlds/social-cafe --agent alice
//! Terminal 2:  a client on the `visitor` seat        ← "the 2D window"
//! Terminal 3:  a client on the `wanderer` seat       ← "the 3D window"
//! ```
//!
//! All three alive at once, against one process, and the claim is that the Alice the second window
//! meets is the Alice the first window spoke to. `docs/MVP.md` §9 says this is the criterion that
//! distinguishes this project from a game: `AC-13` proves two clients *ask* the same question, and
//! `AC-15` proves they are looking at the same world rather than at two consistent copies of one.
//!
//! # What this test is allowed to count as evidence
//!
//! `MVP.md` §9.1 is explicit, and the false success it excludes is the dangerous one:
//!
//! > "Both clients showed the same thing" is **not** evidence. Two clients each holding their own
//! > Alice, kept in step well enough to look identical, would pass that reading and would mean the
//! > opposite of what this project claims.
//!
//! So nothing here asserts that two windows looked alike. Three identities are asserted instead, and
//! each is written out where the test prints its evidence:
//!
//! ```text
//! same world instance            both welcomes and GET /status name one running world
//! same Alice EntityId            the entity one window talked to is the entity the other one met,
//!                                and is the speaker of the reply both of them received
//! same authoritative event       one monotonic EventId sequence across both windows and the agent,
//!   sequence                     with no repeats — two worlds would each start theirs at 1
//! ```
//!
//! and then the thing those three make possible: **Alice tells the second window what the first one
//! said.** That sentence cannot be produced by two synchronised copies, because neither copy was ever
//! told the other's conversation — it can only be produced by one Person holding both.
//!
//! and, since S5, the fourth line of `MVP.md` §9.1:
//!
//! ```text
//! same persisted state       the server runs with --save; both windows' observations name the same
//!   revision                 revision, which GET /status reports as the world's persisted head, and
//!                            which the save on disk holds as its journal head once the server is gone
//!                            (`docs/DECISIONS.md` ARC-25, `server/PROTOCOL.md` §5)
//! ```
//!
//! # Why the evidence is read off frames rather than out of the server
//!
//! Every number below arrived over a WebSocket, in a frame `server/PROTOCOL.md` specifies, from a
//! process started with the command an operator types. Nothing reaches into the world: this test
//! knows exactly what a client knows, which is the only position from which "the two clients see one
//! world" can honestly be checked.

mod support;

use mineworld_contracts::EntityId;
use mineworld_server::RefusalCode;
use serde_json::json;
use support::{Client, SaveDir, Server, may_talk_to, own_history, run_command, tagged, talk, walk};

/// What the 2D window says, and what the 3D window says. Distinct on purpose: the test asserts that
/// the *first* window's words come back out of the mouth of the person the *second* window meets.
const FROM_THE_2D_WINDOW: &str = "hello Alice, this is the 2D window";
const FROM_THE_3D_WINDOW: &str = "hello Alice, this is the 3D window";

/// Conversational distance from Alice, who stands at (1200, 2400) mm: inside `talk`'s declared
/// three-metre reach, which the **server** decides and this test only walks into.
const NEXT_TO_ALICE: (i32, i32) = (1_200, 1_000);
const ALSO_NEXT_TO_ALICE: (i32, i32) = (2_400, 2_400);

/// Where the pack seats the two players (`worlds/social-cafe/people/{visitor,wanderer}.yaml`), which is
/// where each walk starts. The walks are 3 493 mm and 2 970 mm: two `move` strides each, since the
/// server takes at most 2 m per request (`server/PROTOCOL.md` §6.2).
const VISITOR_AT_THE_DOOR: (i32, i32) = (4_600, 200);
const WANDERER_AT_THE_DOOR: (i32, i32) = (4_600, 4_400);

#[tokio::test]
async fn there_is_only_one_alice() {
    let save = SaveDir::new("ac15");
    let server = Server::start(&[
        "server",
        support::PACK,
        "--agent",
        "alice",
        "--save",
        save.path(),
    ])
    .await;

    // ── Three participants, one server. ──────────────────────────────────────────────────────
    let mut two_d = Client::connect(server.address).await;
    let mut three_d = Client::connect(server.address).await;
    let (visitor, world_as_2d_was_told) = two_d.join("visitor").await;
    let (wanderer, world_as_3d_was_told) = three_d.join("wanderer").await;
    // The agent is the third, and it is not this test's client: it was started by the server command
    // and occupies the `alice` seat through the same roster these two just used.
    let status = server.status().await;
    assert_eq!(
        status["clients"],
        json!(3),
        "two clients and one agent-driven Person are connected at once: {status}"
    );

    // ── EVIDENCE 1: same world instance. ─────────────────────────────────────────────────────
    let instance = world_as_2d_was_told.instance;
    assert_eq!(
        instance, world_as_3d_was_told.instance,
        "both windows were told the identity of ONE running world"
    );
    assert_eq!(
        status["instance"],
        json!(instance.to_string()),
        "and it is the instance the world reports about itself"
    );

    // ── EVIDENCE 2: same Alice. ──────────────────────────────────────────────────────────────
    // Found by the tag the pack gives her, never by a hard-coded id: a client learns who somebody is
    // from the world, and this test is allowed to know only what a client knows.
    let seen_by_2d = two_d.observation().await;
    let seen_by_3d = three_d.observation().await;
    let alice = tagged(&seen_by_2d, "barista").expect("the 2D window perceives the barista");
    assert_eq!(
        Some(alice),
        tagged(&seen_by_3d, "barista"),
        "the person the 3D window perceives as the barista is the SAME EntityId"
    );
    assert_ne!(visitor, wanderer, "and the two windows are two people");

    // ── The 2D window speaks to her. ─────────────────────────────────────────────────────────
    // First it has to be in reach, which the server decides: the pack puts the visitor at the door.
    assert!(
        !may_talk_to(&seen_by_2d, alice),
        "the server refuses at the door, and the client did not work that out for itself"
    );
    let cafe = seen_by_2d
        .self_location()
        .expect("the visitor knows where it is")
        .place()
        .entity_id();
    let strides = two_d
        .walk_accepted(walk(visitor, cafe, VISITOR_AT_THE_DOOR, NEXT_TO_ALICE))
        .await;
    println!("the 2D window walked to Alice in {} strides", strides.len());
    assert_eq!(strides.len(), 2, "3 493 mm is two strides of at most 2 m");
    let in_reach = two_d
        .observation_where("talk to alice available", |observation| {
            may_talk_to(observation, alice)
        })
        .await;
    assert!(may_talk_to(&in_reach, alice));

    let (first_action, facts_of_the_2d_talk) = two_d
        .submit_accepted(talk(visitor, alice, FROM_THE_2D_WINDOW))
        .await;
    assert_eq!(
        facts_of_the_2d_talk.len(),
        2,
        "a first exchange records both that a conversation started and what was said"
    );

    // Alice answers, because a controller read her history out of her own observation and decided to.
    // The 2D window learns of it from *its* own history, which is where the reply was written.
    let answered = two_d
        .observation_where("a reply from alice", |observation| {
            own_history(observation).is_some_and(|history| !history.is_empty())
        })
        .await;
    let what_2d_was_told = own_history(&answered).expect("a history was disclosed");
    let reply_to_2d = what_2d_was_told.heard().last().expect("one entry at least");
    assert_eq!(
        reply_to_2d.speaker().entity_id(),
        alice,
        "the reply came from the barista this window perceives"
    );
    assert!(
        reply_to_2d
            .utterance()
            .as_str()
            .contains(FROM_THE_2D_WINDOW),
        "and she quoted it back, so she plainly heard it: {}",
        reply_to_2d.utterance()
    );

    // ── The 3D window walks up to the same Alice. ────────────────────────────────────────────
    let strides = three_d
        .walk_accepted(walk(
            wanderer,
            cafe,
            WANDERER_AT_THE_DOOR,
            ALSO_NEXT_TO_ALICE,
        ))
        .await;
    println!("the 3D window walked to Alice in {} strides", strides.len());
    assert_eq!(strides.len(), 2, "2 970 mm is two strides of at most 2 m");
    three_d
        .observation_where("talk to alice available", |observation| {
            may_talk_to(observation, alice)
        })
        .await;
    let (second_action, facts_of_the_3d_talk) = three_d
        .submit_accepted(talk(wanderer, alice, FROM_THE_3D_WINDOW))
        .await;

    // ── THE CLAIM: she knows it happened. ────────────────────────────────────────────────────
    let carried_forward = three_d
        .observation_where("a reply from alice", |observation| {
            own_history(observation).is_some_and(|history| !history.is_empty())
        })
        .await;
    let what_3d_was_told = own_history(&carried_forward).expect("a history was disclosed");
    let reply_to_3d = what_3d_was_told.heard().last().expect("one entry at least");
    assert_eq!(
        reply_to_3d.speaker().entity_id(),
        alice,
        "the same entity answered both windows"
    );
    assert!(
        reply_to_3d
            .utterance()
            .as_str()
            .contains(FROM_THE_2D_WINDOW),
        "SHE TOLD THE 3D WINDOW WHAT THE 2D WINDOW SAID. Two synchronised copies of Alice could \
         not produce this sentence, because neither was told the other's conversation. What she \
         said was: {}",
        reply_to_3d.utterance()
    );
    assert!(
        reply_to_3d
            .utterance()
            .as_str()
            .contains(&visitor.raw().to_string()),
        "and she named the other speaker by identity: {}",
        reply_to_3d.utterance()
    );

    // ── EVIDENCE 3: one authoritative event sequence. ────────────────────────────────────────
    // Every fact either window caused, in the order they were caused. If each window had its own
    // world, each would have allocated its own identities from the same starting point and these
    // would collide. They do not: they are one strictly increasing sequence, allocated once.
    let sequence: Vec<u64> = facts_of_the_2d_talk
        .iter()
        .chain(facts_of_the_3d_talk.iter())
        .map(|event| event.raw())
        .collect();
    assert!(
        sequence.windows(2).all(|pair| pair[0] < pair[1]),
        "one monotonic event sequence across both windows: {sequence:?}"
    );
    assert!(
        second_action.raw() > first_action.raw(),
        "and one request-identity allocator: {first_action} then {second_action}"
    );
    // Alice's two replies were dispatched between the two talks, so the gap between the windows'
    // facts is the agent's own — the third participant's actions are in the same sequence.
    assert!(
        facts_of_the_3d_talk[0].raw() > facts_of_the_2d_talk[1].raw() + 1,
        "the agent's facts are in the same sequence, between the two windows': {sequence:?}"
    );

    // ── EVIDENCE 4: one persisted state revision. ────────────────────────────────────────────
    // Alice has answered both windows and nobody else is acting, so the world's persisted head is
    // settled. Both windows must come to observe exactly that revision — the state they look at is one
    // committed state of one world, not two states that happen to look alike.
    let head = server.status().await["revision"]
        .as_u64()
        .expect("a persisted world reports its revision");
    assert!(
        head >= 7,
        "genesis, two arrivals, two talks and Alice's two replies are seven revisions at least: {head}"
    );
    let mut seen_at = Vec::new();
    for window in [&mut two_d, &mut three_d] {
        let deadline = std::time::Instant::now() + support::PATIENCE;
        loop {
            let (revision, _) = window.perceived().await;
            let revision =
                revision.expect("every observation of a persisted world names a revision");
            assert!(
                revision.raw() <= head,
                "no window sees past the persisted head"
            );
            if revision.raw() == head {
                seen_at.push(revision.raw());
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the window reached revision {head}"
            );
        }
    }
    assert_eq!(
        seen_at,
        vec![head, head],
        "both windows observe the same persisted revision"
    );
    drop(two_d);
    drop(three_d);
    drop(server);
    // And it is persisted: with the server gone, the save on disk holds exactly that head, and its
    // whole history re-executes.
    let (status, printed) = run_command(&["replay", support::PACK, "--save", save.path()]);
    assert!(status.success(), "the save verifies: {printed}");
    assert!(
        printed.contains(&format!("head revision {head}")),
        "the save's head is the revision both windows observed: {printed}"
    );

    // ── What the evidence actually was. ──────────────────────────────────────────────────────
    println!("AC-15 evidence, read off the frames two clients received:");
    println!("  same world instance          {instance}");
    println!("  same Alice EntityId          {alice}");
    println!("  one event sequence           {sequence:?}");
    println!("  one persisted revision       {head} (both windows; GET /status; the save's head)");
    println!("  one action-id allocator      {first_action}, {second_action}");
    println!("  2D window ({visitor}) said   {FROM_THE_2D_WINDOW:?}");
    println!("  3D window ({wanderer}) said  {FROM_THE_3D_WINDOW:?}");
    println!(
        "  Alice told the 3D window     {:?}",
        reply_to_3d.utterance().as_str()
    );
}

/// The false success `AC-15` exists to exclude, built on purpose so that the evidence above is
/// demonstrably discriminative.
///
/// Two servers, two Alices, one client on each. Everything an "appearance" test could look at agrees:
/// both windows perceive a barista, at the same position, with the same tags, **and with the same
/// `EntityId`** — because two loads of one World Pack resolve the same keys to the same ids, which is
/// `AC-12` working. A test that asserted "both clients showed the same thing" would pass here, and it
/// would be wrong.
///
/// Two things separate this from the real thing, and they are the two the evidence names: the world
/// instances differ, and the carry-forward does not happen. Alice number two was never told what was
/// said to Alice number one, so she cannot repeat it — which is precisely why that sentence is the
/// claim rather than a flourish.
#[tokio::test]
async fn two_servers_are_two_worlds_and_the_evidence_can_tell() {
    let first = Server::start(&["server", support::PACK, "--agent", "alice"]).await;
    let second = Server::start(&["server", support::PACK, "--agent", "alice"]).await;

    let mut two_d = Client::connect(first.address).await;
    let mut three_d = Client::connect(second.address).await;
    let (visitor, first_world) = two_d.join("visitor").await;
    let (wanderer, second_world) = three_d.join("wanderer").await;

    let seen_by_2d = two_d.observation().await;
    let seen_by_3d = three_d.observation().await;
    let alice_here = tagged(&seen_by_2d, "barista").expect("a barista");
    let alice_there = tagged(&seen_by_3d, "barista").expect("a barista");

    // The part that makes an appearance test worthless.
    assert_eq!(
        alice_here, alice_there,
        "two loads of one pack resolve the same key to the same id, so an EntityId alone proves          NOTHING about identity across two worlds"
    );
    assert_eq!(
        seen_by_2d.entity(alice_here).map(|alice| alice.location()),
        seen_by_3d.entity(alice_there).map(|alice| alice.location()),
        "and she is standing in the same place in both, because it is the same authored world"
    );

    // The part that tells them apart.
    assert_ne!(
        first_world.instance, second_world.instance,
        "two running worlds are two instances"
    );

    let cafe = seen_by_2d
        .self_location()
        .expect("a location")
        .place()
        .entity_id();
    for (client, actor, start, position, words) in [
        (
            &mut two_d,
            visitor,
            VISITOR_AT_THE_DOOR,
            NEXT_TO_ALICE,
            FROM_THE_2D_WINDOW,
        ),
        (
            &mut three_d,
            wanderer,
            WANDERER_AT_THE_DOOR,
            ALSO_NEXT_TO_ALICE,
            FROM_THE_3D_WINDOW,
        ),
    ] {
        client
            .walk_accepted(walk(actor, cafe, start, position))
            .await;
        client
            .observation_where("talk available", |observation| {
                may_talk_to(observation, alice_here)
            })
            .await;
        client.submit_accepted(talk(actor, alice_here, words)).await;
    }

    let answered = three_d
        .observation_where("a reply", |observation| {
            own_history(observation).is_some_and(|history| !history.is_empty())
        })
        .await;
    let reply = own_history(&answered).expect("a history");
    let said = reply.heard().last().expect("an entry").utterance();
    assert!(
        !said.as_str().contains(FROM_THE_2D_WINDOW),
        "the other server's Alice cannot repeat what was said to this one, because she is a \
         different person who was never told: {said}"
    );
    assert!(
        said.as_str().contains("first person to speak to me"),
        "she believes she has met one person, which is true of her world: {said}"
    );
}

/// Killing one client leaves the world and the other client running.
///
/// One of this PR's objective gates, and the reason the world never waits for a client: a connection
/// that vanishes takes its subscription with it and nothing else.
#[tokio::test]
async fn killing_one_window_leaves_the_world_and_the_other_window_running() {
    let server = Server::start(&["server", support::PACK, "--agent", "alice"]).await;
    let mut two_d = Client::connect(server.address).await;
    let mut three_d = Client::connect(server.address).await;
    two_d.join("visitor").await;
    let (wanderer, _) = three_d.join("wanderer").await;

    let seen = three_d.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there");
    let cafe = seen
        .self_location()
        .expect("the wanderer knows where it is")
        .place()
        .entity_id();

    two_d.observation().await;
    two_d.disconnect().await;

    // The world is still there, and so is the other client — which can still act, and is still
    // answered by the same Alice.
    three_d
        .walk_accepted(walk(
            wanderer,
            cafe,
            WANDERER_AT_THE_DOOR,
            ALSO_NEXT_TO_ALICE,
        ))
        .await;
    three_d
        .observation_where("talk to alice available", |observation| {
            may_talk_to(observation, alice)
        })
        .await;
    let (_, facts) = three_d
        .submit_accepted(talk(wanderer, alice, "still here"))
        .await;
    assert!(!facts.is_empty());

    let status = server.status().await;
    assert_eq!(
        status["clients"],
        json!(2),
        "the dead connection is gone and the live ones are not: {status}"
    );
    assert_eq!(
        status["faults"],
        json!(0),
        "and nothing broke while it died: {status}"
    );
}

/// A client cannot read a stranger's memory, which is what makes the disclosure in the test above a
/// disclosure rather than a leak.
///
/// The counterfactual to `INV-13`: if an observation exposed a component because it exists, this
/// window would be able to read what Alice remembers, and `AC-15`'s carry-forward would prove nothing
/// about identity — it would only prove that a client can see everything.
#[tokio::test]
async fn a_window_is_told_what_it_heard_and_never_what_somebody_else_heard() {
    let server = Server::start(&["server", support::PACK, "--agent", "alice"]).await;
    let mut window = Client::connect(server.address).await;
    let (visitor, _) = window.join("visitor").await;

    let seen = window.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there");
    let cafe = seen
        .self_location()
        .expect("the visitor knows where it is")
        .place()
        .entity_id();
    window
        .walk_accepted(walk(visitor, cafe, VISITOR_AT_THE_DOOR, NEXT_TO_ALICE))
        .await;
    window
        .observation_where("talk to alice available", |observation| {
            may_talk_to(observation, alice)
        })
        .await;
    window
        .submit_accepted(talk(visitor, alice, "do you remember me"))
        .await;

    let answered = window
        .observation_where("a reply from alice", |observation| {
            own_history(observation).is_some_and(|history| !history.is_empty())
        })
        .await;

    // Alice is perceived — she is in the same room — and what she remembers is not in this frame.
    let she = answered.entity(alice).expect("alice is perceived");
    assert!(
        !she.components()
            .iter()
            .any(|record| record.component_type().as_str() == "conversation-history"),
        "a client may not read the memory of the person it is talking to: {:?}",
        she.components()
    );
    assert!(
        own_history(&answered).is_some(),
        "while its own is disclosed, through the same mechanism"
    );
}

/// A request that asks the world to act as somebody else is refused — so nothing in `AC-15` can be
/// arranged by one window driving the other's Person, or by either of them driving Alice.
#[tokio::test]
async fn one_window_cannot_act_as_the_other_nor_as_alice() {
    let server = Server::start(&["server", support::PACK]).await;
    let mut two_d = Client::connect(server.address).await;
    two_d.join("visitor").await;
    let seen = two_d.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there");
    let wanderer = EntityId::from_raw(5);

    assert_eq!(
        two_d
            .submit_refused(talk(wanderer, alice, "I am the wanderer"))
            .await,
        RefusalCode::ActorNotObserver,
    );
    assert_eq!(
        two_d
            .submit_refused(talk(alice, wanderer, "I am Alice"))
            .await,
        RefusalCode::ActorNotObserver,
        "a seat a client did not occupy is not a Person it may act as, even one an agent is driving"
    );
}

/// The reply the agent produced is a projection of the log, not a memory the controller kept.
///
/// If the controller held Alice's memory, disabling nothing and restarting nothing would still show a
/// history that the *world* does not have. Here the entry a window reads is the world's own component,
/// disclosed — so the count it sees is the count the log produced.
#[tokio::test]
async fn what_a_window_reads_is_the_world_s_own_projection() {
    let server = Server::start(&["server", support::PACK, "--agent", "alice"]).await;
    let mut window = Client::connect(server.address).await;
    let (visitor, _) = window.join("visitor").await;
    let seen = window.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there");
    let cafe = seen
        .self_location()
        .expect("the visitor knows where it is")
        .place()
        .entity_id();
    window
        .walk_accepted(walk(visitor, cafe, VISITOR_AT_THE_DOOR, NEXT_TO_ALICE))
        .await;
    window
        .observation_where("talk to alice available", |observation| {
            may_talk_to(observation, alice)
        })
        .await;

    for said in ["one", "two"] {
        window.submit_accepted(talk(visitor, alice, said)).await;
        window
            .observation_where("a reply", |observation| {
                own_history(observation).is_some_and(|history| {
                    history.heard().last().is_some_and(|heard| {
                        heard.speaker().entity_id() == alice
                            && heard.utterance().as_str().contains(said)
                    })
                })
            })
            .await;
    }

    let final_view = window.observation().await;
    let history = own_history(&final_view).expect("a history");
    assert_eq!(
        history.len(),
        2,
        "one entry per thing Alice said to this window, each written while reducing a `spoke` \
         fact: {:?}",
        history.heard()
    );
    assert!(
        history
            .heard()
            .iter()
            .all(|heard| heard.speaker().entity_id() == alice),
        "and every one of them is from her"
    );
}
