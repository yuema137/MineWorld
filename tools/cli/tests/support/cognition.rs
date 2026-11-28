//! The LM controller as an operator starts it, and the artefacts it leaves, for the tests that put the real
//! cognition process on a seat of the real server (Milestone D, `milestone_d.rs`; the corrected-memory
//! scenario, `correction_memory.rs`; `pr-playable-dialogue-memory.md` §5 C4).
//!
//! ```text
//! <workspace python> <script or -m mineworld_cognition> --config cognition.toml     the process
//! <store>/<seat>.resume, <store>/<seat>.decisions.jsonl                               what it writes
//! <store>/<instance>/memory.sqlite                                                    its memory home
//! ```
//!
//! Nothing here interprets a model's words: these are the readers of the save, the store and the decision
//! log that every assertion is made from, shared so that no test grows a parallel harness.

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mineworld_test_support::process::{self, InterruptibleChild, Killed};
use serde_json::Value;

use super::{INVITE, run_command};

/// How long a cognition process may take to hold its seat, and a consult to be logged.
pub const SEAT_PATIENCE: Duration = Duration::from_secs(60);

/// The repository root.
pub fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The workspace's own interpreter, started directly so that stopping the child stops the process that
/// holds the seat's socket (a killed `uv run` could leave its child behind; R-P7-3).
pub fn interpreter() -> PathBuf {
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

/// What the cognition process runs: a test's scripted entry (a file beside the shipped controller that
/// calls its `main` with a scripted model), or the shipped module itself.
pub enum Program {
    Script(PathBuf),
    Module,
}

/// One cognition process, its output kept in memory (for the planted-secret scans and failure messages).
/// Spawned so that [`Cognition::stop`] reaches it on every platform (`mineworld_test_support::process`).
pub struct Cognition {
    child: InterruptibleChild,
    output: Arc<Mutex<Vec<String>>>,
}

impl Cognition {
    /// Starts `program --config config` in `directory`, with the test invite and `env`.
    pub fn start(directory: &Path, program: &Program, config: &Path, env: &[(&str, &str)]) -> Self {
        let mut command = Command::new(interpreter());
        match program {
            Program::Script(script) => command.arg(script),
            Program::Module => command.args(["-m", "mineworld_cognition"]),
        };
        command
            .arg("--config")
            .arg(config)
            .current_dir(directory)
            .env("MINEWORLD_INVITE", INVITE)
            .envs(env.iter().copied())
            .env("PYTHONUNBUFFERED", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = process::interruptible(command)
            .spawn()
            .expect("the cognition process starts");
        let output = Arc::new(Mutex::new(Vec::new()));
        drain(child.stdout.take().expect("stdout"), &output);
        drain(child.stderr.take().expect("stderr"), &output);
        Self { child, output }
    }

    /// Every line the process printed so far, both streams.
    pub fn output(&self) -> Vec<String> {
        self.output.lock().expect("sound").clone()
    }

    /// Whether the process has ended, and how.
    pub fn ended(&mut self) -> Option<ExitStatus> {
        self.child.try_wait().expect("waitable")
    }

    /// `SIGKILL` (Windows: `TerminateProcess`): no leave, no closed cassette.
    pub fn kill(&mut self) {
        let killed: Killed = process::kill(&mut self.child);
        assert!(
            killed.killed(),
            "the cognition process was killed, not finished: {:?}",
            self.output()
        );
    }

    /// The operator's stop: `SIGINT` (Windows: Ctrl-Break), which the process's `Stopper` turns into a
    /// cancellation, a `leave` from each seat within 5 s and a closed router (`shutdown.py`); it exits within
    /// its 15 s force bound. Returns how it ended.
    pub fn stop(&mut self) -> ExitStatus {
        let status = process::interrupt(&mut self.child).expect("the stop is delivered");
        assert!(
            status.success(),
            "the cognition process stopped by its graceful path ({status}): {:?}",
            self.output()
        );
        status
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
            eprintln!("[cognition] {line}");
            sink.lock().expect("sound").push(line);
        }
    });
}

/// `mineworld decisions <pack> --decisions <file> --json`, the assignment the cognition process reads (P6
/// §5.9 step 3), written to `<directory>/seats.json`. `decisions` is relative to the repository.
pub fn write_seats(directory: &Path, pack: &str, decisions: &str) {
    let decisions = repository().join(decisions);
    let (status, json) = run_command(&[
        "decisions",
        pack,
        "--decisions",
        decisions.to_str().expect("UTF-8"),
        "--json",
    ]);
    assert!(status.success(), "mineworld decisions --json ran");
    std::fs::write(directory.join("seats.json"), json).expect("seats.json is written");
}

/// Waits until `seat` is held by the cognition process: its resume file is written after the welcome (P6
/// §5.10).
pub async fn seat_taken(store: &Path, seat: &str, cognition: &mut Cognition) {
    let resume = store.join(format!("{seat}.resume"));
    let deadline = Instant::now() + SEAT_PATIENCE;
    while !resume.is_file() {
        if let Some(status) = cognition.ended() {
            panic!(
                "the cognition process ended ({status}): {:?}",
                cognition.output()
            );
        }
        assert!(
            Instant::now() < deadline,
            "seat {seat} was not taken within {SEAT_PATIENCE:?}: {:?}",
            cognition.output()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// The decision lines of a seat's log (`controller/log.py`); none while the log does not exist.
pub fn decisions(store: &Path, seat: &str) -> Vec<Value> {
    let text =
        std::fs::read_to_string(store.join(format!("{seat}.decisions.jsonl"))).unwrap_or_default();
    text.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|line| line["kind"] == "decision")
        .collect()
}

/// Waits until a seat's decision log holds `acted` consults that acted: a reply reaches a client as soon as
/// the world accepts it, and the consult is logged just after.
pub async fn logged(store: &Path, seat: &str, acted: usize) {
    let deadline = Instant::now() + SEAT_PATIENCE;
    let count = || {
        decisions(store, seat)
            .iter()
            .filter(|line| line["record"]["outcome"] == "acted")
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

/// The facts a Person perceived, as `mineworld perceived --json` exports them from the save (the offline half
/// of the stream the cognition process reads; `ARC-43`).
pub fn perceived(pack: &str, save: &str, person: &str) -> Vec<Value> {
    let (status, out) = run_command(&[
        "perceived",
        pack,
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

/// An id as the wire writes it: a decimal string.
pub fn id_of(value: &Value) -> u64 {
    value
        .as_str()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| panic!("an id: {value}"))
}

/// One `spoke` fact: who said what to whom, and the request that caused it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spoken {
    pub id: u64,
    pub speaker: String,
    pub listener: String,
    pub words: String,
    pub action: Option<u64>,
}

/// The `spoke` facts in `facts`, in order.
pub fn spoken(facts: &[Value]) -> Vec<Spoken> {
    facts
        .iter()
        .filter(|fact| fact["event_type"] == "spoke")
        .map(|fact| {
            let body = &fact["payload"]["payload"];
            Spoken {
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
                action: fact["caused_by"].get("action").map(id_of),
            }
        })
        .collect()
}

/// The actor of the request the save journaled for `action` (`input.dispatch.intent.actor`).
pub fn actor_of(save: &str, action: u64) -> String {
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

/// A seat's memory: the home's creation marker, its cursor, and its held records as (rowid, event id), in
/// the order it took them. Since S28-a the file is a memory home of schema 4 (`memory/home/`, `ARC-109`): a
/// seat's rows are its `meta` and `access` rows, each `access` row naming a shared `fact` record. The marker
/// (`home_meta.created`, `ARC-152`) is written once when the home is created and never again, so it is what
/// tells a resumed home from one a replay rebuilt with the same rows. Opened for writing, never written: a
/// home a SIGKILL left with a hot journal can only be read once SQLite has rolled that journal back, which a
/// read-only connection may not do. Since S28-f the home is the process's, one per world instance, at
/// `<store>/<instance>/memory.sqlite` (`[memory] sharing = "home"`, `pr-s28f-home-wiring.md` §5.4).
pub fn store_rows(store: &Path, seat: &str) -> (String, Option<u64>, Vec<(i64, u64)>) {
    let connection = rusqlite::Connection::open(home_file(store)).expect("the memory home opens");
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
             WHERE s.key = ?1 AND m.key = 'cursor'",
            [seat],
            |row| row.get(0),
        )
        .expect("the seat has a cursor row");
    let mut statement = connection
        .prepare(
            "SELECT a.rowid, f.event_id FROM access a JOIN fact f ON f.event_key = a.event_key \
             JOIN seats s ON s.seat = a.seat WHERE s.key = ?1 ORDER BY a.rowid",
        )
        .expect("the access and fact tables");
    let rows = statement
        .query_map([seat], |row| {
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

/// The one memory home under `store`.
pub fn home_file(store: &Path) -> PathBuf {
    let homes: Vec<_> = std::fs::read_dir(store)
        .expect("the store directory")
        .map(|entry| entry.expect("a store entry").path().join("memory.sqlite"))
        .filter(|path| path.is_file())
        .collect();
    let [home] = homes.as_slice() else {
        panic!("one memory home under the store, found {homes:?}");
    };
    home.clone()
}

/// Every Event ID a consult's memory section cites (`MemorySection.citations()`).
pub fn cited(consult: &Value) -> BTreeSet<u64> {
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

/// The text of every line a consult's memory section showed, as logged.
pub fn shown_lines(consult: &Value) -> Vec<String> {
    consult["input"]["memory"]["lines"]
        .as_array()
        .expect("lines")
        .iter()
        .map(|line| line["text"].as_str().expect("a line's text").to_owned())
        .collect()
}

/// The `recalls` of a consult's proposal: the citation tokens the model relied on.
pub fn recalls(consult: &Value) -> Vec<String> {
    consult["record"]["proposal"]["recalls"]
        .as_array()
        .map(|recalls| {
            recalls
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The ids a recall names (a shown token such as `[#12]` or `[#10-14]`, or a bare id), `None` when it is
/// not a token at all.
pub fn recall_ids(recall: &str) -> Option<BTreeSet<u64>> {
    let inner = recall.trim_start_matches('[').trim_end_matches(']');
    let mut ids = BTreeSet::new();
    for range in inner.split(',') {
        let range = range.trim().trim_start_matches('#');
        let (first, last) = range.split_once('-').unwrap_or((range, range));
        let (first, last) = (first.parse::<u64>().ok()?, last.parse::<u64>().ok()?);
        ids.extend(first..=last);
    }
    Some(ids)
}

/// Whether a recall (a shown token such as `[#12]` or `[#10-14]`, or a bare id) names `id`.
pub fn recall_names(recall: &str, id: u64) -> bool {
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
