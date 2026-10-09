//! `mineworld` — the command that runs a world.
//!
//! ```text
//! mineworld server <world> [--listen ADDRESS] [--invite TOKEN] [--agent SEAT]... [--town]
//!                  [--seed N] [--pace SECONDS] [--hold SECONDS] [--time-scale N] [--save DIR]
//!                  [--admin-token TOKEN]
//!                                       load the pack and host it; with --save, persisted;
//!                                       clients join with the invite (generated and printed
//!                                       when neither --invite nor MINEWORLD_INVITE gives one);
//!                                       in-server controllers drive seats nobody plays (ARC-42)
//! mineworld validate <world>            load it, say what it is, and stop
//! mineworld replay <world> --save DIR   re-execute a save's whole history and check it
//! mineworld run <world> --headless --seed N --days N [--save DIR]
//!                                       run it headless, every seat a seeded rule (ARC-27)
//! mineworld inspect <save> [--last N]   what a save holds; every fact's cause checked (AC-9)
//! mineworld biography <world> --save DIR --person KEY [--json]
//!                                       a Person's objective biography, from the fact log (ARC-29)
//! mineworld perceived <world> --save DIR --person KEY …   what a Person perceived (ARC-43)
//! mineworld create <directory>         a new, minimal World Pack
//! mineworld packs list|show|validate|resolve
//!                                       package identities, and a world's composition (ARC-53, 54)
//! ```
//!
//! Every command that reads a world takes `--packs DIR` (repeatable), then `MINEWORLD_PACKS`: the
//! directories its `requires:` is resolved in, and nothing else (`ARC-54`).
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

mod biography;
mod create;
mod history;
mod hosted;
mod inspect;
mod invite;
mod packs;
mod perceive;
mod perceived;
mod run;
mod serve;

// Where `run` and `replay` find it since it moved beside `persisted` (step-12 D-SD1).
pub(crate) use serve::saved_genesis;

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_packages::PACKS_VARIABLE;
use mineworld_persistence::{Durability, SqliteBackend, verify};
use mineworld_worldpack::{PackError, PackRoots, WorldPack};

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
        /// The invite every client must present to join; else one is generated and printed once.
        #[arg(
            long,
            value_name = "TOKEN",
            env = "MINEWORLD_INVITE",
            hide_env_values = true
        )]
        invite: Option<String>,
        /// The bearer token that opens the admin surface under /admin; without it (and without
        /// MINEWORLD_ADMIN_TOKEN) there is none. Never printed; must differ from the invite.
        #[arg(long, env = "MINEWORLD_ADMIN_TOKEN", hide_env_values = true)]
        admin_token: Option<String>,
        /// Drive that seat with the reactive rule controller, on the world thread, whenever no
        /// player holds it. Repeat it for more than one.
        #[arg(long = "agent", value_name = "SEAT", value_parser = seat)]
        agents: Vec<EntityKey>,
        /// Drive every other seat with the paced rule controller whenever no player holds it.
        #[arg(long)]
        town: bool,
        /// The paced controllers' seed.
        #[arg(long, default_value_t = 0)]
        seed: u64,
        /// How often each paced seat is consulted, in wall seconds (time scale never speeds it up).
        #[arg(long, value_name = "SECONDS", default_value = "5")]
        pace: NonZeroU32,
        /// How long a dropped connection's seat is held for its resume, in wall seconds; 0: none.
        #[arg(long, value_name = "SECONDS", default_value_t = 30)]
        hold: u32,
        /// How many world seconds pass per wall second.
        #[arg(long, value_name = "N", default_value = "1")]
        time_scale: NonZeroU32,
        /// Every Nth frame to a client is a whole observation; the others are deltas (DEP-15).
        #[arg(long, value_name = "N", default_value = "50")]
        keyframe_every: NonZeroU32,
        /// Keep the world in DIR/world.sqlite: created from the pack the first time, resumed — the
        /// same world, where it stopped — every time after.
        #[arg(long, value_name = "DIR")]
        save: Option<PathBuf>,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Check a World Pack and say what it is.
    Validate {
        /// The World Pack directory.
        world: PathBuf,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Re-execute a saved world's whole history from its beginning and check that every fact and
    /// every snapshot reproduces, byte for byte.
    Replay {
        /// The World Pack the save was created from.
        world: PathBuf,
        /// The save to check.
        #[arg(long, value_name = "DIR")]
        save: PathBuf,
        #[command(flatten)]
        packs: PackDirs,
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
        #[command(flatten)]
        packs: PackDirs,
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
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Print the facts a Person perceived, from a save's fact log, by the server's audience rule.
    Perceived(perceived::PerceivedArgs),
    /// Package identities: what each pack is, its version, licence and provenance (ARC-53).
    Packs {
        #[command(subcommand)]
        command: PacksCommand,
    },
}

/// What `mineworld packs` was asked to do.
#[derive(Debug, clap::Subcommand)]
enum PacksCommand {
    /// Every pack this build provides, then the data packs in each pack directory.
    List {
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Every package field of one pack.
    Show {
        /// The pack's id, such as mineworld-presence.
        id: String,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Check one data pack: its package fields, all required, its licence, then its content.
    Validate {
        /// The pack's directory.
        directory: PathBuf,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// A world's composition: every requirement and the pack that met it, every enabled system's
    /// pack (ARC-54).
    Resolve {
        /// The World Pack directory.
        world: PathBuf,
        #[command(flatten)]
        packs: PackDirs,
    },
}

/// The pack roots a command resolves a world's requirements in (`docs/DECISIONS.md` `ARC-54`): each
/// `--packs DIR` in order, then `MINEWORLD_PACKS`. The environment is read here, in the composition
/// root, and nowhere else.
#[derive(Debug, clap::Args)]
struct PackDirs {
    /// A directory whose immediate subdirectories are packs, searched for the world's requires:.
    /// Repeat it for more; MINEWORLD_PACKS adds more after these.
    #[arg(long = "packs", value_name = "DIR")]
    dirs: Vec<PathBuf>,
}

impl PackDirs {
    fn roots(self) -> Result<PackRoots, String> {
        PackRoots::new(self.dirs, std::env::var_os(PACKS_VARIABLE).as_deref())
            .map_err(|refusal| format!("[mineworld] {refusal}"))
    }
}

/// A seat name on the command line, checked as the key it must be.
fn seat(text: &str) -> Result<EntityKey, String> {
    EntityKey::new(text).map_err(|error| format!("'{text}' is not a seat name: {error}"))
}

#[tokio::main]
async fn main() -> ExitCode {
    let outcome = match Cli::parse().command {
        Subcommand::Validate { world, packs } => {
            packs.roots().and_then(|roots| validate(&world, &roots))
        }
        Subcommand::Replay { world, save, packs } => packs
            .roots()
            .and_then(|roots| replay(&world, &save, &roots)),
        Subcommand::Server {
            world,
            listen,
            invite,
            admin_token,
            agents,
            town,
            seed,
            pace,
            hold,
            time_scale,
            keyframe_every,
            save,
            packs,
        } => match packs.roots() {
            Ok(roots) => {
                serve::serve(serve::ServeRequest {
                    world,
                    listen,
                    invite,
                    admin_token,
                    agents,
                    town,
                    seed,
                    pace,
                    hold,
                    time_scale,
                    keyframe_every,
                    save,
                    roots,
                })
                .await
            }
            Err(refusal) => Err(refusal),
        },
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
            packs,
        } => packs.roots().and_then(|roots| {
            run::run(&run::RunRequest {
                world,
                seed,
                days,
                save,
                roots,
            })
        }),
        Subcommand::Biography {
            world,
            save,
            person,
            json,
            packs,
        } => packs.roots().and_then(|roots| {
            biography::biography(&biography::BiographyRequest {
                world: &world,
                save: &save,
                person: &person,
                json,
                roots: &roots,
            })
        }),
        Subcommand::Perceived(args) => perceived::perceived(args),
        Subcommand::Packs { command } => match command {
            PacksCommand::List { packs } => packs.roots().and_then(|roots| packs::list(&roots)),
            PacksCommand::Show { id, packs } => {
                packs.roots().and_then(|roots| packs::show(&id, &roots))
            }
            PacksCommand::Validate { directory, packs } => packs
                .roots()
                .and_then(|roots| packs::validate(&directory, &roots)),
            PacksCommand::Resolve { world, packs } => packs
                .roots()
                .and_then(|roots| packs::resolve(&world, &roots)),
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
         mineworld replay, mineworld run, mineworld inspect, mineworld biography, mineworld perceived, \
         mineworld create, \
         mineworld packs (see mineworld --help)."
    ))
}

/// Checks a pack and says what world it describes.
///
/// Loads it as well as reading it, deliberately: reading answers *is this pack internally
/// consistent*, and loading answers *can the world it describes be composed at all* — a dependency
/// between systems is the kernel's judgement, and a `validate` that skipped it would pass a pack the
/// server then refused to host.
fn validate(world: &PathBuf, roots: &PackRoots) -> Result<(), String> {
    let pack = WorldPack::read_with(world, roots).map_err(described)?;
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
    // Only for a world that states requires:, by the same rule (ARC-54 point 7).
    for required in &pack.composition().required {
        println!(
            "  requires   {} \"{}\" → {} ({})",
            required.identity.id,
            required.range,
            required.identity.version,
            packs::source(&required.source),
        );
    }
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

/// Re-executes a save's whole history from genesis and reports what it compared.
fn replay(world: &Path, save: &Path, roots: &PackRoots) -> Result<(), String> {
    let pack = WorldPack::read_with(world, roots).map_err(described)?;
    let backend = SqliteBackend::open(save, Durability::PowerLoss)
        .map_err(|error| format!("[mineworld] {error}"))?;
    let genesis = serve::saved_genesis(&backend).map_err(|error| format!("[mineworld] {error}"))?;
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
