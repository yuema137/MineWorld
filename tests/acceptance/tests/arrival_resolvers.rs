//! Presence resolves an arrival before it records it (step-11 §16, SC-2 … SC-5, SC-8;
//! `docs/DECISIONS.md` `ARC-39`), through the real `World::dispatch`, with the synthetic resolvers of
//! `resolvers/mod.rs`.
//!
//! This process registers `[grid, rogue, fences]` — every test registers first, a no-op after the
//! first — and each world installs only the packs its test needs; a registered resolver whose pack a
//! world does not install finds none of its state there and changes nothing. This file never composes
//! through `worldpack`, which would register the build's own, different list (SD-R12).
//!
//! ```text
//! RS-3   resolve before record: the stop, the displaced person, stopped-short, causation (SC-2)
//! RS-4   presence still decides: each breach refused by name, nothing written; one kept (SC-3)
//! RS-5   order: ascending SystemId, whatever the registration order (SC-4)
//! RS-6   placement: arrival refuses what arrivals records (SC-5)
//! RS-7   inert where absent, in a registered process (I-13)
//! RS-12  a system that states arrived and subscribes to it (SC-8)
//! RS-14  runtime disable is not honoured (QB-17), shown
//! ```

mod resolvers;

use mineworld_contracts::{
    ActionResult, Causation, EntityType, Event, PersonId, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::{KernelError, SystemIdentity};
use mineworld_movement::MovementSystem;
use mineworld_presence::{Arrived, PresenceSystem, StoppedShort, arrival, registered_resolvers};
use resolvers::{
    Cast, FENCES, Fact, Pack, Place, Plan, Town, Violation, Where, facts, owned_state,
    register_all, run_inert_script,
};

/// RS-3's layout: a fence at x = 5 000 across the yard; alice west of it, bob and carol east.
fn fenced_yard() -> Plan {
    let mut plan = Plan::new(
        &[Pack::Movement, Pack::Fences],
        &[
            ("alice", Some((Where::Yard, 4_000, 2_000))),
            ("bob", Some((Where::Yard, 5_100, 2_100))),
            ("carol", Some((Where::Yard, 5_400, 2_000))),
        ],
    );
    plan.fence = Some(5_000);
    plan
}

/// RS-3 (SC-2). The stopped arrival is the recorded one, the person in the way is displaced as a true
/// fact caused by alice's request, and the log never says alice reached where she asked.
#[test]
fn an_arrival_is_resolved_before_it_is_recorded() {
    register_all();
    let mut town = Town::begin(&fenced_yard());
    let (alice, bob) = (town.person("alice"), town.person("bob"));
    let carol_before = town.where_is("carol");

    let (id, answer) = town.walk("alice", town.yard(5_500, 2_000));
    let dispatched = answer.expect("the move is answered");
    assert!(
        matches!(dispatched.result(), ActionResult::Accepted { .. }),
        "{:?}",
        dispatched.result()
    );
    assert_eq!(
        facts(&dispatched),
        [
            Fact::Arrived(alice, town.yard(4_990, 2_000)),
            Fact::Arrived(bob, town.yard(5_200, 2_100)),
            Fact::StoppedShort {
                person: alice,
                wanted: town.yard(5_500, 2_000),
                reached: town.yard(4_990, 2_000),
                by: None,
            },
        ],
        "alice stops 10 mm short of the fence; bob, 110 and 100 mm from the stop, makes room; \
         carol, 410 mm away, does not"
    );
    for event in dispatched.events() {
        assert_eq!(*event.caused_by(), Causation::Action(id), "AC-9");
        assert_eq!(event.provenance().emitted_by(), &MovementSystem::ID);
        assert_eq!(event.provenance().controller_decision(), Some(id));
    }
    assert_eq!(
        dispatched.events()[2].event_type(),
        &StoppedShort::EVENT_TYPE
    );
    assert_eq!(StoppedShort::OWNER, PresenceSystem::ID);

    assert_eq!(town.where_is("alice"), Some(town.yard(4_990, 2_000)));
    assert_eq!(town.where_is("bob"), Some(town.yard(5_200, 2_100)));
    assert_eq!(town.where_is("carol"), carol_before);

    let untrue = Fact::Arrived(alice, town.yard(5_500, 2_000));
    assert!(
        town.log
            .iter()
            .all(|event| resolvers::fact(event) != untrue),
        "the log holds only true arrivals"
    );
}

/// The refusal a rogue answer must produce, naming the rogue and the rule.
fn refused_by_rogue(answer: Result<mineworld_kernel::Dispatched, KernelError>, rule: &str) {
    match answer {
        Err(KernelError::FactRefusedByOwner {
            system,
            event_type,
            reason: Rejection::System { code, detail },
        }) => {
            assert_eq!(system, PresenceSystem::ID);
            assert_eq!(event_type, Arrived::EVENT_TYPE);
            assert_eq!(code, RejectionCode::from_static("resolution-refused"));
            assert_eq!(
                detail.as_deref(),
                Some(format!("test-rogue: {rule}").as_str())
            );
        }
        Err(other) => panic!("{rule}: refused for another reason: {other:?}"),
        Ok(dispatched) => panic!("{rule}: recorded {:?}", facts(&dispatched)),
    }
}

/// RS-4's layout: the rogue in the yard; alice, bob and carol in the yard, erin in the lane.
fn rogue_yard(violation: fn(&Cast) -> Violation) -> Plan {
    let mut plan = Plan::new(
        &[Pack::Movement, Pack::Rogue],
        &[
            ("alice", Some((Where::Yard, 4_000, 2_000))),
            ("bob", Some((Where::Yard, 5_100, 2_100))),
            ("carol", Some((Where::Yard, 5_400, 2_000))),
            ("erin", Some((Where::Lane, 1_000, 2_000))),
        ],
    );
    plan.rogue = Some(violation);
    plan
}

/// One of RS-4's rows: the rule presence must name, and the rogue's behaviour that breaks it.
type Row = (&'static str, fn(&Cast) -> Violation);

/// RS-4 (SC-3). Every breach of presence's rules is refused as the owner's refusal, naming the
/// resolver and the rule, and writes nothing; an answer that keeps every rule is recorded.
#[test]
fn presence_refuses_a_resolution_that_breaks_its_rules_and_names_the_resolver() {
    register_all();
    let rows: [Row; 14] = [
        ("lengthened the arrival", |_| Violation::Lengthen),
        ("ended the arrival in another place", |cast| {
            Violation::OtherPlace(cast.lane)
        }),
        ("turned the person arriving", |_| Violation::Turn),
        ("invented or dropped a local position", |_| {
            Violation::DropLocal
        }),
        ("displaced the person arriving", |_| {
            Violation::DisplaceWalker
        }),
        ("displaced one person twice", |cast| {
            Violation::DisplaceTwice(cast.person("bob"))
        }),
        ("displaced somebody who is not in the place", |cast| {
            Violation::DisplaceAbsent(cast.person("erin"))
        }),
        ("displaced something that is not a living person", |cast| {
            Violation::DisplaceNonPerson(
                PersonId::new(cast.yard.entity_id(), EntityType::Person)
                    .expect("the id trusts its claim"),
            )
        }),
        ("displaced something that is not a living person", |cast| {
            Violation::DisplaceDestroyed(cast.person("carol"))
        }),
        ("displaced somebody into another place", |cast| {
            Violation::DisplaceElsewhere(cast.person("bob"), cast.lane)
        }),
        ("turned a displaced person", |cast| {
            Violation::TurnDisplaced(cast.person("bob"))
        }),
        (
            "invented or dropped a displaced person's local position",
            |cast| Violation::DropDisplacedLocal(cast.person("bob")),
        ),
        ("named a stopper that is not in this world", |_| {
            Violation::UnknownStopper
        }),
        ("named what stopped a person who was not stopped", |cast| {
            Violation::StopperWithoutStop(cast.person("bob").entity_id())
        }),
    ];
    for (index, (rule, violation)) in rows.into_iter().enumerate() {
        let mut town = Town::begin(&rogue_yard(violation));
        if index == 8 {
            // The destroyed-person row: carol is destroyed before alice walks.
            let carol = town.person("carol").entity_id();
            town.world
                .destroy_entity(carol)
                .expect("carol is destroyed");
        }
        let before = owned_state(&town.world);
        let recorded = town.log.len();
        let (_, answer) = town.walk("alice", town.yard(5_500, 2_000));
        refused_by_rogue(answer, rule);
        assert_eq!(town.log.len(), recorded, "{rule}: no fact is recorded");
        assert_eq!(
            owned_state(&town.world),
            before,
            "{rule}: no component row and no edge changed"
        );
    }

    // The positive control: a resolution that keeps every rule is recorded as resolved.
    let mut town = Town::begin(&rogue_yard(|cast| Violation::Obey {
        stopper: cast.person("bob").entity_id(),
        other: cast.person("carol"),
    }));
    let (alice, bob, carol) = (
        town.person("alice"),
        town.person("bob"),
        town.person("carol"),
    );
    let (_, answer) = town.walk("alice", town.yard(5_500, 2_000));
    let dispatched = answer.expect("a rule-keeping resolution is recorded");
    assert_eq!(
        facts(&dispatched),
        [
            Fact::Arrived(alice, town.yard(4_500, 2_000)),
            Fact::Arrived(carol, town.yard(5_500, 2_000)),
            Fact::StoppedShort {
                person: alice,
                wanted: town.yard(5_500, 2_000),
                reached: town.yard(4_500, 2_000),
                by: Some(bob.entity_id()),
            },
        ]
    );
}

/// RS-5 (SC-4). Registered as [grid, rogue, fences], asked in ascending id: fences (5 500 → 4 990),
/// then grid (4 990 → 4 000). Grid first would give 5 000, then fences 4 990.
#[test]
fn resolvers_are_asked_in_ascending_system_id() {
    register_all();
    let mut plan = Plan::new(
        &[Pack::Movement, Pack::Grid, Pack::Fences],
        &[("alice", Some((Where::Yard, 3_600, 2_000)))],
    );
    plan.fence = Some(5_000);
    plan.grid = Some(1_000);
    let mut town = Town::begin(&plan);
    let alice = town.person("alice");
    let (_, answer) = town.walk("alice", town.yard(5_500, 2_000));
    assert_eq!(
        facts(&answer.expect("answered")),
        [
            Fact::Arrived(alice, town.yard(4_000, 2_000)),
            Fact::StoppedShort {
                person: alice,
                wanted: town.yard(5_500, 2_000),
                reached: town.yard(4_000, 2_000),
                by: None,
            },
        ],
        "fences, then grid"
    );
    assert_eq!(
        registered_resolvers(),
        Some(vec![
            SystemId::from_static("test-fences"),
            SystemId::from_static("test-grid"),
            SystemId::from_static("test-rogue"),
        ]),
        "the order they are asked in is the ids' order, not the registration's"
    );
}

/// RS-6 (SC-5). A placement the fence would change is refused by `arrival`, naming the fence, and
/// recorded resolved by a system that places through `arrivals`; a placement it does not change is
/// exactly `Arrived::new`'s bytes.
#[test]
fn arrival_refuses_a_placement_a_resolver_would_change_and_arrivals_records_it() {
    register_all();
    let mut plan = Plan::new(
        &[Pack::Movement, Pack::Fences, Pack::Placer],
        &[
            ("alice", Some((Where::Yard, 4_000, 2_000))),
            ("bob", Some((Where::Yard, 5_100, 2_100))),
            ("dan", None),
        ],
    );
    plan.fence = Some(5_000);
    let mut town = Town::begin(&plan);
    let dan = town.person("dan");
    assert_eq!(town.where_is("dan"), None, "dan has no presence");

    let refused = arrival(&town.world.read(), dan, town.yard(5_500, 1_000));
    assert_eq!(
        refused,
        Err(Rejection::System {
            code: RejectionCode::from_static("resolution-refused"),
            detail: Some(
                "test-fences: would change a placement; a system that moves people states it \
                 through arrivals"
                    .to_owned()
            ),
        })
    );

    let (_, answer) = town.request(
        dan,
        &Place {
            person: dan,
            to: town.yard(5_500, 1_000),
        },
    );
    let dispatched = answer.expect("answered");
    assert!(matches!(dispatched.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        facts(&dispatched),
        [
            Fact::Arrived(dan, town.yard(4_990, 1_000)),
            Fact::StoppedShort {
                person: dan,
                wanted: town.yard(5_500, 1_000),
                reached: town.yard(4_990, 1_000),
                by: None,
            },
        ]
    );

    let placed = arrival(&town.world.read(), dan, town.yard(4_000, 1_000))
        .expect("a placement the fence does not change");
    assert_eq!(
        placed.record().payload(),
        &serde_json::to_vec(&Arrived::new(dan, town.yard(4_000, 1_000))).expect("encodes"),
        "exactly Arrived::new's encoding"
    );
}

/// RS-7, the registered half (I-13). A world with no resolver's pack, and a world with fences
/// installed but no fence anywhere, each run the twelve-move script exactly as if no resolver existed.
#[test]
fn a_registered_resolver_is_inert_where_its_state_is_absent() {
    register_all();
    for packs in [&[Pack::Movement][..], &[Pack::Movement, Pack::Fences][..]] {
        let start = resolvers::SCRIPT_START;
        let mut town = Town::begin(&Plan::new(packs, &[("alice", Some(start))]));
        run_inert_script(&mut town);
    }
}

/// RS-12 (SC-8, the one kernel-adjacent fact S15 relies on). A system may state presence's `arrived`
/// and subscribe to it, and it hears the arrival it stated; the cascade ends because echo answers its
/// own follower's arrival with nothing but `heard`.
#[test]
fn a_system_may_state_arrived_and_hear_it() {
    register_all();
    let mut plan = Plan::new(
        &[Pack::Movement],
        &[
            ("alice", Some((Where::Yard, 4_000, 2_000))),
            ("bob", Some((Where::Yard, 1_000, 1_000))),
        ],
    );
    plan.echo = Some(("alice", "bob"));
    let mut town = Town::begin(&plan);
    let (alice, bob) = (town.person("alice"), town.person("bob"));
    let (id, answer) = town.walk("alice", town.yard(5_000, 2_000));
    let dispatched = answer.expect("answered");
    let events = dispatched.events();
    assert_eq!(
        facts(&dispatched),
        [
            Fact::Arrived(alice, town.yard(5_000, 2_000)),
            Fact::Heard(alice),
            Fact::Arrived(bob, town.yard(5_600, 2_000)),
            Fact::Heard(bob),
        ],
        "and nothing more"
    );
    assert_eq!(*events[0].caused_by(), Causation::Action(id));
    assert_eq!(*events[1].caused_by(), Causation::Event(events[0].id()));
    assert_eq!(*events[2].caused_by(), Causation::Event(events[0].id()));
    assert_eq!(
        *events[3].caused_by(),
        Causation::Event(events[2].id()),
        "echo heard the arrival it stated"
    );
    assert_eq!(
        events[2].provenance().emitted_by(),
        &SystemId::from_static("test-echo")
    );
}

/// RS-14 (QB-15 bound 3, QB-17). Disabling fences at runtime does not stop its resolver: the catalog
/// is per process and a `WorldRead` cannot see enabled state, so the fence still stops alice.
#[test]
fn runtime_disable_of_a_resolver_pack_is_not_honoured() {
    register_all();
    let mut town = Town::begin(&fenced_yard());
    town.world.disable(&FENCES).expect("fences is disabled");
    let alice = town.person("alice");
    let (_, answer) = town.walk("alice", town.yard(5_500, 2_000));
    let recorded = facts(&answer.expect("answered"));
    assert_eq!(
        recorded[0],
        Fact::Arrived(alice, town.yard(4_990, 2_000)),
        "still resolved by the disabled pack's resolver: {recorded:?}"
    );
}
