//! Haggling (`ARC-118`; S31-h AC-h-2, AC-h-3, AC-h-4, AC-h-5, AC-h-7) over the hand-built store
//! through presence's real `observe` and the kernel's real dispatch. Oracles are literal prices,
//! wallets and codes written here.
//!
//! W1: Grace, a person in the store 2.2 m from Bob, asks bread 300, pen 150 and apple 120 and carries
//! two loaves, a pen and three apples; Bob has 1 000. The store (keeper Felix) asks apple 100, bread 300
//! and juice 200, with no juice in stock.

mod support;

use mineworld_contracts::{
    ActionResult, Affordance, EntityType, LocalPosition, Location, Millimetres, Observation,
    PlaceId, Rejection, RejectionCode,
};
use mineworld_economy::code::{
    CANNOT_CARRY, INSUFFICIENT_FUNDS, NO_SUCH_PROPOSAL, NOT_FOR_SALE, OUT_OF_STOCK,
    PRICE_OUT_OF_RANGE,
};
use mineworld_economy::{
    AcceptProposal, DeclineProposal, PRICE_MAX, ProposalId, ProposalStatus, Proposals,
    ProposalsView, ProposePrice, WithdrawProposal,
};
use mineworld_kernel::Dispatched;
use mineworld_movement::Move;
use serde_json::Value;
use support::{SHOP, Setup, Town};

fn code(code: RejectionCode) -> Rejection {
    Rejection::System { code, detail: None }
}

fn limit(detail: &str) -> Rejection {
    Rejection::System {
        code: mineworld_economy::code::PROPOSAL_LIMIT,
        detail: Some(detail.to_owned()),
    }
}

/// W1 as the module's header describes it.
fn w1() -> Setup {
    Setup::store(SHOP)
        .with_economy(
            "grace",
            "{ wallet: 1000, asks: { bread: 300, pen: 150, apple: 120 } }",
        )
        .carrying("grace", "{ bread: 2, pen: 1, apple: 3 }")
        .with_economy("bob", "{ wallet: 1000 }")
}

fn town(setup: &Setup) -> Town {
    Town::build(setup, |_| {}).expect("begins")
}

fn propose(
    town: &mut Town,
    buyer: &str,
    seller: Option<&str>,
    item: &str,
    price: u64,
    id: u64,
) -> Dispatched {
    let action = ProposePrice::new(town.item(item), price);
    town.request(buyer, seller, &action, id).expect("answers")
}

fn accept(town: &mut Town, actor: &str, buyer: &str, proposal: u64, id: u64) -> Dispatched {
    let action = AcceptProposal::new(ProposalId::from_raw(proposal));
    town.request(actor, Some(buyer), &action, id)
        .expect("answers")
}

fn decline(town: &mut Town, actor: &str, proposal: u64, id: u64) -> Dispatched {
    let action = DeclineProposal::new(ProposalId::from_raw(proposal));
    town.request(actor, None, &action, id).expect("answers")
}

fn withdraw(town: &mut Town, actor: &str, proposal: u64, id: u64) -> Dispatched {
    let action = WithdrawProposal::new(ProposalId::from_raw(proposal));
    town.request(actor, None, &action, id).expect("answers")
}

fn kinds(done: &Dispatched) -> Vec<&str> {
    match done.result() {
        ActionResult::Accepted { .. } => done
            .events()
            .iter()
            .map(|fact| fact.event_type().as_str())
            .collect(),
        other => panic!("refused: {other:?}"),
    }
}

fn refusal(done: &Dispatched) -> Rejection {
    match done.result() {
        ActionResult::Rejected(reason) => {
            assert!(done.events().is_empty(), "a refusal states nothing");
            reason.clone()
        }
        other => panic!("accepted: {other:?}"),
    }
}

/// One holder's name, wallet, holdings and asks (apple, bread, juice, pen).
type Holder = (String, Option<u64>, Vec<(String, u32)>, Vec<u64>);

/// Every wallet, holding and ask of the town's holders: "nothing is reserved" is compared on these.
fn money_and_goods(town: &Town) -> Vec<Holder> {
    support::HOLDERS
        .iter()
        .map(|holder| {
            let asks = ["apple", "bread", "juice", "pen"]
                .iter()
                .map(|item| town.ask(holder, item))
                .collect();
            (
                (*holder).to_owned(),
                town.wallet(holder),
                town.holdings(holder),
                asks,
            )
        })
        .collect()
}

/// The `proposals` record `observer` is told about itself, if any.
fn record(town: &Town, observer: &str) -> Option<ProposalsView> {
    let seen = town.observation(observer, town.world.now());
    seen.entity(town.id(observer)).and_then(|entity| {
        entity
            .components()
            .iter()
            .find(|record| {
                *record.component_type()
                    == <Proposals as mineworld_contracts::Component>::COMPONENT_TYPE
            })
            .map(|record| serde_json::from_value(record.payload().clone()).expect("a record"))
    })
}

/// The affordances of `action` in an observation, with the target each names.
fn offered<'a>(seen: &'a Observation<Value>, action: &str) -> Vec<&'a Affordance<Value>> {
    seen.affordances()
        .iter()
        .filter(|affordance| affordance.action_type().as_str() == action)
        .collect()
}

fn place(town: &Town, name: &str) -> PlaceId {
    PlaceId::new(town.id(name), EntityType::Place).expect("a place")
}

/// `person` steps to (`x`, `y`) in `at`, as request `id`.
fn step(town: &mut Town, person: &str, at: &str, x: i32, y: i32, id: u64) {
    let to = Location::in_place(place(town, at)).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ));
    let done = town
        .request(person, None, &Move::new(to), id)
        .expect("answers");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{person} steps: {:?}",
        done.result()
    );
}

/// AC-h-2 and §5.7, §5.9: a proposal is heard by the buyer and the seller only, shows in both their
/// records, and writes no wallet, holding or ask; a decline is kept, marked declined.
#[test]
fn a_proposal_is_heard_by_two_shows_in_their_records_and_reserves_nothing() {
    let mut town = town(&w1());
    let before = money_and_goods(&town);
    let done = propose(&mut town, "bob", Some("grace"), "bread", 250, 1);
    assert_eq!(kinds(&done), ["price-proposed"]);
    assert_eq!(money_and_goods(&town), before, "I-h-6: nothing is reserved");
    for (who, hears) in [
        ("bob", true),
        ("grace", true),
        ("alice", false),
        ("felix", false),
    ] {
        assert_eq!(town.perceived(who, "price-proposed"), hears, "{who}");
    }
    let bob = record(&town, "bob").expect("bob's record");
    assert!(bob.incoming().is_empty());
    let line = bob.outgoing()[0];
    assert_eq!(
        (
            line.proposal(),
            line.price(),
            line.ask(),
            line.status(),
            line.for_shop()
        ),
        (
            ProposalId::from_raw(1),
            250,
            Some(300),
            ProposalStatus::Open,
            false
        )
    );
    assert_eq!(line.valid_until().seconds(), 1 + 900);
    let grace = record(&town, "grace").expect("grace's record");
    assert_eq!(grace.incoming(), bob.outgoing());
    assert!(grace.outgoing().is_empty());
    assert!(
        record(&town, "alice").is_none(),
        "nobody else sees a proposal"
    );

    let done = decline(&mut town, "grace", 1, 2);
    assert_eq!(kinds(&done), ["proposal-declined"]);
    assert_eq!(
        money_and_goods(&town),
        before,
        "a decline reserves nothing either"
    );
    assert!(!town.perceived("alice", "proposal-declined"));
    let bob = record(&town, "bob").expect("the decline is kept for bob");
    assert_eq!(bob.outgoing()[0].status(), ProposalStatus::Declined);
    assert!(
        record(&town, "grace")
            .is_some_and(|grace| grace.incoming()[0].status() == ProposalStatus::Declined),
        "grace sees what she declined until it expires"
    );
}

/// AC-h-3: acceptance re-checks everything at that instant, and each refusal writes nothing.
#[test]
fn acceptance_rechecks_every_buy_check_and_a_refusal_moves_nothing() {
    let refused_after = |setup: Setup, between: &dyn Fn(&mut Town)| -> Rejection {
        let mut town = town(&setup);
        assert_eq!(
            kinds(&propose(&mut town, "bob", Some("grace"), "bread", 250, 1)),
            ["price-proposed"]
        );
        between(&mut town);
        let before = town.state();
        let reason = refusal(&accept(&mut town, "grace", "bob", 1, 50));
        assert_eq!(town.state(), before, "a refused acceptance writes nothing");
        reason
    };
    // Bob spends his money first.
    let spent = refused_after(w1().with_economy("bob", "{ wallet: 300 }"), &|town| {
        assert!(matches!(
            town.buy("bob", "apple", 2).expect("answers").result(),
            ActionResult::Accepted { .. }
        ));
    });
    assert_eq!(spent, code(INSUFFICIENT_FUNDS));
    // Grace sells her last loaf to Alice first.
    let sold = refused_after(w1().carrying("grace", "{ bread: 1 }"), &|town| {
        let done = town
            .buy_at("alice", Some("grace"), "bread", 300, 2)
            .expect("answers");
        assert!(matches!(done.result(), ActionResult::Accepted { .. }));
    });
    assert_eq!(sold, code(OUT_OF_STOCK));
    // Bob walks off to 3.2 m.
    let away = refused_after(w1(), &|town| step(town, "bob", "store", 3_500, 1_000, 2));
    assert_eq!(away, Rejection::TooFarAway);
    // Bob fills his hands first.
    let full = refused_after(w1().carrying("bob", "{ pen: 5 }"), &|town| {
        assert!(matches!(
            town.buy("bob", "apple", 2).expect("answers").result(),
            ActionResult::Accepted { .. }
        ));
    });
    assert_eq!(full, code(CANNOT_CARRY));
    // Alice is not the seller.
    let mut other = town(&w1());
    let _ = propose(&mut other, "bob", Some("grace"), "bread", 250, 1);
    assert_eq!(
        refusal(&accept(&mut other, "alice", "bob", 1, 2)),
        Rejection::PermissionDenied
    );
    // Aimed at someone other than the buyer.
    assert_eq!(
        refusal(&accept(&mut other, "grace", "alice", 1, 3)),
        Rejection::PreconditionFailed
    );
}

/// AC-h-3, AC-S31-20: at exactly `made_at + 900 s` the proposal is dead: no offer, and
/// `no-such-proposal`; one second before, it is still answerable.
#[test]
fn a_proposal_expires_at_its_valid_until_with_no_fact() {
    let mut town = town(&w1());
    let _ = propose(&mut town, "bob", Some("grace"), "bread", 250, 1);
    let _ = town.world.advance_to(support::t(900)).expect("advances");
    let seen = town.observation("grace", town.world.now());
    assert_eq!(offered(&seen, "accept-proposal").len(), 1, "alive at 900");
    let mut late = town;
    let before = late.state();
    assert_eq!(
        refusal(&accept(&mut late, "grace", "bob", 1, 901)),
        code(NO_SUCH_PROPOSAL)
    );
    assert_eq!(late.state(), before);
    let seen = late.observation("grace", late.world.now());
    for action in ["accept-proposal", "decline-proposal"] {
        assert!(offered(&seen, action).is_empty(), "{action} gone at 901");
    }
    assert!(
        record(&late, "bob").is_none(),
        "an expired proposal leaves the record"
    );
    assert_eq!(
        refusal(&decline(&mut late, "grace", 1, 902)),
        code(NO_SUCH_PROPOSAL)
    );
    assert_eq!(
        refusal(&withdraw(&mut late, "bob", 1, 903)),
        code(NO_SUCH_PROPOSAL)
    );
}

/// AC-h-3: a keeper answers a store proposal only while standing in the store.
#[test]
fn a_keeper_answers_a_shop_proposal_only_from_inside_the_shop() {
    let mut town = town(&w1());
    assert_eq!(
        kinds(&propose(&mut town, "bob", None, "bread", 250, 1)),
        ["price-proposed"]
    );
    assert!(
        town.perceived("felix", "price-proposed"),
        "the keeper in the store hears it"
    );
    assert!(
        !town.perceived("grace", "price-proposed"),
        "a customer does not"
    );
    let felix = record(&town, "felix").expect("felix's record");
    assert!(felix.incoming()[0].for_shop());
    step(&mut town, "felix", "street", 1_000, 1_000, 2);
    assert!(
        record(&town, "felix").is_none(),
        "outside, felix sees none of the store's"
    );
    let seen = town.observation("felix", town.world.now());
    assert!(offered(&seen, "decline-proposal").is_empty());
    assert_eq!(
        refusal(&accept(&mut town, "felix", "bob", 1, 3)),
        Rejection::NoSupportedInteraction
    );
    step(&mut town, "felix", "store", 4_000, 1_000, 4);
    let done = accept(&mut town, "felix", "bob", 1, 5);
    assert_eq!(
        kinds(&done),
        [
            "proposal-accepted",
            "sold",
            "money-transferred",
            "items-transferred"
        ]
    );
    assert_eq!(town.wallet("bob"), Some(750));
    assert_eq!(town.wallet("corner-store"), Some(1_250));
}

/// AC-h-4: the limits — eight live proposals per seller, one per buyer and kind, the decline memory,
/// and a withdrawal that frees the kind at once.
#[test]
fn the_limits_hold_and_a_decline_blocks_the_kind_until_it_expires() {
    let setup = w1()
        .carrying("erin", "{ pen: 1 }")
        .with_economy("alice", "{ wallet: 1000 }");
    let mut town = town(&setup);
    let mut id = 0;
    for buyer in ["alice", "bob", "erin"] {
        for item in ["apple", "bread", "pen"] {
            id += 1;
            let done = propose(&mut town, buyer, Some("grace"), item, 100, id);
            if id <= 8 {
                assert_eq!(kinds(&done), ["price-proposed"], "{buyer} {item}");
            } else {
                assert_eq!(refusal(&done), limit("seller-full"), "the ninth");
            }
        }
    }
    let seen = town.observation("felix", town.world.now());
    let _ = seen;

    let mut town = Town::build(&w1(), |_| {}).expect("begins");
    let _ = propose(&mut town, "bob", Some("grace"), "bread", 250, 1);
    assert_eq!(
        refusal(&propose(&mut town, "bob", Some("grace"), "bread", 260, 2)),
        limit("already-proposed")
    );
    assert_eq!(
        kinds(&decline(&mut town, "grace", 1, 3)),
        ["proposal-declined"]
    );
    assert_eq!(
        refusal(&propose(&mut town, "bob", Some("grace"), "bread", 270, 4)),
        limit("already-proposed"),
        "Q-S31h-6: the decline blocks bread to grace until it expires"
    );
    assert_eq!(
        kinds(&propose(&mut town, "bob", Some("grace"), "apple", 100, 5)),
        ["price-proposed"],
        "another kind is free"
    );
    assert_eq!(
        kinds(&propose(&mut town, "bob", Some("grace"), "bread", 270, 901)),
        ["price-proposed"],
        "at valid_until the decline is gone"
    );
    assert_eq!(
        kinds(&withdraw(&mut town, "bob", 901, 902)),
        ["proposal-withdrawn"]
    );
    assert_eq!(
        kinds(&propose(&mut town, "bob", Some("grace"), "bread", 280, 903)),
        ["price-proposed"],
        "a withdrawal frees the kind at once"
    );
}

/// AC-h-5: each named refusal of `propose-price` in §5.5's order, and of `withdraw-proposal`.
#[test]
fn each_refusal_of_a_proposal_is_named() {
    let mut town = town(&w1());
    let cases: [(&str, Option<&str>, &str, u64, Rejection); 9] = [
        (
            "carol",
            None,
            "bread",
            100,
            Rejection::NoSupportedInteraction,
        ),
        ("bob", Some("grace"), "juice", 100, code(NOT_FOR_SALE)),
        ("bob", Some("grace"), "bread", 0, code(PRICE_OUT_OF_RANGE)),
        (
            "bob",
            Some("grace"),
            "bread",
            PRICE_MAX + 1,
            code(PRICE_OUT_OF_RANGE),
        ),
        ("felix", None, "bread", 100, Rejection::PreconditionFailed),
        (
            "bob",
            Some("grace"),
            "bread",
            2_000,
            code(INSUFFICIENT_FUNDS),
        ),
        ("bob", None, "juice", 100, code(OUT_OF_STOCK)),
        ("erin", None, "apple", 50, code(CANNOT_CARRY)),
        ("felix", Some("grace"), "bread", 100, Rejection::TooFarAway),
    ];
    for (id, (buyer, seller, item, price, expected)) in (1..).zip(cases) {
        let before = town.state();
        let done = propose(&mut town, buyer, seller, item, price, id);
        assert_eq!(
            refusal(&done),
            expected,
            "{buyer} → {seller:?} {item} at {price}"
        );
        assert_eq!(town.state(), before);
    }
    // The offer reports the reasons that hold for every kind: a keeper in its own shop.
    let seen = town.observation("felix", town.world.now());
    let to_store: Vec<_> = offered(&seen, "propose-price")
        .into_iter()
        .filter(|affordance| affordance.target().is_none())
        .collect();
    assert_eq!(
        to_store[0].unavailable_reason(),
        Some(&Rejection::PreconditionFailed)
    );

    let _ = propose(&mut town, "bob", Some("grace"), "bread", 250, 20);
    assert_eq!(
        refusal(&withdraw(&mut town, "bob", 77, 21)),
        code(NO_SUCH_PROPOSAL)
    );
    assert_eq!(
        refusal(&withdraw(&mut town, "alice", 20, 22)),
        Rejection::PermissionDenied
    );
}

/// AC-h-7: every haggling offer is incomplete; one `propose-price` per seller, one `accept-proposal`
/// per buyer with an open proposal, one `decline-proposal`, one `withdraw-proposal`.
#[test]
fn every_haggling_offer_is_incomplete_and_per_target() {
    let mut town = town(&w1());
    let seen = town.observation("bob", town.world.now());
    let proposals = offered(&seen, "propose-price");
    let targets: Vec<_> = proposals
        .iter()
        .map(|affordance| affordance.target())
        .collect();
    assert!(targets.contains(&None), "to the store");
    assert!(targets.contains(&Some(town.id("grace"))), "to grace");
    assert_eq!(
        proposals
            .iter()
            .filter(|affordance| affordance.target() == Some(town.id("grace")))
            .count(),
        1,
        "one per seller, whatever its kinds"
    );
    assert!(offered(&seen, "withdraw-proposal").is_empty());

    let _ = propose(&mut town, "bob", Some("grace"), "bread", 250, 1);
    let _ = propose(&mut town, "bob", Some("grace"), "apple", 100, 2);
    let bob = town.observation("bob", town.world.now());
    assert_eq!(offered(&bob, "withdraw-proposal").len(), 1);
    assert!(
        offered(&bob, "accept-proposal").is_empty(),
        "a buyer accepts nothing"
    );
    let grace = town.observation("grace", town.world.now());
    let accepts = offered(&grace, "accept-proposal");
    assert_eq!(accepts.len(), 1, "one per buyer, not per proposal");
    assert_eq!(accepts[0].target(), Some(town.id("bob")));
    assert!(accepts[0].is_available());
    assert_eq!(offered(&grace, "decline-proposal").len(), 1);
    let alice = town.observation("alice", town.world.now());
    assert!(offered(&alice, "accept-proposal").is_empty());
    assert!(offered(&alice, "decline-proposal").is_empty());

    for seen in [&bob, &grace] {
        for action in [
            "propose-price",
            "accept-proposal",
            "decline-proposal",
            "withdraw-proposal",
        ] {
            assert!(
                offered(seen, action)
                    .iter()
                    .all(|affordance| affordance.payload().is_none()),
                "{action} is incomplete"
            );
        }
    }
}
