//! D-7, `AC-2` for item-transfer at pack level: a world without it still loads and runs, nobody is
//! offered a give, a give is answered `Unavailable`, holdings never change, and every other pack's
//! facts are exactly those of the same script with it, minus the gives. A world that enables it
//! without inventory is refused by the registry, naming inventory.

mod support;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityType, Event, EventEnvelope,
    LocalPosition, Location, Millimetres, PlaceId, WorldTime,
};
use mineworld_inventory::{InventorySystem, ItemsTransferred};
use mineworld_item::ItemSystem;
use mineworld_item_transfer::ItemTransferSystem;
use mineworld_kernel::{Dispatched, KernelError, SystemIdentity, World};
use mineworld_movement::{Move, MovementSystem};
use mineworld_presence::PresenceSystem;
use support::{GENESIS, HOLDINGS, Town, t};

/// The only switch between the two worlds compared here. Set to `true`, the "without" world enables
/// item-transfer after all — the negative control (M-D6) that shows the assertions can fail.
const WITHOUT: bool = false;

fn walk(town: &mut Town, who: &str, x: i32, id: u64) -> Dispatched {
    let cafe = PlaceId::new(town.id("cafe"), EntityType::Place).expect("a place");
    let to = Location::in_place(cafe).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(1_000),
    ));
    let at = t(i64::try_from(id).expect("small"));
    let intent = ActionIntent::new(
        ActionId::from_raw(id),
        town.id(who),
        ActionRecord::new::<Move>(serde_json::to_vec(&Move::new(to)).expect("encodes")),
        at,
    );
    town.world.dispatch(&intent, at).expect("dispatch answers")
}

/// A fact as two worlds can share it although their event ids differ: (instant, type, payload,
/// cause).
type Shared = (WorldTime, String, Vec<u8>, String);

fn shared(fact: &EventEnvelope) -> Shared {
    (
        fact.at(),
        fact.event_type().as_str().to_owned(),
        fact.payload().payload().to_vec(),
        format!("{:?}", fact.caused_by()),
    )
}

/// One step of the script: who acts, a give (taker, item) or else a walk to `x` millimetres.
type Step<'a> = (&'a str, Option<(&'a str, &'a str)>, i32);

/// One script — gives and walks interleaved — and every fact it produced, genesis first.
fn script(with_item_transfer: bool) -> (Town, Vec<EventEnvelope>) {
    let mut town = Town::begun(with_item_transfer, &HOLDINGS);
    let mut facts = town.genesis.clone();
    let steps: [Step; 5] = [
        ("alice", Some(("bob", "coffee")), 0),
        ("bob", None, 3_000),
        ("bob", Some(("alice", "apple")), 0),
        ("alice", None, 2_500),
        ("alice", Some(("bob", "tea")), 0),
    ];
    for (index, (who, give, x)) in steps.into_iter().enumerate() {
        let id = u64::try_from(index).expect("small") + 1;
        let done = match give {
            Some((taker, item)) => town.give(who, Some(taker), item, 1, id).expect("answers"),
            None => walk(&mut town, who, x, id),
        };
        let accepted = matches!(done.result(), ActionResult::Accepted { .. });
        if give.is_some() && !with_item_transfer {
            assert_eq!(*done.result(), ActionResult::Unavailable, "step {index}");
        } else {
            assert!(accepted, "step {index}: {:?}", done.result());
        }
        facts.extend(done.events().iter().cloned());
    }
    (town, facts)
}

#[test]
fn a_world_without_item_transfer_offers_no_give_answers_unavailable_and_holds_still() {
    let mut town = Town::begun(WITHOUT, &HOLDINGS);
    for observer in ["alice", "bob", "carol"] {
        let seen = town.observation(observer, GENESIS);
        assert!(
            seen.affordances()
                .iter()
                .all(|a| a.action_type().as_str() != "give"),
            "{observer} is offered a give in a world without item-transfer"
        );
    }
    let done = town
        .give("alice", Some("bob"), "coffee", 1, 1)
        .expect("dispatch answers");
    assert_eq!(*done.result(), ActionResult::Unavailable);
    assert!(done.events().is_empty());
    assert_eq!(
        town.holdings("alice"),
        [("coffee".to_owned(), 1), ("tea".to_owned(), 1)],
        "holdings never change"
    );
}

#[test]
fn every_other_packs_facts_are_the_same_script_with_it_minus_the_gives() {
    let (with, with_facts) = script(true);
    let (without, without_facts) = script(WITHOUT);
    let gives = with_facts
        .iter()
        .filter(|fact| *fact.event_type() == ItemsTransferred::EVENT_TYPE)
        .count();
    assert_eq!(
        gives, 3,
        "located before compared: the script gave three times"
    );
    let expected: Vec<Shared> = with_facts
        .iter()
        .filter(|fact| *fact.event_type() != ItemsTransferred::EVENT_TYPE)
        .map(shared)
        .collect();
    let actual: Vec<Shared> = without_facts.iter().map(shared).collect();
    assert_eq!(actual, expected);
    assert_ne!(
        with.holdings("bob"),
        without.holdings("bob"),
        "the gives changed holdings only where item-transfer was enabled"
    );
    assert_eq!(without.holdings("bob"), [("apple".to_owned(), 1)]);
}

#[test]
fn a_world_enabling_item_transfer_without_inventory_is_refused_naming_inventory() {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(MovementSystem).expect("movement");
    world.install(ItemSystem).expect("item");
    match world.install(ItemTransferSystem) {
        Err(KernelError::SystemDependencyMissing { system, dependency }) => {
            assert_eq!(system, ItemTransferSystem::ID);
            assert_eq!(dependency, InventorySystem::ID);
        }
        other => panic!("the registry must refuse, naming inventory: {other:?}"),
    }
}
