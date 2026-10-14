//! Forgetting in relationships, scripted over a real café (`pr-fg-a-relationship-forgetting.md` §6;
//! `docs/DECISIONS.md` `ARC-86`).
//!
//! ```text
//! FA-5  lazy, and no decay fact: half-life 1 day for everyone. Alice and Bob reach familiarity 300,
//!       regard 140 (Friendly) at t0; a day passes with nobody meeting:
//!   (a) Alice's observation at t0 + 1 d discloses familiarity 150, same record shape as unconfigured
//!   (b) the stored values still hold 300, last contact t0
//!   (c) no relationships fact between t0 and t0 + 1 d
//!   (d) a talk at t0 + 1 d + 1 s stores 150 + 10 = 160 with last contact then, and states
//!       relationship-changed Friendly → Acquaintance, caused by that talk's spoke
//!   (e) the same script unconfigured discloses 300 at t0 + 1 d
//!       M-FA5a (disclosure returns stored) → (a); M-FA5b (contact adds to stored) → (d);
//!       M-FA5c (the stated level read after the rebase) → (d)'s fact missing
//! ```
//!
//! Hand arithmetic: two hours together are familiarity 2 × 50 = 100 and regard 2 × (50 + 20) = 140;
//! twenty talks add 20 × 10 = 200 more familiarity, all within 40 s of each other — under a 64th of
//! the half-life (1 350 s), so the table's first step, exactly 1 (`HALF[0]`), applies between them.
//! A day later: k = 1, i = 0, 300 >> 1 = 150; one second more is still i = 0.

mod support;

use mineworld_contracts::{ActionResult, Causation, EventEnvelope, WorldTime};
use mineworld_relationships::{Level, RelationshipChanged};
use serde_json::Value;
use support::{ALL, Cafe, of_type};

const DAY: i64 = 86_400;

const HALF_LIFE_A_DAY: &str =
    r#"{ "parameters": [ { "familiarity_decay": "half-life", "familiarity_decay_days": 1 } ] }"#;

fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

/// Two hours together, then twenty talks: familiarity 300, regard 140 both ways — Friendly.
fn become_friendly(cafe: &mut Cafe) {
    let (alice, bob) = (cafe.alice, cafe.bob);
    for _ in 0..2 {
        assert!(accepted(&cafe.invite(alice, bob).0));
        assert!(accepted(&cafe.accept(bob, alice).0));
        assert!(accepted(&cafe.leave(bob).0));
    }
    for _ in 0..20 {
        assert!(accepted(&cafe.talk(alice, bob).0));
    }
}

/// Alice's own disclosed acquaintances record, as the observation carries it.
fn disclosed(cafe: &Cafe) -> Value {
    let alice = cafe.alice;
    cafe.observation(alice)
        .entity(alice)
        .expect("herself")
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "acquaintances")
        .expect("her acquaintances are disclosed to her")
        .payload()
        .clone()
}

/// The familiarity `record` discloses for its only entry.
fn familiarity(record: &Value) -> i64 {
    record["known"][0]["values"]["familiarity"]
        .as_i64()
        .expect("an integer")
}

/// The shape of a record: its keys, recursively, without the values.
fn shape(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), shape(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(shape).collect()),
        _ => Value::Null,
    }
}

fn changed(fact: &EventEnvelope) -> RelationshipChanged {
    serde_json::from_slice(
        fact.payload()
            .payload_for::<RelationshipChanged>()
            .expect("its own type"),
    )
    .expect("decodes")
}

#[test]
fn a_day_without_contact_halves_familiarity_in_disclosure_and_at_the_next_contact_only() {
    let mut cafe = Cafe::configured(ALL, HALF_LIFE_A_DAY);
    let (alice, bob) = (cafe.alice, cafe.bob);
    become_friendly(&mut cafe);
    let t0 = cafe.now();
    let stored = cafe.values(alice, bob).expect("alice knows bob");
    assert_eq!(
        (stored.familiarity(), stored.regard(), stored.level()),
        (300, 140, Level::Friendly)
    );
    assert_eq!(stored.last_contact(), t0);

    // (c) A day passes with nobody meeting, and relationships states nothing.
    let passed = cafe.pass(DAY);
    assert!(
        passed
            .iter()
            .all(|fact| fact.provenance().emitted_by().as_str() != "relationships"),
        "decay writes no fact: {passed:?}"
    );

    // (a) What Alice is shown has faded; (b) what is stored has not.
    let record = disclosed(&cafe);
    assert_eq!(familiarity(&record), 150);
    assert_eq!(
        record["known"][0]["values"]["regard"], 140,
        "regard does not decay here"
    );
    let still = cafe.values(alice, bob).expect("still known");
    assert_eq!((still.familiarity(), still.last_contact()), (300, t0));

    // (d) The next contact adds to the faded value, and states the level the fade crossed, once.
    let (result, facts) = cafe.talk(alice, bob);
    assert!(accepted(&result));
    let at = cafe.now();
    assert_eq!(at, WorldTime::from_seconds(t0.seconds() + DAY + 1));
    let after = cafe.values(alice, bob).expect("still known");
    assert_eq!(
        (after.familiarity(), after.regard(), after.last_contact()),
        (160, 140, at)
    );
    let spoke = of_type(&facts, "spoke");
    assert_eq!(spoke.len(), 1);
    let crossings: Vec<RelationshipChanged> = of_type(&facts, "relationship-changed")
        .into_iter()
        .inspect(|fact| assert_eq!(*fact.caused_by(), Causation::Event(spoke[0].id())))
        .map(changed)
        .collect();
    assert_eq!(
        crossings,
        [
            RelationshipChanged::new(
                cafe.person(alice),
                cafe.person(bob),
                Level::Friendly,
                Level::Acquaintance
            ),
            RelationshipChanged::new(
                cafe.person(bob),
                cafe.person(alice),
                Level::Friendly,
                Level::Acquaintance
            ),
        ]
    );

    // (e) Unconfigured, the same day changes nothing, and the record has the same shape.
    let mut twin = Cafe::new(ALL);
    become_friendly(&mut twin);
    twin.pass(DAY);
    let unconfigured = disclosed(&twin);
    assert_eq!(familiarity(&unconfigured), 300);
    assert_eq!(shape(&unconfigured), shape(&record));
}
