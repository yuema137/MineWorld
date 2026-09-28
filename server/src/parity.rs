//! `AC-13`: what it means for two clients to ask the world the same thing, defined once.
//!
//! ```text
//! the semantic core   actor · action_type · target · payload
//! permitted to differ actor_location, and nothing else
//! ```
//!
//! `docs/MVP.md` §9 `AC-13` requires that a `Talk` initiated by clicking a person in the 2D client
//! and one initiated by walking up to them, looking at them and pressing a key in the 3D client have
//! an **identical semantic core** — and its 2026-09-26 correction requires the comparison to be
//! *"defined once, in the server, so the S12/S14 test cannot quietly compare a different set of
//! fields"*. This module is that definition. A test that wrote its own comparison could drop a field
//! and still pass; a test that calls [`semantic_core`] and [`differing_fields`] cannot.
//!
//! # Why one field may differ, and why it used to be three
//!
//! `spike/FINDINGS.md` F6 measured three: `action_id` and `issued_at`, which each client invented
//! because the contract made it, and `actor_location`, which is *designed* to differ. PR 04 removed
//! the first two from the type a client submits — an [`ActionRequest`] has no identity and no instant
//! — so only the designed difference is left:
//!
//! > a 3D client sends the position it walked to; a 2D client that models no position sends none
//! > (`contracts/src/action.rs`, `server/PROTOCOL.md` §6).
//!
//! Demanding byte equality would therefore force the two clients to converge on the one thing the
//! contract deliberately lets them disagree about.
//!
//! # Why the difference is reported rather than ignored
//!
//! [`differing_fields`] walks the **whole** request, field by field, and names what differs. A
//! comparison that simply skipped `actor_location` would also skip a field `ActionRequest` gains
//! later, silently. [`RequestField`] is a closed enum matched exhaustively here, so a new field on
//! the contract makes this module fail to compile rather than quietly stop comparing.

use mineworld_contracts::{ActionRequest, ActionTypeId, EntityId};

/// One field of an [`ActionRequest`], for saying which ones two requests disagree about.
///
/// A closed enum rather than a string: a test asserts `[RequestField::ActorLocation]`, and a typo
/// cannot turn that into an assertion about nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequestField {
    /// Who is asking. Part of the semantic core.
    Actor,
    /// What kind of request it is. Part of the semantic core.
    ActionType,
    /// What it is about. Part of the semantic core.
    Target,
    /// The owning system's own payload, as submitted. Part of the semantic core.
    Payload,
    /// Where the client reports the actor was — a report, not an assertion, and the one field two
    /// clients of different dimensions are expected to disagree about.
    ActorLocation,
}

impl RequestField {
    /// Whether this field belongs to the semantic core `AC-13` compares.
    pub const fn is_semantic_core(self) -> bool {
        match self {
            Self::Actor | Self::ActionType | Self::Target | Self::Payload => true,
            Self::ActorLocation => false,
        }
    }
}

/// What the world is being asked for, stripped of who reported being where.
///
/// Borrowed rather than owned: this is a comparison, not a record, and copying a payload to compare
/// it would invite somebody to store one.
#[derive(Debug, PartialEq, Eq)]
pub struct SemanticCore<'a, P> {
    /// Who is asking.
    pub actor: EntityId,
    /// What kind of request it is.
    pub action_type: &'a ActionTypeId,
    /// What it is about, if anything.
    pub target: Option<EntityId>,
    /// The payload the owning system will decode.
    pub payload: &'a P,
}

/// The four fields `AC-13` names, and no fifth.
///
/// Generic in the payload type so that the same definition compares what a client submitted
/// ([`WirePayload`](crate::WirePayload), the bytes of the JSON it wrote) and what was dispatched
/// (`Vec<u8>`). Two requests that differ only in how their payload was *carried* are not what
/// `AC-13` is about, so the comparison is between two requests of one representation.
pub fn semantic_core<P>(request: &ActionRequest<P>) -> SemanticCore<'_, P> {
    SemanticCore {
        actor: request.actor(),
        action_type: request.action_type(),
        target: request.target(),
        payload: request.payload().payload(),
    }
}

/// Every field these two requests disagree about, in field order.
///
/// Empty means the two requests are identical. `[RequestField::ActorLocation]` is what `AC-13`
/// expects of a 2D and a 3D client asking for the same thing: the semantic core agrees and the
/// reported position does not.
pub fn differing_fields<P: PartialEq>(
    left: &ActionRequest<P>,
    right: &ActionRequest<P>,
) -> Vec<RequestField> {
    let mut differing = Vec::new();
    for field in [
        RequestField::Actor,
        RequestField::ActionType,
        RequestField::Target,
        RequestField::Payload,
        RequestField::ActorLocation,
    ] {
        let same = match field {
            RequestField::Actor => left.actor() == right.actor(),
            RequestField::ActionType => left.action_type() == right.action_type(),
            RequestField::Target => left.target() == right.target(),
            RequestField::Payload => left.payload().payload() == right.payload().payload(),
            RequestField::ActorLocation => left.actor_location() == right.actor_location(),
        };
        if !same {
            differing.push(field);
        }
    }
    differing
}

#[cfg(test)]
mod tests {
    use mineworld_contracts::{
        Action, ActionRecord, ActionRequest, ActionTypeId, EntityId, EntityType, LocalPosition,
        Location, Millimetres, PlaceId, SystemId,
    };
    use serde::{Deserialize, Serialize};

    use super::{RequestField, differing_fields, semantic_core};

    /// An action belonging to no real system: this crate must invent no domain vocabulary
    /// (`INV-12`), and the comparison does not care what the action means.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Example {
        topic: String,
    }

    impl Action for Example {
        const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("example-action");
        const OWNER: SystemId = SystemId::from_static("example-system");
    }

    const ACTOR: u64 = 4;
    const TARGET: u64 = 2;

    fn payload(topic: &str) -> ActionRecord<Vec<u8>> {
        ActionRecord::new::<Example>(
            serde_json::to_vec(&Example {
                topic: topic.to_owned(),
            })
            .expect("a payload encodes"),
        )
    }

    /// What a 2D client submits: no position, because it models none.
    fn from_a_click(topic: &str) -> ActionRequest {
        ActionRequest::new(EntityId::from_raw(ACTOR), payload(topic))
            .with_target(EntityId::from_raw(TARGET))
    }

    /// What a 3D client submits: the same request, plus the position it walked to.
    fn from_a_walk_up(topic: &str) -> ActionRequest {
        let place = PlaceId::new(EntityId::from_raw(1), EntityType::Place).expect("a place");
        from_a_click(topic).from_location(Location::in_place(place).with_local(
            LocalPosition::on_ground(Millimetres::new(867), Millimetres::new(-46)),
        ))
    }

    /// The whole of `AC-13` at this layer: the two acquisitions differ, the request does not — except
    /// in the one field the contract intends.
    #[test]
    fn a_click_and_a_walk_up_agree_on_the_semantic_core_and_differ_only_in_the_reported_position() {
        let clicked = from_a_click("greeting");
        let walked = from_a_walk_up("greeting");

        assert_eq!(
            semantic_core(&clicked),
            semantic_core(&walked),
            "actor, action type, target and payload are what the world is being asked for",
        );
        assert_eq!(
            differing_fields(&clicked, &walked),
            vec![RequestField::ActorLocation],
            "and the reported position is the only legitimate difference left after PR 04",
        );
        assert!(
            !RequestField::ActorLocation.is_semantic_core(),
            "which is why it is not part of the core",
        );
    }

    /// The counterfactual that makes the test above load-bearing: a comparison that ignored the
    /// payload would call these two the same request, and they are not.
    #[test]
    fn a_different_payload_is_a_different_request() {
        let greeting = from_a_click("greeting");
        let complaint = from_a_walk_up("complaint");

        assert_ne!(semantic_core(&greeting), semantic_core(&complaint));
        assert_eq!(
            differing_fields(&greeting, &complaint),
            vec![RequestField::Payload, RequestField::ActorLocation],
        );
    }

    /// And the same for the other three core fields, one at a time, so that no single comparison
    /// carries all of them.
    #[test]
    fn each_core_field_is_actually_compared() {
        let base = from_a_click("greeting");

        let other_actor = ActionRequest::new(EntityId::from_raw(99), payload("greeting"))
            .with_target(EntityId::from_raw(TARGET));
        assert_eq!(
            differing_fields(&base, &other_actor),
            vec![RequestField::Actor],
        );

        let no_target = ActionRequest::new(EntityId::from_raw(ACTOR), payload("greeting"));
        assert_eq!(
            differing_fields(&base, &no_target),
            vec![RequestField::Target],
        );

        // A different action type comes with a different payload record, because the contract will
        // not let a record claim a type it was not written from — which is itself the guarantee that
        // makes comparing the two fields separately meaningful.
        #[derive(Debug, Serialize, Deserialize)]
        struct Other;
        impl Action for Other {
            const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("other-action");
            const OWNER: SystemId = SystemId::from_static("example-system");
        }
        let other_action = ActionRequest::new(
            EntityId::from_raw(ACTOR),
            ActionRecord::new::<Other>(serde_json::to_vec(&Other).expect("a payload encodes")),
        )
        .with_target(EntityId::from_raw(TARGET));
        assert_eq!(
            differing_fields(&base, &other_action),
            vec![RequestField::ActionType, RequestField::Payload],
        );
    }
}
