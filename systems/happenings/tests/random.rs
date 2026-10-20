//! The bundled `random` source (design §5.3): an entry's draws are keyed by its own names, not its
//! position (T7), and fire at their configured frequency (T8).

mod support;

use std::collections::BTreeMap;

use mineworld_happenings::HappeningAnnounced;
use support::{DAY, begun, of, run};

/// The days each (kind, place) is announced for, from `facts`.
fn firing_days(facts: &[mineworld_contracts::EventEnvelope]) -> BTreeMap<(String, u64), Vec<i64>> {
    let mut days: BTreeMap<(String, u64), Vec<i64>> = BTreeMap::new();
    for (_, fact) in of::<HappeningAnnounced>(facts) {
        let happening = fact.happening();
        days.entry((
            happening.kind().to_string(),
            happening.place().entity_id().raw(),
        ))
        .or_default()
        .push(happening.start().seconds().div_euclid(DAY));
    }
    days
}

const KINDS: &str = "seed: 7
kinds:
  - { id: street-musician, places: [street, park], hours: \"11:00-13:00\" }
  - { id: juggler, places: [square], hours: \"15:00-16:00\" }
";

/// T7: reordering `random:` and adding an entry leaves the other entries' firing days unchanged.
#[test]
fn rolls_are_stable_under_reordering_and_additions() {
    let first = format!(
        "{KINDS}random:
  - {{ kind: street-musician, place: street, per_mille_per_day: 150 }}
  - {{ kind: street-musician, place: park, per_mille_per_day: 60 }}
"
    );
    let second = format!(
        "{KINDS}random:
  - {{ kind: street-musician, place: park, per_mille_per_day: 60 }}
  - {{ kind: juggler, place: square, per_mille_per_day: 300 }}
  - {{ kind: street-musician, place: street, per_mille_per_day: 150 }}
"
    );
    let end = 365 * DAY;
    let (_, _, a) = run(Some(&first), end);
    let (_, _, b) = run(Some(&second), end);
    let (mut a, b) = (firing_days(&a), firing_days(&b));
    assert_eq!(a.len(), 2, "both musicians fire in a year: {a:?}");
    assert_eq!(b.len(), 3, "and the juggler: {b:?}");
    let juggler = b
        .keys()
        .find(|(kind, _)| kind == "juggler")
        .cloned()
        .expect("the juggler");
    let mut b = b;
    b.remove(&juggler);
    assert_eq!(a, b, "the musicians fire on the same days");
    // A different seed moves them: the days are the seed's, not fixed.
    let (_, _, c) = run(Some(&first.replace("seed: 7", "seed: 8")), end);
    a.retain(|_, days| !days.is_empty());
    assert_ne!(a, firing_days(&c));
}

/// T8: over 10 000 days a 150 ‰ entry fires 1 500 ± 105 times (about 3σ of a binomial with n = 10 000,
/// p = 0.15: σ = √(10 000 × 0.15 × 0.85) ≈ 35.7, fixed before running); 0 ‰ never; 1 000 ‰ every day.
/// The three are at three places, so no limit intervenes.
#[test]
fn frequency_is_within_tolerance() {
    let text = "seed: 7
kinds:
  - { id: street-musician, places: [street], hours: \"11:00-13:00\" }
  - { id: silent-mime, places: [park], hours: \"11:00-13:00\" }
  - { id: town-crier, places: [square], hours: \"11:00-13:00\" }
random:
  - { kind: street-musician, place: street, per_mille_per_day: 150 }
  - { kind: silent-mime, place: park, per_mille_per_day: 0 }
  - { kind: town-crier, place: square, per_mille_per_day: 1000 }
";
    let (mut world, _, genesis) = begun(Some(text));
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut count = |facts: &[mineworld_contracts::EventEnvelope]| {
        for (_, fact) in of::<HappeningAnnounced>(facts) {
            *counts
                .entry(fact.happening().kind().to_string())
                .or_default() += 1;
        }
    };
    count(&genesis);
    // Days 0 … 9 999 are announced at their midnights; stop before day 10 000's.
    for chunk in 1..=100 {
        let end = chunk * 100 * DAY - 1;
        count(
            &world
                .advance_to(support::t(end))
                .expect("advances")
                .into_events(),
        );
    }
    println!("10 000 days: {counts:?}");
    let musician = counts.get("street-musician").copied().unwrap_or(0);
    assert!(
        (1_395..=1_605).contains(&musician),
        "150 per mille over 10 000 days: {musician}"
    );
    assert_eq!(counts.get("silent-mime"), None, "0 per mille never fires");
    assert_eq!(
        counts.get("town-crier"),
        Some(&10_000),
        "1000 per mille every day"
    );
}
