//! The launcher run for real (`LAUNCHER.md`; step-23 §6.4, §13.3, A-R5, A-R6): a scratch bundle with the
//! real `mineworld` server, a real World Pack, and a stub in the Godot runtime's place.
//!
//! **The stub is this executable.** The test target has no libtest harness: `main` acts as the stub client
//! when `MINEWORLD_LAUNCH_STUB` names a record folder, and runs the cases below otherwise. The scratch
//! bundle's engine path is a hard link to this executable, so the launcher starts it exactly as it would
//! start Godot. The stub writes its process id and arguments to `<record>/<2d|3d>.args`, then exits — at
//! once, with `MINEWORLD_LAUNCH_STUB_CODE`, or, with `MINEWORLD_LAUNCH_STUB_WAIT`, when the test writes an
//! exit code into `<record>/<2d|3d>.release`: the moment the player closes the game.
//!
//! **The oracle** is what the processes do, never what the launcher says about itself: the server's own
//! log (its graceful path prints `[mineworld] stopping`, then `[world] ticks …` once the world thread has
//! checkpointed), whether a process id is still alive, a real WebSocket join with the arguments the stub
//! was given, and the save a second run resumes.
//!
//! The server binary is the one `cargo test --workspace` builds for `mineworld-cli`'s own tests, at
//! `target/<profile>/mineworld`. Run alone, build it first: `cargo build -p mineworld-cli`.

use std::collections::BTreeSet;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output, Stdio};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use mineworld_test_support::process::kill;
use mineworld_test_support::{Scratch, scratch};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

const STUB: &str = "MINEWORLD_LAUNCH_STUB";
const STUB_WAIT: &str = "MINEWORLD_LAUNCH_STUB_WAIT";
const STUB_CODE: &str = "MINEWORLD_LAUNCH_STUB_CODE";

/// How long the test waits for a server to come up and a client to be started.
const PATIENCE: Duration = Duration::from_secs(90);
/// The launcher's own patience for a stopping server (`LAUNCHER.md` §8), plus a margin for a slow runner.
const STOP: Duration = Duration::from_secs(15);

fn main() -> ExitCode {
    if let Some(record) = std::env::var_os(STUB) {
        return stub(Path::new(&record));
    }
    let cases: [(&str, fn()); 4] = [
        (
            "play_joins_with_working_arguments_stops_gracefully_and_resumes",
            play_joins_with_working_arguments_stops_gracefully_and_resumes,
        ),
        (
            "killing_the_launcher_stops_its_server",
            killing_the_launcher_stops_its_server,
        ),
        (
            "smoke_both_runs_two_headless_clients_on_one_server_and_leaves_no_scratch",
            smoke_both_runs_two_headless_clients_on_one_server_and_leaves_no_scratch,
        ),
        (
            "a_missing_world_and_a_failing_server_are_reported_with_their_logs",
            a_missing_world_and_a_failing_server_are_reported_with_their_logs,
        ),
    ];
    // libtest's filter, as a substring of the name; flags (`--nocapture`, `--test-threads=1`) are ignored.
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| !argument.starts_with('-'))
        .collect();
    let chosen: Vec<_> = cases
        .iter()
        .filter(|(name, _)| filters.is_empty() || filters.iter().any(|f| name.contains(f.as_str())))
        .collect();
    println!("\nrunning {} tests", chosen.len());
    let mut failed = Vec::new();
    for (name, case) in &chosen {
        let outcome = panic::catch_unwind(AssertUnwindSafe(case));
        println!(
            "test {name} ... {}",
            if outcome.is_ok() { "ok" } else { "FAILED" }
        );
        if outcome.is_err() {
            failed.push(*name);
        }
    }
    println!(
        "\ntest result: {}. {} passed; {} failed",
        if failed.is_empty() { "ok" } else { "FAILED" },
        chosen.len() - failed.len(),
        failed.len()
    );
    if failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// ---------------------------------------------------------------------------------------------------
// The stub client.

fn stub(record: &Path) -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let label = arguments
        .windows(2)
        .find(|pair| pair[0] == "--main-pack")
        .and_then(|pair| Path::new(&pair[1]).file_stem()?.to_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned());
    let text = format!("pid={}\n{}\n", std::process::id(), arguments.join("\n"));
    // Written whole, then renamed: the test never reads half a record.
    let partial = record.join(format!("{label}.partial"));
    fs::write(&partial, text).expect("the record folder is writable");
    fs::rename(&partial, record.join(format!("{label}.args"))).expect("the record is renamed");

    if std::env::var_os(STUB_WAIT).is_some() {
        let release = record.join(format!("{label}.release"));
        let deadline = Instant::now() + Duration::from_secs(300);
        while Instant::now() < deadline {
            if let Ok(code) = fs::read_to_string(&release) {
                return ExitCode::from(code.trim().parse::<u8>().unwrap_or(1));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        return ExitCode::from(99);
    }
    let code = std::env::var(STUB_CODE)
        .ok()
        .and_then(|code| code.parse::<u8>().ok())
        .unwrap_or(0);
    ExitCode::from(code)
}

/// What a stub was started with.
struct Record {
    pid: u32,
    arguments: Vec<String>,
}

impl Record {
    /// The value of `--<name>=` among the client's own arguments (after `--`).
    fn user(&self, name: &str) -> &str {
        let prefix = format!("--{name}=");
        self.after_separator()
            .iter()
            .find_map(|argument| argument.strip_prefix(prefix.as_str()))
            .unwrap_or_else(|| panic!("no {prefix} in {:?}", self.arguments))
    }

    fn after_separator(&self) -> &[String] {
        let at = self
            .arguments
            .iter()
            .position(|argument| argument == "--")
            .unwrap_or_else(|| panic!("no -- in {:?}", self.arguments));
        &self.arguments[at + 1..]
    }

    fn engine(&self) -> &[String] {
        let at = self
            .arguments
            .iter()
            .position(|argument| argument == "--")
            .expect("a separator");
        &self.arguments[..at]
    }
}

// ---------------------------------------------------------------------------------------------------
// A scratch bundle.

struct Bundle {
    scratch: Scratch,
    root: PathBuf,
}

impl Bundle {
    /// A bundle whose root has a space and a non-ASCII character in its path (step-23 §9.1 step 1),
    /// holding the real server, the given worlds of the repository, two empty packs and the stub.
    fn new(name: &str, worlds: &[&str]) -> Self {
        let scratch = scratch!(empty name);
        let root = scratch.join("MineWorld 世界");
        let runtime = root.join("runtime");
        for folder in ["worlds", "clients", "godot"] {
            fs::create_dir_all(runtime.join(folder)).expect("bundle folders");
        }
        link(&server_binary(), &runtime.join(exe("mineworld")));
        let engine = engine_path(&runtime);
        fs::create_dir_all(engine.parent().expect("a folder")).expect("the engine folder");
        link(
            &std::env::current_exe().expect("this test's executable"),
            &engine,
        );
        for pack in ["2d.pck", "3d.pck"] {
            fs::write(runtime.join("clients").join(pack), b"").expect("a pack");
        }
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../worlds");
        for world in worlds {
            copy_tree(&repository.join(world), &runtime.join("worlds").join(world));
        }
        for folder in ["user", "record"] {
            fs::create_dir_all(scratch.join(folder)).expect("scratch folders");
        }
        Self { scratch, root }
    }

    fn runtime(&self) -> PathBuf {
        self.root.join("runtime")
    }

    fn user(&self) -> PathBuf {
        self.scratch.join("user")
    }

    fn record(&self) -> PathBuf {
        self.scratch.join("record")
    }

    fn overrides(&self, text: &str) {
        fs::write(self.runtime().join("launch.toml"), text).expect("launch.toml");
    }

    /// The launcher binary of `mode`, pointed at this bundle and its per-user folder, with no dialog.
    fn launcher(&self, mode: &str, arguments: &[&str]) -> Command {
        let binary = match mode {
            "2d" => env!("CARGO_BIN_EXE_mineworld-2d-launch"),
            "3d" => env!("CARGO_BIN_EXE_mineworld-3d-launch"),
            _ => env!("CARGO_BIN_EXE_mineworld-both-launch"),
        };
        let mut command = Command::new(binary);
        command
            .arg(prefixed("--bundle=", &self.root))
            .arg(prefixed("--user-dir=", &self.user()))
            .arg("--no-dialog")
            .args(arguments)
            .env(STUB, self.record())
            .env_remove(STUB_WAIT)
            .env_remove(STUB_CODE)
            .stdin(Stdio::null());
        command
    }

    /// Waits for the stub started for `label` (`2d`, `3d`) and reads what it was given.
    fn record_of(&self, label: &str) -> Record {
        let path = self.record().join(format!("{label}.args"));
        let deadline = Instant::now() + PATIENCE;
        while !path.exists() {
            assert!(
                Instant::now() < deadline,
                "no {label} client was started within {PATIENCE:?}; the launcher's log: {}",
                self.launch_logs()
            );
            std::thread::sleep(Duration::from_millis(50));
        }
        let text = fs::read_to_string(&path).expect("the record");
        let mut lines = text.lines();
        let pid = lines
            .next()
            .and_then(|line| line.strip_prefix("pid="))
            .and_then(|pid| pid.parse().ok())
            .expect("the stub's pid");
        Record {
            pid,
            arguments: lines.map(str::to_owned).collect(),
        }
    }

    /// The exit code the stub for `label` exits with: the player closes the game.
    fn release(&self, label: &str, code: u8) {
        fs::write(
            self.record().join(format!("{label}.release")),
            code.to_string(),
        )
        .expect("the release file");
    }

    fn logs(&self, what: &str) -> BTreeSet<PathBuf> {
        let suffix = format!("-{what}.log");
        fs::read_dir(self.user().join("logs"))
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.to_string_lossy().ends_with(&suffix))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn launch_logs(&self) -> String {
        self.logs("launch")
            .iter()
            .map(|path| fs::read_to_string(path).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n---\n")
    }

    /// The launcher of `mode` started in the background, its stubs waiting to be released.
    fn background(&self, mode: &str) -> Background<'_> {
        let child = self
            .launcher(mode, &[])
            .env(STUB_WAIT, "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the launcher starts");
        Background {
            bundle: self,
            child,
        }
    }
}

/// A launcher started in the background. Dropped while it still runs — a failed assertion — it is ended
/// the way a player would end it: every stub is released, the launcher stops its own server, and only a
/// launcher still running after that is killed. The scratch can then be removed on every OS.
struct Background<'a> {
    bundle: &'a Bundle,
    child: std::process::Child,
}

impl Drop for Background<'_> {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(None)) {
            return;
        }
        for label in ["2d", "3d"] {
            self.bundle.release(label, 0);
        }
        let deadline = Instant::now() + STOP + STOP;
        while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn exe(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

fn engine_path(runtime: &Path) -> PathBuf {
    let godot = runtime.join("godot");
    if cfg!(target_os = "macos") {
        godot.join("Godot.app/Contents/MacOS/Godot")
    } else if cfg!(windows) {
        godot.join("godot.exe")
    } else {
        godot.join("godot")
    }
}

fn prefixed(prefix: &str, path: &Path) -> std::ffi::OsString {
    let mut argument = std::ffi::OsString::from(prefix);
    argument.push(path);
    argument
}

/// `target/<profile>/mineworld`, beside this test's own `deps/` folder.
fn server_binary() -> PathBuf {
    let this = std::env::current_exe().expect("this test's executable");
    let profile = this
        .parent()
        .and_then(Path::parent)
        .expect("target/<profile>/deps/<test>");
    let server = profile.join(exe("mineworld"));
    assert!(
        server.exists(),
        "{} is missing: the launcher's tests run the real server. Build it first: \
         cargo build -p mineworld-cli (cargo test --workspace does)",
        server.display()
    );
    server
}

/// A hard link (same volume: target/), else a copy. Always a copy on Windows, where a name of an image
/// that is running — this test executable is — cannot be deleted, and the scratch must go.
fn link(from: &Path, to: &Path) {
    if cfg!(windows) || fs::hard_link(from, to).is_err() {
        fs::copy(from, to).expect("the file is copied into the bundle");
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("a folder");
    for entry in fs::read_dir(from).expect("a readable folder") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("a file is copied");
        }
    }
}

// ---------------------------------------------------------------------------------------------------
// Observations.

/// Whether a process id is alive: `kill -0` on Unix, `tasklist` on Windows (test-only; the launcher
/// itself never looks a process up).
fn alive(pid: u32) -> bool {
    if cfg!(windows) {
        let listed = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output()
            .expect("tasklist runs");
        String::from_utf8_lossy(&listed.stdout).contains(&format!("\"{pid}\""))
    } else {
        Command::new("sh")
            .args(["-c", &format!("kill -0 {pid} 2>/dev/null")])
            .status()
            .expect("sh runs")
            .success()
    }
}

/// Ends a process this test's launcher left behind, by id (test-only cleanup after a failure).
fn end(pid: u32) {
    let _ = if cfg!(windows) {
        Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output()
    } else {
        Command::new("sh")
            .args(["-c", &format!("kill -9 {pid}")])
            .output()
    };
}

fn gone_within(pid: u32, patience: Duration) -> bool {
    let deadline = Instant::now() + patience;
    while Instant::now() < deadline {
        if !alive(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    !alive(pid)
}

/// The server's graceful path, in order: the stop, then the statistics printed after the checkpoint.
fn stopped_gracefully(log: &str) -> bool {
    let stopping = log.find("[mineworld] stopping");
    let ticks = log.find("[world] ticks ");
    matches!((stopping, ticks), (Some(stop), Some(tick)) if stop < tick)
}

/// The only file in `after` that was not in `before`, read.
fn new_log(before: &BTreeSet<PathBuf>, after: &BTreeSet<PathBuf>) -> String {
    let new: Vec<_> = after.difference(before).collect();
    assert_eq!(new.len(), 1, "exactly one new log: {new:?}");
    fs::read_to_string(new[0]).expect("the log")
}

/// Joins the server at `address` as `seat` with `invite`, and returns the type of the first frame.
fn first_frame(address: &str, seat: &str, invite: &str) -> String {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime");
    runtime.block_on(async {
        let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
            .await
            .expect("the server accepts a WebSocket at the address the client was given");
        let join = json!({ "t": "join", "protocol": 2, "invite": invite,
                           "nickname": "launch-test", "seat": seat });
        socket
            .send(Message::Text(join.to_string().into()))
            .await
            .expect("the join is sent");
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_secs(20), socket.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    let frame: Value = serde_json::from_str(&text).expect("a JSON frame");
                    let kind = frame["t"].as_str().unwrap_or("?").to_owned();
                    let _ = socket.close(None).await;
                    return kind;
                }
                Ok(Some(Ok(_))) => {}
                other => panic!("no frame after the join: {other:?}"),
            }
        }
        panic!("no frame within 20 s of the join");
    })
}

fn finished(output: &Output) -> String {
    format!(
        "{} — stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    )
}

// ---------------------------------------------------------------------------------------------------
// The cases.

/// A-R5 and the resume: the client gets arguments that actually join the world, closing it stops the
/// server by its graceful path, and the next run resumes the save; `--fresh` starts it over.
fn play_joins_with_working_arguments_stops_gracefully_and_resumes() {
    let bundle = Bundle::new("launch-play", &["social-cafe"]);
    bundle.overrides("# the tests' world\n[2d]\nworld = \"social-cafe\"\nseat = \"visitor\"\n");

    let mut launcher = bundle.background("2d");
    let client = bundle.record_of("2d");
    let runtime = bundle.runtime();
    assert_eq!(
        client.engine(),
        [
            "--main-pack".to_owned(),
            runtime.join("clients/2d.pck").display().to_string()
        ],
        "a play run is windowed and loads the 2D pack"
    );
    assert_eq!(
        client.user("root"),
        runtime.display().to_string(),
        "the client is told the bundle's runtime folder (step-23 §6.5)"
    );
    assert_eq!(client.user("seat"), "visitor", "launch.toml's seat");
    let address = client.user("server").to_owned();
    assert!(
        address.starts_with("127.0.0.1:"),
        "loopback only: {address}"
    );
    let invite = client.user("invite").to_owned();
    assert!(
        invite.len() == 32 && invite.chars().all(|c| c.is_ascii_hexdigit()),
        "the server's generated invite: {invite}"
    );
    assert_eq!(
        first_frame(&address, client.user("seat"), &invite),
        "welcome",
        "the address and invite the client was given admit a player"
    );
    assert!(
        !bundle.launch_logs().contains(&invite),
        "the invite is never in the launcher's log"
    );

    // The player closes the game.
    let before = BTreeSet::new();
    bundle.release("2d", 0);
    let deadline = Instant::now() + PATIENCE;
    let status = loop {
        if let Some(status) = launcher.child.try_wait().expect("the launcher's state") {
            break status;
        }
        assert!(Instant::now() < deadline, "the launcher did not finish");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(
        status.success(),
        "{status}; the launcher's log: {}",
        bundle.launch_logs()
    );
    let servers = bundle.logs("server");
    let first = new_log(&before, &servers);
    assert!(first.contains("[mineworld] created "), "{first}");
    assert!(
        stopped_gracefully(&first),
        "closing the client stops the server by its graceful path: {first}"
    );
    assert!(
        bundle
            .user()
            .join("saves/social-cafe/world.sqlite")
            .exists(),
        "the save is in the per-user folder"
    );

    // The next run resumes; the stub exits at once.
    fs::remove_file(bundle.record().join("2d.args")).expect("the old record");
    let resumed = bundle
        .launcher("2d", &[])
        .output()
        .expect("the launcher runs");
    assert!(resumed.status.success(), "{}", finished(&resumed));
    let after = bundle.logs("server");
    let second = new_log(&servers, &after);
    assert!(second.contains("[mineworld] resumed "), "{second}");
    assert!(stopped_gracefully(&second), "{second}");

    // --fresh starts the world over.
    let fresh = bundle
        .launcher("2d", &["--fresh"])
        .output()
        .expect("the launcher runs");
    assert!(fresh.status.success(), "{}", finished(&fresh));
    let third = new_log(&after, &bundle.logs("server"));
    assert!(third.contains("[mineworld] created "), "{third}");
}

/// A-R6: a launcher that dies — here, killed — does not orphan its server, which stops by the same
/// graceful path within the launcher's own patience.
fn killing_the_launcher_stops_its_server() {
    let bundle = Bundle::new("launch-kill", &["social-cafe"]);
    let mut launcher = bundle.background("3d");
    let client = bundle.record_of("3d");
    assert_eq!(client.user("seat"), "visitor", "the 3D default seat");

    let log = bundle.launch_logs();
    let server: u32 = log
        .split("server started (pid ")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .and_then(|pid| pid.parse().ok())
        .unwrap_or_else(|| panic!("the server's pid in the launcher's log: {log}"));
    assert!(
        alive(server),
        "the server is running while the game is open"
    );

    let killed = kill(&mut launcher.child);
    assert!(
        killed.killed(),
        "the launcher was running and the kill ended it: {killed:?}"
    );
    let stopped = gone_within(server, STOP);
    // The stub is an orphan now; release it before anything else can fail, so the scratch can go.
    bundle.release("3d", 0);
    assert!(gone_within(client.pid, STOP), "the stub client exits");
    if !stopped {
        // The failure is the finding; the orphan is ended so that it does not outlive the test.
        end(server);
        panic!("the server (pid {server}) outlived its launcher by {STOP:?}");
    }
    let printed: Vec<String> = bundle
        .logs("server")
        .iter()
        .map(|path| fs::read_to_string(path).expect("the log"))
        .collect();
    assert!(
        printed.len() == 1 && stopped_gracefully(&printed[0]),
        "the orphaned server took its graceful path: {printed:?}"
    );
}

/// `--smoke=both`: one server, two headless clients with their probes and the `AC-15` seats; the exit
/// status is the clients'; the scratch save is removed; old runs' logs beyond five are pruned.
fn smoke_both_runs_two_headless_clients_on_one_server_and_leaves_no_scratch() {
    let bundle = Bundle::new("launch-smoke", &["social-cafe"]);
    let logs = bundle.user().join("logs");
    fs::create_dir_all(&logs).expect("the logs folder");
    for run in 1..=6 {
        fs::write(logs.join(format!("100000000{run}-7-launch.log")), "old").expect("an old log");
    }
    fs::write(logs.join("keep-me.log"), "not a run's").expect("an unrelated file");

    let output = bundle
        .launcher("both", &["--smoke=both"])
        .output()
        .expect("the launcher runs");
    assert!(output.status.success(), "{}", finished(&output));

    let (two_d, three_d) = (bundle.record_of("2d"), bundle.record_of("3d"));
    for (client, probe) in [(&two_d, "--drive"), (&three_d, "--slice-link")] {
        assert!(
            client
                .engine()
                .iter()
                .any(|argument| argument == "--headless"),
            "{:?}",
            client.arguments
        );
        for expected in [probe, "--settings=none"] {
            assert!(
                client
                    .after_separator()
                    .iter()
                    .any(|argument| argument == expected),
                "{expected} in {:?}",
                client.arguments
            );
        }
    }
    assert_eq!(two_d.user("seat"), "visitor");
    assert_eq!(three_d.user("seat"), "wanderer");
    assert_eq!(
        two_d.user("server"),
        three_d.user("server"),
        "one server for both clients"
    );
    assert!(
        !bundle.user().join("scratch").exists(),
        "the smoke run's scratch save is removed"
    );
    assert!(
        !bundle.user().join("saves/social-cafe").exists(),
        "a smoke run never touches the player's save"
    );
    let kept: BTreeSet<String> = fs::read_dir(&logs)
        .expect("the logs folder")
        .filter_map(|entry| entry.ok()?.file_name().to_str().map(str::to_owned))
        .filter(|name| name.starts_with("100000000"))
        .collect();
    assert_eq!(
        kept,
        (3..=6)
            .map(|run| format!("100000000{run}-7-launch.log"))
            .collect(),
        "this run and the four newest before it are kept"
    );
    assert!(
        logs.join("keep-me.log").exists(),
        "a file not of a run is never touched"
    );

    // A failing client's exit status is the launcher's.
    bundle.overrides("[2d]\nworld = \"social-cafe\"\n");
    for name in ["2d.args", "3d.args"] {
        fs::remove_file(bundle.record().join(name)).expect("the old record");
    }
    let failing = bundle
        .launcher("2d", &["--smoke=2d"])
        .env(STUB_CODE, "3")
        .output()
        .expect("the launcher runs");
    assert_eq!(failing.status.code(), Some(3), "{}", finished(&failing));
    assert!(
        !bundle.user().join("scratch").exists(),
        "removed after a failure too"
    );
}

/// A failure names what failed and the log to read, and exits 1.
fn a_missing_world_and_a_failing_server_are_reported_with_their_logs() {
    let bundle = Bundle::new("launch-fail", &["social-cafe"]);

    // The 2D default world, market-town, is not in this bundle.
    let missing = bundle
        .launcher("2d", &[])
        .output()
        .expect("the launcher runs");
    let said = String::from_utf8_lossy(&missing.stderr);
    assert_eq!(missing.status.code(), Some(1), "{}", finished(&missing));
    assert!(
        said.contains(
            &bundle
                .runtime()
                .join("worlds")
                .join("market-town")
                .display()
                .to_string()
        ) && said.contains("is missing")
            && said.contains("The launcher's log: "),
        "{said}"
    );

    // A world folder the server refuses: it stops before printing its join line.
    fs::create_dir_all(bundle.runtime().join("worlds/broken")).expect("an empty world");
    bundle.overrides("[3d]\nworld = \"broken\"\n");
    let refused = bundle
        .launcher("3d", &[])
        .output()
        .expect("the launcher runs");
    let said = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(1), "{}", finished(&refused));
    let server_log = bundle
        .logs("server")
        .into_iter()
        .next()
        .expect("the server's log");
    assert!(
        said.contains("the server stopped before it was ready")
            && said.contains(&server_log.display().to_string()),
        "{said}"
    );
    assert!(
        !bundle.record().join("3d.args").exists(),
        "no client is started without a server"
    );
}
