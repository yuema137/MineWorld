//! `AC-7` — **several human clients inhabit one world at once and meet the same NPCs** (step-12 §9.5
//! CP-E1, §20.3 SE-1).
//!
//! ```text
//! mineworld server worlds/market-town --town --agent alice --save DIR
//! four connections   visitor, wanderer, bob, carol — four nicknames, each seat taken from the town
//! all four           walk to Alice at once, talk to her, and are each answered
//! one buys           the visitor buys at the café; the other three see the shelf one lower, and
//!                    nothing of the visitor's own
//! one world          at every revision two of them observe, the people they all perceive stand where
//!                    the others see them; a fifth connection, seated on Alice, reads all four lines
//!                    in her own history
//! ```
//!
//! Every client paces itself at one request per [`PACE`] — under the per-seat budget every seat has
//! (10 a second, a burst of 20; `docs/DECISIONS.md` `ARC-97`) — as a real client must. Nothing is read
//! out of the server: only frames, `GET /status` and the CLI's own `replay`.

mod fixture;
mod headless;
mod market;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::{Duration, Instant};

use futures_util::future::join_all;
use market::{MARKET_TOWN, Town};
use mineworld_contracts::{ActionResult, EntityId, Rejection};
use mineworld_server::{RefusalCode, ServerFrame, TookOver, WireObservation};
use serde_json::{Value, json};
use support::{Client, SaveDir, Server, join_frame, may_talk_to, own_history, run_command, talk};

/// The four players: seat and nickname. Nicknames are not world data (step-12 QS11-7): each player
/// says theirs, which is how Alice can answer each of them by it.
const PLAYERS: [(&str, &str); 4] = [
    ("visitor", "Mira"),
    ("wanderer", "Jun"),
    ("bob", "Sol"),
    ("carol", "Ines"),
];

/// The spacing between one client's requests: 110 ms, nine a second, inside the default budget of
/// ten a second (`ARC-97`) — what the protocol module's own walk uses.
const PACE: Duration = Duration::from_millis(110);

/// How long all four are read at once after the purchase.
const WATCH: Duration = Duration::from_secs(3);

/// While all four are read, the visitor paces this far east and back, one stride every
/// [`SHUFFLE`] — two requests a second, inside the budget — so that the state compared across
/// players at one revision is state that changes: a client shown a stale observation under the
/// current revision then disagrees with the others (review finding R1, step-12 D-SE9).
const SHUFFLE_MM: i32 = 300;
const SHUFFLE: Duration = Duration::from_millis(500);

/// What a player says to Alice.
fn line(nickname: &str) -> String {
    format!("hello Alice, it is {nickname} at the counter")
}

/// One request, sent no sooner than [`PACE`] after this client's previous one.
async fn paced(client: &mut Client, request: Value) -> ActionResult {
    tokio::time::sleep(PACE).await;
    client.submit(request).await.1
}

fn reference(entity: EntityId, kind: &str) -> Value {
    json!({ "entity": entity.to_string(), "entity_type": kind })
}

fn request(actor: EntityId, action: &str, payload: Value) -> Value {
    json!({
        "actor": actor,
        "action_type": action,
        "target": null,
        "payload": { "action_type": action, "payload": payload },
        "actor_location": null,
    })
}

/// `walk-to` a place (entering it) or a person, then `walk-step` until the server says the walk is
/// over. The server plans the route through the doorways; the client only asks (`ARC-75`).
async fn walk_to(client: &mut Client, destination: Value) {
    let me = client.observer.expect("seated");
    // `walk-to` has no "already there" refusal (movement's `walk::begin`): a refusal here is a
    // failure, named.
    let asked = paced(
        client,
        request(me, "walk-to", json!({ "to": destination.clone() })),
    )
    .await;
    assert!(
        matches!(asked, ActionResult::Accepted { .. }),
        "walk-to {destination} is accepted: {asked:?}"
    );
    for _ in 0..80 {
        match paced(client, request(me, "walk-step", json!({}))).await {
            ActionResult::Accepted { .. } => {}
            ActionResult::Rejected(Rejection::PreconditionFailed) => return,
            other => panic!("a step is accepted, or refused once the walk is over: {other:?}"),
        }
    }
    panic!("the walk did not end in 80 steps");
}

/// Walks a player from wherever the town left them to within talking reach of Alice.
async fn walk_to_alice(client: &mut Client, town: &Town) {
    let (alice, cafe) = (town.id("alice"), town.id("cafe"));
    let seen = client.observation().await;
    let here = seen
        .self_location()
        .expect("the player knows where it is")
        .place()
        .entity_id();
    if here != cafe {
        walk_to(
            client,
            json!({ "place": { "place": reference(cafe, "place"), "local": null, "facing": null } }),
        )
        .await;
    }
    walk_to(client, json!({ "person": reference(alice, "person") })).await;
    client
        .observation_where("talk to alice available", |seen| may_talk_to(seen, alice))
        .await;
}

/// The shop listing an observation discloses, if any: per item, (price, in stock).
fn listing_in(observation: &WireObservation) -> Option<BTreeMap<EntityId, (u64, u32)>> {
    observation.entities().iter().find_map(|entity| {
        entity
            .components()
            .iter()
            .find(|record| record.component_type().as_str() == "shop")
            .map(|record| market::listing(record.payload()).1)
    })
}

/// The observer's own holdings, by item.
fn held(observation: &WireObservation) -> BTreeMap<EntityId, u32> {
    observation
        .entity(observation.observer())
        .and_then(|me| {
            me.components()
                .iter()
                .find(|record| record.component_type().as_str() == "holdings")
        })
        .map_or_else(BTreeMap::new, |record| market::holdings(record.payload()))
}

/// What this observation discloses of `holder` that is only the holder's own.
fn private_of(observation: &WireObservation, holder: EntityId) -> Vec<String> {
    observation
        .entity(holder)
        .map(|entity| {
            entity
                .components()
                .iter()
                .map(|record| record.component_type().as_str().to_owned())
                .filter(|kind| {
                    ["wallet", "holdings", "conversation-history"].contains(&kind.as_str())
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Every frame a client receives for `watch`, with the revision each named. With `strides`, the
/// client also sends them in turn, one every [`SHUFFLE`], without waiting for the answers (which
/// `perceived` skips): the visitor pacing while the others watch.
async fn record(
    client: &mut Client,
    watch: Duration,
    strides: Option<[Value; 2]>,
) -> Vec<(u64, WireObservation)> {
    let ends = Instant::now() + watch;
    let mut frames = Vec::new();
    let mut next = Instant::now();
    let mut sent = 0_usize;
    while Instant::now() < ends {
        if let Some(strides) = &strides
            && Instant::now() >= next
        {
            let token = format!("pace-{sent}");
            client
                .send(json!({ "t": "submit", "token": token, "request": strides[sent % 2] }))
                .await;
            sent += 1;
            next += SHUFFLE;
        }
        let (revision, observation) = client.perceived().await;
        let revision = revision.expect("a persisted world names a revision on every frame");
        frames.push((revision.raw(), observation));
    }
    frames
}

#[tokio::test(flavor = "multi_thread")]
async fn four_players_in_one_world_meet_the_same_alice() {
    let town = Town::read(Path::new(MARKET_TOWN));
    let alice = town.id("alice");
    let save = SaveDir::new("ac7-many-players");
    let mut server = Server::start(&[
        "server",
        MARKET_TOWN,
        "--town",
        "--agent",
        "alice",
        "--save",
        save.path(),
    ])
    .await;
    let instance = server.status().await["instance"].clone();

    // ── Four players, four seats, one world. ─────────────────────────────────────────────────
    let mut players: Vec<(String, String, Client)> = Vec::new();
    for (seat, nickname) in PLAYERS {
        let mut client = Client::connect(server.address).await;
        let mut join = join_frame(seat);
        join["nickname"] = json!(nickname);
        match client.join_as(join).await {
            ServerFrame::Welcome {
                observer,
                nickname: accepted,
                took_over,
                world,
                ..
            } => {
                assert_eq!(
                    observer,
                    town.id(seat),
                    "{seat} is the Person the pack names"
                );
                assert_eq!(
                    accepted.as_str(),
                    nickname,
                    "{seat}'s welcome is its own player's"
                );
                assert_eq!(
                    took_over,
                    TookOver::Hosted,
                    "{seat} was taken from the town, which was driving it"
                );
                assert_eq!(
                    json!(world.instance.to_string()),
                    instance,
                    "{seat}'s welcome names the one world /status names"
                );
            }
            other => panic!("{seat}: a welcome was expected, and the server said {other:?}"),
        }
        players.push((seat.to_owned(), nickname.to_owned(), client));
    }
    // A seat is one player's: a fifth connection is refused each of the four, by name.
    for (seat, _) in PLAYERS {
        let mut fifth = Client::connect(server.address).await;
        match fifth.join_as(join_frame(seat)).await {
            ServerFrame::Refused { code, .. } => assert_eq!(
                code,
                RefusalCode::SeatOccupied,
                "a second join on the occupied seat {seat} is refused seat_occupied"
            ),
            other => panic!(
                "a second join on the occupied seat {seat} must be refused seat_occupied, and the \
                 server said {other:?}"
            ),
        }
    }
    let status = server.status().await;
    assert_eq!(status["clients"], json!(4), "four connections: {status}");

    // ── All four walk to Alice at once, and talk to her. ─────────────────────────────────────
    join_all(
        players
            .iter_mut()
            .map(|(_, _, client)| walk_to_alice(client, &town)),
    )
    .await;
    for (index, (seat, nickname, client)) in players.iter_mut().enumerate() {
        let me = client.observer.expect("seated");
        tokio::time::sleep(PACE).await;
        client
            .submit_accepted(talk(me, alice, &line(nickname)))
            .await;
        let said = line(nickname);
        let answered = client
            .observation_where("a reply from alice", |seen| {
                own_history(seen).is_some_and(|history| {
                    history.heard().iter().any(|heard| {
                        heard.speaker().entity_id() == alice
                            && heard.utterance().as_str().contains(&said)
                    })
                })
            })
            .await;
        let reply = own_history(&answered)
            .expect("a history")
            .heard()
            .iter()
            .rev()
            .find(|heard| heard.speaker().entity_id() == alice)
            .map(|heard| heard.utterance().as_str().to_owned())
            .expect("found above");
        println!("alice answered {seat} ({nickname}): {reply}");
        // One Alice: from the second player on, she tells each what one of the others said — which
        // four synchronised copies, each told one conversation, could not.
        if index > 0 {
            let earlier = PLAYERS[..index]
                .iter()
                .any(|(_, other)| reply.contains("Earlier, ") && reply.contains(&line(other)));
            assert!(
                earlier,
                "Alice tells {seat} what a player before it said to her: {reply}"
            );
        }
    }

    // ── The visitor buys; the other three see the shelf change and nothing of the visitor's. ─
    let visitor = town.id("visitor");
    let mut before: BTreeMap<String, BTreeMap<EntityId, (u64, u32)>> = BTreeMap::new();
    for (seat, _, client) in players.iter_mut().skip(1) {
        let seen = client
            .observation_where("the café's listing", |seen| listing_in(seen).is_some())
            .await;
        before.insert(seat.clone(), listing_in(&seen).expect("found above"));
    }
    let buyer = &mut players[0].2;
    let seen = buyer
        .observation_where("an available complete buy", |seen| {
            seen.affordances().iter().any(|offer| {
                offer.action_type().as_str() == "buy"
                    && offer.is_available()
                    && offer.payload().is_some()
            })
        })
        .await;
    let held_before = held(&seen);
    let buy = seen
        .affordances()
        .iter()
        .find(|offer| {
            offer.action_type().as_str() == "buy"
                && offer.is_available()
                && offer.payload().is_some()
        })
        .cloned()
        .expect("found above");
    tokio::time::sleep(PACE).await;
    buyer
        .submit_accepted(json!({
            "actor": visitor,
            "action_type": buy.action_type().as_str(),
            "target": buy.target(),
            "payload": { "action_type": buy.action_type().as_str(), "payload": buy.payload() },
            "actor_location": null,
        }))
        .await;
    let bought_at = server.status().await["revision"]
        .as_u64()
        .expect("persisted");
    let after = buyer
        .observation_where("the visitor's holdings changed", |seen| {
            held(seen) != held_before
        })
        .await;
    let gained: Vec<EntityId> = held(&after)
        .iter()
        .filter(|(item, count)| held_before.get(item).copied().unwrap_or(0) + 1 == **count)
        .map(|(item, _)| *item)
        .collect();
    let [kind] = gained[..] else {
        panic!("the visitor carries one more of exactly one kind: {gained:?}");
    };
    for (seat, _, client) in players.iter_mut().skip(1) {
        let (_, in_stock) = before[seat.as_str()][&kind];
        let shelf = client
            .observation_where("the shelf one lower", |seen| {
                listing_in(seen).is_some_and(|shelf| shelf[&kind].1 != in_stock)
            })
            .await;
        assert_eq!(
            listing_in(&shelf).expect("a listing")[&kind].1,
            in_stock - 1,
            "{seat} sees the stock of what the visitor bought exactly one lower"
        );
        assert!(
            shelf.entity(visitor).is_some(),
            "{seat} perceives the visitor"
        );
        assert_eq!(
            private_of(&shelf, visitor),
            Vec::<String>::new(),
            "{seat} is disclosed none of the visitor's own state (INV-13)"
        );
    }

    // ── One world: every revision two of them observe, they observe alike. ───────────────────
    let standing = players[0].2.observation().await;
    let here = *standing
        .self_location()
        .expect("the visitor knows where it is");
    let place = here.place().entity_id();
    let local = here.local().expect("a position");
    let (x, y) = (local.x().value(), local.y().value());
    let pacing = [
        support::stride(visitor, place, x + SHUFFLE_MM, y),
        support::stride(visitor, place, x, y),
    ];
    let recorded: Vec<Vec<(u64, WireObservation)>> = join_all(players.iter_mut().enumerate().map(
        |(index, (_, _, client))| record(client, WATCH, (index == 0).then(|| pacing.clone())),
    ))
    .await;
    let mut at: BTreeMap<u64, Vec<(usize, &WireObservation)>> = BTreeMap::new();
    for (index, frames) in recorded.iter().enumerate() {
        for (revision, observation) in frames {
            at.entry(*revision).or_default().push((index, observation));
        }
    }
    let shared: Vec<u64> = at
        .iter()
        .filter(|(revision, seen)| {
            **revision >= bought_at
                && seen
                    .iter()
                    .map(|(index, _)| *index)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == PLAYERS.len()
        })
        .map(|(revision, _)| *revision)
        .collect();
    assert!(
        !shared.is_empty(),
        "at least one revision at or after the purchase ({bought_at}) is observed by all four: {:?}",
        at.keys().collect::<Vec<_>>()
    );
    let mut compared = 0;
    for (revision, seen) in &at {
        let observers: BTreeSet<usize> = seen.iter().map(|(index, _)| *index).collect();
        if observers.len() < 2 {
            continue;
        }
        let everyone: BTreeSet<EntityId> = seen
            .iter()
            .map(|(_, observation)| {
                observation
                    .entities()
                    .iter()
                    .map(mineworld_contracts::PerceivedEntity::id)
                    .collect::<BTreeSet<_>>()
            })
            .reduce(|left, right| &left & &right)
            .unwrap_or_default();
        assert!(
            everyone.contains(&alice),
            "at revision {revision} every player perceives Alice"
        );
        for entity in &everyone {
            let places: BTreeSet<String> = seen
                .iter()
                .map(|(_, observation)| {
                    format!("{:?}", observation.entity(*entity).map(|it| it.location()))
                })
                .collect();
            assert_eq!(
                places.len(),
                1,
                "at revision {revision}, {} stands in one place for every player who sees it: \
                 {places:?}",
                town.key(*entity)
            );
            compared += 1;
        }
        let shelves: BTreeSet<String> = seen
            .iter()
            .map(|(_, observation)| format!("{:?}", listing_in(observation)))
            .collect();
        assert_eq!(
            shelves.len(),
            1,
            "at revision {revision}, one shelf: {shelves:?}"
        );
    }
    // Not vacuous: across the revisions all four observed, somebody they all perceive stood in two
    // different places — so the agreement above was over state that changed.
    let location_at = |revision: u64, entity: EntityId| {
        at[&revision]
            .first()
            .and_then(|(_, observation)| observation.entity(entity))
            .map(|it| format!("{:?}", it.location()))
    };
    let moved: Vec<String> = town
        .people
        .iter()
        .filter(|entity| {
            shared
                .iter()
                .filter_map(|revision| location_at(*revision, **entity))
                .collect::<BTreeSet<_>>()
                .len()
                >= 2
        })
        .map(|entity| town.key(*entity))
        .collect();
    assert!(
        !moved.is_empty(),
        "somebody perceived moved between two of the shared revisions {shared:?}, \
         so the comparison was over changing state"
    );
    println!(
        "one world: {} revisions observed by all four after the purchase ({shared:?}); {compared} \
         (revision, person) locations compared; moved across them: {moved:?}",
        shared.len()
    );

    // ── Alice heard all four: read from her own seat, taken from her agent. ──────────────────
    let mut as_alice = Client::connect(server.address).await;
    match as_alice.join_as(join_frame("alice")).await {
        ServerFrame::Welcome { took_over, .. } => {
            assert_eq!(took_over, TookOver::Hosted, "her agent was driving her");
        }
        other => panic!("a welcome was expected, and the server said {other:?}"),
    }
    let hers = as_alice
        .observation_where("alice's own history", |seen| own_history(seen).is_some())
        .await;
    let history = own_history(&hers).expect("found above");
    for (seat, nickname) in PLAYERS {
        let said = line(nickname);
        assert!(
            history.heard().iter().any(|heard| {
                heard.speaker().entity_id() == town.id(seat) && heard.utterance().as_str() == said
            }),
            "Alice's own history holds {seat}'s line {said:?}: {:?}",
            history.heard()
        );
    }

    // ── And it is one persisted world. ───────────────────────────────────────────────────────
    drop(players);
    drop(as_alice);
    server.kill();
    let (replayed, printed) = run_command(&["replay", MARKET_TOWN, "--save", save.path()]);
    assert!(replayed.success(), "the save verifies: {printed}");
    println!("AC-7: one instance {instance}; four players; the save replays");
}
