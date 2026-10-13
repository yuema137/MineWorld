//! The two planes `docs/NETWORKING.md` §3 names, and the one binary they both live in.
//!
//! ```text
//! GET /health   liveness. Answered by the transport, so it stays true while a world is busy.
//! GET /status   what the world is. Answered by the WORLD, so it is evidence that the world runs.
//! GET /ws       the live connection: observations out, intents in.
//! /admin/…      the admin surface (crate::admin), only when an admin token was given.
//! ```
//!
//! One binary for localhost, LAN and cloud alike (`NETWORKING.md` §1): the address it listens on is
//! configuration, and there is no second code path anywhere in this crate for a single player.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Json;
use axum::Router;
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{any, get};
use serde::Serialize;
use tokio::net::TcpListener;

use crate::admin::{self, registry::Registry};
use crate::admission::{AdminToken, Admission};
use crate::host::WorldHost;
use crate::protocol::{PROTOCOL_VERSION, SessionId, WorldSummary};
use crate::session;

/// What `GET /health` answers: that this process is up, and which protocol it speaks.
///
/// Deliberately does not consult the world. Liveness of the process and liveness of the world are
/// different questions, and `GET /status` is the second one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Health {
    /// Always `"ok"` — the answer is the fact that it arrived.
    pub status: &'static str,
    /// Which revision of the wire protocol this server speaks.
    pub protocol: u32,
}

/// What every route is served over: the world, who may join it, and the next connection's number.
///
/// Admission belongs to the transport, never to the world: the world thread is asked for a seat
/// only after a connection has passed it (`PROTOCOL.md` §4.1).
#[derive(Clone)]
struct Hosting {
    host: WorldHost,
    admission: Arc<Admission>,
    sessions: Arc<AtomicU64>,
    registry: Arc<Registry>,
}

/// Who may do what on this server: the invite every player presents, and — only if the operator
/// gave one — the admin token that opens `/admin` (`PROTOCOL.md` §11).
///
/// An [`Admission`] alone converts into an `Access` with no admin surface, so every existing call
/// that passes one keeps its meaning.
#[derive(Debug, Clone)]
pub struct Access {
    /// The invite check.
    pub admission: Admission,
    /// The admin token, or `None` for no admin surface at all.
    pub admin: Option<AdminToken>,
}

impl From<Admission> for Access {
    fn from(admission: Admission) -> Self {
        Self {
            admission,
            admin: None,
        }
    }
}

/// The routes, over one hosted world, admitting the holders of one invite — and, with an admin
/// token, the admin surface. Without one, no `/admin` path exists: each answers `404` as any
/// unknown path does.
pub fn router(host: WorldHost, access: impl Into<Access>) -> Router {
    let Access { admission, admin } = access.into();
    let registry = Arc::new(Registry::default());
    let public = Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/ws", any(upgrade))
        .with_state(Hosting {
            host: host.clone(),
            admission: Arc::new(admission),
            sessions: Arc::new(AtomicU64::new(1)),
            registry: Arc::clone(&registry),
        });
    match admin {
        Some(token) => public.merge(admin::routes(admin::Admin {
            host,
            registry,
            token: Arc::new(token),
        })),
        None => public,
    }
}

/// Serves until the process is stopped.
pub async fn serve(
    listener: TcpListener,
    host: WorldHost,
    access: impl Into<Access>,
) -> std::io::Result<()> {
    axum::serve(listener, router(host, access)).await
}

/// Serves until `shutdown` completes, and then stops accepting connections.
pub async fn serve_with_shutdown<S>(
    listener: TcpListener,
    host: WorldHost,
    access: impl Into<Access>,
    shutdown: S,
) -> std::io::Result<()>
where
    S: Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, router(host, access))
        .with_graceful_shutdown(shutdown)
        .await
}

/// Binds an address and reports what was actually bound, which is what an ephemeral port needs.
pub async fn bind(address: SocketAddr) -> std::io::Result<(TcpListener, SocketAddr)> {
    let listener = TcpListener::bind(address).await?;
    let bound = listener.local_addr()?;
    Ok((listener, bound))
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        protocol: PROTOCOL_VERSION,
    })
}

/// The world's own answer about itself.
///
/// `503` when the world thread is gone, because a server whose world has stopped is not a server
/// that should report itself healthy on this route.
async fn status(State(hosting): State<Hosting>) -> Result<Json<WorldSummary>, StatusCode> {
    hosting
        .host
        .status()
        .await
        .map(Json)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

async fn upgrade(upgrade: WebSocketUpgrade, State(hosting): State<Hosting>) -> impl IntoResponse {
    let connection = session::Connection {
        session: SessionId::new(hosting.sessions.fetch_add(1, Ordering::Relaxed)),
        host: hosting.host,
        admission: hosting.admission,
        registry: hosting.registry,
    };
    upgrade.on_upgrade(move |socket| session::run(socket, connection))
}
