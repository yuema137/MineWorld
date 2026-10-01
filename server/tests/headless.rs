//! The host with no transport attached: the world, its seats, and two observers.
//!
//! `docs/ENGINEERING_STANDARDS.md` §22 requires the authoritative simulation to run and be testable
//! with nothing rendering and nobody connected, and `NETWORKING.md` §10 requires networking to be
//! removable from the picture entirely. These tests are that requirement exercised: everything here
//! goes through the same `WorldHost` a WebSocket session uses, with no socket in sight.
//!
//! They also own the two claims a socket cannot honestly demonstrate:
//!
//! ```text
//! the world is not blocked by a client that never reads its observations
//! the server allocates request identity, and the numbers are the server's own
//! ```

mod support;

use std::time::Duration;

use mineworld_contracts::{ActionRecord, ActionRequest, ActionResult, EntityId};
use mineworld_server::{RefusalCode, WorldHost};

use support::{ALICE, BOB, CAROL, Speak, Whisper, brisk, build, key, payload};

/// How long a test waits for something that should take milliseconds.
const PATIENCE: Duration = Duration::from_secs(5);

async fn host() -> WorldHost {
    WorldHost::spawn(brisk(), build)
        .await
        .expect("the world is assembled and its thread starts")
}

fn speak(actor: EntityId, target: EntityId, words: &str) -> ActionRequest {
    ActionRequest::new(
        actor,
        ActionRecord::new::<Speak>(payload(&Speak {
            words: words.to_owned(),
        })),
    )
    .with_target(target)
}

fn whisper(actor: EntityId, words: &str) -> ActionRequest {
    ActionRequest::new(
        actor,
        ActionRecord::new::<Whisper>(payload(&Whisper {
            words: words.to_owned(),
        })),
    )
}

/// Two seats, two observers, two different views of one world — before any client exists.
#[tokio::test]
async fn two_observers_of_one_world_receive_two_different_observations() {
    let host = host().await;
    let mut alice = host.join(key(ALICE)).await.expect("alice is a seat");
    let mut bob = host.join(key(BOB)).await.expect("bob is a seat");

    assert_ne!(
        alice.observer(),
        bob.observer(),
        "two seats are two observers"
    );

    let seen_by_alice = tokio::time::timeout(PATIENCE, alice.observations().recv())
        .await
        .expect("an observation arrives")
        .expect("the world is running");
    assert_eq!(
        seen_by_alice.revision, None,
        "a world that is not persisted has no persisted revision to name"
    );
    let seen_by_alice = seen_by_alice.observation;
    let seen_by_bob = tokio::time::timeout(PATIENCE, bob.observations().recv())
        .await
        .expect("an observation arrives")
        .expect("the world is running")
        .observation;

    assert_eq!(seen_by_alice.observer(), alice.observer());
    assert_eq!(seen_by_bob.observer(), bob.observer());

    let alice_sees = support::perceived_ids(&seen_by_alice);
    let bob_sees = support::perceived_ids(&seen_by_bob);
    assert!(
        alice_sees.contains(&alice.observer()) && !alice_sees.contains(&bob.observer()),
        "alice perceives her own room and not bob's: {alice_sees:?}"
    );
    assert!(
        bob_sees.contains(&bob.observer()) && !bob_sees.contains(&alice.observer()),
        "bob perceives his own room and not alice's: {bob_sees:?}"
    );
    assert_eq!(alice_sees.len(), 2, "alice and carol are in the cafe");
    assert_eq!(bob_sees.len(), 2, "bob and dave are in the street");
}

/// The constraint that cannot be shown through a socket, because an operating system's own buffers
/// would absorb it: a subscriber that never reads loses observations, and the world keeps working.
#[tokio::test]
async fn the_world_keeps_working_while_a_subscriber_never_reads() {
    let host = host().await;
    let unread = host.join(key(ALICE)).await.expect("alice is a seat");
    let mut reading = host.join(key(BOB)).await.expect("bob is a seat");

    // Far longer than the backlog is deep: at 20 ms and a backlog of 8, this is a channel that has
    // been full for many sweeps.
    tokio::time::sleep(Duration::from_millis(400)).await;

    let status = tokio::time::timeout(PATIENCE, host.status())
        .await
        .expect("the world answers while a client is not reading")
        .expect("the world is running");
    assert_eq!(status.clients, 2);
    assert!(
        status.observations_dropped > 0,
        "the world dropped frames for the client that stopped reading rather than waiting for it,          and says so: {status:?}"
    );
    assert_eq!(status.faults, 0, "and nothing about it was an error");

    let observation = tokio::time::timeout(PATIENCE, reading.observations().recv())
        .await
        .expect("the reading client still receives observations")
        .expect("the world is running")
        .observation;
    assert_eq!(observation.observer(), reading.observer());

    let dave = *support::perceived_ids(&observation)
        .iter()
        .find(|entity| **entity != reading.observer())
        .expect("dave is in the street too");
    let answered = tokio::time::timeout(
        PATIENCE,
        host.submit(
            reading.observer(),
            speak(reading.observer(), dave, "still here"),
        ),
    )
    .await
    .expect("dispatch is not waiting on the client that stopped reading")
    .expect("the request is dispatched");
    assert!(matches!(answered.result(), ActionResult::Accepted { .. }));

    // The unread subscription is still a subscription: nothing about it failed, it simply missed
    // frames.
    drop(unread);
}

/// `INV-6` with numbers: the server allocates every identity, and a caller supplies none.
#[tokio::test]
async fn the_server_allocates_the_identity_of_every_request() {
    let host = host().await;
    let mut alice = host.join(key(ALICE)).await.expect("alice is a seat");
    let observation = tokio::time::timeout(PATIENCE, alice.observations().recv())
        .await
        .expect("an observation arrives")
        .expect("the world is running")
        .observation;
    let carol = *support::perceived_ids(&observation)
        .iter()
        .find(|entity| **entity != alice.observer())
        .expect("carol is in the cafe with alice");

    let first = host
        .submit(alice.observer(), speak(alice.observer(), carol, "hello"))
        .await
        .expect("dispatched");
    let second = host
        .submit(alice.observer(), whisper(alice.observer(), "to myself"))
        .await
        .expect("dispatched");

    assert_eq!(
        (first.action_id().raw(), second.action_id().raw()),
        (1, 2),
        "the world's own allocator, from one upwards, with nothing a caller could have chosen"
    );
    assert_ne!(first.action_id(), second.action_id());
}

/// Two refusals that belong to the host rather than to the wire, so they hold for every caller —
/// a controller written in Rust included, not only a WebSocket client.
#[tokio::test]
async fn a_seat_that_does_not_exist_and_an_actor_that_is_not_the_observer_are_both_refused() {
    let host = host().await;

    let refused = host
        .join(key(CAROL))
        .await
        .expect_err("carol is a person in this world, but not a seat");
    assert_eq!(refused.code(), RefusalCode::UnknownSeat);

    let alice = host.join(key(ALICE)).await.expect("alice is a seat");
    let bob = host.join(key(BOB)).await.expect("bob is a seat");

    let impersonation = host
        .submit(
            bob.observer(),
            speak(alice.observer(), bob.observer(), "I am Alice"),
        )
        .await
        .expect_err("a client may only act as its own observer");
    assert_eq!(impersonation.code(), RefusalCode::ActorNotObserver);
}
