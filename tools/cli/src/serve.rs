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

use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend};
use mineworld_presence::PerceptionProvider;
use mineworld_server::{
    Admission, HostConfig, HostError, HostedWorld, SeatRoster, WorldHost, WorldInstanceId, app,
};
use mineworld_worldpack::{PackRoots, WorldPack};

use crate::perceive::PackPerception;
use crate::{described, hosted, invite, listed};

/// What `mineworld server` was asked.
pub struct ServeRequest {
    /// The World Pack directory.
    pub world: PathBuf,
    /// Where to listen.
    pub listen: SocketAddr,
    /// The operator's invite, from `--invite` or `MINEWORLD_INVITE`.
    pub invite: Option<String>,
    /// The seats the reactive rule controller drives.
    pub agents: Vec<EntityKey>,
    /// Whether the paced rule controller drives every other seat.
    pub town: bool,
    /// The paced controllers' seed.
    pub seed: u64,
    /// How often each paced seat is consulted, in wall seconds (QTW-13).
    pub pace: NonZeroU32,
    /// How long a dropped connection's seat is held, in wall seconds.
    pub hold: u32,
    /// World seconds per wall second.
    pub time_scale: NonZeroU32,
    /// Where the world is kept, when it is persisted.
    pub save: Option<PathBuf>,
    /// Where the world's `requires:` is resolved (`--packs`, then `MINEWORLD_PACKS`; `ARC-54`).
    pub roots: PackRoots,
}

/// Which controller drives a seat nobody plays.
#[derive(Clone, Copy)]
enum Driver {
    Reactive,
    /// The paced controller, as seat number `k` in roster order.
    Paced(usize),
}

/// The seats in-server controllers drive, with the tally each one's statistics go to.
fn drivers(
    seats: &[EntityKey],
    agents: &[EntityKey],
    town: bool,
) -> Vec<(EntityKey, Driver, hosted::Tally)> {
    seats
        .iter()
        .enumerate()
        .filter_map(|(k, seat)| {
            let driver = if agents.contains(seat) {
                Driver::Reactive
            } else if town {
                Driver::Paced(k)
            } else {
                return None;
            };
            Some((seat.clone(), driver, hosted::Tally::default()))
        })
        .collect()
}

/// Loads a pack and hosts it until interrupted, with a controller on each requested seat.
pub async fn serve(request: ServeRequest) -> Result<(), String> {
    let ServeRequest {
        world,
        listen,
        invite,
        agents,
        town,
        seed,
        pace,
        hold,
        time_scale,
        save,
        roots,
    } = request;
    // Read on this thread, before anything binds a socket: an operator who mistyped a path should be
    // told so immediately, and by the pack's own refusal rather than by a server that failed to start.
    let pack = WorldPack::read_with(&world, &roots).map_err(described)?;
    let invite = invite::Invite::resolve(invite)?;
    let config = HostConfig {
        hold: Duration::from_secs(u64::from(hold)),
        time_scale,
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

    let roster: Vec<EntityKey> = pack.seats().iter().cloned().collect();
    let driven = drivers(&roster, &agents, town);
    let tallies: Vec<(EntityKey, hosted::Tally)> = driven
        .iter()
        .map(|(seat, _, tally)| (seat.clone(), Arc::clone(tally)))
        .collect();
    // Cadence is wall time (QTW-13): the adapters scale it into world seconds themselves.
    let pacing = hosted::Pacing {
        seed,
        pace,
        scale: time_scale,
    };

    let recent = config.recent_events;
    let host = WorldHost::spawn(config, move || {
        let seats = SeatRoster::new(pack.seats().iter().cloned());
        let hosted = match save {
            None => {
                let running = pack.load(epoch).map_err(HostError::build)?.into_running();
                HostedWorld::new(running.world).perceiving(PackPerception::new(running.providers))
            }
            Some(save) => {
                let (persisted, providers) = persisted(&pack, &save, epoch)?;
                HostedWorld::persisted(persisted, recent)?
                    .perceiving(PackPerception::new(providers))
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
                        Box::new(hosted::ReactiveSeat::bound(bound, time_scale, &tally))
                    }),
                    Driver::Paced(k) => hosted.hosting(seat, move |_bound| {
                        Box::new(hosted::PacedSeat::new(pacing, epoch, k, &tally))
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
    let admission = Admission::new(invite.token().clone());

    println!(
        "[mineworld] hold {hold} s, time scale {}, {} seat(s) driven in-server{}",
        status.time_scale,
        tallies.len(),
        if town {
            format!(" (town: seed {seed}, pace {pace} wall s)")
        } else {
            String::new()
        },
    );

    app::serve_with_shutdown(listener, host.clone(), admission, async {
        let _ = tokio::signal::ctrl_c().await;
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
