//! Conversation for a played run (`LAUNCHER.md` §10; design
//! `.structured-coding/plans/mvp0/pr-playable-dialogue-memory.md` §3): the per-user profile's cognition
//! keys, the world's cognition store, and the assignment the cognition process reads.
//!
//! The launcher handles only opaque paths and the cognition process's closed status vocabulary
//! (`docs/reference/cognition-supervised.md` §2). It never reads the cognition configuration, names no
//! model, endpoint or key, and conversation failing never fails the run: the world plays rule-only.

use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::bundle::Bundle;
use crate::children::{log_files, no_window};
use crate::config::{CognitionKeys, Seat};
use crate::join::Join;
use crate::user::Log;

/// How long `mineworld decisions --json`, and `--preflight`, may take before it is killed.
const ASSIGNMENT_PATIENCE: Duration = Duration::from_secs(30);
const PREFLIGHT_PATIENCE: Duration = Duration::from_secs(30);
/// How long the process has to hold its seat and home (`ready`) before it is stopped.
const READINESS_PATIENCE: Duration = Duration::from_secs(60);
/// How long the process has to stop after its standard input closes: its own forced exit is at 15 s
/// (`mineworld_cognition.shutdown.FORCE_AFTER_S`), plus a margin; then this child is killed.
const STOP_PATIENCE: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(50);
/// The longest status line read (`docs/reference/cognition-supervised.md` §2).
const STATUS_BYTES: usize = 512;
/// The status line format this launcher reads.
const STATUS_FORMAT: &str = "1";

/// Why conversation is off for this run (`docs/reference/cognition-supervised.md` §2.3): a closed set,
/// carrying no endpoint, model, token, invite or path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    // Sent by the cognition process.
    ConfigInvalid,
    AssignmentInvalid,
    RuntimeMismatch,
    BackendNotLoopback,
    HomeBusy,
    HomeIdentityMismatch,
    InstanceMismatch,
    ServerUnreachable,
    SeatUnavailable,
    ModelUnavailable,
    Stopping,
    // The launcher's own; never accepted from the process.
    ReadinessTimeout,
    ChildExited,
    StatusInvalid,
    PreflightFailed,
    ProfileInvalid,
}

impl Reason {
    /// The reasons the cognition process may send, by their wire names.
    const SENT: [(&'static str, Reason); 11] = [
        ("config_invalid", Reason::ConfigInvalid),
        ("assignment_invalid", Reason::AssignmentInvalid),
        ("runtime_mismatch", Reason::RuntimeMismatch),
        ("backend_not_loopback", Reason::BackendNotLoopback),
        ("home_busy", Reason::HomeBusy),
        ("home_identity_mismatch", Reason::HomeIdentityMismatch),
        ("instance_mismatch", Reason::InstanceMismatch),
        ("server_unreachable", Reason::ServerUnreachable),
        ("seat_unavailable", Reason::SeatUnavailable),
        ("model_unavailable", Reason::ModelUnavailable),
        ("stopping", Reason::Stopping),
    ];

    /// A reason the cognition process sent, or `None` for anything else — the launcher's own included.
    pub fn sent(name: &str) -> Option<Self> {
        Self::SENT
            .iter()
            .find(|(wire, _)| *wire == name)
            .map(|(_, reason)| *reason)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadinessTimeout => "readiness_timeout",
            Self::ChildExited => "child_exited",
            Self::StatusInvalid => "status_invalid",
            Self::PreflightFailed => "preflight_failed",
            Self::ProfileInvalid => "profile_invalid",
            sent => Self::SENT
                .iter()
                .find(|(_, reason)| *reason == sent)
                .map_or("?", |(wire, _)| wire),
        }
    }

    /// What the player can do about it, for the launcher's log.
    pub fn advice(self) -> &'static str {
        match self {
            Self::ProfileInvalid => "check cognition/profile.toml",
            Self::PreflightFailed | Self::RuntimeMismatch => {
                "regenerate the runtime manifest with the profile's interpreter \
                 (python -m mineworld_cognition --write-runtime-manifest <file>)"
            }
            Self::ModelUnavailable => "start the local model service and pull the configured model",
            Self::ConfigInvalid | Self::BackendNotLoopback => "check the cognition configuration",
            Self::InstanceMismatch => {
                "this world's memories belong to another world instance; start it over with --fresh"
            }
            Self::HomeBusy => "another cognition process holds this world's memories",
            _ => "see the cognition log",
        }
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The profile's cognition keys for one world, resolved against the bundle and the per-user folder.
#[derive(Debug)]
pub struct Setup {
    pub python: PathBuf,
    pub config: PathBuf,
    pub manifest: PathBuf,
    /// The decision list, inside the bundle; the server and the assignment are given the same file.
    pub decisions: PathBuf,
    /// `cognition/<world>/`, the world's store (`LAUNCHER.md` §10.3).
    pub store: PathBuf,
}

impl Setup {
    /// The keys resolved, or `ProfileInvalid` when the bundle does not hold the decision list.
    pub fn new(bundle: &Bundle, keys: CognitionKeys, store: PathBuf) -> Result<Self, Reason> {
        let decisions = bundle.file(&keys.decisions).ok_or(Reason::ProfileInvalid)?;
        Ok(Self {
            python: keys.python,
            config: keys.config,
            manifest: keys.manifest,
            decisions,
            store,
        })
    }

    /// `cognition/<world>/run/`, rewritten by every run.
    fn run_folder(&self) -> PathBuf {
        self.store.join("run")
    }

    /// The interpreter, run directly with `-I` and an explicit argument list: no shell, no `uv run`.
    fn command(&self) -> Result<Command, Reason> {
        // Windows: the profile names the interpreter itself, never a launcher script or a shim.
        let named_exe = !cfg!(windows)
            || self
                .python
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));
        if !named_exe || !self.python.is_file() {
            return Err(Reason::PreflightFailed);
        }
        let mut command = Command::new(&self.python);
        command
            .args(["-I", "-m", "mineworld_cognition", "--config"])
            .arg(&self.config)
            // Neither child capability is the cognition process's: the admin token never, the invite only
            // when it joins (`Cognition::start`).
            .env_remove("MINEWORLD_ADMIN_TOKEN")
            .env_remove("MINEWORLD_INVITE");
        no_window(&mut command);
        Ok(command)
    }

    /// Runs `--preflight` (`docs/reference/cognition-supervised.md` §3) before the server starts: the
    /// runtime against its manifest, the backends' hosts, the local model. Its one status line names
    /// what failed; anything else — a missing interpreter, no line, a crash — is `PreflightFailed`.
    pub fn preflight(&self, log_path: &Path) -> Result<(), Reason> {
        let mut command = self.command()?;
        let (_, err) = log_files(log_path).map_err(|_| Reason::PreflightFailed)?;
        command
            .arg("--preflight")
            .arg(&self.manifest)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(err);
        let mut child = command.spawn().map_err(|_| Reason::PreflightFailed)?;
        let mut out = child.stdout.take().ok_or(Reason::PreflightFailed)?;
        let reader = std::thread::spawn(move || {
            let mut text = Vec::new();
            let _ = (&mut out)
                .take(STATUS_BYTES as u64 * 4)
                .read_to_end(&mut text);
            text
        });
        let status = wait_bounded(&mut child, PREFLIGHT_PATIENCE);
        let printed = reader.join().unwrap_or_default();
        let line = String::from_utf8_lossy(&printed);
        let parsed = line.lines().next().map(Status::parse);
        match (status, parsed) {
            (Some(code), Some(Ok(status))) if code.success() && status.state == State::Starting => {
                Ok(())
            }
            (
                Some(_),
                Some(Ok(Status {
                    state: State::Degraded,
                    reason: Some(reason),
                    ..
                })),
            ) => Err(reason),
            _ => Err(Reason::PreflightFailed),
        }
    }

    /// Writes `run/assignment.json`: the owned server binary's `decisions <world> --decisions <file>
    /// --json`, the format-3 document of the same decision list the server was started with. A command
    /// that fails, or runs past its patience (then killed), is `AssignmentInvalid`.
    pub fn assignment(&self, bundle: &Bundle, world: &Path) -> Result<PathBuf, Reason> {
        let folder = self.run_folder();
        std::fs::create_dir_all(&folder).map_err(|_| Reason::AssignmentInvalid)?;
        let path = folder.join("assignment.json");
        let out = File::create(&path).map_err(|_| Reason::AssignmentInvalid)?;
        let mut command = Command::new(bundle.server());
        command
            .arg("decisions")
            .arg(world)
            .arg("--decisions")
            .arg(&self.decisions)
            .arg("--json")
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(Stdio::null());
        no_window(&mut command);
        let mut child = command.spawn().map_err(|_| Reason::AssignmentInvalid)?;
        match wait_bounded(&mut child, ASSIGNMENT_PATIENCE) {
            Some(status) if status.success() => Ok(path),
            _ => Err(Reason::AssignmentInvalid),
        }
    }
}

/// Waits for an owned short child; past `patience` it is killed and reaped (only this child, never a
/// process found by name), and `None` is returned.
fn wait_bounded(child: &mut Child, patience: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + patience;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

/// A status line's state (`docs/reference/cognition-supervised.md` §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Starting,
    Ready,
    Degraded,
    Stopped,
}

/// One status line, read strictly: a flat JSON object with exactly the keys `status`, `state`, `seat`
/// and `reason`, whose strings hold no escape (the closed vocabulary and seat keys need none). Read here
/// rather than by a JSON library, as `launch.toml` is (`ARC-78`): four flat keys do not justify one.
#[derive(Debug, PartialEq, Eq)]
struct Status {
    state: State,
    seat: Option<String>,
    reason: Option<Reason>,
}

impl Status {
    fn parse(line: &str) -> Result<Self, ()> {
        if line.len() > STATUS_BYTES {
            return Err(());
        }
        let inner = line
            .trim()
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .ok_or(())?;
        let mut fields: Vec<(&str, Option<&str>)> = Vec::new();
        for pair in inner.split(',') {
            let (key, value) = pair.split_once(':').ok_or(())?;
            let key = quoted(key.trim()).ok_or(())?;
            let value = value.trim();
            let value = if value == "null" {
                None
            } else {
                Some(quoted(value).or_else(|| number(value)).ok_or(())?)
            };
            if fields.iter().any(|(seen, _)| *seen == key) {
                return Err(());
            }
            fields.push((key, value));
        }
        let field = |name: &str| fields.iter().find(|(key, _)| *key == name).map(|(_, v)| *v);
        if fields.len() != 4 || field("status") != Some(Some(STATUS_FORMAT)) {
            return Err(());
        }
        let state = match field("state").ok_or(())? {
            Some("starting") => State::Starting,
            Some("ready") => State::Ready,
            Some("degraded") => State::Degraded,
            Some("stopped") => State::Stopped,
            _ => return Err(()),
        };
        let seat = field("seat").ok_or(())?.map(str::to_owned);
        if seat.as_deref().is_some_and(|key| Seat::new(key).is_err()) {
            return Err(());
        }
        let reason = match field("reason").ok_or(())? {
            None => None,
            Some(name) => Some(Reason::sent(name).ok_or(())?),
        };
        // A reason exactly when the process is degraded or stopped.
        if reason.is_some() != matches!(state, State::Degraded | State::Stopped) {
            return Err(());
        }
        Ok(Self {
            state,
            seat,
            reason,
        })
    }
}

/// `"text"` with no quote or backslash inside, as text.
fn quoted(value: &str) -> Option<&str> {
    value
        .strip_prefix('"')?
        .strip_suffix('"')
        .filter(|inner| !inner.contains(['"', '\\']))
}

fn number(value: &str) -> Option<&str> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())).then_some(value)
}

/// The supervised cognition process of a played run (`LAUNCHER.md` §10.4): started after the server's
/// join line, watched by its own thread, stopped through its standard input before the server is.
pub struct Cognition {
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stopping: Arc<AtomicBool>,
    monitor: Option<JoinHandle<()>>,
}

impl Cognition {
    /// Starts `<python> -I -m mineworld_cognition --supervised …` with an environment built here and
    /// nowhere else: no admin token, the invite in `MINEWORLD_INVITE`, standard output the status pipe,
    /// standard error `log_path`. A process that cannot be started leaves the run rule-only.
    pub fn start(
        setup: &Setup,
        join: &Join,
        assignment: &Path,
        humans: Vec<Seat>,
        log_path: &Path,
        log: &Log,
    ) -> Result<Self, Reason> {
        let mut command = setup.command()?;
        let (_, err) = log_files(log_path).map_err(|_| Reason::ChildExited)?;
        command
            .arg("--supervised")
            .arg(format!("--server-url=ws://{}/ws", join.address))
            .arg("--store-dir")
            .arg(&setup.store)
            .arg("--assignment")
            .arg(assignment)
            .env("MINEWORLD_INVITE", &join.invite)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(err);
        let mut child = command.spawn().map_err(|_| Reason::ChildExited)?;
        let mut log = log.clone();
        log.line(&format!(
            "cognition: started (pid {}), its log: {}",
            child.id(),
            log_path.display()
        ));
        let stdin = Arc::new(Mutex::new(child.stdin.take()));
        let lines = read_lines(child.stdout.take());
        let stopping = Arc::new(AtomicBool::new(false));
        let watch = Watch {
            child,
            lines,
            stdin: Arc::clone(&stdin),
            stopping: Arc::clone(&stopping),
            humans,
            log,
        };
        let monitor = std::thread::Builder::new()
            .name("cognition".to_owned())
            .spawn(move || watch.run())
            .map_err(|_| Reason::ChildExited)?;
        Ok(Self {
            stdin,
            stopping,
            monitor: Some(monitor),
        })
    }

    /// Closes the process's standard input and waits for its watcher to see it stop (killed after
    /// `STOP_PATIENCE`). Called when the last client has exited, before the server is stopped.
    pub fn stop(mut self) {
        self.end();
    }

    fn end(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        close(&self.stdin);
        if let Some(monitor) = self.monitor.take() {
            let _ = monitor.join();
        }
    }
}

impl Drop for Cognition {
    /// A failure after the process started stops it the same way, before the server.
    fn drop(&mut self) {
        self.end();
    }
}

fn close(stdin: &Mutex<Option<ChildStdin>>) {
    if let Ok(mut stdin) = stdin.lock() {
        drop(stdin.take());
    }
}

/// The status pipe, line by line, on its own thread; a line longer than the bound arrives cut, and is
/// refused as malformed.
fn read_lines(out: Option<std::process::ChildStdout>) -> Receiver<String> {
    let (send, receive) = mpsc::channel();
    if let Some(out) = out {
        std::thread::spawn(move || {
            let mut reader = BufReader::new(out);
            loop {
                let mut line = Vec::new();
                let read = (&mut reader)
                    .take(STATUS_BYTES as u64 + 1)
                    .read_until(b'\n', &mut line);
                if !matches!(read, Ok(n) if n > 0) {
                    return;
                }
                if send
                    .send(String::from_utf8_lossy(&line).into_owned())
                    .is_err()
                {
                    return;
                }
            }
        });
    }
    receive
}

/// The watcher's side: the child, its status lines, and the launcher's states (`LAUNCHER.md` §10.5).
struct Watch {
    child: Child,
    lines: Receiver<String>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    stopping: Arc<AtomicBool>,
    humans: Vec<Seat>,
    log: Log,
}

impl Watch {
    fn run(mut self) {
        let ready_by = Instant::now() + READINESS_PATIENCE;
        let mut state = State::Starting;
        loop {
            match self.lines.recv_timeout(POLL) {
                Ok(line) => match Status::parse(line.trim_end_matches(['\r', '\n'])) {
                    Ok(status) => {
                        if let Some(reason) = self.refused(&status) {
                            return self.degrade(reason);
                        }
                        state = self.observe(state, &status);
                    }
                    Err(()) => return self.degrade(Reason::StatusInvalid),
                },
                Err(RecvTimeoutError::Timeout) => {}
                // The pipe closed: the process is ending. Wait for its exit without spinning.
                Err(RecvTimeoutError::Disconnected) => std::thread::sleep(POLL),
            }
            if self.stopping.load(Ordering::SeqCst) {
                return self.finish(state);
            }
            if let Ok(Some(exit)) = self.child.try_wait() {
                if state != State::Degraded {
                    self.log.line(&format!(
                        "cognition: degraded({}): the process exited ({exit}); residents play by \
                         their rules",
                        Reason::ChildExited
                    ));
                }
                return;
            }
            if state == State::Starting && Instant::now() >= ready_by {
                return self.degrade(Reason::ReadinessTimeout);
            }
        }
    }

    /// A status the launcher refuses: the process's seat is a human seat of this mode.
    fn refused(&self, status: &Status) -> Option<Reason> {
        let seat = status.seat.as_deref()?;
        self.humans
            .iter()
            .any(|human| human.as_str() == seat)
            .then_some(Reason::ProfileInvalid)
    }

    fn observe(&mut self, was: State, status: &Status) -> State {
        if status.state != was {
            let seat = status.seat.as_deref().unwrap_or("-");
            match status.reason {
                Some(reason) if status.state == State::Degraded => self.log.line(&format!(
                    "cognition: degraded({reason}) for {seat}: residents play by their rules; {}",
                    reason.advice()
                )),
                _ => self.log.line(&format!(
                    "cognition: {} for {seat}",
                    match status.state {
                        State::Starting => "starting",
                        State::Ready => "ready: the seat and its memory are held",
                        State::Degraded => "degraded",
                        State::Stopped => "stopped",
                    }
                )),
            }
        }
        status.state
    }

    /// The launcher's own degraded state: logged, and the process stopped.
    fn degrade(mut self, reason: Reason) {
        self.log.line(&format!(
            "cognition: degraded({reason}): residents play by their rules; {}",
            reason.advice()
        ));
        self.finish(State::Degraded);
    }

    /// Closes standard input and waits for the exit; past `STOP_PATIENCE` the child is killed.
    fn finish(mut self, state: State) {
        close(&self.stdin);
        let deadline = Instant::now() + STOP_PATIENCE;
        loop {
            // Keep draining, so a process writing its last lines never blocks on a full pipe.
            while let Ok(line) = self.lines.try_recv() {
                if let Ok(status) = Status::parse(line.trim_end_matches(['\r', '\n'])) {
                    self.observe(state, &status);
                }
            }
            match self.child.try_wait() {
                Ok(Some(exit)) => {
                    self.log
                        .line(&format!("cognition: process exited ({exit})"));
                    return;
                }
                Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL),
                _ => break,
            }
        }
        // Only this child, never a process found by name; a model service is never touched.
        let _ = self.child.kill();
        let exit = self.child.wait();
        self.log.line(&format!(
            "cognition: did not stop within {} s and was killed ({})",
            STOP_PATIENCE.as_secs(),
            exit.map_or_else(|error| error.to_string(), |exit| exit.to_string())
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reason_is_accepted_from_the_process_only_when_the_process_may_send_it() {
        for (wire, reason) in Reason::SENT {
            assert_eq!(Reason::sent(wire), Some(reason));
            assert_eq!(reason.as_str(), wire);
        }
        for own in [
            Reason::ReadinessTimeout,
            Reason::ChildExited,
            Reason::StatusInvalid,
            Reason::PreflightFailed,
            Reason::ProfileInvalid,
        ] {
            assert_eq!(Reason::sent(own.as_str()), None, "{own} is the launcher's");
        }
        assert_eq!(Reason::sent("http://127.0.0.1:11434"), None);
    }

    #[test]
    fn a_status_line_is_read_strictly_and_anything_else_is_malformed() {
        // The lines `mineworld_cognition.supervised.Status` writes (`json.dumps`, default separators).
        assert_eq!(
            Status::parse(r#"{"status": 1, "state": "ready", "seat": "alice", "reason": null}"#),
            Ok(Status {
                state: State::Ready,
                seat: Some("alice".to_owned()),
                reason: None
            })
        );
        assert_eq!(
            Status::parse(
                r#"{"status": 1, "state": "degraded", "seat": null, "reason": "home_busy"}"#
            ),
            Ok(Status {
                state: State::Degraded,
                seat: None,
                reason: Some(Reason::HomeBusy)
            })
        );
        for line in [
            r#"{"status": 2, "state": "ready", "seat": "alice", "reason": null}"#,
            r#"{"status": 1, "state": "ready", "seat": "alice"}"#,
            r#"{"status": 1, "state": "ready", "seat": "alice", "reason": null, "model": "x"}"#,
            r#"{"status": 1, "state": "ready", "seat": "alice", "reason": "stopping"}"#,
            r#"{"status": 1, "state": "degraded", "seat": "alice", "reason": null}"#,
            r#"{"status": 1, "state": "degraded", "seat": null, "reason": "readiness_timeout"}"#,
            r#"{"status": 1, "state": "awake", "seat": null, "reason": null}"#,
            r#"{"status": 1, "state": "ready", "seat": "al\"ice", "reason": null}"#,
            r#"{"status": 1, "state": "ready", "seat": {"key": "alice"}, "reason": null}"#,
            r#"{"status": 1, "status": 1, "state": "ready", "seat": null}"#,
            "[mineworld_cognition] a stray print",
            "",
        ] {
            assert_eq!(Status::parse(line), Err(()), "{line}");
        }
        let long = format!(
            r#"{{"status": 1, "state": "ready", "seat": "{}", "reason": null}}"#,
            "a".repeat(STATUS_BYTES)
        );
        assert_eq!(Status::parse(&long), Err(()), "over the bound");
    }
}
