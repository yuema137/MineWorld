//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`, `ARC-65`;
//! `configure/conversation.yaml`).
//!
//! ```yaml
//! rules:
//!   - { action: talk, actor: noble, target: commoner, effect: forbid }
//! parameters:
//!   - { gap: 600, range: 3000, remembered: 32 }   # the world's
//!   - { actor: guard, range: 6000 }               # a class's own
//! consequences:
//!   - { fact: spoke, biography: on }
//!   - { fact: spoke, actor: servant, biography: off }
//!   - { fact: spoke, audience: participants }
//!   - { fact: spoke, target: servant, remember: off }
//! regions:
//!   library: { rules: [ { action: talk, effect: forbid } ] }
//! ```
//!
//! ```text
//! rule         talk          actor = speaker, target = listener, place = the speaker's; regional.
//!                            Asked in validate after the two are known to be people and the speaker
//!                            is located, before space is judged; the offer asks the same
//! parameters   gap           seconds of silence before a new conversation, 1 … 86 400, default 300
//!              range         millimetres a voice reaches, 1 … 100 000, default 3 000
//!              remembered    entries a listener's history keeps, 1 … 64, default 32
//! facts        spoke                  audience place → participants, biography configurable (off)
//!              conversation-started   audience participants, biography configurable (off)
//! knob         remember      whether the listener's history keeps the line (on when unsaid)
//! ```
//!
//! Every default is today's constant ([`CONVERSATION_GAP`](crate::CONVERSATION_GAP),
//! [`INTERACTION_RANGE`](crate::INTERACTION_RANGE), [`REMEMBERED_AT_MOST`](crate::REMEMBERED_AT_MOST)),
//! so a world that does not configure the section decides exactly as before.

use mineworld_contracts::{Action, ComponentTypeId, Event, EventTypeId, PlaceId, Visibility};
use mineworld_sdk::interactions::{
    ActionDecl, Audience, FactDecl, InteractionSection, Position, Resolved, Role,
};

use crate::action::Talk;
use crate::codec;
use crate::event::{ConversationStarted, Spoke};
use crate::system::ConversationSystem;

mineworld_sdk::parameters! {
    /// What a world may choose about conversation.
    pub struct ConversationParameters, partial ConversationPartial {
        /// Seconds of silence after which the next exchange starts a new conversation.
        gap: u32 = 300, 1 ..= 86_400;
        /// Millimetres a voice reaches: how close a listener must be.
        range: u32 = 3_000, 1 ..= 100_000;
        /// How many exchanges a listener's history keeps.
        remembered: u16 = 32, 1 ..= 64;
    }
}

mineworld_sdk::parameters! {
    /// The values of this pack's consequence knobs when a list says nothing.
    pub struct ConversationKnobValues, partial ConversationKnobs {
        /// Whether the listener's `ConversationHistory` keeps the line.
        remember: bool = true, false ..= true;
    }
}

/// The roles a `talk` rule, a parameter entry and both facts may name.
const ROLES: &[Role] = &[Role::Actor, Role::Target, Role::Place];

/// Where a role is found in either fact's envelope: speaker and listener are its participants, in that
/// order.
const FACT_ROLES: &[(Role, Position)] = &[
    (Role::Actor, Position::Participant(0)),
    (Role::Target, Position::Participant(1)),
    (Role::Place, Position::Place),
];

impl InteractionSection for ConversationSystem {
    type Parameters = ConversationParameters;
    type Knobs = ConversationKnobs;
    const CONFIGURED: EventTypeId =
        EventTypeId::from_static("conversation-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("conversation-interactions");
    const ACTIONS: &'static [ActionDecl] = &[ActionDecl {
        action: Talk::ACTION_TYPE,
        roles: ROLES,
        regional: true,
    }];
    const FACTS: &'static [FactDecl] = &[
        FactDecl {
            fact: Spoke::EVENT_TYPE,
            roles: FACT_ROLES,
            default_audience: Audience::Place,
            narrowest: Audience::Participants,
            biography_configurable: true,
        },
        FactDecl {
            fact: ConversationStarted::EVENT_TYPE,
            roles: FACT_ROLES,
            default_audience: Audience::Participants,
            narrowest: Audience::Participants,
            biography_configurable: true,
        },
    ];
    const PARAMETER_ROLES: &'static [Role] = ROLES;

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        codec::encode(resolved)
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

/// The audience this pack states `fact` with when no list narrows it, from the fact's `FactDecl` — the
/// one place each owner default is written (SD-IE-5).
pub(crate) fn owner_default(fact: &EventTypeId, place: PlaceId) -> Visibility {
    let declared = <ConversationSystem as InteractionSection>::FACTS
        .iter()
        .find(|declared| declared.fact == *fact)
        .expect("conversation states only the facts it declares");
    match declared.default_audience {
        Audience::Public => Visibility::Public,
        Audience::Place => Visibility::Place(place),
        Audience::Participants => Visibility::Participants,
    }
}

#[cfg(test)]
mod tests {
    use mineworld_kernel::System;

    use super::*;
    use crate::{CONVERSATION_GAP, INTERACTION_RANGE, REMEMBERED_AT_MOST};

    /// The section's `default` is pinned to the pack's version (step-18 §4.10 item 3): changing a
    /// compiled default, or the names the section is stored under, is a version change.
    #[test]
    fn the_default_section_is_pinned_to_the_version() {
        assert_eq!(ConversationSystem::VERSION.get(), 3);
        let defaults = ConversationParameters::default();
        assert_eq!(i64::from(defaults.gap), CONVERSATION_GAP.seconds());
        assert_eq!(CONVERSATION_GAP.seconds(), 300);
        assert_eq!(
            i64::from(defaults.range),
            i64::from(INTERACTION_RANGE.value())
        );
        assert_eq!(INTERACTION_RANGE.value(), 3_000);
        assert_eq!(usize::from(defaults.remembered), REMEMBERED_AT_MOST);
        assert_eq!(REMEMBERED_AT_MOST, 32);
        assert!(ConversationKnobValues::default().remember);
        assert_eq!(
            <ConversationSystem as InteractionSection>::CONFIGURED.as_str(),
            "conversation-interactions-configured"
        );
        assert_eq!(
            <ConversationSystem as InteractionSection>::COMPONENT.as_str(),
            "conversation-interactions"
        );
    }

    /// Unconfigured, each emission's audience is its `FactDecl`'s default, and that default is the
    /// audience the pack stated before it had a section (IL-I1, SD-IE-5): an exchange is heard by the
    /// place, a conversation's beginning only by the two. `tests/conversation_and_presence.rs` pins
    /// the emitted envelopes to the same two values.
    #[test]
    fn each_fact_is_stated_with_its_declared_default_which_is_todays_audience() {
        let place = PlaceId::new(
            mineworld_contracts::EntityId::from_raw(9),
            mineworld_contracts::EntityType::Place,
        )
        .expect("a place");
        assert_eq!(
            owner_default(&Spoke::EVENT_TYPE, place),
            Visibility::Place(place)
        );
        assert_eq!(
            owner_default(&ConversationStarted::EVENT_TYPE, place),
            Visibility::Participants
        );
    }
}
