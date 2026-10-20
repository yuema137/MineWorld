//! AC-Nb-6 (pack-level `AC-2`): flat-and-cafe without `needs` — and without its sections — runs the same
//! scripted meals and sleeps, and every other pack states the same facts as in the world with it.

mod support;

use mineworld_contracts::EventEnvelope;
use support::{Kind, Town, accepted, hour};

/// Meals, a night's sleep, an early wake, and two days passing.
fn script(town: &mut Town) {
    town.pass_to(hour(5));
    assert!(accepted(&town.eat("ana", "soup")));
    town.pass_to(hour(6));
    assert!(accepted(&town.sleep("ana", 480)));
    assert!(accepted(&town.sleep("ben", 600)));
    town.pass_to(hour(7));
    assert!(accepted(&town.eat("ben", "apple")));
    assert!(accepted(&town.wake("ben")));
    town.pass_to(hour(8));
    assert!(accepted(&town.eat("cy", "soup")));
    town.pass_to(hour(48));
}

/// Every fact as (instant, type, payload bytes), without `needs`' own.
fn others(log: &[EventEnvelope]) -> Vec<(i64, String, Vec<u8>)> {
    log.iter()
        .filter(|fact| {
            !matches!(
                fact.event_type().as_str(),
                "needs-assigned" | "nourishment-assigned" | "needs-interactions-configured"
            )
        })
        .map(|fact| {
            (
                fact.at().seconds(),
                fact.event_type().as_str().to_owned(),
                fact.payload().payload().clone(),
            )
        })
        .collect()
}

#[test]
fn without_needs_every_other_pack_states_the_same_facts() {
    let mut with = Town::new(Kind::FlatAndCafe);
    let mut without = Town::without_needs(Kind::FlatAndCafe);
    script(&mut with);
    script(&mut without);
    let (a, b) = (others(&with.log), others(&without.log));
    assert!(a.len() > 10, "the script states facts: {}", a.len());
    assert!(a.iter().any(|(_, kind, _)| kind == "items-consumed"));
    assert!(a.iter().any(|(_, kind, _)| kind == "woke"));
    assert_eq!(a, b);
}
