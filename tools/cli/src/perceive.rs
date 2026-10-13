//! Adapting a loaded world's perception onto the seam the server asks for.
//!
//! Two layers meet here and neither may know about the other:
//!
//! ```text
//! mineworld-server    asks: given this observer, what does it perceive?  (Perception)
//! systems/presence    answers it, for a world that has locations         (observe)
//! ```
//!
//! The server must not depend on a System Pack — a transport that knew what a conversation was would
//! be the one-way dependency rule inverted — and a System Pack must not depend on the transport. So
//! the adapter lives in the **composition root**, which is this binary: the only crate in the
//! workspace that depends on both.
//!
//! # What this adapter decides
//!
//! Nothing, and since PR 05d it does not even convert. It holds the providers the pack composed and
//! calls
//! [`observe`](mineworld_presence::observe). Every judgement about what an observer may know or may
//! attempt is made inside the perception system and the kernel's route map
//! (`ENGINEERING_RULES.md` §8).
//!
//! Until PR 05d this adapter also re-parameterized the observation from the contract's opaque
//! `Vec<u8>` payload onto the wire's `serde_json::Value`, and carried a note that the day a
//! perception system exposed a *component* through it, that record's payload would reach a client as
//! `spike/FINDINGS.md` F8.2's array of byte integers — with the fix belonging in the perception
//! system, which knows the component's type. That day came in the same PR: a pack now discloses its
//! own state as an already-encoded value, `observe` produces `Observation<Value>` directly, and the
//! conversion is deleted rather than extended.

use mineworld_contracts::{EntityId, EventEnvelope};
use mineworld_kernel::World;
use mineworld_presence::audience::{Whereabouts, admits};
use mineworld_presence::{PerceptionProvider, observe};
use mineworld_server::{EventPerception, Perception, PerceptionContext, WireObservation};

/// What a loaded World Pack's observers perceive.
///
/// Holds the [`PerceptionProvider`]s the pack composed, in the order it composed them, because that
/// order is the order affordances reach an observation and an observation that reordered between two
/// runs of one world would break replay comparison (`AC-12`).
pub struct PackPerception {
    providers: Vec<Box<dyn PerceptionProvider>>,
}

impl PackPerception {
    /// Perceives a world through these packs.
    pub const fn new(providers: Vec<Box<dyn PerceptionProvider>>) -> Self {
        Self { providers }
    }
}

impl Perception for PackPerception {
    fn observe(&self, context: &PerceptionContext<'_>) -> WireObservation {
        let borrowed: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        observe(context.world(), context.observer(), context.at(), &borrowed)
    }
}

/// Which recorded facts a loaded world's observers learn of: presence's audience rule
/// (`docs/DECISIONS.md` `ARC-43`), adapted onto the server's second seam.
///
/// Holds presence's [`Whereabouts`], seeded from the world it is built with — on the world's own
/// thread, at start — and advanced by every fact the server records. The fold from a save's first
/// fact and this seed agree (step-12 E-SC2), which is what lets a live stream and a backfill read
/// from the save judge alike. Decides nothing itself, as [`PackPerception`] does not.
pub struct PackEventPerception {
    whereabouts: Whereabouts,
}

impl PackEventPerception {
    /// Judges facts from where everybody in `world` is now.
    pub fn seeded(world: &World) -> Self {
        Self {
            whereabouts: Whereabouts::from_world(&world.read()),
        }
    }
}

impl EventPerception for PackEventPerception {
    fn record(&mut self, fact: &EventEnvelope) {
        self.whereabouts.apply(fact);
    }

    fn admits(&self, fact: &EventEnvelope, observer: EntityId) -> bool {
        admits(&self.whereabouts, fact, observer)
    }
}
