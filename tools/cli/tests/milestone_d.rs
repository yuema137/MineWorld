//! Milestone D — **speak to Alice in 2D, meet her in 3D, and she reacts consistently with what
//! happened** (`docs/HUMAN_REVIEW_QUEUE.md`; `step-17-cognition.md` §3.14; `pr-s10-p7-milestone-d.md`).
//!
//! ```text
//! mineworld server worlds/social-cafe --save DIR --hold 3                 the world, persisted
//! python -m mineworld_cognition --config cognition.toml                   Alice's `lm` seat
//! a protocol client tagged `2d` on the `visitor` seat                     "the 2D window"
//!    SIGKILL both, start both again
//! a protocol client tagged `3d` on the `visitor` seat                     "the 3D window"
//! ```
//!
//! The claim is made checkable as four statements (§3.14.1), and each is asserted from an artefact a
//! reviewer can open, never from what a window looked like:
//!
//! ```text
//! same world     one world instance, in both welcomes, across the restart
//! same Alice     one EntityId, found by her `barista` tag, never by a literal
//! same memory    the context her 3D reply was decided from (her decision log) cites the Event IDs of
//!                the 2D lines (the save's facts); her store resumed from its cursor, not from scratch
//! consistent     her reply's `recalls` cites one of those ids, and the words it produced are a `spoke`
//!                fact in the save, caused by a request her seat submitted
//! ```
//!
//! # The model, and where the recorded one is checked (D-P7-1)
//!
//! Everything here is the shipped controller — assignment, seat runner, memory store, triggers, the
//! consult engine, the decision log — started as a process the way an operator starts it. Only the
//! model is scripted (`cognition/lm-controller/tests/milestone_d_cognition.py`): a pure function of the
//! request that quotes the earliest line it is shown as said to it and cites that line's token. A live
//! hosted run is not reproducible at the request level (P6 F-P6-5), so a strict cassette cannot answer
//! it; the recorded local-model run of this scenario is replayed from its decision log by
//! `cognition/lm-controller/tests/test_milestone_d_replay.py` (R-P7-P6-1). No model is reachable here:
//! the backend's address is a listener this test opens, and it must accept nothing.
//!
//! # Where these tests run (D-P7-4)
//!
//! The model-driven test needs the Python workspace, which the `core` job does not install, so it is
//! ignored with a reason and run by the `python` layer (`scripts/ci_layer.py`), never skipped there.
//! The unplugged run needs no Python and runs everywhere.

mod support;

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mineworld_contracts::EntityId;
use mineworld_server::{ServerFrame, WorldSummary};
use mineworld_test_support::process;
use serde_json::{Value, json};
use support::{
    Client, INVITE, SaveDir, Server, may_talk_to, own_history, run_command, tagged, talk,
};

/// What the 2D window says (step-17 §3.14.2 step 2), and what the 3D window says (step 5).
const FROM_THE_2D_WINDOW: &str = "Hi, I'm new in town. I left a red umbrella here yesterday.";
const FROM_THE_3D_WINDOW: &str = "Hello again.";

/// Where the pack seats the visitor (`worlds/social-cafe/people/visitor.yaml`), and a place at the
/// counter inside `talk`'s three-metre reach of Alice, who stands at (6000, 8000) mm.
const AT_THE_DOOR: (i32, i32) = (1_610, 600);
const AT_THE_COUNTER: (i32, i32) = (6_000, 6_200);

/// How long a reply may take: the consult deadline is 20 s (P6 §5.7), plus the process's start-up.
const REPLY_PATIENCE: Duration = Duration::from_secs(60);
/// How long the unplugged world is watched for a reply that must not come: past any consult.
const SILENCE: Duration = Duration::from_secs(6);
/// How long the visitor stands at the counter before speaking (D-P6b-30).
const SETTLE: Duration = Duration::from_secs(2);
/// The seat's hold after a dropped socket (`--hold`), short so the restart is not waiting on it.
const HOLD: &str = "3";

/// A key planted in the cognition process's environment, which no artefact may contain (IC-10).
const PLANTED_KEY_ENV: &str = "MILESTONE_D_FAKE_KEY";
const PLANTED_KEY: &str = "sk-milestone-d-planted-4f1c9e2a7b3d5068";

/// The decision file that assigns Alice to `lm`, falling back to `rule` (P6 §5.9).
const DECISIONS: &str = "cognition/lm-controller/examples/alice-lm.decisions.yaml";

/// Which model the cognition process asks.
#[derive(Clone)]
enum Model {
    /// `tests/milestone_d_cognition.py`: the controller with a scripted model.
    Scripted,
    /// `python -m mineworld_cognition` in `record` mode against a local OpenAI-compatible server.
    Recording { base_url: String, model: String },
}

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The workspace's own interpreter, started directly so that killing the child kills the process that
/// holds Alice's socket (a killed `uv run` could leave its child behind; R-P7-3).
fn interpreter() -> PathBuf {
    // `uv run` names the environment it activated; otherwise the workspace's default, `.venv`.
    let venv =
        std::env::var_os("VIRTUAL_ENV").map_or_else(|| repository().join(".venv"), PathBuf::from);
    let python = if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    };
    assert!(
        python.is_file(),
        "{} is missing: run `uv sync --locked` at the repository root first (the python layer does)",
        python.display()
    );
    python
}

/// One cognition process, its output kept in memory for the planted-key scan.
struct Cognition {
    child: Child,
    output: Arc<Mutex<Vec<String>>>,
}

impl Cognition {
    fn start(directory: &Path, server: &Server, model: &Model, phase: u32) -> Self {
        let config = write_config(
            directory,
            &format!("ws://{}/ws", server.address),
            model,
            phase,
        );
        let mut command = Command::new(interpreter());
        match model {
            Model::Scripted => command
                .arg(repository().join("cognition/lm-controller/tests/milestone_d_cognition.py")),
            Model::Recording { .. } => command.args(["-m", "mineworld_cognition"]),
        };
        command
            .arg("--config")
            .arg(&config)
            .current_dir(directory)
            .env("MINEWORLD_INVITE", INVITE)
            .env(PLANTED_KEY_ENV, PLANTED_KEY)
            .env("PYTHONUNBUFFERED", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().expect("the cognition process starts");
        let output = Arc::new(Mutex::new(Vec::new()));
        drain(child.stdout.take().expect("stdout"), &output);
        drain(child.stderr.take().expect("stderr"), &output);
        Self { child, output }
    }

    fn output(&self) -> Vec<String> {
        self.output.lock().expect("sound").clone()
    }

    fn kill(&mut self) {
        let killed = process::kill(&mut self.child);
        assert!(
            killed.killed(),
            "the cognition process was killed, not finished: {:?}",
            self.output()
        );
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
            eprintln!("[milestone-d cognition] {line}");
            sink.lock().expect("sound").push(line);
        }
    });
}

/// `cognition.toml` (P6 §5.13). The backend's address is the sink this test listens on in the
/// scripted run; its key is read from the planted variable only in `live` or `record` mode, which the
/// scripted run never uses.
fn write_config(directory: &Path, server: &str, model: &Model, phase: u32) -> PathBuf {
    let (recording, backend) = match model {
        Model::Scripted => (
            "mode = \"scripted\"".to_owned(),
            format!(
                "base_url = \"{}\"\nmodel = \"milestone-d-model\"\nstructured_output = \"none\"\nkey_env = \"{PLANTED_KEY_ENV}\"",
                sink_url(directory)
            ),
        ),
        Model::Recording { base_url, model } => (
            format!("mode = \"record\"\ncassette = \"cassette-{phase}.jsonl\""),
            format!(
                "base_url = \"{base_url}\"\nmodel = \"{model}\"\nstructured_output = \"none\"\nreasoning = \"off\""
            ),
        ),
    };
    let text = format!(
        "server = \"{server}\"\ninvite_env = \"MINEWORLD_INVITE\"\nnickname = \"cognition\"\n\
         store = \"store\"\ndecisions_json = \"seats.json\"\nseats = [\"alice\"]\n\n\
         [recording]\n{recording}\n\n[tiers]\nsocial = \"local\"\nsummarize = \"none\"\n\n\
         [backends.local]\n{backend}\n\n[controller]\nconsult_deadline_s = 20\n"
    );
    let path = directory.join("cognition.toml");
    std::fs::write(&path, text).expect("the configuration is written");
    path
}

fn sink_url(directory: &Path) -> String {
    let port = std::fs::read_to_string(directory.join("sink.port")).expect("the sink's port");
    format!("http://127.0.0.1:{}/v1", port.trim())
}

/// `mineworld decisions … --json`, the assignment the cognition process reads (P6 §5.9 step 3).
fn write_seats(directory: &Path) {
    let decisions = repository().join(DECISIONS);
    let (status, json) = run_command(&[
        "decisions",
        support::PACK,
        "--decisions",
        decisions.to_str().expect("UTF-8"),
        "--json",
    ]);
    assert!(status.success(), "mineworld decisions --json ran");
    std::fs::write(directory.join("seats.json"), json).expect("seats.json is written");
}

/// Waits until Alice's seat is held by the cognition process: its resume file is written after the
/// welcome (P6 §5.10).
async fn seat_taken(store: &Path, cognition: &mut Cognition) {
    let resume = store.join("alice.resume");
    let deadline = Instant::now() + REPLY_PATIENCE;
    while !resume.is_file() {
        if let Some(status) = cognition.child.try_wait().expect("waitable") {
            panic!(
                "the cognition process ended ({status}): {:?}",
                cognition.output()
            );
        }
        assert!(
            Instant::now() < deadline,
            "Alice's seat was not taken within {REPLY_PATIENCE:?}: {:?}",
            cognition.output()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// A protocol client on the visitor's seat, tagged as the window it stands for (as `AC-13`'s harness is).
async fn window(server: &Server, tag: &str) -> (Client, EntityId, WorldSummary) {
    let mut client = Client::connect(server.address).await;
    let answer = client
        .join_as(json!({ "t": "join", "protocol": 2, "invite": INVITE,
                         "nickname": format!("visitor-{tag}"), "seat": "visitor" }))
        .await;
    match answer {
        ServerFrame::Welcome {
            observer, world, ..
        } => (client, observer, world),
        other => panic!("the {tag} window is welcomed: {other:?}"),
    }
}

/// Walks the visitor to `to` inside its place, by the route the server plans (`ARC-75`): the café's
/// chairs are solids (ARC-306), so a straight line a test drew could cross one.
async fn walk_to(client: &mut Client, visitor: EntityId, to: (i32, i32)) {
    let observation = client.observation().await;
    let place = observation
        .self_location()
        .expect("the visitor knows where it is")
        .place()
        .entity_id();
    client.walk_there(visitor, place, Some(to)).await;
}

/// The next line Alice says to this window after the `already` lines it has heard from her.
async fn reply(client: &mut Client, alice: EntityId, already: usize, store: &Path) -> String {
    let deadline = Instant::now() + REPLY_PATIENCE;
    loop {
        let observation = client.observation().await;
        let lines: Vec<String> = own_history(&observation)
            .map(|history| {
                history
                    .heard()
                    .iter()
                    .filter(|heard| heard.speaker().entity_id() == alice)
                    .map(|heard| heard.utterance().as_str().to_owned())
                    .collect()
            })
            .unwrap_or_default();
        if lines.len() > already {
            return lines[already].clone();
        }
        assert!(
            Instant::now() < deadline,
            "Alice did not answer within {REPLY_PATIENCE:?}; her decision log: {}",
            std::fs::read_to_string(store.join("alice.decisions.jsonl")).unwrap_or_default()
        );
    }
}

/// Waits until Alice's decision log holds `acted` consults that acted: a reply reaches the visitor as soon
/// as the world accepts it, and the consult is logged just after, so a kill must wait for the log.
async fn logged(store: &Path, acted: usize) {
    let deadline = Instant::now() + REPLY_PATIENCE;
    let count = || {
        std::fs::read_to_string(store.join("alice.decisions.jsonl"))
            .unwrap_or_default()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|line| line["kind"] == "decision" && line["record"]["outcome"] == "acted")
            .count()
    };
    while count() < acted {
        assert!(
            Instant::now() < deadline,
            "{acted} acted consults were not logged"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Watches the window for `SILENCE` and returns how many lines it heard from Alice.
async fn lines_from(client: &mut Client, alice: EntityId, watch: Duration) -> usize {
    let until = Instant::now() + watch;
    let mut heard = 0;
    while Instant::now() < until {
        let observation = client.observation().await;
        heard = own_history(&observation).map_or(0, |history| {
            history
                .heard()
                .iter()
                .filter(|line| line.speaker().entity_id() == alice)
                .count()
        });
    }
    heard
}

/// Speaks from the counter, once the server offers `talk` to Alice there, and returns the facts.
async fn say(client: &mut Client, visitor: EntityId, alice: EntityId, words: &str) -> Vec<u64> {
    client
        .observation_where("talk to Alice available", |observation| {
            may_talk_to(observation, alice)
        })
        .await;
    // A person who has just walked up waits a moment before speaking. Alice's view of the walk can trail
    // the visitor's own by a frame or more, and her prompt is built from her newest view (P6 D-P6b-30):
    // spoken at once, the line can be decided on a view in which the visitor is still out of reach.
    lines_from(client, alice, SETTLE).await;
    let (_, facts) = client.submit_accepted(talk(visitor, alice, words)).await;
    facts.iter().map(|id| id.raw()).collect()
}

/// The facts a Person perceived, as `mineworld perceived --json` exports them from the save (the
/// offline half of the stream the cognition process reads; `ARC-43`).
fn perceived(save: &str, person: &str) -> Vec<Value> {
    let (status, out) = run_command(&[
        "perceived",
        support::PACK,
        "--save",
        save,
        "--person",
        person,
        "--json",
    ]);
    assert!(status.success(), "mineworld perceived ran");
    out.lines()
        .map(|line| serde_json::from_str(line).expect("one PerceivedEvent per line"))
        .collect()
}

fn id_of(value: &Value) -> u64 {
    value
        .as_str()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| panic!("an id: {value}"))
}

/// The `spoke` facts in `facts`, as (event id, speaker, listener, utterance, causing action).
fn spoken(facts: &[Value]) -> Vec<(u64, String, String, String, Option<u64>)> {
    facts
        .iter()
        .filter(|fact| fact["event_type"] == "spoke")
        .map(|fact| {
            let body = &fact["payload"]["payload"];
            (
                id_of(&fact["id"]),
                body["speaker"]["entity"]
                    .as_str()
                    .expect("a speaker")
                    .to_owned(),
                body["listener"]["entity"]
                    .as_str()
                    .expect("a listener")
                    .to_owned(),
                body["utterance"].as_str().expect("words").to_owned(),
                fact["caused_by"].get("action").map(id_of),
            )
        })
        .collect()
}

/// The actor of the request the save journaled for `action` (`input.dispatch.intent.actor`).
fn actor_of(save: &str, action: u64) -> String {
    let file = Path::new(save).join("world.sqlite");
    let connection = rusqlite::Connection::open(file).expect("the save opens");
    let entry: Vec<u8> = connection
        .query_row(
            "SELECT entry FROM journal WHERE action_id = ?1",
            [i64::try_from(action).expect("small")],
            |row| row.get(0),
        )
        .expect("the action is journaled");
    let entry: Value = serde_json::from_slice(&entry).expect("a JSON journal entry");
    entry["input"]["dispatch"]["intent"]["actor"]
        .as_str()
        .expect("a request's actor")
        .to_owned()
}

/// Alice's memory: the home's creation marker, her cursor, and her held records as (rowid, event id), in the
/// order she took them. Since S28-a the file is a memory home of schema 4 (`memory/home/`, `ARC-109`): her rows
/// are her seat's `meta` and `access` rows, each `access` row naming a shared `fact` record. The marker
/// (`home_meta.created`, `ARC-152`) is written once when the home is created and never again, so it is what
/// tells a resumed home from one a replay rebuilt with the same rows. Opened for writing, never written: a home
/// a SIGKILL left with a hot journal can only be read once SQLite has rolled that journal back, which a
/// read-only connection may not do. Since S28-f the home is the process's, one per world instance, at
/// `<store>/<instance>/memory.sqlite` (`[memory] sharing = "home"`, `pr-s28f-home-wiring.md` §5.4).
fn store_rows(store: &Path) -> (String, Option<u64>, Vec<(i64, u64)>) {
    let homes: Vec<_> = std::fs::read_dir(store)
        .expect("the store directory")
        .map(|entry| entry.expect("a store entry").path().join("memory.sqlite"))
        .filter(|path| path.is_file())
        .collect();
    let [home] = homes.as_slice() else {
        panic!("one memory home under the store, found {homes:?}");
    };
    let connection = rusqlite::Connection::open(home).expect("Alice's memory opens");
    let created: String = connection
        .query_row(
            "SELECT value FROM home_meta WHERE key = 'created'",
            [],
            |row| row.get(0),
        )
        .expect("the home has its creation marker");
    let cursor: String = connection
        .query_row(
            "SELECT m.value FROM meta m JOIN seats s ON s.seat = m.seat \
             WHERE s.key = 'alice' AND m.key = 'cursor'",
            [],
            |row| row.get(0),
        )
        .expect("Alice's seat has a cursor row");
    let mut statement = connection
        .prepare(
            "SELECT a.rowid, f.event_id FROM access a JOIN fact f ON f.event_key = a.event_key \
             JOIN seats s ON s.seat = a.seat WHERE s.key = 'alice' ORDER BY a.rowid",
        )
        .expect("the access and fact tables");
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .expect("rows")
        .map(|row| {
            let (rowid, id) = row.expect("a row");
            (rowid, id.parse().expect("a numeric event id"))
        })
        .collect();
    (
        created,
        (!cursor.is_empty()).then(|| cursor.parse().expect("a numeric cursor")),
        rows,
    )
}

/// The decision lines of Alice's log (`controller/log.py`).
fn decisions(store: &Path) -> Vec<Value> {
    let text = std::fs::read_to_string(store.join("alice.decisions.jsonl")).expect("the log");
    text.lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("JSON"))
        .filter(|line| line["kind"] == "decision")
        .collect()
}

/// Every Event ID a consult's memory section cites (`MemorySection.citations()`).
fn cited(consult: &Value) -> BTreeSet<u64> {
    let mut ids = BTreeSet::new();
    for line in consult["input"]["memory"]["lines"]
        .as_array()
        .expect("lines")
    {
        for range in line["citation"]["ranges"].as_array().expect("ranges") {
            ids.extend(id_of(&range["first"])..=id_of(&range["last"]));
        }
    }
    ids
}

/// Whether a recall (a shown token such as `[#12]` or `[#10-14]`, or a bare id) names `id`.
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

/// The bytes of every file under `directory` contain `needle`?
fn anywhere(directory: &Path, needle: &[u8]) -> Option<PathBuf> {
    support::file_containing(directory, needle)
}

/// What the run left for a reviewer, and the ids the assertions need.
struct Run {
    directory: SaveDir,
    two_d_ids: (u64, u64),
}

/// Steps 1–7 of step-17 §3.14.2, with `model` asked by the cognition process.
#[allow(clippy::too_many_lines)]
async fn she_remembers(model: &Model) -> Run {
    let directory = SaveDir::new("milestone-d");
    let root = PathBuf::from(directory.path());
    std::fs::create_dir_all(&root).expect("the scratch directory");
    let save = root.join("save");
    let save = save.to_str().expect("UTF-8").to_owned();
    let store = root.join("store");
    write_seats(&root);
    let sink = TcpListener::bind("127.0.0.1:0").expect("a sink for the backend's address");
    sink.set_nonblocking(true).expect("non-blocking");
    std::fs::write(
        root.join("sink.port"),
        sink.local_addr().expect("bound").port().to_string(),
    )
    .expect("the sink's port");

    // ── Step 1: the world and Alice's cognition. ────────────────────────────────────────────
    let mut server =
        Server::start(&["server", support::PACK, "--save", &save, "--hold", HOLD]).await;
    let mut cognition = Cognition::start(&root, &server, model, 1);
    seat_taken(&store, &mut cognition).await;

    // ── Step 2: the 2D window speaks to her. ────────────────────────────────────────────────
    let (mut two_d, visitor, world_2d) = window(&server, "2d").await;
    let seen = two_d.observation().await;
    let alice = tagged(&seen, "barista").expect("the 2D window perceives the barista");
    walk_to(&mut two_d, visitor, AT_THE_COUNTER).await;
    let line_2d = say(&mut two_d, visitor, alice, FROM_THE_2D_WINDOW).await;

    // ── Step 3: Alice's seat decides and replies. ───────────────────────────────────────────
    let reply_2d = reply(&mut two_d, alice, 0, &store).await;
    logged(&store, 1).await;
    println!("2D: the visitor said {FROM_THE_2D_WINDOW:?}; Alice answered {reply_2d:?}");
    walk_to(&mut two_d, visitor, AT_THE_DOOR).await;
    two_d.disconnect().await;

    // ── Step 4: SIGKILL both; the store as the kill left it; start both again. ──────────────
    cognition.kill();
    let killed = server.kill();
    assert!(
        killed.killed(),
        "the server was killed, not finished: {killed:?}"
    );
    let (created_before, cursor_before, rows_before) = store_rows(&store);
    let cursor_before =
        cursor_before.expect("Alice's store had ingested something before the kill");
    if let Model::Recording { .. } = model {
        keep_partial(&root, 1);
    }
    let mut server =
        Server::start(&["server", support::PACK, "--save", &save, "--hold", HOLD]).await;
    let mut cognition = Cognition::start(&root, &server, model, 2);
    seat_taken(&store, &mut cognition).await;

    // ── Step 5: the 3D window walks in from the door and speaks. ────────────────────────────
    let (mut three_d, visitor_again, world_3d) = window(&server, "3d").await;
    assert_eq!(
        world_2d.instance, world_3d.instance,
        "both windows, either side of the restart, were told the identity of ONE world"
    );
    assert_eq!(visitor, visitor_again, "the same visitor");
    let seen = three_d.observation().await;
    assert_eq!(
        tagged(&seen, "barista"),
        Some(alice),
        "the barista the 3D window meets is the SAME EntityId"
    );
    walk_to(&mut three_d, visitor, AT_THE_COUNTER).await;
    let line_3d = say(&mut three_d, visitor, alice, FROM_THE_3D_WINDOW).await;
    let reply_3d = reply(&mut three_d, alice, 1, &store).await;
    logged(&store, 2).await;
    println!("3D: the visitor said {FROM_THE_3D_WINDOW:?}; Alice answered {reply_3d:?}");
    three_d.disconnect().await;
    cognition.kill();
    let output = cognition.output();
    let killed = server.kill();
    assert!(
        killed.killed(),
        "the server was killed at the end: {killed:?}"
    );
    if let Model::Recording { .. } = model {
        keep_partial(&root, 2);
    }

    // ── The evidence, read from the save, the store and the log. ────────────────────────────
    let facts = spoken(&perceived(&save, "visitor"));
    let alice_text = alice.to_string();
    let visitor_text = visitor.to_string();
    let find_line = |ids: &[u64]| {
        facts
            .iter()
            .find(|fact| ids.contains(&fact.0))
            .unwrap_or_else(|| panic!("a spoke among the facts {ids:?}"))
            .clone()
    };
    let visitor_2d = find_line(&line_2d);
    let visitor_3d = find_line(&line_3d);
    assert_eq!(visitor_2d.3, FROM_THE_2D_WINDOW);
    assert_eq!(visitor_3d.3, FROM_THE_3D_WINDOW);
    let from_alice: Vec<_> = facts
        .iter()
        .filter(|fact| fact.1 == alice_text && fact.2 == visitor_text)
        .collect();
    assert_eq!(
        from_alice.len(),
        2,
        "Alice said exactly two lines to the visitor, one per window: {from_alice:?}"
    );
    let (alice_2d, alice_3d) = (from_alice[0], from_alice[1]);
    assert!(
        alice_2d.0 > visitor_2d.0 && alice_2d.0 < visitor_3d.0,
        "her first reply answers the 2D line"
    );
    assert!(
        alice_3d.0 > visitor_3d.0,
        "her second reply answers the 3D line"
    );
    assert_eq!(
        (alice_2d.3.as_str(), alice_3d.3.as_str()),
        (reply_2d.as_str(), reply_3d.as_str())
    );

    let log = decisions(&store);
    for (fact, words) in [(alice_2d, &reply_2d), (alice_3d, &reply_3d)] {
        let action = fact.4.expect("Alice's line was caused by a request");
        assert_eq!(
            actor_of(&save, action),
            alice_text,
            "the request that caused Alice's line was submitted as Alice"
        );
        assert!(
            log.iter()
                .any(|consult| consult["record"]["outcome"] == "acted"
                    && consult["record"]["request"]["payload"]["payload"]["utterance"]
                        == words.as_str()),
            "and it is the request her seat's consult submitted: {words:?}"
        );
    }

    // ── Step 6: the 3D reply was decided from the 2D lines, and cites them. ─────────────────
    let two_d_ids = (visitor_2d.0, alice_2d.0);
    let consult = log
        .iter()
        .find(|consult| id_of(&consult["input"]["trigger"]["event"]) == visitor_3d.0)
        .unwrap_or_else(|| panic!("a consult on the 3D line {}: {log:?}", visitor_3d.0));
    assert_eq!(consult["record"]["outcome"], "acted");
    let context = cited(consult);
    for id in [two_d_ids.0, two_d_ids.1] {
        assert!(
            context.contains(&id),
            "IC-8: the context of Alice's 3D reply holds the 2D line #{id}; it cites {context:?}"
        );
    }
    let recalls: Vec<&str> = consult["record"]["proposal"]["recalls"]
        .as_array()
        .expect("recalls")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        recalls
            .iter()
            .any(|recall| recall_names(recall, two_d_ids.0) || recall_names(recall, two_d_ids.1)),
        "her reply's recalls cite a 2D line: {recalls:?}"
    );

    // Step 4's second half: the store resumed from its cursor rather than being re-ingested.
    let (created_after, cursor_after, rows_after) = store_rows(&store);
    assert_eq!(
        created_after, created_before,
        "the home's creation marker is the one it had before the kill: the store resumed, it was not \
         deleted and rebuilt by a replay (which recreates the same rows and cursor)"
    );
    assert!(
        cursor_after.expect("a cursor after the restart") >= cursor_before,
        "the cursor did not go back"
    );
    assert!(
        rows_before.iter().all(|row| rows_after.contains(row)),
        "every fact ingested before the kill is the same row after it: the store resumed, it was not \
         rebuilt from scratch"
    );

    // ── Step 7: every cause in the save resolves. ───────────────────────────────────────────
    let (status, _) = run_command(&["inspect", &save, "--last", "0"]);
    assert!(status.success(), "mineworld inspect resolves every cause");

    // ── No model was reachable, and the planted key is nowhere (IC-5, IC-10). ───────────────
    if let Model::Scripted = model {
        assert!(
            matches!(sink.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
            "nothing connected to the backend's address"
        );
    }
    assert_eq!(
        anywhere(&root, PLANTED_KEY.as_bytes()),
        None,
        "the planted key is in no file"
    );
    assert!(
        !output.iter().any(|line| line.contains(PLANTED_KEY)),
        "the planted key is in no line the cognition process printed"
    );
    Run {
        directory,
        two_d_ids,
    }
}

/// A SIGKILLed `record` session leaves its cassette as `.partial`, every entry flushed (`record.py`);
/// it is kept under its own name so the next phase records a cassette of its own.
fn keep_partial(root: &Path, phase: u32) {
    let partial = root.join(format!("cassette-{phase}.jsonl.partial"));
    if partial.is_file() {
        std::fs::rename(&partial, root.join(format!("cassette-{phase}.jsonl"))).expect("kept");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the Python workspace (uv sync --locked): run by the python layer, scripts/ci_layer.py"]
async fn she_remembers_across_the_two_views() {
    let run = she_remembers(&Model::Scripted).await;
    println!(
        "Milestone D: the 2D lines #{} and #{} were in the context of Alice's 3D reply and cited by it ({})",
        run.two_d_ids.0,
        run.two_d_ids.1,
        run.directory.path()
    );
}

/// Records `cognition/lm-controller/tests/cassettes/milestone_d.jsonl` and its decision log from this
/// scenario, against a local model (QS10-19: local models only, no key). Operator-run:
///
/// ```sh
/// ollama serve &
/// MINEWORLD_MILESTONE_D_RECORD=/tmp/milestone-d-recording \
///   cargo test -p mineworld-cli --test milestone_d -- --ignored --exact record_the_milestone_d_cassette
/// ```
///
/// `MINEWORLD_MILESTONE_D_BASE_URL` and `MINEWORLD_MILESTONE_D_MODEL` override the default local
/// binding (`http://127.0.0.1:11434/v1`, `qwen3.5:4b`, the admitted local triple of P6 §0.1).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "operator-run: records the cassette from a local model"]
async fn record_the_milestone_d_cassette() {
    let out = std::env::var("MINEWORLD_MILESTONE_D_RECORD")
        .expect("MINEWORLD_MILESTONE_D_RECORD names the directory the recording is written to");
    let model = Model::Recording {
        base_url: std::env::var("MINEWORLD_MILESTONE_D_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:11434/v1".to_owned()),
        model: std::env::var("MINEWORLD_MILESTONE_D_MODEL")
            .unwrap_or_else(|_| "qwen3.5:4b".to_owned()),
    };
    let run = she_remembers(&model).await;
    let root = PathBuf::from(run.directory.path());
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).expect("the output directory");
    let mut cassette = String::new();
    for phase in [1, 2] {
        let text = std::fs::read_to_string(root.join(format!("cassette-{phase}.jsonl")))
            .expect("each phase recorded a cassette");
        let mut lines = text.lines();
        let header = lines.next().expect("a header");
        if cassette.is_empty() {
            cassette.push_str(header);
            cassette.push('\n');
        }
        for line in lines {
            cassette.push_str(line);
            cassette.push('\n');
        }
    }
    std::fs::write(out.join("milestone_d.jsonl"), cassette).expect("the cassette");
    std::fs::copy(
        root.join("store/alice.decisions.jsonl"),
        out.join("milestone_d.decisions.jsonl"),
    )
    .expect("the decision log");
    println!(
        "recorded into {}: the 2D lines are #{} and #{}",
        out.display(),
        run.two_d_ids.0,
        run.two_d_ids.1
    );
}

/// Step 8 — **the same run with Alice's seat unplugged**: no cognition process. The world runs, the
/// visitor's lines are recorded, and Alice says nothing. This is the "world without models" line
/// (step-17 §3.16.1), and IC-7 runs it again with `cognition/lm-controller/` deleted.
#[tokio::test(flavor = "multi_thread")]
async fn the_world_runs_with_alice_unplugged() {
    let directory = SaveDir::new("milestone-d-unplugged");
    std::fs::create_dir_all(directory.path()).expect("scratch");
    let save = Path::new(directory.path()).join("save");
    let save = save.to_str().expect("UTF-8").to_owned();

    let mut server =
        Server::start(&["server", support::PACK, "--save", &save, "--hold", HOLD]).await;
    let (mut two_d, visitor, world_2d) = window(&server, "2d").await;
    let seen = two_d.observation().await;
    let alice = tagged(&seen, "barista").expect("the barista is there, unplugged");
    walk_to(&mut two_d, visitor, AT_THE_COUNTER).await;
    let line_2d = say(&mut two_d, visitor, alice, FROM_THE_2D_WINDOW).await;
    assert_eq!(
        lines_from(&mut two_d, alice, SILENCE).await,
        0,
        "unplugged, Alice says nothing"
    );
    walk_to(&mut two_d, visitor, AT_THE_DOOR).await;
    two_d.disconnect().await;
    let killed = server.kill();
    assert!(killed.killed(), "the server was killed: {killed:?}");

    let mut server =
        Server::start(&["server", support::PACK, "--save", &save, "--hold", HOLD]).await;
    let (mut three_d, _, world_3d) = window(&server, "3d").await;
    assert_eq!(
        world_2d.instance, world_3d.instance,
        "one world across the restart"
    );
    walk_to(&mut three_d, visitor, AT_THE_COUNTER).await;
    let line_3d = say(&mut three_d, visitor, alice, FROM_THE_3D_WINDOW).await;
    assert_eq!(
        lines_from(&mut three_d, alice, SILENCE).await,
        0,
        "and nothing after the restart"
    );
    three_d.disconnect().await;
    let killed = server.kill();
    assert!(
        killed.killed(),
        "the server was killed at the end: {killed:?}"
    );

    let facts = spoken(&perceived(&save, "visitor"));
    let words: Vec<&str> = facts.iter().map(|fact| fact.3.as_str()).collect();
    assert_eq!(
        words,
        [FROM_THE_2D_WINDOW, FROM_THE_3D_WINDOW],
        "the visitor's two lines are recorded, and they are the only lines spoken"
    );
    assert!(facts.iter().any(|fact| line_2d.contains(&fact.0)));
    assert!(facts.iter().any(|fact| line_3d.contains(&fact.0)));
    let (status, _) = run_command(&["inspect", &save, "--last", "0"]);
    assert!(status.success(), "mineworld inspect resolves every cause");
}
