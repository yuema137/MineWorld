//! The world thread: one loop, one world, and every access to it a message.
//!
//! This is the only code that touches a [`World`], and it is single-threaded by construction (see
//! [`crate::host`] for why a `World` cannot be anything else). It holds the four things a running
//! world needs that the kernel deliberately does not own:
//!
//! ```text
//! the pacing         how fast a hosted world's seconds pass in real time: one per wall second.
//!                    The world's own clock and schedule are the kernel's (S4); this only says how
//!                    far to advance them, on each tick and before each request
//! the allocator      the server allocates every ActionId, because a client has no allocator
//!                    (INV-6, FINDINGS.md F4)
//! a recent window    the facts perception may look back over; the durable log is the save's
//! the subscribers    one bounded channel per connected client
//! ```
//!
//! # A persisted world
//!
//! A world may have a save ([`HostedWorld::persisted`]). Then every request and every advance that
//! fired something is committed to it **before** the request is answered and before any observation
//! is swept, so a revision a client is told is a revision on disk (step-06 I-6). The world's
//! instance, its request identities and its clock resume from the save: the instance is the save's,
//! the allocator starts past every `ActionId` the journal holds, and the host's pacing starts from
//! the instant of the world's last revision — world time does not pass while no process hosts it
//! (step-06 §10.1 Q4). A save that can no longer be written stops the world: its thread ends, and
//! every later command is answered `world_stopped` (Q6).
//!
//! Nothing in this module decides whether an action is admissible, and nothing in it decides what
//! an observer may see. Those are the kernel's and the perception seam's, which is what keeps a
//! world rule out of the server (`ENGINEERING_RULES.md` §8).

use std::sync::Arc;
use std::time::Instant;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRequest, EntityId, EntityKey, EventEnvelope, WorldTime,
};
use mineworld_kernel::{Advanced, Dispatched, KernelError, World};
use mineworld_persistence::{PersistError, WorldRevision};
use tokio::sync::mpsc;

use crate::host::{
    Command, HostConfig, Hosted, HostedWorld, Perceived, SeatRoster, Seated, Submitted,
    SubscriptionId, SubscriptionIdSource,
};
use crate::perception::{Perception, PerceptionContext};
use crate::protocol::{
    PROTOCOL_VERSION, Refusal, RefusalCode, SystemSummary, WorldInstanceId, WorldSummary,
};

/// One connected client, as the world knows it: which observer, and where to put its observations.
struct Subscriber {
    subscription: SubscriptionId,
    observer: EntityId,
    observations: mpsc::Sender<Perceived>,
}

/// The host's pacing: which instant a hosted world should have reached by now.
///
/// Wall time in whole simulated seconds from an epoch. Not the world's clock — that is the kernel's,
/// moved only by advancing and dispatching (S4) — and not a scheduler: it answers "how far should the
/// world be advanced", which is a deployment decision rather than a simulation one. A headless run
/// that wants a hundred days in a second advances the kernel directly and has no use for this.
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
    /// An allocator whose first identity is `first` — one for a world with no requests behind it,
    /// past the journal's highest for a resumed one.
    const fn starting_at(first: u64) -> Self {
        Self { next: first }
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

/// Why an input to the hosted world did not complete.
enum Failure {
    /// A system broke its own contract. Counted and reported; the world goes on. For a persisted
    /// world the fault is already journaled as the input's outcome.
    Fault(KernelError),
    /// The save could not be written: the world is ahead of it and must stop.
    Stopped(String),
}

impl Failure {
    fn from_persistence(error: PersistError) -> Self {
        match error {
            PersistError::Kernel(fault) => Self::Fault(fault),
            other => Self::Stopped(other.to_string()),
        }
    }
}

impl Hosted {
    fn world(&self) -> &World {
        match self {
            Self::Ephemeral(world) => world,
            Self::Persisted(world) => world.world(),
        }
    }

    fn revision(&self) -> Option<WorldRevision> {
        match self {
            Self::Ephemeral(_) => None,
            Self::Persisted(world) => Some(world.revision()),
        }
    }

    fn dispatch(&mut self, intent: &ActionIntent, at: WorldTime) -> Result<Dispatched, Failure> {
        match self {
            Self::Ephemeral(world) => world.dispatch(intent, at).map_err(Failure::Fault),
            Self::Persisted(world) => world
                .dispatch(intent, at)
                .map_err(Failure::from_persistence),
        }
    }

    fn advance_to(&mut self, at: WorldTime) -> Result<Advanced, Failure> {
        match self {
            Self::Ephemeral(world) => world.advance_to(at).map_err(Failure::Fault),
            Self::Persisted(world) => world.advance_to(at).map_err(Failure::from_persistence),
        }
    }
}

/// The world, and everything the server holds around it.
pub(crate) struct WorldRuntime {
    world: Hosted,
    perception: Box<dyn Perception>,
    seats: SeatRoster,
    /// Which world this is: the save's, for a persisted world; allocated once, here, otherwise.
    /// Reported unchanged to every client and every status answer (`AC-15`, `MVP.md` §9.1).
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
    /// Dispatches and advances that ended in a `KernelError` — a system breaking its own contract.
    faults: u64,
    /// Set when the save could not be written. The loop ends after the command that found it.
    stopped: Option<String>,
}

impl WorldRuntime {
    pub(crate) fn new(hosted: HostedWorld, config: Arc<HostConfig>) -> Self {
        let (instance, epoch) = match &hosted.world {
            Hosted::Ephemeral(_) => (WorldInstanceId::allocate(), config.epoch),
            Hosted::Persisted(world) => (
                WorldInstanceId::from_raw(world.instance()),
                world.world().now(),
            ),
        };
        Self {
            world: hosted.world,
            perception: hosted.perception,
            seats: hosted.seats,
            instance,
            clock: HostClock::new(epoch),
            actions: ActionIds::starting_at(hosted.first_action),
            subscriptions: SubscriptionIdSource::new(),
            subscribers: Vec::new(),
            recent: hosted.recent,
            dropped: 0,
            faults: 0,
            stopped: None,
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
                Command::Sweep => self.tick(),
                Command::Shutdown => {
                    self.checkpoint();
                    break;
                }
            }
            if let Some(cause) = &self.stopped {
                eprintln!("[world] stopped: the save can no longer be written ({cause})");
                break;
            }
        }
    }

    /// On a clean shutdown, writes a snapshot at the head so the next start re-executes nothing —
    /// when the clock still stands at the head's instant (`PersistentWorld::checkpoint`).
    fn checkpoint(&mut self) {
        if let Hosted::Persisted(world) = &mut self.world
            && let Err(error) = world.checkpoint()
        {
            eprintln!("[world] the shutdown checkpoint was not written: {error}");
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
            .world()
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

        // Whatever was due by now happens first: it was scheduled before this request arrived, and
        // the kernel refuses a request that would overtake it (`ScheduledWorkDue`).
        if let Err(failure) = self.advance(at) {
            return Err(self.refusal(failure));
        }

        match self.world.dispatch(&intent, at) {
            Ok(dispatched) => {
                // What the request deferred is already in the world's schedule (S4); it fires when
                // the world is advanced to its instant, at a later tick or a later request.
                let (result, events, _deferred) = dispatched.into_parts();
                let recorded = !events.is_empty();
                self.remember(events);
                // Straight away rather than at the next tick, so that the facts a request caused
                // reach every client entitled to them without waiting out the cadence.
                if recorded {
                    self.sweep();
                }
                Ok(Submitted::new(action_id, result))
            }
            Err(failure) => Err(self.refusal(failure)),
        }
    }

    /// The refusal a client is sent for an input that did not complete — and the bookkeeping that
    /// goes with it: a fault is counted, a save that cannot be written stops the world.
    fn refusal(&mut self, failure: Failure) -> Refusal {
        match failure {
            Failure::Fault(error) => {
                // `kernel/src/dispatch.rs`: an `Err` out of dispatch is a bug in a system rather
                // than a rejected request. The world is not torn down over it — other clients are
                // connected to it — but it is counted and reported rather than swallowed.
                self.faults += 1;
                eprintln!("[world] a system broke its own contract: {error}");
                Refusal::new(RefusalCode::DispatchFailed).detailed(error)
            }
            Failure::Stopped(cause) => {
                let refusal = Refusal::new(RefusalCode::WorldStopped).detail(cause.clone());
                self.stopped = Some(cause);
                refusal
            }
        }
    }

    /// Moves the world to the host's instant, firing whatever was scheduled up to it, and keeps the
    /// facts that produced for perception.
    fn advance(&mut self, at: WorldTime) -> Result<(), Failure> {
        let advanced = self.world.advance_to(at)?;
        self.remember(advanced.into_events());
        Ok(())
    }

    /// One tick of the host's cadence: let the world's time catch up with the host's, then show
    /// every client what it may now perceive.
    fn tick(&mut self) {
        let at = self.clock.now();
        if let Err(failure) = self.advance(at) {
            // Counted and reported, or the world stopped; either way nobody is waiting for an answer.
            let _ = self.refusal(failure);
            if self.stopped.is_some() {
                return;
            }
        }
        self.sweep();
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
    /// computed for anybody else. Each carries the world's persisted revision, which every input
    /// so far has already been committed to.
    ///
    /// `try_send`, never `send`: a client that has stopped reading loses frames, and the world does
    /// not wait. A client whose channel has closed is dropped here, which is also how a killed
    /// connection is reaped if its session never got to release the subscription.
    fn sweep(&mut self) {
        let at = self.clock.now();
        let revision = self.world.revision();
        let mut dropped = 0;
        let mut closed: Vec<SubscriptionId> = Vec::new();

        for subscriber in &self.subscribers {
            let context =
                PerceptionContext::new(self.world.world(), subscriber.observer, at, &self.recent);
            let observation = self.perception.observe(&context);
            match subscriber.observations.try_send(Perceived {
                revision,
                observation,
            }) {
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
        let systems = self.world.world().systems();
        WorldSummary {
            protocol: PROTOCOL_VERSION,
            instance: self.instance,
            at: self.clock.now(),
            entities: self.world.world().entities().len(),
            systems: systems
                .order()
                .iter()
                .map(|system| {
                    // The declaration a system made when it was installed: its vocabulary, which is
                    // composition rather than state, so it is public (`PROTOCOL.md` §5.7).
                    let declaration = systems.declaration(system);
                    SystemSummary {
                        system: system.clone(),
                        enabled: systems.is_enabled(system),
                        provides: declaration.map_or_else(Vec::new, |d| d.provides().to_vec()),
                        states: declaration.map_or_else(Vec::new, |d| d.emits().to_vec()),
                    }
                })
                .collect(),
            seats: self.seats.iter().cloned().collect(),
            clients: self.subscribers.len(),
            observations_dropped: self.dropped,
            // No fact is delivered to an observer before S11-C, so none is dropped.
            events_dropped: 0,
            faults: self.faults,
            revision: self.world.revision(),
        }
    }
}
