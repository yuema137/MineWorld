//! S31-d C7 — **the price clerk on a scripted LM and on `price-reporter` post the same kind of notice**
//! (`pr-s31d-reporter-policies.md` §8 AC-d-11; step-31 AC-S31-11; `docs/decisions/ARC-116.md`).
//!
//! ```text
//! arm D   mineworld run <W6> --headless --seed 7 --days 2 --save D --decisions <clerk on price-reporter>
//! arm L   mineworld server <W6> --save L --solo --time-scale 1440 --decisions <clerk on lm, fallback
//!         price-reporter>; tests/price_reporter_cognition.py holds the clerk's seat (the shipped controller,
//!         a scripted model); a protocol client on the visitor's seat buys bread three times beside the clerk;
//!         the clerk's first LM notice is waited for (≤ 150 s wall), then both processes are killed
//! ```
//!
//! W6 is a scratch copy of `worlds/market-town` with §8 W2's edits (notices and price-survey enabled, the
//! topic declared with the clerk as its publisher, the clerk a surveyor of food and drink, Alice and Felix
//! subscribed, both shops naming their keeper, the ten food and drink kinds given their shop price as
//! reference value, the café also asking juice) and the clerk and the visitor placed in the store at genesis.
//! W6b is the same on the second world's edits (W3): the euro, the topic `market.daily` with Hana as clerk.
//!
//! Asserted from the saves, never from a value a pack reports about itself (`ARC-35` item 6, by slug):
//!
//! ```text
//! same kind     L's notice and D's notices: event `notice-published`, the same topic, the same publisher
//! same audience each notice is seen by exactly the topic's subscribers at that instant (folded from the
//!               save's `subscriptions-seeded`, `subscribed`, `unsubscribed` facts) and its publisher
//! LM's own      L's notice is the one the cognition process submitted (its decision log's acted request,
//!               `valid_for` from cognition.toml), its body is `prices`, and every line names a kind in the
//!               clerk's book at that instant (folded from the clerk's `price-noted` facts)
//! config only   the world directory is byte for byte the same before arm D and after arm L
//! ```
//!
//! No model is reachable: the backend's address is a listener this test opens and never answers. The test
//! needs the Python workspace, so it is ignored here and run by the python layer at the milestone (§7).

mod headless;
mod market;
mod price_world;
mod support;

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use market::Town;
use mineworld_contracts::{EntityId, EventEnvelope};
use price_world::{
    CAROL, Clerk, HANA, PLACED, Posted, assert_audiences, entity, facts, files, notices, payload,
    world,
};
use serde_json::{Value, json};
use support::{Client, INVITE, SaveDir, Server, run_command};

/// How long the clerk's first LM notice may take, in wall seconds (AC-d-11).
const PATIENCE: Duration = Duration::from_secs(150);
/// A world day per wall minute (`--solo` only, `tools/cli/src/main.rs`).
const TIME_SCALE: &str = "1440";
/// The hosted seats' consult cadence: slow enough that the clerk's fallback is not consulted before the
/// cognition process holds the seat.
const CADENCE: &str = "30";
/// The LM clerk's `valid_minutes` (cognition.toml): unlike the deterministic clerk's 360, so its notice is
/// told apart in the save.
const LM_VALID_MINUTES: u64 = 300;
/// The visitor's purchases, and of what.
const BUYS: usize = 3;
const BOUGHT: &str = "bread";

/// W6: Carol on `prices.ingredients`, in US dollars.
const W6: Clerk = CAROL;
/// W6b: Hana on `market.daily`, in euros.
const W6B: Clerk = HANA;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The clerk's `price-reporter` block, the same in both arms' files.
fn reporter_block(clerk: Clerk) -> String {
    format!(
        "price-reporter: {{ parameters: {{ topic: {}, first_minute: 480, last_minute: 1200, every_minutes: 240, \
         valid_minutes: 360, min_basis: 1 }} }}",
        clerk.topic
    )
}

/// The two decision files: the clerk on `price-reporter` (arm D), or on `lm` with it as fallback (arm L).
fn decisions(directory: &Path, clerk: Clerk) -> (PathBuf, PathBuf) {
    let deterministic = directory.join("deterministic.decisions.yaml");
    let lm = directory.join("lm.decisions.yaml");
    std::fs::write(
        &deterministic,
        format!(
            "models:\n  {}\nassign:\n  - {{ seat: {}, model: price-reporter }}\n",
            reporter_block(clerk),
            clerk.key
        ),
    )
    .expect("writes");
    std::fs::write(
        &lm,
        format!(
            "models:\n  lm: {{ fallback: price-reporter }}\n  {}\nassign:\n  - {{ seat: {}, model: lm }}\n",
            reporter_block(clerk),
            clerk.key
        ),
    )
    .expect("writes");
    (deterministic, lm)
}

/// The workspace's own interpreter, started directly so that killing the child kills the seat's process.
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
        "{} is missing: run `uv sync --locked` at the repository root first (the python layer does)",
        python.display()
    );
    python
}

/// The cognition process holding the clerk's seat, its output kept for a failure message.
struct Cognition {
    child: Child,
    output: Arc<Mutex<Vec<String>>>,
}

impl Cognition {
    fn start(directory: &Path, server: &Server, clerk: Clerk, sink_port: u16) -> Self {
        let config = directory.join("cognition.toml");
        std::fs::write(
            &config,
            format!(
                "server = \"ws://{}/ws\"\ninvite_env = \"MINEWORLD_INVITE\"\nnickname = \"cognition\"\n\
                 store = \"store\"\ndecisions_json = \"seats.json\"\nseats = [\"{}\"]\n\n\
                 [recording]\nmode = \"scripted\"\n\n[tiers]\nsocial = \"local\"\nsummarize = \"none\"\n\n\
                 [backends.local]\nbase_url = \"http://127.0.0.1:{sink_port}/v1\"\nmodel = \"scripted\"\n\
                 structured_output = \"none\"\n\n[controller]\nconsult_deadline_s = 20\n\n\
                 [fill.publish-notice]\ntopic = \"{}\"\nvalid_minutes = {LM_VALID_MINUTES}\n",
                server.address, clerk.key, clerk.topic
            ),
        )
        .expect("the configuration is written");
        let mut child = Command::new(interpreter())
            .arg(repository().join("cognition/lm-controller/tests/price_reporter_cognition.py"))
            .arg("--config")
            .arg(&config)
            .current_dir(directory)
            .env("MINEWORLD_INVITE", INVITE)
            .env("PYTHONUNBUFFERED", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the cognition process starts");
        let output = Arc::new(Mutex::new(Vec::new()));
        drain(child.stdout.take().expect("stdout"), &output);
        drain(child.stderr.take().expect("stderr"), &output);
        Self { child, output }
    }

    fn output(&self) -> Vec<String> {
        self.output.lock().expect("sound").clone()
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
            eprintln!("[price-reporter cognition] {line}");
            sink.lock().expect("sound").push(line);
        }
    });
}

/// The kinds in the clerk's book at `at`: every item of its `price-noted` facts up to then.
fn book_kinds(facts: &[EventEnvelope], clerk: EntityId, at: i64) -> BTreeSet<EntityId> {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "price-noted" && fact.at().seconds() <= at)
        .map(payload)
        .filter(|body| entity(&body["surveyor"]) == clerk)
        .flat_map(|body| {
            body["notes"]
                .as_array()
                .expect("notes")
                .iter()
                .map(|note| entity(&note["item"]))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The acted `publish-notice` requests of the clerk's decision log.
fn acted_notices(store: &Path, clerk: Clerk) -> Vec<Value> {
    std::fs::read_to_string(store.join(format!("{}.decisions.jsonl", clerk.key)))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|line| {
            line["kind"] == "decision"
                && line["record"]["outcome"] == "acted"
                && line["record"]["request"]["action_type"] == "publish-notice"
        })
        .map(|line| line["record"]["request"].clone())
        .collect()
}

/// A protocol client on the visitor's seat buys `BOUGHT` `BUYS` times at the store's counter.
async fn buy_bread(server: &Server, town: &Town) {
    let mut client = Client::connect(server.address).await;
    let (visitor, _) = client.join("visitor").await;
    let bread = town.id(BOUGHT);
    for _ in 0..BUYS {
        let seen = client
            .observation_where("bread for sale here", |observation| {
                observation.affordances().iter().any(|offer| {
                    offer.action_type().as_str() == "buy"
                        && offer.is_available()
                        && offer.payload().is_some_and(|p| entity(&p["item"]) == bread)
                })
            })
            .await;
        let offer = seen
            .affordances()
            .iter()
            .find(|offer| {
                offer.action_type().as_str() == "buy"
                    && offer.is_available()
                    && offer.payload().is_some_and(|p| entity(&p["item"]) == bread)
            })
            .expect("the offer just seen");
        let request = json!({
            "actor": visitor,
            "action_type": "buy",
            "target": offer.target(),
            "payload": { "action_type": "buy", "payload": offer.payload().expect("complete") },
            "actor_location": null,
        });
        client.submit_accepted(request).await;
    }
}

/// AC-d-11 on one world: arm D, arm L, and the comparisons.
#[allow(clippy::too_many_lines)]
async fn same_kind_of_notice(clerk: Clerk) {
    let directory = SaveDir::new(&format!("price-reporter-lm-{}", clerk.key));
    let root = PathBuf::from(directory.path());
    std::fs::create_dir_all(&root).expect("the scratch directory");
    let world = world(&root, clerk, PLACED);
    let pack = world.to_str().expect("UTF-8").to_owned();
    let before = files(&world);
    let town = Town::read(&world);
    let clerk_id = town.id(clerk.key);
    let (deterministic, lm) = decisions(&root, clerk);

    // ── Arm D: the deterministic clerk, two days headless. ──────────────────────────────────
    let save_d = root.join("D");
    let (status, _) = run_command(&[
        "run",
        &pack,
        "--headless",
        "--seed",
        "7",
        "--days",
        "2",
        "--save",
        save_d.to_str().expect("UTF-8"),
        "--decisions",
        deterministic.to_str().expect("UTF-8"),
    ]);
    assert!(status.success(), "arm D ran");
    let facts_d = facts(&save_d);
    let posted_d: Vec<Posted> = notices(&facts_d)
        .into_iter()
        .filter(|notice| notice.publisher == clerk_id)
        .collect();
    assert!(!posted_d.is_empty(), "the deterministic clerk posted");

    // ── Arm L: the LM clerk, hosted at a world day per wall minute. ─────────────────────────
    let (status, seats) = run_command(&[
        "decisions",
        &pack,
        "--decisions",
        lm.to_str().expect("UTF-8"),
        "--json",
    ]);
    assert!(status.success(), "mineworld decisions --json ran");
    std::fs::write(root.join("seats.json"), seats).expect("seats.json is written");
    let sink = TcpListener::bind("127.0.0.1:0").expect("a sink for the backend's address");
    sink.set_nonblocking(true).expect("non-blocking");
    let sink_port = sink.local_addr().expect("bound").port();
    let save_l = root.join("L");
    let mut server = Server::start(&[
        "server",
        &pack,
        "--save",
        save_l.to_str().expect("UTF-8"),
        "--solo",
        "--time-scale",
        TIME_SCALE,
        "--cadence",
        CADENCE,
        "--decisions",
        lm.to_str().expect("UTF-8"),
    ])
    .await;
    let mut cognition = Cognition::start(&root, &server, clerk, sink_port);
    let resume = root.join("store").join(format!("{}.resume", clerk.key));
    let deadline = Instant::now() + PATIENCE;
    while !resume.is_file() {
        assert!(
            Instant::now() < deadline,
            "the clerk's seat was not taken: {:?}",
            cognition.output()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    buy_bread(&server, &town).await;
    let store = root.join("store");
    while acted_notices(&store, clerk).is_empty() {
        if let Some(status) = cognition.child.try_wait().expect("waitable") {
            panic!(
                "the cognition process ended ({status}): {:?}",
                cognition.output()
            );
        }
        assert!(
            Instant::now() < deadline,
            "no LM notice within {PATIENCE:?}: {:?}",
            cognition.output()
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let submitted = acted_notices(&store, clerk);
    let _ = cognition.child.kill();
    let _ = cognition.child.wait();
    let _ = server.kill();
    assert!(
        sink.accept().is_err(),
        "no model was reachable: nothing connected to the sink"
    );

    let facts_l = facts(&save_l);
    let posted_l: Vec<Posted> = notices(&facts_l)
        .into_iter()
        .filter(|notice| {
            notice.publisher == clerk_id
                && notice.valid_for == i64::try_from(LM_VALID_MINUTES * 60).expect("small")
        })
        .collect();
    let first = posted_l
        .first()
        .expect("the LM clerk's notice is in the save");
    let request = &submitted[0];
    assert_eq!(
        request["actor"],
        json!(clerk_id),
        "the clerk's seat submitted it"
    );
    assert_eq!(
        request["payload"]["payload"]["body"], first.body,
        "the notice is the request the cognition process submitted"
    );

    // ── The comparisons. ────────────────────────────────────────────────────────────────────
    for notice in &posted_d {
        assert_eq!(
            (notice.topic.as_str(), notice.publisher),
            (first.topic.as_str(), first.publisher),
            "the same topic and publisher"
        );
    }
    assert_eq!(first.topic, clerk.topic);
    assert_audiences(&facts_d, &posted_d);
    assert_audiences(&facts_l, &posted_l);
    let lines = first.body["prices"].as_array().expect("a prices body");
    assert!(!lines.is_empty(), "the LM notice has lines");
    let kinds = book_kinds(&facts_l, clerk_id, first.at);
    for line in lines {
        assert!(
            kinds.contains(&entity(&line["item"])),
            "{line} names a kind in the clerk's book"
        );
    }
    assert_eq!(
        files(&world),
        before,
        "only the decisions file differs between the arms"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the Python workspace (uv sync --locked): run by the python layer"]
async fn the_clerk_posts_the_same_kind_of_notice_on_either_backend() {
    same_kind_of_notice(W6).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the Python workspace (uv sync --locked): run by the python layer"]
async fn the_same_swap_in_a_second_world() {
    same_kind_of_notice(W6B).await;
}
