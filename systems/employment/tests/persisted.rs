//! E-6, the restart half: a world stopped mid-shift, saved and resumed, ends the shift with the same
//! facts as the uninterrupted run (the `schedule`/`inventory` persisted pattern). The open present
//! span, the closed one and the process's next wake all ride the save (`ARC-25`).

mod support;

use std::path::PathBuf;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityType, EventEnvelope, LocalPosition,
    Location, Millimetres, PlaceId,
};
use mineworld_employment::{Employment, ShiftEnded};
use mineworld_movement::Move;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, verify,
};
use support::{GENESIS, HOUR, Town, compose, of, t};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0011_0e06;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "mineworld-employment-{}-{name}",
            std::process::id()
        ));
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

/// One step of day 0: advance to an instant, then perhaps walk somebody somewhere.
type Step = (i64, Option<(&'static str, &'static str)>);

/// Bob leaves at 10:00; the stop falls at 10:30; Carol arrives at 11:00; the shift ends at 12:00.
const BEFORE_STOP: [Step; 2] = [
    (10 * HOUR, Some(("bob", "street"))),
    (10 * HOUR + 1_800, None),
];
const AFTER_STOP: [Step; 2] = [(11 * HOUR, Some(("carol", "cafe"))), (13 * HOUR, None)];

struct Keys(
    std::collections::BTreeMap<mineworld_contracts::EntityKey, mineworld_contracts::EntityId>,
);

impl Keys {
    fn id(&self, name: &str) -> mineworld_contracts::EntityId {
        self.0[&support::key(name)]
    }
}

fn create(scratch: &Scratch) -> (PersistentWorld, Keys) {
    let (world, keys, facts) = Town::assemble(compose(true), true);
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "employment-test".to_owned(),
            at: GENESIS,
            facts,
        },
    )
    .expect("a save is created");
    (persisted, Keys(keys))
}

fn run(
    persisted: &mut PersistentWorld,
    keys: &Keys,
    steps: &[Step],
    first_id: u64,
) -> Vec<EventEnvelope> {
    let mut facts = Vec::new();
    for (index, (at, walk)) in steps.iter().enumerate() {
        facts.extend(
            persisted
                .advance_to(t(*at))
                .expect("advances")
                .into_events(),
        );
        let Some((who, place)) = walk else {
            continue;
        };
        let place = PlaceId::new(keys.id(place), EntityType::Place).expect("a place");
        let to = Location::in_place(place).with_local(LocalPosition::on_ground(
            Millimetres::new(500),
            Millimetres::new(500),
        ));
        let id = first_id + u64::try_from(index).expect("small");
        let intent = ActionIntent::new(
            ActionId::from_raw(id),
            keys.id(who),
            ActionRecord::new::<Move>(serde_json::to_vec(&Move::new(to)).expect("encodes")),
            t(*at),
        );
        let done = persisted.dispatch(&intent, t(*at)).expect("dispatches");
        assert!(matches!(done.result(), ActionResult::Accepted { .. }));
        facts.extend(done.events().iter().cloned());
    }
    facts
}

fn employment(persisted: &PersistentWorld, keys: &Keys, name: &str) -> Employment {
    persisted
        .world()
        .read()
        .component::<Employment>(keys.id(name))
        .cloned()
        .expect("employed")
}

#[test]
fn a_world_stopped_mid_shift_ends_the_shift_as_the_uninterrupted_world_does() {
    let straight = Scratch::new("straight");
    let (mut uninterrupted, keys) = create(&straight);
    run(&mut uninterrupted, &keys, &BEFORE_STOP, 1);
    let straight_after = run(&mut uninterrupted, &keys, &AFTER_STOP, 10);

    let stopped = Scratch::new("stopped");
    let (mut first, keys) = create(&stopped);
    run(&mut first, &keys, &BEFORE_STOP, 1);
    let alice = employment(&first, &keys, "alice");
    let bob = employment(&first, &keys, "bob");
    let shift = alice
        .shift()
        .expect("located: alice is mid-shift at the stop");
    assert_eq!(
        shift.present_since(),
        Some(t(8 * HOUR)),
        "alice there since 08:00"
    );
    assert_eq!(
        bob.shift()
            .expect("bob is on shift")
            .worked_until(t(10 * HOUR + 1_800)),
        7_200,
        "located: bob's closed span, 08:00–10:00, is in the state being saved"
    );
    drop(first);

    let (mut resumed, how) =
        PersistentWorld::resume(open(&stopped), compose(true)).expect("resumes");
    println!(
        "resumed: snapshot {:?}, replayed {}",
        how.snapshot, how.replayed
    );
    assert_eq!(
        employment(&resumed, &keys, "alice"),
        alice,
        "the open span survived"
    );
    assert_eq!(
        employment(&resumed, &keys, "bob"),
        bob,
        "the closed span survived"
    );
    let resumed_after = run(&mut resumed, &keys, &AFTER_STOP, 10);

    let worked: Vec<u64> = of::<ShiftEnded>(&resumed_after)
        .iter()
        .map(|(event, _)| event.worked())
        .collect();
    assert_eq!(
        worked,
        [14_400, 7_200, 3_600, 0],
        "the resumed world ends the shift with every span counted"
    );
    assert_eq!(
        serde_json::to_vec(&resumed_after).expect("encodes"),
        serde_json::to_vec(&straight_after).expect("encodes"),
        "and its facts after the stop are byte-identical to the uninterrupted world's"
    );
    drop(resumed);
    let verified = verify(open(&stopped).as_ref(), compose(true)).expect("the history verifies");
    assert!(verified.revisions >= 2);
}
