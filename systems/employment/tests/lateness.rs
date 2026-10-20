//! Lateness and what it costs (`ARC-287`; `pr-le-leave-early.md` §0.6), over the support café: Alice and
//! Bob are there when the 08:00 shift begins, Bob leaves at 10:00, Carol walks in at 11:00, Dave never
//! comes. Every number below is worked out by hand from the job (08:00–12:00) and the defaults
//! (grace 0, window 7 days, warning at 2, final at 3; 10 per started late minute, at most 600).
//!
//! ```text
//! default            complaints only: Carol late 10 800 s, Dave absent 14 400 s; no deduction, no offer
//! deduction, no boss wage-deduction-due at once, 600 each (capped), caused by the complaint
//! deduction, a boss  Alice manages: she is told, and offered `penalize` up to 600 per lateness; the
//!                    refusals in AC-LE2-3's order; one accepted penalty, caused by the request
//! a second day       Dave absent again: count 2, a warning
//! two managers       refused at genesis, by name
//! ```

mod support;

use mineworld_authoring::AuthoredSection;
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, Causation, EntityId, EventEnvelope,
    PersonId, Rejection, Visibility,
};
use mineworld_employment::{
    DisclosedEmployment, Employment, EmploymentSystem, LateForShift, Level, PENALTY_OUT_OF_BOUNDS,
    Penalize, WageDeductionDue,
};
use mineworld_kernel::{Emission, World};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::interactions::{self, InteractionSection, Section, SectionConfigured};
use serde::Deserialize;
use support::{GENESIS, HOUR, JOB, PLACED, Town, compose, job, key, of, t};

const DAY: i64 = 86_400;

/// The section that switches deduction on, at the defaults' numbers.
const DEDUCTION_ON: &str = r#"{ "parameters": [ { "deduction_enabled": true } ] }"#;

/// The configured fact, built as the loader builds it.
fn configured(section: &str) -> Emission {
    let section: Section<EmploymentSystem> =
        Section::deserialize(&mut serde_json::Deserializer::from_str(section)).expect("decodes");
    let classes = serde_json::from_str("[]").expect("no classes");
    let resolved = interactions::resolve(&section, &classes).expect("resolves");
    Emission::new::<SectionConfigured<EmploymentSystem>>(
        <EmploymentSystem as InteractionSection>::encode(&resolved),
        Visibility::SystemInternal,
    )
}

/// The café with every job seeded, `managers` saying `manages: true`, and `section` configured.
fn town(section: Option<&str>, managers: &[&str]) -> Result<Town, mineworld_kernel::KernelError> {
    let (mut world, keys, mut facts) = Town::assemble(compose(true), false);
    {
        let read = world.read();
        let seeding = mineworld_authoring::Seeding::new(&read, &keys);
        for (person, _) in PLACED {
            // JOB ends in its closing brace: the flag goes just before it.
            let text = if managers.contains(&person) {
                format!("{}, manages: true }}", JOB[..JOB.len() - 1].trim_end())
            } else {
                JOB.to_owned()
            };
            let authored = job(&text).expect("a job");
            facts.extend(
                EmploymentSystem::seed(&seeding, keys[&key(person)], &authored).expect("seeded"),
            );
        }
    }
    if let Some(section) = section {
        facts.push(configured(section));
    }
    let genesis = world.genesis(GENESIS, facts)?;
    Ok(Town {
        world,
        keys,
        genesis,
    })
}

fn name_of(town: &Town, person: PersonId) -> &'static str {
    ["alice", "bob", "carol", "dave"]
        .into_iter()
        .find(|name| town.person(name) == person)
        .expect("one of the four")
}

/// One `late-for-shift`: (who, late_by, count, level, absent, manager).
type Complaint = (&'static str, u64, u32, Level, bool, Option<&'static str>);

/// Each `late-for-shift`: (who, late_by, count, level, absent, manager).
fn complaints(town: &Town, facts: &[EventEnvelope]) -> Vec<Complaint> {
    of::<LateForShift>(facts)
        .into_iter()
        .map(|(late, _)| {
            (
                name_of(town, late.employee()),
                late.late_by(),
                late.count(),
                late.level(),
                late.absent(),
                late.manager().map(|manager| name_of(town, manager)),
            )
        })
        .collect()
}

fn penalize(town: &mut Town, actor: &str, employee: &str, amount: u64, id: u64) -> ActionResult {
    let payload = Penalize::new(town.person(employee), amount);
    let intent = ActionIntent::new(
        ActionId::from_raw(id),
        town.id(actor),
        ActionRecord::new::<Penalize>(serde_json::to_vec(&payload).expect("encodes")),
        town.world.now(),
    );
    let now = town.world.now();
    let done = town.world.dispatch(&intent, now).expect("dispatch answers");
    if !matches!(done.result(), ActionResult::Accepted { .. }) {
        assert!(done.events().is_empty(), "a refusal records nothing");
    }
    done.result().clone()
}

fn disclosed(world: &World, person: EntityId) -> DisclosedEmployment {
    let records = EmploymentSystem.discloses(&world.read(), person, person);
    serde_json::from_value(records[0].payload().clone()).expect("the disclosed record")
}

#[test]
fn by_default_lateness_is_a_complaint_and_moves_no_money() {
    let mut town = Town::begun(true);
    let facts = town.work_one_day();
    assert_eq!(
        complaints(&town, &facts),
        [
            ("carol", 3 * HOUR as u64, 1, Level::Notice, false, None),
            ("dave", 4 * HOUR as u64, 1, Level::Notice, true, None),
        ]
    );
    let (late, envelope) = &of::<LateForShift>(&facts)[0];
    assert_eq!(late.shift(), t(8 * HOUR), "the shift's start");
    assert_eq!(envelope.at(), t(11 * HOUR), "stated as Carol walks in");
    assert!(
        of::<WageDeductionDue>(&facts).is_empty(),
        "deduction is off"
    );
    assert!(
        EmploymentSystem
            .offers(&town.world.read(), town.id("alice"), None)
            .is_empty()
    );
    // Kept on the employee: the shift's instant, and nothing open.
    let carol = town
        .world
        .read()
        .component::<Employment>(town.id("carol"))
        .cloned()
        .expect("employed");
    assert_eq!(carol.late(), [t(8 * HOUR)]);
    assert_eq!(carol.open(), None);
}

#[test]
fn with_deduction_on_and_no_manager_the_authors_numbers_decide() {
    let mut town = town(Some(DEDUCTION_ON), &[]).expect("begins");
    let facts = town.work_one_day();
    let deductions: Vec<(&str, u64)> = of::<WageDeductionDue>(&facts)
        .iter()
        .map(|(due, _)| (name_of(&town, due.employee()), due.amount()))
        .collect();
    assert_eq!(deductions, [("carol", 600), ("dave", 600)], "capped at 600");
    let complaint_ids: Vec<_> = of::<LateForShift>(&facts)
        .iter()
        .map(|(_, envelope)| envelope.id())
        .collect();
    for ((due, envelope), cause) in of::<WageDeductionDue>(&facts).iter().zip(complaint_ids) {
        assert_eq!(*envelope.caused_by(), Causation::Event(cause));
        assert_eq!(due.employer(), town.employer());
        assert_eq!(due.shift(), t(8 * HOUR));
    }
}

#[test]
fn with_a_manager_the_boss_decides_and_the_refusals_come_in_order() {
    let mut town = town(Some(DEDUCTION_ON), &["alice"]).expect("begins");
    let facts = town.work_one_day();
    assert_eq!(
        complaints(&town, &facts),
        [
            (
                "carol",
                3 * HOUR as u64,
                1,
                Level::Notice,
                false,
                Some("alice")
            ),
            (
                "dave",
                4 * HOUR as u64,
                1,
                Level::Notice,
                true,
                Some("alice")
            ),
        ],
        "the manager is told"
    );
    let (_, envelope) = &of::<LateForShift>(&facts)[0];
    assert_eq!(
        *envelope.visibility(),
        Visibility::Entities([town.id("carol"), town.id("alice")].into_iter().collect())
    );
    assert!(
        of::<WageDeductionDue>(&facts).is_empty(),
        "nothing automatic"
    );

    let offers = EmploymentSystem.offers(&town.world.read(), town.id("alice"), None);
    let offered: Vec<Penalize> = offers
        .iter()
        .map(|offer| {
            serde_json::from_value(offer.payload().expect("complete").clone()).expect("a penalty")
        })
        .collect();
    assert_eq!(
        offered,
        [
            Penalize::new(town.person("carol"), 600),
            Penalize::new(town.person("dave"), 600)
        ]
    );
    assert!(
        EmploymentSystem
            .offers(&town.world.read(), town.id("bob"), None)
            .is_empty(),
        "only to the manager"
    );
    let staff = disclosed(&town.world, town.id("alice"));
    assert_eq!(staff.late_staff().len(), 2);
    assert_eq!(staff.late_staff()[0].lateness().count(), 1);
    assert!(
        disclosed(&town.world, town.id("bob"))
            .late_staff()
            .is_empty()
    );

    assert_eq!(
        penalize(&mut town, "bob", "carol", 100, 10),
        ActionResult::Rejected(Rejection::PermissionDenied),
        "not the manager"
    );
    let ActionResult::Rejected(Rejection::System { code, detail }) =
        penalize(&mut town, "alice", "carol", 700, 11)
    else {
        panic!("above the bound")
    };
    assert_eq!(code, PENALTY_OUT_OF_BOUNDS);
    assert_eq!(detail.as_deref(), Some("amount is 1 … 600, not 700"));
    assert!(matches!(
        penalize(&mut town, "alice", "carol", 0, 12),
        ActionResult::Rejected(Rejection::System { .. })
    ));
    let ActionResult::Accepted { events } = penalize(&mut town, "alice", "carol", 250, 13) else {
        panic!("accepted")
    };
    assert_eq!(events.len(), 1, "one wage-deduction-due");
    assert_eq!(
        penalize(&mut town, "alice", "carol", 100, 14),
        ActionResult::Rejected(Rejection::NoSupportedInteraction),
        "a lateness is penalized once"
    );
    assert_eq!(
        EmploymentSystem
            .offers(&town.world.read(), town.id("alice"), None)
            .len(),
        1,
        "Dave's is still open"
    );
    // Dave's lateness stays open for the window and no longer (he is absent again every day after,
    // which opens a new one each time, so the window is read off the record).
    let dave = town
        .world
        .read()
        .component::<Employment>(town.id("dave"))
        .cloned()
        .expect("employed");
    let window = 7 * DAY;
    assert!(dave.open_at(t(8 * HOUR + window - 1), window).is_some());
    assert!(dave.open_at(t(8 * HOUR + window), window).is_none());
}

#[test]
fn the_managers_own_lateness_is_complained_about_to_nobody_and_never_penalized() {
    let mut town = town(Some(DEDUCTION_ON), &["dave"]).expect("begins");
    let facts = town.work_one_day();
    let dave: Vec<_> = complaints(&town, &facts)
        .into_iter()
        .filter(|(who, ..)| *who == "dave")
        .collect();
    assert_eq!(
        dave,
        [("dave", 4 * HOUR as u64, 1, Level::Notice, true, None)]
    );
    let deductions: Vec<&str> = of::<WageDeductionDue>(&facts)
        .iter()
        .map(|(due, _)| name_of(&town, due.employee()))
        .collect();
    assert!(deductions.is_empty(), "nothing automatic with a manager");
    let staff: Vec<&str> = disclosed(&town.world, town.id("dave"))
        .late_staff()
        .iter()
        .map(|late| name_of(&town, late.employee()))
        .collect();
    assert_eq!(staff, ["carol"], "never the manager's own");
}

#[test]
fn a_second_absence_within_the_window_is_a_warning() {
    let mut town = Town::begun(true);
    town.work_one_day();
    let facts = town.advance(DAY + 13 * HOUR);
    let dave: Vec<_> = complaints(&town, &facts)
        .into_iter()
        .filter(|(who, ..)| *who == "dave")
        .collect();
    assert_eq!(
        dave,
        [("dave", 4 * HOUR as u64, 2, Level::Warning, true, None)]
    );
}

#[test]
fn two_managers_for_one_employer_are_refused_at_genesis() {
    let refused = town(None, &["alice", "bob"]).err().expect("refused");
    assert!(
        format!("{refused:?}").contains("second-manager"),
        "{refused:?}"
    );
}
