//! Who is told what (AC-Nb-5, `INV-13`): a person's `needs` record reaches that person alone, and its
//! `nourishment` lists only the kinds that person carries. Sampled every hour of one day in both worlds,
//! with a meal and a night's sleep on the way. ana and ben (dee and eli) share a place, so a record that
//! widened to the place would reach the other; cy (fay) has no needs at all.

mod support;

use mineworld_contracts::ItemId;
use support::{Kind, Town, hour};

/// The kinds a record says nourish its holder, in order.
fn nourishing(record: &serde_json::Value) -> Vec<(u64, u64)> {
    record["nourishment"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|line| {
            let item: ItemId = serde_json::from_value(line["item"].clone()).expect("an item");
            (
                item.entity_id().raw(),
                line["hunger"].as_u64().expect("a number"),
            )
        })
        .collect()
}

#[test]
fn needs_are_disclosed_to_their_holder_alone_all_day() {
    for (kind, [first, second, third], food, nourishes) in [
        (Kind::FlatAndCafe, ["ana", "ben", "cy"], "soup", 600),
        (Kind::NightShift, ["dee", "eli", "fay"], "stew", 500),
    ] {
        let mut town = Town::new(kind);
        for h in 0..24 {
            town.pass_to(hour(h));
            if h == 7 {
                assert!(support::accepted(&town.eat(first, food)));
            }
            if h == 22 {
                assert!(support::accepted(&town.sleep(first, 480)));
            }
            assert!(town.disclosed(first, first).is_some(), "{kind:?} {h}:00");
            assert!(town.disclosed(second, second).is_some());
            assert!(
                town.disclosed(second, first).is_none(),
                "{second} never sees {first}'s needs"
            );
            assert!(town.disclosed(first, second).is_none());
            assert!(town.disclosed(third, first).is_none());
            assert!(
                town.disclosed(third, third).is_none(),
                "no section, no record"
            );
            let record = town.disclosed(first, first).expect("disclosed");
            assert_eq!(
                nourishing(&record),
                vec![(town.item(food).entity_id().raw(), nourishes)],
                "only the kinds {first} holds, with nourishment"
            );
        }
    }
}
