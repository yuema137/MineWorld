//! The admin surface: HTTP under `/admin`, behind one bearer token (`PROTOCOL.md` §11,
//! `docs/DECISIONS.md` `ARC-44`).
//!
//! ```text
//! GET  /admin/sessions                  every seated connection
//! GET  /admin/seats                     every seat's binding
//! POST /admin/sessions/{session}/kick   closing {kicked}; the seat returns to its default, no hold
//! POST /admin/seats/{seat}/release      whoever holds it is closed; the seat returns to its default
//! GET  /admin/clock                     { at, time_scale, paused }
//! POST /admin/clock                     { "paused": bool }, { "time_scale": n }, or both
//! ```
//!
//! Mounted only when the server has an admin token ([`crate::app::Access`]); without one no `/admin`
//! path exists. Every request passes [`permitted`] first: a failure of any kind is answered `401`
//! no sooner than [`UNAUTHORIZED_DELAY`] after the request arrived, slept in that request's own task
//! and never on the world's thread. A handler holds no binding and touches no `World`: it reads the
//! transport's [`registry`] or sends one command to the world thread, whose answer is host state
//! (`ARC-40`). No route and no field names an instant.

pub(crate) mod registry;

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;
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

use crate::admission::{
    AdminToken, Credential, CredentialKind, Nickname, SeatRights, UNAUTHORIZED_DELAY,
};
use crate::host::WorldHost;
use crate::protocol::SessionId;
use crate::runtime::{ClockChange, ControlAnswer, ControlCommand, DAY_LENGTH_SCALE};
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
    credential: CredentialView,
}

/// Which credential admitted a session, as `GET /admin/sessions` shows it (`PROTOCOL.md` §11.2,
/// ruled QS27a-1): its kind and, for a seat grant, its seats and take-over right. Never a token, and
/// nothing about what kind of Controller drives the connection (`INV-S27-4`).
#[derive(Serialize)]
struct CredentialView {
    kind: CredentialKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    seats: Option<BTreeSet<EntityKey>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    take_over: Option<bool>,
}

impl From<Credential> for CredentialView {
    fn from(credential: Credential) -> Self {
        match credential.rights {
            SeatRights::Any => Self {
                kind: credential.kind,
                seats: None,
                take_over: None,
            },
            SeatRights::Named { seats, take_over } => Self {
                kind: credential.kind,
                seats: Some(seats),
                take_over: Some(take_over),
            },
        }
    }
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
                credential: listed.joined.credential.into(),
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

/// The largest scale a live change may name: one world hour per wall second (`PROTOCOL.md` §11.2).
const LIVE_SCALE_CEILING: u64 = 3_600;

/// What `POST /admin/clock` may carry: `paused`, `time_scale`, or both (`PROTOCOL.md` §11.2). Any other
/// field is refused, so no body can name an instant (step-12 SD-D10). The scale is read as a JSON value so
/// that a number that is not a whole one, or is a string, is refused as `malformed` rather than guessed at.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockBody {
    paused: Option<bool>,
    time_scale: Option<Value>,
}

/// The scale a body names, checked against the live range (1 to 3 600). Anything else is `malformed`.
fn scale_named(value: &Value) -> Result<NonZeroU32, String> {
    let refusal = || {
        format!(
            "time_scale is a whole number of world seconds per wall second, 1 to {LIVE_SCALE_CEILING}"
        )
    };
    let Some(named) = value.as_u64() else {
        return Err(refusal());
    };
    if !(1..=LIVE_SCALE_CEILING).contains(&named) {
        return Err(refusal());
    }
    u32::try_from(named)
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or_else(refusal)
}

async fn set_clock(State(admin): State<Admin>, body: Bytes) -> Response {
    let body: ClockBody = match serde_json::from_slice(&body) {
        Ok(body) => body,
        Err(refusal) => return malformed(refusal.to_string()),
    };
    if body.paused.is_none() && body.time_scale.is_none() {
        return malformed(
            "the body is { \"paused\": true | false }, { \"time_scale\": n }, or both",
        );
    }
    let time_scale = match body.time_scale.as_ref().map(scale_named).transpose() {
        Ok(scale) => scale,
        Err(detail) => return malformed(detail),
    };
    // The fast-forward rule (`ARC-69` item 5): above a day in 24 hours only a `--solo` world may run, and
    // a refusal changes nothing, not even the pause the same body named.
    if time_scale.is_some_and(|scale| scale.get() > DAY_LENGTH_SCALE) && !admin.host.solo() {
        return error(
            StatusCode::CONFLICT,
            "fast_forward_solo_only",
            Some(format!(
                "only a world started --solo may run above {DAY_LENGTH_SCALE}x; this one may not"
            )),
        );
    }
    let change = ClockChange {
        paused: body.paused,
        time_scale,
    };
    match ask(&admin, ControlCommand::Change(change)).await {
        Ok(ControlAnswer::Clock(state)) => Json(state).into_response(),
        Ok(other) => unexpected(&other),
        Err(response) => response,
    }
}
