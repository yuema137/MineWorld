//! `mineworld` — the command that runs a world.
//!
//! ```text
//! mineworld server <world> [--listen ADDRESS] [--agent SEAT]... [--save DIR]
//!                                       load the pack and host it; with --save, persisted
//! mineworld validate <world>            load it, say what it is, and stop
//! mineworld replay <world> --save DIR   re-execute a save's whole history and check it
//! ```
//!
//! `ARC-6` makes the artefacts the deliverable: MineWorld is *an installable world runtime plus
//! developer tools plus reference clients*, and this is the tool a person actually types. One binary
//! for every deployment (`docs/NETWORKING.md` §1) — `127.0.0.1:7878` by default, and
//! `--listen 0.0.0.0:7878` is how friends on a LAN reach it. There is no separate single-player mode
//! to choose, because there is no separate single-player path to choose it with.
//!
//! `create` and `inspect` are S7's and are absent rather than stubbed: a command that exists and
//! does nothing is worse than one that does not exist, because a person builds a habit on it.
//! `validate` is here because it is [`WorldPack::read`] plus [`WorldPack::load`] with no new code.
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
//! # Why the arguments are parsed by hand
//!
//! Two subcommands and one option. `clap` is the right answer the day `create` and `inspect` arrive
//! with real option surfaces, and adopting it now would be a dependency decision made for a command
//! line that does not exist yet (`REUSE_POLICY.md`: never adopt a dependency merely because it
//! exists). Recorded so it is a decision rather than an omission.

mod agent;
mod perceive;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend, verify};
use mineworld_presence::PerceptionProvider;
use mineworld_server::{
    HostConfig, HostError, HostedWorld, SeatRoster, WorldHost, WorldInstanceId, app,
};
use mineworld_worldpack::{PackError, WorldPack};

use crate::perceive::PackPerception;

/// Where the server listens when nothing says otherwise: the local player's own machine.
const DEFAULT_LISTEN: &str = "127.0.0.1:7878";

/// What this command can do, as a person is told.
const USAGE: &str = "\
mineworld — run a MineWorld world

    mineworld server <world> [--listen ADDRESS] [--agent SEAT]... [--save DIR]
        host a World Pack (default 127.0.0.1:7878)
        --agent SEAT drives that seat with a rule controller, in this process, over the same
        path a client's connection uses. Repeat it for more than one.
        --save DIR keeps the world in DIR/world.sqlite: created from the pack the first time,
        resumed — the same world, where it stopped — every time after.

    mineworld validate <world>
        check a World Pack and say what it is

    mineworld replay <world> --save DIR
        re-execute the saved world's whole history from its beginning and check that every
        fact and every snapshot reproduces, byte for byte

`mineworld create` and `mineworld inspect` do not exist yet (they are S7).";

#[tokio::main]
async fn main() -> ExitCode {
    let command = match Command::from_arguments(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(complaint) => {
            eprintln!("{complaint}");
            return ExitCode::FAILURE;
        }
    };

    let outcome = match command {
        Command::Help => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Command::Validate { world } => validate(&world),
        Command::Replay { world, save } => replay(&world, &save),
        Command::Server {
            world,
            listen,
            agents,
            save,
        } => serve(world, listen, agents, save).await,
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(complaint) => {
            eprintln!("{complaint}");
            ExitCode::FAILURE
        }
    }
}

/// What this invocation was asked to do.
enum Command {
    /// Host a world, and optionally drive some of its seats with a controller.
    Server {
        world: PathBuf,
        listen: SocketAddr,
        agents: Vec<EntityKey>,
        save: Option<PathBuf>,
    },
    /// Check a world and report it.
    Validate { world: PathBuf },
    /// Re-execute a save's history and check it.
    Replay { world: PathBuf, save: PathBuf },
    /// Say what the command can do.
    Help,
}

impl Command {
    /// Reads the arguments, or explains what is wrong with them.
    ///
    /// Every complaint names the offending argument and then points at `--help`, because a command
    /// line is the first thing a person meets and *usage: ...* is not an explanation.
    fn from_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut arguments = arguments.into_iter();
        let Some(subcommand) = arguments.next() else {
            return Err(format!("mineworld needs a command.\n\n{USAGE}"));
        };
        if subcommand == "--help" || subcommand == "-h" || subcommand == "help" {
            return Ok(Self::Help);
        }

        match subcommand.as_str() {
            "server" => {
                let world = world_argument(&subcommand, arguments.next())?;
                let mut listen = DEFAULT_LISTEN.to_owned();
                let mut agents = Vec::new();
                let mut save = None;
                while let Some(argument) = arguments.next() {
                    match argument.as_str() {
                        "--save" => save = Some(save_argument(arguments.next())?),
                        "--listen" => {
                            listen = arguments.next().ok_or_else(|| {
                                "--listen needs an address, such as 0.0.0.0:7878".to_owned()
                            })?;
                        }
                        "--agent" => {
                            let seat = arguments.next().ok_or_else(|| {
                                "--agent needs a seat, such as --agent alice".to_owned()
                            })?;
                            agents.push(EntityKey::new(&seat).map_err(|error| {
                                format!("--agent {seat} is not a seat name: {error}")
                            })?);
                        }
                        other => return Err(unexpected(other)),
                    }
                }
                let listen = listen
                    .parse()
                    .map_err(|error| format!("--listen {listen} is not an address: {error}"))?;
                Ok(Self::Server {
                    world,
                    listen,
                    agents,
                    save,
                })
            }
            "replay" => {
                let world = world_argument(&subcommand, arguments.next())?;
                let mut save = None;
                while let Some(argument) = arguments.next() {
                    match argument.as_str() {
                        "--save" => save = Some(save_argument(arguments.next())?),
                        other => return Err(unexpected(other)),
                    }
                }
                let save = save.ok_or_else(|| {
                    "mineworld replay needs --save DIR, the save to check".to_owned()
                })?;
                Ok(Self::Replay { world, save })
            }
            "validate" => {
                let world = world_argument(&subcommand, arguments.next())?;
                if let Some(extra) = arguments.next() {
                    return Err(unexpected(&extra));
                }
                Ok(Self::Validate { world })
            }
            "create" | "inspect" | "run" => Err(format!(
                "mineworld {subcommand} does not exist yet — it is S7's.\n\n{USAGE}"
            )),
            other => Err(format!("mineworld has no command '{other}'.\n\n{USAGE}")),
        }
    }
}

fn world_argument(subcommand: &str, argument: Option<String>) -> Result<PathBuf, String> {
    argument
        .filter(|world| !world.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| {
            format!(
                "mineworld {subcommand} needs a World Pack directory, such as worlds/social-cafe"
            )
        })
}

fn save_argument(argument: Option<String>) -> Result<PathBuf, String> {
    argument
        .filter(|directory| !directory.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| "--save needs a directory, such as --save saves/social-cafe".to_owned())
}

fn unexpected(argument: &str) -> String {
    format!("unexpected argument '{argument}'.\n\n{USAGE}")
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
    agents: Vec<EntityKey>,
    save: Option<PathBuf>,
) -> Result<(), String> {
    // Read on this thread, before anything binds a socket: an operator who mistyped a path should be
    // told so immediately, and by the pack's own refusal rather than by a server that failed to start.
    let pack = WorldPack::read(&world).map_err(described)?;
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

    // Each controller is a task of its own, occupying a seat exactly as a client's connection does.
    for seat in agents {
        tokio::spawn(agent::drive(host.clone(), seat));
    }

    app::serve_with_shutdown(listener, host.clone(), async {
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
    let composed = pack.compose().map_err(described)?;
    let backend = SqliteBackend::open(save, Durability::PowerLoss)
        .map_err(|error| format!("[mineworld] {error}"))?;
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
