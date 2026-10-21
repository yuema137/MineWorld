//! AC-c-3 (M-c-1, M-c-5), AC-c-5: `survey` copies exactly the tracked asks economy shows the
//! surveyor where it stands, at most once per place per `resurvey_after`; the offer answers as the
//! dispatch does; nobody but the holder sees the book.

mod support;

use mineworld_contracts::{ActionRecord, ActionResult, Rejection};
use mineworld_price_survey::{PriceNoted, Source, Survey};
use support::{HOUR, Setup, Town, code, of, records_of, rejection, surveys};

const T: i64 = 1_000;

/// The offer's verdict for `who` at `at`: `None` when not offered, else its refusal (or none).
fn offered(town: &mut Town, who: &str, at: i64) -> Option<Option<Rejection>> {
    let observation = town.observation(who, at);
    let offers = surveys(&observation);
    assert!(offers.len() <= 1, "at most one survey offer");
    offers.first().map(|offer| {
        assert!(offer.payload().is_some(), "complete");
        offer.unavailable_reason().cloned()
    })
}

/// The dispatch's verdict: `None` accepted, else its refusal.
fn dispatched(town: &mut Town, who: &str, at: i64) -> Option<Rejection> {
    let result = town.survey(who, at);
    rejection(&result).cloned()
}

#[test]
fn a_survey_copies_the_tracked_asks_on_display() {
    let mut town = Town::begun();
    assert_eq!(
        offered(&mut town, "pia", T),
        Some(None),
        "offered, available"
    );
    let result = town.survey("pia", T);
    assert!(matches!(result.result(), ActionResult::Accepted { .. }));
    let noted = of::<PriceNoted>(result.events());
    assert_eq!(noted.len(), 1);
    let (fact, envelope) = &noted[0];
    assert_eq!(fact.source(), Source::Ask);
    assert_eq!(
        *envelope.visibility(),
        mineworld_contracts::Visibility::SystemInternal
    );
    let lines: Vec<_> = fact
        .notes()
        .iter()
        .map(|n| (n.item(), n.seller(), n.price(), n.in_stock()))
        .collect();
    let (store, grace) = (town.id("corner-store"), town.id("grace"));
    // By (item, seller) id: within bread, grace (a person, created first) sorts before the store's
    // operator.
    assert!(
        grace < store,
        "the fixture creates people before organizations"
    );
    assert_eq!(
        lines,
        [
            (town.item("apple"), store, 100, Some(3)),
            (town.item("bread"), grace, 260, None),
            (town.item("bread"), store, 300, Some(2)),
            (town.item("juice"), store, 200, Some(0)),
        ],
        "no pen; grace's note has no count"
    );
    let book = town.book("pia", T).expect("a book");
    assert_eq!(
        book.last_surveyed(town.place("store"))
            .map(|at| at.seconds()),
        Some(T)
    );
}

#[test]
fn the_same_place_cools_down_for_resurvey_after() {
    let mut town = Town::begun();
    assert_eq!(dispatched(&mut town, "pia", T), None);
    let before = town.state();
    let recently = code("surveyed-recently");
    assert_eq!(
        offered(&mut town, "pia", T + HOUR - 1),
        Some(Some(recently.clone()))
    );
    assert_eq!(dispatched(&mut town, "pia", T + HOUR - 1), Some(recently));
    assert_eq!(town.state(), before, "state unchanged");
    assert_eq!(offered(&mut town, "pia", T + HOUR), Some(None));
    assert_eq!(
        dispatched(&mut town, "pia", T + HOUR),
        None,
        "accepted again"
    );
}

#[test]
fn nothing_on_display_is_not_offered_and_refused_by_name() {
    let mut town = Town::with(&Setup::default().pia_at("street"));
    assert_eq!(offered(&mut town, "pia", T), None);
    assert_eq!(
        dispatched(&mut town, "pia", T),
        Some(code("nothing-to-survey"))
    );
}

#[test]
fn a_person_without_a_book_is_never_offered_survey() {
    let mut town = Town::begun();
    assert_eq!(offered(&mut town, "nat", T), None);
    assert_eq!(
        dispatched(&mut town, "nat", T),
        Some(Rejection::NoSupportedInteraction)
    );
}

#[test]
fn a_malformed_payload_is_a_failed_precondition() {
    let mut town = Town::begun();
    let record = ActionRecord::new::<Survey>(b"{\"all\":true}".to_vec());
    let result = town.act("pia", None, record, T);
    assert_eq!(rejection(&result), Some(&Rejection::PreconditionFailed));
}

#[test]
fn offer_and_dispatch_agree() {
    let cases: [(Setup, &str, i64); 4] = [
        (Setup::default(), "pia", T),
        (Setup::default().pia_at("street"), "pia", T),
        (Setup::default(), "nat", T),
        (Setup::default(), "alice", T),
    ];
    for (setup, who, at) in cases {
        let mut town = Town::with(&setup);
        let offer = offered(&mut town, who, at);
        let dispatch = dispatched(&mut town, who, at);
        match offer {
            Some(verdict) => assert_eq!(verdict, dispatch, "{who}"),
            None => assert!(dispatch.is_some(), "{who}: not offered, so refused"),
        }
    }
    let mut town = Town::begun();
    dispatched(&mut town, "pia", T);
    let offer = offered(&mut town, "pia", T + 10).expect("offered while cooling");
    assert_eq!(offer, dispatched(&mut town, "pia", T + 10));
}

#[test]
fn surveys_ring_and_window_end_to_end() {
    // depth 4: six surveys an hour apart leave the four newest apple notes.
    let mut town = Town::begun();
    for n in 0..6 {
        assert_eq!(
            dispatched(&mut town, "pia", T + n * HOUR),
            None,
            "survey {n}"
        );
    }
    let book = town.book("pia", T + 5 * HOUR).expect("a book");
    let apple: Vec<i64> = book
        .kind(town.item("apple"))
        .expect("apple")
        .notes()
        .iter()
        .map(|note| note.at().seconds())
        .collect();
    assert_eq!(
        apple,
        [T + 2 * HOUR, T + 3 * HOUR, T + 4 * HOUR, T + 5 * HOUR]
    );
    // window 1 d: the newest note leaves the view exactly a day after it was seen, with no fact.
    let facts = town.log.len();
    let last = T + 5 * HOUR;
    assert!(
        town.book("pia", last + 86_399)
            .expect("a book")
            .kind(town.item("apple"))
            .is_some()
    );
    let gone = town.book("pia", last + 86_400).expect("a book");
    assert!(gone.kinds().is_empty() && gone.surveyed().is_empty());
    assert_eq!(town.log.len(), facts, "no fact between");
}

#[test]
fn only_the_holder_sees_its_book_and_no_note_holds_a_persons_count() {
    let mut town = Town::begun();
    dispatched(&mut town, "pia", T);
    for observer in ["alice", "grace", "nat"] {
        assert_eq!(
            records_of(&town.observation(observer, T), "survey-book"),
            0,
            "{observer}"
        );
    }
    assert_eq!(records_of(&town.observation("pia", T), "survey-book"), 1);
    let grace = town.id("grace");
    for (fact, _) in of::<PriceNoted>(&town.log) {
        for note in fact.notes() {
            if note.seller() == grace {
                assert_eq!(note.in_stock(), None);
            }
        }
    }
}
