//! D-6, CP-3 for a real pack: the **unchanged** paced controller gives, although it was never compiled
//! against item-transfer (`ARC-34`).
//!
//! The café of `support` is driven exactly as `mineworld run` drives a World Pack (`ARC-27`): seat *k*
//! is consulted at `genesis + k + m·P`, with P = 900 s; at each consult the world advances, presence's
//! real `observe` builds the observation from every provider, `PacedRuleController::decide` decides,
//! the request is allocated an identity and an instant, and the kernel dispatches it.
//!
//! Criterion, stated before running (step-10 §4.4.3 D-6): over 10 days people give — at least two
//! different people, every give the controller asked for accepted, every `items-transferred` caused by
//! the give that asked for it — holdings are conserved and nobody passes six; two runs of one seed are
//! byte-identical; the controller's manifest names no market pack.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::Action;
use mineworld_contracts::{
    ActionId, ActionIntent, ActionResult, ActionTypeId, Causation, EntityId, Event, SimDuration,
    WorldTime,
};
use mineworld_inventory::{Holdings, ItemsTransferred, PERSON_CAPACITY};
use mineworld_item_transfer::Give;
use mineworld_presence::observe;
use mineworld_rule_controller::PacedRuleController;
use support::{HOLDINGS, Town, providers};

/// `mineworld run`'s pace (`tools/cli/src/run.rs` `PACE`).
const PACE: i64 = 900;
const DAY: i64 = 86_400;
const DAYS: i64 = 10;
const SEED: u64 = 7;

struct Run {
    /// Every fact, encoded, in the order recorded — the bytes two runs are compared by.
    facts: Vec<u8>,
    /// Every request: (action id, actor, action type, payload as JSON, accepted).
    requests: Vec<(ActionId, EntityId, ActionTypeId, serde_json::Value, bool)>,
    /// Every `items-transferred` recorded.
    moved: Vec<mineworld_contracts::EventEnvelope>,
    town: Town,
}

fn encode(fact: &mineworld_contracts::EventEnvelope, into: &mut Vec<u8>) {
    into.extend(serde_json::to_vec(fact).expect("a fact encodes"));
    into.push(b'\n');
}

fn run() -> Run {
    let mut town = Town::begun(true, &HOLDINGS);
    let mut facts = Vec::new();
    for fact in &town.genesis {
        encode(fact, &mut facts);
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
    let mut moved = Vec::new();
    'rounds: for round in 0.. {
        for (k, seat) in seats.iter().enumerate() {
            let at = i64::try_from(k).expect("small") + round * PACE;
            if at >= end {
                break 'rounds;
            }
            let now = WorldTime::from_seconds(at);
            for fact in town.world.advance_to(now).expect("advance").into_events() {
                encode(&fact, &mut facts);
            }
            let seen = observe(&town.world, *seat, now, &providers);
            let Some(request) = controller.decide(&seen) else {
                continue;
            };
            let action_type = request.action_type().clone();
            let payload: serde_json::Value = serde_json::from_slice(request.payload().payload())
                .expect("the controller encodes JSON");
            let id = ActionId::from_raw(next_action);
            next_action += 1;
            let intent = ActionIntent::allocate(request, id, now);
            let dispatched = town.world.dispatch(&intent, now).expect("dispatch answers");
            let accepted = matches!(dispatched.result(), ActionResult::Accepted { .. });
            requests.push((id, *seat, action_type, payload, accepted));
            for fact in dispatched.events() {
                encode(fact, &mut facts);
                if *fact.event_type() == ItemsTransferred::EVENT_TYPE {
                    moved.push(fact.clone());
                }
            }
        }
    }
    Run {
        facts,
        requests,
        moved,
        town,
    }
}

#[test]
fn the_unchanged_paced_controller_gives_what_it_is_offered() {
    let run = run();
    let gives: Vec<_> = run
        .requests
        .iter()
        .filter(|(_, _, action_type, ..)| *action_type == Give::ACTION_TYPE)
        .collect();
    let givers: BTreeSet<EntityId> = gives.iter().map(|(_, who, ..)| *who).collect();
    let mut per_day: BTreeMap<i64, u64> = BTreeMap::new();
    for fact in &run.moved {
        *per_day.entry(fact.at().seconds() / DAY).or_default() += 1;
    }
    println!(
        "{} requests, {} gives by {} givers, accepted transfers per day {per_day:?}",
        run.requests.len(),
        gives.len(),
        givers.len()
    );
    assert!(
        givers.len() >= 2,
        "at least two people give: {givers:?} ({} gives)",
        gives.len()
    );
    assert!(
        gives.iter().all(|(.., accepted)| *accepted),
        "every give the controller asked for was an available complete offer, and was accepted"
    );
    assert_eq!(gives.len(), run.moved.len(), "one transfer per give");

    for fact in &run.moved {
        let moved: ItemsTransferred = serde_json::from_slice(
            fact.payload()
                .payload_for::<ItemsTransferred>()
                .expect("inventory's fact"),
        )
        .expect("decodes");
        let Causation::Action(cause) = fact.caused_by() else {
            panic!("a transfer caused by something other than a request: {fact:?}");
        };
        let asked = run
            .requests
            .iter()
            .find(|(id, ..)| id == cause)
            .expect("the cause is a request this run made");
        assert_eq!(asked.1, moved.from(), "given by whoever asked (AC-9)");
        assert_eq!(asked.2, Give::ACTION_TYPE);
        let give: Give = serde_json::from_value(asked.3.clone()).expect("a give");
        assert_eq!((give.item(), give.count()), (moved.item(), moved.count()));
    }

    let read = run.town.world.read();
    let mut total = 0;
    for name in ["alice", "bob", "carol"] {
        let held = read
            .component::<Holdings>(run.town.id(name))
            .map_or(0, Holdings::total);
        assert!(held <= u64::from(PERSON_CAPACITY), "{name} holds {held}");
        total += held;
    }
    assert_eq!(
        total, 3,
        "a give conserves items: the town began with three (HOLDINGS: a coffee, a tea, an apple)"
    );
}

#[test]
fn two_runs_of_one_seed_are_byte_identical() {
    let (first, second) = (run(), run());
    assert!(
        !first.moved.is_empty(),
        "the comparison is of runs that gave"
    );
    assert!(first.facts == second.facts, "the two runs' facts differ");
}

/// The controller's manifest names no market pack: it was never compiled against the pack it gives
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
    ] {
        assert!(!text.contains(pack), "{pack} in {text}");
    }
}
