//! The world thread: one loop, one world, and every access to it a message.
//!
//! This is the only code that touches a [`World`], and it is single-threaded by construction (see
//! [`crate::host`] for why a `World` cannot be anything else). It holds the four things a running
//! world needs that the kernel deliberately does not own:
//!
//! ```text
//! the clock          dispatch is told the instant; the authoritative clock is S4's, and this is a
//!                    provisional stand-in that advances with wall time
//! the allocator      the server allocates every ActionId, because a client has no allocator
//!                    (INV-6, FINDINGS.md F4)
//! a recent window    the facts perception may look back over; the durable log is S5's
//! the subscribers    one bounded channel per connected client
//! ```
//!
//! Nothing in this module decides whether an action is admissible, and nothing in it decides what
//! an observer may see. Those are the kernel's and the perception seam's, which is what keeps a
//! world rule out of the server (`ENGINEERING_RULES.md` §8).

use std::sync::Arc;
use std::time::Instant;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRequest, EntityId, EntityKey, EventEnvelope, WorldTime,
};
use mineworld_kernel::World;
use tokio::sync::mpsc;

use crate::host::{
    Command, HostConfig, HostedWorld, SeatRoster, Seated, Submitted, SubscriptionId,
    SubscriptionIdSource,
};
use crate::perception::{Perception, PerceptionContext};
use crate::protocol::{
    PROTOCOL_VERSION, Refusal, RefusalCode, SystemSummary, WireObservation, WorldInstanceId,
    WorldSummary,
};

/// The first request identity a server allocates.
///
/// One, not zero, for the reason `kernel/src/dispatch.rs` gives for event identity: zero is what an
/// absent, defaulted or truncated number looks like in a serialized record.
const FIRST_ACTION_ID: u64 = 1;

/// One connected client, as the world knows it: which observer, and where to put its observations.
struct Subscriber {
    subscription: SubscriptionId,
    observer: EntityId,
    observations: mpsc::Sender<WireObservation>,
}

/// The world's clock while S4 does not exist yet.
///
/// Wall time in whole simulated seconds from a configured epoch. Deliberately the smallest thing
/// that can answer "which instant is this dispatch working in", and deliberately not a scheduler:
/// `WorldTime` is seconds (`contracts/src/time.rs`), advancing it is S4's, and a server that
/// invented tick semantics here would have to have them re-plumbed when the real clock lands.
struct HostClock {
    epoch: WorldTime,
    started: Instant,
}

impl HostClock {
    fn new(epoch: WorldTime) -> Self {
        Self {
            epoch,
            started: Instant::now(),
        }
    }

    fn now(&self) -> WorldTime {
        let elapsed = i64::try_from(self.started.elapsed().as_secs()).unwrap_or(i64::MAX);
        WorldTime::from_seconds(self.epoch.seconds().saturating_add(elapsed))
    }
}

/// The server's request-identity allocator: monotonic, never reused, consulted by nothing else.
struct ActionIds {
    next: u64,
}

impl ActionIds {
    const fn new() -> Self {
        Self {
            next: FIRST_ACTION_ID,
        }
    }

    /// Allocates the next identity, or `None` when the space is spent — which no world reaches, and
    /// which must still not wrap, because a reused `ActionId` would make two requests
    /// indistinguishable in the event log's causal chain.
    fn allocate(&mut self) -> Option<ActionId> {
        let id = ActionId::from_raw(self.next);
        self.next = self.next.checked_add(1)?;
        Some(id)
    }
}

/// The world, and everything the server holds around it.
pub(crate) struct WorldRuntime {
    world: World,
    perception: Box<dyn Perception>,
    seats: SeatRoster,
    /// Which running world this is: allocated once, here, and reported unchanged to every client
    /// and every status answer for as long as the world lives (`AC-15`, `MVP.md` §9.1).
    instance: WorldInstanceId,
    config: Arc<HostConfig>,
    clock: HostClock,
    actions: ActionIds,
    subscriptions: SubscriptionIdSource,
    subscribers: Vec<Subscriber>,
    /// The facts perception may look back over, oldest first.
    recent: Vec<EventEnvelope>,
    /// Observations dropped because a client was not reading. Counted rather than waited on.
    dropped: u64,
    /// Deferrals dispatch handed back with no scheduler to queue them.
    unscheduled: u64,
    /// Dispatches that ended in a `KernelError` — a system breaking its own contract.
    faults: u64,
}

impl WorldRuntime {
    pub(crate) fn new(hosted: HostedWorld, config: Arc<HostConfig>) -> Self {
        Self {
            world: hosted.world,
            perception: hosted.perception,
            seats: hosted.seats,
            instance: WorldInstanceId::allocate(),
            clock: HostClock::new(config.epoch),
            actions: ActionIds::new(),
            subscriptions: SubscriptionIdSource::new(),
            subscribers: Vec::new(),
            recent: Vec::new(),
            dropped: 0,
            unscheduled: 0,
            faults: 0,
            config,
        }
    }

    /// The loop. Blocking, because this thread is not part of the async runtime: it owns a world
    /// and answers messages about it.
    pub(crate) fn run(mut self, mut commands: mpsc::Receiver<Command>) {
        while let Some(command) = commands.blocking_recv() {
            match command {
                Command::Status(reply) => {
                    let _ = reply.send(self.summary());
                }
                Command::Join { seat, reply } => {
                    let _ = reply.send(self.join(seat));
                }
                Command::Leave(subscription) => {
                    self.subscribers
                        .retain(|subscriber| subscriber.subscription != subscription);
                }
                Command::Submit {
                    observer,
                    request,
                    reply,
                } => {
                    let answer = self.submit(observer, *request);
                    let _ = reply.send(answer);
                }
                Command::Sweep => self.sweep(),
                Command::Shutdown => break,
            }
        }
    }

    /// Seats a connection, or refuses it.
    ///
    /// Two refusals, and the difference matters: an unknown seat is the client asking for something
    /// the roster does not offer, while a seat whose key this world cannot resolve is a fault in the
    /// world's own composition — a roster naming an entity the World Pack did not create.
    fn join(&mut self, seat: EntityKey) -> Result<Seated, Refusal> {
        if !self.seats.contains(&seat) {
            return Err(Refusal::new(RefusalCode::UnknownSeat)
                .detail("this world offers no such seat; GET /status lists the seats it has"));
        }
        let observer = self
            .world
            .read()
            .resolve_key(&seat)
            .map_err(|error| Refusal::new(RefusalCode::SeatNotInWorld).detailed(error))?;

        let (sender, receiver) = mpsc::channel(self.config.observation_backlog);
        let subscription = self.subscriptions.allocate();
        self.subscribers.push(Subscriber {
            subscription,
            observer,
            observations: sender,
        });
        let summary = self.summary();
        Ok(Seated::new(seat, observer, summary, subscription, receiver))
    }

    /// Allocates identity and an instant for one submitted request, and dispatches it.
    ///
    /// The actor check is here rather than in the transport on purpose: it is an authority rule
    /// (`NETWORKING.md` §2 — a client may act, and may not assert), so it must hold for everything
    /// that reaches the world, not only for what arrives over a WebSocket.
    fn submit(&mut self, observer: EntityId, request: ActionRequest) -> Result<Submitted, Refusal> {
        if request.actor() != observer {
            return Err(Refusal::new(RefusalCode::ActorNotObserver).detail(
                "a client may only ask the world to act as its own observer; the actor of this \
                 request is somebody else",
            ));
        }
        let Some(action_id) = self.actions.allocate() else {
            return Err(Refusal::new(RefusalCode::DispatchFailed)
                .detail("this world has exhausted its request identities"));
        };
        let at = self.clock.now();
        let intent = ActionIntent::allocate(request, action_id, at);

        match self.world.dispatch(&intent, at) {
            Ok(dispatched) => {
                let (result, events, deferred) = dispatched.into_parts();
                if !deferred.is_empty() {
                    // Not dropped silently: dispatch hands deferrals back for a scheduler, and this
                    // server has none until S4. A system that defers here is asking for something
                    // the host cannot yet do, and the count is what makes that visible.
                    self.unscheduled += deferred.len() as u64;
                    eprintln!(
                        "[world] {} deferral(s) had nowhere to go: this server has no scheduler \
                         (S4). Total so far: {}",
                        deferred.len(),
                        self.unscheduled
                    );
                }
                let recorded = !events.is_empty();
                self.remember(events);
                // Straight away rather than at the next tick, so that the facts a request caused
                // reach every client entitled to them without waiting out the cadence.
                if recorded {
                    self.sweep();
                }
                Ok(Submitted::new(action_id, result))
            }
            Err(error) => {
                // `kernel/src/dispatch.rs`: an `Err` out of dispatch is a bug in a system rather
                // than a rejected request. The world is not torn down over it — other clients are
                // connected to it — but it is counted and reported rather than swallowed.
                self.faults += 1;
                eprintln!("[world] a system broke its own contract while resolving: {error}");
                Err(Refusal::new(RefusalCode::DispatchFailed).detailed(error))
            }
        }
    }

    /// Keeps the newest facts and forgets the rest.
    fn remember(&mut self, events: Vec<EventEnvelope>) {
        self.recent.extend(events);
        let limit = self.config.recent_events;
        if self.recent.len() > limit {
            self.recent.drain(..self.recent.len() - limit);
        }
    }

    /// Sends every connected client the observation *its* observer is entitled to.
    ///
    /// One perception call per subscriber, which is the whole of `INV-13` at this layer: there is no
    /// world frame that is then filtered per client, and no client receives anything that was
    /// computed for anybody else.
    ///
    /// `try_send`, never `send`: a client that has stopped reading loses frames, and the world does
    /// not wait. A client whose channel has closed is dropped here, which is also how a killed
    /// connection is reaped if its session never got to release the subscription.
    fn sweep(&mut self) {
        let at = self.clock.now();
        let mut dropped = 0;
        let mut closed: Vec<SubscriptionId> = Vec::new();

        for subscriber in &self.subscribers {
            let context =
                PerceptionContext::new(&self.world, subscriber.observer, at, &self.recent);
            let observation = self.perception.observe(&context);
            match subscriber.observations.try_send(observation) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => dropped += 1,
                Err(mpsc::error::TrySendError::Closed(_)) => closed.push(subscriber.subscription),
            }
        }

        self.dropped += dropped;
        if !closed.is_empty() {
            self.subscribers
                .retain(|subscriber| !closed.contains(&subscriber.subscription));
        }
    }

    /// What the world is, as a status answer or a welcome states it.
    fn summary(&self) -> WorldSummary {
        let systems = self.world.systems();
        WorldSummary {
            protocol: PROTOCOL_VERSION,
            instance: self.instance,
            at: self.clock.now(),
            entities: self.world.entities().len(),
            systems: systems
                .order()
                .iter()
                .map(|system| SystemSummary {
                    system: system.clone(),
                    enabled: systems.is_enabled(system),
                })
                .collect(),
            seats: self.seats.iter().cloned().collect(),
            clients: self.subscribers.len(),
            observations_dropped: self.dropped,
            deferrals_unscheduled: self.unscheduled,
            faults: self.faults,
        }
    }
}
