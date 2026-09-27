//! The one seam the server needs from a world: what each observer perceives.
//!
//! `docs/ARCHITECTURE.md` §6 and `contracts/src/observation.rs` both state whose decision this is:
//! *which* facts a world exposes belongs to the perception systems a world enables, not to the
//! kernel and certainly not to a transport. And the kernel's `System` trait has no hook that
//! produces an [`Observation`](mineworld_contracts::Observation) — its four methods are `install`,
//! `validate`, `resolve` and `react`.
//!
//! So hosting a world needs exactly one seam, and this is it. The server asks; something a world
//! was assembled with answers. The transport computes no perception of its own, which is what
//! keeps a world rule out of the renderer-facing layer (`ENGINEERING_RULES.md` §8).
//!
//! # The default exposes nothing, and that is the safe direction
//!
//! [`PerceivesNothing`] answers with an empty observation. A world with no perception system is a
//! world whose observers perceive nothing — not a world that leaks everything — because
//! `Observation::new`'s own documentation names an empty observation as the safe starting point and
//! `INV-13` requires that what is exposed be added deliberately.

use mineworld_contracts::{EntityId, EventEnvelope, Observation, WorldTime};
use mineworld_kernel::WorldRead;

use crate::protocol::WireObservation;

/// What one perception decision is made against.
///
/// Read-only by construction: it carries a [`WorldRead`], which is the same view a system's
/// `validate` is handed, so producing an observation cannot change the world.
pub struct PerceptionContext<'a> {
    world: WorldRead<'a>,
    observer: EntityId,
    at: WorldTime,
    recent_events: &'a [EventEnvelope],
}

impl<'a> PerceptionContext<'a> {
    /// Assembles the context for one observer at one instant.
    pub const fn new(
        world: WorldRead<'a>,
        observer: EntityId,
        at: WorldTime,
        recent_events: &'a [EventEnvelope],
    ) -> Self {
        Self {
            world,
            observer,
            at,
            recent_events,
        }
    }

    /// The world, read-only.
    pub const fn world(&self) -> &WorldRead<'a> {
        &self.world
    }

    /// Whose observation is being produced.
    pub const fn observer(&self) -> EntityId {
        self.observer
    }

    /// The instant it is being produced for.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// The facts the world has recorded recently, newest last, as the log entries they are.
    ///
    /// Log entries, not perceived events: deciding which of these this observer is entitled to —
    /// from each event's declared `Visibility` — is exactly the judgement this seam exists to ask
    /// for. Handing them straight to a client would be the omniscience `INV-13` forbids.
    ///
    /// The window is bounded, because a running server holds no durable log: the event log is S5's.
    pub const fn recent_events(&self) -> &'a [EventEnvelope] {
        self.recent_events
    }
}

/// What a world hands its observers.
///
/// Implemented by a perception system's adapter — PR 05a's `PresenceSystem` is the first one — and
/// by test stubs. Not `Send`, and it does not need to be: a perception implementation is built
/// inside the world's own thread, together with the world it perceives, and never crosses a thread
/// boundary (see [`WorldHost::spawn`](crate::WorldHost::spawn)).
pub trait Perception: 'static {
    /// The observation this observer is entitled to, and nothing else.
    fn observe(&self, context: &PerceptionContext<'_>) -> WireObservation;
}

/// A world whose observers perceive nothing: the default, and the safe one.
#[derive(Debug, Clone, Copy, Default)]
pub struct PerceivesNothing;

impl Perception for PerceivesNothing {
    fn observe(&self, context: &PerceptionContext<'_>) -> WireObservation {
        Observation::new(context.observer(), context.at())
    }
}
