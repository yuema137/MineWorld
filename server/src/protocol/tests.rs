//! What the frames pin: the closed vocabulary, the encoding of identity, and the conversion onto
//! the payload type the kernel dispatches.
//!
//! Every expected JSON string here is written by hand. None of it is produced by the code under
//! test, so a change in the contract's encoding or in a frame's shape shows up as a failure rather
//! than as agreement with itself.

use mineworld_contracts::{
    Action, ActionId, ActionRecord, ActionRequest, ActionResult, ActionTypeId, EntityId, EntityKey,
    Rejection, SystemId,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{
    ClientFrame, CorrelationToken, MAX_TOKEN_LENGTH, PROTOCOL_VERSION, RefusalCode, ServerFrame,
    WirePayload, WorldInstanceId, into_kernel_request,
};

/// An action belonging to no real system: the tests need a payload the contract will label, and
/// this crate must not invent domain vocabulary (`INV-12`).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Example {
    topic: String,
}

impl Action for Example {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("example-action");
    const OWNER: SystemId = SystemId::from_static("example-system");
}

/// An `EntityId` above 2^53, where a double stops being able to count: `9007199254740995` and
/// `9007199254740997` are the same double, which is the corruption the string encoding prevents.
const BIG: u64 = 9_007_199_254_740_995;

fn token(value: &str) -> CorrelationToken {
    CorrelationToken::new(value).expect("a legal token")
}

// -------------------------------------------------------------------------------------------
// The closed vocabulary: what a client may say, and what happens to everything else.
// -------------------------------------------------------------------------------------------

/// `NETWORKING.md` §2's example, as a frame. The point is not that a check rejected it — it is that
/// there is no frame of this kind at all, so the assertion never reaches anything that could act on
/// it.
#[test]
fn a_frame_that_asserts_state_is_refused_as_a_kind_this_protocol_does_not_have() {
    let asserted = r#"{"t":"set_state","entity":"101","money":5000}"#;

    let refusal = ClientFrame::decode(asserted).expect_err("a state assertion is not a frame");

    assert_eq!(refusal.code(), RefusalCode::UnknownFrame);
}

/// The same assertion without even a frame kind: still refused, and distinguishably so.
#[test]
fn a_frame_with_no_kind_is_malformed_rather_than_unknown() {
    let refusal = ClientFrame::decode(r#"{"entity":"101","money":5000}"#)
        .expect_err("a frame states its kind");

    assert_eq!(refusal.code(), RefusalCode::MalformedFrame);
}

#[test]
fn text_that_is_not_json_is_malformed() {
    let refusal = ClientFrame::decode("my money is now 5000").expect_err("not a frame");

    assert_eq!(refusal.code(), RefusalCode::MalformedFrame);
}

/// The two frames that do exist, decoded from exactly what a client sends.
#[test]
fn the_two_client_frames_decode() {
    let joined = ClientFrame::decode(r#"{"t":"join","seat":"player"}"#).expect("a join frame");
    assert_eq!(
        joined,
        ClientFrame::Join {
            seat: EntityKey::new("player").expect("a legal key"),
        }
    );

    let submitted = ClientFrame::decode(
        r#"{"t":"submit","token":"c1","request":{
             "actor":"101",
             "action_type":"example-action",
             "target":"9007199254740995",
             "payload":{"action_type":"example-action","payload":{"topic":"greeting"}},
             "actor_location":null}}"#,
    )
    .expect("a submit frame");

    let ClientFrame::Submit {
        token: sent,
        request,
    } = submitted
    else {
        panic!("a submit frame decodes as a submission");
    };
    assert_eq!(sent, token("c1"));
    assert_eq!(request.actor(), EntityId::from_raw(101));
    assert_eq!(
        request.target(),
        Some(EntityId::from_raw(BIG)),
        "the 64-bit target survived the client's decimal string"
    );
}

/// A frame whose envelope and payload disagree about the action type is refused by the contract, not
/// by this crate — which is the check `FINDINGS.md` F8.1 says every hand-building client needs.
#[test]
fn a_submission_whose_halves_disagree_about_the_action_type_is_malformed() {
    let refusal = ClientFrame::decode(
        r#"{"t":"submit","token":"c1","request":{
             "actor":"101",
             "action_type":"example-action",
             "payload":{"action_type":"another-action","payload":{}},
             "target":null,"actor_location":null}}"#,
    )
    .expect_err("the contract refuses a request that disagrees with itself");

    assert_eq!(refusal.code(), RefusalCode::MalformedFrame);
}

/// A refusal carries the token of the frame that caused it where one was recoverable, so a client
/// can pair even a refusal with what it sent.
#[test]
fn a_refusal_recovers_the_token_of_a_frame_that_failed() {
    let refusal = ClientFrame::decode(r#"{"t":"submit","token":"c7","request":{"actor":"101"}}"#)
        .expect_err("an incomplete request");

    let ServerFrame::Refused { token, code, .. } = refusal.into_frame() else {
        panic!("a refusal becomes a refused frame");
    };
    assert_eq!(code, RefusalCode::MalformedFrame);
    assert_eq!(
        token,
        Some(super::CorrelationToken::new("c7").expect("a legal token"))
    );
}

// -------------------------------------------------------------------------------------------
// Identity on the wire. The transport writes none of this: the contract does.
// -------------------------------------------------------------------------------------------

/// The load-bearing one. Every identity in a server frame is a decimal string, because the contract
/// encodes it that way and this module hands it whole contract types rather than mirroring their
/// shapes (`DEP-3`, `FINDINGS.md` F2).
#[test]
fn every_identity_in_a_server_frame_is_a_decimal_string() {
    let result = ServerFrame::Result {
        token: token("c1"),
        action_id: ActionId::from_raw(BIG),
        result: ActionResult::Accepted {
            events: vec![
                mineworld_contracts::EventId::from_raw(BIG + 2),
                mineworld_contracts::EventId::from_raw(7),
            ],
        },
    };

    let encoded = serde_json::to_value(&result).expect("a result frame serializes");

    assert_eq!(
        encoded,
        json!({
            "t": "result",
            "token": "c1",
            "action_id": "9007199254740995",
            "result": { "accepted": { "events": ["9007199254740997", "7"] } },
        }),
        "an id reaches a client as a decimal string, in every position it appears"
    );
}

#[test]
fn a_welcome_names_the_observer_as_a_decimal_string() {
    let welcome = ServerFrame::Welcome {
        protocol: PROTOCOL_VERSION,
        seat: EntityKey::new("player").expect("a legal key"),
        observer: EntityId::from_raw(BIG),
        world: super::WorldSummary {
            protocol: PROTOCOL_VERSION,
            instance: super::WorldInstanceId::from_raw(0x0123_4567_89ab_cdef),
            at: mineworld_contracts::WorldTime::from_seconds(32_400),
            entities: 4,
            systems: Vec::new(),
            seats: vec![EntityKey::new("player").expect("a legal key")],
            clients: 1,
            observations_dropped: 0,
            deferrals_unscheduled: 0,
            faults: 0,
            revision: Some(mineworld_persistence::WorldRevision::from_raw(7)),
        },
    };

    let encoded = serde_json::to_value(&welcome).expect("a welcome serializes");

    assert_eq!(encoded["observer"], json!("9007199254740995"));
    assert_eq!(encoded["world"]["at"], json!(32_400));
    assert_eq!(
        encoded["world"]["instance"],
        json!("00000000000000000123456789abcdef"),
        "a world instance reaches a client as a string, because 128 bits are not a double",
    );
    assert_eq!(
        encoded["world"]["revision"],
        json!(7),
        "the persisted revision is a plain integer, like every non-identity number (PROTOCOL.md §7)",
    );
}

/// `AC-15`'s first evidence line is *same world instance*, and it is only evidence if two worlds
/// are told apart. Two allocations differ; one instance says the same thing every time it is asked.
#[test]
fn two_running_worlds_have_two_identities_and_each_keeps_its_own() {
    let first = WorldInstanceId::allocate();
    let second = WorldInstanceId::allocate();

    assert_ne!(first, second, "two worlds are never one world");
    assert_eq!(
        first.to_string(),
        first.to_string(),
        "and an instance's identity does not change while it is being asked",
    );
    assert_eq!(first.to_string().len(), 32, "128 bits of hexadecimal");

    let round_tripped: WorldInstanceId =
        serde_json::from_value(serde_json::to_value(first).expect("an instance serializes"))
            .expect("and reads back");
    assert_eq!(round_tripped, first);
}

/// A rejection is a value a client renders, and it arrives inside a result rather than as a refusal.
#[test]
fn a_rejection_is_carried_inside_the_answer_and_not_as_a_refusal() {
    let frame = ServerFrame::Result {
        token: token("c1"),
        action_id: ActionId::from_raw(3),
        result: ActionResult::Rejected(Rejection::TooFarAway),
    };

    let encoded = serde_json::to_value(&frame).expect("a result frame serializes");

    assert_eq!(encoded["t"], json!("result"));
    assert_eq!(encoded["result"], json!({ "rejected": "too_far_away" }));
}

/// A refused frame says why in a closed vocabulary and leaves out what it has nothing to say about.
#[test]
fn a_refusal_omits_what_it_has_nothing_to_say() {
    let encoded = serde_json::to_value(super::Refusal::new(RefusalCode::NotJoined).into_frame())
        .expect("a refusal serializes");

    assert_eq!(encoded, json!({ "t": "refused", "code": "not_joined" }));
}

// -------------------------------------------------------------------------------------------
// The payload, and the conversion onto the type the kernel dispatches.
// -------------------------------------------------------------------------------------------

/// The payload a client wrote reaches the owning system as the JSON bytes of that payload, and the
/// contract's label still refuses to hand it to the wrong action type.
#[test]
fn a_submitted_payload_reaches_the_kernel_as_the_json_the_client_sent() {
    let ClientFrame::Submit { request, .. } = ClientFrame::decode(
        r#"{"t":"submit","token":"c1","request":{
             "actor":"101",
             "action_type":"example-action",
             "payload":{"action_type":"example-action","payload":{"topic":"greeting"}},
             "target":null,"actor_location":null}}"#,
    )
    .expect("a submit frame") else {
        panic!("a submit frame decodes as a submission");
    };

    let kernel = into_kernel_request(&request).expect("the request re-parameterizes");

    let bytes = kernel
        .payload()
        .payload_for::<Example>()
        .expect("the record was written for this action");
    assert_eq!(
        serde_json::from_slice::<Example>(bytes).expect("the payload is the client's JSON"),
        Example {
            topic: "greeting".to_owned(),
        }
    );
    assert_eq!(kernel.actor(), EntityId::from_raw(101));
    assert_eq!(*kernel.action_type(), Example::ACTION_TYPE);
}

/// The conversion is the contract's own serialization in both directions, so a request that a Rust
/// caller built is unchanged by a round trip through it.
#[test]
fn re_parameterizing_a_request_changes_nothing_but_the_payload_type() {
    let payload = WirePayload::from_value(&json!({ "topic": "greeting" }))
        .expect("a value becomes canonical bytes");
    let request = ActionRequest::new(
        EntityId::from_raw(101),
        ActionRecord::new::<Example>(payload),
    )
    .with_target(EntityId::from_raw(BIG));

    let kernel = into_kernel_request(&request).expect("the request re-parameterizes");

    assert_eq!(kernel.actor(), request.actor());
    assert_eq!(kernel.target(), request.target());
    assert_eq!(kernel.action_type(), request.action_type());
    assert_eq!(
        kernel.payload().payload(),
        &serde_json::to_vec(&json!({ "topic": "greeting" })).expect("bytes"),
        "the kernel receives the JSON text of the payload, which is how a system decodes it"
    );
}

/// `WirePayload` carries bytes and writes them as bytes, which is what makes the round trip above a
/// use of the contract's serde rather than a rewrite of its shapes.
#[test]
fn a_wire_payload_reads_json_and_writes_bytes() {
    let payload: WirePayload =
        serde_json::from_str(r#"{"topic":"greeting"}"#).expect("any JSON value");

    assert_eq!(payload.as_bytes(), br#"{"topic":"greeting"}"#);
    assert_eq!(
        serde_json::to_value(&payload).expect("a payload serializes"),
        serde_json::to_value(br#"{"topic":"greeting"}"#.to_vec()).expect("bytes serialize"),
    );
}

// -------------------------------------------------------------------------------------------
// The correlation token, which is the client's and is never interpreted.
// -------------------------------------------------------------------------------------------

#[test]
fn a_token_is_bounded_and_printable() {
    assert!(
        CorrelationToken::new("").is_err(),
        "an empty token is not one"
    );
    assert!(
        CorrelationToken::new("a".repeat(MAX_TOKEN_LENGTH)).is_ok(),
        "the limit itself is legal"
    );
    assert!(
        CorrelationToken::new("a".repeat(MAX_TOKEN_LENGTH + 1)).is_err(),
        "a client cannot make the server hold an unbounded string"
    );
    assert!(
        CorrelationToken::new("c1\n").is_err(),
        "a control character in a token is refused"
    );
}

/// Opaque: whatever a client chose comes back unchanged, and the server never parses it.
#[test]
fn a_token_survives_a_round_trip_unchanged() {
    let chosen = "2d-client:42/talk";
    let encoded = serde_json::to_value(token(chosen)).expect("a token serializes");

    assert_eq!(encoded, Value::String(chosen.to_owned()));
    assert_eq!(
        serde_json::from_value::<CorrelationToken>(encoded).expect("and reads back"),
        token(chosen)
    );
}
