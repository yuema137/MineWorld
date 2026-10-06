//! IC-1: movement survives a restart (step-07 §5, MD-12).
//!
//! A two-place town is created into a real SQLite save, walked — a stride, a refused stride, a walk
//! to the doorway and a crossing into the street — and dropped. A freshly composed world is resumed
//! from the file: the positions, the occupancy edges and the passages are those that were saved, the
//! tail of the journal was re-executed to get there (located, not assumed), the whole history
//! verifies from genesis, and the next move continues at the next event identity.
//!
//! S5 proved the restart mechanics on a test ledger; this proves *movement* rides them, including
//! the fact one system states in another's vocabulary (`ARC-26`), which replay must reproduce byte
//! for byte like any other.

mod support;

use std::path::PathBuf;

use mineworld_contracts::{ActionIntent, ActionRecord, ActionResult, Rejection, WorldTime};
use mineworld_kernel::World;
use mineworld_movement::{Move, Passages};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision, verify,
};
use support::{Layout, Movement, NOW, Town, at, compose, encode};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0000_0608;

/// A directory of this test's own, emptied when made and removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mineworld-movement-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(&scratch.0, Durability::ProcessCrash).expect("opens"))
}

/// What movement and presence hold: every component row (Presence and Passages among them) and every
/// edge, as bytes.
fn spatial_state(world: &World) -> Vec<u8> {
    let snapshot = world.snapshot().expect("a snapshot");
    encode(&(&snapshot.components, &snapshot.relations))
}

fn t(offset: i64) -> WorldTime {
    WorldTime::from_seconds(NOW.seconds() + offset)
}

#[test]
fn a_walk_through_the_doorway_is_saved_and_a_resumed_world_holds_and_continues_it() {
    let scratch = Scratch::new("walk");
    let (town, facts) = Town::assemble(Movement::Enabled, Layout::Joined, |town| {
        at(town.cafe, 2_600, 2_000)
    });
    let (cafe, street, visitor) = (town.cafe, town.street, town.visitor);
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    let (mut persisted, began) = PersistentWorld::create(
        Box::new(backend),
        town.world,
        Creation {
            instance: INSTANCE,
            pack: "movement-test".to_owned(),
            at: NOW,
            facts,
        },
    )
    .expect("a save is created");
    println!("genesis: {} facts", began.len());
    assert_eq!(began.len(), 3, "one passage and two placements");

    let walk = [
        (at(cafe, 3_600, 2_000), true),
        (at(cafe, 6_000, 2_000), false), // 2 400 mm from (3 600, 2 000): refused
        (at(cafe, 4_600, 2_000), true),
        (at(street, 1_000, 2_000), true),
    ];
    let mut answers = Vec::new();
    for (index, (to, expected)) in walk.iter().enumerate() {
        let step = i64::try_from(index).expect("small");
        let intent = ActionIntent::new(
            mineworld_contracts::ActionId::from_raw(u64::try_from(index).expect("small") + 1),
            visitor,
            ActionRecord::new::<Move>(encode(&Move::new(*to))),
            t(step + 1),
        );
        let dispatched = persisted
            .dispatch(&intent, t(step + 1))
            .expect("dispatches");
        let result = dispatched.result().clone();
        if *expected {
            assert!(
                matches!(result, ActionResult::Accepted { .. }),
                "{to:?}: {result:?}"
            );
        } else {
            assert_eq!(result, ActionResult::Rejected(Rejection::TooFarAway));
        }
        answers.push(result);
    }
    println!("answers: {answers:?}");

    let head = persisted.revision();
    let saved = spatial_state(persisted.world());
    let saved_passages = persisted
        .world()
        .components()
        .get::<Passages>(cafe.entity_id())
        .cloned();
    let saved_facts = persisted.recent_facts(16).expect("the log reads");
    drop(persisted);

    // Located first: genesis is revision 1, and each of the four requests is a revision whatever its
    // answer (`ARC-25`); the default snapshot interval (64) means only genesis is snapshotted.
    println!("head {head:?}; {} facts in the log", saved_facts.len());
    assert_eq!(head.raw(), 1 + 4);
    assert_eq!(
        saved_facts.len(),
        3 + 3 + 1,
        "genesis, then three accepted moves, and presence's occupancy change for the crossing; \
         the refusal has none"
    );
    let kinds: Vec<&str> = saved_facts
        .iter()
        .map(|fact| fact.event_type().as_str())
        .collect();
    println!("facts: {kinds:?}");
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == "person-entered-place")
            .count(),
        1,
        "the crossing, and only the crossing, changed occupancy"
    );

    let (composed, _) = compose(Movement::Enabled);
    let (mut resumed, how) = PersistentWorld::resume(open(&scratch), composed).expect("resumes");
    println!(
        "resumed: snapshot {:?}, replayed {}, facts {}",
        how.snapshot, how.replayed, how.facts
    );
    assert_eq!(how.snapshot, WorldRevision::GENESIS);
    assert_eq!(
        how.replayed, 4,
        "the whole walk was re-executed, the refusal included"
    );
    assert_eq!(how.facts, 4, "and reproduced its four facts byte for byte");
    assert_eq!(
        spatial_state(resumed.world()),
        saved,
        "Presence, present-in and Passages"
    );
    assert!(saved_passages.is_some_and(|passages| passages.to(street).is_some()));

    let verified = verify(open(&scratch).as_ref(), compose(Movement::Enabled).0)
        .expect("the history verifies from genesis");
    assert_eq!(verified.revisions, 5);

    let back_inside = at(cafe, 4_600, 2_000);
    let intent = ActionIntent::new(
        mineworld_contracts::ActionId::from_raw(5),
        visitor,
        ActionRecord::new::<Move>(encode(&Move::new(back_inside))),
        t(10),
    );
    let dispatched = resumed.dispatch(&intent, t(10)).expect("dispatches");
    let next = dispatched.events()[0].id().raw();
    println!("the next move's fact: EventId({next})");
    assert_eq!(next, 3 + 4 + 1, "identity continues at the next EventId");
    assert_eq!(
        resumed
            .world()
            .components()
            .get::<mineworld_presence::Presence>(visitor)
            .map(mineworld_presence::Presence::location),
        Some(back_inside),
        "and back through the doorway"
    );
}
