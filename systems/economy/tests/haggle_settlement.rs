//! Settlement at acceptance (`ARC-118`; S31-h AC-h-1, AC-h-6): an accepted proposal is one dispatch
//! stating `proposal-accepted` and a whole sale at the proposed price, heard as a sale is heard, while
//! the proposal itself reaches only its buyer and those who may answer it.
//!
//! Two compositions: W1 (the store and Grace, a person seller, in test coin) and W2 (the same streets
//! re-authored: a crown with no minor units, the store kept by two keepers, one of them outside). The
//! property's states come from an in-test SplitMix64 (the tree's idiom; no property-test crate,
//! SD-S31a-10), seed [`SEED`], so every run checks the same states.

mod support;

use mineworld_contracts::{
    ActionResult, Affordance, EntityType, LocalPosition, Location, Millimetres, PlaceId, Rejection,
};
use mineworld_economy::{
    AcceptProposal, MoneyTransferred, ProposalAccepted, ProposalId, ProposePrice, Sold,
};
use mineworld_inventory::ItemsTransferred;
use mineworld_kernel::Dispatched;
use mineworld_movement::Move;
use serde_json::Value;
use support::{SHOP, Setup, Town, of, t};

/// The generator's seed.
const SEED: u64 = 0x5331_0a06;

/// How many states the property checks.
const STATES: usize = 10_000;

/// The four facts of an acceptance, in order.
const ACCEPTED: [&str; 4] = [
    "proposal-accepted",
    "sold",
    "money-transferred",
    "items-transferred",
];

/// The four facts a proposal states over its life.
const PROPOSAL_FACTS: [&str; 4] = [
    "price-proposed",
    "proposal-accepted",
    "proposal-declined",
    "proposal-withdrawn",
];

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

/// W1: Grace asks bread 300 and carries two loaves; Bob has 1 000.
fn w1() -> Setup {
    Setup::store(SHOP)
        .with_economy(
            "grace",
            "{ wallet: 1000, asks: { bread: 300, pen: 150, apple: 120 } }",
        )
        .carrying("grace", "{ bread: 2, pen: 1, apple: 3 }")
        .with_economy("bob", "{ wallet: 1000 }")
}

/// W2: crowns with no minor units, store prices in whole crowns, keepers Felix and Alice.
fn w2() -> Setup {
    let mut setup = Setup::store(SHOP)
        .with_economy(
            "corner-store",
            "{ wallet: 50, shop: { at: store, prices: { apple: 3, bread: 5, juice: 4 }, \
             keepers: [felix, alice] } }",
        )
        .with_economy("bob", "{ wallet: 20 }");
    setup.configuration =
        Some("{ currency: { name: crown, symbol: cr, exponent: 0, symbol_at: after } }".to_owned());
    setup
}

fn propose(town: &mut Town, seller: Option<&str>, price: u64, id: u64) -> Dispatched {
    let action = ProposePrice::new(town.item("bread"), price);
    town.request("bob", seller, &action, id).expect("answers")
}

fn accept(town: &mut Town, actor: &str, proposal: u64, id: u64) -> Dispatched {
    let action = AcceptProposal::new(ProposalId::from_raw(proposal));
    town.request(actor, Some("bob"), &action, id)
        .expect("answers")
}

fn kinds(done: &Dispatched) -> Vec<&str> {
    done.events()
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect()
}

fn step(town: &mut Town, person: &str, at: &str, x: i32, y: i32, id: u64) -> bool {
    let place = PlaceId::new(town.id(at), EntityType::Place).expect("a place");
    let to = Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ));
    let done = town
        .request(person, None, &Move::new(to), id)
        .expect("answers");
    matches!(done.result(), ActionResult::Accepted { .. })
}

/// AC-h-1 in W1: Bob, 2.2 m from Grace, proposes 250 against her 300; she accepts. One dispatch holds
/// the four facts sharing one cause; the money is the proposed price; Alice, in the same place, hears
/// the sale and none of the proposal.
#[test]
fn an_accepted_proposal_settles_atomically_at_the_proposed_price() {
    let mut town = Town::build(&w1(), |_| {}).expect("begins");
    assert_eq!(
        kinds(&propose(&mut town, Some("grace"), 250, 1)),
        ["price-proposed"]
    );
    let done = accept(&mut town, "grace", 1, 300);
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    assert_eq!(kinds(&done), ACCEPTED);
    let sold = of::<Sold>(done.events());
    let paid = of::<MoneyTransferred>(done.events());
    let goods = of::<ItemsTransferred>(done.events());
    let agreed = of::<ProposalAccepted>(done.events());
    assert_eq!(sold[0].0.price(), 250);
    assert_eq!(
        paid[0].0.amount(),
        250,
        "I-h-4: the amount is the proposed price"
    );
    assert_eq!(agreed[0].0.terms().price(), 250);
    assert_eq!(goods.len(), 1);
    let cause = done.events()[0].caused_by();
    assert!(done.events().iter().all(|fact| fact.caused_by() == cause));
    assert_eq!(town.wallet("bob"), Some(750));
    assert_eq!(town.wallet("grace"), Some(1_250));
    assert_eq!(town.holdings("bob"), [("bread".to_owned(), 1)]);
    assert!(
        town.perceived("alice", "sold"),
        "a sale is heard by its place"
    );
    for fact in PROPOSAL_FACTS {
        assert!(!town.perceived("alice", fact), "alice never hears {fact}");
        assert!(!town.perceived("carol", fact), "carol never hears {fact}");
    }
    assert!(town.perceived("bob", "proposal-accepted"));
    assert!(town.perceived("grace", "price-proposed"));
}

/// AC-h-1 in W2: a shop proposal accepted by the keeper inside; the other keeper, outside the store,
/// perceives nothing of it.
#[test]
fn a_keeper_settles_a_shop_proposal_and_the_keeper_outside_hears_nothing() {
    let mut town = Town::build(&w2(), |_| {}).expect("begins");
    assert!(step(&mut town, "alice", "street", 2_000, 1_000, 1));
    assert_eq!(kinds(&propose(&mut town, None, 4, 2)), ["price-proposed"]);
    let done = accept(&mut town, "felix", 2, 3);
    assert_eq!(kinds(&done), ACCEPTED);
    assert_eq!(of::<Sold>(done.events())[0].0.price(), 4);
    assert_eq!(town.wallet("bob"), Some(16));
    assert_eq!(town.wallet("corner-store"), Some(54));
    assert!(town.perceived("felix", "price-proposed"));
    for fact in PROPOSAL_FACTS.iter().chain(&["sold"]) {
        assert!(
            !town.perceived("alice", fact),
            "alice, outside, hears no {fact}"
        );
    }
}

/// The `accept-proposal` offer `seen` carries toward Bob.
fn toward_bob(
    seen: &mineworld_contracts::Observation<Value>,
    bob: mineworld_contracts::EntityId,
) -> Option<&Affordance<Value>> {
    seen.affordances().iter().find(|affordance| {
        affordance.action_type().as_str() == "accept-proposal" && affordance.target() == Some(bob)
    })
}

/// AC-h-6: over seeded states — a balance spent or not, stock sold off or not, a buyer who fills its
/// hands or walks away, proposals of every age up to 1 200 s, a person seller or a shop — the
/// `accept-proposal` offer's verdict equals the dispatch's, and every admitted acceptance is the four
/// facts with no kernel fault.
#[test]
fn the_accept_offer_and_the_dispatch_agree_and_every_admitted_acceptance_settles() {
    let mut random = SplitMix64(SEED);
    let (mut accepted, mut refused, mut expired) = (0, 0, 0);
    for state in 0..STATES {
        let person = random.below(2) == 0;
        let wallet = 50 + random.below(551);
        let price = 1 + random.below(500);
        let stock = 1 + random.below(2);
        let carried = random.below(6);
        let (spend, sell_off, walk) = (
            random.below(3) == 0,
            random.below(3) == 0,
            random.below(4) == 0,
        );
        let age = random.below(1_201);
        let mut setup = w1()
            .with_economy("bob", &format!("{{ wallet: {wallet} }}"))
            .carrying("grace", &format!("{{ bread: {stock} }}"));
        setup.stock = format!("{{ apple: 3, bread: {stock} }}");
        if carried > 0 {
            setup = setup.carrying("bob", &format!("{{ pen: {carried} }}"));
        }
        let mut town = Town::build(&setup, |_| {}).expect("begins");
        let (seller, answerer) = if person {
            (Some("grace"), "grace")
        } else {
            (None, "felix")
        };
        let proposed = propose(&mut town, seller, price, 1);
        if !matches!(proposed.result(), ActionResult::Accepted { .. }) {
            continue;
        }
        if spend {
            let _ = town.buy("bob", "apple", 2).expect("answers");
        }
        if sell_off {
            for id in 3..=4 {
                let _ = town
                    .buy_at("alice", seller, "bread", 300, id)
                    .expect("answers");
            }
        }
        if walk {
            step(&mut town, "bob", "store", 3_500, 1_000, 5);
        }
        let at = (1 + age).max(6);
        let _ = town
            .world
            .advance_to(t(i64::try_from(at).expect("small")))
            .expect("advances");
        let seen = town.observation(answerer, town.world.now());
        let verdict = toward_bob(&seen, town.id("bob")).map(|affordance| {
            if affordance.is_available() {
                Ok(())
            } else {
                Err(affordance
                    .unavailable_reason()
                    .cloned()
                    .expect("an unavailable offer says why"))
            }
        });
        let before = town.state();
        let done = town
            .request(
                answerer,
                Some("bob"),
                &AcceptProposal::new(ProposalId::from_raw(1)),
                at,
            )
            .unwrap_or_else(|fault| panic!("state {state}: a kernel fault {fault:?}"));
        match (&verdict, done.result()) {
            (Some(Ok(())), ActionResult::Accepted { .. }) => {
                accepted += 1;
                assert_eq!(kinds(&done), ACCEPTED, "state {state}");
            }
            (Some(Err(reason)), ActionResult::Rejected(answer)) => {
                refused += 1;
                assert_eq!(reason, answer, "state {state}: offer ≡ dispatch");
                assert_eq!(town.state(), before, "state {state}: nothing written");
            }
            (None, ActionResult::Rejected(answer)) => {
                expired += 1;
                assert_eq!(
                    *answer,
                    Rejection::System {
                        code: mineworld_economy::code::NO_SUCH_PROPOSAL,
                        detail: None
                    },
                    "state {state}: no offer means no proposal"
                );
                assert!(
                    at >= 901,
                    "state {state}: only an expired proposal has no offer"
                );
            }
            (verdict, result) => {
                panic!("state {state}: the offer said {verdict:?}, the dispatch {result:?}")
            }
        }
    }
    assert!(
        accepted > STATES / 20 && refused > STATES / 20 && expired > STATES / 20,
        "every outcome is exercised: {accepted} accepted, {refused} refused, {expired} expired"
    );
}
