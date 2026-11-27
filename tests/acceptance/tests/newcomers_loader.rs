//! Unmodified public WorldPack loader and restart guard: no test creator or catalogue adapter.
use mineworld_attributes_model::{AttributesComposed, Traits};
use mineworld_contracts::{EntityKey, Event, WorldTime};
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, WorldRevision, format,
};
use mineworld_worldpack::{PackError, WorldPack};
use std::path::Path;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/cli/tests/fixtures/attributes-newcomers"
);

fn copy(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for item in std::fs::read_dir(source).unwrap() {
        let item = item.unwrap();
        let to = target.join(item.file_name());
        if item.path().is_dir() {
            copy(&item.path(), &to);
        } else {
            std::fs::copy(item.path(), to).unwrap();
        }
    }
}

#[test]
fn absent_newcomers_preserves_reviewed_foundation_composed_bytes() {
    // Captured from reviewed6059812e foundation binary, before this implementation,
    // in its unchanged attributes-sampled world. SHA256 and command are in the PR ledger.
    let pack = WorldPack::read(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/cli/tests/fixtures/attributes-sampled"
    )))
    .unwrap();
    let assembled = pack.assemble().unwrap();
    let fact = assembled
        .facts
        .iter()
        .find(|fact| fact.event_type() == &AttributesComposed::EVENT_TYPE)
        .unwrap();
    assert_eq!(
        fact.record().payload().as_slice(),
        include_bytes!("newcomers/attributes-sampled-composed.json")
    );
}

#[test]
fn final_context_is_complete_and_authored_locked_restriction_edits_are_real_disk_drift() {
    let root = mineworld_test_support::scratch!(empty "newcomer-loader-drift");
    let source = root.path().join("attributes-newcomers");
    copy(Path::new(FIXTURE), &source);
    let pack = WorldPack::read(&source).unwrap();
    let assembled = pack.assemble().unwrap();
    let payload: AttributesComposed = serde_json::from_slice(
        assembled
            .facts
            .iter()
            .find(|fact| fact.event_type() == &AttributesComposed::EVENT_TYPE)
            .unwrap()
            .record()
            .payload(),
    )
    .unwrap();
    let deferred = payload.newcomers.unwrap().deferred;
    assert_eq!(
        deferred.keys().map(|key| key.as_str()).collect::<Vec<_>>(),
        ["nadia", "omar", "pia", "quin"]
    );
    assert!(deferred[&EntityKey::new("nadia").unwrap()].values.len() >= 3);
    assert_eq!(deferred[&EntityKey::new("omar").unwrap()].values.len(), 1);
    assert_eq!(
        deferred[&EntityKey::new("pia").unwrap()].restrictions.len(),
        1
    );
    assert!(deferred[&EntityKey::new("quin").unwrap()].values.is_empty());
    let save_path = root.path().join("save");
    let backend = SqliteBackend::create(&save_path, Durability::PowerLoss).unwrap();
    let (saved, _) = PersistentWorld::create(
        Box::new(backend),
        assembled.world,
        Creation {
            instance: 0x530b,
            pack: pack.id().to_string(),
            at: WorldTime::EPOCH,
            facts: assembled.facts,
        },
    )
    .unwrap();
    for name in ["nadia", "omar", "pia", "quin"] {
        assert!(
            saved
                .world()
                .read()
                .component::<Traits>(assembled.ids[&EntityKey::new(name).unwrap()])
                .is_none()
        );
    }
    drop(saved);
    let backend = SqliteBackend::open(&save_path, Durability::PowerLoss).unwrap();
    let disk = backend.facts_of(WorldRevision::GENESIS).unwrap();
    let genesis: Vec<_> = disk
        .iter()
        .map(|row| format::decode(&row.bytes, "fact").unwrap())
        .collect();
    pack.check_configuration(&genesis)
        .expect("unchanged resume guard");
    for (file, old, new) in [
        ("people/nadia.yaml", "stature: 1.66", "stature: 1.67"),
        ("locks/cast.yaml", "1.870", "1.880"),
        ("people/pia.yaml", "allowed: [blond]", "allowed: [brown]"),
        ("populations/test-town.yaml", "mean: 1.626", "mean: 1.627"),
        ("configure/attributes.yaml", "salt: 0", "salt: 1"),
        (
            "configure/attributes.yaml",
            "seed: fixture-pop-v1",
            "seed: different",
        ),
        (
            "configure/attributes.yaml",
            "nadia, omar, pia, quin",
            "nadia, omar, pia",
        ),
    ] {
        let path = source.join(file);
        let original = std::fs::read_to_string(&path).unwrap();
        assert!(original.contains(old), "mutation anchor {file}");
        std::fs::write(&path, original.replace(old, new)).unwrap();
        let changed = WorldPack::read(&source).unwrap();
        assert!(
            matches!(changed.check_configuration(&genesis),Err(PackError::ConfigurationDrift { system,.. }) if system.as_str()=="attributes"),
            "{file}"
        );
        std::fs::write(path, original).unwrap();
    }
    assert_eq!(backend.head().unwrap(), WorldRevision::GENESIS);
    assert_eq!(
        backend
            .facts_of(WorldRevision::GENESIS)
            .unwrap()
            .iter()
            .map(|row| &row.bytes)
            .collect::<Vec<_>>(),
        disk.iter().map(|row| &row.bytes).collect::<Vec<_>>()
    );
}

#[test]
fn deferred_keys_fail_closed_and_a_profile_is_optional() {
    let root = mineworld_test_support::scratch!(empty "newcomer-invalid-keys");
    let source = root.path().join("attributes-newcomers");
    copy(Path::new(FIXTURE), &source);
    let config = source.join("configure/attributes.yaml");
    let original = std::fs::read_to_string(&config).unwrap();
    for (list, expected) in [
        ("nadia, ghost", "UnknownPerson"),
        ("nadia, nadia", "DuplicateNewcomer"),
        ("cafe", "UnknownPerson"),
    ] {
        std::fs::write(&config, original.replace("nadia, omar, pia, quin", list)).unwrap();
        let error = WorldPack::read(&source).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
    std::fs::write(&config, original.replace("profile: test-town\n", "")).unwrap();
    let assembled = WorldPack::read(&source).unwrap().assemble().unwrap();
    let context: AttributesComposed = serde_json::from_slice(
        assembled
            .facts
            .iter()
            .find(|fact| fact.event_type() == &AttributesComposed::EVENT_TYPE)
            .unwrap()
            .record()
            .payload(),
    )
    .unwrap();
    let context = context.newcomers.unwrap();
    assert!(context.profile.is_none());
    assert_eq!(context.deferred.len(), 4);
    assert!(
        context.deferred[&EntityKey::new("quin").unwrap()]
            .values
            .is_empty()
    );
}
