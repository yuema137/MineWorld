//! A wage deduction is money, so economy alone answers it (`ARC-287`, I-LE2-2): employment states
//! `wage-deduction-due`; economy moves the amount from the employee to the employer as one
//! `money-transferred` caused by it, or records `deduction-unpaid` and moves nothing.
//!
//! Alice (1 000) and Bob (150) step out of the store at 07:00 and walk back in at 09:00, an hour into
//! their 08:00 shift: 60 started minutes × 10 = 600 each, the cap. Alice can pay; Bob cannot.

mod support;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Causation, EntityType, EventEnvelope,
    LocalPosition, Location, Millimetres, PlaceId, Visibility,
};
use mineworld_economy::{DeductionUnpaid, MoneyTransferred};
use mineworld_employment::{EmploymentSystem, WageDeductionDue};
use mineworld_kernel::Emission;
use mineworld_movement::Move;
use mineworld_sdk::interactions::{self, InteractionSection, Section, SectionConfigured};
use serde::Deserialize;
use support::{HOUR, Packs, STOCK, Town, of, t};

const WAGES: Packs = Packs {
    economy: true,
    employment: true,
};

/// Employment's section with deduction on, as the loader states it.
fn deduction_on() -> Emission {
    let section: Section<EmploymentSystem> =
        Section::deserialize(&mut serde_json::Deserializer::from_str(
            r#"{ "parameters": [ { "deduction_enabled": true } ] }"#,
        ))
        .expect("decodes");
    let classes = serde_json::from_str("[]").expect("no classes");
    let resolved = interactions::resolve(&section, &classes).expect("resolves");
    Emission::new::<SectionConfigured<EmploymentSystem>>(
        <EmploymentSystem as InteractionSection>::encode(&resolved),
        Visibility::SystemInternal,
    )
}

fn walk(town: &mut Town, who: &str, place: &str, at: i64, id: u64) -> Vec<EventEnvelope> {
    let mut facts = town
        .world
        .advance_to(t(at))
        .expect("advances")
        .into_events();
    let place = PlaceId::new(town.id(place), EntityType::Place).expect("a place");
    let to = Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(500),
        Millimetres::new(500),
    ));
    let intent = ActionIntent::new(
        ActionId::from_raw(id),
        town.id(who),
        ActionRecord::new::<Move>(serde_json::to_vec(&Move::new(to)).expect("encodes")),
        t(at),
    );
    let done = town.world.dispatch(&intent, t(at)).expect("answers");
    assert!(matches!(done.result(), ActionResult::Accepted { .. }));
    facts.extend(done.events().iter().cloned());
    facts
}

#[test]
fn a_deduction_the_employee_can_pay_moves_and_one_they_cannot_is_recorded_unpaid() {
    let mut town = Town::begun_with(WAGES, STOCK, |_| {}, vec![deduction_on()]);
    walk(&mut town, "alice", "street", 7 * HOUR, 1);
    walk(&mut town, "bob", "street", 7 * HOUR, 2);
    let mut facts = Vec::new();
    for (who, id) in [("alice", 3), ("bob", 4)] {
        facts.extend(walk(&mut town, who, "store", 9 * HOUR, id));
    }
    let due = of::<WageDeductionDue>(&facts);
    assert_eq!(
        due.iter()
            .map(|(event, _)| (event.employee().entity_id(), event.amount()))
            .collect::<Vec<_>>(),
        [(town.id("alice"), 600), (town.id("bob"), 600)]
    );
    let paid: Vec<_> = of::<MoneyTransferred>(&facts);
    assert_eq!(paid.len(), 1, "only Alice could pay");
    let (payment, fact) = &paid[0];
    assert_eq!(
        (payment.from(), payment.to(), payment.amount()),
        (town.id("alice"), town.id("corner-store"), 600)
    );
    assert_eq!(*fact.caused_by(), Causation::Event(due[0].1.id()));
    let unpaid = of::<DeductionUnpaid>(&facts);
    assert_eq!(unpaid.len(), 1);
    assert_eq!(*unpaid[0].1.caused_by(), Causation::Event(due[1].1.id()));
    assert_eq!(
        (town.wallet("alice"), town.wallet("bob")),
        (Some(400), Some(150)),
        "never a negative balance"
    );
}
