//! The integration checkpoint of PR 03a (§2.6 of the step document): a whole small world, built by
//! two systems that own one component type each, exercised through every store at once.
//!
//! What this file is for that the per-module tests are not: the per-module tests check one store's
//! contract, and this one checks that a *world* made of all three behaves. Four claims:
//!
//! ```text
//! each system writes its own component and reads the other's
//! the cross-write is a compile error          — tests/compile_fail/, it cannot be written here
//! the whole world round-trips through serde unchanged
//! replaying the same operations on a fresh world reproduces identical ids and state
//! ```
//!
//! The third and fourth are the same property from two directions, and together they are what
//! `AC-6` and `AC-12` need from the kernel: state that can be written down and read back, and a
//! world that does not depend on when anything happened.

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityKey, EntityType, EntityTypeSet, LifecycleState, Metadata,
    RelationTypeDeclaration, RelationTypeId, SystemId, Tag, Tags,
};
use mineworld_kernel::{
    ComponentStore, EntityRegistry, EntityRegistrySnapshot, KernelError, RelationStore,
    RelationStoreSnapshot, SystemIdentity, WriteAccess, WriteToken, owned_component,
};
use serde::{Deserialize, Serialize};

/// The first of the two systems this world is made of.
struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

/// The second. It owns different state and declares no edge types, so every cross-system access in
/// this file is a real cross-system access.
struct SecondStub;
impl SystemIdentity for SecondStub {
    const ID: SystemId = SystemId::from_static("second-stub");
}

/// State the first system owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Measured {
    amount: u32,
}

owned_component! {
    component = Measured,
    owner = FirstStub,
    component_type = "measured",
    schema_version = 1,
}

/// State the second system owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Flagged {
    raised: bool,
}

owned_component! {
    component = Flagged,
    owner = SecondStub,
    component_type = "flagged",
    schema_version = 1,
}

/// Everything one world holds. The kernel does not provide this type yet — assembling a world is
/// PR 03b's job, once there is a system registry to assemble it from — so the checkpoint builds the
/// smallest honest stand-in: the three stores, together, with nothing else.
struct World {
    entities: EntityRegistry,
    components: ComponentStore,
    relations: RelationStore,
}

impl World {
    fn new(first: &WriteToken<FirstStub>, second: &WriteToken<SecondStub>) -> Self {
        let mut components = ComponentStore::new();
        components
            .declare::<Measured, _>(first)
            .expect("each system declares its own component type");
        components
            .declare::<Flagged, _>(second)
            .expect("each system declares its own component type");

        let mut relations = RelationStore::new();
        relations
            .declare(first, attachment())
            .expect("the first system declares its edge type");

        Self {
            entities: EntityRegistry::new(),
            components,
            relations,
        }
    }
}

/// Everything one world holds, written down: what persistence would carry, and what two worlds are
/// compared by.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct WorldSnapshot {
    entities: EntityRegistrySnapshot,
    measured: Vec<ComponentRecord<serde_json::Value>>,
    flagged: Vec<ComponentRecord<serde_json::Value>>,
    relations: RelationStoreSnapshot,
}

impl WorldSnapshot {
    fn of(world: &World) -> Self {
        Self {
            entities: EntityRegistrySnapshot::from(&world.entities),
            measured: records::<Measured>(&world.components),
            flagged: records::<Flagged>(&world.components),
            relations: RelationStoreSnapshot::from(&world.relations),
        }
    }
}

fn records<C>(store: &ComponentStore) -> Vec<ComponentRecord<serde_json::Value>>
where
    C: mineworld_contracts::Component + 'static,
{
    store
        .iter::<C>()
        .map(|(entity, component)| {
            ComponentRecord::new::<C>(
                entity,
                serde_json::to_value(component).expect("a component serializes"),
            )
        })
        .collect()
}

fn attachment() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        RelationTypeId::new("attached-to").expect("a legal declared name"),
        SystemId::from_static("first-stub"),
        EntityTypeSet::new([EntityType::Person]).expect("a non-empty set"),
        EntityTypeSet::new([EntityType::Organization]).expect("a non-empty set"),
    )
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring name")
}

/// One scripted run of a world: the same creations, writes, edges and destruction every time, in
/// the same order. Everything this checkpoint claims is a claim about this function being run twice.
fn run(
    world: &mut World,
    first: &WriteToken<FirstStub>,
    second: &WriteToken<SecondStub>,
) -> Result<(), KernelError> {
    let alice = world.entities.create_authored(
        key("alice"),
        EntityType::Person,
        Tags::new([Tag::new("resident").expect("a legal tag")]),
        Some(Metadata {
            source_pack: "checkpoint".to_owned(),
            source_path: "people/alice.yaml".to_owned(),
            authoring_note: None,
        }),
    )?;
    let guild = world
        .entities
        .create(key("guild"), EntityType::Organization)?;
    let bob = world.entities.create(key("bob"), EntityType::Person)?;
    let leaving = world.entities.create(key("leaving"), EntityType::Person)?;

    // Each system writes its own state, for whichever entities it cares about.
    world
        .components
        .insert(first, alice, Measured { amount: 3 })?;
    world
        .components
        .insert(first, bob, Measured { amount: 8 })?;
    world
        .components
        .insert(second, alice, Flagged { raised: true })?;
    world
        .components
        .insert(second, guild, Flagged { raised: false })?;

    // The first system reads the second's state and reacts to it by changing its own — the whole
    // point of open reads, and the only way two independent systems can compose.
    let raised = world
        .components
        .get::<Flagged>(alice)
        .is_some_and(|flagged| flagged.raised);
    if raised {
        world
            .components
            .get_mut::<Measured, _>(first, alice)?
            .expect("alice has this component")
            .amount += 1;
    }

    // Edges, written by the system that declared the type.
    let alice_record = world.entities.require(alice)?.clone();
    let guild_record = world.entities.require(guild)?.clone();
    let leaving_record = world.entities.require(leaving)?.clone();
    let relation_type = RelationTypeId::new("attached-to").expect("a legal declared name");
    world
        .relations
        .insert(first, &relation_type, &alice_record, &guild_record)?;
    world
        .relations
        .insert(first, &relation_type, &leaving_record, &guild_record)?;

    // And one entity leaves the world, which spends its identity and clears its edges.
    world
        .entities
        .transition(leaving, LifecycleState::Destroyed)?;
    let destroyed = world.entities.require(leaving)?.clone();
    world
        .relations
        .remove_edges_of_destroyed_entity(&destroyed)?;
    world.components.remove::<Measured, _>(first, leaving)?;

    Ok(())
}

/// Two systems, one world: each writes only its own component type, and each sees the other's.
#[test]
fn two_systems_write_their_own_state_and_read_each_others() {
    let mut access = WriteAccess::new();
    let first = access.grant(&FirstStub).expect("a first grant succeeds");
    let second = access.grant(&SecondStub).expect("a first grant succeeds");
    let mut world = World::new(&first, &second);

    run(&mut world, &first, &second).expect("the scripted run succeeds");

    let alice = world
        .entities
        .resolve(&key("alice"))
        .expect("alice was created");

    // The first system's own state, including the increment it made after reading the second's.
    assert_eq!(
        world.components.get::<Measured>(alice),
        Some(&Measured { amount: 4 })
    );
    // The second system's state, read without a token by whoever wants it.
    assert_eq!(
        world.components.get::<Flagged>(alice),
        Some(&Flagged { raised: true })
    );
    assert_eq!(world.components.count::<Measured>(), 2);
    assert_eq!(world.components.count::<Flagged>(), 2);
    assert_eq!(world.relations.len(), 1);
    assert_eq!(world.entities.len(), 4);

    // The cross-write — the first system's token on the second system's component — is absent from
    // this file because it cannot be compiled. See
    // tests/compile_fail/a_system_cannot_write_another_systems_component.rs.
}

/// The whole world, written down and read back, is the same world.
#[test]
fn a_whole_world_round_trips_through_serde_unchanged() {
    let mut access = WriteAccess::new();
    let first = access.grant(&FirstStub).expect("a first grant succeeds");
    let second = access.grant(&SecondStub).expect("a first grant succeeds");
    let mut world = World::new(&first, &second);
    run(&mut world, &first, &second).expect("the scripted run succeeds");

    let written = serde_json::to_string(&WorldSnapshot::of(&world)).expect("a world serializes");
    let read: WorldSnapshot = serde_json::from_str(&written).expect("a world deserializes");

    assert_eq!(read, WorldSnapshot::of(&world));

    // And the parts that carry their own restore path come back as usable stores, not just as
    // matching bytes.
    let entities = EntityRegistry::try_from(read.entities).expect("a consistent registry restores");
    assert_eq!(entities, world.entities);
    let relations = RelationStore::try_from(read.relations).expect("a consistent graph restores");
    assert_eq!(relations, world.relations);
}

/// Replay: the same operations on a fresh world produce the same identities and the same state,
/// byte for byte. Nothing in the kernel depends on when anything happened, or on anything outside
/// the world.
#[test]
fn replaying_the_same_operations_reproduces_an_identical_world() {
    let snapshots: Vec<String> = (0..2)
        .map(|_| {
            let mut access = WriteAccess::new();
            let first = access.grant(&FirstStub).expect("a first grant succeeds");
            let second = access.grant(&SecondStub).expect("a first grant succeeds");
            let mut world = World::new(&first, &second);
            run(&mut world, &first, &second).expect("the scripted run succeeds");
            serde_json::to_string(&WorldSnapshot::of(&world)).expect("a world serializes")
        })
        .collect();

    assert_eq!(snapshots[0], snapshots[1]);
    assert!(
        snapshots[0].contains("\"alice\""),
        "the comparison must be of a world with something in it: {}",
        snapshots[0]
    );
    // The identities in particular: allocation is a counter, so the fourth entity is 4 in both runs
    // and in every run after them.
    let world: serde_json::Value =
        serde_json::from_str(&snapshots[0]).expect("the snapshot parses");
    assert_eq!(world["entities"]["next_id"], 5);
    assert_eq!(
        world["entities"]["entities"]["4"]["key"],
        serde_json::json!("leaving")
    );
    assert_eq!(
        world["relations"]["edges"][0]["from"],
        serde_json::json!(EntityId::from_raw(1).raw())
    );
}
