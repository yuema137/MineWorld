//! T9 (AC-WE-4): the host's limits hold over 1 000 days, counted on every day and every place, not
//! sampled; every draft is either announced or withheld with the reason that refused it.

mod support;

use std::collections::BTreeMap;

use mineworld_happenings::{HappeningAnnounced, HappeningBegan, HappeningWithheld, WithheldReason};
use support::{DAY, begun, of};

/// Seven kinds, every one every day, crowding three places with overlapping hours, at the default
/// limits (3 a day, 1 at a time per place).
const CROWDED: &str = "seed: 7
kinds:
  - { id: a-market, places: [street], hours: \"08:00-14:00\" }
  - { id: b-busker, places: [street], hours: \"12:00-13:00\" }
  - { id: c-fair, places: [park], hours: \"10:00-18:00\" }
  - { id: d-picnic, places: [park], hours: \"12:00-15:00\" }
  - { id: e-parade, places: [square], hours: \"09:00-11:00\" }
  - { id: f-concert, places: [square], hours: \"20:00-22:00\" }
  - { id: g-vigil, places: [square], hours: \"21:00-24:00\" }
calendar:
  - { kind: a-market, place: street, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: b-busker, place: street, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: c-fair, place: park, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: d-picnic, place: park, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: e-parade, place: square, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: f-concert, place: square, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
  - { kind: g-vigil, place: square, weekdays: [monday, tuesday, wednesday, thursday, friday, saturday, sunday] }
limits: { per_day: 3, per_place_running: 1 }
";

#[test]
fn limits_hold_for_a_thousand_days() {
    let (mut world, _, mut facts) = begun(Some(CROWDED));
    for chunk in 1..=10 {
        facts.extend(
            world
                .advance_to(support::t(chunk * 100 * DAY - 1))
                .expect("advances")
                .into_events(),
        );
    }
    let announced = of::<HappeningAnnounced>(&facts);
    let withheld = of::<HappeningWithheld>(&facts);
    // Every draft is accounted for: seven a day, for days 0 … 999.
    assert_eq!(announced.len() + withheld.len(), 7 * 1_000);

    // Per day, counted from what began.
    let mut beginning: BTreeMap<i64, usize> = BTreeMap::new();
    for (at, _) in of::<HappeningBegan>(&facts) {
        *beginning.entry(at.seconds().div_euclid(DAY)).or_default() += 1;
    }
    assert_eq!(beginning.len(), 1_000, "something begins every day");
    assert!(
        beginning.values().all(|count| *count <= 3),
        "per_day: {:?}",
        beginning.iter().find(|(_, count)| **count > 3)
    );

    // Per place, every pair of announced happenings at one place is disjoint in time.
    let mut by_place: BTreeMap<u64, Vec<(i64, i64)>> = BTreeMap::new();
    for (_, fact) in &announced {
        by_place
            .entry(fact.happening().place().entity_id().raw())
            .or_default()
            .push((fact.happening().start().seconds(), fact.until().seconds()));
    }
    for spans in by_place.values_mut() {
        spans.sort_unstable();
        for pair in spans.windows(2) {
            assert!(pair[0].1 <= pair[1].0, "overlap at one place: {pair:?}");
        }
    }

    // Every refusal says why, and only the two limits refuse these drafts.
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for (_, fact) in &withheld {
        assert!(
            matches!(
                fact.reason(),
                WithheldReason::OverDailyLimit | WithheldReason::PlaceBusy
            ),
            "{fact:?}"
        );
        *reasons.entry(format!("{:?}", fact.reason())).or_default() += 1;
    }
    println!(
        "1 000 days: {} announced, {} withheld {reasons:?}",
        announced.len(),
        withheld.len()
    );
    assert_eq!(reasons.len(), 2, "both limits refuse: {reasons:?}");
}
