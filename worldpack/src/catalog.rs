//! Which System Packs this build provides, and what authored content each one's state comes from.
//!
//! A closed enum, because in MVP-0 the systems are **linked**: a pack asks for `presence` and gets
//! the `mineworld-presence` crate this binary was built with. `ARC-8` makes system extension a WASM
//! component later, at which point this becomes a registry populated at startup and
//! [`PackError::UnknownSystem`](crate::PackError::UnknownSystem) changes from *this build does not
//! provide it* to *nothing installed it*. The shape of the question does not change, which is why
//! this seam is one type rather than a `match` spread through the loader.
//!
//! # Why the loader knows that a `location` is presence's
//!
//! A pack's `location:` field exists because some system owns location, and in this build that
//! system is `presence` — so [`located`] is where the pack format's vocabulary meets a System Pack's.
//! It is a function rather than a `ContentSeeder` trait deliberately: there is exactly one such
//! mapping today, and `docs/ENGINEERING_STANDARDS.md` §28 asks for the abstraction after the second
//! implementation rather than in anticipation of it. The second one — an inventory pack seeding
//! items, an employment pack seeding jobs — is when the trait gets defined, and it will have two real
//! implementations to be shaped by.

use mineworld_contracts::{Event, Location, PersonId, SystemId};
use mineworld_conversation::ConversationSystem;
use mineworld_kernel::{Emission, KernelError, SystemIdentity, World, WorldRead};
use mineworld_presence::{Arrived, PerceptionProvider, PresenceSystem, arrival};

/// One System Pack this build can install.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    /// Where people are, and what each of them perceives.
    Presence,
    /// Speaking to somebody, and remembering that they spoke to you.
    Conversation,
}

/// Every system this build provides, in a fixed order — the order an error message lists them in.
pub const AVAILABLE: [Capability; 2] = [Capability::Presence, Capability::Conversation];

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
            Self::Conversation => ConversationSystem::ID,
        }
    }

    /// Installs it into a world under assembly.
    ///
    /// Takes the system by value, as `World::install` requires: installation is the world assembler's
    /// act, and a system that is already installed cannot be installed again under the same name.
    pub fn install(self, world: &mut World) -> Result<(), KernelError> {
        match self {
            Self::Presence => world.install(PresenceSystem),
            Self::Conversation => world.install(ConversationSystem),
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
            Self::Conversation => Box::new(ConversationSystem),
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
