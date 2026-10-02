//! Hosting a world: the handle the transport holds, and the thread the world lives on.
//!
//! # Why the world has a thread of its own
//!
//! `kernel/src/registry.rs` holds each installed system as a `Box<dyn DynSystem>` with no `Send`
//! bound, so a [`World`] cannot be moved between threads at all. That is not an obstacle to work
//! around; it is the architecture stated for us:
//!
//! ```text
//! HTTP handler ─┐
//! WS session  ──┼──► commands ──►  world thread   owns the World, the clock, the allocator,
//! WS session  ──┘   (bounded)      │              the perception seam and the subscriber list
//!                                  ▼
//!                   observations ──┴──►  one bounded channel per connected client
//! ```
//!
//! The world is **built inside** its own thread, from a `Send` closure, so neither the world nor
//! its systems nor its perception implementation ever crosses a thread boundary. What crosses is
//! plain data: commands in, `Observation`s and `ActionResult`s out.
//!
//! Three properties follow, and they are the ones `NETWORKING.md` §4 requires:
//!
//! ```text
//! the world is never blocked by a client   observations are delivered with try_send on a bounded
//!                                          channel; a client that stops reading loses frames and
//!                                          nothing else waits for it
//! dispatch and streaming cannot deadlock   there is no lock between them, because there is no
//!                                          lock: one thread owns the world and every access is a
//!                                          message
//! a client is never blocked by the world   a session awaits a oneshot reply for its own request
//!                                          and nothing else
//! ```
//!
//! The spike held its world in an `Arc<Mutex<_>>` locked across `.await` points, which is exactly
//! what a second client turns into a coupling between one client's socket and another client's
//! dispatch. It is not reproduced here.
//!
//! # What this module does not own
//!
//! It does not own history: recorded events are kept in a small window for perception to read, and
//! the durable log belongs to the world's save, when it has one (`mineworld-persistence`, `ARC-25`;
//! `NETWORKING.md` §10 — networking never owns world state). It does not
//! own a scheduler: the world's clock and queue are the kernel's (S4), and the world thread only
//! advances them to the host's instant, on each tick and before each request. And it owns no world
//! rule: what an observer perceives comes from the
//! [`Perception`] seam, and whether an action is admissible comes from the kernel.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use mineworld_contracts::{
    ActionId, ActionRequest, ActionResult, EntityId, EntityKey, EventEnvelope, WorldTime,
};
use mineworld_kernel::{KernelError, World};
use mineworld_persistence::{PersistentWorld, WorldRevision};
use tokio::sync::{mpsc, oneshot};

use crate::perception::{PerceivesNothing, Perception};
use crate::protocol::{Refusal, RefusalCode, WireObservation, WorldSummary};
use crate::runtime::WorldRuntime;

/// The first request identity a server allocates for a world with no history of requests.
///
/// One, not zero, for the reason `kernel/src/dispatch.rs` gives for event identity: zero is what an
/// absent, defaulted or truncated number looks like in a serialized record.
pub(crate) const FIRST_ACTION_ID: u64 = 1;

/// How many commands may be in flight before a submitter waits. Generous: a command is a request
/// or a sweep, and the world answers each in microseconds.
const COMMAND_BACKLOG: usize = 256;

/// A world, its perception, and the seats a client may occupy: everything the host needs and
/// nothing the transport may reach into.
///
/// Assembled by the closure [`WorldHost::spawn`] runs on the world's own thread, which is why this
/// type carries a [`World`] without being `Send`.
pub struct HostedWorld {
    pub(crate) world: Hosted,
    pub(crate) perception: Box<dyn Perception>,
    pub(crate) seats: SeatRoster,
    /// Where the request allocator starts: past every `ActionId` the world's journal already holds,
    /// so that a resumed world never issues one twice (step-06 §2.4, F-7).
    pub(crate) first_action: u64,
    /// The facts perception may look back over at the start — the tail of a save's log, so that a
    /// restarted world does not silently forget what just happened (PD-13).
    pub(crate) recent: Vec<EventEnvelope>,
}

/// The world a host holds: one that lives only as long as its process, or one with a save.
pub(crate) enum Hosted {
    /// Not persisted: its state ends with the process.
    Ephemeral(World),
    /// Persisted: every input committed to its save before anybody is told (S5, `ARC-25`).
    Persisted(PersistentWorld),
}

impl HostedWorld {
    /// A world with no seats and no perception: legal, and what an unauthored world is.
    pub fn new(world: World) -> Self {
        Self {
            world: Hosted::Ephemeral(world),
            perception: Box::new(PerceivesNothing),
            seats: SeatRoster::empty(),
            first_action: FIRST_ACTION_ID,
            recent: Vec::new(),
        }
    }

    /// A world with a save: the server commits every request and every advance that fired something
    /// before it answers or sweeps, tells clients the persisted revision, and resumes the world's
    /// instance, request identities, clock and recent facts from the save rather than from zero.
    ///
    /// `recent` is how many of the log's newest facts perception starts with — normally
    /// [`HostConfig::recent_events`].
    pub fn persisted(world: PersistentWorld, recent: usize) -> Result<Self, HostError> {
        let first_action = world
            .highest_action_id()
            .map_err(HostError::build)?
            .map_or(FIRST_ACTION_ID, |highest| highest.saturating_add(1));
        let recent = world.recent_facts(recent).map_err(HostError::build)?;
        Ok(Self {
            world: Hosted::Persisted(world),
            perception: Box::new(PerceivesNothing),
            seats: SeatRoster::empty(),
            first_action,
            recent,
        })
    }

    /// Declares the seats a client may ask for.
    #[must_use]
    pub fn seating(mut self, seats: SeatRoster) -> Self {
        self.seats = seats;
        self
    }

    /// Declares what this world's observers perceive.
    #[must_use]
    pub fn perceiving(mut self, perception: impl Perception) -> Self {
        self.perception = Box::new(perception);
        self
    }
}

/// The seats of a world: which entities a client may connect *as*.
///
/// The roster is the world host's, never the client's, and that is what bounds identity: a client
/// names a seat and the server resolves it, so there is no frame in which a client names an
/// `EntityId` — not its own and not anyone else's. A seat is named by the authoring key of the
/// entity it belongs to, because that is the name a World Pack already gives it (S7).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeatRoster(BTreeSet<EntityKey>);

impl SeatRoster {
    /// A world nobody can connect into. Valid: the simulation runs headless with no client
    /// (`ENGINEERING_STANDARDS.md` §22).
    pub const fn empty() -> Self {
        Self(BTreeSet::new())
    }

    /// The seats a client may ask for.
    pub fn new(seats: impl IntoIterator<Item = EntityKey>) -> Self {
        Self(seats.into_iter().collect())
    }

    /// Whether this seat exists.
    pub fn contains(&self, seat: &EntityKey) -> bool {
        self.0.contains(seat)
    }

    /// Every seat, in key order.
    pub fn iter(&self) -> impl Iterator<Item = &EntityKey> {
        self.0.iter()
    }
}

/// How a world is hosted: the clock's starting point, the stream's cadence, and two bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConfig {
    /// The instant the world's clock starts from.
    pub epoch: WorldTime,
    /// How often each connected client is sent a fresh observation.
    ///
    /// 10 Hz by default, as the spike ran. `NETWORKING.md` §4 is explicit that this is a life
    /// simulation rather than a competitive shooter, and that the rate of replication is not the
    /// rate of the simulation nor of rendering.
    pub observation_interval: Duration,
    /// How many observations may be queued for one client before the oldest are lost.
    ///
    /// Small on purpose. A slow client should receive the *current* world late, not a backlog of
    /// stale ones, and the world must never wait for it.
    pub observation_backlog: usize,
    /// How many recorded events perception may look back over.
    ///
    /// A window, not a log: the durable event log is S5's.
    pub recent_events: usize,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            epoch: WorldTime::EPOCH,
            observation_interval: Duration::from_millis(100),
            observation_backlog: 8,
            recent_events: 64,
        }
    }
}

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
    /// What the observer perceives.
    pub observation: WireObservation,
}

/// A seated connection: which observer it is, and its own stream of observations.
#[derive(Debug)]
pub struct Seated {
    seat: EntityKey,
    observer: EntityId,
    world: WorldSummary,
    subscription: SubscriptionId,
    observations: mpsc::Receiver<Perceived>,
}

impl Seated {
    pub(crate) const fn new(
        seat: EntityKey,
        observer: EntityId,
        world: WorldSummary,
        subscription: SubscriptionId,
        observations: mpsc::Receiver<Perceived>,
    ) -> Self {
        Self {
            seat,
            observer,
            world,
            subscription,
            observations,
        }
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

    /// This connection's own observation stream.
    pub const fn observations(&mut self) -> &mut mpsc::Receiver<Perceived> {
        &mut self.observations
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

/// What the transport asks of the world thread.
pub(crate) enum Command {
    /// What the world is: time, entities, systems, seats, clients.
    Status(oneshot::Sender<WorldSummary>),
    /// Occupy a seat, and begin receiving that observer's observations.
    Join {
        seat: EntityKey,
        reply: oneshot::Sender<Result<Seated, Refusal>>,
    },
    /// Release a subscription. Fire and forget: a connection that has already died cannot wait.
    Leave(SubscriptionId),
    /// Dispatch one request as one observer.
    Submit {
        observer: EntityId,
        request: Box<ActionRequest>,
        reply: oneshot::Sender<Result<Submitted, Refusal>>,
    },
    /// Send every connected client a fresh observation.
    Sweep,
    /// Stop the world thread.
    Shutdown,
}

/// The handle every part of the transport holds. Cheap to clone; the world is elsewhere.
#[derive(Clone)]
pub struct WorldHost {
    commands: mpsc::Sender<Command>,
}

impl WorldHost {
    /// Starts a world on a thread of its own and returns the handle to it.
    ///
    /// The world is assembled by `build`, **inside** that thread: a [`World`] is not `Send`, so
    /// this is the only shape in which a hosted world can exist at all. `build` returning an error
    /// is a failed world assembly, which `kernel/src/world.rs` documents as unrecoverable — the
    /// caller reports it rather than retrying, and this function surfaces it before the handle
    /// exists.
    ///
    /// Also starts the observation ticker: one task that asks the world to sweep at the configured
    /// cadence. It uses `try_send`, so a sweep is skipped rather than queued behind a backlog —
    /// the next one carries the current world anyway.
    pub async fn spawn<B>(config: HostConfig, build: B) -> Result<Self, HostError>
    where
        B: FnOnce() -> Result<HostedWorld, HostError> + Send + 'static,
    {
        let (commands, receiver) = mpsc::channel(COMMAND_BACKLOG);
        let (ready, started) = oneshot::channel();
        let runtime_config = Arc::new(config);
        let thread_config = Arc::clone(&runtime_config);

        std::thread::Builder::new()
            .name("mineworld-world".to_owned())
            .spawn(move || match build() {
                Ok(hosted) => {
                    if ready.send(Ok(())).is_err() {
                        return;
                    }
                    WorldRuntime::new(hosted, thread_config).run(receiver);
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                }
            })?;

        started.await.map_err(|_| HostError::WorldStopped)??;

        let ticker = commands.clone();
        let interval = runtime_config.observation_interval;
        tokio::spawn(async move {
            let mut clock = tokio::time::interval(interval);
            loop {
                clock.tick().await;
                match ticker.try_send(Command::Sweep) {
                    Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {}
                    Err(mpsc::error::TrySendError::Closed(_)) => break,
                }
            }
        });

        Ok(Self { commands })
    }

    /// What the world is, answered by the world itself rather than by a cache in the transport.
    pub async fn status(&self) -> Result<WorldSummary, HostError> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Status(reply))
            .await
            .map_err(|_| HostError::WorldStopped)?;
        answer.await.map_err(|_| HostError::WorldStopped)
    }

    /// Occupies a seat, or the refusal to send back.
    ///
    /// The observer comes back from the world; it is never something a caller supplies.
    pub async fn join(&self, seat: EntityKey) -> Result<Seated, Refusal> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Join { seat, reply })
            .await
            .map_err(|_| Refusal::new(RefusalCode::WorldStopped))?;
        answer
            .await
            .map_err(|_| Refusal::new(RefusalCode::WorldStopped))?
    }

    /// Submits one request as one observer: the world allocates the identity and the instant, and
    /// dispatches.
    pub async fn submit(
        &self,
        observer: EntityId,
        request: ActionRequest,
    ) -> Result<Submitted, Refusal> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Submit {
                observer,
                request: Box::new(request),
                reply,
            })
            .await
            .map_err(|_| Refusal::new(RefusalCode::WorldStopped))?;
        answer
            .await
            .map_err(|_| Refusal::new(RefusalCode::WorldStopped))?
    }

    /// Releases a subscription whose connection has ended.
    ///
    /// Fire and forget, and deliberately: a client that vanished must not be able to make the
    /// transport wait, and the world drops a subscriber whose channel has closed on its next sweep
    /// in any case.
    pub fn leave(&self, subscription: SubscriptionId) {
        let _ = self.commands.try_send(Command::Leave(subscription));
    }

    /// Stops the world thread. A world that is not persisted goes with it; a persisted one is
    /// checkpointed first, when its clock still stands at its last revision, and is otherwise already
    /// whole on disk — every revision was committed before it was told to anybody.
    pub async fn shutdown(&self) {
        let _ = self.commands.send(Command::Shutdown).await;
    }
}

/// What can go wrong in hosting, as distinct from what a client can get wrong.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    /// The world could not be assembled. `kernel/src/world.rs`: a failed installation is a failed
    /// world assembly, not a recoverable operation.
    #[error("the world could not be assembled: {0}")]
    Assembly(#[from] KernelError),
    /// Whatever describes this world could not be turned into one.
    ///
    /// The builder [`WorldHost::spawn`] runs is the caller's, so its failure is the caller's too, and
    /// this variant carries it without naming it: a World Pack that does not load, a snapshot that
    /// does not restore, a fixture that could not be written. The server learns nothing about what a
    /// World Pack is by carrying one — which is the point, because a transport that knew would be the
    /// one-way dependency rule inverted.
    #[error("the world could not be built: {0}")]
    Build(Box<dyn std::error::Error + Send + Sync>),
    /// The world's thread could not be started.
    #[error("the world thread could not be started: {0}")]
    Thread(#[from] std::io::Error),
    /// The world is no longer running.
    #[error("the world is not running")]
    WorldStopped,
}

impl HostError {
    /// Wraps whatever a world builder failed with.
    pub fn build(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Build(Box::new(error))
    }
}
