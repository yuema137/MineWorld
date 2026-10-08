//! D-8: holdings survive a restart. A town is created into a real SQLite save, items change hands,
//! the world is dropped, and a freshly composed world resumed from the file holds exactly what was
//! saved and continues exactly as a world that never stopped (the `schedule`/`movement` pattern).
//!
//! The property is persistence's (`ARC-25`). What this test owns is that inventory's state rides it:
//! `Holdings` must survive a JSON snapshot, which a map keyed by `ItemId` would not (step-10 F-38).

mod support;

use mineworld_contracts::{ActionResult, EventEnvelope};
use mineworld_kernel::World;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, verify,
};
use support::{GENESIS, Pass, Town, compose, holdings_of, pass_intent, state, t};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0011_0d08;

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

/// The passes, in order: (giver, taker, item, count).
const PASSES: [(&str, &str, &str, u32); 4] = [
    ("alice", "bob", "coffee", 1),
    ("kiosk", "alice", "apple", 2),
    ("bob", "alice", "apple", 1),
    ("alice", "bob", "tea", 1),
];

/// Creates a save of the begun town, and returns it with the town's keys.
fn create(scratch: &Scratch) -> (PersistentWorld, Town) {
    let (town, facts) = Town::assemble();
    let Town { world, keys } = town;
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "inventory-test".to_owned(),
            at: GENESIS,
            facts,
        },
    )
    .expect("a save is created");
    (
        persisted,
        Town {
            world: World::new(),
            keys,
        },
    )
}

fn make(persisted: &mut PersistentWorld, town: &Town, index: usize) -> Vec<EventEnvelope> {
    let (from, to, item, count) = PASSES[index];
    let id = u64::try_from(index).expect("small") + 1;
    let at = t(i64::try_from(id).expect("small"));
    let request = Pass {
        to: town.id(to),
        item: town.item(item),
        count,
        forged: false,
    };
    let done = persisted
        .dispatch(&pass_intent(town.id(from), &request, id, at), at)
        .expect("dispatches");
    assert!(
        matches!(done.result(), ActionResult::Accepted { .. }),
        "pass {index}: {:?}",
        done.result()
    );
    done.events().to_vec()
}

#[test]
fn holdings_saved_mid_trade_are_resumed_and_continue_as_an_uninterrupted_world() {
    // The world that never stops.
    let straight = Scratch::new("straight");
    let (mut uninterrupted, town) = create(&straight);
    let mut last_straight = Vec::new();
    for index in 0..PASSES.len() {
        last_straight = make(&mut uninterrupted, &town, index);
    }

    // The world stopped after two passes.
    let stopped = Scratch::new("stopped");
    let (mut first, town) = create(&stopped);
    for index in 0..2 {
        make(&mut first, &town, index);
    }
    let saved = state(first.world());
    let saved_alice = holdings_of(first.world(), &town.keys, "alice");
    drop(first);

    let (mut resumed, how) = PersistentWorld::resume(open(&stopped), compose()).expect("resumes");
    println!(
        "resumed: snapshot {:?}, replayed {}",
        how.snapshot, how.replayed
    );
    assert_eq!(
        state(resumed.world()),
        saved,
        "the resumed world holds what was saved"
    );
    assert_eq!(
        holdings_of(resumed.world(), &town.keys, "alice"),
        saved_alice
    );
    assert_eq!(
        saved_alice,
        [
            ("apple".to_owned(), 2),
            ("coffee".to_owned(), 1),
            ("tea".to_owned(), 1)
        ],
        "located, not assumed: alice gave a coffee and was given two apples"
    );

    let mut last_resumed = Vec::new();
    for index in 2..PASSES.len() {
        last_resumed = make(&mut resumed, &town, index);
    }
    assert_eq!(
        state(resumed.world()),
        state(uninterrupted.world()),
        "the resumed world ends where the uninterrupted one does"
    );
    assert_eq!(
        serde_json::to_vec(&last_resumed).expect("encodes"),
        serde_json::to_vec(&last_straight).expect("encodes"),
        "and its last facts are byte-identical, identities included"
    );
    drop(resumed);
    let verified = verify(open(&stopped).as_ref(), compose()).expect("the history verifies");
    assert!(verified.revisions >= 2);
}
