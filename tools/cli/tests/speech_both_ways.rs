//! PR 16g1 — **say what you want to say, in 3D**: the 3D client's typed talk line, checked through the
//! real Godot clients against the real server (`pr-16g1-3d-speech.md` §5).
//!
//! ```text
//! mineworld server <world> --save DIR --hold 0                 the world, persisted
//! milestone_d_cognition.py --config cognition.toml             Alice's `lm` seat, the scripted model
//! godot clients/3d-spike … --slice-market --steps=…            the real 3D client: E, type, Enter
//! godot clients/2d … --drive=steps --steps=…                   the real 2D client: click, talk, type
//! ```
//!
//! AC-3DS-4 is the claim "told in one view, recalled in the other", both ways round. The model is
//! Milestone D's `remembers` (`cognition/lm-controller/tests/milestone_d_cognition.py`): it answers the
//! person who just spoke with the **earliest** line it remembers them saying, and cites that line's
//! Event ID. So in a fresh world the first client tells Alice a line unique to the run, the second
//! client asks her something, and her answer to the second client can only quote the first line if the
//! memory her controller assembled holds it. The evidence is the save's `spoke` facts and her decision
//! log, as in Milestone D — never what a window showed.
//!
//! AC-3DS-1, -2, -3 and -6 are checked by one run with no cognition: the 3D client types an English and
//! a Chinese line, an empty and an over-long one, and the typing gate; the 2D client types the same
//! English line; the server's `differing_fields` compares the two `talk` requests.
//!
//! Every scenario runs in the café town and in a generated variant of it (a scratch copy whose barista
//! has another name), so nothing here holds only for one world. No file under `worlds/` changes.
//!
//! # Where these tests run
//!
//! They need Godot 4.7 and the Python workspace (`uv sync --locked`), so they are `#[ignore]`d in
//! `core` and run by the nightly `speech` layer (`scripts/ci_layer.py`):
//!
//! ```text
//! cargo test -p mineworld-cli --test speech_both_ways -- --ignored --test-threads=1
//! ```

mod godot2d;
mod godot3d;
mod headless;
mod support;

use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use godot2d::tagged;
use godot2d::worlds::{copy_dir, ids};
use godot3d::{evidence, whole};
use mineworld_contracts::ActionRequest;
use mineworld_server::{RequestField, WirePayload, differing_fields};
use serde_json::{Value, json};
use support::{INVITE, SaveDir, Server, run_command};

/// The decision file that assigns Alice to `lm` (Milestone D's).
const DECISIONS: &str = "cognition/lm-controller/examples/alice-lm.decisions.yaml";
/// How long a consult may take to be logged: the consult deadline is 20 s, plus the process's start.
const REPLY_PATIENCE: Duration = Duration::from_secs(60);
/// The barista's name in the generated variant.
const VARIANT_NAME: &str = "Mira Okafor";
/// The words both clients type for AC-3DS-2 (13b's and AC-13's line).
const COFFEE_PLEASE: &str = "Hello! A coffee, please.";
/// AC-3DS-1's Chinese line, sent byte for byte.
const CHINESE: &str = "我叫小魏，我画船。";
/// The most bytes an utterance may have (`systems/conversation` `UTTERANCE_MAX_BYTES`).
const UTTERANCE_MAX_BYTES: usize = 480;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// ── The worlds ─────────────────────────────────────────────────────────────────────────────────────

/// The café town, or a generated variant of it: a scratch copy whose barista is named
/// [`VARIANT_NAME`] (16d D-16d-10's precedent). The guard keeps the copy alive.
struct World {
    pack: String,
    _copy: Option<mineworld_test_support::Scratch>,
}

impl World {
    fn cafe() -> Self {
        Self {
            pack: support::PACK.to_owned(),
            _copy: None,
        }
    }

    fn variant(name: &str) -> Self {
        let dir = mineworld_test_support::scratch!(empty name);
        let pack = dir.join("social-cafe");
        copy_dir(Path::new(support::PACK), &pack);
        let alice = pack.join("people").join("alice.yaml");
        let text = std::fs::read_to_string(&alice).expect("alice.yaml reads");
        assert!(text.contains("name: Alice Moreau"), "the barista is named");
        std::fs::write(
            &alice,
            text.replace("name: Alice Moreau", &format!("name: {VARIANT_NAME}")),
        )
        .expect("the variant is written");
        Self {
            pack: pack.to_str().expect("UTF-8").to_owned(),
            _copy: Some(dir),
        }
    }
}

// ── Alice's seat, as Milestone D runs it ───────────────────────────────────────────────────────────

/// The workspace's own interpreter (Milestone D's `interpreter`).
fn interpreter() -> PathBuf {
    let venv =
        std::env::var_os("VIRTUAL_ENV").map_or_else(|| repository().join(".venv"), PathBuf::from);
    let python = if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    };
    assert!(
        python.is_file(),
        "{} is missing: run `uv sync --locked` at the repository root first",
        python.display()
    );
    python
}

/// The cognition process with the scripted model, on Alice's seat; killed when dropped.
struct Cognition {
    child: Child,
    output: Arc<Mutex<Vec<String>>>,
    store: PathBuf,
    _sink: TcpListener,
}

impl Cognition {
    /// Writes the assignment and the configuration into `root` and starts the process; returns once
    /// Alice's seat is held.
    async fn start(root: &Path, pack: &str, server: &Server) -> Self {
        let decisions = repository().join(DECISIONS);
        let (status, json) = run_command(&[
            "decisions",
            pack,
            "--decisions",
            decisions.to_str().expect("UTF-8"),
            "--json",
        ]);
        assert!(status.success(), "mineworld decisions --json ran");
        std::fs::write(root.join("seats.json"), json).expect("seats.json is written");
        // The backend's address is a listener nothing may connect to: the scripted model needs none.
        let sink = TcpListener::bind("127.0.0.1:0").expect("a sink");
        let port = sink.local_addr().expect("bound").port();
        let config = format!(
            "server = \"ws://{}/ws\"\ninvite_env = \"MINEWORLD_INVITE\"\nnickname = \"cognition\"\n\
             store = \"store\"\ndecisions_json = \"seats.json\"\nseats = [\"alice\"]\n\n\
             [recording]\nmode = \"scripted\"\n\n[tiers]\nsocial = \"local\"\nsummarize = \"none\"\n\n\
             [backends.local]\nbase_url = \"http://127.0.0.1:{port}/v1\"\nmodel = \"speech-model\"\n\
             structured_output = \"none\"\n\n[controller]\nconsult_deadline_s = 20\n",
            server.address
        );
        let path = root.join("cognition.toml");
        std::fs::write(&path, config).expect("the configuration is written");
        let mut child = Command::new(interpreter())
            .arg(repository().join("cognition/lm-controller/tests/milestone_d_cognition.py"))
            .arg("--config")
            .arg(&path)
            .current_dir(root)
            .env("MINEWORLD_INVITE", INVITE)
            .env("PYTHONUNBUFFERED", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the cognition process starts");
        let output = Arc::new(Mutex::new(Vec::new()));
        drain(child.stdout.take().expect("stdout"), &output);
        drain(child.stderr.take().expect("stderr"), &output);
        let cognition = Self {
            child,
            output,
            store: root.join("store"),
            _sink: sink,
        };
        cognition.seat_taken().await
    }

    async fn seat_taken(mut self) -> Self {
        let resume = self.store.join("alice.resume");
        let deadline = Instant::now() + REPLY_PATIENCE;
        while !resume.is_file() {
            if let Some(status) = self.child.try_wait().expect("waitable") {
                panic!(
                    "the cognition process ended ({status}): {:?}",
                    self.output()
                );
            }
            assert!(
                Instant::now() < deadline,
                "Alice's seat was not taken: {:?}",
                self.output()
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        self
    }

    fn output(&self) -> Vec<String> {
        self.output.lock().expect("sound").clone()
    }

    /// The decision lines of Alice's log.
    fn decisions(&self) -> Vec<Value> {
        std::fs::read_to_string(self.store.join("alice.decisions.jsonl"))
            .unwrap_or_default()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|line| line["kind"] == "decision")
            .collect()
    }

    /// Waits until the log holds `acted` consults that acted.
    async fn acted(&self, acted: usize) {
        let deadline = Instant::now() + REPLY_PATIENCE;
        let count = || {
            self.decisions()
                .iter()
                .filter(|line| line["record"]["outcome"] == "acted")
                .count()
        };
        while count() < acted {
            assert!(
                Instant::now() < deadline,
                "{acted} acted consults were not logged: {:?}\n{:?}",
                self.decisions(),
                self.output()
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

impl Drop for Cognition {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn drain(pipe: impl Read + Send + 'static, sink: &Arc<Mutex<Vec<String>>>) {
    let sink = Arc::clone(sink);
    std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            eprintln!("[speech cognition] {line}");
            sink.lock().expect("sound").push(line);
        }
    });
}

// ── The two clients ────────────────────────────────────────────────────────────────────────────────

/// Which client a line is typed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    TwoD,
    ThreeD,
}

/// Plays the real client of `view` on the visitor's seat: it walks up to `alice` and says each of
/// `lines` through its typed line, then lingers so her answer arrives before it leaves. Returns what
/// it printed.
async fn speak(
    view: View,
    server: &Server,
    alice: &str,
    lines: &[&str],
    name: &str,
) -> Vec<String> {
    match view {
        View::TwoD => {
            let mut steps = vec![
                json!({ "open": alice }),
                json!({ "choose": { "walk": true } }),
                json!({ "sleep": 2 }),
            ];
            for line in lines {
                steps.push(json!({ "open": alice }));
                steps.push(json!({ "choose": { "action_type": "talk" }, "input": line }));
            }
            steps.push(json!({ "sleep": 4 }));
            let (status, printed) = godot2d::worlds::play(
                server.address,
                "visitor",
                name,
                &Value::Array(steps),
                &["--world-key=social-cafe"],
            )
            .await;
            assert!(
                status.success() && godot2d::passed(&printed),
                "the 2D drive passes"
            );
            printed
        }
        View::ThreeD => {
            let mut steps = vec![
                json!({ "open": alice }),
                json!({ "choose": { "walk": true } }),
                json!({ "sleep": 2 }),
            ];
            for line in lines {
                steps.push(json!({ "say": line, "to": alice }));
            }
            steps.push(json!({ "sleep": 4 }));
            let (status, printed) = godot3d::play_in(
                "social-cafe",
                server.address,
                "visitor",
                name,
                &Value::Array(steps),
                &godot2d::SLICE_3D_CI,
            );
            assert!(
                status.success() && godot3d::passed(&printed),
                "the 3D probe passes"
            );
            printed
        }
    }
}

/// The `talk` requests a client printed, in order: the line as it went out and the contract's reading.
fn talks(printed: &[String]) -> Vec<(Value, ActionRequest<WirePayload>)> {
    tagged(printed, "REQUEST ")
        .into_iter()
        .filter(|r| r["request"]["action_type"] == "talk")
        .map(|r| {
            let request =
                serde_json::from_value(whole(&r["request"])).expect("a legal ActionRequest");
            (r, request)
        })
        .collect()
}

/// The answer printed for `token`.
fn result_of(printed: &[String], token: &Value) -> Value {
    tagged(printed, "RESULT ")
        .into_iter()
        .find(|r| r["token"] == *token)
        .unwrap_or_else(|| panic!("request {token} was never answered"))
}

// ── The save ───────────────────────────────────────────────────────────────────────────────────────

/// One `spoke` fact: its Event ID, speaker, listener and words.
#[derive(Clone, Debug)]
struct Spoke {
    id: u64,
    speaker: String,
    listener: String,
    words: String,
}

fn id_of(value: &Value) -> u64 {
    value
        .as_str()
        .and_then(|text| text.parse().ok())
        .or_else(|| value.as_u64())
        .unwrap_or_else(|| panic!("an id: {value}"))
}

/// Every `spoke` fact the visitor perceived, in log order (`mineworld perceived --json`).
fn spoken(pack: &str, save: &str) -> Vec<Spoke> {
    let (status, out) = run_command(&[
        "perceived",
        pack,
        "--save",
        save,
        "--person",
        "visitor",
        "--json",
    ]);
    assert!(status.success(), "mineworld perceived ran");
    out.lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("one PerceivedEvent per line"))
        .filter(|fact| fact["event_type"] == "spoke")
        .map(|fact| {
            let body = &fact["payload"]["payload"];
            Spoke {
                id: id_of(&fact["id"]),
                speaker: body["speaker"]["entity"]
                    .as_str()
                    .expect("a speaker")
                    .to_owned(),
                listener: body["listener"]["entity"]
                    .as_str()
                    .expect("a listener")
                    .to_owned(),
                words: body["utterance"].as_str().expect("words").to_owned(),
            }
        })
        .collect()
}

/// Whether a recall (`[#12]`, `[#10-14]`, or a bare id) names `id` (Milestone D's `recall_names`).
fn recall_names(recall: &str, id: u64) -> bool {
    let inner = recall.trim_start_matches('[').trim_end_matches(']');
    inner.split(',').any(|range| {
        let range = range.trim_start_matches('#');
        let (first, last) = range.split_once('-').unwrap_or((range, range));
        match (first.parse::<u64>(), last.parse::<u64>()) {
            (Ok(first), Ok(last)) => (first..=last).contains(&id),
            _ => false,
        }
    })
}

/// A short tag no other run shares, so a line can only be recalled from this run's memory.
fn nonce() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("after 1970")
        .subsec_nanos();
    format!("{:x}{:x}", std::process::id(), nanos)
}

// ── AC-3DS-4: told in one view, recalled in the other ──────────────────────────────────────────────

/// One fresh world: `tells` says a line unique to the run, then `asks` asks Alice whether she
/// remembers; her answer to `asks` must quote the told line and cite its Event ID.
async fn told_then_recalled(world: &World, tells: View, asks: View, told: &str) {
    let label = format!("{tells:?}-{asks:?}").to_lowercase();
    let directory = SaveDir::new(&format!("speech-{label}"));
    let root = PathBuf::from(directory.path());
    std::fs::create_dir_all(&root).expect("the scratch directory");
    let save = root.join("save").to_str().expect("UTF-8").to_owned();
    let town = ids(&world.pack);
    let (alice, visitor) = (town["alice"].clone(), town["visitor"].clone());

    let mut server = Server::start(&["server", &world.pack, "--save", &save, "--hold", "0"]).await;
    let cognition = Cognition::start(&root, &world.pack, &server).await;

    // The first view tells her; she answers it (the scripted model quotes the line itself).
    let first = speak(tells, &server, &alice, &[told], &format!("{label}-tells")).await;
    cognition.acted(1).await;
    // The second view, the same visitor, asks.
    let asked = format!("Do you remember me? ({})", nonce());
    let second = speak(asks, &server, &alice, &[&asked], &format!("{label}-asks")).await;
    cognition.acted(2).await;
    let log = cognition.decisions();
    drop(cognition);
    let killed = server.kill();
    assert!(killed.killed(), "the server was killed: {killed:?}");

    // Each view sent exactly what was typed, to Alice.
    for (printed, words) in [(&first, told), (&second, asked.as_str())] {
        let sent = talks(printed);
        assert_eq!(sent.len(), 1, "{label}: one talk per view: {sent:?}");
        assert_eq!(
            sent[0].0["request"]["target"],
            json!(alice),
            "{label}: to Alice"
        );
        assert_eq!(
            sent[0].0["request"]["payload"]["payload"]["utterance"],
            json!(words),
            "{label}: the typed words, byte for byte"
        );
        assert!(
            result_of(printed, &sent[0].0["token"])["result"]
                .get("accepted")
                .is_some(),
            "{label}: the talk is accepted"
        );
    }

    // The save: the told line, the asked line, and Alice's answer to the asking view.
    let facts = spoken(&world.pack, &save);
    let find = |words: &str| {
        facts
            .iter()
            .find(|f| f.speaker == visitor && f.listener == alice && f.words == words)
            .cloned()
            .unwrap_or_else(|| panic!("{label}: the visitor's line {words:?} is a fact: {facts:?}"))
    };
    let told_fact = find(told);
    let asked_fact = find(&asked);
    assert!(told_fact.id < asked_fact.id, "{label}: told before asked");
    let answer = facts
        .iter()
        .find(|f| f.id > asked_fact.id && f.speaker == alice && f.listener == visitor)
        .unwrap_or_else(|| panic!("{label}: Alice answers the asking view: {facts:?}"));
    assert!(
        answer.words.contains(told),
        "{label}: her answer to the {asks:?} view quotes what the {tells:?} view told her: {:?}",
        answer.words
    );

    // Her decision log: the consult on the asked line cites the told line in its recalls.
    let consult = log
        .iter()
        .find(|c| id_of(&c["input"]["trigger"]["event"]) == asked_fact.id)
        .unwrap_or_else(|| {
            panic!(
                "{label}: a consult on the asked line #{}: {log:?}",
                asked_fact.id
            )
        });
    assert_eq!(consult["record"]["outcome"], "acted", "{label}");
    let recalls: Vec<&str> = consult["record"]["proposal"]["recalls"]
        .as_array()
        .expect("recalls")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        recalls.iter().any(|r| recall_names(r, told_fact.id)),
        "{label}: her recalls cite the told line #{}: {recalls:?}",
        told_fact.id
    );
    assert_eq!(
        consult["record"]["request"]["payload"]["payload"]["utterance"],
        json!(answer.words),
        "{label}: the answer is the request her consult submitted"
    );
    println!(
        "AC-3DS-4 {label}: told #{} in {tells:?}, recalled in {asks:?}: {:?}",
        told_fact.id, answer.words
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs Godot 4.7 and the Python workspace: run by the speech layer, scripts/ci_layer.py"]
async fn told_in_3d_recalled_in_2d() {
    let told = format!("My name is Wei and I paint boats. ({})", nonce());
    told_then_recalled(&World::cafe(), View::ThreeD, View::TwoD, &told).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs Godot 4.7 and the Python workspace: run by the speech layer, scripts/ci_layer.py"]
async fn told_in_2d_recalled_in_3d() {
    let told = format!("I left a red umbrella by the door. ({})", nonce());
    told_then_recalled(&World::cafe(), View::TwoD, View::ThreeD, &told).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs Godot 4.7 and the Python workspace: run by the speech layer, scripts/ci_layer.py"]
async fn told_in_3d_recalled_in_2d_in_a_generated_world() {
    let world = World::variant("speech-variant-3d-2d");
    let told = format!("{CHINESE}（{}）", nonce());
    told_then_recalled(&world, View::ThreeD, View::TwoD, &told).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs Godot 4.7 and the Python workspace: run by the speech layer, scripts/ci_layer.py"]
async fn told_in_2d_recalled_in_3d_in_a_generated_world() {
    let world = World::variant("speech-variant-2d-3d");
    let told = format!("The bakery on the corner opens at six. ({})", nonce());
    told_then_recalled(&world, View::TwoD, View::ThreeD, &told).await;
}

// ── AC-3DS-1, -2, -3, -6: what the 3D line sends, and what typing does not do ──────────────────────

/// The 3D client, in the generated world with Alice unplugged: the typing gate; English typed with the
/// body turned away after E (the line stays pinned); an empty and an over-long line, both sent and
/// refused by the server; Chinese. Then the 2D client types the same English line, and the server's
/// comparison finds only the actor's position differing.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs Godot 4.7: run by the speech layer, scripts/ci_layer.py"]
async fn the_3d_line_sends_what_was_typed_and_typing_does_not_play() {
    let world = World::variant("speech-variant-typed");
    let town = ids(&world.pack);
    let alice = town["alice"].clone();
    let too_long = "w".repeat(UTTERANCE_MAX_BYTES + 1);
    let typed = [COFFEE_PLEASE, "", too_long.as_str(), CHINESE];

    let save_3d = SaveDir::new("speech-typed-3d");
    let mut server = Server::start(&[
        "server",
        &world.pack,
        "--save",
        save_3d.path(),
        "--hold",
        "0",
    ])
    .await;
    let steps = json!([
        { "open": alice }, { "choose": { "walk": true } }, { "sleep": 1 },
        { "typing_gate": alice },
        { "say": typed[0], "to": alice, "turn_deg": 75 },
        { "say": typed[1], "to": alice },
        { "say": typed[2], "to": alice },
        { "say": typed[3], "to": alice },
        { "sleep": 2 }, { "report": true }
    ]);
    let (status, lines_3d) = godot3d::play_in(
        "social-cafe",
        server.address,
        "visitor",
        "speech-typed-3d-steps",
        &steps,
        &godot2d::SLICE_3D_CI,
    );
    let killed = server.kill();
    assert!(killed.killed(), "the server was killed: {killed:?}");
    assert!(
        status.success() && godot3d::passed(&lines_3d),
        "the 3D probe passes"
    );

    // AC-3DS-3: typing holds the body and every action key; Esc closes, sending nothing; E reopens.
    let gate = evidence(&lines_3d, "typing_gate");
    assert_eq!(
        gate["typed"], "wasd e",
        "the keys went into the line: {gate}"
    );
    assert_eq!(
        gate["moved_mm"], 0,
        "W held while typing walks nowhere: {gate}"
    );
    assert_eq!(
        gate["requests_while_open"], 0,
        "no move, no talk while typing: {gate}"
    );
    assert_eq!(
        gate["open_after_typing"], true,
        "typing e did not close or send: {gate}"
    );
    assert_eq!(
        gate["after_esc"],
        json!({ "line_open": false, "settings_open": false, "requests": 0 }),
        "Esc closes the line before the settings menu, and sends nothing: {gate}"
    );
    assert_eq!(gate["reopened"], true, "E opens it again: {gate}");
    assert_eq!(gate["closed_again"], true, "{gate}");
    assert_eq!(gate["requests"], 0, "the gate sent nothing at all: {gate}");

    // AC-3DS-1 / -6: every typed line sent as typed, to the person pinned when E was pressed.
    let sent = talks(&lines_3d);
    let words: Vec<&str> = sent
        .iter()
        .map(|(r, _)| {
            r["request"]["payload"]["payload"]["utterance"]
                .as_str()
                .expect("an utterance")
        })
        .collect();
    assert_eq!(words, typed, "the 3D client sends exactly what was typed");
    for (r, _) in &sent {
        assert_eq!(r["request"]["target"], json!(alice), "pinned to Alice: {r}");
    }
    let turned = tagged(&lines_3d, "STEP_DONE ")
        .into_iter()
        .find(|d| d["step"] == 4)
        .expect("the turned say step completed");
    assert_ne!(
        turned["facing_at_enter"],
        json!(alice),
        "the body was turned away from Alice before Enter: {turned}"
    );
    let answers: Vec<Value> = sent
        .iter()
        .map(|(r, _)| result_of(&lines_3d, &r["token"]))
        .collect();
    for i in [0, 3] {
        assert!(
            answers[i]["result"].get("accepted").is_some(),
            "{:?} is accepted: {}",
            typed[i],
            answers[i]
        );
    }
    for i in [1, 2] {
        assert!(
            answers[i]["result"].get("accepted").is_none(),
            "the server refuses {} bytes: {}",
            typed[i].len(),
            answers[i]
        );
        let toast = toast_before(&lines_3d, &sent[i].0["token"]);
        assert!(
            toast["text"].as_str().is_some_and(|t| !t.is_empty()),
            "the refusal is shown: {toast}"
        );
    }
    let facts = spoken(&world.pack, save_3d.path());
    let said: Vec<&str> = facts
        .iter()
        .filter(|f| f.listener == alice)
        .map(|f| f.words.as_str())
        .collect();
    assert_eq!(
        said,
        [COFFEE_PLEASE, CHINESE],
        "the save holds the two accepted lines, UTF-8 byte for byte"
    );

    // AC-3DS-2: the 2D client types the same line; the two requests differ only in ActorLocation.
    let save_2d = SaveDir::new("speech-typed-2d");
    let mut server = Server::start(&[
        "server",
        &world.pack,
        "--save",
        save_2d.path(),
        "--hold",
        "0",
    ])
    .await;
    let steps = json!([
        { "open": alice }, { "choose": { "walk": true } },
        { "open": alice }, { "choose": { "action_type": "talk" }, "input": COFFEE_PLEASE }
    ]);
    let (status, lines_2d) = godot2d::worlds::play(
        server.address,
        "visitor",
        "speech-typed-2d-steps",
        &steps,
        &["--world-key=social-cafe"],
    )
    .await;
    let killed = server.kill();
    assert!(killed.killed(), "the server was killed: {killed:?}");
    assert!(
        status.success() && godot2d::passed(&lines_2d),
        "the 2D drive passes"
    );
    let two = talks(&lines_2d);
    assert_eq!(two.len(), 1, "one 2D talk: {two:?}");
    let differing = differing_fields(&two[0].1, &sent[0].1);
    assert!(
        differing.iter().all(|f| *f == RequestField::ActorLocation),
        "AC-3DS-2: the typed talk differs between the clients in {differing:?}"
    );
}

/// The toast printed just before the answer to `token` (the slice words an answer as it arrives).
fn toast_before(printed: &[String], token: &Value) -> Value {
    let at = printed
        .iter()
        .position(|line| {
            line.strip_prefix("RESULT ")
                .and_then(|rest| serde_json::from_str::<Value>(rest).ok())
                .is_some_and(|r| r["token"] == *token)
        })
        .unwrap_or_else(|| panic!("request {token} was never answered"));
    printed[..at]
        .iter()
        .rev()
        .find_map(|line| line.strip_prefix("TOAST "))
        .map(|rest| serde_json::from_str(rest).expect("JSON"))
        .unwrap_or_else(|| panic!("no toast before the answer to {token}"))
}
