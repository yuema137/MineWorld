//! `mineworld-launch`: the downloadable bundle's supervisor (`DECISIONS.md` `ARC-78`; specification
//! [`LAUNCHER.md`](../LAUNCHER.md)).
//!
//! ```text
//! find the bundle and the per-user folder      bundle.rs, user.rs
//! read what this mode plays                    config.rs (defaults, launch.toml)
//! start the server, read its join line         children.rs Server, join.rs
//! start the client(s), wait for them           children.rs Clients
//! stop the server: close its standard input    Server::stop (mineworld server --stop-on-stdin-eof)
//! ```
//!
//! A process supervisor and nothing else: it reads nothing from the server but the join line, holds no
//! world state and no world rule, and stops only the processes it started.

// `deny`, not `forbid`: Windows' `MessageBoxW` in `dialog` — its one declaration and its one call —
// allows it.
#![deny(unsafe_code)]

mod bundle;
mod children;
mod cognition;
mod config;
mod dialog;
mod hosting;
mod join;
mod user;

use std::ffi::OsString;
use std::fmt;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::ExitCode;

use bundle::Bundle;
use children::{Clients, Server};
use cognition::{Cognition, Reason, Setup};
use config::Choice;
use hosting::Hosting;
use user::{Log, Run, UserDir};

/// What an entry point plays (`LAUNCHER.md` §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// "MineWorld 2D".
    TwoD,
    /// "MineWorld 3D".
    ThreeD,
    /// "MineWorld 2D + 3D": both clients on one server.
    Both,
}

/// Why a run failed, in words a player can act on. Shown once, with the log to read.
#[derive(Debug)]
pub struct Failure(String);

impl Failure {
    fn new(message: String) -> Self {
        Self(message)
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The command line (`LAUNCHER.md` §3).
#[derive(Debug, Default)]
struct Options {
    fresh: bool,
    smoke: Option<Mode>,
    dialog: bool,
    bundle: Option<PathBuf>,
    user_dir: Option<PathBuf>,
}

impl Options {
    fn parse(arguments: impl Iterator<Item = OsString>) -> Result<Self, Failure> {
        let mut options = Self {
            dialog: true,
            ..Self::default()
        };
        for argument in arguments {
            let text = argument.to_string_lossy();
            // macOS before 10.9 passed a process serial number to an app opened from the Finder.
            if text.starts_with("-psn_") {
                continue;
            }
            if text == "--fresh" {
                options.fresh = true;
            } else if text == "--no-dialog" {
                options.dialog = false;
            } else if let Some(mode) = text.strip_prefix("--smoke=") {
                options.smoke = Some(match mode {
                    "2d" => Mode::TwoD,
                    "3d" => Mode::ThreeD,
                    "both" => Mode::Both,
                    other => {
                        return Err(Failure::new(format!(
                            "--smoke={other}: expected 2d, 3d or both"
                        )));
                    }
                });
            } else if let Some(path) = path_value(&argument, "--bundle=") {
                options.bundle = Some(path);
            } else if let Some(path) = path_value(&argument, "--user-dir=") {
                options.user_dir = Some(path);
            } else {
                return Err(Failure::new(format!(
                    "unknown argument {text}; expected --fresh, --smoke=2d|3d|both, --no-dialog, \
                     --bundle=<dir> or --user-dir=<dir>"
                )));
            }
        }
        Ok(options)
    }
}

/// The path after `prefix` in `argument`, kept as the OS gave it (a folder name need not be UTF-8).
fn path_value(argument: &OsString, prefix: &str) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        argument
            .as_bytes()
            .strip_prefix(prefix.as_bytes())
            .map(|rest| PathBuf::from(std::ffi::OsStr::from_bytes(rest)))
    }
    #[cfg(not(unix))]
    {
        argument.to_str()?.strip_prefix(prefix).map(PathBuf::from)
    }
}

/// The entry point of every binary: runs `mode`, and turns a failure into a message, a dialog and exit
/// status 1 (`LAUNCHER.md` §9).
pub fn main(mode: Mode) -> ExitCode {
    let mut log = Log::stderr_only();
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    let (outcome, dialog) = match Options::parse(arguments.iter().cloned()) {
        Ok(options) => (
            run(mode, &options, &mut log),
            options.dialog && options.smoke.is_none(),
        ),
        Err(failure) => (
            Err(failure),
            !arguments.iter().any(|argument| argument == "--no-dialog"),
        ),
    };
    match outcome {
        Ok(code) => ExitCode::from(code),
        Err(failure) => {
            let message = match log.path() {
                Some(path) => format!("{failure}\n\nThe launcher's log: {}", path.display()),
                None => failure.to_string(),
            };
            log.line(&format!("FAILED: {message}"));
            if dialog {
                dialog::show(&message);
            }
            ExitCode::from(1)
        }
    }
}

/// One run (`LAUNCHER.md` §7). Returns the exit status: the clients' in a smoke run, else 0.
fn run(mode: Mode, options: &Options, log: &mut Log) -> Result<u8, Failure> {
    let run = Run::now();
    let user = UserDir::open(options.user_dir.clone())?;
    log.write_to(&user.log(run, "launch"))?;
    user.prune_logs();
    let bundle = Bundle::locate(options.bundle.clone())?;

    let played = options.smoke.unwrap_or(mode);
    // A smoke run plays its fixed fixture and never reads launch.toml or the cognition profile
    // (`pr-launcher-solo.md` Q-LS5; `LAUNCHER.md` §10.1).
    let profile = if options.smoke.is_some() {
        None
    } else {
        read_optional(&user.profile())?
    };
    let (choice, keys) = match (options.smoke, &profile) {
        (Some(smoke), _) => (config::smoke(smoke), None),
        (None, Some(text)) => config::profile(mode, &user.profile().display().to_string(), text)
            .map_err(Failure::new)?,
        (None, None) => (chosen(&bundle, mode)?, None),
    };
    let world = bundle.world(&choice.world)?;
    let clients = choice
        .clients
        .iter()
        .map(|(client, seat)| Ok((*client, seat.clone(), bundle.client(*client)?)))
        .collect::<Result<Vec<_>, Failure>>()?;

    // Declared before the server, so that it is removed after the server has stopped (drop order).
    let scratch;
    let save = if options.smoke.is_some() {
        scratch = Scratch(user.scratch(run));
        scratch.0.clone()
    } else {
        let save = user.save(&choice.world);
        if options.fresh {
            // B1: a world's memories go with its save, so none outlives the world it belongs to.
            for folder in [save.clone(), user.store(&choice.world)?] {
                if folder.exists() {
                    std::fs::remove_dir_all(&folder).map_err(|error| {
                        Failure::new(format!(
                            "cannot start over: removing {}: {error}",
                            folder.display()
                        ))
                    })?;
                    log.line(&format!("--fresh: removed {}", folder.display()));
                }
            }
        }
        save
    };
    log.line(&format!(
        "run {run}: {} on {} (save {})",
        label(played, options.smoke.is_some()),
        choice.world,
        save.display()
    ));

    // One player is a single-player world; Both seats two players and is shared (Q-LS2).
    let hosting = Hosting::new(options.smoke.is_some(), mode != Mode::Both)?;
    let setup = match keys {
        Some(keys) => conversation(
            &bundle,
            keys,
            user.store(&choice.world)?,
            &user.log(run, "preflight"),
            log,
        ),
        None => None,
    };
    let mut server = Server::start(
        &bundle,
        &world,
        &save,
        &hosting,
        setup.as_ref().map(|setup| setup.decisions.as_path()),
        &user.log(run, "server"),
        log,
    )?;
    let join = server.join()?;
    log.line(&format!("the server is ready at {}", join.address));
    // Declared after the server, so that a failure stops it first (drop order); the clients never wait
    // for it (`LAUNCHER.md` §10.4).
    let cognition = setup.and_then(|setup| {
        let humans = choice
            .clients
            .iter()
            .map(|(_, seat)| seat.clone())
            .collect();
        let started = setup.assignment(&bundle, &world).and_then(|assignment| {
            Cognition::start(
                &setup,
                &join,
                &assignment,
                humans,
                &user.log(run, "cognition"),
                log,
            )
        });
        started.map_err(|reason| off(log, reason)).ok()
    });
    let code = Clients {
        bundle: &bundle,
        join: &join,
        world: &choice.world,
        hosting: &hosting,
    }
    .run(
        &clients,
        |client| user.log(run, &format!("client-{}", client.label())),
        log,
    )?;
    // The last client has exited: cognition leaves its seat and stops, then the server checkpoints.
    if let Some(cognition) = cognition {
        cognition.stop();
    }
    server.stop(log)?;
    Ok(if options.smoke.is_some() { code } else { 0 })
}

/// What `mode` plays in this bundle: its defaults, overridden by `runtime/launch.toml` when present.
fn chosen(bundle: &Bundle, mode: Mode) -> Result<Choice, Failure> {
    let overrides = bundle.overrides();
    let text = read_optional(&overrides)?;
    let named = overrides.display().to_string();
    config::choice(mode, text.as_deref().map(|text| (named.as_str(), text))).map_err(Failure::new)
}

/// A file's text, or `None` when there is no such file.
fn read_optional(path: &std::path::Path) -> Result<Option<String>, Failure> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Failure::new(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

/// The profile's cognition keys for this world, with its store created; `None`, logged, when
/// conversation cannot be set up and the world plays rule-only.
fn conversation(
    bundle: &Bundle,
    keys: config::CognitionKeys,
    store: PathBuf,
    preflight_log: &std::path::Path,
    log: &mut Log,
) -> Option<Setup> {
    let setup = match Setup::new(bundle, keys, store) {
        Ok(setup) => setup,
        Err(reason) => {
            off(log, reason);
            return None;
        }
    };
    if let Err(error) = std::fs::create_dir_all(&setup.store) {
        log.line(&format!(
            "cognition: cannot create {}: {error}",
            setup.store.display()
        ));
        off(log, Reason::ProfileInvalid);
        return None;
    }
    match setup.preflight(preflight_log) {
        Ok(()) => Some(setup),
        Err(reason) => {
            off(log, reason);
            None
        }
    }
}

/// Conversation is off for this run: the world plays with its rule controllers.
fn off(log: &mut Log, reason: Reason) {
    log.line(&format!(
        "cognition: degraded({reason}): residents play by their rules; {}",
        reason.advice()
    ));
}

fn label(mode: Mode, smoke: bool) -> String {
    let name = match mode {
        Mode::TwoD => "2D",
        Mode::ThreeD => "3D",
        Mode::Both => "2D + 3D",
    };
    if smoke {
        format!("smoke check {name}")
    } else {
        name.to_owned()
    }
}

/// A smoke run's save, removed when the run ends, pass or fail.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
        // The `scratch` folder itself, once no run is using it.
        if let Some(parent) = self.0.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(arguments: &[&str]) -> Result<Options, Failure> {
        Options::parse(arguments.iter().map(OsString::from))
    }

    #[test]
    fn the_command_line_takes_exactly_its_five_arguments() {
        let options = parsed(&[
            "--fresh",
            "--smoke=both",
            "--no-dialog",
            "--bundle=/b c",
            "--user-dir=/u 世界",
            "-psn_0_12345",
        ])
        .expect("valid");
        assert!(options.fresh && !options.dialog);
        assert_eq!(options.smoke, Some(Mode::Both));
        assert_eq!(options.bundle, Some(PathBuf::from("/b c")));
        assert_eq!(options.user_dir, Some(PathBuf::from("/u 世界")));

        assert!(parsed(&[]).expect("no arguments").dialog);
        for wrong in ["--smoke=4d", "--bundle", "--server=127.0.0.1:1", "play"] {
            assert!(parsed(&[wrong]).is_err(), "{wrong} is refused");
        }
    }
}
