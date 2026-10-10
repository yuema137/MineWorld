//! `mineworld server --stop-on-stdin-eof` (`DECISIONS.md` `ARC-78`, step-23 §6.4): the end of standard
//! input stops the server by the same graceful path as Ctrl-C, and without the flag it changes nothing.
//!
//! The real binary on the real pack, with a real save. The oracle is what the server prints on its
//! graceful path — `[mineworld] stopping`, then the world's `[world] ticks …` statistics, which are
//! printed only after the world thread has checkpointed — its exit status, and a second process that
//! resumes the save the first one left.

mod support;

use std::io::{BufRead, BufReader, Read};
use std::net::SocketAddr;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mineworld_test_support::process::{self, InterruptibleChild};
use support::{INVITE, PACK, PATIENCE, SaveDir, get};

/// A server process whose standard output is kept in memory, line by line.
struct Hosted {
    process: InterruptibleChild,
    stdin: Option<ChildStdin>,
    lines: Arc<Mutex<Vec<String>>>,
}

impl Drop for Hosted {
    fn drop(&mut self) {
        let _ = process::kill(&mut self.process);
    }
}

impl Hosted {
    /// Starts the server on `save` with `stdin` as its standard input, and waits until it answers
    /// `/health` on the port it chose and printed.
    async fn start(save: &str, flag: bool, stdin: fn() -> Stdio) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mineworld"));
        command
            .args(["server", PACK, "--invite", INVITE, "--save", save])
            // Port 0, read back from what the server prints: no port race (step-14 F-13w-3, #151).
            .args(["--listen", "127.0.0.1:0"])
            .args(flag.then_some("--stop-on-stdin-eof"))
            .env_remove("MINEWORLD_INVITE")
            .env_remove("MINEWORLD_ADMIN_TOKEN")
            .stdin(stdin())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut process = process::interruptible(command)
            .spawn()
            .expect("the mineworld binary runs");
        let stdin = process.stdin.take();
        let lines = drain(process.stdout.take().expect("piped stdout"));
        let hosted = Self {
            process,
            stdin,
            lines,
        };
        let listening = |printed: &[String]| {
            printed
                .iter()
                .find_map(|line| line.strip_prefix("[mineworld] listening on http://"))
                .and_then(|rest| rest.split(' ').next())
                .and_then(|address| address.parse::<SocketAddr>().ok())
        };
        assert!(
            hosted.shows(|printed| listening(printed).is_some()).await,
            "the server printed no listening line: {:?}",
            hosted.printed()
        );
        let address = listening(&hosted.printed()).expect("the listening address");
        let deadline = Instant::now() + PATIENCE;
        while get(address, "/health").await.is_none() {
            assert!(
                Instant::now() < deadline,
                "the server did not answer /health within {PATIENCE:?}"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        hosted
    }

    fn printed(&self) -> Vec<String> {
        self.lines.lock().expect("the reader is sound").clone()
    }

    /// Whether what the server printed satisfies `claim` within `PATIENCE`: the reader thread may lag
    /// the process, which may also have ended already.
    async fn shows(&self, claim: impl Fn(&[String]) -> bool) -> bool {
        let deadline = Instant::now() + PATIENCE;
        while Instant::now() < deadline {
            if claim(&self.printed()) {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        claim(&self.printed())
    }

    /// Waits for the process to end by itself, up to `PATIENCE`; `None` if it is still running.
    async fn ended(&mut self) -> Option<std::process::ExitStatus> {
        wait_for(&mut self.process, PATIENCE).await
    }
}

async fn wait_for(child: &mut Child, patience: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + patience;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("the child's state can be read") {
            return Some(status);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    None
}

/// Reads a pipe on a thread of its own until it closes.
fn drain(pipe: impl Read + Send + 'static) -> Arc<Mutex<Vec<String>>> {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            sink.lock().expect("the test is sound").push(line);
        }
    });
    lines
}

/// The lines of the graceful path, in order: the stop, then the world's statistics (printed once the
/// world thread has checkpointed and ended).
fn stopped_gracefully(printed: &[String]) -> bool {
    let stopping = printed
        .iter()
        .position(|line| line == "[mineworld] stopping");
    let ticks = printed
        .iter()
        .position(|line| line.starts_with("[world] ticks "));
    matches!((stopping, ticks), (Some(stop), Some(tick)) if stop < tick)
}

#[tokio::test]
async fn closing_standard_input_stops_the_server_gracefully_and_the_save_resumes() {
    let save = SaveDir::new("stop-on-stdin-eof");

    let mut first = Hosted::start(save.path(), true, Stdio::piped).await;
    assert!(
        first
            .shows(|printed| starts(printed, "[mineworld] created "))
            .await,
        "a new world: {:?}",
        first.printed()
    );
    // What a launcher does when its client has exited: close the pipe it holds.
    drop(first.stdin.take());
    let ended = first.ended().await.unwrap_or_else(|| {
        panic!(
            "the server was still running {PATIENCE:?} after its standard input closed: {:?}",
            first.printed()
        )
    });
    assert!(ended.success(), "end of input is a clean stop: {ended:?}");
    assert!(
        first.shows(stopped_gracefully).await,
        "the graceful path's lines, in order: {:?}",
        first.printed()
    );

    let mut second = Hosted::start(save.path(), true, Stdio::piped).await;
    assert!(
        second
            .shows(|printed| starts(printed, "[mineworld] resumed "))
            .await,
        "the second process resumes the checkpointed save: {:?}",
        second.printed()
    );
    drop(second.stdin.take());
    let ended = second.ended().await.expect("the second server stops too");
    assert!(ended.success(), "{ended:?}");
}

fn starts(printed: &[String], prefix: &str) -> bool {
    printed.iter().any(|line| line.starts_with(prefix))
}

#[tokio::test]
async fn without_the_flag_an_ended_standard_input_changes_nothing() {
    let save = SaveDir::new("stdin-eof-without-flag");
    // Standard input that is already at its end: with the flag this would stop the server at once.
    let mut hosted = Hosted::start(save.path(), false, Stdio::null).await;

    assert!(
        wait_for(&mut hosted.process, Duration::from_secs(2))
            .await
            .is_none(),
        "the server is still running with its standard input ended: {:?}",
        hosted.printed()
    );
    assert!(
        !starts(&hosted.printed(), "[mineworld] stopping"),
        "nothing asked it to stop: {:?}",
        hosted.printed()
    );
    let ended = process::interrupt(&mut hosted.process).expect("the interrupt is delivered");
    assert!(ended.success(), "Ctrl-C still stops it cleanly: {ended:?}");
    assert!(
        hosted.shows(stopped_gracefully).await,
        "{:?}",
        hosted.printed()
    );
}
