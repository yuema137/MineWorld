//! The two planes `docs/NETWORKING.md` §3 names, and the one binary they both live in.
//!
//! ```text
//! GET /health   liveness. Answered by the transport, so it stays true while a world is busy.
//! GET /status   what the world is. Answered by the WORLD, so it is evidence that the world runs.
//! GET /ws       the live connection: observations out, intents in.
//! ```
//!
//! One binary for localhost, LAN and cloud alike (`NETWORKING.md` §1): the address it listens on is
//! configuration, and there is no second code path anywhere in this crate for a single player.

use std::net::SocketAddr;

use axum::Json;
use axum::Router;
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{any, get};
use serde::Serialize;
use tokio::net::TcpListener;

use crate::host::WorldHost;
use crate::protocol::{PROTOCOL_VERSION, WorldSummary};
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

/// The routes, over one hosted world.
pub fn router(host: WorldHost) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/ws", any(upgrade))
        .with_state(host)
}

/// Serves until the process is stopped.
pub async fn serve(listener: TcpListener, host: WorldHost) -> std::io::Result<()> {
    axum::serve(listener, router(host)).await
}

/// Serves until `shutdown` completes, and then stops accepting connections.
pub async fn serve_with_shutdown<S>(
    listener: TcpListener,
    host: WorldHost,
    shutdown: S,
) -> std::io::Result<()>
where
    S: Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, router(host))
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
async fn status(State(host): State<WorldHost>) -> Result<Json<WorldSummary>, StatusCode> {
    host.status()
        .await
        .map(Json)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

async fn upgrade(upgrade: WebSocketUpgrade, State(host): State<WorldHost>) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| session::run(socket, host))
}
