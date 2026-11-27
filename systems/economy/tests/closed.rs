//! One availability judgement for every way of buying from a shop (`pr-nightlife.md` N1-C2; N1-I3,
//! N1-I5, N1-I6; `ARC-364`), over the hand-built store of `support` through presence's real `observe`
//! and the kernel's real dispatch. Oracles are hand-computed instants and literal codes.
//!
//! The store sells 08:00–20:00 (or, where a test says, 22:00–06:00); it asks bread 300, apple 100;
//! Felix keeps it; Grace, a person, sells her own bread at 260 in the store; Bob has 1 000.

mod support;

use mineworld_contracts::{ActionResult, Affordance, Observation, Rejection, RejectionCode};
use mineworld_economy::code::{
    INSUFFICIENT_FUNDS, NOT_FOR_SALE, OUT_OF_STOCK, PRICE_CHANGED, SHOP_CLOSED,
};
use mineworld_economy::{AcceptProposal, ProposalId, ProposePrice, SetPrice, ShopSide, ShopStock};
use mineworld_kernel::Dispatched;
use serde_json::Value;
use support::{HOUR, SHOP, STOCK, Setup, Town, t};

const DAY: i64 = 86_400;

/// The store's `economy:` with `hours` (a YAML list).
fn store(hours: &str) -> String {
    format!(
        "{{ wallet: 1000, shop: {{ at: store, prices: {{ apple: 100, bread: 300, juice: 200 }}, \
         keepers: [felix], hours: {hours} }} }}"
    )
}

const DAYTIME: &str = "[{ from: \"08:00\", until: \"20:00\" }]";
const OVERNIGHT: &str = "[{ from: \"22:00\", until: \"06:00\" }]";

/// The store with `hours`, holding `stock`, begun at instant 0 and advanced to `at`.
fn town(hours: &str, stock: &str, at: i64) -> Town {
    let mut setup = Setup::store(SHOP)
        .with_economy("corner-store", &store(hours))
        .with_economy("bob", "{ wallet: 1000 }");
    setup.stock = stock.to_owned();
    let mut town = Town::build(&setup, |_| {}).expect("begins");
    let _ = town.world.advance_to(t(at)).expect("advances");
    town
}

fn code(code: RejectionCode) -> Rejection {
    Rejection::System { code, detail: None }
}

fn refusal(done: &Dispatched) -> Option<Rejection> {
    match done.result() {
        ActionResult::Rejected(reason) => Some(reason.clone()),
        _ => None,
    }
}

/// The affordances of `action` aimed at `target` (`None`: the shop).
fn offered<'a>(
    seen: &'a Observation<Value>,
    action: &str,
    target: Option<mineworld_contracts::EntityId>,
) -> Vec<&'a Affordance<Value>> {
    seen.affordances()
        .iter()
        .filter(|affordance| {
            affordance.action_type().as_str() == action && affordance.target() == target
        })
        .collect()
}

/// What `town`'s store offers `observer` for `action` with no target — for `buy`, the offer of
/// `item` — as `Ok` or its refusal.
fn offer_verdict(
    town: &Town,
    observer: &str,
    action: &str,
    item: Option<&str>,
) -> Result<(), Rejection> {
    let seen = town.observation(observer, town.world.now());
    let wanted = item.map(|item| town.item(item));
    let offers: Vec<_> = offered(&seen, action, None)
        .into_iter()
        .filter(|offer| {
            wanted.is_none_or(|item| {
                offer.payload().is_some_and(|payload| {
                    serde_json::from_value::<mineworld_economy::Buy>(payload.clone())
                        .is_ok_and(|buy| buy.item() == item)
                })
            })
        })
        .collect();
    assert_eq!(offers.len(), 1, "{action} {item:?} offered to {observer}");
    offers[0].unavailable_reason().cloned().map_or(Ok(()), Err)
}

/// The kinds the store's `shop-stock` entry lists for `observer`, by item key.
fn stocked(town: &Town, observer: &str) -> Vec<mineworld_contracts::ItemId> {
    let seen = town.observation(observer, town.world.now());
    let record = seen
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .find(|record| record.component_type().as_str() == "shop-stock")
        .expect("shop-stock disclosed");
    let stock: ShopStock = serde_json::from_value(record.payload().clone()).expect("decodes");
    let store = town.id("store");
    stock
        .shops()
        .iter()
        .find(|shop| shop.place().entity_id() == store)
        .expect("the store is listed, open or closed")
        .stocked()
        .to_vec()
}

/// N1-I3 hand-computed boundaries, 08:00–20:00 and 22:00–06:00: at each instant the `buy` offer, the
/// direct `buy`, the `propose-price` offer and the direct `propose-price` all give the same answer;
/// `shop-stock` agrees; a refusal changes nothing.
#[test]
fn every_way_of_buying_agrees_at_the_open_and_close_instants() {
    let cases: [(&str, i64, bool); 8] = [
        (DAYTIME, 8 * HOUR - 1, false),
        (DAYTIME, 8 * HOUR, true),
        (DAYTIME, 20 * HOUR - 1, true),
        (DAYTIME, 20 * HOUR, false),
        (OVERNIGHT, 22 * HOUR - 1, false),
        (OVERNIGHT, DAY - 1, true),
        (OVERNIGHT, DAY, true),
        (OVERNIGHT, DAY + 6 * HOUR, false),
    ];
    for (hours, at, open) in cases {
        let why = format!("{hours} at {at} s");
        let expected = if open { Ok(()) } else { Err(code(SHOP_CLOSED)) };
        let mut town = town(hours, STOCK, at);
        assert_eq!(
            offer_verdict(&town, "alice", "buy", Some("bread")),
            expected,
            "{why}: buy offer"
        );
        assert_eq!(
            offer_verdict(&town, "bob", "propose-price", None),
            expected,
            "{why}: propose offer"
        );
        let bread = town.item("bread");
        assert_eq!(
            stocked(&town, "alice").contains(&bread),
            open,
            "{why}: shop-stock"
        );
        let before = (town.state(), town.wallet("alice"), town.holdings("alice"));
        let id = u64::try_from(at).expect("positive");
        let bought = town.buy("alice", "bread", id).expect("answers");
        assert_eq!(refusal(&bought), expected.clone().err(), "{why}: buy");
        if !open {
            assert!(bought.events().is_empty(), "{why}: no sold, no transfer");
            assert_eq!(
                (town.state(), town.wallet("alice"), town.holdings("alice")),
                before,
                "{why}: nothing moved"
            );
        }
        // The proposal at the same instant, in a store nobody has bought from.
        let mut fresh = self::town(hours, STOCK, at);
        let proposed = fresh
            .request("bob", None, &ProposePrice::new(bread, 250), id)
            .expect("answers");
        assert_eq!(refusal(&proposed), expected.clone().err(), "{why}: propose");
    }
}

/// N1-I3: a proposal to the shop made while open and accepted after closing is refused `shop-closed`,
/// offer and dispatch alike, and nothing moves; accepted while open it settles; the next morning the
/// shop sells again.
#[test]
fn a_proposal_made_open_cannot_be_accepted_after_closing() {
    // Proposed at 19:50, accepted at 19:55: a sale.
    let mut town = town(DAYTIME, STOCK, 19 * HOUR + 50 * 60);
    let bread = town.item("bread");
    let proposed = town
        .request("bob", None, &ProposePrice::new(bread, 250), 71_400)
        .expect("answers");
    assert_eq!(refusal(&proposed), None);
    let accept = AcceptProposal::new(ProposalId::from_raw(71_400)); // the proposing request's id (SD-S31h-1)
    let accepted = town
        .request("felix", Some("bob"), &accept, 71_700)
        .expect("answers");
    assert_eq!(refusal(&accepted), None, "accepted at 19:55, open");
    assert_eq!(town.wallet("bob"), Some(750));

    // Proposed at 19:50, accepted at 20:02: refused, offer and dispatch.
    let mut town = self::town(DAYTIME, STOCK, 19 * HOUR + 50 * 60);
    let _ = town
        .request("bob", None, &ProposePrice::new(bread, 250), 71_400)
        .expect("answers");
    let _ = town.world.advance_to(t(72_120)).expect("advances");
    let seen = town.observation("felix", town.world.now());
    let accepts = offered(&seen, "accept-proposal", Some(town.id("bob")));
    assert_eq!(accepts.len(), 1);
    assert_eq!(accepts[0].unavailable_reason(), Some(&code(SHOP_CLOSED)));
    let before = (town.state(), town.wallet("bob"), town.holdings("bob"));
    let accepted = town
        .request("felix", Some("bob"), &accept, 72_120)
        .expect("answers");
    assert_eq!(refusal(&accepted), Some(code(SHOP_CLOSED)));
    assert!(accepted.events().is_empty());
    assert_eq!(
        (town.state(), town.wallet("bob"), town.holdings("bob")),
        before
    );

    // The next morning at 08:00 the shop sells again.
    let morning = DAY + 8 * HOUR;
    let bought = town
        .buy("alice", "bread", u64::try_from(morning).expect("positive"))
        .expect("answers");
    assert_eq!(refusal(&bought), None, "reopened at 08:00");
}

/// N1-I3 precedence: while closed, a kind the store holds none of and a buyer who cannot pay both
/// report `shop-closed`, not `out-of-stock` or `insufficient-funds`; a kind it does not sell is still
/// `not-for-sale`, and a stale price still `price-changed`. Open, each reports its own reason.
#[test]
fn closed_comes_after_the_seller_and_price_and_before_funds_and_stock() {
    // The store holds apples only: bread is out of stock.
    for (at, closed) in [(9 * HOUR, false), (21 * HOUR, true)] {
        let mut town = town(DAYTIME, "{ apple: 3 }", at);
        let id = u64::try_from(at).expect("positive");
        let or_closed = |reason: RejectionCode| code(if closed { SHOP_CLOSED } else { reason });
        let out = town.buy("alice", "bread", id).expect("answers");
        assert_eq!(refusal(&out), Some(or_closed(OUT_OF_STOCK)), "at {at}");
        let broke = Setup::store(SHOP).with_economy("corner-store", &store(DAYTIME));
        let mut broke = Town::build(&broke, |_| {}).expect("begins");
        let _ = broke.world.advance_to(t(at)).expect("advances");
        let dear = broke.buy("bob", "bread", id + 2).expect("answers");
        assert_eq!(
            refusal(&dear),
            Some(or_closed(INSUFFICIENT_FUNDS)),
            "bob has 150 for bread at 300, at {at}"
        );
        let pen = town
            .buy_at("alice", None, "pen", 100, id + 3)
            .expect("answers");
        assert_eq!(refusal(&pen), Some(code(NOT_FOR_SALE)), "at {at}");
        let stale = town
            .buy_at("alice", None, "apple", 99, id + 4)
            .expect("answers");
        assert_eq!(refusal(&stale), Some(code(PRICE_CHANGED)), "at {at}");
    }
}

/// N1-I3, N1-I5: a person selling for herself has no hours — Grace sells in the closed store at 23:00
/// — and a keeper may still re-price the closed shop.
#[test]
fn a_person_still_sells_and_a_keeper_still_reprices_in_a_closed_shop() {
    let mut town = town(DAYTIME, STOCK, 23 * HOUR);
    let at = u64::try_from(23 * HOUR).expect("positive");
    // Closed comes before self-trade (N1-I3): the keeper's own offers say `shop-closed` now, and the
    // keeper's refusal (`PreconditionFailed`) once the shop is open.
    assert_eq!(
        offer_verdict(&town, "felix", "propose-price", None),
        Err(code(SHOP_CLOSED))
    );
    assert_eq!(
        offer_verdict(&town, "felix", "buy", Some("bread")),
        Err(code(SHOP_CLOSED))
    );
    let open = self::town(DAYTIME, STOCK, 9 * HOUR);
    assert_eq!(
        offer_verdict(&open, "felix", "propose-price", None),
        Err(Rejection::PreconditionFailed)
    );
    assert_eq!(
        offer_verdict(&open, "felix", "buy", Some("bread")),
        Err(Rejection::PreconditionFailed)
    );
    let from_grace = town
        .buy_at("alice", Some("grace"), "bread", 260, at)
        .expect("answers");
    assert_eq!(refusal(&from_grace), None, "person to person");
    let bread = town.item("bread");
    let repriced = town
        .sell(
            "felix",
            &SetPrice::new(bread, 320, Some(ShopSide::Shop)),
            at + 1,
        )
        .expect("answers");
    assert_eq!(refusal(&repriced), None, "a keeper re-prices while closed");
    assert_eq!(town.ask("corner-store", "bread"), 320);
}

/// Without `hours` the store behaves exactly as before: open at 03:00, every kind on sale.
#[test]
fn a_shop_without_hours_is_always_open() {
    let mut town = Town::begun(SHOP);
    let _ = town.world.advance_to(t(3 * HOUR)).expect("advances");
    assert_eq!(offer_verdict(&town, "alice", "buy", Some("apple")), Ok(()));
    assert_eq!(stocked(&town, "alice").len(), 2);
    let bought = town
        .buy("alice", "apple", u64::try_from(3 * HOUR).expect("positive"))
        .expect("answers");
    assert_eq!(refusal(&bought), None);
}
