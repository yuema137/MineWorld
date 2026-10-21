//! AC-c-4 (M-c-2, M-c-3), I-c-5 (M-c-4): a kind keeps at most `depth` notes, kinds never evict each
//! other, a note leaves the view exactly when it ages out and leaves state at the next note; the book
//! goes to its holder only.

mod support;

use mineworld_price_survey::PriceBook;
use support::{DAY, Setup, Town, records_of};

fn times(book: &PriceBook, town: &Town, item: &str) -> Vec<i64> {
    book.kind(town.item(item))
        .map(|kind| {
            kind.notes()
                .iter()
                .map(|note| note.at().seconds())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_kind_keeps_its_newest_depth_notes_and_kinds_never_evict_each_other() {
    let mut town = Town::with(&Setup::default().stock("{ apple: 3, bread: 8 }"));
    town.buy("alice", None, "apple", 100, 10);
    for (n, buyer) in ["alice", "alice", "alice", "bob", "bob", "bob"]
        .iter()
        .enumerate()
    {
        let at = 100 * (i64::try_from(n).expect("small") + 1);
        town.buy(buyer, None, "bread", 300, at);
    }
    let book = town.book("pia", 600).expect("a book");
    assert_eq!(
        times(&book, &town, "bread"),
        [300, 400, 500, 600],
        "the four newest"
    );
    assert_eq!(
        times(&book, &town, "apple"),
        [10],
        "bread never evicted the apple"
    );
}

#[test]
fn a_note_leaves_the_view_at_its_window_and_state_at_the_next_note() {
    let mut town = Town::begun();
    town.buy("alice", None, "bread", 300, 100);
    let facts = town.log.len();
    assert_eq!(
        times(
            &town.book("pia", 100 + DAY - 1).expect("a book"),
            &town,
            "bread"
        ),
        [100]
    );
    assert!(
        times(
            &town.book("pia", 100 + DAY).expect("a book"),
            &town,
            "bread"
        )
        .is_empty(),
        "gone exactly at t + window"
    );
    assert_eq!(town.log.len(), facts, "no fact between");
    let held = |town: &Town| {
        town.world
            .read()
            .component::<PriceBook>(town.id("pia"))
            .cloned()
            .expect("pia's book")
    };
    assert_eq!(times(&held(&town), &town, "bread"), [100], "still in state");
    town.buy("alice", None, "apple", 100, 100 + DAY);
    let state = held(&town);
    assert!(
        times(&state, &town, "bread").is_empty(),
        "pruned at the next note"
    );
    assert_eq!(times(&state, &town, "apple"), [100 + DAY]);
}

#[test]
fn only_the_holder_sees_its_book() {
    let mut town = Town::begun();
    for observer in ["alice", "bob", "grace", "nat"] {
        assert_eq!(
            records_of(&town.observation(observer, 0), "survey-book"),
            0,
            "{observer}"
        );
    }
    assert_eq!(records_of(&town.observation("pia", 0), "survey-book"), 1);
}
