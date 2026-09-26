//! Contract tests for the request half of the pipeline.
//!
//! The action types here are named the way the specification's examples name them — `talk`,
//! `give_item` — precisely to demonstrate that this crate does not know what they mean. Nothing in
//! the contract layer branches on either name; to it they are opaque slugs a system declared, and
//! that is the whole content of `INV-12`.
//!
//! Encoding is `serde_json` because a test needs *some* encoding. The contract layer does not
//! choose one, so these tests stand in for the transport that eventually will. Every expected
//! value is written by hand.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, ContractError,
    EntityId, EntityType, EventId, LocalPosition, Location, Millidegrees, Millimetres, Orientation,
    PlaceId, Rejection, RejectionCode, SystemId, WorldTime,
};
use serde::{Deserialize, Serialize};

/// A request one stub system provides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Talk {
    opening_line: String,
}

impl Action for Talk {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("talk");
    const OWNER: SystemId = SystemId::from_static("conversation-stub");
}

/// A request a *different* stub system provides, so that ownership is visibly a property of the
/// action type rather than of the crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GiveItem {
    item: EntityId,
}

impl Action for GiveItem {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("give_item");
    const OWNER: SystemId = SystemId::from_static("inventory-stub");
}

/// A request whose owning system is *not* installed in the world these tests describe: the case
/// `INV-10` is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Shoot {
    at: EntityId,
}

impl Action for Shoot {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("shoot");
    const OWNER: SystemId = SystemId::from_static("combat-stub");
}

fn encode<A: Action>(action: &A) -> Vec<u8> {
    serde_json::to_vec(action).expect("an action payload must be encodable")
}

fn talk_request() -> ActionRecord {
    ActionRecord::new::<Talk>(encode(&Talk {
        opening_line: "the contract layer never reads this".to_owned(),
    }))
}

/// Erasing a request and reading it back loses nothing, and the record carries the label the
/// action type gave it rather than one the caller chose. A record is handed only to the action
/// type it was written for: without that check a routing mistake would decode one system's bytes
/// as another system's request, and with `serde` defaults it could even succeed, producing a
/// plausible request nobody made.
#[test]
fn a_request_survives_erasure_and_is_readable_only_as_its_own_action_type() {
    let written = Talk {
        opening_line: "hello".to_owned(),
    };
    let record = ActionRecord::new::<Talk>(encode(&written));

    assert_eq!(record.action_type(), &ActionTypeId::new("talk").unwrap());
    let payload = record
        .payload_for::<Talk>()
        .expect("a record written from Talk is readable as Talk");
    assert_eq!(serde_json::from_slice::<Talk>(payload).unwrap(), written);

    assert_eq!(
        record.payload_for::<GiveItem>().unwrap_err(),
        ContractError::ActionTypeMismatch {
            expected: ActionTypeId::new("give_item").unwrap(),
            actual: ActionTypeId::new("talk").unwrap(),
        }
    );
}

/// The five answers `ENGINEERING_RULES.md` §8 requires a client to be able to show are kernel
/// vocabulary, and a system's own reason travels beside them without the kernel learning it. All
/// of them must survive the transport, because a rejection a client cannot read is a rejection a
/// player cannot be shown.
#[test]
fn every_rejection_a_client_must_render_survives_the_transport() {
    let rejections = [
        Rejection::Busy,
        Rejection::TooFarAway,
        Rejection::PermissionDenied,
        Rejection::NoSupportedInteraction,
        Rejection::TargetUnavailable,
        Rejection::Unavailable,
        Rejection::PreconditionFailed,
        Rejection::System {
            code: RejectionCode::new("closed-for-the-night").unwrap(),
            detail: Some("stub system reason".to_owned()),
        },
        Rejection::System {
            code: RejectionCode::from_static("no-table-free"),
            detail: None,
        },
    ];

    for rejection in &rejections {
        let text = serde_json::to_string(rejection).unwrap();
        assert_eq!(
            &serde_json::from_str::<Rejection>(&text).unwrap(),
            rejection,
            "{text} must round-trip"
        );
    }

    // The stored forms, pinned: these travel to a client that is not written in Rust, so changing
    // one is a protocol change rather than a refactor.
    assert_eq!(
        serde_json::to_string(&Rejection::TooFarAway).unwrap(),
        r#""too_far_away""#
    );
    assert_eq!(
        serde_json::to_string(&Rejection::System {
            code: RejectionCode::from_static("closed-for-the-night"),
            detail: None,
        })
        .unwrap(),
        r#"{"system":{"code":"closed-for-the-night","detail":null}}"#
    );
}

/// `INV-10` is a statement about a world, and this is the shape it takes in the contract: an
/// intent for an action no enabled system provides is perfectly well formed — the kernel must be
/// able to receive it, not to refuse to represent it — and the answer to it is `Unavailable`.
///
/// The two answers a client could confuse must also be distinguishable on the wire: "this action
/// does not exist in this world" and "the action exists and your attempt was refused without
/// further classification" are different facts, and a client that showed the first as the second
/// would tell a player to try again in a world where the capability is simply absent.
#[test]
fn an_intent_for_an_action_no_system_provides_is_representable_and_answered_unavailable() {
    let intent = ActionIntent::new(
        ActionId::from_raw(7),
        EntityId::from_raw(1),
        ActionRecord::new::<Shoot>(encode(&Shoot {
            at: EntityId::from_raw(2),
        })),
        WorldTime::from_seconds(120),
    )
    .with_target(EntityId::from_raw(2));

    assert_eq!(intent.action_type(), &ActionTypeId::new("shoot").unwrap());

    let answer = ActionResult::Unavailable;
    assert_eq!(serde_json::to_string(&answer).unwrap(), r#""unavailable""#);
    assert_eq!(
        serde_json::to_string(&ActionResult::Rejected(Rejection::Unavailable)).unwrap(),
        r#"{"rejected":"unavailable"}"#
    );

    // An accepted answer names the events the request caused; that list is the only link from a
    // request to the facts it produced.
    let accepted = ActionResult::Accepted {
        events: vec![EventId::from_raw(9001), EventId::from_raw(9002)],
    };
    assert_eq!(
        serde_json::to_string(&accepted).unwrap(),
        r#"{"accepted":{"events":[9001,9002]}}"#
    );
    assert_eq!(
        serde_json::from_str::<ActionResult>(r#"{"accepted":{"events":[9001,9002]}}"#).unwrap(),
        accepted
    );
}

/// An intent names an action type in its envelope, which is what dispatch routes by, and carries a
/// payload record written from an action type, which is what the owning system decodes. If those
/// two could disagree, one of them would act on a request nobody made — so neither construction
/// nor deserialization may produce a disagreeing pair.
#[test]
fn an_intents_envelope_cannot_disagree_with_its_payload() {
    let intent = ActionIntent::new(
        ActionId::from_raw(3),
        EntityId::from_raw(1),
        talk_request(),
        WorldTime::EPOCH,
    );
    // The envelope's type is read off the record, so in code there is no second value to get
    // wrong.
    assert_eq!(intent.action_type(), intent.payload().action_type());

    let agreeing = r#"{"action_id":3,"actor":1,"action_type":"talk","target":null,"payload":{"action_type":"talk","payload":[]},"issued_at":0,"actor_location":null}"#;
    assert!(serde_json::from_str::<ActionIntent>(agreeing).is_ok());

    let disagreeing = r#"{"action_id":3,"actor":1,"action_type":"give_item","target":null,"payload":{"action_type":"talk","payload":[]},"issued_at":0,"actor_location":null}"#;
    let error = serde_json::from_str::<ActionIntent>(disagreeing)
        .expect_err("an intent whose envelope and payload name different action types is refused");
    assert_eq!(
        error.to_string(),
        ContractError::ActionIntentPayloadMismatch {
            action_type: ActionTypeId::new("give_item").unwrap(),
            payload_action_type: ActionTypeId::new("talk").unwrap(),
        }
        .to_string()
    );
}

/// The stored shape of an intent, asserted exactly: this is what a client sends and what a
/// persistence layer or a protocol reads, so changing it is a protocol change.
///
/// The intent carried here is the one an embodied 3D client produces — a target obtained from a
/// camera ray, and the position and heading the player actually walked to — and a 2D client's
/// differs from it only by leaving `actor_location` out. Neither client evaluates anything: both
/// report, and the server answers.
#[test]
fn an_intent_is_stored_as_its_documented_shape() {
    let counter = PlaceId::new(EntityId::from_raw(7), EntityType::Place).unwrap();
    let intent = ActionIntent::new(
        ActionId::from_raw(3),
        EntityId::from_raw(41),
        ActionRecord::new::<GiveItem>(r#"{"item":18517}"#.to_owned()),
        WorldTime::from_seconds(64_800),
    )
    .with_target(EntityId::from_raw(42))
    .from_location(
        Location::in_place(counter)
            .with_local(LocalPosition::new(
                Millimetres::new(1_200),
                Millimetres::new(-350),
                Millimetres::ZERO,
            ))
            .with_facing(Orientation::facing(Millidegrees::new(90_000))),
    );

    let text = r#"{"action_id":3,"actor":41,"action_type":"give_item","target":42,"payload":{"action_type":"give_item","payload":"{\"item\":18517}"},"issued_at":64800,"actor_location":{"place":{"entity":7,"entity_type":"place"},"local":{"x":1200,"y":-350,"z":0},"facing":{"yaw":90000,"pitch":null}}}"#;
    assert_eq!(serde_json::to_string(&intent).unwrap(), text);
    assert_eq!(
        serde_json::from_str::<ActionIntent<String>>(text).unwrap(),
        intent
    );

    // The same request from a client that models no position at all: one field fewer, and no other
    // difference anywhere in the contract.
    let flat = ActionIntent::new(
        ActionId::from_raw(3),
        EntityId::from_raw(41),
        ActionRecord::new::<GiveItem>(r#"{"item":18517}"#.to_owned()),
        WorldTime::from_seconds(64_800),
    )
    .with_target(EntityId::from_raw(42));
    assert_eq!(flat.actor_location(), None);
    assert_eq!(flat.action_type(), intent.action_type());
    assert_eq!(flat.target(), intent.target());
}
