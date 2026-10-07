//! E-4, the wage half: employment states `wage-due`; economy alone answers it. An employer who can
//! pay moves the wage as one `money-transferred` caused by the `wage-due`; an employer who cannot gets
//! `wage-unpaid`, both balances unchanged and no money moved — never a negative balance (`u64`), and
//! the refusal path is the one taken (`CORE_CONCEPTS.md` §13.1, `ARC-28`).

mod support;

use mineworld_contracts::{Causation, Visibility};
use mineworld_economy::{MoneyTransferred, WageUnpaid};
use mineworld_employment::WageDue;
use support::{HOUR, Packs, Town, of, t};

const WAGES: Packs = Packs {
    economy: true,
    employment: true,
};

#[test]
fn a_wage_the_employer_can_pay_moves_and_one_it_cannot_is_recorded_unpaid() {
    let mut town = Town::begun(WAGES);
    assert_eq!(
        (town.wallet("corner-store"), town.wallet("poor-co")),
        (Some(1_000), Some(100)),
        "located: the store can pay a shift's 480, poor-co cannot"
    );
    let facts = town
        .world
        .advance_to(t(13 * HOUR))
        .expect("advances")
        .into_events();

    let due = of::<WageDue>(&facts);
    assert_eq!(
        due.iter()
            .map(|(event, _)| (event.employee().entity_id(), event.amount()))
            .collect::<Vec<_>>(),
        [(town.id("alice"), 480), (town.id("bob"), 480)],
        "both were in the store for the whole shift"
    );
    let due_id = |who| {
        due.iter()
            .find(|(event, _)| event.employee().entity_id() == who)
            .map(|(_, fact)| fact.id())
            .expect("a wage-due")
    };

    let paid = of::<MoneyTransferred>(&facts);
    assert_eq!(paid.len(), 1, "one wage moved: alice's");
    let (payment, fact) = &paid[0];
    assert_eq!(
        (payment.from(), payment.to(), payment.amount()),
        (town.id("corner-store"), town.id("alice"), 480)
    );
    assert_eq!(
        *fact.caused_by(),
        Causation::Event(due_id(town.id("alice"))),
        "caused by employment's wage-due"
    );
    assert_eq!(*fact.visibility(), Visibility::Participants);

    let unpaid = of::<WageUnpaid>(&facts);
    assert_eq!(unpaid.len(), 1, "bob's wage could not be paid");
    let (record, fact) = &unpaid[0];
    assert_eq!(
        (
            record.employee().entity_id(),
            record.employer().entity_id(),
            record.amount()
        ),
        (town.id("bob"), town.id("poor-co"), 480)
    );
    assert_eq!(*fact.caused_by(), Causation::Event(due_id(town.id("bob"))));

    assert_eq!(
        (
            town.wallet("alice"),
            town.wallet("corner-store"),
            town.wallet("bob"),
            town.wallet("poor-co")
        ),
        (Some(1_480), Some(520), Some(150), Some(100)),
        "alice paid; bob and poor-co unchanged — no partial payment, no negative balance"
    );
}
