//! AC-c-2 (X-7, M-c-6): a surveyor notes exactly the tracked sales it perceived, by presence's own
//! audience rule — each note caused by its `sold`, in the same dispatch.

mod support;

use mineworld_contracts::EventEnvelope;
use mineworld_contracts::{Causation, Visibility};
use mineworld_economy::Sold;
use mineworld_price_survey::{PriceNoted, Source};
use support::{Setup, Town, of};

/// The `price-noted` facts of one dispatch, checked against its one `sold`: caused by it, in the same
/// dispatch.
fn noted_in(town: &Town, facts: &[EventEnvelope]) -> Vec<PriceNoted> {
    let sold = of::<Sold>(facts);
    assert_eq!(sold.len(), 1, "one sale");
    let sold_id = sold[0].1.id();
    of::<PriceNoted>(facts)
        .into_iter()
        .map(|(noted, envelope)| {
            assert_eq!(*envelope.caused_by(), Causation::Event(sold_id));
            assert_eq!(*envelope.visibility(), Visibility::SystemInternal);
            assert_eq!(noted.source(), Source::Sale);
            assert_eq!(noted.place(), town.place("store"));
            noted
        })
        .collect()
}

#[test]
fn a_present_surveyor_notes_the_sale() {
    let mut town = Town::begun();
    let dispatched = town.buy("alice", None, "bread", 300, 100);
    let noted = noted_in(&town, &dispatched);
    assert_eq!(noted.len(), 1);
    assert_eq!(noted[0].surveyor().entity_id(), town.id("pia"));
    let note = noted[0].notes()[0];
    assert_eq!(note.item(), town.item("bread"));
    assert_eq!(note.seller(), town.id("corner-store"));
    assert_eq!((note.price(), note.in_stock()), (300, None));
    let book = town.book("pia", 100).expect("a book");
    let bread = book.kind(town.item("bread")).expect("bread noted");
    assert_eq!(bread.notes().len(), 1);
    assert_eq!(bread.notes()[0].at().seconds(), 100);
}

#[test]
fn an_absent_surveyor_notes_nothing() {
    let mut town = Town::with(&Setup::default().pia_at("street"));
    let dispatched = town.buy("alice", None, "apple", 100, 100);
    assert!(
        noted_in(&town, &dispatched).is_empty(),
        "pia was on the street"
    );
    assert!(town.book("pia", 100).expect("a book").kinds().is_empty());
}

#[test]
fn the_surveyors_own_purchase_is_noted() {
    let mut town = Town::begun();
    let dispatched = town.buy("pia", None, "apple", 100, 100);
    let noted = noted_in(&town, &dispatched);
    assert_eq!(noted.len(), 1, "a participant perceives its own sale");
}

#[test]
fn an_untracked_kind_is_not_noted() {
    let mut town = Town::begun();
    let dispatched = town.buy("bob", None, "pen", 150, 100);
    assert!(
        noted_in(&town, &dispatched).is_empty(),
        "goods are untracked"
    );
}

#[test]
fn a_person_sellers_sale_is_noted_at_its_price() {
    let mut town = Town::begun();
    let dispatched = town.buy("alice", Some("grace"), "bread", 260, 100);
    let noted = noted_in(&town, &dispatched);
    assert_eq!(noted.len(), 1);
    let note = noted[0].notes()[0];
    assert_eq!((note.seller(), note.price()), (town.id("grace"), 260));
}

#[test]
fn nobody_without_a_book_is_named_and_the_book_equals_the_perceived_sales() {
    let mut town = Town::begun();
    town.buy("alice", None, "bread", 300, 100);
    town.buy("bob", None, "pen", 150, 200);
    town.buy("alice", Some("grace"), "bread", 260, 300);
    let nat = town.id("nat");
    for (noted, envelope) in of::<PriceNoted>(&town.log) {
        assert_ne!(noted.surveyor().entity_id(), nat);
        assert!(!envelope.subjects().contains(&nat));
    }
    // The oracle is presence's fold over the real log: pia's tracked perceived sales.
    let perceived: Vec<_> = of::<Sold>(&town.perceived("pia"))
        .into_iter()
        .filter(|(sold, _)| sold.item() != town.item("pen"))
        .map(|(_, envelope)| envelope.id())
        .collect();
    let causes: Vec<_> = of::<PriceNoted>(&town.log)
        .into_iter()
        .map(|(_, envelope)| match envelope.caused_by() {
            Causation::Event(id) => *id,
            other => panic!("caused by {other:?}"),
        })
        .collect();
    assert_eq!(perceived, causes);
    assert_eq!(perceived.len(), 2);
}
