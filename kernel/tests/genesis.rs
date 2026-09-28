//! World assembly's third step, from outside the kernel: what a world begins with, and what its own
//! log says about where that came from.
//!
//! The question this file answers is the one PR 05b could not: *where did Alice's initial position
//! come from?* A world's initial state is component state, component state is written only by its
//! owning system while it resolves or reacts, and there is no request at assembly time to resolve.
//! [`World::genesis`] is the answer, and these are the claims it has to make good:
//!
//! ```text
//! the facts are recorded, in the order they were stated
//! every one names Causation::WorldGenesis and NO controller decision — no ActionId is invented
//! identity comes from the world's own counter and continues into dispatch
//! the owning system reduces them into the state it owns, by the same path a request takes
//! a fact whose owner is not installed is refused by name
//! a fact its owner does not declare is refused by name
//! a fact stated after the world has run is refused by name
//! a reaction to a genesis fact is reduced in the same instant, and names its parent fact
//! two identically assembled worlds produce identical histories                    — AC-12
//! ```
//!
//! An external crate, so both systems here are written the way a System Pack writes one.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionTypeId, Causation, EntityId, EntityKey,
    EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId, SystemId, Visibility,
    WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    World, WorldView, owned_component,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// `weather`: owns the season, emits the fact that sets it, and reduces its own fact.
// ---------------------------------------------------------------------------------------------

struct Weather;

impl SystemIdentity for Weather {
    const ID: SystemId = SystemId::from_static("weather");
}

/// The state `weather` owns — the thing a World Pack wants to be true before anybody acts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Season {
    name: String,
}

owned_component! {
    component = Season,
    owner = Weather,
    component_type = "season",
    schema_version = 1,
}

/// The fact that changes it. The only way the component above is ever written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SeasonTurned {
    name: String,
}

impl Event for SeasonTurned {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("season-turned");
    const OWNER: SystemId = Weather::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// A fact `weather` declares but never emits at genesis, for the "not in the declaration" case to be
/// about a *declared* type rather than about an unknown one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Rained;

impl Event for Rained {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("rained");
    const OWNER: SystemId = Weather::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The request that changes the season while the world runs, so that a genesis fact and a dispatched
/// one can be compared in the same log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TurnSeason {
    name: String,
}

impl Action for TurnSeason {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("turn-season");
    const OWNER: SystemId = Weather::ID;
}

impl System for Weather {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Season>()
            .providing::<TurnSeason>()
            .emitting::<SeasonTurned>()
            .subscribing_to::<SeasonTurned>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Season>()
    }

    fn resolve(
        &self,
        _world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let request: TurnSeason = decode_action(intent);
        Ok(vec![turning(intent.actor(), &request.name)])
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != SeasonTurned::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let turned: SeasonTurned = decode_event(event);
        let subject = event.subjects()[0];
        world.insert(subject, Season { name: turned.name })?;
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------------------------
// `almanac`: writes nothing and only reacts, so that a genesis fact's *consequences* are visible.
// ---------------------------------------------------------------------------------------------

struct Almanac;

impl SystemIdentity for Almanac {
    const ID: SystemId = SystemId::from_static("almanac");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Noted;

impl Event for Noted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("noted");
    const OWNER: SystemId = Almanac::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Almanac {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .emitting::<Noted>()
            .subscribing_to::<SeasonTurned>()
    }

    fn react(
        &self,
        _world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != SeasonTurned::EVENT_TYPE {
            return Ok(Vec::new());
        }
        Ok(vec![Emission::new::<Noted>(
            encode(&Noted),
            Visibility::SystemInternal,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a test payload encodes")
}

fn decode_action(intent: &ActionIntent) -> TurnSeason {
    let bytes = intent
        .payload()
        .payload_for::<TurnSeason>()
        .expect("the test submits a well-formed request");
    serde_json::from_slice(bytes).expect("the test submits a well-formed payload")
}

fn decode_event(event: &EventEnvelope) -> SeasonTurned {
    serde_json::from_slice(event.payload().payload()).expect("this system's own payload")
}

/// The genesis fact a World Pack would state: `subject`'s season is `name`.
fn turning(subject: EntityId, name: &str) -> Emission {
    Emission::new::<SeasonTurned>(
        encode(&SeasonTurned {
            name: name.to_owned(),
        }),
        Visibility::Public,
    )
    .about(vec![subject])
}

/// A world with both systems and one place, ready to be told what is true of it.
fn assembled() -> (World, EntityId) {
    let mut world = World::new();
    world.install(Weather).expect("weather installs");
    world.install(Almanac).expect("almanac installs");
    let valley = world
        .create_entity(
            EntityKey::new("valley").expect("a legal key"),
            EntityType::Place,
        )
        .expect("the place is created");
    (world, valley)
}

// ---------------------------------------------------------------------------------------------
// The claims
// ---------------------------------------------------------------------------------------------

#[test]
fn a_genesis_fact_is_recorded_as_caused_by_the_world_coming_into_existence() {
    let (mut world, valley) = assembled();

    let recorded = world
        .genesis(WorldTime::EPOCH, vec![turning(valley, "spring")])
        .expect("a fact of an installed system's declared type is accepted");

    let genesis = &recorded[0];
    assert_eq!(
        *genesis.caused_by(),
        Causation::WorldGenesis,
        "initial state is explained by the world coming into existence and by nothing else",
    );
    assert_eq!(
        *genesis.event_type(),
        SeasonTurned::EVENT_TYPE,
        "the fact recorded is the fact stated",
    );
    assert_eq!(
        genesis.at(),
        WorldTime::EPOCH,
        "the instant is the one assembly was told to work in",
    );
}

#[test]
fn no_action_id_is_invented_for_a_fact_nobody_requested() {
    let (mut world, valley) = assembled();

    let recorded = world
        .genesis(WorldTime::EPOCH, vec![turning(valley, "spring")])
        .expect("genesis is accepted");

    // The whole of the decision this test exists for: `controller_decision` is `Option<ActionId>`
    // and a genesis fact leaves it empty, rather than carrying a number chosen to avoid colliding
    // with a request a client might later make.
    assert_eq!(
        recorded[0].provenance().controller_decision(),
        None,
        "a fact no controller asked for must not claim a controller decision",
    );
    assert_eq!(
        recorded[0].provenance().emitted_by(),
        &Weather::ID,
        "the emitter is the system that owns the kind of fact, read off the event type",
    );
    assert!(
        recorded
            .iter()
            .all(|event| *event.caused_by() == Causation::WorldGenesis
                || matches!(event.caused_by(), Causation::Event(_))),
        "nothing in a genesis chain is caused by an action",
    );
}

#[test]
fn the_owning_system_reduces_a_genesis_fact_into_the_state_it_owns() {
    let (mut world, valley) = assembled();

    world
        .genesis(WorldTime::EPOCH, vec![turning(valley, "spring")])
        .expect("genesis is accepted");

    assert_eq!(
        world.read().component::<Season>(valley),
        Some(&Season {
            name: "spring".to_owned()
        }),
        "the component is a projection of the genesis fact, written by its owner",
    );
}

#[test]
fn a_reaction_to_a_genesis_fact_is_reduced_in_the_same_instant_and_names_its_parent() {
    let (mut world, valley) = assembled();

    let recorded = world
        .genesis(WorldTime::EPOCH, vec![turning(valley, "spring")])
        .expect("genesis is accepted");

    assert_eq!(recorded.len(), 2, "the fact, and the consequence of it");
    assert_eq!(*recorded[1].event_type(), Noted::EVENT_TYPE);
    assert_eq!(
        *recorded[1].caused_by(),
        Causation::Event(recorded[0].id()),
        "a consequence of a genesis fact is caused by the fact, not by genesis again",
    );
    assert_eq!(
        recorded[1].provenance().controller_decision(),
        None,
        "and it, too, came from no request",
    );
}

#[test]
fn genesis_identity_comes_from_the_worlds_own_counter_and_continues_into_dispatch() {
    let (mut world, valley) = assembled();

    let recorded = world
        .genesis(WorldTime::EPOCH, vec![turning(valley, "spring")])
        .expect("genesis is accepted");
    assert_eq!(
        recorded
            .iter()
            .map(|event| event.id().raw())
            .collect::<Vec<_>>(),
        vec![1, 2],
        "a world's first facts are its first event identities",
    );

    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        valley,
        ActionRecord::new::<TurnSeason>(encode(&TurnSeason {
            name: "summer".to_owned(),
        })),
        WorldTime::EPOCH,
    );
    let dispatched = world
        .dispatch(&intent, WorldTime::EPOCH)
        .expect("the request resolves");

    assert_eq!(
        dispatched
            .events()
            .iter()
            .map(|event| event.id().raw())
            .collect::<Vec<_>>(),
        vec![3, 4],
        "one counter, so a genesis fact and a dispatched fact are never confusable",
    );
    assert_eq!(
        dispatched.events()[0].provenance().controller_decision(),
        Some(ActionId::from_raw(1)),
        "and a dispatched fact still names the request it came from",
    );
}

#[test]
fn a_genesis_fact_whose_owner_is_not_installed_is_refused_by_name() {
    let mut world = World::new();
    world.install(Weather).expect("weather installs");
    let valley = world
        .create_entity(
            EntityKey::new("valley").expect("a legal key"),
            EntityType::Place,
        )
        .expect("the place is created");

    // `almanac` is not in this world, so nothing would ever reduce its fact.
    let orphan =
        Emission::new::<Noted>(encode(&Noted), Visibility::SystemInternal).about(vec![valley]);
    let refusal = world
        .genesis(WorldTime::EPOCH, vec![orphan])
        .expect_err("a world cannot begin with a fact belonging to a system it lacks");

    assert!(
        matches!(
            refusal,
            KernelError::GenesisFactHasNoInstalledOwner { ref system, ref event_type }
                if *system == Almanac::ID && *event_type == Noted::EVENT_TYPE
        ),
        "the refusal names the system and the kind of fact: {refusal}",
    );
}

#[test]
fn a_genesis_fact_its_owner_does_not_declare_is_refused_by_name() {
    let (mut world, valley) = assembled();

    // `weather` owns `rained` — the event type names it — but its declaration does not list it, and
    // the declaration is what a reader of a world's composition goes by.
    let undeclared =
        Emission::new::<Rained>(encode(&Rained), Visibility::Public).about(vec![valley]);
    let refusal = world
        .genesis(WorldTime::EPOCH, vec![undeclared])
        .expect_err("an undeclared fact is refused at genesis as it is during dispatch");

    assert!(
        matches!(
            refusal,
            KernelError::EventTypeNotInSystemDeclaration { ref system, ref event_type }
                if *system == Weather::ID && *event_type == Rained::EVENT_TYPE
        ),
        "the same refusal dispatch gives, for the same reason: {refusal}",
    );
}

#[test]
fn genesis_after_the_world_has_run_is_refused_by_name() {
    let (mut world, valley) = assembled();
    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        valley,
        ActionRecord::new::<TurnSeason>(encode(&TurnSeason {
            name: "summer".to_owned(),
        })),
        WorldTime::EPOCH,
    );
    let _ = world
        .dispatch(&intent, WorldTime::EPOCH)
        .expect("the request resolves");

    let refusal = world
        .genesis(WorldTime::EPOCH, vec![turning(valley, "autumn")])
        .expect_err("assembly is over once a world has dispatched something");

    assert!(
        matches!(
            refusal,
            KernelError::GenesisAfterTheWorldHasRun { facts: 1 }
        ),
        "the refusal says how many facts arrived too late: {refusal}",
    );
    assert_eq!(
        world.read().component::<Season>(valley),
        Some(&Season {
            name: "summer".to_owned()
        }),
        "and the refusal changed nothing",
    );
}

#[test]
fn two_identically_assembled_worlds_produce_identical_histories() {
    let history = |seasons: [&str; 2]| -> Vec<(u64, EventTypeId, Causation, Option<ActionId>)> {
        let mut world = World::new();
        world.install(Weather).expect("weather installs");
        world.install(Almanac).expect("almanac installs");
        let valley = world
            .create_entity(
                EntityKey::new("valley").expect("a legal key"),
                EntityType::Place,
            )
            .expect("the place is created");
        let meadow = world
            .create_entity(
                EntityKey::new("meadow").expect("a legal key"),
                EntityType::Place,
            )
            .expect("the place is created");

        world
            .genesis(
                WorldTime::EPOCH,
                vec![turning(valley, seasons[0]), turning(meadow, seasons[1])],
            )
            .expect("genesis is accepted")
            .iter()
            .map(|event| {
                (
                    event.id().raw(),
                    event.event_type().clone(),
                    event.caused_by().clone(),
                    event.provenance().controller_decision(),
                )
            })
            .collect()
    };

    let first = history(["spring", "winter"]);
    let second = history(["spring", "winter"]);

    assert_eq!(
        first, second,
        "the same assembly stated twice records the same identities, causes and order (AC-12)",
    );
    assert_eq!(first.len(), 4, "two facts, each with its one consequence");
}
