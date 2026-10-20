//! Publishing: who receives a notice (AC-b-1, I-b-3, X-4), each refusal by name with nothing moved
//! (AC-b-4, M-b-2), the subscriptions route (AC-b-5), and offers agreeing with dispatch (I-b-7).
//!
//! The audience oracle is presence's own audience rule over the log (`perceived_by`, `ARC-43`), never
//! a value this pack reports about itself.

mod support;

use mineworld_contracts::{ActionRecord, ActionResult, Rejection, RejectionCode, Visibility};
use mineworld_notices::{NOTICE_PRICE_MAX, NoticeBody, NoticePublished, PublishNotice};
use support::{Cafe, HOUR, Recipe, accepted, of, text};

fn code(name: &'static str) -> RejectionCode {
    RejectionCode::from_static(name)
}

/// The rejection a dispatch reported.
fn rejection(result: &ActionResult) -> Rejection {
    match result {
        ActionResult::Rejected(rejection) => rejection.clone(),
        other => panic!("expected a rejection, got {other:?}"),
    }
}

/// The code of a pack refusal, or the kernel rejection's debug name.
fn named(result: &ActionResult) -> String {
    match rejection(result) {
        Rejection::System { code, .. } => code.as_str().to_owned(),
        other => format!("{other:?}"),
    }
}

fn notices(cafe: &Cafe, name: &str) -> usize {
    of::<NoticePublished>(&cafe.perceived(name)).len()
}

#[test]
fn a_notice_reaches_exactly_the_subscribers_and_its_publisher() {
    let mut cafe = Cafe::begun();
    let before_nat = cafe.perceived("nat").len();
    let dispatched = cafe
        .publish(
            "pia",
            "news.daily",
            text("The market opens late tomorrow."),
            HOUR,
            10_000,
        )
        .expect("dispatches");
    assert!(
        matches!(dispatched.result(), ActionResult::Accepted { .. }),
        "{dispatched:?}"
    );
    let published = of::<NoticePublished>(dispatched.events());
    assert_eq!(published.len(), 1, "one fact");
    let (fact, envelope) = &published[0];
    let expected: std::collections::BTreeSet<_> = ["pia", "sam", "sue"]
        .iter()
        .map(|name| cafe.id(name))
        .collect();
    assert_eq!(
        *envelope.visibility(),
        Visibility::Entities(expected),
        "the audience"
    );
    assert_eq!(fact.notice().issued_at().seconds(), 10_000);
    assert_eq!(fact.notice().valid_until().seconds(), 13_600);
    for name in ["pia", "sam", "sue"] {
        assert_eq!(notices(&cafe, name), 1, "{name} perceives the notice");
    }
    assert_eq!(notices(&cafe, "nat"), 0, "nat does not");
    assert_eq!(
        cafe.perceived("nat").len(),
        before_nat,
        "nat learns nothing of notices"
    );
}

#[test]
fn every_refusal_is_named_and_moves_nothing() {
    let mut cafe = Cafe::begun();
    let bread = cafe.line("bread", 250, 3);
    let milk = cafe.line("milk", 120, 2);
    let pia = cafe.id("pia");
    let many: Vec<_> = (0..33).map(|_| bread).collect();
    let cases: Vec<(&str, &str, NoticeBody, u32, String)> = vec![
        (
            "pia",
            "news.weekly",
            text("x"),
            HOUR,
            "unknown-topic".into(),
        ),
        (
            "sam",
            "news.daily",
            text("x"),
            HOUR,
            "PermissionDenied".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            text("x"),
            HOUR,
            "body-mismatch".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(many),
            HOUR,
            "body-too-large".into(),
        ),
        (
            "pia",
            "news.daily",
            text(&"a".repeat(281)),
            HOUR,
            "body-too-large".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(vec![mineworld_notices::PriceLine::new(pia, 100, 1)]),
            HOUR,
            "body-invalid".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(vec![cafe.line("bread", 0, 1)]),
            HOUR,
            "body-invalid".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(vec![cafe.line("bread", NOTICE_PRICE_MAX + 1, 1)]),
            HOUR,
            "body-invalid".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(vec![cafe.line("bread", 1, 0)]),
            HOUR,
            "body-invalid".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(vec![milk, bread]),
            HOUR,
            "body-invalid".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(vec![bread, bread]),
            HOUR,
            "body-invalid".into(),
        ),
        (
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(Vec::new()),
            HOUR,
            "body-invalid".into(),
        ),
        ("pia", "news.daily", text(""), HOUR, "body-invalid".into()),
        (
            "pia",
            "news.daily",
            text("two\nlines"),
            HOUR,
            "body-invalid".into(),
        ),
        (
            "pia",
            "news.daily",
            text("x"),
            HOUR - 1,
            "validity-out-of-range".into(),
        ),
        (
            "pia",
            "news.daily",
            text("x"),
            7 * 86_400 + 1,
            "validity-out-of-range".into(),
        ),
    ];
    // The bread line sorts before the milk line only if bread's id is lower: keys allocate in order.
    assert!(cafe.id("bread") < cafe.id("milk"), "fixture order");
    for (at, (actor, on, body, valid_for, expected)) in (1_000_i64..).zip(cases) {
        let before = cafe.state();
        let dispatched = cafe
            .publish(actor, on, body, valid_for, at)
            .expect("dispatches");
        assert_eq!(named(dispatched.result()), expected, "{actor} on {on}");
        assert!(dispatched.events().is_empty(), "no fact for {expected}");
        assert_eq!(cafe.state(), before, "nothing moved for {expected}");
    }
}

#[test]
fn the_daily_limit_is_a_rolling_24_hours() {
    let mut cafe = Cafe::begun();
    let first = 10_000;
    for at in [first, first + 100] {
        let dispatched = cafe
            .publish("pia", "news.daily", text("ok"), HOUR, at)
            .expect("ok");
        assert!(matches!(dispatched.result(), ActionResult::Accepted { .. }));
    }
    let third = cafe
        .publish("pia", "news.daily", text("third"), HOUR, first + 86_399)
        .expect("dispatches");
    assert_eq!(named(third.result()), "over-daily-limit");
    let again = cafe
        .publish("pia", "news.daily", text("again"), HOUR, first + 86_400)
        .expect("dispatches");
    assert!(
        matches!(again.result(), ActionResult::Accepted { .. }),
        "the first post left the window at first + 86 400: {:?}",
        again.result()
    );
    // The other topic has its own window.
    let line = cafe.line("bread", 250, 3);
    let other = cafe
        .publish(
            "pia",
            "prices.ingredients",
            NoticeBody::Prices(vec![line]),
            HOUR,
            first + 86_401,
        )
        .expect("dispatches");
    assert!(matches!(other.result(), ActionResult::Accepted { .. }));
}

#[test]
fn subscribing_later_receives_only_later_notices_and_unsubscribing_stops_them() {
    let mut cafe = Cafe::begun();
    accepted(
        cafe.publish("pia", "news.daily", text("first"), HOUR, 10_000),
        "first",
    );
    let joined = cafe
        .subscribe("nat", "news.daily", 10_500)
        .expect("subscribes");
    assert!(matches!(joined.result(), ActionResult::Accepted { .. }));
    let left = cafe
        .unsubscribe("sue", "news.daily", 10_600)
        .expect("unsubscribes");
    assert!(matches!(left.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        notices(&cafe, "nat"),
        0,
        "nat did not receive the old notice"
    );
    let board = cafe.board("nat", 11_000);
    assert_eq!(
        board.len(),
        1,
        "but sees it on the board while it is current"
    );
    assert_eq!(cafe.board("nat", 13_600).len(), 0, "until it expires");
    accepted(
        cafe.publish("pia", "news.daily", text("second"), HOUR, 20_000),
        "second",
    );
    assert_eq!(notices(&cafe, "nat"), 1, "nat receives the next one");
    assert_eq!(notices(&cafe, "sue"), 1, "sue only the first");
    // Each change is one fact visible to that person alone.
    let joined = &joined.events()[0];
    assert_eq!(joined.event_type().as_str(), "subscribed");
    assert_eq!(
        *joined.visibility(),
        Visibility::Entities([cafe.id("nat")].into_iter().collect())
    );
    let left = &left.events()[0];
    assert_eq!(left.event_type().as_str(), "unsubscribed");
    assert_eq!(
        *left.visibility(),
        Visibility::Entities([cafe.id("sue")].into_iter().collect())
    );
}

#[test]
fn a_world_without_configuration_offers_and_admits_nothing() {
    let recipe = Recipe {
        configuration: None,
        sections: Vec::new(),
        tags: Vec::new(),
    };
    let mut cafe =
        Cafe::try_begin(&recipe).expect("a world with the pack and no configuration begins");
    let dispatched = cafe
        .publish("pia", "news.daily", text("x"), HOUR, 100)
        .expect("dispatches");
    assert_eq!(named(dispatched.result()), "NoSupportedInteraction");
    let dispatched = cafe
        .subscribe("pia", "news.daily", 101)
        .expect("dispatches");
    assert_eq!(named(dispatched.result()), "NoSupportedInteraction");
    let observation = cafe.observation("pia", 102);
    assert!(
        observation
            .affordances()
            .iter()
            .all(
                |affordance| affordance.action_type().as_str() != "publish-notice"
                    && affordance.action_type().as_str() != "subscribe"
            ),
        "nothing is offered"
    );
}

#[test]
fn a_payload_that_does_not_decode_is_a_failed_precondition() {
    let mut cafe = Cafe::begun();
    let record = ActionRecord::new::<PublishNotice>(
        br#"{"topic":"news.daily","body":{"poem":"x"},"valid_for":3600}"#.to_vec(),
    );
    let dispatched = cafe.act_raw("pia", record, 100).expect("dispatches");
    assert_eq!(named(dispatched.result()), "PreconditionFailed");
    let record = ActionRecord::new::<PublishNotice>(
        br#"{"topic":"news.daily","body":{"text":"x"},"valid_for":-1}"#.to_vec(),
    );
    let dispatched = cafe.act_raw("pia", record, 101).expect("dispatches");
    assert_eq!(named(dispatched.result()), "PreconditionFailed");
}

#[test]
fn offer_and_dispatch_agree() {
    let mut cafe = Cafe::begun();
    // Pia publishes, so she is offered publish-notice, and her post is admitted.
    let observation = cafe.observation("pia", 50);
    assert!(
        observation
            .affordances()
            .iter()
            .any(|affordance| affordance.action_type().as_str() == "publish-notice")
    );
    let ok = cafe
        .publish("pia", "news.daily", text("ok"), HOUR, 60)
        .expect("dispatches");
    assert!(matches!(ok.result(), ActionResult::Accepted { .. }));
    // Sam publishes nothing, so he is not offered it, and his post is refused.
    let observation = cafe.observation("sam", 70);
    assert!(
        observation
            .affordances()
            .iter()
            .all(|affordance| affordance.action_type().as_str() != "publish-notice")
    );
    let refused = cafe
        .publish("sam", "news.daily", text("no"), HOUR, 80)
        .expect("dispatches");
    assert_eq!(rejection(refused.result()), Rejection::PermissionDenied);
    // Sam subscribes to every topic, so he is not offered subscribe, and a subscribe is refused.
    let observation = cafe.observation("sam", 90);
    assert!(
        observation
            .affordances()
            .iter()
            .all(|affordance| affordance.action_type().as_str() != "subscribe")
    );
    let again = cafe
        .subscribe("sam", "news.daily", 100)
        .expect("dispatches");
    assert_eq!(
        rejection(again.result()),
        Rejection::System {
            code: code("already-subscribed"),
            detail: None
        }
    );
}
