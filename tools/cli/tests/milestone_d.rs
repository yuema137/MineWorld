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

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use mineworld_contracts::EntityId;
use mineworld_server::{ServerFrame, WorldSummary};
use serde_json::{Value, json};
use support::cognition::{
    Cognition, Program, actor_of, cited, decisions, id_of, perceived, recall_names, repository,
    seat_taken, spoken, store_rows, write_seats,
};
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

/// Alice's cognition process for one phase of the run (shared start-up: `support::cognition`).
fn start_cognition(directory: &Path, server: &Server, model: &Model, phase: u32) -> Cognition {
    let config = write_config(
        directory,
        &format!("ws://{}/ws", server.address),
        model,
        phase,
    );
    let program = match model {
        Model::Scripted => Program::Script(
            repository().join("cognition/lm-controller/tests/milestone_d_cognition.py"),
        ),
        Model::Recording { .. } => Program::Module,
    };
    Cognition::start(
        directory,
        &program,
        &config,
        &[(PLANTED_KEY_ENV, PLANTED_KEY)],
    )
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
    write_seats(&root, support::PACK, DECISIONS);
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
    let mut cognition = start_cognition(&root, &server, model, 1);
    seat_taken(&store, "alice", &mut cognition).await;

    // ── Step 2: the 2D window speaks to her. ────────────────────────────────────────────────
    let (mut two_d, visitor, world_2d) = window(&server, "2d").await;
    let seen = two_d.observation().await;
    let alice = tagged(&seen, "barista").expect("the 2D window perceives the barista");
    walk_to(&mut two_d, visitor, AT_THE_COUNTER).await;
    let line_2d = say(&mut two_d, visitor, alice, FROM_THE_2D_WINDOW).await;

    // ── Step 3: Alice's seat decides and replies. ───────────────────────────────────────────
    let reply_2d = reply(&mut two_d, alice, 0, &store).await;
    support::cognition::logged(&store, "alice", 1).await;
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
    let (created_before, cursor_before, rows_before) = store_rows(&store, "alice");
    let cursor_before =
        cursor_before.expect("Alice's store had ingested something before the kill");
    if let Model::Recording { .. } = model {
        keep_partial(&root, 1);
    }
    let mut server =
        Server::start(&["server", support::PACK, "--save", &save, "--hold", HOLD]).await;
    let mut cognition = start_cognition(&root, &server, model, 2);
    seat_taken(&store, "alice", &mut cognition).await;

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
    support::cognition::logged(&store, "alice", 2).await;
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
    let facts = spoken(&perceived(support::PACK, &save, "visitor"));
    let alice_text = alice.to_string();
    let visitor_text = visitor.to_string();
    let find_line = |ids: &[u64]| {
        facts
            .iter()
            .find(|fact| ids.contains(&fact.id))
            .unwrap_or_else(|| panic!("a spoke among the facts {ids:?}"))
            .clone()
    };
    let visitor_2d = find_line(&line_2d);
    let visitor_3d = find_line(&line_3d);
    assert_eq!(visitor_2d.words, FROM_THE_2D_WINDOW);
    assert_eq!(visitor_3d.words, FROM_THE_3D_WINDOW);
    let from_alice: Vec<_> = facts
        .iter()
        .filter(|fact| fact.speaker == alice_text && fact.listener == visitor_text)
        .collect();
    assert_eq!(
        from_alice.len(),
        2,
        "Alice said exactly two lines to the visitor, one per window: {from_alice:?}"
    );
    let (alice_2d, alice_3d) = (from_alice[0], from_alice[1]);
    assert!(
        alice_2d.id > visitor_2d.id && alice_2d.id < visitor_3d.id,
        "her first reply answers the 2D line"
    );
    assert!(
        alice_3d.id > visitor_3d.id,
        "her second reply answers the 3D line"
    );
    assert_eq!(
        (alice_2d.words.as_str(), alice_3d.words.as_str()),
        (reply_2d.as_str(), reply_3d.as_str())
    );

    let log = decisions(&store, "alice");
    for (fact, words) in [(alice_2d, &reply_2d), (alice_3d, &reply_3d)] {
        let action = fact.action.expect("Alice's line was caused by a request");
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
    let two_d_ids = (visitor_2d.id, alice_2d.id);
    let consult = log
        .iter()
        .find(|consult| id_of(&consult["input"]["trigger"]["event"]) == visitor_3d.id)
        .unwrap_or_else(|| panic!("a consult on the 3D line {}: {log:?}", visitor_3d.id));
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
    let (created_after, cursor_after, rows_after) = store_rows(&store, "alice");
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
        support::file_containing(&root, PLANTED_KEY.as_bytes()),
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

    let facts = spoken(&perceived(support::PACK, &save, "visitor"));
    let words: Vec<&str> = facts.iter().map(|fact| fact.words.as_str()).collect();
    assert_eq!(
        words,
        [FROM_THE_2D_WINDOW, FROM_THE_3D_WINDOW],
        "the visitor's two lines are recorded, and they are the only lines spoken"
    );
    assert!(facts.iter().any(|fact| line_2d.contains(&fact.id)));
    assert!(facts.iter().any(|fact| line_3d.contains(&fact.id)));
    let (status, _) = run_command(&["inspect", &save, "--last", "0"]);
    assert!(status.success(), "mineworld inspect resolves every cause");
}
