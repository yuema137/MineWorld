//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`, `ARC-65`;
//! `configure/relationships.yaml`).
//!
//! ```yaml
//! rules:
//!   - { action: acquaint, actor: noble, target: commoner, effect: forbid }
//! parameters:
//!   - { spoke_familiarity: 10, accepted_regard: 50, declined_regard: -30,
//!       activity_familiarity: 50, activity_regard: 20 }
//!   - { actor: grump, declined_regard: -100 }
//!   - { place: cafe, spoke_familiarity: 20 }
//! consequences:
//!   - { fact: became-acquainted, actor: servant, biography: off }
//! ```
//!
//! ```text
//! rule         acquaint   a rule name, not an action: whether `actor` (the person who would come
//!                         to know) forms a relationship with `target` (the counterpart). Asked per
//!                         directed change while this pack reduces another pack's fact, before any
//!                         state is written. Directed like `knows`; not regional; no place role
//! parameters   the five contact increments, looked up with actor = the person whose values change,
//!              target = the counterpart, place = where the causing fact happened
//! facts        became-acquainted, relationship-changed   participants; biography configurable (on)
//! ```
//!
//! No installed pack provides an action type `acquaint` (a test holds it), so the rule name cannot be
//! mistaken for something a client sends (`ARC-63` note of 2026-10-10). Every default is today's
//! constant, so a world that does not configure the section reduces exactly as before.

use mineworld_contracts::{
    ActionTypeId, ComponentTypeId, Event, EventTypeId, PersonId, PlaceId, Visibility,
};
use mineworld_kernel::WorldRead;
use mineworld_sdk::interactions::{
    self, ActionDecl, Audience, FactDecl, InteractionSection, Position, Resolved, Role, Roles,
};

use crate::codec;
use crate::event::{BecameAcquainted, RelationshipChanged};
use crate::system::RelationshipsSystem;

/// The rule name's text.
///
/// Named, rather than written inline as `from_static("…")`, on purpose: the client-text catalog check
/// (`tests/acceptance/tests/client_text.rs`, AC-SET-4) collects every inline action-type literal under
/// `systems/` as a code a client may be shown and needs a label for. `acquaint` is never sent to a
/// client — it is not offered, not dispatchable and never a request's outcome — so it has no label,
/// and it is not one of those literals (`pr-il-e-social.md` D-IE-3).
const ACQUAINT_NAME: &str = "acquaint";

/// The rule name relationships asks before a person comes to know another. It names no action type
/// any pack provides (SD-IE-8).
pub const ACQUAINT: ActionTypeId = ActionTypeId::from_static(ACQUAINT_NAME);

mineworld_sdk::parameters! {
    /// What a world may choose about how contact moves a relationship.
    pub struct RelationshipParameters, partial RelationshipPartial {
        /// What one exchange of words adds to familiarity, each way.
        spoke_familiarity: i32 = 10, 0 ..= 1_000;
        /// What an accepted invitation adds to regard, each way.
        accepted_regard: i32 = 50, -1_000 ..= 1_000;
        /// What a declined invitation does to the inviter's regard for the person who declined.
        declined_regard: i32 = -30, -1_000 ..= 1_000;
        /// What an activity done together adds to familiarity, per ordered pair of members.
        activity_familiarity: i32 = 50, 0 ..= 1_000;
        /// What an activity done together adds to regard, per ordered pair of members.
        activity_regard: i32 = 20, -1_000 ..= 1_000;
    }
}

/// Where a role is found in both facts' envelopes: the holder is the subject, the counterpart the
/// second participant.
const FACT_ROLES: &[(Role, Position)] = &[
    (Role::Actor, Position::Subject(0)),
    (Role::Target, Position::Participant(1)),
    (Role::Place, Position::Place),
];

impl InteractionSection for RelationshipsSystem {
    type Parameters = RelationshipParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId =
        EventTypeId::from_static("relationships-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("relationships-interactions");
    const ACTIONS: &'static [ActionDecl] = &[ActionDecl {
        action: ACQUAINT,
        roles: &[Role::Actor, Role::Target],
        regional: false,
    }];
    const FACTS: &'static [FactDecl] = &[
        FactDecl {
            fact: BecameAcquainted::EVENT_TYPE,
            roles: FACT_ROLES,
            default_audience: Audience::Participants,
            narrowest: Audience::Participants,
            biography_configurable: true,
        },
        FactDecl {
            fact: RelationshipChanged::EVENT_TYPE,
            roles: FACT_ROLES,
            default_audience: Audience::Participants,
            narrowest: Audience::Participants,
            biography_configurable: true,
        },
    ];
    const PARAMETER_ROLES: &'static [Role] = &[Role::Actor, Role::Target, Role::Place];

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        codec::encode(resolved)
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

/// The roles of a directed change: the person whose values change, and the counterpart.
fn directed(person: PersonId, counterpart: PersonId) -> Roles {
    Roles::new()
        .with(Role::Actor, person.entity_id())
        .with(Role::Target, counterpart.entity_id())
}

/// Whether the world's list lets `person` come to know `counterpart`, asked at the causing fact's
/// place (`acquaint` is not regional, so every place answers alike). A cause without a place — none
/// of this pack's four causes is one — is permitted, the compiled default.
pub(crate) fn may_acquaint(
    world: &WorldRead<'_>,
    at: Option<PlaceId>,
    person: PersonId,
    counterpart: PersonId,
) -> bool {
    at.is_none_or(|place| {
        interactions::permits::<RelationshipsSystem>(
            world,
            place,
            &ACQUAINT,
            &directed(person, counterpart),
        )
        .is_ok()
    })
}

/// The increments for `person`'s values about `counterpart`, at the causing fact's place: the world's,
/// or the compiled defaults without a place.
pub(crate) fn increments(
    world: &WorldRead<'_>,
    at: Option<PlaceId>,
    person: PersonId,
    counterpart: PersonId,
) -> RelationshipParameters {
    at.map_or_else(RelationshipParameters::default, |place| {
        interactions::parameters::<RelationshipsSystem>(
            world,
            place,
            &directed(person, counterpart),
        )
    })
}

/// The audience of one of this pack's facts about `person` and `counterpart`: the two of them, its
/// owner default (written once, in its `FactDecl`), unless the list says otherwise.
pub(crate) fn audience(
    world: &WorldRead<'_>,
    at: Option<PlaceId>,
    fact: &EventTypeId,
    person: PersonId,
    counterpart: PersonId,
) -> Visibility {
    let declared = <RelationshipsSystem as InteractionSection>::FACTS
        .iter()
        .find(|declared| declared.fact == *fact)
        .expect("relationships states only the facts it declares");
    let owner_default = match declared.default_audience {
        Audience::Public => Visibility::Public,
        Audience::Place => at.map_or(Visibility::Participants, Visibility::Place),
        Audience::Participants => Visibility::Participants,
    };
    interactions::consequence::<RelationshipsSystem>(
        world,
        at,
        fact,
        &directed(person, counterpart),
        owner_default,
    )
    .visibility
}

#[cfg(test)]
mod tests {
    use mineworld_kernel::System;

    use super::*;
    use crate::{
        ACCEPTED_REGARD, ACTIVITY_FAMILIARITY, ACTIVITY_REGARD, DECLINED_REGARD, SPOKE_FAMILIARITY,
    };

    /// The section's `default` is pinned to the pack's version (step-18 §4.10 item 3): changing a
    /// compiled increment, or the names the section is stored under, is a version change.
    #[test]
    fn the_default_section_is_pinned_to_the_version() {
        assert_eq!(RelationshipsSystem::VERSION.get(), 2);
        let defaults = RelationshipParameters::default();
        assert_eq!(
            [
                defaults.spoke_familiarity,
                defaults.accepted_regard,
                defaults.declined_regard,
                defaults.activity_familiarity,
                defaults.activity_regard,
            ],
            [
                SPOKE_FAMILIARITY,
                ACCEPTED_REGARD,
                DECLINED_REGARD,
                ACTIVITY_FAMILIARITY,
                ACTIVITY_REGARD,
            ]
        );
        assert_eq!(
            [
                SPOKE_FAMILIARITY,
                ACCEPTED_REGARD,
                DECLINED_REGARD,
                ACTIVITY_FAMILIARITY,
                ACTIVITY_REGARD
            ],
            [10, 50, -30, 50, 20]
        );
        assert_eq!(
            <RelationshipsSystem as InteractionSection>::CONFIGURED.as_str(),
            "relationships-interactions-configured"
        );
        assert_eq!(
            <RelationshipsSystem as InteractionSection>::COMPONENT.as_str(),
            "relationships-interactions"
        );
    }
}
