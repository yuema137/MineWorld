//! E-8 through the unchanged controller (§4.5 E-C5's 10-day paced run): the paced controller eats and
//! drinks, although it was never compiled against consumption (`ARC-34`).
//!
//! The café of `support` is driven exactly as `mineworld run` drives a World Pack (`ARC-27`): seat *k*
//! is consulted at `genesis + k + m·P`, with P = 900 s.
//!
//! Criterion, stated before running: over 10 days at least two people eat or drink; every meal the
//! controller asked for is accepted and is one `items-consumed` caused by it; goods are never
//! consumed; two runs of one seed are byte-identical.

mod support;

use std::collections::BTreeSet;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionResult, Causation, EntityId, Event, EventEnvelope, SimDuration,
    WorldTime,
};
use mineworld_inventory::ItemsConsumed;
use mineworld_presence::observe;
use mineworld_rule_controller::PacedRuleController;
use support::{Town, providers};

const PACE: i64 = 900;
const DAY: i64 = 86_400;
const DAYS: i64 = 10;
const SEED: u64 = 7;
const CARRIED: [(&str, &str); 2] = [
    ("alice", "{ coffee: 3, croissant: 2, mug: 1 }"),
    ("bob", "{ tea: 3, croissant: 3 }"),
];

struct Run {
    facts: Vec<u8>,
    meals: Vec<(ActionId, EntityId, bool)>,
    recorded: Vec<EventEnvelope>,
    town: Town,
}

fn run() -> Run {
    let mut town = Town::begun(true, &CARRIED);
    let mut facts = Vec::new();
    let mut recorded = town.genesis.clone();
    for fact in &town.genesis {
        facts.extend(serde_json::to_vec(fact).expect("encodes"));
        facts.push(b'\n');
    }
    let seats: Vec<EntityId> = ["alice", "bob", "carol"]
        .iter()
        .map(|name| town.id(name))
        .collect();
    let controller = PacedRuleController::new(SEED, SimDuration::from_seconds(PACE));
    let providers = providers();
    let mut next_action = 1;
    let mut meals = Vec::new();
    'rounds: for round in 0.. {
        for (k, seat) in seats.iter().enumerate() {
            let at = i64::try_from(k).expect("small") + round * PACE;
            if at >= DAYS * DAY {
                break 'rounds;
            }
            let now = WorldTime::from_seconds(at);
            let mut step = town.world.advance_to(now).expect("advance").into_events();
            let seen = observe(&town.world, *seat, now, &providers);
            if let Some(request) = controller.decide(&seen) {
                let is_meal = matches!(request.action_type().as_str(), "eat" | "drink");
                let id = ActionId::from_raw(next_action);
                next_action += 1;
                let intent = ActionIntent::allocate(request, id, now);
                let dispatched = town.world.dispatch(&intent, now).expect("dispatch answers");
                if is_meal {
                    let accepted = matches!(dispatched.result(), ActionResult::Accepted { .. });
                    meals.push((id, *seat, accepted));
                }
                step.extend(dispatched.events().iter().cloned());
            }
            for fact in step {
                facts.extend(serde_json::to_vec(&fact).expect("encodes"));
                facts.push(b'\n');
                recorded.push(fact);
            }
        }
    }
    Run {
        facts,
        meals,
        recorded,
        town,
    }
}

#[test]
fn the_unchanged_paced_controller_eats_and_drinks_what_it_carries() {
    let run = run();
    let eaters: BTreeSet<EntityId> = run.meals.iter().map(|(_, who, _)| *who).collect();
    println!(
        "{} meals by {} people; alice now {:?}, bob {:?}",
        run.meals.len(),
        eaters.len(),
        run.town.holdings("alice"),
        run.town.holdings("bob")
    );
    assert!(
        eaters.len() >= 2,
        "at least two people eat or drink: {eaters:?}"
    );
    assert!(
        run.meals.iter().all(|(.., accepted)| *accepted),
        "every meal the controller asked for was an available complete offer, and was accepted"
    );
    for (id, who, _) in &run.meals {
        let caused: Vec<&EventEnvelope> = run
            .recorded
            .iter()
            .filter(|fact| *fact.caused_by() == Causation::Action(*id))
            .collect();
        assert_eq!(caused.len(), 1, "one fact per meal");
        assert_eq!(*caused[0].event_type(), ItemsConsumed::EVENT_TYPE);
        assert_eq!(caused[0].participants(), &[*who], "eaten by whoever asked");
    }
    assert_eq!(
        run.town.holdings("alice"),
        [("mug".to_owned(), 1)],
        "everything edible or drinkable is gone; the mug is never consumed"
    );
}

#[test]
fn two_runs_of_one_seed_are_byte_identical() {
    let (first, second) = (run(), run());
    assert!(
        !first.meals.is_empty(),
        "the comparison is of runs that ate"
    );
    assert!(first.facts == second.facts, "the two runs' facts differ");
}
