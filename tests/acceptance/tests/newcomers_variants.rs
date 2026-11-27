//! Source-authored variants exercise admission without a profile and a masked restriction.
mod newcomers;
use mineworld_attributes_model::{Traits, TraitsStated};
use mineworld_contracts::{ActionResult, EntityKey, Event, Rejection, WorldTime};
use mineworld_worldpack::WorldPack;
use newcomers::*;
use std::path::Path;

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
fn no_profile_preserves_fixed_values_and_records_an_empty_arrival() {
    let scratch = mineworld_test_support::scratch!(empty "newcomer-no-profile");
    let source = scratch.path().join("attributes-newcomers");
    copy(Path::new(FIXTURE), &source);
    let config = source.join("configure/attributes.yaml");
    let original = std::fs::read_to_string(&config).unwrap();
    std::fs::write(config, original.replace("profile: test-town\n", "")).unwrap();
    let assembled = assembled(&WorldPack::read(&source).unwrap());
    let quin = assembled.ids[&EntityKey::new("quin").unwrap()];
    let mut world = assembled.world;
    let _ = world.genesis(WorldTime::EPOCH, assembled.facts).unwrap();
    for index in [0, 1, 3] {
        let result = world.dispatch(&intent(index), WorldTime::EPOCH).unwrap();
        assert!(matches!(result.result(), ActionResult::Accepted { .. }));
        complete_admissions(result.events()).unwrap();
        let stated: TraitsStated = serde_json::from_slice(
            result
                .events()
                .iter()
                .find(|event| event.event_type() == &TraitsStated::EVENT_TYPE)
                .unwrap()
                .payload()
                .payload(),
        )
        .unwrap();
        if index == 3 {
            assert!(stated.values.is_empty());
        } else {
            let stature = stated
                .values
                .iter()
                .find(|(id, _)| id.as_str() == "attributes-newcomers.stature")
                .unwrap();
            assert_eq!(
                stature.1.display(),
                if index == 0 { "1.660" } else { "1.870" }
            );
        }
    }
    assert!(world.read().component::<Traits>(quin).unwrap().is_empty());
}

#[test]
fn impossible_authored_restriction_is_an_ordinary_named_refusal() {
    let scratch = mineworld_test_support::scratch!(empty "newcomer-unsatisfiable");
    let source = scratch.path().join("attributes-newcomers");
    copy(Path::new(FIXTURE), &source);
    let person = source.join("people/nadia.yaml");
    let original = std::fs::read_to_string(&person).unwrap();
    std::fs::write(
        person,
        format!("{original}  facial-hair: {{ allowed: [beard] }}\n"),
    )
    .unwrap();
    let assembled = assembled(&WorldPack::read(&source).unwrap());
    let mut world = assembled.world;
    let _ = world.genesis(WorldTime::EPOCH, assembled.facts).unwrap();
    let result = world.dispatch(&intent(0), WorldTime::EPOCH).unwrap();
    let ActionResult::Rejected(Rejection::System { code, detail }) = result.result() else {
        panic!("a checked invalid admission must be rejected");
    };
    assert_eq!(code.as_str(), "newcomer-unsatisfiable");
    assert!(detail.as_ref().unwrap().contains("nadia"));
    assert!(result.events().is_empty());
}
