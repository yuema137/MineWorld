//! The world thread: one loop, one world, and every access to it a message.
//!
//! This is the only code that touches a [`World`], and it is single-threaded by construction (see
//! [`crate::host`] for why a `World` cannot be anything else). It holds the things a running world
//! needs that the kernel deliberately does not own:
//!
//! ```text
//! the pacing         how fast a hosted world's seconds pass in real time: `time_scale` per wall
//!                    second. The world's own clock and schedule are the kernel's (S4); this only
//!                    says how far to advance them, on each tick and before each request
//! the allocator      the server allocates every ActionId, because a client has no allocator
//!                    (INV-6, FINDINGS.md F4)
//! a recent window    the facts perception may look back over; the durable log is the save's
//! the subscribers    one bounded channel per connected client
//! the seat table     who drives each seat (crate::seats), and the in-server controllers it binds,
//!                    consulted on each tick before the sweep (crate::hosted, ARC-42)
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
//! world rule out of the server (`ENGINEERING_RULES.md` §8). Nothing in it waits for a client or a
//! controller either: observations go out with `try_send`, and a hosted controller's `decide` is a
//! bounded synchronous call (step-12 I-11).

mod delivery;
mod world;

use std::sync::Arc;
use std::time::Instant;

use mineworld_contracts::{
    ActionIntent, ActionRequest, EntityId, EntityKey, EventEnvelope, EventId, WorldTime,
};
use mineworld_persistence::WorldRevision;
use tokio::sync::{mpsc, oneshot};

use crate::admission::ResumeSecret;
use crate::host::{
    Binding, Command, HostConfig, Hosted, HostedWorld, SeatRoster, Seated, Submitted,
    SubscriptionId, SubscriptionIdSource,
};
use crate::hosted::HostedAnswer;
use crate::perception::{EventPerception, PerceivedHistory, Perception, PerceptionContext};
use crate::protocol::{
    PROTOCOL_VERSION, PerceivedJoin, Refusal, RefusalCode, SystemSummary, WorldInstanceId,
    WorldSummary,
};
use crate::seats::{JoinRequest, SeatTable};
use delivery::Subscriber;
use world::{ActionIds, Failure, HostClock, TickTimes};

/// The world, and everything the server holds around it.
pub(crate) struct WorldRuntime {
    world: Hosted,
    perception: Box<dyn Perception>,
    /// Who learns of each recorded fact (`ARC-43`), asked at record time.
    audience: Box<dyn EventPerception>,
    /// Where a `perceived` backfill is read, off this thread; `None` keeps no history.
    history: Option<Arc<dyn PerceivedHistory>>,
    /// The newest fact the world has recorded, which a joining connection's live stream follows.
    head: Option<EventId>,
    seats: SeatRoster,
    /// Who drives each seat. Its only writer is this thread (step-12 I-3).
    table: SeatTable,
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
    /// Learned facts dropped from a full per-connection queue before a frame carried them.
    events_dropped: u64,
    /// Dispatches and advances that ended in a `KernelError` — a system breaking its own contract.
    faults: u64,
    /// Set when the save could not be written. The loop ends after the command that found it.
    stopped: Option<String>,
    ticks: TickTimes,
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
        let bound = first_binding(&hosted.world, epoch);
        let table = SeatTable::new(
            hosted.seats.iter().cloned(),
            hosted.hosted,
            config.hold,
            bound,
        );
        // The kernel's next event identity, read once: the facts before it — genesis included —
        // were recorded before this host, and a perceived stream's live part starts after them.
        let next_event = hosted.world.world().schedule_snapshot().next_event;
        let head = next_event
            .checked_sub(1)
            .filter(|last| *last > 0)
            .map(EventId::from_raw);
        Self {
            world: hosted.world,
            perception: hosted.perception,
            audience: hosted.events,
            history: hosted.history,
            head,
            seats: hosted.seats,
            table,
            instance,
            clock: HostClock::new(epoch, config.time_scale),
            actions: ActionIds::starting_at(hosted.first_action),
            subscriptions: SubscriptionIdSource::new(),
            subscribers: Vec::new(),
            recent: hosted.recent,
            dropped: 0,
            events_dropped: 0,
            faults: 0,
            stopped: None,
            ticks: TickTimes::default(),
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
                Command::Join {
                    request,
                    perceived,
                    reply,
                } => {
                    let _ = reply.send(self.join(&request, perceived));
                }
                Command::Leave(subscription, departure) => self.depart(subscription, departure),
                Command::Submit {
                    subscription,
                    observer,
                    request,
                    reply,
                } => {
                    let answer = self.submit(subscription, observer, *request);
                    let _ = reply.send(answer);
                }
                Command::Sweep => self.tick(),
                Command::Shutdown(done) => {
                    self.checkpoint();
                    println!("{}", self.ticks.report());
                    let _ = done.send(());
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
    /// Three kinds of refusal, and the difference matters: an unknown seat is the client asking for
    /// something the roster does not offer; a seat whose key this world cannot resolve is a fault in
    /// the world's own composition; and a seat somebody else holds is the seat table's answer
    /// (`PROTOCOL.md` §4.2). A granted seat changes no world state: binding is host state (`ARC-40`).
    fn join(
        &mut self,
        request: &JoinRequest,
        perceived: Option<PerceivedJoin>,
    ) -> Result<Seated, Refusal> {
        let seat = &request.seat;
        if !self.seats.contains(seat) {
            return Err(Refusal::new(RefusalCode::UnknownSeat)
                .detail("this world offers no such seat; GET /status lists the seats it has"));
        }
        let observer = self.observer_of(seat)?;
        // Before the seat table is asked, so that a cursor this world cannot serve grants nothing.
        let start = self.perceived_start(perceived)?;
        let resume = ResumeSecret::generate()
            .map_err(|error| Refusal::new(RefusalCode::DispatchFailed).detailed(error))?;
        let subscription = self.subscriptions.allocate();
        let grant = self.table.join(
            request,
            subscription,
            resume.clone(),
            Instant::now(),
            self.clock.now(),
        )?;
        // The connection that held the seat is unbound before the new one is seated: never two.
        if let Some((displaced, reason)) = grant.displaced {
            self.release(displaced, reason);
        }

        let (sender, receiver) = mpsc::channel(self.config.observation_backlog);
        let (released, on_release) = oneshot::channel();
        self.subscribers.push(Subscriber::new(
            subscription,
            observer,
            (sender, released),
            start.as_ref().map(|start| start.head),
        ));
        let binding = Binding {
            took_over: grant.took_over,
            resume,
            hold_seconds: u32::try_from(self.config.hold.as_secs()).unwrap_or(u32::MAX),
            keyframe_every: self.config.keyframe_every,
        };
        let summary = self.summary();
        Ok(Seated::new(
            seat.clone(),
            observer,
            summary,
            subscription,
            (receiver, on_release),
            binding,
            start,
        ))
    }

    fn observer_of(&self, seat: &EntityKey) -> Result<EntityId, Refusal> {
        self.world
            .world()
            .read()
            .resolve_key(seat)
            .map_err(|error| Refusal::new(RefusalCode::SeatNotInWorld).detailed(error))
    }

    /// A session's request, at the host's instant; its facts are swept to every client at once.
    /// On a connection's behalf, the dispatched request becomes its `acted_through` before any
    /// observation reflecting it is computed.
    fn submit(
        &mut self,
        subscription: Option<SubscriptionId>,
        observer: EntityId,
        request: ActionRequest,
    ) -> Result<Submitted, Refusal> {
        let at = self.clock.now();
        let (submitted, recorded) = self.submit_at(observer, request, at)?;
        if let Some(subscription) = subscription {
            self.acted(subscription, submitted.action_id());
        }
        // Straight away rather than at the next tick, so that the facts a request caused reach
        // every client entitled to them without waiting out the cadence.
        if recorded {
            self.sweep();
        }
        Ok(submitted)
    }

    /// Allocates identity for one request and dispatches it at `at` — the one authority path, for a
    /// session and for an in-server controller alike (step-12 I-8). Returns whether it recorded facts.
    ///
    /// The actor check is here rather than in the transport on purpose: it is an authority rule
    /// (`NETWORKING.md` §2 — a client may act, and may not assert), so it must hold for everything
    /// that reaches the world, not only for what arrives over a WebSocket.
    fn submit_at(
        &mut self,
        observer: EntityId,
        request: ActionRequest,
        at: WorldTime,
    ) -> Result<(Submitted, bool), Refusal> {
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
                Ok((Submitted::new(action_id, result), recorded))
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

    /// Moves the world to `at`, firing whatever was scheduled up to it, and keeps the facts that
    /// produced for perception.
    fn advance(&mut self, at: WorldTime) -> Result<(), Failure> {
        let advanced = self.world.advance_to(at)?;
        self.remember(advanced.into_events());
        Ok(())
    }

    /// One tick of the host's cadence: holds that ended are released, due in-server controllers
    /// are consulted, the world's time catches up with the host's, and every client is shown what it
    /// may now perceive. Timed, for the operator's statistics.
    fn tick(&mut self) {
        let started = Instant::now();
        let now = self.clock.now();
        self.table.expire(started, now);
        self.consult(now);
        if self.stopped.is_none() {
            if let Err(failure) = self.advance(now) {
                // Counted and reported, or the world stopped; nobody is waiting for an answer.
                let _ = self.refusal(failure);
            }
            if self.stopped.is_none() {
                self.sweep();
            }
        }
        self.ticks.record(started.elapsed());
    }

    /// Consults every in-server controller due by `now`, in instant order then seat order, each at
    /// its own instant (or the world's, if a request already moved the world past it), through the
    /// same perception call and the same authority path a session's request takes.
    fn consult(&mut self, now: WorldTime) {
        for (due, seat) in self.table.due(now) {
            if self.stopped.is_some() {
                return;
            }
            let at = due.max(self.world.world().now());
            if let Err(failure) = self.advance(at) {
                let _ = self.refusal(failure);
                continue;
            }
            let Ok(observer) = self.observer_of(&seat) else {
                continue;
            };
            let context = PerceptionContext::new(self.world.world(), observer, at, &self.recent);
            let observation = self.perception.observe(&context);
            let Some(slot) = self.table.hosted_mut(&seat) else {
                continue;
            };
            let Some(request) = slot.decide(&observation, now) else {
                continue;
            };
            let answer = match self.submit_at(observer, request, at) {
                Ok((submitted, _)) => HostedAnswer::Answered(submitted),
                Err(refusal) => HostedAnswer::Refused(refusal.code()),
            };
            if let Some(slot) = self.table.hosted_mut(&seat) {
                slot.answered(&answer);
            }
        }
    }

    /// Judges each new fact for every connection as it is recorded (`ARC-43`), then keeps the
    /// newest facts for perception and forgets the rest.
    fn remember(&mut self, events: Vec<EventEnvelope>) {
        self.learn(&events);
        self.recent.extend(events);
        let limit = self.config.recent_events;
        if self.recent.len() > limit {
            self.recent.drain(..self.recent.len() - limit);
        }
    }

    /// What the world is, as a status answer or a welcome states it.
    fn summary(&self) -> WorldSummary {
        let systems = self.world.world().systems();
        WorldSummary {
            protocol: PROTOCOL_VERSION,
            instance: self.instance,
            at: self.clock.now(),
            time_scale: self.clock.scale().get(),
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
            // Connections only: an in-server controller is not a client (`PROTOCOL.md` §5.7).
            clients: self.subscribers.len(),
            observations_dropped: self.dropped,
            events_dropped: self.events_dropped,
            faults: self.faults,
            revision: self.world.revision(),
        }
    }
}

/// The instant the server's first controllers are bound at (`F-13`): every line heard at or before
/// it was said to whoever drove the Person before this process, and is not theirs to answer.
///
/// For a world resumed with history, that is the world's own instant — its last input. A world that
/// holds nothing but its genesis has heard nothing: its controllers are bound a second before it, so
/// that a line said in the first wall second of hosting, which the host's clock still stamps with the
/// genesis instant, is answered (step-12 D-SB3).
fn first_binding(world: &Hosted, epoch: WorldTime) -> WorldTime {
    let only_genesis = match world {
        Hosted::Ephemeral(_) => true,
        Hosted::Persisted(world) => world.revision() == WorldRevision::GENESIS,
    };
    if only_genesis {
        WorldTime::from_seconds(epoch.seconds() - 1)
    } else {
        epoch
    }
}
