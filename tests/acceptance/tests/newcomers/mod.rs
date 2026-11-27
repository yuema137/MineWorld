pub mod creator;
use mineworld_attributes_model::{AttributesComposed, Catalogue, DataFile};
use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};
use mineworld_contracts::{ActionId, ActionIntent, ActionRecord, EntityKey, WorldTime};
use mineworld_contracts::{Event, EventEnvelope, Visibility};
use mineworld_kernel::World;
use mineworld_kernel::{Emission, SystemIdentity};
use mineworld_worldpack::{WorldPack, load::AssembledWorld};
use std::collections::BTreeMap;
use std::path::Path;

pub const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/cli/tests/fixtures/attributes-newcomers"
);
pub const PEOPLE: [&str; 4] = ["nadia", "omar", "pia", "quin"];
/// The scenario requires each creator arrival to be seeded in this dispatch's facts.
/// Return the missing person so the adversarial case diagnoses the same check as the
/// positive persistence case, rather than asserting a separate weakened expectation.
#[allow(dead_code)] // The subprocess binary additionally compares the complete saved log.
pub fn complete_admissions(events: &[EventEnvelope]) -> Result<(), mineworld_contracts::PersonId> {
    for (index, event) in events.iter().enumerate() {
        if event.event_type() != &creator::Arrived::EVENT_TYPE {
            continue;
        }
        let arrival: creator::Arrived = serde_json::from_slice(event.payload().payload()).unwrap();
        let stated = events[index + 1..].iter().any(|fact| {
            if fact.event_type() != &mineworld_attributes_model::TraitsStated::EVENT_TYPE {
                return false;
            }
            let stated: mineworld_attributes_model::TraitsStated =
                serde_json::from_slice(fact.payload().payload()).unwrap();
            stated.person == arrival.person
        });
        if !stated {
            return Err(arrival.person);
        }
    }
    Ok(())
}
pub fn pack() -> WorldPack {
    WorldPack::read(Path::new(FIXTURE)).expect("fixture reads")
}
pub fn assembled(pack: &WorldPack) -> AssembledWorld {
    assembled_with_creator(pack, creator::Mode::Checked)
}
pub fn assembled_with_creator(pack: &WorldPack, mode: creator::Mode) -> AssembledWorld {
    let mut assembled = pack.assemble().expect("assemble real loader");
    assembled
        .world
        .install(creator::Creator { mode })
        .expect("install test-only source");
    // Closed installed registry: only the test source's Uses line is added by this adapter.
    // The production loader's complete newcomer map and every other fact remain untouched.
    // This declaration belongs to the installed test System, outside world-data discovery.
    let file = serde_saphyr::from_str(include_str!("arrivals.yaml")).expect("uses file");
    let origin = DataOrigin {
        pack: creator::Creator::ID.to_string(),
        source: DataSource::SystemPack,
        directory: DataDirectory::from_static("attributes"),
        path: "attributes/arrivals.yaml".into(),
    };
    // Validate the proposal against the original fixture declarations; extract only Uses.
    let attributes = serde_saphyr::from_str(include_str!(
        "../../../../tools/cli/tests/fixtures/attributes-newcomers/attributes/people.yaml"
    ))
    .unwrap();
    let attributes_origin = DataOrigin {
        pack: "attributes-newcomers".into(),
        source: DataSource::World,
        directory: DataDirectory::from_static("attributes"),
        path: "attributes/people.yaml".into(),
    };
    let uses = Catalogue::build(&[
        (attributes_origin, DataFile::Attributes(attributes)),
        (origin, DataFile::Attributes(file)),
    ])
    .expect("creator declares admits")
    .projection()
    .uses;
    assert_eq!(uses.len(), 1);
    assert!(uses.contains_key(&creator::Creator::ID));
    let original_facts = assembled.facts.clone();
    let mut normalized_facts = original_facts.clone();
    let mut patched = 0;
    for (index, fact) in assembled.facts.iter_mut().enumerate() {
        if fact.event_type() == &AttributesComposed::EVENT_TYPE {
            patched += 1;
            let original = fact.clone();
            let mut composed: AttributesComposed =
                serde_json::from_slice(fact.record().payload()).expect("composed");
            assert!(
                !composed.catalogue.uses.contains_key(&creator::Creator::ID),
                "never overwrite uses"
            );
            composed.catalogue.uses.extend(uses.clone());
            *fact = Emission::new::<AttributesComposed>(
                serde_json::to_vec(&composed).expect("encode"),
                Visibility::SystemInternal,
            );
            let mut normalized: AttributesComposed =
                serde_json::from_slice(fact.record().payload()).unwrap();
            assert_eq!(
                normalized.catalogue.uses.remove(&creator::Creator::ID),
                uses.get(&creator::Creator::ID).cloned()
            );
            let normalized = Emission::new::<AttributesComposed>(
                serde_json::to_vec(&normalized).unwrap(),
                Visibility::SystemInternal,
            );
            assert_eq!(
                normalized, original,
                "entire payload and emission metadata must survive the test adapter"
            );
            normalized_facts[index] = normalized;
        } else {
            normalized_facts[index] = fact.clone();
        }
    }
    assert_eq!(patched, 1, "exactly one composed fact");
    assert_eq!(
        normalized_facts, original_facts,
        "all other facts unchanged"
    );
    assembled
}
/// Test host comparison seam: actual saved genesis against the same narrow test composition.
/// This does not establish public discovery or loading of arbitrary System Packs.
#[allow(dead_code)] // Used by persistence binaries, not the refusal-only binary.
pub fn check_configuration(
    pack: &WorldPack,
    saved: &[EventEnvelope],
) -> Result<(), mineworld_worldpack::configure::Drift> {
    mineworld_worldpack::configure::compare(
        saved,
        &assembled(pack).facts,
        &BTreeMap::from([(
            AttributesComposed::EVENT_TYPE,
            mineworld_attributes_model::ATTRIBUTES,
        )]),
    )
}
#[allow(dead_code)] // Used by persistence binaries, not the refusal-only binary.
pub fn composed() -> World {
    let mut world = pack().compose().expect("compose").world;
    world.install(creator::Creator::default()).expect("source");
    world
}
pub fn intent(index: usize) -> ActionIntent {
    let assembled = assembled(&pack());
    let key = EntityKey::new(PEOPLE[index]).expect("key");
    ActionIntent::new(
        ActionId::from_raw(index as u64 + 1),
        assembled.ids[&key],
        ActionRecord::new::<creator::Arrive>(serde_json::to_vec(&creator::Arrive).expect("encode")),
        WorldTime::EPOCH,
    )
}
