mod newcomers;
use mineworld_attributes::AttributesSystem;
use mineworld_attributes_model::{Catalogue, DataFile, Newcomers, Traits, check_newcomer};
use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionResult, EntityKey, EntityType, PersonId, Rejection, Visibility,
    WorldTime,
};
use mineworld_kernel::{KernelError, SystemIdentity, World};
use mineworld_presence::PerceptionProvider;
use newcomers::{
    creator::{Creator, Mode},
    *,
};

fn code(reason: &Rejection) -> &str {
    match reason {
        Rejection::System { code, .. } => code.as_str(),
        other => panic!("unexpected {other:?}"),
    }
}
fn begin(mode: Mode) -> World {
    let assembled = assembled_with_creator(&pack(), mode);
    let mut world = assembled.world;
    let _ = world.genesis(WorldTime::EPOCH, assembled.facts).unwrap();
    world
}
fn actor(world: &World, key: &str) -> PersonId {
    let id = assembled(&pack()).ids[&EntityKey::new(key).unwrap()];
    assert!(world.read().entity(id).is_some());
    PersonId::new(id, EntityType::Person).unwrap()
}

#[test]
fn ordinary_refusals_and_private_admission_do_not_leak_fixed_context() {
    let mut world = begin(Mode::Checked);
    let nadia = actor(&world, "nadia");
    let dev = actor(&world, "dev");
    assert_eq!(
        code(&check_newcomer(&world.read(), &Creator::ID, dev).unwrap_err()),
        "not-a-newcomer"
    );
    assert_eq!(
        code(
            &mineworld_attributes_model::newcomer::check_arrived(&world.read(), nadia).unwrap_err()
        ),
        "not-arrived"
    );
    let hair =
        mineworld_attributes_model::QualifiedId::new("attributes-newcomers.hair-colour").unwrap();
    let brown = mineworld_attributes_model::AttributeValue::Category(
        mineworld_attributes_model::ValueSlug::new("brown").unwrap(),
    );
    assert_eq!(
        code(
            &mineworld_attributes_model::facts::admit_proposal(
                &world.read(),
                &Creator::ID,
                nadia,
                &hair,
                &brown
            )
            .unwrap_err()
        ),
        "not-arrived"
    );
    assert!(
        AttributesSystem::default()
            .discloses(&world.read(), dev.entity_id(), nadia.entity_id())
            .is_empty()
    );
    assert!(
        AttributesSystem::default()
            .discloses(&world.read(), nadia.entity_id(), nadia.entity_id())
            .is_empty()
    );
    let result = world.dispatch(&intent(0), WorldTime::EPOCH).unwrap();
    assert!(matches!(result.result(), ActionResult::Accepted { .. }));
    assert_eq!(result.events()[1].visibility(), &Visibility::SystemInternal);
    assert_eq!(
        result.events()[2].visibility(),
        &Visibility::Entities([nadia.entity_id()].into())
    );
    let disclosed =
        AttributesSystem::default().discloses(&world.read(), dev.entity_id(), nadia.entity_id());
    let json = serde_json::to_string(&disclosed).unwrap();
    for private in [
        "deferred",
        "fixture-pop-v1",
        "Pinned",
        "locked",
        "sampled",
        "restrictions",
    ] {
        assert!(!json.contains(private), "private context leaked: {private}");
    }
    let original = intent(0);
    let again = ActionIntent::new(
        ActionId::from_raw(2),
        original.actor(),
        original.payload().clone(),
        WorldTime::EPOCH,
    );
    let result = world.dispatch(&again, WorldTime::EPOCH).unwrap();
    let ActionResult::Rejected(reason) = result.result() else {
        panic!("must reject")
    };
    assert_eq!(code(reason), "already-arrived");
    assert!(result.events().is_empty());
    assert_eq!(
        Newcomers::in_world(&world.read()).unwrap().1.arrived.len(),
        1
    );
}

#[test]
fn world_origin_cannot_grant_admission_and_missing_system_uses_refuses() {
    let data = serde_saphyr::from_str(include_str!("newcomers/arrivals.yaml")).unwrap();
    let origin = DataOrigin {
        pack: Creator::ID.to_string(),
        source: DataSource::World,
        directory: DataDirectory::from_static("attributes"),
        path: "attributes/arrivals.yaml".into(),
    };
    let error = Catalogue::build(&[(origin, DataFile::Attributes(data))]).unwrap_err();
    assert!(error.to_string().contains("only a System Pack"));
    let assembled = pack().assemble().unwrap();
    let mut world = assembled.world;
    world.install(Creator::default()).unwrap();
    let _ = world.genesis(WorldTime::EPOCH, assembled.facts).unwrap();
    let result = world.dispatch(&intent(0), WorldTime::EPOCH).unwrap();
    let ActionResult::Rejected(reason) = result.result() else {
        panic!("must reject")
    };
    assert_eq!(code(reason), "not-an-admitter");
    assert!(result.events().is_empty());
}

#[test]
fn unchecked_creator_is_stopped_by_owner_and_duplicate_generation_is_refused() {
    let mut world = begin(Mode::Unchecked);
    let dev = actor(&world, "dev");
    let template = intent(0);
    let request = ActionIntent::new(
        ActionId::from_raw(1),
        dev.entity_id(),
        template.payload().clone(),
        WorldTime::EPOCH,
    );
    let error = world.dispatch(&request, WorldTime::EPOCH).unwrap_err();
    assert!(
        matches!(error, KernelError::FactRefusedByOwner { reason, system, .. }
        if system == mineworld_attributes_model::ATTRIBUTES && code(&reason) == "not-a-newcomer")
    );
    let mut world = begin(Mode::Duplicate);
    let error = world.dispatch(&intent(0), WorldTime::EPOCH).unwrap_err();
    assert!(
        matches!(error, KernelError::FactRefusedByOwner { reason, system, .. }
        if system == mineworld_attributes_model::ATTRIBUTES && code(&reason) == "already-arrived")
    );
    let mut world = begin(Mode::FutureCause);
    let error = world.dispatch(&intent(0), WorldTime::EPOCH).unwrap_err();
    assert!(
        matches!(error, KernelError::FactRefusedByOwner { reason, system, .. }
        if system == mineworld_attributes_model::ATTRIBUTES && code(&reason) == "unknown-cause")
    );
}

#[test]
fn absent_catalogue_and_absent_newcomers_have_distinct_refusals() {
    let assembled = assembled(&pack());
    let nadia = mineworld_contracts::PersonId::new(
        assembled.ids[&EntityKey::new("nadia").unwrap()],
        EntityType::Person,
    )
    .unwrap();
    assert_eq!(
        code(&check_newcomer(&assembled.world.read(), &Creator::ID, nadia).unwrap_err()),
        "attributes-uncomposed"
    );
    let original = mineworld_worldpack::WorldPack::read(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/cli/tests/fixtures/attributes-sampled"
    )))
    .unwrap()
    .assemble()
    .unwrap();
    let person = PersonId::new(
        original.ids[&EntityKey::new("dev").unwrap()],
        EntityType::Person,
    )
    .unwrap();
    let mut world = original.world;
    let _ = world.genesis(WorldTime::EPOCH, original.facts).unwrap();
    assert_eq!(
        code(&check_newcomer(&world.read(), &Creator::ID, person).unwrap_err()),
        "no-newcomers"
    );
}

#[test]
fn missing_request_plant_is_detected_by_same_revision_completeness() {
    let mut world = begin(Mode::Missing);
    let nadia = actor(&world, "nadia");
    let result = world.dispatch(&intent(0), WorldTime::EPOCH).unwrap();
    assert_eq!(result.events().len(), 1);
    assert_eq!(complete_admissions(result.events()), Err(nadia));
    assert_eq!(result.events()[0].event_type().as_str(), "test-arrived");
    assert!(
        !result
            .events()
            .iter()
            .any(|event| event.event_type().as_str() == "traits-stated")
    );
    assert!(
        world
            .read()
            .component::<Traits>(nadia.entity_id())
            .is_none()
    );
    assert!(
        Newcomers::in_world(&world.read())
            .unwrap()
            .1
            .arrived
            .is_empty()
    );
    // The scenario's mandatory sequence assertion is falsified by this plant; the kernel
    // cannot infer an admission request the creator never emitted (frozen F-S30j-4).
}
