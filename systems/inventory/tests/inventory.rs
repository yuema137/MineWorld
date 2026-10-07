//! Holdings over a hand-built world through the kernel's real genesis and dispatch: seeded through the
//! section contract, changed only by inventory's reductions, decided by the owner even when a stater
//! skips the checked constructor (`ARC-26`), bounded for a person, and disclosed to the holder alone.

mod support;

use mineworld_authoring::AuthoredSection;
use mineworld_contracts::{
    ActionResult, Causation, Component, Event, EventEnvelope, Rejection, Visibility,
};
use mineworld_inventory::{
    AuthoredHoldings, Holdings, InventorySystem, ItemsConsumed, ItemsProduced, ItemsTransferred,
    PERSON_CAPACITY, Stocked, can_take, consume, produce, transfer,
};
use mineworld_kernel::{Emission, KernelError, SystemIdentity};
use support::{Change, GENESIS, Pass, Town, Way, begun_with_workshop, change, pass, state};

fn owned(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
    pairs.iter().map(|(k, n)| ((*k).to_owned(), *n)).collect()
}

fn refusal(result: Result<impl core::fmt::Debug, KernelError>) -> (String, Rejection) {
    match result {
        Err(KernelError::FactRefusedByOwner {
            system,
            event_type,
            reason,
        }) => {
            assert_eq!(
                system,
                InventorySystem::ID,
                "refused by inventory, the owner"
            );
            (event_type.as_str().to_owned(), reason)
        }
        other => panic!("inventory must refuse as the owner, but got {other:?}"),
    }
}

#[test]
fn genesis_stocks_people_and_an_organization_each_visible_to_its_holder() {
    let (town, genesis) = Town::begun();
    assert_eq!(town.holdings("alice"), owned(&[("coffee", 2), ("tea", 1)]));
    assert_eq!(town.holdings("bob"), owned(&[("apple", 1)]));
    assert_eq!(
        town.holdings("kiosk"),
        owned(&[("apple", 9), ("coffee", 20)]),
        "an organization holds past a person's capacity"
    );
    assert!(town.holdings("carol").is_empty(), "no section, no holdings");

    let stocked: Vec<&EventEnvelope> = genesis
        .iter()
        .filter(|fact| *fact.event_type() == Stocked::EVENT_TYPE)
        .collect();
    assert_eq!(stocked.len(), 5, "one `stocked` per authored kind");
    for fact in stocked {
        assert_eq!(*fact.caused_by(), Causation::WorldGenesis);
        let holder = serde_json::from_slice::<Stocked>(
            fact.payload().payload_for::<Stocked>().expect("labelled"),
        )
        .expect("decodes")
        .holder();
        assert_eq!(
            *fact.visibility(),
            Visibility::Entities([holder].into_iter().collect()),
            "what somebody holds is theirs to know"
        );
    }
}

#[test]
fn a_transfer_through_the_constructor_moves_counts_and_leaves_no_zero_entry() {
    let (mut town, _) = Town::begun();
    let (alice, bob) = (town.id("alice"), town.id("bob"));
    let coffee = town.item("coffee");
    let done = pass(
        &mut town.world,
        alice,
        &Pass {
            to: bob,
            item: coffee,
            count: 2,
            forged: false,
        },
        1,
    )
    .expect("dispatch answers");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "{:?}",
        done.result()
    );
    let moved: Vec<&EventEnvelope> = done
        .events()
        .iter()
        .filter(|fact| *fact.event_type() == ItemsTransferred::EVENT_TYPE)
        .collect();
    assert_eq!(moved.len(), 1);
    assert_eq!(*moved[0].visibility(), Visibility::Participants);
    assert_eq!(moved[0].participants(), &[alice, bob]);
    assert_eq!(
        town.holdings("alice"),
        owned(&[("tea", 1)]),
        "no zero entry"
    );
    assert_eq!(
        town.holdings("bob"),
        owned(&[("apple", 1), ("coffee", 2)]),
        "kept in item order"
    );
}

/// D-4: a stater that skips the constructor is refused by the owner at reduction — more than the giver
/// holds, an undeclared kind, a count of zero, to oneself — and nothing is written. The same forged
/// path with a valid value is reduced: the positive control that makes the refusal the check.
#[test]
fn transfers_stated_past_the_constructor_are_refused_by_the_owner_and_write_nothing() {
    let (mut town, _) = Town::begun();
    let (alice, bob) = (town.id("alice"), town.id("bob"));
    let (coffee, lantern) = (town.item("coffee"), town.item("lantern"));
    let forged = |to, item, count| Pass {
        to,
        item,
        count,
        forged: true,
    };
    for (id, (what, request)) in [
        ("more than the giver holds", forged(bob, coffee, 3)),
        ("an undeclared kind", forged(bob, lantern, 1)),
        ("a count of zero", forged(bob, coffee, 0)),
        ("to oneself", forged(alice, coffee, 1)),
    ]
    .into_iter()
    .enumerate()
    {
        let before = state(&town.world);
        let (event_type, reason) = refusal(pass(
            &mut town.world,
            alice,
            &request,
            u64::try_from(id).expect("small") + 1,
        ));
        assert_eq!(event_type, "items-transferred", "{what}");
        assert_eq!(reason, Rejection::PreconditionFailed, "{what}");
        assert_eq!(state(&town.world), before, "{what}: nothing written");
    }
    let accepted = pass(&mut town.world, alice, &forged(bob, coffee, 1), 9)
        .expect("a valid forged transfer reduces");
    assert!(matches!(accepted.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        town.holdings("alice"),
        owned(&[("coffee", 1), ("tea", 1)]),
        "positive control: the owner reduced a fact another system stated"
    );
}

/// D-4 for `stocked`: stated at genesis past the seed, an undeclared kind or a count of zero is
/// refused by inventory and the world does not begin.
#[test]
fn stock_stated_past_the_seed_is_refused_by_the_owner() {
    for (what, item, count) in [("undeclared", "lantern", 1), ("zero", "coffee", 0)] {
        let (mut town, mut facts) = Town::assemble();
        let forged = serde_json::to_vec(&serde_json::json!({
            "holder": town.id("carol"),
            "item": town.item(item),
            "count": count,
        }))
        .expect("encodes");
        facts.push(Emission::new::<Stocked>(forged, Visibility::SystemInternal));
        let (event_type, reason) = refusal(town.world.genesis(GENESIS, facts));
        assert_eq!(event_type, "stocked", "{what}");
        assert_eq!(reason, Rejection::PreconditionFailed, "{what}");
    }
}

/// D-5: a person carries at most six, all kinds together; an organization is not bounded.
#[test]
fn a_person_carries_at_most_six_and_an_organization_is_unbounded() {
    assert_eq!(PERSON_CAPACITY, 6);
    let (mut town, _) = Town::begun();
    let (kiosk, bob) = (town.id("kiosk"), town.id("bob"));
    let coffee = town.item("coffee");
    let honest = |count| Pass {
        to: bob,
        item: coffee,
        count,
        forged: false,
    };
    let five = pass(&mut town.world, kiosk, &honest(5), 1).expect("answers");
    assert!(matches!(five.result(), ActionResult::Accepted { .. }));
    assert_eq!(town.holdings("bob"), owned(&[("apple", 1), ("coffee", 5)]));
    assert!(!can_take(&town.world.read(), bob, 1), "bob is full");

    assert_eq!(
        transfer(&town.world.read(), kiosk, bob, coffee, 1).expect_err("past capacity"),
        Rejection::TargetUnavailable,
        "the constructor refuses a transfer past capacity"
    );
    let refused = pass(&mut town.world, kiosk, &honest(1), 2).expect("answers");
    assert_eq!(
        *refused.result(),
        ActionResult::Rejected(Rejection::TargetUnavailable),
        "and so does a deciding pack's validate"
    );
    let before = state(&town.world);
    let forged = Pass {
        forged: true,
        ..honest(1)
    };
    let (_, reason) = refusal(pass(&mut town.world, kiosk, &forged, 3));
    assert_eq!(
        reason,
        Rejection::TargetUnavailable,
        "and so does the reduction"
    );
    assert_eq!(state(&town.world), before);

    let back = pass(
        &mut town.world,
        bob,
        &Pass {
            to: kiosk,
            item: coffee,
            count: 5,
            forged: false,
        },
        4,
    )
    .expect("answers");
    assert!(matches!(back.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        town.holdings("kiosk"),
        owned(&[("apple", 9), ("coffee", 20)]),
        "an organization takes any number"
    );
}

#[test]
fn authored_holdings_past_capacity_are_refused_at_seeding() {
    let town = Town::empty();
    assert_eq!(
        town.seed_holdings("carol", "{ coffee: 4, tea: 3 }")
            .expect_err("seven"),
        Rejection::TargetUnavailable,
        "a person authored with seven items"
    );
    assert_eq!(
        town.seed_holdings("carol", "{ coffee: 4, tea: 2 }")
            .expect("six is the bound, inclusive")
            .len(),
        2
    );
    assert!(
        town.seed_holdings("kiosk", "{ coffee: 400 }").is_ok(),
        "an organization is unbounded"
    );
}

#[test]
fn a_section_is_checked_as_it_is_decoded_and_as_it_is_seeded() {
    let decode =
        |text: &str| serde_saphyr::from_str::<<InventorySystem as AuthoredSection>::Authored>(text);
    assert!(
        decode("{ coffee: 0 }").is_err(),
        "a count of zero is refused at its line"
    );
    assert!(decode("{ coffee: -1 }").is_err());
    let authored: AuthoredHoldings = decode("{ tea: 1, coffee: 2 }").expect("valid");
    assert_eq!(
        InventorySystem::references(&authored)
            .iter()
            .map(|reference| (reference.key.as_str(), reference.entity_type))
            .collect::<Vec<_>>(),
        [
            ("coffee", mineworld_contracts::EntityType::Item),
            ("tea", mineworld_contracts::EntityType::Item)
        ],
        "every key must name an Item"
    );
    let town = Town::empty();
    assert_eq!(
        town.seed_holdings("cafe", "{ coffee: 1 }")
            .expect_err("a place holds nothing"),
        Rejection::PreconditionFailed
    );
    assert_eq!(
        town.seed_holdings("alice", "{ bob: 1 }")
            .expect_err("bob is not an item"),
        Rejection::PreconditionFailed
    );
}

#[test]
fn holdings_are_disclosed_to_the_holder_and_to_nobody_else() {
    let (town, _) = Town::begun();
    let alice = town.observation("alice");
    let held = |subject| {
        alice.entity(subject).map(|entity| {
            entity
                .components()
                .iter()
                .filter(|record| *record.component_type() == Holdings::COMPONENT_TYPE)
                .count()
        })
    };
    assert_eq!(held(town.id("alice")), Some(1), "Alice is told her own");
    assert_eq!(
        held(town.id("bob")),
        Some(0),
        "Alice perceives Bob and is not told what he carries"
    );
}

/// E-3, the positive half: through the checked constructors, production and consumption change
/// exactly one holder by exactly the count, as inventory's own facts, visible to that holder and
/// caused by the request that decided them.
#[test]
fn production_and_consumption_through_the_constructors_change_one_holder_by_the_count() {
    let mut town = begun_with_workshop();
    let (alice, kiosk) = (town.id("alice"), town.id("kiosk"));
    let (coffee, tea) = (town.item("coffee"), town.item("tea"));
    let before = (town.holdings("alice"), town.holdings("bob"));

    let made = change(
        &mut town.world,
        alice,
        &Change {
            holder: kiosk,
            item: coffee,
            count: 3,
            way: Way::Make,
            forged: false,
        },
        1,
    )
    .expect("dispatch answers");
    assert!(
        matches!(made.result(), ActionResult::Accepted { .. }),
        "{:?}",
        made.result()
    );
    assert_eq!(
        town.holdings("kiosk"),
        owned(&[("apple", 9), ("coffee", 23)]),
        "three more coffee, and nothing else changed"
    );
    assert_eq!(
        (town.holdings("alice"), town.holdings("bob")),
        before,
        "nobody else's holdings moved"
    );

    let used = change(
        &mut town.world,
        alice,
        &Change {
            holder: alice,
            item: tea,
            count: 1,
            way: Way::UseUp,
            forged: false,
        },
        2,
    )
    .expect("dispatch answers");
    assert!(matches!(used.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        town.holdings("alice"),
        owned(&[("coffee", 2)]),
        "the last tea is gone, and no zero entry is left"
    );

    for (done, event_type, holder) in [
        (&made, ItemsProduced::EVENT_TYPE, kiosk),
        (&used, ItemsConsumed::EVENT_TYPE, alice),
    ] {
        let facts: Vec<&EventEnvelope> = done
            .events()
            .iter()
            .filter(|fact| *fact.event_type() == event_type)
            .collect();
        assert_eq!(facts.len(), 1, "one {event_type} fact");
        assert_eq!(*facts[0].visibility(), Visibility::Participants);
        assert_eq!(facts[0].participants(), &[holder], "the holder alone");
        assert!(
            matches!(facts[0].caused_by(), Causation::Action(_)),
            "caused by the request that decided it"
        );
    }
}

/// E-3: `items-produced` and `items-consumed` stated past the constructors are refused by the owner
/// at reduction and write nothing — an undeclared kind, a count of zero, a non-holder, production past
/// a person's six, consumption of more than is held. The constructors refuse the same; and the same
/// forged path with a valid value is reduced, the positive control that makes the refusal the check.
#[test]
fn production_and_consumption_stated_past_the_constructors_are_refused_by_the_owner() {
    let mut town = begun_with_workshop();
    let (alice, bob, cafe, kiosk) = (
        town.id("alice"),
        town.id("bob"),
        town.id("cafe"),
        town.id("kiosk"),
    );
    let (apple, coffee, lantern, tea) = (
        town.item("apple"),
        town.item("coffee"),
        town.item("lantern"),
        town.item("tea"),
    );
    let forged = |holder, item, count, way| Change {
        holder,
        item,
        count,
        way,
        forged: true,
    };
    let cases = [
        (
            "produced: an undeclared kind",
            forged(kiosk, lantern, 1, Way::Make),
            Rejection::PreconditionFailed,
        ),
        (
            "produced: a count of zero",
            forged(kiosk, coffee, 0, Way::Make),
            Rejection::PreconditionFailed,
        ),
        (
            "produced: into a place",
            forged(cafe, coffee, 1, Way::Make),
            Rejection::PreconditionFailed,
        ),
        (
            "produced: past a person's six (bob holds one)",
            forged(bob, apple, 6, Way::Make),
            Rejection::TargetUnavailable,
        ),
        (
            "consumed: an undeclared kind",
            forged(alice, lantern, 1, Way::UseUp),
            Rejection::PreconditionFailed,
        ),
        (
            "consumed: a count of zero",
            forged(alice, coffee, 0, Way::UseUp),
            Rejection::PreconditionFailed,
        ),
        (
            "consumed: from a place",
            forged(cafe, coffee, 1, Way::UseUp),
            Rejection::PreconditionFailed,
        ),
        (
            "consumed: more than is held (alice holds one tea)",
            forged(alice, tea, 2, Way::UseUp),
            Rejection::PreconditionFailed,
        ),
    ];
    for (id, (what, request, expected)) in cases.into_iter().enumerate() {
        let before = state(&town.world);
        let (event_type, reason) = refusal(change(
            &mut town.world,
            alice,
            &request,
            u64::try_from(id).expect("small") + 1,
        ));
        let expected_type = match request.way {
            Way::Make => "items-produced",
            Way::UseUp => "items-consumed",
        };
        assert_eq!(event_type, expected_type, "{what}");
        assert_eq!(reason, expected, "{what}");
        assert_eq!(state(&town.world), before, "{what}: nothing written");

        let read = town.world.read();
        let constructor = match request.way {
            Way::Make => produce(&read, request.holder, request.item, request.count),
            Way::UseUp => consume(&read, request.holder, request.item, request.count),
        };
        assert_eq!(
            constructor.expect_err("the constructor refuses it too"),
            expected,
            "{what}"
        );
    }

    let made = change(
        &mut town.world,
        alice,
        &forged(bob, apple, 5, Way::Make),
        20,
    )
    .expect("a valid forged production reduces");
    assert!(matches!(made.result(), ActionResult::Accepted { .. }));
    let used = change(
        &mut town.world,
        alice,
        &forged(alice, coffee, 2, Way::UseUp),
        21,
    )
    .expect("a valid forged consumption reduces");
    assert!(matches!(used.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        (town.holdings("bob"), town.holdings("alice")),
        (owned(&[("apple", 6)]), owned(&[("tea", 1)])),
        "positive control: the owner reduced facts another system stated, up to six exactly"
    );
}
