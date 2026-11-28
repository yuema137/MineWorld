//! **Corrected memory across a real save, and the same Alice under every controller**
//! (`pr-playable-dialogue-memory.md` §5 C4, §6; operator requirement check 2).
//!
//! ```text
//! mineworld server worlds/market-town --town --decisions … --solo --time-scale 3600 --save DIR
//!           --stop-on-stdin-eof                      plain clients hold alice and visitor (and, with the
//!                                                    second seat, dev and erin) from genesis: nobody they
//!                                                    hold moves, so Alice stays at her counter
//! POST /admin/clock {"time_scale": 12} at ≥ 07:00    her punctual 05:30 part has begun; the next is not punctual
//! python … --config cognition-1.toml                 her `lm` seat; refused while a person holds her
//! alice's client leaves                              rule → person → LM: cognition takes her
//! 1  protocol client "2d" on visitor                 a runtime nonce code + where the bag is
//!    leave · the MCP agent (`mineworld_agent`) takes visitor
//! 2  question  3  correction (new place, same code)  4  an unrelated line
//!    [second seat] erin tells dev a private nonce in the park, which Alice never perceives
//! SIGINT cognition, stdin EOF server                 both graceful; the same save, the same store
//! restart both; protocol client "3d" on visitor
//! 5  where is my bag now, and what is my code?
//! ```
//!
//! The model is either the scripted `corrects` (`cognition/lm-controller/tests/correction_cognition.py`), which
//! answers a question only from the remembered lines the request shows (the newest place line, the code line,
//! each cited), or the local model in `record` mode (the §6 live gate, operator-run). Every claim is read from
//! an artefact a reviewer can open: the save (`perceived`, the journal), the memory home, the decision logs, and
//! what each client was told. The questions never contain the answers. Entity ids come from welcomes and tags.
//!
//! # Where these tests run
//!
//! The scripted scenario needs the Python workspace, so it is ignored with a reason and run by the `python`
//! layer (`scripts/ci_layer.py`). The live gate is operator-run, once, on the final head (§6):
//!
//! ```sh
//! MINEWORLD_CORRECTION_GATE_OUT=/abs/dir cargo test -p mineworld-cli --test correction_memory -- \
//!   --ignored --exact the_bounded_live_gate --nocapture
//! ```
//!
//! `MINEWORLD_CORRECTION_GATE_BASE_URL` and `MINEWORLD_CORRECTION_GATE_MODEL` override the local binding
//! (`http://127.0.0.1:11434/v1`, `qwen3.5:4b`); `MINEWORLD_CORRECTION_GATE_DRY=1` runs the same harness, gate
//! configuration and evidence with the scripted model instead (no warm-up, no model call).

mod correction;
mod support;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ChildStdin;
use std::time::Instant;

use correction::evidence::{
    dir_size, facts, isolated, machine_checks, no_trigger_replayed, others_acted, store_resumed,
    write_evidence,
};
use correction::people::{Agent, Held, Player, feed, hold, leave, own_component, whereabouts};
use correction::{
    ADMIN_TOKEN, ALICE_AND_DEV, ALICE_ONLY, ALICE_SPOT, AT_THE_COUNTER, CALL_CAP, CLOSES,
    COUNTER_MM, FAST, LAST, Lines, Mode, NOT_STOLEN, OPENS, PLAYED, QUESTION, Run, SERVER_STOP,
    UNRELATED, consult_on, fast_forward_to, keep_cassette, server_args, warm_up,
};
use serde_json::{Value, json};
use support::cognition::{Cognition, decisions, repository, seat_taken, store_rows, write_seats};
use support::{Captured, MARKET_PACK, SaveDir, Server, may_talk_to, run_command, talk};

/// A hosted world and the standard input whose end stops it gracefully (`--stop-on-stdin-eof`).
struct Hosting {
    server: Server,
    stdin: ChildStdin,
    captured: Captured,
}

impl Hosting {
    async fn start(decisions: &str, save: &str, scale: &str) -> Self {
        let env = [("MINEWORLD_ADMIN_TOKEN", ADMIN_TOKEN)];
        let (server, stdin, captured) =
            Server::start_stoppable(&server_args(decisions, save, scale), &env).await;
        Self {
            server,
            stdin,
            captured,
        }
    }

    /// The launcher's stop: stdin EOF, then the graceful path (checkpoint, statistics, exit 0).
    async fn stop(self) {
        let Self {
            mut server,
            stdin,
            captured,
        } = self;
        let status = server.stopped_by_eof(stdin, SERVER_STOP).await;
        assert!(status.success(), "the server stopped gracefully: {status}");
        assert!(
            captured
                .stdout()
                .iter()
                .any(|line| line == "[mineworld] stopping"),
            "the server took its graceful path"
        );
    }
}

/// The people held from genesis while the clock runs to the pinned window.
struct Positioned {
    alice: Held,
    window: Held,
    pair: Option<(Held, Held)>,
}

/// Positioning (§6): plain clients hold Alice and the visitor (and Dev and Erin) from genesis, so nobody they
/// hold moves; the clock runs at 3 600× to 07:00 and then at Market Town's 12×. Alice is shown to be at her
/// counter by her own observation, before any cognition process exists: no model is asked.
async fn position(mode: &Mode) -> (Run, Hosting, Positioned) {
    let directory = SaveDir::new("correction-memory");
    let root = PathBuf::from(directory.path());
    std::fs::create_dir_all(&root).expect("the scratch directory");
    let save = root.join("save").to_str().expect("UTF-8").to_owned();
    let decisions_file = if mode.second_seat() {
        ALICE_AND_DEV
    } else {
        ALICE_ONLY
    };
    write_seats(&root, MARKET_PACK, decisions_file);
    let decisions_path = repository().join(decisions_file);
    let decisions_arg = decisions_path.to_str().expect("UTF-8").to_owned();
    let hosting = Hosting::start(&decisions_arg, &save, FAST).await;
    let mut alice = hold(&hosting.server, "alice").await;
    let window = hold(&hosting.server, "visitor").await;
    let pair = if mode.second_seat() {
        Some((
            hold(&hosting.server, "dev").await,
            hold(&hosting.server, "erin").await,
        ))
    } else {
        None
    };
    let t0 = fast_forward_to(&hosting.server, OPENS).await;
    let meals = feed(&mut alice).await;
    let needs = own_component(&mut alice, "needs").await;
    let (place, spot) = whereabouts(&mut alice).await;
    let off = i64::from(spot.0 - ALICE_SPOT.0).pow(2) + i64::from(spot.1 - ALICE_SPOT.1).pow(2);
    assert!(
        off <= COUNTER_MM * COUNTER_MM,
        "Alice is at her counter before cognition starts: {spot:?}"
    );
    let holdings = (
        own_component(&mut alice, "holdings").await,
        own_component(&mut alice, "wallet").await,
    );
    assert!(
        holdings.0.is_object() && holdings.1.is_object(),
        "Alice sees what she holds"
    );
    let mut run = Run {
        store: root.join("store"),
        root,
        save,
        decisions: decisions_arg,
        lines: Lines::draw(),
        turns: Vec::new(),
        alice: alice.id,
        visitor: window.id,
        pair: pair.as_ref().map(|(dev, erin)| (dev.id, erin.id)),
        holdings,
        warmed: 0,
        logged_before: 0,
        observations: Vec::new(),
        stopped_at: None,
        directory,
    };
    run.note(
        "positioned",
        json!({ "world_at": t0, "alice_place": place.raw(), "alice_at": [spot.0, spot.1],
                "alice_took_over": format!("{:?}", alice.took_over),
                "holdings": run.holdings.0, "wallet": run.holdings.1,
                "meals": meals, "needs": needs,
                "store_bytes": dir_size(&run.store), "save_bytes": dir_size(Path::new(&run.save)) }),
    );
    assert_eq!(
        format!("{:?}", alice.took_over),
        "Hosted",
        "rule → person: the pack's rule controller was driving Alice when the person took her"
    );
    (
        run,
        hosting,
        Positioned {
            alice,
            window,
            pair,
        },
    )
}

/// Before the restart: cognition is refused while a person holds Alice, takes her once the person leaves (rule
/// → person → LM), and hears turns 1–4 — the first from a protocol window, the rest from the MCP agent that
/// takes the visitor's seat after the window leaves. With the second seat, Erin tells Dev a private nonce.
/// Then the full stop: cognition by its operator stop, the server by stdin EOF.
async fn before_the_restart(mode: &Mode, run: &mut Run, hosting: Hosting, people: Positioned) {
    let Positioned {
        alice,
        mut window,
        pair,
    } = people;
    let config = mode.write_config(&run.root, &hosting.server, "1", "store");
    let mut cognition = Cognition::start(&run.root, &mode.program(), &config, &[]);
    tokio::time::sleep(NOT_STOLEN).await;
    assert!(
        cognition.ended().is_none() && !run.store.join("alice.resume").exists(),
        "a seat a person holds is never taken (take_over is false): {:?}",
        cognition.output()
    );
    leave(alice).await;
    seat_taken(&run.store, "alice", &mut cognition).await;
    let erin = match pair {
        Some((dev, erin)) => {
            leave(dev).await;
            seat_taken(&run.store, "dev", &mut cognition).await;
            Some(erin)
        }
        None => None,
    };
    let place = whereabouts(&mut window).await.0;
    window
        .client
        .walk_there(window.id, place, Some(AT_THE_COUNTER))
        .await;
    let mut player = Player::Window {
        held: Box::new(window),
        alice: run.alice,
    };
    let first = run.lines.first();
    run.turn(&hosting.server, &mut player, &first).await;
    if let Player::Window { held, .. } = player {
        leave(*held).await;
    }
    let agent = Agent::start(&hosting.server, &run.root);
    run.note("mcp agent joined", agent.ready.clone());
    let mut player = Player::Agent(agent);
    let correction = run.lines.correction();
    for words in [QUESTION, correction.as_str(), UNRELATED] {
        run.turn(&hosting.server, &mut player, words).await;
    }
    if let Some(erin) = erin {
        private_exchange(run, erin).await;
    }
    if let Player::Agent(agent) = player {
        agent.leave();
    }
    cognition.stop();
    hosting.stop().await;
    keep_cassette(&run.root, "1");
}

/// Erin tells Dev a private nonce in the park, where Alice is not: a fact Alice never perceives, held in the
/// same memory home as hers.
async fn private_exchange(run: &Run, mut erin: Held) {
    let (dev, _) = run.pair.expect("the second seat");
    erin.client
        .observation_where("talk to Dev available", |seen| may_talk_to(seen, dev))
        .await;
    let secret = run.lines.secret();
    erin.client
        .submit_accepted(talk(erin.id, dev, &secret))
        .await;
    assert!(
        consult_on(&run.store, "dev", &secret).await.is_some(),
        "Dev's seat consulted on Erin's line"
    );
    leave(erin).await;
}

/// After the restart on the same save and the same store: turn 5 from a protocol window; the store resumed;
/// (scripted) the null-store rebuild for contrast; then LM → rule → person: the same Alice, with what she held
/// and what she heard.
async fn after_the_restart(
    mode: &Mode,
    run: &mut Run,
    before: (String, Option<u64>, Vec<(i64, u64)>),
) {
    let hosting = Hosting::start(&run.decisions, &run.save, &PLAYED.to_string()).await;
    let mut window = hold(&hosting.server, "visitor").await;
    assert_eq!(window.id, run.visitor, "the same visitor after the restart");
    let config = mode.write_config(&run.root, &hosting.server, "2", "store");
    let mut cognition = Cognition::start(&run.root, &mode.program(), &config, &[]);
    seat_taken(&run.store, "alice", &mut cognition).await;
    let place = whereabouts(&mut window).await.0;
    window
        .client
        .walk_there(window.id, place, Some(AT_THE_COUNTER))
        .await;
    let mut player = Player::Window {
        held: Box::new(window),
        alice: run.alice,
    };
    run.turn(&hosting.server, &mut player, LAST).await;
    cognition.stop();
    let after = store_rows(&run.store, "alice");
    keep_cassette(&run.root, "2");
    run.note(
        "store across the restart",
        json!({ "created_before": before.0, "created_after": after.0,
                "cursor_before": before.1, "cursor_after": after.1,
                "rows_before": before.2.len(), "rows_after": after.2.len(),
                "consults_before_restart": run.logged_before }),
    );
    store_resumed(&before, &after);
    if let Mode::Scripted { .. } = mode {
        null_store_rebuild(mode, run, &hosting.server, &after).await;
    }
    if let Player::Window { held, .. } = player {
        leave(*held).await;
    }
    the_same_alice(run, &hosting.server).await;
    hosting.stop().await;
    run.note(
        "disk at the end",
        json!({ "store_bytes": dir_size(&run.store), "save_bytes": dir_size(Path::new(&run.save)) }),
    );
}

/// A third cognition process on the same world with an empty store: its home is a different home (its creation
/// marker differs), which is what makes the resumed store's marker a discriminating check.
async fn null_store_rebuild(
    mode: &Mode,
    run: &mut Run,
    server: &Server,
    retained: &(String, Option<u64>, Vec<(i64, u64)>),
) {
    let config = mode.write_config(&run.root, server, "null", "store-null");
    let mut rebuilt = Cognition::start(&run.root, &mode.program(), &config, &[]);
    let empty = run.root.join("store-null");
    seat_taken(&empty, "alice", &mut rebuilt).await;
    rebuilt.stop();
    let fresh = store_rows(&empty, "alice");
    let kept: BTreeSet<u64> = retained.2.iter().map(|row| row.1).collect();
    let rebuilt_ids: BTreeSet<u64> = fresh.2.iter().map(|row| row.1).collect();
    run.note(
        "null-store rebuild",
        json!({ "created": fresh.0, "rows": fresh.2.len(), "same_event_ids_as_retained": kept == rebuilt_ids,
                "consults": decisions(&empty, "alice").iter().map(|consult| json!([
                    consult["input"]["trigger"]["kind"], consult["input"]["trigger"]["event"],
                    consult["record"]["outcome"]])).collect::<Vec<_>>() }),
    );
    assert_ne!(
        fresh.0, retained.0,
        "a rebuild from nothing is a different home: the retained store's marker discriminates"
    );
}

/// LM → rule → person: a person takes Alice again and is the same Person, holding what she held, with every
/// player line in her own conversation history.
async fn the_same_alice(run: &mut Run, server: &Server) {
    let mut alice = hold(server, "alice").await;
    assert_eq!(alice.id, run.alice, "the same Alice under every controller");
    let holdings = (
        own_component(&mut alice, "holdings").await,
        own_component(&mut alice, "wallet").await,
    );
    let observation = alice.client.observation().await;
    let heard: Vec<String> = support::own_history(&observation).map_or_else(Vec::new, |history| {
        history
            .heard()
            .iter()
            .map(|line| line.utterance().as_str().to_owned())
            .collect()
    });
    run.note(
        "alice under a person again",
        json!({ "took_over": format!("{:?}", alice.took_over), "holdings": holdings.0,
                "wallet": holdings.1, "heard": heard }),
    );
    explained(run, &holdings);
    for turn in &run.turns {
        assert!(
            heard.contains(&turn.words),
            "her history holds {:?}",
            turn.words
        );
    }
    leave(alice).await;
}

/// Her holdings and money are hers across every swap: equal to what she held when a person first took her, or
/// changed only by requests she made herself that her LM seat did not submit — the rule controller drives her
/// in the moments between holders (hungry at 07:00, it may buy or eat), which is the world working, not a swap
/// losing anything. Every request her LM consults submitted is a `talk`.
fn explained(run: &mut Run, holdings: &(Value, Value)) {
    let submitted: Vec<Value> = decisions(&run.store, "alice")
        .iter()
        .filter(|consult| consult["record"]["outcome"] == "acted")
        .map(|consult| consult["record"]["request"]["action_type"].clone())
        .collect();
    assert!(
        submitted.iter().all(|kind| kind == "talk"),
        "her LM seat only spoke: {submitted:?}"
    );
    if *holdings == run.holdings {
        return;
    }
    let own = alice_requests(run);
    run.note(
        "holdings changed between holders",
        json!({ "before": [run.holdings.0, run.holdings.1], "after": [holdings.0, holdings.1],
                "her_other_requests": own }),
    );
    assert!(
        !own.is_empty(),
        "a change to what she holds is explained by a request she made under the rule controller"
    );
}

/// The action types of Alice's journaled requests other than `talk`, since the save began.
fn alice_requests(run: &Run) -> Vec<String> {
    let connection =
        rusqlite::Connection::open(Path::new(&run.save).join("world.sqlite")).expect("the save");
    let mut statement = connection
        .prepare("SELECT entry FROM journal")
        .expect("the journal");
    let alice = run.alice.to_string();
    statement
        .query_map([], |row| row.get::<_, Vec<u8>>(0))
        .expect("rows")
        .filter_map(Result::ok)
        .filter_map(|entry| serde_json::from_slice::<Value>(&entry).ok())
        .map(|entry| entry["input"]["dispatch"]["intent"].clone())
        .filter(|intent| intent["actor"] == alice.as_str() && intent["action_type"] != "talk")
        .map(|intent| {
            intent["action_type"]
                .as_str()
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

/// The whole scenario, through the real server, the real cognition process and the real clients.
async fn corrected(mode: &Mode) -> Run {
    let (mut run, hosting, people) = position(mode).await;
    if let Mode::Live { base_url, model } = mode {
        run.warmed = warm_up(base_url, model).await;
    }
    before_the_restart(mode, &mut run, hosting, people).await;
    let before = store_rows(&run.store, "alice");
    run.logged_before = decisions(&run.store, "alice").len();
    after_the_restart(mode, &mut run, before).await;
    if let Mode::Scripted { sink, .. } = mode {
        assert!(
            matches!(sink.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
            "nothing connected to the backend's address"
        );
    }
    run
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the Python workspace (uv sync --locked): run by the python layer, scripts/ci_layer.py"]
async fn she_answers_with_the_corrected_place_after_a_restart() {
    let mode = Mode::scripted(false);
    let run = corrected(&mode).await;
    let facts = facts(&run);
    let checks = machine_checks(&run, &facts);
    for (check, holds, detail) in &checks {
        assert!(holds, "{check}: {detail}");
    }
    let reply = run.turns[4].reply.clone().expect("turn 5 was answered");
    assert!(
        reply.contains(run.lines.new),
        "(b) the corrected place: {reply:?}"
    );
    assert!(
        !reply.contains(run.lines.old),
        "(d) not the superseded place: {reply:?}"
    );
    let question = run.turns[1].reply.clone().expect("turn 2 was answered");
    assert!(
        question.contains(run.lines.old) && question.contains(&run.lines.code),
        "the agent's question was answered from the line the window said before the rejoin: {question:?}"
    );
    for turn in &run.turns {
        assert!(
            (OPENS..CLOSES).contains(&turn.world_at),
            "turn {} inside the pinned window",
            turn.n
        );
    }
    no_trigger_replayed(&run);
    isolated(&run, &facts);
    let others = others_acted(&run, &facts);
    assert!(others > 0, "another resident acted during the conversation");
    let (status, _) = run_command(&["inspect", &run.save, "--last", "0"]);
    assert!(status.success(), "mineworld inspect resolves every cause");
    if let Ok(out) = std::env::var("MINEWORLD_CORRECTION_GATE_OUT") {
        let calls = json!({ "other_resident_actions": others });
        write_evidence(&run, Path::new(&out), &checks, calls);
    }
    println!(
        "[correction] PASS: {} ({})",
        run.directory.path(),
        run.lines.code
    );
}

/// The §6 live gate (operator-run, once, on the final head): the same scenario with the local model in `record`
/// mode and the gate's configuration. It writes every artefact to `MINEWORLD_CORRECTION_GATE_OUT` and asserts
/// only the harness's own integrity: the rubric is scored from that evidence (§6 Judges).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "operator-run: the bounded live gate against the local model (pr-playable-dialogue-memory.md §6)"]
async fn the_bounded_live_gate() {
    let out = PathBuf::from(
        std::env::var("MINEWORLD_CORRECTION_GATE_OUT")
            .expect("MINEWORLD_CORRECTION_GATE_OUT names the directory the evidence is written to"),
    );
    let mode = if std::env::var("MINEWORLD_CORRECTION_GATE_DRY").is_ok_and(|dry| dry == "1") {
        Mode::scripted(true)
    } else {
        Mode::Live {
            base_url: std::env::var("MINEWORLD_CORRECTION_GATE_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434/v1".to_owned()),
            model: std::env::var("MINEWORLD_CORRECTION_GATE_MODEL")
                .unwrap_or_else(|_| "qwen3.5:4b".to_owned()),
        }
    };
    let started = Instant::now();
    let run = corrected(&mode).await;
    let facts = facts(&run);
    let checks = machine_checks(&run, &facts);
    let warmed = run.warmed;
    let ledger = run.ledger_calls();
    let calls = json!({
        "warm_up": warmed, "ledger_calls": ledger, "total": ledger + warmed, "cap": CALL_CAP,
        "consults": decisions(&run.store, "alice").iter().map(|consult| json!({
            "trigger": consult["input"]["trigger"]["kind"], "outcome": consult["record"]["outcome"],
            "attempts": consult["record"]["attempts"] })).collect::<Vec<_>>(),
        "other_resident_actions": if run.turns.len() == 5 { others_acted(&run, &facts) } else { 0 },
        "run_wall_s": started.elapsed().as_secs(),
    });
    write_evidence(&run, &out, &checks, calls);
    for (check, holds, detail) in &checks {
        println!(
            "[correction] {} {check}: {detail}",
            if *holds { "HOLDS" } else { "FAILS" }
        );
    }
    println!("[correction] evidence in {}", out.display());
    assert!(ledger + warmed <= CALL_CAP, "the call cap held");
}
