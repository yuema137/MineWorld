//! The system-facing surface: what a declaration says, what the views allow, and that the
//! object-safe half really calls through to the typed system.
//!
//! These tests live inside the crate rather than in `kernel/tests/` because what they exercise is
//! deliberately not reachable from outside it. [`DynSystem`] and [`WorldParts`] are crate-private
//! (`BD-3`), and a [`WriteToken`] exists only inside a world — an external test can obtain neither,
//! which is the guarantee, not an inconvenience. The behaviour a *pack author* sees — routing,
//! reduction order, `Unavailable`, enable and disable — is tested from outside the crate, where a
//! pack lives.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionTypeId, Component, ComponentDeclaration,
    ComponentSchemaVersion, EntityId, EntityKey, EntityType, EntityTypeSet, Event,
    EventSchemaVersion, EventTypeId, Rejection, RelationTypeDeclaration, RelationTypeId, SystemId,
    Visibility, WorldTime,
};
use serde::{Deserialize, Serialize};

use super::{
    Cause, DynSystem, Emission, InstalledSystem, System, SystemDeclaration, SystemVersion,
};
use crate::access::{SystemIdentity, WriteAccess, WriteToken};
use crate::components::ComponentStore;
use crate::entities::EntityRegistry;
use crate::error::KernelError;
use crate::process::ProcessStore;
use crate::registry::SystemRegistry;
use crate::relations::RelationStore;
use crate::schedule::Scheduled;
use crate::view::WorldRead;
use crate::view::{Declarations, WorldParts, WorldView};
use mineworld_contracts::{Causation, EventId};

/// A system that owns state, provides an action and reacts to its own fact.
struct Alpha;

impl SystemIdentity for Alpha {
    const ID: SystemId = SystemId::from_static("alpha");
}

/// The state `Alpha` owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Counted {
    hits: u32,
}

crate::owned_component! {
    component = Counted,
    owner = Alpha,
    component_type = "counted",
    schema_version = 3,
}

/// The request `Alpha` answers.
#[derive(Serialize, Deserialize)]
struct Tick {
    step: u32,
}

impl Action for Tick {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("tick");
    const OWNER: SystemId = Alpha::ID;
}

/// The fact `Alpha` emits, and also subscribes to — which is how a system reduces its own facts.
#[derive(Serialize, Deserialize)]
struct Ticked {
    step: u32,
}

impl Event for Ticked {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("ticked");
    const OWNER: SystemId = Alpha::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Alpha {
    const VERSION: SystemVersion = SystemVersion::new(2);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Counted>()
            .providing::<Tick>()
            .emitting::<Ticked>()
            .subscribing_to::<Ticked>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Counted>()
    }

    /// Reads its own state and refuses a third request. A stub rule, but a real one: it is decided
    /// from authoritative state through the read-only view, which is what `validate` is for.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        if world
            .component::<Counted>(intent.actor())
            .is_some_and(|counted| counted.hits >= 2)
        {
            return Err(Rejection::PreconditionFailed);
        }
        Ok(())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let hits = world
            .read()
            .component::<Counted>(intent.actor())
            .map_or(1, |counted| counted.hits + 1);
        world.insert(intent.actor(), Counted { hits })?;
        Ok(vec![Emission::new::<Ticked>(
            serde_json::to_vec(&Ticked { step: hits }).expect("a payload encodes"),
            Visibility::SystemInternal,
        )])
    }
}

/// The smallest legal system: it owns nothing, provides nothing and emits nothing. It exists
/// because a world may legitimately install a system that only depends on another and reacts to
/// nothing yet, and because every default of the trait has to be exercised by something.
struct Beta;

impl SystemIdentity for Beta {
    const ID: SystemId = SystemId::from_static("beta");
}

impl System for Beta {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().depending_on([Alpha::ID])
    }
}

/// A system whose `declaration()` hands back somebody else's declaration. Nothing in the type
/// system stops it, which is exactly why installation compares the two.
struct Liar;

impl SystemIdentity for Liar {
    const ID: SystemId = SystemId::from_static("liar");
}

impl System for Liar {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Beta>()
    }
}

fn tick_intent(actor: EntityId) -> ActionIntent {
    ActionIntent::new(
        ActionId::from_raw(1),
        actor,
        ActionRecord::new::<Tick>(
            serde_json::to_vec(&Tick { step: 1 }).expect("a payload encodes"),
        ),
        WorldTime::EPOCH,
    )
}

/// The three stores a call needs, plus one entity to act on.
struct Bench {
    entities: EntityRegistry,
    components: ComponentStore,
    relations: RelationStore,
    /// The cause every call made through this bench is running under: reacting to fact 9, in a
    /// chain that began with request 3.
    cause: Cause,
    pending: Vec<(WorldTime, Scheduled)>,
    processes: ProcessStore,
    systems: SystemRegistry,
    foreign: Vec<(SystemId, Vec<Emission>)>,
}

impl Bench {
    fn new() -> (Self, WriteToken<Alpha>) {
        let mut access = WriteAccess::new();
        let token = access.grant(&Alpha).expect("a first grant succeeds");
        let mut entities = EntityRegistry::new();
        entities
            .create(
                EntityKey::new("subject").expect("a legal authoring name"),
                EntityType::Person,
            )
            .expect("the first entity is created");
        (
            Self {
                entities,
                components: ComponentStore::new(),
                relations: RelationStore::new(),
                cause: Cause {
                    caused_by: Causation::Event(EventId::from_raw(9)),
                    decision: Some(ActionId::from_raw(3)),
                },
                pending: Vec::new(),
                processes: ProcessStore::new(),
                systems: SystemRegistry::new(),
                foreign: Vec::new(),
            },
            token,
        )
    }

    fn subject(&self) -> EntityId {
        EntityId::from_raw(1)
    }

    fn parts(&mut self, at: WorldTime) -> WorldParts<'_> {
        WorldParts {
            entities: &self.entities,
            components: &mut self.components,
            relations: &mut self.relations,
            processes: &mut self.processes,
            systems: &self.systems,
            at,
            cause: &self.cause,
            pending: &mut self.pending,
            foreign: &mut self.foreign,
            depth: 0,
        }
    }

    fn read(&self) -> WorldRead<'_> {
        WorldRead::new(
            &self.entities,
            &self.components,
            &self.relations,
            &self.processes,
        )
    }
}

/// A declaration says exactly what its system's type declares, and nothing is restated by hand:
/// the name and the version come off the type, and every claim comes off a Rust type too.
#[test]
fn a_declaration_reports_what_its_type_declares() {
    let declaration = Alpha.declaration();

    assert_eq!(*declaration.system(), Alpha::ID);
    assert_eq!(declaration.version(), SystemVersion::new(2));
    assert!(declaration.depends_on().is_empty());

    assert_eq!(declaration.owns().len(), 1);
    let owned = &declaration.owns()[0];
    assert_eq!(*owned.component_type(), Counted::COMPONENT_TYPE);
    assert_eq!(*owned.owner(), Alpha::ID);
    assert_eq!(owned.schema_version(), ComponentSchemaVersion::new(3));

    assert_eq!(declaration.provides(), [Tick::ACTION_TYPE]);
    assert_eq!(declaration.emits(), [Ticked::EVENT_TYPE]);
    assert_eq!(declaration.subscribes(), [Ticked::EVENT_TYPE]);

    assert!(declaration.provides_action(&Tick::ACTION_TYPE));
    assert!(declaration.emits_event(&Ticked::EVENT_TYPE));
    assert!(declaration.subscribes_to(&Ticked::EVENT_TYPE));
    assert!(declaration.owns_component(&Counted::COMPONENT_TYPE));
}

/// A system that answers for nothing is a legal system. It has to be: a world composes systems
/// that own state, systems that provide actions and systems that only react, and the registry must
/// not require all three of everyone.
#[test]
fn a_system_that_provides_no_action_is_legal() {
    let declaration = Beta.declaration();

    assert!(declaration.provides().is_empty());
    assert!(declaration.owns().is_empty());
    assert!(declaration.emits().is_empty());
    assert!(declaration.subscribes().is_empty());
    assert_eq!(declaration.depends_on(), [Alpha::ID]);
    assert!(!declaration.provides_action(&Tick::ACTION_TYPE));

    // Two systems in one world disagree about everything they claim, which is what makes the
    // registry's conflict checks decidable from declarations alone.
    let other = Alpha.declaration();
    assert_ne!(declaration, other);
    assert_ne!(declaration.system(), other.system());
    assert!(!declaration.owns_component(&Counted::COMPONENT_TYPE));
}

/// `declaration()` is a method, so it can hand back a declaration built for another system. The
/// type system cannot catch that — `SystemDeclaration::of::<Beta>()` is a perfectly ordinary
/// sentence — so this pins the hole that installation closes by comparing the two names.
#[test]
fn a_declaration_can_name_another_system_which_is_why_installation_compares_them() {
    let declaration = Liar.declaration();

    assert_eq!(*declaration.system(), Beta::ID);
    assert_ne!(*declaration.system(), Liar::ID);
}

/// A system that provides an action and does not resolve it is refused by name. The alternative —
/// the default returning no events — would answer `Accepted` with an empty log and look like a
/// world that worked.
#[test]
fn the_default_resolution_refuses_and_names_the_action() {
    let (mut bench, _) = Bench::new();
    let mut access = WriteAccess::new();
    let token = access.grant(&Beta).expect("a first grant succeeds");
    let intent = tick_intent(bench.subject());
    let parts = bench.parts(WorldTime::EPOCH);
    let mut view = WorldView::new(parts, &token);

    let refusal = Beta
        .resolve(&mut view, &intent)
        .expect_err("the default resolution refuses");

    assert_eq!(
        refusal,
        KernelError::ActionNotResolvedBySystem {
            system: Beta::ID,
            action_type: Tick::ACTION_TYPE,
        }
    );
}

/// The object-safe half is a pass-through and nothing more: installing through it declares the
/// system's table, and resolving through it runs the system's own resolution, with the write
/// landing in the store through the token this wrapper holds.
#[test]
fn the_erased_half_calls_through_to_the_system() {
    let (mut bench, token) = Bench::new();
    let declaration = Alpha.declaration();
    let installed: Box<dyn DynSystem> = Box::new(InstalledSystem::new(Alpha, token));

    installed
        .install(
            &mut bench.components,
            &mut bench.relations,
            declaration.owns(),
        )
        .expect("the system declares its own table");
    assert!(bench.components.is_declared(&Counted::COMPONENT_TYPE));

    let subject = bench.subject();
    let intent = tick_intent(subject);
    let emissions = installed
        .resolve(bench.parts(WorldTime::from_seconds(10)), &intent)
        .expect("the system resolves its own action");

    assert_eq!(emissions.len(), 1);
    assert_eq!(*emissions[0].event_type(), Ticked::EVENT_TYPE);
    assert_eq!(*emissions[0].visibility(), Visibility::SystemInternal);
    assert_eq!(
        bench.components.get::<Counted>(subject),
        Some(&Counted { hits: 1 })
    );

    // Validation goes through the same wrapper, reads the state that write produced, and admits.
    let read = bench.read();
    assert_eq!(installed.validate(&read, &intent), Ok(()));

    // Run again: the system reads its own state through the view and writes the next value, which
    // is the ordinary shape of a resolution and the reason the view offers reads at all.
    installed
        .resolve(bench.parts(WorldTime::from_seconds(11)), &intent)
        .expect("the system resolves its own action again");
    assert_eq!(
        bench.components.get::<Counted>(subject),
        Some(&Counted { hits: 2 })
    );

    // And now the system's own rule refuses, through the erased half, from state alone.
    let read = bench.read();
    assert_eq!(
        installed.validate(&read, &intent),
        Err(Rejection::PreconditionFailed)
    );

    // Reaction reaches the system too. `Alpha` keeps the default, so the fact it subscribed to
    // produces nothing further — which is what the erased call must report rather than swallow.
    let envelope = envelope_of(Ticked::EVENT_TYPE);
    let further = installed
        .react(bench.parts(WorldTime::from_seconds(12)), &envelope)
        .expect("reaction reaches the system");
    assert!(further.is_empty());
}

/// One recorded fact, as the world would have built it, for the reaction seam above.
fn envelope_of(event_type: EventTypeId) -> mineworld_contracts::EventEnvelope {
    assert_eq!(event_type, Ticked::EVENT_TYPE);
    mineworld_contracts::EventEnvelope::new(
        mineworld_contracts::EventId::from_raw(1),
        WorldTime::from_seconds(12),
        mineworld_contracts::EventRecord::new::<Ticked>(
            serde_json::to_vec(&Ticked { step: 2 }).expect("a payload encodes"),
        ),
        mineworld_contracts::Causation::Action(ActionId::from_raw(1)),
        Visibility::SystemInternal,
        mineworld_contracts::Provenance::new(Alpha::ID),
    )
}

/// Relating two entities goes through this world's registry, so an identity the world never
/// allocated is refused by name and no edge is stored.
#[test]
fn relating_refuses_an_entity_this_world_never_allocated() {
    let (mut bench, token) = Bench::new();
    let declaration = attachment();
    bench
        .relations
        .declare(&token, declaration.clone())
        .expect("the owning system declares its edge type");
    let subject = bench.subject();
    let parts = bench.parts(WorldTime::EPOCH);
    let mut view = WorldView::new(parts, &token);

    let refusal = view
        .relate(declaration.relation_type(), subject, EntityId::from_raw(99))
        .expect_err("an unknown endpoint is refused");

    assert_eq!(
        refusal,
        KernelError::UnknownEntity {
            entity: EntityId::from_raw(99)
        }
    );
    assert!(bench.relations.is_empty());
}

/// Deferral means *later*. A system asking for the instant it is already in is refused, because
/// the queue `D-6` describes orders work by `(WorldTime, sequence)` and "later" that is not later
/// would lose the ordering a replay depends on.
#[test]
fn a_deferral_must_name_a_later_instant() {
    let (mut bench, token) = Bench::new();
    let now = WorldTime::from_seconds(40);

    {
        let parts = bench.parts(now);
        let mut view = WorldView::new(parts, &token);
        assert_eq!(view.at(), now);
        assert_eq!(view.writer(), Alpha::ID);

        let refusal = view
            .defer(now, emission())
            .expect_err("the current instant is not later than itself");
        assert_eq!(
            refusal,
            KernelError::DeferralNotInTheFuture { at: now, now }
        );

        view.defer(WorldTime::from_seconds(39), emission())
            .expect_err("an earlier instant is refused too");

        view.defer(WorldTime::from_seconds(41), emission())
            .expect("a later instant is queued");
    }

    assert_eq!(bench.pending.len(), 1);
    let (at, Scheduled::Fact(deferral)) = &bench.pending[0] else {
        panic!("a deferral is queued as a fact, not a wake");
    };
    assert_eq!(*at, WorldTime::from_seconds(41));
    assert_eq!(deferral.at(), WorldTime::from_seconds(41));
    assert_eq!(*deferral.emission().event_type(), Ticked::EVENT_TYPE);

    // The deferral carries who asked and why, taken from the kernel's side of the call rather than
    // from anything the system said: the writer's token and the cause the bench supplied.
    assert_eq!(*deferral.emitter(), Alpha::ID);
    assert_eq!(
        *deferral.caused_by(),
        Causation::Event(EventId::from_raw(9))
    );
    assert_eq!(deferral.controller_decision(), Some(ActionId::from_raw(3)));
}

/// A system may only declare tables its own declaration lists. Otherwise it could hold state that
/// no conflict check ever compared against another system's claims, which is the one thing the
/// registry's composition check relies on being complete.
#[test]
fn a_system_cannot_declare_a_table_its_declaration_does_not_list() {
    let (mut bench, token) = Bench::new();
    let nothing_owned: [ComponentDeclaration; 0] = [];
    let mut tables = Declarations::new(
        &mut bench.components,
        &mut bench.relations,
        &token,
        &nothing_owned,
    );

    let refusal = tables
        .component::<Counted>()
        .expect_err("an undeclared table is refused");

    assert_eq!(
        refusal,
        KernelError::ComponentTypeNotInSystemDeclaration {
            system: Alpha::ID,
            component_type: Counted::COMPONENT_TYPE,
        }
    );
    assert!(!bench.components.is_declared(&Counted::COMPONENT_TYPE));
}

fn emission() -> Emission {
    Emission::new::<Ticked>(
        serde_json::to_vec(&Ticked { step: 7 }).expect("a payload encodes"),
        Visibility::Public,
    )
}

/// An edge type `Alpha` owns, for the one test that needs a declared relation type.
fn attachment() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        RelationTypeId::new("attached-to").expect("a legal declared name"),
        Alpha::ID,
        EntityTypeSet::new([EntityType::Person]).expect("a non-empty set"),
        EntityTypeSet::new([EntityType::Person]).expect("a non-empty set"),
    )
}
