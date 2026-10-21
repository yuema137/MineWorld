//! Putting a thing down and picking one up (S35 N-h R4; AC-Nh-5 rows 2 and 3), over the hand-built café
//! through presence's real `observe` and the kernel's real dispatch: a full person puts a thing down in the
//! place; the place's holdings are disclosed to a second person present, who picks it up; a pick-up is
//! refused `cannot-carry` when the picker is full and `not-here` when nothing lies there; items are
//! conserved.

mod support;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Event, Rejection, RejectionCode, Visibility,
};
use mineworld_inventory::{ItemsTransferred, PERSON_CAPACITY};
use mineworld_item_transfer::{PickUp, PutDown};
use mineworld_kernel::Dispatched;
use serde_json::Value;
use support::{GENESIS, Town, t};

/// Alice's hands are full (six), Bob holds one, Carol nothing.
const FULL: [(&str, &str); 2] = [("alice", "{ coffee: 3, tea: 3 }"), ("bob", "{ apple: 1 }")];

fn hands(town: &mut Town, who: &str, record: ActionRecord, id: u64) -> Dispatched {
    let at = t(i64::try_from(id).expect("small"));
    let intent = ActionIntent::new(ActionId::from_raw(id), town.id(who), record, at);
    town.world.dispatch(&intent, at).expect("dispatch answers")
}

fn put_down(town: &mut Town, who: &str, item: &str, id: u64) -> Dispatched {
    let record = ActionRecord::new::<PutDown>(
        serde_json::to_vec(&PutDown::new(town.item(item))).expect("encodes"),
    );
    hands(town, who, record, id)
}

fn pick_up(town: &mut Town, who: &str, item: &str, id: u64) -> Dispatched {
    let record = ActionRecord::new::<PickUp>(
        serde_json::to_vec(&PickUp::new(town.item(item))).expect("encodes"),
    );
    hands(town, who, record, id)
}

fn owned(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
    pairs.iter().map(|(k, n)| ((*k).to_owned(), *n)).collect()
}

/// The `holdings` record an observation discloses about `entity`, as JSON.
fn holdings_of(
    observation: &mineworld_contracts::Observation<Value>,
    entity: u64,
) -> Option<Value> {
    observation
        .entities()
        .iter()
        .find(|seen| seen.id().raw() == entity)?
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "holdings")
        .map(|record| record.payload().clone())
}

#[test]
fn a_full_person_puts_a_thing_down_and_a_second_person_present_sees_it_and_picks_it_up() {
    let mut town = Town::begun(true, &FULL);
    let (alice, bob, cafe) = (town.id("alice"), town.id("bob"), town.id("cafe"));
    // Capacity is disclosed with my holdings, so no controller hard-codes the bound (R2).
    let mine = holdings_of(&town.observation("alice", GENESIS), alice.raw()).expect("disclosed");
    assert_eq!(mine["capacity"], PERSON_CAPACITY);
    // Full: one put-down per kind held, available.
    let seen = town.observation("alice", GENESIS);
    let offered: Vec<_> = seen
        .affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == "put-down")
        .collect();
    assert_eq!(offered.len(), 2, "one per kind held: {offered:?}");
    assert!(offered.iter().all(|affordance| affordance.is_available()));

    let done = put_down(&mut town, "alice", "coffee", 1);
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    let fact = &done.events()[0];
    assert_eq!(*fact.event_type(), ItemsTransferred::EVENT_TYPE);
    assert!(
        matches!(fact.visibility(), Visibility::Place(place) if place.entity_id() == cafe),
        "heard in the café: {:?}",
        fact.visibility()
    );
    assert_eq!(town.holdings("alice"), owned(&[("coffee", 2), ("tea", 3)]));
    assert_eq!(town.holdings("cafe"), owned(&[("coffee", 1)]));

    // Bob perceives the café and what lies in it, and is offered the pick-up.
    let bobs = town.observation("bob", t(2));
    let lying = holdings_of(&bobs, cafe.raw()).expect("a place's holdings are seen");
    assert_eq!(lying["held"].as_array().map(Vec::len), Some(1));
    assert!(lying.get("capacity").is_none(), "a place is unbounded");
    assert!(
        holdings_of(&bobs, alice.raw()).is_none(),
        "what Alice carries is still hers to know"
    );
    let pick = bobs
        .affordances()
        .iter()
        .find(|affordance| affordance.action_type().as_str() == "pick-up")
        .expect("offered");
    assert!(pick.is_available());
    let done = pick_up(&mut town, "bob", "coffee", 3);
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    assert_eq!(done.events()[0].participants(), &[cafe, bob]);
    assert_eq!(town.holdings("bob"), owned(&[("apple", 1), ("coffee", 1)]));
    assert!(
        town.holdings("cafe").is_empty(),
        "items are conserved: nothing left behind"
    );
}

#[test]
fn a_pick_up_is_refused_with_a_reason_and_writes_nothing() {
    let mut town = Town::begun(
        true,
        &[("alice", "{ coffee: 3, tea: 3 }"), ("bob", "{ apple: 1 }")],
    );
    let bob_put = put_down(&mut town, "bob", "apple", 1);
    assert!(matches!(bob_put.result(), ActionResult::Accepted { .. }));
    // Alice is full: the apple lying there is offered, refused `cannot-carry`.
    let seen = town.observation("alice", t(2));
    let pick = seen
        .affordances()
        .iter()
        .find(|affordance| affordance.action_type().as_str() == "pick-up")
        .expect("offered");
    let cannot = Rejection::System {
        code: RejectionCode::new("cannot-carry").expect("a code"),
        detail: None,
    };
    assert_eq!(pick.unavailable_reason(), Some(&cannot));
    let refused = pick_up(&mut town, "alice", "apple", 3);
    assert_eq!(*refused.result(), ActionResult::Rejected(cannot));
    assert!(refused.events().is_empty());
    // Nothing of that kind lies here: `not-here`.
    let absent = pick_up(&mut town, "carol", "tea", 4);
    assert_eq!(
        *absent.result(),
        ActionResult::Rejected(Rejection::System {
            code: RejectionCode::new("not-here").expect("a code"),
            detail: None,
        })
    );
    // Putting down what one does not hold is refused, and nobody empty-handed is offered a put-down.
    let nothing = put_down(&mut town, "carol", "tea", 5);
    assert!(matches!(nothing.result(), ActionResult::Rejected(_)));
    assert!(
        town.observation("carol", t(6))
            .affordances()
            .iter()
            .all(|affordance| affordance.action_type().as_str() != "put-down")
    );
    assert_eq!(town.holdings("cafe"), vec![("apple".to_owned(), 1)]);
}
