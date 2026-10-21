//! The board: a notice leaves it exactly at `valid_until` with no fact stated (AC-b-2, X-5), a newer
//! notice from the same publisher on the same topic replaces the older one (AC-b-3, M-b-3), and the
//! board goes to its holder only (I-b-5).

mod support;

use support::{Cafe, accepted, records_of, text};

#[test]
fn a_notice_leaves_the_board_exactly_at_its_end_and_nothing_is_stated() {
    let mut cafe = Cafe::begun();
    accepted(
        cafe.publish("pia", "news.daily", text("Late opening."), 3_600, 10_000),
        "pia publishes",
    );
    let facts_before = cafe.log.len();
    let board = cafe.board("sam", 13_599);
    assert_eq!(board.len(), 1, "current one second before its end");
    assert_eq!(board[0].valid_until().seconds(), 13_600);
    assert!(cafe.board("sam", 13_600).is_empty(), "gone at valid_until");
    assert_eq!(
        cafe.log.len(),
        facts_before,
        "no fact was stated between them"
    );
    let perceived = cafe.perceived("sam");
    assert!(
        perceived
            .iter()
            .any(|fact| fact.event_type().as_str() == "notice-published"),
        "the inbox keeps the notice"
    );
}

#[test]
fn a_newer_notice_from_the_same_publisher_replaces_the_older() {
    let mut cafe = Cafe::begun();
    accepted(
        cafe.publish("pia", "news.daily", text("first"), 3_600, 10_000),
        "first",
    );
    accepted(
        cafe.publish("pia", "news.daily", text("second"), 7_200, 11_000),
        "second",
    );
    let board = cafe.board("sam", 12_000);
    assert_eq!(board.len(), 1, "only the newest");
    assert_eq!(board[0].body(), &text("second"));
    assert_eq!(board[0].id().get(), 2, "numbered in posting order");
    assert!(cafe.board("sam", 18_200).is_empty(), "and then neither");
}

#[test]
fn the_publisher_sees_its_own_topics_and_nobody_sees_another_persons_board() {
    let mut cafe = Cafe::begun();
    accepted(
        cafe.publish("pia", "news.daily", text("hello"), 3_600, 10_000),
        "pia publishes",
    );
    assert_eq!(
        cafe.board("pia", 10_100).len(),
        1,
        "pia subscribes to nothing but publishes the topic (F-S31b-8)"
    );
    assert!(
        cafe.board("nat", 10_100).is_empty(),
        "nat subscribes to nothing"
    );
    let observation = cafe.observation("sam", 10_200);
    assert_eq!(
        records_of(&observation, "notice-board"),
        1,
        "sam's own board, and no one else's"
    );
}
