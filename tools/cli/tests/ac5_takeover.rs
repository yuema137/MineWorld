//! `AC-5` — a person takes over a Person an in-server controller drives, and the Person is intact
//! (step-12 SB-1, SB-4; CP-B1). Control is host state: no binding change moves the revision (`ARC-40`).
//!
//! ```text
//! SB-1  mineworld server worlds/social-cafe --save DIR --agent alice --hold 3
//!       join alice (hosted) · leave · join visitor · drop · resume (held) · take over (connection)
//!       · leave · drop and let the hold expire: the persisted revision never moves, and the save
//!       holds the genesis facts and nothing else
//! SB-4  mineworld server worlds/market-town --save DIR --agent alice
//!       a player takes alice, talks to bob, buys what the café offers and leaves: her biography
//!       before is a prefix of her biography after, and every entry added was caused by a request a
//!       player made — none by her controller, which yielded while she was played
//! ```

mod fixture;
mod headless;
mod market;
mod support;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use market::{MARKET_TOWN, Town};
use mineworld_server::{ClosingReason, ServerFrame, TookOver};
use serde_json::{Value, json};
use support::{Client, SaveDir, Server, join_frame, run_command, talk};

/// How long `SB-1`'s server holds a dropped seat, in wall seconds.
const HOLD: u64 = 3;

fn took_over(frame: &ServerFrame) -> TookOver {
    match frame {
        ServerFrame::Welcome { took_over, .. } => *took_over,
        other => panic!("a welcome was expected, and the server said {other:?}"),
    }
}

fn resume_of(frame: &ServerFrame) -> String {
    match frame {
        ServerFrame::Welcome { resume, .. } => {
            resume.as_ref().expect("a resume").reveal().to_owned()
        }
        other => panic!("a welcome was expected, and the server said {other:?}"),
    }
}

async fn closed_because(client: &mut Client, expected: ClosingReason) {
    match client.answer().await {
        ServerFrame::Closing { reason, .. } => assert_eq!(reason, expected),
        other => panic!("a closing was expected, and the server said {other:?}"),
    }
}

async fn leave(client: &mut Client) {
    client.send(json!({ "t": "leave" })).await;
    closed_because(client, ClosingReason::Left).await;
}

#[tokio::test]
async fn binding_changes_are_host_state_and_move_no_revision() {
    // Nobody speaks and no routine boundary falls in the first minutes after genesis, so the only
    // thing that could move the revision here is a binding change.
    fixture::assert_quiet(0, 3_600, "ac5_takeover.rs");
    let save = SaveDir::new("ac5-host-state");
    let hold = HOLD.to_string();
    let mut server = Server::start(&[
        "server",
        support::PACK,
        "--agent",
        "alice",
        "--hold",
        &hold,
        "--save",
        save.path(),
    ])
    .await;
    let revision = server.status().await["revision"].clone();
    assert_eq!(revision, json!(1), "a new world, at genesis");
    let unmoved = |what: &str, status: &Value| {
        assert_eq!(
            status["revision"], revision,
            "{what} moved the revision: {status}"
        );
    };

    // A player takes alice from her controller, and gives her back.
    let mut player = Client::connect(server.address).await;
    let welcome = player.join_as(join_frame("alice")).await;
    assert_eq!(
        took_over(&welcome),
        TookOver::Hosted,
        "the agent was driving alice"
    );
    unmoved("taking alice from her controller", &server.status().await);
    leave(&mut player).await;
    unmoved("leaving alice", &server.status().await);
    let mut again = Client::connect(server.address).await;
    assert_eq!(
        took_over(&again.join_as(join_frame("alice")).await),
        TookOver::Hosted,
        "alice went back to her controller when the player left"
    );
    leave(&mut again).await;

    // A dropped player's seat is held, and resumed.
    let mut first = Client::connect(server.address).await;
    let secret = resume_of(&first.join_as(join_frame("visitor")).await);
    first.disconnect().await;
    unmoved("a dropped socket", &server.status().await);
    let mut resumed = Client::connect(server.address).await;
    let mut join = join_frame("visitor");
    join["resume"] = json!(secret);
    assert_eq!(took_over(&resumed.join_as(join).await), TookOver::Held);
    unmoved("a resume", &server.status().await);

    // Another connection takes it with take_over; the holder is told and closed.
    let mut taker = Client::connect(server.address).await;
    let mut take = join_frame("visitor");
    take["take_over"] = json!(true);
    assert_eq!(took_over(&taker.join_as(take).await), TookOver::Connection);
    closed_because(&mut resumed, ClosingReason::TakenOver).await;
    unmoved("a takeover", &server.status().await);
    leave(&mut taker).await;
    unmoved("leaving the visitor", &server.status().await);

    // A drop whose hold runs out.
    let mut dropped = Client::connect(server.address).await;
    dropped.join_as(join_frame("visitor")).await;
    dropped.disconnect().await;
    tokio::time::sleep(Duration::from_millis(HOLD * 1_000 + 500)).await;
    let mut after = Client::connect(server.address).await;
    assert_eq!(
        took_over(&after.join_as(join_frame("visitor")).await),
        TookOver::None,
        "the hold ended and the seat is free again"
    );
    unmoved("a hold expiring", &server.status().await);

    // The save holds genesis and nothing else.
    server.kill();
    let (validated, said) = run_command(&["validate", support::PACK]);
    assert!(validated.success(), "{said}");
    let genesis = said
        .lines()
        .find_map(|line| {
            line.split_once(" genesis fact(s)")
                .map(|(count, _)| count.trim().to_owned())
        })
        .expect("validate counts the genesis facts");
    let (inspected, report) = run_command(&["inspect", save.path()]);
    assert!(inspected.success(), "{report}");
    assert!(
        report
            .lines()
            .any(|line| line == format!("facts      {genesis}")),
        "the save holds the {genesis} genesis facts and nothing else: {report}"
    );
}

/// Alice's biography, one JSON object per entry, read from a stopped server's save.
fn biography(save: &str) -> Vec<Value> {
    let (status, printed) = run_command(&[
        "biography",
        MARKET_TOWN,
        "--save",
        save,
        "--person",
        "alice",
        "--json",
    ]);
    assert!(status.success(), "{printed}");
    printed
        .lines()
        .map(|line| serde_json::from_str(line).expect("one JSON entry per line"))
        .collect()
}

/// The observer's own disclosed components, by type.
fn own_components(observation: &mineworld_server::WireObservation) -> Vec<(String, Value)> {
    observation
        .entity(observation.observer())
        .expect("the observer perceives itself")
        .components()
        .iter()
        .map(|record| {
            (
                record.component_type().as_str().to_owned(),
                record.payload().clone(),
            )
        })
        .collect()
}

#[tokio::test]
async fn a_player_takes_alice_from_her_controller_and_she_is_the_same_person() {
    fixture::assert_quiet_in(
        &Path::new(MARKET_TOWN).join("people"),
        0,
        3_600,
        "ac5_takeover.rs",
    );
    let town = Town::read(Path::new(MARKET_TOWN));
    let (alice, bob) = (town.id("alice"), town.id("bob"));
    let save = SaveDir::new("ac5-takeover");
    let command = [
        "server",
        MARKET_TOWN,
        "--agent",
        "alice",
        "--save",
        save.path(),
    ];

    // ── Before: a new save, read with the server stopped. ────────────────────────────────────
    Server::start(&command).await.kill();
    let before = biography(save.path());

    // ── A player takes alice. ────────────────────────────────────────────────────────────────
    let mut server = Server::start(&command).await;
    let head = server.status().await["revision"]
        .as_u64()
        .expect("persisted");
    let mut player = Client::connect(server.address).await;
    let welcome = player.join_as(join_frame("alice")).await;
    assert_eq!(took_over(&welcome), TookOver::Hosted);
    assert_eq!(
        player.observer,
        Some(alice),
        "the seat is the same Person validate lists"
    );
    let first = player
        .observation_where("talk to bob available", |seen| {
            support::may_talk_to(seen, bob)
        })
        .await;
    let mut caused: BTreeSet<u64> = BTreeSet::new();
    let (_, events) = player
        .submit_accepted(talk(alice, bob, "morning, Bob"))
        .await;
    caused.extend(events.iter().map(|event| event.raw()));

    // Bob's player speaks to her while she is played: her controller must not answer for her.
    let mut bobs = Client::connect(server.address).await;
    bobs.join("bob").await;
    let (_, events) = bobs
        .submit_accepted(talk(bob, alice, "morning, Alice"))
        .await;
    caused.extend(events.iter().map(|event| event.raw()));

    let buy = player
        .observation_where("an available complete buy", |seen| {
            seen.affordances().iter().any(|offer| {
                offer.action_type().as_str() == "buy"
                    && offer.is_available()
                    && offer.payload().is_some()
            })
        })
        .await
        .affordances()
        .iter()
        .find(|offer| {
            offer.action_type().as_str() == "buy"
                && offer.is_available()
                && offer.payload().is_some()
        })
        .cloned()
        .expect("found above");
    let request = json!({
        "actor": alice,
        "action_type": buy.action_type().as_str(),
        "target": buy.target(),
        "payload": { "action_type": buy.action_type().as_str(), "payload": buy.payload() },
        "actor_location": null,
    });
    let (_, events) = player.submit_accepted(request).await;
    caused.extend(events.iter().map(|event| event.raw()));
    // Three wall seconds: three consults of a reactive controller that would have answered Bob.
    tokio::time::sleep(Duration::from_secs(3)).await;
    let last = player.observation().await;
    leave(&mut player).await;
    // Nobody acted but the two players: three requests, three revisions past the save's head. Her
    // controller answering Bob would be a fourth (`talk` is not a biographical fact, so the
    // biography alone would not show it).
    let status = server.status().await;
    assert_eq!(
        status["revision"],
        json!(head + 3),
        "only the players' three requests were journaled: {status}"
    );
    server.kill();

    // ── After: the same life, longer by what the players did and nothing else. ──────────────
    let after = biography(save.path());
    assert!(
        after.len() > before.len() && after[..before.len()] == before[..],
        "the biography before is a prefix of the biography after"
    );
    for entry in &after[before.len()..] {
        let event = entry["event_id"].as_u64().expect("an event id");
        assert!(
            caused.contains(&event),
            "an entry no player's request caused — her controller acted while she was played: \
             {entry}"
        );
    }

    let (from, to) = (own_components(&first), own_components(&last));
    let changed: BTreeSet<&str> = from
        .iter()
        .chain(&to)
        .filter(|component| !(from.contains(component) && to.contains(component)))
        .map(|(kind, _)| kind.as_str())
        .collect();
    println!("alice's own components that changed: {changed:?}");
    // What she paid, what she bought, what was said — and, because a first talk makes two people
    // acquainted, her acquaintance with bob (step-12 D-SB7).
    let explained: BTreeSet<&str> = [
        "wallet",
        "holdings",
        "conversation-history",
        "acquaintances",
    ]
    .into_iter()
    .collect();
    assert!(
        changed.is_subset(&explained) && changed.contains("wallet") && changed.contains("holdings"),
        "alice changed in exactly what she bought, paid and said: {changed:?}"
    );
    let component = |seen: &[(String, Value)], kind: &str| {
        seen.iter()
            .find(|(found, _)| found == kind)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| panic!("alice's own {kind}"))
    };
    let (paid_from, paid_to) = (
        market::wallet_balance(&component(&from, "wallet")),
        market::wallet_balance(&component(&to, "wallet")),
    );
    assert!(paid_to < paid_from, "she paid: {paid_from} → {paid_to}");
    let carried = |seen: &[(String, Value)]| -> u32 {
        market::holdings(&component(seen, "holdings"))
            .values()
            .sum()
    };
    assert_eq!(
        carried(&to),
        carried(&from) + 1,
        "and carries one more thing"
    );
}
