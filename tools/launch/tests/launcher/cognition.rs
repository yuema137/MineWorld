//! The cognition child of a played run (`LAUNCHER.md` §10; design `pr-playable-dialogue-memory.md` §§3.3,
//! 4.10–4.11, C2–C3), run for real with a scripted process in the interpreter's place.
//!
//! **The scripted process is this executable**, as the stub client is: the profile's `cognition-python`
//! is this test binary, which acts as `python -I -m mineworld_cognition …` when started with `-I`. It
//! records its arguments, its environment's two capabilities and whether the server still answered when
//! its standard input closed, and it speaks the status protocol (`docs/reference/cognition-supervised.md`
//! §2) as `MINEWORLD_LAUNCH_FAKE_COGNITION_MODE` says — well, malformed, silently, or not at all. So every
//! case here is the launcher's own behaviour, on every OS, with no Python: the real process against the real
//! server is `cognition/lm-controller/tests/test_supervised_process.py`.
//!
//! The oracle is what the processes did: the record files, process liveness, the server's log, a real join,
//! the folders left on disk. The launcher's log is read only for the closed state names it must write.
use super::*;
use std::io::{BufRead, Write};
use std::net::TcpStream;

pub const FAKE: &str = "MINEWORLD_LAUNCH_FAKE_COGNITION";
const MODE: &str = "MINEWORLD_LAUNCH_FAKE_COGNITION_MODE";
const DECISIONS: &str = "runtime/decisions/market-town-alice-lm.yaml";

// ---------------------------------------------------------------------------------------------------
// The scripted process.

/// `python -I -m mineworld_cognition …`, scripted. Returns the exit status.
pub fn fake(record: &Path, arguments: &[String]) -> ExitCode {
    let mode = std::env::var(MODE).unwrap_or_default();
    let has = |flag: &str| arguments.iter().any(|argument| argument == flag);
    let status = |state: &str, seat: Option<&str>, reason: Option<&str>| {
        let text = |value: Option<&str>| value.map_or("null".to_owned(), |v| format!("\"{v}\""));
        let mut out = std::io::stdout().lock();
        let _ = writeln!(
            out,
            "{{\"status\": 1, \"state\": \"{state}\", \"seat\": {}, \"reason\": {}}}",
            text(seat),
            text(reason)
        );
        let _ = out.flush();
    };
    if has("--preflight") {
        if let Some(reason) = mode.strip_prefix("preflight=") {
            status("degraded", None, Some(reason));
            return ExitCode::from(3);
        }
        status("starting", None, None);
        return ExitCode::SUCCESS;
    }
    let capability = |name: &str| std::env::var(name).unwrap_or_else(|_| "<absent>".to_owned());
    let text = format!(
        "pid={}\nadmin={}\ninvite={}\n{}\n",
        std::process::id(),
        capability("MINEWORLD_ADMIN_TOKEN"),
        capability("MINEWORLD_INVITE"),
        arguments.join("\n")
    );
    fs::write(record.join("cognition.partial"), text).expect("the record");
    fs::rename(
        record.join("cognition.partial"),
        record.join("cognition.args"),
    )
    .expect("renamed");
    let server = arguments
        .iter()
        .find_map(|argument| argument.strip_prefix("--server-url=ws://"))
        .and_then(|rest| rest.strip_suffix("/ws"))
        .unwrap_or_default()
        .to_owned();
    status("starting", None, None);
    match mode.as_str() {
        "exit" => {
            status("ready", Some("alice"), None);
            return ExitCode::from(3);
        }
        "garbage" => println!("[cognition] a stray print on the status channel"),
        "silent" => {}
        "seat-visitor" => status("starting", Some("visitor"), None),
        _ => {
            status("starting", Some("alice"), None);
            status("ready", Some("alice"), None);
        }
    }
    // Until the launcher closes standard input (or dies, and the OS closes it).
    let mut sink = String::new();
    let stdin = std::io::stdin();
    while stdin.lock().read_line(&mut sink).is_ok_and(|read| read > 0) {}
    let up = TcpStream::connect(server.as_str()).is_ok();
    fs::write(
        record.join("cognition.eof"),
        if up { "server-up" } else { "server-down" },
    )
    .expect("the record");
    if mode == "deaf" {
        std::thread::sleep(Duration::from_secs(120));
    }
    status("stopped", Some("alice"), Some("stopping"));
    ExitCode::SUCCESS
}

// ---------------------------------------------------------------------------------------------------
// Helpers.

/// A Market Town bundle with the shipped decision list, a profile for 2D whose interpreter is this test
/// binary, and the two operator files it names (opaque to the launcher; never read by the scripted process).
fn conversing(name: &str) -> Bundle {
    let bundle = Bundle::new(name, &["market-town"]);
    let decisions = bundle.root.join(DECISIONS);
    fs::create_dir_all(decisions.parent().expect("a folder")).expect("the decisions folder");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packaging/decisions/market-town-alice-lm.yaml"),
        &decisions,
    )
    .expect("the decision list is in the bundle");
    let operator = bundle.scratch.join("operator");
    fs::create_dir_all(&operator).expect("the operator's folder");
    for file in ["cognition.toml", "manifest.json"] {
        fs::write(operator.join(file), "opaque to the launcher").expect("an operator file");
    }
    let python = std::env::current_exe().expect("this test's executable");
    let profile = format!(
        "# written by the test\n[2d]\nworld = \"market-town\"\nseat = \"visitor\"\n\
         cognition-python = \"{}\"\ncognition-config = \"{}\"\ncognition-manifest = \"{}\"\n\
         cognition-decisions = \"{DECISIONS}\"\n",
        python.display(),
        operator.join("cognition.toml").display(),
        operator.join("manifest.json").display(),
    );
    let folder = bundle.user().join("cognition");
    fs::create_dir_all(&folder).expect("the cognition folder");
    fs::write(folder.join("profile.toml"), profile).expect("the profile");
    bundle
}

impl Bundle {
    /// The 2D launcher in the background, its cognition scripted as `mode`.
    fn conversing_in_background(&self, mode: &str) -> Background<'_> {
        let child = self
            .launcher("2d", &[])
            .env(STUB_WAIT, "1")
            .env(FAKE, self.record())
            .env(MODE, mode)
            .env(
                "MINEWORLD_ADMIN_TOKEN",
                "launcher-test-unrelated-parent-capability",
            )
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the launcher starts");
        Background {
            bundle: self,
            child,
        }
    }

    fn cognition_record(&self) -> Option<Vec<String>> {
        fs::read_to_string(self.record().join("cognition.args"))
            .ok()
            .map(|text| text.lines().map(str::to_owned).collect())
    }

    /// Waits until the launcher's log says `text`.
    fn logged(&self, text: &str, patience: Duration) {
        let deadline = Instant::now() + patience;
        while !self.launch_logs().contains(text) {
            assert!(
                Instant::now() < deadline,
                "the launcher's log never said {text:?}: {}",
                self.launch_logs()
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn store(&self, world: &str) -> PathBuf {
        self.user().join("cognition").join(world)
    }
}

impl Background<'_> {
    /// The player closes the game; the launcher's exit status.
    fn close(mut self) -> std::process::ExitStatus {
        self.bundle.release("2d", 0);
        let deadline = Instant::now() + PATIENCE;
        loop {
            if let Some(status) = self.child.try_wait().expect("the launcher's state") {
                return status;
            }
            assert!(Instant::now() < deadline, "the launcher did not finish");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn pid_of(record: &[String]) -> u32 {
    record[0]
        .strip_prefix("pid=")
        .and_then(|pid| pid.parse().ok())
        .expect("the scripted process's pid")
}

fn after<'a>(record: &'a [String], flag: &str) -> &'a str {
    let at = record
        .iter()
        .position(|argument| argument == flag)
        .unwrap_or_else(|| panic!("{flag} in {record:?}"));
    &record[at + 1]
}

fn wait_for_file(path: &Path, patience: Duration) -> String {
    let deadline = Instant::now() + patience;
    loop {
        if let Ok(text) = fs::read_to_string(path) {
            return text;
        }
        assert!(
            Instant::now() < deadline,
            "{} never appeared",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

// ---------------------------------------------------------------------------------------------------
// The cases.

/// §§3.3, 4.11: preflight, then the server with the decision list, the assignment, the child with an
/// environment built for it; `ready` logged; the child's stdin closed before the server's when the last
/// client exits; the run's exit status unchanged.
pub fn cognition_child_is_started_with_its_own_environment_and_stopped_before_the_server() {
    let bundle = conversing("launch-cognition");
    let launcher = bundle.conversing_in_background("ready");
    let client = bundle.record_of("2d");
    assert_eq!(client.user("seat"), "visitor", "the profile's seat");
    assert_eq!(
        client.user("world-key"),
        "market-town",
        "the profile's world"
    );
    bundle.logged("cognition: ready", PATIENCE);
    let record = bundle.cognition_record().expect("the child was started");

    // Argv: the interpreter directly, isolated, the supervisor's flags, no capability.
    let store = bundle.store("market-town");
    assert_eq!(
        &record[3..9],
        ["-I", "-m", "mineworld_cognition", "--config"]
            .iter()
            .map(ToString::to_string)
            .chain([bundle
                .scratch
                .join("operator/cognition.toml")
                .display()
                .to_string()])
            .chain(["--supervised".to_owned()])
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        record[9],
        format!("--server-url=ws://{}/ws", client.user("server"))
    );
    assert_eq!(after(&record, "--store-dir"), store.display().to_string());
    let assignment = store.join("run/assignment.json");
    assert_eq!(
        after(&record, "--assignment"),
        assignment.display().to_string()
    );
    let invite = client.user("invite");
    assert!(
        !record[3..].iter().any(|argument| argument.contains(invite)),
        "the invite is never in the child's argv: {record:?}"
    );
    // Environment: the invite in its variable; no admin capability, neither the run's nor the parent's.
    assert_eq!(
        record[1], "admin=<absent>",
        "no admin token reaches cognition"
    );
    assert_eq!(record[2], format!("invite={invite}"));

    // The server was given the same decision list as the assignment (`ARC-84` clause in its town line).
    let server_log = bundle
        .logs("server")
        .iter()
        .map(|path| fs::read_to_string(path).expect("the server's log"))
        .collect::<String>();
    assert!(server_log.contains("; decisions "), "{server_log}");
    let document: Value =
        serde_json::from_str(&fs::read_to_string(&assignment).expect("the assignment"))
            .expect("format-3 JSON");
    assert_eq!(document["format"], 3);
    let lm: Vec<&str> = document["seats"]
        .as_array()
        .expect("seats")
        .iter()
        .filter(|seat| seat["model"]["assigned"] == "lm")
        .filter_map(|seat| seat["seat"].as_str())
        .collect();
    assert_eq!(lm, ["alice"], "the shipped list drives exactly Alice");

    let status = launcher.close();
    assert!(
        status.success(),
        "{status}; the launcher's log: {}",
        bundle.launch_logs()
    );
    assert_eq!(
        fs::read_to_string(bundle.record().join("cognition.eof")).expect("the child saw EOF"),
        "server-up",
        "cognition's stdin closed while the server still ran: it stops first"
    );
    assert!(gone_within(pid_of(&record), STOP), "the child exited");
    assert!(
        stopped_gracefully(&server_log_now(&bundle)),
        "the server then stopped by its graceful path"
    );
    let invite_files = files_containing(&bundle.user(), invite.as_bytes());
    assert!(
        invite_files
            .iter()
            .all(|path| path.to_string_lossy().ends_with("-server.log")),
        "the invite is in no per-user file but the server's log: {invite_files:?}"
    );
}

fn server_log_now(bundle: &Bundle) -> String {
    bundle
        .logs("server")
        .iter()
        .map(|path| fs::read_to_string(path).expect("the server's log"))
        .collect()
}

/// Every file under `root` whose bytes contain `needle`.
fn files_containing(root: &Path, needle: &[u8]) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            } else if fs::read(&path)
                .is_ok_and(|bytes| bytes.windows(needle.len()).any(|window| window == needle))
            {
                found.push(path);
            }
        }
    }
    found
}

/// §4.10, §4.12: whatever goes wrong with conversation — a failed preflight, a malformed status line, a
/// process that exits, one that claims a player's seat, one that never becomes ready, one that ignores the
/// end of its input — is logged as its closed reason, the owned child is stopped (killed only past the
/// bound), and the world keeps playing: another player still joins, and the run ends normally.
pub fn cognition_failures_leave_the_world_playing() {
    let bundle = conversing("launch-cognition-fail");
    for (mode, logged, started, patience) in [
        (
            "preflight=runtime_mismatch",
            "degraded(runtime_mismatch)",
            false,
            PATIENCE,
        ),
        ("garbage", "degraded(status_invalid)", true, PATIENCE),
        ("exit", "degraded(child_exited)", true, PATIENCE),
        ("seat-visitor", "degraded(profile_invalid)", true, PATIENCE),
        (
            "silent",
            "degraded(readiness_timeout)",
            true,
            Duration::from_secs(90),
        ),
        ("deaf", "cognition: ready", true, PATIENCE),
    ] {
        for name in ["2d.args", "2d.release", "cognition.args", "cognition.eof"] {
            let _ = fs::remove_file(bundle.record().join(name));
        }
        eprintln!("[cognition case] {mode}");
        let launcher = bundle.conversing_in_background(mode);
        let client = bundle.record_of("2d");
        bundle.logged(logged, patience);
        assert_eq!(
            first_frame(client.user("server"), "wanderer", client.user("invite")),
            "welcome",
            "{mode}: the world still admits a player"
        );
        let record = bundle.cognition_record();
        assert_eq!(record.is_some(), started, "{mode}: a child started?");
        let server_log = server_log_now(&bundle);
        let status = launcher.close();
        assert!(
            status.success(),
            "{mode}: {status}; the launcher's log: {}",
            bundle.launch_logs()
        );
        if let Some(record) = record {
            assert!(
                gone_within(pid_of(&record), STOP),
                "{mode}: the owned child is gone"
            );
        } else {
            assert!(
                !server_log.contains("; decisions "),
                "{mode}: with conversation off before the server, it plays exactly as without a profile"
            );
        }
        if mode == "deaf" {
            bundle.logged("cognition: did not stop within 20 s and was killed", STOP);
        }
    }
}

/// §4.11: a launcher that dies closes its children's pipes; the cognition child stops on its own EOF,
/// like the server.
pub fn killing_the_launcher_stops_its_cognition_child() {
    let bundle = conversing("launch-cognition-kill");
    let mut launcher = bundle.conversing_in_background("ready");
    let client = bundle.record_of("2d");
    bundle.logged("cognition: ready", PATIENCE);
    let record = bundle.cognition_record().expect("the child");
    let killed = kill(&mut launcher.child);
    assert!(killed.killed(), "{killed:?}");
    let eof = wait_for_file(&bundle.record().join("cognition.eof"), STOP);
    bundle.release("2d", 0);
    assert!(gone_within(client.pid, STOP), "the stub client exits");
    let pid = pid_of(&record);
    if !gone_within(pid, STOP) {
        end(pid);
        panic!("the cognition child (pid {pid}) outlived its launcher");
    }
    assert!(
        eof == "server-up" || eof == "server-down",
        "the child saw its input end: {eof}"
    );
}

/// B1: `--fresh` deletes this world's save and its cognition store — and nothing else: another world's
/// store and the profile survive. With no profile, a run creates no cognition folder at all.
pub fn fresh_deletes_this_worlds_store_and_keeps_the_profile_and_other_worlds() {
    let bundle = conversing("launch-cognition-fresh");
    let ours = bundle.store("market-town");
    let theirs = bundle.store("social-cafe");
    for (folder, file) in [(&ours, "old-memory.sqlite"), (&theirs, "kept.sqlite")] {
        fs::create_dir_all(folder).expect("a store");
        fs::write(folder.join(file), "private conversation").expect("a store file");
    }
    let save = bundle.user().join("saves/market-town");
    fs::create_dir_all(&save).expect("a save");
    fs::write(save.join("marker"), "old world").expect("a save file");

    let fresh = bundle
        .launcher("2d", &["--fresh"])
        .env(FAKE, bundle.record())
        .env(MODE, "ready")
        .output()
        .expect("the launcher runs");
    assert!(fresh.status.success(), "{}", finished(&fresh));
    assert!(
        !ours.join("old-memory.sqlite").exists() && !save.join("marker").exists(),
        "this world's memories went with its save"
    );
    assert!(
        theirs.join("kept.sqlite").exists(),
        "another world's store is untouched"
    );
    assert!(
        bundle.user().join("cognition/profile.toml").exists(),
        "the profile is untouched"
    );

    // No profile: conversation is off, and no cognition folder is made.
    let plain = Bundle::new("launch-cognition-none", &["market-town"]);
    let run = plain
        .launcher("2d", &[])
        .env(FAKE, plain.record())
        .output()
        .expect("the launcher runs");
    assert!(run.status.success(), "{}", finished(&run));
    assert!(!plain.user().join("cognition").exists());
    assert!(
        plain.cognition_record().is_none(),
        "no child without a profile"
    );
}
