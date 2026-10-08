//! The MineWorld world server: one authoritative world, and the clients connected to it.
//!
//! ```text
//! HTTP        /health, /status            control and status
//! WebSocket   /ws                         per-observer observations out, ActionRequests in
//! ```
//!
//! `docs/NETWORKING.md` is the specification this crate implements, and `server/PROTOCOL.md` is the
//! frame-level contract a client is written against. Four of this crate's properties are not
//! negotiable, and each is placed where it cannot be forgotten:
//!
//! ```text
//! INV-13   one perception call per connected observer, and no frame that widens a scope
//!          — see runtime::WorldRuntime::sweep and protocol::ClientFrame
//! INV-6    the server allocates every ActionId and every instant; a client has neither
//!          — see runtime::ActionIds and host::WorldHost::submit
//! §2       a client may act, and may never assert. A frame that states a fact is a protocol
//!          violation — see protocol::ClientFrame::decode and RefusalCode::UnknownFrame
//! §1       one binary for localhost and network alike; there is no single-player path here
//!          — see app::serve
//! ```
//!
//! # The shape of it
//!
//! ```text
//! admission   who may join: the invite, the offered invite, the nickname
//! app         the router: /health, /status, /ws
//! parity      AC-13's comparison: what it means for two clients to ask the same thing
//! session     one client's conversation: join, then observations out and requests in
//! host        WorldHost — the handle, the seats, and the thread a World must live on
//! runtime     the world thread: the clock, the request allocator, the subscribers
//! perception  the one seam a hosted world provides: what each observer perceives
//! protocol    the frames, and nothing else a client may say
//! ```
//!
//! The kernel never sees an HTTP or WebSocket type, and nothing in this crate decides a world rule:
//! admissibility is the kernel's and the systems', and perception is a perception system's
//! (`INV-14`, `ENGINEERING_RULES.md` §8).
//!
//! # Hosting a world
//!
//! ```no_run
//! use mineworld_server::{HostConfig, HostedWorld, SeatRoster, WorldHost, app};
//! use mineworld_kernel::World;
//! use mineworld_contracts::EntityKey;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // The world is built inside its own thread, because a World is not Send.
//! let host = WorldHost::spawn(HostConfig::default(), || {
//!     let mut world = World::new();
//!     // world.install(...) every System Pack this world is composed of, then:
//!     Ok(HostedWorld::new(world).seating(SeatRoster::empty()))
//! })
//! .await?;
//!
//! let (listener, address) = app::bind("127.0.0.1:7878".parse()?).await?;
//! println!("listening on ws://{address}/ws");
//! app::serve(listener, host).await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod admission;
pub mod app;
pub mod host;
pub mod parity;
pub mod perception;
pub mod protocol;
mod runtime;
mod session;

pub use admission::{
    Admission, AdmissionError, InviteToken, Nickname, OfferedInvite, UNAUTHORIZED_DELAY,
    Unauthorized,
};
pub use host::{
    HostConfig, HostError, HostedWorld, Perceived, SeatRoster, Seated, Submitted, SubscriptionId,
    WorldHost,
};
pub use mineworld_persistence::WorldRevision;
pub use parity::{RequestField, SemanticCore, differing_fields, semantic_core};
pub use perception::{PerceivesNothing, Perception, PerceptionContext};
pub use protocol::{
    ClientFrame, CorrelationToken, PROTOCOL_VERSION, ProtocolError, Refusal, RefusalCode,
    ServerFrame, SystemSummary, WireObservation, WirePayload, WorldInstanceId, WorldSummary,
};
