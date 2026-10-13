//! The shared client settings module (`clients/shared/settings`, S20 PR SET-a) run for real: its own
//! headless checks, and the runtime criteria of step-20 §7 in the 2D client (and, from C6, the 3D one).
//!
//! Every test here starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_settings -- --ignored --test-threads=1
//! ```
//!
//! Oracles are independent of the client where the property allows: the stub's record of every frame
//! the client sent (AC-SET-5), the settings file's bytes (AC-SET-6, AC-SET-8), what the window and the
//! engine report (AC-SET-11). The client's own `[PASS]` lines are required too, by name, never alone.

mod godot2d;
// `godot2d::worlds` reads saves through it.
mod headless;
mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use godot2d::{Drive, MARKET_TOWN, StubScript, World, evidence, passed, stub};
use serde_json::Value;
use support::{INVITE, SaveDir};

const SHARED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/shared");
const CLIENT_2D: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/2d");
const CLIENT_3D: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../clients/3d-spike");
const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

/// Runs the 3D slice's settings probe (`--slice-settings`) in a window against `server`, with
/// `arguments`, and returns how it ended and what it printed. One window at a time.
fn run_slice(
    server: std::net::SocketAddr,
    arguments: &[&str],
) -> (std::process::ExitStatus, Vec<String>) {
    use std::io::{BufRead, BufReader};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    let binary = godot2d::godot();
    let imported = Command::new(&binary)
        .args(["--headless", "--path", CLIENT_3D, "--import"])
        .output()
        .expect("godot runs");
    assert!(imported.status.success(), "the 3D project imports");
    let mut child = Command::new(&binary)
        .args([
            "--always-on-top",
            "--path",
            CLIENT_3D,
            "res://scenes/slice.tscn",
            "--",
        ])
        .arg("--slice-settings")
        .arg(format!("--server={server}"))
        .arg(format!("--invite={INVITE}"))
        .args(arguments)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("godot runs");
    let stdout = child.stdout.take().expect("piped");
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            eprintln!("[3d] {line}");
            sink.lock().expect("lines").push(line);
        }
    });
    let deadline = Instant::now() + Duration::from_secs(240);
    let status = loop {
        if let Some(status) = child.try_wait().expect("waitable") {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!(
                "the slice hung:\n{}",
                lines.lock().expect("lines").join("\n")
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    reader.join().expect("the reader ends");
    let printed = lines.lock().expect("lines").clone();
    (status, printed)
}

/// Whether the 3D probe's own checks all passed.
fn slice_passed(lines: &[String]) -> bool {
    lines.iter().any(|l| l == "all settings checks pass")
        && !lines.iter().any(|l| l.contains("[FAIL]"))
}

/// The 3D client's request transcript, token-normalized, as JSON values.
fn requests(lines: &[String]) -> Vec<Value> {
    let tagged: Vec<Value> = godot2d::tagged(lines, "REQUEST ");
    without_tokens(
        &tagged
            .iter()
            .map(|r| r["request"].clone())
            .collect::<Vec<_>>(),
    )
}

/// Runs a script of `clients/shared/checks/` headless inside `project` and returns what it printed.
fn check(project: &str, script: &str, arguments: &[&str]) -> (bool, Vec<String>) {
    let binary = godot2d::godot();
    let imported = Command::new(&binary)
        .args(["--headless", "--path", project, "--import"])
        .output()
        .expect("godot runs");
    assert!(imported.status.success(), "{project} imports");
    let path = Path::new(SHARED).join("checks").join(script);
    let done = Command::new(&binary)
        .args(["--headless", "--path", project, "--script"])
        .arg(path)
        .arg("--")
        .args(arguments)
        .output()
        .expect("godot runs");
    let lines: Vec<String> = String::from_utf8_lossy(&done.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    for line in &lines {
        eprintln!("[{script}] {line}");
    }
    (done.status.success(), lines)
}

/// The JSON of the first `EVIDENCE` line carrying `key`.
fn first(lines: &[String], key: &str) -> Value {
    evidence(lines, key)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no EVIDENCE {key} in:\n{}", lines.join("\n")))
}

/// Every `[PASS]`/`[FAIL]` check the run printed, by name, without its measured detail.
fn verdicts(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| line.starts_with("[PASS]") || line.starts_with("[FAIL]"))
        // The name ends where the measured detail starts: `drive.gd` pads the name and then prints
        // numbers, which differ between runs for reasons of their own (positions, text counts).
        .map(|line| {
            line[..line
                .find(|c: char| c.is_ascii_digit())
                .unwrap_or(line.len())]
                .trim()
                .to_owned()
        })
        .collect()
}

fn require(lines: &[String], names: &[&str]) {
    let shown = verdicts(lines);
    for name in names {
        assert!(
            shown
                .iter()
                .any(|v| v.starts_with(&format!("[PASS] {name}"))),
            "the run did not pass \"{name}\":\n{}",
            lines.join("\n")
        );
    }
}

/// A settings file in a scratch folder.
fn settings_file(scratch: &SaveDir, name: &str, body: &str) -> PathBuf {
    let path = Path::new(scratch.path()).join(name);
    std::fs::create_dir_all(scratch.path()).expect("the scratch folder");
    std::fs::write(&path, body).expect("the settings file writes");
    path
}

/// The frames a client sent, with each request's token replaced by its position, so two runs compare.
fn without_tokens(frames: &[Value]) -> Vec<Value> {
    frames
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            let mut frame = frame.clone();
            if frame.get("token").is_some() {
                frame["token"] = Value::from(format!("#{i}"));
            }
            frame
        })
        .collect()
}

// --- the module's own checks ---------------------------------------------------------------------

/// C2–C4: the store (AC-SET-8's five cases, the atomic save, the merge), the text layer and the clock,
/// glyph coverage (AC-SET-13) and the menu alone — each prints its verdict and exits non-zero on a
/// failure.
#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
fn the_settings_module_checks_pass() {
    for script in [
        "store_check.gd",
        "text_check.gd",
        "glyph_check.gd",
        "menu_check.gd",
    ] {
        let (ok, lines) = check(SHARED, script, &[]);
        let name = script.trim_end_matches(".gd");
        assert!(
            ok && lines.iter().any(|l| l == &format!("{name}: PASS")),
            "{script} failed:\n{}",
            lines.join("\n")
        );
    }
}

/// AC-SET-15: the shared settings folder resolves to `…/MineWorld` on this OS, in the module's own
/// project and in the 2D client's (the 3D client's from C6).
#[test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
fn the_settings_folder_is_shared() {
    let mut folders = Vec::new();
    for project in [SHARED, CLIENT_2D, CLIENT_3D] {
        let (ok, lines) = check(project, "store_check.gd", &["--where"]);
        assert!(ok, "{project}: {}", lines.join("\n"));
        folders.push(first(&lines, "where")["user_dir"].clone());
    }
    assert!(
        folders.windows(2).all(|pair| pair[0] == pair[1]),
        "one folder: {folders:?}"
    );
    // A file one client's project saves is the file the other's reads (a probe file, removed by the
    // reader; the player's own settings.cfg is never touched).
    let probe = format!("settings-probe-{}.cfg", std::process::id());
    let (ok, lines) = check(CLIENT_2D, "store_check.gd", &[&format!("--write={probe}")]);
    assert!(ok, "{}", lines.join("\n"));
    let (ok, lines) = check(CLIENT_3D, "store_check.gd", &[&format!("--read={probe}")]);
    assert!(ok, "{}", lines.join("\n"));
    let read = first(&lines, "probe");
    assert_eq!(read["language"], "zh_Hans", "{read}");
    assert_eq!(
        read["window_size"],
        serde_json::json!([1280, 720]),
        "{read}"
    );
}

// --- the 2D client --------------------------------------------------------------------------------

/// AC-SET-1, AC-SET-2 and AC-SET-12 in the connected 2D client (market-town): with the settings menu
/// open over the HUD, every UI text turns Chinese at once and back exactly; under the marker catalog
/// every UI text is a catalog entry; holding W and clicking the floor ask the world for nothing.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
async fn two_d_switches_language_live_and_an_open_menu_takes_input() {
    let save = SaveDir::new("settings-2d-live");
    let world = World::start(
        &[
            "server",
            MARKET_TOWN,
            "--agent",
            "alice",
            "--save",
            save.path(),
        ],
        None,
    )
    .await;
    let drive = Drive::start(world.address, "carol", &["--drive=settings", "--marker"]);
    let (status, lines) = drive.finish().await;
    assert!(status.success() && passed(&lines), "{}", lines.join("\n"));
    require(
        &lines,
        &[
            "every UI text turns from en to zh_Hans at once",
            "switching back reproduces every text exactly",
            "under the marker catalog every UI text comes from a catalog",
            "an open menu takes gameplay input",
        ],
    );
    let texts = first(&lines, "texts");
    assert!(
        texts["en"].as_u64() >= Some(30),
        "the walk read the HUD and the menu: {texts}"
    );
    let pairs = evidence(&lines, "text");
    for wanted in [
        "Settings",
        "Quit the game",
        "Window mode",
        "Esc: close, menu",
    ] {
        assert!(
            pairs
                .iter()
                .any(|p| p["en"].as_str().is_some_and(|t| t.contains(wanted))),
            "the walk read \"{wanted}\""
        );
    }
    assert!(
        pairs
            .iter()
            .any(|p| p["en"].as_str().is_some_and(|t| t.starts_with("day 1"))),
        "the walk read the HUD's day and time"
    );
}

/// AC-SET-5 (and AC-SET-10's 2D analogue): the same scenario on defaults and on a settings file of
/// Chinese, 24-hour, borderless and 30 fps sends the same frames — every one, from `join` to `leave` —
/// and prints the same checks and `drive:` lines.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
async fn two_d_settings_never_reach_the_server() {
    let scratch = SaveDir::new("settings-2d-frames");
    let file = settings_file(
        &scratch,
        "settings.cfg",
        "[meta]\nversion=1\n[general]\nlanguage=\"zh_Hans\"\nclock=\"24h\"\n\
         [display]\nwindow_mode=\"borderless\"\nmax_fps=30\n",
    );
    let mut runs = Vec::new();
    for settings in [None, Some(&file)] {
        let (address, log) = stub(StubScript::default()).await;
        let mut arguments = vec!["--drive=settings".to_owned()];
        if let Some(file) = settings {
            arguments.push(format!("--settings={}", file.display()));
        }
        let owned: Vec<&str> = arguments.iter().map(String::as_str).collect();
        let (status, lines) = Drive::start(address, "carol", &owned).finish().await;
        assert!(status.success() && passed(&lines), "{}", lines.join("\n"));
        let frames = log.lock().expect("log").frames.clone();
        runs.push((lines, frames));
    }
    let (en_lines, en_frames) = &runs[0];
    let (zh_lines, zh_frames) = &runs[1];
    assert_eq!(first(en_lines, "settings")["language"], "en");
    assert_eq!(first(zh_lines, "settings")["language"], "zh_Hans");
    assert_eq!(first(zh_lines, "settings")["max_fps"], 30);
    assert!(
        en_frames.iter().filter(|f| f["t"] == "submit").count() >= 2,
        "the scenario sent requests: {en_frames:?}"
    );
    assert_eq!(
        en_frames.last().map(|f| f["t"].clone()),
        Some(Value::from("leave")),
        "the stub saw the client to its end"
    );
    assert_eq!(
        without_tokens(en_frames),
        without_tokens(zh_frames),
        "the frames sent do not depend on the settings (AC-SET-5)"
    );
    let logs = |lines: &[String]| -> Vec<String> {
        lines
            .iter()
            .filter(|l| l.starts_with("drive:"))
            .cloned()
            .chain(verdicts(lines).into_iter().map(|v| {
                // The language pair a check names is the run's own, not a log of its language.
                v.replace("from en to zh_Hans", "between languages")
                    .replace("from zh_Hans to en", "between languages")
            }))
            .collect()
    };
    assert_eq!(
        logs(en_lines),
        logs(zh_lines),
        "the logs do not depend on the language (INV-SET-6)"
    );
}

/// AC-SET-6 and AC-SET-8 in the 2D client: a player's Apply writes a file that holds the settings and
/// nothing of the join (invite, nickname, seat, address); a bad file starts the client on defaults,
/// says which file and key, and is left byte-identical.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
async fn two_d_the_settings_file_holds_only_settings_and_a_bad_one_is_kept() {
    let scratch = SaveDir::new("settings-2d-file");
    let file = Path::new(scratch.path()).join("applied.cfg");
    std::fs::create_dir_all(scratch.path()).expect("scratch");
    let (address, _log) = stub(StubScript::default()).await;
    let owned = [
        "--drive=settings".to_owned(),
        format!("--settings={}", file.display()),
        "--apply=zh_Hans,24h,1280x720".to_owned(),
    ];
    let owned: Vec<&str> = owned.iter().map(String::as_str).collect();
    let (status, lines) = Drive::start(address, "carol", &owned).finish().await;
    assert!(status.success() && passed(&lines), "{}", lines.join("\n"));
    let written = std::fs::read_to_string(&file).expect("Apply wrote the file");
    for secret in [
        INVITE,
        "client-2d-test",
        "carol",
        &address.to_string(),
        "127.0.0.1",
    ] {
        assert!(
            !written.contains(secret),
            "the settings file holds {secret:?}:\n{written}"
        );
    }
    let mut keys: Vec<String> = written
        .lines()
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().to_owned()))
        .collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "clock",
            "language",
            "max_fps",
            "render_scale",
            "version",
            "vsync",
            "window_mode",
            "window_size"
        ],
        "exactly the keys of §3.3:\n{written}"
    );
    assert!(written.contains("language=\"zh_Hans\"") && written.contains("clock=\"24h\""));
    assert!(written.contains("window_size=Vector2i(1280, 720)"));

    let bad = settings_file(
        &scratch,
        "bad.cfg",
        "[meta]\nversion=1\n[display]\nwindow_mode=7\n",
    );
    let before = std::fs::read(&bad).expect("bytes");
    let (address, _log) = stub(StubScript::default()).await;
    let owned = format!("--settings={}", bad.display());
    let (status, lines) = Drive::start(address, "carol", &["--drive", &owned])
        .finish()
        .await;
    assert!(status.success() && passed(&lines), "{}", lines.join("\n"));
    let shown = first(&lines, "settings");
    let problems = shown["problems"].to_string();
    assert!(
        problems.contains("bad.cfg") && problems.contains("display/window_mode"),
        "{shown}"
    );
    assert_eq!(
        std::fs::read(&bad).expect("bytes"),
        before,
        "the bad file is left as it was"
    );
}

/// Restores the player's own settings file after a test planted one in the shared folder.
struct Planted {
    path: PathBuf,
    kept: Option<Vec<u8>>,
}

impl Planted {
    fn new(user_dir: &str, body: &str) -> Self {
        let path = Path::new(user_dir).join("settings.cfg");
        let kept = std::fs::read(&path).ok();
        std::fs::create_dir_all(user_dir).expect("the settings folder");
        std::fs::write(&path, body).expect("the planted file writes");
        Self { path, kept }
    }
}

impl Drop for Planted {
    fn drop(&mut self) {
        match &self.kept {
            Some(bytes) => {
                std::fs::write(&self.path, bytes).expect("the player's file is restored")
            }
            None => {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

/// AC-SET-9 in the 2D client: with a planted file in the player's settings folder (Chinese, 24-hour,
/// borderless, 30 fps), `--drive=walk` reports the same checks and runs on defaults, in English.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
async fn two_d_harness_ignores_the_players_settings() {
    let (ok, lines) = check(CLIENT_2D, "store_check.gd", &["--where"]);
    assert!(ok);
    let user_dir = first(&lines, "where")["user_dir"]
        .as_str()
        .expect("a folder")
        .to_owned();
    let mut runs = Vec::new();
    for plant in [false, true] {
        let _planted = plant.then(|| {
            Planted::new(
                &user_dir,
                "[meta]\nversion=1\n[general]\nlanguage=\"zh_Hans\"\nclock=\"24h\"\n\
                 [display]\nwindow_mode=\"borderless\"\nmax_fps=30\n",
            )
        });
        let save = SaveDir::new("settings-2d-harness");
        let world = World::start(
            &[
                "server",
                MARKET_TOWN,
                "--agent",
                "alice",
                "--save",
                save.path(),
            ],
            None,
        )
        .await;
        let (status, lines) = Drive::start(world.address, "carol", &["--drive=walk"])
            .finish()
            .await;
        assert!(status.success() && passed(&lines), "{}", lines.join("\n"));
        runs.push(lines);
    }
    for lines in &runs {
        let shown = first(lines, "settings");
        assert_eq!(
            shown["file"], "",
            "a harness reads no settings file: {shown}"
        );
        assert_eq!(shown["language"], "en");
        assert_eq!(shown["max_fps"], 0);
    }
    assert_eq!(
        verdicts(&runs[0]),
        verdicts(&runs[1]),
        "the same checks, planted file or not"
    );
}

/// AC-SET-11 in the 2D client, in a real window: 1280×720 with VSync off and a 30 fps cap gives that
/// window, that VSync mode and a mean frame rate of at most 31; with no cap the same scene runs faster.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
async fn two_d_display_settings_take_effect() {
    let scratch = SaveDir::new("settings-2d-display");
    let mut means = Vec::new();
    for cap in [30, 0] {
        let file = settings_file(
            &scratch,
            &format!("display-{cap}.cfg"),
            &format!(
                "[meta]\nversion=1\n[display]\nwindow_mode=\"windowed\"\n\
                 window_size=Vector2i(1280, 720)\nvsync=\"off\"\nmax_fps={cap}\n"
            ),
        );
        let (address, _log) = stub(StubScript::default()).await;
        let owned = format!("--settings={}", file.display());
        let drive = Drive::start_with(
            address,
            "carol",
            &["--drive=display", &owned],
            &["--always-on-top"],
        );
        let (status, lines) = drive.finish().await;
        assert!(status.success() && passed(&lines), "{}", lines.join("\n"));
        let shown = first(&lines, "display");
        eprintln!("[display] cap {cap}: {shown}");
        assert_eq!(shown["window"], serde_json::json!([1280, 720]), "{shown}");
        assert_eq!(shown["vsync"], 0, "VSync off: {shown}");
        assert_eq!(shown["max_fps"], cap, "{shown}");
        means.push(shown["mean_fps"].as_f64().expect("a frame rate"));
    }
    assert!(means[0] <= 31.0, "capped at 30: {means:?}");
    assert!(means[1] > 31.0, "uncapped runs faster: {means:?}");
}

// --- the 3D client --------------------------------------------------------------------------------

/// AC-SET-1, AC-SET-2, AC-SET-5, AC-SET-10, AC-SET-11 (render scale) and AC-SET-12 in the connected 3D
/// slice (social-cafe): on defaults and on a file of Chinese, 24-hour and a 67 % render scale, the
/// probe's checks pass; the requests sent and the `[link]` log lines are the same in both runs; the
/// render scale is the file's.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
async fn three_d_switches_language_live_and_settings_never_reach_the_server() {
    let scratch = SaveDir::new("settings-3d");
    let file = settings_file(
        &scratch,
        "zh.cfg",
        "[meta]\nversion=1\n[general]\nlanguage=\"zh_Hans\"\nclock=\"24h\"\n\
         [display]\nrender_scale=67\n",
    );
    let mut runs = Vec::new();
    for settings in [None, Some(&file)] {
        let save = SaveDir::new("settings-3d-world");
        let world = World::start(
            &[
                "server",
                SOCIAL_CAFE,
                "--agent",
                "alice",
                "--save",
                save.path(),
            ],
            None,
        )
        .await;
        let mut owned = vec!["--marker".to_owned()];
        if let Some(file) = settings {
            owned.push(format!("--settings={}", file.display()));
        }
        let owned: Vec<&str> = owned.iter().map(String::as_str).collect();
        let (status, lines) = run_slice(world.address, &owned);
        assert!(
            status.success() && slice_passed(&lines),
            "{}",
            lines.join("\n")
        );
        require(
            &lines,
            &[
                "seated",
                "every UI text turns from",
                "switching back reproduces every text exactly",
                "under the marker catalog every UI text comes from a catalog",
                "an open menu takes gameplay input",
            ],
        );
        runs.push(lines);
    }
    let (en, zh) = (&runs[0], &runs[1]);
    assert_eq!(first(en, "first_frame")["language"], "en");
    assert_eq!(first(zh, "first_frame")["language"], "zh_Hans");
    let scale = |lines: &[String]| {
        first(lines, "display")["render_scale"]
            .as_f64()
            .expect("a scale")
    };
    assert!(
        (scale(en) - 1.0).abs() < 1e-6 && (scale(zh) - 0.67).abs() < 1e-6,
        "render scale"
    );
    assert!(
        first(en, "texts")["en"].as_u64() >= Some(30),
        "the walk read the HUD and the menu"
    );
    assert!(!requests(en).is_empty(), "the probe sent its requests");
    assert_eq!(
        requests(en),
        requests(zh),
        "the requests do not depend on the settings (AC-SET-5)"
    );
    let link_lines = |lines: &[String]| -> Vec<String> {
        let mut out: Vec<String> = lines
            .iter()
            .filter(|l| l.starts_with("[link] "))
            .map(|line| {
                // The port and the world's instance differ per run; the words do not.
                line.split(' ')
                    .map(|token| {
                        if token.chars().any(|c| c.is_ascii_digit()) {
                            "#"
                        } else {
                            token
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        out.sort();
        out
    };
    assert!(!link_lines(en).is_empty());
    assert_eq!(
        link_lines(en),
        link_lines(zh),
        "the [link] lines stay English (AC-SET-10)"
    );
}

/// AC-SET-7: the language, clock and window size applied in the 2D client are in effect in the 3D
/// client's first frame, through the one shared file. The player's own file is kept and restored.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_settings -- --ignored"]
async fn the_language_chosen_in_2d_is_in_the_3d_clients_first_frame() {
    let (ok, lines) = check(CLIENT_2D, "store_check.gd", &["--where"]);
    assert!(ok);
    let user_dir = first(&lines, "where")["user_dir"]
        .as_str()
        .expect("a folder")
        .to_owned();
    let _planted = Planted::new(&user_dir, "[meta]\nversion=1\n");
    let (address, _log) = stub(StubScript::default()).await;
    let (status, lines) = Drive::start(
        address,
        "carol",
        &[
            "--drive=settings",
            "--settings=user://settings.cfg",
            "--apply=zh_Hans,24h,1280x720",
        ],
    )
    .finish()
    .await;
    assert!(status.success() && passed(&lines), "{}", lines.join("\n"));
    let save = SaveDir::new("settings-3d-first-frame");
    let world = World::start(
        &[
            "server",
            SOCIAL_CAFE,
            "--agent",
            "alice",
            "--save",
            save.path(),
        ],
        None,
    )
    .await;
    let (status, lines) = run_slice(world.address, &["--settings=user://settings.cfg"]);
    assert!(
        status.success() && slice_passed(&lines),
        "{}",
        lines.join("\n")
    );
    let shown = first(&lines, "first_frame");
    assert_eq!(shown["language"], "zh_Hans", "{shown}");
    assert_eq!(shown["clock"], 2, "24-hour: {shown}");
    assert_eq!(shown["window"], serde_json::json!([1280, 720]), "{shown}");
}
