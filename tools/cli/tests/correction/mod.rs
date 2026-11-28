//! The corrected-memory scenario's harness (`correction_memory.rs`; `pr-playable-dialogue-memory.md` §5 C4, §6):
//! its nonces, its model modes and their configuration, the turn ledger, and the §6 warm-up. The people it plays
//! are in [`people`]; what it asserts and writes is in [`evidence`].

pub mod evidence;
pub mod people;

use std::hash::{BuildHasher, Hasher};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use mineworld_contracts::EntityId;
use serde_json::{Value, json};

use crate::support::cognition::{Program, decisions, repository};
use crate::support::{MARKET_PACK, SaveDir, Server, admin};
use people::Player;

/// The decision files: Alice alone on `lm` (the §6 profile), and Alice with Dev for the isolation run.
pub const ALICE_ONLY: &str = "cognition/lm-controller/examples/alice-lm.decisions.yaml";
pub const ALICE_AND_DEV: &str = "tools/cli/tests/fixtures/decisions/market-alice-dev-lm.yaml";
/// The admin surface's token, for the clock only; never given to the cognition process.
pub const ADMIN_TOKEN: &str = "correction-memory-admin-7d21c0";
/// World seconds per wall second while positioning, and while playing (Market Town's own `hosting:`).
pub const FAST: &str = "3600";
pub const PLAYED: u32 = 12;
/// The pinned window (§6): every turn between 07:00 and 13:00 of day 1, world time.
pub const OPENS: i64 = 7 * 3_600;
pub const CLOSES: i64 = 13 * 3_600;
/// A place at the café counter inside `talk`'s three-metre reach of Alice, who stands at (6000, 8000) mm
/// (`worlds/market-town/people/alice.yaml`), and how far from that spot she may be to count as "at the counter".
pub const AT_THE_COUNTER: (i32, i32) = (6_000, 6_200);
pub const ALICE_SPOT: (i32, i32) = (6_000, 8_000);
pub const COUNTER_MM: i64 = 1_000;
/// How long a consult may take to be logged: the 20 s deadline plus the process's own work.
pub const CONSULT_PATIENCE: Duration = Duration::from_secs(60);
/// How long a person stands before speaking (D-P6b-30), and how long a held seat is kept (`--hold`).
pub const SETTLE: Duration = Duration::from_secs(2);
pub const HOLD: &str = "3";
/// How long a seat a person holds is watched for a theft that must not happen (`seat_retry_s` is 1).
pub const NOT_STOLEN: Duration = Duration::from_secs(4);
/// The graceful stop's bound for the server after stdin EOF.
pub const SERVER_STOP: Duration = Duration::from_secs(30);
/// The §6 caps: model calls including the warm-up, and the ledger's own per-hour ceiling.
pub const CALL_CAP: u64 = 8;
pub const LEDGER_CALLS_PER_HOUR: u64 = 7;

pub const QUESTION: &str = "Do you remember what I told you about my bag and my locker?";
pub const UNRELATED: &str = "By the way, the weather is lovely this morning.";
pub const LAST: &str = "Where is my bag now, and what is my locker code?";
pub const PLACES: [&str; 8] = [
    "bakery", "library", "pharmacy", "museum", "bookshop", "harbour", "garage", "theatre",
];

/// The run's nonces, drawn at run time so no recording or literal can answer for the memory.
pub struct Lines {
    pub code: String,
    pub old: &'static str,
    pub new: &'static str,
    pub private: String,
}

impl Lines {
    pub fn draw() -> Self {
        let random = || {
            std::collections::hash_map::RandomState::new()
                .build_hasher()
                .finish()
        };
        let (a, b) = (random(), random());
        let old = PLACES[usize::try_from(a % 8).expect("small")];
        let new = PLACES[usize::try_from((a % 8 + 1 + b % 7) % 8).expect("small")];
        let letters = |n: u64| -> String {
            (0..3)
                .map(|k| char::from(b'A' + u8::try_from((n >> (5 * k)) % 26).expect("small")))
                .collect()
        };
        Self {
            code: format!("{}-{:04}", letters(b), (b >> 20) % 10_000),
            old,
            new,
            private: format!("{}-{:04}", letters(a >> 7), (a >> 24) % 10_000),
        }
    }

    pub fn first(&self) -> String {
        format!(
            "Please remember this: my locker code is {} and my bag is at the {}.",
            self.code, self.old
        )
    }

    pub fn correction(&self) -> String {
        format!(
            "Correction: my bag is now at the {}, not the {}. Same locker code.",
            self.new, self.old
        )
    }

    pub fn secret(&self) -> String {
        format!("Keep this between us: my locker code is {}.", self.private)
    }
}

/// Which model answers, and with which configuration.
pub enum Mode {
    /// The scripted `corrects`, behind a backend address this test listens on and that must accept nothing.
    /// `gate` runs the §6 configuration (one LM seat, the gate's budgets); otherwise Dev is a second LM seat.
    Scripted { sink: TcpListener, gate: bool },
    /// The local model in `record` mode, with the §6 configuration (operator-run).
    Live { base_url: String, model: String },
}

impl Mode {
    pub fn scripted(gate: bool) -> Self {
        let sink = TcpListener::bind("127.0.0.1:0").expect("a sink for the backend's address");
        sink.set_nonblocking(true).expect("non-blocking");
        Self::Scripted { sink, gate }
    }

    pub const fn gate(&self) -> bool {
        matches!(self, Self::Live { .. } | Self::Scripted { gate: true, .. })
    }

    pub const fn second_seat(&self) -> bool {
        !self.gate()
    }

    pub fn seats(&self) -> &'static [&'static str] {
        if self.second_seat() {
            &["alice", "dev"]
        } else {
            &["alice"]
        }
    }

    pub fn program(&self) -> Program {
        match self {
            Self::Scripted { .. } => Program::Script(
                repository().join("cognition/lm-controller/tests/correction_cognition.py"),
            ),
            Self::Live { .. } => Program::Module,
        }
    }

    /// `cognition-<phase>.toml`, its store `store` (or `store-null` for the null-store rebuild).
    pub fn write_config(&self, root: &Path, server: &Server, phase: &str, store: &str) -> PathBuf {
        let (recording, backend) = match self {
            Self::Scripted { sink, .. } => (
                "mode = \"scripted\"".to_owned(),
                format!(
                    "base_url = \"http://{}/v1\"\nmodel = \"correction-model\"",
                    sink.local_addr().expect("bound")
                ),
            ),
            Self::Live { base_url, model } => (
                format!("mode = \"record\"\ncassette = \"cassette-{phase}.jsonl\""),
                format!("base_url = \"{base_url}\"\nmodel = \"{model}\""),
            ),
        };
        let seats: Vec<String> = self
            .seats()
            .iter()
            .map(|seat| format!("\"{seat}\""))
            .collect();
        let (controller, budgets) = if self.gate() {
            (
                "max_calls = 1\ngreet_cooldown_s = 86400\nfeeling = false\n",
                format!(
                    "\n[budgets]\ncalls_per_wall_hour = {LEDGER_CALLS_PER_HOUR}\ntokens_per_wall_day = 30000\n\
                     max_in_flight = 1\nledger = \"gate-ledger.sqlite\"\n"
                ),
            )
        } else {
            ("", String::new())
        };
        let text = format!(
            "server = \"ws://{}/ws\"\ninvite_env = \"MINEWORLD_INVITE\"\nnickname = \"cognition\"\n\
             store = \"{store}\"\ndecisions_json = \"seats.json\"\nseats = [{}]\n\n\
             [recording]\n{recording}\n\n\
             [tiers]\nsocial = \"local\"\nordinary_decision = \"none\"\nmajor_decision = \"none\"\nsummarize = \"none\"\n\n\
             [backends.local]\n{backend}\nstructured_output = \"none\"\nreasoning = \"off\"\n\n\
             [controller]\nconsult_deadline_s = 20\nseat_retry_s = 1\n{controller}{budgets}",
            server.address,
            seats.join(", "),
        );
        let path = root.join(format!("cognition-{phase}.toml"));
        std::fs::write(&path, text).expect("the configuration is written");
        path
    }
}

/// The world's clock (`/status`'s `at`), in world seconds.
pub async fn now(server: &Server) -> i64 {
    server.status().await["at"].as_i64().expect("an instant")
}

/// Runs the fast-forward until `at`, then plays at Market Town's own scale.
pub async fn fast_forward_to(server: &Server, at: i64) -> i64 {
    let deadline = Instant::now() + Duration::from_secs(120);
    while now(server).await < at {
        assert!(Instant::now() < deadline, "the world did not reach {at}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let body = json!({ "time_scale": PLAYED }).to_string();
    let answer = admin(
        server.address,
        "POST",
        "/admin/clock",
        Some(ADMIN_TOKEN),
        &body,
    )
    .await;
    assert_eq!(
        answer.status, 200,
        "the clock is rescaled: {:?}",
        answer.body
    );
    now(server).await
}

/// One player line and what came of it.
pub struct Turn {
    pub n: usize,
    pub player: &'static str,
    pub words: String,
    pub world_at: i64,
    /// Alice's consult on the line (her decision log), when one was logged.
    pub consult: Option<Value>,
    /// Wall time from the accepted line to the logged consult: the reply's inference wall and its overhead.
    pub wall: Duration,
    pub reply: Option<String>,
}

/// Waits until `seat`'s decision log holds the consult whose trigger is a line with these `words`.
pub async fn consult_on(store: &Path, seat: &str, words: &str) -> Option<Value> {
    let deadline = Instant::now() + CONSULT_PATIENCE;
    loop {
        let found = decisions(store, seat)
            .into_iter()
            .find(|consult| consult["input"]["trigger"]["words"] == words);
        if found.is_some() || Instant::now() >= deadline {
            return found;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// The words of the request an acted consult submitted.
pub fn submitted(consult: &Value) -> Option<String> {
    (consult["record"]["outcome"] == "acted").then(|| {
        consult["record"]["request"]["payload"]["payload"]["utterance"]
            .as_str()
            .expect("an acted consult's words")
            .to_owned()
    })
}

/// Everything the run keeps: the clients' transcripts, the processes' artefacts, the identities.
pub struct Run {
    pub directory: SaveDir,
    pub root: PathBuf,
    pub save: String,
    /// The decision list the server and the assignment were given (absolute).
    pub decisions: String,
    pub store: PathBuf,
    pub lines: Lines,
    pub turns: Vec<Turn>,
    pub alice: EntityId,
    pub visitor: EntityId,
    /// Dev and Erin, with the second seat.
    pub pair: Option<(EntityId, EntityId)>,
    /// Alice's `holdings` and `wallet` while a person held her, before the LM took her.
    pub holdings: (Value, Value),
    /// The model calls the warm-up counted (§6: one in the live gate, none otherwise).
    pub warmed: u64,
    /// How many of Alice's consults were logged before the restart.
    pub logged_before: usize,
    pub observations: Vec<(String, Value)>,
    pub stopped_at: Option<usize>,
}

impl Run {
    pub fn note(&mut self, what: &str, value: Value) {
        println!("[correction] {what}: {value}");
        self.observations.push((what.to_owned(), value));
    }

    /// The model calls the gate ledger holds so far, for every seat.
    pub fn ledger_calls(&self) -> u64 {
        let file = self.root.join("gate-ledger.sqlite");
        if !file.is_file() {
            return 0;
        }
        let connection = rusqlite::Connection::open(file).expect("the ledger opens");
        connection
            .query_row(
                "SELECT COALESCE(SUM(calls), 0) FROM budget_ledger",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_or(0, |calls| u64::try_from(calls).expect("a count"))
    }

    /// One player turn: unless the §6 cap would be passed, the line is said, its consult awaited, and the reply
    /// checked against what the player was told.
    pub async fn turn(&mut self, server: &Server, player: &mut Player, words: &str) {
        let n = self.turns.len() + 1;
        if self.stopped_at.is_some() || self.ledger_calls() + self.warmed + 1 > CALL_CAP {
            self.stopped_at.get_or_insert(n);
            println!("[correction] turn {n} not said: the call cap of {CALL_CAP} would be passed");
            return;
        }
        let world_at = now(server).await;
        let started = Instant::now();
        player.say(words).await;
        let consult = consult_on(&self.store, "alice", words).await;
        let wall = started.elapsed();
        let reply = consult.as_ref().and_then(submitted);
        if let Some(reply) = &reply {
            let deadline = Instant::now() + CONSULT_PATIENCE;
            while !player.heard().await.contains(reply) {
                assert!(Instant::now() < deadline, "the player was told {reply:?}");
            }
        }
        println!(
            "[correction] turn {n} ({}, {world_at} s): {words:?} -> {reply:?} in {wall:?}",
            player.name()
        );
        self.turns.push(Turn {
            n,
            player: player.name(),
            words: words.to_owned(),
            world_at,
            consult,
            wall,
            reply,
        });
    }
}

/// The server's command line: Market Town hosted with its in-server controllers, the decision list, a save.
pub fn server_args<'a>(decisions: &'a str, save: &'a str, scale: &'a str) -> Vec<&'a str> {
    vec![
        "server",
        MARKET_PACK,
        "--town",
        "--decisions",
        decisions,
        "--solo",
        "--time-scale",
        scale,
        "--save",
        save,
        "--hold",
        HOLD,
    ]
}

/// A `record` session's cassette, kept under its phase's name (a graceful stop closes it; a `.partial` is kept).
pub fn keep_cassette(root: &Path, phase: &str) {
    let partial = root.join(format!("cassette-{phase}.jsonl.partial"));
    if partial.is_file() {
        std::fs::rename(&partial, root.join(format!("cassette-{phase}.jsonl"))).expect("kept");
    }
}

/// The counted warm-up (§6): one Ollama load request with an empty prompt, which loads the model and generates
/// nothing, made before cognition starts. Returns the calls it counts for (1).
pub async fn warm_up(base_url: &str, model: &str) -> u64 {
    let authority = base_url
        .trim_start_matches("http://")
        .split('/')
        .next()
        .expect("an authority");
    let address: SocketAddr = authority
        .replace("localhost", "127.0.0.1")
        .parse()
        .expect("a loopback IP:port base URL");
    assert!(address.ip().is_loopback(), "the gate's model is local");
    let body = json!({ "model": model, "prompt": "", "stream": false, "keep_alive": "30m" });
    let started = Instant::now();
    let answer = admin(address, "POST", "/api/generate", None, &body.to_string()).await;
    println!(
        "[correction] warm-up: HTTP {} in {:?}: {}",
        answer.status,
        started.elapsed(),
        answer.body
    );
    assert_eq!(answer.status, 200, "the model loaded");
    1
}
