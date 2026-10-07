//! Which System Packs this build provides, and what authored content each one's state comes from.
//!
//! A closed enum, because in MVP-0 the systems are **linked**: a pack asks for `presence` and gets
//! the `mineworld-presence` crate this binary was built with. `ARC-8` makes system extension a WASM
//! component later, at which point this becomes a registry populated at startup and
//! [`PackError::UnknownSystem`](crate::PackError::UnknownSystem) changes from *this build does not
//! provide it* to *nothing installed it*. The shape of the question does not change, which is why
//! this seam is one type rather than a `match` spread through the loader.
//!
//! # Why the loader knows that a `location` is presence's, and a `passage` movement's
//!
//! A pack's `location:` field exists because some system owns location, and in this build that
//! system is `presence` — so [`located`] is where the pack format's vocabulary meets a System Pack's.
//! [`opened`] is the second such meeting: a place's `passages:` become movement's fact.
//!
//! Those two are fields of the format, mapped here by name. They predate the section seam and stay
//! where they are: moving them would edit presence and movement and change refusals authors already
//! see (`DECISIONS.md` `ARC-31`).
//!
//! # Sections: content a pack owns, which the loader does not understand
//!
//! Everything newer is a **section** (`ARC-31`): a pack implements
//! [`AuthoredSection`](mineworld_authoring::AuthoredSection), and the only thing this file says about
//! it is which capability that is — [`Capability::section`] and [`Capability::decode_section`] — in
//! the same closed match that already says which crate each capability is. The loader decodes a
//! section with the owner's type and seeds it with the owner's function, and never learns what it
//! means. Adding a section-owning pack adds an arm here and no loader code.

use std::sync::Arc;

use mineworld_authoring::{AuthoredContent, AuthoredSection, ContentKind, Decode, SectionName};
use mineworld_contracts::{
    Event, EventTypeId, LocalPosition, Location, PersonId, PlaceId, SystemId,
};
use mineworld_conversation::ConversationSystem;
use mineworld_group_activity::GroupActivitySystem;
use mineworld_kernel::{Emission, KernelError, SystemIdentity, World, WorldRead};
use mineworld_movement::{MovementSystem, passage};
use mineworld_naming::NamingSystem;
use mineworld_presence::{Arrived, PerceptionProvider, PresenceSystem, arrival};
use mineworld_relationships::RelationshipsSystem;
use mineworld_schedule::ScheduleSystem;
use serde::de::{Error as _, MapAccess};

/// One System Pack this build can install.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    /// Where people are, and what each of them perceives.
    Presence,
    /// Whether a person may walk where they ask, and which places open onto which.
    Movement,
    /// Speaking to somebody, and remembering that they spoke to you.
    Conversation,
    /// Inviting, answering, joining and leaving something done together.
    GroupActivity,
    /// Who knows whom, and how well — changed only by what happened between them.
    Relationships,
    /// What people are called.
    Naming,
    /// A person's day: where they mean to be, and when — an agenda, never a mover.
    Schedule,
}

/// Every system this build provides, in a fixed order — the order an error message lists them in.
pub const AVAILABLE: [Capability; 7] = [
    Capability::Presence,
    Capability::Movement,
    Capability::Conversation,
    Capability::GroupActivity,
    Capability::Relationships,
    Capability::Naming,
    Capability::Schedule,
];

/// The section of authored content a capability owns: its key, and the files that may carry it
/// (`DECISIONS.md` `ARC-31`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionOwner {
    /// The key.
    pub name: SectionName,
    /// The kinds of content file that may carry it.
    pub carried_by: &'static [ContentKind],
}

impl SectionOwner {
    const fn of<S: AuthoredSection>() -> Self {
        Self {
            name: S::SECTION,
            carried_by: S::CARRIED_BY,
        }
    }

    /// Whether a file of this kind may carry the section.
    pub fn carried_by(&self, kind: ContentKind) -> bool {
        self.carried_by.contains(&kind)
    }
}

impl Capability {
    /// Which capability a pack is asking for, or [`None`] if this build has no such system.
    pub fn resolve(name: &SystemId) -> Option<Self> {
        AVAILABLE
            .into_iter()
            .find(|capability| capability.id() == *name)
    }

    /// The name this capability is enabled by.
    pub fn id(self) -> SystemId {
        match self {
            Self::Presence => PresenceSystem::ID,
            Self::Movement => MovementSystem::ID,
            Self::Conversation => ConversationSystem::ID,
            Self::GroupActivity => GroupActivitySystem::ID,
            Self::Relationships => RelationshipsSystem::ID,
            Self::Naming => NamingSystem::ID,
            Self::Schedule => ScheduleSystem::ID,
        }
    }

    /// The section of authored content this capability owns, if any (`ARC-31`).
    pub const fn section(self) -> Option<SectionOwner> {
        match self {
            Self::Presence
            | Self::Movement
            | Self::Conversation
            | Self::GroupActivity
            | Self::Relationships => None,
            Self::Naming => Some(SectionOwner::of::<NamingSystem>()),
            Self::Schedule => Some(SectionOwner::of::<ScheduleSystem>()),
        }
    }

    /// The capability that owns the section with this key, and that section, if this build has one.
    pub fn owning_section(key: &str) -> Option<(Self, SectionOwner)> {
        AVAILABLE.into_iter().find_map(|capability| {
            capability
                .section()
                .filter(|section| section.name.as_str() == key)
                .map(|section| (capability, section))
        })
    }

    /// Decodes this capability's section as the next value of an authored file's map, with the
    /// owner's own type — straight from the stream, so a refusal keeps its line and column (`DEP-10`).
    /// The loader holds the result without knowing its type.
    pub(crate) fn decode_section<'de, A: MapAccess<'de>>(
        self,
        map: &mut A,
    ) -> Result<Arc<dyn AuthoredContent>, A::Error> {
        match self {
            Self::Naming => map.next_value_seed(Decode::<NamingSystem>::new()),
            Self::Schedule => map.next_value_seed(Decode::<ScheduleSystem>::new()),
            Self::Presence
            | Self::Movement
            | Self::Conversation
            | Self::GroupActivity
            | Self::Relationships => Err(A::Error::custom(format!(
                "the '{self}' system owns no section"
            ))),
        }
    }

    /// Which of this capability's event types belong in a person's objective biography
    /// (`DECISIONS.md` `ARC-29`) — each pack's own judgement over its own vocabulary, aggregated here.
    ///
    /// A pack that declares none contributes none: presence, movement and conversation export no such
    /// list in S8, and they are answered `&[]` here rather than edited to say so (`step-09-social.md`
    /// B-5, I-1). Adding a biographical pack adds its constant to this match and no biography code.
    pub fn biographical(self) -> &'static [EventTypeId] {
        match self {
            Self::Presence | Self::Movement | Self::Conversation => &[],
            Self::GroupActivity => mineworld_group_activity::BIOGRAPHICAL,
            Self::Relationships => mineworld_relationships::BIOGRAPHICAL,
            Self::Naming => mineworld_naming::BIOGRAPHICAL,
            Self::Schedule => mineworld_schedule::BIOGRAPHICAL,
        }
    }

    /// Installs it into a world under assembly.
    ///
    /// Takes the system by value, as `World::install` requires: installation is the world assembler's
    /// act, and a system that is already installed cannot be installed again under the same name.
    pub fn install(self, world: &mut World) -> Result<(), KernelError> {
        match self {
            Self::Presence => world.install(PresenceSystem),
            Self::Movement => world.install(MovementSystem),
            Self::Conversation => world.install(ConversationSystem),
            Self::GroupActivity => world.install(GroupActivitySystem),
            Self::Relationships => world.install(RelationshipsSystem),
            Self::Naming => world.install(NamingSystem),
            Self::Schedule => world.install(ScheduleSystem),
        }
    }

    /// This capability as an answerer of "what may this observer attempt against that target".
    ///
    /// A second value of the system type, which `systems/presence/src/interaction.rs` documents as
    /// safe by construction rather than merely convenient: a system holds no fields, because its
    /// mutable state is the components it owns and those live in the world.
    pub fn provider(self) -> Box<dyn PerceptionProvider> {
        match self {
            Self::Presence => Box::new(PresenceSystem),
            Self::Movement => Box::new(MovementSystem),
            Self::Conversation => Box::new(ConversationSystem),
            Self::GroupActivity => Box::new(GroupActivitySystem),
            Self::Relationships => Box::new(RelationshipsSystem),
            Self::Naming => Box::new(NamingSystem),
            Self::Schedule => Box::new(ScheduleSystem),
        }
    }
}

impl core::fmt::Display for Capability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.id().as_str())
    }
}

/// Which capability owns the state an authored `location` becomes.
///
/// Named here rather than assumed in the loader, so that the refusal an author sees — *you placed
/// somebody without enabling the system that owns placement* — says the name of the system they have
/// to enable.
pub const LOCATION_OWNER: Capability = Capability::Presence;

/// Which capability owns the state an authored `passage` becomes: the places a place opens onto.
pub const PASSAGE_OWNER: Capability = Capability::Movement;

/// The genesis fact an authored passage becomes: `a` opens onto `b`, through a doorway at `a_at` in
/// `a` and `b_at` in `b`. Built by the pack that declared the event type
/// ([`mineworld_movement::passage`]); this function only says which fact a passage is.
pub fn opened(
    a: PlaceId,
    a_at: Option<LocalPosition>,
    b: PlaceId,
    b_at: Option<LocalPosition>,
) -> Emission {
    passage(a, a_at, b, b_at)
}

/// The genesis fact an authored location becomes.
///
/// The payload is built by the pack that declared the event type, not here: this function only says
/// *which* fact a location is. See [`mineworld_presence::arrival`]. It is built against the
/// assembled world, so an authored placement passes the same check the owner applies to every other
/// arrival (`DECISIONS.md` `ARC-26`: the owner still decides).
///
/// # Errors
///
/// [`KernelError::FactRefusedByOwner`] when presence refuses the value. `read` has already refused
/// a location naming a place the pack does not declare, so reaching this is the two disagreeing.
pub fn located(
    world: &WorldRead<'_>,
    person: PersonId,
    location: Location,
) -> Result<Emission, KernelError> {
    arrival(world, person, location).map_err(|reason| KernelError::FactRefusedByOwner {
        system: PresenceSystem::ID,
        event_type: Arrived::EVENT_TYPE,
        reason,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::content::{PERSON_FIELDS, PLACE_FIELDS};

    /// Section names are one namespace across every pack this build provides, and none may shadow a
    /// field of the format (`ARC-31`). A structural guard over a closed catalog: in this build a
    /// refusal at read time could never be reached, so the property is held here.
    #[test]
    fn no_two_sections_share_a_name_and_none_shadows_a_field() {
        let sections: Vec<SectionOwner> = AVAILABLE
            .into_iter()
            .filter_map(Capability::section)
            .collect();
        assert!(!sections.is_empty(), "the guard guards something");
        let names: BTreeSet<&str> = sections
            .iter()
            .map(|section| section.name.as_str())
            .collect();
        assert_eq!(
            names.len(),
            sections.len(),
            "two packs claim one section: {names:?}"
        );
        for name in &names {
            assert!(
                !PERSON_FIELDS.contains(name) && !PLACE_FIELDS.contains(name),
                "the section `{name}` shadows a field of the format"
            );
        }
        for section in &sections {
            assert!(
                !section.carried_by.is_empty(),
                "`{}` is carried by no kind of file, so it could never be authored",
                section.name
            );
        }
    }
}
