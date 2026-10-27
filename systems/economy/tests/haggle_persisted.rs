//! A proposal survives a restart (`ARC-118`, `ARC-25`; S31-h AC-h-8). W1 is created into a real SQLite
//! save; Bob proposes; the world is dropped and a freshly composed world resumed from the file. Grace
//! accepts five minutes later and the sale settles exactly as in a world that never stopped; resumed
//! after the proposal's fifteen minutes, the same acceptance is `no-such-proposal`, because world time
//! is persisted and expiry is read from it.

mod support;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, EventEnvelope, Rejection,
};
use mineworld_economy::{AcceptProposal, ProposalId, ProposePrice};
use mineworld_kernel::World;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, verify,
};
use support::{GENESIS, SHOP, Setup, Town, t};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0031_0d08;

/// A save directory of this test's own, removed when the test ends (scratch, DEP-29).
struct Scratch(mineworld_test_support::Scratch);

impl Scratch {
    fn new(name: &str) -> Self {
        Self(mineworld_test_support::scratch!(empty name))
    }
}

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(&scratch.0, Durability::ProcessCrash).expect("opens"))
}

fn w1() -> Setup {
    Setup::store(SHOP)
        .with_economy(
            "grace",
            "{ wallet: 1000, asks: { bread: 300, pen: 150, apple: 120 } }",
        )
        .carrying("grace", "{ bread: 2, pen: 1, apple: 3 }")
        .with_economy("bob", "{ wallet: 1000 }")
}

/// A save of W1, and a town that names its entities (its own world is unused).
fn create(scratch: &Scratch) -> (PersistentWorld, Town) {
    let (world, keys, facts) = Town::assemble(&w1(), |_| {});
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "economy-haggle-test".to_owned(),
            at: GENESIS,
            facts,
        },
    )
    .expect("a save is created");
    let names = Town {
        world: World::new(),
        keys,
        genesis: Vec::new(),
        log: Vec::new(),
    };
    (persisted, names)
}

fn intent<A: Action + serde::Serialize>(
    town: &Town,
    actor: &str,
    target: Option<&str>,
    action: &A,
    id: u64,
) -> ActionIntent {
    let record = ActionRecord::new::<A>(serde_json::to_vec(action).expect("encodes"));
    let at = t(i64::try_from(id).expect("small"));
    let intent = ActionIntent::new(ActionId::from_raw(id), town.id(actor), record, at);
    match target {
        Some(target) => intent.with_target(town.id(target)),
        None => intent,
    }
}

/// Bob proposes 250 for bread to Grace, as request 1 at second 1.
fn propose(persisted: &mut PersistentWorld, town: &Town) {
    let action = ProposePrice::new(town.item("bread"), 250);
    let done = persisted
        .dispatch(&intent(town, "bob", Some("grace"), &action, 1), t(1))
        .expect("dispatches");
    assert!(matches!(done.result(), ActionResult::Accepted { .. }));
}

/// Grace accepts proposal 1, as request `id` at second `id`.
fn accept(
    persisted: &mut PersistentWorld,
    town: &Town,
    id: u64,
) -> (ActionResult, Vec<EventEnvelope>) {
    let action = AcceptProposal::new(ProposalId::from_raw(1));
    let at = t(i64::try_from(id).expect("small"));
    let done = persisted
        .dispatch(&intent(town, "grace", Some("bob"), &action, id), at)
        .expect("dispatches");
    (done.result().clone(), done.events().to_vec())
}

fn state(world: &World) -> Vec<u8> {
    let snapshot = world.snapshot().expect("a snapshot");
    serde_json::to_vec(&(&snapshot.components, &snapshot.relations)).expect("encodes")
}

#[test]
fn a_proposal_open_at_a_restart_is_accepted_after_it_and_expires_on_world_time() {
    // The world that never stops.
    let straight = Scratch::new("straight");
    let (mut uninterrupted, town) = create(&straight);
    propose(&mut uninterrupted, &town);
    let (result, straight_facts) = accept(&mut uninterrupted, &town, 301);
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );

    // Stopped after the proposal, resumed, accepted five minutes on.
    let stopped = Scratch::new("stopped");
    let (mut first, _) = create(&stopped);
    propose(&mut first, &town);
    let saved = state(first.world());
    drop(first);
    let compose = || Town::compose(&w1());
    let (mut resumed, _) = PersistentWorld::resume(open(&stopped), compose()).expect("resumes");
    assert_eq!(
        state(resumed.world()),
        saved,
        "the open proposal is resumed"
    );
    let (result, resumed_facts) = accept(&mut resumed, &town, 301);
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    let kinds: Vec<&str> = resumed_facts
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect();
    assert_eq!(
        kinds,
        [
            "proposal-accepted",
            "sold",
            "money-transferred",
            "items-transferred"
        ]
    );
    assert_eq!(
        state(resumed.world()),
        state(uninterrupted.world()),
        "the resumed world ends where the uninterrupted one does"
    );
    assert_eq!(
        serde_json::to_vec(&resumed_facts).expect("encodes"),
        serde_json::to_vec(&straight_facts).expect("encodes"),
        "and its facts are byte-identical"
    );
    drop(resumed);
    verify(open(&stopped).as_ref(), compose()).expect("the history verifies");

    // Stopped after the proposal, resumed after its fifteen minutes.
    let late = Scratch::new("late");
    let (mut first, _) = create(&late);
    propose(&mut first, &town);
    drop(first);
    let (mut resumed, _) = PersistentWorld::resume(open(&late), compose()).expect("resumes");
    let before = state(resumed.world());
    let (result, facts) = accept(&mut resumed, &town, 960);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::System {
            code: mineworld_economy::code::NO_SUCH_PROPOSAL,
            detail: None
        })
    );
    assert!(facts.is_empty());
    assert_eq!(state(resumed.world()), before);
}
