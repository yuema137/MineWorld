//! Golden frames: one example of every frame kind, kept as reviewed JSON in `tests/frames/`
//! (step-12 §15.3 SD-A12; S10's R-S11-7).
//!
//! The files are the oracle, not the code: each was read and checked against `PROTOCOL.md` when it
//! was committed, and a change to a frame's shape fails here, naming the file, until somebody edits
//! that file by hand — which is the reviewable act. Clients in other languages (the Python cognition
//! SDK, the GDScript module) test their codecs against the same files, so the Rust types stay the one
//! source of truth without a parallel schema (`docs/DECISIONS.md` `ARC-41`).
//!
//! A server frame is checked in both directions: the example serializes to the file's JSON, and the
//! file decodes back to the example. A client frame, which the server only ever reads, is checked by
//! decoding the file through `ClientFrame::decode` — the server's own entry point — and comparing it
//! with the example. Nothing here writes a file.

use std::path::PathBuf;

use mineworld_contracts::{
    ActionId, ActionResult, ActionTypeId, Causation, EntityId, EntityKey, EntityType, Event,
    EventEnvelope, EventId, EventRecord, EventSchemaVersion, EventTypeId, Observation,
    PerceivedEvent, PlaceId, Provenance, SystemId, Visibility, WorldTime,
};
use mineworld_server::{
    ClientFrame, ClosingReason, CorrelationToken, Nickname, OfferedInvite, PayloadForm,
    PerceivedJoin, RefusalCode, ResumeSecret, ServerFrame, SessionId, SystemSummary, TookOver,
    WorldInstanceId, WorldRevision, WorldSummary, wire_fact,
};
use serde_json::Value;

fn golden(kind: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/frames")
        .join(format!("{kind}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is missing: {error}", path.display()));
    (path, text)
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal key")
}

fn token(value: &str) -> CorrelationToken {
    CorrelationToken::new(value).expect("a legal token")
}

/// The server frame serializes to the file, and the file decodes to the server frame.
fn server_frame_matches(kind: &str, example: &ServerFrame) {
    let (path, text) = golden(kind);
    let file: Value = serde_json::from_str(&text).expect("the golden file is JSON");
    let written = serde_json::to_value(example).expect("the example serializes");
    assert_eq!(
        written,
        file,
        "{} no longer matches the frame the server writes. If the change is intended, update the file \
         by hand; the server now writes:\n{}",
        path.display(),
        serde_json::to_string_pretty(&written).expect("printable"),
    );
    let read: ServerFrame = serde_json::from_str(&text).expect("the golden file decodes");
    assert_eq!(
        &read,
        example,
        "{} decodes to another frame",
        path.display()
    );
}

/// The file is read by the server's own decoder as the example.
fn client_frame_matches(kind: &str, example: &ClientFrame) {
    let (path, text) = golden(kind);
    let read = ClientFrame::decode(&text)
        .unwrap_or_else(|refusal| panic!("{} is refused: {refusal:?}", path.display()));
    assert_eq!(
        &read,
        example,
        "{} decodes to another frame",
        path.display()
    );
}

fn world() -> WorldSummary {
    WorldSummary {
        protocol: 2,
        instance: WorldInstanceId::from_raw(0x1a2b_3c4d_5e6f_7081_9293_a4b5_c6d7_e8f9),
        at: WorldTime::from_seconds(4112),
        time_scale: 1,
        entities: 41,
        systems: vec![SystemSummary {
            system: SystemId::from_static("conversation"),
            enabled: true,
            provides: vec![ActionTypeId::from_static("talk")],
            states: vec![
                EventTypeId::from_static("spoke"),
                EventTypeId::from_static("conversation-started"),
            ],
        }],
        seats: vec![key("visitor"), key("wanderer"), key("alice")],
        clients: 2,
        observations_dropped: 0,
        events_dropped: 0,
        faults: 0,
        revision: Some(WorldRevision::from_raw(7)),
    }
}

#[test]
fn join() {
    client_frame_matches(
        "join",
        &ClientFrame::Join {
            protocol: 2,
            invite: OfferedInvite::new("3f9c0a1b2c3d4e5f60718293a4b5c6d7"),
            nickname: "Yue".to_owned(),
            seat: key("visitor"),
            resume: None,
            take_over: false,
            perceived: Some(PerceivedJoin {
                since: Some(EventId::from_raw(1873)),
            }),
        },
    );
}

/// A line said in a place, as a client receives it: the contract's envelope with the owning pack's
/// JSON payload (`PROTOCOL.md` §5.2). Built through the server's own `wire_fact`, so the golden file
/// pins the shape the server writes.
fn spoke() -> PerceivedEvent<Value> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Spoke;
    impl Event for Spoke {
        const EVENT_TYPE: EventTypeId = EventTypeId::from_static("spoke");
        const OWNER: SystemId = SystemId::from_static("conversation");
        const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
    }
    let cafe = PlaceId::new(EntityId::from_raw(3), EntityType::Place).expect("a place");
    let fact = EventEnvelope::new(
        EventId::from_raw(1890),
        WorldTime::from_seconds(4100),
        EventRecord::new::<Spoke>(br#"{"utterance":"hello"}"#.to_vec()),
        Causation::Action(ActionId::from_raw(41)),
        Visibility::Place(cafe),
        Provenance::new(SystemId::from_static("conversation"))
            .from_controller_decision(ActionId::from_raw(41)),
    )
    .about(vec![EntityId::from_raw(5)])
    .with_participants(vec![EntityId::from_raw(5), EntityId::from_raw(7)])
    .at_place(cafe);
    let (event, form) = wire_fact(&fact).expect("renders");
    assert_eq!(form, PayloadForm::Json);
    event
}

#[test]
fn perceived() {
    server_frame_matches(
        "perceived",
        &ServerFrame::Perceived {
            through: EventId::from_raw(1907),
            events: vec![spoke()],
        },
    );
}

#[test]
fn refused_cursor_unavailable() {
    server_frame_matches(
        "refused-cursor_unavailable",
        &ServerFrame::Refused {
            token: None,
            code: RefusalCode::CursorUnavailable,
            detail: Some("the cursor is newer than any fact this world has recorded".to_owned()),
        },
    );
}

#[test]
fn refused_lagged() {
    server_frame_matches(
        "refused-lagged",
        &ServerFrame::Refused {
            token: None,
            code: RefusalCode::Lagged,
            detail: Some(
                "the perceived stream fell too far behind; rejoin with resume and your cursor"
                    .to_owned(),
            ),
        },
    );
}

#[test]
fn closing_lagged() {
    server_frame_matches(
        "closing-lagged",
        &ServerFrame::Closing {
            reason: ClosingReason::Lagged,
            detail: None,
        },
    );
}

#[test]
fn submit() {
    let (_, text) = golden("submit");
    let ClientFrame::Submit {
        token: sent,
        request,
    } = ClientFrame::decode(&text).expect("submit.json decodes")
    else {
        panic!("submit.json is a submit frame");
    };
    // `ActionRequest<WirePayload>` has no public constructor from parts, so the example is checked
    // field by field rather than by equality with a built value.
    assert_eq!(sent, token("c1"));
    assert_eq!(request.actor(), EntityId::from_raw(101));
    assert_eq!(request.action_type().as_str(), "talk");
    assert_eq!(
        request.target(),
        Some(EntityId::from_raw(9_007_199_254_740_995))
    );
    let payload: Value =
        serde_json::from_slice(request.payload().payload().as_bytes()).expect("JSON bytes");
    assert_eq!(payload, serde_json::json!({ "utterance": "hello" }));
}

#[test]
fn leave() {
    client_frame_matches("leave", &ClientFrame::Leave {});
}

#[test]
fn welcome() {
    server_frame_matches(
        "welcome",
        &ServerFrame::Welcome {
            protocol: 2,
            seat: key("visitor"),
            observer: EntityId::from_raw(101),
            nickname: Nickname::new("Yue").expect("a legal nickname"),
            session: SessionId::new(7),
            resume: Some(ResumeSecret::from(
                "5f0c2a9e8d7b6c5a4f3e2d1c0b9a8f7e".to_owned(),
            )),
            hold_seconds: 30,
            took_over: TookOver::Hosted,
            world: world(),
        },
    );
}

#[test]
fn observation() {
    server_frame_matches(
        "observation",
        &ServerFrame::Observation {
            seq: 1,
            revision: Some(WorldRevision::from_raw(7)),
            acted_through: Some(ActionId::from_raw(41)),
            observation: Observation::new(EntityId::from_raw(101), WorldTime::from_seconds(4112))
                .with_events(vec![spoke()]),
        },
    );
}

#[test]
fn result() {
    server_frame_matches(
        "result",
        &ServerFrame::Result {
            token: token("c1"),
            action_id: ActionId::from_raw(1),
            result: ActionResult::Accepted {
                events: vec![EventId::from_raw(1)],
            },
        },
    );
}

#[test]
fn refused() {
    server_frame_matches(
        "refused",
        &ServerFrame::Refused {
            token: Some(token("c1")),
            code: RefusalCode::ActorNotObserver,
            detail: Some("a connection acts only as its own observer".to_owned()),
        },
    );
}

#[test]
fn closing() {
    server_frame_matches(
        "closing",
        &ServerFrame::Closing {
            reason: ClosingReason::TakenOver,
            detail: None,
        },
    );
}
