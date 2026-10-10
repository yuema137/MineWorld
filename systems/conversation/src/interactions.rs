//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`;
//! `configure/conversation.yaml`).
//!
//! ```yaml
//! parameters:
//!   - { gap: 600 }                      # the world's gap
//!   - { actor: regular, gap: 3600 }     # a class's own
//! regions:
//!   cafe: { parameters: [ { gap: 1800 } ] }
//! ```
//!
//! In S17's PR IL-b the section holds one parameter: `gap`, the silence after which the next exchange
//! starts a new conversation — whole seconds, 1 … 86 400, default 300 ([`CONVERSATION_GAP`]). It is
//! looked up where `resolve` decides, with the speaker as `actor`, the listener as `target` and the
//! speaker's place. The section declares no action and no fact yet, so a `rules:` or `consequences:`
//! entry is refused as naming one this pack does not declare. A world that does not configure the
//! section decides exactly as before.

use mineworld_contracts::{ComponentTypeId, EventTypeId};
use mineworld_sdk::interactions::{InteractionSection, Resolved, Role};

use crate::codec;
#[cfg(test)]
use crate::system::CONVERSATION_GAP;
use crate::system::ConversationSystem;

mineworld_sdk::parameters! {
    /// What a world may choose about conversation.
    pub struct ConversationParameters, partial ConversationPartial {
        /// Seconds of silence after which the next exchange starts a new conversation.
        gap: u32 = 300, 1 ..= 86_400;
    }
}

impl InteractionSection for ConversationSystem {
    type Parameters = ConversationParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId =
        EventTypeId::from_static("conversation-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("conversation-interactions");
    const PARAMETER_ROLES: &'static [Role] = &[Role::Actor, Role::Target, Role::Place];

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        codec::encode(resolved)
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use mineworld_kernel::System;

    use super::*;

    /// The section's `default` is pinned to the pack's version (step-18 §4.10 item 3): changing the
    /// compiled gap, or the names the section is stored under, is a version change.
    #[test]
    fn the_default_section_is_pinned_to_the_version() {
        assert_eq!(ConversationSystem::VERSION.get(), 2);
        assert_eq!(
            i64::from(ConversationParameters::default().gap),
            CONVERSATION_GAP.seconds()
        );
        assert_eq!(CONVERSATION_GAP.seconds(), 300);
        assert_eq!(
            <ConversationSystem as InteractionSection>::CONFIGURED.as_str(),
            "conversation-interactions-configured"
        );
        assert_eq!(
            <ConversationSystem as InteractionSection>::COMPONENT.as_str(),
            "conversation-interactions"
        );
    }
}
