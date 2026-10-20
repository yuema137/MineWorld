//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`, `ARC-65`;
//! `configure/sleep.yaml`).
//!
//! ```yaml
//! rules:
//!   - { action: sleep, place: bench, effect: permit }      # `bench` defined in configure/classes.yaml
//!   - { action: sleep, actor: guest, effect: forbid }
//! parameters:
//!   - { woken_by_talk: false, longest_minutes: 600 }
//! consequences:
//!   - { fact: fell-asleep, audience: participants }
//! ```
//!
//! ```text
//! rule         sleep         actor = the sleeper, place = where they are; regional. When no rule
//!                            matches and the list does not say `default: forbid`, the pack's own rule
//!                            decides: a place tagged `residential` (src/place.rs). `wake` is not a
//!                            section action: getting up is never forbidden
//! parameters   woken_by_talk      whether being spoken to wakes a sleeper, default true
//!              longest_minutes    the most `for` may be, 60 … 840, default 840
//!              shortest_minutes   the fewest `for` may be, 15 … 120, default 15
//! facts        fell-asleep   audience place → participants, biography configurable (off)
//!              woke          audience place → participants, biography configurable (off)
//! ```

use mineworld_contracts::{
    Action, ComponentTypeId, EntityId, Event, EventTypeId, PlaceId, Visibility,
};
use mineworld_kernel::WorldRead;
use mineworld_sdk::interactions::{
    self, ActionDecl, Audience, FactDecl, InteractionSection, Position, Resolved, Role, Roles,
};

use crate::action::Sleep;
use crate::codec;
use crate::event::{FellAsleep, Woke};
use crate::system::SleepSystem;

mineworld_sdk::parameters! {
    /// What a world may choose about sleep.
    pub struct SleepParameters, partial SleepPartial {
        /// Whether being spoken to wakes a sleeper.
        woken_by_talk: bool = true, false ..= true;
        /// The most minutes one may ask to sleep: an upper bound, not a norm.
        longest_minutes: u16 = 840, 60 ..= 840;
        /// The fewest minutes one may ask to sleep: a nap.
        shortest_minutes: u16 = 15, 15 ..= 120;
    }
}

/// The roles a `sleep` rule, a parameter entry and both facts may name.
const ROLES: &[Role] = &[Role::Actor, Role::Place];

/// Where a role is found in either fact's envelope: the sleeper is its one participant.
const FACT_ROLES: &[(Role, Position)] = &[
    (Role::Actor, Position::Participant(0)),
    (Role::Place, Position::Place),
];

impl InteractionSection for SleepSystem {
    type Parameters = SleepParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("sleep-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("sleep-interactions");
    const ACTIONS: &'static [ActionDecl] = &[ActionDecl {
        action: Sleep::ACTION_TYPE,
        roles: ROLES,
        regional: true,
    }];
    const FACTS: &'static [FactDecl] = &[
        FactDecl {
            fact: FellAsleep::EVENT_TYPE,
            roles: FACT_ROLES,
            default_audience: Audience::Place,
            narrowest: Audience::Participants,
            biography_configurable: true,
        },
        FactDecl {
            fact: Woke::EVENT_TYPE,
            roles: FACT_ROLES,
            default_audience: Audience::Place,
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

/// The roles of a sleeper's request or fact: the sleeper acts.
pub(crate) fn sleeper(person: EntityId) -> Roles {
    Roles::new().with(Role::Actor, person)
}

/// The parameters in force for `person` at `place`.
pub(crate) fn terms(read: &WorldRead<'_>, person: EntityId, place: PlaceId) -> SleepParameters {
    interactions::parameters::<SleepSystem>(read, place, &sleeper(person))
}

/// The audience `fact` about `person` is stated with at `place`: the place unless the world's list
/// narrows it.
pub(crate) fn audience(
    read: &WorldRead<'_>,
    fact: &EventTypeId,
    person: EntityId,
    place: PlaceId,
) -> Visibility {
    interactions::consequence::<SleepSystem>(
        read,
        Some(place),
        fact,
        &sleeper(person),
        Visibility::Place(place),
    )
    .visibility
}

#[cfg(test)]
mod tests {
    use mineworld_kernel::System;

    use super::*;

    /// The section's defaults are fixed before any run (`ARC-23`, I-Na-8) and pinned to the version.
    #[test]
    fn the_default_section_is_pinned_to_the_version() {
        assert_eq!(SleepSystem::VERSION.get(), 1);
        let defaults = SleepParameters::default();
        assert!(defaults.woken_by_talk);
        assert_eq!(defaults.shortest_minutes, 15);
        assert_eq!(defaults.longest_minutes, 840);
    }
}
