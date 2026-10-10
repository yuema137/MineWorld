//! The processes a run starts — one server, one or two clients — and how each is stopped
//! (`LAUNCHER.md` §§7–8). Only these children are ever stopped; nothing is looked up by name.

use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::Failure;
use crate::bundle::{Bundle, Client};
use crate::config::Seat;
use crate::join::{self, Join};
use crate::user::Log;

/// How long the server has to print its join line.
const JOIN_PATIENCE: Duration = Duration::from_secs(60);
/// How long the server has to stop after its standard input closes, before it is killed (step-23 §6.4).
const STOP_PATIENCE: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(100);

/// A log file opened twice, for a child's standard output and its standard error.
fn log_files(path: &Path) -> Result<(File, File), Failure> {
    let cannot =
        |error: std::io::Error| Failure::new(format!("cannot write {}: {error}", path.display()));
    let out = File::create(path).map_err(cannot)?;
    let err = out.try_clone().map_err(cannot)?;
    Ok((out, err))
}

/// The server this run started, stopped through its standard input. Dropped while still running — a
/// failure after it started — it is stopped the same way.
pub struct Server {
    child: Child,
    stdin: Option<ChildStdin>,
    log: PathBuf,
    stopped: bool,
}

impl Server {
    /// `mineworld server <world> --listen 127.0.0.1:0 --save <save> --agent alice --stop-on-stdin-eof`,
    /// its standard input a pipe this launcher holds, its output in `log_path`.
    pub fn start(
        bundle: &Bundle,
        world: &Path,
        save: &Path,
        log_path: &Path,
        log: &mut Log,
    ) -> Result<Self, Failure> {
        // A file, not a pipe: a server writing into the pipe of a launcher that has died would get
        // EPIPE, and `println!` panics on it — the graceful stop would become a crash (step-23 D-RC-2).
        let (out, err) = log_files(log_path)?;
        let mut command = Command::new(bundle.server());
        command
            .arg("server")
            .arg(world)
            .args(["--listen", "127.0.0.1:0", "--save"])
            .arg(save)
            .args(["--agent", "alice", "--stop-on-stdin-eof"])
            // A supplied invite is never printed, and the launcher reads the generated one
            // (`server/PROTOCOL.md` §4.1); no admin surface is mounted.
            .env_remove("MINEWORLD_INVITE")
            .env_remove("MINEWORLD_ADMIN_TOKEN")
            .stdin(Stdio::piped())
            .stdout(out)
            .stderr(err);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // A console program started by a GUI program would otherwise open a console window.
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = command.spawn().map_err(|error| {
            Failure::new(format!(
                "cannot start the server ({}): {error}",
                bundle.server().display()
            ))
        })?;
        log.line(&format!(
            "server started (pid {}), its log: {}",
            child.id(),
            log_path.display()
        ));
        let stdin = child.stdin.take();
        Ok(Self {
            child,
            stdin,
            log: log_path.to_path_buf(),
            stopped: false,
        })
    }

    /// Waits for the join line in the server's log.
    pub fn join(&mut self) -> Result<Join, Failure> {
        let deadline = Instant::now() + JOIN_PATIENCE;
        loop {
            let printed = std::fs::read(&self.log).unwrap_or_default();
            if let Some(join) = join::find(&String::from_utf8_lossy(&printed)) {
                return Ok(join);
            }
            if let Ok(Some(status)) = self.child.try_wait() {
                self.stopped = true;
                return Err(Failure::new(format!(
                    "the server stopped before it was ready ({status}); its log: {}",
                    self.log.display()
                )));
            }
            if Instant::now() >= deadline {
                return Err(Failure::new(format!(
                    "the server was not ready within {} s; its log: {}",
                    JOIN_PATIENCE.as_secs(),
                    self.log.display()
                )));
            }
            std::thread::sleep(POLL);
        }
    }

    /// Closes the server's standard input and waits for it to stop; kills it after `STOP_PATIENCE`. A
    /// server that had to be killed, or ended with a failure, is a failure of the run.
    pub fn stop(&mut self, log: &mut Log) -> Result<(), Failure> {
        let outcome = self.ended();
        match &outcome {
            Ok(status) => log.line(&format!("server stopped: {status}")),
            Err(failure) => log.line(&failure.to_string()),
        }
        let status = outcome?;
        if status.success() {
            Ok(())
        } else {
            Err(Failure::new(format!(
                "the server ended with {status}; its log: {}",
                self.log.display()
            )))
        }
    }

    fn ended(&mut self) -> Result<ExitStatus, Failure> {
        self.stopped = true;
        drop(self.stdin.take());
        let deadline = Instant::now() + STOP_PATIENCE;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(status)) => return Ok(status),
                Ok(None) => std::thread::sleep(POLL),
                Err(_) => break,
            }
        }
        // Only this child, never a process found by name.
        let _ = self.child.kill();
        let status = self.child.wait();
        Err(Failure::new(format!(
            "the server did not stop within {} s and was killed ({}); its log: {}",
            STOP_PATIENCE.as_secs(),
            status.map_or_else(|error| error.to_string(), |status| status.to_string()),
            self.log.display()
        )))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if !self.stopped {
            let _ = self.ended();
        }
    }
}

/// What every client of one run is started with.
pub struct Clients<'a> {
    pub bundle: &'a Bundle,
    pub join: &'a Join,
    pub smoke: bool,
}

impl Clients<'_> {
    /// Starts every client, then waits until all have exited. Returns the first non-zero exit status
    /// (0 if none). A client that cannot be started ends those already started.
    pub fn run(
        &self,
        clients: &[(Client, Seat, PathBuf)],
        logs: impl Fn(Client) -> PathBuf,
        log: &mut Log,
    ) -> Result<u8, Failure> {
        let mut running: Vec<(Client, Child)> = Vec::new();
        for (client, seat, pack) in clients {
            match self.start(*client, seat, pack, &logs(*client)) {
                Ok(child) => {
                    log.line(&format!(
                        "client {} started as {} (pid {})",
                        client.label(),
                        seat.as_str(),
                        child.id()
                    ));
                    running.push((*client, child));
                }
                Err(failure) => {
                    for (_, mut child) in running {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    return Err(failure);
                }
            }
        }
        let mut code = 0;
        for (client, mut child) in running {
            let status = child.wait().map_err(|error| {
                Failure::new(format!(
                    "cannot wait for the {} client: {error}",
                    client.label()
                ))
            })?;
            log.line(&format!("client {} exited: {status}", client.label()));
            if code == 0 && !status.success() {
                code = failure_code(status);
            }
        }
        Ok(code)
    }

    /// Starts a client in the Godot runtime (`LAUNCHER.md` §7 step 5). The invite is on its command line,
    /// as the root launchers pass it, and never in the launcher's log.
    fn start(
        &self,
        client: Client,
        seat: &Seat,
        pack: &Path,
        log: &Path,
    ) -> Result<Child, Failure> {
        let (out, err) = log_files(log)?;
        let mut root = OsString::from("--root=");
        root.push(self.bundle.runtime());
        let mut command = Command::new(self.bundle.engine());
        command.arg("--main-pack").arg(pack);
        if self.smoke {
            command.arg("--headless");
        }
        command
            .arg("--")
            .arg(root)
            .arg(format!("--server={}", self.join.address))
            .arg(format!("--seat={}", seat.as_str()))
            .arg(format!("--invite={}", self.join.invite));
        if self.smoke {
            command.args(match client {
                Client::TwoD => ["--drive", "--settings=none"],
                Client::ThreeD => ["--slice-link", "--settings=none"],
            });
        }
        command
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err)
            .spawn()
            .map_err(|error| {
                Failure::new(format!(
                    "cannot start the {} client ({}): {error}",
                    client.label(),
                    self.bundle.engine().display()
                ))
            })
    }
}

/// A client's non-zero exit as the launcher's own exit status.
fn failure_code(status: ExitStatus) -> u8 {
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .filter(|code| *code != 0)
        .unwrap_or(1)
}
