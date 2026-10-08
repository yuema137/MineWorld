//! An activity survives a restart, and its owner still ends it (`step-09-social.md` C2, F-8).
//!
//! The first real use of a Process across a process restart: an activity is started in a real SQLite
//! save, the world is dropped mid-activity, a freshly composed world resumes from the file, and
//! advancing past the expected end wakes the owner, which states `group-activity-ended` — caused by
//! that process, naming both founders. Located first: the process is in the resumed world, with the
//! same expected end, before anything is advanced.

mod support;

use mineworld_contracts::{ActionId, ActionIntent, ActionResult, Causation, WorldTime};
use mineworld_group_activity::{AcceptInvitation, Invite, Participation};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, verify,
};
use support::{Cafe, GENESIS, compose, kind, record, t, types};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0010_0b02;

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

fn dispatch(
    world: &mut PersistentWorld,
    id: u64,
    intent_at: WorldTime,
    actor: mineworld_contracts::EntityId,
    target: mineworld_contracts::EntityId,
    payload: mineworld_contracts::ActionRecord,
) -> ActionResult {
    let intent =
        ActionIntent::new(ActionId::from_raw(id), actor, payload, intent_at).with_target(target);
    world
        .dispatch(&intent, intent_at)
        .expect("dispatches")
        .result()
        .clone()
}

#[test]
fn an_activity_started_before_a_restart_is_ended_by_its_owner_after_it() {
    let scratch = Scratch::new("activity");
    let (world, providers) = compose();
    let (cafe, facts) = Cafe::assemble(world, providers);
    let (alice, bob) = (cafe.alice, cafe.bob);
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    let (mut persisted, _) = PersistentWorld::create(
        Box::new(backend),
        cafe.world,
        Creation {
            instance: INSTANCE,
            pack: "group-activity-test".to_owned(),
            at: GENESIS,
            facts,
        },
    )
    .expect("a save is created");

    let invited = dispatch(
        &mut persisted,
        1,
        t(10),
        alice,
        bob,
        record(&Invite::new(kind("coffee"))),
    );
    assert!(
        matches!(invited, ActionResult::Accepted { .. }),
        "{invited:?}"
    );
    let accepted = dispatch(
        &mut persisted,
        2,
        t(20),
        bob,
        alice,
        record(&AcceptInvitation {}),
    );
    assert!(
        matches!(accepted, ActionResult::Accepted { .. }),
        "{accepted:?}"
    );
    let activity = persisted
        .world()
        .read()
        .component::<Participation>(alice)
        .expect("alice is part of it")
        .activity();
    let expected_end = persisted
        .world()
        .read()
        .process(activity)
        .and_then(mineworld_kernel::Process::expected_end);
    assert_eq!(expected_end, Some(t(20 + 3_600)));
    drop(persisted);

    // Resumed into a freshly composed world: the process and its pending wake come from the file.
    let (composed, _) = compose();
    let (mut resumed, how) = PersistentWorld::resume(open(&scratch), composed).expect("resumes");
    println!(
        "resumed: snapshot {:?}, replayed {}",
        how.snapshot, how.replayed
    );
    let found = resumed.world().read().process(activity).cloned();
    assert_eq!(
        found
            .as_ref()
            .and_then(mineworld_kernel::Process::expected_end),
        Some(t(20 + 3_600)),
        "the activity is running in the resumed world, with its end: {found:?}"
    );

    let early = resumed.advance_to(t(20 + 3_599)).expect("advances");
    assert!(early.events().is_empty(), "nothing before the end");
    let due = resumed.advance_to(t(20 + 3_600)).expect("advances");
    assert_eq!(types(due.events()), ["group-activity-ended"]);
    assert_eq!(*due.events()[0].caused_by(), Causation::Process(activity));
    assert!(
        resumed.world().read().process(activity).is_none(),
        "ended by its owner"
    );
    assert!(
        resumed
            .world()
            .read()
            .component::<Participation>(bob)
            .is_none()
    );
    drop(resumed);

    let verified =
        verify(open(&scratch).as_ref(), compose().0).expect("the history verifies from genesis");
    println!("verified {} revisions", verified.revisions);
    assert!(
        verified.revisions >= 4,
        "genesis, two requests, and the advance that fired the wake"
    );
}
