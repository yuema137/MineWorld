//! A seated connection is on the admin surface before its `welcome` leaves the server
//! (step-12 §18.13 F-SD1).
//!
//! The welcome is a client's proof that it is seated, so `GET /admin/sessions` must list the session
//! from the moment the client could have read it. The sink below stands in for the socket and looks
//! at the registry at the instant each frame is handed to it — the earliest a client could hold it —
//! so the order is checked directly rather than by racing a real connection.

use std::convert::Infallible;
use std::sync::{Arc, Mutex, PoisonError};

use axum::extract::ws::Message;
use futures_util::sink;
use mineworld_contracts::{EntityId, EntityKey, WorldTime};

use super::{Greeting, greet};
use crate::admin::registry::{Joined, Registry};
use crate::admission::Nickname;
use crate::protocol::{
    PROTOCOL_VERSION, ServerFrame, SessionId, TookOver, WorldInstanceId, WorldSummary,
};

const SESSION: SessionId = SessionId::new(7);

fn greeting() -> Greeting {
    let seat = EntityKey::new("visitor").expect("a legal key");
    let observer = EntityId::from_raw(101);
    let nickname = Nickname::new("Yue").expect("a legal nickname");
    let at = WorldTime::from_seconds(32_400);
    Greeting {
        session: SESSION,
        joined: Joined {
            nickname: nickname.clone(),
            seat: seat.clone(),
            observer,
            connected_at: 1_791_441_600,
        },
        welcome: ServerFrame::Welcome {
            protocol: PROTOCOL_VERSION,
            seat: seat.clone(),
            observer,
            nickname,
            session: SESSION,
            resume: None,
            hold_seconds: 0,
            took_over: TookOver::None,
            world: WorldSummary {
                protocol: PROTOCOL_VERSION,
                instance: WorldInstanceId::from_raw(1),
                at,
                time_scale: 1,
                paused: false,
                entities: 1,
                systems: Vec::new(),
                seats: vec![seat],
                clients: 1,
                observations_dropped: 0,
                events_dropped: 0,
                faults: 0,
                revision: None,
            },
        },
        clock: ServerFrame::Clock {
            at,
            time_scale: 1,
            paused: false,
        },
        backfill: Vec::new(),
    }
}

/// Each frame's tag, beside the sessions the registry listed when the frame was sent.
type Seen = Arc<Mutex<Vec<(String, Vec<SessionId>)>>>;

#[tokio::test]
async fn a_session_is_listed_before_its_welcome_is_sent() {
    let registry = Arc::new(Registry::default());
    let seen: Seen = Arc::default();
    let mut socket = Box::pin(sink::unfold(
        (Arc::clone(&registry), Arc::clone(&seen)),
        |(registry, seen), message: Message| async move {
            let Message::Text(text) = message else {
                panic!("the server sends text frames: {message:?}");
            };
            let frame: serde_json::Value = serde_json::from_str(&text).expect("a JSON frame");
            let listed = registry
                .listed()
                .iter()
                .map(|entry| entry.session)
                .collect();
            seen.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push((frame["t"].as_str().expect("a tag").to_owned(), listed));
            Ok::<_, Infallible>((registry, seen))
        },
    ));

    let listed = greet(&mut socket, &registry, greeting()).await;

    assert!(listed.is_some(), "both frames were sent");
    assert_eq!(
        *seen.lock().unwrap_or_else(PoisonError::into_inner),
        vec![
            ("welcome".to_owned(), vec![SESSION]),
            ("clock".to_owned(), vec![SESSION]),
        ],
        "a client holding its welcome finds itself on /admin/sessions"
    );
    drop(listed);
    assert!(
        registry.listed().is_empty(),
        "listed until the session ends"
    );
}

#[tokio::test]
async fn a_connection_that_goes_during_its_welcome_is_not_left_listed() {
    let registry = Arc::new(Registry::default());
    let mut socket = Box::pin(sink::unfold((), |(), _: Message| async {
        Err::<(), _>("the connection is gone")
    }));

    let listed = greet(&mut socket, &registry, greeting()).await;

    assert!(listed.is_none(), "the welcome could not be sent");
    assert!(
        registry.listed().is_empty(),
        "a session that was never seated on the wire is not listed"
    );
}
