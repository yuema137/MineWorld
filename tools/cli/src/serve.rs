//! `mineworld server` — a World Pack hosted for clients until the operator stops it.
//!
//! ```text
//! read the pack            on this thread, so a mistyped path is reported before anything binds
//! build the world          inside its own thread (a World is not Send), created or resumed from --save
//! bind, print, serve       the join line, then /health, /status and /ws until Ctrl-C
//! ```
//!
//! The server crate is a library that knows no System Pack and no controller; this module is where
//! the composition root hands it a world, a perception and the controllers that drive seats.

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use mineworld_contracts::{EntityKey, EventEnvelope, WorldTime};
use mineworld_decision::{ResolvedDecisions, SeatModel};
use mineworld_persistence::{
    Creation, Durability, PersistError, PersistenceBackend, PersistentWorld, SqliteBackend,
    WorldRevision, format,
};
use mineworld_presence::PerceptionProvider;
use mineworld_server::{
    Access, AdminToken, Admission, HostConfig, HostError, HostedWorld, RequestBudget, SeatRoster,
    TimeScaleChoice, WorldHost, WorldInstanceId, app,
};
use mineworld_worldpack::{PackRoots, WorldPack};

use crate::decisions::{self, DecisionArgs};
use crate::history::SavedHistory;
use crate::perceive::{PackEventPerception, PackPerception};
use crate::{described, hosted, invite, listed};

/// The largest scale that is a day in 24 hours (`ARC-69` item 5): above it, only `--solo` may run.
const DAY_LENGTH_SCALE: u32 = 24;

/// What `mineworld server` was asked.
pub struct ServeRequest {
    /// The World Pack directory.
    pub world: PathBuf,
    /// Where to listen.
    pub listen: SocketAddr,
    /// The operator's invite, from `--invite` or `MINEWORLD_INVITE`.
    pub invite: Option<String>,
    /// The admin surface's bearer token, from `--admin-token` or `MINEWORLD_ADMIN_TOKEN`; `None`
    /// mounts no admin surface.
    pub admin_token: Option<String>,
    /// The `--grant`s, as typed; each is issued a generated token once the roster is known.
    pub grants: Vec<invite::GrantFlag>,
    /// Each seat's request budget (`--request-budget`); `None`: no budget.
    pub request_budget: Option<RequestBudget>,
    /// The seats the reactive rule controller drives.
    pub agents: Vec<EntityKey>,
    /// Whether the paced rule controller drives every other seat.
    pub town: bool,
    /// The paced controllers' seed.
    pub seed: u64,
    /// How often each paced seat is consulted, in wall seconds (QTW-13; `--cadence`).
    pub cadence: NonZeroU32,
    /// How long a dropped connection's seat is held, in wall seconds.
    pub hold: u32,
    /// World seconds per wall second, if the operator gave one (`--time-scale`); else the save's journal, then
    /// the World Pack's `hosting:`, then 1 (`ARC-69` item 8).
    pub time_scale: Option<NonZeroU32>,
    /// Whether this is a single-player world (`--solo`): loopback only, and the only kind that may fast-forward.
    pub solo: bool,
    /// Every how many frames a client is sent a whole observation (`DEP-15`).
    pub keyframe_every: NonZeroU32,
    /// Where the world is kept, when it is persisted.
    pub save: Option<PathBuf>,
    /// Where the world's `requires:` is resolved (`--packs`, then `MINEWORLD_PACKS`; `ARC-54`).
    pub roots: PackRoots,
    /// Whether the end of standard input also stops the server (`--stop-on-stdin-eof`, `ARC-78`).
    pub stop_on_stdin_eof: bool,
    /// `--decisions` and `--seat-model`: which model drives each `--town` seat (`ARC-84`).
    pub decisions: DecisionArgs,
}

/// Which controller drives a seat nobody plays.
#[derive(Clone)]
enum Driver {
    Reactive,
    /// The seat's resolved decision model, as seat number `k` in roster order.
    Modelled {
        k: usize,
        model: Arc<dyn SeatModel>,
    },
}

/// The seats in-server controllers drive, with the tally each one's statistics go to. A `--town` seat
/// whose fallback chain ends at `none` is not driven in-server.
fn drivers(
    seats: &[EntityKey],
    agents: &[EntityKey],
    town: bool,
    resolved: &ResolvedDecisions,
) -> Vec<(EntityKey, Driver, hosted::Tally)> {
    seats
        .iter()
        .enumerate()
        .filter_map(|(k, seat)| {
            let driver = if agents.contains(seat) {
                Driver::Reactive
            } else if town {
                let model = Arc::clone(resolved.drives(k)?.model());
                Driver::Modelled { k, model }
            } else {
                return None;
            };
            Some((seat.clone(), driver, hosted::Tally::default()))
        })
        .collect()
}

/// The operator's decision flags checked against the rest of the command line (QDLa-1): they choose how
/// in-server seats decide, so without `--town` they would do nothing, and a seat cannot be both an
/// `--agent` and a `--seat-model`.
fn check_decision_flags(
    decisions: &DecisionArgs,
    agents: &[EntityKey],
    town: bool,
) -> Result<(), String> {
    if decisions.given() && !town {
        let flag = if decisions.file.is_some() {
            "--decisions"
        } else {
            "--seat-model"
        };
        return Err(format!(
            "[mineworld] {flag}: decisions choose how in-server seats decide; add --town"
        ));
    }
    if let Some(both) = decisions
        .seat_models
        .iter()
        .find(|given| agents.contains(&given.seat))
    {
        return Err(format!(
            "[mineworld] --seat-model {}={}: seat '{}' is also an --agent, which drives it with the \
             reactive controller; name it once",
            both.seat, both.model, both.seat
        ));
    }
    Ok(())
}

/// Loads a pack and hosts it until interrupted, with a controller on each requested seat.
pub async fn serve(request: ServeRequest) -> Result<(), String> {
    let ServeRequest {
        world,
        listen,
        invite,
        admin_token,
        grants,
        request_budget,
        agents,
        town,
        seed,
        cadence,
        hold,
        time_scale,
        solo,
        keyframe_every,
        save,
        roots,
        stop_on_stdin_eof,
        decisions,
    } = request;
    check_decision_flags(&decisions, &agents, town)?;
    // Refused before anything else, and before anything binds a socket: a fast-forward is a single-player
    // world's and only that (`ARC-69` item 5). A single-player world listens on loopback alone, because
    // nobody off the machine can join one (`ARC-69` item 5, Q-TWc-1).
    if let Some(scale) = time_scale
        && scale.get() > DAY_LENGTH_SCALE
        && !solo
    {
        return Err(format!(
            "[mineworld] --time-scale {scale}: above {DAY_LENGTH_SCALE} world seconds per wall second \
             is a fast-forward, which only a world started --solo may run"
        ));
    }
    if solo && !listen.ip().is_loopback() {
        return Err(format!(
            "[mineworld] --solo: a single-player world listens on a loopback address only, and {listen} \
             is not one; use 127.0.0.1 or ::1, or drop --solo"
        ));
    }
    // Read on this thread, before anything binds a socket: an operator who mistyped a path should be
    // told so immediately, and by the pack's own refusal rather than by a server that failed to start.
    let pack = WorldPack::read_with(&world, &roots).map_err(described)?;
    // Read whether or not --town drives anything, so a pack's bad decision list is refused the same way
    // by every command that reads the pack.
    let resolved = decisions::resolve(&pack, &decisions)?;
    let invite = invite::Invite::resolve(invite)?;
    let admin = admin_token.map(admin_token_checked).transpose()?;
    // Generated once the roster is known, and before anything binds: a grant naming a seat this world
    // does not offer refuses to start.
    let roster: Vec<EntityKey> = pack.seats().iter().cloned().collect();
    let issued = invite::issue(grants, &roster)?;
    let admission = Admission::new(invite.token().clone())
        .with_grants(issued.iter().map(|issued| issued.grant().clone()))
        .map_err(|refusal| format!("[mineworld] --grant: {refusal}"))?;
    // Checked before anything binds: an admin token equal to the invite or a grant refuses to start.
    let access = Access {
        solo,
        ..access(admission, admin)?
    };
    let config = HostConfig {
        hold: Duration::from_secs(u64::from(hold)),
        time_scale: TimeScaleChoice {
            flag: time_scale,
            pack: pack.hosting_time_scale(),
        },
        keyframe_every,
        request_budget,
        ..HostConfig::default()
    };
    let epoch = config.epoch;
    println!(
        "[mineworld] {} ({}) — {} system(s), {} seat(s)",
        pack.name(),
        pack.id(),
        pack.systems().len(),
        pack.seats().len(),
    );

    // The world itself is built *inside* its own thread, because a `World` is not `Send`. What crosses
    // the boundary is the pack, which is plain data — and loading it there rather than here is also
    // what keeps the world and its systems from ever being moved between threads.
    // Checked against the pack before anything starts: an operator who mistyped a seat should be told
    // so, not left watching a world in which nothing happens. The seat roster is the world's, and this
    // is the same roster a client's `join` is answered from.
    for seat in &agents {
        if !pack.seats().contains(seat) {
            return Err(format!(
                "[mineworld] --agent {seat}: this world offers no such seat. It offers: {}",
                listed(pack.seats().iter()),
            ));
        }
    }

    let driven = drivers(&roster, &agents, town, &resolved);
    let tallies: Vec<(EntityKey, hosted::Tally)> = driven
        .iter()
        .map(|(seat, _, tally)| (seat.clone(), Arc::clone(tally)))
        .collect();
    // Cadence is wall time (QTW-13): the adapters scale it into world seconds themselves, at the scale in force
    // when each controller is bound (`ARC-69` item 2).
    let pacing = hosted::Pacing {
        seed,
        pace: cadence,
        scale: NonZeroU32::MIN,
    };

    let recent = config.recent_events;
    let host = WorldHost::spawn(config, move || {
        let seats = SeatRoster::new(pack.seats().iter().cloned());
        let hosted = match save {
            None => {
                let running = pack.load(epoch).map_err(HostError::build)?.into_running();
                let events = PackEventPerception::seeded(&running.world);
                HostedWorld::new(running.world)
                    .perceiving(PackPerception::new(running.providers))
                    .perceiving_events(events)
            }
            Some(save) => {
                let (persisted, providers) = persisted(&pack, &save, epoch)?;
                let events = PackEventPerception::seeded(persisted.world());
                HostedWorld::persisted(persisted, recent)?
                    .perceiving(PackPerception::new(providers))
                    .perceiving_events(events)
                    .with_history(SavedHistory::new(save))
            }
        };
        // Each seat's factory builds a fresh controller, bound at the instant it is given: at start,
        // and every time a person gives the seat back (`PROTOCOL.md` §4.2). The paced lattice begins
        // at the epoch every world this command creates begins at, so it is the same after a resume.
        let hosted =
            driven
                .into_iter()
                .fold(hosted, |hosted, (seat, driver, tally)| match driver {
                    Driver::Reactive => hosted.hosting(seat, move |bound| {
                        Box::new(hosted::ReactiveSeat::bound(bound.at, bound.scale, &tally))
                    }),
                    Driver::Modelled { k, model } => hosted.hosting(seat, move |bound| {
                        // The scale in force when this controller is bound (`ARC-69` item 2).
                        let pacing = hosted::Pacing {
                            scale: bound.scale,
                            ..pacing
                        };
                        Box::new(hosted::PacedSeat::new(
                            pacing,
                            epoch,
                            k,
                            Arc::clone(&model),
                            &tally,
                        ))
                    }),
                });
        Ok(hosted.seating(seats))
    })
    .await
    .map_err(|error| format!("[mineworld] {error}"))?;

    let (listener, address) = app::bind(listen)
        .await
        .map_err(|error| format!("[mineworld] cannot listen on {listen}: {error}"))?;
    let status = host
        .status()
        .await
        .map_err(|error| format!("[mineworld] {error}"))?;
    println!(
        "[mineworld] listening on http://{address} (ws://{address}/ws), protocol {}",
        status.protocol,
    );
    println!(
        "[mineworld] {} entities, {} system(s), seats: {}",
        status.entities,
        status.systems.len(),
        listed(status.seats.iter()),
    );
    println!("[mineworld] world instance {}", status.instance);
    if let Some(seat) = invite::suggested_seat(&status.seats, &agents) {
        println!("{}", invite.join_line(address, seat));
    }
    // Each grant's token is printed here, once, and nowhere else (`PROTOCOL.md` §4.1 "Seat grants").
    for issued in &issued {
        println!("{}", issued.line(address));
    }
    // The token is never printed: an operator gave it, and already has it (step-12 SD-D12, I-5).
    if access.admin.is_some() {
        println!("[mineworld] admin surface: http://{address}/admin (bearer token as given)");
    } else {
        println!("[mineworld] no admin surface (no --admin-token)");
    }

    println!(
        "[mineworld] hold {hold} s, time scale {}, {}, {} seat(s) driven in-server{}",
        status.time_scale,
        request_budget.map_or_else(
            || "no request budget".to_owned(),
            |budget| format!(
                "request budget {}/s burst {} per seat",
                budget.per_second, budget.burst
            ),
        ),
        tallies.len(),
        if town {
            // The decisions clause only when a list is in effect (`MODULE_SPEC.md` §4.1 rule 7).
            format!(
                " (town: seed {seed}, cadence {cadence} wall s{})",
                resolved
                    .clause(decisions::SERVER_FALLBACK)
                    .unwrap_or_default()
            )
        } else {
            String::new()
        },
    );

    // Watched from here, after the join line: a launcher that closes the pipe early stops a server
    // that is already listening, by the one graceful path below.
    let stdin_ended = stop_on_stdin_eof.then(stdin_ended);
    app::serve_with_shutdown(listener, host.clone(), access, async move {
        match stdin_ended {
            Some(ended) => {
                tokio::select! {
                    () = stop_requested() => {}
                    _ = ended => {}
                }
            }
            None => stop_requested().await,
        }
        println!("\n[mineworld] stopping");
    })
    .await
    .map_err(|error| format!("[mineworld] {error}"))?;

    // Returns once the world thread has checkpointed and printed its tick statistics; then each
    // in-server controller's (`ARC-42`, step-12 SD-B11).
    host.shutdown().await;
    hosted::report(&tallies);
    Ok(())
}

/// The operator's admin token, checked under the invite's rules — legal, never echoed.
fn admin_token_checked(token: String) -> Result<AdminToken, String> {
    AdminToken::given(token).map_err(|refusal| format!("[mineworld] --admin-token: {refusal}"))
}

/// Who may join and who is the host: refused when the admin token equals the invite or a seat grant,
/// which would make every holder of that credential the host (`PROTOCOL.md` §11.1). Nothing is echoed.
fn access(admission: Admission, admin: Option<AdminToken>) -> Result<Access, String> {
    Access::new(admission, admin).map_err(|refusal| format!("[mineworld] --admin-token: {refusal}"))
}

/// Completes when standard input ends or cannot be read (`--stop-on-stdin-eof`, `ARC-78`): how a parent
/// with no console to signal through — a windowless launcher — asks for the stop, by closing the pipe it
/// holds or by dying. Whatever arrives before the end is discarded.
///
/// Read on a plain thread rather than tokio's blocking pool: a read still blocked when the server returns
/// must not hold the runtime's shutdown, and a detached thread ends with the process.
fn stdin_ended() -> tokio::sync::oneshot::Receiver<()> {
    let (ended, receiver) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut input = std::io::stdin().lock();
        let mut discarded = [0_u8; 256];
        loop {
            match input.read(&mut discarded) {
                Ok(0) => break,
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
        let _ = ended.send(());
    });
    receiver
}

/// Completes when the operator asks the server to stop, on every platform (step-12 SD-D13, as
/// amended by QW-1): Ctrl-C anywhere; on Windows also Ctrl-Break — which `ctrl_c()` does not catch
/// there — closing the console window, and logging off or shutting down. Each takes the same
/// graceful path: stop accepting, checkpoint, statistics.
async fn stop_requested() {
    #[cfg(windows)]
    {
        use tokio::signal::windows::{ctrl_break, ctrl_close, ctrl_shutdown};
        // A handler that cannot be installed waits forever rather than stopping the server.
        let on_break = async {
            match ctrl_break() {
                Ok(mut signal) => {
                    let _ = signal.recv().await;
                }
                Err(_) => std::future::pending::<()>().await,
            }
        };
        let on_close = async {
            match ctrl_close() {
                Ok(mut signal) => {
                    let _ = signal.recv().await;
                }
                Err(_) => std::future::pending::<()>().await,
            }
        };
        let on_shutdown = async {
            match ctrl_shutdown() {
                Ok(mut signal) => {
                    let _ = signal.recv().await;
                }
                Err(_) => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            () = on_break => {}
            () = on_close => {}
            () = on_shutdown => {}
        }
    }
    #[cfg(not(windows))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// A save's genesis facts, as recorded: what a host hands [`WorldPack::check_configuration`] before it
/// resumes or verifies, so that a save never runs on against a configuration other than the one it was
/// created with (`DECISIONS.md` `ARC-61` item 7). Shared by `persisted` here and `replay` in main.rs
/// (moved here from main.rs by S11-D, D-SD1).
pub(crate) fn saved_genesis(backend: &SqliteBackend) -> Result<Vec<EventEnvelope>, PersistError> {
    backend
        .facts_of(WorldRevision::GENESIS)?
        .iter()
        .map(|row| format::decode(&row.bytes, "fact"))
        .collect()
}

/// The pack's world with a save in `save`: created from the pack, beginning at `epoch`, if `save` holds
/// none; otherwise resumed — composed from the pack, everything else from the save.
///
/// Runs on the world's own thread (a `World` is not `Send`), so it reports what it did on stdout
/// itself: an operator should be able to tell a new world from a resumed one, and see that a resume
/// read a snapshot and re-executed what came after it.
fn persisted(
    pack: &WorldPack,
    save: &Path,
    epoch: WorldTime,
) -> Result<(PersistentWorld, Vec<Box<dyn PerceptionProvider>>), HostError> {
    if SqliteBackend::exists(save) {
        let backend = SqliteBackend::open(save, Durability::PowerLoss).map_err(HostError::build)?;
        // The configuration-drift check before resume (IL-a, `ARC-61` item 7), carried here when
        // `persisted` moved out of main.rs (step-12 QS11B-6).
        pack.check_configuration(&saved_genesis(&backend).map_err(HostError::build)?)
            .map_err(HostError::build)?;
        let composed = pack.compose().map_err(HostError::build)?;
        let (world, how) =
            PersistentWorld::resume(Box::new(backend), composed.world).map_err(HostError::build)?;
        println!(
            "[mineworld] resumed {}: revision {}, from the snapshot at revision {} plus {} \
             re-executed revision(s) whose {} fact(s) reproduced byte for byte",
            SqliteBackend::file(save).display(),
            how.head.raw(),
            how.snapshot.raw(),
            how.replayed,
            how.facts,
        );
        return Ok((world, composed.providers));
    }
    let backend = SqliteBackend::create(save, Durability::PowerLoss).map_err(HostError::build)?;
    let assembled = pack.assemble().map_err(HostError::build)?;
    let (world, began) = PersistentWorld::create(
        Box::new(backend),
        assembled.world,
        Creation {
            instance: WorldInstanceId::allocate().raw(),
            pack: pack.id().to_owned(),
            at: epoch,
            facts: assembled.facts,
        },
    )
    .map_err(HostError::build)?;
    println!(
        "[mineworld] created {}: a new world, {} genesis fact(s) at revision {}",
        SqliteBackend::file(save).display(),
        began.len(),
        world.revision().raw(),
    );
    Ok((world, assembled.providers))
}
