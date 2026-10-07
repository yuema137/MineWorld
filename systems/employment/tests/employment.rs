//! E-6, work is attendance, exactly: a shift over a hand-built café through the kernel's real genesis,
//! dispatch and processes, with people walking in and out under `move` (presence's real
//! `person-entered-place`). Who was there, for how long, is what is paid and what is produced — in
//! integers, with no controller knowing that work exists (`ARC-38` item 2).

mod support;

use mineworld_authoring::AuthoredSection;
use mineworld_contracts::{Causation, Component, Event, EventEnvelope, Visibility};
use mineworld_employment::{
    BIOGRAPHICAL, Employment, EmploymentSystem, Hired, ShiftEnded, ShiftStarted, WageDue,
    employed_by,
};
use mineworld_inventory::ItemsProduced;
use mineworld_sdk::SystemPack;
use support::{GENESIS, HOUR, JOB, Town, job, of, providers};

const DAY: i64 = 86_400;

fn per_employee<E: Event + serde::de::DeserializeOwned>(
    town: &Town,
    facts: &[EventEnvelope],
    employee: impl Fn(&E) -> mineworld_contracts::PersonId,
) -> Vec<(String, E)> {
    of::<E>(facts)
        .into_iter()
        .map(|(event, _)| {
            let who = employee(&event);
            let name = ["alice", "bob", "carol", "dave"]
                .into_iter()
                .find(|name| town.person(name) == who)
                .expect("one of the four")
                .to_owned();
            (name, event)
        })
        .collect()
}

/// The core claim, with every number worked out by hand from the job (08:00–12:00, 120 an hour, four
/// coffees and two croissants a full shift of 14 400 s):
///
/// ```text
/// alice  there all shift       14 400 s   480   coffee 4, croissant 2
/// bob    leaves at 10:00        7 200 s   240   coffee 2, croissant 1
/// carol  arrives at 11:00       3 600 s   120   coffee 1  (croissant 2 × 3 600 ÷ 14 400 = 0.5 → 0)
/// dave   never comes                0 s   no wage-due, nothing produced
/// ```
#[test]
fn a_shift_is_paid_and_produces_for_the_time_present_and_nothing_for_absence() {
    let mut town = Town::begun(true);
    let facts = town.work_one_day();

    let started = per_employee::<ShiftStarted>(&town, &facts, ShiftStarted::employee);
    let present: Vec<(&str, bool)> = started
        .iter()
        .map(|(name, event)| (name.as_str(), event.present()))
        .collect();
    assert_eq!(
        present,
        [
            ("alice", true),
            ("bob", true),
            ("carol", false),
            ("dave", false)
        ],
        "who was at the café when the shift began"
    );

    let ended = per_employee::<ShiftEnded>(&town, &facts, ShiftEnded::employee);
    let worked: Vec<(&str, u64)> = ended
        .iter()
        .map(|(name, event)| (name.as_str(), event.worked()))
        .collect();
    assert_eq!(
        worked,
        [
            ("alice", 14_400),
            ("bob", 7_200),
            ("carol", 3_600),
            ("dave", 0)
        ],
        "seconds at the workplace during the shift"
    );

    let due = per_employee::<WageDue>(&town, &facts, WageDue::employee);
    let paid: Vec<(&str, u64)> = due
        .iter()
        .map(|(name, event)| (name.as_str(), event.amount()))
        .collect();
    assert_eq!(
        paid,
        [("alice", 480), ("bob", 240), ("carol", 120)],
        "wage × worked ÷ 3 600; nothing is due for nothing worked"
    );
    assert!(
        due.iter()
            .all(|(_, event)| event.employer() == town.employer()),
        "owed by the employer"
    );

    let produced: Vec<(String, u32)> = of::<ItemsProduced>(&facts)
        .into_iter()
        .map(|(event, _)| {
            assert_eq!(event.holder(), town.id("cafe-company"), "for the employer");
            let name = if event.item() == town.item("coffee") {
                "coffee"
            } else {
                "croissant"
            };
            (name.to_owned(), event.count())
        })
        .collect();
    assert_eq!(
        produced,
        [
            ("coffee".to_owned(), 4),
            ("croissant".to_owned(), 2),
            ("coffee".to_owned(), 2),
            ("croissant".to_owned(), 1),
            ("coffee".to_owned(), 1),
        ],
        "prorated by attendance, rounded down, skipped at zero — alice, bob, carol in turn"
    );
    assert_eq!(
        town.stock(),
        [
            ("coffee".to_owned(), 1 + 4 + 2 + 1),
            ("croissant".to_owned(), 3)
        ],
        "inventory reduced every production into the employer's holdings"
    );

    for fact in facts.iter().filter(|fact| {
        [
            ShiftStarted::EVENT_TYPE,
            ShiftEnded::EVENT_TYPE,
            WageDue::EVENT_TYPE,
            ItemsProduced::EVENT_TYPE,
        ]
        .contains(fact.event_type())
    }) {
        assert!(
            matches!(fact.caused_by(), Causation::Process(_)),
            "a shift's facts are caused by its process: {fact:?}"
        );
    }
    for (_, fact) in of::<WageDue>(&facts) {
        assert_eq!(*fact.visibility(), Visibility::Participants);
        assert_eq!(
            fact.participants().len(),
            2,
            "the employee and the employer"
        );
    }
    assert_eq!(
        facts
            .iter()
            .filter(|fact| fact.event_type().as_str() == "money-transferred")
            .count(),
        0,
        "employment moves no money: nobody here pays the wage"
    );
}

/// A wake reschedules exactly once: the same shift happens again the next day, at the same times,
/// and nothing happens between shifts.
#[test]
fn the_shift_comes_round_every_day_at_its_times() {
    let mut town = Town::begun(true);
    town.work_one_day();
    let quiet = town.advance(DAY + 8 * HOUR - 1);
    assert!(
        of::<ShiftStarted>(&quiet).is_empty() && of::<ShiftEnded>(&quiet).is_empty(),
        "nothing between 13:00 and the next 08:00"
    );
    let next = town.advance(DAY + 13 * HOUR);
    let starts: Vec<i64> = of::<ShiftStarted>(&next)
        .iter()
        .map(|(_, fact)| fact.at().seconds())
        .collect();
    let ends: Vec<i64> = of::<ShiftEnded>(&next)
        .iter()
        .map(|(_, fact)| fact.at().seconds())
        .collect();
    assert_eq!(starts, [DAY + 8 * HOUR; 4], "one start per employee");
    assert_eq!(ends, [DAY + 12 * HOUR; 4], "one end per employee");
    let worked: Vec<u64> = of::<ShiftEnded>(&next)
        .iter()
        .map(|(event, _)| event.worked())
        .collect();
    assert_eq!(
        worked,
        [14_400, 0, 14_400, 0],
        "day 1: alice and carol are in the café, bob and dave on the street"
    );
}

#[test]
fn hiring_is_a_genesis_fact_the_owner_reduces_into_employment_and_an_edge() {
    let town = Town::begun(true);
    let hired = of::<Hired>(&town.genesis);
    assert_eq!(hired.len(), 4, "one per job");
    for (event, fact) in &hired {
        assert_eq!(*fact.caused_by(), Causation::WorldGenesis);
        let who = event.employee().entity_id();
        assert_eq!(
            *fact.visibility(),
            Visibility::Entities([who].into_iter().collect()),
            "one's job is one's own to know"
        );
        let employment = town
            .world
            .read()
            .component::<Employment>(who)
            .cloned()
            .expect("employment written");
        assert_eq!(employment.job(), event.job());
        assert!(employment.shift().is_none(), "off shift at midnight");
        let edge = employed_by();
        let read = town.world.read();
        let edges: Vec<_> = read
            .relations_of_type(&edge)
            .filter(|relation| relation.from() == who)
            .map(|relation| relation.to())
            .collect();
        assert_eq!(
            edges,
            [town.id("cafe-company")],
            "the employed-by edge, Person → Organization"
        );
    }
    assert_eq!(BIOGRAPHICAL, [Hired::EVENT_TYPE]);
    assert_eq!(
        <EmploymentSystem as SystemPack>::BIOGRAPHICAL,
        [Hired::EVENT_TYPE],
        "the pack says so to the build"
    );
}

#[test]
fn employment_is_disclosed_to_the_employee_and_to_nobody_else() {
    let town = Town::begun(true);
    let seen = mineworld_presence::observe(&town.world, town.id("alice"), GENESIS, &providers());
    let disclosed = |subject| {
        seen.entity(subject).map(|entity| {
            entity
                .components()
                .iter()
                .filter(|record| *record.component_type() == Employment::COMPONENT_TYPE)
                .count()
        })
    };
    assert_eq!(
        disclosed(town.id("alice")),
        Some(1),
        "Alice is told her own"
    );
    assert_eq!(
        disclosed(town.id("bob")),
        Some(0),
        "Alice perceives Bob and is not told his job"
    );
}

#[test]
fn a_job_section_is_checked_as_it_is_decoded() {
    let refused = job(
        "{ employer: cafe-company, workplace: cafe, from: \"12:00\", until: \"08:00\", \
                       wage: 120 }",
    )
    .expect_err("a shift that ends before it starts");
    assert!(
        refused.to_string().contains("must end after it starts"),
        "refused with the pack's own message: {refused}"
    );
    assert!(
        job("{ employer: cafe-company, workplace: cafe, from: \"08:00\", until: \"08:00\", wage: 1 }")
            .is_err(),
        "an empty shift"
    );
    assert!(
        job("{ employer: cafe-company, workplace: cafe, from: \"08:00\", until: \"09:00\", wage: 1, \
             bonus: 3 }")
        .is_err(),
        "an unknown key"
    );
    assert!(
        job("{ employer: cafe-company, workplace: cafe, from: \"08:00\", until: \"09:00\", wage: 1, \
             produces: { coffee: 0 } }")
        .is_err(),
        "a production of zero"
    );
    assert!(
        job("{ employer: cafe-company, workplace: cafe, from: \"08:00\", until: \"09:00\", wage: -1 }")
            .is_err(),
        "a negative wage: integer minor units, never below zero"
    );
    let authored = job(JOB).expect("valid");
    assert_eq!(
        EmploymentSystem::references(&authored)
            .iter()
            .map(|reference| (reference.key.as_str(), reference.entity_type))
            .collect::<Vec<_>>(),
        [
            (
                "cafe-company",
                mineworld_contracts::EntityType::Organization
            ),
            ("cafe", mineworld_contracts::EntityType::Place),
            ("coffee", mineworld_contracts::EntityType::Item),
            ("croissant", mineworld_contracts::EntityType::Item),
        ],
        "the employer, the workplace and every produced kind must resolve, by kind"
    );
}
