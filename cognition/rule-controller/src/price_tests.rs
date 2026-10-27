//! The pricing block, the seller band and the buyer hook, decided from observations built by hand
//! (`pr-s31d-reporter-policies.md` §8: AC-d-5 (a)–(l), AC-d-6 (a), the slot arithmetic of AC-d-2; W1 in US
//! dollars and W1b in crowns of exponent 0). Every record is written as the world discloses it — slug and
//! JSON — since this crate names no market crate (`ARC-182`).

use mineworld_contracts::{
    ActionRequest, ActionTypeId, Affordance, Causation, ComponentRecord, EntityId, EntityType,
    EventEnvelope, EventId, EventRecord, Location, Observation, PerceivedEntity, PerceivedEvent,
    PlaceId, Provenance, SimDuration, SpatialRequirement, SystemId, Tag, Tags, Visibility,
    WorldTime,
};
use mineworld_decision::ModelParameters;
use mineworld_notices::NoticePublished;
use serde_json::{Value, json};

use crate::price::slots::{in_window, period_index, publish_due};
use crate::price::{self, BuyChoice, Pricing, SellPolicy};
use crate::{PacedRuleController, RuleParameters};

const ME: u64 = 3;
const GRACE: u64 = 4;
const PUBLISHER: u64 = 5;
const OTHER_PUBLISHER: u64 = 6;
const STORE: u64 = 31;
const COMPANY: u64 = 40;
const APPLE: u64 = 10;
const BREAD: u64 = 11;
const JUICE: u64 = 12;
const SCARF: u64 = 13;
const PEN: u64 = 14;

const PACE: i64 = 900;
/// 08:00 on day 2: index 0 of a 240-minute period.
const SLOT: i64 = 86_400 + 8 * 3_600;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn item(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "item" })
}

fn person(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "person" })
}

fn record(entity: u64, slug: &str, schema: u32, payload: Value) -> ComponentRecord<Value> {
    serde_json::from_value(json!({
        "entity": entity.to_string(),
        "component_type": slug,
        "schema_version": schema,
        "payload": payload,
    }))
    .expect("a record")
}

/// A notice on the board: topic, publisher, issued at, lines (item, price, basis).
type Posted = (&'static str, u64, i64, Vec<(u64, u64, u16)>);

/// Where I stand, what I hold and ask, what the place shows, the board, and what is offered.
#[derive(Clone)]
struct Market {
    at: i64,
    /// Every amount divided by this, rounded up (W1b: 100, a crown of exponent 0).
    scale: u64,
    tags: Vec<&'static str>,
    own_asks: Vec<(u64, u64)>,
    held: Vec<(u64, u32)>,
    /// The store's listing: item, price, in stock. `None`: not in a shop.
    listing: Option<Vec<(u64, u64, u32)>>,
    references: Vec<(u64, u64)>,
    board: Vec<Posted>,
    /// Remembered or perceived notices that are not on the board (X-6).
    perceived: Vec<Posted>,
    set_price: bool,
    list_for_sale: bool,
    /// Complete `buy` offers: item, price, seller (None: the shop).
    buys: Vec<(u64, u64, Option<u64>)>,
}

impl Market {
    /// Felix, keeper of the store, at `at`: the store asks apple 100, bread 300, juice 250.
    fn keeper(at: i64) -> Self {
        Self {
            at,
            scale: 1,
            tags: Vec::new(),
            own_asks: Vec::new(),
            held: Vec::new(),
            listing: Some(vec![(APPLE, 100, 4), (BREAD, 300, 5), (JUICE, 250, 4)]),
            references: vec![(APPLE, 100), (BREAD, 300), (JUICE, 250), (SCARF, 2_500)],
            board: Vec::new(),
            perceived: Vec::new(),
            set_price: true,
            list_for_sale: false,
            buys: Vec::new(),
        }
    }

    fn crowns(mut self) -> Self {
        self.scale = 100;
        self
    }

    fn suggest(mut self, price: u64) -> Self {
        self.board.push((
            "prices.ingredients",
            PUBLISHER,
            self.at - 600,
            vec![(BREAD, price, 3)],
        ));
        self
    }

    fn amount(&self, value: u64) -> u64 {
        value.div_ceil(self.scale)
    }

    fn notices(&self, posted: &[Posted]) -> Vec<Value> {
        posted
            .iter()
            .enumerate()
            .map(|(n, (topic, publisher, at, lines))| {
                let prices: Vec<Value> = lines
                    .iter()
                    .map(|(kind, price, basis)| {
                        json!({ "item": kind.to_string(), "price": self.amount(*price), "basis": basis })
                    })
                    .collect();
                json!({
                    "id": n + 1,
                    "topic": topic,
                    "publisher": person(*publisher),
                    "body": { "prices": prices },
                    "issued_at": at,
                    "valid_until": at + 21_600,
                })
            })
            .collect()
    }

    fn build(&self) -> Observation<Value> {
        let place = PlaceId::new(id(STORE), EntityType::Place).expect("a place");
        let here = Location::in_place(place);
        let catalogue = json!({ "kinds": [
            { "item": item(APPLE), "category": "food", "name": "Apple" },
            { "item": item(BREAD), "category": "food", "name": "Bread" },
            { "item": item(JUICE), "category": "drink", "name": "Juice" },
            { "item": item(SCARF), "category": "goods", "name": "Scarf" },
            { "item": item(PEN), "category": "goods", "name": "Pen" },
        ]});
        let mut site_records = vec![record(STORE, "item-catalogue", 1, catalogue)];
        if let Some(listing) = &self.listing {
            let listed: Vec<Value> = listing
                .iter()
                .map(|(kind, price, stock)| {
                    json!({ "item": item(*kind), "price": self.amount(*price), "in_stock": stock })
                })
                .collect();
            site_records.push(record(
                STORE,
                "shop",
                2,
                json!({
                    "operator": { "entity": COMPANY.to_string(), "entity_type": "organization" },
                    "listed": listed,
                }),
            ));
        }
        let site = PerceivedEntity::new(id(STORE), EntityType::Place).with_components(site_records);
        let references: Vec<Value> = self
            .references
            .iter()
            .map(|(kind, value)| json!({ "item": item(*kind), "value": self.amount(*value) }))
            .collect();
        let (name, symbol, exponent, at) = if self.scale == 1 {
            ("US dollar", "$", 2, "before")
        } else {
            ("crown", "cr", 0, "after")
        };
        let mut mine = vec![
            record(
                ME,
                "currency",
                1,
                json!({ "name": name, "symbol": symbol, "exponent": exponent, "symbol_at": at,
                        "references": references }),
            ),
            record(
                ME,
                "notice-board",
                1,
                json!({ "notices": self.notices(&self.board) }),
            ),
            record(
                ME,
                "holdings",
                1,
                json!({ "held": self.held.iter().map(|(kind, count)| json!({ "item": item(*kind), "count": count })).collect::<Vec<_>>(), "capacity": 6 }),
            ),
        ];
        if !self.own_asks.is_empty() {
            let asks: Vec<Value> = self
                .own_asks
                .iter()
                .map(|(kind, price)| {
                    json!({ "item": item(*kind), "price": self.amount(*price), "available": true })
                })
                .collect();
            mine.push(record(ME, "asks", 1, json!({ "asks": asks })));
        }
        let tags = Tags::new(self.tags.iter().map(|tag| Tag::new(*tag).expect("a tag")));
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(here)
            .with_tags(tags)
            .with_components(mine);
        let grace = PerceivedEntity::new(id(GRACE), EntityType::Person).at(here);
        let mut affordances = Vec::new();
        if self.set_price {
            affordances.push(Affordance::available(
                ActionTypeId::from_static("set-price"),
                None,
                SpatialRequirement::at_place(place),
            ));
        }
        if self.list_for_sale {
            affordances.push(Affordance::available(
                ActionTypeId::from_static("list-for-sale"),
                None,
                SpatialRequirement::NONE,
            ));
        }
        for (kind, price, seller) in &self.buys {
            affordances.push(
                Affordance::available(
                    ActionTypeId::from_static("buy"),
                    seller.map(id),
                    SpatialRequirement::NONE,
                )
                .with_payload(json!({ "item": item(*kind), "price": self.amount(*price) })),
            );
        }
        let events: Vec<PerceivedEvent<Value>> = self
            .notices(&self.perceived)
            .into_iter()
            .enumerate()
            .map(|(n, notice)| {
                PerceivedEvent::new(EventEnvelope::new(
                    EventId::from_raw(100 + n as u64),
                    WorldTime::from_seconds(self.at - 30_000),
                    EventRecord::new::<NoticePublished>(json!({ "notice": notice })),
                    Causation::WorldGenesis,
                    Visibility::Public,
                    Provenance::new(SystemId::from_static("notices")),
                ))
            })
            .collect();
        Observation::new(id(ME), WorldTime::from_seconds(self.at))
            .at_location(here)
            .perceiving(vec![site, me, grace])
            .offering(affordances)
            .with_events(events)
    }
}

fn pricing(text: Value) -> Pricing {
    serde_json::from_value(text).expect("a pricing block")
}

fn toward() -> Pricing {
    pricing(json!({ "sell": { "policy": "toward-suggestion" } }))
}

fn band(market: &Market, pricing: &Pricing) -> Option<ActionRequest> {
    price::seller_band(&market.build(), SimDuration::from_seconds(PACE), pricing)
}

/// The request's action type and payload.
fn what(request: Option<ActionRequest>) -> Option<(String, Value)> {
    let request = request?;
    let payload: Value = serde_json::from_slice(request.payload().payload()).expect("JSON");
    Some((request.action_type().as_str().to_owned(), payload))
}

fn set_price(kind: u64, price: u64, shop: bool) -> Option<(String, Value)> {
    let mut payload = json!({ "item": item(kind), "price": price });
    if shop {
        payload["for"] = json!("shop");
    }
    Some(("set-price".to_owned(), payload))
}

/// The consult at index `n` of the period that starts at [`SLOT`].
const fn index(n: i64) -> i64 {
    SLOT + n * PACE
}

#[test]
fn hold_submits_nothing_in_any_observation_of_the_suite() {
    let hold = Pricing::default();
    for n in 0..16 {
        for market in [
            Market::keeper(index(n)).suggest(330),
            Market::keeper(index(n)).suggest(2_000).crowns(),
            Market {
                own_asks: vec![(BREAD, 300)],
                ..Market::keeper(index(n)).suggest(330)
            },
        ] {
            assert_eq!(band(&market, &hold), None, "index {n}");
        }
    }
}

#[test]
fn toward_moves_the_bread_ask_one_step_at_its_index_only() {
    // The store's priceable list is apple, bread, juice: bread is index 2.
    let market = |n| Market::keeper(index(n)).suggest(330);
    assert_eq!(
        what(band(&market(2), &toward())),
        set_price(BREAD, 303, true)
    );
    for n in [0, 1, 3, 4, 15] {
        assert_eq!(band(&market(n), &toward()), None, "index {n}");
    }
    // W1b: 3 crowns toward 4 (3.30 rounded up): gap 1, a tenth of it truncates to 0, so one unit.
    let crowns = Market::keeper(index(2)).suggest(330).crowns();
    assert_eq!(what(band(&crowns, &toward())), set_price(BREAD, 4, true));
}

#[test]
fn a_gap_inside_the_deadband_is_ignored() {
    let market = Market::keeper(index(2)).suggest(309);
    assert_eq!(band(&market, &toward()), None);
    let outside = Market::keeper(index(2)).suggest(310);
    assert_eq!(what(band(&outside, &toward())), set_price(BREAD, 301, true));
}

#[test]
fn every_move_is_bounded_by_the_reference_value() {
    let far = Market {
        listing: Some(vec![(APPLE, 100, 4), (BREAD, 590, 5), (JUICE, 250, 4)]),
        ..Market::keeper(index(2)).suggest(2_000)
    };
    assert_eq!(what(band(&far, &toward())), set_price(BREAD, 600, true));
    let at_ceiling = Market {
        listing: Some(vec![(APPLE, 100, 4), (BREAD, 600, 5), (JUICE, 250, 4)]),
        ..Market::keeper(index(2)).suggest(2_000)
    };
    assert_eq!(band(&at_ceiling, &toward()), None);
    let low = Market {
        listing: Some(vec![(APPLE, 100, 4), (BREAD, 160, 5), (JUICE, 250, 4)]),
        ..Market::keeper(index(2)).suggest(1)
    };
    assert_eq!(what(band(&low, &toward())), set_price(BREAD, 150, true));
}

#[test]
fn a_kind_with_no_reference_value_is_never_moved() {
    let market = Market {
        references: vec![(APPLE, 100), (JUICE, 250)],
        ..Market::keeper(index(2)).suggest(330)
    };
    assert_eq!(band(&market, &toward()), None);
}

/// X-6: an expired suggestion is on no board; one perceived in the observation's events is never read.
#[test]
fn a_notice_not_on_the_board_is_never_a_suggestion() {
    let market = Market {
        perceived: vec![(
            "prices.ingredients",
            PUBLISHER,
            index(2) - 30_000,
            vec![(BREAD, 330, 3)],
        )],
        ..Market::keeper(index(2))
    };
    assert!(market.board.is_empty());
    assert_eq!(band(&market, &toward()), None);
}

#[test]
fn the_newest_notice_wins_and_a_tie_goes_to_the_lower_publisher() {
    let at = index(2);
    let newest = Market {
        board: vec![
            (
                "prices.ingredients",
                PUBLISHER,
                at - 900,
                vec![(BREAD, 320, 3)],
            ),
            (
                "prices.daily",
                OTHER_PUBLISHER,
                at - 300,
                vec![(BREAD, 340, 3)],
            ),
        ],
        ..Market::keeper(at)
    };
    assert_eq!(what(band(&newest, &toward())), set_price(BREAD, 304, true));
    let tied = Market {
        board: vec![
            (
                "prices.daily",
                OTHER_PUBLISHER,
                at - 300,
                vec![(BREAD, 340, 3)],
            ),
            (
                "prices.ingredients",
                PUBLISHER,
                at - 300,
                vec![(BREAD, 320, 3)],
            ),
        ],
        ..Market::keeper(at)
    };
    assert_eq!(what(band(&tied, &toward())), set_price(BREAD, 302, true));
    let suggested = price::suggestion(
        &tied.build(),
        serde_json::from_value(item(BREAD)).expect("an item"),
    )
    .expect("a suggestion");
    assert_eq!((suggested.price, suggested.basis), (320, 3));
}

#[test]
fn scarcity_moves_by_stock() {
    let scarcity = pricing(json!({ "sell": { "policy": "scarcity" } }));
    let stocked = |count| Market {
        listing: Some(vec![(APPLE, 100, 4), (BREAD, 300, count), (JUICE, 250, 4)]),
        ..Market::keeper(index(2))
    };
    assert_eq!(
        what(band(&stocked(1), &scarcity)),
        set_price(BREAD, 330, true)
    );
    assert_eq!(
        what(band(&stocked(12), &scarcity)),
        set_price(BREAD, 270, true)
    );
    assert_eq!(band(&stocked(5), &scarcity), None);
    // An own ask with none held is raised.
    let own = Market {
        own_asks: vec![(BREAD, 300)],
        held: Vec::new(),
        ..Market::keeper(index(1))
    };
    assert_eq!(what(band(&own, &scarcity)), set_price(BREAD, 330, false));
    // W1b: 3 crowns, scarce: a tenth of 3 is 0, so one unit.
    assert_eq!(
        what(band(&stocked(1).crowns(), &scarcity)),
        set_price(BREAD, 4, true)
    );
}

#[test]
fn list_lists_a_held_kind_of_a_listed_category_at_its_reference() {
    let list = pricing(json!({ "sell": { "list": { "categories": ["goods"] } } }));
    let grace = Market {
        listing: None,
        set_price: false,
        list_for_sale: true,
        held: vec![(APPLE, 2), (SCARF, 1)],
        ..Market::keeper(index(0))
    };
    let listed = Some((
        "list-for-sale".to_owned(),
        json!({ "item": item(SCARF), "price": 2_500 }),
    ));
    assert_eq!(what(band(&grace, &list)), listed);
    // Only at index 0.
    assert_eq!(
        band(
            &Market {
                at: index(1),
                ..grace.clone()
            },
            &list
        ),
        None
    );
    // A goods kind with no reference is not listed, nor a food kind.
    let pen = Market {
        held: vec![(APPLE, 2), (PEN, 1)],
        ..grace.clone()
    };
    assert_eq!(band(&pen, &list), None);
    // Already asked for: not listed again.
    let asked = Market {
        own_asks: vec![(SCARF, 2_400)],
        ..grace.clone()
    };
    assert_eq!(band(&asked, &list), None);
    // Not offered: nothing.
    let refused = Market {
        list_for_sale: false,
        ..grace
    };
    assert_eq!(band(&refused, &list), None);
}

#[test]
fn the_sell_half_applies_only_to_seats_with_its_tags() {
    let keepers = pricing(json!({ "sell": { "policy": "toward-suggestion", "tags": ["keeper"] } }));
    let untagged = Market::keeper(index(2)).suggest(330);
    assert_eq!(band(&untagged, &keepers), None);
    let tagged = Market {
        tags: vec!["keeper"],
        ..untagged
    };
    assert_eq!(what(band(&tagged, &keepers)), set_price(BREAD, 303, true));
}

#[test]
fn a_keeper_is_inferred_from_the_offer_and_own_asks_come_first() {
    // Own asks present: only they are priced (bread is index 1 of my own list).
    let own = Market {
        own_asks: vec![(BREAD, 300)],
        ..Market::keeper(index(1)).suggest(330)
    };
    assert_eq!(what(band(&own, &toward())), set_price(BREAD, 303, false));
    assert_eq!(
        band(
            &Market {
                at: index(2),
                ..own
            },
            &toward()
        ),
        None
    );
    // Not in a shop, no asks of my own: nothing.
    let away = Market {
        listing: None,
        ..Market::keeper(index(2)).suggest(330)
    };
    assert_eq!(band(&away, &toward()), None);
    // In the shop but not offered set-price (not a keeper): nothing.
    let visitor = Market {
        set_price: false,
        ..Market::keeper(index(2)).suggest(330)
    };
    assert_eq!(band(&visitor, &toward()), None);
}

#[test]
fn cheapest_buys_the_same_kind_from_the_cheapest_offer() {
    let market = Market {
        buys: vec![
            (BREAD, 300, None),
            (APPLE, 90, None),
            (BREAD, 260, Some(GRACE)),
        ],
        ..Market::keeper(index(5))
    };
    let observation = market.build();
    let chosen = observation
        .affordances()
        .iter()
        .find(|offer| {
            offer.target().is_none() && offer.payload().is_some_and(|p| p["price"] == 300)
        })
        .expect("the store's bread");
    let cheapest = pricing(json!({ "buy": { "choose": "cheapest" } }));
    let bought = price::on_chosen_buy(&observation, &cheapest, chosen).expect("a request");
    assert_eq!(bought.target(), Some(id(GRACE)));
    let payload: Value = serde_json::from_slice(bought.payload().payload()).expect("JSON");
    assert_eq!(payload, json!({ "item": item(BREAD), "price": 260 }));
    // `any`: unchanged.
    let any = price::on_chosen_buy(&observation, &Pricing::default(), chosen).expect("a request");
    assert_eq!(any.target(), None);
    // `buy.tags` not met: unchanged.
    let tagged = pricing(json!({ "buy": { "choose": "cheapest", "tags": ["thrifty"] } }));
    assert_eq!(
        price::on_chosen_buy(&observation, &tagged, chosen)
            .expect("a request")
            .target(),
        None
    );
    // A tie: the first in observation order.
    let tie = Market {
        buys: vec![(BREAD, 260, None), (BREAD, 260, Some(GRACE))],
        ..Market::keeper(index(5))
    };
    let observation = tie.build();
    let second = &observation.affordances()[2];
    assert_eq!(
        price::on_chosen_buy(&observation, &cheapest, second)
            .expect("a request")
            .target(),
        None
    );
}

#[test]
fn the_block_decodes_its_defaults_and_refuses_each_bound_at_its_key() {
    let empty = RuleParameters::decode(json!({})).expect("an empty rule block");
    assert_eq!(empty.pricing, Pricing::default());
    let explicit = RuleParameters::decode(
        json!({ "pricing": { "sell": { "policy": "hold" }, "buy": { "choose": "any" } } }),
    )
    .expect("explicit defaults");
    assert_eq!(explicit, empty);
    let chosen = RuleParameters::decode(
        json!({ "pricing": { "sell": { "policy": "scarcity" }, "buy": { "choose": "cheapest" } } }),
    )
    .expect("a block");
    assert_eq!(chosen.pricing.sell.policy, SellPolicy::Scarcity);
    assert_eq!(chosen.pricing.buy.choose, BuyChoice::Cheapest);
    let refused = |block: Value| {
        RuleParameters::decode(block)
            .expect_err("refused")
            .to_string()
    };
    assert!(
        refused(json!({ "x": 1 })).contains("pricing"),
        "names pricing"
    );
    for (block, names) in [
        (
            json!({ "every_minutes": 59 }),
            "'every_minutes' is 60 … 10080, not 59",
        ),
        (
            json!({ "every_minutes": 10_081 }),
            "'every_minutes' is 60 … 10080",
        ),
        (
            json!({ "step_permille": 0 }),
            "'step_permille' is 1 … 500, not 0",
        ),
        (
            json!({ "step_permille": 501 }),
            "'step_permille' is 1 … 500",
        ),
        (
            json!({ "deadband_permille": 501 }),
            "'deadband_permille' is 0 … 500",
        ),
        (
            json!({ "floor_permille": 0 }),
            "'floor_permille' is 1 … 1000",
        ),
        (
            json!({ "floor_permille": 1_001 }),
            "'floor_permille' is 1 … 1000",
        ),
        (
            json!({ "ceiling_permille": 999 }),
            "'ceiling_permille' is 1000 … 10000",
        ),
        (
            json!({ "ceiling_permille": 10_001 }),
            "'ceiling_permille' is 1000 … 10000",
        ),
        (json!({ "low_below": 1_001 }), "'low_below' is 0 … 1000"),
        (json!({ "full_above": 0 }), "'full_above' is 1 … 100000"),
        (json!({ "policy": "follow" }), "unknown variant `follow`"),
        (
            json!({ "list": { "categories": ["Goods"] } }),
            "'Goods' is not an item category",
        ),
        (json!({ "list": { "kinds": [] } }), "unknown field `kinds`"),
        (
            json!({ "tags": ["a", "b", "c", "d", "e", "f", "g", "h", "i"] }),
            "'sell.tags' lists at most 8 tags, not 9",
        ),
        (
            json!({ "accept_at_least_permille": 900 }),
            "unknown field `accept_at_least_permille`",
        ),
    ] {
        let message = refused(json!({ "pricing": { "sell": block } }));
        assert!(message.contains(names), "{message} should name {names}");
    }
    for (block, names) in [
        (json!({ "choose": "nearest" }), "unknown variant `nearest`"),
        (
            json!({ "haggle": { "offer_permille": 900 } }),
            "unknown field `haggle`",
        ),
    ] {
        let message = refused(json!({ "pricing": { "buy": block } }));
        assert!(message.contains(names), "{message} should name {names}");
    }
}

#[test]
fn a_rule_seat_with_the_default_block_decides_exactly_as_before() {
    let plain = PacedRuleController::new(7, SimDuration::from_seconds(PACE));
    let priced = plain.clone().pricing(Pricing::default());
    for n in 0..32 {
        let observation = Market::keeper(index(n)).suggest(330).build();
        assert_eq!(
            plain.decide(&observation),
            priced.decide(&observation),
            "consult {n}"
        );
    }
    // And a rule seat with `toward-suggestion` prices bread at bread's index.
    let keeper = plain.pricing(toward());
    let decided = keeper.decide(&Market::keeper(index(2)).suggest(330).build());
    assert_eq!(what(decided), set_price(BREAD, 303, true));
}

#[test]
fn the_slots_fire_once_per_window() {
    // A slot at 08:00 lies in exactly one consult window of a seat with offset 0 and of one with offset 3.
    for offset in [0, 3] {
        let holding: Vec<i64> = (0..96 * 2)
            .map(|m| offset + m * PACE)
            .filter(|t| in_window(*t, PACE, 8 * 3_600))
            .collect();
        assert_eq!(holding.len(), 1, "offset {offset}");
        let due: Vec<i64> = (0..96)
            .map(|m| 86_400 + offset + m * PACE)
            .filter(|t| publish_due(*t, PACE, 480, 1_200, 240))
            .collect();
        let hours: Vec<i64> = due.iter().map(|t| (t - 86_400) / 3_600).collect();
        assert_eq!(hours, [8, 12, 16, 20], "offset {offset}");
    }
    // Consecutive consults take consecutive indices within a period.
    let indices: Vec<u64> = (0..17)
        .map(|n| period_index(SLOT + 3 + n * PACE, PACE, 240).expect("an index"))
        .collect();
    assert_eq!(indices, (0..16).chain(0..1).collect::<Vec<u64>>());
}
