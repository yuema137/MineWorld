//! The settlement property (S31-a AC-a-5, I-a-10): over seeded states that straddle every boundary — a
//! balance below, at and above the price, a seller holding none or some, a buyer carrying up to its
//! capacity, a seller near or too far, a shop or a person — the offer's verdict equals the dispatch's,
//! and whenever dispatch admits a `buy` it is `Accepted` with exactly the sale's three facts and no
//! kernel fault.
//!
//! The states come from an in-test SplitMix64 (the tree's idiom, `A-28`; no property-test crate,
//! SD-S31a-10), seed [`SEED`], so every run checks the same states.

mod support;

use mineworld_contracts::{ActionResult, Affordance, EntityId, ItemId, Rejection};
use mineworld_economy::Buy;
use serde_json::Value;
use support::{GENESIS, SHOP, Setup, Town, buys};

/// The generator's seed.
const SEED: u64 = 0x5331_0a05;

/// How many states are checked.
const STATES: usize = 10_000;

/// SplitMix64 (Steele, Lea and Flood 2014): small, seedable, and enough to spread states.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// The buy `observer` is offered for `item` from `seller` (a person) or the shop (`None`).
fn offered(
    seen: &mineworld_contracts::Observation<Value>,
    seller: Option<EntityId>,
    item: ItemId,
) -> Option<&Affordance<Value>> {
    buys(seen).into_iter().find(|affordance| {
        affordance.target() == seller
            && affordance.payload().is_some_and(|payload| {
                serde_json::from_value::<Buy>(payload.clone()).is_ok_and(|buy| buy.item() == item)
            })
    })
}

#[test]
fn the_offer_and_the_dispatch_agree_and_every_admitted_buy_settles_in_three_facts() {
    let mut random = SplitMix64(SEED);
    let (mut accepted, mut refused) = (0, 0);
    for state in 0..STATES {
        let wallet = random.below(2_001);
        let ask = 1 + random.below(1_500);
        let stock = random.below(4);
        let carried = random.below(7);
        let person_seller = random.below(2) == 0;
        // Alice stands 2 m from Grace, Felix 3.6 m: a near and a too-far buyer of a person seller.
        let buyer = if random.below(3) == 0 {
            "felix"
        } else {
            "alice"
        };
        let mut setup = Setup::store(SHOP)
            .with_economy(buyer, &format!("{{ wallet: {wallet} }}"))
            .with_economy(
                "grace",
                &format!("{{ wallet: 1000, asks: {{ bread: {ask} }} }}"),
            );
        setup.stock = if stock == 0 {
            "{ apple: 3 }".to_owned()
        } else {
            format!("{{ apple: 3, bread: {stock} }}")
        };
        setup = setup
            .with_economy(
                "corner-store",
                &format!(
                    "{{ wallet: 1000, shop: {{ at: store, prices: {{ apple: 100, bread: {ask}, \
                     juice: 200 }}, keepers: [felix] }} }}"
                ),
            )
            .carrying(
                "grace",
                &if stock == 0 {
                    "{ pen: 1 }".to_owned()
                } else {
                    format!("{{ bread: {stock} }}")
                },
            );
        if carried > 0 {
            setup = setup.carrying(buyer, &format!("{{ pen: {carried} }}"));
        }
        let mut town = Town::build(&setup, |_| {}).expect("begins");
        let seller = person_seller.then_some("grace");
        let target = seller.map(|seller| town.id(seller));
        let seen = town.observation(buyer, GENESIS);
        let bread = town.item("bread");
        let affordance = offered(&seen, target, bread)
            .unwrap_or_else(|| panic!("state {state}: {buyer} is offered the bread"));
        let price = serde_json::from_value::<Buy>(affordance.payload().expect("complete").clone())
            .expect("a buy")
            .price();
        assert_eq!(price, ask, "state {state}: the offer carries the ask");
        let verdict: Result<(), Rejection> = if affordance.is_available() {
            Ok(())
        } else {
            Err(affordance
                .unavailable_reason()
                .cloned()
                .expect("an unavailable offer says why"))
        };
        let before = town.state();
        let done = town
            .buy_at(buyer, seller, "bread", price, 1)
            .unwrap_or_else(|fault| panic!("state {state}: a kernel fault {fault:?}"));
        match (&verdict, done.result()) {
            (Ok(()), ActionResult::Accepted { .. }) => {
                accepted += 1;
                let kinds: Vec<&str> = done
                    .events()
                    .iter()
                    .map(|fact| fact.event_type().as_str())
                    .collect();
                assert_eq!(
                    kinds,
                    ["sold", "money-transferred", "items-transferred"],
                    "state {state}"
                );
            }
            (Err(reason), ActionResult::Rejected(answer)) => {
                refused += 1;
                assert_eq!(reason, answer, "state {state}: offer ≡ dispatch");
                assert!(done.events().is_empty(), "state {state}");
                assert_eq!(town.state(), before, "state {state}: nothing written");
            }
            (verdict, result) => {
                panic!("state {state}: the offer said {verdict:?}, the dispatch {result:?}")
            }
        }
    }
    assert!(
        accepted > STATES / 10 && refused > STATES / 10,
        "both outcomes are exercised: {accepted} accepted, {refused} refused"
    );
}
