//! E-7, the employment half of `AC-2` at pack level.
//!
//! - Employment installed **without economy** states `wage-due` that nobody reduces, and the world
//!   runs on, shift after shift, with no fault: stating a wage moves no money (`ARC-28`).
//! - A world **without employment** runs the same walks; every other pack's facts are those of the
//!   world with it, minus employment's own and the production it stated.
//! - Employment without inventory is refused by the registry, naming inventory.

mod support;

use mineworld_contracts::{Causation, Event, EventEnvelope, WorldTime};
use mineworld_employment::{EmploymentSystem, Hired, ShiftEnded, ShiftStarted, WageDue};
use mineworld_inventory::{InventorySystem, ItemsProduced};
use mineworld_item::ItemSystem;
use mineworld_kernel::{KernelError, SystemIdentity, World};
use mineworld_presence::PresenceSystem;
use support::{Town, of};

const DAY: i64 = 86_400;

/// A fact as two worlds can share it although their event ids differ: (instant, type, payload,
/// cause). A cause that is another fact is named by that fact's (instant, type, payload), because
/// the hired facts shift every later event id by four.
type Shared = (WorldTime, String, Vec<u8>, String);

fn shared(all: &[EventEnvelope]) -> impl Fn(&EventEnvelope) -> Shared + '_ {
    move |fact| {
        let cause = match fact.caused_by() {
            Causation::Event(id) => {
                let cause = all
                    .iter()
                    .find(|other| other.id() == *id)
                    .expect("the cause is in the same history");
                format!(
                    "event {:?} {} {:?}",
                    cause.at(),
                    cause.event_type().as_str(),
                    cause.payload().payload()
                )
            }
            other => format!("{other:?}"),
        };
        (
            fact.at(),
            fact.event_type().as_str().to_owned(),
            fact.payload().payload().to_vec(),
            cause,
        )
    }
}

fn employments(fact: &EventEnvelope) -> bool {
    [
        Hired::EVENT_TYPE,
        ShiftStarted::EVENT_TYPE,
        ShiftEnded::EVENT_TYPE,
        WageDue::EVENT_TYPE,
        ItemsProduced::EVENT_TYPE,
    ]
    .contains(fact.event_type())
}

#[test]
fn without_economy_a_wage_is_due_that_nobody_pays_and_the_world_runs_on() {
    let mut town = Town::begun(true);
    let mut facts = town.work_one_day();
    facts.extend(town.advance(3 * DAY));
    let due = of::<WageDue>(&facts);
    assert_eq!(
        due.len(),
        3 + 2 + 2,
        "day 0: alice, bob, carol; days 1 and 2: alice and carol (bob and dave on the street)"
    );
    assert!(
        facts
            .iter()
            .all(|fact| !fact.event_type().as_str().starts_with("money")
                && fact.event_type().as_str() != "wage-unpaid"),
        "nobody answers a wage-due in a world without economy"
    );
    assert_eq!(
        of::<ShiftEnded>(&facts).len(),
        3 * 4,
        "three days, four shifts a day, no fault"
    );
}

#[test]
fn every_other_packs_facts_are_the_same_walks_with_it_minus_employments() {
    let mut with = Town::begun(true);
    let mut with_facts = with.genesis.clone();
    with_facts.extend(with.work_one_day());
    let mut without = Town::begun(false);
    let mut without_facts = without.genesis.clone();
    without_facts.extend(without.work_one_day());

    assert!(
        of::<WageDue>(&with_facts).len() == 3 && of::<ItemsProduced>(&with_facts).len() == 5,
        "located before compared: the world with employment worked a shift"
    );
    let expected: Vec<Shared> = with_facts
        .iter()
        .filter(|fact| !employments(fact))
        .map(shared(&with_facts))
        .collect();
    let actual: Vec<Shared> = without_facts.iter().map(shared(&without_facts)).collect();
    assert_eq!(actual, expected);
    assert_eq!(
        without.stock(),
        [("coffee".to_owned(), 1)],
        "nothing produced without employment"
    );
}

#[test]
fn a_world_enabling_employment_without_inventory_is_refused_naming_inventory() {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence");
    world.install(ItemSystem).expect("item");
    match world.install(EmploymentSystem) {
        Err(KernelError::SystemDependencyMissing { system, dependency }) => {
            assert_eq!(system, EmploymentSystem::ID);
            assert_eq!(dependency, InventorySystem::ID);
        }
        other => panic!("the registry must refuse, naming inventory: {other:?}"),
    }
}
