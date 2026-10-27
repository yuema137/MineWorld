//! S31-b2 Part B (`pr-s31b2-notices-attributes.md` §5.6): the `topics` record. Every living person's
//! own observation lists every declared topic in id order, with whether they may publish on it and
//! whether they subscribe now; a non-subscriber sees it too; nobody sees it about another person; a
//! world without the configuration has none (AC-b2-6's pack half; M-b2-4, M-b2-5).

mod support;

use mineworld_notices::{BodyKind, TopicLine, TopicsRecord};
use support::{Cafe, PEOPLE, Recipe, record, records_of};

/// The fixture café's topics as `name` should see them, from (topic, body, valid, per_day, publishes,
/// subscribed).
fn line(
    topic: &str,
    body: BodyKind,
    valid: (u32, u32),
    per_day: u8,
    may_publish: bool,
    subscribed: bool,
) -> TopicLine {
    TopicLine {
        topic: support::topic(topic),
        body,
        valid_min: valid.0,
        valid_max: valid.1,
        per_day,
        may_publish,
        subscribed,
    }
}

fn topics_of(cafe: &mut Cafe, observer: &str, at: i64) -> Option<Vec<TopicLine>> {
    let me = cafe.id(observer);
    let observation = cafe.observation(observer, at);
    record::<TopicsRecord>(&observation, me, "topics").map(|record| record.topics)
}

const HOUR: u32 = 3_600;

/// W1 (S31-b's fixture): every person sees both topics, in id order, with its own `may_publish` and
/// `subscribed`; `subscribe` flips `subscribed` (from the component, not the board: M-b2-5); no
/// observation carries a `topics` record about anyone but the observer (M-b2-4).
#[test]
fn every_person_sees_every_topic_about_themself_only() {
    let mut cafe = Cafe::begun();
    let expect = |publishes: bool, subscribed: bool| {
        vec![
            line(
                "news.daily",
                BodyKind::Text,
                (HOUR, 7 * 24 * HOUR),
                2,
                publishes,
                subscribed,
            ),
            line(
                "prices.ingredients",
                BodyKind::Prices,
                (HOUR, 48 * HOUR),
                4,
                publishes,
                subscribed,
            ),
        ]
    };
    assert_eq!(
        topics_of(&mut cafe, "pia", 0),
        Some(expect(true, false)),
        "the publisher"
    );
    assert_eq!(
        topics_of(&mut cafe, "sam", 0),
        Some(expect(false, true)),
        "a subscriber"
    );
    assert_eq!(
        topics_of(&mut cafe, "nat", 0),
        Some(expect(false, false)),
        "a non-subscriber sees the record too"
    );
    for observer in PEOPLE {
        let observation = cafe.observation(observer, 0);
        assert_eq!(
            records_of(&observation, "topics"),
            1,
            "{observer}: one, about themself"
        );
    }

    support::accepted(cafe.subscribe("nat", "news.daily", 10), "nat subscribes");
    let after = topics_of(&mut cafe, "nat", 11).expect("nat's record");
    assert!(after[0].subscribed, "news.daily is now followed");
    assert!(!after[1].subscribed, "prices.ingredients is not");
}

/// W2 (a generated variant: topics renamed, a third topic, another publisher): the record follows the
/// world's own topics, in id order.
#[test]
fn the_record_follows_the_worlds_topics() {
    let configuration = "topics:\n  weather.warnings:\n    body: text\n    publishers: [nat]\n    \
        valid_for: { min: \"1h\", max: \"1d\" }\n    per_day: 3\n  market.daily:\n    body: prices\n    \
        publishers: [pia]\n    valid_for: { min: \"1h\", max: \"48h\" }\n    per_day: 4\n  news.daily:\n    \
        body: text\n    publishers: [pia]\n    valid_for: { min: \"1h\", max: \"7d\" }\n    per_day: 2\n";
    let mut cafe = Cafe::try_begin(&Recipe {
        configuration: Some(configuration),
        sections: vec![("sue", "subscribe: [weather.warnings]")],
        tags: Vec::new(),
    })
    .expect("genesis");
    let nat = topics_of(&mut cafe, "nat", 0).expect("nat's record");
    let names: Vec<&str> = nat.iter().map(|line| line.topic.as_str()).collect();
    assert_eq!(names, ["market.daily", "news.daily", "weather.warnings"]);
    assert_eq!(
        nat.iter().map(|line| line.may_publish).collect::<Vec<_>>(),
        [false, false, true]
    );
    let sue = topics_of(&mut cafe, "sue", 0).expect("sue's record");
    assert_eq!(
        sue.iter().map(|line| line.subscribed).collect::<Vec<_>>(),
        [false, false, true]
    );
}

/// A world without `configure/notices.yaml` has no `topics` record.
#[test]
fn an_unconfigured_world_has_no_record() {
    let mut cafe = Cafe::try_begin(&Recipe {
        configuration: None,
        sections: Vec::new(),
        tags: Vec::new(),
    })
    .expect("genesis");
    let observation = cafe.observation("sam", 0);
    assert_eq!(records_of(&observation, "topics"), 0);
}
