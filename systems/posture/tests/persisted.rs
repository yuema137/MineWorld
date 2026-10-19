//! A seated person is still seated after the world is saved, dropped and resumed from SQLite, with
//! the same `since`, and the history verifies from genesis (AC-a-5; AC-S34-4's resume half).

mod support;

use mineworld_contracts::{ActionId, ActionIntent, ActionResult};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, verify,
};
use mineworld_posture::Posture;
use mineworld_worldpack::WorldPack;
use support::{GENESIS, Layout, key, sit, t};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0034_0a05;

#[test]
fn a_person_seated_before_a_restart_is_seated_after_it() {
    // The second world: the pier, with bodies.
    let layout = Layout::pier();
    let (pack_dir, assembled) = layout.assemble("resume-pack");
    let gus = *assembled
        .ids
        .get(&mineworld_contracts::EntityKey::new("gus").expect("a key"))
        .expect("gus");
    let pier = *assembled
        .ids
        .get(&mineworld_contracts::EntityKey::new("pier").expect("a key"))
        .expect("pier");
    let pier = mineworld_contracts::PlaceId::new(pier, mineworld_contracts::EntityType::Place)
        .expect("a place");
    let save = mineworld_test_support::scratch!(empty "resume-save");
    let backend = SqliteBackend::create(&save, Durability::ProcessCrash).expect("creates");
    let (mut persisted, _) = PersistentWorld::create(
        Box::new(backend),
        assembled.world,
        Creation {
            instance: INSTANCE,
            pack: "posture-pier".to_owned(),
            at: GENESIS,
            facts: assembled.facts,
        },
    )
    .expect("a save is created");

    let intent = ActionIntent::new(ActionId::from_raw(1), gus, sit("pier-bench.1"), t(10));
    let result = persisted
        .dispatch(&intent, t(10))
        .expect("dispatches")
        .result()
        .clone();
    assert!(
        matches!(result, ActionResult::Accepted { .. }),
        "{result:?}"
    );
    let seated = Some(Posture::seated(pier, key("pier-bench.1"), t(10)));
    assert_eq!(
        persisted.world().read().component::<Posture>(gus).cloned(),
        seated
    );
    drop(persisted);

    let compose = || {
        WorldPack::read(pack_dir.path())
            .and_then(|pack| pack.compose())
            .expect("composes")
            .world
    };
    let open = || -> Box<dyn PersistenceBackend> {
        Box::new(SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens"))
    };
    let (resumed, _) = PersistentWorld::resume(open(), compose()).expect("resumes");
    assert_eq!(
        resumed.world().read().component::<Posture>(gus).cloned(),
        seated,
        "seated on the same point, since the same instant"
    );
    drop(resumed);

    let verified = verify(open().as_ref(), compose()).expect("the history verifies from genesis");
    assert!(verified.revisions >= 2, "genesis and the sit");
}
