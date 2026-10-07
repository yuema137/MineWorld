//! E-5, CP-3 for buy: the **unchanged** paced controller buys, although it was never compiled against
//! economy (`ARC-34`).
//!
//! The store of `support`, stocked for a run, is driven exactly as `mineworld run` drives a World Pack
//! (`ARC-27`): seat *k* is consulted at `genesis + k + m·P`, with P = 900 s; at each consult the world
//! advances, presence's real `observe` builds the observation from every provider,
//! `PacedRuleController::decide` decides, the request is allocated an identity and an instant, and the
//! kernel dispatches it.
//!
//! Criterion, stated before running (step-10 §4.5.3 E-5): over 10 days people buy — at least two
//! different people, every buy the controller asked for accepted, each one paid by exactly one
//! `money-transferred` and handed over by exactly one `items-transferred`, both caused by the buy that
//! asked (AC-9) — nobody passes six; two runs of one seed are byte-identical; the controller's
//! manifest names no market pack.

mod support;

use std::collections::BTreeSet;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionResult, ActionTypeId, Causation, EntityId, Event,
    EventEnvelope, SimDuration, WorldTime,
};
use mineworld_economy::{Buy, MoneyTransferred};
use mineworld_inventory::{Holdings, ItemsTransferred, PERSON_CAPACITY};
use mineworld_presence::observe;
use mineworld_rule_controller::PacedRuleController;
use support::{SHOP, Town, providers};

/// `mineworld run`'s pace (`tools/cli/src/run.rs` `PACE`).
const PACE: i64 = 900;
const DAY: i64 = 86_400;
const DAYS: i64 = 10;
const SEED: u64 = 7;
const STOCKED: &str = "{ apple: 40, bread: 40 }";

struct Run {
    facts: Vec<u8>,
    requests: Vec<(ActionId, EntityId, ActionTypeId, bool)>,
    recorded: Vec<EventEnvelope>,
    town: Town,
}

fn run() -> Run {
    let mut town = Town::begun_plus(SHOP, STOCKED, |_| {});
    let mut facts = Vec::new();
    let mut recorded = town.genesis.clone();
    for fact in &town.genesis {
        facts.extend(serde_json::to_vec(fact).expect("a fact encodes"));
        facts.push(b'\n');
    }
    let seats: Vec<EntityId> = ["alice", "bob", "carol"]
        .iter()
        .map(|name| town.id(name))
        .collect();
    let controller = PacedRuleController::new(SEED, SimDuration::from_seconds(PACE));
    let providers = providers();
    let end = DAYS * DAY;
    let mut next_action = 1;
    let mut requests = Vec::new();
    'rounds: for round in 0.. {
        for (k, seat) in seats.iter().enumerate() {
            let at = i64::try_from(k).expect("small") + round * PACE;
            if at >= end {
                break 'rounds;
            }
            let now = WorldTime::from_seconds(at);
            let mut step = town.world.advance_to(now).expect("advance").into_events();
            let seen = observe(&town.world, *seat, now, &providers);
            if let Some(request) = controller.decide(&seen) {
                let action_type = request.action_type().clone();
                let id = ActionId::from_raw(next_action);
                next_action += 1;
                let intent = ActionIntent::allocate(request, id, now);
                let dispatched = town.world.dispatch(&intent, now).expect("dispatch answers");
                let accepted = matches!(dispatched.result(), ActionResult::Accepted { .. });
                requests.push((id, *seat, action_type, accepted));
                step.extend(dispatched.events().iter().cloned());
            }
            for fact in step {
                facts.extend(serde_json::to_vec(&fact).expect("a fact encodes"));
                facts.push(b'\n');
                recorded.push(fact);
            }
        }
    }
    Run {
        facts,
        requests,
        recorded,
        town,
    }
}

#[test]
fn the_unchanged_paced_controller_buys_what_it_is_offered() {
    let run = run();
    let bought: Vec<_> = run
        .requests
        .iter()
        .filter(|(_, _, action_type, _)| *action_type == Buy::ACTION_TYPE)
        .collect();
    let buyers: BTreeSet<EntityId> = bought.iter().map(|(_, who, ..)| *who).collect();
    println!(
        "{} requests, {} buys by {} buyers",
        run.requests.len(),
        bought.len(),
        buyers.len()
    );
    assert!(
        buyers.len() >= 2,
        "at least two people buy: {buyers:?} ({} buys)",
        bought.len()
    );
    assert!(
        bought.iter().all(|(.., accepted)| *accepted),
        "every buy the controller asked for was an available complete offer, and was accepted"
    );
    for (id, buyer, ..) in &bought {
        let caused: Vec<&EventEnvelope> = run
            .recorded
            .iter()
            .filter(|fact| *fact.caused_by() == Causation::Action(*id))
            .collect();
        let types: Vec<&str> = caused
            .iter()
            .map(|fact| fact.event_type().as_str())
            .collect();
        assert_eq!(
            types,
            [
                MoneyTransferred::EVENT_TYPE.as_str(),
                ItemsTransferred::EVENT_TYPE.as_str()
            ],
            "one payment and one hand-over, caused by the buy that asked (AC-9)"
        );
        let paid: MoneyTransferred = serde_json::from_slice(
            caused[0]
                .payload()
                .payload_for::<MoneyTransferred>()
                .expect("economy's fact"),
        )
        .expect("decodes");
        assert_eq!(paid.from(), *buyer, "paid by whoever asked");
    }
    let read = run.town.world.read();
    for name in ["alice", "bob", "carol"] {
        let held = read
            .component::<Holdings>(run.town.id(name))
            .map_or(0, Holdings::total);
        assert!(held <= u64::from(PERSON_CAPACITY), "{name} holds {held}");
    }
}

#[test]
fn two_runs_of_one_seed_are_byte_identical() {
    let (first, second) = (run(), run());
    assert!(
        first
            .requests
            .iter()
            .any(|(_, _, action_type, accepted)| *action_type == Buy::ACTION_TYPE && *accepted),
        "the comparison is of runs that bought"
    );
    assert!(first.facts == second.facts, "the two runs' facts differ");
}

/// The controller's manifest names no market pack: it was never compiled against the pack it buys
/// through.
#[test]
fn the_controller_was_never_compiled_against_a_market_pack() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cognition/rule-controller/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("the controller's manifest is readable");
    assert!(
        text.contains("mineworld-contracts"),
        "this is the controller's manifest"
    );
    for pack in [
        "mineworld-item",
        "mineworld-inventory",
        "mineworld-item-transfer",
        "mineworld-economy",
        "mineworld-employment",
        "mineworld-consumption",
    ] {
        assert!(!text.contains(pack), "{pack} in {text}");
    }
}
