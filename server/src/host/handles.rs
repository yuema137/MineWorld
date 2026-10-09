//! What the world thread hands back across the boundary: a seated connection, a dispatched request,
//! one observation, and the identity of a subscription.
//!
//! Plain data and channel ends only. Nothing here touches a `World`; everything here crosses from the
//! world's thread to a connection's task.

use std::sync::Arc;

use mineworld_contracts::{ActionId, ActionResult, EntityId, EntityKey, EventId, PerceivedEvent};
use mineworld_persistence::WorldRevision;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::admission::ResumeSecret;
use crate::perception::PerceivedHistory;
use crate::protocol::{ClosingReason, TookOver, WireObservation, WorldSummary};

/// Which connection a subscription belongs to. Allocated by the world, never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    pub(crate) const fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

/// Where subscription identities come from: the world's own counter, never reused.
pub(crate) struct SubscriptionIdSource {
    next: u64,
}

impl SubscriptionIdSource {
    pub(crate) const fn new() -> Self {
        Self { next: 1 }
    }

    pub(crate) fn allocate(&mut self) -> SubscriptionId {
        let id = SubscriptionId::from_raw(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

/// One observation as the world thread hands it to a connection: what the observer perceives, and
/// the persisted revision of the state it was computed from.
#[derive(Debug, Clone)]
pub struct Perceived {
    /// The world's persisted revision when the observation was computed; `None` for a world that is
    /// not persisted.
    pub revision: Option<WorldRevision>,
    /// The newest request this connection submitted that had been dispatched when the observation
    /// was computed (`PROTOCOL.md` §5.2).
    pub acted_through: Option<ActionId>,
    /// What the observer perceives, with the facts it learned since its previous frame.
    pub observation: WireObservation,
}

/// A fact as it crosses to a connection: rendered once for every connection that learned of it.
pub type WireFact = Arc<PerceivedEvent<Value>>;

/// What a connection's stream carries, in the order the world thread queued it — which is the order
/// `PROTOCOL.md` §5.8 requires: facts before the observations they explain.
#[derive(Debug, Clone)]
pub enum Streamed {
    /// Facts for the reliable `perceived` stream, and the cursor after them.
    Facts {
        /// The newest fact considered for this connection.
        through: EventId,
        /// The facts its observer learned, oldest first.
        events: Vec<WireFact>,
    },
    /// One observation.
    Observation(Perceived),
}

/// Where a connection's `perceived` stream starts (`PROTOCOL.md` §5.8).
#[derive(Clone)]
pub struct PerceivedStart {
    /// The newest fact the world had recorded when the connection was seated; the live stream
    /// carries only later facts. `None` when the world had recorded none.
    pub head: Option<EventId>,
    /// The facts in `(since, head]` to read before the live stream, when there are any.
    pub backfill: Option<Backfill>,
}

/// A backfill to read off the world's thread: the history, and the interval of it.
#[derive(Clone)]
pub struct Backfill {
    /// Where to read it.
    pub history: Arc<dyn PerceivedHistory>,
    /// The client's cursor; `None` reads from the world's first fact.
    pub since: Option<EventId>,
    /// The head: the last fact the backfill covers.
    pub through: EventId,
}

impl std::fmt::Debug for PerceivedStart {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PerceivedStart")
            .field("head", &self.head)
            .field(
                "backfill",
                &self
                    .backfill
                    .as_ref()
                    .map(|backfill| (backfill.since, backfill.through)),
            )
            .finish()
    }
}

/// A seated connection: which observer it is, how it came by the seat, and its own stream of
/// observations.
#[derive(Debug)]
pub struct Seated {
    seat: EntityKey,
    observer: EntityId,
    world: WorldSummary,
    subscription: SubscriptionId,
    observations: mpsc::Receiver<Streamed>,
    binding: Binding,
    released: oneshot::Receiver<ClosingReason>,
    perceived: Option<PerceivedStart>,
}

/// A connection's stream read for its observations only, skipping the `perceived` facts — for an
/// in-process caller that did not ask for them and so is never sent any.
pub struct Observations<'a>(&'a mut mpsc::Receiver<Streamed>);

impl Observations<'_> {
    /// The next observation, or `None` once the world has stopped streaming to this connection.
    pub async fn recv(self) -> Option<Perceived> {
        loop {
            match self.0.recv().await? {
                Streamed::Observation(perceived) => return Some(perceived),
                Streamed::Facts { .. } => {}
            }
        }
    }
}

/// What the seat table answered about a granted seat: the welcome's control fields.
#[derive(Debug, Clone)]
pub(crate) struct Binding {
    pub(crate) took_over: TookOver,
    pub(crate) resume: ResumeSecret,
    pub(crate) hold_seconds: u32,
}

impl Seated {
    pub(crate) fn new(
        seat: EntityKey,
        observer: EntityId,
        world: WorldSummary,
        subscription: SubscriptionId,
        streams: (mpsc::Receiver<Streamed>, oneshot::Receiver<ClosingReason>),
        binding: Binding,
        perceived: Option<PerceivedStart>,
    ) -> Self {
        let (observations, released) = streams;
        Self {
            seat,
            observer,
            world,
            subscription,
            observations,
            binding,
            released,
            perceived,
        }
    }

    /// Where this connection's `perceived` stream starts, when its join asked for one.
    pub const fn perceived(&self) -> Option<&PerceivedStart> {
        self.perceived.as_ref()
    }

    /// Whether control of the Person changed hands when this connection was seated.
    pub const fn took_over(&self) -> TookOver {
        self.binding.took_over
    }

    /// The secret that re-takes this seat if this connection's socket drops.
    pub const fn resume(&self) -> &ResumeSecret {
        &self.binding.resume
    }

    /// How long the seat is held after a dropped socket, in wall seconds.
    pub const fn hold_seconds(&self) -> u32 {
        self.binding.hold_seconds
    }

    /// Completes, with the reason, when the world unbinds this connection from its seat — another
    /// connection took it over or superseded it. The session closes with that reason.
    pub const fn released(&mut self) -> &mut oneshot::Receiver<ClosingReason> {
        &mut self.released
    }

    /// Both of this connection's streams at once — its observations and its release — for a task
    /// that waits on either.
    pub const fn streams(
        &mut self,
    ) -> (
        &mut mpsc::Receiver<Streamed>,
        &mut oneshot::Receiver<ClosingReason>,
    ) {
        (&mut self.observations, &mut self.released)
    }

    /// The seat that was granted.
    pub const fn seat(&self) -> &EntityKey {
        &self.seat
    }

    /// The observer this connection sees the world as — chosen by the server, never asked for.
    pub const fn observer(&self) -> EntityId {
        self.observer
    }

    /// What the world was when the connection joined.
    pub const fn world(&self) -> &WorldSummary {
        &self.world
    }

    /// This connection's subscription, to be released when it ends.
    pub const fn subscription(&self) -> SubscriptionId {
        self.subscription
    }

    /// This connection's own observation stream, without its `perceived` facts.
    pub const fn observations(&mut self) -> Observations<'_> {
        Observations(&mut self.observations)
    }
}

/// One dispatched request: the identity the server allocated, and the world's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submitted {
    action_id: ActionId,
    result: ActionResult,
}

impl Submitted {
    pub(crate) const fn new(action_id: ActionId, result: ActionResult) -> Self {
        Self { action_id, result }
    }

    /// The identity the **server** allocated for the request (`INV-6`).
    pub const fn action_id(&self) -> ActionId {
        self.action_id
    }

    /// What the world answered.
    pub const fn result(&self) -> &ActionResult {
        &self.result
    }
}
