//! The admin surface: HTTP under `/admin`, behind one bearer token (`PROTOCOL.md` §11,
//! `docs/DECISIONS.md` `ARC-44`).
//!
//! ```text
//! GET  /admin/sessions                  every seated connection
//! GET  /admin/seats                     every seat's binding
//! POST /admin/sessions/{session}/kick   closing {kicked}; the seat returns to its default, no hold
//! POST /admin/seats/{seat}/release      whoever holds it is closed; the seat returns to its default
//! GET  /admin/clock                     { at, time_scale, paused }
//! POST /admin/clock                     { "paused": bool } — pause and resume
//! ```
//!
//! Mounted only when the server has an admin token ([`crate::app::Access`]); without one no `/admin`
//! path exists. Every request passes [`permitted`] first: a failure of any kind is answered `401`
//! no sooner than [`UNAUTHORIZED_DELAY`] after the request arrived, slept in that request's own task
//! and never on the world's thread. A handler holds no binding and touches no `World`: it reads the
//! transport's [`registry`] or sends one command to the world thread, whose answer is host state
//! (`ARC-40`). No route and no field names an instant.

pub(crate) mod registry;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use mineworld_contracts::{EntityId, EntityKey};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::time::Instant;

use crate::admission::{AdminToken, Nickname, UNAUTHORIZED_DELAY};
use crate::host::WorldHost;
use crate::protocol::SessionId;
use crate::runtime::{ControlAnswer, ControlCommand};
use crate::seats::SeatReport;
use registry::Registry;

/// What the admin routes are served over.
#[derive(Clone)]
pub(crate) struct Admin {
    pub(crate) host: WorldHost,
    pub(crate) registry: Arc<Registry>,
    pub(crate) token: Arc<AdminToken>,
}

/// The `/admin` routes, every one behind the bearer check.
pub(crate) fn routes(admin: Admin) -> Router {
    Router::new()
        .route("/admin/sessions", get(sessions))
        .route("/admin/seats", get(seats))
        .route("/admin/sessions/{session}/kick", post(kick))
        .route("/admin/seats/{seat}/release", post(release))
        .route("/admin/clock", get(clock).post(set_clock))
        .route_layer(middleware::from_fn_with_state(admin.clone(), permitted))
        .with_state(admin)
}

/// The bearer check (step-12 SD-D3): `Authorization: Bearer <token>`, compared in constant time.
/// Anything else — no header, another scheme, a wrong token, trailing whitespace — waits until
/// [`UNAUTHORIZED_DELAY`] after the request arrived, then is answered `401`. A token in the query
/// string is never read.
async fn permitted(State(admin): State<Admin>, request: Request, next: Next) -> Response {
    let arrived = Instant::now();
    if presented(request.headers()).is_some_and(|offered| admin.token.admits(offered)) {
        return next.run(request).await;
    }
    tokio::time::sleep_until(arrived + UNAUTHORIZED_DELAY).await;
    error(StatusCode::UNAUTHORIZED, "unauthorized", None)
}

/// The bytes after `Bearer ` in the one `Authorization` header, exactly as sent.
fn presented(headers: &HeaderMap) -> Option<&[u8]> {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let value = values.next()?;
    if values.next().is_some() {
        return None;
    }
    value.as_bytes().strip_prefix(b"Bearer ")
}

/// `{ "error": <code>, "detail": … }` with its status.
fn error(status: StatusCode, code: &str, detail: Option<String>) -> Response {
    let body = match detail {
        Some(detail) => json!({ "error": code, "detail": detail }),
        None => json!({ "error": code }),
    };
    (status, Json(body)).into_response()
}

fn malformed(detail: impl Into<String>) -> Response {
    error(StatusCode::BAD_REQUEST, "malformed", Some(detail.into()))
}

/// Asks the world thread, or answers `503 world_stopped`.
async fn ask(admin: &Admin, command: ControlCommand) -> Result<ControlAnswer, Response> {
    admin.host.control(command).await.map_err(|_| {
        error(
            StatusCode::SERVICE_UNAVAILABLE,
            "world_stopped",
            Some("the world is no longer running".to_owned()),
        )
    })
}

/// A world answer a route did not ask for: a defect in the server, said rather than hidden.
fn unexpected(answer: &ControlAnswer) -> Response {
    error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        Some(format!("the world answered {answer:?}")),
    )
}

/// One seated connection on `GET /admin/sessions`.
#[derive(Serialize)]
struct SessionView {
    session: SessionId,
    nickname: Nickname,
    seat: EntityKey,
    observer: EntityId,
    connected_at: u64,
    seq: u64,
    observations_dropped: u64,
}

/// Every seated connection: the transport's registry joined with the world's per-session counts. A
/// session the world no longer seats — kicked, released, ended — is not listed even if its task has
/// not yet finished closing.
async fn sessions(State(admin): State<Admin>) -> Response {
    let counts = match ask(&admin, ControlCommand::Sessions).await {
        Ok(ControlAnswer::Sessions(counts)) => counts,
        Ok(other) => return unexpected(&other),
        Err(response) => return response,
    };
    let dropped: BTreeMap<SessionId, u64> = counts
        .into_iter()
        .map(|count| (count.session, count.observations_dropped))
        .collect();
    let sessions: Vec<SessionView> = admin
        .registry
        .listed()
        .into_iter()
        .filter_map(|listed| {
            let observations_dropped = *dropped.get(&listed.session)?;
            Some(SessionView {
                session: listed.session,
                nickname: listed.joined.nickname,
                seat: listed.joined.seat,
                observer: listed.joined.observer,
                connected_at: listed.joined.connected_at,
                seq: listed.seq,
                observations_dropped,
            })
        })
        .collect();
    Json(json!({ "sessions": sessions })).into_response()
}

async fn seats(State(admin): State<Admin>) -> Response {
    match ask(&admin, ControlCommand::Seats).await {
        Ok(ControlAnswer::Seats(seats)) => Json(json!({ "seats": seats })).into_response(),
        Ok(other) => unexpected(&other),
        Err(response) => response,
    }
}

async fn kick(State(admin): State<Admin>, Path(session): Path<String>) -> Response {
    let Ok(session) = SessionId::try_from(session) else {
        return malformed("a session is a decimal integer");
    };
    match ask(&admin, ControlCommand::Kick(session)).await {
        Ok(ControlAnswer::Kicked(SeatReport { seat, state })) => {
            let mut body = json!({ "session": session, "seat": seat });
            merge_state(&mut body, state);
            Json(body).into_response()
        }
        Ok(ControlAnswer::UnknownSession) => error(
            StatusCode::NOT_FOUND,
            "unknown_session",
            Some("no seated connection has that session".to_owned()),
        ),
        Ok(other) => unexpected(&other),
        Err(response) => response,
    }
}

async fn release(State(admin): State<Admin>, Path(seat): Path<String>) -> Response {
    let Ok(seat) = EntityKey::new(&seat) else {
        return malformed("a seat is named by an entity key");
    };
    match ask(&admin, ControlCommand::Release(seat)).await {
        Ok(ControlAnswer::Released { released, now }) => {
            let mut body = json!({ "seat": now.seat, "released": released });
            merge_state(&mut body, now.state);
            Json(body).into_response()
        }
        Ok(ControlAnswer::UnknownSeat) => error(
            StatusCode::NOT_FOUND,
            "unknown_seat",
            Some("this world offers no such seat".to_owned()),
        ),
        Ok(other) => unexpected(&other),
        Err(response) => response,
    }
}

/// Adds a seat state's `state` (and its `session` or `seconds_left`) to a JSON object.
fn merge_state(body: &mut Value, state: crate::seats::SeatState) {
    if let (Value::Object(body), Ok(Value::Object(fields))) = (body, serde_json::to_value(state)) {
        body.extend(fields);
    }
}

async fn clock(State(admin): State<Admin>) -> Response {
    match ask(&admin, ControlCommand::Clock).await {
        Ok(ControlAnswer::Clock(state)) => Json(state).into_response(),
        Ok(other) => unexpected(&other),
        Err(response) => response,
    }
}

/// What `POST /admin/clock` may carry. `time_scale` is a field the protocol specifies and this
/// server does not yet change live (TW-c): it is read only to be answered `409`. Any other field is
/// refused, so no body can name an instant (step-12 SD-D10).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockChange {
    paused: Option<bool>,
    time_scale: Option<Value>,
}

async fn set_clock(State(admin): State<Admin>, body: Bytes) -> Response {
    let change: ClockChange = match serde_json::from_slice(&body) {
        Ok(change) => change,
        Err(refusal) => return malformed(refusal.to_string()),
    };
    if change.time_scale.is_some() {
        return error(
            StatusCode::CONFLICT,
            "time_scale_fixed",
            Some(
                "this server's time scale is --time-scale, fixed for the process; a live change \
                 lands with TW-c"
                    .to_owned(),
            ),
        );
    }
    let Some(paused) = change.paused else {
        return malformed("the body is { \"paused\": true | false }");
    };
    match ask(&admin, ControlCommand::Pause(paused)).await {
        Ok(ControlAnswer::Clock(state)) => Json(state).into_response(),
        Ok(other) => unexpected(&other),
        Err(response) => response,
    }
}
