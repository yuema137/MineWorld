//! AC-Na-7 (pack-level `AC-2`): flat-and-park without `sleep` runs the same scripted walks and talks,
//! and every other pack's facts are those of the world with it.

mod support;

use mineworld_contracts::EventEnvelope;
use support::{Kind, Town};

/// The script: talks, walks there and back, a talk elsewhere, a day passing.
fn script(town: &mut Town) {
    town.talk("ben", "ana");
    town.talk("ana", "ben");
    town.walk("cy", "park");
    town.walk("ana", "street");
    town.talk("ana", "ben");
    town.walk("ana", "flat");
    town.talk("ben", "ana");
    town.pass(86_400);
}

/// Every fact as (instant, type, payload bytes), without `sleep`'s own.
fn others(log: &[EventEnvelope]) -> Vec<(i64, String, Vec<u8>)> {
    log.iter()
        .filter(|fact| {
            !matches!(
                fact.event_type().as_str(),
                "fell-asleep" | "alarm-set" | "woke" | "sleep-interactions-configured"
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
fn without_sleep_every_other_pack_states_the_same_facts() {
    let mut with = Town::new(Kind::FlatAndPark);
    let mut without = Town::without_sleep(Kind::FlatAndPark);
    script(&mut with);
    script(&mut without);
    let (a, b) = (others(&with.log), others(&without.log));
    assert!(a.len() > 10, "the script states facts: {}", a.len());
    assert_eq!(a, b);
}
