//! The pricing block through `motivated` (`pr-s31d-reporter-policies.md` AC-d-5 (m)): the seller band after
//! the reply and before scoring, and the buyer hook on a chosen offer — the same policies as `rule`'s.

use mineworld_contracts::{
    ActionRequest, ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType, Location,
    Observation, PerceivedEntity, PlaceId, SimDuration, SpatialRequirement, WorldTime,
};
use mineworld_decision::ModelParameters;
use serde_json::{Value, json};

use crate::model::decide;
use crate::options::{self, Choice, Context};
use crate::parameters::MotivatedParameters;

const ME: u64 = 3;
const GRACE: u64 = 4;
const STORE: u64 = 31;
const BREAD: u64 = 11;
const PACE: SimDuration = SimDuration::from_seconds(900);
const STRIDE: SimDuration = SimDuration::from_seconds(30);
/// Index 2 of the 240-minute period that starts at 08:00 on day 2: bread's.
const BREAD_SLOT: i64 = 86_400 + 8 * 3_600 + 2 * 900;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn item(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "item" })
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

/// Felix keeps the store (apple 100, bread 300); the board suggests bread at 330; two bread offers.
fn store(at: i64) -> Observation<Value> {
    let place = PlaceId::new(id(STORE), EntityType::Place).expect("a place");
    let here = Location::in_place(place);
    let listing = json!({
        "operator": { "entity": "40", "entity_type": "organization" },
        "listed": [
            { "item": item(10), "price": 100, "in_stock": 4 },
            { "item": item(BREAD), "price": 300, "in_stock": 5 },
        ],
    });
    let site = PerceivedEntity::new(id(STORE), EntityType::Place)
        .with_components(vec![record(STORE, "shop", 2, listing)]);
    let board = json!({ "notices": [{
        "id": 1,
        "topic": "prices.ingredients",
        "publisher": { "entity": "5", "entity_type": "person" },
        "body": { "prices": [{ "item": BREAD.to_string(), "price": 330, "basis": 3 }] },
        "issued_at": at - 600,
        "valid_until": at + 20_000,
    }]});
    let currency = json!({
        "name": "US dollar", "symbol": "$", "exponent": 2, "symbol_at": "before",
        "references": [{ "item": item(10), "value": 100 }, { "item": item(BREAD), "value": 300 }],
    });
    let me = PerceivedEntity::new(id(ME), EntityType::Person)
        .at(here)
        .with_components(vec![
            record(ME, "currency", 1, currency),
            record(ME, "notice-board", 1, board),
        ]);
    let grace = PerceivedEntity::new(id(GRACE), EntityType::Person).at(here);
    let buy = |price: u64, seller: Option<u64>| {
        Affordance::available(
            ActionTypeId::from_static("buy"),
            seller.map(id),
            SpatialRequirement::NONE,
        )
        .with_payload(json!({ "item": item(BREAD), "price": price }))
    };
    Observation::new(id(ME), WorldTime::from_seconds(at))
        .at_location(here)
        .perceiving(vec![site, me, grace])
        .offering(vec![
            Affordance::available(
                ActionTypeId::from_static("set-price"),
                None,
                SpatialRequirement::at_place(place),
            ),
            buy(300, None),
            buy(260, Some(GRACE)),
        ])
}

fn parameters(pricing: Value) -> MotivatedParameters {
    MotivatedParameters::decode(json!({ "pricing": pricing })).expect("a block")
}

fn payload(request: &ActionRequest) -> Value {
    serde_json::from_slice(request.payload().payload()).expect("JSON")
}

#[test]
fn a_motivated_keeper_moves_toward_the_suggestion_at_its_index() {
    let toward = parameters(json!({ "sell": { "policy": "toward-suggestion" } }));
    let decided =
        decide(toward.settings(), 7, PACE, STRIDE, &store(BREAD_SLOT)).expect("a request");
    assert_eq!(decided.action_type().as_str(), "set-price");
    assert_eq!(
        payload(&decided),
        json!({ "item": item(BREAD), "price": 303, "for": "shop" })
    );
    // The default block decides as before: never a set-price.
    let plain = MotivatedParameters::default();
    for n in 0..16 {
        let at = BREAD_SLOT + n * 900;
        let decided = decide(plain.settings(), 7, PACE, STRIDE, &store(at));
        assert!(
            decided.is_none_or(|request| request.action_type().as_str() != "set-price"),
            "consult {n}"
        );
    }
}

#[test]
fn a_motivated_buyer_buys_the_cheapest_when_its_block_says_so() {
    let observation = store(BREAD_SLOT);
    let cheapest = parameters(json!({ "buy": { "choose": "cheapest" } }));
    let context = Context::new(&observation, cheapest.settings()).expect("a context");
    let bought =
        options::request(&Choice::Offered(1), &context, "Hello", "chat", (0, 0)).expect("a buy");
    assert_eq!(bought.target(), Some(id(GRACE)));
    assert_eq!(
        payload(&bought),
        json!({ "item": item(BREAD), "price": 260 })
    );
    let plain = MotivatedParameters::default();
    let context = Context::new(&observation, plain.settings()).expect("a context");
    let bought =
        options::request(&Choice::Offered(1), &context, "Hello", "chat", (0, 0)).expect("a buy");
    assert_eq!(bought.target(), None);
}
