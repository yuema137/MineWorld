//! The integration checkpoint: a whole small world, composed of two systems, exercised through the
//! whole pipeline at once.
//!
//! This file is PR 03b's §4.5 checkpoint and it also restores PR 03a's. Three of the four claims 03a
//! made here moved elsewhere when the real [`World`] replaced its hand-rolled stand-in; two did not
//! move anywhere, and they are back, against the real world rather than a stub:
//!
//! ```text
//! disabling a system removes its action and nothing else changes     ← AC-2, the claim of the PR
//! a whole world round-trips through serde unchanged                  ← AC-6, restored from 03a
//! replaying the same operations reproduces an identical snapshot      ← AC-12, restored from 03a
//! ```
//!
//! # Why the first one is the important test in this PR
//!
//! MineWorld's frozen top-level acceptance criterion is that *materially different games can be
//! constructed by composing the same core entities with different independently installable
//! interaction systems, without modifying the kernel.* Everything else in the kernel exists to make
//! that true. `AC-2` is the first place it is directly demonstrated, and the demonstration has to be
//! honest about what changed: the two worlds below differ in **one boolean**, passed to one function,
//! and nothing else — not a line of system code, not a declaration, not a dispatch. In one world
//! there is weather and an action to ask about it. In the other there is neither, and the world is
//! not broken, it is simply a different world.
//!
//! The systems are called `Places` and `Weather` because they have to be called something. Neither
//! knows what it means: one counts occupants, the other keeps a flag, and the kernel never learns
//! that either is about a place or the sky (`INV-12`).

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, Component,
    ComponentRecord, EntityId, EntityKey, EntityType, EntityTypeSet, Event, EventEnvelope,
    EventSchemaVersion, EventTypeId, Relation, RelationTypeDeclaration, RelationTypeId, SystemId,
    Visibility, WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, EntityRegistry, EntityRegistrySnapshot, KernelError, RelationStore,
    RelationStoreSnapshot, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldView, owned_component,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// The first system: it owns how full a place is, answers for entering one, and records the edge.
// ---------------------------------------------------------------------------------------------

struct Places;

impl SystemIdentity for Places {
    const ID: SystemId = SystemId::from_static("places");
}

/// State `Places` owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Occupancy {
    present: u32,
}

owned_component! {
    component = Occupancy,
    owner = Places,
    component_type = "occupancy",
    schema_version = 1,
}

/// The request `Places` answers.
#[derive(Serialize, Deserialize)]
struct Enter;

impl Action for Enter {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("enter");
    const OWNER: SystemId = Places::ID;
}

/// The fact it records.
#[derive(Serialize, Deserialize)]
struct Entered {
    present: u32,
}

impl Event for Entered {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("entered");
    const OWNER: SystemId = Places::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The edge type `Places` declares: who is in which place.
fn located_in() -> RelationTypeId {
    RelationTypeId::new("located-in").expect("a legal declared name")
}

fn located_in_declaration() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        located_in(),
        Places::ID,
        EntityTypeSet::new([EntityType::Person]).expect("a non-empty set"),
        EntityTypeSet::new([EntityType::Place]).expect("a non-empty set"),
    )
}

impl System for Places {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Occupancy>()
            .providing::<Enter>()
            .emitting::<Entered>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Occupancy>()?;
        tables.relation(located_in_declaration())
    }

    /// Counts the arrival, records the edge, and states the fact. Everything it touches is its own.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let place = intent
            .target()
            .expect("this world only submits `enter` with a place");
        let present = world
            .read()
            .component::<Occupancy>(place)
            .map_or(1, |occupancy| occupancy.present + 1);
        world.insert(place, Occupancy { present })?;
        world.relate(&located_in(), intent.actor(), place)?;

        Ok(vec![
            Emission::new::<Entered>(payload(&Entered { present }), Visibility::Public)
                .about(vec![intent.actor()])
                .with_participants(vec![intent.actor(), place]),
        ])
    }
}

// ---------------------------------------------------------------------------------------------
// The second system: the one the configuration switches off. It owns a flag, answers for asking
// about it, and reacts to the first system's facts without the first system knowing it exists.
// ---------------------------------------------------------------------------------------------

struct Weather;

impl SystemIdentity for Weather {
    const ID: SystemId = SystemId::from_static("weather");
}

/// State `Weather` owns. `Places` can read it and can never write it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Overcast {
    reports: u32,
}

owned_component! {
    component = Overcast,
    owner = Weather,
    component_type = "overcast",
    schema_version = 1,
}

/// The request only a world with weather in it can answer.
#[derive(Serialize, Deserialize)]
struct Forecast;

impl Action for Forecast {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("forecast");
    const OWNER: SystemId = Weather::ID;
}

/// The fact `Weather` states.
#[derive(Serialize, Deserialize)]
struct Forecasted {
    reports: u32,
}

impl Event for Forecasted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("forecasted");
    const OWNER: SystemId = Weather::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Weather {
    const VERSION: SystemVersion = SystemVersion::new(2);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Overcast>()
            .providing::<Forecast>()
            .emitting::<Forecasted>()
            .subscribing_to::<Entered>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Overcast>()
    }

    /// Reacts to somebody entering a place by noting it in **its own** state. This is the
    /// cross-domain effect `ARCHITECTURE.md` §3 describes: `Places` emitted a fact and decided
    /// nothing about weather, and `Weather` decided what it means for the state it owns.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let Some(&place) = event.participants().last() else {
            return Ok(Vec::new());
        };
        let reports = world
            .read()
            .component::<Overcast>(place)
            .map_or(1, |overcast| overcast.reports + 1);
        world.insert(place, Overcast { reports })?;
        Ok(Vec::new())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let place = intent.target().expect("a forecast is asked about a place");
        let reports = world
            .read()
            .component::<Overcast>(place)
            .map_or(1, |overcast| overcast.reports + 1);
        world.insert(place, Overcast { reports })?;
        Ok(vec![Emission::new::<Forecasted>(
            payload(&Forecasted { reports }),
            Visibility::Public,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// The configuration. This — and only this — is what the AC-2 test changes.
// ---------------------------------------------------------------------------------------------

/// What a world pack chooses when it composes this world.
///
/// One field, because one field is the claim: `AC-2` is about a capability being added or removed by
/// **configuration**, not by editing anything. A real world pack would read this from a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Composition {
    /// Whether this world has weather in it.
    weather: bool,
}

/// Composes the world. Identical for every configuration: the same systems are installed in the same
/// order, and the only thing the configuration decides is whether one of them acts.
fn compose(configuration: Composition) -> World {
    let mut world = World::new();
    world.install(Places).expect("a system installs");
    world.install(Weather).expect("a second system installs");
    if !configuration.weather {
        world
            .disable(&Weather::ID)
            .expect("a system is disabled by configuration");
    }
    world
}

// ---------------------------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------------------------

fn payload<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring name")
}

/// The instant this whole checkpoint happens in. Supplied to dispatch, never read from a clock.
const NOW: WorldTime = WorldTime::from_seconds(3600);

fn request<A: Action>(id: u64, actor: EntityId, target: EntityId, payload_of: &A) -> ActionIntent {
    ActionIntent::new(
        ActionId::from_raw(id),
        actor,
        ActionRecord::new::<A>(payload(payload_of)),
        NOW,
    )
    .with_target(target)
}

/// Everything one world holds, written down: what persistence would carry, and what two worlds are
/// compared by.
///
/// The event envelopes are part of it, which is more than 03a's snapshot carried: history is state
/// too, and comparing two runs' facts compares their identities, their causation and the order they
/// were recorded in — which is where reduction order becomes observable (`AC-12`).
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct WorldSnapshot {
    entities: EntityRegistrySnapshot,
    occupancy: Vec<ComponentRecord<serde_json::Value>>,
    overcast: Vec<ComponentRecord<serde_json::Value>>,
    relations: RelationStoreSnapshot,
    history: Vec<EventEnvelope>,
}

impl WorldSnapshot {
    fn of(world: &World, history: &[EventEnvelope]) -> Self {
        Self {
            entities: EntityRegistrySnapshot::from(world.entities()),
            occupancy: records::<Occupancy>(world),
            overcast: records::<Overcast>(world),
            relations: RelationStoreSnapshot::from(world.relations()),
            history: history.to_vec(),
        }
    }
}

fn records<C: Component + 'static>(world: &World) -> Vec<ComponentRecord<serde_json::Value>> {
    world
        .components()
        .iter::<C>()
        .map(|(entity, component)| {
            ComponentRecord::new::<C>(
                entity,
                serde_json::to_value(component).expect("a component serializes"),
            )
        })
        .collect()
}

/// One scripted run of a world: the same entities, the same requests, in the same order, every time.
/// Everything the last two tests claim is a claim about running this function twice.
///
/// It takes the world rather than composing one, because the AC-2 test runs it against two
/// differently configured worlds and the script must be the part that does not change.
fn run(world: &mut World) -> Result<Vec<EventEnvelope>, KernelError> {
    let alice = world.create_entity(key("alice"), EntityType::Person)?;
    let cafe = world.create_entity(key("cafe"), EntityType::Place)?;
    let bob = world.create_entity(key("bob"), EntityType::Person)?;
    let leaving = world.create_entity(key("leaving"), EntityType::Person)?;

    let mut history = Vec::new();
    for (id, actor) in [(1, alice), (2, bob), (3, leaving)] {
        let dispatched = world.dispatch(&request(id, actor, cafe, &Enter), NOW)?;
        history.extend(dispatched.events().iter().cloned());
    }

    // An action the second system provides. In a world composed without weather this is answered
    // `Unavailable` and nothing else about the run changes.
    let dispatched = world.dispatch(&request(4, alice, cafe, &Forecast), NOW)?;
    history.extend(dispatched.events().iter().cloned());

    // And one entity leaves the world for good, which spends its identity and clears its edges.
    let cleared = world.destroy_entity(leaving)?;
    assert_eq!(cleared.len(), 1, "the edge it had is returned to its owner");

    Ok(history)
}

// ---------------------------------------------------------------------------------------------
// AC-2. The claim this whole PR exists to make.
// ---------------------------------------------------------------------------------------------

/// **`AC-2`.** Two systems, installed the same way in the same order. One configuration value
/// differs. In one world `forecast` is an action; in the other it does not exist, and asking for it
/// is answered [`ActionResult::Unavailable`] — the same answer any world gives for an action no
/// enabled system provides (`INV-10`).
///
/// Everything else is held fixed so that the difference is attributable: the same `compose`, the same
/// scripted `run`, the same requests in the same order, the same instant. No system's code differs
/// between the two worlds, because there is only one copy of it.
#[test]
fn disabling_a_system_removes_its_action_and_nothing_else_changes() {
    // ── The only difference between the two worlds. ──────────────────────────────────────────
    let with_weather = Composition { weather: true };
    let without_weather = Composition { weather: false };

    let mut rainy = compose(with_weather);
    let mut dry = compose(without_weather);

    let rainy_history = run(&mut rainy).expect("the scripted run succeeds");
    let dry_history = run(&mut dry).expect("the scripted run succeeds");

    let cafe = rainy
        .entities()
        .resolve(&key("cafe"))
        .expect("the cafe was created");

    // ── What the configuration changed. ─────────────────────────────────────────────────────
    // The action the disabled system provided does not exist in the world composed without it.
    assert_eq!(
        rainy.systems().provider(&Forecast::ACTION_TYPE),
        Some(&Weather::ID)
    );
    assert_eq!(dry.systems().provider(&Forecast::ACTION_TYPE), None);

    let asked_again = request(9, cafe, cafe, &Forecast);
    assert!(matches!(
        dry.dispatch(&asked_again, NOW)
            .expect("dispatch answers")
            .result(),
        ActionResult::Unavailable,
    ));
    assert!(matches!(
        rainy
            .dispatch(&asked_again, NOW)
            .expect("dispatch answers")
            .result(),
        ActionResult::Accepted { .. },
    ));

    // The disabled system's reaction to the other system's facts is gone too, so its state was
    // never written — and the facts it would have emitted were never recorded.
    assert_eq!(
        rainy.components().get::<Overcast>(cafe),
        Some(&Overcast { reports: 5 }),
        "three arrivals, one forecast, and the second forecast just asked for"
    );
    assert_eq!(dry.components().get::<Overcast>(cafe), None);
    assert_eq!(rainy_history.len(), 4, "three arrivals and one forecast");
    assert_eq!(
        dry_history.len(),
        3,
        "the forecast is not a fact in this world"
    );

    // ── What it did not change. ─────────────────────────────────────────────────────────────
    // The other system is untouched: its action still resolves, its state is identical, and the
    // facts it recorded are the same facts.
    assert_eq!(
        rainy.components().get::<Occupancy>(cafe),
        Some(&Occupancy { present: 3 })
    );
    assert_eq!(
        dry.components().get::<Occupancy>(cafe),
        Some(&Occupancy { present: 3 })
    );
    let entered = |history: &[EventEnvelope]| -> Vec<serde_json::Value> {
        history
            .iter()
            .filter(|event| *event.event_type() == Entered::EVENT_TYPE)
            .map(|event| {
                serde_json::from_slice(event.payload().payload()).expect("a payload decodes")
            })
            .collect()
    };
    assert_eq!(entered(&rainy_history), entered(&dry_history));
    assert_eq!(
        rainy.relations().len(),
        2,
        "alice and bob; leaving's edge went"
    );
    assert_eq!(dry.relations().len(), 2);

    // And the disabled system is still installed, still owns its table, and still holds the write
    // token this world granted it. Disabling removes a system's effect, not its state.
    assert!(dry.systems().is_installed(&Weather::ID));
    assert!(!dry.systems().is_enabled(&Weather::ID));
    assert!(dry.components().is_declared(&Overcast::COMPONENT_TYPE));
    assert_eq!(dry.writers().count(), 2);
}

// ---------------------------------------------------------------------------------------------
// The two properties PR 03a's checkpoint asserted, restored against the real world.
// ---------------------------------------------------------------------------------------------

/// The whole world, written down and read back, is the same world.
///
/// Restored from PR 03a's checkpoint, which C2 deleted along with the stand-in `World` it was written
/// against (§4.8.4). Stronger here than there: the snapshot carries the event envelopes as well, so
/// what round-trips is the world's state *and* its history.
#[test]
fn a_whole_world_round_trips_through_serde_unchanged() {
    let mut world = compose(Composition { weather: true });
    let history = run(&mut world).expect("the scripted run succeeds");

    let written =
        serde_json::to_string(&WorldSnapshot::of(&world, &history)).expect("a world serializes");
    let read: WorldSnapshot = serde_json::from_str(&written).expect("a world deserializes");

    assert_eq!(read, WorldSnapshot::of(&world, &history));

    // And the parts that carry their own restore path come back as usable stores, not merely as
    // matching bytes.
    let entities = EntityRegistry::try_from(read.entities).expect("a consistent registry restores");
    assert_eq!(&entities, world.entities());
    let relations = RelationStore::try_from(read.relations).expect("a consistent graph restores");
    assert_eq!(&relations, world.relations());
}

/// Replay: the same operations on a fresh world produce the same identities, the same state and the
/// same history, byte for byte. Nothing in the kernel depends on when anything happened, on the order
/// a map iterated, or on anything outside the world (`AC-12`).
///
/// This is the property that was lost when 03a's checkpoint was deleted, and it is the reason this
/// file exists rather than the AC-2 test living on its own.
#[test]
fn replaying_the_same_operations_reproduces_an_identical_world() {
    let snapshots: Vec<String> = (0..2)
        .map(|_| {
            let mut world = compose(Composition { weather: true });
            let history = run(&mut world).expect("the scripted run succeeds");
            serde_json::to_string(&WorldSnapshot::of(&world, &history)).expect("a world serializes")
        })
        .collect();

    assert_eq!(snapshots[0], snapshots[1]);

    // The comparison must be of a world with something in it, and of the parts a determinism claim
    // is really about: allocated identities, recorded facts, and their causes.
    let written: serde_json::Value =
        serde_json::from_str(&snapshots[0]).expect("the snapshot parses");
    assert_eq!(written["entities"]["next_id"], 5);
    assert_eq!(
        written["entities"]["entities"]["4"]["key"],
        serde_json::json!("leaving")
    );
    assert_eq!(written["history"][0]["id"], "1");
    assert_eq!(
        written["history"][0]["caused_by"],
        serde_json::json!({ "action": "1" })
    );
    assert_eq!(written["history"][3]["event_type"], "forecasted");
    assert_eq!(
        written["relations"]["edges"][0]["from"],
        serde_json::json!(EntityId::from_raw(1).to_string())
    );
}

/// The cross-system half of 03a's checkpoint, now through the pipeline rather than through the
/// stores: each system writes only its own state, reads the other's freely, and the write it must not
/// make cannot be written at all — see
/// `tests/compile_fail/a_system_cannot_write_another_systems_component.rs`, which is where a claim
/// about code that must not compile has to live.
#[test]
fn two_systems_write_their_own_state_and_read_each_others() {
    let mut world = compose(Composition { weather: true });
    run(&mut world).expect("the scripted run succeeds");

    let cafe = world
        .entities()
        .resolve(&key("cafe"))
        .expect("the cafe was created");

    // `Places` wrote the occupancy; `Weather` wrote its own tally by reacting to `Places`' facts and
    // by resolving its own action. Neither could have written the other's.
    assert_eq!(
        world.components().get::<Occupancy>(cafe),
        Some(&Occupancy { present: 3 })
    );
    assert_eq!(
        world.components().get::<Overcast>(cafe),
        Some(&Overcast { reports: 4 })
    );

    // The edge `Places` declared and wrote, and the destroyed entity's edge that went with it.
    let alice = world
        .entities()
        .resolve(&key("alice"))
        .expect("alice was created");
    let edge = Relation::between(
        &located_in_declaration(),
        world.entities().require(alice).expect("alice exists"),
        world.entities().require(cafe).expect("the cafe exists"),
    )
    .expect("a legal edge");
    assert!(world.read().is_related(&edge));
    assert_eq!(world.relations().len(), 2);
}
