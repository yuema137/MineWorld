//! The clerk, decided from observations built by hand (`pr-s31d-reporter-policies.md` §8: AC-d-1 the
//! suggestion rule, AC-d-2 the publishing slots, AC-d-3 the survey band; W1 in US cents, W1b in crowns of
//! exponent 0). Books, offers and times are literal; the oracle is the rule of §5.3 worked by hand.

use mineworld_contracts::Action;
use mineworld_contracts::{
    ActionRequest, Affordance, ComponentRecord, EntityId, EntityType, Location, Observation,
    PerceivedEntity, PlaceId, Rejection, RejectionCode, SimDuration, SpatialRequirement, WorldTime,
};
use mineworld_decision::{Consult, DecisionModel, ModelParameters, Seed};
use mineworld_notices::{NoticeBody, PublishNotice};
use mineworld_price_survey::{PriceBook, Survey};
use mineworld_rule_controller::{Rule, RuleParameters};
use serde_json::{Value, json};

use crate::{PriceReporter, ReporterParameters, suggest};

const CAROL: u64 = 3;
const STORE: u64 = 31;
const CAFE: u64 = 33;
const CORNER_STORE: u64 = 40;
const CAFE_COMPANY: u64 = 41;
const GRACE: u64 = 4;
const APPLE: u64 = 10;
const BREAD: u64 = 11;
const PACE: i64 = 900;
const DAY: i64 = 86_400;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn item(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "item" })
}

fn place(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "place" })
}

/// A sale note at `at`.
fn sale(at: i64, price: u64) -> Value {
    json!({ "at": at, "place": place(STORE), "seller": CORNER_STORE.to_string(), "price": price,
            "source": "sale", "in_stock": null })
}

/// An ask note at `at` by `seller` at `site`, with a stock count for a shop's ask.
fn ask(at: i64, seller: u64, site: u64, price: u64, stock: Option<u32>) -> Value {
    json!({ "at": at, "place": place(site), "seller": seller.to_string(), "price": price,
            "source": "ask", "in_stock": stock })
}

/// A book: per kind, its notes.
fn book(kinds: Vec<(u64, Vec<Value>)>) -> Value {
    let kinds: Vec<Value> = kinds
        .into_iter()
        .map(|(kind, notes)| json!({ "item": item(kind), "notes": notes }))
        .collect();
    json!({ "categories": ["food", "drink"], "depth": 64, "window": 604_800, "resurvey_after": 3_600,
            "kinds": kinds, "surveyed": [] })
}

fn decoded(book: &Value) -> PriceBook {
    serde_json::from_value(book.clone()).expect("a book")
}

fn defaults() -> ReporterParameters {
    ReporterParameters::default()
}

/// The lines as (item, price, basis).
fn lines(book: &Value, parameters: &ReporterParameters) -> Vec<(u64, u64, u16)> {
    suggest(&decoded(book), parameters)
        .iter()
        .map(|line| (line.item().raw(), line.price(), line.basis()))
        .collect()
}

#[test]
fn a_kind_with_enough_sales_takes_their_median_adjusted_by_stock() {
    // (a) Bread: sales 300, 320, 300; asks corner-store with stock 1 and grace with none → median stock 1 < 3.
    let a = book(vec![(
        BREAD,
        vec![
            sale(10, 300),
            sale(20, 320),
            sale(30, 300),
            ask(40, CORNER_STORE, STORE, 300, Some(1)),
            ask(50, GRACE, STORE, 260, None),
        ],
    )]);
    assert_eq!(lines(&a, &defaults()), [(BREAD, 330, 3)]);
    // (b) An even count with no stock: floor of the two middle values' mean.
    let b = book(vec![(
        BREAD,
        vec![sale(10, 300), sale(20, 310), sale(30, 320), sale(40, 330)],
    )]);
    assert_eq!(lines(&b, &defaults()), [(BREAD, 315, 4)]);
    // (e) Median stock 25 > 20: 300 × 950 ÷ 1 000 = 285.
    let e = book(vec![(
        BREAD,
        vec![
            sale(10, 300),
            sale(20, 300),
            sale(30, 300),
            ask(40, CORNER_STORE, STORE, 300, Some(25)),
        ],
    )]);
    assert_eq!(lines(&e, &defaults()), [(BREAD, 285, 3)]);
    // (f) A base of 1 with plenty stays 1.
    let f = book(vec![(
        BREAD,
        vec![
            sale(10, 1),
            sale(20, 1),
            sale(30, 1),
            ask(40, CORNER_STORE, STORE, 1, Some(50)),
        ],
    )]);
    assert_eq!(lines(&f, &defaults()), [(BREAD, 1, 3)]);
}

#[test]
fn too_few_sales_fall_back_to_deduped_asks() {
    // (c) One sale; asks: corner-store at the store (an older 290 and a newer 300, stock 5), grace 260,
    // cafe-company at the café 280 (stock 9) → three deduped asks, median 280; stock median 7: no adjustment.
    let c = book(vec![(
        BREAD,
        vec![
            sale(5, 250),
            ask(10, CORNER_STORE, STORE, 290, Some(5)),
            ask(20, CORNER_STORE, STORE, 300, Some(5)),
            ask(30, GRACE, STORE, 260, None),
            ask(40, CAFE_COMPANY, CAFE, 280, Some(9)),
        ],
    )]);
    assert_eq!(lines(&c, &defaults()), [(BREAD, 280, 3)]);
    // (d) Two sales and two asks: no line.
    let d = book(vec![(
        BREAD,
        vec![
            sale(5, 250),
            sale(6, 260),
            ask(10, CORNER_STORE, STORE, 300, Some(5)),
            ask(30, GRACE, STORE, 260, None),
        ],
    )]);
    assert!(lines(&d, &defaults()).is_empty());
}

#[test]
fn at_most_thirty_two_lines_keep_the_largest_basis() {
    // (g) 33 kinds: kind k has 3 + (k mod 2) sales; the two smallest-basis kinds tie at 3, so the higher id of
    // the lowest-basis kinds is dropped.
    let kinds: Vec<(u64, Vec<Value>)> = (0..33_u64)
        .map(|k| {
            let count: i64 = if k == 32 { 3 } else { 4 };
            let notes = (0..count).map(|n| sale(n, 100 + k)).collect();
            (100 + k, notes)
        })
        .collect();
    let all = lines(&book(kinds), &defaults());
    assert_eq!(all.len(), 32);
    assert!(
        all.iter().all(|(kind, _, _)| *kind != 132),
        "the lowest basis is dropped"
    );
    assert!(
        all.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "sorted by item"
    );
}

#[test]
fn integer_floors_show_at_small_scales() {
    // (h) W1b: sales 3, 3, 4 crowns, scarce: 3 × 1 100 ÷ 1 000 = 3.
    let h = book(vec![(
        BREAD,
        vec![
            sale(10, 3),
            sale(20, 3),
            sale(30, 4),
            ask(40, CORNER_STORE, STORE, 3, Some(1)),
        ],
    )]);
    assert_eq!(lines(&h, &defaults()), [(BREAD, 3, 3)]);
}

/// Carol in the store at `at`, with her book, and these offers.
fn carol(at: i64, book: Option<&Value>, offers: Vec<Affordance<Value>>) -> Observation<Value> {
    let store = PlaceId::new(id(STORE), EntityType::Place).expect("a place");
    let here = Location::in_place(store);
    let mut records = Vec::new();
    if let Some(book) = book {
        let record: ComponentRecord<Value> = serde_json::from_value(json!({
            "entity": CAROL.to_string(),
            "component_type": "survey-book",
            "schema_version": 1,
            "payload": book,
        }))
        .expect("a record");
        records.push(record);
    }
    let me = PerceivedEntity::new(id(CAROL), EntityType::Person)
        .at(here)
        .with_components(records);
    Observation::new(id(CAROL), WorldTime::from_seconds(at))
        .at_location(here)
        .perceiving(vec![PerceivedEntity::new(id(STORE), EntityType::Place), me])
        .offering(offers)
}

fn publish_offer(available: bool) -> Affordance<Value> {
    if available {
        Affordance::available(PublishNotice::ACTION_TYPE, None, SpatialRequirement::NONE)
    } else {
        Affordance::unavailable(
            PublishNotice::ACTION_TYPE,
            None,
            SpatialRequirement::NONE,
            Rejection::System {
                code: RejectionCode::from_static("over-daily-limit"),
                detail: None,
            },
        )
    }
}

fn survey_offer(available: bool) -> Affordance<Value> {
    let offer = if available {
        Affordance::available(Survey::ACTION_TYPE, None, SpatialRequirement::NONE)
    } else {
        Affordance::unavailable(
            Survey::ACTION_TYPE,
            None,
            SpatialRequirement::NONE,
            Rejection::System {
                code: RejectionCode::from_static("surveyed-recently"),
                detail: None,
            },
        )
    };
    offer.with_payload(json!({}))
}

fn bread_book() -> Value {
    book(vec![
        (APPLE, vec![sale(10, 100)]),
        (
            BREAD,
            vec![
                sale(10, 300),
                sale(20, 320),
                sale(30, 300),
                ask(40, CORNER_STORE, STORE, 300, Some(1)),
            ],
        ),
    ])
}

fn decide(
    parameters: &ReporterParameters,
    observation: &Observation<Value>,
) -> Option<ActionRequest> {
    PriceReporter.decide(
        parameters,
        &Consult {
            seed: Seed::new(7),
            pace: SimDuration::from_seconds(PACE),
            stride: SimDuration::from_seconds(30),
            observation,
        },
    )
}

fn rule(observation: &Observation<Value>) -> Option<ActionRequest> {
    Rule.decide(
        &RuleParameters::default(),
        &Consult {
            seed: Seed::new(7),
            pace: SimDuration::from_seconds(PACE),
            stride: SimDuration::from_seconds(30),
            observation,
        },
    )
}

/// The publish requests over one day of consults at seat offset `offset`, as (hour, request).
fn day_of_publishing(parameters: &ReporterParameters, offset: i64) -> Vec<(i64, PublishNotice)> {
    let book = bread_book();
    (0..96)
        .map(|m| DAY + offset + m * PACE)
        .filter_map(|t| {
            let decided = decide(
                parameters,
                &carol(t, Some(&book), vec![publish_offer(true)]),
            )?;
            (decided.action_type() == &PublishNotice::ACTION_TYPE).then(|| {
                let notice: PublishNotice = serde_json::from_slice(decided.payload().payload())
                    .expect("notices decodes it");
                ((t - DAY) / 3_600, notice)
            })
        })
        .collect()
}

#[test]
fn the_clerk_publishes_once_at_each_slot() {
    for offset in [0, 3] {
        let published = day_of_publishing(&defaults(), offset);
        let hours: Vec<i64> = published.iter().map(|(hour, _)| *hour).collect();
        assert_eq!(hours, [8, 12, 16, 20], "offset {offset}");
        for (_, notice) in &published {
            assert_eq!(notice.topic().as_str(), "prices.ingredients");
            assert_eq!(notice.valid_for(), 21_600);
            let NoticeBody::Prices(lines) = notice.body() else {
                panic!("a prices body");
            };
            let lines: Vec<(u64, u64, u16)> = lines
                .iter()
                .map(|line| (line.item().raw(), line.price(), line.basis()))
                .collect();
            assert_eq!(lines, [(BREAD, 330, 3)]);
        }
    }
    let evening = ReporterParameters::decode(
        json!({ "first_minute": 960, "last_minute": 1_200, "every_minutes": 120 }),
    )
    .expect("parameters");
    let hours: Vec<i64> = day_of_publishing(&evening, 3)
        .iter()
        .map(|(hour, _)| *hour)
        .collect();
    assert_eq!(hours, [16, 18, 20]);
}

#[test]
fn no_offer_no_book_or_no_line_means_no_bulletin() {
    let slot = DAY + 8 * 3_600;
    let book = bread_book();
    let empty = book_without_lines();
    for observation in [
        carol(slot, Some(&book), Vec::new()),
        carol(slot, Some(&book), vec![publish_offer(false)]),
        carol(slot, None, vec![publish_offer(true)]),
        carol(slot, Some(&empty), vec![publish_offer(true)]),
    ] {
        assert_eq!(decide(&defaults(), &observation), rule(&observation));
    }
}

fn book_without_lines() -> Value {
    book(vec![(BREAD, vec![sale(10, 300)])])
}

#[test]
fn the_clerk_takes_the_survey_it_is_offered_and_otherwise_lives_by_rule() {
    let at = DAY + 10 * 3_600;
    for scale_book in [bread_book(), book(vec![(BREAD, vec![sale(10, 3)])])] {
        let offered = carol(at, Some(&scale_book), vec![survey_offer(true)]);
        let decided = decide(&defaults(), &offered).expect("a survey");
        let expected = offered.affordances()[0]
            .request(id(CAROL), |payload| {
                serde_json::to_vec(payload).expect("JSON")
            })
            .expect("its own request");
        assert_eq!(decided, expected);
        for observation in [
            carol(at, Some(&scale_book), vec![survey_offer(false)]),
            carol(at, Some(&scale_book), Vec::new()),
        ] {
            assert_eq!(decide(&defaults(), &observation), rule(&observation));
        }
    }
}

#[test]
fn the_parameters_refuse_each_bound_at_its_key() {
    assert_eq!(
        ReporterParameters::decode(json!({})).expect("defaults"),
        defaults()
    );
    for (block, names) in [
        (
            json!({ "first_minute": 1_440 }),
            "'first_minute' is 0 … 1439, not 1440",
        ),
        (
            json!({ "every_minutes": 14 }),
            "'every_minutes' is 15 … 1440",
        ),
        (
            json!({ "valid_minutes": 0 }),
            "'valid_minutes' is 1 … 43200",
        ),
        (json!({ "min_basis": 65 }), "'min_basis' is 1 … 64"),
        (
            json!({ "plenty_permille": 901 }),
            "'plenty_permille' is 0 … 900",
        ),
        (json!({ "plenty_above": 0 }), "'plenty_above' is 1 … 100000"),
        (
            json!({ "first_minute": 600, "last_minute": 500 }),
            "'last_minute'",
        ),
        (json!({ "topic": "Prices" }), "a topic is"),
        (json!({ "pricing": {} }), "unknown field `pricing`"),
    ] {
        let message = ReporterParameters::decode(block)
            .expect_err("refused")
            .to_string();
        assert!(message.contains(names), "{message} should name {names}");
    }
}
