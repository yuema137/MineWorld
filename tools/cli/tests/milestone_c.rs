//! **Milestone C** — *work → earn → buy → holdings change → another client perceives it → it survives
//! a server restart* (`docs/HUMAN_REVIEW_QUEUE.md`; step-10 §4.6 SD-36, P-8, CP-7).
//!
//! Real processes, S5's real persistence and real WebSocket clients throughout.
//!
//! ```text
//! worked    mineworld run worlds/market-town --headless --seed 7 --days 2 --save K
//! located   in K: alice hired; a shift-started, her arrival at the café during it, its shift-ended
//!           with worked > 0; a wage-due for her, and the money-transferred to her caused by it
//! hosted    mineworld server worlds/market-town --save K: clients alice and bob walk from where the
//!           save left them, through the pack's doorways, into the café (alice's employer's shop),
//!           every stride accepted; alice's own wallet = the replayed balance at K's head
//! bought    alice submits one available complete `buy` affordance unchanged: accepted, two events
//! changed   alice's wallet is lower by the listed price, her holdings higher by one of that kind;
//!           bob's next observation shows that kind's in_stock one lower, and discloses neither
//!           alice's wallet nor her holdings
//! survived  the server is SIGKILLed and started again on K: the same instance, the same revision,
//!           alice's wallet and holdings and bob's listing unchanged
//! inspect   mineworld inspect K: every cause resolves
//! ```
//!
//! The test knows no market action's shape: the buy is the affordance the world offered, submitted
//! unchanged (`server/PROTOCOL.md` §6, `ARC-34`); what it bought is read from what changed.

mod fixture;
mod headless;
mod market;
mod support;

use std::collections::BTreeMap;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;

use headless::{Tables, fresh, mineworld, stderr, stdout};
use market::{Ledger, MARKET_TOWN, MarketFact, Town, market_fact, run_market};
use mineworld_contracts::{Causation, EntityId, EventEnvelope, PersonId};
use mineworld_server::WireObservation;
use serde_json::{Value, json};
use support::{Client, Server, stride, walk};

/// The café: the shop alice works for, which keeps stock through the day (step-10 F-65).
const SHOP: &str = "cafe";
/// The place every doorway opens onto.
const STREET: &str = "street";

/// The located work and pay: the facts that show alice worked and was paid in the saved run.
fn locate_work_and_pay(town: &Town, facts: &[EventEnvelope], alice: PersonId) {
    let ids: BTreeMap<u64, &EventEnvelope> = facts.iter().map(|f| (f.id().raw(), f)).collect();
    let hired = facts
        .iter()
        .find(|fact| {
            matches!(market_fact(fact), Some(MarketFact::Hired { employee, .. }) if employee == alice)
        })
        .expect("alice is hired");
    let payload = |fact: &EventEnvelope| -> Value {
        serde_json::from_slice(fact.payload().payload()).expect("JSON")
    };
    let is_alice = |value: &Value| {
        serde_json::from_value::<PersonId>(value["employee"].clone()).ok() == Some(alice)
    };
    let worked = facts
        .iter()
        .find(|fact| {
            fact.event_type().as_str() == "shift-ended"
                && is_alice(&payload(fact))
                && payload(fact)["worked"]
                    .as_u64()
                    .is_some_and(|worked| worked > 0)
        })
        .expect("alice worked a shift");
    let started = facts
        .iter()
        .rev()
        .find(|fact| {
            fact.event_type().as_str() == "shift-started"
                && is_alice(&payload(fact))
                && fact.at() < worked.at()
        })
        .expect("the shift she worked began");
    // Work is attendance (ARC-38 item 2). Her routine and her shift both begin at 05:30, so she is
    // still on her way when the shift starts (`present: false`, step-10 §4.6.8 DP-5): her presence
    // is located as her arrival at the café during the shift, the fact employment reacts to.
    let cafe = town.id(SHOP);
    let arrived = facts
        .iter()
        .find(|fact| {
            fact.at() >= started.at()
                && fact.at() <= worked.at()
                && social_entry(fact)
                    .is_some_and(|(person, place)| person == alice.entity_id() && place == cafe)
        })
        .expect("alice entered the café during the shift she worked");
    let (due, paid) = facts
        .iter()
        .find_map(|fact| match (market_fact(fact), fact.caused_by()) {
            (Some(MarketFact::MoneyTransferred { to, .. }), Causation::Event(cause))
                if to == alice.entity_id() =>
            {
                let due = ids.get(&cause.raw())?;
                matches!(market_fact(due), Some(MarketFact::WageDue { employee, .. }) if employee == alice)
                    .then_some((*due, fact))
            }
            _ => None,
        })
        .expect("a wage-due for alice, and the money-transferred to her caused by it");
    for (what, fact) in [
        ("hired", hired),
        ("shift started", started),
        ("arrived at work", arrived),
        ("worked", worked),
        ("wage-due", due),
        ("paid", paid),
    ] {
        eprintln!(
            "located {what}: #{} {} at t{} (day {}) for {}",
            fact.id().raw(),
            fact.event_type().as_str(),
            fact.at().seconds(),
            fact.at().seconds() / 86_400 + 1,
            town.key(alice.entity_id())
        );
    }
}

/// A `person-entered-place` as (person, place), decoded with presence's own type.
fn social_entry(fact: &EventEnvelope) -> Option<(EntityId, EntityId)> {
    use mineworld_presence::PersonEnteredPlace;
    let bytes = fact.payload().payload_for::<PersonEnteredPlace>().ok()?;
    let entry: PersonEnteredPlace = serde_json::from_slice(bytes).ok()?;
    Some((entry.person().entity_id(), entry.place().entity_id()))
}

/// The observer's own disclosed component of this type, as JSON.
fn own(observation: &WireObservation, component: &str) -> Option<Value> {
    observation
        .entity(observation.observer())?
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == component)
        .map(|record| record.payload().clone())
}

/// The shop listing this observation discloses, if any: per item, (price, in stock).
fn listing_in(observation: &WireObservation) -> Option<BTreeMap<EntityId, (u64, u32)>> {
    observation.entities().iter().find_map(|entity| {
        entity
            .components()
            .iter()
            .find(|record| record.component_type().as_str() == "shop")
            .map(|record| market::listing(record.payload()).1)
    })
}

/// Whether this observation discloses `holder`'s wallet or holdings to its observer.
fn discloses_private(observation: &WireObservation, holder: EntityId) -> Vec<String> {
    observation
        .entity(holder)
        .map(|entity| {
            entity
                .components()
                .iter()
                .map(|record| record.component_type().as_str().to_owned())
                .filter(|kind| kind == "wallet" || kind == "holdings")
                .collect()
        })
        .unwrap_or_default()
}

/// Walks `client` from where it stands into the café, through the pack's doorways, every stride
/// accepted. The doorways are read from the pack (`WorldPack::read`), the start from the observer's
/// own position — no coordinate is copied into the test.
async fn walk_into_the_cafe(client: &mut Client, town: &Town, seen: &WireObservation) {
    let me = client.observer.expect("seated");
    let pack = mineworld_worldpack::WorldPack::read(MARKET_TOWN).expect("reads");
    let door = |place: &str| {
        let passage = pack.places()[&market_key(place)]
            .passages
            .iter()
            .find(|passage| passage.to.as_str() == STREET)
            .unwrap_or_else(|| panic!("{place} opens onto the street"));
        let point = |position: Option<mineworld_worldpack::AuthoredPosition>| {
            let position = position.expect("a positioned doorway");
            (position.x.value(), position.y.value())
        };
        (point(passage.here), point(passage.there))
    };
    let location = seen
        .self_location()
        .expect("the observer knows where it is");
    let place = town.key(location.place().entity_id());
    let local = location.local().expect("a position");
    let mut at = (local.x().value(), local.y().value());
    let start = at;
    let mut strides = 0;
    if place != SHOP {
        if place != STREET {
            let (inside, outside) = door(&place);
            strides += client
                .walk_accepted(walk(me, town.id(&place), at, inside))
                .await
                .len();
            client
                .submit_accepted(stride(me, town.id(STREET), outside.0, outside.1))
                .await;
            strides += 1;
            at = outside;
        }
        let (inside, outside) = door(SHOP);
        strides += client
            .walk_accepted(walk(me, town.id(STREET), at, outside))
            .await
            .len();
        client
            .submit_accepted(stride(me, town.id(SHOP), inside.0, inside.1))
            .await;
        strides += 1;
    }
    eprintln!(
        "{} walked from {place} {start:?} into the café in {strides} accepted strides",
        town.key(me)
    );
}

fn market_key(key: &str) -> mineworld_contracts::EntityKey {
    mineworld_contracts::EntityKey::new(key).expect("a key")
}

/// Seats `seat` on the server and returns the client with its first observation and revision.
async fn seated(server: &Server, seat: &str) -> (Client, WireObservation, Option<u64>) {
    let mut client = Client::connect(server.address).await;
    client.join(seat).await;
    let (revision, observation) = client.perceived().await;
    (client, observation, revision.map(|revision| revision.raw()))
}

/// The next observation of `client` that is in the café, with the listing.
async fn in_the_cafe(client: &mut Client, who: &str) -> WireObservation {
    client
        .observation_where(&format!("{who} perceiving the café's listing"), |seen| {
            listing_in(seen).is_some()
        })
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn alice_works_earns_buys_and_bob_sees_the_shelf_change_across_a_restart() {
    let town = Town::read(Path::new(MARKET_TOWN));
    let (alice, bob) = (town.person("alice"), town.person("bob"));

    // ── Work and pay, in a saved 2-day run. ──────────────────────────────────────────────────────
    let save = fresh("milestone-c");
    run_market(7, 2, Some(&save));
    let facts: Vec<EventEnvelope> = Tables::read(&save)
        .facts
        .iter()
        .map(|(_, bytes)| mineworld_persistence::format::decode(bytes, "fact").expect("a fact"))
        .collect();
    locate_work_and_pay(&town, &facts, alice);
    let mut ledger = Ledger::default();
    for fact in facts.iter().filter_map(market_fact) {
        ledger.apply(&fact).expect("replays");
    }

    // ── Hosted on the save. ──────────────────────────────────────────────────────────────────────
    // "The same revision after the restart" holds only while no routine or shift boundary falls as
    // the hosted clock runs on from the head: checked first, by name.
    let head_at = Tables::read(&save).head_instant();
    fixture::assert_quiet_in(
        &Path::new(MARKET_TOWN).join("people"),
        head_at,
        3_600,
        "milestone_c.rs (hosting the 2-day save)",
    );
    let saved = save.to_str().expect("path");
    let command = ["server", MARKET_TOWN, "--save", saved];
    let mut first = Server::start(&command).await;
    let instance = first.status().await["instance"].clone();
    let (mut alice_client, alice_seen, _) = seated(&first, "alice").await;
    let (mut bob_client, bob_seen, _) = seated(&first, "bob").await;
    walk_into_the_cafe(&mut alice_client, &town, &alice_seen).await;
    walk_into_the_cafe(&mut bob_client, &town, &bob_seen).await;

    let mut alice_now = in_the_cafe(&mut alice_client, "alice").await;
    let wallet = market::wallet_balance(&own(&alice_now, "wallet").expect("alice's own wallet"));
    assert_eq!(
        Some(&wallet),
        ledger.wallets.get(&alice.entity_id()),
        "alice's disclosed wallet is the replayed balance at the head, her wages included"
    );
    let held = |seen: &WireObservation| {
        own(seen, "holdings").map_or_else(BTreeMap::new, |value| market::holdings(&value))
    };
    if held(&alice_now).values().sum::<u32>() >= market::PERSON_CAPACITY {
        // A buyer carrying six is offered buy unavailable (ARC-38 item 3): free one place first,
        // through a meal the world offers (F-66).
        let meal = alice_now
            .affordances()
            .iter()
            .find(|offer| {
                matches!(offer.action_type().as_str(), "eat" | "drink")
                    && offer.is_available()
                    && offer.payload().is_some()
            })
            .cloned()
            .expect("alice, carrying six, is offered something to eat or drink");
        alice_client
            .submit_accepted(unchanged(alice.entity_id(), &meal))
            .await;
        alice_now = in_the_cafe(&mut alice_client, "alice").await;
        eprintln!(
            "alice was carrying six: she had a {} first",
            meal.action_type().as_str()
        );
    }
    let wallet_before = market::wallet_balance(&own(&alice_now, "wallet").expect("wallet"));
    let held_before = held(&alice_now);
    let shelf_before = listing_in(&in_the_cafe(&mut bob_client, "bob").await).expect("a listing");

    // ── The buy: an offered, available, complete affordance, submitted unchanged. ────────────────
    let buy = alice_now
        .affordances()
        .iter()
        .find(|offer| {
            offer.action_type().as_str() == "buy"
                && offer.is_available()
                && offer.payload().is_some()
        })
        .cloned()
        .expect("alice in the café is offered an available complete buy");
    let (bought, events) = alice_client
        .submit_accepted(unchanged(alice.entity_id(), &buy))
        .await;
    assert_eq!(
        events.len(),
        2,
        "a purchase records the payment and the transfer"
    );

    let alice_after = alice_client
        .observation_where("alice's holdings changed", |seen| held(seen) != held_before)
        .await;
    let wallet_after = market::wallet_balance(&own(&alice_after, "wallet").expect("wallet"));
    let held_after = held(&alice_after);
    let gained: Vec<(EntityId, i64)> = held_after
        .keys()
        .chain(held_before.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|item| {
            let delta = i64::from(held_after.get(item).copied().unwrap_or(0))
                - i64::from(held_before.get(item).copied().unwrap_or(0));
            (*item, delta)
        })
        .filter(|(_, delta)| *delta != 0)
        .collect();
    let [(kind, 1)] = gained[..] else {
        panic!("alice's holdings changed by exactly one of one kind: {gained:?}");
    };
    let (price, in_stock) = shelf_before[&kind];
    assert_eq!(
        wallet_before - wallet_after,
        price,
        "alice paid the listed price of what she bought"
    );

    // ── Another client perceives it, and is told nothing private. ────────────────────────────────
    let bob_after = bob_client
        .observation_where("the café's stock of what alice bought fell", |seen| {
            listing_in(seen).is_some_and(|shelf| shelf[&kind].1 != in_stock)
        })
        .await;
    let shelf_after = listing_in(&bob_after).expect("a listing");
    assert_eq!(
        shelf_after[&kind],
        (price, in_stock - 1),
        "bob sees that kind's stock one lower"
    );
    assert!(
        discloses_private(&bob_after, alice.entity_id()).is_empty()
            && bob_after.entity(alice.entity_id()).is_some(),
        "bob perceives alice and is disclosed neither her wallet nor her holdings (INV-13): {:?}",
        discloses_private(&bob_after, alice.entity_id())
    );
    let revision = first.status().await["revision"].clone();
    eprintln!(
        "alice bought item #{} for {price} (action {}): wallet {wallet_before} → {wallet_after}, \
         shelf {in_stock} → {}; revision {revision}",
        kind.raw(),
        bought.raw(),
        in_stock - 1
    );

    // ── SIGKILL, and the same command again. ─────────────────────────────────────────────────────
    drop((alice_client, bob_client));
    let died = first.kill();
    assert_eq!(
        died.signal(),
        Some(9),
        "the server died of SIGKILL: {died:?}"
    );
    let second = Server::start(&command).await;
    assert_eq!(
        second.status().await["instance"],
        instance,
        "the same world"
    );
    let (_alice_again, alice_restarted, alice_revision) = seated(&second, "alice").await;
    let (_bob_again, bob_restarted, bob_revision) = seated(&second, "bob").await;
    for seen in [alice_revision, bob_revision] {
        assert_eq!(
            seen.map(|raw| json!(raw)),
            Some(revision.clone()),
            "at the revision it was killed at"
        );
    }
    assert_eq!(
        own(&alice_restarted, "wallet").map(|value| market::wallet_balance(&value)),
        Some(wallet_after),
        "alice's wallet survived the kill"
    );
    assert_eq!(held(&alice_restarted), held_after, "and her holdings");
    assert_eq!(
        listing_in(&bob_restarted),
        Some(shelf_after),
        "and bob, still in the café, sees the shelf as it was"
    );
    drop(second);

    // ── Every cause resolves. ────────────────────────────────────────────────────────────────────
    let inspected = mineworld(&["inspect", saved, "--last", "0"]);
    assert!(inspected.status.success(), "{}", stderr(&inspected));
    let report = stdout(&inspected);
    assert!(
        report.contains("AC-9       every cause resolves"),
        "{report}"
    );
    let _ = bob;
}

/// A complete affordance as the request it names, unchanged (`server/PROTOCOL.md` §6).
fn unchanged(actor: EntityId, offer: &mineworld_contracts::Affordance<Value>) -> Value {
    json!({
        "actor": actor,
        "action_type": offer.action_type().as_str(),
        "target": offer.target(),
        "payload": { "action_type": offer.action_type().as_str(), "payload": offer.payload() },
        "actor_location": null,
    })
}
