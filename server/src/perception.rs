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

use std::fmt;

use mineworld_contracts::{EntityId, EventEnvelope, EventId, Observation, WorldTime};
use mineworld_kernel::{World, WorldRead};

use crate::protocol::WireObservation;

/// What one perception decision is made against.
///
/// Read-only by construction: it carries a `&World`, and [`World`] has no `&self` method that
/// changes anything — every component write goes through a system holding its own write token
/// (`INV-7`), and there is no interior mutability anywhere in the kernel. So producing an
/// observation cannot change the world, for the same reason a system's `validate` cannot.
///
/// # Why the whole world rather than a [`WorldRead`]
///
/// A perception system's answer includes **affordances**, and an affordance depends on two different
/// things: where everybody is, which is state, and whether this world still *provides* the action,
/// which is the system registry's answer. `mineworld_presence::observe` therefore takes a `&World`,
/// and its own documentation says why — that check has to be the kernel's route map rather than a
/// list kept in a perception implementation, so that disabling a pack removes its affordances from
/// every observation in the world with no edit anywhere (`AC-2`).
///
/// A context carrying only the stores would force every perception implementation to keep its own
/// copy of the route map, which is the one thing it must not do. [`PerceptionContext::read`] is still
/// here for an implementation that needs nothing but state.
pub struct PerceptionContext<'a> {
    world: &'a World,
    observer: EntityId,
    at: WorldTime,
    recent_events: &'a [EventEnvelope],
}

impl<'a> PerceptionContext<'a> {
    /// Assembles the context for one observer at one instant.
    pub const fn new(
        world: &'a World,
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

    /// The world, read-only — including its composition, which is what an affordance needs.
    pub const fn world(&self) -> &'a World {
        self.world
    }

    /// The world's state, as the same view a system's `validate` is handed.
    pub fn read(&self) -> WorldRead<'a> {
        self.world.read()
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

/// Which recorded facts each observer learns of (`docs/DECISIONS.md` `ARC-43`).
///
/// A second seam beside [`Perception`], because it is a different kind of judgement: an observation
/// is a pure read of the world at one instant, while who learns of a fact depends on where people
/// were *when it was recorded* — a fold that must see every fact, in log order, as it happens. The
/// world thread calls [`EventPerception::record`] for each new fact and then asks
/// [`EventPerception::admits`] for every connected observer, before the next fact is recorded.
///
/// Built on the world's thread with the world it judges, like [`Perception`], so it need not be
/// `Send`. The server names no perception system: a composition root adapts one onto this.
pub trait EventPerception: 'static {
    /// Advances the judgement past `fact`, which the world has just recorded.
    fn record(&mut self, fact: &EventEnvelope);

    /// Whether `observer` learns of `fact`, judged right after it was recorded.
    fn admits(&self, fact: &EventEnvelope, observer: EntityId) -> bool;
}

/// A world whose observers learn of no fact: the default, and the safe direction, as
/// [`PerceivesNothing`] is for observations.
#[derive(Debug, Clone, Copy, Default)]
pub struct PerceivesNoEvents;

impl EventPerception for PerceivesNoEvents {
    fn record(&mut self, _fact: &EventEnvelope) {}

    fn admits(&self, _fact: &EventEnvelope, _observer: EntityId) -> bool {
        false
    }
}

/// Where a `perceived` backfill comes from: the facts an observer learned before it joined
/// (`PROTOCOL.md` §5.8).
///
/// Read off the world's thread — on a blocking task of a connection's own — because a long world's
/// history is hundreds of thousands of facts and the world must never wait for it (step-12 I-11). So,
/// unlike the two seams above, it is `Send + Sync`: it holds no world, only the means to read a
/// history (a persisted world's save) and the same judgement [`EventPerception`] makes live, so that
/// a resumed stream and a live one agree.
pub trait PerceivedHistory: Send + Sync + 'static {
    /// The facts with an identity in `(since, through]` that `observer` learned of, oldest first —
    /// every one, from the world's first fact when `since` is `None`.
    ///
    /// # Errors
    ///
    /// [`HistoryUnavailable`] when the history cannot be read now; the connection is told its cursor
    /// is unavailable and may try again.
    fn perceived(
        &self,
        observer: EntityId,
        since: Option<EventId>,
        through: EventId,
    ) -> Result<Vec<EventEnvelope>, HistoryUnavailable>;
}

/// A history that could not be read just now, and why — for the `detail` of `cursor_unavailable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryUnavailable(pub String);

impl fmt::Display for HistoryUnavailable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for HistoryUnavailable {}
