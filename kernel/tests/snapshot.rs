//! A world written down and read back: `World::snapshot` and `World::restore` (S5, step-06 C2).
//!
//! ```text
//! ledger   owns `tally`; `note` records a fact, relates the noter to whom they noted, and defers a
//!          reminder; `brew` starts a process that ends five minutes later
//! echo     owns `heard`; reduces ledger's facts about their subject — a second system's state
//! ```
//!
//! The claims:
//!
//! ```text
//! a snapshot carries every kind of state a world holds: rows of two systems, an edge, a pending
//!   deferral, a running process, and the counters                                 (located first)
//! restored into a freshly composed world it reads back byte for byte, and the two worlds then
//!   continue identically — same facts, same identities                             (I-2, I-3)
//! every way a snapshot can fail to fit the world is refused by name, and changes nothing (I-5)
//! ```

use std::collections::BTreeSet;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionTypeId, Component, ComponentRecord,
    ComponentSchemaVersion, ComponentTypeId, ContractError, Entity, EntityId, EntityKey,
    EntityType, EntityTypeSet, Event, EventEnvelope, EventSchemaVersion, EventTypeId,
    ProcessTypeId, Relation, RelationTypeDeclaration, RelationTypeId, SystemId, Visibility,
    WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessKind, ProcessStart, System,
    SystemDeclaration, SystemIdentity, SystemVersion, World, WorldSnapshot, WorldView,
    owned_component,
};
use serde::{Deserialize, Serialize};

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

// ---------------------------------------------------------------------------------------------
// ledger
// ---------------------------------------------------------------------------------------------

struct Ledger;

impl SystemIdentity for Ledger {
    const ID: SystemId = SystemId::from_static("ledger");
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Tally {
    count: u32,
    notes: Vec<String>,
}

owned_component! {
    component = Tally,
    owner = Ledger,
    component_type = "tally",
    schema_version = 1,
}

/// The same component type name and owner at a newer schema, as a later version of `ledger` would
/// declare it. Not installed anywhere: it exists to write a record no version-1 world can read.
#[derive(Serialize, Deserialize)]
struct TallyFromTheFuture {
    count: u32,
}

impl Component for TallyFromTheFuture {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("tally");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(2);
}

/// And at an older one.
#[derive(Serialize, Deserialize)]
struct TallyFromThePast {
    count: u32,
}

impl Component for TallyFromThePast {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("tally");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(0);
}

/// State of a system this world is never composed of.
#[derive(Serialize, Deserialize)]
struct Wallet {
    coins: u32,
}

impl Component for Wallet {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("wallet");
    const OWNER: SystemId = SystemId::from_static("economy");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

fn noted_by() -> RelationTypeId {
    RelationTypeId::new("noted-by").expect("a legal name")
}

fn noted_by_declaration() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        noted_by(),
        Ledger::ID,
        EntityTypeSet::new([EntityType::Person]).expect("non-empty"),
        EntityTypeSet::new([EntityType::Person]).expect("non-empty"),
    )
}

#[derive(Serialize, Deserialize)]
struct Note {
    text: String,
}

impl Action for Note {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("note");
    const OWNER: SystemId = Ledger::ID;
}

#[derive(Serialize, Deserialize)]
struct Brew;

impl Action for Brew {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("brew");
    const OWNER: SystemId = Ledger::ID;
}

#[derive(Serialize, Deserialize)]
struct Noted {
    text: String,
}

impl Event for Noted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("noted");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
struct Reminded {
    text: String,
}

impl Event for Reminded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("reminded");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
struct Brewed;

impl Event for Brewed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("brewed");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

struct Brewing;

impl ProcessKind for Brewing {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("brewing");
    type Owner = Ledger;
}

/// How long a brew takes, and how long after a note its reminder comes.
const BREW: i64 = 300;
const REMINDER: i64 = 100;

impl System for Ledger {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Tally>()
            .providing::<Note>()
            .providing::<Brew>()
            .emitting::<Noted>()
            .emitting::<Reminded>()
            .emitting::<Brewed>()
            .subscribing_to::<Noted>()
            .subscribing_to::<Reminded>()
            .subscribing_to::<Brewed>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Tally>()?;
        tables.relation(noted_by_declaration())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let subject = intent.target().unwrap_or(intent.actor());
        if *intent.action_type() == Brew::ACTION_TYPE {
            world.start_process(
                ProcessStart::<Brewing>::new(Vec::new())
                    .with_participants(vec![subject])
                    .ending_at(t(world.at().seconds() + BREW)),
            )?;
            return Ok(Vec::new());
        }
        let note: Note = serde_json::from_slice(intent.payload().payload()).expect("a note");
        if let Some(target) = intent.target() {
            world.relate(&noted_by(), target, intent.actor())?;
        }
        world.defer(
            t(world.at().seconds() + REMINDER),
            Emission::new::<Reminded>(
                encode(&Reminded {
                    text: note.text.clone(),
                }),
                Visibility::Public,
            )
            .about(vec![subject]),
        )?;
        Ok(vec![
            Emission::new::<Noted>(encode(&Noted { text: note.text }), Visibility::Public)
                .about(vec![subject]),
        ])
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let subject = event.subjects()[0];
        let mut tally = world
            .read()
            .component::<Tally>(subject)
            .cloned()
            .unwrap_or_default();
        tally.count += 1;
        tally.notes.push(event.event_type().to_string());
        world.insert(subject, tally)?;
        Ok(Vec::new())
    }

    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        world.end_process::<Brewing>(process.id())?;
        Ok(vec![
            Emission::new::<Brewed>(Vec::new(), Visibility::Public)
                .about(process.participants().to_vec()),
        ])
    }
}

// ---------------------------------------------------------------------------------------------
// echo
// ---------------------------------------------------------------------------------------------

struct Echo;

impl SystemIdentity for Echo {
    const ID: SystemId = SystemId::from_static("echo");
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Heard {
    times: u32,
}

owned_component! {
    component = Heard,
    owner = Echo,
    component_type = "heard",
    schema_version = 1,
}

impl System for Echo {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Heard>()
            .subscribing_to::<Noted>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Heard>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let subject = event.subjects()[0];
        let mut heard = world
            .read()
            .component::<Heard>(subject)
            .cloned()
            .unwrap_or_default();
        heard.times += 1;
        world.insert(subject, heard)?;
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------------------------
// the world
// ---------------------------------------------------------------------------------------------

/// The systems, installed and nothing else: the shape a restart begins from.
fn composed() -> World {
    let mut world = World::new();
    world.install(Ledger).expect("ledger installs");
    world.install(Echo).expect("echo installs");
    world
}

const PEOPLE: [&str; 3] = ["ada", "bo", "cy"];

/// A world that has lived a little: three people, notes between them, a brew in progress and
/// reminders still pending.
fn lived() -> (World, Vec<EntityId>) {
    let mut world = composed();
    let people: Vec<EntityId> = PEOPLE
        .iter()
        .map(|key| {
            world
                .create_entity(EntityKey::new(*key).expect("a key"), EntityType::Person)
                .expect("created")
        })
        .collect();
    let mut next_action = 1;
    for (at, (actor, target, text)) in [
        (10, (0, Some(1), "bread")),
        (20, (1, Some(2), "salt")),
        (30, (2, None, "rain")),
    ] {
        note(
            &mut world,
            &mut next_action,
            at,
            people[actor],
            target.map(|i| people[i]),
            text,
        );
    }
    brew(&mut world, &mut next_action, 40, people[0]);
    (world, people)
}

fn note(
    world: &mut World,
    next_action: &mut u64,
    at: i64,
    actor: EntityId,
    target: Option<EntityId>,
    text: &str,
) -> Vec<EventEnvelope> {
    let mut intent = ActionIntent::new(
        ActionId::from_raw(*next_action),
        actor,
        ActionRecord::new::<Note>(encode(&Note {
            text: text.to_owned(),
        })),
        t(at),
    );
    if let Some(target) = target {
        intent = intent.with_target(target);
    }
    *next_action += 1;
    run(world, &intent, at)
}

fn brew(world: &mut World, next_action: &mut u64, at: i64, actor: EntityId) -> Vec<EventEnvelope> {
    let intent = ActionIntent::new(
        ActionId::from_raw(*next_action),
        actor,
        ActionRecord::new::<Brew>(encode(&Brew)),
        t(at),
    );
    *next_action += 1;
    run(world, &intent, at)
}

/// Advances to `at`, then dispatches there: the order a host follows.
fn run(world: &mut World, intent: &ActionIntent, at: i64) -> Vec<EventEnvelope> {
    let mut facts = world.advance_to(t(at)).expect("advances").into_events();
    facts.extend(
        world
            .dispatch(intent, t(at))
            .expect("dispatches")
            .events()
            .to_vec(),
    );
    facts
}

/// What happens next, identically for any world it is given.
fn continuation(world: &mut World, people: &[EntityId]) -> Vec<EventEnvelope> {
    let mut next_action = 100;
    let mut facts = Vec::new();
    facts.extend(note(
        world,
        &mut next_action,
        200,
        people[1],
        Some(people[0]),
        "jam",
    ));
    facts.extend(brew(world, &mut next_action, 250, people[2]));
    facts.extend(world.advance_to(t(2_000)).expect("advances").into_events());
    facts
}

fn bytes(snapshot: &WorldSnapshot) -> Vec<u8> {
    serde_json::to_vec(snapshot).expect("a snapshot encodes")
}

fn read_back(bytes: &[u8]) -> WorldSnapshot {
    serde_json::from_slice(bytes).expect("a snapshot decodes")
}

// ---------------------------------------------------------------------------------------------
// round trip and continuation
// ---------------------------------------------------------------------------------------------

#[test]
fn a_restored_world_reads_back_byte_for_byte_and_continues_exactly_as_the_original() {
    let (mut original, people) = lived();
    let snapshot = original.snapshot().expect("snapshots");

    // Locate before comparing (ARC-23): the snapshot holds every kind of state the scenario put
    // there, so an equality below is not two empty worlds agreeing.
    let types: BTreeSet<&str> = snapshot
        .components
        .iter()
        .map(|record| record.component_type().as_str())
        .collect();
    assert_eq!(
        types,
        BTreeSet::from(["heard", "tally"]),
        "rows of two systems"
    );
    assert_eq!(
        snapshot.components.len(),
        4,
        "bo and cy were noted: a tally and a heard row each"
    );
    assert_eq!(snapshot.entities.entities.len(), 3);
    assert_eq!(
        snapshot.relations.edges.len(),
        2,
        "two notes named somebody"
    );
    assert_eq!(
        snapshot.time.entries.len(),
        4,
        "three reminders and one brew's end pending"
    );
    assert_eq!(snapshot.time.processes.len(), 1, "one brew in progress");
    assert_eq!(snapshot.time.next_event, 4, "three facts recorded so far");
    assert!(snapshot.clock_started && snapshot.ran);

    let saved = bytes(&snapshot);
    let mut restored = composed();
    restored.restore(read_back(&saved)).expect("restores");
    assert_eq!(
        bytes(&restored.snapshot().expect("snapshots")),
        saved,
        "written down again, the restored world is the same bytes"
    );
    assert_eq!(restored.now(), t(40));

    let after_original = continuation(&mut original, &people);
    let after_restored = continuation(&mut restored, &people);
    // The three pending reminders (110, 120, 130), "jam" noted at 200, its reminder at 300, and two
    // brews ending (340, 550): seven facts, derived from the script rather than read off the code.
    assert_eq!(after_original.len(), 7);
    assert_eq!(
        after_original[0].id().raw(),
        4,
        "identity continues where the saved world stopped"
    );
    assert_eq!(
        encode(&after_restored),
        encode(&after_original),
        "the same inputs produce the same facts, byte for byte"
    );
    assert_eq!(
        bytes(&restored.snapshot().expect("snapshots")),
        bytes(&original.snapshot().expect("snapshots")),
        "and the same state"
    );
}

// ---------------------------------------------------------------------------------------------
// refusals
// ---------------------------------------------------------------------------------------------

/// Restoring `snapshot` into `world` is refused with `expected`, and leaves `world` as it was.
fn refused(mut world: World, snapshot: WorldSnapshot, expected: &KernelError) {
    let before = bytes(&world.snapshot().expect("snapshots"));
    let error = world.restore(snapshot).expect_err("the restore is refused");
    assert_eq!(&error, expected);
    assert_eq!(
        bytes(&world.snapshot().expect("snapshots")),
        before,
        "a refusal changes nothing"
    );
}

fn saved() -> (WorldSnapshot, Vec<EntityId>) {
    let (world, people) = lived();
    (world.snapshot().expect("snapshots"), people)
}

#[test]
fn a_world_that_has_run_or_holds_state_cannot_be_restored_into() {
    let (snapshot, _) = saved();

    let (ran, _) = lived();
    refused(
        ran,
        snapshot.clone(),
        &KernelError::RestoreAfterTheWorldHasRun,
    );

    let mut populated = composed();
    populated
        .create_entity(EntityKey::new("stray").expect("a key"), EntityType::Person)
        .expect("created");
    refused(populated, snapshot, &KernelError::RestoreIntoPopulatedWorld);
}

#[test]
fn a_snapshot_of_a_differently_composed_world_is_refused_at_the_first_difference() {
    let (snapshot, _) = saved();

    let mut missing = World::new();
    missing.install(Ledger).expect("installs");
    refused(
        missing,
        snapshot.clone(),
        &KernelError::RestoredCompositionDiffers {
            position: 1,
            saved: Some(Echo::ID),
            installed: None,
        },
    );

    // The same systems in the other order reduce in the other order: another world.
    let mut swapped = World::new();
    swapped.install(Echo).expect("installs");
    swapped.install(Ledger).expect("installs");
    refused(
        swapped,
        snapshot.clone(),
        &KernelError::RestoredCompositionDiffers {
            position: 0,
            saved: Some(Ledger::ID),
            installed: Some(Echo::ID),
        },
    );

    let mut disabled = composed();
    disabled.disable(&Echo::ID).expect("disables");
    refused(
        disabled,
        snapshot,
        &KernelError::RestoredCompositionDiffers {
            position: 1,
            saved: Some(Echo::ID),
            installed: Some(Echo::ID),
        },
    );
}

#[test]
fn a_component_row_that_does_not_fit_this_world_is_refused_by_name() {
    let (snapshot, people) = saved();
    let with = |record: ComponentRecord| {
        let mut changed = snapshot.clone();
        changed.components.push(record);
        changed
    };
    let tally_type = ComponentTypeId::from_static("tally");
    let first = snapshot.components[0].clone();

    refused(
        composed(),
        with(ComponentRecord::new::<Wallet>(
            people[0],
            encode(&Wallet { coins: 5 }),
        )),
        &KernelError::PersistedComponentTypeNotInstalled {
            component_type: Wallet::COMPONENT_TYPE,
            entity: people[0],
        },
    );
    refused(
        composed(),
        with(ComponentRecord::new::<Tally>(
            EntityId::from_raw(999),
            encode(&Tally::default()),
        )),
        &KernelError::PersistedComponentForUnknownEntity {
            component_type: tally_type.clone(),
            entity: EntityId::from_raw(999),
        },
    );
    refused(
        composed(),
        with(first.clone()),
        &KernelError::PersistedComponentRepeated {
            component_type: first.component_type().clone(),
            entity: first.entity(),
        },
    );

    // The remaining three replace one row rather than add one, so that the row in question is the
    // only one for its entity.
    let replacing = |record: ComponentRecord| {
        let mut changed = snapshot.clone();
        let position = changed
            .components
            .iter()
            .position(|row| row.component_type() == &tally_type)
            .expect("a tally row");
        changed.components[position] = record;
        changed
    };
    let entity = snapshot
        .components
        .iter()
        .find(|row| row.component_type() == &tally_type)
        .expect("a tally row")
        .entity();

    let undecodable = refusal(replacing(ComponentRecord::new::<Tally>(
        entity,
        b"{".to_vec(),
    )));
    assert!(
        matches!(
            &undecodable,
            KernelError::PersistedComponentUndecodable { component_type, entity: at, .. }
                if *component_type == tally_type && *at == entity
        ),
        "{undecodable:?}"
    );
    refused(
        composed(),
        replacing(ComponentRecord::new::<TallyFromTheFuture>(
            entity,
            encode(&TallyFromTheFuture { count: 1 }),
        )),
        &KernelError::Contract(ContractError::ComponentSchemaTooNew {
            component_type: tally_type.clone(),
            record: ComponentSchemaVersion::new(2),
            supported: ComponentSchemaVersion::new(1),
        }),
    );
    refused(
        composed(),
        replacing(ComponentRecord::new::<TallyFromThePast>(
            entity,
            encode(&TallyFromThePast { count: 1 }),
        )),
        &KernelError::Contract(ContractError::ComponentSchemaOutdated {
            component_type: tally_type,
            record: ComponentSchemaVersion::new(0),
            supported: ComponentSchemaVersion::new(1),
        }),
    );
}

/// The refusal restoring `snapshot` into a freshly composed world produces, for an assertion on its
/// shape when the decoder's own message is part of it.
fn refusal(snapshot: WorldSnapshot) -> KernelError {
    let mut world = composed();
    world.restore(snapshot).expect_err("the restore is refused")
}

#[test]
fn an_edge_or_a_relation_declaration_that_does_not_fit_this_world_is_refused_by_name() {
    let (snapshot, people) = saved();

    let mut undeclared = snapshot.clone();
    undeclared.relations.declarations.clear();
    refused(
        composed(),
        undeclared,
        &KernelError::RestoredRelationDeclarationsDiffer {
            relation_type: noted_by(),
        },
    );

    let ghost = Entity::new(
        EntityId::from_raw(999),
        EntityKey::new("ghost").expect("a key"),
        EntityType::Person,
    );
    let ada = Entity::new(
        people[0],
        EntityKey::new("ada").expect("a key"),
        EntityType::Person,
    );
    let mut dangling = snapshot;
    dangling.relations.edges.insert(
        Relation::between(&noted_by_declaration(), &ada, &ghost).expect("a well-formed edge"),
    );
    refused(
        composed(),
        dangling,
        &KernelError::PersistedRelationForUnknownEntity {
            relation_type: noted_by(),
            entity: EntityId::from_raw(999),
        },
    );
}
