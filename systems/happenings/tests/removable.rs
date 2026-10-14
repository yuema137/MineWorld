//! T12: the pack is removable and silent when unconfigured (INV-WE-4, INV-WE-5; AC-WE-2, AC-WE-3).
//! (a) enabled without `configure/happenings.yaml`: no Process, no fact, no disclosure. (b) with and
//! without the pack, every fact that is not the pack's is the same. (c), refusal without calendar, is
//! in `configuration.rs`, through the real loader.

mod support;

use mineworld_contracts::EventEnvelope;
use mineworld_presence::observe;
use support::{DAY, HOUR, compose, key, ours, providers, run, t};

/// The pack's own processes in a world.
fn processes(world: &mineworld_kernel::World) -> usize {
    world
        .read()
        .processes()
        .filter(|process| process.owner().as_str() == "happenings")
        .count()
}

/// A small configuration: a market every Saturday on the street.
pub const MARKET: &str = "seed: 7
kinds:
  - { id: market-day, places: [street], hours: \"08:00-14:00\" }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
";

/// (a) Enabled, unconfigured: over ten days the pack states nothing, starts nothing and discloses
/// nothing. Configured, it has its almanac — the control.
#[test]
fn an_unconfigured_pack_states_starts_and_discloses_nothing() {
    let end = 10 * DAY;
    let (world, keys, facts) = run(None, end);
    assert!(facts.iter().all(|fact| !ours(fact)), "no fact of the pack");
    assert_eq!(processes(&world), 0, "no process of the pack");
    let seen = observe(&world, keys[&key("ada")], t(end), &providers());
    assert!(
        seen.entities()
            .iter()
            .flat_map(|entity| entity.components())
            .all(|record| !record.component_type().as_str().starts_with("happening")),
        "no disclosure"
    );

    let (configured, _, facts) = run(Some(MARKET), HOUR);
    assert!(
        facts.iter().any(ours),
        "configured, it states its configuration"
    );
    assert!(
        processes(&configured) >= 1,
        "configured, it keeps an almanac"
    );
}

/// Calendar's and presence's facts through `end` in a world without happenings installed.
fn without_happenings(end: i64) -> Vec<EventEnvelope> {
    let (mut world, _, facts) = support::assemble(compose(false), Some(support::SAN_DIEGO), None);
    let mut all = world.genesis(t(0), facts).expect("genesis");
    all.extend(world.advance_to(t(end)).expect("advances").into_events());
    all
}

/// A fact's projection without its id: when, what, to whom, about whom, and the payload's bytes.
/// Ids are one world counter, so the pack's own facts shift them (weather's M-TWb-1).
fn projection(facts: &[EventEnvelope]) -> Vec<String> {
    facts
        .iter()
        .filter(|fact| !ours(fact))
        .map(|fact| {
            format!(
                "{} {} {:?} {:?} {}",
                fact.at().seconds(),
                fact.event_type(),
                fact.visibility(),
                fact.subjects(),
                String::from_utf8_lossy(fact.payload().payload())
            )
        })
        .collect()
}

/// (a, continued) Enabled and unconfigured, every other pack's facts are what they are without the
/// pack installed at all.
#[test]
fn an_unconfigured_pack_changes_no_other_fact() {
    let end = 10 * DAY;
    let (_, _, with) = run(None, end);
    assert_eq!(projection(&with), projection(&without_happenings(end)));
}
