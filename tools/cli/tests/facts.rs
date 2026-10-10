//! Facts in observations through the real binary, judged by presence's audience rule
//! (step-12 §17 CA-3, CA-4; `DECISIONS.md` `ARC-43`).
//!
//! ```text
//! CA-3  a line said in the café reaches a place-mate who is not part of the exchange, never a
//!       person in another place; once the place-mate has walked out, the next line does not
//!       reach them; conversation-started (Participants) reaches only its participants
//! CA-4  a perceived connection that stops reading on a hosted market town: no fact in two frames'
//!       events, and events + events_dropped = the perceived stream
//! ```
//!
//! The judgement made *at record time within one dispatch* is pinned at the server, where a
//! test-local system can state two facts in one dispatch (`server/tests/facts.rs`, step-12 D-SC9):
//! through this binary every request is swept on its own, so no sweep-free window can be built.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use mineworld_contracts::{EntityId, EntityKey, EventId, WorldTime};
use mineworld_server::{RefusalCode, ServerFrame};
use mineworld_worldpack::WorldPack;
use serde_json::json;
use support::{Client, MARKET_PACK, PACK, Server, stride, tagged, talk, walk};

/// Every fact carried by this client's observations for `window`: its id and event type.
async fn learned(client: &mut Client, window: Duration) -> BTreeMap<EventId, String> {
    let deadline = Instant::now() + window;
    let mut facts = BTreeMap::new();
    while Instant::now() < deadline {
        let observation = client.observation().await;
        facts.extend(observation.events().iter().map(|event| {
            (
                event.envelope().id(),
                event.envelope().event_type().as_str().to_owned(),
            )
        }));
    }
    facts
}

/// The fact among `ids` that is a line said (`spoke`), as the speaker learned it.
fn line(ids: &[EventId], speaker: &BTreeMap<EventId, String>) -> EventId {
    *ids.iter()
        .find(|id| speaker.get(id).is_some_and(|kind| kind == "spoke"))
        .expect("the talk stated a line the speaker heard")
}

fn id_in(pack: &str, key: &str) -> EntityId {
    WorldPack::read(pack)
        .expect("reads")
        .load(WorldTime::EPOCH)
        .expect("loads")
        .id(&EntityKey::new(key).expect("a key"))
        .expect("declared")
}

#[tokio::test]
async fn a_line_reaches_who_was_there_and_nobody_else() {
    let server = Server::start(&["server", PACK]).await;
    let mut a = Client::connect(server.address).await;
    let mut b = Client::connect(server.address).await;
    let mut c = Client::connect(server.address).await;
    // From the authored `location:` lines: visitor and wanderer in the café, carol in the apartments.
    let (visitor, _) = a.join("visitor").await;
    let (wanderer, _) = b.join("wanderer").await;
    c.join("carol").await;
    let seen = a.observation().await;
    let alice = tagged(&seen, "barista").expect("alice is in the café");
    let (cafe, street) = (id_in(PACK, "cafe"), id_in(PACK, "street"));
    // Into reach of Alice, which the server decides; the positions are ac15's literals.
    a.walk_accepted(walk(visitor, cafe, (1_610, 600), (6_000, 6_200)))
        .await;

    // A line to Alice: a Place(café) fact naming the visitor and Alice, not the wanderer.
    let (_, events) = a
        .submit_accepted(talk(visitor, alice, "good morning"))
        .await;
    let by_a = learned(&mut a, Duration::from_millis(600)).await;
    let by_b = learned(&mut b, Duration::from_millis(600)).await;
    let by_c = learned(&mut c, Duration::from_millis(600)).await;
    let first = line(&events, &by_a);
    assert!(
        by_b.contains_key(&first),
        "the place-mate overhears the line"
    );
    assert!(
        !by_c.contains_key(&first),
        "a person in another place does not"
    );
    // conversation-started is stated for its participants only: the speaker (and Alice, unseated).
    let participants_only: Vec<EventId> = events
        .iter()
        .copied()
        .filter(|id| {
            by_a.get(id)
                .is_some_and(|kind| kind == "conversation-started")
        })
        .collect();
    assert!(
        !participants_only.is_empty(),
        "the first exchange opens a conversation"
    );
    for id in &participants_only {
        assert!(
            !by_b.contains_key(id) && !by_c.contains_key(id),
            "#{} reached a non-participant",
            id.raw()
        );
    }

    // The wanderer walks out to the street through the café's front door.
    let here = b.observation().await;
    let local = here
        .self_location()
        .and_then(|location| location.local())
        .expect("a position");
    let from = (local.x().value(), local.y().value());
    eprintln!("the wanderer walks from {from:?} to the door");
    b.walk_accepted(walk(wanderer, cafe, from, (1_610, 200)))
        .await;
    eprintln!("and crosses");
    b.submit_accepted(stride(wanderer, street, 0, 3_000)).await;
    b.observation_where("the wanderer in the street", |observation| {
        observation
            .self_location()
            .is_some_and(|location| location.place().entity_id() == street)
    })
    .await;

    let (_, later) = a.submit_accepted(talk(visitor, alice, "and goodbye")).await;
    let by_a = learned(&mut a, Duration::from_millis(600)).await;
    let by_b = learned(&mut b, Duration::from_millis(600)).await;
    let second = line(&later, &by_a);
    assert!(
        !by_b.contains_key(&second),
        "a line said after the wanderer left does not reach them"
    );
    eprintln!(
        "line #{} overheard in the café, #{} not after leaving; {} participant-only fact(s)",
        first.raw(),
        second.raw(),
        participants_only.len()
    );
}

/// A live-only perceived join on a world without a save: the cursor must be its head. The head is
/// read off the `cursor_unavailable` a too-new cursor is answered with, and the join retried if the
/// town recorded another fact in between.
async fn join_live(client: &mut Client, seat: &str) {
    let joining = |since: String| {
        json!({ "t": "join", "protocol": 2, "invite": support::INVITE, "nickname": "slow",
                "seat": seat, "perceived": { "since": since } })
    };
    for _ in 0..20 {
        let ServerFrame::Refused { code, detail, .. } =
            client.join_as(joining(u64::MAX.to_string())).await
        else {
            panic!("a cursor newer than any fact is refused");
        };
        assert_eq!(code, RefusalCode::CursorUnavailable);
        let head = detail
            .as_deref()
            .and_then(|detail| detail.rsplit("the newest is ").next())
            .and_then(|tail| tail.trim_end_matches(')').parse::<u64>().ok())
            .expect("the refusal names the newest fact");
        match client.join_as(joining(head.to_string())).await {
            ServerFrame::Welcome { .. } => return,
            ServerFrame::Refused {
                code: RefusalCode::CursorUnavailable,
                ..
            } => {}
            other => panic!("{other:?}"),
        }
    }
    panic!("the town never paused long enough to join at its head");
}

/// Reads every frame for `window`.
async fn frames_for(client: &mut Client, window: Duration, into: &mut Vec<ServerFrame>) {
    let deadline = Instant::now() + window;
    while Instant::now() < deadline {
        into.push(client.frame().await);
    }
}

/// CA-4 through the binary: a perceived connection on a hosted market town stops reading for 5 s,
/// then reads on. At an observation boundary, the facts its observations carried plus the facts
/// `/status` counts as dropped are exactly the facts its perceived stream carried, and no fact is
/// in two observations.
#[tokio::test]
async fn observation_events_are_exact_or_accounted_on_a_hosted_town() {
    let server = Server::start(&["server", MARKET_PACK, "--town"]).await;
    let mut client = Client::connect(server.address).await;
    join_live(&mut client, "wanderer").await;
    let before = server.status().await["events_dropped"]
        .as_u64()
        .expect("a count");

    let mut frames = Vec::new();
    frames_for(&mut client, Duration::from_secs(3), &mut frames).await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    frames_for(&mut client, Duration::from_secs(5), &mut frames).await;
    // Stop at an observation boundary: every fact flushed so far has had its observation.
    loop {
        let frame = client.frame().await;
        let boundary = matches!(frame, ServerFrame::Observation { .. });
        frames.push(frame);
        if boundary {
            break;
        }
    }
    let dropped = server.status().await["events_dropped"]
        .as_u64()
        .expect("a count")
        - before;

    let mut reliable: Vec<u64> = Vec::new();
    let mut framed: Vec<u64> = Vec::new();
    for frame in frames {
        match frame {
            ServerFrame::Perceived { events, .. } => {
                reliable.extend(events.iter().map(|event| event.envelope().id().raw()));
            }
            ServerFrame::Observation { observation, .. } => {
                framed.extend(
                    observation
                        .events()
                        .iter()
                        .map(|event| event.envelope().id().raw()),
                );
            }
            _ => {}
        }
    }
    let distinct: BTreeSet<u64> = framed.iter().copied().collect();
    eprintln!(
        "perceived {} facts, {} in observations, {dropped} dropped",
        reliable.len(),
        framed.len()
    );
    assert!(
        !reliable.is_empty(),
        "the town said something the wanderer learned of"
    );
    assert_eq!(
        distinct.len(),
        framed.len(),
        "no fact in two frames' events"
    );
    assert!(framed.iter().all(|id| reliable.contains(id)));
    assert_eq!(
        framed.len() as u64 + dropped,
        reliable.len() as u64,
        "every fact is in one observation or counted as dropped"
    );
}
