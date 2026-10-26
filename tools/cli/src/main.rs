//! `mineworld` — the command that runs a world.
//!
//! ```text
//! mineworld server <world> [--listen ADDRESS] [--invite TOKEN] [OPTIONS]   (every flag: §8.1)
//!                                       host the pack (persisted with --save) for clients with the
//!                                       invite; in-server controllers drive seats nobody plays
//! mineworld validate <world>            load it, say what it is, and stop
//! mineworld replay <world> --save DIR   re-execute a save's whole history and check it
//! mineworld run <world> --headless --seed N --days N [--save DIR] [--decisions FILE]
//!               [--seat-model SEAT=MODEL]...
//!                                       run it headless, every seat its decision model (ARC-27, 84)
//! mineworld decisions <world> [--json]  each seat's decision model and memory choice (ARC-84)
//! mineworld inspect <save> [--last N]   what a save holds; every fact's cause checked (AC-9)
//! mineworld biography|perceived <world> --save DIR --person KEY [--json]
//!                                       a Person's biography (ARC-29), or what they perceived (ARC-43)
//! mineworld create <directory>          a new, minimal World Pack
//! mineworld packs list|show|validate|resolve
//!                                       package identities, and a world's composition (ARC-53, 54)
//! mineworld reference --out DIR | --check DIR
//!                                       the configuration reference generated from this build (ARC-90)
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

mod attributes;
mod attributes_resolve;
mod biography;
mod create;
mod decisions;
mod history;
mod hosted;
mod inspect;
mod interactions;
mod invite;
mod packs;
mod perceive;
mod perceived;
mod reference;
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
use mineworld_worldpack::{PackError, PackRoots, SettlementKind, WorldPack};

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
        /// The bearer token that opens /admin (else none); never printed; not the invite.
        #[arg(long, env = "MINEWORLD_ADMIN_TOKEN", hide_env_values = true)]
        admin_token: Option<String>,
        /// Issue a seat grant admitting only these seats (seat or seat+seat), with :take-over the
        /// right to take a held one; its token is generated and printed once; repeatable.
        #[arg(long = "grant", value_name = "SEATS[:take-over]", value_parser = invite::grant_flag)]
        grants: Vec<invite::GrantFlag>,
        /// Each seat's request budget: R submits a second sustained, B at once (B ≥ R); 0: none.
        #[arg(long, value_name = "R/B", value_parser = request_budget,
              default_value_t = Budget(Some(mineworld_server::RequestBudget::DEFAULT)))]
        request_budget: Budget,
        /// Drive that seat with the reactive rule controller whenever no player holds it; repeatable.
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
        cadence: NonZeroU32,
        /// How long a dropped connection's seat is held for its resume, in wall seconds; 0: none.
        #[arg(long, value_name = "SECONDS", default_value_t = 30)]
        hold: u32,
        /// How many world seconds pass per wall second. Default: the save's last day-length scale, then the
        /// World Pack's `hosting:`, then 1. Above 24 only with --solo.
        #[arg(long, value_name = "N")]
        time_scale: Option<NonZeroU32>,
        /// A single-player world: it listens on loopback only, and only it may fast-forward above 24x.
        #[arg(long)]
        solo: bool,
        /// Every Nth frame to a client is a whole observation; the others are deltas (DEP-15).
        #[arg(long, value_name = "N", default_value = "50")]
        keyframe_every: NonZeroU32,
        /// Keep the world in DIR/world.sqlite: created the first time, resumed where it stopped after.
        #[arg(long, value_name = "DIR")]
        save: Option<PathBuf>,
        /// Also stop, the same graceful way as Ctrl-C, when standard input ends (a launcher's pipe).
        #[arg(long)]
        stop_on_stdin_eof: bool,
        #[command(flatten)]
        decisions: DecisionFlags,
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
        /// World seconds between a seat's consults (default 900). A measurement knob for FX-24
        /// (`ARC-69` item 11): a multiple of the run's stride step (`--stride`).
        #[arg(long, value_name = "SECONDS", value_parser = clap::value_parser!(u64).range(1..))]
        pace: Option<u64>,
        /// World seconds between a walking seat's stride consults (default 30). A divisor of the pace;
        /// a run takes at most one seat fewer than the stride, so a larger world passes a larger one
        /// (`ARC-341`).
        #[arg(long, value_name = "SECONDS", value_parser = clap::value_parser!(u64).range(1..))]
        stride: Option<u64>,
        #[command(flatten)]
        decisions: DecisionFlags,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Each seat's decision model and memory choice, read as `run` reads them; starts no world.
    Decisions {
        /// The World Pack directory.
        world: PathBuf,
        #[command(flatten)]
        decisions: DecisionFlags,
        /// One JSON document, the form the cognition process reads.
        #[arg(long)]
        json: bool,
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
    /// What a World Pack's Interaction List resolves to: each configured section, base then regions,
    /// and each entity's class (ARC-63, ARC-64). Reads the pack only.
    Interactions {
        /// The World Pack directory.
        world: PathBuf,
        /// Only the resolution that applies at this place.
        #[arg(long, value_name = "KEY", value_parser = seat)]
        place: Option<EntityKey>,
        /// One JSON document, keys sorted.
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// A world's attribute catalogue, what each person resolves to, and whether it is clean
    /// (MODULE_SPEC §8.1, S30-c). Reads the pack only.
    Attributes {
        #[command(subcommand)]
        command: AttributesCommand,
    },
    /// Package identities: what each pack is, its version, licence and provenance (ARC-53).
    Packs {
        #[command(subcommand)]
        command: PacksCommand,
    },
    /// Write the configuration reference generated from this build's code — every field a World
    /// Pack's files may set — or check a directory against it (ARC-90). Loads no world.
    #[command(group(clap::ArgGroup::new("target").required(true).args(["out", "check"])))]
    Reference {
        /// Write the reference into DIR, creating it; refused when DIR holds a file it did not write.
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
        /// Compare DIR with what this build generates; exit 1 naming each file that differs, is
        /// missing or is extra.
        #[arg(long, value_name = "DIR")]
        check: Option<PathBuf>,
    },
}

/// What `mineworld attributes` was asked to do.
#[derive(Debug, clap::Subcommand)]
enum AttributesCommand {
    /// The declarations in effect, in catalogue order, and a count per namespace.
    Catalogue {
        /// The World Pack directory.
        world: PathBuf,
        /// One JSON document.
        #[arg(long)]
        json: bool,
        /// Only the declarations carrying this facet.
        #[arg(long, value_name = "FACET")]
        facet: Option<String>,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Every person's resolved values with their origin, and the holder's and another observer's lines.
    Show {
        /// The World Pack directory.
        world: PathBuf,
        /// Only this person, by authoring key.
        #[arg(long, value_name = "KEY", value_parser = seat)]
        person: Option<EntityKey>,
        /// One JSON document.
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Everything catalogue validates, restrictions never drawn, counts, verify marks, the built-in
    /// catalogue's cap and reader lines, and every pack's uses.
    Check {
        /// The World Pack directory.
        world: PathBuf,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// The person-generation backends the world's packs provide, and the one it uses (S30-e).
    Backends {
        /// The World Pack directory.
        world: PathBuf,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Every person's resolved values with their origins and the profile's report (C-1 … C-7, S30-d).
    Resolve {
        /// The World Pack directory.
        world: PathBuf,
        /// One JSON document.
        #[arg(long)]
        json: bool,
        /// Only this person, by authoring key.
        #[arg(long, value_name = "KEY", value_parser = seat)]
        person: Option<EntityKey>,
        /// Only values whose declaration carries this facet.
        #[arg(long, value_name = "FACET")]
        facet: Option<String>,
        /// A salt instead of the configuration's: changes sampled values only.
        #[arg(long, value_name = "N")]
        salt: Option<u32>,
        /// N synthetic people instead of the world's, for the report only.
        #[arg(long, value_name = "N")]
        synthetic: Option<u32>,
        #[command(flatten)]
        packs: PackDirs,
    },
    /// Write or replace one entry of the world's lock (locks/<name>.yaml), in its canonical form.
    Lock {
        /// The World Pack directory.
        world: PathBuf,
        /// The person, by authoring key.
        #[arg(long, value_name = "KEY", value_parser = seat)]
        person: EntityKey,
        /// The attribute, qualified or by its unique short slug.
        #[arg(long, value_name = "ID")]
        attribute: String,
        /// ruled or locked.
        #[arg(long, value_name = "ORIGIN", value_parser = pinned)]
        origin: mineworld_attributes_model::lock::Pinned,
        /// When, YYYY-MM-DD.
        #[arg(long, value_name = "DATE")]
        at: String,
        /// By whom or what.
        #[arg(long, value_name = "TEXT")]
        by: Option<String>,
        /// The value, in the kind's YAML form; absent: the value resolve gives now.
        #[arg(long, value_name = "VALUE")]
        value: Option<String>,
        #[command(flatten)]
        packs: PackDirs,
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

/// The decision-list flags of `run`, `server` and `decisions` (`MODULE_SPEC.md` §4.1 rule 5).
#[derive(Debug, clap::Args)]
struct DecisionFlags {
    /// A decision list to use instead of the pack's decisions.yaml.
    #[arg(long = "decisions", value_name = "FILE")]
    file: Option<PathBuf>,
    /// Drive that seat with that installed decision model; repeatable.
    #[arg(long = "seat-model", value_name = "SEAT=MODEL", value_parser = decisions::seat_model)]
    seat_models: Vec<mineworld_decision::SeatOverride>,
}

impl DecisionFlags {
    fn args(self) -> decisions::DecisionArgs {
        decisions::DecisionArgs {
            file: self.file,
            seat_models: self.seat_models,
        }
    }
}

/// `--request-budget`, parsed: a budget, or none (`docs/DECISIONS.md` `ARC-97`).
#[derive(Debug, Clone, Copy)]
struct Budget(Option<mineworld_server::RequestBudget>);

/// The flag's own form, so `--help` shows the server's default from its one definition.
impl std::fmt::Display for Budget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(budget) => write!(f, "{}/{}", budget.per_second, budget.burst),
            None => f.write_str("0"),
        }
    }
}

/// `R/B` — two positive integers, `B ≥ R` — or `0` alone for no budget. Anything else is refused,
/// and clap's message names the value.
fn request_budget(text: &str) -> Result<Budget, String> {
    if text == "0" {
        return Ok(Budget(None));
    }
    let malformed = || {
        "a request budget is R/B — requests a second and a burst, positive integers with B ≥ R — or 0 \
         for none"
            .to_owned()
    };
    let (rate, burst) = text.split_once('/').ok_or_else(malformed)?;
    let positive = |part: &str| -> Result<NonZeroU32, String> {
        if part.bytes().all(|b| b.is_ascii_digit()) {
            part.parse().map_err(|_| malformed())
        } else {
            Err(malformed())
        }
    };
    let (per_second, burst) = (positive(rate)?, positive(burst)?);
    if burst < per_second {
        return Err(malformed());
    }
    Ok(Budget(Some(mineworld_server::RequestBudget {
        per_second,
        burst,
    })))
}

/// `ruled` or `locked`.
fn pinned(text: &str) -> Result<mineworld_attributes_model::lock::Pinned, String> {
    match text {
        "ruled" => Ok(mineworld_attributes_model::lock::Pinned::Ruled),
        "locked" => Ok(mineworld_attributes_model::lock::Pinned::Locked),
        other => Err(format!("'{other}' is not ruled or locked")),
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
            stop_on_stdin_eof,
            decisions,
            packs,
        } => match packs.roots() {
            Ok(roots) => {
                serve::serve(serve::ServeRequest {
                    world,
                    listen,
                    invite,
                    admin_token,
                    grants,
                    request_budget: request_budget.0,
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
                    decisions: decisions.args(),
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
            pace,
            stride,
            decisions,
            packs,
        } => packs.roots().and_then(|roots| {
            let seconds = |given: u64| {
                mineworld_contracts::SimDuration::from_seconds(
                    i64::try_from(given).unwrap_or(i64::MAX),
                )
            };
            run::run(&run::RunRequest {
                world,
                seed,
                days,
                save,
                roots,
                pace: pace.map_or(run::PACE, seconds),
                stride: stride.map_or(run::RUN_STEP, seconds),
                decisions: decisions.args(),
            })
        }),
        Subcommand::Decisions {
            world,
            decisions,
            json,
            packs,
        } => packs.roots().and_then(|roots| {
            decisions::decisions(&decisions::DecisionsRequest {
                world: &world,
                args: decisions.args(),
                json,
                roots: &roots,
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
        Subcommand::Interactions {
            world,
            place,
            json,
            packs,
        } => packs.roots().and_then(|roots| {
            interactions::interactions(&interactions::InteractionsRequest {
                world: &world,
                place: place.as_ref(),
                json,
                roots: &roots,
            })
        }),
        Subcommand::Attributes { command } => match command {
            AttributesCommand::Catalogue {
                world,
                json,
                facet,
                packs,
            } => packs
                .roots()
                .and_then(|roots| attributes::catalogue(&world, json, facet.as_deref(), &roots)),
            AttributesCommand::Show {
                world,
                person,
                json,
                packs,
            } => packs
                .roots()
                .and_then(|roots| attributes::show(&world, person.as_ref(), json, &roots)),
            AttributesCommand::Check { world, packs } => packs
                .roots()
                .and_then(|roots| attributes::check(&world, &roots)),
            AttributesCommand::Backends { world, packs } => packs
                .roots()
                .and_then(|roots| attributes::backends(&world, &roots)),
            AttributesCommand::Resolve {
                world,
                json,
                person,
                facet,
                salt,
                synthetic,
                packs,
            } => packs.roots().and_then(|roots| {
                attributes_resolve::resolve(&attributes_resolve::ResolveRequest {
                    world: &world,
                    json,
                    person: person.as_ref(),
                    facet: facet.as_deref(),
                    salt,
                    synthetic,
                    roots: &roots,
                })
            }),
            AttributesCommand::Lock {
                world,
                person,
                attribute,
                origin,
                at,
                by,
                value,
                packs,
            } => packs.roots().and_then(|roots| {
                attributes_resolve::lock(&attributes_resolve::LockRequest {
                    world: &world,
                    person: &person,
                    attribute: &attribute,
                    origin,
                    at: &at,
                    by: by.as_deref(),
                    value: value.as_deref(),
                    roots: &roots,
                })
            }),
        },
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
        Subcommand::Reference { out, check } => {
            let target = match (out, check) {
                (Some(dir), _) => reference::Target::Out(dir),
                (None, Some(dir)) => reference::Target::Check(dir),
                (None, None) => unreachable!("clap requires one of --out and --check"),
            };
            reference::reference(&target)
        }
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
         mineworld create, mineworld packs, mineworld reference (see mineworld --help)."
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
    // The authoring gate: every world says whether it is a town (ARC-105, MODULE_SPEC §4.3). `run` and
    // `server` do not ask, so a world without the key still runs.
    let settlement = pack.require_settlement().map_err(described)?;
    // A town's far field fills its slots with the declared places (ARC-107, S29-c).
    pack.check_far_field_slots(roots).map_err(described)?;

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
    match settlement.kind() {
        SettlementKind::Town { station, landmark } => {
            println!("  settlement town (station {station}, landmark {landmark})");
        }
        SettlementKind::None => println!("  settlement none"),
    }
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
    // Only for a world that names a decision list, by the same rule (MODULE_SPEC §8.1): decoded and
    // checked against this world as run decodes it.
    if pack.decisions().is_some() {
        let roster = decisions::roster(&pack);
        let (list, _) = decisions::list(&pack, &roster, None)?;
        println!(
            "  decisions  {} assignment(s), default {}",
            list.assignments(),
            list.default_model(),
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
