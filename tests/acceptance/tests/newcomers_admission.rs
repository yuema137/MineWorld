mod newcomers;
use mineworld_attributes_model::{
    AttributeValue, AttributesComposed, Newcomers, QualifiedId, Traits,
};
use mineworld_contracts::{EntityKey, Event, WorldTime};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision,
    format, verify,
};
use newcomers::*;

#[test]
fn complete_context_admission_and_real_restart_keep_authored_locked_and_restricted_values() {
    let pack = pack();
    let assembled = assembled(&pack);
    let composed_fact = assembled
        .facts
        .iter()
        .find(|fact| fact.event_type() == &AttributesComposed::EVENT_TYPE)
        .expect("composed");
    let composed_payload: AttributesComposed =
        serde_json::from_slice(composed_fact.record().payload()).expect("composed payload");
    let deferred = &composed_payload
        .newcomers
        .as_ref()
        .expect("runtime context")
        .deferred;
    assert_eq!(deferred.len(), 4);
    assert!(deferred[&EntityKey::new("quin").unwrap()].values.is_empty());
    let scratch = mineworld_test_support::scratch!(empty "newcomer-admission");
    let backend = SqliteBackend::create(scratch.path(), Durability::PowerLoss).expect("SQLite");
    let (mut saved, _) = PersistentWorld::create(
        Box::new(backend),
        assembled.world,
        Creation {
            instance: 0x530a,
            pack: "attributes-newcomers".to_owned(),
            at: WorldTime::EPOCH,
            facts: assembled.facts,
        },
    )
    .expect("genesis");
    for name in PEOPLE {
        assert!(
            saved
                .world()
                .read()
                .component::<Traits>(assembled.ids[&EntityKey::new(name).unwrap()])
                .is_none(),
            "deferred {name}"
        );
    }
    for index in 0..4 {
        let result = saved
            .dispatch(&intent(index), WorldTime::EPOCH)
            .expect("dispatch");
        complete_admissions(result.events()).expect("each arrival must be seeded in this revision");
        let kinds: Vec<_> = result
            .events()
            .iter()
            .map(|event| event.event_type().as_str())
            .collect();
        assert_eq!(
            kinds,
            ["test-arrived", "newcomer-admitted", "traits-stated"]
        );
    }
    let before = saved.world().snapshot().expect("snapshot");
    drop(saved);
    let backend = SqliteBackend::open(scratch.path(), Durability::PowerLoss).expect("reopen");
    let genesis: Vec<_> = backend
        .facts_of(WorldRevision::GENESIS)
        .unwrap()
        .iter()
        .map(|row| format::decode(&row.bytes, "fact").unwrap())
        .collect();
    check_configuration(&pack, &genesis).expect("runtime facts are not genesis drift");
    let verified = verify(&backend, composed()).expect("verify from genesis");
    assert_eq!(verified.revisions, 5);
    let (resumed, report) = PersistentWorld::resume(Box::new(backend), composed()).expect("resume");
    assert!(report.replayed > 0);
    assert_eq!(resumed.world().snapshot().unwrap(), before);
    let read = resumed.world().read();
    let value = |person: &str, attribute: &str| {
        read.component::<Traits>(assembled.ids[&EntityKey::new(person).unwrap()])
            .unwrap()
            .value(&QualifiedId::new(&format!("attributes-newcomers.{attribute}")).unwrap())
            .unwrap()
            .clone()
    };
    assert_eq!(value("nadia", "stature").display(), "1.660");
    assert_eq!(value("omar", "stature").display(), "1.870");
    assert!(
        matches!(value("pia","hair-colour"),AttributeValue::Category(ref slug) if slug.as_str()=="blond")
    );
    assert_eq!(Newcomers::in_world(&read).unwrap().1.arrived.len(), 4);
}
