//! `mineworld` — the command that runs a world.
//!
//! ```text
//! mineworld server <world> [--listen ADDRESS] [--invite TOKEN] [--agent SEAT]... [--save DIR]
//!                                       load the pack and host it; with --save, persisted;
//!                                       clients join with the invite (generated and printed
//!                                       when neither --invite nor MINEWORLD_INVITE gives one)
//! mineworld validate <world>            load it, say what it is, and stop
//! mineworld replay <world> --save DIR   re-execute a save's whole history and check it
//! mineworld run <world> --headless --seed N --days N [--save DIR]
//!                                       run it headless, every seat a seeded rule (ARC-27)
//! mineworld inspect <save> [--last N]   what a save holds; every fact's cause checked (AC-9)
//! mineworld biography <world> --save DIR --person KEY [--json]
//!                                       a Person's objective biography, from the fact log (ARC-29)
//! mineworld create <directory>          a new, minimal World Pack
//! mineworld packs list|show|validate    package identities: this build's packs and the data packs
//!                                       in each --packs DIR (ARC-53)
//! ```
//!
//! `docs/MODULE_SPEC.md` §8.1 specifies the command surface.
//!
//! `ARC-6` makes the artefacts the deliverable: MineWorld is *an installable world runtime plus
//! developer tools plus reference clients*, and this is the tool a person actually types. One binary
//! for every deployment (`docs/NETWORKING.md` §1) — `127.0.0.1:7878` by default, and
//! `--listen 0.0.0.0:7878` is how friends on a LAN reach it. There is no separate single-player mode
//! to choose, because there is no separate single-player path to choose it with.
//!
//! `install` and `add-system` (`MODULE_SPEC.md` §8's intended shape) are refused rather than
//! stubbed: a command that exists and does nothing is worse than one that does not exist, because a
//! person builds a habit on it. `validate` is [`WorldPack::read`] plus [`WorldPack::load`] with no
//! new code.
//!
//! # This is the composition root, and it is the only one
//!
//! ```text
//! mineworld-worldpack   what a world is, read off the disk
//! mineworld-presence    ┐ the System Packs a pack may enable
//! mineworld-conversation┘
//! mineworld-server      the transport, which knows about none of them
//! ```
//!
//! The server crate is a **library**: `WorldHost::spawn` takes a closure that assembles a world, and
//! this binary is what supplies one. Putting the binary in `server/` instead would make the transport
//! depend on the loader and therefore on every System Pack, which is `ENGINEERING_STANDARDS.md` §4's
//! one-way rule inverted — and two binaries that both start a server would contradict
//! `NETWORKING.md` §1 besides.
//!
//! # Arguments
//!
//! Parsed by `clap` (`docs/DECISIONS.md` `DEP-11`), in this file only: the derived [`Cli`] is turned
//! into plain values before anything runs, so no other module names the parser. The command surface
//! is specified in `docs/MODULE_SPEC.md` §8.1.

mod agent;
mod biography;
mod create;
mod inspect;
mod invite;
mod packs;
mod perceive;
mod run;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use mineworld_contracts::{EntityKey, EventEnvelope, WorldTime};
use mineworld_persistence::{
    Creation, Durability, PersistError, PersistenceBackend, PersistentWorld, SqliteBackend,
    WorldRevision, format, verify,
};
use mineworld_presence::PerceptionProvider;
use mineworld_server::{
    Admission, HostConfig, HostError, HostedWorld, SeatRoster, WorldHost, WorldInstanceId, app,
};
use mineworld_worldpack::{PackError, WorldPack};

use crate::perceive::PackPerception;

/// Where the server listens when nothing says otherwise: the local player's own machine.
const DEFAULT_LISTEN: &str = "127.0.0.1:7878";

/// `mineworld` — run a MineWorld world.
#[derive(Debug, Parser)]
#[command(name = "mineworld", about = "Run a MineWorld world", version)]
struct Cli {
    #[command(subcommand)]
    command: Subcommand,
}

/// What this invocation was asked to do.
#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    /// Host a World Pack for clients.
    Server {
        /// The World Pack directory, such as worlds/social-cafe.
        world: PathBuf,
        /// Where to listen; 0.0.0.0:7878 lets friends on a LAN reach it.
        #[arg(long, default_value = DEFAULT_LISTEN)]
        listen: SocketAddr,
        /// The invite every client must present to join. Without it (and without
        /// MINEWORLD_INVITE) one is generated and printed once.
        #[arg(
            long,
            value_name = "TOKEN",
            env = "MINEWORLD_INVITE",
            hide_env_values = true
        )]
        invite: Option<String>,
        /// Drive that seat with a rule controller, in this process, over the same path a client's
        /// connection uses. Repeat it for more than one.
        #[arg(long = "agent", value_name = "SEAT", value_parser = seat)]
        agents: Vec<EntityKey>,
        /// Keep the world in DIR/world.sqlite: created from the pack the first time, resumed — the
        /// same world, where it stopped — every time after.
        #[arg(long, value_name = "DIR")]
        save: Option<PathBuf>,
    },
    /// Check a World Pack and say what it is.
    Validate {
        /// The World Pack directory.
        world: PathBuf,
    },
    /// Re-execute a saved world's whole history from its beginning and check that every fact and
    /// every snapshot reproduces, byte for byte.
    Replay {
        /// The World Pack the save was created from.
        world: PathBuf,
        /// The save to check.
        #[arg(long, value_name = "DIR")]
        save: PathBuf,
    },
    /// Write a new, minimal World Pack into a directory that does not exist yet; its name becomes
    /// the world's id.
    Create {
        /// The directory to create, such as worlds/my-town.
        directory: PathBuf,
    },
    /// Not in MVP-0 (MODULE_SPEC §8: the intended shape, not implemented).
    #[command(hide = true)]
    Install {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    /// Not in MVP-0 (MODULE_SPEC §8: the intended shape, not implemented).
    #[command(hide = true, name = "add-system")]
    AddSystem {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    /// Say what a save holds, and check that every fact in it has a cause — without resuming or
    /// writing it.
    Inspect {
        /// The save directory (the one given to --save).
        save: PathBuf,
        /// How many of the newest facts to list.
        #[arg(long, default_value_t = 20)]
        last: usize,
    },
    /// Run a World Pack headless — no renderer, no network, no model — every seat driven by a
    /// seeded rule, and print what happened.
    Run {
        /// The World Pack directory.
        world: PathBuf,
        /// Required: the only mode `run` has (MODULE_SPEC §8.1).
        #[arg(long, required = true)]
        headless: bool,
        /// The controllers' seed. The same seed reproduces the run exactly.
        #[arg(long)]
        seed: u64,
        /// The age to run the world to, in simulated days since its genesis.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        days: u64,
        /// Keep the world in DIR/world.sqlite; an existing save is resumed and run on to the same
        /// age, so re-running a killed command finishes the same world.
        #[arg(long, value_name = "DIR")]
        save: Option<PathBuf>,
    },
    /// Print a Person's objective biography, derived from a save's fact log without resuming or
    /// writing it.
    Biography {
        /// The World Pack the save was created from (it names people by their keys).
        world: PathBuf,
        /// The save to read.
        #[arg(long, value_name = "DIR")]
        save: PathBuf,
        /// The Person, by authoring key, such as alice.
        #[arg(long, value_name = "KEY", value_parser = seat)]
        person: EntityKey,
        /// One JSON object per entry instead of lines for reading.
        #[arg(long)]
        json: bool,
    },
    /// Package identities: what each pack is, its version, licence and provenance (ARC-53).
    Packs {
        #[command(subcommand)]
        command: PacksCommand,
    },
}

/// What `mineworld packs` was asked to do.
#[derive(Debug, clap::Subcommand)]
enum PacksCommand {
    /// Every pack this build provides, then the data packs in each --packs DIR.
    List {
        /// A directory whose immediate subdirectories are data packs. Repeat it for more.
        #[arg(long = "packs", value_name = "DIR")]
        roots: Vec<PathBuf>,
    },
    /// Every package field of one pack.
    Show {
        /// The pack's id, such as mineworld-presence.
        id: String,
        /// A directory whose immediate subdirectories are data packs. Repeat it for more.
        #[arg(long = "packs", value_name = "DIR")]
        roots: Vec<PathBuf>,
    },
    /// Check one data pack: its package fields, all required, then its content.
    Validate {
        /// The pack's directory.
        directory: PathBuf,
    },
}

/// A seat name on the command line, checked as the key it must be.
fn seat(text: &str) -> Result<EntityKey, String> {
    EntityKey::new(text).map_err(|error| format!("'{text}' is not a seat name: {error}"))
}

#[tokio::main]
async fn main() -> ExitCode {
    let outcome = match Cli::parse().command {
        Subcommand::Validate { world } => validate(&world),
        Subcommand::Replay { world, save } => replay(&world, &save),
        Subcommand::Server {
            world,
            listen,
            invite,
            agents,
            save,
        } => serve(world, listen, invite, agents, save).await,
        Subcommand::Create { directory } => create::create(&directory),
        Subcommand::Install { .. } => not_yet("install"),
        Subcommand::AddSystem { .. } => not_yet("add-system"),
        Subcommand::Inspect { save, last } => inspect::inspect(&save, last),
        Subcommand::Run {
            world,
            headless: _,
            seed,
            days,
            save,
        } => run::run(&run::RunRequest {
            world,
            seed,
            days,
            save,
        }),
        Subcommand::Biography {
            world,
            save,
            person,
            json,
        } => biography::biography(&biography::BiographyRequest {
            world: &world,
            save: &save,
            person: &person,
            json,
        }),
        Subcommand::Packs { command } => match command {
            PacksCommand::List { roots } => packs::list(&roots),
            PacksCommand::Show { id, roots } => packs::show(&id, &roots),
            PacksCommand::Validate { directory } => packs::validate(&directory),
        },
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(complaint) => {
            eprintln!("{complaint}");
            ExitCode::FAILURE
        }
    }
}

/// A command `MODULE_SPEC.md` §8 describes as intended and MVP-0 does not implement, refused rather
/// than stubbed: a command that exists and does nothing is worse than one that does not, because a
/// person builds a habit on it.
fn not_yet(command: &str) -> Result<(), String> {
    Err(format!(
        "mineworld {command} does not exist yet — docs/MODULE_SPEC.md §8 describes it as intended, \
         and MVP-0 does not implement it. What works today: mineworld server, mineworld validate, \
         mineworld replay, mineworld run, mineworld inspect, mineworld biography, mineworld create, \
         mineworld packs (see mineworld --help)."
    ))
}

/// Checks a pack and says what world it describes.
///
/// Loads it as well as reading it, deliberately: reading answers *is this pack internally
/// consistent*, and loading answers *can the world it describes be composed at all* — a dependency
/// between systems is the kernel's judgement, and a `validate` that skipped it would pass a pack the
/// server then refused to host.
fn validate(world: &PathBuf) -> Result<(), String> {
    let pack = WorldPack::read(world).map_err(described)?;
    let loaded = pack.load(WorldTime::EPOCH).map_err(described)?;

    println!("{} ({})", pack.name(), pack.id());
    println!("  systems    {}", listed(pack.systems()));
    println!("  places     {}", listed(pack.places().keys()));
    println!("  people     {}", listed(pack.people().keys()));
    // Printed only when declared, so a pack without them reports exactly what it did before they
    // existed (step-10 QS-18).
    if !pack.items().is_empty() {
        println!("  items      {}", listed(pack.items().keys()));
    }
    if !pack.organizations().is_empty() {
        println!("  organizations {}", listed(pack.organizations().keys()));
    }
    println!("  seats      {}", listed(pack.seats().iter()));
    println!();
    // In identity order rather than key order: these are the ids the event log refers to, and the
    // order they were allocated in is the fact an author is checking.
    let mut allocated: Vec<_> = loaded.ids().iter().map(|(key, id)| (*id, key)).collect();
    allocated.sort_unstable();
    for (id, key) in allocated {
        println!("  {id:>4}  {key}");
    }
    println!(
        "\n{} genesis fact(s): the world's initial state, each one caused by the world coming into \
         existence",
        loaded.genesis().len(),
    );
    println!("{} is a valid World Pack.", world.display());
    Ok(())
}

/// Loads a pack and hosts it until interrupted, with a controller on each requested seat.
async fn serve(
    world: PathBuf,
    listen: SocketAddr,
    invite: Option<String>,
    agents: Vec<EntityKey>,
    save: Option<PathBuf>,
) -> Result<(), String> {
    // Read on this thread, before anything binds a socket: an operator who mistyped a path should be
    // told so immediately, and by the pack's own refusal rather than by a server that failed to start.
    let pack = WorldPack::read(&world).map_err(described)?;
    let invite = invite::Invite::resolve(invite)?;
    let config = HostConfig::default();
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

    // Each controller is a task of its own, occupying a seat exactly as a client's connection does.
    // In-process, so it presents no invite: admission gates the socket, not the composition root.
    for seat in agents {
        tokio::spawn(agent::drive(host.clone(), seat));
    }

    app::serve_with_shutdown(listener, host.clone(), admission, async {
        let _ = tokio::signal::ctrl_c().await;
        println!("\n[mineworld] stopping");
    })
    .await
    .map_err(|error| format!("[mineworld] {error}"))?;

    host.shutdown().await;
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

/// Re-executes a save's whole history from genesis and reports what it compared.
fn replay(world: &Path, save: &Path) -> Result<(), String> {
    let pack = WorldPack::read(world).map_err(described)?;
    let backend = SqliteBackend::open(save, Durability::PowerLoss)
        .map_err(|error| format!("[mineworld] {error}"))?;
    let genesis = saved_genesis(&backend).map_err(|error| format!("[mineworld] {error}"))?;
    pack.check_configuration(&genesis).map_err(described)?;
    let composed = pack.compose().map_err(described)?;
    let verified = verify(&backend, composed.world)
        .map_err(|error| format!("[mineworld] the save does not reproduce: {error}"))?;
    println!(
        "{}: {} revision(s) re-executed from genesis, {} fact(s) and {} snapshot(s) reproduced byte \
         for byte; head revision {}",
        SqliteBackend::file(save).display(),
        verified.revisions,
        verified.facts,
        verified.snapshots,
        verified.head.raw(),
    );
    Ok(())
}

/// A save's genesis facts, as recorded: what a host hands [`WorldPack::check_configuration`] before it
/// resumes or verifies, so that a save never runs on against a configuration other than the one it was
/// created with (`DECISIONS.md` `ARC-61` item 7).
fn saved_genesis(backend: &SqliteBackend) -> Result<Vec<EventEnvelope>, PersistError> {
    backend
        .facts_of(WorldRevision::GENESIS)?
        .iter()
        .map(|row| format::decode(&row.bytes, "fact"))
        .collect()
}

/// A pack's own refusal, as a person reads it.
///
/// The chain is printed, not just the outermost message: the outer variant says which file and what
/// was wrong with it, and a `source` beneath it is the kernel's or the contract's own words, which are
/// the precise half.
fn described(error: PackError) -> String {
    let mut message = format!("[mineworld] {error}");
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        message.push_str(&format!("\n           caused by: {cause}"));
        source = cause.source();
    }
    message
}

/// A list of names for a person to read, in the order given.
fn listed(values: impl IntoIterator<Item = impl std::fmt::Display>) -> String {
    let names: Vec<String> = values.into_iter().map(|value| value.to_string()).collect();
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}
