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

mod handles;

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use mineworld_contracts::{ActionRequest, EntityId, EntityKey, EventEnvelope, WorldTime};
use mineworld_kernel::{KernelError, World};
use mineworld_persistence::PersistentWorld;
use tokio::sync::{mpsc, oneshot};

use crate::hosted::{HostedController, HostedFactory};
use crate::perception::{PerceivesNothing, Perception};
use crate::protocol::{Refusal, RefusalCode, SessionId, WorldSummary};
use crate::runtime::{ControlAnswer, ControlCommand, WorldRuntime};
use crate::seats::{Departure, JoinRequest};

pub(crate) use handles::{Binding, SubscriptionIdSource};
pub use handles::{Perceived, Seated, Streams, Submitted, SubscriptionId};

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
    /// The in-server controller each hosted seat returns to (`PROTOCOL.md` §4.2).
    pub(crate) hosted: BTreeMap<EntityKey, HostedFactory>,
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
            hosted: BTreeMap::new(),
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
            hosted: BTreeMap::new(),
        })
    }

    /// Drives `seat` with an in-server controller whenever no person does (`docs/DECISIONS.md`
    /// `ARC-42`). `factory` builds the controller, bound at the instant it is given: when the server
    /// starts, and every time the seat returns to it. A seat the roster does not offer is never
    /// driven.
    #[must_use]
    pub fn hosting(
        mut self,
        seat: EntityKey,
        factory: impl Fn(WorldTime) -> Box<dyn HostedController> + 'static,
    ) -> Self {
        self.hosted.insert(seat, Box::new(factory));
        self
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
    /// How long a seat whose connection dropped is held for its resume, in wall time
    /// (`PROTOCOL.md` §4.2). Zero holds none.
    pub hold: Duration,
    /// How many world seconds pass per wall second.
    pub time_scale: NonZeroU32,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            epoch: WorldTime::EPOCH,
            observation_interval: Duration::from_millis(100),
            observation_backlog: 8,
            recent_events: 64,
            hold: Duration::from_secs(30),
            time_scale: NonZeroU32::MIN,
        }
    }
}

/// What the transport asks of the world thread.
pub(crate) enum Command {
    /// What the world is: time, entities, systems, seats, clients.
    Status(oneshot::Sender<WorldSummary>),
    /// Occupy a seat, and begin receiving that observer's observations.
    Join {
        request: JoinRequest,
        reply: oneshot::Sender<Result<Seated, Refusal>>,
    },
    /// Release a subscription, saying how its connection ended. Fire and forget: a connection that
    /// has already died cannot wait.
    Leave(SubscriptionId, Departure),
    /// Dispatch one request as one observer.
    Submit {
        observer: EntityId,
        request: Box<ActionRequest>,
        reply: oneshot::Sender<Result<Submitted, Refusal>>,
    },
    /// Send every connected client a fresh observation.
    Sweep,
    /// One admin command (`PROTOCOL.md` §11): host state only.
    Control {
        command: ControlCommand,
        reply: oneshot::Sender<ControlAnswer>,
    },
    /// Stop the world thread, answering once it has checkpointed and reported.
    Shutdown(oneshot::Sender<()>),
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

    /// Occupies a free or hosted seat with a plain join — no takeover, no resume — or the refusal to
    /// send back. For an in-process caller that is not a connection: it is labelled session `0`,
    /// which no connection is ever given.
    pub async fn join(&self, seat: EntityKey) -> Result<Seated, Refusal> {
        self.join_with(JoinRequest::plain(seat, SessionId::new(0)))
            .await
    }

    /// Occupies a seat as `request` asks, or the refusal to send back (`PROTOCOL.md` §4.2's rules
    /// decide).
    ///
    /// The observer comes back from the world; it is never something a caller supplies.
    pub async fn join_with(&self, request: JoinRequest) -> Result<Seated, Refusal> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Join { request, reply })
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

    /// Releases a subscription whose connection has ended: its seat returns to its default, or is
    /// held when the connection dropped without `leave` (`PROTOCOL.md` §4.2).
    ///
    /// Fire and forget, and deliberately: a client that vanished must not be able to make the
    /// transport wait, and the world drops a subscriber whose channel has closed on its next sweep
    /// in any case — as a drop.
    pub fn leave(&self, subscription: SubscriptionId, departure: Departure) {
        let _ = self
            .commands
            .try_send(Command::Leave(subscription, departure));
    }

    /// Asks the world thread one admin command and waits for its answer. Called only by the admin
    /// surface, after its permission check (`PROTOCOL.md` §11).
    pub(crate) async fn control(
        &self,
        command: ControlCommand,
    ) -> Result<ControlAnswer, HostError> {
        let (reply, answer) = oneshot::channel();
        self.commands
            .send(Command::Control { command, reply })
            .await
            .map_err(|_| HostError::WorldStopped)?;
        answer.await.map_err(|_| HostError::WorldStopped)
    }

    /// Stops the world thread and returns once it has. A world that is not persisted goes with it;
    /// a persisted one is checkpointed first, when its clock still stands at its last revision, and
    /// is otherwise already whole on disk — every revision was committed before it was told to
    /// anybody. The world thread prints its tick statistics as it stops (`ARC-42`).
    pub async fn shutdown(&self) {
        let (reply, done) = oneshot::channel();
        if self.commands.send(Command::Shutdown(reply)).await.is_ok() {
            let _ = done.await;
        }
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
