//! `mineworld server worlds/social-cafe` — the acceptance this PR is judged on, run for real.
//!
//! Nothing here is mocked and nothing is in-process. The test starts the **actual binary** cargo
//! built, pointed at the **actual World Pack** in the repository, on an ephemeral port; then it
//! connects a real WebSocket client, occupies a seat the pack offers, and reads what the server
//! sends. That is the whole of PR 05c's A4:
//!
//! ```text
//! mineworld server worlds/social-cafe starts
//! a client connects to it
//! and what it perceives is the world the YAML describes
//! ```
//!
//! The last line is what makes this more than a smoke test: the observation has to name Alice, at the
//! position `people/alice.yaml` authored, with `talk` offered against her and refused for distance —
//! all decided by the server, from a file on disk, through the whole stack.
//!
//! The harness moved to `support/mod.rs` in PR 05d, where the `AC-15` test needs the same real binary
//! and real sockets. Nothing about what is asserted here changed with it.

mod support;

use mineworld_server::ServerFrame;
use serde_json::json;
use support::{Client, Server, may_talk_to, tagged};

#[tokio::test]
async fn the_server_starts_from_the_pack_and_says_what_it_is_hosting() {
    let server = Server::start(&["server", support::PACK]).await;

    let status = server.status().await;

    assert_eq!(
        status["entities"], 18,
        "the world the pack describes: six places and twelve people — {status}",
    );
    let systems: Vec<&str> = status["systems"]
        .as_array()
        .expect("a list of systems")
        .iter()
        .map(|system| system["system"].as_str().expect("a name"))
        .collect();
    assert_eq!(
        systems,
        [
            "presence",
            "movement",
            "conversation",
            "group-activity",
            "relationships",
            "naming",
            "schedule"
        ],
        "in the order world.yaml states, which is the order they reduce in",
    );
    assert_eq!(
        status["seats"],
        json!([
            "alice", "bob", "carol", "dev", "erin", "felix", "grace", "hana", "ivan", "visitor",
            "wanderer"
        ]),
        "the seats the pack offers: two for players, Alice for the agent that drives her, and the \
         town's other people, all through the same roster — {status}",
    );
    assert!(
        status["instance"].as_str().is_some_and(|id| id.len() == 32),
        "and which running world this is, which AC-15's evidence names: {status}",
    );
}

#[tokio::test]
async fn a_client_connects_to_the_hosted_pack_and_perceives_the_world_the_yaml_describes() {
    let server = Server::start(&["server", support::PACK]).await;
    let mut client = Client::connect(server.address).await;

    // The seat the pack offers, by the authoring key `world.yaml` names — a client never names an
    // entity, and this is the whole of why the roster is the world's.
    let (observer, world) = client.join("visitor").await;
    assert_eq!(
        observer.raw(),
        17,
        "the visitor is the seventeenth entity the pack allocates (six places, then ten people \
         before it in key order) — deterministically, every time",
    );
    assert_eq!(world.entities, 18);

    // And then the world itself, as this observer perceives it.
    let observation = client.observation().await;
    let perceived: Vec<u64> = observation
        .entities()
        .iter()
        .map(|entity| entity.id().raw())
        .collect();
    assert_eq!(
        perceived,
        [2, 7, 8, 17, 18],
        "the café and everybody in it, by the ids the pack resolved its keys to — and not the \
         street (5) or anybody in another place, which this observer is not in",
    );

    let alice = tagged(&observation, "barista").expect("alice is perceived, by her tag");
    assert_eq!(alice.raw(), 7);
    let position = observation
        .entity(alice)
        .and_then(|alice| alice.location())
        .and_then(|location| location.local())
        .expect("the position people/alice.yaml authored");
    assert_eq!(
        (position.x().value(), position.y().value()),
        (6000, 8000),
        "millimetres, exactly as the file wrote them, through the whole stack",
    );

    let talk: Vec<(bool, Option<u64>)> = observation
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == "talk")
        .map(|affordance| {
            (
                affordance.is_available(),
                affordance.target().map(|target| target.raw()),
            )
        })
        .collect();
    assert_eq!(
        talk.len(),
        3,
        "talk offered against the three other people: {talk:?}",
    );
    assert!(
        talk.iter().all(|(available, _)| !available),
        "and refused for all of them, because the pack puts the visitor at the door — the server \
         decided that, not the client: {talk:?}",
    );
    assert!(
        !may_talk_to(&observation, alice),
        "which is the same answer read the way a client reads it",
    );

    // `arrive` is retired (`DECISIONS.md` `ARC-26`): no system in this world provides it, so a
    // client that still sends it is answered `unavailable`, and nobody is moved. Moving is `move`.
    let (_, answer) = client
        .submit(json!({
            "actor": observer,
            "action_type": "arrive",
            "target": null,
            "payload": { "action_type": "arrive", "payload": { "location": {
                "place": { "entity": "1", "entity_type": "place" },
                "local": { "x": 1200, "y": 1000, "z": 0 },
                "facing": null,
            } } },
            "actor_location": null,
        }))
        .await;
    assert_eq!(answer, mineworld_contracts::ActionResult::Unavailable);
}

#[tokio::test]
async fn a_seat_the_pack_does_not_offer_is_refused() {
    let server = Server::start(&["server", support::PACK]).await;
    let mut client = Client::connect(server.address).await;

    // `otto` exists in this world and is not a seat: `world.yaml` offers every other Person and not
    // him, so nothing can connect *as* him. The roster is the world's, not the client's.
    client.send(support::join_frame("otto")).await;
    let frame = client.frame().await;
    assert!(
        matches!(frame, ServerFrame::Refused { .. }),
        "got: {frame:?}",
    );
}

// ---------------------------------------------------------------------------------------------
// Protocol revision 2 through the real binary (step-12 §15.4 SA-3, SA-4, SA-5, SA-7).
// ---------------------------------------------------------------------------------------------

/// A join with this invite and nickname, on a fresh connection; the server's first answer.
async fn join_with(address: std::net::SocketAddr, invite: &str, nickname: &str) -> ServerFrame {
    let mut client = Client::connect(address).await;
    client
        .send(
            json!({ "t": "join", "protocol": 2, "invite": invite, "nickname": nickname,
                      "seat": "visitor" }),
        )
        .await;
    client.frame().await
}

/// The next frame that is not part of the stream (an observation or a clock).
async fn answer(client: &mut Client) -> ServerFrame {
    loop {
        let frame = client.frame().await;
        if !support::streamed(&frame) {
            return frame;
        }
    }
}

/// Every byte of every file under a directory, for the claim that a secret is in none of them.
fn bytes_under(directory: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(directory).expect("the directory exists") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            found.extend(bytes_under(&path));
        } else {
            found.push((path.clone(), std::fs::read(&path).expect("readable")));
        }
    }
    found
}

/// SA-3 and SA-5's output half. With no invite given, the server makes one and prints it on exactly
/// one line — §11.4's join line — and nowhere else: not on stderr, not in any byte of the save. A
/// client joining with it is admitted, and the nickname it gives appears in nothing the server writes.
#[tokio::test]
async fn a_generated_invite_is_printed_on_one_line_and_kept_out_of_everything_else() {
    const NICKNAME: &str = "Zephyrine-7";
    let save = support::SaveDir::new("generated-invite");
    let (mut server, output) =
        Server::start_captured(&["server", support::PACK, "--save", save.path()], None).await;

    let line = output.line_starting("[mineworld] invite ").await;
    let token = line["[mineworld] invite ".len()..]
        .split_whitespace()
        .next()
        .expect("a token after the prefix")
        .to_owned();
    assert_eq!(
        token.len(),
        32,
        "a generated invite is 128 bits in hex: {line}"
    );
    // With no `--agent`, the suggested seat is the roster's first, as `/status` lists it.
    let first_seat = server.status().await["seats"][0]
        .as_str()
        .expect("a seat")
        .to_owned();
    assert_eq!(
        line,
        format!(
            "[mineworld] invite {token} — join with: {} seat={first_seat} invite={token}",
            server.address
        ),
        "the join line is the frozen form of step-12 §11.4"
    );

    let joined = join_with(server.address, &token, NICKNAME).await;
    assert!(
        matches!(&joined, ServerFrame::Welcome { nickname, .. } if nickname.as_str() == NICKNAME),
        "the printed invite admits a client: {joined:?}"
    );

    let status = server.kill();
    assert!(status.killed(), "SIGKILL ended the server: {status:?}");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let (stdout, stderr) = (output.stdout(), output.stderr());
    assert_eq!(
        stdout.iter().filter(|line| line.contains(&token)).count(),
        1,
        "the invite is on exactly one stdout line: {stdout:?}"
    );
    for line in stdout.iter().chain(&stderr) {
        assert!(!line.contains(NICKNAME), "a nickname was printed: {line}");
    }
    assert!(
        stderr.iter().all(|line| !line.contains(&token)),
        "the invite reached stderr: {stderr:?}"
    );
    for (path, bytes) in bytes_under(std::path::Path::new(save.path())) {
        assert!(
            !bytes
                .windows(token.len())
                .any(|window| window == token.as_bytes()),
            "the invite is in {}",
            path.display()
        );
    }
}

/// SA-3. An invite the operator gives through `MINEWORLD_INVITE` admits, and is printed nowhere; a
/// flag beats the variable; an illegal one stops the server without repeating it.
#[tokio::test]
async fn a_given_invite_admits_is_never_printed_and_the_flag_beats_the_environment() {
    const FROM_ENV: &str = "env-invite-55aa66bb";
    const FROM_FLAG: &str = "flag-invite-77cc88dd";

    let (mut server, output) =
        Server::start_captured(&["server", support::PACK], Some(FROM_ENV)).await;
    let line = output.line_starting("[mineworld] join with: ").await;
    assert!(line.ends_with("invite=<the invite you gave>"), "{line}");
    assert!(matches!(
        join_with(server.address, FROM_ENV, "env").await,
        ServerFrame::Welcome { .. }
    ));
    server.kill();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    for written in output.stdout().iter().chain(&output.stderr()) {
        assert!(
            !written.contains(FROM_ENV),
            "a given invite was printed: {written}"
        );
    }

    let (server, _output) = Server::start_captured(
        &["server", support::PACK, "--invite", FROM_FLAG],
        Some(FROM_ENV),
    )
    .await;
    assert!(matches!(
        join_with(server.address, FROM_FLAG, "flag").await,
        ServerFrame::Welcome { .. }
    ));
    assert!(
        matches!(
            join_with(server.address, FROM_ENV, "env").await,
            ServerFrame::Refused {
                code: mineworld_server::RefusalCode::Unauthorized,
                ..
            }
        ),
        "with both given, the flag's invite is the server's"
    );

    let refused = std::process::Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(["server", support::PACK, "--invite", "abc12"])
        .env_remove("MINEWORLD_INVITE")
        .output()
        .expect("the binary runs");
    assert!(
        !refused.status.success(),
        "an illegal invite stops the server"
    );
    let said = String::from_utf8_lossy(&refused.stderr).into_owned()
        + &String::from_utf8_lossy(&refused.stdout);
    assert!(said.contains("not usable"), "{said}");
    assert!(
        !said.contains("abc12"),
        "the refusal repeated the invite: {said}"
    );
}

/// SA-4, `INV-9` over revision 2 against a persisted world. Every state-asserting message is refused,
/// and the save does not move: the revision `/status` reports is unchanged, and after the process dies
/// the save holds exactly the genesis facts `validate` counts — an oracle that never asks the server.
/// Inside the hosted world's first minutes (from 00:00, before any routine) nothing else is due.
#[tokio::test]
async fn state_assertions_are_refused_and_the_save_does_not_move() {
    let save = support::SaveDir::new("inv9-revision-2");
    let mut server = Server::start(&["server", support::PACK, "--save", save.path()]).await;
    let mut client = Client::connect(server.address).await;
    let (visitor, _) = client.join("visitor").await;
    let before = server.status().await["revision"].clone();

    for asserted in [
        json!({ "t": "set_state", "entity": visitor, "money": 5000 }),
        json!({ "t": "move_to", "position": { "x": 1000, "y": 2000, "z": 0 } }),
        json!({ "t": "give", "item": "1", "to": visitor }),
    ] {
        client.send(asserted.clone()).await;
        assert!(
            matches!(
                answer(&mut client).await,
                ServerFrame::Refused {
                    code: mineworld_server::RefusalCode::UnknownFrame,
                    ..
                }
            ),
            "{asserted}"
        );
    }
    let somebody_else = mineworld_contracts::EntityId::from_raw(visitor.raw() + 1);
    assert_eq!(
        client
            .submit_refused(support::talk(somebody_else, visitor, "I am somebody else"))
            .await,
        mineworld_server::RefusalCode::ActorNotObserver
    );

    let mut naming_itself = Client::connect(server.address).await;
    let mut join = support::join_frame("wanderer");
    join["observer"] = json!(visitor);
    naming_itself.send(join).await;
    assert!(
        matches!(
            naming_itself.frame().await,
            ServerFrame::Refused {
                code: mineworld_server::RefusalCode::MalformedFrame,
                ..
            }
        ),
        "a join that names an observer is malformed, and no welcome follows"
    );

    assert_eq!(
        server.status().await["revision"],
        before,
        "nothing was committed"
    );
    server.kill();

    let (_, validated) = support::run_command(&["validate", support::PACK]);
    let genesis = validated
        .lines()
        .find_map(|line| line.trim().strip_suffix(" genesis fact(s): the world's initial state, each one caused by the world coming into existence"))
        .expect("validate counts the genesis facts")
        .trim()
        .to_owned();
    let (_, inspected) = support::run_command(&["inspect", save.path()]);
    let facts = inspected
        .lines()
        .find_map(|line| line.strip_prefix("facts"))
        .expect("inspect counts the facts")
        .trim()
        .to_owned();
    assert_eq!(
        facts, genesis,
        "the save holds the genesis facts and nothing else"
    );
}

/// SA-7: `/status` and `/health` of revision 2, from the real pack. The world's vocabulary is
/// public composition: what each system provides and what it states.
#[tokio::test]
async fn status_names_each_systems_actions_and_facts() {
    let server = Server::start(&["server", support::PACK]).await;
    let status = server.status().await;

    assert_eq!(status["protocol"], json!(2));
    assert!(status.get("deferrals_unscheduled").is_none(), "{status}");
    assert_eq!(status["events_dropped"], json!(0));
    let conversation = status["systems"]
        .as_array()
        .expect("a list")
        .iter()
        .find(|system| system["system"] == json!("conversation"))
        .expect("social-cafe enables conversation");
    assert!(
        conversation["provides"]
            .as_array()
            .expect("a list")
            .contains(&json!("talk")),
        "{conversation}"
    );
    assert!(
        conversation["states"]
            .as_array()
            .expect("a list")
            .contains(&json!("spoke")),
        "{conversation}"
    );

    let health = support::body(
        &support::get(server.address, "/health")
            .await
            .expect("health"),
    );
    assert_eq!(health["protocol"], json!(2));
}

/// step-12 SB-8. `--time-scale N` is how many world seconds pass per wall second, reported in every
/// welcome and in `/status`; without the flag it is one.
#[tokio::test]
async fn time_scale_sets_how_fast_the_hosted_world_s_seconds_pass_and_is_reported() {
    let plain = Server::start(&["server", support::PACK]).await;
    assert_eq!(plain.status().await["time_scale"], json!(1));
    drop(plain);

    let server = Server::start(&["server", support::PACK, "--time-scale", "60"]).await;
    let mut client = Client::connect(server.address).await;
    let (_, world) = client.join("visitor").await;
    assert_eq!(world.time_scale, 60, "reported in the welcome");

    let before = server.status().await;
    let wall = std::time::Instant::now();
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let after = server.status().await;
    let elapsed = wall.elapsed().as_secs_f64();
    assert_eq!(before["time_scale"], json!(60), "and in /status");
    let world_seconds =
        after["at"].as_i64().expect("an instant") - before["at"].as_i64().expect("an instant");
    assert!(
        (100..=140).contains(&world_seconds),
        "two answers {elapsed:.2} wall seconds apart differ by {world_seconds} world seconds, not \
         about 120"
    );
}

/// CA-11, `INV-9` over S11-C's surfaces (step-12 §17.5): a client cannot send what only a server
/// says (`perceived`, `delta`), cannot widen what it asks for with a field the protocol does not have,
/// cannot state `acted_through`, and a cursor this world cannot serve grants nothing — the same
/// connection then joins. The save does not move: `/status`'s revision is unchanged, and after the
/// process dies the save holds exactly the genesis facts `validate` counts.
#[tokio::test]
async fn s11c_frames_a_client_may_not_send_are_refused_and_the_save_does_not_move() {
    use mineworld_server::RefusalCode::{CursorUnavailable, MalformedFrame, UnknownFrame};

    let save = support::SaveDir::new("inv9-s11c");
    let mut server = Server::start(&["server", support::PACK, "--save", save.path()]).await;
    let mut client = Client::connect(server.address).await;
    let (visitor, _) = client.join("visitor").await;
    let before = server.status().await["revision"].clone();
    let refused = |frame: ServerFrame| match frame {
        ServerFrame::Refused { code, .. } => Some(code),
        _ => None,
    };

    for server_only in [
        json!({ "t": "perceived", "through": "1", "events": [] }),
        json!({ "t": "delta", "seq": 2, "base": 1, "revision": 1, "delta": {} }),
    ] {
        client.send(server_only.clone()).await;
        assert_eq!(
            refused(answer(&mut client).await),
            Some(UnknownFrame),
            "{server_only}"
        );
    }
    let mut stated = json!({ "t": "submit", "token": "c1",
                             "request": support::talk(visitor, visitor, "hello") });
    stated["acted_through"] = json!("41");
    client.send(stated).await;
    assert_eq!(
        refused(answer(&mut client).await),
        Some(MalformedFrame),
        "acted_through"
    );

    for extra in ["events", "observer"] {
        let mut widening = Client::connect(server.address).await;
        let mut join = support::join_frame("wanderer");
        join["perceived"] = json!({ "since": null, extra: [] });
        widening.send(join).await;
        assert_eq!(
            refused(widening.frame().await),
            Some(MalformedFrame),
            "perceived carrying {extra} is malformed, and no welcome follows"
        );
    }

    let mut cursor = Client::connect(server.address).await;
    let mut join = support::join_frame("wanderer");
    join["perceived"] = json!({ "since": "1000000" });
    cursor.send(join.clone()).await;
    assert_eq!(refused(cursor.frame().await), Some(CursorUnavailable));
    join["perceived"] = json!({ "since": null });
    cursor.send(join).await;
    assert!(
        matches!(cursor.frame().await, ServerFrame::Welcome { .. }),
        "the same connection joins once it asks for a cursor the world can serve"
    );

    assert_eq!(
        server.status().await["revision"],
        before,
        "nothing was committed"
    );
    server.kill();

    let (_, validated) = support::run_command(&["validate", support::PACK]);
    let genesis = validated
        .lines()
        .find_map(|line| line.trim().strip_suffix(" genesis fact(s): the world's initial state, each one caused by the world coming into existence"))
        .expect("validate counts the genesis facts")
        .trim()
        .to_owned();
    let (_, inspected) = support::run_command(&["inspect", save.path()]);
    let facts = inspected
        .lines()
        .find_map(|line| line.strip_prefix("facts"))
        .expect("inspect counts the facts")
        .trim()
        .to_owned();
    assert_eq!(
        facts, genesis,
        "the save holds the genesis facts and nothing else"
    );
}

/// step-12 DA-9's command-line half. `--help` names `MINEWORLD_ADMIN_TOKEN` and never its value; an
/// illegal admin token, or one equal to the invite, stops the server before it listens, non-zero,
/// echoing neither.
#[test]
fn an_unusable_admin_token_stops_the_server_and_help_shows_no_value() {
    const SET: &str = "help-admin-token-0d9a";
    let help = std::process::Command::new(env!("CARGO_BIN_EXE_mineworld"))
        .args(["server", "--help"])
        .env("MINEWORLD_ADMIN_TOKEN", SET)
        .output()
        .expect("the binary runs");
    let help = String::from_utf8_lossy(&help.stdout).into_owned();
    assert!(help.contains("MINEWORLD_ADMIN_TOKEN"), "{help}");
    assert!(
        !help.contains(SET),
        "--help showed the variable's value: {help}"
    );

    const INVITE: &str = "same-secret-for-both";
    for (case, arguments) in [
        (
            "equal to the invite",
            vec!["--invite", INVITE, "--admin-token", INVITE],
        ),
        (
            "illegal",
            vec!["--invite", "an-invite-0001", "--admin-token", "short"],
        ),
    ] {
        let refused = std::process::Command::new(env!("CARGO_BIN_EXE_mineworld"))
            .args(["server", support::PACK, "--listen", "127.0.0.1:0"])
            .args(&arguments)
            .env_remove("MINEWORLD_INVITE")
            .env_remove("MINEWORLD_ADMIN_TOKEN")
            .output()
            .expect("the binary runs");
        assert!(
            !refused.status.success(),
            "an admin token {case} stops the server"
        );
        let said = String::from_utf8_lossy(&refused.stderr).into_owned()
            + &String::from_utf8_lossy(&refused.stdout);
        assert!(said.contains("--admin-token"), "{case}: {said}");
        assert!(
            !said.contains("listening"),
            "{case}: the server listened: {said}"
        );
        for secret in [arguments[1], arguments[3]] {
            assert!(
                !said.contains(secret),
                "{case}: the refusal repeated {secret:?}: {said}"
            );
        }
    }
}
